//! Text input component for entering melodies

use crate::app::AppState;
use crate::tauri;
use leptos::prelude::*;

#[component]
pub fn TextInput() -> impl IntoView {
    let state = expect_context::<AppState>();
    let text_value = RwSignal::new(String::new());
    let is_loading = RwSignal::new(false);
    let parse_error = RwSignal::new(None::<String>);

    // Load current melody text on mount
    let state_effect = state.clone();
    Effect::new(move || {
        let _ = state_effect.notes.get(); // Subscribe to notes changes
        leptos::task::spawn_local(async move {
            match tauri::get_melody_text().await {
                Ok(text) => text_value.set(text),
                Err(e) => web_sys::console::error_1(&format!("Failed to get melody text: {}", e).into()),
            }
        });
    });

    let state_parse = state.clone();
    let on_parse = move |_| {
        let text = text_value.get();
        if text.trim().is_empty() {
            return;
        }

        is_loading.set(true);
        parse_error.set(None);

        let state = state_parse.clone();
        leptos::task::spawn_local(async move {
            match tauri::parse_text_melody(&text).await {
                Ok(()) => {
                    state.refresh().await;
                    parse_error.set(None);
                }
                Err(e) => {
                    parse_error.set(Some(e));
                }
            }
            is_loading.set(false);
        });
    };

    let state_clear = state.clone();
    let on_clear = move |_| {
        let state = state_clear.clone();
        leptos::task::spawn_local(async move {
            if let Err(e) = tauri::clear_notes().await {
                state.show_error(format!("Failed to clear notes: {}", e));
            } else {
                state.refresh().await;
                text_value.set(String::new());
            }
        });
    };

    view! {
        <div class="text-input-container">
            <div class="text-input-header">
                <h2>"Text Input"</h2>
                <p class="help-text">
                    "Enter notes like: C4q D4q E4h F#5q Rq"
                </p>
            </div>

            <div class="text-input-help">
                <details>
                    <summary>"Notation Help"</summary>
                    <div class="help-content">
                        <h4>"Note Format"</h4>
                        <p>"<Note><Octave><Duration> (e.g., C4q = C in octave 4, quarter note)"</p>

                        <h4>"Notes"</h4>
                        <p>"C, D, E, F, G, A, B (add # or b for accidentals: C#, Bb)"</p>

                        <h4>"Durations"</h4>
                        <ul>
                            <li>"w = whole note"</li>
                            <li>"h = half note"</li>
                            <li>"q = quarter note"</li>
                            <li>"e = eighth note"</li>
                            <li>"s = sixteenth note"</li>
                            <li>"Add . for dotted (e.g., q. = dotted quarter)"</li>
                        </ul>

                        <h4>"Rests"</h4>
                        <p>"R followed by duration (e.g., Rq = quarter rest)"</p>

                        <h4>"Example"</h4>
                        <code>"C4q D4q E4q F4q G4h Rq G4q A4q B4q C5w"</code>
                    </div>
                </details>
            </div>

            <textarea
                class="text-input-area"
                prop:value=move || text_value.get()
                on:input=move |e| {
                    let value = event_target_value(&e);
                    text_value.set(value);
                }
                placeholder="Enter melody notation here..."
                spellcheck="false"
            />

            {move || parse_error.get().map(|err| view! {
                <div class="parse-error">
                    <span class="error-icon">"⚠"</span>
                    <span>{err}</span>
                </div>
            })}

            <div class="text-input-actions">
                <button
                    class="btn btn-primary"
                    on:click=on_parse
                    disabled=move || is_loading.get()
                >
                    {move || if is_loading.get() { "Parsing..." } else { "Parse & Apply" }}
                </button>

                <button
                    class="btn btn-secondary"
                    on:click=on_clear
                >
                    "Clear All"
                </button>
            </div>

            <div class="note-preview">
                <h4>"Current Notes"</h4>
                <div class="note-count">
                    {
                        let state = state.clone();
                        move || {
                            let count = state.notes.get().len();
                            format!("{} note{}", count, if count == 1 { "" } else { "s" })
                        }
                    }
                </div>
            </div>
        </div>
    }
}
