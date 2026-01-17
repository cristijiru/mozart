//! Piano Roll editor component

use crate::app::AppState;
use crate::tauri::{self, NoteData};
use leptos::prelude::*;
use web_sys::{HtmlCanvasElement, MouseEvent};

/// Piano roll constants
const PITCH_HEIGHT: f64 = 12.0;
const TICK_WIDTH: f64 = 0.2;
const PIANO_KEY_WIDTH: f64 = 60.0;
const MIN_PITCH: u8 = 36;  // C2
const MAX_PITCH: u8 = 96;  // C7
const TICKS_PER_QUARTER: u32 = 480;

#[component]
pub fn PianoRoll() -> impl IntoView {
    let state = expect_context::<AppState>();
    let canvas_ref = NodeRef::<leptos::html::Canvas>::new();

    // Scroll position
    let scroll_x = RwSignal::new(0.0);
    let scroll_y = RwSignal::new((MAX_PITCH - 60) as f64 * PITCH_HEIGHT);

    // Selected note index
    let selected_note = RwSignal::new(None::<usize>);

    // Canvas dimensions
    let canvas_width = RwSignal::new(800.0);
    let canvas_height = RwSignal::new(400.0);

    // Redraw canvas when notes change
    Effect::new(move || {
        let notes = state.notes.get();
        let scroll_x = scroll_x.get();
        let scroll_y = scroll_y.get();
        let width = canvas_width.get();
        let height = canvas_height.get();
        let selected = selected_note.get();

        if let Some(canvas) = canvas_ref.get() {
            draw_piano_roll(&canvas, &notes, scroll_x, scroll_y, width, height, selected);
        }
    });

    // Handle canvas click
    let on_click = move |e: MouseEvent| {
        let canvas = canvas_ref.get().unwrap();
        let rect = canvas.get_bounding_client_rect();
        let x = e.client_x() as f64 - rect.left();
        let y = e.client_y() as f64 - rect.top();

        // Check if clicking on piano keys (preview note)
        if x < PIANO_KEY_WIDTH {
            let pitch = y_to_pitch(y + scroll_y.get());
            if pitch >= MIN_PITCH && pitch <= MAX_PITCH {
                let state_clone = state.clone();
                leptos::spawn::spawn_local(async move {
                    let _ = tauri::play_note_preview(pitch, 100, 500).await;
                });
            }
            return;
        }

        // Check if clicking on an existing note
        let notes = state.notes.get();
        let click_tick = x_to_tick(x - PIANO_KEY_WIDTH + scroll_x.get());
        let click_pitch = y_to_pitch(y + scroll_y.get());

        for (i, note) in notes.iter().enumerate() {
            if note.pitch == click_pitch
                && click_tick >= note.start_tick
                && click_tick < note.start_tick + note.duration_ticks
            {
                selected_note.set(Some(i));
                return;
            }
        }

        // Click on empty space - add a new note
        let quantized_tick = quantize_tick(click_tick);
        let new_note = NoteData {
            pitch: click_pitch,
            start_tick: quantized_tick,
            duration_ticks: TICKS_PER_QUARTER, // Default quarter note
            velocity: 100,
        };

        let state_clone = state.clone();
        leptos::spawn::spawn_local(async move {
            if let Err(e) = tauri::add_note(new_note).await {
                state_clone.show_error(format!("Failed to add note: {}", e));
            } else {
                state_clone.refresh().await;
            }
        });

        selected_note.set(None);
    };

    // Handle delete key
    let on_keydown = move |e: web_sys::KeyboardEvent| {
        if e.key() == "Delete" || e.key() == "Backspace" {
            if let Some(idx) = selected_note.get() {
                let state_clone = state.clone();
                leptos::spawn::spawn_local(async move {
                    if let Err(e) = tauri::remove_note(idx).await {
                        state_clone.show_error(format!("Failed to remove note: {}", e));
                    } else {
                        state_clone.refresh().await;
                    }
                });
                selected_note.set(None);
            }
        }
    };

    view! {
        <div class="piano-roll-container">
            <canvas
                node_ref=canvas_ref
                class="piano-roll-canvas"
                width="800"
                height="400"
                tabindex="0"
                on:click=on_click
                on:keydown=on_keydown
            />

            <div class="piano-roll-controls">
                <button class="scroll-btn" on:click=move |_| scroll_y.update(|y| *y = (*y - 50.0).max(0.0))>
                    "\u{25B2}"
                </button>
                <button class="scroll-btn" on:click=move |_| scroll_y.update(|y| *y += 50.0)>
                    "\u{25BC}"
                </button>
                <button class="scroll-btn" on:click=move |_| scroll_x.update(|x| *x = (*x - 100.0).max(0.0))>
                    "\u{25C0}"
                </button>
                <button class="scroll-btn" on:click=move |_| scroll_x.update(|x| *x += 100.0)>
                    "\u{25B6}"
                </button>
            </div>

            <div class="note-info">
                {move || selected_note.get().map(|idx| {
                    let notes = state.notes.get();
                    if let Some(note) = notes.get(idx) {
                        let pitch_name = pitch_to_name(note.pitch);
                        view! {
                            <span>"Selected: "{pitch_name}" at tick "{note.start_tick}</span>
                        }.into_any()
                    } else {
                        view! { <span></span> }.into_any()
                    }
                })}
            </div>
        </div>
    }
}

/// Draw the piano roll on canvas
fn draw_piano_roll(
    canvas: &HtmlCanvasElement,
    notes: &[NoteData],
    scroll_x: f64,
    scroll_y: f64,
    width: f64,
    height: f64,
    selected: Option<usize>,
) {
    let ctx = canvas
        .get_context("2d")
        .ok()
        .flatten()
        .and_then(|c| c.dyn_into::<web_sys::CanvasRenderingContext2d>().ok());

    let Some(ctx) = ctx else {
        return;
    };

    // Clear canvas
    ctx.set_fill_style_str("#1a1a2e");
    ctx.fill_rect(0.0, 0.0, width, height);

    // Draw grid lines
    ctx.set_stroke_style_str("#2d2d44");
    ctx.set_line_width(1.0);

    // Horizontal lines (pitch)
    for pitch in MIN_PITCH..=MAX_PITCH {
        let y = pitch_to_y(pitch) - scroll_y;
        if y >= 0.0 && y <= height {
            // Black keys have darker background
            if is_black_key(pitch) {
                ctx.set_fill_style_str("#15152a");
                ctx.fill_rect(PIANO_KEY_WIDTH, y, width - PIANO_KEY_WIDTH, PITCH_HEIGHT);
            }

            ctx.begin_path();
            ctx.move_to(PIANO_KEY_WIDTH, y);
            ctx.line_to(width, y);
            ctx.stroke();
        }
    }

    // Vertical lines (beats)
    let start_tick = (scroll_x / TICK_WIDTH) as u32;
    let end_tick = start_tick + ((width - PIANO_KEY_WIDTH) / TICK_WIDTH) as u32;

    let tick_step = TICKS_PER_QUARTER / 4; // Sixteenth notes
    let mut tick = (start_tick / tick_step) * tick_step;
    while tick < end_tick {
        let x = tick_to_x(tick) - scroll_x + PIANO_KEY_WIDTH;
        if x >= PIANO_KEY_WIDTH && x <= width {
            // Measure line
            if tick % (TICKS_PER_QUARTER * 4) == 0 {
                ctx.set_stroke_style_str("#4a4a6a");
                ctx.set_line_width(2.0);
            }
            // Beat line
            else if tick % TICKS_PER_QUARTER == 0 {
                ctx.set_stroke_style_str("#3a3a5a");
                ctx.set_line_width(1.5);
            }
            // Subdivision line
            else {
                ctx.set_stroke_style_str("#2d2d44");
                ctx.set_line_width(0.5);
            }

            ctx.begin_path();
            ctx.move_to(x, 0.0);
            ctx.line_to(x, height);
            ctx.stroke();
        }
        tick += tick_step;
    }

    // Draw piano keys
    draw_piano_keys(&ctx, scroll_y, height);

    // Draw notes
    for (i, note) in notes.iter().enumerate() {
        let x = tick_to_x(note.start_tick) - scroll_x + PIANO_KEY_WIDTH;
        let y = pitch_to_y(note.pitch) - scroll_y;
        let w = note.duration_ticks as f64 * TICK_WIDTH;
        let h = PITCH_HEIGHT - 2.0;

        // Skip if not visible
        if x + w < PIANO_KEY_WIDTH || x > width || y + h < 0.0 || y > height {
            continue;
        }

        // Note color based on selection
        let color = if Some(i) == selected {
            "#ff6b6b"
        } else {
            "#4ecdc4"
        };

        ctx.set_fill_style_str(color);
        ctx.fill_rect(x.max(PIANO_KEY_WIDTH), y, w, h);

        // Note border
        ctx.set_stroke_style_str("#ffffff");
        ctx.set_line_width(1.0);
        ctx.stroke_rect(x.max(PIANO_KEY_WIDTH), y, w, h);
    }
}

/// Draw the piano keyboard on the left side
fn draw_piano_keys(ctx: &web_sys::CanvasRenderingContext2d, scroll_y: f64, height: f64) {
    for pitch in MIN_PITCH..=MAX_PITCH {
        let y = pitch_to_y(pitch) - scroll_y;
        if y >= -PITCH_HEIGHT && y <= height {
            let is_black = is_black_key(pitch);

            if is_black {
                ctx.set_fill_style_str("#2a2a3a");
            } else {
                ctx.set_fill_style_str("#e8e8e8");
            }

            ctx.fill_rect(0.0, y, PIANO_KEY_WIDTH - 2.0, PITCH_HEIGHT - 1.0);

            // Key label (only for C notes)
            if pitch % 12 == 0 {
                ctx.set_fill_style_str(if is_black { "#ffffff" } else { "#000000" });
                ctx.set_font("10px sans-serif");
                let label = pitch_to_name(pitch);
                let _ = ctx.fill_text(&label, 5.0, y + PITCH_HEIGHT - 3.0);
            }
        }
    }

    // Border
    ctx.set_stroke_style_str("#3a3a5a");
    ctx.set_line_width(2.0);
    ctx.begin_path();
    ctx.move_to(PIANO_KEY_WIDTH - 1.0, 0.0);
    ctx.line_to(PIANO_KEY_WIDTH - 1.0, height);
    ctx.stroke();
}

// Utility functions

fn pitch_to_y(pitch: u8) -> f64 {
    (MAX_PITCH - pitch) as f64 * PITCH_HEIGHT
}

fn y_to_pitch(y: f64) -> u8 {
    let pitch = MAX_PITCH as i32 - (y / PITCH_HEIGHT) as i32;
    pitch.clamp(MIN_PITCH as i32, MAX_PITCH as i32) as u8
}

fn tick_to_x(tick: u32) -> f64 {
    tick as f64 * TICK_WIDTH
}

fn x_to_tick(x: f64) -> u32 {
    (x / TICK_WIDTH).max(0.0) as u32
}

fn quantize_tick(tick: u32) -> u32 {
    let grid = TICKS_PER_QUARTER / 4; // Quantize to sixteenth notes
    (tick / grid) * grid
}

fn is_black_key(pitch: u8) -> bool {
    matches!(pitch % 12, 1 | 3 | 6 | 8 | 10)
}

fn pitch_to_name(pitch: u8) -> String {
    let names = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];
    let name = names[(pitch % 12) as usize];
    let octave = (pitch / 12) as i32 - 1;
    format!("{}{}", name, octave)
}
