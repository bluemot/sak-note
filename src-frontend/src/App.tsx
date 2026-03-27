import { useState, useCallback, useEffect } from 'react'
import { open } from '@tauri-apps/plugin-dialog'
import { invoke } from '@tauri-apps/api/core'
import Editor from './components/Editor'
import HexViewer from './components/HexViewer'
import Sidebar from './components/Sidebar'
import Toolbar from './components/Toolbar'
import './App.css'

// Import plugin service for initialization and logging
import * as pluginService from './services/pluginService'

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
  const [pluginsInitialized, setPluginsInitialized] = useState(false)

  // Initialize plugins on app startup
  useEffect(() => {
    console.log('[App] Component mounted, initializing plugins...')
    
    const initPlugins = async () => {
      try {
        console.log('[App] Starting plugin initialization...')
        const result = await pluginService.initializeAndLoadPlugins()
        console.log('[App] Plugins initialized:', result)
        setPluginsInitialized(true)
        
        // Broadcast startup event
        await pluginService.broadcastEvent('Startup', { timestamp: Date.now() })
        console.log('[App] Startup event broadcasted')
      } catch (err) {
        console.error('[App] Failed to initialize plugins:', err)
        // Don't set error - plugins are optional
      }
    }
    
    initPlugins()
    
    // Cleanup on unmount
    return () => {
      console.log('[App] Component unmounting...')
      // Broadcast shutdown event
      pluginService.broadcastEvent('Shutdown', { timestamp: Date.now() })
        .catch(e => console.error('[App] Failed to broadcast shutdown:', e))
    }
  }, [])

  const handleOpenFile = useCallback(async () => {
    console.log('[App] Opening file dialog...')
    try {
      setIsLoading(true)
      setError(null)
      
      console.log('[App] Calling Tauri dialog open...')
      const selected = await open({
        multiple: false,
        filters: [
          { name: 'All Files', extensions: ['*'] },
          { name: 'Text Files', extensions: ['txt', 'md', 'json', 'js', 'ts', 'py', 'rs', 'html', 'css'] },
          { name: 'Code Files', extensions: ['c', 'cpp', 'h', 'hpp', 'java', 'go', 'rb', 'php'] },
        ]
      })
      
      console.log('[App] Dialog result:', selected)
      
      if (selected && typeof selected === 'string') {
        console.log('[App] Opening file:', selected)
        const fileInfo = await invoke<FileInfo>('open_file', { path: selected })
        console.log('[App] File opened:', fileInfo)
        setCurrentFile(fileInfo)
        
        // Broadcast file opened event to plugins
        if (pluginsInitialized) {
          console.log('[App] Broadcasting FileOpened event...')
          await pluginService.broadcastEvent('FileOpened', { path: selected })
        }
      } else {
        console.log('[App] No file selected')
      }
    } catch (err) {
      console.error('[App] Error opening file:', err)
      setError(err instanceof Error ? err.message : String(err))
    } finally {
      setIsLoading(false)
    }
  }, [pluginsInitialized])

  const handleCloseFile = useCallback(async () => {
    if (currentFile) {
      console.log('[App] Closing file:', currentFile.path)
      try {
        // Broadcast file closed event
        if (pluginsInitialized) {
          console.log('[App] Broadcasting FileClosed event...')
          await pluginService.broadcastEvent('FileClosed', { path: currentFile.path })
        }
        
        await invoke('close_file', { path: currentFile.path })
        console.log('[App] File closed successfully')
        setCurrentFile(null)
      } catch (err) {
        console.error('[App] Error closing file:', err)
        setError(err instanceof Error ? err.message : String(err))
      }
    }
  }, [currentFile, pluginsInitialized])

  const handleToggleView = useCallback(() => {
    console.log('[App] Toggling view mode:', viewMode, '->', viewMode === 'text' ? 'hex' : 'text')
    setViewMode(prev => prev === 'text' ? 'hex' : 'text')
  }, [viewMode])

  // Log state changes
  useEffect(() => {
    console.log('[App] State update:', {
      hasFile: !!currentFile,
      viewMode,
      isLoading,
      hasError: !!error,
      pluginsInitialized
    })
  }, [currentFile, viewMode, isLoading, error, pluginsInitialized])

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
              
              {pluginsInitialized && (
                <div className="plugin-status">
                  <span className="plugin-indicator">🔌 Plugins Active</span>
                </div>
              )}
              
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
                <div className="feature">
                  <span className="icon">🔌</span>
                  <span>Plugin system with WASM support</span>
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
