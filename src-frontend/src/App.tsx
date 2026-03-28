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

// Phase 4: Advanced UI Components
import { ResizableContainer } from './ui-system/components/ResizableContainer'
import { NotificationContainer } from './ui-system/components/NotificationContainer'
import { SearchPanel } from './ui-system/components/SearchPanel'
import { ShortcutManager } from './ui-system/ShortcutManager'

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

// Phase 4: Plugin Loader
import { loadAllPlugins, triggerHook } from './ui-system/pluginLoader'

interface FileInfo {
  path: string
  size: number
  chunks: number
  chunk_size: number
}

function App() {
  const [currentFile, setCurrentFile] = useState<FileInfo | null>(null)
  const [viewMode] = useState<'text' | 'hex'>('text')
  const [isLoading, setIsLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [pluginsInitialized, setPluginsInitialized] = useState(false)
  
  // Phase 4: Search panel state
  const [searchQuery] = useState('')
  const [searchResults] = useState<Array<{
    id: string
    line: number
    column: number
    text: string
    context: string
    filePath?: string
  }>>([])
  const [isSearchVisible, setIsSearchVisible] = useState(false)

  // Helper for formatted logging
  const log = useCallback((msg: string, ...args: any[]) => {
    console.log(`[${new Date().toISOString()}] ${msg}`, ...args)
  }, [])

  // Phase 4: Initialize ShortcutManager and load plugins
  useEffect(() => {
    log('[App] Phase 4: Initializing ShortcutManager...')
    const shortcutManager = new ShortcutManager()
    shortcutManager.init()
    
    // Register custom shortcuts
    shortcutManager.register('Ctrl+Shift+F', 'edit:find_in_files', {
      description: 'Find in Files',
      preventDefault: true,
    })
    shortcutManager.register('Ctrl+Shift+H', 'view:toggle_search_panel', {
      description: 'Toggle Search Panel',
      preventDefault: true,
    })
    
    log('[App] ShortcutManager initialized')
    
    return () => {
      shortcutManager.destroy()
    }
  }, [log])

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

        // Phase 4: Load WASM plugins
        log('[App] Phase 4: Loading plugins...')
        try {
          const plugins = await loadAllPlugins('/plugins')
          log('[App] Plugins loaded:', plugins.length)
          setPluginsInitialized(true)
          
          // Trigger startup hook
          await triggerHook('onStartup')
        } catch (pluginErr) {
          console.warn('[App] Plugin loading failed (optional):', pluginErr)
          // Plugins are optional, continue anyway
        }

        // Broadcast startup event
        log('[App] Broadcasting Startup event...')
        await pluginService.broadcastEvent('Startup', { timestamp: Date.now() })
        log('[App] Startup event broadcasted')
      } catch (err) {
        console.error(`[${new Date().toISOString()}] [App] Failed to initialize:`, err)
        // Don't set error - plugins are optional
      }
    }

    initAll()

    // Cleanup on unmount
    return () => {
      log('[App] Component unmounting...')
      // Trigger shutdown hook
      triggerHook('onShutdown').catch((e: any) => console.error('[App] Shutdown hook error:', e))
      // Broadcast shutdown event
      pluginService.broadcastEvent('Shutdown', { timestamp: Date.now() })
        .catch((e: any) => console.error(`[${new Date().toISOString()}] [App] Failed to broadcast shutdown:`, e))
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
          // Phase 4: Trigger plugin hook
          await triggerHook('onFileOpen', { path: selected });
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


  
  // Phase 4: Handle search result navigation
  const handleSearchNavigate = useCallback((direction: 'prev' | 'next') => {
    log(`[App] Search navigate: ${direction}`);
  }, [log])
  
  // Phase 4: Handle search result click
  const handleSearchResultClick = useCallback((result: any) => {
    log('[App] Search result clicked:', result);
    // Could navigate to the specific location in editor
  }, [log])

  // Log state changes
  useEffect(() => {
    log('[App] State update:', {
      hasFile: !!currentFile,
      viewMode,
      isLoading,
      hasError: !!error,
      pluginsInitialized,
      isSearchVisible
    })
  }, [currentFile, viewMode, isLoading, error, pluginsInitialized, isSearchVisible, log])

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
        <div className="feature">
          <span className="icon">[resize]</span>
          <span>Resizable panels</span>
        </div>
        <div className="feature">
          <span className="icon">[search]</span>
          <span>Enhanced search panel</span>
        </div>
      </div>
    </div>
  );

  return (
    <div className="app">
      <DynamicMenuBar />
      <DynamicToolbar />

      <div className="main-container">
        {/* Phase 4: Wrap Sidebar in ResizableContainer */}
        <ResizableContainer 
          direction="horizontal"
          defaultSize={250}
          minSize={200}
          maxSize={400}
          storageKey="sidebar-width"
        >
          <DynamicSidebar currentFile={currentFile} />
        </ResizableContainer>

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

      {/* Phase 4: Search Panel */}
      {isSearchVisible && (
        <SearchPanel 
          query={searchQuery}
          results={searchResults}
          onResultClick={handleSearchResultClick}
          onNavigate={handleSearchNavigate}
          onClose={() => setIsSearchVisible(false)}
        />
      )}

      {/* Phase 4: Notification Container */}
      <NotificationContainer position="top-right" />

      <DynamicStatusBar />
    </div>
  )
}

export default App
