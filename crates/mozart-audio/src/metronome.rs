//! Metronome with accent support
//!
//! Generates click sounds following the time signature's accent pattern

use mozart_core::time::{AccentLevel, TimeSignature};
use rodio::Source;
use std::time::Duration;

/// Metronome click generator
pub struct Metronome {
    /// Time signature (for accent pattern)
    time_signature: TimeSignature,
    /// Sample rate
    sample_rate: u32,
    /// Base frequency for clicks (Hz)
    base_frequency: f32,
    /// Click duration (seconds)
    click_duration: f32,
    /// Whether metronome is enabled
    enabled: bool,
    /// Volume (0.0 - 1.0)
    volume: f32,
}

impl Metronome {
    /// Create a new metronome
    pub fn new(time_signature: TimeSignature) -> Self {
        Metronome {
            time_signature,
            sample_rate: 44100,
            base_frequency: 1000.0, // Default click frequency
            click_duration: 0.02,    // 20ms click
            enabled: true,
            volume: 0.5,
        }
    }

    /// Set the time signature
    pub fn set_time_signature(&mut self, ts: TimeSignature) {
        tracing::debug!("Metronome: setting time signature to {}", ts);
        self.time_signature = ts;
    }

    /// Enable/disable the metronome
    pub fn set_enabled(&mut self, enabled: bool) {
        tracing::debug!("Metronome: enabled = {}", enabled);
        self.enabled = enabled;
    }

    /// Check if enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Set volume (0.0 - 1.0)
    pub fn set_volume(&mut self, volume: f32) {
        self.volume = volume.clamp(0.0, 1.0);
    }

    /// Get volume
    pub fn volume(&self) -> f32 {
        self.volume
    }

    /// Generate a click sound for the given beat
    pub fn click(&self, beat: u32) -> Option<ClickSource> {
        if !self.enabled {
            return None;
        }

        let accent = self.time_signature.accents.get(beat as usize);
        Some(self.generate_click(accent))
    }

    /// Generate a click for a specific accent level
    fn generate_click(&self, accent: AccentLevel) -> ClickSource {
        // Adjust frequency and amplitude based on accent
        let (freq_mult, amp_mult) = match accent {
            AccentLevel::Strong => (1.5, 1.0),   // Higher pitch, full volume
            AccentLevel::Medium => (1.25, 0.75), // Medium pitch and volume
            AccentLevel::Weak => (1.0, 0.5),     // Base pitch, half volume
        };

        ClickSource {
            sample_rate: self.sample_rate,
            frequency: self.base_frequency * freq_mult,
            amplitude: self.volume * amp_mult,
            duration_samples: (self.click_duration * self.sample_rate as f32) as usize,
            position: 0,
        }
    }

    /// Get the time signature
    pub fn time_signature(&self) -> &TimeSignature {
        &self.time_signature
    }
}

/// A rodio Source for a metronome click
pub struct ClickSource {
    sample_rate: u32,
    frequency: f32,
    amplitude: f32,
    duration_samples: usize,
    position: usize,
}

impl Iterator for ClickSource {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        if self.position >= self.duration_samples {
            return None;
        }

        let t = self.position as f32 / self.sample_rate as f32;

        // Sine wave with exponential decay
        let decay = (-t * 50.0).exp(); // Quick decay
        let sample = (2.0 * std::f32::consts::PI * self.frequency * t).sin()
            * decay
            * self.amplitude;

        self.position += 1;
        Some(sample)
    }
}

impl Source for ClickSource {
    fn current_frame_len(&self) -> Option<usize> {
        Some(self.duration_samples - self.position)
    }

    fn channels(&self) -> u16 {
        1
    }

    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn total_duration(&self) -> Option<Duration> {
        Some(Duration::from_secs_f32(
            self.duration_samples as f32 / self.sample_rate as f32,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metronome_creation() {
        let ts = TimeSignature::common();
        let metro = Metronome::new(ts);
        assert!(metro.is_enabled());
    }

    #[test]
    fn test_metronome_click_generation() {
        let ts = TimeSignature::common();
        let metro = Metronome::new(ts);

        // Beat 0 should be strong
        let click = metro.click(0).unwrap();
        assert!(click.frequency > metro.base_frequency);

        // Beat 1 should be weak
        let click = metro.click(1).unwrap();
        assert!((click.frequency - metro.base_frequency).abs() < 0.01);

        // Beat 2 should be medium
        let click = metro.click(2).unwrap();
        assert!(click.frequency > metro.base_frequency);
        assert!(click.frequency < metro.base_frequency * 1.5);
    }

    #[test]
    fn test_metronome_disabled() {
        let ts = TimeSignature::common();
        let mut metro = Metronome::new(ts);
        metro.set_enabled(false);

        assert!(metro.click(0).is_none());
    }

    #[test]
    fn test_click_source_length() {
        let ts = TimeSignature::common();
        let metro = Metronome::new(ts);
        let click = metro.click(0).unwrap();

        // Should have the expected number of samples
        let samples: Vec<f32> = click.collect();
        let expected = (0.02 * 44100.0) as usize;
        assert_eq!(samples.len(), expected);
    }
}
