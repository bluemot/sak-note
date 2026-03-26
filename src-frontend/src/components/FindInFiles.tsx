import { useState, useEffect, useRef } from 'react'
import { invoke } from '@tauri-apps/api/core'
import './FindInFiles.css'

interface SearchResult {
  path: string
  line: number
  column: number
  text: string
  context: string
}

interface FindInFilesProps {
  isOpen: boolean
  onClose: () => void
  onOpenResult: (result: SearchResult) => void
}

export function FindInFiles({ isOpen, onClose, onOpenResult }: FindInFilesProps) {
  const [query, setQuery] = useState('')
  const [directory, setDirectory] = useState('')
  const [filters, setFilters] = useState('*')
  const [caseSensitive, setCaseSensitive] = useState(false)
  const [regex, setRegex] = useState(false)
  const [recursive, setRecursive] = useState(true)
  const [results, setResults] = useState<SearchResult[]>([])
  const [isSearching, setIsSearching] = useState(false)
  const inputRef = useRef<HTMLInputElement>(null)

  useEffect(() => {
    if (isOpen) {
      setTimeout(() => inputRef.current?.focus(), 100)
    }
  }, [isOpen])

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (!isOpen) return
      if (e.key === 'Escape') onClose()
    }

    window.addEventListener('keydown', handleKeyDown)
    return () => window.removeEventListener('keydown', handleKeyDown)
  }, [isOpen])

  const handleSearch = async () => {
    if (!query) return
    
    setIsSearching(true)
    try {
      const res = await invoke<SearchResult[]>('find_in_files', {
        query,
        directory: directory || undefined,
        filters,
        caseSensitive,
        regex,
        recursive
      })
      setResults(res)
    } catch (err) {
      console.error('Search failed:', err)
    } finally {
      setIsSearching(false)
    }
  }

  const formatPath = (path: string): string => {
    const parts = path.split('/')
    return parts.slice(-2).join('/')
  }

  if (!isOpen) return null

  return (
    <div className="find-in-files-overlay" onClick={onClose}>
      <div className="find-in-files-dialog" onClick={e => e.stopPropagation()}>
        <div className="find-in-files-header">
          <h3>🔍 Find in Files</h3>
          <button className="btn-close" onClick={onClose}>×</button>
        </div>

        <div className="find-in-files-body">
          <div className="search-form">
            <div className="form-row">
              <input
                ref={inputRef}
                type="text"
                value={query}
                onChange={e => setQuery(e.target.value)}
                placeholder="Find what"
                onKeyDown={e => e.key === 'Enter' && handleSearch()}
              />
            </div>

            <div className="form-row form-row-half">
              <input
                type="text"
                value={directory}
                onChange={e => setDirectory(e.target.value)}
                placeholder="Directory (leave empty for current)"
              />
              <input
                type="text"
                value={filters}
                onChange={e => setFilters(e.target.value)}
                placeholder="Filters: *.rs,*.ts"
              />
            </div>

            <div className="search-options">
              <label className="checkbox-label">
                <input
                  type="checkbox"
                  checked={caseSensitive}
                  onChange={e => setCaseSensitive(e.target.checked)}
                />
                Match case
              </label>
              <label className="checkbox-label">
                <input
                  type="checkbox"
                  checked={regex}
                  onChange={e => setRegex(e.target.checked)}
                />
                Regular expression
              </label>
              <label className="checkbox-label">
                <input
                  type="checkbox"
                  checked={recursive}
                  onChange={e => setRecursive(e.target.checked)}
                />
                Recursive search
              </label>
            </div>

            <button 
              className="btn-search" 
              onClick={handleSearch}
              disabled={!query || isSearching}
            >
              {isSearching ? 'Searching...' : 'Find All'}
            </button>
          </div>

          {results.length > 0 && (
            <div className="search-results">
              <div className="results-header">
                Found {results.length} matches
              </div>
              <div className="results-list">
                {results.map((result, idx) => (
                  <div 
                    key={idx} 
                    className="result-item"
                    onClick={() => onOpenResult(result)}
                  >
                    <div className="result-path">{formatPath(result.path)}</div>
                    <div className="result-context">
                      <span className="result-line">Line {result.line}:</span>
                      {' '}
                      <span dangerouslySetInnerHTML={{ 
                        __html: result.context.replace(
                          new RegExp(query, caseSensitive ? 'g' : 'gi'),
                          match => `<mark>${match}</mark>`
                        ) 
                      }} />
                    </div>
                  </div>
                ))}
              </div>
            </div>
          )}
        </div>
      </div>
    </div>
  )
}

export default FindInFiles
