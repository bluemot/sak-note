import { useEffect, useRef, useState, useCallback } from 'react'
import { EditorView, keymap, lineNumbers, highlightActiveLine, highlightSpecialChars, drawSelection, rectangularSelection, crosshairCursor, dropCursor, scrollPastEnd, Decoration, type DecorationSet, type ViewUpdate, ViewPlugin, placeholder as cmPlaceholder } from '@codemirror/view'
import { EditorState, Compartment, RangeSetBuilder } from '@codemirror/state'
import { defaultKeymap, history, historyKeymap, indentWithTab } from '@codemirror/commands'
import { syntaxHighlighting, defaultHighlightStyle, bracketMatching, foldGutter, indentOnInput } from '@codemirror/language'
import { closeBrackets, closeBracketsKeymap } from '@codemirror/autocomplete'
import { searchKeymap, highlightSelectionMatches } from '@codemirror/search'
import { oneDark } from '@codemirror/theme-one-dark'
import { javascript } from '@codemirror/lang-javascript'
import { python } from '@codemirror/lang-python'
import { rust } from '@codemirror/lang-rust'
import { html } from '@codemirror/lang-html'
import { css } from '@codemirror/lang-css'
import { json } from '@codemirror/lang-json'
import { markdown } from '@codemirror/lang-markdown'
import { cpp } from '@codemirror/lang-cpp'
import { java } from '@codemirror/lang-java'
import { go } from '@codemirror/lang-go'
import { php } from '@codemirror/lang-php'
import { yaml } from '@codemirror/lang-yaml'
import { xml } from '@codemirror/lang-xml'
import { sql } from '@codemirror/lang-sql'
import { StreamLanguage } from '@codemirror/language'
import { shell } from '@codemirror/legacy-modes/mode/shell'
import { invoke } from '@tauri-apps/api/core'
import './Editor.css'

interface EditorProps {
  filePath: string
  fileSize: number
  onNavigateToMark?: (start: number) => void
}

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

const INITIAL_CHUNK = 500
const SCROLL_CHUNK = 2000
const BUFFER_LINES = 50
const MAX_MARKS_PER_HIGHLIGHT = 500

const log = (msg: string, ...args: any[]) => {
  console.log(`[${new Date().toISOString()}] ${msg}`, ...args)
}

function getLanguageExtension(path: string) {
  const ext = path.split('.').pop()?.toLowerCase()
  switch (ext) {
    case 'js': case 'jsx': return javascript()
    case 'ts': case 'tsx': return javascript({ typescript: true })
    case 'py': return python()
    case 'rs': return rust()
    case 'html': return html()
    case 'css': return css()
    case 'json': return json()
    case 'md': return markdown()
    case 'c': case 'h': case 'cpp': case 'hpp': return cpp()
    case 'java': return java()
    case 'go': return go()
    case 'php': return php()
    case 'yaml': case 'yml': return yaml()
    case 'xml': case 'svg': return xml()
    case 'sql': return sql()
    case 'sh': case 'bash': case 'log': return StreamLanguage.define(shell)
    default: return []
  }
}

// Mark decoration plugin - only decorates visible viewport
function buildMarkDecorations(view: EditorView, marks: Mark[]): DecorationSet {
  const builder = new RangeSetBuilder<Decoration>()
  const viewport = view.viewport
  const doc = view.state.doc

  type Deco = { from: number; to: number; deco: Decoration }
  const decos: Deco[] = []

  for (const mark of marks) {
    if (mark.start > doc.length || mark.end > doc.length) continue
    const from = Math.min(mark.start, doc.length - 1)
    const to = Math.min(mark.end, doc.length)
    if (from <= viewport.to && to >= viewport.from) {
      decos.push({ from, to, deco: Decoration.mark({ class: `cm-mark-${mark.color}` }) })
    }
  }

  decos.sort((a, b) => a.from - b.from || a.to - b.to)
  for (const d of decos) {
    try { builder.add(d.from, d.to, d.deco) } catch { /* overlapping deco, skip */ }
  }
  return builder.finish()
}

const markCompartment = new Compartment()
const langCompartment = new Compartment()

function FileEditor({ filePath: filePathProp, fileSize: _fileSize, onNavigateToMark }: EditorProps) {
  const filePath = filePathProp
  const [isLoading, setIsLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [isModified, setIsModified] = useState(false)
  const [canUndo, setCanUndo] = useState(false)
  const [canRedo, setCanRedo] = useState(false)
  const [isSaving, setIsSaving] = useState(false)
  const [loadedInfo, setLoadedInfo] = useState<{ loaded: number; total: number }>({ loaded: 0, total: 0 })

  const editorContainerRef = useRef<HTMLDivElement>(null)
  const editorViewRef = useRef<EditorView | null>(null)
  const fileLineCountRef = useRef(0)
  const loadedEndLineRef = useRef(0)
  const isLoadingMoreRef = useRef(false)
  const marksRef = useRef<Mark[]>([])
  const contextMenuRef = useRef<HTMLDivElement | null>(null)
  const editorCreatedRef = useRef(false)  // Prevent re-creation

  const handleCreateMark = useCallback(async (color: string) => {
    const view = editorViewRef.current
    if (!view || !filePath) return
    const sel = view.state.selection.main
    if (sel.empty) return
    const selectedText = view.state.sliceDoc(sel.from, sel.to)
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
  }, [filePath])

  const refreshMarkDecorations = useCallback(async () => {
    const view = editorViewRef.current
    if (!view || !filePath) return
    try {
      const response = await invoke<{ marks: Mark[] }>('get_marks', {
        req: { path: filePath, start: null, end: null }
      })
      marksRef.current = response.marks || []
      const markPlugin = ViewPlugin.fromClass(class {
        decorations: DecorationSet
        constructor(v: EditorView) { this.decorations = buildMarkDecorations(v, marksRef.current) }
        update(u: ViewUpdate) {
          if (u.docChanged || u.viewportChanged) this.decorations = buildMarkDecorations(u.view, marksRef.current)
        }
      }, { decorations: v => v.decorations })
      view.dispatch({ effects: markCompartment.reconfigure(markPlugin) })
    } catch (err) { log('[Editor::refreshMarkDecorations] ERROR:', err) }
  }, [filePath])

  const navigateToMarkPosition = useCallback((startOffset: number) => {
    const view = editorViewRef.current
    if (!view) return
    const pos = Math.min(startOffset, view.state.doc.length)
    view.dispatch({ selection: { anchor: pos }, scrollIntoView: true })
    view.focus()
  }, [])

  useEffect(() => { if (onNavigateToMark) (window as any).__sakNavigateToMark = navigateToMarkPosition }, [onNavigateToMark, navigateToMarkPosition])

  useEffect(() => {
    const handler = (e: Event) => { const color = (e as CustomEvent).detail?.color; if (color) handleCreateMark(color) }
    window.addEventListener('sak-mark-highlight', handler)
    return () => window.removeEventListener('sak-mark-highlight', handler)
  }, [handleCreateMark])

  // Load more content - append to existing document
  const loadMoreContent = useCallback(async (fromLine: number, toLine: number) => {
    const view = editorViewRef.current
    if (!view || !filePath || isLoadingMoreRef.current) return
    isLoadingMoreRef.current = true
    try {
      const text = await invoke<string>('get_lines', {
        req: { path: filePath, start_line: fromLine, end_line: toLine + 1 }
      })
      // Check view still exists after async
      const currentView = editorViewRef.current
      if (!currentView || currentView.state.doc.length === 0) { isLoadingMoreRef.current = false; return }

      // Append at end of document
      const insertText = '\n' + text
      currentView.dispatch({
        changes: { from: currentView.state.doc.length, insert: insertText }
      })

      loadedEndLineRef.current = toLine
      setLoadedInfo({ loaded: toLine, total: fileLineCountRef.current })
      log(`[Editor::loadMoreContent] Appended lines ${fromLine}-${toLine}`)
    } catch (err) {
      log('[Editor::loadMoreContent] ERROR:', err)
    } finally {
      isLoadingMoreRef.current = false
    }
  }, [filePath])

  const checkEditStatus = useCallback(async () => {
    try {
      const status = await invoke<{ has_changes: boolean; can_undo: boolean; can_redo: boolean }>('get_edit_status', { path: filePath })
      setIsModified(status.has_changes)
      setCanUndo(status.can_undo)
      setCanRedo(status.can_redo)
    } catch { /* ignore */ }
  }, [filePath])

  const handleUndo = useCallback(async () => {
    try {
      if (await invoke<boolean>('undo', { path: filePath })) { await checkEditStatus() }
    } catch { /* ignore */ }
  }, [filePath, checkEditStatus])

  const handleRedo = useCallback(async () => {
    try {
      if (await invoke<boolean>('redo', { path: filePath })) { await checkEditStatus() }
    } catch { /* ignore */ }
  }, [filePath, checkEditStatus])

  const handleSave = useCallback(async () => {
    setIsSaving(true)
    try { await invoke('save_file', { path: filePath }); setIsModified(false); await checkEditStatus() }
    catch (err) { setError(err instanceof Error ? err.message : 'Save failed') }
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

  // Context menu
  const handleContextMenu = useCallback((e: MouseEvent) => {
    e.preventDefault()
    if (contextMenuRef.current) { contextMenuRef.current.remove(); contextMenuRef.current = null }
    const menu = document.createElement('div')
    menu.className = 'cm-context-menu'
    MARK_COLORS.forEach(color => {
      const item = document.createElement('div')
      item.className = 'cm-context-menu-item'
      item.textContent = `Mark: ${color.charAt(0).toUpperCase() + color.slice(1)}`
      item.onclick = () => { handleCreateMark(color); menu.remove(); contextMenuRef.current = null }
      menu.appendChild(item)
    })
    menu.style.left = `${e.clientX}px`
    menu.style.top = `${e.clientY}px`
    document.body.appendChild(menu)
    contextMenuRef.current = menu
    const closeMenu = (ev: MouseEvent) => {
      if (!menu.contains(ev.target as Node)) { menu.remove(); contextMenuRef.current = null; document.removeEventListener('mousedown', closeMenu) }
    }
    setTimeout(() => document.addEventListener('mousedown', closeMenu), 0)
  }, [handleCreateMark])

  // === MAIN INITIALIZATION ===
  // This runs once per filePath change. Creates EditorView and loads initial content.
  useEffect(() => {
    if (!filePath) return
    log(`[Editor::init] filePath="${filePath}"`)

    // Destroy previous editor if any
    if (editorViewRef.current) {
      editorViewRef.current.dom.removeEventListener('contextmenu', handleContextMenu)
      editorViewRef.current.destroy()
      editorViewRef.current = null
      editorCreatedRef.current = false
    }

    let cancelled = false

    const initializeEditor = async () => {
      try {
        setIsLoading(true)
        setError(null)
        loadedEndLineRef.current = 0
        isLoadingMoreRef.current = false

        // Step 1: Build line index (builds in background, ~300ms for 514MB)
        log('[Editor::init] Building line index...')
        const indexStart = performance.now()
        await invoke('build_line_index', { path: filePath }).catch(err => {
          log('[Editor::init] Line index build failed (non-fatal):', err)
        })
        log(`[Editor::init] Line index built in ${Math.round(performance.now() - indexStart)}ms`)

        if (cancelled) return

        // Step 2: Get file info (O(1) with index)
        const fileInfo = await invoke<{ line_count: number }>('get_file_info', { path: filePath })
          .catch(() => ({ line_count: 0 }))
        const totalLines = fileInfo.line_count || 0
        fileLineCountRef.current = totalLines
        log(`[Editor::init] File has ${totalLines} lines`)

        if (cancelled) return

        // Step 3: Load first chunk
        const firstChunkEnd = Math.min(INITIAL_CHUNK, totalLines)
        const firstChunk = await invoke<string>('get_lines', {
          req: { path: filePath, start_line: 0, end_line: firstChunkEnd }
        })
        loadedEndLineRef.current = firstChunkEnd - 1
        log(`[Editor::init] Loaded first ${firstChunkEnd} lines`)

        if (cancelled) return

        // Step 4: Create EditorView with initial content
        const container = editorContainerRef.current
        if (!container) { setIsLoading(false); return }

        const markPlugin = ViewPlugin.fromClass(class {
          decorations: DecorationSet
          constructor(_v: EditorView) { this.decorations = Decoration.none }
          update(_u: ViewUpdate) { /* marks loaded separately */ }
        }, { decorations: v => v.decorations })

        const state = EditorState.create({
          doc: firstChunk,
          extensions: [
            lineNumbers(),
            highlightActiveLine(),
            highlightSpecialChars(),
            highlightSelectionMatches(),
            foldGutter(),
            drawSelection(),
            rectangularSelection(),
            crosshairCursor(),
            dropCursor(),
            scrollPastEnd(),
            indentOnInput(),
            bracketMatching(),
            closeBrackets(),
            history(),
            keymap.of([
              ...closeBracketsKeymap,
              ...defaultKeymap,
              ...searchKeymap,
              ...historyKeymap,
              indentWithTab,
            ]),
            syntaxHighlighting(defaultHighlightStyle, { fallback: true }),
            oneDark,
            langCompartment.of(getLanguageExtension(filePath)),
            markCompartment.of(markPlugin),
            // Viewport change listener - load more content on scroll
            EditorView.updateListener.of((update) => {
              if (!update.viewportChanged) return
              const view = update.view
              const docLines = view.state.doc.lines
              const viewport = view.viewport
              const lastVisibleLine = view.state.doc.lineAt(viewport.to).number
              const totalLines = fileLineCountRef.current

              // Load more when near bottom of loaded content
              if (lastVisibleLine >= docLines - BUFFER_LINES
                  && loadedEndLineRef.current < totalLines - 1
                  && !isLoadingMoreRef.current) {
                const fromLine = loadedEndLineRef.current + 1
                const toLine = Math.min(fromLine + SCROLL_CHUNK, totalLines - 1)
                log(`[Editor::viewport] Near bottom, loading lines ${fromLine}-${toLine}`)
                // Fire and forget - isLoadingMoreRef prevents re-entry
                loadMoreContent(fromLine, toLine)
              }
            }),
            EditorView.theme({
              '&': { height: '100%' },
              '.cm-scroller': { overflow: 'auto' },
            }),
            cmPlaceholder('Loading...'),
          ]
        })

        const view = new EditorView({ state, parent: container })
        editorViewRef.current = view
        editorCreatedRef.current = true

        // Add context menu
        view.dom.addEventListener('contextmenu', handleContextMenu)

        setIsLoading(false)
        setLoadedInfo({ loaded: firstChunkEnd - 1, total: totalLines })

        // Load marks after editor is ready
        setTimeout(() => refreshMarkDecorations(), 200)
        checkEditStatus()

        log(`[Editor::init] Editor ready in ${Math.round(performance.now() - indexStart)}ms`)
      } catch (err) {
        if (cancelled) return
        const errorMsg = err instanceof Error ? err.message : String(err)
        log(`[Editor::init] ERROR:`, errorMsg)
        setError(errorMsg)
        setIsLoading(false)
      }
    }

    initializeEditor()

    return () => {
      cancelled = true
      if (editorViewRef.current) {
        editorViewRef.current.dom.removeEventListener('contextmenu', handleContextMenu)
        editorViewRef.current.destroy()
        editorViewRef.current = null
        editorCreatedRef.current = false
      }
      if (contextMenuRef.current) { contextMenuRef.current.remove(); contextMenuRef.current = null }
    }
  }, [filePath]) // ONLY re-run when filePath changes - NOT content

  if (isLoading) return <div className="editor-loading">Loading file...</div>
  if (error) return <div className="editor-error"><div className="error-message">Error loading file: {error}</div></div>

  const loadedLines = loadedInfo.loaded + 1
  const totalLines = loadedInfo.total

  return (
    <div className="editor-wrapper">
      <div className="editor-toolbar">
        <div className="editor-info">
          <span className={isModified ? 'modified' : ''}>{filePath}{isModified && ' *'}</span>
          <span className="editor-stats">
            Lines {loadedLines.toLocaleString()}{totalLines > loadedLines ? ` (loading... ${totalLines.toLocaleString()} total)` : ` of ${totalLines.toLocaleString()}`}
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
      <div ref={editorContainerRef} className="editor-container" />
    </div>
  )
}

export default FileEditor