//! Transposition engine
//!
//! Supports both chromatic and diatonic transposition

use crate::error::{MozartError, Result};
use crate::note::Note;
use crate::pitch::{Pitch, PitchClass};
use crate::scale::Scale;
use serde::{Deserialize, Serialize};

/// Transposition mode
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TransposeMode {
    /// Chromatic transposition by semitones
    Chromatic {
        /// Number of semitones to transpose (-24 to +24)
        semitones: i8,
    },
    /// Diatonic transposition within a scale
    Diatonic {
        /// The source scale context
        source_scale: Scale,
        /// The target scale (can be same as source for pure diatonic shift)
        target_scale: Scale,
        /// Number of scale degrees to transpose (-7 to +7)
        degrees: i8,
    },
}

impl TransposeMode {
    /// Create a chromatic transposition
    pub fn chromatic(semitones: i8) -> Self {
        TransposeMode::Chromatic { semitones }
    }

    /// Create a diatonic transposition within the same scale
    pub fn diatonic(scale: Scale, degrees: i8) -> Self {
        TransposeMode::Diatonic {
            source_scale: scale,
            target_scale: scale,
            degrees,
        }
    }

    /// Create a diatonic transposition with key change
    pub fn diatonic_with_key_change(source: Scale, target: Scale, degrees: i8) -> Self {
        TransposeMode::Diatonic {
            source_scale: source,
            target_scale: target,
            degrees,
        }
    }

    /// Get human-readable description
    pub fn description(&self) -> String {
        match self {
            TransposeMode::Chromatic { semitones } => {
                let dir = if *semitones >= 0 { "up" } else { "down" };
                let interval = semitones.abs();
                let name = match interval {
                    0 => return "No transposition".to_string(),
                    1 => "minor 2nd",
                    2 => "major 2nd",
                    3 => "minor 3rd",
                    4 => "major 3rd",
                    5 => "perfect 4th",
                    6 => "tritone",
                    7 => "perfect 5th",
                    8 => "minor 6th",
                    9 => "major 6th",
                    10 => "minor 7th",
                    11 => "major 7th",
                    12 => "octave",
                    _ => return format!("{} {} semitones", dir, interval),
                };
                format!("{} a {}", dir, name)
            }
            TransposeMode::Diatonic {
                source_scale,
                target_scale,
                degrees,
            } => {
                let dir = if *degrees >= 0 { "up" } else { "down" };
                let degree = degrees.abs();
                let name = match degree {
                    0 => "unison",
                    1 => "2nd",
                    2 => "3rd",
                    3 => "4th",
                    4 => "5th",
                    5 => "6th",
                    6 => "7th",
                    7 => "octave",
                    _ => return format!("{} {} degrees", dir, degree),
                };

                if source_scale == target_scale {
                    format!("Diatonic {} a {} in {}", dir, name, source_scale)
                } else {
                    format!(
                        "Diatonic {} a {} from {} to {}",
                        dir, name, source_scale, target_scale
                    )
                }
            }
        }
    }
}

/// Transpose a single pitch chromatically
pub fn transpose_pitch_chromatic(pitch: Pitch, semitones: i8) -> Result<Pitch> {
    tracing::trace!(
        "Chromatic transpose: {} by {} semitones",
        pitch,
        semitones
    );
    pitch.transpose(semitones)
}

/// Transpose a single pitch diatonically within a scale
///
/// This finds the note's position in the source scale, moves by the given
/// number of degrees, and returns the corresponding note in the target scale.
/// Non-scale tones are moved to the nearest scale tone first.
pub fn transpose_pitch_diatonic(
    pitch: Pitch,
    source_scale: &Scale,
    target_scale: &Scale,
    degrees: i8,
) -> Result<Pitch> {
    let pc = pitch.pitch_class();
    let octave = pitch.octave();

    tracing::debug!(
        "Diatonic transpose: {} ({}) by {} degrees from {} to {}",
        pitch,
        pc,
        degrees,
        source_scale,
        target_scale
    );

    // Find the degree in the source scale (or nearest)
    let (source_degree, octave_adjustment) = if let Some(deg) = source_scale.degree_of(pc) {
        tracing::trace!("{} is degree {} in {}", pc, deg, source_scale);
        (deg as i8, 0i8)
    } else {
        // Note is not in the scale - find nearest and track the chromatic offset
        let (nearest_pc, _adjustment) = source_scale.nearest_scale_tone(pc);
        let deg = source_scale
            .degree_of(nearest_pc)
            .ok_or_else(|| MozartError::TranspositionError("Could not find scale degree".into()))?;

        tracing::trace!(
            "{} not in scale, using nearest {} (degree {})",
            pc,
            nearest_pc,
            deg
        );
        (deg as i8, 0)
    };

    // Calculate the new degree in the target scale
    // Degrees are 1-7, so we need to handle octave wrapping
    let new_degree_raw = source_degree + degrees;

    // Handle octave changes from degree wrapping
    let octave_change = if new_degree_raw > 7 {
        (new_degree_raw - 1) / 7
    } else if new_degree_raw < 1 {
        (new_degree_raw - 7) / 7
    } else {
        0
    };

    // Normalize degree to 1-7
    let new_degree = ((new_degree_raw - 1).rem_euclid(7) + 1) as u8;

    tracing::trace!(
        "New degree: {} (raw: {}, octave_change: {})",
        new_degree,
        new_degree_raw,
        octave_change
    );

    // Get the pitch class from the target scale
    let new_pc = target_scale.degree(new_degree).ok_or_else(|| {
        MozartError::TranspositionError(format!("Invalid degree {} in target scale", new_degree))
    })?;

    // Calculate octave change based on actual pitch movement
    // We need to account for:
    // 1. Complete scale cycles (every 7 degrees = 1 octave)
    // 2. Crossing the octave boundary (C) within a scale cycle
    let old_semitones = pc.semitones() as i16;
    let new_semitones = new_pc.semitones() as i16;

    // Count complete octaves from degree movement
    let full_octaves = degrees / 7;

    // Check if remaining movement crosses the C boundary (semitone 0)
    let remaining_degrees = degrees % 7;
    let boundary_cross = if remaining_degrees > 0 {
        // Moving up: crossed if new semitone is less than old (wrapped past C)
        if new_semitones < old_semitones { 1 } else { 0 }
    } else if remaining_degrees < 0 {
        // Moving down: crossed if new semitone is greater than old (wrapped past C going down)
        if new_semitones > old_semitones { -1 } else { 0 }
    } else {
        0
    };

    let new_octave = octave + full_octaves as i8 + boundary_cross + octave_adjustment;

    tracing::debug!(
        "Result: degree {} in {} = {}{} (octave {})",
        new_degree,
        target_scale,
        new_pc,
        new_octave,
        new_octave
    );

    Pitch::new(new_pc, new_octave)
}

/// Transpose a note (updates pitch, preserves timing/duration/velocity)
pub fn transpose_note(note: &Note, mode: &TransposeMode) -> Result<Note> {
    let pitch = Pitch::from_midi(note.pitch)?;

    let new_pitch = match mode {
        TransposeMode::Chromatic { semitones } => {
            transpose_pitch_chromatic(pitch, *semitones)?
        }
        TransposeMode::Diatonic {
            source_scale,
            target_scale,
            degrees,
        } => transpose_pitch_diatonic(pitch, source_scale, target_scale, *degrees)?,
    };

    Ok(Note {
        pitch: new_pitch.midi(),
        start_tick: note.start_tick,
        duration_ticks: note.duration_ticks,
        velocity: note.velocity,
    })
}

/// Transpose a collection of notes
pub fn transpose_notes(notes: &[Note], mode: &TransposeMode) -> Result<Vec<Note>> {
    tracing::info!(
        "Transposing {} notes: {}",
        notes.len(),
        mode.description()
    );

    notes.iter().map(|note| transpose_note(note, mode)).collect()
}

/// Analyze a melody to suggest likely scale
/// Only considers Major and Natural Minor scales.
/// Prefers minor over major when ambiguous.
/// Prefers scales where the root matches the last note.
pub fn detect_scale(notes: &[Note]) -> Option<Scale> {
    if notes.is_empty() {
        return None;
    }

    // Collect all pitch classes used
    let mut pitch_classes = std::collections::HashSet::new();
    for note in notes {
        if let Ok(pitch) = Pitch::from_midi(note.pitch) {
            pitch_classes.insert(pitch.pitch_class().semitones());
        }
    }

    // Get the last note's pitch class for tiebreaking
    let last_note_pc = notes.last()
        .and_then(|n| Pitch::from_midi(n.pitch).ok())
        .map(|p| p.pitch_class().semitones());

    tracing::debug!(
        "Detecting scale from pitch classes: {:?}, last note: {:?}",
        pitch_classes,
        last_note_pc
    );

    // Only consider Major and Natural Minor
    let scale_types = [
        crate::scale::ScaleType::NaturalMinor, // Minor first (preferred)
        crate::scale::ScaleType::Major,
    ];

    // Score: (matches_all, last_note_is_root, is_minor, match_count)
    // Higher is better, compared lexicographically
    let mut best_match: Option<(Scale, (bool, bool, bool, usize))> = None;

    for root in PitchClass::all() {
        for scale_type in &scale_types {
            let scale = Scale::new(root, *scale_type);
            let scale_pcs: std::collections::HashSet<u8> = scale
                .pitch_classes()
                .iter()
                .map(|pc| pc.semitones())
                .collect();

            // Count how many of the melody's notes are in this scale
            let matches = pitch_classes.intersection(&scale_pcs).count();
            let total = pitch_classes.len();
            let matches_all = matches == total;
            let last_note_is_root = last_note_pc == Some(root.semitones());
            let is_minor = *scale_type == crate::scale::ScaleType::NaturalMinor;

            let score = (matches_all, last_note_is_root, is_minor, matches);

            if best_match.is_none() || score > best_match.as_ref().unwrap().1 {
                best_match = Some((scale, score));
            }
        }
    }

    best_match.map(|(scale, score)| {
        tracing::info!("Detected scale: {} (score: {:?})", scale, score);
        scale
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::note::parse_melody;
    use crate::scale::ScaleType;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_chromatic_transpose() {
        let c4 = Pitch::MIDDLE_C;

        // Up a major 3rd
        let e4 = transpose_pitch_chromatic(c4, 4).unwrap();
        assert_eq!(e4.pitch_class(), PitchClass::E);
        assert_eq!(e4.octave(), 4);

        // Down a perfect 5th
        let f3 = transpose_pitch_chromatic(c4, -7).unwrap();
        assert_eq!(f3.pitch_class(), PitchClass::F);
        assert_eq!(f3.octave(), 3);
    }

    #[test]
    fn test_diatonic_transpose_within_scale() {
        let c_major = Scale::c_major();

        // C4 up a third in C major = E4
        let c4 = Pitch::MIDDLE_C;
        let result = transpose_pitch_diatonic(c4, &c_major, &c_major, 2).unwrap();
        assert_eq!(result.pitch_class(), PitchClass::E);
        assert_eq!(result.octave(), 4);

        // E4 up a third in C major = G4
        let e4 = Pitch::new(PitchClass::E, 4).unwrap();
        let result = transpose_pitch_diatonic(e4, &c_major, &c_major, 2).unwrap();
        assert_eq!(result.pitch_class(), PitchClass::G);
        assert_eq!(result.octave(), 4);

        // G4 up a third in C major = B4
        let g4 = Pitch::new(PitchClass::G, 4).unwrap();
        let result = transpose_pitch_diatonic(g4, &c_major, &c_major, 2).unwrap();
        assert_eq!(result.pitch_class(), PitchClass::B);
        assert_eq!(result.octave(), 4);

        // B4 up a third in C major = D5 (crosses octave)
        let b4 = Pitch::new(PitchClass::B, 4).unwrap();
        let result = transpose_pitch_diatonic(b4, &c_major, &c_major, 2).unwrap();
        assert_eq!(result.pitch_class(), PitchClass::D);
        assert_eq!(result.octave(), 5);
    }

    #[test]
    fn test_diatonic_transpose_down() {
        let c_major = Scale::c_major();

        // E4 down a third in C major = C4
        let e4 = Pitch::new(PitchClass::E, 4).unwrap();
        let result = transpose_pitch_diatonic(e4, &c_major, &c_major, -2).unwrap();
        assert_eq!(result.pitch_class(), PitchClass::C);
        assert_eq!(result.octave(), 4);

        // C4 down a third in C major = A3
        let c4 = Pitch::MIDDLE_C;
        let result = transpose_pitch_diatonic(c4, &c_major, &c_major, -2).unwrap();
        assert_eq!(result.pitch_class(), PitchClass::A);
        assert_eq!(result.octave(), 3);
    }

    #[test]
    fn test_diatonic_transpose_a_minor_octave_crossing() {
        // Test the bug fix: A4 up a third in A minor should be C5, not C4
        let a_minor = Scale::a_minor();

        // A4 up a third in A minor = C5 (crosses octave within scale)
        let a4 = Pitch::new(PitchClass::A, 4).unwrap();
        let result = transpose_pitch_diatonic(a4, &a_minor, &a_minor, 2).unwrap();
        assert_eq!(result.pitch_class(), PitchClass::C);
        assert_eq!(result.octave(), 5); // Must be 5, not 4!

        // G4 up a third in A minor = B4 (no octave crossing)
        let g4 = Pitch::new(PitchClass::G, 4).unwrap();
        let result = transpose_pitch_diatonic(g4, &a_minor, &a_minor, 2).unwrap();
        assert_eq!(result.pitch_class(), PitchClass::B);
        assert_eq!(result.octave(), 4);

        // C5 down a third in A minor = A4 (crosses octave going down)
        let c5 = Pitch::new(PitchClass::C, 5).unwrap();
        let result = transpose_pitch_diatonic(c5, &a_minor, &a_minor, -2).unwrap();
        assert_eq!(result.pitch_class(), PitchClass::A);
        assert_eq!(result.octave(), 4);
    }

    #[test]
    fn test_diatonic_transpose_key_change() {
        let c_major = Scale::c_major();
        let g_major = Scale::new(PitchClass::G, ScaleType::Major);

        // C in C major (degree 1) -> G in G major (degree 1)
        let c4 = Pitch::MIDDLE_C;
        let result = transpose_pitch_diatonic(c4, &c_major, &g_major, 0).unwrap();
        assert_eq!(result.pitch_class(), PitchClass::G);

        // E in C major (degree 3) up a third -> D in G major (degree 5)
        let e4 = Pitch::new(PitchClass::E, 4).unwrap();
        let result = transpose_pitch_diatonic(e4, &c_major, &g_major, 2).unwrap();
        assert_eq!(result.pitch_class(), PitchClass::D);
    }

    #[test]
    fn test_transpose_melody() {
        let melody = parse_melody("C4q D4q E4q").unwrap();
        let mode = TransposeMode::chromatic(2); // Up a whole step

        let transposed = transpose_notes(&melody, &mode).unwrap();

        assert_eq!(transposed.len(), 3);
        assert_eq!(transposed[0].pitch, 62); // D4
        assert_eq!(transposed[1].pitch, 64); // E4
        assert_eq!(transposed[2].pitch, 66); // F#4
    }

    #[test]
    fn test_transpose_melody_diatonic() {
        let melody = parse_melody("C4q E4q G4q").unwrap(); // C major triad
        let c_major = Scale::c_major();
        let mode = TransposeMode::diatonic(c_major, 2); // Up a third

        let transposed = transpose_notes(&melody, &mode).unwrap();

        assert_eq!(transposed.len(), 3);
        // C -> E, E -> G, G -> B (all diatonic thirds in C major)
        assert_eq!(Pitch::from_midi(transposed[0].pitch).unwrap().pitch_class(), PitchClass::E);
        assert_eq!(Pitch::from_midi(transposed[1].pitch).unwrap().pitch_class(), PitchClass::G);
        assert_eq!(Pitch::from_midi(transposed[2].pitch).unwrap().pitch_class(), PitchClass::B);
    }

    #[test]
    fn test_detect_scale_c_major() {
        let melody = parse_melody("C4q D4q E4q F4q G4q A4q B4q").unwrap();
        let detected = detect_scale(&melody);

        assert!(detected.is_some());
        let scale = detected.unwrap();
        // Should detect C Major (or possibly A minor, which shares the same notes)
        assert!(
            scale.root == PitchClass::C || scale.root == PitchClass::A,
            "Expected C or A root, got {}",
            scale.root
        );
    }

    #[test]
    fn test_transpose_mode_description() {
        let chromatic = TransposeMode::chromatic(4);
        assert!(chromatic.description().contains("major 3rd"));

        let diatonic = TransposeMode::diatonic(Scale::c_major(), 2);
        assert!(diatonic.description().contains("3rd"));
        assert!(diatonic.description().contains("C Major"));
    }
}
