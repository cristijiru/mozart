//! Audio error types

use thiserror::Error;

/// Errors that can occur in the audio engine
#[derive(Error, Debug)]
pub enum AudioError {
    #[error("Audio device error: {0}")]
    DeviceError(String),

    #[error("Sample loading error: {0}")]
    SampleError(String),

    #[error("Playback error: {0}")]
    PlaybackError(String),

    #[error("Invalid audio format: {0}")]
    FormatError(String),

    #[error("Audio stream error: {0}")]
    StreamError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, AudioError>;
