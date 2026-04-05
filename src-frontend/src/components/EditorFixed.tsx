import { useEffect, useRef, useCallback } from 'react'
import Editor from '@monaco-editor/react'
import { invoke } from '@tauri-apps/api/core'
import './Editor.css'

interface EditorProps {
  filePath: string
  fileSize: number
}

const CHUNK_SIZE = 64 * 1024 // 64KB chunks
const log = (msg: string, ...args: any[]) => {
  console.log(`[${new Date().toISOString()}] ${msg}`, ...args)
}

function FileEditor({ filePath, fileSize }: EditorProps) {
  const editorRef = useRef<any>(null)
  const monacoRef = useRef<any>(null)
  const loadedEndRef = useRef<number>(0)
  const isLoadingRef = useRef<boolean>(false)

  // Initialize editor with first chunk
  const handleEditorDidMount = useCallback((editor: any, monaco: any) => {
    log('[Editor] Mounted')
    editorRef.current = editor
    monacoRef.current = monaco

    // Load initial content
    loadChunk(0, Math.min(CHUNK_SIZE, fileSize))

    // Configure scroll listener
    editor.onDidScrollChange((e: any) => {
      const scrollTop = e.scrollTop
      const scrollHeight = e.scrollHeight
      const clientHeight = e.height

      // Load more when within 20% of bottom
      if (scrollTop + clientHeight >= scrollHeight * 0.8) {
        if (loadedEndRef.current < fileSize && !isLoadingRef.current) {
          loadChunk(loadedEndRef.current, Math.min(loadedEndRef.current + CHUNK_SIZE, fileSize))
        }
      }
    })
  }, [filePath, fileSize])

  // Load a chunk and append to editor (without resetting scroll)
  const loadChunk = useCallback(async (start: number, end: number) => {
    if (isLoadingRef.current) return
    isLoadingRef.current = true

    log(`[Editor] Loading chunk ${start}-${end}`)

    try {
      const text = await invoke<string>('get_text', {
        req: { path: filePath, start, end }
      })

      const editor = editorRef.current
      const monaco = monacoRef.current

      if (editor && monaco) {
        const model = editor.getModel()
        if (model) {
          // Get current scroll position
          const scrollTop = editor.getScrollTop()

          if (start === 0) {
            // First chunk - set initial content
            model.setValue(text)
          } else {
            // Append to existing content
            const lineCount = model.getLineCount()
            const lastLineLength = model.getLineMaxColumn(lineCount)

            model.applyEdits([{
              range: new monaco.Range(lineCount, lastLineLength, lineCount, lastLineLength),
              text: text
            }])

            // Restore scroll position
            editor.setScrollTop(scrollTop)
          }

          loadedEndRef.current = end
          log(`[Editor] Loaded ${text.length} bytes, total: ${end}/${fileSize}`)
        }
      }
    } catch (err) {
      log('[Editor] Error loading chunk:', err)
    } finally {
      isLoadingRef.current = false
    }
  }, [filePath, fileSize])

  return (
    <div className="editor-wrapper">
      <div className="editor-info">
        <span>{filePath}</span>
        <span className="editor-stats">
          Loaded: {loadedEndRef.current.toLocaleString()} / {fileSize.toLocaleString()} bytes
        </span>
      </div>
      <Editor
        height="calc(100% - 32px)"
        language="plaintext"
        value="" // Content set via model
        theme="vs-dark"
        onMount={handleEditorDidMount}
        options={{
          selectOnLineNumbers: true,
          automaticLayout: true,
          scrollBeyondLastLine: false,
        }}
      />
    </div>
  )
}

export default FileEditor
