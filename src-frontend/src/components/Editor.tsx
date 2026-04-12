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
const BUFFER_LINES = 150  // Lines to load beyond visible area
const MAX_MARKS_PER_HIGHLIGHT = 500

// Loaded range tracking
interface LoadedRange {
  start: number  // 0-based line number
  end: number    // 0-based line number (inclusive)
}

// Helper for formatted logging
const log = (msg: string, ...args: any[]) => {
  console.log(`[${new Date().toISOString()}] ${msg}`, ...args)
}

// Debounce utility
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

function FileEditor({ filePath, fileSize, onNavigateToMark }: EditorProps) {
  const [content, setContent] = useState('')
  const [isLoading, setIsLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [isModified, setIsModified] = useState(false)
  const [canUndo, setCanUndo] = useState(false)
  const [canRedo, setCanRedo] = useState(false)
  const [isSaving, setIsSaving] = useState(false)

  // Virtual scroll state
  const [fileLineCount, setFileLineCount] = useState(0)
  const [, setCurrentLine] = useState(0)

  const editorRef = useRef<any>(null)
  const monacoRef = useRef<any>(null)
  const isLoadingContentRef = useRef(false)
  const decorationIdsRef = useRef<string[]>([])
  const marksRef = useRef<Mark[]>([])
  const loadedRangesRef = useRef<LoadedRange[]>([])
  const initialContentLoadedRef = useRef(false)
  const fileLineCountRef = useRef(0)

  // Keep ref in sync with state
  useEffect(() => {
    fileLineCountRef.current = fileLineCount
  }, [fileLineCount])

  // --- Loaded range management ---

  const isRangeLoaded = useCallback((start: number, end: number): boolean => {
    return loadedRangesRef.current.some(
      range => range.start <= start && range.end >= end
    )
  }, [])

  const addLoadedRange = useCallback((start: number, end: number) => {
    loadedRangesRef.current.push({ start, end })
    // Merge overlapping ranges
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

  // --- Mark decoration helpers ---

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
      log(`[Editor::refreshMarkDecorations] ERROR:`, err)
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
          range: new monaco.Range(
            startPos.lineNumber,
            startPos.column,
            endPos.lineNumber,
            endPos.column
          ),
          options: {
            inlineClassName: `mark-highlight-${mark.color}`,
            isWholeLine: false,
            hoverMessage: { value: mark.label || mark.color },
          }
        })
      }
    }

    decorationIdsRef.current = editor.deltaDecorations(
      decorationIdsRef.current || [],
      newDecorations
    )

    log(`[Editor::applyVisibleMarkDecorations] Applied ${newDecorations.length} visible mark decorations (total: ${marksRef.current.length})`)
  }, [])

  const debouncedApplyMarks = useDebounce(applyVisibleMarkDecorations, 100)

  // Create marks for all occurrences of selected text using backend search
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
      const searchResults = await invoke<{ results: Array<{ offset: number; length: number; preview: string }>; total: number }>('search', {
        req: {
          path: filePath,
          pattern: selectedText,
          is_hex: false,
          start_offset: 0,
        }
      })

      const totalMatches = searchResults.total
      const limitedResults = searchResults.results.slice(0, MAX_MARKS_PER_HIGHLIGHT)

      log(`[Editor::handleCreateMark] Backend found ${totalMatches} occurrences for "${selectedText}" with color ${color}`)

      if (totalMatches > MAX_MARKS_PER_HIGHLIGHT) {
        log(`[Editor::handleCreateMark] WARNING: Found ${totalMatches} matches, highlighting first ${MAX_MARKS_PER_HIGHLIGHT}`)
      }

      for (const result of limitedResults) {
        try {
          await invoke('create_mark', {
            req: {
              path: filePath,
              start: result.offset,
              end: result.offset + result.length,
              color,
              label: selectedText,
              note: null,
            }
          })
        } catch (err) {
          log(`[Editor::handleCreateMark] Failed to create mark:`, err)
        }
      }

      await refreshMarkDecorations()
    } catch (err) {
      log(`[Editor::handleCreateMark] Backend search failed:`, err)
    }
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
    if (onNavigateToMark) {
      ;(window as any).__sakNavigateToMark = navigateToMarkPosition
    }
  }, [onNavigateToMark, navigateToMarkPosition])

  useEffect(() => {
    const handleHighlightEvent = (e: Event) => {
      const customEvent = e as CustomEvent
      const color = customEvent.detail?.color as string
      if (color) {
        handleCreateMark(color)
      }
    }
    window.addEventListener('sak-mark-highlight', handleHighlightEvent)
    return () => window.removeEventListener('sak-mark-highlight', handleHighlightEvent)
  }, [handleCreateMark])

  // --- Virtual scroll: load content on demand ---

  const loadVisibleContent = useCallback(async (
    startLine: number,  // 0-based
    endLine: number      // 0-based, inclusive
  ) => {
    const editor = editorRef.current
    const monaco = monacoRef.current
    if (!editor || !monaco || !filePath) return

    const model = editor.getModel()
    if (!model) return

    if (isLoadingContentRef.current) return
    isLoadingContentRef.current = true

    try {
      // Fetch real content from backend
      // get_lines: start_line is 0-based, end_line is exclusive (0-based + 1)
      const text = await invoke<string>('get_lines', {
        req: {
          path: filePath,
          start_line: startLine,
          end_line: endLine + 1,  // Backend end_line is exclusive
        }
      })

      // Calculate how many lines the returned text contains
      const returnedLines = text.split('\n').length
      // The text may or may not end with \n. If it does, the last element is empty.
      // We need to figure out the actual number of content lines to replace.

      // Monaco line numbers are 1-based
      const startLineNumber = startLine + 1  // Convert 0-based to 1-based
      // Ensure endLineNumber doesn't exceed model line count
      const endLineNumber = Math.min(startLine + returnedLines, model.getLineCount())

      if (startLineNumber > model.getLineCount()) {
        isLoadingContentRef.current = false
        return
      }

      // Replace the lines in the model
      editor.executeEdits('virtual-scroll', [{
        range: new monaco.Range(
          startLineNumber,
          1,
          endLineNumber,
          model.getLineMaxColumn(endLineNumber)
        ),
        text: text.endsWith('\n') ? text : text + '\n',
        forceMoveMarkers: false,
      }])

      addLoadedRange(startLine, endLine)
      log(`[Editor::loadVisibleContent] Loaded lines ${startLine}-${endLine} (${text.length} chars, ${returnedLines} lines)`)
    } catch (err) {
      log(`[Editor::loadVisibleContent] ERROR:`, err)
    } finally {
      isLoadingContentRef.current = false
    }
  }, [filePath, addLoadedRange])

  // Initial file load
  useEffect(() => {
    log(`[Editor::useEffect] === VIRTUAL SCROLL INITIALIZATION ===`)
    log(`[Editor::useEffect] filePath="${filePath}", fileSize=${fileSize}`)

    const initializeEditor = async () => {
      try {
        setIsLoading(true)
        setError(null)
        initialContentLoadedRef.current = false
        loadedRangesRef.current = []

        // Get total line count from backend
        const fileInfo = await invoke<{ line_count: number }>('get_file_info', {
          path: filePath
        }).catch(() => ({ line_count: 0 }))

        const totalLines = fileInfo.line_count || 0
        setFileLineCount(totalLines)
        log(`[Editor::initializeEditor] File has ${totalLines} lines`)

        // Build line index in backend for O(1) line lookups
        invoke('build_line_index', { path: filePath }).catch(err => {
          log('[Editor::initializeEditor] Line index build failed (non-fatal):', err)
        })

        // Create placeholder content efficiently
        // Instead of joining 1M empty strings, use a single string with newlines
        let placeholderContent: string
        if (totalLines <= 0) {
          placeholderContent = ''
        } else if (totalLines <= 10000) {
          // Small files: create all placeholder lines
          placeholderContent = Array(totalLines).fill('').join('\n')
        } else {
          // Large files: just set the line count via model, load first chunk immediately
          // Create a minimal placeholder: first few real lines + empty lines for the rest
          // Load the first 500 lines immediately
          const firstChunk = await invoke<string>('get_lines', {
            req: {
              path: filePath,
              start_line: 0,
              end_line: 500,
            }
          })
          const firstChunkLines = firstChunk.split('\n').length
          const remainingLines = totalLines - firstChunkLines
          if (remainingLines > 0) {
            placeholderContent = firstChunkLines + firstChunk + '\n' + Array(remainingLines).fill('').join('\n')
          } else {
            placeholderContent = firstChunk
          }
          // Mark first chunk as loaded
          addLoadedRange(0, Math.min(499, totalLines - 1))
          setFileLineCount(totalLines)
          setContent(placeholderContent)
          setIsLoading(false)
          return
        }

        setContent(placeholderContent)
      } catch (err) {
        const errorMsg = err instanceof Error ? err.message : String(err)
        log(`[Editor::initializeEditor] ERROR:`, errorMsg)
        setError(errorMsg)
      } finally {
        setIsLoading(false)
      }
    }

    if (filePath) {
      initializeEditor()
    }

    return () => {
      log(`[Editor::useEffect] Cleanup - filePath changed or component unmounted`)
    }
  }, [filePath])

  // After editor mounts, load visible content (for small files or as fallback)
  const loadInitialVisibleContent = useCallback(async () => {
    if (initialContentLoadedRef.current) return

    const editor = editorRef.current
    if (!editor) return

    const model = editor.getModel()
    if (!model) return

    // Check if we already loaded content (large file path)
    if (loadedRangesRef.current.length > 0) {
      initialContentLoadedRef.current = true
      // Just refresh marks
      await refreshMarkDecorations()
      return
    }

    initialContentLoadedRef.current = true

    // Small file: load first chunk
    const loadEnd = Math.min(BUFFER_LINES * 3, fileLineCountRef.current - 1)
    await loadVisibleContent(0, loadEnd)

    // Load marks
    await refreshMarkDecorations()
  }, [loadVisibleContent, refreshMarkDecorations])

  // Handle scroll: load content on demand
  const handleScroll = useCallback(async (_e: any) => {
    if (!editorRef.current || !monacoRef.current || isLoadingContentRef.current) return

    const editor = editorRef.current
    const model = editor.getModel()
    if (!model) return

    const visibleRanges = editor.getVisibleRanges()
    if (!visibleRanges || visibleRanges.length === 0) return

    const firstVisible = visibleRanges[0].startLineNumber - 1  // 0-based
    const lastVisible = visibleRanges[visibleRanges.length - 1].endLineNumber - 1  // 0-based

    setCurrentLine(lastVisible)

    const loadStart = Math.max(0, firstVisible - BUFFER_LINES)
    const loadEnd = Math.min(fileLineCountRef.current - 1, lastVisible + BUFFER_LINES)

    if (!isRangeLoaded(loadStart, loadEnd)) {
      await loadVisibleContent(loadStart, loadEnd)
    }

    debouncedApplyMarks()
  }, [isRangeLoaded, loadVisibleContent, debouncedApplyMarks])

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
      const fileInfo = await invoke<{ line_count: number }>('get_file_info', {
        path: filePath
      }).catch(() => ({ line_count: fileLineCountRef.current }))

      const newLineCount = fileInfo.line_count || fileLineCountRef.current
      setFileLineCount(newLineCount)
      fileLineCountRef.current = newLineCount

      // Reset loaded ranges and reload visible content
      loadedRangesRef.current = []

      const editor = editorRef.current
      if (!editor) return

      const visibleRanges = editor.getVisibleRanges()
      if (!visibleRanges || visibleRanges.length === 0) return

      const firstLine = visibleRanges[0].startLineNumber - 1
      const lastLine = visibleRanges[visibleRanges.length - 1].endLineNumber - 1

      const loadStart = Math.max(0, firstLine - BUFFER_LINES)
      const loadEnd = Math.min(newLineCount - 1, lastLine + BUFFER_LINES)

      await loadVisibleContent(loadStart, loadEnd)
    } catch (err) {
      log(`[Editor::reloadContent] ERROR:`, err)
    }
  }, [filePath, loadVisibleContent])

  // Handle content changes from Monaco
  const handleEditorChange = useCallback((value: string | undefined) => {
    if (!value || !editorRef.current) return
    // For virtual scroll, we track changes but don't send edits yet
    // TODO: implement proper diff-based editing for large files
    setContent(value || '')
  }, [])

  const handleEditorDidMount = useCallback((editor: any, monaco: any) => {
    log(`[Editor::handleEditorDidMount] Monaco editor mounted`)

    editorRef.current = editor
    monacoRef.current = monaco

    // Configure editor for large files
    editor.updateOptions({
      readOnly: false,
      minimap: { enabled: true },
      scrollBeyondLastLine: false,
      wordWrap: 'on',
      lineNumbers: 'on',
      renderWhitespace: 'selection',
      largeFileOptimizations: true,
    })

    // Scroll handler for virtual scroll
    editor.onDidScrollChange((e: any) => {
      if (e.scrollTopChanged) {
        handleScroll(e)
      }
    })

    // Register context menu actions for mark colors
    MARK_COLORS.forEach((color, index) => {
      editor.addAction({
        id: `mark-${color}`,
        label: `Mark: ${color.charAt(0).toUpperCase() + color.slice(1)}`,
        contextMenuGroupId: 'marks',
        contextMenuOrder: index + 1,
        run: () => {
          handleCreateMark(color)
        }
      })
    })

    // Check initial edit status
    checkEditStatus()

    log(`[Editor::handleEditorDidMount] Editor initialized with virtual scroll and mark context menu`)

    // Load initial visible content after mount
    setTimeout(() => loadInitialVisibleContent(), 100)
  }, [handleScroll, checkEditStatus, handleCreateMark, loadInitialVisibleContent])

  // Handle undo
  const handleUndo = useCallback(async () => {
    log(`[Editor::handleUndo] Undo triggered`)
    try {
      const success = await invoke<boolean>('undo', { path: filePath })
      if (success) {
        log(`[Editor::handleUndo] Undo successful`)
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

  // Keyboard shortcuts
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key === 's') {
        log(`[Editor::handleKeyDown] Save shortcut triggered`)
        e.preventDefault()
        handleSave()
      } else if ((e.metaKey || e.ctrlKey) && e.key === 'z' && !e.shiftKey) {
        log(`[Editor::handleKeyDown] Undo shortcut triggered`)
        e.preventDefault()
        handleUndo()
      } else if ((e.metaKey || e.ctrlKey) && (e.key === 'y' || (e.key === 'z' && e.shiftKey))) {
        log(`[Editor::handleKeyDown] Redo shortcut triggered`)
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
      'js': 'javascript', 'ts': 'typescript', 'jsx': 'javascript', 'tsx': 'typescript',
      'py': 'python', 'rs': 'rust', 'html': 'html', 'css': 'css', 'json': 'json',
      'md': 'markdown', 'txt': 'plaintext', 'c': 'c', 'cpp': 'cpp', 'h': 'c',
      'hpp': 'cpp', 'java': 'java', 'go': 'go', 'rb': 'ruby', 'php': 'php',
      'sh': 'shell', 'yaml': 'yaml', 'yml': 'yaml', 'toml': 'toml',
    }
    return langMap[ext || ''] || 'plaintext'
  }

  // Calculate display range for status bar
  const getDisplayRange = () => {
    const editor = editorRef.current
    if (!editor || !editor.getModel()) {
      return { start: 0, end: 0 }
    }

    const visibleRanges = editor.getVisibleRanges()
    if (!visibleRanges || visibleRanges.length === 0) {
      return { start: 0, end: 0 }
    }

    return {
      start: visibleRanges[0].startLineNumber,
      end: visibleRanges[visibleRanges.length - 1].endLineNumber,
    }
  }

  if (isLoading) {
    return <div className="editor-loading">Loading file...</div>
  }

  if (error) {
    return (
      <div className="editor-error">
        <div className="error-message">Error loading file: {error}</div>
      </div>
    )
  }

  const detectedLang = getLanguage(filePath)
  const displayRange = getDisplayRange()

  return (
    <div className="editor-wrapper">
      <div className="editor-toolbar">
        <div className="editor-info">
          <span className={isModified ? 'modified' : ''}>
            {filePath}{isModified && ' *'}
          </span>
          <span className="editor-stats">
            Lines {displayRange.start.toLocaleString()} - {displayRange.end.toLocaleString()}
            {fileLineCount > 0 && ` of ${fileLineCount.toLocaleString()}`}
          </span>
        </div>
        <div className="editor-actions">
          <button onClick={handleUndo} disabled={!canUndo || isSaving} title="Undo (Ctrl+Z)" className="editor-btn">
            Undo
          </button>
          <button onClick={handleRedo} disabled={!canRedo || isSaving} title="Redo (Ctrl+Y)" className="editor-btn">
            Redo
          </button>
          <button onClick={handleSave} disabled={isSaving || !isModified} title="Save (Ctrl+S)"
            className={`editor-btn save ${isModified ? 'modified' : ''}`}>
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