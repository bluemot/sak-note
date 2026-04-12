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
const INITIAL_CHUNK = 500       // Lines loaded initially
const SCROLL_CHUNK = 1000       // Lines loaded per expansion
const BUFFER_LINES = 150        // Extra lines beyond visible range
const MAX_MARKS_PER_HIGHLIGHT = 500

interface LoadedRange { start: number; end: number }

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
  const [content, setContent] = useState('')

  // Virtual scroll state
  const fileLineCountRef = useRef(0)
  const loadedEndLineRef = useRef(0)  // Last line that has real content (0-based, inclusive)
  const isLoadingMoreRef = useRef(false)
  const editorRef = useRef<any>(null)
  const monacoRef = useRef<any>(null)
  const decorationIdsRef = useRef<string[]>([])
  const marksRef = useRef<Mark[]>([])
  const loadedRangesRef = useRef<LoadedRange[]>([])

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

  const isRangeLoaded = useCallback((start: number, end: number): boolean => {
    return loadedRangesRef.current.some(r => r.start <= start && r.end >= end)
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
    } catch (err) { log('[Editor::refreshMarkDecorations] ERROR:', err) }
  }, [filePath])

  const applyVisibleMarkDecorations = useCallback(() => {
    const editor = editorRef.current
    const monaco = monacoRef.current
    if (!editor || !monaco) return
    const model = editor.getModel()
    if (!model) return
    const visibleRanges = editor.getVisibleRanges()
    if (!visibleRanges || visibleRanges.length === 0) return
    const startLine = Math.max(1, visibleRanges[0].startLineNumber - 20)
    const endLine = visibleRanges[visibleRanges.length - 1].endLineNumber + 20
    const newDecorations: any[] = []
    for (const mark of marksRef.current) {
      const startPos = model.getPositionAt(mark.start)
      const endPos = model.getPositionAt(mark.end)
      if (!startPos || !endPos) continue
      if (startPos.lineNumber >= startLine && startPos.lineNumber <= endLine) {
        newDecorations.push({
          range: new monaco.Range(startPos.lineNumber, startPos.column, endPos.lineNumber, endPos.column),
          options: { inlineClassName: `mark-highlight-${mark.color}`, isWholeLine: false, hoverMessage: { value: mark.label || mark.color } }
        })
      }
    }
    decorationIdsRef.current = editor.deltaDecorations(decorationIdsRef.current || [], newDecorations)
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
      const limited = searchResults.results.slice(0, MAX_MARKS_PER_HIGHLIGHT)
      for (const r of limited) {
        try { await invoke('create_mark', { req: { path: filePath, start: r.offset, end: r.offset + r.length, color, label: selectedText, note: null } }) }
        catch (e) { log('[Editor::handleCreateMark] Failed:', e) }
      }
      await refreshMarkDecorations()
    } catch (e) { log('[Editor::handleCreateMark] Search failed:', e) }
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

  useEffect(() => { if (onNavigateToMark) (window as any).__sakNavigateToMark = navigateToMarkPosition }, [onNavigateToMark, navigateToMarkPosition])

  useEffect(() => {
    const handler = (e: Event) => { const color = (e as CustomEvent).detail?.color; if (color) handleCreateMark(color) }
    window.addEventListener('sak-mark-highlight', handler)
    return () => window.removeEventListener('sak-mark-highlight', handler)
  }, [handleCreateMark])

  // --- Load more content at the bottom of the model ---
  const loadMoreContent = useCallback(async (fromLine: number, toLine: number) => {
    const editor = editorRef.current
    const monaco = monacoRef.current
    if (!editor || !monaco || !filePath) return
    if (isLoadingMoreRef.current) return
    isLoadingMoreRef.current = true

    try {
      const text = await invoke<string>('get_lines', {
        req: { path: filePath, start_line: fromLine, end_line: toLine + 1 }
      })
      const model = editor.getModel()
      if (!model) { isLoadingMoreRef.current = false; return }

      // Append content at the end of the model
      const lastLine = model.getLineCount()
      const lastCol = model.getLineMaxColumn(lastLine)

      editor.executeEdits('virtual-scroll-append', [{
        range: new monaco.Range(lastLine, lastCol, lastLine, lastCol),
        text: '\n' + text,
        forceMoveMarkers: false,
      }])

      loadedEndLineRef.current = toLine
      addLoadedRange(fromLine, toLine)
      log(`[Editor::loadMoreContent] Appended lines ${fromLine}-${toLine} (model now has ${model.getLineCount()} lines)`)
    } catch (err) {
      log('[Editor::loadMoreContent] ERROR:', err)
    } finally {
      isLoadingMoreRef.current = false
    }
  }, [filePath, addLoadedRange])

  // --- Replace placeholder lines with real content (for non-sequential scrolling) ---
  const loadVisibleContent = useCallback(async (startLine: number, endLine: number) => {
    const editor = editorRef.current
    const monaco = monacoRef.current
    if (!editor || !monaco || !filePath) return
    if (isLoadingMoreRef.current) return
    isLoadingMoreRef.current = true
    try {
      const text = await invoke<string>('get_lines', {
        req: { path: filePath, start_line: startLine, end_line: endLine + 1 }
      })
      const model = editor.getModel()
      if (!model) { isLoadingMoreRef.current = false; return }
      const returnedLines = text.split('\n').length
      const startLineNumber = startLine + 1
      const endLineNumber = Math.min(startLine + returnedLines, model.getLineCount())
      if (startLineNumber > model.getLineCount()) { isLoadingMoreRef.current = false; return }
      editor.executeEdits('virtual-scroll-replace', [{
        range: new monaco.Range(startLineNumber, 1, endLineNumber, model.getLineMaxColumn(endLineNumber)),
        text: text.endsWith('\n') ? text : text + '\n',
        forceMoveMarkers: false,
      }])
      addLoadedRange(startLine, endLine)
      log(`[Editor::loadVisibleContent] Replaced lines ${startLine}-${endLine}`)
    } catch (err) {
      log('[Editor::loadVisibleContent] ERROR:', err)
    } finally {
      isLoadingMoreRef.current = false
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
        loadedRangesRef.current = []
        loadedEndLineRef.current = 0

        // Step 1: Build line index first (O(1) line count after this)
        await invoke('build_line_index', { path: filePath }).catch(err => {
          log('[Editor::init] Line index build failed (non-fatal):', err)
        })

        // Step 2: Get file info (uses indexed line count = O(1))
        const fileInfo = await invoke<{ line_count: number }>('get_file_info', { path: filePath })
          .catch(() => ({ line_count: 0 }))
        const totalLines = fileInfo.line_count || 0
        fileLineCountRef.current = totalLines
        log(`[Editor::init] File has ${totalLines} lines`)

        if (totalLines <= 0) {
          setContent('')
          setIsLoading(false)
          return
        }

        // Step 3: Load ONLY the first chunk of real content
        // Do NOT create placeholder lines for the entire file
        // Instead, we'll expand the model as the user scrolls
        const firstChunkEnd = Math.min(INITIAL_CHUNK, totalLines)
        const firstChunk = await invoke<string>('get_lines', {
          req: { path: filePath, start_line: 0, end_line: firstChunkEnd }
        })

        addLoadedRange(0, firstChunkEnd - 1)
        loadedEndLineRef.current = firstChunkEnd - 1
        setContent(firstChunk)
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
  }, [filePath, addLoadedRange])

  // Scroll handler: load more content when near bottom
  const handleScroll = useCallback(async (_e: any) => {
    if (!editorRef.current || !monacoRef.current || isLoadingMoreRef.current) return
    const editor = editorRef.current
    const model = editor.getModel()
    if (!model) return

    const visibleRanges = editor.getVisibleRanges()
    if (!visibleRanges || visibleRanges.length === 0) return

    const lastVisibleLine = visibleRanges[visibleRanges.length - 1].endLineNumber
    const modelLineCount = model.getLineCount()
    const totalLines = fileLineCountRef.current

    // If user scrolled near the bottom of the loaded content, load more
    if (lastVisibleLine >= modelLineCount - BUFFER_LINES && loadedEndLineRef.current < totalLines - 1) {
      const fromLine = loadedEndLineRef.current + 1
      const toLine = Math.min(fromLine + SCROLL_CHUNK, totalLines - 1)
      log(`[Editor::handleScroll] Near bottom (line ${lastVisibleLine}/${modelLineCount}), loading lines ${fromLine}-${toLine}`)
      await loadMoreContent(fromLine, toLine)
    }

    // Also check if visible range has unloaded content (for jumping/scrolling to middle)
    const firstVisibleLine = visibleRanges[0].startLineNumber
    if (!isRangeLoaded(firstVisibleLine - 1, lastVisibleLine - 1)) {
      const loadStart = Math.max(0, firstVisibleLine - 1 - BUFFER_LINES)
      const loadEnd = Math.min(totalLines - 1, lastVisibleLine - 1 + BUFFER_LINES)
      // For non-sequential access, we need the model to be big enough
      // Extend the model if needed
      if (loadEnd > loadedEndLineRef.current) {
        await loadMoreContent(loadedEndLineRef.current + 1, loadEnd)
      }
      // Also replace placeholder lines in the visible range
      if (!isRangeLoaded(firstVisibleLine - 1, lastVisibleLine - 1)) {
        await loadVisibleContent(loadStart, loadEnd)
      }
    }

    debouncedApplyMarks()
  }, [loadMoreContent, loadVisibleContent, isRangeLoaded, debouncedApplyMarks])

  const checkEditStatus = useCallback(async () => {
    try {
      const status = await invoke<{ has_changes: boolean; can_undo: boolean; can_redo: boolean }>('get_edit_status', { path: filePath })
      setIsModified(status.has_changes)
      setCanUndo(status.can_undo)
      setCanRedo(status.can_redo)
    } catch (err) { log('[Editor::checkEditStatus] ERROR:', err) }
  }, [filePath])

  const reloadContent = useCallback(async () => {
    try {
      const fileInfo = await invoke<{ line_count: number }>('get_file_info', { path: filePath }).catch(() => ({ line_count: fileLineCountRef.current }))
      fileLineCountRef.current = fileInfo.line_count || fileLineCountRef.current
      loadedRangesRef.current = []
      loadedEndLineRef.current = 0
      const editor = editorRef.current
      if (!editor) return
      const firstChunkEnd = Math.min(INITIAL_CHUNK, fileLineCountRef.current)
      const text = await invoke<string>('get_lines', { req: { path: filePath, start_line: 0, end_line: firstChunkEnd } })
      addLoadedRange(0, firstChunkEnd - 1)
      loadedEndLineRef.current = firstChunkEnd - 1
      setContent(text)
    } catch (err) { log('[Editor::reloadContent] ERROR:', err) }
  }, [filePath, addLoadedRange])

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
    setTimeout(() => refreshMarkDecorations(), 500)
  }, [handleScroll, checkEditStatus, handleCreateMark, refreshMarkDecorations])

  const handleUndo = useCallback(async () => {
    try {
      if (await invoke<boolean>('undo', { path: filePath })) { await reloadContent(); await checkEditStatus() }
    } catch (err) { log('[Editor::undo] ERROR:', err) }
  }, [filePath, reloadContent, checkEditStatus])

  const handleRedo = useCallback(async () => {
    try {
      if (await invoke<boolean>('redo', { path: filePath })) { await reloadContent(); await checkEditStatus() }
    } catch (err) { log('[Editor::redo] ERROR:', err) }
  }, [filePath, reloadContent, checkEditStatus])

  const handleSave = useCallback(async () => {
    setIsSaving(true)
    try {
      await invoke('save_file', { path: filePath })
      setIsModified(false)
      await checkEditStatus()
    } catch (err) { setError(err instanceof Error ? err.message : 'Save failed') }
    finally { setIsSaving(false) }
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
      'log': 'plaintext',
    }
    return langMap[ext || ''] || 'plaintext'
  }

  if (isLoading) return <div className="editor-loading">Loading file...</div>
  if (error) return <div className="editor-error"><div className="error-message">Error loading file: {error}</div></div>

  const modelLineCount = editorRef.current?.getModel()?.getLineCount() || 0
  const totalLines = fileLineCountRef.current

  return (
    <div className="editor-wrapper">
      <div className="editor-toolbar">
        <div className="editor-info">
          <span className={isModified ? 'modified' : ''}>{filePath}{isModified && ' *'}</span>
          <span className="editor-stats">
            Lines {modelLineCount.toLocaleString()}{totalLines > modelLineCount ? ` (loading... ${totalLines.toLocaleString()} total)` : ` of ${totalLines.toLocaleString()}`}
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