import { useState, useEffect, useCallback } from 'react'
import './FindReplace.css'

interface FindReplaceProps {
  isOpen: boolean
  onClose: () => void
  onFind: (query: string, options: FindOptions) => void
  onReplace: (query: string, replacement: string, options: FindOptions) => void
  onReplaceAll: (query: string, replacement: string, options: FindOptions) => void
  findResult?: {
    current: number
    total: number
    currentMatch?: { line: number; column: number }
  } | null
}

export interface FindOptions {
  caseSensitive: boolean
  wholeWord: boolean
  useRegex: boolean
  wrapAround: boolean
  searchInSelection: boolean
}

export function FindReplace({ 
  isOpen, 
  onClose, 
  onFind, 
  onReplace, 
  onReplaceAll,
  findResult 
}: FindReplaceProps) {
  const [findText, setFindText] = useState('')
  const [replaceText, setReplaceText] = useState('')
  const [options, setOptions] = useState<FindOptions>({
    caseSensitive: false,
    wholeWord: false,
    useRegex: false,
    wrapAround: true,
    searchInSelection: false,
  })
  const [mode, setMode] = useState<'find' | 'replace'>('find')

  // Handle keyboard shortcuts
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (!isOpen) return
      
      if (e.key === 'Escape') {
        onClose()
      } else if (e.key === 'Enter' && e.shiftKey) {
        e.preventDefault()
        handleFindPrevious()
      } else if (e.key === 'Enter') {
        e.preventDefault()
        if (mode === 'replace') {
          handleFindNext()
        } else {
          handleFindNext()
        }
      } else if (e.key === 'F3') {
        e.preventDefault()
        handleFindNext()
      } else if (e.key === 'F3' && e.shiftKey) {
        e.preventDefault()
        handleFindPrevious()
      }
    }

    window.addEventListener('keydown', handleKeyDown)
    return () => window.removeEventListener('keydown', handleKeyDown)
  }, [isOpen, findText, replaceText, options, mode])

  const handleFindNext = useCallback(() => {
    if (findText) {
      onFind(findText, options)
    }
  }, [findText, options, onFind])

  const handleFindPrevious = useCallback(() => {
    if (findText) {
      // Implementation would handle direction
      onFind(findText, { ...options })
    }
  }, [findText, options, onFind])

  const handleReplace = useCallback(() => {
    if (findText) {
      onReplace(findText, replaceText, options)
    }
  }, [findText, replaceText, options, onReplace])

  const handleReplaceAll = useCallback(() => {
    if (findText) {
      onReplaceAll(findText, replaceText, options)
    }
  }, [findText, replaceText, options, onReplaceAll])

  if (!isOpen) return null

  return (
    <div className="find-replace-overlay">
      <div className="find-replace-dialog">
        <div className="find-replace-header">
          <div className="find-replace-tabs">
            <button 
              className={`tab ${mode === 'find' ? 'active' : ''}`}
              onClick={() => setMode('find')}
            >
              🔍 Find
            </button>
            <button 
              className={`tab ${mode === 'replace' ? 'active' : ''}`}
              onClick={() => setMode('replace')}
            >
              🔄 Replace
            </button>
          </div>
          <button className="close-btn" onClick={onClose}>×</button>
        </div>

        <div className="find-replace-body">
          <div className="input-group">
            <label>
              {mode === 'find' ? 'Find what:' : 'Find what:'}
            </label>
            <div className="input-wrapper">
              <input
                type="text"
                value={findText}
                onChange={(e) => setFindText(e.target.value)}
                placeholder="Enter search text..."
                autoFocus
              />
              {findResult && (
                <span className="match-count">
                  {findResult.total > 0 
                    ? `${findResult.current}/${findResult.total}` 
                    : 'No matches'}
                </span>
              )}
            </div>
          </div>

          {mode === 'replace' && (
            <div className="input-group">
              <label>Replace with:</label>
              <input
                type="text"
                value={replaceText}
                onChange={(e) => setReplaceText(e.target.value)}
                placeholder="Enter replacement text..."
              />
            </div>
          )}

          <div className="options-row">
            <label className="checkbox">
              <input
                type="checkbox"
                checked={options.caseSensitive}
                onChange={(e) => setOptions({ ...options, caseSensitive: e.target.checked })}
              />
              Match case
            </label>

            <label className="checkbox">
              <input
                type="checkbox"
                checked={options.wholeWord}
                onChange={(e) => setOptions({ ...options, wholeWord: e.target.checked })}
              />
              Whole word
            </label>

            <label className="checkbox">
              <input
                type="checkbox"
                checked={options.useRegex}
                onChange={(e) => setOptions({ ...options, useRegex: e.target.checked })}
              />
              Regular expression
            </label>
          </div>

          <div className="options-row">
            <label className="checkbox">
              <input
                type="checkbox"
                checked={options.wrapAround}
                onChange={(e) => setOptions({ ...options, wrapAround: e.target.checked })}
              />
              Wrap around
            </label>

            <label className="checkbox">
              <input
                type="checkbox"
                checked={options.searchInSelection}
                onChange={(e) => setOptions({ ...options, searchInSelection: e.target.checked })}
              />
              In selection
            </label>
          </div>
        </div>

        <div className="find-replace-footer">
          {mode === 'find' ? (
            <>
              <button onClick={handleFindNext} disabled={!findText}>
                Find Next
              </button>
              <button onClick={handleFindPrevious} disabled={!findText}>
                Find Previous
              </button>
              <button onClick={handleFindNext} disabled={!findText}>
                🔍 Find All in Current
              </button>
            </>
          ) : (
            <>
              <button onClick={handleFindNext} disabled={!findText}>
                Find Next
              </button>
              <button onClick={handleReplace} disabled={!findText}>
                Replace
              </button>
              <button onClick={handleReplaceAll} disabled={!findText}>
                Replace All
              </button>
            </>
          )}
        </div>
      </div>
    </div>
  )
}

export default FindReplace
