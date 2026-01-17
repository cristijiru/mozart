//! Pitch representation
//!
//! Handles pitch classes (C, C#, D, etc.) and absolute pitches (MIDI note numbers)

use serde::{Deserialize, Serialize};
use std::fmt;
use crate::error::{MozartError, Result};

/// Pitch class (note name without octave)
/// Uses semitones from C (0 = C, 1 = C#/Db, ..., 11 = B)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PitchClass(u8);

impl PitchClass {
    pub const C: PitchClass = PitchClass(0);
    pub const C_SHARP: PitchClass = PitchClass(1);
    pub const D_FLAT: PitchClass = PitchClass(1);
    pub const D: PitchClass = PitchClass(2);
    pub const D_SHARP: PitchClass = PitchClass(3);
    pub const E_FLAT: PitchClass = PitchClass(3);
    pub const E: PitchClass = PitchClass(4);
    pub const F: PitchClass = PitchClass(5);
    pub const F_SHARP: PitchClass = PitchClass(6);
    pub const G_FLAT: PitchClass = PitchClass(6);
    pub const G: PitchClass = PitchClass(7);
    pub const G_SHARP: PitchClass = PitchClass(8);
    pub const A_FLAT: PitchClass = PitchClass(8);
    pub const A: PitchClass = PitchClass(9);
    pub const A_SHARP: PitchClass = PitchClass(10);
    pub const B_FLAT: PitchClass = PitchClass(10);
    pub const B: PitchClass = PitchClass(11);

    /// Create a new pitch class from semitones (0-11)
    pub fn new(semitones: u8) -> Self {
        PitchClass(semitones % 12)
    }

    /// Get the semitone value (0-11)
    pub fn semitones(&self) -> u8 {
        self.0
    }

    /// Parse a pitch class from string (e.g., "C", "C#", "Db")
    pub fn parse(s: &str) -> Result<Self> {
        let s = s.trim();
        tracing::trace!("Parsing pitch class: {}", s);

        let result = match s.to_uppercase().as_str() {
            "C" => PitchClass::C,
            "C#" | "C♯" | "CS" => PitchClass::C_SHARP,
            "DB" | "D♭" => PitchClass::D_FLAT,
            "D" => PitchClass::D,
            "D#" | "D♯" | "DS" => PitchClass::D_SHARP,
            "EB" | "E♭" => PitchClass::E_FLAT,
            "E" => PitchClass::E,
            "F" => PitchClass::F,
            "F#" | "F♯" | "FS" => PitchClass::F_SHARP,
            "GB" | "G♭" => PitchClass::G_FLAT,
            "G" => PitchClass::G,
            "G#" | "G♯" | "GS" => PitchClass::G_SHARP,
            "AB" | "A♭" => PitchClass::A_FLAT,
            "A" => PitchClass::A,
            "A#" | "A♯" | "AS" => PitchClass::A_SHARP,
            "BB" | "B♭" => PitchClass::B_FLAT,
            "B" => PitchClass::B,
            _ => return Err(MozartError::InvalidPitch(format!("Unknown pitch class: {}", s))),
        };

        tracing::trace!("Parsed {} -> {:?}", s, result);
        Ok(result)
    }

    /// Transpose by semitones (positive = up, negative = down)
    pub fn transpose(&self, semitones: i8) -> Self {
        let new_val = (self.0 as i16 + semitones as i16).rem_euclid(12) as u8;
        PitchClass(new_val)
    }

    /// Get interval in semitones to another pitch class (ascending)
    pub fn interval_to(&self, other: PitchClass) -> u8 {
        ((other.0 as i16 - self.0 as i16).rem_euclid(12)) as u8
    }

    /// Get the natural note name (without accidentals for display)
    pub fn natural_name(&self) -> &'static str {
        match self.0 {
            0 => "C",
            1 => "C#",
            2 => "D",
            3 => "Eb",
            4 => "E",
            5 => "F",
            6 => "F#",
            7 => "G",
            8 => "Ab",
            9 => "A",
            10 => "Bb",
            11 => "B",
            _ => unreachable!(),
        }
    }

    /// All pitch classes in chromatic order
    pub fn all() -> [PitchClass; 12] {
        [
            PitchClass::C,
            PitchClass::C_SHARP,
            PitchClass::D,
            PitchClass::D_SHARP,
            PitchClass::E,
            PitchClass::F,
            PitchClass::F_SHARP,
            PitchClass::G,
            PitchClass::G_SHARP,
            PitchClass::A,
            PitchClass::A_SHARP,
            PitchClass::B,
        ]
    }
}

impl fmt::Display for PitchClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.natural_name())
    }
}

/// Absolute pitch (MIDI note number with pitch class and octave)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Pitch {
    /// MIDI note number (0-127, where 60 = middle C)
    midi: u8,
}

impl Pitch {
    /// Middle C (C4)
    pub const MIDDLE_C: Pitch = Pitch { midi: 60 };

    /// Create a pitch from MIDI note number
    pub fn from_midi(midi: u8) -> Result<Self> {
        if midi > 127 {
            return Err(MozartError::InvalidPitch(format!(
                "MIDI note {} out of range (0-127)",
                midi
            )));
        }
        Ok(Pitch { midi })
    }

    /// Create a pitch from pitch class and octave
    /// Octave follows scientific pitch notation (middle C = C4)
    pub fn new(pitch_class: PitchClass, octave: i8) -> Result<Self> {
        let midi = (octave + 1) as i16 * 12 + pitch_class.semitones() as i16;
        if midi < 0 || midi > 127 {
            return Err(MozartError::InvalidPitch(format!(
                "Pitch {}{} out of MIDI range",
                pitch_class, octave
            )));
        }
        Ok(Pitch { midi: midi as u8 })
    }

    /// Parse a pitch from string (e.g., "C4", "F#5", "Bb3")
    pub fn parse(s: &str) -> Result<Self> {
        let s = s.trim();
        tracing::trace!("Parsing pitch: {}", s);

        // Find where the octave number starts
        let octave_start = s
            .chars()
            .position(|c| c.is_ascii_digit() || c == '-')
            .ok_or_else(|| MozartError::ParseError(format!("No octave in pitch: {}", s)))?;

        let pitch_class_str = &s[..octave_start];
        let octave_str = &s[octave_start..];

        let pitch_class = PitchClass::parse(pitch_class_str)?;
        let octave: i8 = octave_str
            .parse()
            .map_err(|_| MozartError::ParseError(format!("Invalid octave: {}", octave_str)))?;

        Self::new(pitch_class, octave)
    }

    /// Get the MIDI note number
    pub fn midi(&self) -> u8 {
        self.midi
    }

    /// Get the pitch class
    pub fn pitch_class(&self) -> PitchClass {
        PitchClass::new(self.midi % 12)
    }

    /// Get the octave (scientific pitch notation)
    pub fn octave(&self) -> i8 {
        (self.midi / 12) as i8 - 1
    }

    /// Transpose by semitones
    pub fn transpose(&self, semitones: i8) -> Result<Self> {
        let new_midi = self.midi as i16 + semitones as i16;
        if new_midi < 0 || new_midi > 127 {
            return Err(MozartError::TranspositionError(format!(
                "Transposition would put note out of MIDI range: {} + {} = {}",
                self.midi, semitones, new_midi
            )));
        }
        Ok(Pitch {
            midi: new_midi as u8,
        })
    }

    /// Get frequency in Hz (A4 = 440 Hz)
    pub fn frequency(&self) -> f64 {
        440.0 * 2.0_f64.powf((self.midi as f64 - 69.0) / 12.0)
    }
}

impl fmt::Display for Pitch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.pitch_class(), self.octave())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_pitch_class_parse() {
        assert_eq!(PitchClass::parse("C").unwrap(), PitchClass::C);
        assert_eq!(PitchClass::parse("c").unwrap(), PitchClass::C);
        assert_eq!(PitchClass::parse("C#").unwrap(), PitchClass::C_SHARP);
        assert_eq!(PitchClass::parse("Db").unwrap(), PitchClass::D_FLAT);
        assert_eq!(PitchClass::parse("F#").unwrap(), PitchClass::F_SHARP);
        assert_eq!(PitchClass::parse("Bb").unwrap(), PitchClass::B_FLAT);
    }

    #[test]
    fn test_pitch_class_transpose() {
        assert_eq!(PitchClass::C.transpose(2), PitchClass::D);
        assert_eq!(PitchClass::C.transpose(12), PitchClass::C);
        assert_eq!(PitchClass::C.transpose(-1), PitchClass::B);
        assert_eq!(PitchClass::A.transpose(3), PitchClass::C);
    }

    #[test]
    fn test_pitch_parse() {
        let c4 = Pitch::parse("C4").unwrap();
        assert_eq!(c4.midi(), 60);
        assert_eq!(c4.pitch_class(), PitchClass::C);
        assert_eq!(c4.octave(), 4);

        let fsharp5 = Pitch::parse("F#5").unwrap();
        assert_eq!(fsharp5.midi(), 78);

        let bb3 = Pitch::parse("Bb3").unwrap();
        assert_eq!(bb3.midi(), 58);
    }

    #[test]
    fn test_pitch_transpose() {
        let c4 = Pitch::MIDDLE_C;
        let d4 = c4.transpose(2).unwrap();
        assert_eq!(d4.midi(), 62);
        assert_eq!(d4.pitch_class(), PitchClass::D);
        assert_eq!(d4.octave(), 4);

        let c5 = c4.transpose(12).unwrap();
        assert_eq!(c5.midi(), 72);
        assert_eq!(c5.octave(), 5);
    }

    #[test]
    fn test_pitch_frequency() {
        let a4 = Pitch::from_midi(69).unwrap();
        assert!((a4.frequency() - 440.0).abs() < 0.001);

        let a5 = Pitch::from_midi(81).unwrap();
        assert!((a5.frequency() - 880.0).abs() < 0.001);
    }
}
