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

  // Helper for formatted logging
  const log = useCallback((msg: string, ...args: any[]) => {
    console.log(`[${new Date().toISOString()}] ${msg}`, ...args)
  }, [])

  // Initialize plugins on app startup
  useEffect(() => {
    log('[App] Component mounted, initializing plugins...')
    
    const initPlugins = async () => {
      try {
        log('[App] initPlugins: Starting plugin initialization...')
        const result = await pluginService.initializeAndLoadPlugins()
        log('[App] initPlugins: Plugins initialized:', result)
        setPluginsInitialized(true)
        
        // Broadcast startup event
        log('[App] initPlugins: Broadcasting Startup event...')
        await pluginService.broadcastEvent('Startup', { timestamp: Date.now() })
        log('[App] initPlugins: Startup event broadcasted')
      } catch (err) {
        console.error(`[${new Date().toISOString()}] [App] initPlugins: Failed to initialize plugins:`, err)
        // Don't set error - plugins are optional
      }
    }
    
    initPlugins()
    
    // Cleanup on unmount
    return () => {
      log('[App] Component unmounting...')
      // Broadcast shutdown event
      pluginService.broadcastEvent('Shutdown', { timestamp: Date.now() })
        .catch(e => console.error(`[${new Date().toISOString()}] [App] Failed to broadcast shutdown:`, e))
    }
  }, [log])

  const handleOpenFile = useCallback(async () => {
    const timestamp = new Date().toISOString();
    log(`[App::handleOpenFile] === OPEN FILE WORKFLOW STARTED ===`);
    log(`[App::handleOpenFile] Entry point - pluginsInitialized=${pluginsInitialized}, timestamp=${timestamp}`);
    
    try {
      setIsLoading(true);
      setError(null);
      log(`[App::handleOpenFile] State updated: isLoading=true, error=null`);
      
      const dialogStartTime = new Date().toISOString();
      log(`[App::handleOpenFile] Calling Tauri dialog.open() at ${dialogStartTime}`);
      log(`[App::handleOpenFile] Dialog params: multiple=false, filters=[All Files, Text Files, Code Files]`);
      
      const selected = await open({
        multiple: false,
        filters: [
          { name: 'All Files', extensions: ['*'] },
          { name: 'Text Files', extensions: ['txt', 'md', 'json', 'js', 'ts', 'py', 'rs', 'html', 'css'] },
          { name: 'Code Files', extensions: ['c', 'cpp', 'h', 'hpp', 'java', 'go', 'rb', 'php'] },
        ]
      });
      
      const dialogEndTime = new Date().toISOString();
      log(`[App::handleOpenFile] Dialog returned at ${dialogEndTime}`);
      log(`[App::handleOpenFile] Dialog result type:`, typeof selected);
      log(`[App::handleOpenFile] Dialog result value:`, selected);
      
      if (selected && typeof selected === 'string') {
        log(`[App::handleOpenFile] VALID FILE SELECTED: path="${selected}"`);
        
        const invokeStartTime = new Date().toISOString();
        log(`[App::handleOpenFile] Invoking 'open_file' command at ${invokeStartTime}`);
        log(`[App::handleOpenFile] Invoke params: { path: "${selected}" }`);
        
        const fileInfo = await invoke<FileInfo>('open_file', { path: selected });
        
        const invokeEndTime = new Date().toISOString();
        log(`[App::handleOpenFile] 'open_file' command completed at ${invokeEndTime}`);
        log(`[App::handleOpenFile] Received fileInfo:`, {
          path: fileInfo.path,
          size: fileInfo.size,
          chunks: fileInfo.chunks,
          chunk_size: fileInfo.chunk_size
        });
        
        setCurrentFile(fileInfo);
        log(`[App::handleOpenFile] currentFile state updated with path="${fileInfo.path}"`);
        
        // Broadcast file opened event to plugins
        if (pluginsInitialized) {
          log(`[App::handleOpenFile] Broadcasting FileOpened event for: "${selected}"`);
          await pluginService.broadcastEvent('FileOpened', { path: selected });
          log(`[App::handleOpenFile] FileOpened event broadcasted successfully`);
        } else {
          log(`[App::handleOpenFile] Skipping plugin broadcast - plugins not initialized`);
        }
      } else {
        log(`[App::handleOpenFile] NO FILE SELECTED: Dialog cancelled or returned invalid value (result=${selected})`);
      }
    } catch (err) {
      const errorTime = new Date().toISOString();
      const errorMsg = err instanceof Error ? err.message : String(err);
      log(`[App::handleOpenFile] ERROR at ${errorTime}:`, errorMsg);
      log(`[App::handleOpenFile] Error type:`, err?.constructor?.name || typeof err);
      if (err instanceof Error && err.stack) {
        log(`[App::handleOpenFile] Error stack:`, err.stack);
      }
      setError(errorMsg);
    } finally {
      setIsLoading(false);
      const endTime = new Date().toISOString();
      log(`[App::handleOpenFile] === OPEN FILE WORKFLOW COMPLETED at ${endTime} ===`);
    }
  }, [pluginsInitialized, log])

  const handleCloseFile = useCallback(async () => {
    if (currentFile) {
      log('[App] handleCloseFile: Closing file: ' + currentFile.path)
      try {
        // Broadcast file closed event
        if (pluginsInitialized) {
          log('[App] handleCloseFile: Broadcasting FileClosed event for: ' + currentFile.path)
          await pluginService.broadcastEvent('FileClosed', { path: currentFile.path })
        }
        
        log('[App] handleCloseFile: Calling invoke("close_file", { path: "' + currentFile.path + '" })')
        await invoke('close_file', { path: currentFile.path })
        log('[App] handleCloseFile: File closed successfully')
        setCurrentFile(null)
      } catch (err) {
        console.error(`[${new Date().toISOString()}] [App] handleCloseFile: Error closing file:`, err)
        setError(err instanceof Error ? err.message : String(err))
      }
    }
  }, [currentFile, pluginsInitialized, log])

  const handleToggleView = useCallback(() => {
    console.log('[App] Toggling view mode:', viewMode, '->', viewMode === 'text' ? 'hex' : 'text')
    setViewMode(prev => prev === 'text' ? 'hex' : 'text')
  }, [viewMode])

  // Log state changes
  useEffect(() => {
    log('[App] State update:', {
      hasFile: !!currentFile,
      viewMode,
      isLoading,
      hasError: !!error,
      pluginsInitialized
    })
  }, [currentFile, viewMode, isLoading, error, pluginsInitialized, log])

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
