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
  const [chunkRange, _setChunkRange] = useState({ start: 0, end: Math.min(CHUNK_SIZE * 2, fileSize) })
  const [error, setError] = useState<string | null>(null)
  const editorRef = useRef<any>(null)

  // Load initial chunks
  useEffect(() => {
    const timestamp = new Date().toISOString()
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
    const timestamp = new Date().toISOString()
    log(`[Editor::handleEditorDidMount] Monaco editor mounted at ${timestamp}`)
    
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
    editor.onDidScrollChange(() => {
      // TODO: Implement virtual scrolling to load chunks on demand
      // This is a placeholder for the actual implementation
    })
    
    log(`[Editor::handleEditorDidMount] Content displayed in Monaco editor`)
  }, [])

  const handleSave = useCallback(() => {
    log(`[Editor::handleSave] Save triggered`)
    // TODO: Implement save functionality
  }, [])

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
