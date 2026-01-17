//! Mozart Audio - Audio playback engine
//!
//! Provides audio playback capabilities for Mozart:
//! - Sample-based instrument playback
//! - Metronome with accent support
//! - Transport controls (play, pause, stop, loop)

pub mod engine;
pub mod sampler;
pub mod metronome;
pub mod error;

pub use engine::{AudioEngine, PlaybackState, TransportCommand};
pub use sampler::{Sampler, Instrument};
pub use metronome::Metronome;
pub use error::AudioError;

/// Initialize logging for the audio crate
pub fn init_logging() {
    use tracing_subscriber::{fmt, prelude::*, EnvFilter};

    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("mozart_audio=debug"));

    tracing_subscriber::registry()
        .with(fmt::layer().with_target(true))
        .with(filter)
        .try_init()
        .ok();
}
