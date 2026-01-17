//! Mozart Demo - Demonstrates all core features

use mozart_core::*;

fn main() {
    init_logging();

    println!("═══════════════════════════════════════════════════════");
    println!("                    MOZART DEMO");
    println!("═══════════════════════════════════════════════════════\n");

    // 1. Create a song
    println!("▶ Creating a new song...");
    let mut song = Song::with_title("Demo Melody");
    song.set_tempo(120);
    song.set_key(scale::Scale::c_major());

    // 2. Parse a melody
    println!("▶ Parsing melody: C4q D4q E4q F4q G4h");
    let notes = note::parse_melody("C4q D4q E4q F4q G4h").unwrap();
    song.add_notes(notes);

    println!("  Notes: {}", note::format_melody(&song.notes));
    println!("  Duration: {:.2}s", song.duration_seconds());
    println!();

    // 3. Chromatic transposition
    println!("▶ Chromatic transpose UP 4 semitones (major 3rd):");
    let mode = transpose::TransposeMode::chromatic(4);
    let transposed = transpose::transpose_notes(&song.notes, &mode).unwrap();
    println!("  Before: {}", note::format_melody(&song.notes));
    println!("  After:  {}", note::format_melody(&transposed));
    println!();

    // 4. Diatonic transposition (the key feature!)
    println!("▶ Diatonic transpose UP 2 degrees (a 3rd) in C Major:");
    println!("  This keeps all notes within the C major scale!");
    let mode = transpose::TransposeMode::diatonic(scale::Scale::c_major(), 2);
    let diatonic = transpose::transpose_notes(&song.notes, &mode).unwrap();
    println!("  Before: {}", note::format_melody(&song.notes));
    println!("  After:  {}", note::format_melody(&diatonic));
    println!("  Notice: C→E (major 3rd), D→F (minor 3rd), E→G (minor 3rd)...");
    println!();

    // 5. Different scales
    println!("▶ Available scales:");
    for scale_type in scale::ScaleType::all() {
        let scale = scale::Scale::new(pitch::PitchClass::C, *scale_type);
        let notes: Vec<_> = scale.pitch_classes().iter().map(|p| p.to_string()).collect();
        println!("  {}: {}", scale_type.name(), notes.join(" "));
    }
    println!();

    // 6. Time signatures with accents
    println!("▶ Time signature examples:");
    for num in [3, 4, 5, 7, 9, 11] {
        let ts = time::TimeSignature::new(num, 8).unwrap();
        println!("  {}/8: {}", num, ts.accents);
    }
    println!();

    // 7. Complex example: Diatonic transposition with key change
    println!("▶ Diatonic transpose from C Major to G Major:");
    let c_major = scale::Scale::c_major();
    let g_major = scale::Scale::new(pitch::PitchClass::G, scale::ScaleType::Major);
    let mode = transpose::TransposeMode::diatonic_with_key_change(c_major, g_major, 0);

    let melody = note::parse_melody("C4q E4q G4q").unwrap(); // C major triad
    let transposed = transpose::transpose_notes(&melody, &mode).unwrap();
    println!("  C Major triad: {}", note::format_melody(&melody));
    println!("  G Major triad: {}", note::format_melody(&transposed));
    println!();

    // 8. Export to JSON
    println!("▶ Song JSON format:");
    song.notes = diatonic; // Use the diatonically transposed melody
    let json = song.to_json().unwrap();
    // Just show first few lines
    for line in json.lines().take(15) {
        println!("  {}", line);
    }
    println!("  ...");
    println!();

    // 9. MIDI export
    println!("▶ MIDI export:");
    let midi_data = midi::export_to_midi(&song).unwrap();
    println!("  Generated {} bytes of MIDI data", midi_data.len());
    println!("  (Use 'cargo run --bin mozart-test' then 'midi filename' to save)");
    println!();

    println!("═══════════════════════════════════════════════════════");
    println!("  Run 'cargo run --bin mozart-test' for interactive mode");
    println!("═══════════════════════════════════════════════════════");
}
