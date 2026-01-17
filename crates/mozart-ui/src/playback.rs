//! Playback sequencer using Web Audio API
//!
//! Handles playing notes in sequence with proper timing

use crate::audio;
use crate::tauri::NoteData;
use std::sync::{Arc, Mutex};
use wasm_bindgen::prelude::*;

/// Ticks per quarter note (matching backend)
const TICKS_PER_QUARTER: u32 = 480;

/// Playback state shared across the app
#[derive(Clone)]
pub struct Playback {
    inner: Arc<Mutex<PlaybackInner>>,
}

struct PlaybackInner {
    /// Current playback position in ticks
    position_ticks: u32,
    /// Whether currently playing
    is_playing: bool,
    /// Tempo in BPM
    tempo: u16,
    /// Notes to play
    notes: Vec<NoteData>,
    /// Index of next note to play
    next_note_idx: usize,
    /// Timer handle for the playback loop
    timer_handle: Option<i32>,
}

impl Playback {
    pub fn new() -> Self {
        Playback {
            inner: Arc::new(Mutex::new(PlaybackInner {
                position_ticks: 0,
                is_playing: false,
                tempo: 120,
                notes: Vec::new(),
                next_note_idx: 0,
                timer_handle: None,
            })),
        }
    }

    /// Set the notes to play
    pub fn set_notes(&self, mut notes: Vec<NoteData>) {
        // Sort notes by start time
        notes.sort_by_key(|n| n.start_tick);

        let mut inner = self.inner.lock().unwrap();
        inner.notes = notes;
        inner.next_note_idx = 0;
    }

    /// Set the tempo
    pub fn set_tempo(&self, tempo: u16) {
        self.inner.lock().unwrap().tempo = tempo;
    }

    /// Start or resume playback
    pub fn play(&self, on_state_change: impl Fn(bool) + 'static + Send + Sync) {
        {
            let mut inner = self.inner.lock().unwrap();

            if inner.is_playing {
                return;
            }

            inner.is_playing = true;

            // Find the first note that should play from current position
            inner.next_note_idx = inner.notes.iter()
                .position(|n| n.start_tick >= inner.position_ticks)
                .unwrap_or(inner.notes.len());
        }

        on_state_change(true);

        // Start the playback timer
        self.start_timer(on_state_change);
    }

    /// Pause playback
    pub fn pause(&self, on_state_change: impl Fn(bool) + 'static) {
        let mut inner = self.inner.lock().unwrap();
        inner.is_playing = false;

        if let Some(handle) = inner.timer_handle.take() {
            let window = web_sys::window().unwrap();
            window.clear_interval_with_handle(handle);
        }

        drop(inner);
        on_state_change(false);
    }

    /// Stop playback and reset position
    pub fn stop(&self, on_state_change: impl Fn(bool) + 'static) {
        let mut inner = self.inner.lock().unwrap();
        inner.is_playing = false;
        inner.position_ticks = 0;
        inner.next_note_idx = 0;

        if let Some(handle) = inner.timer_handle.take() {
            let window = web_sys::window().unwrap();
            window.clear_interval_with_handle(handle);
        }

        drop(inner);
        on_state_change(false);
    }

    /// Check if currently playing
    pub fn is_playing(&self) -> bool {
        self.inner.lock().unwrap().is_playing
    }

    /// Get current position in ticks
    pub fn position(&self) -> u32 {
        self.inner.lock().unwrap().position_ticks
    }

    fn start_timer(&self, on_state_change: impl Fn(bool) + 'static + Send + Sync) {
        let inner = self.inner.clone();
        let on_state_change = Arc::new(on_state_change);

        // Timer interval in milliseconds (update ~60 times per second)
        let interval_ms = 16;

        let callback = Closure::wrap(Box::new(move || {
            let mut state = inner.lock().unwrap();

            if !state.is_playing {
                return;
            }

            // Calculate how many ticks pass per interval
            // ticks_per_second = tempo * TICKS_PER_QUARTER / 60
            // ticks_per_interval = ticks_per_second * interval_ms / 1000
            let ticks_per_second = state.tempo as f64 * TICKS_PER_QUARTER as f64 / 60.0;
            let ticks_per_interval = ticks_per_second * interval_ms as f64 / 1000.0;

            state.position_ticks += ticks_per_interval as u32;

            // Play any notes that should start
            while state.next_note_idx < state.notes.len() {
                let note = &state.notes[state.next_note_idx];
                if note.start_tick <= state.position_ticks {
                    // Calculate note duration in milliseconds
                    let duration_ms = (note.duration_ticks as f64 / ticks_per_second * 1000.0) as u32;
                    audio::play_sine_note(note.pitch, note.velocity, duration_ms);
                    state.next_note_idx += 1;
                } else {
                    break;
                }
            }

            // Check if we've reached the end
            if state.next_note_idx >= state.notes.len() {
                // Find the end of the last note
                let last_end = state.notes.last()
                    .map(|n| n.start_tick + n.duration_ticks)
                    .unwrap_or(0);

                if state.position_ticks >= last_end {
                    state.is_playing = false;
                    state.position_ticks = 0;
                    state.next_note_idx = 0;

                    if let Some(handle) = state.timer_handle.take() {
                        let window = web_sys::window().unwrap();
                        window.clear_interval_with_handle(handle);
                    }

                    drop(state);
                    on_state_change(false);
                    return;
                }
            }
        }) as Box<dyn FnMut()>);

        let window = web_sys::window().unwrap();
        let handle = window
            .set_interval_with_callback_and_timeout_and_arguments_0(
                callback.as_ref().unchecked_ref(),
                interval_ms,
            )
            .unwrap();

        self.inner.lock().unwrap().timer_handle = Some(handle);

        // Prevent the closure from being dropped
        callback.forget();
    }
}

impl Default for Playback {
    fn default() -> Self {
        Self::new()
    }
}
