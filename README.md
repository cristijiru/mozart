# Mozart

A cross-platform melody transposition app built with Tauri and Rust.

## Features

- **Melody Input**: Piano roll grid editor and text notation input
- **Transposition**:
  - Chromatic transposition by semitones
  - Diatonic transposition that keeps notes within the scale
  - Key change with transposition
- **Time Signatures**: Full support for 2-15 beats per measure with customizable accent patterns
- **Audio Playback**: Built-in instrument samples (piano, strings, synth) with metronome
- **File Formats**: Native JSON format (.mozart.json) and MIDI export

## Project Structure

```
mozart/
├── crates/
│   ├── mozart-core/     # Music theory engine
│   │   ├── note.rs      # Note representation
│   │   ├── pitch.rs     # Pitch classes and absolute pitches
│   │   ├── scale.rs     # Scale definitions (major, minor, modes)
│   │   ├── time.rs      # Time signatures and accents
│   │   ├── transpose.rs # Transposition engine
│   │   ├── song.rs      # Song format
│   │   └── midi.rs      # MIDI export
│   ├── mozart-audio/    # Audio playback engine
│   │   ├── sampler.rs   # Sample-based playback
│   │   ├── metronome.rs # Metronome with accents
│   │   └── engine.rs    # Transport controls
│   └── mozart-ui/       # Leptos frontend (WASM)
├── src-tauri/           # Tauri backend
├── assets/samples/      # Instrument samples
└── spec.md              # Full specification
```

## Development

### Prerequisites

- Rust 1.75+
- Node.js 18+ (for Tauri)
- Xcode (for iOS development)

### Build

```bash
# Build all crates
cargo build

# Run tests
cargo test

# Run the test CLI
cargo run --bin mozart-test
```

### Test CLI

The `mozart-test` CLI lets you test the music engine interactively:

```
$ cargo run --bin mozart-test

mozart> melody C4q D4q E4q F4q G4h
Melody set: 5 notes

mozart> transpose diatonic 2
Transposing: Diatonic up a 3rd in C Major
Transposed 5 notes
New melody: E4q F4q G4q A4q B4h

mozart> key D major
Key set to D Major

mozart> transpose diatonic 2
Transposing: Diatonic up a 3rd in D Major
Transposed 5 notes
New melody: F#4q G4q A4q B4q C#5h
```

### Running the App

```bash
# Desktop
cargo tauri dev

# iOS (requires Xcode)
cargo tauri ios dev
```

## Notation Format

Notes are written as `<Pitch><Octave><Duration>`:

- **Pitches**: C, D, E, F, G, A, B (with # or b for accidentals)
- **Octaves**: 0-9 (C4 = middle C)
- **Durations**: w (whole), h (half), q (quarter), e (eighth), s (sixteenth)
- **Dotted**: Add `.` for dotted notes (e.g., `q.` = dotted quarter)
- **Rests**: R followed by duration (e.g., `Rq` = quarter rest)

Example: `C4q D4q E4q. F4e G4h Rq A4q B4q C5w`

## Time Signatures

Supports time signatures from 2/4 to 15/4 (and corresponding /8 variants).

Custom accent patterns use three levels:
- **Strong (>)**: Downbeat emphasis
- **Medium (-)**: Secondary accent
- **Weak (.)**: Normal beat

Example: 7/8 as 3+2+2 = `>..-.-.`

## License

MIT
