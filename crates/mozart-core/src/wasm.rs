//! WebAssembly bindings for Mozart Core
//!
//! This module provides JavaScript-friendly wrappers around the core music engine.

use wasm_bindgen::prelude::*;
use crate::note::{Note, parse_melody, format_melody};
use crate::pitch::{Pitch, PitchClass};
use crate::scale::{Scale, ScaleType};
use crate::time::{TimeSignature, AccentPattern};
use crate::transpose::{TransposeMode, transpose_notes};
use crate::song::Song;
use crate::midi::export_to_midi;

/// Initialize panic hook for better error messages in the browser console
#[wasm_bindgen(start)]
pub fn init() {
    console_error_panic_hook::set_once();
}

/// Main Mozart interface exposed to JavaScript
#[wasm_bindgen]
pub struct Mozart {
    song: Song,
}

#[wasm_bindgen]
impl Mozart {
    /// Create a new Mozart instance with an empty song
    #[wasm_bindgen(constructor)]
    pub fn new() -> Mozart {
        Mozart { song: Song::new() }
    }

    /// Create with a title
    #[wasm_bindgen(js_name = withTitle)]
    pub fn with_title(title: &str) -> Mozart {
        Mozart {
            song: Song::with_title(title),
        }
    }

    // ==================== Song Metadata ====================

    /// Get the song title
    #[wasm_bindgen(getter)]
    pub fn title(&self) -> String {
        self.song.metadata.title.clone()
    }

    /// Set the song title
    #[wasm_bindgen(setter)]
    pub fn set_title(&mut self, title: String) {
        self.song.metadata.title = title;
    }

    /// Get the composer name
    #[wasm_bindgen(getter)]
    pub fn composer(&self) -> String {
        self.song.metadata.composer.clone()
    }

    /// Set the composer name
    #[wasm_bindgen(setter)]
    pub fn set_composer(&mut self, composer: String) {
        self.song.metadata.composer = composer;
    }

    // ==================== Song Settings ====================

    /// Get the tempo in BPM
    #[wasm_bindgen(getter)]
    pub fn tempo(&self) -> u16 {
        self.song.settings.tempo
    }

    /// Set the tempo in BPM (20-300)
    #[wasm_bindgen(setter)]
    pub fn set_tempo(&mut self, tempo: u16) {
        self.song.set_tempo(tempo);
    }

    /// Get time signature as "numerator/denominator"
    #[wasm_bindgen(js_name = getTimeSignature)]
    pub fn get_time_signature(&self) -> String {
        self.song.settings.time_signature.to_string()
    }

    /// Set time signature from string (e.g., "4/4", "7/8")
    #[wasm_bindgen(js_name = setTimeSignature)]
    pub fn set_time_signature(&mut self, ts: &str) -> Result<(), JsValue> {
        let time_sig = TimeSignature::parse(ts)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        self.song.set_time_signature(time_sig);
        Ok(())
    }

    /// Get the time signature numerator
    #[wasm_bindgen(js_name = getTimeSignatureNumerator)]
    pub fn get_time_signature_numerator(&self) -> u8 {
        self.song.settings.time_signature.numerator
    }

    /// Get the time signature denominator
    #[wasm_bindgen(js_name = getTimeSignatureDenominator)]
    pub fn get_time_signature_denominator(&self) -> u8 {
        self.song.settings.time_signature.denominator
    }

    /// Get the key/scale as string (e.g., "C Major")
    #[wasm_bindgen(js_name = getKey)]
    pub fn get_key(&self) -> String {
        self.song.settings.key.to_string()
    }

    /// Set the key/scale from string (e.g., "C major", "F# minor")
    #[wasm_bindgen(js_name = setKey)]
    pub fn set_key(&mut self, key: &str) -> Result<(), JsValue> {
        let scale = Scale::parse(key)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        self.song.set_key(scale);
        Ok(())
    }

    // ==================== Note Management ====================

    /// Get the number of notes
    #[wasm_bindgen(js_name = noteCount)]
    pub fn note_count(&self) -> usize {
        self.song.notes.len()
    }

    /// Add a note by MIDI pitch, start tick, and duration ticks
    #[wasm_bindgen(js_name = addNote)]
    pub fn add_note(&mut self, pitch: u8, start_tick: u32, duration_ticks: u32) {
        self.song.add_note(Note::new(pitch, start_tick, duration_ticks));
    }

    /// Add a note with velocity
    #[wasm_bindgen(js_name = addNoteWithVelocity)]
    pub fn add_note_with_velocity(&mut self, pitch: u8, start_tick: u32, duration_ticks: u32, velocity: u8) {
        self.song.add_note(Note::with_velocity(pitch, start_tick, duration_ticks, velocity));
    }

    /// Remove a note at index
    #[wasm_bindgen(js_name = removeNote)]
    pub fn remove_note(&mut self, index: usize) -> bool {
        self.song.remove_note(index).is_some()
    }

    /// Clear all notes
    #[wasm_bindgen(js_name = clearNotes)]
    pub fn clear_notes(&mut self) {
        self.song.clear_notes();
    }

    /// Get all notes as JSON array
    #[wasm_bindgen(js_name = getNotesJson)]
    pub fn get_notes_json(&self) -> String {
        serde_json::to_string(&self.song.notes).unwrap_or_else(|_| "[]".to_string())
    }

    /// Get a single note as JSON
    #[wasm_bindgen(js_name = getNoteJson)]
    pub fn get_note_json(&self, index: usize) -> Option<String> {
        self.song.notes.get(index).map(|n| serde_json::to_string(n).unwrap_or_default())
    }

    // ==================== Melody Parsing ====================

    /// Parse a melody string and add the notes
    /// Format: "C4q D4q E4q" (pitch + duration, space-separated)
    /// Durations: w=whole, h=half, q=quarter, e=eighth, s=sixteenth
    /// Add . for dotted (e.g., "q." for dotted quarter)
    /// Use "R" for rests (e.g., "Rq" for quarter rest)
    #[wasm_bindgen(js_name = parseMelody)]
    pub fn parse_melody_str(&mut self, melody: &str) -> Result<usize, JsValue> {
        let notes = parse_melody(melody)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        let count = notes.len();

        // Auto-detect key from the melody
        if let Some(detected_scale) = crate::transpose::detect_scale(&notes) {
            self.song.set_key(detected_scale);
        }

        self.song.add_notes(notes);
        Ok(count)
    }

    /// Format all notes as a melody string
    #[wasm_bindgen(js_name = formatMelody)]
    pub fn format_melody_str(&self) -> String {
        format_melody(&self.song.notes)
    }

    // ==================== Transposition ====================

    /// Transpose all notes chromatically by semitones
    /// Positive = up, negative = down
    #[wasm_bindgen(js_name = transposeChromatic)]
    pub fn transpose_chromatic(&mut self, semitones: i8) -> Result<(), JsValue> {
        let mode = TransposeMode::chromatic(semitones);
        let transposed = transpose_notes(&self.song.notes, &mode)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        self.song.notes = transposed;
        Ok(())
    }

    /// Transpose all notes diatonically within the current key
    /// Positive = up, negative = down (in scale degrees)
    #[wasm_bindgen(js_name = transposeDiatonic)]
    pub fn transpose_diatonic(&mut self, degrees: i8) -> Result<(), JsValue> {
        let scale = self.song.settings.key;
        let mode = TransposeMode::diatonic(scale, degrees);
        let transposed = transpose_notes(&self.song.notes, &mode)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        self.song.notes = transposed;
        Ok(())
    }

    /// Transpose diatonically with a key change
    #[wasm_bindgen(js_name = transposeDiatonicWithKeyChange)]
    pub fn transpose_diatonic_with_key_change(
        &mut self,
        target_key: &str,
        degrees: i8,
    ) -> Result<(), JsValue> {
        let source_scale = self.song.settings.key;
        let target_scale = Scale::parse(target_key)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        let mode = TransposeMode::diatonic_with_key_change(source_scale, target_scale, degrees);
        let transposed = transpose_notes(&self.song.notes, &mode)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        self.song.notes = transposed;
        self.song.set_key(target_scale);
        Ok(())
    }

    /// Invert all notes around a pivot pitch (mirror reflection)
    /// new_pitch = 2 * pivot - old_pitch
    #[wasm_bindgen(js_name = invert)]
    pub fn invert(&mut self, pivot: u8) -> Result<(), JsValue> {
        for note in &mut self.song.notes {
            let new_pitch = (2 * pivot as i16) - (note.pitch as i16);
            if new_pitch < 0 || new_pitch > 127 {
                return Err(JsValue::from_str(&format!(
                    "Inversion would put note out of MIDI range (pivot: {}, original: {}, result: {})",
                    pivot, note.pitch, new_pitch
                )));
            }
            note.pitch = new_pitch as u8;
        }
        Ok(())
    }

    // ==================== Accents ====================

    /// Get the accent pattern as an array of levels (1=weak, 2=medium, 3=strong)
    #[wasm_bindgen(js_name = getAccents)]
    pub fn get_accents(&self) -> Vec<u8> {
        self.song.settings.time_signature.accents.accents
            .iter()
            .map(|a| *a as u8)
            .collect()
    }

    /// Set the accent pattern from an array of levels (1=weak, 2=medium, 3=strong)
    #[wasm_bindgen(js_name = setAccents)]
    pub fn set_accents(&mut self, accents: &[u8]) -> Result<(), JsValue> {
        if accents.len() != self.song.settings.time_signature.numerator as usize {
            return Err(JsValue::from_str(&format!(
                "Accent pattern length ({}) must match time signature numerator ({})",
                accents.len(),
                self.song.settings.time_signature.numerator
            )));
        }
        let pattern = AccentPattern::from_values(accents);
        self.song.settings.time_signature.set_accents(pattern);
        Ok(())
    }

    /// Cycle accent at beat index (weak -> medium -> strong -> weak)
    #[wasm_bindgen(js_name = cycleAccent)]
    pub fn cycle_accent(&mut self, beat: usize) {
        self.song.settings.time_signature.accents.cycle(beat);
    }

    /// Get accent pattern as visual string (e.g., ">.-.")
    #[wasm_bindgen(js_name = getAccentVisual)]
    pub fn get_accent_visual(&self) -> String {
        self.song.settings.time_signature.accents.to_visual()
    }

    // ==================== Duration Info ====================

    /// Get total duration in ticks
    #[wasm_bindgen(js_name = durationTicks)]
    pub fn duration_ticks(&self) -> u32 {
        self.song.duration_ticks()
    }

    /// Get total duration in seconds
    #[wasm_bindgen(js_name = durationSeconds)]
    pub fn duration_seconds(&self) -> f64 {
        self.song.duration_seconds()
    }

    /// Get number of measures
    #[wasm_bindgen(js_name = measureCount)]
    pub fn measure_count(&self) -> u32 {
        self.song.measure_count()
    }

    /// Get ticks per beat
    #[wasm_bindgen(js_name = ticksPerBeat)]
    pub fn ticks_per_beat(&self) -> u32 {
        self.song.settings.time_signature.ticks_per_beat()
    }

    /// Get ticks per measure
    #[wasm_bindgen(js_name = ticksPerMeasure)]
    pub fn ticks_per_measure(&self) -> u32 {
        self.song.settings.time_signature.ticks_per_measure()
    }

    // ==================== Serialization ====================

    /// Export the song to JSON
    #[wasm_bindgen(js_name = toJson)]
    pub fn to_json(&self) -> Result<String, JsValue> {
        self.song.to_json()
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Load a song from JSON
    #[wasm_bindgen(js_name = fromJson)]
    pub fn from_json(json: &str) -> Result<Mozart, JsValue> {
        let song = Song::from_json(json)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        Ok(Mozart { song })
    }

    /// Export to MIDI bytes
    #[wasm_bindgen(js_name = toMidi)]
    pub fn to_midi(&self) -> Result<Vec<u8>, JsValue> {
        export_to_midi(&self.song)
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    // ==================== Pitch Utilities ====================

    /// Get the frequency in Hz for a MIDI note number
    #[wasm_bindgen(js_name = midiToFrequency)]
    pub fn midi_to_frequency(midi: u8) -> f64 {
        Pitch::from_midi(midi)
            .map(|p| p.frequency())
            .unwrap_or(0.0)
    }

    /// Get the note name for a MIDI note number (e.g., "C4", "F#5")
    #[wasm_bindgen(js_name = midiToNoteName)]
    pub fn midi_to_note_name(midi: u8) -> String {
        Pitch::from_midi(midi)
            .map(|p| p.to_string())
            .unwrap_or_else(|_| "?".to_string())
    }

    /// Parse a note name to MIDI number
    #[wasm_bindgen(js_name = noteNameToMidi)]
    pub fn note_name_to_midi(name: &str) -> Result<u8, JsValue> {
        Pitch::parse(name)
            .map(|p| p.midi())
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }
}

// ==================== Constants ====================

/// Get ticks per quarter note (480)
#[wasm_bindgen(js_name = TICKS_PER_QUARTER)]
pub fn ticks_per_quarter() -> u32 {
    crate::TICKS_PER_QUARTER
}

/// Get all scale types as a JSON array
#[wasm_bindgen(js_name = getScaleTypes)]
pub fn get_scale_types() -> String {
    let types: Vec<&str> = ScaleType::all().iter().map(|t| t.name()).collect();
    serde_json::to_string(&types).unwrap_or_else(|_| "[]".to_string())
}

/// Get all pitch class names
#[wasm_bindgen(js_name = getPitchClasses)]
pub fn get_pitch_classes() -> String {
    let names: Vec<&str> = PitchClass::all().iter().map(|pc| pc.natural_name()).collect();
    serde_json::to_string(&names).unwrap_or_else(|_| "[]".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mozart_wasm_basic() {
        let mut mozart = Mozart::new();
        mozart.set_title("Test".to_string());
        assert_eq!(mozart.title(), "Test");

        mozart.add_note(60, 0, 480);
        assert_eq!(mozart.note_count(), 1);
    }

    #[test]
    fn test_melody_parsing() {
        let mut mozart = Mozart::new();
        let count = mozart.parse_melody_str("C4q D4q E4q").unwrap();
        assert_eq!(count, 3);
        assert_eq!(mozart.note_count(), 3);
    }

    #[test]
    fn test_transpose() {
        let mut mozart = Mozart::new();
        mozart.add_note(60, 0, 480); // C4
        mozart.transpose_chromatic(2).unwrap();

        let json = mozart.get_notes_json();
        assert!(json.contains("62")); // D4
    }
}
