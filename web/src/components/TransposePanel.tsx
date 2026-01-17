import React, { useState } from 'react'
import { useMozartStore } from '../store'

export function TransposePanel() {
  const { key, setKey, transposeChromatic, transposeDiatonic, invert, notes } = useMozartStore()
  const [keepOriginal, setKeepOriginal] = useState(false)

  // Parse current key into root and scale type
  const keyParts = key.split(' ')
  const currentRoot = keyParts[0] || 'C'
  const currentScaleType = keyParts.slice(1).join(' ') || 'Major'

  const roots = ['C', 'C#', 'D', 'Eb', 'E', 'F', 'F#', 'G', 'Ab', 'A', 'Bb', 'B']
  const scaleTypes = ['Major', 'Natural Minor', 'Harmonic Minor', 'Melodic Minor', 'Dorian', 'Phrygian', 'Lydian', 'Mixolydian', 'Locrian']

  const chromaticOptions = [
    { label: '-Oct', value: -12 },
    { label: '-5th', value: -7 },
    { label: '-4th', value: -5 },
    { label: '-3rd', value: -4 },
    { label: '-2nd', value: -2 },
    { label: '-1', value: -1 },
    { label: '+1', value: 1 },
    { label: '+2nd', value: 2 },
    { label: '+3rd', value: 4 },
    { label: '+4th', value: 5 },
    { label: '+5th', value: 7 },
    { label: '+Oct', value: 12 },
  ]

  const diatonicOptions = [
    { label: '-Oct', value: -7 },
    { label: '-6th', value: -5 },
    { label: '-5th', value: -4 },
    { label: '-4th', value: -3 },
    { label: '-3rd', value: -2 },
    { label: '-2nd', value: -1 },
    { label: '+2nd', value: 1 },
    { label: '+3rd', value: 2 },
    { label: '+4th', value: 3 },
    { label: '+5th', value: 4 },
    { label: '+6th', value: 5 },
    { label: '+Oct', value: 7 },
  ]

  const handleRootChange = (e: React.ChangeEvent<HTMLSelectElement>) => {
    setKey(`${e.target.value} ${currentScaleType}`)
  }

  const handleScaleTypeChange = (e: React.ChangeEvent<HTMLSelectElement>) => {
    setKey(`${currentRoot} ${e.target.value}`)
  }

  const handleChromatic = (e: React.ChangeEvent<HTMLSelectElement>) => {
    const value = parseInt(e.target.value)
    if (!isNaN(value)) {
      transposeChromatic(value, keepOriginal)
    }
    e.target.value = '' // Reset to placeholder
  }

  const handleDiatonic = (e: React.ChangeEvent<HTMLSelectElement>) => {
    const value = parseInt(e.target.value)
    if (!isNaN(value)) {
      transposeDiatonic(value, keepOriginal)
    }
    e.target.value = '' // Reset to placeholder
  }

  const handleInvert = () => {
    if (notes.length === 0) return
    // Use the average pitch as the pivot point
    const avgPitch = Math.round(notes.reduce((sum, n) => sum + n.pitch, 0) / notes.length)
    invert(avgPitch, keepOriginal)
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
        <select onChange={handleChromatic} style={styles.select} defaultValue="">
          <option value="" disabled>
            ...
          </option>
          {chromaticOptions.map((opt) => (
            <option key={opt.value} value={opt.value}>
              {opt.label}
            </option>
          ))}
        </select>
      </div>

      <div style={styles.section}>
        <h3 style={styles.title}>Diatonic</h3>
        <select onChange={handleDiatonic} style={styles.select} defaultValue="">
          <option value="" disabled>
            ...
          </option>
          {diatonicOptions.map((opt) => (
            <option key={opt.value} value={opt.value}>
              {opt.label}
            </option>
          ))}
        </select>
      </div>

      <div style={styles.section}>
        <h3 style={styles.title}>Transform</h3>
        <button style={styles.button} onClick={handleInvert}>
          Invert
        </button>
      </div>

      <label style={styles.checkbox}>
        <input
          type="checkbox"
          checked={keepOriginal}
          onChange={(e) => setKeepOriginal(e.target.checked)}
        />
        Keep original
      </label>
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
    flexWrap: 'wrap',
    alignItems: 'center',
  },
  section: {
    display: 'flex',
    alignItems: 'center',
    gap: '8px',
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
    minWidth: '70px',
    cursor: 'pointer',
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
  button: {
    padding: '6px 12px',
    background: '#0f3460',
    border: 'none',
    borderRadius: '4px',
    color: '#eee',
    cursor: 'pointer',
    fontSize: '13px',
  },
  checkbox: {
    display: 'flex',
    alignItems: 'center',
    gap: '6px',
    color: '#aaa',
    fontSize: '13px',
    cursor: 'pointer',
    marginLeft: 'auto',
  },
}
