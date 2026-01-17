//! Scale definitions
//!
//! Supports major, minor (natural/harmonic/melodic), and all church modes

use serde::{Deserialize, Serialize};
use std::fmt;
use crate::pitch::PitchClass;
use crate::error::{MozartError, Result};

/// Scale types supported by the transposition engine
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ScaleType {
    // Major and minor
    Major,          // Ionian mode
    NaturalMinor,   // Aeolian mode
    HarmonicMinor,
    MelodicMinor,   // Ascending form

    // Church modes
    Dorian,
    Phrygian,
    Lydian,
    Mixolydian,
    Locrian,
}

impl ScaleType {
    /// Get the intervals (in semitones from root) for this scale
    pub fn intervals(&self) -> &'static [u8] {
        match self {
            // W W H W W W H
            ScaleType::Major => &[0, 2, 4, 5, 7, 9, 11],
            // W H W W H W W
            ScaleType::NaturalMinor => &[0, 2, 3, 5, 7, 8, 10],
            // W H W W H W+H H
            ScaleType::HarmonicMinor => &[0, 2, 3, 5, 7, 8, 11],
            // W H W W W W H (ascending)
            ScaleType::MelodicMinor => &[0, 2, 3, 5, 7, 9, 11],
            // W H W W W H W
            ScaleType::Dorian => &[0, 2, 3, 5, 7, 9, 10],
            // H W W W H W W
            ScaleType::Phrygian => &[0, 1, 3, 5, 7, 8, 10],
            // W W W H W W H
            ScaleType::Lydian => &[0, 2, 4, 6, 7, 9, 11],
            // W W H W W H W
            ScaleType::Mixolydian => &[0, 2, 4, 5, 7, 9, 10],
            // H W W H W W W
            ScaleType::Locrian => &[0, 1, 3, 5, 6, 8, 10],
        }
    }

    /// Get the scale degree names
    pub fn degree_names(&self) -> &'static [&'static str] {
        &["1", "2", "3", "4", "5", "6", "7"]
    }

    /// Parse from string
    pub fn parse(s: &str) -> Result<Self> {
        let s = s.to_lowercase();
        let s = s.trim();

        match s {
            "major" | "maj" | "ionian" => Ok(ScaleType::Major),
            "minor" | "min" | "natural minor" | "natural_minor" | "aeolian" => {
                Ok(ScaleType::NaturalMinor)
            }
            "harmonic minor" | "harmonic_minor" | "harm" => Ok(ScaleType::HarmonicMinor),
            "melodic minor" | "melodic_minor" | "mel" => Ok(ScaleType::MelodicMinor),
            "dorian" | "dor" => Ok(ScaleType::Dorian),
            "phrygian" | "phryg" => Ok(ScaleType::Phrygian),
            "lydian" | "lyd" => Ok(ScaleType::Lydian),
            "mixolydian" | "mixo" => Ok(ScaleType::Mixolydian),
            "locrian" | "loc" => Ok(ScaleType::Locrian),
            _ => Err(MozartError::InvalidScale(format!("Unknown scale: {}", s))),
        }
    }

    /// Get display name
    pub fn name(&self) -> &'static str {
        match self {
            ScaleType::Major => "Major",
            ScaleType::NaturalMinor => "Natural Minor",
            ScaleType::HarmonicMinor => "Harmonic Minor",
            ScaleType::MelodicMinor => "Melodic Minor",
            ScaleType::Dorian => "Dorian",
            ScaleType::Phrygian => "Phrygian",
            ScaleType::Lydian => "Lydian",
            ScaleType::Mixolydian => "Mixolydian",
            ScaleType::Locrian => "Locrian",
        }
    }

    /// All scale types
    pub fn all() -> &'static [ScaleType] {
        &[
            ScaleType::Major,
            ScaleType::NaturalMinor,
            ScaleType::HarmonicMinor,
            ScaleType::MelodicMinor,
            ScaleType::Dorian,
            ScaleType::Phrygian,
            ScaleType::Lydian,
            ScaleType::Mixolydian,
            ScaleType::Locrian,
        ]
    }
}

impl fmt::Display for ScaleType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// A complete scale definition with root and type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Scale {
    pub root: PitchClass,
    pub scale_type: ScaleType,
}

impl Scale {
    /// Create a new scale
    pub fn new(root: PitchClass, scale_type: ScaleType) -> Self {
        Scale { root, scale_type }
    }

    /// C Major scale
    pub fn c_major() -> Self {
        Scale::new(PitchClass::C, ScaleType::Major)
    }

    /// A Minor scale (natural)
    pub fn a_minor() -> Self {
        Scale::new(PitchClass::A, ScaleType::NaturalMinor)
    }

    /// Get the pitch classes in this scale
    pub fn pitch_classes(&self) -> Vec<PitchClass> {
        self.scale_type
            .intervals()
            .iter()
            .map(|&interval| self.root.transpose(interval as i8))
            .collect()
    }

    /// Check if a pitch class is in this scale
    pub fn contains(&self, pitch_class: PitchClass) -> bool {
        let interval = self.root.interval_to(pitch_class);
        self.scale_type.intervals().contains(&interval)
    }

    /// Get the scale degree (1-7) for a pitch class, if it's in the scale
    pub fn degree_of(&self, pitch_class: PitchClass) -> Option<u8> {
        let interval = self.root.interval_to(pitch_class);
        self.scale_type
            .intervals()
            .iter()
            .position(|&i| i == interval)
            .map(|pos| (pos + 1) as u8)
    }

    /// Get the pitch class at a given scale degree (1-7)
    pub fn degree(&self, degree: u8) -> Option<PitchClass> {
        if degree < 1 || degree > 7 {
            return None;
        }
        let interval = self.scale_type.intervals()[(degree - 1) as usize];
        Some(self.root.transpose(interval as i8))
    }

    /// Find the nearest scale tone for a given pitch class
    /// Returns the pitch class and the adjustment in semitones
    pub fn nearest_scale_tone(&self, pitch_class: PitchClass) -> (PitchClass, i8) {
        let interval = self.root.interval_to(pitch_class);

        // Check if it's already in the scale
        if self.scale_type.intervals().contains(&interval) {
            return (pitch_class, 0);
        }

        // Find the nearest scale tone
        let intervals = self.scale_type.intervals();
        let mut best_adjustment = i8::MAX;
        let mut best_pc = pitch_class;

        for &scale_interval in intervals {
            let diff = scale_interval as i8 - interval as i8;
            // Consider both up and down, wrapping around
            let adjustments = [diff, diff - 12, diff + 12];
            for adj in adjustments {
                if adj.abs() < best_adjustment.abs() {
                    best_adjustment = adj;
                    best_pc = pitch_class.transpose(adj);
                }
            }
        }

        tracing::trace!(
            "Nearest scale tone for {} in {}: {} (adjustment: {})",
            pitch_class,
            self,
            best_pc,
            best_adjustment
        );

        (best_pc, best_adjustment)
    }

    /// Parse from string (e.g., "C major", "F# minor", "Bb dorian")
    pub fn parse(s: &str) -> Result<Self> {
        let s = s.trim();
        let parts: Vec<&str> = s.splitn(2, ' ').collect();

        if parts.is_empty() {
            return Err(MozartError::InvalidScale("Empty scale string".to_string()));
        }

        let root = PitchClass::parse(parts[0])?;
        let scale_type = if parts.len() > 1 {
            ScaleType::parse(parts[1])?
        } else {
            ScaleType::Major // Default to major
        };

        Ok(Scale::new(root, scale_type))
    }
}

impl fmt::Display for Scale {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.root, self.scale_type)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_major_scale_intervals() {
        assert_eq!(ScaleType::Major.intervals(), &[0, 2, 4, 5, 7, 9, 11]);
    }

    #[test]
    fn test_c_major_pitch_classes() {
        let scale = Scale::c_major();
        let pcs = scale.pitch_classes();
        assert_eq!(pcs.len(), 7);
        assert_eq!(pcs[0], PitchClass::C);
        assert_eq!(pcs[1], PitchClass::D);
        assert_eq!(pcs[2], PitchClass::E);
        assert_eq!(pcs[3], PitchClass::F);
        assert_eq!(pcs[4], PitchClass::G);
        assert_eq!(pcs[5], PitchClass::A);
        assert_eq!(pcs[6], PitchClass::B);
    }

    #[test]
    fn test_scale_contains() {
        let c_major = Scale::c_major();
        assert!(c_major.contains(PitchClass::C));
        assert!(c_major.contains(PitchClass::G));
        assert!(!c_major.contains(PitchClass::C_SHARP));
        assert!(!c_major.contains(PitchClass::F_SHARP));
    }

    #[test]
    fn test_scale_degree() {
        let c_major = Scale::c_major();
        assert_eq!(c_major.degree_of(PitchClass::C), Some(1));
        assert_eq!(c_major.degree_of(PitchClass::E), Some(3));
        assert_eq!(c_major.degree_of(PitchClass::G), Some(5));
        assert_eq!(c_major.degree_of(PitchClass::C_SHARP), None);
    }

    #[test]
    fn test_nearest_scale_tone() {
        let c_major = Scale::c_major();

        // F# should go to F or G
        let (nearest, adj) = c_major.nearest_scale_tone(PitchClass::F_SHARP);
        assert!(nearest == PitchClass::F || nearest == PitchClass::G);
        assert!(adj.abs() == 1);

        // C# should go to C or D
        let (nearest, adj) = c_major.nearest_scale_tone(PitchClass::C_SHARP);
        assert!(nearest == PitchClass::C || nearest == PitchClass::D);
        assert!(adj.abs() == 1);
    }

    #[test]
    fn test_scale_parse() {
        let scale = Scale::parse("C major").unwrap();
        assert_eq!(scale.root, PitchClass::C);
        assert_eq!(scale.scale_type, ScaleType::Major);

        let scale = Scale::parse("F# minor").unwrap();
        assert_eq!(scale.root, PitchClass::F_SHARP);
        assert_eq!(scale.scale_type, ScaleType::NaturalMinor);

        let scale = Scale::parse("Bb dorian").unwrap();
        assert_eq!(scale.root, PitchClass::B_FLAT);
        assert_eq!(scale.scale_type, ScaleType::Dorian);
    }
}
