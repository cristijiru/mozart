// WASM loader for Mozart Core
// This module loads and initializes the WASM package

import type { Mozart } from './types'

// eslint-disable-next-line @typescript-eslint/no-explicit-any
let wasmModule: any = null
let initialized = false

export async function initWasm(): Promise<void> {
  if (initialized) return

  try {
    // Import the WASM module
    wasmModule = await import('./pkg/mozart_core')

    // Call the default export to initialize WASM
    // This loads and instantiates the .wasm file
    await wasmModule.default()

    initialized = true
  } catch (err) {
    console.error('Failed to load WASM module:', err)
    throw err
  }
}

export function getMozartClass(): new () => Mozart {
  if (!initialized) {
    throw new Error('WASM not initialized. Call initWasm() first.')
  }
  return wasmModule.Mozart
}

export function createMozart(): Mozart {
  const MozartClass = getMozartClass()
  return new MozartClass()
}

export function createMozartWithTitle(title: string): Mozart {
  if (!initialized) {
    throw new Error('WASM not initialized. Call initWasm() first.')
  }
  return wasmModule.Mozart.withTitle(title)
}

export function loadMozartFromJson(json: string): Mozart {
  if (!initialized) {
    throw new Error('WASM not initialized. Call initWasm() first.')
  }
  return wasmModule.Mozart.fromJson(json)
}

// Re-export utility functions
export function getTicksPerQuarter(): number {
  if (!initialized) {
    throw new Error('WASM not initialized. Call initWasm() first.')
  }
  return wasmModule.TICKS_PER_QUARTER()
}

export function getScaleTypes(): string[] {
  if (!initialized) {
    throw new Error('WASM not initialized. Call initWasm() first.')
  }
  return JSON.parse(wasmModule.getScaleTypes())
}

export function getPitchClasses(): string[] {
  if (!initialized) {
    throw new Error('WASM not initialized. Call initWasm() first.')
  }
  return JSON.parse(wasmModule.getPitchClasses())
}

export function midiToFrequency(midi: number): number {
  if (!initialized) {
    throw new Error('WASM not initialized. Call initWasm() first.')
  }
  return wasmModule.Mozart.midiToFrequency(midi)
}

export function midiToNoteName(midi: number): string {
  if (!initialized) {
    throw new Error('WASM not initialized. Call initWasm() first.')
  }
  return wasmModule.Mozart.midiToNoteName(midi)
}

export function noteNameToMidi(name: string): number {
  if (!initialized) {
    throw new Error('WASM not initialized. Call initWasm() first.')
  }
  return wasmModule.Mozart.noteNameToMidi(name)
}

export type { Mozart } from './types'
