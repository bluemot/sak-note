import { useState, useEffect, useCallback } from 'react'
import { invoke } from '@tauri-apps/api/core'
import './BookmarkPanel.css'

interface Bookmark {
  line: number
  label?: string
  note?: string
}

interface BookmarkPanelProps {
  filePath: string | null
  currentLine: number
  onNavigate: (line: number) => void
}

export function BookmarkPanel({ filePath, currentLine, onNavigate }: BookmarkPanelProps) {
  const [bookmarks, setBookmarks] = useState<Bookmark[]>([])
  const [editingLabel, setEditingLabel] = useState<number | null>(null)
  const [editValue, setEditValue] = useState('')

  // Load bookmarks when file changes
  useEffect(() => {
    if (filePath) {
      loadBookmarks()
    } else {
      setBookmarks([])
    }
  }, [filePath])

  const loadBookmarks = useCallback(async () => {
    if (!filePath) return
    try {
      const result = await invoke<{ bookmarks: Bookmark[] }>('bookmark_get_all', {
        path: filePath
      })
      setBookmarks(result.bookmarks)
    } catch (err) {
      console.error('Failed to load bookmarks:', err)
    }
  }, [filePath])

  const handleToggleBookmark = useCallback(async () => {
    if (!filePath) return
    try {
      const result = await invoke<{ added: boolean; line: number }>('bookmark_toggle', {
        path: filePath,
        line: currentLine
      })
      
      if (result.added) {
        // Add to list
        setBookmarks(prev => [...prev, { line: result.line, label: `Line ${result.line}` }].sort((a, b) => a.line - b.line))
      } else {
        // Remove from list
        setBookmarks(prev => prev.filter(b => b.line !== result.line))
      }
    } catch (err) {
      console.error('Failed to toggle bookmark:', err)
    }
  }, [filePath, currentLine])

  const handleNextBookmark = useCallback(async () => {
    if (!filePath || bookmarks.length === 0) return
    try {
      const result = await invoke<{ line: number }>('bookmark_next', {
        path: filePath,
        currentLine
      })
      if (result.line) {
        onNavigate(result.line)
      }
    } catch (err) {
      console.error('Failed to navigate to next bookmark:', err)
    }
  }, [filePath, currentLine, bookmarks, onNavigate])

  const handlePrevBookmark = useCallback(async () => {
    if (!filePath || bookmarks.length === 0) return
    try {
      const result = await invoke<{ line: number }>('bookmark_prev', {
        path: filePath,
        currentLine
      })
      if (result.line) {
        onNavigate(result.line)
      }
    } catch (err) {
      console.error('Failed to navigate to prev bookmark:', err)
    }
  }, [filePath, currentLine, bookmarks, onNavigate])

  const handleClearAll = useCallback(async () => {
    if (!filePath) return
    try {
      await invoke('bookmark_clear', { path: filePath })
      setBookmarks([])
    } catch (err) {
      console.error('Failed to clear bookmarks:', err)
    }
  }, [filePath])

  const handleDeleteBookmark = useCallback(async (line: number) => {
    if (!filePath) return
    try {
      await invoke('bookmark_remove', { path: filePath, line })
      setBookmarks(prev => prev.filter(b => b.line !== line))
    } catch (err) {
      console.error('Failed to delete bookmark:', err)
    }
  }, [filePath])

  const handleUpdateLabel = useCallback(async (line: number) => {
    if (!filePath) return
    try {
      await invoke('bookmark_update_label', {
        path: filePath,
        line,
        label: editValue
      })
      setBookmarks(prev => prev.map(b => 
        b.line === line ? { ...b, label: editValue } : b
      ))
      setEditingLabel(null)
    } catch (err) {
      console.error('Failed to update label:', err)
    }
  }, [filePath, editValue])

  const startEditing = useCallback((bookmark: Bookmark) => {
    setEditingLabel(bookmark.line)
    setEditValue(bookmark.label || `Line ${bookmark.line}`)
  }, [])

  // Keyboard shortcuts
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      // Ctrl+F2 - Toggle bookmark
      if (e.ctrlKey && e.key === 'F2') {
        e.preventDefault()
        handleToggleBookmark()
      }
      // F2 - Next bookmark
      else if (e.key === 'F2' && !e.shiftKey) {
        e.preventDefault()
        handleNextBookmark()
      }
      // Shift+F2 - Previous bookmark
      else if (e.key === 'F2' && e.shiftKey) {
        e.preventDefault()
        handlePrevBookmark()
      }
    }

    window.addEventListener('keydown', handleKeyDown)
    return () => window.removeEventListener('keydown', handleKeyDown)
  }, [handleToggleBookmark, handleNextBookmark, handlePrevBookmark])

  if (!filePath) {
    return (
      <div className="bookmark-panel">
        <div className="bookmark-empty">
          <p>📑 No file open</p>
          <p className="hint">Open a file to manage bookmarks</p>
        </div>
      </div>
    )
  }

  return (
    <div className="bookmark-panel">
      <div className="bookmark-header">
        <h3>📑 Bookmarks</h3>
        <div className="bookmark-actions">
          <button 
            className="btn-icon" 
            onClick={handleToggleBookmark}
            title="Toggle bookmark (Ctrl+F2)"
          >
            🔖
          </button>
          <button 
            className="btn-icon" 
            onClick={handlePrevBookmark}
            disabled={bookmarks.length === 0}
            title="Previous bookmark (Shift+F2)"
          >
            ⬆️
          </button>
          <button 
            className="btn-icon" 
            onClick={handleNextBookmark}
            disabled={bookmarks.length === 0}
            title="Next bookmark (F2)"
          >
            ⬇️
          </button>
          {bookmarks.length > 0 && (
            <button 
              className="btn-icon danger" 
              onClick={handleClearAll}
              title="Clear all bookmarks"
            >
              🗑️
            </button>
          )}
        </div>
      </div>

      <div className="bookmark-shortcuts">
        <span className="shortcut">Ctrl+F2</span> Toggle
        <span className="shortcut">F2</span> Next
        <span className="shortcut">Shift+F2</span> Previous
      </div>

      <div className="bookmark-list">
        {bookmarks.length === 0 ? (
          <div className="bookmark-empty">
            <p>No bookmarks yet</p>
            <p className="hint">Press Ctrl+F2 to add a bookmark</p>
          </div>
        ) : (
          bookmarks.map((bookmark) => (
            <div 
              key={bookmark.line}
              className={`bookmark-item ${bookmark.line === currentLine ? 'active' : ''}`}
            >
              <div className="bookmark-marker">🔖</div>
              <div className="bookmark-content">
                {editingLabel === bookmark.line ? (
                  <div className="bookmark-edit">
                    <input
                      type="text"
                      value={editValue}
                      onChange={(e) => setEditValue(e.target.value)}
                      onBlur={() => handleUpdateLabel(bookmark.line)}
                      onKeyDown={(e) => {
                        if (e.key === 'Enter') {
                          handleUpdateLabel(bookmark.line)
                        } else if (e.key === 'Escape') {
                          setEditingLabel(null)
                        }
                      }}
                      autoFocus
                    />
                  </div>
                ) : (
                  <div 
                    className="bookmark-label"
                    onClick={() => onNavigate(bookmark.line)}
                    onDoubleClick={() => startEditing(bookmark)}
                  >
                    <span className="line-number">Line {bookmark.line}</span>
                    {bookmark.label && <span className="label-text">: {bookmark.label}</span>}
                  </div>
                )}
                {bookmark.note && (
                  <div className="bookmark-note">{bookmark.note}</div>
                )}
              </div>
              <button 
                className="bookmark-delete"
                onClick={() => handleDeleteBookmark(bookmark.line)}
                title="Delete bookmark"
              >
                ×
              </button>
            </div>
          ))
        )}
      </div>
    </div>
  )
}

export default BookmarkPanel