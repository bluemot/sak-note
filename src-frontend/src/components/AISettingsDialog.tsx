import { useState, useEffect, useRef } from 'react'
import { useSettingsStore, AVAILABLE_MODELS } from '../store/settingsStore'
import './AISettingsDialog.css'

interface AISettings {
  apiUrl: string
  model: string
  apiKey: string
  temperature: number
}

interface AISettingsDialogProps {
  isOpen: boolean
  onClose: () => void
  onSave?: (settings: AISettings) => void
}

export function AISettingsDialog({ isOpen, onClose, onSave }: AISettingsDialogProps) {
  const aiSettings = useSettingsStore((s) => s.aiSettings)
  const updateAISettings = useSettingsStore((s) => s.updateAISettings)
  const applyAISettings = useSettingsStore((s) => s.applyAISettings)
  const isApplying = useSettingsStore((s) => s.isApplying)

  const [localSettings, setLocalSettings] = useState<AISettings>(aiSettings)
  const [isTesting, setIsTesting] = useState(false)
  const [testStatus, setTestStatus] = useState<'idle' | 'success' | 'error'>('idle')
  const [customModel, setCustomModel] = useState('')
  const [showCustomModel, setShowCustomModel] = useState(false)
  
  const apiUrlInputRef = useRef<HTMLInputElement>(null)

  // Sync local state from store when dialog opens
  useEffect(() => {
    if (isOpen) {
      setLocalSettings(aiSettings)
      setTestStatus('idle')
      setShowCustomModel(false)
    }
  }, [isOpen])

  // Focus management
  useEffect(() => {
    if (isOpen) {
      setTimeout(() => apiUrlInputRef.current?.focus(), 100)
    }
  }, [isOpen])

  // Keyboard navigation
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (!isOpen) return
      
      if (e.key === 'Escape') {
        onClose()
        return
      }

      if (e.key === 'Enter' && e.ctrlKey) {
        handleSave()
      }
    }

    window.addEventListener('keydown', handleKeyDown)
    return () => window.removeEventListener('keydown', handleKeyDown)
  }, [isOpen, localSettings])

  const handleSave = async () => {
    const finalSettings = showCustomModel && customModel
      ? { ...localSettings, model: customModel }
      : localSettings

    updateAISettings(finalSettings)
    await applyAISettings()
    onSave?.(finalSettings)
    onClose()
  }

  const handleTestConnection = async () => {
    setIsTesting(true)
    setTestStatus('idle')
    
    try {
      const response = await fetch(`${localSettings.apiUrl}/api/tags`)
      if (response.ok) {
        setTestStatus('success')
      } else {
        setTestStatus('error')
      }
    } catch (err) {
      console.error('Failed to test connection:', err)
      setTestStatus('error')
    } finally {
      setIsTesting(false)
    }
  }

  const handleModelChange = (value: string) => {
    if (value === 'custom') {
      setShowCustomModel(true)
    } else {
      setShowCustomModel(false)
      setLocalSettings({ ...localSettings, model: value })
    }
  }

  const updateSetting = <K extends keyof AISettings>(key: K, value: AISettings[K]) => {
    setLocalSettings(prev => ({ ...prev, [key]: value }))
    setTestStatus('idle')
  }

  if (!isOpen) return null

  return (
    <div className="ai-dialog-overlay" onClick={onClose}>
      <div className="ai-dialog" onClick={e => e.stopPropagation()}>
        <div className="ai-dialog-header">
          <h3>AI Settings</h3>
          <button className="btn-close" onClick={onClose}>×</button>
        </div>

        <div className="ai-dialog-body">
          <div className="form-row">
            <label>Ollama API URL</label>
            <div className="input-with-button">
              <input
                ref={apiUrlInputRef}
                type="text"
                value={localSettings.apiUrl}
                onChange={e => updateSetting('apiUrl', e.target.value)}
                placeholder="http://localhost:11434"
              />
              <button 
                className="btn-test-connection"
                onClick={handleTestConnection}
                disabled={isTesting}
              >
                {isTesting ? 'Testing...' : 'Test'}
              </button>
            </div>
            {testStatus === 'success' && (
              <span className="test-status success">Connected successfully</span>
            )}
            {testStatus === 'error' && (
              <span className="test-status error">Connection failed</span>
            )}
          </div>

          <div className="form-row">
            <label>Model</label>
            <select
              value={showCustomModel ? 'custom' : localSettings.model}
              onChange={e => handleModelChange(e.target.value)}
            >
              {AVAILABLE_MODELS.map(model => (
                <option key={model.value} value={model.value}>
                  {model.label}
                </option>
              ))}
              <option value="custom">-- Custom --</option>
            </select>
            {showCustomModel && (
              <input
                type="text"
                value={customModel}
                onChange={e => setCustomModel(e.target.value)}
                placeholder="Enter model name (e.g., my-model:latest)"
                className="custom-model-input"
              />
            )}
          </div>

          <div className="form-row">
            <label>API Key (Optional)</label>
            <input
              type="password"
              value={localSettings.apiKey}
              onChange={e => updateSetting('apiKey', e.target.value)}
              placeholder="Enter API key if required"
            />
            <span className="field-hint">Required for cloud providers or authenticated endpoints</span>
          </div>

          <div className="form-row">
            <label>Temperature: {localSettings.temperature}</label>
            <input
              type="range"
              min="0"
              max="2"
              step="0.1"
              value={localSettings.temperature}
              onChange={e => updateSetting('temperature', parseFloat(e.target.value))}
            />
            <div className="range-labels">
              <span>Precise (0)</span>
              <span>Balanced (1)</span>
              <span>Creative (2)</span>
            </div>
          </div>

          <div className="form-help">
            <p><kbd>Ctrl</kbd> + <kbd>Enter</kbd> to save</p>
            <p><kbd>Esc</kbd> to close</p>
          </div>
        </div>

        <div className="ai-dialog-footer">
          <button className="btn-secondary" onClick={onClose}>
            Cancel
          </button>
          <button 
            className="btn-primary" 
            onClick={handleSave}
            disabled={!localSettings.apiUrl || (!localSettings.model && !customModel) || isApplying}
          >
            {isApplying ? 'Applying...' : 'Save Settings'}
          </button>
        </div>
      </div>
    </div>
  )
}

export default AISettingsDialog