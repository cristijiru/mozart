import { create } from 'zustand'
import type { Mozart, Note } from '../wasm/types'
import { createMozart, initWasm, loadMozartFromJson } from '../wasm'
import { AudioEngine, Sequencer, Metronome } from '../audio'

export type PlaybackState = 'stopped' | 'playing' | 'paused'

interface MozartState {
  // WASM instance
  mozart: Mozart | null
  isWasmLoaded: boolean

  // Audio
  audioEngine: AudioEngine | null
  sequencer: Sequencer | null
  metronome: Metronome | null

  // Playback state
  playbackState: PlaybackState
  currentTick: number
  isMetronomeEnabled: boolean

  // UI state
  selectedNoteIndex: number | null
  gridDivision: number // 4 = quarter notes, 8 = eighth notes, etc.
  viewportStart: number // Start tick of the visible area
  viewportEnd: number // End tick of the visible area

  // Derived state (cached for performance)
  notes: Note[]
  tempo: number
  timeSignature: { numerator: number; denominator: number }
  key: string
  accents: number[]

  // Actions
  init: () => Promise<void>

  // Song actions
  newSong: (title?: string) => void
  loadFromJson: (json: string) => void
  saveToJson: () => string | null
  exportToMidi: () => Uint8Array | null

  // Note actions
  addNote: (pitch: number, startTick: number, durationTicks: number, velocity?: number) => void
  removeNote: (index: number) => void
  clearNotes: () => void
  parseMelody: (melody: string) => number
  formatMelody: () => string
  selectNote: (index: number | null) => void

  // Settings actions
  setTempo: (tempo: number) => void
  setTimeSignature: (ts: string) => void
  setKey: (key: string) => void
  setAccents: (accents: number[]) => void
  cycleAccent: (beat: number) => void

  // Transposition
  transposeChromatic: (semitones: number, keepOriginal?: boolean) => void
  transposeDiatonic: (degrees: number, keepOriginal?: boolean) => void

  // Playback actions
  play: () => void
  pause: () => void
  stop: () => void
  seekTo: (tick: number) => void
  toggleMetronome: () => void

  // Preview
  playNotePreview: (pitch: number, velocity?: number, duration?: number) => void

  // UI actions
  setGridDivision: (division: number) => void
  setViewport: (start: number, end: number) => void

  // Sync state from WASM
  syncFromWasm: () => void
}

export const useMozartStore = create<MozartState>((set, get) => ({
  // Initial state
  mozart: null,
  isWasmLoaded: false,
  audioEngine: null,
  sequencer: null,
  metronome: null,
  playbackState: 'stopped',
  currentTick: 0,
  isMetronomeEnabled: false,
  selectedNoteIndex: null,
  gridDivision: 4,
  viewportStart: 0,
  viewportEnd: 1920 * 4, // 4 measures at 4/4

  notes: [],
  tempo: 120,
  timeSignature: { numerator: 4, denominator: 4 },
  key: 'C Major',
  accents: [3, 1, 2, 1],

  // Initialize WASM and audio
  init: async () => {
    try {
      await initWasm()
      const mozart = createMozart()
      const audioEngine = new AudioEngine()
      await audioEngine.init()

      const sequencer = new Sequencer(audioEngine, {
        tempo: 120,
        ticksPerBeat: 480,
        onTick: (tick) => set({ currentTick: tick }),
        onStateChange: (playbackState) => set({ playbackState }),
      })

      const metronome = new Metronome(audioEngine, {
        tempo: 120,
        beatsPerMeasure: 4,
      })

      set({
        mozart,
        isWasmLoaded: true,
        audioEngine,
        sequencer,
        metronome,
      })

      // Sync initial state
      get().syncFromWasm()
    } catch (err) {
      console.error('Failed to initialize:', err)
    }
  },

  // Song actions
  newSong: (title = 'Untitled') => {
    const { mozart } = get()
    if (!mozart) return

    const newMozart = createMozart()
    newMozart.title = title
    set({ mozart: newMozart, selectedNoteIndex: null })
    get().syncFromWasm()
  },

  loadFromJson: (json: string) => {
    try {
      const mozart = loadMozartFromJson(json)
      set({ mozart, selectedNoteIndex: null })
      get().syncFromWasm()
    } catch (err) {
      console.error('Failed to load song:', err)
    }
  },

  saveToJson: () => {
    const { mozart } = get()
    if (!mozart) return null
    try {
      return mozart.toJson()
    } catch (err) {
      console.error('Failed to save song:', err)
      return null
    }
  },

  exportToMidi: () => {
    const { mozart } = get()
    if (!mozart) return null
    try {
      return mozart.toMidi()
    } catch (err) {
      console.error('Failed to export MIDI:', err)
      return null
    }
  },

  // Note actions
  addNote: (pitch, startTick, durationTicks, velocity = 100) => {
    const { mozart } = get()
    if (!mozart) return

    mozart.addNoteWithVelocity(pitch, startTick, durationTicks, velocity)
    get().syncFromWasm()
  },

  removeNote: (index) => {
    const { mozart, selectedNoteIndex } = get()
    if (!mozart) return

    mozart.removeNote(index)
    if (selectedNoteIndex === index) {
      set({ selectedNoteIndex: null })
    }
    get().syncFromWasm()
  },

  clearNotes: () => {
    const { mozart } = get()
    if (!mozart) return

    mozart.clearNotes()
    set({ selectedNoteIndex: null })
    get().syncFromWasm()
  },

  parseMelody: (melody) => {
    const { mozart } = get()
    if (!mozart) return 0

    try {
      const count = mozart.parseMelody(melody)
      get().syncFromWasm()
      return count
    } catch (err) {
      console.error('Failed to parse melody:', err)
      return 0
    }
  },

  formatMelody: () => {
    const { mozart } = get()
    if (!mozart) return ''

    try {
      return mozart.formatMelody()
    } catch (err) {
      console.error('Failed to format melody:', err)
      return ''
    }
  },

  selectNote: (index) => {
    set({ selectedNoteIndex: index })
  },

  // Settings actions
  setTempo: (tempo) => {
    const { mozart, sequencer, metronome } = get()
    if (!mozart) return

    mozart.tempo = tempo
    sequencer?.setTempo(tempo)
    metronome?.setTempo(tempo)
    set({ tempo })
  },

  setTimeSignature: (ts) => {
    const { mozart, sequencer, metronome } = get()
    if (!mozart) return

    try {
      mozart.setTimeSignature(ts)
      get().syncFromWasm()

      const numerator = mozart.getTimeSignatureNumerator()
      sequencer?.setTicksPerMeasure(mozart.ticksPerMeasure())
      metronome?.setBeatsPerMeasure(numerator)
    } catch (err) {
      console.error('Failed to set time signature:', err)
    }
  },

  setKey: (key) => {
    const { mozart } = get()
    if (!mozart) return

    try {
      mozart.setKey(key)
      set({ key: mozart.getKey() })
    } catch (err) {
      console.error('Failed to set key:', err)
    }
  },

  setAccents: (accents) => {
    const { mozart, metronome } = get()
    if (!mozart) return

    try {
      mozart.setAccents(new Uint8Array(accents))
      metronome?.setAccents(accents)
      set({ accents })
    } catch (err) {
      console.error('Failed to set accents:', err)
    }
  },

  cycleAccent: (beat) => {
    const { mozart, metronome } = get()
    if (!mozart) return

    mozart.cycleAccent(beat)
    const newAccents = Array.from(mozart.getAccents())
    metronome?.setAccents(newAccents)
    set({ accents: newAccents })
  },

  // Transposition
  transposeChromatic: (semitones, keepOriginal = false) => {
    const { mozart, notes } = get()
    if (!mozart) return

    try {
      // Save original notes if keeping them
      const originalNotes = keepOriginal ? [...notes] : []

      mozart.transposeChromatic(semitones)

      // Add back original notes
      if (keepOriginal) {
        for (const note of originalNotes) {
          mozart.addNoteWithVelocity(note.pitch, note.start_tick, note.duration_ticks, note.velocity)
        }
      }

      get().syncFromWasm()
    } catch (err) {
      console.error('Failed to transpose:', err)
    }
  },

  transposeDiatonic: (degrees, keepOriginal = false) => {
    const { mozart, notes } = get()
    if (!mozart) return

    try {
      // Save original notes if keeping them
      const originalNotes = keepOriginal ? [...notes] : []

      mozart.transposeDiatonic(degrees)

      // Add back original notes
      if (keepOriginal) {
        for (const note of originalNotes) {
          mozart.addNoteWithVelocity(note.pitch, note.start_tick, note.duration_ticks, note.velocity)
        }
      }

      get().syncFromWasm()
    } catch (err) {
      console.error('Failed to transpose:', err)
    }
  },

  // Playback actions
  play: () => {
    const { sequencer, metronome, isMetronomeEnabled, notes } = get()
    if (!sequencer) return

    sequencer.setNotes(notes)
    sequencer.play()

    if (isMetronomeEnabled) {
      metronome?.start()
    }
  },

  pause: () => {
    const { sequencer, metronome } = get()
    sequencer?.pause()
    metronome?.stop()
  },

  stop: () => {
    const { sequencer, metronome } = get()
    sequencer?.stop()
    metronome?.stop()
    set({ currentTick: 0 })
  },

  seekTo: (tick) => {
    const { sequencer } = get()
    sequencer?.seekTo(tick)
    set({ currentTick: tick })
  },

  toggleMetronome: () => {
    const { isMetronomeEnabled, metronome, playbackState } = get()
    const newEnabled = !isMetronomeEnabled

    if (playbackState === 'playing') {
      if (newEnabled) {
        metronome?.start()
      } else {
        metronome?.stop()
      }
    }

    set({ isMetronomeEnabled: newEnabled })
  },

  // Preview
  playNotePreview: (pitch, velocity = 100, duration = 0.3) => {
    const { audioEngine } = get()
    if (!audioEngine) return

    audioEngine.playMidiNote(pitch, velocity, duration)
  },

  // UI actions
  setGridDivision: (division) => {
    set({ gridDivision: division })
  },

  setViewport: (start, end) => {
    set({ viewportStart: start, viewportEnd: end })
  },

  // Sync state from WASM
  syncFromWasm: () => {
    const { mozart, sequencer } = get()
    if (!mozart) return

    try {
      const notesJson = mozart.getNotesJson()
      const notes: Note[] = JSON.parse(notesJson)

      const numerator = mozart.getTimeSignatureNumerator()
      const denominator = mozart.getTimeSignatureDenominator()
      const accents = Array.from(mozart.getAccents())

      sequencer?.setNotes(notes)
      sequencer?.setTicksPerBeat(mozart.ticksPerBeat())
      sequencer?.setTicksPerMeasure(mozart.ticksPerMeasure())

      set({
        notes,
        tempo: mozart.tempo,
        timeSignature: { numerator, denominator },
        key: mozart.getKey(),
        accents,
      })
    } catch (err) {
      console.error('Failed to sync from WASM:', err)
    }
  },
}))
