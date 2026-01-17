//! Transposition panel component

use crate::app::AppState;
use crate::tauri::{self, TransposeRequest};
use leptos::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq)]
enum TransposeType {
    Chromatic,
    Diatonic,
}

#[component]
pub fn TransposePanel() -> impl IntoView {
    let state = expect_context::<AppState>();

    let transpose_type = RwSignal::new(TransposeType::Chromatic);
    let chromatic_semitones = RwSignal::new(0i8);
    let diatonic_degrees = RwSignal::new(0i8);
    let target_root = RwSignal::new("C".to_string());
    let target_scale_type = RwSignal::new("Major".to_string());
    let change_key = RwSignal::new(false);

    let pitch_classes = ["C", "C#", "D", "Eb", "E", "F", "F#", "G", "Ab", "A", "Bb", "B"];
    let scale_types = [
        "Major",
        "Natural Minor",
        "Harmonic Minor",
        "Melodic Minor",
        "Dorian",
        "Phrygian",
        "Lydian",
        "Mixolydian",
        "Locrian",
    ];

    let current_key = move || {
        state
            .song_info
            .get()
            .map(|info| info.key.clone())
            .unwrap_or_else(|| "C Major".to_string())
    };

    let interval_name = move || {
        let semitones = chromatic_semitones.get();
        let dir = if semitones >= 0 { "up" } else { "down" };
        let interval = semitones.abs();
        let name = match interval {
            0 => return "No transposition".to_string(),
            1 => "minor 2nd",
            2 => "major 2nd",
            3 => "minor 3rd",
            4 => "major 3rd",
            5 => "perfect 4th",
            6 => "tritone",
            7 => "perfect 5th",
            8 => "minor 6th",
            9 => "major 6th",
            10 => "minor 7th",
            11 => "major 7th",
            12 => "octave",
            _ => return format!("{} semitones {}", interval, dir),
        };
        format!("{} a {}", dir, name)
    };

    let degree_name = move || {
        let degrees = diatonic_degrees.get();
        let dir = if degrees >= 0 { "up" } else { "down" };
        let degree = degrees.abs();
        let name = match degree {
            0 => return "No transposition".to_string(),
            1 => "2nd",
            2 => "3rd",
            3 => "4th",
            4 => "5th",
            5 => "6th",
            6 => "7th",
            7 => "octave",
            _ => return format!("{} degrees {}", degree, dir),
        };
        format!("Diatonic {} a {}", dir, name)
    };

    let on_transpose = move |_| {
        let request = match transpose_type.get() {
            TransposeType::Chromatic => TransposeRequest::Chromatic {
                semitones: chromatic_semitones.get(),
            },
            TransposeType::Diatonic => {
                if change_key.get() {
                    TransposeRequest::DiatonicWithKeyChange {
                        degrees: diatonic_degrees.get(),
                        target_root: target_root.get(),
                        target_scale_type: target_scale_type.get(),
                    }
                } else {
                    TransposeRequest::Diatonic {
                        degrees: diatonic_degrees.get(),
                    }
                }
            }
        };

        let state = state.clone();
        leptos::spawn::spawn_local(async move {
            match tauri::transpose(request).await {
                Ok(()) => {
                    state.refresh().await;
                }
                Err(e) => {
                    state.show_error(format!("Transposition failed: {}", e));
                }
            }
        });
    };

    view! {
        <div class="transpose-panel">
            <h2>"Transpose"</h2>

            <div class="current-key">
                <span class="label">"Current Key:"</span>
                <span class="value">{current_key}</span>
            </div>

            <div class="transpose-type-selector">
                <button
                    class="type-btn"
                    class:active=move || transpose_type.get() == TransposeType::Chromatic
                    on:click=move |_| transpose_type.set(TransposeType::Chromatic)
                >
                    "Chromatic"
                </button>
                <button
                    class="type-btn"
                    class:active=move || transpose_type.get() == TransposeType::Diatonic
                    on:click=move |_| transpose_type.set(TransposeType::Diatonic)
                >
                    "Diatonic"
                </button>
            </div>

            {move || match transpose_type.get() {
                TransposeType::Chromatic => view! {
                    <div class="chromatic-controls">
                        <label class="control-label">"Semitones"</label>

                        <div class="semitone-slider">
                            <button
                                class="adjust-btn"
                                on:click=move |_| chromatic_semitones.update(|v| *v = (*v - 1).max(-24))
                            >
                                "-"
                            </button>

                            <span class="semitone-value">{move || chromatic_semitones.get()}</span>

                            <button
                                class="adjust-btn"
                                on:click=move |_| chromatic_semitones.update(|v| *v = (*v + 1).min(24))
                            >
                                "+"
                            </button>
                        </div>

                        <div class="interval-name">{interval_name}</div>

                        <div class="quick-buttons">
                            <button on:click=move |_| chromatic_semitones.set(-12)>"-8ve"</button>
                            <button on:click=move |_| chromatic_semitones.set(-7)>"-5th"</button>
                            <button on:click=move |_| chromatic_semitones.set(-5)>"-4th"</button>
                            <button on:click=move |_| chromatic_semitones.set(0)>"0"</button>
                            <button on:click=move |_| chromatic_semitones.set(5)>"+4th"</button>
                            <button on:click=move |_| chromatic_semitones.set(7)>"+5th"</button>
                            <button on:click=move |_| chromatic_semitones.set(12)>"+8ve"</button>
                        </div>
                    </div>
                }.into_any(),

                TransposeType::Diatonic => view! {
                    <div class="diatonic-controls">
                        <label class="control-label">"Scale Degrees"</label>

                        <div class="degree-slider">
                            <button
                                class="adjust-btn"
                                on:click=move |_| diatonic_degrees.update(|v| *v = (*v - 1).max(-7))
                            >
                                "-"
                            </button>

                            <span class="degree-value">{move || diatonic_degrees.get()}</span>

                            <button
                                class="adjust-btn"
                                on:click=move |_| diatonic_degrees.update(|v| *v = (*v + 1).min(7))
                            >
                                "+"
                            </button>
                        </div>

                        <div class="interval-name">{degree_name}</div>

                        <div class="quick-buttons">
                            <button on:click=move |_| diatonic_degrees.set(-2)>"-3rd"</button>
                            <button on:click=move |_| diatonic_degrees.set(-1)>"-2nd"</button>
                            <button on:click=move |_| diatonic_degrees.set(0)>"0"</button>
                            <button on:click=move |_| diatonic_degrees.set(1)>"+2nd"</button>
                            <button on:click=move |_| diatonic_degrees.set(2)>"+3rd"</button>
                            <button on:click=move |_| diatonic_degrees.set(4)>"+5th"</button>
                        </div>

                        <div class="key-change-option">
                            <label>
                                <input
                                    type="checkbox"
                                    prop:checked=move || change_key.get()
                                    on:change=move |e| change_key.set(event_target_checked(&e))
                                />
                                " Change key after transpose"
                            </label>
                        </div>

                        {move || change_key.get().then(|| view! {
                            <div class="target-key-selector">
                                <label>"Target Key:"</label>
                                <select
                                    on:change=move |e| target_root.set(event_target_value(&e))
                                >
                                    {pitch_classes.iter().map(|&pc| view! {
                                        <option value=pc selected=move || target_root.get() == pc>
                                            {pc}
                                        </option>
                                    }).collect_view()}
                                </select>

                                <select
                                    on:change=move |e| target_scale_type.set(event_target_value(&e))
                                >
                                    {scale_types.iter().map(|&st| view! {
                                        <option value=st selected=move || target_scale_type.get() == st>
                                            {st}
                                        </option>
                                    }).collect_view()}
                                </select>
                            </div>
                        })}
                    </div>
                }.into_any(),
            }}

            <button
                class="btn btn-primary transpose-btn"
                on:click=on_transpose
            >
                "Apply Transposition"
            </button>
        </div>
    }
}
