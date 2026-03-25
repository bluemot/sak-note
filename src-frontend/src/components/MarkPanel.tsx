import { useState, useCallback, useEffect } from 'react'
import { invoke } from '@tauri-apps/api/core'
import './MarkPanel.css'

export interface Mark {
  id: string
  start: number
  end: number
  color: 'red' | 'orange' | 'yellow' | 'green' | 'cyan' | 'blue' | 'purple' | 'pink' | 'gray'
  label?: string
  note?: string
  created_at: number
  updated_at: number
}

export interface MarkColorConfig {
  name: string
  color: Mark['color']
  hex: string
}

const MARK_COLORS: MarkColorConfig[] = [
  { name: 'Red', color: 'red', hex: '#ff6b6b' },
  { name: 'Orange', color: 'orange', hex: '#ff9f43' },
  { name: 'Yellow', color: 'yellow', hex: '#feca57' },
  { name: 'Green', color: 'green', hex: '#1dd1a1' },
  { name: 'Cyan', color: 'cyan', hex: '#00d2d3' },
  { name: 'Blue', color: 'blue', hex: '#54a0ff' },
  { name: 'Purple', color: 'purple', hex: '#5f27cd' },
  { name: 'Pink', color: 'pink', hex: '#ff9ff3' },
  { name: 'Gray', color: 'gray', hex: '#8395a7' },
]

interface MarkPanelProps {
  filePath: string | null
  currentSelection?: { start: number; end: number } | null
  onMarkClick?: (mark: Mark) => void
  onApplyMark?: (color: Mark['color']) => void
}

export function MarkPanel({ filePath, currentSelection, onMarkClick, onApplyMark }: MarkPanelProps) {
  const [marks, setMarks] = useState<Mark[]>([])
  const [selectedColor, setSelectedColor] = useState<Mark['color']>('yellow')
  const [isLoading, setIsLoading] = useState(false)
  const [editingMark, setEditingMark] = useState<Mark | null>(null)
  const [filterColor, setFilterColor] = useState<Mark['color'] | null>(null)

  // Load marks when file changes
  useEffect(() => {
    if (filePath) {
      loadMarks()
    } else {
      setMarks([])
    }
  }, [filePath])

  const loadMarks = useCallback(async () => {
    if (!filePath) return
    try {
      setIsLoading(true)
      const response = await invoke<{ marks: Mark[] }>('get_marks', {
        req: { path: filePath, start: null, end: null }
      })
      setMarks(response.marks)
    } catch (err) {
      console.error('Failed to load marks:', err)
    } finally {
      setIsLoading(false)
    }
  }, [filePath])

  const createMark = useCallback(async () => {
    if (!filePath || !currentSelection) return
    
    try {
      const newMark = await invoke<Mark>('create_mark', {
        req: {
          path: filePath,
          start: currentSelection.start,
          end: currentSelection.end,
          color: selectedColor,
          label: null,
          note: null
        }
      })
      setMarks(prev => [...prev, newMark])
      onApplyMark?.(selectedColor)
    } catch (err) {
      console.error('Failed to create mark:', err)
    }
  }, [filePath, currentSelection, selectedColor, onApplyMark])

  const deleteMark = useCallback(async (id: string) => {
    if (!filePath) return
    try {
      await invoke('delete_mark', {
        req: { path: filePath, id }
      })
      setMarks(prev => prev.filter(m => m.id !== id))
    } catch (err) {
      console.error('Failed to delete mark:', err)
    }
  }, [filePath])

  const updateMark = useCallback(async (id: string, updates: Partial<Mark>) => {
    if (!filePath) return
    try {
      const updated = await invoke<Mark>('update_mark', {
        req: {
          path: filePath,
          id,
          updates: {
            color: updates.color,
            label: updates.label,
            note: updates.note,
            clear_label: updates.label === null,
            clear_note: updates.note === null
          }
        }
      })
      setMarks(prev => prev.map(m => m.id === id ? updated : m))
      setEditingMark(null)
    } catch (err) {
      console.error('Failed to update mark:', err)
    }
  }, [filePath])

  const clearAllMarks = useCallback(async () => {
    if (!filePath) return
    if (!confirm('Clear all marks?')) return
    try {
      await invoke('clear_marks', { path: filePath })
      setMarks([])
    } catch (err) {
      console.error('Failed to clear marks:', err)
    }
  }, [filePath])

  const filteredMarks = filterColor 
    ? marks.filter(m => m.color === filterColor)
    : marks

  const sortedMarks = [...filteredMarks].sort((a, b) => a.start - b.start)

  if (!filePath) {
    return (
      <div className="mark-panel">
        <p className="mark-placeholder">Open a file to use marks</p>
      </div>
    )
  }

  return (
    <div className="mark-panel">
      <div className="mark-toolbar">
        <h3>Color Marks</h3>
        <button 
          className="mark-btn clear-all"
          onClick={clearAllMarks}
          disabled={marks.length === 0}
        >
          Clear All
        </button>
      </div>

      <div className="color-picker">
        <span className="color-label">Select color:</span>
        <div className="color-buttons">
          {MARK_COLORS.map(({ color, hex }) => (
            <button
              key={color}
              className={`color-btn ${selectedColor === color ? 'active' : ''}`}
              style={{ backgroundColor: hex }}
              onClick={() => setSelectedColor(color)}
              title={color}
            />
          ))}
        </div>
      </div>

      {currentSelection && (
        <div className="selection-info">
          <span>Selected: {currentSelection.start} - {currentSelection.end}</span>
          <button className="mark-btn primary" onClick={createMark}>
            Add Mark
          </button>
        </div>
      )}

      <div className="mark-filter">
        <span>Filter:</span>
        <button 
          className={`filter-btn ${filterColor === null ? 'active' : ''}`}
          onClick={() => setFilterColor(null)}
        >
          All ({marks.length})
        </button>
        {MARK_COLORS.map(({ color }) => {
          const count = marks.filter(m => m.color === color).length
          if (count === 0) return null
          return (
            <button
              key={color}
              className={`filter-btn ${filterColor === color ? 'active' : ''}`}
              style={{ 
                backgroundColor: filterColor === color ? MARK_COLORS.find(c => c.color === color)?.hex : undefined 
              }}
              onClick={() => setFilterColor(color)}
            >
              {color} ({count})
            </button>
          )
        })}
      </div>

      <div className="mark-list">
        {isLoading ? (
          <p className="mark-placeholder">Loading marks...</p>
        ) : sortedMarks.length === 0 ? (
          <p className="mark-placeholder">No marks yet</p>
        ) : (
          sortedMarks.map(mark => (
            <div 
              key={mark.id}
              className={`mark-item ${editingMark?.id === mark.id ? 'editing' : ''}`}
              onClick={() => onMarkClick?.(mark)}
            >
              <div 
                className="mark-color-bar"
                style={{ backgroundColor: MARK_COLORS.find(c => c.color === mark.color)?.hex }}
              />
              
              {editingMark?.id === mark.id ? (
                <div className="mark-edit-form">
                  <input
                    type="text"
                    value={editingMark.label || ''}
                    onChange={(e) => setEditingMark({ ...editingMark, label: e.target.value })}
                    placeholder="Label"
                    onClick={(e) => e.stopPropagation()}
                  />
                  <input
                    type="text"
                    value={editingMark.note || ''}
                    onChange={(e) => setEditingMark({ ...editingMark, note: e.target.value })}
                    placeholder="Note"
                    onClick={(e) => e.stopPropagation()}
                  />
                  <select
                    value={editingMark.color}
                    onChange={(e) => setEditingMark({ ...editingMark, color: e.target.value as Mark['color'] })}
                    onClick={(e) => e.stopPropagation()}
                  >
                    {MARK_COLORS.map(c => (
                      <option key={c.color} value={c.color}>{c.name}</option>
                    ))}
                  </select>
                  
                  <div className="mark-edit-actions">
                    <button onClick={(e) => {
                      e.stopPropagation()
                      updateMark(mark.id, editingMark)
                    }}>Save</button>
                    <button onClick={(e) => {
                      e.stopPropagation()
                      setEditingMark(null)
                    }}>Cancel</button>
                  </div>
                </div>
              ) : (
                <div className="mark-content">
                  <div className="mark-header">
                    <span className="mark-range">
                      {mark.start} - {mark.end}
                    </span>
                    <span className="mark-actions">
                      <button onClick={(e) => {
                        e.stopPropagation()
                        setEditingMark(mark)
                      }}>✏️</button>
                      <button onClick={(e) => {
                        e.stopPropagation()
                        deleteMark(mark.id)
                      }}>🗑️</button>
                    </span>
                  </div>
                  {mark.label && <span className="mark-label">{mark.label}</span>}
                  {mark.note && <span className="mark-note">{mark.note}</span>}
                </div>
              )}
            </div>
          ))
        )}
      </div>
    </div>
  )
}

export default MarkPanel
