//! Error types for Mozart core

use thiserror::Error;

/// Errors that can occur in the Mozart music engine
#[derive(Error, Debug)]
pub enum MozartError {
    #[error("Invalid pitch: {0}")]
    InvalidPitch(String),

    #[error("Invalid note duration: {0}")]
    InvalidDuration(String),

    #[error("Invalid time signature: {numerator}/{denominator}")]
    InvalidTimeSignature { numerator: u8, denominator: u8 },

    #[error("Invalid scale: {0}")]
    InvalidScale(String),

    #[error("Transposition error: {0}")]
    TranspositionError(String),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("File error: {0}")]
    FileError(String),

    #[error("MIDI export error: {0}")]
    MidiError(String),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, MozartError>;
