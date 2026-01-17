// Sequencer for scheduled note playback
// Handles timing and scheduling of notes for playback

import { AudioEngine } from './AudioEngine'
import type { Note } from '../wasm/types'

export type SequencerState = 'stopped' | 'playing' | 'paused'

export interface SequencerOptions {
  tempo: number
  ticksPerBeat: number
  onTick?: (tick: number) => void
  onBeat?: (beat: number, isDownbeat: boolean) => void
  onNotePlay?: (note: Note) => void
  onStateChange?: (state: SequencerState) => void
}

export class Sequencer {
  private audioEngine: AudioEngine
  private notes: Note[] = []
  private tempo: number
  private ticksPerBeat: number
  private _ticksPerMeasure: number = 1920
  private currentTick: number = 0
  private state: SequencerState = 'stopped'
  private startTime: number = 0
  private pausedTick: number = 0
  private scheduledNotes: Set<number> = new Set()
  private animationFrameId: number | null = null
  private lookaheadTime: number = 0.1 // seconds to look ahead for scheduling

  // Callbacks
  private onTick?: (tick: number) => void
  private _onBeat?: (beat: number, isDownbeat: boolean) => void
  private onNotePlay?: (note: Note) => void
  private onStateChange?: (state: SequencerState) => void

  constructor(audioEngine: AudioEngine, options: SequencerOptions) {
    this.audioEngine = audioEngine
    this.tempo = options.tempo
    this.ticksPerBeat = options.ticksPerBeat
    this.onTick = options.onTick
    this._onBeat = options.onBeat
    this.onNotePlay = options.onNotePlay
    this.onStateChange = options.onStateChange
  }

  setNotes(notes: Note[]): void {
    this.notes = [...notes]
  }

  setTempo(tempo: number): void {
    this.tempo = Math.max(20, Math.min(300, tempo))
  }

  setTicksPerBeat(ticks: number): void {
    this.ticksPerBeat = ticks
  }

  setTicksPerMeasure(ticks: number): void {
    this._ticksPerMeasure = ticks
  }

  getState(): SequencerState {
    return this.state
  }

  getCurrentTick(): number {
    return this.currentTick
  }

  play(): void {
    if (this.state === 'playing') return

    this.audioEngine.resume()

    // Start from current position (pausedTick holds the seek position)
    this.startTime = this.audioEngine.currentTime - this.tickToSeconds(this.pausedTick)
    this.currentTick = this.pausedTick

    if (this.state === 'stopped') {
      this.scheduledNotes.clear()
    }

    this.state = 'playing'
    this.onStateChange?.(this.state)
    this.scheduleLoop()
  }

  pause(): void {
    if (this.state !== 'playing') return

    this.pausedTick = this.currentTick
    this.state = 'paused'
    this.onStateChange?.(this.state)

    if (this.animationFrameId !== null) {
      cancelAnimationFrame(this.animationFrameId)
      this.animationFrameId = null
    }
  }

  stop(): void {
    this.state = 'stopped'
    this.currentTick = 0
    this.pausedTick = 0
    this.scheduledNotes.clear()
    this.onStateChange?.(this.state)

    if (this.animationFrameId !== null) {
      cancelAnimationFrame(this.animationFrameId)
      this.animationFrameId = null
    }
  }

  seekTo(tick: number): void {
    this.currentTick = Math.max(0, tick)
    this.pausedTick = this.currentTick
    this.scheduledNotes.clear()

    if (this.state === 'playing') {
      this.startTime = this.audioEngine.currentTime - this.tickToSeconds(this.currentTick)
    }
  }

  private scheduleLoop = (): void => {
    if (this.state !== 'playing') return

    const ctx = this.audioEngine.context
    if (!ctx) return

    const elapsedTime = ctx.currentTime - this.startTime
    this.currentTick = this.secondsToTick(elapsedTime)

    // Report current tick
    this.onTick?.(this.currentTick)

    // Schedule notes within lookahead window
    const lookaheadTick = this.secondsToTick(elapsedTime + this.lookaheadTime)

    for (let i = 0; i < this.notes.length; i++) {
      const note = this.notes[i]

      // Skip if already scheduled
      if (this.scheduledNotes.has(i)) continue

      // Skip if note starts after lookahead
      if (note.start_tick > lookaheadTick) continue

      // Skip if note already passed
      if (note.start_tick + note.duration_ticks < this.currentTick) {
        this.scheduledNotes.add(i)
        continue
      }

      // Schedule the note
      const noteStartTime = this.startTime + this.tickToSeconds(note.start_tick)
      const noteDuration = this.tickToSeconds(note.duration_ticks)

      this.audioEngine.playMidiNote(
        note.pitch,
        note.velocity,
        noteDuration,
        noteStartTime,
        note.voice ?? 0
      )

      this.scheduledNotes.add(i)
      this.onNotePlay?.(note)
    }

    // Check if we've passed all notes
    const maxTick = this.notes.reduce(
      (max, n) => Math.max(max, n.start_tick + n.duration_ticks),
      0
    )

    if (this.currentTick > maxTick + this.ticksPerBeat) {
      this.stop()
      return
    }

    this.animationFrameId = requestAnimationFrame(this.scheduleLoop)
  }

  private tickToSeconds(tick: number): number {
    const beatsPerSecond = this.tempo / 60
    const ticksPerSecond = beatsPerSecond * this.ticksPerBeat
    return tick / ticksPerSecond
  }

  private secondsToTick(seconds: number): number {
    const beatsPerSecond = this.tempo / 60
    const ticksPerSecond = beatsPerSecond * this.ticksPerBeat
    return Math.floor(seconds * ticksPerSecond)
  }

  dispose(): void {
    this.stop()
  }
}
