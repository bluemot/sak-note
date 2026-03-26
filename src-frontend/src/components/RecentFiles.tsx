import { useState, useEffect } from 'react'
import { invoke } from '@tauri-apps/api/core'
import './RecentFiles.css'

interface RecentFile {
  path: string
  name: string
  lastOpened: number
}

interface RecentFilesProps {
  onOpenFile: (path: string) => void
}

export function RecentFiles({ onOpenFile }: RecentFilesProps) {
  const [recentFiles, setRecentFiles] = useState<RecentFile[]>([])
  const [isLoading, setIsLoading] = useState(false)

  useEffect(() => {
    loadRecentFiles()
  }, [])

  const loadRecentFiles = async () => {
    setIsLoading(true)
    try {
      const files = await invoke<RecentFile[]>('file_get_recent_files')
      setRecentFiles(files)
    } catch (err) {
      console.error('Failed to load recent files:', err)
    } finally {
      setIsLoading(false)
    }
  }

  const handleClearRecent = async () => {
    if (!confirm('Clear recent files list?')) return
    try {
      await invoke('file_clear_recent_files')
      setRecentFiles([])
    } catch (err) {
      console.error('Failed to clear recent files:', err)
    }
  }

  const formatDate = (timestamp: number): string => {
    const date = new Date(timestamp)
    const now = new Date()
    const diff = now.getTime() - date.getTime()
    const days = Math.floor(diff / (1000 * 60 * 60 * 24))

    if (days === 0) {
      return date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })
    } else if (days === 1) {
      return 'Yesterday'
    } else if (days < 7) {
      return `${days} days ago`
    } else {
      return date.toLocaleDateString()
    }
  }

  const getFileIcon = (path: string): string => {
    const ext = path.split('.').pop()?.toLowerCase() || ''
    const icons: Record<string, string> = {
      'rs': '🦀',
      'js': '📜',
      'ts': '📘',
      'tsx': '⚛️',
      'jsx': '⚛️',
      'py': '🐍',
      'go': '🐹',
      'c': '🔧',
      'cpp': '🔧',
      'h': '🔧',
      'java': '☕',
      'md': '📝',
      'json': '📋',
      'toml': '⚙️',
      'yaml': '⚙️',
      'yml': '⚙️',
      'css': '🎨',
      'scss': '🎨',
      'html': '🌐',
      'sql': '🗄️',
      'sh': '🔨',
      'bash': '🔨',
      'dockerfile': '🐳',
    }
    return icons[ext] || '📄'
  }

  return (
    <div className="recent-files">
      <div className="recent-files-header">
        <h3>🕐 Recent Files</h3>
        {recentFiles.length > 0 && (
          <button className="btn-clear" onClick={handleClearRecent}>
            Clear
          </button>
        )}
      </div>

      <div className="recent-files-list">
        {isLoading ? (
          <div className="recent-files-loading">Loading...</div>
        ) : recentFiles.length === 0 ? (
          <div className="recent-files-empty">
            <p>No recent files</p>
            <p className="hint">Files you open will appear here</p>
          </div>
        ) : (
          recentFiles.map((file) => (
            <div
              key={file.path}
              className="recent-file-item"
              onClick={() => onOpenFile(file.path)}
              title={file.path}
            >
              <span className="recent-file-icon">{getFileIcon(file.path)}</span>
              <div className="recent-file-info">
                <div className="recent-file-name">{file.name}</div>
                <div className="recent-file-path">{file.path}</div>
              </div>
              <span className="recent-file-date">{formatDate(file.lastOpened)}</span>
            </div>
          ))
        )}
      </div>
    </div>
  )
}

export default RecentFiles
