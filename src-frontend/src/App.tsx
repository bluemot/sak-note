import { useState, useCallback, useEffect } from 'react'
import { open } from '@tauri-apps/plugin-dialog'
import { invoke } from '@tauri-apps/api/core'
import Editor from './components/Editor'
import HexViewer from './components/HexViewer'

// Import dynamic UI components from ui-system
import { DynamicToolbar } from './ui-system/components/DynamicToolbar'
import { DynamicSidebar } from './ui-system/components/DynamicSidebar'
import { DynamicMenuBar } from './ui-system/components/DynamicMenuBar'
import { DynamicStatusBar } from './ui-system/components/DynamicStatusBar'

import './App.css'

// Import plugin service for initialization and logging
import * as pluginService from './services/pluginService'

// Import all Phase 2 modules
import {
  registerFileModule,
  registerFileActions,
  registerEditModule,
  registerEditActions,
  registerSftpModule,
  registerSftpActions,
  registerMarksModule,
  registerMarksActions,
  registerBookmarkModule,
  registerBookmarkActions,
  registerLlmModule,
  registerLlmActions,
  registerPrintModule,
  registerPrintActions
} from './modules'

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

  // Initialize all modules and plugins on app startup
  useEffect(() => {
    log('[App] Component mounted, initializing modules...')

    const initAll = async () => {
      try {
        // Register UI modules
        log('[App] Registering UI modules...')
        registerFileModule()
        registerEditModule()
        registerSftpModule()
        registerMarksModule()
        registerBookmarkModule()
        registerLlmModule()
        registerPrintModule()
        log('[App] All UI modules registered')

        // Register action handlers
        log('[App] Registering action handlers...')
        registerFileActions()
        registerEditActions()
        registerSftpActions()
        registerMarksActions()
        registerBookmarkActions()
        registerLlmActions()
        registerPrintActions()
        log('[App] All action handlers registered')

        // Initialize plugins
        log('[App] initPlugins: Starting plugin initialization...')
        const result = await pluginService.initializeAndLoadPlugins()
        log('[App] initPlugins: Plugins initialized:', result)
        setPluginsInitialized(true)

        // Broadcast startup event
        log('[App] initPlugins: Broadcasting Startup event...')
        await pluginService.broadcastEvent('Startup', { timestamp: Date.now() })
        log('[App] initPlugins: Startup event broadcasted')
      } catch (err) {
        console.error(`[${new Date().toISOString()}] [App] initPlugins: Failed to initialize:`, err)
        // Don't set error - plugins are optional
      }
    }

    initAll()

    // Cleanup on unmount
    return () => {
      log('[App] Component unmounting...')
      // Broadcast shutdown event
      pluginService.broadcastEvent('Shutdown', { timestamp: Date.now() })
        .catch(e => console.error(`[${new Date().toISOString()}] [App] Failed to broadcast shutdown:`, e))
    }
  }, [log])

  const handleOpenFile = useCallback(async () => {
    log(`[App::handleOpenFile] === OPEN FILE WORKFLOW STARTED ===`);

    try {
      setIsLoading(true);
      setError(null);

      const selected = await open({
        multiple: false,
        filters: [
          { name: 'All Files', extensions: ['*'] },
          { name: 'Text Files', extensions: ['txt', 'md', 'json', 'js', 'ts', 'py', 'rs', 'html', 'css'] },
          { name: 'Code Files', extensions: ['c', 'cpp', 'h', 'hpp', 'java', 'go', 'rb', 'php'] },
        ]
      });

      if (selected && typeof selected === 'string') {
        log(`[App::handleOpenFile] Opening file: ${selected}`);

        const fileInfo = await invoke<FileInfo>('open_file', { path: selected });

        setCurrentFile(fileInfo);

        // Broadcast file opened event to plugins
        if (pluginsInitialized) {
          await pluginService.broadcastEvent('FileOpened', { path: selected });
        }
      }
    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : String(err);
      log(`[App::handleOpenFile] ERROR:`, errorMsg);
      setError(errorMsg);
    } finally {
      setIsLoading(false);
    }
  }, [pluginsInitialized, log]);

  const handleCloseFile = useCallback(async () => {
    if (currentFile) {
      log('[App] handleCloseFile: Closing file:', currentFile.path);
      try {
        // Broadcast file closed event
        if (pluginsInitialized) {
          await pluginService.broadcastEvent('FileClosed', { path: currentFile.path });
        }

        await invoke('close_file', { path: currentFile.path });
        setCurrentFile(null);
        setError(null);
      } catch (err) {
        console.error(`[${new Date().toISOString()}] [App] handleCloseFile: Error closing file:`, err);
        setError(err instanceof Error ? err.message : String(err));
      }
    }
  }, [currentFile, pluginsInitialized, log]);

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

  // These handlers are currently unused but will be connected to action registry in future
  const _handleCloseFile = handleCloseFile;
  const _handleToggleView = handleToggleView;

  // Use the underscored variables to prevent TypeScript errors
  void _handleCloseFile;
  void _handleToggleView;

  // Welcome screen component
  const WelcomeScreen = () => (
    <div className="welcome-screen">
      <h1>SAK Editor</h1>
      <p>A modern editor for large files with LLM integration</p>

      {pluginsInitialized && (
        <div className="plugin-status">
          <span className="plugin-indicator">Plugins Active</span>
        </div>
      )}

      <button onClick={handleOpenFile} className="open-btn">
        Open File
      </button>

      <div className="features">
        <div className="feature">
          <span className="icon">[file]</span>
          <span>Large file support (memory-mapped)</span>
        </div>
        <div className="feature">
          <span className="icon">[hex]</span>
          <span>Hex viewer mode</span>
        </div>
        <div className="feature">
          <span className="icon">[llm]</span>
          <span>LLM chat & summary</span>
        </div>
        <div className="feature">
          <span className="icon">[color]</span>
          <span>Color highlighting</span>
        </div>
        <div className="feature">
          <span className="icon">[plugin]</span>
          <span>Plugin system with WASM support</span>
        </div>
      </div>
    </div>
  );

  return (
    <div className="app">
      <DynamicMenuBar />
      <DynamicToolbar />

      <div className="main-container">
        <DynamicSidebar currentFile={currentFile} />

        <div className="editor-container">
          {error && (
            <div className="error-banner">
              Error: {error}
            </div>
          )}

          {!currentFile ? (
            <WelcomeScreen />
          ) : viewMode === 'text' ? (
            <Editor filePath={currentFile.path} fileSize={currentFile.size} />
          ) : (
            <HexViewer filePath={currentFile.path} fileSize={currentFile.size} />
          )}
        </div>
      </div>

      <DynamicStatusBar />
    </div>
  )
}

export default App
