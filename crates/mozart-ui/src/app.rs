//! Main App Component

use crate::components::*;
use crate::playback::Playback;
use crate::tauri::{self, NoteData, SongInfo};
use leptos::prelude::*;

/// Main application state
#[derive(Clone)]
pub struct AppState {
    pub song_info: RwSignal<Option<SongInfo>>,
    pub notes: RwSignal<Vec<NoteData>>,
    pub playback_state: RwSignal<String>,
    pub selected_tab: RwSignal<Tab>,
    pub error_message: RwSignal<Option<String>>,
    pub playback: Playback,
}

impl AppState {
    pub fn new() -> Self {
        AppState {
            song_info: RwSignal::new(None),
            notes: RwSignal::new(Vec::new()),
            playback_state: RwSignal::new("stopped".to_string()),
            selected_tab: RwSignal::new(Tab::PianoRoll),
            error_message: RwSignal::new(None),
            playback: Playback::new(),
        }
    }

    pub async fn refresh(&self) {
        // Fetch song info
        match tauri::get_song_info().await {
            Ok(info) => self.song_info.set(Some(info)),
            Err(e) => web_sys::console::error_1(&format!("Failed to get song info: {}", e).into()),
        }

        // Fetch notes
        match tauri::get_notes().await {
            Ok(notes) => self.notes.set(notes),
            Err(e) => web_sys::console::error_1(&format!("Failed to get notes: {}", e).into()),
        }

        // Fetch playback state
        match tauri::get_playback_state().await {
            Ok(state) => self.playback_state.set(state),
            Err(e) => {
                web_sys::console::error_1(&format!("Failed to get playback state: {}", e).into())
            }
        }
    }

    pub fn show_error(&self, msg: String) {
        self.error_message.set(Some(msg));
    }

    pub fn clear_error(&self) {
        self.error_message.set(None);
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tab {
    PianoRoll,
    TextInput,
    Transpose,
    Accents,
    Settings,
}

/// Main App component
#[component]
pub fn App() -> impl IntoView {
    let state = AppState::new();

    // Initial data fetch
    let state_clone = state.clone();
    leptos::task::spawn_local(async move {
        state_clone.refresh().await;
    });

    // Provide state to all children
    provide_context(state.clone());

    view! {
        <div class="app">
            <Header />

            <main class="main-content">
                <div class="editor-area">
                    {move || match state.selected_tab.get() {
                        Tab::PianoRoll => view! { <PianoRoll /> }.into_any(),
                        Tab::TextInput => view! { <TextInput /> }.into_any(),
                        Tab::Transpose => view! { <TransposePanel /> }.into_any(),
                        Tab::Accents => view! { <AccentEditor /> }.into_any(),
                        Tab::Settings => view! { <Settings /> }.into_any(),
                    }}
                </div>
            </main>

            <Transport />

            <TabBar />

            // Error toast
            {
                let state = state.clone();
                move || {
                    let state = state.clone();
                    state.error_message.get().map(|msg| {
                        let state = state.clone();
                        view! {
                            <div class="error-toast" on:click=move |_| state.clear_error()>
                                <span class="error-icon">"!"</span>
                                <span class="error-text">{msg}</span>
                            </div>
                        }
                    })
                }
            }
        </div>
    }
}

/// Tab bar at bottom of screen
#[component]
fn TabBar() -> impl IntoView {
    let state = expect_context::<AppState>();

    let tabs = [
        (Tab::PianoRoll, "Piano Roll", "grid"),
        (Tab::TextInput, "Text", "text"),
        (Tab::Transpose, "Transpose", "transpose"),
        (Tab::Accents, "Accents", "accent"),
        (Tab::Settings, "Settings", "settings"),
    ];

    view! {
        <nav class="tab-bar">
            {tabs.into_iter().map(|(tab, label, icon)| {
                let is_active = move || state.selected_tab.get() == tab;
                view! {
                    <button
                        class="tab-item"
                        class:active=is_active
                        on:click=move |_| state.selected_tab.set(tab)
                    >
                        <span class="tab-icon" data-icon=icon></span>
                        <span class="tab-label">{label}</span>
                    </button>
                }
            }).collect_view()}
        </nav>
    }
}
