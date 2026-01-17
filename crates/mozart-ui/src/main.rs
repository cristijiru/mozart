//! Mozart UI - Leptos frontend
//!
//! Main entry point for the WASM frontend

mod app;
pub mod audio;
mod components;
mod tauri;

use leptos::prelude::*;

fn main() {
    console_error_panic_hook::set_once();
    web_sys::console::log_1(&"Mozart UI starting...".into());

    leptos::mount::mount_to_body(app::App);
}
