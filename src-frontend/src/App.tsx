import { useState, useCallback } from 'react'
import { open } from '@tauri-apps/plugin-dialog'
import { invoke } from '@tauri-apps/api/core'
import Editor from './components/Editor'
import HexViewer from './components/HexViewer'
import Sidebar from './components/Sidebar'
import Toolbar from './components/Toolbar'
import './App.css'

interface FileInfo {
  path: string
  size: number
  chunks: number
  chunk_size: number
}

function App() {
  const [currentFile, setCurrentFile] = useState<FileInfo | null>(null)
  const [viewMode, setViewMode] = useState<'text' | 'hex'>('text')
  const [isLoading, setIsLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)

  const handleOpenFile = useCallback(async () => {
    try {
      setIsLoading(true)
      setError(null)
      
      const selected = await open({
        multiple: false,
        filters: [
          { name: 'All Files', extensions: ['*'] },
          { name: 'Text Files', extensions: ['txt', 'md', 'json', 'js', 'ts', 'py', 'rs', 'html', 'css'] },
          { name: 'Code Files', extensions: ['c', 'cpp', 'h', 'hpp', 'java', 'go', 'rb', 'php'] },
        ]
      })
      
      if (selected && typeof selected === 'string') {
        const fileInfo = await invoke<FileInfo>('open_file', { path: selected })
        setCurrentFile(fileInfo)
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err))
    } finally {
      setIsLoading(false)
    }
  }, [])

  const handleCloseFile = useCallback(async () => {
    if (currentFile) {
      await invoke('close_file', { path: currentFile.path })
      setCurrentFile(null)
    }
  }, [currentFile])

  const handleToggleView = useCallback(() => {
    setViewMode(prev => prev === 'text' ? 'hex' : 'text')
  }, [])

  return (
    <div className="app">
      <Toolbar
        onOpenFile={handleOpenFile}
        onCloseFile={handleCloseFile}
        onToggleView={handleToggleView}
        viewMode={viewMode}
        hasFile={!!currentFile}
        isLoading={isLoading}
      />
      
      <div className="main-container">
        <Sidebar currentFile={currentFile} />
        
        <div className="editor-container">
          {error && (
            <div className="error-banner">
              Error: {error}
            </div>
          )}
          
          {!currentFile ? (
            <div className="welcome-screen">
              <h1>SAK Editor</h1>
              <p>A modern editor for large files with LLM integration</p>
              <button onClick={handleOpenFile} className="open-btn">
                Open File
              </button>
              <div className="features">
                <div className="feature">
                  <span className="icon">📄</span>
                  <span>Large file support (memory-mapped)</span>
                </div>
                <div className="feature">
                  <span className="icon">🔍</span>
                  <span>Hex viewer mode</span>
                </div>
                <div className="feature">
                  <span className="icon">🤖</span>
                  <span>LLM chat & summary</span>
                </div>
                <div className="feature">
                  <span className="icon">🎨</span>
                  <span>Color highlighting</span>
                </div>
              </div>
            </div>
          ) : viewMode === 'text' ? (
            <Editor filePath={currentFile.path} fileSize={currentFile.size} />
          ) : (
            <HexViewer filePath={currentFile.path} fileSize={currentFile.size} />
          )}
        </div>
      </div>
    </div>
  )
}

export default App
