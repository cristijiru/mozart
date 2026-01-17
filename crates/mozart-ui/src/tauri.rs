//! Tauri command bindings
//!
//! Provides async functions to call Tauri backend commands

use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;

// JS interop for Tauri
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;
}

/// Call a Tauri command with arguments
pub async fn call<T, R>(cmd: &str, args: T) -> Result<R, String>
where
    T: Serialize,
    R: for<'de> Deserialize<'de>,
{
    let args_js = serde_wasm_bindgen::to_value(&args).map_err(|e| e.to_string())?;

    let result = invoke(cmd, args_js).await;

    if result.is_undefined() || result.is_null() {
        // Commands that return () will be undefined
        return serde_wasm_bindgen::from_value(JsValue::NULL).map_err(|e| e.to_string());
    }

    serde_wasm_bindgen::from_value(result).map_err(|e| e.to_string())
}

/// Call a Tauri command with no arguments
pub async fn call_no_args<R>(cmd: &str) -> Result<R, String>
where
    R: for<'de> Deserialize<'de>,
{
    call(cmd, serde_json::json!({})).await
}

// ============================================================================
// Data Types (matching Tauri backend)
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SongInfo {
    pub title: String,
    pub tempo: u16,
    pub time_signature: String,
    pub key: String,
    pub note_count: usize,
    pub duration_seconds: f64,
    pub measure_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteData {
    pub pitch: u8,
    pub start_tick: u32,
    pub duration_ticks: u32,
    pub velocity: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccentInfo {
    pub numerator: u8,
    pub denominator: u8,
    pub accents: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum TransposeRequest {
    Chromatic { semitones: i8 },
    Diatonic { degrees: i8 },
    DiatonicWithKeyChange {
        degrees: i8,
        target_root: String,
        target_scale_type: String,
    },
}

// ============================================================================
// API Functions
// ============================================================================

// Song commands
pub async fn get_song_info() -> Result<SongInfo, String> {
    call_no_args("get_song_info").await
}

pub async fn new_song() -> Result<(), String> {
    call_no_args::<()>("new_song").await
}

pub async fn set_title(title: &str) -> Result<(), String> {
    call("set_title", serde_json::json!({ "title": title })).await
}

pub async fn set_tempo(tempo: u16) -> Result<(), String> {
    call("set_tempo", serde_json::json!({ "tempo": tempo })).await
}

pub async fn set_time_signature(numerator: u8, denominator: u8) -> Result<(), String> {
    call(
        "set_time_signature",
        serde_json::json!({ "numerator": numerator, "denominator": denominator }),
    )
    .await
}

pub async fn set_key(root: &str, scale_type: &str) -> Result<(), String> {
    call(
        "set_key",
        serde_json::json!({ "root": root, "scaleType": scale_type }),
    )
    .await
}

pub async fn get_accents() -> Result<AccentInfo, String> {
    call_no_args("get_accents").await
}

pub async fn set_accents(accents: Vec<u8>) -> Result<(), String> {
    call("set_accents", serde_json::json!({ "accents": accents })).await
}

pub async fn cycle_accent(beat: usize) -> Result<(), String> {
    call("cycle_accent", serde_json::json!({ "beat": beat })).await
}

// Note commands
pub async fn get_notes() -> Result<Vec<NoteData>, String> {
    call_no_args("get_notes").await
}

pub async fn add_note(note: NoteData) -> Result<(), String> {
    call("add_note", serde_json::json!({ "note": note })).await
}

pub async fn remove_note(index: usize) -> Result<Option<NoteData>, String> {
    call("remove_note", serde_json::json!({ "index": index })).await
}

pub async fn clear_notes() -> Result<(), String> {
    call_no_args::<()>("clear_notes").await
}

pub async fn parse_text_melody(text: &str) -> Result<(), String> {
    call("parse_text_melody", serde_json::json!({ "text": text })).await
}

pub async fn get_melody_text() -> Result<String, String> {
    call_no_args("get_melody_text").await
}

// Transposition
pub async fn transpose(request: TransposeRequest) -> Result<(), String> {
    call("transpose", serde_json::json!({ "request": request })).await
}

pub async fn get_transposition_description(request: TransposeRequest) -> Result<String, String> {
    call(
        "get_transposition_description",
        serde_json::json!({ "request": request }),
    )
    .await
}

// File commands
pub async fn save_song(path: &str) -> Result<(), String> {
    call("save_song", serde_json::json!({ "path": path })).await
}

pub async fn load_song(path: &str) -> Result<SongInfo, String> {
    call("load_song", serde_json::json!({ "path": path })).await
}

pub async fn export_midi(path: &str) -> Result<(), String> {
    call("export_midi", serde_json::json!({ "path": path })).await
}

pub async fn get_song_json() -> Result<String, String> {
    call_no_args("get_song_json").await
}

pub async fn load_song_json(json: &str) -> Result<SongInfo, String> {
    call("load_song_json", serde_json::json!({ "json": json })).await
}

// Audio commands
pub async fn play() -> Result<(), String> {
    call_no_args::<()>("play").await
}

pub async fn pause() -> Result<(), String> {
    call_no_args::<()>("pause").await
}

pub async fn stop() -> Result<(), String> {
    call_no_args::<()>("stop").await
}

pub async fn get_playback_state() -> Result<String, String> {
    call_no_args("get_playback_state").await
}

pub async fn get_playback_position() -> Result<u32, String> {
    call_no_args("get_playback_position").await
}

pub async fn set_playback_position(tick: u32) -> Result<(), String> {
    call(
        "set_playback_position",
        serde_json::json!({ "tick": tick }),
    )
    .await
}

pub async fn toggle_metronome() -> Result<(), String> {
    call_no_args::<()>("toggle_metronome").await
}

pub async fn get_metronome_enabled() -> Result<bool, String> {
    call_no_args("get_metronome_enabled").await
}

pub async fn set_loop(start: Option<u32>, end: Option<u32>) -> Result<(), String> {
    call("set_loop", serde_json::json!({ "start": start, "end": end })).await
}

pub async fn play_note_preview(pitch: u8, velocity: u8, duration_ms: u64) -> Result<(), String> {
    call(
        "play_note_preview",
        serde_json::json!({
            "pitch": pitch,
            "velocity": velocity,
            "durationMs": duration_ms
        }),
    )
    .await
}

pub async fn set_instrument(instrument: &str) -> Result<(), String> {
    call(
        "set_instrument",
        serde_json::json!({ "instrument": instrument }),
    )
    .await
}

// Undo/Redo
pub async fn undo() -> Result<bool, String> {
    call_no_args("undo").await
}

pub async fn redo() -> Result<bool, String> {
    call_no_args("redo").await
}

pub async fn can_undo() -> Result<bool, String> {
    call_no_args("can_undo").await
}

pub async fn can_redo() -> Result<bool, String> {
    call_no_args("can_redo").await
}

// Theory
pub async fn get_scale_types() -> Result<Vec<String>, String> {
    call_no_args("get_scale_types").await
}

pub async fn get_pitch_classes() -> Result<Vec<String>, String> {
    call_no_args("get_pitch_classes").await
}

pub async fn get_scale_notes(root: &str, scale_type: &str) -> Result<Vec<String>, String> {
    call(
        "get_scale_notes",
        serde_json::json!({ "root": root, "scaleType": scale_type }),
    )
    .await
}
