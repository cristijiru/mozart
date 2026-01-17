//! Web Audio API sine wave player
//!
//! Simple sine wave oscillator for note preview using Web Audio API

use wasm_bindgen::prelude::*;
use web_sys::{AudioContext, OscillatorNode, GainNode, OscillatorType};

/// Play a sine wave note at the given MIDI pitch
pub fn play_sine_note(pitch: u8, velocity: u8, duration_ms: u32) {
    // Spawn the audio playback
    wasm_bindgen_futures::spawn_local(async move {
        if let Err(e) = play_sine_note_inner(pitch, velocity, duration_ms).await {
            web_sys::console::error_1(&format!("Audio error: {:?}", e).into());
        }
    });
}

async fn play_sine_note_inner(pitch: u8, velocity: u8, duration_ms: u32) -> Result<(), JsValue> {
    // Create audio context
    let ctx = AudioContext::new()?;

    // Create oscillator
    let osc = ctx.create_oscillator()?;
    osc.set_type(OscillatorType::Sine);

    // Convert MIDI pitch to frequency: f = 440 * 2^((pitch - 69) / 12)
    let frequency = 440.0 * 2.0_f64.powf((pitch as f64 - 69.0) / 12.0);
    osc.frequency().set_value(frequency as f32);

    // Create gain node for volume control and envelope
    let gain = ctx.create_gain()?;
    let volume = (velocity as f32 / 127.0) * 0.3; // Scale velocity, keep overall volume moderate

    let current_time = ctx.current_time();
    let attack_time = 0.01; // 10ms attack
    let release_time = 0.1; // 100ms release
    let sustain_time = (duration_ms as f64 / 1000.0) - attack_time - release_time;
    let sustain_time = sustain_time.max(0.0);

    // ADSR envelope
    gain.gain().set_value_at_time(0.0, current_time)?;
    gain.gain().linear_ramp_to_value_at_time(volume, current_time + attack_time)?;
    gain.gain().set_value_at_time(volume, current_time + attack_time + sustain_time)?;
    gain.gain().linear_ramp_to_value_at_time(0.0, current_time + attack_time + sustain_time + release_time)?;

    // Connect nodes: oscillator -> gain -> destination
    osc.connect_with_audio_node(&gain)?;
    gain.connect_with_audio_node(&ctx.destination())?;

    // Start and stop oscillator
    osc.start()?;
    osc.stop_with_when(current_time + attack_time + sustain_time + release_time + 0.01)?;

    Ok(())
}

/// Play a click sound for metronome
pub fn play_click(is_downbeat: bool) {
    wasm_bindgen_futures::spawn_local(async move {
        if let Err(e) = play_click_inner(is_downbeat).await {
            web_sys::console::error_1(&format!("Click error: {:?}", e).into());
        }
    });
}

async fn play_click_inner(is_downbeat: bool) -> Result<(), JsValue> {
    let ctx = AudioContext::new()?;

    let osc = ctx.create_oscillator()?;
    osc.set_type(OscillatorType::Sine);

    // Higher pitch for downbeat
    let frequency = if is_downbeat { 1000.0 } else { 800.0 };
    osc.frequency().set_value(frequency);

    let gain = ctx.create_gain()?;
    let volume = if is_downbeat { 0.3 } else { 0.2 };

    let current_time = ctx.current_time();

    // Short percussive envelope
    gain.gain().set_value_at_time(volume, current_time)?;
    gain.gain().exponential_ramp_to_value_at_time(0.001, current_time + 0.05)?;

    osc.connect_with_audio_node(&gain)?;
    gain.connect_with_audio_node(&ctx.destination())?;

    osc.start()?;
    osc.stop_with_when(current_time + 0.06)?;

    Ok(())
}
