//! Song representation and serialization
//!
//! Handles the .mozart.json file format

use crate::error::{MozartError, Result};
use crate::note::Note;
use crate::scale::Scale;
use crate::time::TimeSignature;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Song metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SongMetadata {
    /// Song title
    pub title: String,
    /// Composer name
    #[serde(default)]
    pub composer: String,
    /// Creation timestamp (ISO 8601)
    #[serde(default)]
    pub created: String,
    /// Last modified timestamp (ISO 8601)
    #[serde(default)]
    pub modified: String,
}

impl Default for SongMetadata {
    fn default() -> Self {
        let now = chrono_lite_now();
        SongMetadata {
            title: "Untitled".to_string(),
            composer: String::new(),
            created: now.clone(),
            modified: now,
        }
    }
}

/// Song settings (tempo, time signature, key)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SongSettings {
    /// Tempo in BPM
    pub tempo: u16,
    /// Time signature
    pub time_signature: TimeSignature,
    /// Key/scale for the song
    pub key: Scale,
}

impl Default for SongSettings {
    fn default() -> Self {
        SongSettings {
            tempo: 120,
            time_signature: TimeSignature::common(),
            key: Scale::c_major(),
        }
    }
}

/// A complete song with metadata, settings, and notes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Song {
    /// File format version
    pub version: String,
    /// Song metadata
    pub metadata: SongMetadata,
    /// Song settings
    pub settings: SongSettings,
    /// The notes in the melody
    pub notes: Vec<Note>,
}

impl Song {
    /// Create a new empty song
    pub fn new() -> Self {
        tracing::info!("Creating new song");
        Song {
            version: "1.0".to_string(),
            metadata: SongMetadata::default(),
            settings: SongSettings::default(),
            notes: Vec::new(),
        }
    }

    /// Create a song with a title
    pub fn with_title(title: impl Into<String>) -> Self {
        let mut song = Song::new();
        song.metadata.title = title.into();
        song
    }

    /// Set the tempo
    pub fn set_tempo(&mut self, tempo: u16) {
        tracing::debug!("Setting tempo to {} BPM", tempo);
        self.settings.tempo = tempo.clamp(20, 300);
    }

    /// Set the time signature
    pub fn set_time_signature(&mut self, ts: TimeSignature) {
        tracing::debug!("Setting time signature to {}", ts);
        self.settings.time_signature = ts;
    }

    /// Set the key
    pub fn set_key(&mut self, key: Scale) {
        tracing::debug!("Setting key to {}", key);
        self.settings.key = key;
    }

    /// Add a note
    pub fn add_note(&mut self, note: Note) {
        tracing::trace!("Adding note: {}", note);
        self.notes.push(note);
        self.sort_notes();
        self.update_modified();
    }

    /// Add multiple notes
    pub fn add_notes(&mut self, notes: impl IntoIterator<Item = Note>) {
        self.notes.extend(notes);
        self.sort_notes();
        self.update_modified();
    }

    /// Remove a note at index
    pub fn remove_note(&mut self, index: usize) -> Option<Note> {
        if index < self.notes.len() {
            let note = self.notes.remove(index);
            tracing::trace!("Removed note at index {}: {}", index, note);
            self.update_modified();
            Some(note)
        } else {
            None
        }
    }

    /// Clear all notes
    pub fn clear_notes(&mut self) {
        tracing::debug!("Clearing all notes");
        self.notes.clear();
        self.update_modified();
    }

    /// Sort notes by start time
    fn sort_notes(&mut self) {
        self.notes.sort_by_key(|n| n.start_tick);
    }

    /// Update the modified timestamp
    fn update_modified(&mut self) {
        self.metadata.modified = chrono_lite_now();
    }

    /// Get the total duration in ticks
    pub fn duration_ticks(&self) -> u32 {
        self.notes.iter().map(|n| n.end_tick()).max().unwrap_or(0)
    }

    /// Get the duration in seconds
    pub fn duration_seconds(&self) -> f64 {
        let ticks = self.duration_ticks();
        let ticks_per_beat = crate::TICKS_PER_QUARTER;
        let beats = ticks as f64 / ticks_per_beat as f64;
        beats * 60.0 / self.settings.tempo as f64
    }

    /// Get the number of measures
    pub fn measure_count(&self) -> u32 {
        let ticks = self.duration_ticks();
        let ticks_per_measure = self.settings.time_signature.ticks_per_measure();
        (ticks + ticks_per_measure - 1) / ticks_per_measure
    }

    /// Serialize to JSON string
    pub fn to_json(&self) -> Result<String> {
        tracing::debug!("Serializing song to JSON");
        serde_json::to_string_pretty(self).map_err(MozartError::from)
    }

    /// Deserialize from JSON string
    pub fn from_json(json: &str) -> Result<Self> {
        tracing::debug!("Deserializing song from JSON");
        serde_json::from_str(json).map_err(MozartError::from)
    }

    /// Save to file
    pub fn save(&self, path: impl AsRef<Path>) -> Result<()> {
        let path = path.as_ref();
        tracing::info!("Saving song to {:?}", path);

        let json = self.to_json()?;
        std::fs::write(path, json)
            .map_err(|e| MozartError::FileError(format!("Failed to write file: {}", e)))?;

        tracing::info!("Song saved successfully");
        Ok(())
    }

    /// Load from file
    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        tracing::info!("Loading song from {:?}", path);

        let json = std::fs::read_to_string(path)
            .map_err(|e| MozartError::FileError(format!("Failed to read file: {}", e)))?;

        let song = Self::from_json(&json)?;
        tracing::info!(
            "Loaded song: {} ({} notes)",
            song.metadata.title,
            song.notes.len()
        );
        Ok(song)
    }
}

impl Default for Song {
    fn default() -> Self {
        Song::new()
    }
}

/// Simple timestamp generator (no external deps)
fn chrono_lite_now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};

    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();

    // Convert to a simple ISO-ish format
    let secs = duration.as_secs();
    // This is a simplified timestamp - in production you'd want proper date formatting
    format!("{}Z", secs)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::note::{NoteDuration, NoteValue};
    use crate::pitch::{Pitch, PitchClass};
    use pretty_assertions::assert_eq;

    #[test]
    fn test_song_creation() {
        let song = Song::new();
        assert_eq!(song.version, "1.0");
        assert_eq!(song.metadata.title, "Untitled");
        assert_eq!(song.settings.tempo, 120);
        assert!(song.notes.is_empty());
    }

    #[test]
    fn test_song_with_notes() {
        let mut song = Song::with_title("Test Song");

        let c4 = Note::from_pitch(
            Pitch::new(PitchClass::C, 4).unwrap(),
            0,
            NoteDuration::new(NoteValue::Quarter),
        );
        let e4 = Note::from_pitch(
            Pitch::new(PitchClass::E, 4).unwrap(),
            480,
            NoteDuration::new(NoteValue::Quarter),
        );

        song.add_note(c4);
        song.add_note(e4);

        assert_eq!(song.notes.len(), 2);
        assert_eq!(song.duration_ticks(), 960);
    }

    #[test]
    fn test_song_serialization() {
        let mut song = Song::with_title("Serialization Test");
        song.add_note(Note::new(60, 0, 480));
        song.add_note(Note::new(64, 480, 480));

        let json = song.to_json().unwrap();
        let loaded = Song::from_json(&json).unwrap();

        assert_eq!(loaded.metadata.title, song.metadata.title);
        assert_eq!(loaded.notes.len(), 2);
        assert_eq!(loaded.notes[0].pitch, 60);
        assert_eq!(loaded.notes[1].pitch, 64);
    }

    #[test]
    fn test_song_duration() {
        let mut song = Song::new();
        song.add_note(Note::new(60, 0, 480));    // Quarter note
        song.add_note(Note::new(62, 480, 960));  // Half note starting at beat 2

        assert_eq!(song.duration_ticks(), 1440); // 480 + 960

        // Duration in seconds at 120 BPM
        // 1440 ticks = 3 quarter notes = 1.5 seconds at 120 BPM
        let duration = song.duration_seconds();
        assert!((duration - 1.5).abs() < 0.001);
    }

    #[test]
    fn test_measure_count() {
        let mut song = Song::new();
        // 4/4 time, fill exactly 2 measures
        let ticks_per_measure = song.settings.time_signature.ticks_per_measure();
        song.add_note(Note::new(60, 0, ticks_per_measure * 2));

        assert_eq!(song.measure_count(), 2);
    }
}
