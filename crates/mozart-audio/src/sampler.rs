//! Sample-based instrument playback
//!
//! Loads WAV samples and plays them at correct pitches

use crate::error::{AudioError, Result};
use parking_lot::RwLock;
use rodio::Source;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

/// Available instruments
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Instrument {
    Piano,
    Strings,
    Synth,
}

impl Instrument {
    /// Get display name
    pub fn name(&self) -> &'static str {
        match self {
            Instrument::Piano => "Piano",
            Instrument::Strings => "Strings",
            Instrument::Synth => "Synth",
        }
    }

    /// All available instruments
    pub fn all() -> &'static [Instrument] {
        &[Instrument::Piano, Instrument::Strings, Instrument::Synth]
    }
}

/// A loaded audio sample
#[derive(Clone)]
pub struct Sample {
    /// Raw sample data (mono, f32)
    data: Arc<Vec<f32>>,
    /// Sample rate
    sample_rate: u32,
    /// Base pitch (MIDI note number this sample was recorded at)
    base_pitch: u8,
}

impl Sample {
    /// Create a sample from raw data
    pub fn new(data: Vec<f32>, sample_rate: u32, base_pitch: u8) -> Self {
        Sample {
            data: Arc::new(data),
            sample_rate,
            base_pitch,
        }
    }

    /// Load a WAV file
    pub fn load_wav(path: impl AsRef<Path>, base_pitch: u8) -> Result<Self> {
        let path = path.as_ref();
        tracing::debug!("Loading sample from {:?}", path);

        let reader = hound::WavReader::open(path).map_err(|e| {
            AudioError::SampleError(format!("Failed to open WAV file: {}", e))
        })?;

        let spec = reader.spec();
        let sample_rate = spec.sample_rate;

        // Convert to mono f32
        let data: Vec<f32> = match spec.sample_format {
            hound::SampleFormat::Float => {
                reader.into_samples::<f32>().filter_map(|s| s.ok()).collect()
            }
            hound::SampleFormat::Int => {
                let bits = spec.bits_per_sample;
                let max = (1 << (bits - 1)) as f32;
                reader
                    .into_samples::<i32>()
                    .filter_map(|s| s.ok())
                    .map(|s| s as f32 / max)
                    .collect()
            }
        };

        // Convert stereo to mono if needed
        let data = if spec.channels == 2 {
            data.chunks(2)
                .map(|c| (c[0] + c.get(1).unwrap_or(&0.0)) / 2.0)
                .collect()
        } else {
            data
        };

        tracing::debug!(
            "Loaded sample: {} samples at {} Hz, base pitch {}",
            data.len(),
            sample_rate,
            base_pitch
        );

        Ok(Sample::new(data, sample_rate, base_pitch))
    }

    /// Create a simple sine wave sample for testing/fallback
    pub fn sine_wave(frequency: f32, duration_secs: f32, sample_rate: u32) -> Self {
        let num_samples = (sample_rate as f32 * duration_secs) as usize;
        let data: Vec<f32> = (0..num_samples)
            .map(|i| {
                let t = i as f32 / sample_rate as f32;
                (2.0 * std::f32::consts::PI * frequency * t).sin() * 0.5
            })
            .collect();

        // Calculate MIDI pitch from frequency
        let midi_pitch = (12.0 * (frequency / 440.0).log2() + 69.0).round() as u8;

        Sample::new(data, sample_rate, midi_pitch)
    }

    /// Get a source that plays this sample at the given pitch
    pub fn play_at_pitch(&self, target_pitch: u8, velocity: u8) -> SampleSource {
        // Calculate pitch ratio
        let semitone_diff = target_pitch as f32 - self.base_pitch as f32;
        let pitch_ratio = 2.0_f32.powf(semitone_diff / 12.0);

        // Velocity to amplitude
        let amplitude = (velocity as f32 / 127.0).powi(2); // Squared for more natural response

        SampleSource {
            data: Arc::clone(&self.data),
            sample_rate: self.sample_rate,
            position: 0.0,
            pitch_ratio,
            amplitude,
        }
    }
}

/// A rodio Source for playing a sample
pub struct SampleSource {
    data: Arc<Vec<f32>>,
    sample_rate: u32,
    position: f32,
    pitch_ratio: f32,
    amplitude: f32,
}

impl Iterator for SampleSource {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        let pos = self.position as usize;
        if pos >= self.data.len() {
            return None;
        }

        // Linear interpolation
        let frac = self.position - pos as f32;
        let sample = if pos + 1 < self.data.len() {
            self.data[pos] * (1.0 - frac) + self.data[pos + 1] * frac
        } else {
            self.data[pos]
        };

        self.position += self.pitch_ratio;
        Some(sample * self.amplitude)
    }
}

impl Source for SampleSource {
    fn current_frame_len(&self) -> Option<usize> {
        None
    }

    fn channels(&self) -> u16 {
        1
    }

    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn total_duration(&self) -> Option<Duration> {
        let samples = self.data.len() as f32 / self.pitch_ratio;
        Some(Duration::from_secs_f32(samples / self.sample_rate as f32))
    }
}

/// Sample library for an instrument
pub struct Sampler {
    /// Loaded samples keyed by MIDI pitch
    samples: RwLock<HashMap<u8, Sample>>,
    /// Fallback sample for pitches without dedicated samples
    fallback: RwLock<Option<Sample>>,
    /// Current instrument
    instrument: Instrument,
}

impl Sampler {
    /// Create a new sampler for an instrument
    pub fn new(instrument: Instrument) -> Self {
        tracing::info!("Creating sampler for {:?}", instrument);
        Sampler {
            samples: RwLock::new(HashMap::new()),
            fallback: RwLock::new(None),
            instrument,
        }
    }

    /// Create a sampler with a sine wave fallback (for testing)
    pub fn with_sine_fallback(instrument: Instrument) -> Self {
        let sampler = Self::new(instrument);
        let sine = Sample::sine_wave(440.0, 2.0, 44100); // A4 sine wave
        *sampler.fallback.write() = Some(sine);
        sampler
    }

    /// Load samples from a directory
    pub fn load_samples(&self, dir: impl AsRef<Path>) -> Result<()> {
        let dir = dir.as_ref();
        tracing::info!("Loading samples from {:?}", dir);

        if !dir.exists() {
            tracing::warn!("Sample directory does not exist: {:?}", dir);
            return Ok(());
        }

        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().map_or(false, |e| e == "wav") {
                // Try to parse pitch from filename (e.g., "60.wav" or "C4.wav")
                if let Some(pitch) = self.parse_pitch_from_filename(&path) {
                    match Sample::load_wav(&path, pitch) {
                        Ok(sample) => {
                            self.samples.write().insert(pitch, sample);
                            tracing::debug!("Loaded sample for pitch {}", pitch);
                        }
                        Err(e) => {
                            tracing::warn!("Failed to load {:?}: {}", path, e);
                        }
                    }
                }
            }
        }

        tracing::info!(
            "Loaded {} samples for {:?}",
            self.samples.read().len(),
            self.instrument
        );
        Ok(())
    }

    fn parse_pitch_from_filename(&self, path: &Path) -> Option<u8> {
        let stem = path.file_stem()?.to_str()?;

        // Try numeric pitch first
        if let Ok(pitch) = stem.parse::<u8>() {
            return Some(pitch);
        }

        // Try note name (e.g., "C4", "F#5")
        mozart_core::pitch::Pitch::parse(stem).ok().map(|p| p.midi())
    }

    /// Add a sample manually
    pub fn add_sample(&self, pitch: u8, sample: Sample) {
        self.samples.write().insert(pitch, sample);
    }

    /// Set the fallback sample
    pub fn set_fallback(&self, sample: Sample) {
        *self.fallback.write() = Some(sample);
    }

    /// Get a source for playing a note
    pub fn play(&self, pitch: u8, velocity: u8) -> Option<SampleSource> {
        let samples = self.samples.read();

        // Try exact pitch match
        if let Some(sample) = samples.get(&pitch) {
            return Some(sample.play_at_pitch(pitch, velocity));
        }

        // Find nearest sample
        let nearest = samples
            .keys()
            .min_by_key(|&&p| (p as i16 - pitch as i16).abs());

        if let Some(&nearest_pitch) = nearest {
            let sample = samples.get(&nearest_pitch).unwrap();
            return Some(sample.play_at_pitch(pitch, velocity));
        }

        // Use fallback
        let fallback = self.fallback.read();
        fallback.as_ref().map(|s| s.play_at_pitch(pitch, velocity))
    }

    /// Get the current instrument
    pub fn instrument(&self) -> Instrument {
        self.instrument
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sine_wave_sample() {
        let sample = Sample::sine_wave(440.0, 0.1, 44100);
        assert!(!sample.data.is_empty());
        assert_eq!(sample.sample_rate, 44100);
    }

    #[test]
    fn test_sampler_with_fallback() {
        let sampler = Sampler::with_sine_fallback(Instrument::Piano);

        // Should be able to play any pitch using the fallback
        let source = sampler.play(60, 100);
        assert!(source.is_some());

        let source = sampler.play(72, 100);
        assert!(source.is_some());
    }

    #[test]
    fn test_pitch_transposition() {
        let sample = Sample::sine_wave(440.0, 0.1, 44100); // A4
        let source = sample.play_at_pitch(69, 127); // Play at A4

        // Pitch ratio should be 1.0 for same pitch
        assert!((source.pitch_ratio - 1.0).abs() < 0.001);

        // Play up an octave
        let source = sample.play_at_pitch(81, 127); // A5
        assert!((source.pitch_ratio - 2.0).abs() < 0.001);
    }
}
