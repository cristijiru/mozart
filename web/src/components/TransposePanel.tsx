import React from 'react'
import { useMozartStore } from '../store'

export function TransposePanel() {
  const { key, setKey, transposeChromatic, transposeDiatonic } = useMozartStore()

  // Parse current key into root and scale type
  const keyParts = key.split(' ')
  const currentRoot = keyParts[0] || 'C'
  const currentScaleType = keyParts.slice(1).join(' ') || 'Major'

  const roots = ['C', 'C#', 'D', 'Eb', 'E', 'F', 'F#', 'G', 'Ab', 'A', 'Bb', 'B']
  const scaleTypes = ['Major', 'Natural Minor', 'Harmonic Minor', 'Melodic Minor', 'Dorian', 'Phrygian', 'Lydian', 'Mixolydian', 'Locrian']

  const handleRootChange = (e: React.ChangeEvent<HTMLSelectElement>) => {
    setKey(`${e.target.value} ${currentScaleType}`)
  }

  const handleScaleTypeChange = (e: React.ChangeEvent<HTMLSelectElement>) => {
    setKey(`${currentRoot} ${e.target.value}`)
  }

  return (
    <div style={styles.container}>
      <div style={styles.section}>
        <h3 style={styles.title}>Key</h3>
        <select value={currentRoot} onChange={handleRootChange} style={styles.select}>
          {roots.map((r) => (
            <option key={r} value={r}>
              {r}
            </option>
          ))}
        </select>
        <select value={currentScaleType} onChange={handleScaleTypeChange} style={styles.selectWide}>
          {scaleTypes.map((s) => (
            <option key={s} value={s}>
              {s}
            </option>
          ))}
        </select>
      </div>

      <div style={styles.section}>
        <h3 style={styles.title}>Chromatic</h3>
        <div style={styles.buttons}>
          <button style={styles.button} onClick={() => transposeChromatic(-12)}>
            -Oct
          </button>
          <button style={styles.button} onClick={() => transposeChromatic(-1)}>
            -1
          </button>
          <button style={styles.button} onClick={() => transposeChromatic(1)}>
            +1
          </button>
          <button style={styles.button} onClick={() => transposeChromatic(12)}>
            +Oct
          </button>
        </div>
      </div>

      <div style={styles.section}>
        <h3 style={styles.title}>Diatonic</h3>
        <div style={styles.buttons}>
          <button style={styles.button} onClick={() => transposeDiatonic(-7)}>
            -Oct
          </button>
          <button style={styles.button} onClick={() => transposeDiatonic(-1)}>
            -1
          </button>
          <button style={styles.button} onClick={() => transposeDiatonic(1)}>
            +1
          </button>
          <button style={styles.button} onClick={() => transposeDiatonic(7)}>
            +Oct
          </button>
        </div>
      </div>
    </div>
  )
}

const styles: Record<string, React.CSSProperties> = {
  container: {
    display: 'flex',
    gap: '24px',
    padding: '12px 20px',
    background: '#16213e',
    borderTop: '1px solid #0f3460',
  },
  section: {
    display: 'flex',
    alignItems: 'center',
    gap: '12px',
  },
  title: {
    margin: 0,
    fontSize: '14px',
    color: '#888',
    fontWeight: 'normal',
  },
  select: {
    padding: '6px 12px',
    background: '#1a1a2e',
    border: '1px solid #0f3460',
    borderRadius: '4px',
    color: '#fff',
    fontSize: '14px',
    minWidth: '60px',
  },
  selectWide: {
    padding: '6px 12px',
    background: '#1a1a2e',
    border: '1px solid #0f3460',
    borderRadius: '4px',
    color: '#fff',
    fontSize: '14px',
    minWidth: '140px',
  },
  buttons: {
    display: 'flex',
    gap: '4px',
  },
  button: {
    padding: '6px 12px',
    background: '#0f3460',
    border: 'none',
    borderRadius: '4px',
    color: '#eee',
    cursor: 'pointer',
    fontSize: '13px',
  },
}
