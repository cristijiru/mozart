# Mozart

A music composition tool with a Rust core and React frontend via WebAssembly.

## Project Structure

```
mozart/
├── crates/
│   └── mozart-core/          # Rust music engine
│       ├── src/
│       │   ├── lib.rs        # Library exports
│       │   ├── note.rs       # Note representation
│       │   ├── pitch.rs      # Pitch classes and MIDI
│       │   ├── scale.rs      # Scales and modes
│       │   ├── time.rs       # Time signatures and accents
│       │   ├── transpose.rs  # Chromatic/diatonic transposition
│       │   ├── song.rs       # Song structure and serialization
│       │   ├── midi.rs       # MIDI export
│       │   ├── error.rs      # Error types
│       │   └── wasm.rs       # WebAssembly bindings
│       └── Cargo.toml
├── web/                      # React frontend
│   ├── src/
│   │   ├── main.tsx
│   │   ├── App.tsx
│   │   ├── wasm/             # WASM loader and types
│   │   ├── audio/            # Web Audio API (AudioEngine, Sequencer, Metronome)
│   │   ├── components/       # React components
│   │   └── store/            # Zustand state management
│   ├── package.json
│   ├── vite.config.ts
│   └── index.html
├── build-wasm.sh             # WASM build script
├── spec.md                   # Feature specification
└── Cargo.toml                # Workspace config
```

## Features

- **Music Theory Engine**: Notes, scales (major, minor, modes), time signatures
- **Transposition**: Chromatic (by semitones) and diatonic (by scale degrees)
- **Custom Accents**: Editable accent patterns for any time signature (2-15 beats)
- **Text Notation**: Parse melodies like `C4q D4q E4h` (pitch + duration)
- **Piano Roll**: Visual note editing with playback
- **MIDI Export**: Export songs to Standard MIDI Format
- **Web Audio**: Oscillator-based synthesis for previews

## Quick Start with Docker

```bash
# Production build and run
docker compose up mozart

# Open http://localhost:8080
```

For development with hot reload:

```bash
docker compose --profile dev up mozart-dev

# Open http://localhost:5173
```

## Local Development

### Prerequisites

- Rust (with `wasm32-unknown-unknown` target)
- wasm-pack
- Node.js 18+

### Setup

```bash
# Install wasm-pack
cargo install wasm-pack

# Add WASM target
rustup target add wasm32-unknown-unknown

# Install web dependencies
cd web && npm install
```

### Build WASM

```bash
./build-wasm.sh
```

This builds the Rust core to WebAssembly and outputs to `web/src/wasm/pkg/`.

### Run Development Server

```bash
cd web
npm run dev
```

Open http://localhost:5173

### Run Tests

```bash
# Rust tests
cargo test

# TypeScript check
cd web && npx tsc --noEmit
```

## Usage

### Text Notation Format

```
[Note][Octave][Duration]

Notes: C, C#, D, Eb, E, F, F#, G, Ab, A, Bb, B
Octaves: 0-8 (C4 = middle C)
Durations: w=whole, h=half, q=quarter, e=eighth, s=sixteenth
Dotted: Add . (e.g., q. for dotted quarter)
Rests: R followed by duration (e.g., Rq)

Example: C4q D4q E4q F4q G4h
```

### Keyboard Shortcuts

- Click on piano roll to add notes
- Shift+click to delete notes
- Click piano keys to preview notes

## Architecture

The app uses a hybrid architecture:

1. **Rust Core** (`mozart-core`): All music logic, serialization, MIDI export
2. **WASM Bridge**: wasm-bindgen exports for JavaScript
3. **React Frontend**: UI components, state management (Zustand)
4. **Web Audio**: Synthesis and playback (not in WASM due to audio API limitations)

This separation keeps the core logic testable and portable while leveraging web platform audio capabilities.

## License

MIT
