import { useEffect, useRef, useState, useCallback } from 'react'
import Editor from '@monaco-editor/react'
import { invoke } from '@tauri-apps/api/tauri'
import './Editor.css'

interface EditorProps {
  filePath: string
  fileSize: number
}

const CHUNK_SIZE = 64 * 1024 // 64KB chunks

function FileEditor({ filePath, fileSize }: EditorProps) {
  const [content, setContent] = useState('')
  const [isLoading, setIsLoading] = useState(true)
  const [chunkRange, _setChunkRange] = useState({ start: 0, end: Math.min(CHUNK_SIZE * 2, fileSize) })
  const editorRef = useRef<any>(null)

  // Load initial chunks
  useEffect(() => {
    const loadContent = async () => {
      try {
        setIsLoading(true)
        const text = await invoke<string>('get_text', {
          req: {
            path: filePath,
            start: chunkRange.start,
            end: chunkRange.end
          }
        })
        setContent(text)
      } catch (err) {
        console.error('Failed to load content:', err)
        setContent('// Error loading file')
      } finally {
        setIsLoading(false)
      }
    }

    if (filePath) {
      loadContent()
    }
  }, [filePath, chunkRange])

  const handleEditorDidMount = useCallback((editor: any) => {
    editorRef.current = editor
    
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

    // Virtual scrolling for large files
    editor.onDidScrollChange(() => {
      // TODO: Implement virtual scrolling to load chunks on demand
      // This is a placeholder for the actual implementation
    })
  }, [])

  const handleSave = useCallback(() => {
    // TODO: Implement save functionality
    console.log('Saving file...')
  }, [])

  // Keyboard shortcut for save
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key === 's') {
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
    return langMap[ext || ''] || 'plaintext'
  }

  if (isLoading) {
    return (
      <div className="editor-loading">
        Loading file...
      </div>
    )
  }

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
        language={getLanguage(filePath)}
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
