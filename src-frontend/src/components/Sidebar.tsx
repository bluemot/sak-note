import { useState } from 'react'
import LlmChat from './LlmChat'
import './Sidebar.css'

interface SidebarProps {
  currentFile: { path: string; size: number; chunks: number; chunk_size: number } | null
}

function formatBytes(bytes: number): string {
  if (bytes === 0) return '0 B'
  const k = 1024
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB']
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i]
}

function Sidebar({ currentFile }: SidebarProps) {
  const [activeTab, setActiveTab] = useState<'info' | 'chat' | 'marks'>('info')

  return (
    <div className="sidebar">
      <div className="sidebar-tabs">
        <button
          className={`tab ${activeTab === 'info' ? 'active' : ''}`}
          onClick={() => setActiveTab('info')}
        >
          ℹ️ Info
        </button>
        <button
          className={`tab ${activeTab === 'chat' ? 'active' : ''}`}
          onClick={() => setActiveTab('chat')}
        >
          🤖 Chat
        </button>
        <button
          className={`tab ${activeTab === 'marks' ? 'active' : ''}`}
          onClick={() => setActiveTab('marks')}
        >
          🎨 Marks
        </button>
      </div>

      <div className="sidebar-content">
        {activeTab === 'info' && (
          <div className="info-panel">
            {currentFile ? (
              <>
                <div className="info-item">
                  <span className="label">File:</span>
                  <span className="value" title={currentFile.path}>
                    {currentFile.path.split('/').pop() || currentFile.path}
                  </span>
                </div>
                <div className="info-item">
                  <span className="label">Size:</span>
                  <span className="value">{formatBytes(currentFile.size)}</span>
                </div>
                <div className="info-item">
                  <span className="label">Chunks:</span>
                  <span className="value">{currentFile.chunks}</span>
                </div>
                <div className="info-item">
                  <span className="label">Chunk Size:</span>
                  <span className="value">{formatBytes(currentFile.chunk_size)}</span>
                </div>
              </>
            ) : (
              <p className="no-file">No file open</p>
            )}
          </div>
        )}

        {activeTab === 'chat' && (
          <div className="chat-panel">
            {currentFile ? (
              <LlmChat filePath={currentFile.path} />
            ) : (
              <div className="no-file-message">
                <p>🤖</p>
                <p>Open a file to start chatting</p>
              </div>
            )}
          </div>
        )}

        {activeTab === 'marks' && (
          <div className="marks-panel">
            <p className="placeholder">Color highlights will appear here</p>
          </div>
        )}
      </div>
    </div>
  )
}

export default Sidebar
