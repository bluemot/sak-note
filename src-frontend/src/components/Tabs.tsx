import { useState, useCallback } from 'react'
import './Tabs.css'

interface Tab {
  id: string
  path: string
  name: string
  isDirty: boolean
  isActive: boolean
}

interface TabsProps {
  tabs: Tab[]
  activeTabId: string | null
  onTabClick: (id: string) => void
  onTabClose: (id: string) => void
  onTabReorder?: (dragId: string, dropId: string) => void
}

export function Tabs({ tabs, activeTabId, onTabClick, onTabClose, onTabReorder }: TabsProps) {
  const [draggedTab, setDraggedTab] = useState<string | null>(null)

  const handleDragStart = useCallback((e: React.DragEvent, tabId: string) => {
    setDraggedTab(tabId)
    e.dataTransfer.effectAllowed = 'move'
  }, [])

  const handleDragOver = useCallback((e: React.DragEvent) => {
    e.preventDefault()
    e.dataTransfer.dropEffect = 'move'
  }, [])

  const handleDrop = useCallback((e: React.DragEvent, dropId: string) => {
    e.preventDefault()
    if (draggedTab && draggedTab !== dropId && onTabReorder) {
      onTabReorder(draggedTab, dropId)
    }
    setDraggedTab(null)
  }, [draggedTab, onTabReorder])

  const handleDragEnd = useCallback(() => {
    setDraggedTab(null)
  }, [])

  if (tabs.length === 0) {
    return (
      <div className="tabs-empty">
        <span>No files open</span>
      </div>
    )
  }

  return (
    <div className="tabs-container">
      <div className="tabs-list">
        {tabs.map((tab) => (
          <div
            key={tab.id}
            className={`tab ${tab.isActive ? 'active' : ''} ${draggedTab === tab.id ? 'dragging' : ''}`}
            onClick={() => onTabClick(tab.id)}
            onMouseDown={(e) => {
              if (e.button === 1) {
                // Middle click to close
                e.preventDefault()
                onTabClose(tab.id)
              }
            }}
            draggable
            onDragStart={(e) => handleDragStart(e, tab.id)}
            onDragOver={handleDragOver}
            onDrop={(e) => handleDrop(e, tab.id)}
            onDragEnd={handleDragEnd}
            title={tab.path}
          >
            <span className="tab-icon">
              {getFileIcon(tab.name)}
            </span>
            <span className={`tab-name ${tab.isDirty ? 'unsaved' : ''}`}>
              {tab.name}
              {tab.isDirty && <span className="dirty-indicator">●</span>}
            </span>
            <button
              className="tab-close"
              onClick={(e) => {
                e.stopPropagation()
                onTabClose(tab.id)
              }}
              title="Close (Middle click)"
            >
              ×
            </button>
          </div>
        ))}
      </div>
    </div>
  )
}

function getFileIcon(filename: string): string {
  const ext = filename.split('.').pop()?.toLowerCase()
  
  const iconMap: Record<string, string> = {
    'rs': '🦀',
    'js': '📜',
    'ts': '📘',
    'tsx': '⚛️',
    'jsx': '⚛️',
    'py': '🐍',
    'go': '🐹',
    'c': '🔧',
    'cpp': '🔧',
    'h': '📋',
    'hpp': '📋',
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
  
  return iconMap[ext || ''] || '📄'
}

export type { Tab }
export default Tabs
