//! Note representation
//!
//! A Note combines pitch, timing, duration, and velocity

use serde::{Deserialize, Serialize};
use std::fmt;
use crate::error::{MozartError, Result};
use crate::pitch::Pitch;
use crate::TICKS_PER_QUARTER;

/// Standard note duration values
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NoteValue {
    Whole,
    Half,
    Quarter,
    Eighth,
    Sixteenth,
}

impl NoteValue {
    /// Get duration in ticks
    pub fn ticks(&self) -> u32 {
        match self {
            NoteValue::Whole => TICKS_PER_QUARTER * 4,
            NoteValue::Half => TICKS_PER_QUARTER * 2,
            NoteValue::Quarter => TICKS_PER_QUARTER,
            NoteValue::Eighth => TICKS_PER_QUARTER / 2,
            NoteValue::Sixteenth => TICKS_PER_QUARTER / 4,
        }
    }

    /// Parse from string (w, h, q, e, s)
    pub fn parse(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "w" | "whole" | "1" => Ok(NoteValue::Whole),
            "h" | "half" | "2" => Ok(NoteValue::Half),
            "q" | "quarter" | "4" => Ok(NoteValue::Quarter),
            "e" | "eighth" | "8" => Ok(NoteValue::Eighth),
            "s" | "sixteenth" | "16" => Ok(NoteValue::Sixteenth),
            _ => Err(MozartError::InvalidDuration(format!(
                "Unknown note value: {}",
                s
            ))),
        }
    }

    /// Get short name
    pub fn short_name(&self) -> &'static str {
        match self {
            NoteValue::Whole => "w",
            NoteValue::Half => "h",
            NoteValue::Quarter => "q",
            NoteValue::Eighth => "e",
            NoteValue::Sixteenth => "s",
        }
    }
}

impl fmt::Display for NoteValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            NoteValue::Whole => "whole",
            NoteValue::Half => "half",
            NoteValue::Quarter => "quarter",
            NoteValue::Eighth => "eighth",
            NoteValue::Sixteenth => "sixteenth",
        };
        write!(f, "{}", name)
    }
}

/// Note duration with optional dot
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct NoteDuration {
    pub value: NoteValue,
    pub dotted: bool,
}

impl NoteDuration {
    pub fn new(value: NoteValue) -> Self {
        NoteDuration {
            value,
            dotted: false,
        }
    }

    pub fn dotted(value: NoteValue) -> Self {
        NoteDuration {
            value,
            dotted: true,
        }
    }

    /// Get duration in ticks
    pub fn ticks(&self) -> u32 {
        let base = self.value.ticks();
        if self.dotted {
            base + base / 2
        } else {
            base
        }
    }

    /// Create from raw ticks (finds closest match)
    pub fn from_ticks(ticks: u32) -> Self {
        // Check dotted values first (they're between regular values)
        let values = [
            NoteValue::Whole,
            NoteValue::Half,
            NoteValue::Quarter,
            NoteValue::Eighth,
            NoteValue::Sixteenth,
        ];

        let mut best_match = NoteDuration::new(NoteValue::Quarter);
        let mut best_diff = u32::MAX;

        for value in values {
            // Regular
            let regular = NoteDuration::new(value);
            let diff = (regular.ticks() as i64 - ticks as i64).unsigned_abs() as u32;
            if diff < best_diff {
                best_diff = diff;
                best_match = regular;
            }

            // Dotted
            let dotted = NoteDuration::dotted(value);
            let diff = (dotted.ticks() as i64 - ticks as i64).unsigned_abs() as u32;
            if diff < best_diff {
                best_diff = diff;
                best_match = dotted;
            }
        }

        best_match
    }
}

impl fmt::Display for NoteDuration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.dotted {
            write!(f, "{}.", self.value.short_name())
        } else {
            write!(f, "{}", self.value.short_name())
        }
    }
}

/// A musical note with pitch, timing, duration, and velocity
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Note {
    /// MIDI pitch (0-127)
    pub pitch: u8,
    /// Start position in ticks from beginning of song
    pub start_tick: u32,
    /// Duration in ticks
    pub duration_ticks: u32,
    /// Velocity (0-127, default 100)
    pub velocity: u8,
    /// Voice/layer (0=main melody, 1+=harmony voices)
    #[serde(default)]
    pub voice: u8,
}

impl Note {
    /// Create a new note
    pub fn new(pitch: u8, start_tick: u32, duration_ticks: u32) -> Self {
        Note {
            pitch,
            start_tick,
            duration_ticks,
            velocity: 100,
            voice: 0,
        }
    }

    /// Create a note with specific velocity
    pub fn with_velocity(pitch: u8, start_tick: u32, duration_ticks: u32, velocity: u8) -> Self {
        Note {
            pitch,
            start_tick,
            duration_ticks,
            velocity: velocity.min(127),
            voice: 0,
        }
    }

    /// Create a note with specific velocity and voice
    pub fn with_voice(pitch: u8, start_tick: u32, duration_ticks: u32, velocity: u8, voice: u8) -> Self {
        Note {
            pitch,
            start_tick,
            duration_ticks,
            velocity: velocity.min(127),
            voice,
        }
    }

    /// Create from Pitch and NoteDuration
    pub fn from_pitch(pitch: Pitch, start_tick: u32, duration: NoteDuration) -> Self {
        Note {
            pitch: pitch.midi(),
            start_tick,
            duration_ticks: duration.ticks(),
            velocity: 100,
            voice: 0,
        }
    }

    /// Get the pitch
    pub fn pitch(&self) -> Result<Pitch> {
        Pitch::from_midi(self.pitch)
    }

    /// Get the end tick
    pub fn end_tick(&self) -> u32 {
        self.start_tick + self.duration_ticks
    }

    /// Get the duration as NoteDuration (closest match)
    pub fn duration(&self) -> NoteDuration {
        NoteDuration::from_ticks(self.duration_ticks)
    }

    /// Parse from text notation: "C4q" or "F#5h." etc.
    pub fn parse(s: &str, start_tick: u32) -> Result<Self> {
        let s = s.trim();
        tracing::debug!("Parsing note: {} at tick {}", s, start_tick);

        if s.is_empty() {
            return Err(MozartError::ParseError("Empty note string".to_string()));
        }

        // Find where the duration starts (first letter after digits)
        let mut pitch_end = s.len();
        let mut found_octave = false;

        for (i, c) in s.char_indices() {
            if c.is_ascii_digit() || c == '-' {
                found_octave = true;
            } else if found_octave && c.is_ascii_alphabetic() {
                pitch_end = i;
                break;
            }
        }

        let pitch_str = &s[..pitch_end];
        let duration_str = &s[pitch_end..];

        let pitch = Pitch::parse(pitch_str)?;

        let duration = if duration_str.is_empty() {
            NoteDuration::new(NoteValue::Quarter) // Default to quarter note
        } else {
            let dotted = duration_str.ends_with('.');
            let value_str = if dotted {
                &duration_str[..duration_str.len() - 1]
            } else {
                duration_str
            };
            let value = NoteValue::parse(value_str)?;
            NoteDuration { value, dotted }
        };

        tracing::debug!(
            "Parsed note: pitch={}, duration={}, ticks={}",
            pitch,
            duration,
            duration.ticks()
        );

        Ok(Note::from_pitch(pitch, start_tick, duration))
    }

    /// Format as text notation
    pub fn to_text(&self) -> String {
        let pitch = Pitch::from_midi(self.pitch).unwrap();
        let duration = self.duration();
        format!("{}{}", pitch, duration)
    }
}

impl fmt::Display for Note {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_text())
    }
}

/// Parse a melody string into notes
/// Format: "C4q D4q E4q F4q" (space-separated)
pub fn parse_melody(s: &str) -> Result<Vec<Note>> {
    tracing::info!("Parsing melody: {}", s);
    let mut notes = Vec::new();
    let mut current_tick: u32 = 0;

    for token in s.split_whitespace() {
        if token.is_empty() {
            continue;
        }

        // Handle rest (R or r followed by duration)
        if token.to_uppercase().starts_with('R') {
            let duration_str = &token[1..];
            let duration = if duration_str.is_empty() {
                NoteDuration::new(NoteValue::Quarter)
            } else {
                let dotted = duration_str.ends_with('.');
                let value_str = if dotted {
                    &duration_str[..duration_str.len() - 1]
                } else {
                    duration_str
                };
                let value = NoteValue::parse(value_str)?;
                NoteDuration { value, dotted }
            };
            current_tick += duration.ticks();
            tracing::trace!("Rest: duration={}, new_tick={}", duration, current_tick);
            continue;
        }

        let note = Note::parse(token, current_tick)?;
        current_tick = note.end_tick();
        notes.push(note);
    }

    tracing::info!("Parsed {} notes, total duration: {} ticks", notes.len(), current_tick);
    Ok(notes)
}

/// Format notes as melody string
pub fn format_melody(notes: &[Note]) -> String {
    notes.iter().map(|n| n.to_text()).collect::<Vec<_>>().join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_note_value_ticks() {
        assert_eq!(NoteValue::Whole.ticks(), 1920);
        assert_eq!(NoteValue::Half.ticks(), 960);
        assert_eq!(NoteValue::Quarter.ticks(), 480);
        assert_eq!(NoteValue::Eighth.ticks(), 240);
        assert_eq!(NoteValue::Sixteenth.ticks(), 120);
    }

    #[test]
    fn test_dotted_duration() {
        let dotted_half = NoteDuration::dotted(NoteValue::Half);
        assert_eq!(dotted_half.ticks(), 1440); // 960 + 480

        let dotted_quarter = NoteDuration::dotted(NoteValue::Quarter);
        assert_eq!(dotted_quarter.ticks(), 720); // 480 + 240
    }

    #[test]
    fn test_note_parse() {
        let note = Note::parse("C4q", 0).unwrap();
        assert_eq!(note.pitch, 60);
        assert_eq!(note.duration_ticks, 480);

        let note = Note::parse("F#5h", 0).unwrap();
        assert_eq!(note.pitch, 78);
        assert_eq!(note.duration_ticks, 960);

        let note = Note::parse("Bb3q.", 0).unwrap();
        assert_eq!(note.pitch, 58);
        assert_eq!(note.duration_ticks, 720);
    }

    #[test]
    fn test_parse_melody() {
        let melody = parse_melody("C4q D4q E4q").unwrap();
        assert_eq!(melody.len(), 3);
        assert_eq!(melody[0].pitch, 60);
        assert_eq!(melody[0].start_tick, 0);
        assert_eq!(melody[1].pitch, 62);
        assert_eq!(melody[1].start_tick, 480);
        assert_eq!(melody[2].pitch, 64);
        assert_eq!(melody[2].start_tick, 960);
    }

    #[test]
    fn test_parse_melody_with_rests() {
        let melody = parse_melody("C4q Rq E4q").unwrap();
        assert_eq!(melody.len(), 2);
        assert_eq!(melody[0].pitch, 60);
        assert_eq!(melody[0].start_tick, 0);
        assert_eq!(melody[1].pitch, 64);
        assert_eq!(melody[1].start_tick, 960); // After quarter note + quarter rest
    }
}
