import { useState, useEffect, useRef } from 'react'
import './GoToLineDialog.css'

interface GoToLineDialogProps {
  isOpen: boolean
  onClose: () => void
  onGoToLine: (line: number, offset?: number) => void
  currentLine: number
  totalLines: number
}

export function GoToLineDialog({ 
  isOpen, 
  onClose, 
  onGoToLine,
  currentLine,
  totalLines
}: GoToLineDialogProps) {
  const [lineNumber, setLineNumber] = useState(currentLine.toString())
  const [offset, setOffset] = useState('')
  const inputRef = useRef<HTMLInputElement>(null)

  useEffect(() => {
    if (isOpen) {
      setLineNumber(currentLine.toString())
      setOffset('')
      setTimeout(() => inputRef.current?.focus(), 100)
    }
  }, [isOpen, currentLine])

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (!isOpen) return
      
      if (e.key === 'Escape') {
        onClose()
      } else if (e.key === 'Enter') {
        handleGoTo()
      }
    }

    window.addEventListener('keydown', handleKeyDown)
    return () => window.removeEventListener('keydown', handleKeyDown)
  }, [isOpen, lineNumber, offset])

  const handleGoTo = () => {
    const line = parseInt(lineNumber, 10)
    if (line >= 1 && line <= totalLines) {
      const off = offset ? parseInt(offset, 10) : undefined
      onGoToLine(line, off)
      onClose()
    }
  }

  if (!isOpen) return null

  return (
    <div className="goto-dialog-overlay" onClick={onClose}>
      <div className="goto-dialog" onClick={e => e.stopPropagation()}>
        <div className="goto-dialog-header">
          <h3>Go to Line</h3>
          <button className="btn-close" onClick={onClose}>×</button>
        </div>

        <div className="goto-dialog-body">
          <div className="form-row">
            <label>Line Number (1 - {totalLines}):</label>
            <input
              ref={inputRef}
              type="number"
              min={1}
              max={totalLines}
              value={lineNumber}
              onChange={e => setLineNumber(e.target.value)}
              placeholder="Enter line number"
            />
          </div>

          <div className="form-row">
            <label>Offset (optional):</label>
            <input
              type="number"
              min={0}
              value={offset}
              onChange={e => setOffset(e.target.value)}
              placeholder="Character offset in line"
            />
          </div>

          <div className="goto-info">
            Current: Line {currentLine} of {totalLines}
          </div>
        </div>

        <div className="goto-dialog-footer">
          <button className="btn-secondary" onClick={onClose}>Cancel</button>
          <button 
            className="btn-primary" 
            onClick={handleGoTo}
            disabled={!lineNumber || parseInt(lineNumber) < 1 || parseInt(lineNumber) > totalLines}
          >
            Go To
          </button>
        </div>
      </div>
    </div>
  )
}

export default GoToLineDialog