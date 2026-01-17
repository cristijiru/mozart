import React, { useRef } from 'react'
import { useMozartStore } from '../store'

export function Header() {
  const { mozart, saveToJson, loadFromJson, exportToMidi, newSong } = useMozartStore()
  const fileInputRef = useRef<HTMLInputElement>(null)

  const handleSave = () => {
    const json = saveToJson()
    if (!json) return

    const blob = new Blob([json], { type: 'application/json' })
    const url = URL.createObjectURL(blob)
    const a = document.createElement('a')
    a.href = url
    a.download = `${mozart?.title || 'song'}.mozart.json`
    a.click()
    URL.revokeObjectURL(url)
  }

  const handleLoad = () => {
    fileInputRef.current?.click()
  }

  const handleFileChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0]
    if (!file) return

    const reader = new FileReader()
    reader.onload = () => {
      const json = reader.result as string
      loadFromJson(json)
    }
    reader.readAsText(file)

    // Reset input
    e.target.value = ''
  }

  const handleExportMidi = () => {
    const midi = exportToMidi()
    if (!midi) return

    const blob = new Blob([midi], { type: 'audio/midi' })
    const url = URL.createObjectURL(blob)
    const a = document.createElement('a')
    a.href = url
    a.download = `${mozart?.title || 'song'}.mid`
    a.click()
    URL.revokeObjectURL(url)
  }

  return (
    <header style={styles.header}>
      <h1 style={styles.title}>Mozart</h1>

      <div style={styles.actions}>
        <button style={styles.button} onClick={() => newSong()}>
          New
        </button>
        <button style={styles.button} onClick={handleLoad}>
          Open
        </button>
        <button style={styles.button} onClick={handleSave}>
          Save
        </button>
        <button style={styles.button} onClick={handleExportMidi}>
          Export MIDI
        </button>
      </div>

      <input
        ref={fileInputRef}
        type="file"
        accept=".mozart.json,.json"
        style={{ display: 'none' }}
        onChange={handleFileChange}
      />
    </header>
  )
}

const styles: Record<string, React.CSSProperties> = {
  header: {
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'space-between',
    padding: '12px 20px',
    background: '#16213e',
    borderBottom: '1px solid #0f3460',
  },
  title: {
    fontSize: '24px',
    fontWeight: 'bold',
    color: '#e94560',
    margin: 0,
  },
  actions: {
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
}
