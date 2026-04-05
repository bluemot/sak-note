import { useEffect, useRef, useState, useCallback } from 'react'
import Editor from '@monaco-editor/react'
import { invoke } from '@tauri-apps/api/core'
import './Editor.css'

interface EditorProps {
  filePath: string
  fileSize: number
}

const CHUNK_SIZE = 64 * 1024 // 64KB chunks

// Helper for formatted logging
const log = (msg: string, ...args: any[]) => {
  console.log(`[${new Date().toISOString()}] ${msg}`, ...args)
}

function FileEditor({ filePath, fileSize }: EditorProps) {
  const [content, setContent] = useState('')
  const [isLoading, setIsLoading] = useState(true)
  const [chunkRange, setChunkRange] = useState({ start: 0, end: Math.min(CHUNK_SIZE * 2, fileSize) })
  const [error, setError] = useState<string | null>(null)
  const editorRef = useRef<any>(null)

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

  const handleEditorDidMount = useCallback((editor: any) => {
    log(`[Editor::handleEditorDidMount] Monaco editor mounted`)
    
    editorRef.current = editor
    log(`[Editor::handleEditorDidMount] editorRef set`)
    
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
    editor.onDidScrollChange((e) => {
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
    
    log(`[Editor::handleEditorDidMount] Content displayed in Monaco editor`)
  }, [])

  const handleSave = useCallback(() => {
    log(`[Editor::handleSave] Save triggered`)
    // TODO: Implement save functionality
  }, [])

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

  // Keyboard shortcut for save
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key === 's') {
        log(`[Editor::handleKeyDown] Save keyboard shortcut triggered`)
        e.preventDefault()
        handleSave()
      }
    }
    window.addEventListener('keydown', handleKeyDown)
    return () => window.removeEventListener('keydown', handleKeyDown)
  }, [handleSave])

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
      <div className="editor-info">
        <span>{filePath}</span>
        <span className="editor-stats">
          {chunkRange.start.toLocaleString()} - {chunkRange.end.toLocaleString()} / {fileSize.toLocaleString()} bytes
        </span>
      </div>
      <Editor
        height="calc(100% - 32px)"
        language={detectedLang}
        value={content}
        theme="vs-dark"
        onMount={handleEditorDidMount}
        options={{
          selectOnLineNumbers: true,
          automaticLayout: true,
        }}
      />
    </div>
  )
}

export default FileEditor
