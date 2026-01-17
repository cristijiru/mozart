//! Settings panel component

use crate::app::AppState;
use crate::tauri;
use leptos::prelude::*;

#[component]
pub fn Settings() -> impl IntoView {
    let state = expect_context::<AppState>();

    let tempo = RwSignal::new(120u16);
    let title = RwSignal::new(String::new());
    let key_root = RwSignal::new("C".to_string());
    let key_scale = RwSignal::new("Major".to_string());
    let instrument = RwSignal::new("Piano".to_string());

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
    let instruments = ["Piano", "Strings", "Synth"];

    // Load current settings
    let state_clone = state.clone();
    Effect::new(move || {
        if let Some(info) = state_clone.song_info.get() {
            tempo.set(info.tempo);
            title.set(info.title);

            // Parse key (e.g., "C Major" -> "C", "Major")
            let parts: Vec<&str> = info.key.split_whitespace().collect();
            if parts.len() >= 2 {
                key_root.set(parts[0].to_string());
                key_scale.set(parts[1..].join(" "));
            }
        }
    });

    let on_tempo_change = move |new_tempo: u16| {
        let state = state.clone();
        leptos::spawn::spawn_local(async move {
            if let Err(e) = tauri::set_tempo(new_tempo).await {
                state.show_error(format!("Failed to set tempo: {}", e));
            } else {
                state.refresh().await;
            }
        });
    };

    let on_title_change = move || {
        let new_title = title.get();
        let state = state.clone();
        leptos::spawn::spawn_local(async move {
            if let Err(e) = tauri::set_title(&new_title).await {
                state.show_error(format!("Failed to set title: {}", e));
            } else {
                state.refresh().await;
            }
        });
    };

    let on_key_change = move || {
        let root = key_root.get();
        let scale = key_scale.get();
        let state = state.clone();
        leptos::spawn::spawn_local(async move {
            if let Err(e) = tauri::set_key(&root, &scale).await {
                state.show_error(format!("Failed to set key: {}", e));
            } else {
                state.refresh().await;
            }
        });
    };

    let on_instrument_change = move |inst: String| {
        let state = state.clone();
        leptos::spawn::spawn_local(async move {
            if let Err(e) = tauri::set_instrument(&inst).await {
                state.show_error(format!("Failed to set instrument: {}", e));
            }
        });
    };

    let on_save = move |_| {
        // For now, just get JSON and log it
        // In a real app, we'd use a file dialog
        let state = state.clone();
        leptos::spawn::spawn_local(async move {
            match tauri::get_song_json().await {
                Ok(json) => {
                    web_sys::console::log_1(&format!("Song JSON:\n{}", json).into());
                    // Could copy to clipboard or show in modal
                }
                Err(e) => state.show_error(format!("Failed to get song JSON: {}", e)),
            }
        });
    };

    let on_export_midi = move |_| {
        let state = state.clone();
        leptos::spawn::spawn_local(async move {
            // For now, log that export was requested
            // In a real app, we'd use a file dialog
            web_sys::console::log_1(&"MIDI export requested - needs file dialog".into());
        });
    };

    view! {
        <div class="settings-panel">
            <h2>"Song Settings"</h2>

            <div class="settings-group">
                <label class="setting-label">"Title"</label>
                <input
                    type="text"
                    class="setting-input"
                    prop:value=move || title.get()
                    on:input=move |e| title.set(event_target_value(&e))
                    on:blur=move |_| on_title_change()
                />
            </div>

            <div class="settings-group">
                <label class="setting-label">"Tempo (BPM)"</label>
                <div class="tempo-control">
                    <button
                        class="adjust-btn"
                        on:click=move |_| {
                            let new_tempo = (tempo.get() - 5).max(20);
                            tempo.set(new_tempo);
                            on_tempo_change(new_tempo);
                        }
                    >"-5"</button>
                    <button
                        class="adjust-btn"
                        on:click=move |_| {
                            let new_tempo = (tempo.get() - 1).max(20);
                            tempo.set(new_tempo);
                            on_tempo_change(new_tempo);
                        }
                    >"-"</button>

                    <span class="tempo-value">{move || tempo.get()}</span>

                    <button
                        class="adjust-btn"
                        on:click=move |_| {
                            let new_tempo = (tempo.get() + 1).min(300);
                            tempo.set(new_tempo);
                            on_tempo_change(new_tempo);
                        }
                    >"+"</button>
                    <button
                        class="adjust-btn"
                        on:click=move |_| {
                            let new_tempo = (tempo.get() + 5).min(300);
                            tempo.set(new_tempo);
                            on_tempo_change(new_tempo);
                        }
                    >"+5"</button>
                </div>

                <div class="tempo-presets">
                    {[60, 80, 100, 120, 140, 160].into_iter().map(|t| {
                        view! {
                            <button
                                class="preset-btn"
                                class:active=move || tempo.get() == t
                                on:click=move |_| {
                                    tempo.set(t);
                                    on_tempo_change(t);
                                }
                            >
                                {t}
                            </button>
                        }
                    }).collect_view()}
                </div>
            </div>

            <div class="settings-group">
                <label class="setting-label">"Key"</label>
                <div class="key-selector">
                    <select
                        class="key-select"
                        on:change=move |e| {
                            key_root.set(event_target_value(&e));
                            on_key_change();
                        }
                    >
                        {pitch_classes.iter().map(|&pc| view! {
                            <option value=pc selected=move || key_root.get() == pc>
                                {pc}
                            </option>
                        }).collect_view()}
                    </select>

                    <select
                        class="scale-select"
                        on:change=move |e| {
                            key_scale.set(event_target_value(&e));
                            on_key_change();
                        }
                    >
                        {scale_types.iter().map(|&st| view! {
                            <option value=st selected=move || key_scale.get() == st>
                                {st}
                            </option>
                        }).collect_view()}
                    </select>
                </div>
            </div>

            <div class="settings-group">
                <label class="setting-label">"Instrument"</label>
                <div class="instrument-selector">
                    {instruments.iter().map(|&inst| {
                        let inst_string = inst.to_string();
                        view! {
                            <button
                                class="instrument-btn"
                                class:active=move || instrument.get() == inst
                                on:click=move |_| {
                                    instrument.set(inst.to_string());
                                    on_instrument_change(inst_string.clone());
                                }
                            >
                                {inst}
                            </button>
                        }
                    }).collect_view()}
                </div>
            </div>

            <div class="settings-divider"></div>

            <h3>"File Operations"</h3>

            <div class="file-actions">
                <button class="btn btn-secondary" on:click=on_save>
                    "Save Song (JSON)"
                </button>
                <button class="btn btn-secondary" on:click=on_export_midi>
                    "Export MIDI"
                </button>
            </div>

            <div class="settings-divider"></div>

            <div class="song-stats">
                <h4>"Song Statistics"</h4>
                {move || state.song_info.get().map(|info| view! {
                    <div class="stat-grid">
                        <div class="stat">
                            <span class="stat-label">"Notes"</span>
                            <span class="stat-value">{info.note_count}</span>
                        </div>
                        <div class="stat">
                            <span class="stat-label">"Duration"</span>
                            <span class="stat-value">{format!("{:.1}s", info.duration_seconds)}</span>
                        </div>
                        <div class="stat">
                            <span class="stat-label">"Measures"</span>
                            <span class="stat-value">{info.measure_count}</span>
                        </div>
                    </div>
                })}
            </div>
        </div>
    }
}
