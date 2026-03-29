import { useState, useEffect, useRef, useCallback } from 'react'
import './InputDialog.css'

export interface InputDialogProps {
  isOpen: boolean
  onClose: () => void
  onConfirm: (value: string) => void
  title: string
  placeholder?: string
  defaultValue?: string
  validate?: (value: string) => string | null
  maxLength?: number
}

export function InputDialog({
  isOpen,
  onClose,
  onConfirm,
  title,
  placeholder = 'Enter value...',
  defaultValue = '',
  validate,
  maxLength
}: InputDialogProps) {
  const [value, setValue] = useState(defaultValue)
  const [error, setError] = useState<string | null>(null)
  const inputRef = useRef<HTMLInputElement>(null)

  // Reset value when dialog opens
  useEffect(() => {
    if (isOpen) {
      setValue(defaultValue)
      setError(null)
      setTimeout(() => {
        inputRef.current?.focus()
        inputRef.current?.select()
      }, 100)
    }
  }, [isOpen, defaultValue])

  // Keyboard navigation
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (!isOpen) return

      if (e.key === 'Escape') {
        onClose()
        return
      }

      if (e.key === 'Enter') {
        handleConfirm()
      }
    }

    window.addEventListener('keydown', handleKeyDown)
    return () => window.removeEventListener('keydown', handleKeyDown)
  }, [isOpen, value])

  const handleChange = useCallback((e: React.ChangeEvent<HTMLInputElement>) => {
    let newValue = e.target.value
    if (maxLength && newValue.length > maxLength) {
      newValue = newValue.slice(0, maxLength)
    }
    setValue(newValue)
    setError(null)
  }, [maxLength])

  const handleConfirm = useCallback(() => {
    const trimmedValue = value.trim()
    
    if (!trimmedValue) {
      setError('This field is required')
      return
    }

    if (validate) {
      const validationError = validate(trimmedValue)
      if (validationError) {
        setError(validationError)
        return
      }
    }

    onConfirm(trimmedValue)
    onClose()
  }, [value, validate, onConfirm, onClose])

  if (!isOpen) return null

  return (
    <div className="input-dialog-overlay" onClick={onClose}>
      <div className="input-dialog" onClick={e => e.stopPropagation()}>
        <div className="input-dialog-header">
          <h3>{title}</h3>
          <button className="btn-close" onClick={onClose}>×</button>
        </div>

        <div className="input-dialog-body">
          <div className="form-row">
            <input
              ref={inputRef}
              type="text"
              value={value}
              onChange={handleChange}
              placeholder={placeholder}
              maxLength={maxLength}
              className={error ? 'has-error' : ''}
            />
            {error && <span className="input-error">{error}</span>}
          </div>
        </div>

        <div className="input-dialog-footer">
          <button className="btn-secondary" onClick={onClose}>
            Cancel
          </button>
          <button 
            className="btn-primary" 
            onClick={handleConfirm}
            disabled={!value.trim()}
          >
            OK
          </button>
        </div>
      </div>
    </div>
  )
}

export default InputDialog
