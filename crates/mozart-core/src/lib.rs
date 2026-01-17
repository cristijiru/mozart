//! Mozart Core - Music theory engine
//!
//! This crate provides the core music theory primitives for the Mozart app:
//! - Note representation (pitch, duration, velocity)
//! - Scale definitions (major, minor, modes)
//! - Transposition (chromatic and diatonic)
//! - Time signatures with customizable accents
//! - File format serialization
//! - MIDI export

pub mod note;
pub mod pitch;
pub mod scale;
pub mod time;
pub mod transpose;
pub mod song;
pub mod midi;
pub mod error;

pub use note::{Note, NoteDuration, NoteValue};
pub use pitch::{PitchClass, Pitch};
pub use scale::{Scale, ScaleType};
pub use time::{TimeSignature, AccentLevel, AccentPattern};
pub use transpose::{TransposeMode, transpose_notes};
pub use song::{Song, SongMetadata, SongSettings};
pub use error::MozartError;

/// Ticks per quarter note (standard MIDI resolution)
pub const TICKS_PER_QUARTER: u32 = 480;

/// Initialize logging for the mozart-core crate
pub fn init_logging() {
    use tracing_subscriber::{fmt, prelude::*, EnvFilter};

    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("mozart_core=debug"));

    tracing_subscriber::registry()
        .with(fmt::layer().with_target(true))
        .with(filter)
        .try_init()
        .ok();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_library_loads() {
        init_logging();
        tracing::info!("Mozart core library loaded successfully");
    }
}
