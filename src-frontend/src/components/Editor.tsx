import { useEffect, useRef, useState, useCallback } from 'react'
import Editor from '@monaco-editor/react'
import { invoke } from '@tauri-apps/api/core'
import './Editor.css'

interface EditorProps {
  filePath: string
  fileSize: number
  onNavigateToMark?: (start: number) => void
}

// Mark type matching backend
interface Mark {
  id: string
  start: number
  end: number
  color: 'red' | 'orange' | 'yellow' | 'green' | 'cyan' | 'blue' | 'purple' | 'pink' | 'gray'
  label?: string
  note?: string
  created_at: number
  updated_at: number
}

const MARK_COLORS: Mark['color'][] = ['red', 'orange', 'yellow', 'green', 'cyan', 'blue', 'purple', 'pink', 'gray']

// Virtual scroll configuration
const BUFFER_LINES = 150
const MAX_MARKS_PER_HIGHLIGHT = 500

// Loaded range tracking
interface LoadedRange {
  start: number
  end: number
}

const log = (msg: string, ...args: any[]) => {
  console.log(`[${new Date().toISOString()}] ${msg}`, ...args)
}

function useDebounce<T extends (...args: any[]) => void>(callback: T, delay: number) {
  const timeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null)
  return useCallback((...args: Parameters<T>) => {
    if (timeoutRef.current) clearTimeout(timeoutRef.current)
    timeoutRef.current = setTimeout(() => callback(...args), delay)
  }, [callback, delay])
}

function FileEditor({ filePath: filePathProp, fileSize: _fileSize, onNavigateToMark }: EditorProps) {
  const filePath = filePathProp
  const [isLoading, setIsLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [isModified, setIsModified] = useState(false)
  const [canUndo, setCanUndo] = useState(false)
  const [canRedo, setCanRedo] = useState(false)
  const [isSaving, setIsSaving] = useState(false)
  const [fileLineCount, setFileLineCount] = useState(0)
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  const [, setCurrentLine] = useState(0)
  const [content, setContent] = useState('')

  const editorRef = useRef<any>(null)
  const monacoRef = useRef<any>(null)
  const isLoadingContentRef = useRef(false)
  const decorationIdsRef = useRef<string[]>([])
  const marksRef = useRef<Mark[]>([])
  const loadedRangesRef = useRef<LoadedRange[]>([])
  const initialContentLoadedRef = useRef(false)
  const fileLineCountRef = useRef(0)

  useEffect(() => { fileLineCountRef.current = fileLineCount }, [fileLineCount])

  const isRangeLoaded = useCallback((start: number, end: number): boolean => {
    return loadedRangesRef.current.some(r => r.start <= start && r.end >= end)
  }, [])

  const addLoadedRange = useCallback((start: number, end: number) => {
    loadedRangesRef.current.push({ start, end })
    const ranges = loadedRangesRef.current.sort((a, b) => a.start - b.start)
    const merged: LoadedRange[] = []
    for (const range of ranges) {
      if (merged.length > 0 && merged[merged.length - 1].end >= range.start - 1) {
        merged[merged.length - 1].end = Math.max(merged[merged.length - 1].end, range.end)
      } else {
        merged.push({ ...range })
      }
    }
    loadedRangesRef.current = merged
  }, [])

  // --- Mark decorations ---
  const refreshMarkDecorations = useCallback(async () => {
    const editor = editorRef.current
    const monaco = monacoRef.current
    if (!editor || !monaco || !filePath) return
    const model = editor.getModel()
    if (!model) return
    try {
      const response = await invoke<{ marks: Mark[] }>('get_marks', {
        req: { path: filePath, start: null, end: null }
      })
      marksRef.current = response.marks || []
      applyVisibleMarkDecorations()
    } catch (err) {
      log('[Editor::refreshMarkDecorations] ERROR:', err)
    }
  }, [filePath])

  const applyVisibleMarkDecorations = useCallback(() => {
    const editor = editorRef.current
    const monaco = monacoRef.current
    if (!editor || !monaco) return
    const model = editor.getModel()
    if (!model) return
    const visibleRanges = editor.getVisibleRanges()
    if (!visibleRanges || visibleRanges.length === 0) return
    const visibleStartLine = visibleRanges[0].startLineNumber
    const visibleEndLine = visibleRanges[visibleRanges.length - 1].endLineNumber
    const bufferLines = 20
    const filteredStartLine = Math.max(1, visibleStartLine - bufferLines)
    const filteredEndLine = visibleEndLine + bufferLines
    const newDecorations: any[] = []
    for (const mark of marksRef.current) {
      const startPos = model.getPositionAt(mark.start)
      const endPos = model.getPositionAt(mark.end)
      if (!startPos || !endPos) continue
      if (startPos.lineNumber >= filteredStartLine && startPos.lineNumber <= filteredEndLine) {
        newDecorations.push({
          range: new monaco.Range(startPos.lineNumber, startPos.column, endPos.lineNumber, endPos.column),
          options: { inlineClassName: `mark-highlight-${mark.color}`, isWholeLine: false, hoverMessage: { value: mark.label || mark.color } }
        })
      }
    }
    decorationIdsRef.current = editor.deltaDecorations(decorationIdsRef.current || [], newDecorations)
    log(`[Editor::applyVisibleMarkDecorations] Applied ${newDecorations.length} decorations`)
  }, [])

  const debouncedApplyMarks = useDebounce(applyVisibleMarkDecorations, 100)

  const handleCreateMark = useCallback(async (color: string) => {
    const editor = editorRef.current
    if (!editor || !filePath) return
    const selection = editor.getSelection()
    if (!selection || selection.isEmpty()) return
    const model = editor.getModel()
    if (!model) return
    const selectedText = model.getValueInRange(selection)
    if (!selectedText) return
    try {
      const searchResults = await invoke<{ results: Array<{ offset: number; length: number }>; total: number }>('search', {
        req: { path: filePath, pattern: selectedText, is_hex: false, start_offset: 0 }
      })
      const limitedResults = searchResults.results.slice(0, MAX_MARKS_PER_HIGHLIGHT)
      for (const result of limitedResults) {
        try {
          await invoke('create_mark', { req: { path: filePath, start: result.offset, end: result.offset + result.length, color, label: selectedText, note: null } })
        } catch (err) { log('[Editor::handleCreateMark] Failed:', err) }
      }
      await refreshMarkDecorations()
    } catch (err) { log('[Editor::handleCreateMark] Search failed:', err) }
  }, [filePath, refreshMarkDecorations])

  const navigateToMarkPosition = useCallback((startOffset: number) => {
    const editor = editorRef.current
    const monaco = monacoRef.current
    if (!editor || !monaco) return
    const model = editor.getModel()
    if (!model) return
    const pos = model.getPositionAt(startOffset)
    editor.setPosition(pos)
    editor.revealLineInCenter(pos.lineNumber)
    editor.focus()
  }, [])

  useEffect(() => {
    if (onNavigateToMark) (window as any).__sakNavigateToMark = navigateToMarkPosition
  }, [onNavigateToMark, navigateToMarkPosition])

  useEffect(() => {
    const handler = (e: Event) => { const color = (e as CustomEvent).detail?.color; if (color) handleCreateMark(color) }
    window.addEventListener('sak-mark-highlight', handler)
    return () => window.removeEventListener('sak-mark-highlight', handler)
  }, [handleCreateMark])

  // --- Load visible content on demand ---
  const loadVisibleContent = useCallback(async (startLine: number, endLine: number) => {
    const editor = editorRef.current
    const monaco = monacoRef.current
    if (!editor || !monaco || !filePath) return
    const model = editor.getModel()
    if (!model) return
    if (isLoadingContentRef.current) return
    isLoadingContentRef.current = true
    try {
      const text = await invoke<string>('get_lines', {
        req: { path: filePath, start_line: startLine, end_line: endLine + 1 }
      })
      const returnedLines = text.split('\n').length
      const startLineNumber = startLine + 1
      const endLineNumber = Math.min(startLine + returnedLines, model.getLineCount())
      if (startLineNumber > model.getLineCount()) { isLoadingContentRef.current = false; return }
      editor.executeEdits('virtual-scroll', [{
        range: new monaco.Range(startLineNumber, 1, endLineNumber, model.getLineMaxColumn(endLineNumber)),
        text: text.endsWith('\n') ? text : text + '\n',
        forceMoveMarkers: false,
      }])
      addLoadedRange(startLine, endLine)
      log(`[Editor::loadVisibleContent] Loaded lines ${startLine}-${endLine}`)
    } catch (err) {
      log('[Editor::loadVisibleContent] ERROR:', err)
    } finally {
      isLoadingContentRef.current = false
    }
  }, [filePath, addLoadedRange])

  // --- Initialize editor ---
  useEffect(() => {
    if (!filePath) return
    log(`[Editor::init] filePath="${filePath}"`)

    const initializeEditor = async () => {
      try {
        setIsLoading(true)
        setError(null)
        initialContentLoadedRef.current = false
        loadedRangesRef.current = []

        // Step 1: Build line index FIRST (so get_file_info uses O(1) line count)
        await invoke('build_line_index', { path: filePath }).catch(err => {
          log('[Editor::init] Line index build failed (non-fatal):', err)
        })

        // Step 2: Get file info (now uses indexed line count = O(1))
        const fileInfo = await invoke<{ line_count: number }>('get_file_info', { path: filePath })
          .catch(() => ({ line_count: 0 }))
        const totalLines = fileInfo.line_count || 0
        setFileLineCount(totalLines)
        fileLineCountRef.current = totalLines
        log(`[Editor::init] File has ${totalLines} lines`)

        if (totalLines <= 0) {
          setContent('')
          setIsLoading(false)
          return
        }

        // Step 3: Load first chunk of real content
        const chunkSize = Math.min(500, totalLines)
        const firstChunk = await invoke<string>('get_lines', {
          req: { path: filePath, start_line: 0, end_line: chunkSize }
        })

        // Step 4: Build full-height model efficiently
        // Use first chunk as real content + empty lines for the rest
        // Instead of Array(N).fill('').join('\n'), use repeated newline string
        const firstChunkLineCount = firstChunk.split('\n').length - (firstChunk.endsWith('\n') ? 1 : 0)
        const remainingLines = totalLines - firstChunkLineCount

        let placeholderContent: string
        if (remainingLines > 0) {
          // Efficient: one newline per remaining line, no array allocation
          placeholderContent = firstChunk + '\n'.repeat(remainingLines > 0 ? remainingLines : 0)
        } else {
          placeholderContent = firstChunk
        }

        addLoadedRange(0, Math.min(chunkSize - 1, totalLines - 1))
        setContent(placeholderContent)
      } catch (err) {
        const errorMsg = err instanceof Error ? err.message : String(err)
        log(`[Editor::init] ERROR:`, errorMsg)
        setError(errorMsg)
      } finally {
        setIsLoading(false)
      }
    }

    initializeEditor()
    return () => { log('[Editor::init] Cleanup') }
  }, [filePath])

  // Load remaining visible content after mount
  const loadInitialVisibleContent = useCallback(async () => {
    if (initialContentLoadedRef.current) return
    const editor = editorRef.current
    if (!editor) return
    const model = editor.getModel()
    if (!model) return

    // If first chunk already loaded, just load marks
    if (loadedRangesRef.current.length > 0) {
      initialContentLoadedRef.current = true
      await refreshMarkDecorations()
      return
    }

    initialContentLoadedRef.current = true
    const loadEnd = Math.min(BUFFER_LINES * 3, fileLineCountRef.current - 1)
    await loadVisibleContent(0, loadEnd)
    await refreshMarkDecorations()
  }, [loadVisibleContent, refreshMarkDecorations])

  // Scroll handler
  const handleScroll = useCallback(async (_e: any) => {
    if (!editorRef.current || !monacoRef.current || isLoadingContentRef.current) return
    const editor = editorRef.current
    const model = editor.getModel()
    if (!model) return
    const visibleRanges = editor.getVisibleRanges()
    if (!visibleRanges || visibleRanges.length === 0) return
    const firstVisible = visibleRanges[0].startLineNumber - 1
    const lastVisible = visibleRanges[visibleRanges.length - 1].endLineNumber - 1
    setCurrentLine(lastVisible)
    const loadStart = Math.max(0, firstVisible - BUFFER_LINES)
    const loadEnd = Math.min(fileLineCountRef.current - 1, lastVisible + BUFFER_LINES)
    if (!isRangeLoaded(loadStart, loadEnd)) {
      await loadVisibleContent(loadStart, loadEnd)
    }
    debouncedApplyMarks()
  }, [isRangeLoaded, loadVisibleContent, debouncedApplyMarks])

  const checkEditStatus = useCallback(async () => {
    try {
      const status = await invoke<{ has_changes: boolean; can_undo: boolean; can_redo: boolean; effective_size: number }>('get_edit_status', { path: filePath })
      setIsModified(status.has_changes)
      setCanUndo(status.can_undo)
      setCanRedo(status.can_redo)
    } catch (err) { log('[Editor::checkEditStatus] ERROR:', err) }
  }, [filePath])

  const reloadContent = useCallback(async () => {
    try {
      const fileInfo = await invoke<{ line_count: number }>('get_file_info', { path: filePath }).catch(() => ({ line_count: fileLineCountRef.current }))
      const newLineCount = fileInfo.line_count || fileLineCountRef.current
      setFileLineCount(newLineCount)
      fileLineCountRef.current = newLineCount
      loadedRangesRef.current = []
      const editor = editorRef.current
      if (!editor) return
      const visibleRanges = editor.getVisibleRanges()
      if (!visibleRanges || visibleRanges.length === 0) return
      const firstLine = visibleRanges[0].startLineNumber - 1
      const lastLine = visibleRanges[visibleRanges.length - 1].endLineNumber - 1
      await loadVisibleContent(Math.max(0, firstLine - BUFFER_LINES), Math.min(newLineCount - 1, lastLine + BUFFER_LINES))
    } catch (err) { log('[Editor::reloadContent] ERROR:', err) }
  }, [filePath, loadVisibleContent])

  const handleEditorChange = useCallback((value: string | undefined) => {
    if (!value) return
    setContent(value)
  }, [])

  const handleEditorDidMount = useCallback((editor: any, monaco: any) => {
    log('[Editor::mount] Monaco editor mounted')
    editorRef.current = editor
    monacoRef.current = monaco

    editor.updateOptions({
      readOnly: false,
      minimap: { enabled: true },
      scrollBeyondLastLine: false,
      wordWrap: 'on',
      lineNumbers: 'on',
      renderWhitespace: 'selection',
      largeFileOptimizations: true,
    })

    editor.onDidScrollChange((e: any) => {
      if (e.scrollTopChanged) handleScroll(e)
    })

    MARK_COLORS.forEach((color, index) => {
      editor.addAction({
        id: `mark-${color}`,
        label: `Mark: ${color.charAt(0).toUpperCase() + color.slice(1)}`,
        contextMenuGroupId: 'marks',
        contextMenuOrder: index + 1,
        run: () => handleCreateMark(color)
      })
    })

    checkEditStatus()
    setTimeout(() => loadInitialVisibleContent(), 100)
  }, [handleScroll, checkEditStatus, handleCreateMark, loadInitialVisibleContent])

  const handleUndo = useCallback(async () => {
    try {
      const success = await invoke<boolean>('undo', { path: filePath })
      if (success) { await reloadContent(); await checkEditStatus() }
    } catch (err) { log('[Editor::undo] ERROR:', err) }
  }, [filePath, reloadContent, checkEditStatus])

  const handleRedo = useCallback(async () => {
    try {
      const success = await invoke<boolean>('redo', { path: filePath })
      if (success) { await reloadContent(); await checkEditStatus() }
    } catch (err) { log('[Editor::redo] ERROR:', err) }
  }, [filePath, reloadContent, checkEditStatus])

  const handleSave = useCallback(async () => {
    setIsSaving(true)
    try {
      await invoke('save_file', { path: filePath })
      setIsModified(false)
      await checkEditStatus()
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Save failed')
    } finally {
      setIsSaving(false)
    }
  }, [filePath, checkEditStatus])

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key === 's') { e.preventDefault(); handleSave() }
      else if ((e.metaKey || e.ctrlKey) && e.key === 'z' && !e.shiftKey) { e.preventDefault(); handleUndo() }
      else if ((e.metaKey || e.ctrlKey) && (e.key === 'y' || (e.key === 'z' && e.shiftKey))) { e.preventDefault(); handleRedo() }
    }
    window.addEventListener('keydown', handleKeyDown)
    return () => window.removeEventListener('keydown', handleKeyDown)
  }, [handleSave, handleUndo, handleRedo])

  const getLanguage = (path: string): string => {
    const ext = path.split('.').pop()?.toLowerCase()
    const langMap: Record<string, string> = {
      'js': 'javascript', 'ts': 'typescript', 'jsx': 'javascript', 'tsx': 'typescript',
      'py': 'python', 'rs': 'rust', 'html': 'html', 'css': 'css', 'json': 'json',
      'md': 'markdown', 'txt': 'plaintext', 'c': 'c', 'cpp': 'cpp', 'h': 'c',
      'hpp': 'cpp', 'java': 'java', 'go': 'go', 'rb': 'ruby', 'php': 'php',
      'sh': 'shell', 'yaml': 'yaml', 'yml': 'yaml', 'toml': 'toml',
    }
    return langMap[ext || ''] || 'plaintext'
  }

  const getDisplayRange = () => {
    const editor = editorRef.current
    if (!editor || !editor.getModel()) return { start: 0, end: 0 }
    const visibleRanges = editor.getVisibleRanges()
    if (!visibleRanges || visibleRanges.length === 0) return { start: 0, end: 0 }
    return { start: visibleRanges[0].startLineNumber, end: visibleRanges[visibleRanges.length - 1].endLineNumber }
  }

  if (isLoading) return <div className="editor-loading">Loading file...</div>
  if (error) return <div className="editor-error"><div className="error-message">Error loading file: {error}</div></div>

  const displayRange = getDisplayRange()

  return (
    <div className="editor-wrapper">
      <div className="editor-toolbar">
        <div className="editor-info">
          <span className={isModified ? 'modified' : ''}>{filePath}{isModified && ' *'}</span>
          <span className="editor-stats">
            Lines {displayRange.start.toLocaleString()} - {displayRange.end.toLocaleString()}
            {fileLineCount > 0 && ` of ${fileLineCount.toLocaleString()}`}
          </span>
        </div>
        <div className="editor-actions">
          <button onClick={handleUndo} disabled={!canUndo || isSaving} title="Undo (Ctrl+Z)" className="editor-btn">Undo</button>
          <button onClick={handleRedo} disabled={!canRedo || isSaving} title="Redo (Ctrl+Y)" className="editor-btn">Redo</button>
          <button onClick={handleSave} disabled={isSaving || !isModified} title="Save (Ctrl+S)" className={`editor-btn save ${isModified ? 'modified' : ''}`}>
            {isSaving ? 'Saving...' : 'Save'}
          </button>
        </div>
      </div>
      <Editor
        height="calc(100% - 48px)"
        language={getLanguage(filePath)}
        value={content}
        theme="vs-dark"
        onMount={handleEditorDidMount}
        onChange={handleEditorChange}
        options={{ selectOnLineNumbers: true, automaticLayout: true }}
      />
    </div>
  )
}

export default FileEditor