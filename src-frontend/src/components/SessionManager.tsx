import { useState, useEffect } from 'react'
import { invoke } from '@tauri-apps/api/core'
import './SessionManager.css'

interface SessionFile {
  path: string
  cursor_line: number
  cursor_column: number
  scroll_position: number
  is_active: boolean
}

interface Session {
  name: string
  created_at: number
  modified_at: number
  files: SessionFile[]
}

interface SessionManagerProps {
  onLoadSession: (session: Session) => void
}

export function SessionManager({ onLoadSession }: SessionManagerProps) {
  const [sessions, setSessions] = useState<Session[]>([])
  const [showSaveDialog, setShowSaveDialog] = useState(false)
  const [sessionName, setSessionName] = useState('')

  useEffect(() => {
    loadSessions()
  }, [])

  const loadSessions = async () => {
    try {
      const list = await invoke<Session[]>('session_list')
      setSessions(list)
    } catch (err) {
      console.error('Failed to load sessions:', err)
    }
  }

  const handleSaveSession = async () => {
    if (!sessionName.trim()) return
    
    try {
      await invoke('session_save', {
        session: {
          name: sessionName,
          created_at: Date.now(),
          modified_at: Date.now(),
          files: [] // TODO: Get from editor state
        }
      })
      setShowSaveDialog(false)
      setSessionName('')
      loadSessions()
    } catch (err) {
      console.error('Failed to save session:', err)
    }
  }

  const handleLoadSession = async (name: string) => {
    try {
      const session = await invoke<Session>('session_load', { name })
      onLoadSession(session)
    } catch (err) {
      console.error('Failed to load session:', err)
    }
  }

  const handleDeleteSession = async (name: string) => {
    if (!confirm(`Delete session "${name}"?`)) return
    
    try {
      await invoke('session_delete', { name })
      loadSessions()
    } catch (err) {
      console.error('Failed to delete session:', err)
    }
  }

  const formatDate = (timestamp: number): string => {
    return new Date(timestamp).toLocaleString()
  }

  return (
    <div className="session-manager">
      <div className="session-header">
        <h3>💾 Sessions</h3>
        <button className="btn-save" onClick={() => setShowSaveDialog(true)}>
          Save Current
        </button>
      </div>

      <div className="session-list">
        {sessions.length === 0 ? (
          <div className="session-empty">
            <p>No saved sessions</p>
            <p className="hint">Save your current workspace as a session</p>
          </div>
        ) : (
          sessions.map(session => (
            <div key={session.name} className="session-item">
              <div className="session-info">
                <div className="session-name">{session.name}</div>
                <div className="session-meta">
                  {session.files.length} files • Last modified: {formatDate(session.modified_at)}
                </div>
              </div>
              <div className="session-actions">
                <button 
                  className="btn-load"
                  onClick={() => handleLoadSession(session.name)}
                >
                  Load
                </button>
                <button 
                  className="btn-delete"
                  onClick={() => handleDeleteSession(session.name)}
                >
                  🗑️
                </button>
              </div>
            </div>
          ))
        )}
      </div>

      {showSaveDialog && (
        <div className="session-dialog-overlay" onClick={() => setShowSaveDialog(false)}>
          <div className="session-dialog" onClick={e => e.stopPropagation()}>
            <h3>Save Session</h3>
            <input
              type="text"
              value={sessionName}
              onChange={e => setSessionName(e.target.value)}
              placeholder="Session name"
              autoFocus
            />
            <div className="dialog-actions">
              <button className="btn-secondary" onClick={() => setShowSaveDialog(false)}>
                Cancel
              </button>
              <button 
                className="btn-primary" 
                onClick={handleSaveSession}
                disabled={!sessionName.trim()}
              >
                Save
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  )
}

export default SessionManager
