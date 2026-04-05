import { useEffect, useRef, useState, useCallback } from 'react'
import Editor from '@monaco-editor/react'
import { invoke } from '@tauri-apps/api/core'
import './Editor.css'

interface EditorProps {
  filePath: string
  fileSize: number
}

const CHUNK_SIZE = 64 * 1024 // 64KB chunks
const LINES_PER_CHUNK = 1000 // Approximate lines per chunk

// Helper for formatted logging
const log = (msg: string, ...args: any[]) => {
  console.log(`[${new Date().toISOString()}] ${msg}`, ...args)
}

// Virtual line - represents a line that may not be loaded yet
interface VirtualLine {
  loaded: boolean
  content?: string
  lineNumber: number // 1-indexed
}

function FileEditor({ filePath, fileSize }: EditorProps) {
  const [isLoading, setIsLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [isModified, setIsModified] = useState(false)
  const [canUndo, setCanUndo] = useState(false)
  const [canRedo, setCanRedo] = useState(false)
  const [isSaving, setIsSaving] = useState(false)
  const [totalLines, setTotalLines] = useState(0)
  const [loadedRanges, setLoadedRanges] = useState<{start: number, end: number}[]>([])
  
  const editorRef = useRef<any>(null)
  const monacoRef = useRef<any>(null)
  const virtualLinesRef = useRef<VirtualLine[]>([])
  const contentMapRef = useRef<Map<number, string>>(new Map()) // lineNumber -> content

  // Get total line count first
  useEffect(() => {
    const getLineCount = async () => {
      try {
        // Get line count from backend
        const lineCount = await invoke<number>('get_line_count', { path: filePath })
        setTotalLines(lineCount)
        log(`[Editor::init] File has ${lineCount} total lines`)
        
        // Initialize virtual lines
        virtualLinesRef.current = Array.from({ length: lineCount }, (_, i) => ({
          loaded: false,
          lineNumber: i + 1
        }))
        
        // Load initial viewport (first 100 lines)
        await loadLineRange(1, Math.min(100, lineCount))
      } catch (err) {
        log(`[Editor::init] ERROR getting line count:`, err)
        setError('Failed to get file info')
      }
    }
    
    if (filePath) {
      getLineCount()
    }
  }, [filePath])

  // Load a range of lines
  const loadLineRange = useCallback(async (startLine: number, endLine: number) => {
    log(`[Editor::loadLineRange] Loading lines ${startLine}-${endLine}`)
    
    try {
      const lines = await invoke<string[]>('get_lines', {
        req: {
          path: filePath,
          start_line: startLine,
          count: endLine - startLine + 1
        }
      })
      
      // Update content map
      lines.forEach((content, index) => {
        const lineNum = startLine + index
        contentMapRef.current.set(lineNum, content)
        if (virtualLinesRef.current[lineNum - 1]) {
          virtualLinesRef.current[lineNum - 1].loaded = true
          virtualLinesRef.current[lineNum - 1].content = content
        }
      })
      
      // Update loaded ranges
      setLoadedRanges(prev => [...prev, { start: startLine, end: endLine }])
      
      log(`[Editor::loadLineRange] Loaded ${lines.length} lines`)
      
      // Update editor content
      if (editorRef.current && monacoRef.current) {
        updateEditorContent()
      }
    } catch (err) {
      log(`[Editor::loadLineRange] ERROR:`, err)
    }
  }, [filePath])

  // Update editor content from loaded lines
  const updateEditorContent = useCallback(() => {
    if (!editorRef.current || !monacoRef.current) return
    
    const model = editorRef.current.getModel()
    if (!model) return
    
    // Build content from loaded lines
    const lines: string[] = []
    for (let i = 1; i <= totalLines; i++) {
      const content = contentMapRef.current.get(i)
      if (content !== undefined) {
        lines.push(content)
      } else {
        lines.push('') // Placeholder for unloaded lines
      }
    }
    
    const fullContent = lines.join('\n')
    model.setValue(fullContent)
    log(`[Editor::updateEditorContent] Set content: ${lines.length} lines`)
  }, [totalLines])

  // Handle scroll to load more content
  const handleScrollChange = useCallback((e: any) => {
    if (!editorRef.current || totalLines === 0) return
    
    const editor = editorRef.current
    const visibleRanges = editor.getVisibleRanges()
    if (visibleRanges.length === 0) return
    
    const lastVisibleLine = visibleRanges[visibleRanges.length - 1].endLineNumber
    const firstVisibleLine = visibleRanges[0].startLineNumber
    
    log(`[Editor::handleScrollChange] Visible: ${firstVisibleLine}-${lastVisibleLine}`)
    
    // Check if we need to load more lines at the bottom
    if (lastVisibleLine >= totalLines - 50 && lastVisibleLine < totalLines) {
      const loadStart = lastVisibleLine + 1
      const loadEnd = Math.min(loadStart + 99, totalLines)
      loadLineRange(loadStart, loadEnd)
    }
    
    // Check if we need to load more lines at the top
    if (firstVisibleLine <= 50 && firstVisibleLine > 1) {
      const loadEnd = firstVisibleLine - 1
      const loadStart = Math.max(loadEnd - 99, 1)
      loadLineRange(loadStart, loadEnd)
    }
  }, [totalLines, loadLineRange])

  const handleEditorDidMount = useCallback((editor: any, monaco: any) => {
    log(`[Editor::handleEditorDidMount] Monaco editor mounted`)
    
    editorRef.current = editor
    monacoRef.current = monaco
    
    // Configure editor
    editor.updateOptions({
      readOnly: false,
      minimap: { enabled: true },
      scrollBeyondLastLine: false,
      wordWrap: 'on',
      lineNumbers: 'on',
      renderWhitespace: 'selection',
      largeFileOptimizations: true,
    })
    
    // Listen for scroll changes
    editor.onDidScrollChange(handleScrollChange)
    
    setIsLoading(false)
    log(`[Editor::handleEditorDidMount] Editor ready`)
  }, [handleScrollChange])

  if (isLoading) {
    return <div className="editor-loading">Loading file info...</div>
  }

  if (error) {
    return <div className="editor-error">Error: {error}</div>
  }

  return (
    <div className="editor-wrapper">
      <div className="editor-info">
        <span>{filePath}</span>
        <span className="editor-stats">
          {totalLines.toLocaleString()} lines total | 
          {loadedRanges.length} chunks loaded
          {isModified && ' *'}
        </span>
      </div>
      <Editor
        height="calc(100% - 32px)"
        language="plaintext"
        value={''} // Content set via model after loading
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
