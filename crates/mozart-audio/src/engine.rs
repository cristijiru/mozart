//! Audio playback engine
//!
//! Manages playback of songs with metronome and transport controls

use crate::error::{AudioError, Result};
use crate::metronome::Metronome;
use crate::sampler::{Instrument, Sampler};
use mozart_core::song::Song;
use mozart_core::time::TimeSignature;
use mozart_core::TICKS_PER_QUARTER;
use parking_lot::{Mutex, RwLock};
use rodio::{OutputStream, OutputStreamHandle, Sink, Source};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Playback state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlaybackState {
    Stopped,
    Playing,
    Paused,
}

/// Transport control commands
#[derive(Debug, Clone)]
pub enum TransportCommand {
    Play,
    Pause,
    Stop,
    SetPosition(u32), // tick position
    SetTempo(u16),    // BPM
    SetLoop(Option<(u32, u32)>), // loop start/end in ticks
    ToggleMetronome,
}

/// Audio engine for playing songs
pub struct AudioEngine {
    /// Rodio output stream (must be kept alive)
    _stream: OutputStream,
    /// Stream handle for creating sinks
    stream_handle: OutputStreamHandle,
    /// Main playback sink
    sink: Arc<Mutex<Sink>>,
    /// Metronome sink
    metronome_sink: Arc<Mutex<Sink>>,
    /// Current sampler
    sampler: Arc<RwLock<Sampler>>,
    /// Metronome
    metronome: Arc<RwLock<Metronome>>,
    /// Current playback state
    state: Arc<RwLock<PlaybackState>>,
    /// Current position in ticks
    position_ticks: Arc<AtomicU32>,
    /// Current tempo in BPM
    tempo: Arc<AtomicU32>,
    /// Loop enabled
    loop_enabled: Arc<AtomicBool>,
    /// Loop start tick
    loop_start: Arc<AtomicU32>,
    /// Loop end tick
    loop_end: Arc<AtomicU32>,
    /// Playback start time
    playback_start: Arc<RwLock<Option<Instant>>>,
    /// Playback start tick
    playback_start_tick: Arc<AtomicU32>,
}

impl AudioEngine {
    /// Create a new audio engine
    pub fn new() -> Result<Self> {
        tracing::info!("Initializing audio engine");

        let (stream, stream_handle) = OutputStream::try_default()
            .map_err(|e| AudioError::DeviceError(format!("Failed to open audio device: {}", e)))?;

        let sink = Sink::try_new(&stream_handle)
            .map_err(|e| AudioError::StreamError(format!("Failed to create sink: {}", e)))?;

        let metronome_sink = Sink::try_new(&stream_handle)
            .map_err(|e| AudioError::StreamError(format!("Failed to create metronome sink: {}", e)))?;

        let sampler = Sampler::with_sine_fallback(Instrument::Piano);
        let metronome = Metronome::new(TimeSignature::common());

        tracing::info!("Audio engine initialized successfully");

        Ok(AudioEngine {
            _stream: stream,
            stream_handle,
            sink: Arc::new(Mutex::new(sink)),
            metronome_sink: Arc::new(Mutex::new(metronome_sink)),
            sampler: Arc::new(RwLock::new(sampler)),
            metronome: Arc::new(RwLock::new(metronome)),
            state: Arc::new(RwLock::new(PlaybackState::Stopped)),
            position_ticks: Arc::new(AtomicU32::new(0)),
            tempo: Arc::new(AtomicU32::new(120)),
            loop_enabled: Arc::new(AtomicBool::new(false)),
            loop_start: Arc::new(AtomicU32::new(0)),
            loop_end: Arc::new(AtomicU32::new(0)),
            playback_start: Arc::new(RwLock::new(None)),
            playback_start_tick: Arc::new(AtomicU32::new(0)),
        })
    }

    /// Get current playback state
    pub fn state(&self) -> PlaybackState {
        *self.state.read()
    }

    /// Get current position in ticks
    pub fn position(&self) -> u32 {
        // If playing, calculate position from elapsed time
        if *self.state.read() == PlaybackState::Playing {
            if let Some(start) = *self.playback_start.read() {
                let elapsed = start.elapsed();
                let start_tick = self.playback_start_tick.load(Ordering::Relaxed);
                let tempo = self.tempo.load(Ordering::Relaxed) as f64;

                // Convert elapsed time to ticks
                let elapsed_beats = elapsed.as_secs_f64() * tempo / 60.0;
                let elapsed_ticks = (elapsed_beats * TICKS_PER_QUARTER as f64) as u32;

                return start_tick + elapsed_ticks;
            }
        }

        self.position_ticks.load(Ordering::Relaxed)
    }

    /// Get current tempo
    pub fn tempo(&self) -> u16 {
        self.tempo.load(Ordering::Relaxed) as u16
    }

    /// Set tempo
    pub fn set_tempo(&self, bpm: u16) {
        let bpm = bpm.clamp(20, 300);
        tracing::debug!("Setting tempo to {} BPM", bpm);
        self.tempo.store(bpm as u32, Ordering::Relaxed);
    }

    /// Execute a transport command
    pub fn command(&self, cmd: TransportCommand) {
        tracing::debug!("Transport command: {:?}", cmd);

        match cmd {
            TransportCommand::Play => self.play(),
            TransportCommand::Pause => self.pause(),
            TransportCommand::Stop => self.stop(),
            TransportCommand::SetPosition(tick) => {
                self.position_ticks.store(tick, Ordering::Relaxed);
            }
            TransportCommand::SetTempo(bpm) => self.set_tempo(bpm),
            TransportCommand::SetLoop(range) => {
                if let Some((start, end)) = range {
                    self.loop_start.store(start, Ordering::Relaxed);
                    self.loop_end.store(end, Ordering::Relaxed);
                    self.loop_enabled.store(true, Ordering::Relaxed);
                } else {
                    self.loop_enabled.store(false, Ordering::Relaxed);
                }
            }
            TransportCommand::ToggleMetronome => {
                let current = self.metronome.read().is_enabled();
                self.metronome.write().set_enabled(!current);
            }
        }
    }

    fn play(&self) {
        tracing::info!("Starting playback");
        *self.state.write() = PlaybackState::Playing;
        *self.playback_start.write() = Some(Instant::now());
        self.playback_start_tick.store(
            self.position_ticks.load(Ordering::Relaxed),
            Ordering::Relaxed,
        );
        self.sink.lock().play();
    }

    fn pause(&self) {
        tracing::info!("Pausing playback");
        // Store current position before pausing
        self.position_ticks.store(self.position(), Ordering::Relaxed);
        *self.state.write() = PlaybackState::Paused;
        *self.playback_start.write() = None;
        self.sink.lock().pause();
    }

    fn stop(&self) {
        tracing::info!("Stopping playback");
        *self.state.write() = PlaybackState::Stopped;
        *self.playback_start.write() = None;
        self.position_ticks.store(0, Ordering::Relaxed);
        self.sink.lock().stop();

        // Recreate sink for fresh playback
        if let Ok(new_sink) = Sink::try_new(&self.stream_handle) {
            *self.sink.lock() = new_sink;
        }
    }

    /// Play a single note immediately (for preview/testing)
    pub fn play_note(&self, pitch: u8, velocity: u8, duration_ms: u64) {
        tracing::trace!("Playing note: pitch={}, vel={}", pitch, velocity);

        let sampler = self.sampler.read();
        if let Some(source) = sampler.play(pitch, velocity) {
            let source = source.take_duration(Duration::from_millis(duration_ms));
            self.sink.lock().append(source);
        }
    }

    /// Play a metronome click
    pub fn play_click(&self, beat: u32) {
        let metro = self.metronome.read();
        if let Some(click) = metro.click(beat) {
            self.metronome_sink.lock().append(click);
        }
    }

    /// Set the time signature (updates metronome)
    pub fn set_time_signature(&self, ts: TimeSignature) {
        self.metronome.write().set_time_signature(ts);
    }

    /// Load a song for playback
    pub fn load_song(&self, song: &Song) {
        tracing::info!("Loading song for playback: {}", song.metadata.title);

        self.stop();
        self.set_tempo(song.settings.tempo);
        self.set_time_signature(song.settings.time_signature.clone());
    }

    /// Get the sampler
    pub fn sampler(&self) -> &Arc<RwLock<Sampler>> {
        &self.sampler
    }

    /// Set the instrument
    pub fn set_instrument(&self, instrument: Instrument) {
        tracing::info!("Setting instrument to {:?}", instrument);
        *self.sampler.write() = Sampler::with_sine_fallback(instrument);
    }

    /// Get metronome enabled state
    pub fn metronome_enabled(&self) -> bool {
        self.metronome.read().is_enabled()
    }

    /// Set metronome volume
    pub fn set_metronome_volume(&self, volume: f32) {
        self.metronome.write().set_volume(volume);
    }

    /// Check if loop is enabled
    pub fn loop_enabled(&self) -> bool {
        self.loop_enabled.load(Ordering::Relaxed)
    }

    /// Get loop range
    pub fn loop_range(&self) -> Option<(u32, u32)> {
        if self.loop_enabled() {
            Some((
                self.loop_start.load(Ordering::Relaxed),
                self.loop_end.load(Ordering::Relaxed),
            ))
        } else {
            None
        }
    }
}

impl Default for AudioEngine {
    fn default() -> Self {
        Self::new().expect("Failed to create audio engine")
    }
}

/// Tap tempo calculator
pub struct TapTempo {
    taps: Vec<Instant>,
    max_taps: usize,
    timeout: Duration,
}

impl TapTempo {
    /// Create a new tap tempo calculator
    pub fn new() -> Self {
        TapTempo {
            taps: Vec::new(),
            max_taps: 8,
            timeout: Duration::from_secs(2),
        }
    }

    /// Record a tap and return the calculated tempo
    pub fn tap(&mut self) -> Option<u16> {
        let now = Instant::now();

        // Clear old taps
        self.taps.retain(|t| now.duration_since(*t) < self.timeout);

        // Add new tap
        self.taps.push(now);

        // Keep max number of taps
        while self.taps.len() > self.max_taps {
            self.taps.remove(0);
        }

        // Need at least 2 taps to calculate tempo
        if self.taps.len() < 2 {
            return None;
        }

        // Calculate average interval
        let total_time = self.taps.last().unwrap().duration_since(*self.taps.first().unwrap());
        let intervals = self.taps.len() - 1;
        let avg_interval = total_time.as_secs_f64() / intervals as f64;

        // Convert to BPM
        let bpm = (60.0 / avg_interval).round() as u16;
        let bpm = bpm.clamp(20, 300);

        tracing::debug!(
            "Tap tempo: {} taps, avg interval {:.3}s = {} BPM",
            self.taps.len(),
            avg_interval,
            bpm
        );

        Some(bpm)
    }

    /// Reset the tap tempo
    pub fn reset(&mut self) {
        self.taps.clear();
    }
}

impl Default for TapTempo {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_playback_state() {
        // Skip if no audio device available
        if let Ok(engine) = AudioEngine::new() {
            assert_eq!(engine.state(), PlaybackState::Stopped);

            engine.command(TransportCommand::Play);
            assert_eq!(engine.state(), PlaybackState::Playing);

            engine.command(TransportCommand::Pause);
            assert_eq!(engine.state(), PlaybackState::Paused);

            engine.command(TransportCommand::Stop);
            assert_eq!(engine.state(), PlaybackState::Stopped);
        }
    }

    #[test]
    fn test_tempo() {
        if let Ok(engine) = AudioEngine::new() {
            engine.set_tempo(140);
            assert_eq!(engine.tempo(), 140);

            // Test clamping
            engine.set_tempo(10);
            assert_eq!(engine.tempo(), 20);

            engine.set_tempo(400);
            assert_eq!(engine.tempo(), 300);
        }
    }

    #[test]
    fn test_tap_tempo() {
        let mut tap = TapTempo::new();

        // First tap returns None
        assert!(tap.tap().is_none());

        // Wait and tap again
        std::thread::sleep(Duration::from_millis(500)); // ~120 BPM
        let bpm = tap.tap();

        // Should be around 120 BPM (with some tolerance for timing)
        if let Some(bpm) = bpm {
            assert!(bpm >= 100 && bpm <= 140, "BPM {} not in expected range", bpm);
        }
    }
}
