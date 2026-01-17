import React, { useEffect, useState } from 'react'
import { useMozartStore } from './store'
import {
  Header,
  Transport,
  PianoRoll,
  TextInput,
  TransposePanel,
  AccentEditor,
} from './components'

export default function App() {
  const { init, isWasmLoaded } = useMozartStore()
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    init().catch((err) => {
      console.error('Failed to initialize:', err)
      setError(String(err))
    })
  }, [init])

  if (error) {
    return (
      <div style={styles.error}>
        <h1>Failed to load Mozart</h1>
        <p>{error}</p>
        <p>Make sure you've built the WASM package:</p>
        <code>./build-wasm.sh</code>
      </div>
    )
  }

  if (!isWasmLoaded) {
    return (
      <div style={styles.loading}>
        <h1>Loading Mozart...</h1>
        <p>Initializing WebAssembly module</p>
      </div>
    )
  }

  return (
    <div style={styles.app}>
      <Header />
      <Transport />
      <PianoRoll />
      <TransposePanel />
      <AccentEditor />
      <TextInput />
    </div>
  )
}

const styles: Record<string, React.CSSProperties> = {
  app: {
    display: 'flex',
    flexDirection: 'column',
    height: '100vh',
  },
  loading: {
    display: 'flex',
    flexDirection: 'column',
    alignItems: 'center',
    justifyContent: 'center',
    height: '100vh',
    color: '#eee',
    textAlign: 'center',
  },
  error: {
    display: 'flex',
    flexDirection: 'column',
    alignItems: 'center',
    justifyContent: 'center',
    height: '100vh',
    color: '#e94560',
    textAlign: 'center',
    padding: '20px',
  },
}
