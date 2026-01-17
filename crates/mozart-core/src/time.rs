//! Time signature and accent patterns
//!
//! Supports time signatures from 2-15 with customizable accent patterns

use serde::{Deserialize, Serialize};
use std::fmt;
use crate::error::{MozartError, Result};

/// Accent level for a beat
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum AccentLevel {
    /// Weak beat (normal volume)
    Weak = 1,
    /// Medium accent (secondary emphasis)
    Medium = 2,
    /// Strong accent (downbeat)
    Strong = 3,
}

impl AccentLevel {
    /// Get velocity multiplier for this accent level
    pub fn velocity_multiplier(&self) -> f32 {
        match self {
            AccentLevel::Weak => 0.7,
            AccentLevel::Medium => 0.85,
            AccentLevel::Strong => 1.0,
        }
    }

    /// Parse from numeric value
    pub fn from_value(v: u8) -> Self {
        match v {
            3 => AccentLevel::Strong,
            2 => AccentLevel::Medium,
            _ => AccentLevel::Weak,
        }
    }
}

impl fmt::Display for AccentLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let symbol = match self {
            AccentLevel::Strong => ">",
            AccentLevel::Medium => "-",
            AccentLevel::Weak => ".",
        };
        write!(f, "{}", symbol)
    }
}

/// Accent pattern for a time signature
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccentPattern {
    /// Accent levels for each beat in the measure
    pub accents: Vec<AccentLevel>,
}

impl AccentPattern {
    /// Create a new accent pattern
    pub fn new(accents: Vec<AccentLevel>) -> Self {
        AccentPattern { accents }
    }

    /// Create a pattern from numeric values (1=weak, 2=medium, 3=strong)
    pub fn from_values(values: &[u8]) -> Self {
        AccentPattern {
            accents: values.iter().map(|&v| AccentLevel::from_value(v)).collect(),
        }
    }

    /// Create a default pattern for a given number of beats
    pub fn default_for_beats(beats: u8) -> Self {
        tracing::debug!("Creating default accent pattern for {} beats", beats);

        let pattern = match beats {
            2 => vec![AccentLevel::Strong, AccentLevel::Weak],
            3 => vec![AccentLevel::Strong, AccentLevel::Weak, AccentLevel::Weak],
            4 => vec![
                AccentLevel::Strong,
                AccentLevel::Weak,
                AccentLevel::Medium,
                AccentLevel::Weak,
            ],
            5 => vec![
                AccentLevel::Strong,
                AccentLevel::Weak,
                AccentLevel::Weak,
                AccentLevel::Medium,
                AccentLevel::Weak,
            ], // 3+2
            6 => vec![
                AccentLevel::Strong,
                AccentLevel::Weak,
                AccentLevel::Weak,
                AccentLevel::Medium,
                AccentLevel::Weak,
                AccentLevel::Weak,
            ], // 3+3
            7 => vec![
                AccentLevel::Strong,
                AccentLevel::Weak,
                AccentLevel::Weak,
                AccentLevel::Medium,
                AccentLevel::Weak,
                AccentLevel::Medium,
                AccentLevel::Weak,
            ], // 3+2+2
            8 => vec![
                AccentLevel::Strong,
                AccentLevel::Weak,
                AccentLevel::Medium,
                AccentLevel::Weak,
                AccentLevel::Medium,
                AccentLevel::Weak,
                AccentLevel::Medium,
                AccentLevel::Weak,
            ], // 2+2+2+2
            9 => vec![
                AccentLevel::Strong,
                AccentLevel::Weak,
                AccentLevel::Weak,
                AccentLevel::Medium,
                AccentLevel::Weak,
                AccentLevel::Weak,
                AccentLevel::Medium,
                AccentLevel::Weak,
                AccentLevel::Weak,
            ], // 3+3+3
            10 => vec![
                AccentLevel::Strong,
                AccentLevel::Weak,
                AccentLevel::Weak,
                AccentLevel::Medium,
                AccentLevel::Weak,
                AccentLevel::Medium,
                AccentLevel::Weak,
                AccentLevel::Medium,
                AccentLevel::Weak,
                AccentLevel::Weak,
            ], // 3+2+2+3
            11 => vec![
                AccentLevel::Strong,
                AccentLevel::Weak,
                AccentLevel::Weak,
                AccentLevel::Medium,
                AccentLevel::Weak,
                AccentLevel::Weak,
                AccentLevel::Medium,
                AccentLevel::Weak,
                AccentLevel::Weak,
                AccentLevel::Medium,
                AccentLevel::Weak,
            ], // 3+3+3+2
            12 => vec![
                AccentLevel::Strong,
                AccentLevel::Weak,
                AccentLevel::Weak,
                AccentLevel::Medium,
                AccentLevel::Weak,
                AccentLevel::Weak,
                AccentLevel::Medium,
                AccentLevel::Weak,
                AccentLevel::Weak,
                AccentLevel::Medium,
                AccentLevel::Weak,
                AccentLevel::Weak,
            ], // 3+3+3+3
            13 => vec![
                AccentLevel::Strong,
                AccentLevel::Weak,
                AccentLevel::Weak,
                AccentLevel::Medium,
                AccentLevel::Weak,
                AccentLevel::Weak,
                AccentLevel::Medium,
                AccentLevel::Weak,
                AccentLevel::Weak,
                AccentLevel::Medium,
                AccentLevel::Weak,
                AccentLevel::Medium,
                AccentLevel::Weak,
            ], // 3+3+3+2+2
            14 => vec![
                AccentLevel::Strong,
                AccentLevel::Weak,
                AccentLevel::Medium,
                AccentLevel::Weak,
                AccentLevel::Medium,
                AccentLevel::Weak,
                AccentLevel::Medium,
                AccentLevel::Weak,
                AccentLevel::Medium,
                AccentLevel::Weak,
                AccentLevel::Medium,
                AccentLevel::Weak,
                AccentLevel::Medium,
                AccentLevel::Weak,
            ], // 2+2+2+2+2+2+2
            15 => vec![
                AccentLevel::Strong,
                AccentLevel::Weak,
                AccentLevel::Weak,
                AccentLevel::Medium,
                AccentLevel::Weak,
                AccentLevel::Weak,
                AccentLevel::Medium,
                AccentLevel::Weak,
                AccentLevel::Weak,
                AccentLevel::Medium,
                AccentLevel::Weak,
                AccentLevel::Weak,
                AccentLevel::Medium,
                AccentLevel::Weak,
                AccentLevel::Weak,
            ], // 3+3+3+3+3
            _ => {
                // Generic pattern: strong on 1, weak everywhere else
                let mut p = vec![AccentLevel::Weak; beats as usize];
                if !p.is_empty() {
                    p[0] = AccentLevel::Strong;
                }
                p
            }
        };

        AccentPattern { accents: pattern }
    }

    /// Get the number of beats
    pub fn len(&self) -> usize {
        self.accents.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.accents.is_empty()
    }

    /// Get accent at beat (0-indexed)
    pub fn get(&self, beat: usize) -> AccentLevel {
        self.accents.get(beat).copied().unwrap_or(AccentLevel::Weak)
    }

    /// Set accent at beat (0-indexed)
    pub fn set(&mut self, beat: usize, level: AccentLevel) {
        if beat < self.accents.len() {
            self.accents[beat] = level;
        }
    }

    /// Cycle accent at beat (weak -> medium -> strong -> weak)
    pub fn cycle(&mut self, beat: usize) {
        if beat < self.accents.len() {
            self.accents[beat] = match self.accents[beat] {
                AccentLevel::Weak => AccentLevel::Medium,
                AccentLevel::Medium => AccentLevel::Strong,
                AccentLevel::Strong => AccentLevel::Weak,
            };
        }
    }

    /// Format as visual pattern
    pub fn to_visual(&self) -> String {
        self.accents.iter().map(|a| a.to_string()).collect()
    }
}

impl fmt::Display for AccentPattern {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_visual())
    }
}

/// Time signature with customizable accents
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimeSignature {
    /// Beats per measure (2-15)
    pub numerator: u8,
    /// Beat unit (4 = quarter note, 8 = eighth note)
    pub denominator: u8,
    /// Accent pattern for the measure
    pub accents: AccentPattern,
}

impl TimeSignature {
    /// Create a new time signature with default accents
    pub fn new(numerator: u8, denominator: u8) -> Result<Self> {
        Self::validate(numerator, denominator)?;

        let accents = AccentPattern::default_for_beats(numerator);
        tracing::info!(
            "Created time signature {}/{} with accents: {}",
            numerator,
            denominator,
            accents
        );

        Ok(TimeSignature {
            numerator,
            denominator,
            accents,
        })
    }

    /// Create with custom accent pattern
    pub fn with_accents(numerator: u8, denominator: u8, accents: AccentPattern) -> Result<Self> {
        Self::validate(numerator, denominator)?;

        // Ensure accent pattern matches numerator
        let accents = if accents.len() != numerator as usize {
            tracing::warn!(
                "Accent pattern length {} doesn't match numerator {}, using default",
                accents.len(),
                numerator
            );
            AccentPattern::default_for_beats(numerator)
        } else {
            accents
        };

        Ok(TimeSignature {
            numerator,
            denominator,
            accents,
        })
    }

    fn validate(numerator: u8, denominator: u8) -> Result<()> {
        if numerator < 2 || numerator > 15 {
            return Err(MozartError::InvalidTimeSignature {
                numerator,
                denominator,
            });
        }

        if denominator != 4 && denominator != 8 && denominator != 2 && denominator != 16 {
            return Err(MozartError::InvalidTimeSignature {
                numerator,
                denominator,
            });
        }

        Ok(())
    }

    /// Common time (4/4)
    pub fn common() -> Self {
        TimeSignature::new(4, 4).unwrap()
    }

    /// Waltz time (3/4)
    pub fn waltz() -> Self {
        TimeSignature::new(3, 4).unwrap()
    }

    /// Cut time (2/2)
    pub fn cut() -> Self {
        TimeSignature::new(2, 2).unwrap()
    }

    /// 6/8 compound duple
    pub fn compound_duple() -> Self {
        TimeSignature::new(6, 8).unwrap()
    }

    /// Get ticks per beat based on denominator
    pub fn ticks_per_beat(&self) -> u32 {
        use crate::TICKS_PER_QUARTER;
        match self.denominator {
            2 => TICKS_PER_QUARTER * 2,  // Half note
            4 => TICKS_PER_QUARTER,      // Quarter note
            8 => TICKS_PER_QUARTER / 2,  // Eighth note
            16 => TICKS_PER_QUARTER / 4, // Sixteenth note
            _ => TICKS_PER_QUARTER,
        }
    }

    /// Get ticks per measure
    pub fn ticks_per_measure(&self) -> u32 {
        self.ticks_per_beat() * self.numerator as u32
    }

    /// Get which beat a given tick falls on (0-indexed)
    pub fn beat_at_tick(&self, tick: u32) -> u32 {
        (tick % self.ticks_per_measure()) / self.ticks_per_beat()
    }

    /// Get the accent level at a given tick
    pub fn accent_at_tick(&self, tick: u32) -> AccentLevel {
        let beat = self.beat_at_tick(tick) as usize;
        self.accents.get(beat)
    }

    /// Check if tick is on a beat boundary
    pub fn is_on_beat(&self, tick: u32) -> bool {
        tick % self.ticks_per_beat() == 0
    }

    /// Check if tick is on the downbeat
    pub fn is_downbeat(&self, tick: u32) -> bool {
        tick % self.ticks_per_measure() == 0
    }

    /// Set the accent pattern
    pub fn set_accents(&mut self, accents: AccentPattern) {
        if accents.len() == self.numerator as usize {
            self.accents = accents;
        } else {
            tracing::warn!(
                "Cannot set accent pattern with {} beats for {}/{} time",
                accents.len(),
                self.numerator,
                self.denominator
            );
        }
    }

    /// Parse from string (e.g., "4/4", "7/8")
    pub fn parse(s: &str) -> Result<Self> {
        let parts: Vec<&str> = s.split('/').collect();
        if parts.len() != 2 {
            return Err(MozartError::ParseError(format!(
                "Invalid time signature format: {}",
                s
            )));
        }

        let numerator: u8 = parts[0]
            .trim()
            .parse()
            .map_err(|_| MozartError::ParseError(format!("Invalid numerator: {}", parts[0])))?;

        let denominator: u8 = parts[1]
            .trim()
            .parse()
            .map_err(|_| MozartError::ParseError(format!("Invalid denominator: {}", parts[1])))?;

        TimeSignature::new(numerator, denominator)
    }
}

impl fmt::Display for TimeSignature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.numerator, self.denominator)
    }
}

impl Default for TimeSignature {
    fn default() -> Self {
        TimeSignature::common()
    }
}

/// Predefined grouping patterns for odd meters
pub mod groupings {
    use super::*;

    /// 5/8 groupings
    pub fn five_three_two() -> AccentPattern {
        AccentPattern::from_values(&[3, 1, 1, 2, 1])
    }

    pub fn five_two_three() -> AccentPattern {
        AccentPattern::from_values(&[3, 1, 2, 1, 1])
    }

    /// 7/8 groupings
    pub fn seven_three_two_two() -> AccentPattern {
        AccentPattern::from_values(&[3, 1, 1, 2, 1, 2, 1])
    }

    pub fn seven_two_two_three() -> AccentPattern {
        AccentPattern::from_values(&[3, 1, 2, 1, 2, 1, 1])
    }

    pub fn seven_two_three_two() -> AccentPattern {
        AccentPattern::from_values(&[3, 1, 2, 1, 1, 2, 1])
    }

    /// 11/8 groupings
    pub fn eleven_three_three_three_two() -> AccentPattern {
        AccentPattern::from_values(&[3, 1, 1, 2, 1, 1, 2, 1, 1, 2, 1])
    }

    pub fn eleven_three_three_two_three() -> AccentPattern {
        AccentPattern::from_values(&[3, 1, 1, 2, 1, 1, 2, 1, 2, 1, 1])
    }

    pub fn eleven_three_two_three_three() -> AccentPattern {
        AccentPattern::from_values(&[3, 1, 1, 2, 1, 2, 1, 1, 2, 1, 1])
    }

    pub fn eleven_two_three_three_three() -> AccentPattern {
        AccentPattern::from_values(&[3, 1, 2, 1, 1, 2, 1, 1, 2, 1, 1])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_time_signature_creation() {
        let ts = TimeSignature::new(4, 4).unwrap();
        assert_eq!(ts.numerator, 4);
        assert_eq!(ts.denominator, 4);
        assert_eq!(ts.accents.len(), 4);
    }

    #[test]
    fn test_time_signature_validation() {
        assert!(TimeSignature::new(1, 4).is_err()); // Too few beats
        assert!(TimeSignature::new(16, 4).is_err()); // Too many beats
        assert!(TimeSignature::new(4, 3).is_err()); // Invalid denominator
    }

    #[test]
    fn test_ticks_per_measure() {
        let ts44 = TimeSignature::new(4, 4).unwrap();
        assert_eq!(ts44.ticks_per_measure(), 1920); // 4 * 480

        let ts78 = TimeSignature::new(7, 8).unwrap();
        assert_eq!(ts78.ticks_per_measure(), 1680); // 7 * 240
    }

    #[test]
    fn test_beat_at_tick() {
        let ts = TimeSignature::new(4, 4).unwrap();
        assert_eq!(ts.beat_at_tick(0), 0);
        assert_eq!(ts.beat_at_tick(480), 1);
        assert_eq!(ts.beat_at_tick(960), 2);
        assert_eq!(ts.beat_at_tick(1440), 3);
        assert_eq!(ts.beat_at_tick(1920), 0); // Next measure
    }

    #[test]
    fn test_default_accents() {
        let ts = TimeSignature::common();
        assert_eq!(ts.accents.get(0), AccentLevel::Strong);
        assert_eq!(ts.accents.get(1), AccentLevel::Weak);
        assert_eq!(ts.accents.get(2), AccentLevel::Medium);
        assert_eq!(ts.accents.get(3), AccentLevel::Weak);
    }

    #[test]
    fn test_accent_pattern_cycle() {
        let mut pattern = AccentPattern::default_for_beats(4);
        assert_eq!(pattern.get(1), AccentLevel::Weak);

        pattern.cycle(1);
        assert_eq!(pattern.get(1), AccentLevel::Medium);

        pattern.cycle(1);
        assert_eq!(pattern.get(1), AccentLevel::Strong);

        pattern.cycle(1);
        assert_eq!(pattern.get(1), AccentLevel::Weak);
    }

    #[test]
    fn test_time_signature_parse() {
        let ts = TimeSignature::parse("7/8").unwrap();
        assert_eq!(ts.numerator, 7);
        assert_eq!(ts.denominator, 8);
    }

    #[test]
    fn test_odd_meter_defaults() {
        // 7/8 should have a sensible grouping
        let ts7 = TimeSignature::new(7, 8).unwrap();
        assert_eq!(ts7.accents.len(), 7);
        assert_eq!(ts7.accents.get(0), AccentLevel::Strong);

        // 11/8
        let ts11 = TimeSignature::new(11, 8).unwrap();
        assert_eq!(ts11.accents.len(), 11);
        assert_eq!(ts11.accents.get(0), AccentLevel::Strong);
    }
}
