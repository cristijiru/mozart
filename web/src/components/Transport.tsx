import React from 'react'
import { useMozartStore } from '../store'

export function Transport() {
  const {
    playbackState,
    tempo,
    timeSignature,
    currentTick,
    isMetronomeEnabled,
    play,
    pause,
    stop,
    setTempo,
    setTimeSignature,
    toggleMetronome,
    clearNotes,
  } = useMozartStore()

  const handleTempoChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    setTempo(parseInt(e.target.value, 10) || 120)
  }

  const handleTimeSignatureChange = (e: React.ChangeEvent<HTMLSelectElement>) => {
    setTimeSignature(e.target.value)
  }

  // Calculate beat position for display
  const ticksPerBeat = 480
  const beat = Math.floor(currentTick / ticksPerBeat) + 1
  const ticksPerMeasure = ticksPerBeat * timeSignature.numerator
  const measure = Math.floor(currentTick / ticksPerMeasure) + 1
  const beatInMeasure = ((beat - 1) % timeSignature.numerator) + 1

  return (
    <div style={styles.transport}>
      <div style={styles.controls}>
        <button
          style={{
            ...styles.button,
            ...(playbackState === 'stopped' ? styles.buttonActive : {}),
          }}
          onClick={stop}
        >
          Stop
        </button>

        {playbackState === 'playing' ? (
          <button style={styles.playButton} onClick={pause}>
            Pause
          </button>
        ) : (
          <button style={styles.playButton} onClick={play}>
            Play
          </button>
        )}

        <button
          style={{
            ...styles.button,
            ...(isMetronomeEnabled ? styles.buttonActive : {}),
          }}
          onClick={toggleMetronome}
        >
          Metro
        </button>

        <button
          style={styles.clearButton}
          onClick={clearNotes}
        >
          Clear
        </button>
      </div>

      <div style={styles.position}>
        <span style={styles.positionLabel}>Position:</span>
        <span style={styles.positionValue}>
          {measure}:{beatInMeasure}
        </span>
      </div>

      <div style={styles.settings}>
        <label style={styles.label}>
          <span>Tempo:</span>
          <input
            type="number"
            min="20"
            max="300"
            value={tempo}
            onChange={handleTempoChange}
            style={styles.input}
          />
          <span>BPM</span>
        </label>

        <label style={styles.label}>
          <span>Time:</span>
          <select
            value={`${timeSignature.numerator}/${timeSignature.denominator}`}
            onChange={handleTimeSignatureChange}
            style={styles.select}
          >
            <option value="2/4">2/4</option>
            <option value="3/4">3/4</option>
            <option value="4/4">4/4</option>
            <option value="5/4">5/4</option>
            <option value="6/4">6/4</option>
            <option value="7/4">7/4</option>
            <option value="5/8">5/8</option>
            <option value="6/8">6/8</option>
            <option value="7/8">7/8</option>
            <option value="9/8">9/8</option>
            <option value="11/8">11/8</option>
            <option value="12/8">12/8</option>
          </select>
        </label>
      </div>
    </div>
  )
}

const styles: Record<string, React.CSSProperties> = {
  transport: {
    display: 'flex',
    alignItems: 'center',
    gap: '24px',
    padding: '12px 20px',
    background: '#1a1a2e',
    borderBottom: '1px solid #0f3460',
  },
  controls: {
    display: 'flex',
    gap: '8px',
  },
  button: {
    padding: '8px 16px',
    background: '#0f3460',
    border: 'none',
    borderRadius: '4px',
    color: '#eee',
    cursor: 'pointer',
    fontSize: '14px',
    transition: 'background 0.2s',
  },
  buttonActive: {
    background: '#e94560',
  },
  playButton: {
    padding: '8px 24px',
    background: '#e94560',
    border: 'none',
    borderRadius: '4px',
    color: '#fff',
    cursor: 'pointer',
    fontSize: '14px',
    fontWeight: 'bold',
  },
  clearButton: {
    padding: '8px 16px',
    background: '#4a1a2e',
    border: 'none',
    borderRadius: '4px',
    color: '#eee',
    cursor: 'pointer',
    fontSize: '14px',
  },
  position: {
    display: 'flex',
    alignItems: 'center',
    gap: '8px',
    padding: '8px 16px',
    background: '#16213e',
    borderRadius: '4px',
    minWidth: '100px',
  },
  positionLabel: {
    color: '#888',
    fontSize: '12px',
  },
  positionValue: {
    color: '#fff',
    fontSize: '18px',
    fontFamily: 'monospace',
    fontWeight: 'bold',
  },
  settings: {
    display: 'flex',
    gap: '16px',
    marginLeft: 'auto',
  },
  label: {
    display: 'flex',
    alignItems: 'center',
    gap: '8px',
    color: '#aaa',
    fontSize: '14px',
  },
  input: {
    width: '60px',
    padding: '6px 8px',
    background: '#16213e',
    border: '1px solid #0f3460',
    borderRadius: '4px',
    color: '#fff',
    fontSize: '14px',
    textAlign: 'center',
  },
  select: {
    padding: '6px 8px',
    background: '#16213e',
    border: '1px solid #0f3460',
    borderRadius: '4px',
    color: '#fff',
    fontSize: '14px',
  },
}
