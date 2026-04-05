import { useEffect, useRef, useState, useCallback } from 'react'
import Editor from '@monaco-editor/react'
import { invoke } from '@tauri-apps/api/core'
import './Editor.css'

interface EditorProps {
  filePath: string
  fileSize: number
}

const CHUNK_SIZE = 64 * 1024 // 64KB chunks

// Edit operation types for backend communication
interface EditOperation {
  type: 'insert' | 'delete' | 'replace'
  offset: number
  data?: string
  length?: number
}

// Helper for formatted logging
const log = (msg: string, ...args: any[]) => {
  console.log(`[${new Date().toISOString()}] ${msg}`, ...args)
}

// Debounce utility for edit operations
function useDebounce<T extends (...args: any[]) => void>(
  callback: T,
  delay: number
) {
  const timeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null)

  return useCallback((...args: Parameters<T>) => {
    if (timeoutRef.current) {
      clearTimeout(timeoutRef.current)
    }
    timeoutRef.current = setTimeout(() => {
      callback(...args)
    }, delay)
  }, [callback, delay])
}

function FileEditor({ filePath, fileSize }: EditorProps) {
  const [content, setContent] = useState('')
  const [isLoading, setIsLoading] = useState(true)
  const [chunkRange, setChunkRange] = useState({ start: 0, end: Math.min(CHUNK_SIZE * 2, fileSize) })
  const [error, setError] = useState<string | null>(null)
  const [isModified, setIsModified] = useState(false)
  const [canUndo, setCanUndo] = useState(false)
  const [canRedo, setCanRedo] = useState(false)
  const [isSaving, setIsSaving] = useState(false)
  const editorRef = useRef<any>(null)
  const monacoRef = useRef<any>(null)
  const pendingEditsRef = useRef<EditOperation[]>([])

  // Load initial chunks
  useEffect(() => {
    log(`[Editor::useEffect] === EDITOR CONTENT LOADING STARTED ===`)
    log(`[Editor::useEffect] filePath="${filePath}", fileSize=${fileSize}`)
    log(`[Editor::useEffect] chunkRange: start=${chunkRange.start}, end=${chunkRange.end}`)
    
    const loadContent = async () => {
      const loadStartTime = new Date().toISOString()
      log(`[Editor::loadContent] Function entry at ${loadStartTime}`)
      
      try {
        setIsLoading(true)
        setError(null)
        log(`[Editor::loadContent] State updated: isLoading=true, error=null`)
        
        const invokeStartTime = new Date().toISOString()
        log(`[Editor::loadContent] Invoking 'get_text' command at ${invokeStartTime}`)
        log(`[Editor::loadContent] Request params:`, {
          path: filePath,
          start: chunkRange.start,
          end: chunkRange.end
        })
        
        const text = await invoke<string>('get_text', {
          req: {
            path: filePath,
            start: chunkRange.start,
            end: chunkRange.end
          }
        })
        
        const invokeEndTime = new Date().toISOString()
        log(`[Editor::loadContent] 'get_text' command completed at ${invokeEndTime}`)
        log(`[Editor::loadContent] Received text length: ${text.length} bytes`)
        log(`[Editor::loadContent] Text preview (first 200 chars):`, text.substring(0, 200))
        
        setContent(text)
        log(`[Editor::loadContent] Content state updated with ${text.length} bytes`)
      } catch (err) {
        const errorTime = new Date().toISOString()
        const errorMsg = err instanceof Error ? err.message : String(err)
        log(`[Editor::loadContent] ERROR at ${errorTime}:`, errorMsg)
        log(`[Editor::loadContent] Error type:`, err?.constructor?.name || typeof err)
        if (err instanceof Error && err.stack) {
          log(`[Editor::loadContent] Error stack:`, err.stack)
        }
        setError(errorMsg)
        setContent('// Error loading file')
        log(`[Editor::loadContent] Content set to error placeholder`)
      } finally {
        setIsLoading(false)
        const endTime = new Date().toISOString()
        log(`[Editor::loadContent] === CONTENT LOADING COMPLETED at ${endTime} ===`)
      }
    }

    if (filePath) {
      log(`[Editor::useEffect] filePath is valid, calling loadContent()`)
      loadContent()
    } else {
      log(`[Editor::useEffect] SKIPPED: filePath is empty`)
    }
    
    return () => {
      log(`[Editor::useEffect] Cleanup - filePath changed or component unmounted`)
    }
  }, [filePath, fileSize, chunkRange])

  // Check edit status from backend
  const checkEditStatus = useCallback(async () => {
    try {
      const status = await invoke<{
        has_changes: boolean
        can_undo: boolean
        can_redo: boolean
        effective_size: number
      }>('get_edit_status', { path: filePath })
      setIsModified(status.has_changes)
      setCanUndo(status.can_undo)
      setCanRedo(status.can_redo)
      log(`[Editor::checkEditStatus] Status: has_changes=${status.has_changes}, can_undo=${status.can_undo}, can_redo=${status.can_redo}`)
    } catch (err) {
      log(`[Editor::checkEditStatus] ERROR:`, err)
    }
  }, [filePath])

  // Reload content after undo/redo
  const reloadContent = useCallback(async () => {
    log(`[Editor::reloadContent] Reloading content after undo/redo`)
    try {
      const text = await invoke<string>('get_text', {
        req: {
          path: filePath,
          start: chunkRange.start,
          end: chunkRange.end,
        },
      })
      setContent(text)
      log(`[Editor::reloadContent] Content reloaded: ${text.length} bytes`)
    } catch (err) {
      log(`[Editor::reloadContent] ERROR:`, err)
    }
  }, [filePath, chunkRange.start, chunkRange.end])

  // Send edit to backend
  const sendEditToBackend = useCallback(async (edit: EditOperation) => {
    try {
      log(`[Editor::sendEditToBackend] Sending ${edit.type} edit: offset=${edit.offset}, data_length=${edit.data?.length || 0}`)
      
      switch (edit.type) {
        case 'insert':
          await invoke('insert_bytes', {
            req: {
              path: filePath,
              offset: edit.offset,
              data: Array.from(new TextEncoder().encode(edit.data || '')),
            },
          })
          break
        case 'delete':
          await invoke('delete_bytes', {
            req: {
              path: filePath,
              offset: edit.offset,
              length: edit.length || 0,
            },
          })
          break
        case 'replace':
          await invoke('replace_bytes', {
            req: {
              path: filePath,
              offset: edit.offset,
              length: edit.length || 0,
              data: Array.from(new TextEncoder().encode(edit.data || '')),
            },
          })
          break
      }
      
      // Update status after edit
      await checkEditStatus()
      log(`[Editor::sendEditToBackend] Edit sent successfully`)
    } catch (err) {
      log(`[Editor::sendEditToBackend] ERROR:`, err)
      throw err
    }
  }, [filePath, checkEditStatus])

  // Debounced edit sender
  const debouncedSendEdit = useDebounce(sendEditToBackend, 300)

  // Handle content changes from Monaco
  const handleEditorChange = useCallback((value: string | undefined) => {
    if (!value || !editorRef.current) return

    const model = editorRef.current.getModel()
    if (!model) return

    // Get the current full content
    const newContent = value
    const oldContent = content

    // Simple diff: if content changed, calculate what changed
    if (newContent !== oldContent) {
      log(`[Editor::handleEditorChange] Content changed`)
      
      // For simplicity, we'll track the entire content change
      // In a production app, you'd want more sophisticated diffing
      const edit: EditOperation = {
        type: 'replace',
        offset: chunkRange.start,
        length: oldContent.length,
        data: newContent,
      }
      
      // Update local state immediately for responsiveness
      setContent(newContent)
      
      // Send to backend (debounced)
      debouncedSendEdit(edit)
    }
  }, [content, chunkRange.start, debouncedSendEdit])

  const handleEditorDidMount = useCallback((editor: any, monaco: any) => {
    log(`[Editor::handleEditorDidMount] Monaco editor mounted`)
    
    editorRef.current = editor
    monacoRef.current = monaco
    log(`[Editor::handleEditorDidMount] editorRef and monacoRef set`)
    
    // Configure editor for large files
    log(`[Editor::handleEditorDidMount] Configuring editor options...`)
    editor.updateOptions({
      readOnly: false,
      minimap: { enabled: true },
      scrollBeyondLastLine: false,
      wordWrap: 'on',
      lineNumbers: 'on',
      renderWhitespace: 'selection',
      largeFileOptimizations: true,
    })
    log(`[Editor::handleEditorDidMount] Editor options updated`)

    // Virtual scrolling for large files
    editor.onDidScrollChange((e: any) => {
      // Check if scrolled near bottom to load more content
      const scrollTop = e.scrollTop;
      const scrollHeight = e.scrollHeight;
      const clientHeight = e.height || editor.getLayoutInfo().height;
      
      // Load next chunk when within 20% of bottom
      const threshold = scrollHeight * 0.2;
      if (scrollTop + clientHeight >= scrollHeight - threshold && chunkRange.end < fileSize) {
        log(`[Editor::onDidScrollChange] Near bottom (scrollTop=${scrollTop}, scrollHeight=${scrollHeight}), loading next chunk...`);
        loadNextChunk();
      }
    })

    // Setup undo/redo keyboard handlers
    // Keyboard commands are handled by global event listeners below
    
    log(`[Editor::handleEditorDidMount] Content displayed in Monaco editor`)
    
    // Check initial edit status
    checkEditStatus()
  }, [chunkRange.end, fileSize, checkEditStatus])

  // Handle undo
  const handleUndo = useCallback(async () => {
    log(`[Editor::handleUndo] Undo triggered`)
    try {
      const success = await invoke<boolean>('undo', { path: filePath })
      if (success) {
        log(`[Editor::handleUndo] Undo successful`)
        // Reload content to reflect changes
        await reloadContent()
        await checkEditStatus()
      } else {
        log(`[Editor::handleUndo] Nothing to undo`)
      }
    } catch (err) {
      log(`[Editor::handleUndo] ERROR:`, err)
    }
  }, [filePath, reloadContent, checkEditStatus])

  // Handle redo
  const handleRedo = useCallback(async () => {
    log(`[Editor::handleRedo] Redo triggered`)
    try {
      const success = await invoke<boolean>('redo', { path: filePath })
      if (success) {
        log(`[Editor::handleRedo] Redo successful`)
        // Reload content to reflect changes
        await reloadContent()
        await checkEditStatus()
      } else {
        log(`[Editor::handleRedo] Nothing to redo`)
      }
    } catch (err) {
      log(`[Editor::handleRedo] ERROR:`, err)
    }
  }, [filePath, reloadContent, checkEditStatus])

  const handleSave = useCallback(async () => {
    log(`[Editor::handleSave] Save triggered`)
    setIsSaving(true)
    
    try {
      await invoke('save_file', { path: filePath })
      log(`[Editor::handleSave] File saved successfully`)
      setIsModified(false)
      await checkEditStatus()
    } catch (err) {
      log(`[Editor::handleSave] ERROR:`, err)
      setError(err instanceof Error ? err.message : 'Save failed')
    } finally {
      setIsSaving(false)
    }
  }, [filePath, checkEditStatus])

  // Load next chunk for large files
  const loadNextChunk = useCallback(async () => {
    if (chunkRange.end >= fileSize) {
      log(`[Editor::loadNextChunk] Already at end of file`)
      return
    }

    const newStart = chunkRange.end
    const newEnd = Math.min(newStart + CHUNK_SIZE, fileSize)
    
    log(`[Editor::loadNextChunk] Loading chunk: ${newStart} - ${newEnd}`)
    
    try {
      const text = await invoke<string>('get_text', {
        req: {
          path: filePath,
          start: newStart,
          end: newEnd
        }
      })
      
      setContent(prev => prev + text)
      setChunkRange({ start: chunkRange.start, end: newEnd })
      log(`[Editor::loadNextChunk] Loaded ${text.length} bytes, total: ${newEnd} / ${fileSize}`)
    } catch (err) {
      log(`[Editor::loadNextChunk] ERROR:`, err)
    }
  }, [filePath, fileSize, chunkRange])

  // Keyboard shortcuts for save, undo, redo
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      // Save: Ctrl/Cmd + S
      if ((e.metaKey || e.ctrlKey) && e.key === 's') {
        log(`[Editor::handleKeyDown] Save keyboard shortcut triggered`)
        e.preventDefault()
        handleSave()
      }
      // Undo: Ctrl/Cmd + Z
      else if ((e.metaKey || e.ctrlKey) && e.key === 'z' && !e.shiftKey) {
        log(`[Editor::handleKeyDown] Undo keyboard shortcut triggered`)
        e.preventDefault()
        handleUndo()
      }
      // Redo: Ctrl/Cmd + Y or Ctrl/Cmd + Shift + Z
      else if ((e.metaKey || e.ctrlKey) && (e.key === 'y' || (e.key === 'z' && e.shiftKey))) {
        log(`[Editor::handleKeyDown] Redo keyboard shortcut triggered`)
        e.preventDefault()
        handleRedo()
      }
    }
    window.addEventListener('keydown', handleKeyDown)
    return () => window.removeEventListener('keydown', handleKeyDown)
  }, [handleSave, handleUndo, handleRedo])

  // Detect language from file extension
  const getLanguage = (path: string): string => {
    const ext = path.split('.').pop()?.toLowerCase()
    const langMap: Record<string, string> = {
      'js': 'javascript',
      'ts': 'typescript',
      'jsx': 'javascript',
      'tsx': 'typescript',
      'py': 'python',
      'rs': 'rust',
      'html': 'html',
      'css': 'css',
      'json': 'json',
      'md': 'markdown',
      'txt': 'plaintext',
      'c': 'c',
      'cpp': 'cpp',
      'h': 'c',
      'hpp': 'cpp',
      'java': 'java',
      'go': 'go',
      'rb': 'ruby',
      'php': 'php',
      'sh': 'shell',
      'yaml': 'yaml',
      'yml': 'yaml',
      'toml': 'toml',
    }
    const detectedLang = langMap[ext || ''] || 'plaintext'
    log(`[Editor::getLanguage] Detected language "${detectedLang}" for extension ".${ext}"`)
    return detectedLang
  }

  // Log render state
  useEffect(() => {
    log(`[Editor::render] Render state: isLoading=${isLoading}, hasContent=${content.length > 0}, hasError=${!!error}`)
  }, [isLoading, content, error])

  if (isLoading) {
    log(`[Editor::render] Rendering loading state`)
    return (
      <div className="editor-loading">
        Loading file...
      </div>
    )
  }

  if (error) {
    log(`[Editor::render] Rendering error state: ${error}`)
    return (
      <div className="editor-error">
        <div className="error-message">Error loading file: {error}</div>
      </div>
    )
  }

  const detectedLang = getLanguage(filePath)
  log(`[Editor::render] Rendering Monaco editor with language="${detectedLang}", contentLength=${content.length}`)

  return (
    <div className="editor-wrapper">
      <div className="editor-toolbar">
        <div className="editor-info">
          <span className={isModified ? 'modified' : ''}>
            {filePath}{isModified && ' *'}
          </span>
          <span className="editor-stats">
            {chunkRange.start.toLocaleString()} - {chunkRange.end.toLocaleString()} / {fileSize.toLocaleString()} bytes
          </span>
        </div>
        <div className="editor-actions">
          <button 
            onClick={handleUndo} 
            disabled={!canUndo || isSaving}
            title="Undo (Ctrl+Z)"
            className="editor-btn"
          >
            Undo
          </button>
          <button 
            onClick={handleRedo} 
            disabled={!canRedo || isSaving}
            title="Redo (Ctrl+Y)"
            className="editor-btn"
          >
            Redo
          </button>
          <button 
            onClick={handleSave} 
            disabled={isSaving || !isModified}
            title="Save (Ctrl+S)"
            className={`editor-btn save ${isModified ? 'modified' : ''}`}
          >
            {isSaving ? 'Saving...' : 'Save'}
          </button>
        </div>
      </div>
      <Editor
        height="calc(100% - 48px)"
        language={detectedLang}
        value={content}
        theme="vs-dark"
        onMount={handleEditorDidMount}
        onChange={handleEditorChange}
        options={{
          selectOnLineNumbers: true,
          automaticLayout: true,
        }}
      />
    </div>
  )
}

export default FileEditor
