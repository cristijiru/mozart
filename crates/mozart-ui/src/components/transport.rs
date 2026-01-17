//! Transport controls component

use crate::app::AppState;
use crate::tauri;
use leptos::prelude::*;

#[component]
pub fn Transport() -> impl IntoView {
    let state = expect_context::<AppState>();

    let state_playing = state.clone();
    let is_playing = move || state_playing.playback_state.get() == "playing";

    let state_tempo = state.clone();
    let tempo = move || {
        state_tempo
            .song_info
            .get()
            .map(|info| info.tempo)
            .unwrap_or(120)
    };

    let state_time = state.clone();
    let time_sig = move || {
        state_time
            .song_info
            .get()
            .map(|info| info.time_signature.clone())
            .unwrap_or_else(|| "4/4".to_string())
    };

    let state_play = state.clone();
    let on_play = move |_| {
        let state = state_play.clone();
        leptos::task::spawn_local(async move {
            if let Err(e) = tauri::play().await {
                state.show_error(format!("Play failed: {}", e));
            }
            state.playback_state.set("playing".to_string());
        });
    };

    let state_pause = state.clone();
    let on_pause = move |_| {
        let state = state_pause.clone();
        leptos::task::spawn_local(async move {
            if let Err(e) = tauri::pause().await {
                state.show_error(format!("Pause failed: {}", e));
            }
            state.playback_state.set("paused".to_string());
        });
    };

    let state_stop = state.clone();
    let on_stop = move |_| {
        let state = state_stop.clone();
        leptos::task::spawn_local(async move {
            if let Err(e) = tauri::stop().await {
                state.show_error(format!("Stop failed: {}", e));
            }
            state.playback_state.set("stopped".to_string());
        });
    };

    let state_metro = state.clone();
    let on_toggle_metronome = move |_| {
        let state = state_metro.clone();
        leptos::task::spawn_local(async move {
            if let Err(e) = tauri::toggle_metronome().await {
                state.show_error(format!("Toggle metronome failed: {}", e));
            }
        });
    };

    // Clone callbacks for use inside the reactive closure
    let on_play_inner = on_play.clone();
    let on_pause_inner = on_pause.clone();
    let on_stop_inner = on_stop.clone();

    view! {
        <div class="transport">
            <div class="transport-left">
                <span class="tempo-display">{tempo}" BPM"</span>
                <span class="time-sig-display">{time_sig}</span>
            </div>

            <div class="transport-center">
                // Rewind
                <button class="transport-btn" on:click=on_stop.clone() title="Stop / Rewind">
                    <span class="transport-icon">"\u{23EE}"</span>
                </button>

                // Play/Pause
                {move || {
                    let on_pause = on_pause_inner.clone();
                    let on_play = on_play_inner.clone();
                    if is_playing() {
                        view! {
                            <button class="transport-btn play-btn" on:click=move |_| on_pause(()) title="Pause">
                                <span class="transport-icon">"\u{23F8}"</span>
                            </button>
                        }.into_any()
                    } else {
                        view! {
                            <button class="transport-btn play-btn" on:click=move |_| on_play(()) title="Play">
                                <span class="transport-icon">"\u{25B6}"</span>
                            </button>
                        }.into_any()
                    }
                }}

                // Stop
                <button class="transport-btn" on:click=on_stop_inner title="Stop">
                    <span class="transport-icon">"\u{23F9}"</span>
                </button>

                // Loop
                <button class="transport-btn" title="Loop">
                    <span class="transport-icon">"\u{1F501}"</span>
                </button>
            </div>

            <div class="transport-right">
                // Metronome
                <button class="transport-btn" on:click=on_toggle_metronome title="Metronome">
                    <span class="transport-icon">"\u{1F3B5}"</span>
                </button>

                // Tap tempo
                <button class="transport-btn" title="Tap Tempo">
                    <span class="transport-icon">"\u{1F44F}"</span>
                </button>
            </div>
        </div>
    }
}
