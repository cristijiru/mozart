//! Accent pattern editor component

use crate::app::AppState;
use crate::tauri;
use leptos::prelude::*;

#[component]
pub fn AccentEditor() -> impl IntoView {
    let state = expect_context::<AppState>();

    let accents = RwSignal::new(vec![3u8, 1, 2, 1]); // Default 4/4
    let numerator = RwSignal::new(4u8);
    let denominator = RwSignal::new(4u8);

    // Load current accents
    let state_clone = state.clone();
    Effect::new(move || {
        let _ = state_clone.song_info.get(); // Subscribe to changes
        leptos::spawn::spawn_local(async move {
            match tauri::get_accents().await {
                Ok(info) => {
                    accents.set(info.accents);
                    numerator.set(info.numerator);
                    denominator.set(info.denominator);
                }
                Err(e) => {
                    web_sys::console::error_1(&format!("Failed to get accents: {}", e).into())
                }
            }
        });
    });

    let on_cycle_accent = move |beat: usize| {
        let state = state.clone();
        leptos::spawn::spawn_local(async move {
            if let Err(e) = tauri::cycle_accent(beat).await {
                state.show_error(format!("Failed to cycle accent: {}", e));
            }
            // Refresh accents
            if let Ok(info) = tauri::get_accents().await {
                accents.set(info.accents);
            }
        });
    };

    let on_change_time_sig = move |num: u8, denom: u8| {
        let state = state.clone();
        leptos::spawn::spawn_local(async move {
            if let Err(e) = tauri::set_time_signature(num, denom).await {
                state.show_error(format!("Failed to set time signature: {}", e));
            } else {
                state.refresh().await;
                // Refresh accents
                if let Ok(info) = tauri::get_accents().await {
                    accents.set(info.accents);
                    numerator.set(info.numerator);
                    denominator.set(info.denominator);
                }
            }
        });
    };

    let accent_class = |level: u8| match level {
        3 => "accent strong",
        2 => "accent medium",
        _ => "accent weak",
    };

    let accent_symbol = |level: u8| match level {
        3 => ">",
        2 => "-",
        _ => ".",
    };

    view! {
        <div class="accent-editor">
            <h2>"Time Signature & Accents"</h2>

            <div class="time-sig-selector">
                <div class="time-sig-control">
                    <label>"Beats per measure"</label>
                    <div class="number-input">
                        <button
                            on:click=move |_| {
                                let num = (numerator.get() - 1).max(2);
                                on_change_time_sig(num, denominator.get());
                            }
                        >"-"</button>
                        <span class="number-value">{move || numerator.get()}</span>
                        <button
                            on:click=move |_| {
                                let num = (numerator.get() + 1).min(15);
                                on_change_time_sig(num, denominator.get());
                            }
                        >"+"</button>
                    </div>
                </div>

                <div class="time-sig-display">
                    <span class="numerator">{move || numerator.get()}</span>
                    <span class="divider">"/"</span>
                    <span class="denominator">{move || denominator.get()}</span>
                </div>

                <div class="time-sig-control">
                    <label>"Beat unit"</label>
                    <div class="denom-buttons">
                        <button
                            class:active=move || denominator.get() == 4
                            on:click=move |_| on_change_time_sig(numerator.get(), 4)
                        >"4"</button>
                        <button
                            class:active=move || denominator.get() == 8
                            on:click=move |_| on_change_time_sig(numerator.get(), 8)
                        >"8"</button>
                    </div>
                </div>
            </div>

            <div class="accent-help">
                <p>"Click on a beat to cycle its accent level:"</p>
                <div class="accent-legend">
                    <span class="legend-item"><span class="accent strong">">"</span>" Strong (downbeat)"</span>
                    <span class="legend-item"><span class="accent medium">"-"</span>" Medium"</span>
                    <span class="legend-item"><span class="accent weak">"."</span>" Weak"</span>
                </div>
            </div>

            <div class="accent-pattern">
                {move || {
                    let accents_val = accents.get();
                    accents_val.into_iter().enumerate().map(|(i, level)| {
                        let on_click = move |_| on_cycle_accent(i);
                        view! {
                            <button
                                class=accent_class(level)
                                on:click=on_click
                                title=format!("Beat {} - Click to change", i + 1)
                            >
                                <span class="beat-number">{i + 1}</span>
                                <span class="accent-symbol">{accent_symbol(level)}</span>
                            </button>
                        }
                    }).collect_view()
                }}
            </div>

            <div class="preset-patterns">
                <h4>"Preset Patterns"</h4>
                <div class="presets">
                    {move || {
                        let num = numerator.get();
                        get_presets_for_numerator(num).into_iter().map(|(name, pattern)| {
                            let pattern_clone = pattern.clone();
                            let state = state.clone();
                            let on_click = move |_| {
                                let pattern = pattern_clone.clone();
                                let state = state.clone();
                                leptos::spawn::spawn_local(async move {
                                    if let Err(e) = tauri::set_accents(pattern).await {
                                        state.show_error(format!("Failed to set accents: {}", e));
                                    }
                                    if let Ok(info) = tauri::get_accents().await {
                                        accents.set(info.accents);
                                    }
                                });
                            };
                            view! {
                                <button class="preset-btn" on:click=on_click>
                                    {name}
                                </button>
                            }
                        }).collect_view()
                    }}
                </div>
            </div>
        </div>
    }
}

fn get_presets_for_numerator(num: u8) -> Vec<(&'static str, Vec<u8>)> {
    match num {
        4 => vec![
            ("Standard", vec![3, 1, 2, 1]),
            ("Backbeat", vec![1, 3, 1, 3]),
            ("March", vec![3, 1, 3, 1]),
        ],
        3 => vec![
            ("Waltz", vec![3, 1, 1]),
            ("Sarabande", vec![1, 3, 1]),
        ],
        5 => vec![
            ("3+2", vec![3, 1, 1, 2, 1]),
            ("2+3", vec![3, 1, 2, 1, 1]),
        ],
        6 => vec![
            ("Compound", vec![3, 1, 1, 2, 1, 1]),
            ("Simple", vec![3, 1, 2, 1, 2, 1]),
        ],
        7 => vec![
            ("3+2+2", vec![3, 1, 1, 2, 1, 2, 1]),
            ("2+2+3", vec![3, 1, 2, 1, 2, 1, 1]),
            ("2+3+2", vec![3, 1, 2, 1, 1, 2, 1]),
        ],
        9 => vec![
            ("3+3+3", vec![3, 1, 1, 2, 1, 1, 2, 1, 1]),
            ("2+2+2+3", vec![3, 1, 2, 1, 2, 1, 2, 1, 1]),
        ],
        11 => vec![
            ("3+3+3+2", vec![3, 1, 1, 2, 1, 1, 2, 1, 1, 2, 1]),
            ("3+3+2+3", vec![3, 1, 1, 2, 1, 1, 2, 1, 2, 1, 1]),
            ("3+2+3+3", vec![3, 1, 1, 2, 1, 2, 1, 1, 2, 1, 1]),
            ("2+3+3+3", vec![3, 1, 2, 1, 1, 2, 1, 1, 2, 1, 1]),
        ],
        _ => vec![
            ("Default", (0..num).map(|i| if i == 0 { 3 } else { 1 }).collect()),
        ],
    }
}
