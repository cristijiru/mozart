//! Mozart Tauri Application
//!
//! Main entry point for the Tauri app with commands for the frontend

use mozart_audio::{AudioEngine, Instrument, PlaybackState, TransportCommand};
use mozart_core::{
    midi::export_to_midi,
    note::{format_melody, parse_melody, Note},
    pitch::PitchClass,
    scale::{Scale, ScaleType},
    song::Song,
    time::{AccentLevel, AccentPattern, TimeSignature},
    transpose::{transpose_notes, TransposeMode},
};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tauri::State;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

/// Application state managed by Tauri
pub struct AppState {
    /// Current song being edited
    song: RwLock<Song>,
    /// Audio engine for playback
    audio: RwLock<Option<AudioEngine>>,
    /// Current file path (if saved)
    file_path: RwLock<Option<PathBuf>>,
    /// Undo history
    undo_stack: RwLock<Vec<Song>>,
    /// Redo history
    redo_stack: RwLock<Vec<Song>>,
}

impl Default for AppState {
    fn default() -> Self {
        AppState {
            song: RwLock::new(Song::new()),
            audio: RwLock::new(AudioEngine::new().ok()),
            file_path: RwLock::new(None),
            undo_stack: RwLock::new(Vec::new()),
            redo_stack: RwLock::new(Vec::new()),
        }
    }
}

impl AppState {
    fn push_undo(&self) {
        let song = self.song.read().clone();
        self.undo_stack.write().push(song);
        self.redo_stack.write().clear();

        // Limit undo history
        let mut stack = self.undo_stack.write();
        while stack.len() > 50 {
            stack.remove(0);
        }
    }
}

// ============================================================================
// Song Commands
// ============================================================================

#[derive(Serialize)]
struct SongInfo {
    title: String,
    tempo: u16,
    time_signature: String,
    key: String,
    note_count: usize,
    duration_seconds: f64,
    measure_count: u32,
}

#[tauri::command]
fn get_song_info(state: State<Arc<AppState>>) -> SongInfo {
    let song = state.song.read();
    SongInfo {
        title: song.metadata.title.clone(),
        tempo: song.settings.tempo,
        time_signature: song.settings.time_signature.to_string(),
        key: song.settings.key.to_string(),
        note_count: song.notes.len(),
        duration_seconds: song.duration_seconds(),
        measure_count: song.measure_count(),
    }
}

#[tauri::command]
fn new_song(state: State<Arc<AppState>>) {
    tracing::info!("Creating new song");
    state.push_undo();
    *state.song.write() = Song::new();
    *state.file_path.write() = None;
}

#[tauri::command]
fn set_title(state: State<Arc<AppState>>, title: String) {
    tracing::debug!("Setting title: {}", title);
    state.song.write().metadata.title = title;
}

#[tauri::command]
fn set_tempo(state: State<Arc<AppState>>, tempo: u16) {
    let tempo = tempo.clamp(20, 300);
    tracing::debug!("Setting tempo: {} BPM", tempo);
    state.song.write().set_tempo(tempo);

    if let Some(audio) = state.audio.read().as_ref() {
        audio.set_tempo(tempo);
    }
}

#[tauri::command]
fn set_time_signature(state: State<Arc<AppState>>, numerator: u8, denominator: u8) -> Result<(), String> {
    tracing::debug!("Setting time signature: {}/{}", numerator, denominator);

    let ts = TimeSignature::new(numerator, denominator).map_err(|e| e.to_string())?;

    if let Some(audio) = state.audio.read().as_ref() {
        audio.set_time_signature(ts.clone());
    }

    state.song.write().set_time_signature(ts);
    Ok(())
}

#[tauri::command]
fn set_key(state: State<Arc<AppState>>, root: String, scale_type: String) -> Result<(), String> {
    tracing::debug!("Setting key: {} {}", root, scale_type);

    let root = PitchClass::parse(&root).map_err(|e| e.to_string())?;
    let scale_type = ScaleType::parse(&scale_type).map_err(|e| e.to_string())?;
    let scale = Scale::new(root, scale_type);

    state.song.write().set_key(scale);
    Ok(())
}

#[derive(Serialize)]
struct AccentInfo {
    numerator: u8,
    denominator: u8,
    accents: Vec<u8>,
}

#[tauri::command]
fn get_accents(state: State<Arc<AppState>>) -> AccentInfo {
    let song = state.song.read();
    let ts = &song.settings.time_signature;
    AccentInfo {
        numerator: ts.numerator,
        denominator: ts.denominator,
        accents: ts.accents.accents.iter().map(|a| *a as u8).collect(),
    }
}

#[tauri::command]
fn set_accents(state: State<Arc<AppState>>, accents: Vec<u8>) -> Result<(), String> {
    tracing::debug!("Setting accents: {:?}", accents);

    let mut song = state.song.write();
    let numerator = song.settings.time_signature.numerator;

    if accents.len() != numerator as usize {
        return Err(format!(
            "Accent pattern length {} doesn't match time signature {}",
            accents.len(),
            numerator
        ));
    }

    let pattern = AccentPattern::from_values(&accents);
    song.settings.time_signature.set_accents(pattern);

    if let Some(audio) = state.audio.read().as_ref() {
        audio.set_time_signature(song.settings.time_signature.clone());
    }

    Ok(())
}

#[tauri::command]
fn cycle_accent(state: State<Arc<AppState>>, beat: usize) {
    tracing::debug!("Cycling accent at beat {}", beat);
    state.song.write().settings.time_signature.accents.cycle(beat);
}

// ============================================================================
// Note Commands
// ============================================================================

#[derive(Serialize, Deserialize)]
struct NoteData {
    pitch: u8,
    start_tick: u32,
    duration_ticks: u32,
    velocity: u8,
}

impl From<&Note> for NoteData {
    fn from(note: &Note) -> Self {
        NoteData {
            pitch: note.pitch,
            start_tick: note.start_tick,
            duration_ticks: note.duration_ticks,
            velocity: note.velocity,
        }
    }
}

impl From<NoteData> for Note {
    fn from(data: NoteData) -> Self {
        Note::with_velocity(data.pitch, data.start_tick, data.duration_ticks, data.velocity)
    }
}

#[tauri::command]
fn get_notes(state: State<Arc<AppState>>) -> Vec<NoteData> {
    state.song.read().notes.iter().map(NoteData::from).collect()
}

#[tauri::command]
fn add_note(state: State<Arc<AppState>>, note: NoteData) {
    tracing::debug!("Adding note: pitch={}, start={}", note.pitch, note.start_tick);
    state.push_undo();
    state.song.write().add_note(note.into());
}

#[tauri::command]
fn remove_note(state: State<Arc<AppState>>, index: usize) -> Option<NoteData> {
    tracing::debug!("Removing note at index {}", index);
    state.push_undo();
    state.song.write().remove_note(index).map(|n| NoteData::from(&n))
}

#[tauri::command]
fn clear_notes(state: State<Arc<AppState>>) {
    tracing::debug!("Clearing all notes");
    state.push_undo();
    state.song.write().clear_notes();
}

#[tauri::command]
fn parse_text_melody(state: State<Arc<AppState>>, text: String) -> Result<(), String> {
    tracing::info!("Parsing melody from text: {}", text);
    state.push_undo();

    let notes = parse_melody(&text).map_err(|e| e.to_string())?;
    let mut song = state.song.write();
    song.clear_notes();
    song.add_notes(notes);

    Ok(())
}

#[tauri::command]
fn get_melody_text(state: State<Arc<AppState>>) -> String {
    format_melody(&state.song.read().notes)
}

// ============================================================================
// Transposition Commands
// ============================================================================

#[derive(Deserialize)]
#[serde(tag = "type")]
enum TransposeRequest {
    Chromatic { semitones: i8 },
    Diatonic { degrees: i8 },
    DiatonicWithKeyChange {
        degrees: i8,
        target_root: String,
        target_scale_type: String,
    },
}

#[tauri::command]
fn transpose(state: State<Arc<AppState>>, request: TransposeRequest) -> Result<(), String> {
    tracing::info!("Transposing: {:?}", serde_json::to_string(&request).unwrap_or_default());
    state.push_undo();

    let mut song = state.song.write();
    let current_key = song.settings.key;

    let mode = match request {
        TransposeRequest::Chromatic { semitones } => TransposeMode::chromatic(semitones),
        TransposeRequest::Diatonic { degrees } => TransposeMode::diatonic(current_key, degrees),
        TransposeRequest::DiatonicWithKeyChange {
            degrees,
            target_root,
            target_scale_type,
        } => {
            let root = PitchClass::parse(&target_root).map_err(|e| e.to_string())?;
            let scale_type = ScaleType::parse(&target_scale_type).map_err(|e| e.to_string())?;
            let target = Scale::new(root, scale_type);
            TransposeMode::diatonic_with_key_change(current_key, target, degrees)
        }
    };

    let transposed = transpose_notes(&song.notes, &mode).map_err(|e| e.to_string())?;
    song.notes = transposed;

    // Update key if doing a key change
    if let TransposeMode::Diatonic { target_scale, .. } = &mode {
        song.settings.key = *target_scale;
    }

    Ok(())
}

#[tauri::command]
fn get_transposition_description(request: TransposeRequest) -> Result<String, String> {
    let mode = match request {
        TransposeRequest::Chromatic { semitones } => TransposeMode::chromatic(semitones),
        TransposeRequest::Diatonic { degrees } => TransposeMode::diatonic(Scale::c_major(), degrees),
        TransposeRequest::DiatonicWithKeyChange {
            degrees,
            target_root,
            target_scale_type,
        } => {
            let root = PitchClass::parse(&target_root).map_err(|e| e.to_string())?;
            let scale_type = ScaleType::parse(&target_scale_type).map_err(|e| e.to_string())?;
            let target = Scale::new(root, scale_type);
            TransposeMode::diatonic_with_key_change(Scale::c_major(), target, degrees)
        }
    };

    Ok(mode.description())
}

// ============================================================================
// File Commands
// ============================================================================

#[tauri::command]
fn save_song(state: State<Arc<AppState>>, path: String) -> Result<(), String> {
    tracing::info!("Saving song to: {}", path);

    let path = PathBuf::from(&path);
    state.song.read().save(&path).map_err(|e| e.to_string())?;
    *state.file_path.write() = Some(path);

    Ok(())
}

#[tauri::command]
fn load_song(state: State<Arc<AppState>>, path: String) -> Result<SongInfo, String> {
    tracing::info!("Loading song from: {}", path);

    let path = PathBuf::from(&path);
    let song = Song::load(&path).map_err(|e| e.to_string())?;

    // Update audio engine
    if let Some(audio) = state.audio.read().as_ref() {
        audio.set_tempo(song.settings.tempo);
        audio.set_time_signature(song.settings.time_signature.clone());
    }

    let info = SongInfo {
        title: song.metadata.title.clone(),
        tempo: song.settings.tempo,
        time_signature: song.settings.time_signature.to_string(),
        key: song.settings.key.to_string(),
        note_count: song.notes.len(),
        duration_seconds: song.duration_seconds(),
        measure_count: song.measure_count(),
    };

    state.push_undo();
    *state.song.write() = song;
    *state.file_path.write() = Some(path);

    Ok(info)
}

#[tauri::command]
fn export_midi(state: State<Arc<AppState>>, path: String) -> Result<(), String> {
    tracing::info!("Exporting MIDI to: {}", path);

    let song = state.song.read();
    let midi_data = export_to_midi(&song).map_err(|e| e.to_string())?;

    std::fs::write(&path, midi_data).map_err(|e| format!("Failed to write MIDI file: {}", e))?;

    Ok(())
}

#[tauri::command]
fn get_song_json(state: State<Arc<AppState>>) -> Result<String, String> {
    state.song.read().to_json().map_err(|e| e.to_string())
}

#[tauri::command]
fn load_song_json(state: State<Arc<AppState>>, json: String) -> Result<SongInfo, String> {
    tracing::debug!("Loading song from JSON");

    let song = Song::from_json(&json).map_err(|e| e.to_string())?;

    let info = SongInfo {
        title: song.metadata.title.clone(),
        tempo: song.settings.tempo,
        time_signature: song.settings.time_signature.to_string(),
        key: song.settings.key.to_string(),
        note_count: song.notes.len(),
        duration_seconds: song.duration_seconds(),
        measure_count: song.measure_count(),
    };

    state.push_undo();
    *state.song.write() = song;

    Ok(info)
}

// ============================================================================
// Audio Commands
// ============================================================================

#[tauri::command]
fn play(state: State<Arc<AppState>>) {
    tracing::debug!("Play");
    if let Some(audio) = state.audio.read().as_ref() {
        audio.command(TransportCommand::Play);
    }
}

#[tauri::command]
fn pause(state: State<Arc<AppState>>) {
    tracing::debug!("Pause");
    if let Some(audio) = state.audio.read().as_ref() {
        audio.command(TransportCommand::Pause);
    }
}

#[tauri::command]
fn stop(state: State<Arc<AppState>>) {
    tracing::debug!("Stop");
    if let Some(audio) = state.audio.read().as_ref() {
        audio.command(TransportCommand::Stop);
    }
}

#[tauri::command]
fn get_playback_state(state: State<Arc<AppState>>) -> String {
    if let Some(audio) = state.audio.read().as_ref() {
        match audio.state() {
            PlaybackState::Stopped => "stopped",
            PlaybackState::Playing => "playing",
            PlaybackState::Paused => "paused",
        }
        .to_string()
    } else {
        "stopped".to_string()
    }
}

#[tauri::command]
fn get_playback_position(state: State<Arc<AppState>>) -> u32 {
    state
        .audio
        .read()
        .as_ref()
        .map(|a| a.position())
        .unwrap_or(0)
}

#[tauri::command]
fn set_playback_position(state: State<Arc<AppState>>, tick: u32) {
    if let Some(audio) = state.audio.read().as_ref() {
        audio.command(TransportCommand::SetPosition(tick));
    }
}

#[tauri::command]
fn toggle_metronome(state: State<Arc<AppState>>) {
    if let Some(audio) = state.audio.read().as_ref() {
        audio.command(TransportCommand::ToggleMetronome);
    }
}

#[tauri::command]
fn get_metronome_enabled(state: State<Arc<AppState>>) -> bool {
    state
        .audio
        .read()
        .as_ref()
        .map(|a| a.metronome_enabled())
        .unwrap_or(false)
}

#[tauri::command]
fn set_loop(state: State<Arc<AppState>>, start: Option<u32>, end: Option<u32>) {
    if let Some(audio) = state.audio.read().as_ref() {
        let range = start.zip(end);
        audio.command(TransportCommand::SetLoop(range));
    }
}

#[tauri::command]
fn play_note_preview(state: State<Arc<AppState>>, pitch: u8, velocity: u8, duration_ms: u64) {
    if let Some(audio) = state.audio.read().as_ref() {
        audio.play_note(pitch, velocity, duration_ms);
    }
}

#[tauri::command]
fn set_instrument(state: State<Arc<AppState>>, instrument: String) {
    let instrument = match instrument.to_lowercase().as_str() {
        "piano" => Instrument::Piano,
        "strings" => Instrument::Strings,
        "synth" => Instrument::Synth,
        _ => Instrument::Piano,
    };

    if let Some(audio) = state.audio.read().as_ref() {
        audio.set_instrument(instrument);
    }
}

// ============================================================================
// Undo/Redo Commands
// ============================================================================

#[tauri::command]
fn undo(state: State<Arc<AppState>>) -> bool {
    tracing::debug!("Undo");
    let mut undo_stack = state.undo_stack.write();
    if let Some(prev_song) = undo_stack.pop() {
        let current = state.song.read().clone();
        state.redo_stack.write().push(current);
        *state.song.write() = prev_song;
        true
    } else {
        false
    }
}

#[tauri::command]
fn redo(state: State<Arc<AppState>>) -> bool {
    tracing::debug!("Redo");
    let mut redo_stack = state.redo_stack.write();
    if let Some(next_song) = redo_stack.pop() {
        let current = state.song.read().clone();
        state.undo_stack.write().push(current);
        *state.song.write() = next_song;
        true
    } else {
        false
    }
}

#[tauri::command]
fn can_undo(state: State<Arc<AppState>>) -> bool {
    !state.undo_stack.read().is_empty()
}

#[tauri::command]
fn can_redo(state: State<Arc<AppState>>) -> bool {
    !state.redo_stack.read().is_empty()
}

// ============================================================================
// Scale/Theory Commands
// ============================================================================

#[tauri::command]
fn get_scale_types() -> Vec<String> {
    ScaleType::all()
        .iter()
        .map(|s| s.name().to_string())
        .collect()
}

#[tauri::command]
fn get_pitch_classes() -> Vec<String> {
    PitchClass::all()
        .iter()
        .map(|p| p.natural_name().to_string())
        .collect()
}

#[tauri::command]
fn get_scale_notes(root: String, scale_type: String) -> Result<Vec<String>, String> {
    let root = PitchClass::parse(&root).map_err(|e| e.to_string())?;
    let scale_type = ScaleType::parse(&scale_type).map_err(|e| e.to_string())?;
    let scale = Scale::new(root, scale_type);

    Ok(scale
        .pitch_classes()
        .iter()
        .map(|p| p.natural_name().to_string())
        .collect())
}

// ============================================================================
// App Entry Point
// ============================================================================

fn init_logging() {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("mozart=debug,mozart_core=debug,mozart_audio=info"));

    tracing_subscriber::registry()
        .with(fmt::layer().with_target(true).with_file(true).with_line_number(true))
        .with(filter)
        .try_init()
        .ok();
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    init_logging();
    tracing::info!("Starting Mozart application");

    let app_state = Arc::new(AppState::default());

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            // Song
            get_song_info,
            new_song,
            set_title,
            set_tempo,
            set_time_signature,
            set_key,
            get_accents,
            set_accents,
            cycle_accent,
            // Notes
            get_notes,
            add_note,
            remove_note,
            clear_notes,
            parse_text_melody,
            get_melody_text,
            // Transposition
            transpose,
            get_transposition_description,
            // Files
            save_song,
            load_song,
            export_midi,
            get_song_json,
            load_song_json,
            // Audio
            play,
            pause,
            stop,
            get_playback_state,
            get_playback_position,
            set_playback_position,
            toggle_metronome,
            get_metronome_enabled,
            set_loop,
            play_note_preview,
            set_instrument,
            // Undo/Redo
            undo,
            redo,
            can_undo,
            can_redo,
            // Theory
            get_scale_types,
            get_pitch_classes,
            get_scale_notes,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
