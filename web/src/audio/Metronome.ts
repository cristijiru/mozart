// Metronome for beat-synced click playback

import { AudioEngine } from './AudioEngine'

export interface MetronomeOptions {
  tempo: number
  beatsPerMeasure: number
  accents?: number[] // 1=weak, 2=medium, 3=strong
  onBeat?: (beat: number, isDownbeat: boolean) => void
}

export class Metronome {
  private audioEngine: AudioEngine
  private tempo: number
  private beatsPerMeasure: number
  private accents: number[]
  private isPlaying: boolean = false
  private currentBeat: number = 0
  private nextBeatTime: number = 0
  private timerId: number | null = null
  private lookahead: number = 25 // ms
  private scheduleAhead: number = 0.1 // seconds

  private onBeat?: (beat: number, isDownbeat: boolean) => void

  constructor(audioEngine: AudioEngine, options: MetronomeOptions) {
    this.audioEngine = audioEngine
    this.tempo = options.tempo
    this.beatsPerMeasure = options.beatsPerMeasure
    this.accents = options.accents ?? this.defaultAccents(options.beatsPerMeasure)
    this.onBeat = options.onBeat
  }

  private defaultAccents(beats: number): number[] {
    const accents = new Array(beats).fill(1)
    accents[0] = 3 // Downbeat is strong
    if (beats === 4) accents[2] = 2 // Beat 3 is medium in 4/4
    return accents
  }

  setTempo(tempo: number): void {
    this.tempo = Math.max(20, Math.min(300, tempo))
  }

  setBeatsPerMeasure(beats: number): void {
    this.beatsPerMeasure = beats
    if (this.accents.length !== beats) {
      this.accents = this.defaultAccents(beats)
    }
  }

  setAccents(accents: number[]): void {
    this.accents = accents
  }

  start(): void {
    if (this.isPlaying) return

    this.audioEngine.resume()

    this.isPlaying = true
    this.currentBeat = 0
    this.nextBeatTime = this.audioEngine.currentTime

    this.schedule()
  }

  stop(): void {
    this.isPlaying = false
    if (this.timerId !== null) {
      clearTimeout(this.timerId)
      this.timerId = null
    }
  }

  toggle(): void {
    if (this.isPlaying) {
      this.stop()
    } else {
      this.start()
    }
  }

  private schedule = (): void => {
    if (!this.isPlaying) return

    const ctx = this.audioEngine.context
    if (!ctx) return

    // Schedule beats that fall within the schedule window
    while (this.nextBeatTime < ctx.currentTime + this.scheduleAhead) {
      this.scheduleBeat(this.currentBeat, this.nextBeatTime)

      // Advance beat
      const secondsPerBeat = 60 / this.tempo
      this.nextBeatTime += secondsPerBeat
      this.currentBeat = (this.currentBeat + 1) % this.beatsPerMeasure
    }

    // Schedule next check
    this.timerId = window.setTimeout(this.schedule, this.lookahead)
  }

  private scheduleBeat(beat: number, time: number): void {
    const isDownbeat = beat === 0
    const accent = this.accents[beat] ?? 1

    // Map accent level to frequency and velocity
    let frequency: number
    let velocity: number

    switch (accent) {
      case 3: // Strong
        frequency = 1000
        velocity = 100
        break
      case 2: // Medium
        frequency = 900
        velocity = 80
        break
      default: // Weak
        frequency = 800
        velocity = 60
    }

    // Schedule the click
    // Note: We're scheduling slightly in the future, so we need to use
    // a timeout to actually trigger the sound at the right time
    const delay = Math.max(0, (time - this.audioEngine.currentTime) * 1000)

    setTimeout(() => {
      this.audioEngine.playClickWithParams(frequency, velocity)
      this.onBeat?.(beat, isDownbeat)
    }, delay)
  }

  dispose(): void {
    this.stop()
  }
}
