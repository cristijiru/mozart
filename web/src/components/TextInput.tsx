import React, { useState } from 'react'
import { useMozartStore } from '../store'

export function TextInput() {
  const [input, setInput] = useState('')
  const [error, setError] = useState('')
  const { parseMelody, clearNotes } = useMozartStore()

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault()
    setError('')

    if (!input.trim()) return

    try {
      clearNotes()
      const count = parseMelody(input.trim())
      if (count > 0) {
        setInput('')
      }
    } catch (err) {
      setError(String(err))
    }
  }

  return (
    <div style={styles.container}>
      <form onSubmit={handleSubmit} style={styles.form}>
        <label style={styles.label}>
          <span>Melody:</span>
          <input
            type="text"
            value={input}
            onChange={(e) => setInput(e.target.value)}
            placeholder="e.g., C4q D4q E4q F4q G4h"
            style={styles.input}
          />
        </label>
        <button type="submit" style={styles.button}>
          Parse
        </button>
      </form>

      {error && <p style={styles.error}>{error}</p>}

      <div style={styles.help}>
        <p>
          <strong>Format:</strong> [Note][Octave][Duration] separated by spaces
        </p>
        <p>
          <strong>Notes:</strong> C, C#, D, Eb, E, F, F#, G, Ab, A, Bb, B
        </p>
        <p>
          <strong>Durations:</strong> w=whole, h=half, q=quarter, e=eighth, s=sixteenth
        </p>
        <p>
          <strong>Dotted:</strong> Add . after duration (e.g., q. for dotted quarter)
        </p>
        <p>
          <strong>Rests:</strong> R followed by duration (e.g., Rq for quarter rest)
        </p>
      </div>
    </div>
  )
}

const styles: Record<string, React.CSSProperties> = {
  container: {
    padding: '16px 20px',
    background: '#16213e',
    borderTop: '1px solid #0f3460',
  },
  form: {
    display: 'flex',
    gap: '12px',
    alignItems: 'center',
  },
  label: {
    display: 'flex',
    alignItems: 'center',
    gap: '8px',
    flex: 1,
    color: '#aaa',
    fontSize: '14px',
  },
  input: {
    flex: 1,
    padding: '10px 12px',
    background: '#1a1a2e',
    border: '1px solid #0f3460',
    borderRadius: '4px',
    color: '#fff',
    fontSize: '14px',
    fontFamily: 'monospace',
  },
  button: {
    padding: '10px 20px',
    background: '#e94560',
    border: 'none',
    borderRadius: '4px',
    color: '#fff',
    cursor: 'pointer',
    fontSize: '14px',
    fontWeight: 'bold',
  },
  error: {
    marginTop: '8px',
    color: '#ff6b6b',
    fontSize: '13px',
  },
  help: {
    marginTop: '12px',
    padding: '12px',
    background: '#1a1a2e',
    borderRadius: '4px',
    fontSize: '12px',
    color: '#888',
    lineHeight: '1.6',
  },
}
