# Mozart - Melody Transposition App

## Overview

Mozart is a cross-platform (desktop + iOS) Tauri application for composing melodies and performing intelligent transposition, including diatonic transposition that keeps notes within a specified scale. The music engine is written in Rust for performance and portability.

## Target Platforms

- **Desktop**: macOS, Windows, Linux
- **Mobile**: iOS (primary), Android (future consideration)

## Core Features

### 1. Melody Input

#### Piano Roll / Grid Editor
- Visual grid where X-axis represents time and Y-axis represents pitch
- Click/tap to place notes at specific positions
- Drag to adjust note duration
- Vertical dragging to change pitch
- Touch-optimized for iOS with appropriate hit targets

#### Text Input Mode
- Support for simple text notation: `C4 D4 E4 F4 G4`
- Note format: `<NoteName><Accidental?><Octave><Duration?>`
  - Examples: `C4`, `F#5`, `Bb3q` (Bb octave 3, quarter note)
- Real-time parsing and validation
- Syntax highlighting for note names

### 2. Transposition Engine

#### Chromatic Transposition
- Transpose by any number of semitones (-24 to +24)
- Preserves exact intervals between notes

#### Diatonic Transposition
- Transpose by scale degrees (e.g., "up a third")
- **Supported scales and modes:**
  - Major (Ionian)
  - Natural Minor (Aeolian)
  - Harmonic Minor
  - Melodic Minor
  - Dorian
  - Phrygian
  - Lydian
  - Mixolydian
  - Locrian
- Notes are adjusted to remain within the selected scale
- Key selection: All 12 keys (C through B, with enharmonic options)

#### Transposition Interface
- Source key/scale selection
- Target key/scale selection
- Interval selector for diatonic transposition (unison through octave)
- Preview before applying
- Undo/redo support

### 3. Time Signatures & Accents

#### Supported Time Signatures
- Full range: 2/4 through 15/4 (and corresponding /8 variants)
- **Priority support**: 2/4, 3/4, 4/4, 7/8, 9/8, 11/8
- Compound meters: 6/8, 9/8, 12/8

#### Customizable Accent Patterns
- Visual beat editor showing all beats in the measure
- Three accent levels:
  - **Strong** (downbeat) - highest volume/emphasis
  - **Medium** (secondary accent) - moderate emphasis
  - **Weak** (unaccented) - normal volume
- Preset patterns for common time signatures:
  - 4/4: Strong-Weak-Medium-Weak
  - 3/4: Strong-Weak-Weak
  - 7/8: Strong-Weak-Weak-Medium-Weak-Weak-Weak (3+2+2 or 2+2+3)
  - 9/8: Strong-Weak-Weak-Medium-Weak-Weak-Medium-Weak-Weak
  - 11/8: Configurable groupings (3+3+3+2, 3+3+2+3, etc.)
- Save custom accent patterns as presets

### 4. Playback System

#### Audio Engine (Rust)
- Real-time audio synthesis using `cpal` or `rodio`
- Low-latency playback suitable for mobile
- Sample-based playback with built-in instrument sounds

#### Built-in Instruments
- **Piano**: High-quality piano samples (primary instrument)
- **Strings**: String ensemble pad
- **Synth**: Simple synthesizer tone for previewing

#### Transport Controls
- Play / Pause / Stop
- Loop selection (set loop points, toggle loop)
- Metronome with accent support (follows time signature accents)
- Tempo control: 20-300 BPM
- Tap tempo
- Position indicator / playhead

### 5. Note Values & Rhythm

#### Supported Durations
- Whole note (semibreve)
- Half note (minim)
- Quarter note (crotchet)
- Eighth note (quaver)
- Sixteenth note (semiquaver)
- Dotted variants of all above
- Ties between notes

#### Grid Quantization
- Snap to grid options: 1/4, 1/8, 1/16
- Triplet grid option for swing feel
- Free placement mode (no snap)

### 6. File Management

#### Native Format (.mozart.json)
```json
{
  "version": "1.0",
  "metadata": {
    "title": "My Melody",
    "composer": "User Name",
    "created": "2024-01-15T10:30:00Z",
    "modified": "2024-01-15T14:22:00Z"
  },
  "settings": {
    "tempo": 120,
    "timeSignature": { "numerator": 4, "denominator": 4 },
    "key": { "root": "C", "scale": "major" },
    "accentPattern": [3, 1, 2, 1]
  },
  "notes": [
    { "pitch": 60, "start": 0, "duration": 480, "velocity": 100 },
    { "pitch": 62, "start": 480, "duration": 480, "velocity": 90 }
  ]
}
```

#### MIDI Export
- Standard MIDI File (SMF) Format 0
- Includes tempo, time signature, and key signature
- Compatible with DAWs and notation software

#### File Operations
- New / Open / Save / Save As
- Recent files list
- Auto-save with recovery
- iCloud sync support on iOS/macOS

## Technical Architecture

### Stack

```
┌─────────────────────────────────────────────┐
│              Frontend (Leptos)              │
│         Rust → WASM, Reactive UI            │
├─────────────────────────────────────────────┤
│              Tauri Runtime                  │
│        IPC, Native APIs, File System        │
├─────────────────────────────────────────────┤
│            Music Engine (Rust)              │
│   Transposition, Playback, MIDI Export      │
├─────────────────────────────────────────────┤
│            Audio Backend (Rust)             │
│          cpal/rodio, Sample Player          │
└─────────────────────────────────────────────┘
```

### Rust Crates Structure

```
mozart/
├── src-tauri/           # Tauri backend
│   └── src/
│       ├── main.rs
│       ├── commands.rs  # Tauri commands
│       └── lib.rs
├── mozart-core/         # Music engine library
│   └── src/
│       ├── lib.rs
│       ├── note.rs      # Note representation
│       ├── scale.rs     # Scale definitions
│       ├── transpose.rs # Transposition logic
│       ├── time.rs      # Time signatures, accents
│       └── midi.rs      # MIDI export
├── mozart-audio/        # Audio playback
│   └── src/
│       ├── lib.rs
│       ├── engine.rs    # Audio engine
│       ├── sampler.rs   # Sample playback
│       └── metronome.rs # Metronome
├── mozart-ui/           # Leptos frontend
│   └── src/
│       ├── lib.rs
│       ├── app.rs       # Main app component
│       ├── piano_roll.rs
│       ├── text_input.rs
│       ├── transport.rs
│       └── settings.rs
└── assets/
    └── samples/         # Instrument samples
        ├── piano/
        ├── strings/
        └── synth/
```

### Key Rust Types

```rust
// Note representation (MIDI-compatible)
pub struct Note {
    pub pitch: u8,           // MIDI pitch (0-127)
    pub start_tick: u32,     // Start position in ticks
    pub duration_ticks: u32, // Duration in ticks
    pub velocity: u8,        // 0-127
}

// Scale definition
pub enum ScaleType {
    Major,
    NaturalMinor,
    HarmonicMinor,
    MelodicMinor,
    Dorian,
    Phrygian,
    Lydian,
    Mixolydian,
    Locrian,
}

pub struct Scale {
    pub root: PitchClass,    // C, C#, D, etc.
    pub scale_type: ScaleType,
}

// Time signature with accents
pub struct TimeSignature {
    pub numerator: u8,       // Beats per measure (2-15)
    pub denominator: u8,     // Beat unit (4 or 8)
    pub accents: Vec<AccentLevel>,
}

pub enum AccentLevel {
    Strong,  // 3
    Medium,  // 2
    Weak,    // 1
}

// Transposition request
pub enum TransposeMode {
    Chromatic { semitones: i8 },
    Diatonic {
        source_scale: Scale,
        target_scale: Scale,
        degrees: i8,  // e.g., +2 for "up a third"
    },
}
```

## UI/UX Design

### iOS Native Feel
- Use iOS design conventions (SF Symbols where appropriate)
- Native-feeling gestures (swipe, pinch-to-zoom on piano roll)
- Respect safe areas and notch
- Support Dynamic Type for accessibility
- Dark mode support
- Haptic feedback on note placement

### Layout

#### Main Screen (Portrait - iOS)
```
┌─────────────────────────┐
│  ≡  My Melody    ⚙️ 📁  │  ← Header
├─────────────────────────┤
│ Key: C Major  │ ♩= 120  │  ← Settings bar
├─────────────────────────┤
│                         │
│     Piano Roll Grid     │  ← Main editor
│     (scrollable)        │
│                         │
├─────────────────────────┤
│  [Text Input Field]     │  ← Text input toggle
├─────────────────────────┤
│ ◀◀  ▶  ■  🔁  🎵  ⏱   │  ← Transport
├─────────────────────────┤
│  Transpose  │  Accents  │  ← Tool tabs
└─────────────────────────┘
```

#### Desktop Layout
- Side panel for tools (transpose, accents, settings)
- Larger piano roll with keyboard visible on left
- Toolbar at top
- Transport at bottom

## Implementation Phases

### Phase 1: Foundation
- [ ] Set up Tauri project with iOS support
- [ ] Create mozart-core crate with Note, Scale, TimeSignature types
- [ ] Implement basic chromatic transposition
- [ ] Basic Leptos UI shell

### Phase 2: Core Music Engine
- [ ] Implement all scale types and diatonic transposition
- [ ] Time signature and accent pattern system
- [ ] JSON file format save/load
- [ ] MIDI export

### Phase 3: Audio Playback
- [ ] Audio engine setup with cpal/rodio
- [ ] Sample loading and playback
- [ ] Metronome with accent support
- [ ] Transport controls

### Phase 4: UI Implementation
- [ ] Piano roll editor (desktop)
- [ ] Piano roll editor (iOS optimized)
- [ ] Text input mode
- [ ] Transpose interface
- [ ] Accent pattern editor

### Phase 5: Polish & Platform Features
- [ ] iOS-specific optimizations
- [ ] iCloud sync
- [ ] Auto-save and recovery
- [ ] Dark mode
- [ ] Accessibility features

## Dependencies

### Rust
- `tauri` - Application framework
- `leptos` - Frontend framework
- `cpal` or `rodio` - Audio playback
- `hound` - WAV sample loading
- `midly` - MIDI file writing
- `serde` / `serde_json` - Serialization

### Assets
- Piano samples (public domain or licensed)
- String samples
- Synth samples
- SF Symbols (iOS) or equivalent icons

## Success Criteria

1. User can input a melody via piano roll or text
2. User can transpose chromatically or diatonically within any supported scale
3. User can set any time signature 2-15 with custom accents
4. User can play back the melody with metronome
5. User can save/load .mozart.json files
6. User can export to MIDI
7. App feels native on iOS with responsive touch interactions
8. Audio latency is acceptable for real-time preview (<50ms)
