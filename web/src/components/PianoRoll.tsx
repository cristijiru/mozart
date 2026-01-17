import React, { useRef, useEffect, useCallback } from 'react'
import { useMozartStore } from '../store'
import { midiToNoteName } from '../wasm'

const PIANO_KEY_WIDTH = 60
const NOTE_HEIGHT = 12
const TICK_WIDTH = 0.1 // pixels per tick
const MIN_PITCH = 36 // C2
const MAX_PITCH = 84 // C6
const TOTAL_KEYS = MAX_PITCH - MIN_PITCH + 1

export function PianoRoll() {
  const canvasRef = useRef<HTMLCanvasElement>(null)
  const containerRef = useRef<HTMLDivElement>(null)

  const {
    notes,
    currentTick,
    playbackState,
    timeSignature,
    selectedNoteIndex,
    isWasmLoaded,
    addNote,
    removeNote,
    selectNote,
    playNotePreview,
  } = useMozartStore()

  const ticksPerBeat = 480
  const ticksPerMeasure = ticksPerBeat * timeSignature.numerator

  // Draw the piano roll
  const draw = useCallback(() => {
    const canvas = canvasRef.current
    const container = containerRef.current
    if (!canvas || !container) return

    const ctx = canvas.getContext('2d')
    if (!ctx) return

    // Set canvas size
    const width = container.clientWidth
    const height = TOTAL_KEYS * NOTE_HEIGHT
    canvas.width = width
    canvas.height = height

    // Clear
    ctx.fillStyle = '#1a1a2e'
    ctx.fillRect(0, 0, width, height)

    const gridWidth = width - PIANO_KEY_WIDTH

    // Draw horizontal lines (pitch grid)
    for (let i = 0; i <= TOTAL_KEYS; i++) {
      const y = i * NOTE_HEIGHT
      const pitch = MAX_PITCH - i
      const isBlackKey = [1, 3, 6, 8, 10].includes(pitch % 12)

      // Row background
      ctx.fillStyle = isBlackKey ? '#151525' : '#1a1a2e'
      ctx.fillRect(PIANO_KEY_WIDTH, y, gridWidth, NOTE_HEIGHT)

      // Grid line
      ctx.strokeStyle = '#252540'
      ctx.beginPath()
      ctx.moveTo(PIANO_KEY_WIDTH, y)
      ctx.lineTo(width, y)
      ctx.stroke()
    }

    // Draw vertical lines (beat grid)
    const totalTicks = Math.max(ticksPerMeasure * 8, ...notes.map((n) => n.start_tick + n.duration_ticks))

    for (let tick = 0; tick <= totalTicks; tick += ticksPerBeat / 4) {
      const x = PIANO_KEY_WIDTH + tick * TICK_WIDTH
      if (x > width) break

      const isMeasure = tick % ticksPerMeasure === 0
      const isBeat = tick % ticksPerBeat === 0

      ctx.strokeStyle = isMeasure ? '#404060' : isBeat ? '#303050' : '#252540'
      ctx.lineWidth = isMeasure ? 2 : 1
      ctx.beginPath()
      ctx.moveTo(x, 0)
      ctx.lineTo(x, height)
      ctx.stroke()
      ctx.lineWidth = 1
    }

    // Draw notes
    notes.forEach((note, index) => {
      const x = PIANO_KEY_WIDTH + note.start_tick * TICK_WIDTH
      const y = (MAX_PITCH - note.pitch) * NOTE_HEIGHT
      const noteWidth = note.duration_ticks * TICK_WIDTH

      // Skip if off screen
      if (x + noteWidth < PIANO_KEY_WIDTH || x > width) return
      if (note.pitch < MIN_PITCH || note.pitch > MAX_PITCH) return

      const isSelected = index === selectedNoteIndex

      // Note rectangle
      ctx.fillStyle = isSelected ? '#e94560' : '#4a90d9'
      ctx.fillRect(x, y + 1, noteWidth - 1, NOTE_HEIGHT - 2)

      // Note border
      ctx.strokeStyle = isSelected ? '#ff6b8a' : '#6ab0ff'
      ctx.strokeRect(x, y + 1, noteWidth - 1, NOTE_HEIGHT - 2)
    })

    // Draw playhead
    if (playbackState !== 'stopped') {
      const playheadX = PIANO_KEY_WIDTH + currentTick * TICK_WIDTH
      ctx.strokeStyle = '#e94560'
      ctx.lineWidth = 2
      ctx.beginPath()
      ctx.moveTo(playheadX, 0)
      ctx.lineTo(playheadX, height)
      ctx.stroke()
      ctx.lineWidth = 1
    }

    // Draw piano keys
    for (let i = 0; i < TOTAL_KEYS; i++) {
      const y = i * NOTE_HEIGHT
      const pitch = MAX_PITCH - i
      const isBlackKey = [1, 3, 6, 8, 10].includes(pitch % 12)

      // Key background
      ctx.fillStyle = isBlackKey ? '#333' : '#fff'
      ctx.fillRect(0, y, PIANO_KEY_WIDTH - 2, NOTE_HEIGHT)

      // Key border
      ctx.strokeStyle = '#000'
      ctx.strokeRect(0, y, PIANO_KEY_WIDTH - 2, NOTE_HEIGHT)

      // Note name (on all white keys)
      if (!isBlackKey && isWasmLoaded) {
        ctx.fillStyle = '#000'
        ctx.font = '10px sans-serif'
        ctx.fillText(midiToNoteName(pitch), 4, y + NOTE_HEIGHT - 3)
      }
    }
  }, [notes, currentTick, playbackState, timeSignature, selectedNoteIndex, isWasmLoaded])

  // Redraw on state changes
  useEffect(() => {
    draw()
  }, [draw])

  // Redraw on resize
  useEffect(() => {
    const handleResize = () => draw()
    window.addEventListener('resize', handleResize)
    return () => window.removeEventListener('resize', handleResize)
  }, [draw])

  // Animation loop for playback
  useEffect(() => {
    if (playbackState !== 'playing') return

    let animationId: number
    const animate = () => {
      draw()
      animationId = requestAnimationFrame(animate)
    }
    animate()

    return () => cancelAnimationFrame(animationId)
  }, [playbackState, draw])

  // Handle click to add/select notes
  const handleClick = (e: React.MouseEvent<HTMLCanvasElement>) => {
    const canvas = canvasRef.current
    if (!canvas) return

    const rect = canvas.getBoundingClientRect()
    const x = e.clientX - rect.left
    const y = e.clientY - rect.top

    // Ignore clicks on piano keys
    if (x < PIANO_KEY_WIDTH) {
      // Play the note preview
      const pitch = MAX_PITCH - Math.floor(y / NOTE_HEIGHT)
      if (pitch >= MIN_PITCH && pitch <= MAX_PITCH) {
        playNotePreview(pitch)
      }
      return
    }

    const tick = Math.floor((x - PIANO_KEY_WIDTH) / TICK_WIDTH)
    const pitch = MAX_PITCH - Math.floor(y / NOTE_HEIGHT)

    if (pitch < MIN_PITCH || pitch > MAX_PITCH) return

    // Check if clicking on existing note
    const clickedIndex = notes.findIndex(
      (n) =>
        n.pitch === pitch &&
        tick >= n.start_tick &&
        tick < n.start_tick + n.duration_ticks
    )

    if (clickedIndex !== -1) {
      if (e.shiftKey) {
        // Shift+click to delete
        removeNote(clickedIndex)
      } else {
        // Click to select
        selectNote(selectedNoteIndex === clickedIndex ? null : clickedIndex)
      }
    } else {
      // Click to add new note
      // Snap to grid (quarter note grid by default)
      const snapTicks = ticksPerBeat
      const snappedTick = Math.floor(tick / snapTicks) * snapTicks
      const defaultDuration = ticksPerBeat // Quarter note

      addNote(pitch, snappedTick, defaultDuration)
      playNotePreview(pitch)
    }
  }

  return (
    <div ref={containerRef} style={styles.container}>
      <canvas ref={canvasRef} style={styles.canvas} onClick={handleClick} />
    </div>
  )
}

const styles: Record<string, React.CSSProperties> = {
  container: {
    flex: 1,
    overflow: 'auto',
    background: '#1a1a2e',
  },
  canvas: {
    display: 'block',
    cursor: 'crosshair',
  },
}
