// TypeScript types matching the Rust WASM exports

export interface Note {
  pitch: number
  start_tick: number
  duration_ticks: number
  velocity: number
}

export interface Mozart {
  // Metadata
  title: string
  composer: string

  // Settings
  tempo: number
  getTimeSignature(): string
  setTimeSignature(ts: string): void
  getTimeSignatureNumerator(): number
  getTimeSignatureDenominator(): number
  getKey(): string
  setKey(key: string): void

  // Notes
  noteCount(): number
  addNote(pitch: number, startTick: number, durationTicks: number): void
  addNoteWithVelocity(pitch: number, startTick: number, durationTicks: number, velocity: number): void
  removeNote(index: number): boolean
  clearNotes(): void
  getNotesJson(): string
  getNoteJson(index: number): string | undefined

  // Melody
  parseMelody(melody: string): number
  formatMelody(): string

  // Transposition
  transposeChromatic(semitones: number): void
  transposeDiatonic(degrees: number): void
  transposeDiatonicWithKeyChange(targetKey: string, degrees: number): void
  invert(pivot: number): void

  // Accents
  getAccents(): Uint8Array
  setAccents(accents: Uint8Array | number[]): void
  cycleAccent(beat: number): void
  getAccentVisual(): string

  // Duration info
  durationTicks(): number
  durationSeconds(): number
  measureCount(): number
  ticksPerBeat(): number
  ticksPerMeasure(): number

  // Serialization
  toJson(): string
  toMidi(): Uint8Array

  // Static utility methods (on the class, not instance)
  // These are accessed via the module, not the instance
}

// Utility function types
export interface MozartStatic {
  new(): Mozart
  withTitle(title: string): Mozart
  fromJson(json: string): Mozart
  midiToFrequency(midi: number): number
  midiToNoteName(midi: number): string
  noteNameToMidi(name: string): number
}

// Constants
export const TICKS_PER_QUARTER = 480
