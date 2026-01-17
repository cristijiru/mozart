//! Header component

use crate::app::AppState;
use crate::tauri;
use leptos::prelude::*;

#[component]
pub fn Header() -> impl IntoView {
    let state = expect_context::<AppState>();

    let title = move || {
        state
            .song_info
            .get()
            .map(|info| info.title.clone())
            .unwrap_or_else(|| "Untitled".to_string())
    };

    let on_new = move |_| {
        let state = state.clone();
        leptos::spawn::spawn_local(async move {
            if let Err(e) = tauri::new_song().await {
                state.show_error(format!("Failed to create new song: {}", e));
            } else {
                state.refresh().await;
            }
        });
    };

    let on_undo = move |_| {
        let state = state.clone();
        leptos::spawn::spawn_local(async move {
            match tauri::undo().await {
                Ok(true) => state.refresh().await,
                Ok(false) => web_sys::console::log_1(&"Nothing to undo".into()),
                Err(e) => state.show_error(format!("Undo failed: {}", e)),
            }
        });
    };

    let on_redo = move |_| {
        let state = state.clone();
        leptos::spawn::spawn_local(async move {
            match tauri::redo().await {
                Ok(true) => state.refresh().await,
                Ok(false) => web_sys::console::log_1(&"Nothing to redo".into()),
                Err(e) => state.show_error(format!("Redo failed: {}", e)),
            }
        });
    };

    view! {
        <header class="header">
            <div class="header-left">
                <button class="icon-btn" title="Menu">
                    <span class="icon">"\u{2630}"</span>
                </button>
            </div>

            <div class="header-center">
                <h1 class="song-title">{title}</h1>
            </div>

            <div class="header-right">
                <button class="icon-btn" on:click=on_undo title="Undo">
                    <span class="icon">"\u{21B6}"</span>
                </button>
                <button class="icon-btn" on:click=on_redo title="Redo">
                    <span class="icon">"\u{21B7}"</span>
                </button>
                <button class="icon-btn" on:click=on_new title="New Song">
                    <span class="icon">"+"</span>
                </button>
            </div>
        </header>
    }
}
