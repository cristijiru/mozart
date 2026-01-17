// Web Audio API wrapper for Mozart
// Provides oscillator-based synthesis for note playback

export class AudioEngine {
  private ctx: AudioContext | null = null
  private masterGain: GainNode | null = null
  private activeOscillators: Map<string, { osc: OscillatorNode; gain: GainNode }> = new Map()

  async init(): Promise<void> {
    if (this.ctx) return

    this.ctx = new AudioContext()
    this.masterGain = this.ctx.createGain()
    this.masterGain.gain.value = 0.3
    this.masterGain.connect(this.ctx.destination)
  }

  async resume(): Promise<void> {
    if (this.ctx?.state === 'suspended') {
      await this.ctx.resume()
    }
  }

  get context(): AudioContext | null {
    return this.ctx
  }

  get currentTime(): number {
    return this.ctx?.currentTime ?? 0
  }

  // Play a note using oscillator synthesis
  playSineNote(
    frequency: number,
    velocity: number = 100,
    duration: number = 0.5,
    startTime?: number
  ): void {
    if (!this.ctx || !this.masterGain) return

    const start = startTime ?? this.ctx.currentTime
    const gain = this.ctx.createGain()
    const osc = this.ctx.createOscillator()

    // Velocity scaling (0-127 to 0-1)
    const amp = (velocity / 127) * 0.5

    osc.type = 'sine'
    osc.frequency.value = frequency

    // ADSR envelope (simplified)
    const attack = 0.01
    const decay = 0.1
    const sustain = 0.7
    const release = 0.15

    gain.gain.setValueAtTime(0, start)
    gain.gain.linearRampToValueAtTime(amp, start + attack)
    gain.gain.linearRampToValueAtTime(amp * sustain, start + attack + decay)
    gain.gain.setValueAtTime(amp * sustain, start + duration - release)
    gain.gain.linearRampToValueAtTime(0, start + duration)

    osc.connect(gain)
    gain.connect(this.masterGain)

    osc.start(start)
    osc.stop(start + duration + 0.01)
  }

  // Play a note by MIDI number
  playMidiNote(
    midi: number,
    velocity: number = 100,
    duration: number = 0.5,
    startTime?: number,
    voice: number = 0
  ): void {
    const frequency = this.midiToFrequency(midi)
    this.playNote(frequency, velocity, duration, startTime, voice)
  }

  // Play a note with voice-specific timbre
  playNote(
    frequency: number,
    velocity: number = 100,
    duration: number = 0.5,
    startTime?: number,
    voice: number = 0
  ): void {
    if (!this.ctx || !this.masterGain) return

    const start = startTime ?? this.ctx.currentTime
    const gain = this.ctx.createGain()
    const osc = this.ctx.createOscillator()

    // Velocity scaling (0-127 to 0-1)
    const amp = (velocity / 127) * 0.5

    // Different oscillator types for different voices
    const oscillatorTypes: OscillatorType[] = ['sine', 'triangle', 'square', 'sawtooth']
    osc.type = oscillatorTypes[voice % oscillatorTypes.length]

    // Slightly detune harmony voices for richer sound
    if (voice > 0) {
      osc.detune.value = voice * 5 // slight detune per voice
    }

    osc.frequency.value = frequency

    // ADSR envelope (slightly different per voice)
    const attack = voice === 0 ? 0.01 : 0.02
    const decay = 0.1
    const sustain = voice === 0 ? 0.7 : 0.5
    const release = 0.15

    gain.gain.setValueAtTime(0, start)
    gain.gain.linearRampToValueAtTime(amp, start + attack)
    gain.gain.linearRampToValueAtTime(amp * sustain, start + attack + decay)
    gain.gain.setValueAtTime(amp * sustain, start + duration - release)
    gain.gain.linearRampToValueAtTime(0, start + duration)

    osc.connect(gain)
    gain.connect(this.masterGain)

    osc.start(start)
    osc.stop(start + duration + 0.01)
  }

  // Start a note (for sustained playback)
  startNote(id: string, frequency: number, velocity: number = 100): void {
    if (!this.ctx || !this.masterGain) return
    if (this.activeOscillators.has(id)) {
      this.stopNote(id)
    }

    const osc = this.ctx.createOscillator()
    const gain = this.ctx.createGain()

    const amp = (velocity / 127) * 0.5

    osc.type = 'sine'
    osc.frequency.value = frequency

    // Quick attack
    gain.gain.setValueAtTime(0, this.ctx.currentTime)
    gain.gain.linearRampToValueAtTime(amp, this.ctx.currentTime + 0.01)

    osc.connect(gain)
    gain.connect(this.masterGain)
    osc.start()

    this.activeOscillators.set(id, { osc, gain })
  }

  // Stop a sustained note
  stopNote(id: string): void {
    const entry = this.activeOscillators.get(id)
    if (!entry || !this.ctx) return

    const { osc, gain } = entry

    // Quick release
    gain.gain.linearRampToValueAtTime(0, this.ctx.currentTime + 0.05)
    osc.stop(this.ctx.currentTime + 0.1)

    this.activeOscillators.delete(id)
  }

  // Play metronome click
  playClick(isDownbeat: boolean = false): void {
    if (!this.ctx || !this.masterGain) return

    const osc = this.ctx.createOscillator()
    const gain = this.ctx.createGain()

    osc.type = 'sine'
    osc.frequency.value = isDownbeat ? 1000 : 800

    const amp = isDownbeat ? 0.3 : 0.2
    const duration = 0.05

    gain.gain.setValueAtTime(amp, this.ctx.currentTime)
    gain.gain.exponentialRampToValueAtTime(0.001, this.ctx.currentTime + duration)

    osc.connect(gain)
    gain.connect(this.masterGain)

    osc.start()
    osc.stop(this.ctx.currentTime + duration)
  }

  // Play click with custom frequency and velocity
  playClickWithParams(frequency: number, velocity: number): void {
    if (!this.ctx || !this.masterGain) return

    const osc = this.ctx.createOscillator()
    const gain = this.ctx.createGain()

    osc.type = 'sine'
    osc.frequency.value = frequency

    const amp = (velocity / 127) * 0.4
    const duration = 0.05

    gain.gain.setValueAtTime(amp, this.ctx.currentTime)
    gain.gain.exponentialRampToValueAtTime(0.001, this.ctx.currentTime + duration)

    osc.connect(gain)
    gain.connect(this.masterGain)

    osc.start()
    osc.stop(this.ctx.currentTime + duration)
  }

  // Convert MIDI note number to frequency
  midiToFrequency(midi: number): number {
    return 440 * Math.pow(2, (midi - 69) / 12)
  }

  // Set master volume (0-1)
  setVolume(volume: number): void {
    if (this.masterGain) {
      this.masterGain.gain.value = Math.max(0, Math.min(1, volume))
    }
  }

  dispose(): void {
    // Stop all active oscillators
    for (const [id] of this.activeOscillators) {
      this.stopNote(id)
    }

    if (this.ctx) {
      this.ctx.close()
      this.ctx = null
      this.masterGain = null
    }
  }
}

// Singleton instance
let audioEngine: AudioEngine | null = null

export function getAudioEngine(): AudioEngine {
  if (!audioEngine) {
    audioEngine = new AudioEngine()
  }
  return audioEngine
}
