import React from 'react'
import { useMozartStore } from '../store'

export function AccentEditor() {
  const { accents, cycleAccent } = useMozartStore()

  const getAccentLabel = (level: number): string => {
    switch (level) {
      case 3:
        return '>'
      case 2:
        return '-'
      default:
        return '.'
    }
  }

  const getAccentColor = (level: number): string => {
    switch (level) {
      case 3:
        return '#e94560'
      case 2:
        return '#f5a623'
      default:
        return '#666'
    }
  }

  return (
    <div style={styles.container}>
      <span style={styles.label}>Accents:</span>
      <div style={styles.beats}>
        {accents.map((accent, i) => (
          <button
            key={i}
            style={{
              ...styles.beat,
              color: getAccentColor(accent),
              borderColor: getAccentColor(accent),
            }}
            onClick={() => cycleAccent(i)}
            title={`Beat ${i + 1}: ${accent === 3 ? 'Strong' : accent === 2 ? 'Medium' : 'Weak'}`}
          >
            <span style={styles.beatNumber}>{i + 1}</span>
            <span style={styles.beatAccent}>{getAccentLabel(accent)}</span>
          </button>
        ))}
      </div>
      <span style={styles.hint}>Click to cycle accent level</span>
    </div>
  )
}

const styles: Record<string, React.CSSProperties> = {
  container: {
    display: 'flex',
    alignItems: 'center',
    gap: '12px',
    padding: '12px 20px',
    background: '#1a1a2e',
    borderTop: '1px solid #0f3460',
  },
  label: {
    color: '#888',
    fontSize: '14px',
  },
  beats: {
    display: 'flex',
    gap: '4px',
  },
  beat: {
    display: 'flex',
    flexDirection: 'column',
    alignItems: 'center',
    width: '32px',
    height: '48px',
    padding: '4px',
    background: '#16213e',
    border: '2px solid',
    borderRadius: '4px',
    cursor: 'pointer',
    transition: 'all 0.2s',
  },
  beatNumber: {
    fontSize: '12px',
    color: '#888',
  },
  beatAccent: {
    fontSize: '20px',
    fontWeight: 'bold',
  },
  hint: {
    marginLeft: 'auto',
    color: '#555',
    fontSize: '12px',
  },
}
