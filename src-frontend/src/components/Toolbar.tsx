import { useState, useCallback } from 'react'
import './Toolbar.css'

interface ToolbarProps {
  onOpenFile: () => void
  onCloseFile: () => void
  onToggleView: () => void
  viewMode: 'text' | 'hex'
  hasFile: boolean
  isLoading: boolean
}

function Toolbar({
  onOpenFile,
  onCloseFile,
  onToggleView,
  viewMode,
  hasFile,
  isLoading
}: ToolbarProps) {
  const [searchHistory, setSearchHistory] = useState<string[]>([])
  const [showSearch, setShowSearch] = useState(false)
  const [searchQuery, setSearchQuery] = useState('')

  const handleSearch = useCallback(() => {
    if (searchQuery.trim() && !searchHistory.includes(searchQuery.trim())) {
      setSearchHistory(prev => [searchQuery.trim(), ...prev].slice(0, 20))
    }
    // TODO: Implement actual search
    console.log('Searching for:', searchQuery)
  }, [searchQuery, searchHistory])

  return (
    <div className="toolbar">
      <div className="toolbar-section">
        <button 
          onClick={onOpenFile} 
          disabled={isLoading}
          className="toolbar-btn primary"
        >
          {isLoading ? 'Loading...' : '📂 Open'}
        </button>
        {hasFile && (
          <button 
            onClick={onCloseFile}
            className="toolbar-btn"
          >
            ❌ Close
          </button>
        )}
      </div>

      {hasFile && (
        <div className="toolbar-section">
          <button 
            onClick={onToggleView}
            className="toolbar-btn"
          >
            {viewMode === 'text' ? '🔍 Hex View' : '📝 Text View'}
          </button>
          <button 
            onClick={() => setShowSearch(!showSearch)}
            className={`toolbar-btn ${showSearch ? 'active' : ''}`}
          >
            🔎 Search
          </button>
        </div>
      )}

      {showSearch && (
        <div className="search-bar">
          <input
            type="text"
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            onKeyDown={(e) => e.key === 'Enter' && handleSearch()}
            placeholder="Search..."
            className="search-input"
            list="search-history"
          />
          <datalist id="search-history">
            {searchHistory.map((item, idx) => (
              <option key={idx} value={item} />
            ))}
          </datalist>
          <button onClick={handleSearch} className="toolbar-btn">
            Find
          </button>
        </div>
      )}
    </div>
  )
}

export default Toolbar
