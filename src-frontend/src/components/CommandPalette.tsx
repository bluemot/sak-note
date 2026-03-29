import { useState, useEffect, useRef, useMemo, useCallback } from 'react'
import { actionRegistry } from '../ui-system/actions/actionRegistry'
import './CommandPalette.css'

interface CommandItem {
  id: string
  label: string
  description?: string
  shortcut?: string
  category?: string
}

interface CommandPaletteProps {
  isOpen: boolean
  onClose: () => void
}

// Built-in commands that are always available
const BUILTIN_COMMANDS: CommandItem[] = [
  { id: 'file:new', label: 'New File', description: 'Create a new file', shortcut: 'Ctrl+N', category: 'File' },
  { id: 'file:open', label: 'Open File', description: 'Open an existing file', shortcut: 'Ctrl+O', category: 'File' },
  { id: 'file:save', label: 'Save', description: 'Save current file', shortcut: 'Ctrl+S', category: 'File' },
  { id: 'file:save_as', label: 'Save As...', description: 'Save with a new name', shortcut: 'Ctrl+Shift+S', category: 'File' },
  { id: 'file:close', label: 'Close File', description: 'Close current file', shortcut: 'Ctrl+W', category: 'File' },
  
  { id: 'edit:undo', label: 'Undo', description: 'Undo last action', shortcut: 'Ctrl+Z', category: 'Edit' },
  { id: 'edit:redo', label: 'Redo', description: 'Redo last action', shortcut: 'Ctrl+Y', category: 'Edit' },
  { id: 'edit:cut', label: 'Cut', description: 'Cut selection', shortcut: 'Ctrl+X', category: 'Edit' },
  { id: 'edit:copy', label: 'Copy', description: 'Copy selection', shortcut: 'Ctrl+C', category: 'Edit' },
  { id: 'edit:paste', label: 'Paste', description: 'Paste from clipboard', shortcut: 'Ctrl+V', category: 'Edit' },
  { id: 'edit:find', label: 'Find', description: 'Find in current file', shortcut: 'Ctrl+F', category: 'Edit' },
  { id: 'edit:replace', label: 'Replace', description: 'Find and replace', shortcut: 'Ctrl+H', category: 'Edit' },
  { id: 'edit:find_in_files', label: 'Find in Files', description: 'Search across all files', shortcut: 'Ctrl+Shift+F', category: 'Edit' },
  { id: 'edit:goto_line', label: 'Go to Line', description: 'Jump to specific line', shortcut: 'Ctrl+G', category: 'Edit' },
  
  { id: 'view:toggle_sidebar', label: 'Toggle Sidebar', description: 'Show/hide sidebar', shortcut: 'Ctrl+B', category: 'View' },
  { id: 'view:word_wrap', label: 'Toggle Word Wrap', description: 'Enable/disable word wrap', category: 'View' },
  { id: 'view:line_numbers', label: 'Toggle Line Numbers', description: 'Show/hide line numbers', category: 'View' },
  { id: 'view:minimap', label: 'Toggle Minimap', description: 'Show/hide minimap', category: 'View' },
  
  { id: 'editor:zoom_in', label: 'Zoom In', description: 'Increase font size', shortcut: 'Ctrl+=', category: 'Editor' },
  { id: 'editor:zoom_out', label: 'Zoom Out', description: 'Decrease font size', shortcut: 'Ctrl+-', category: 'Editor' },
  { id: 'editor:zoom_reset', label: 'Reset Zoom', description: 'Reset font size', shortcut: 'Ctrl+0', category: 'Editor' },
  { id: 'editor:fold_all', label: 'Fold All', description: 'Collapse all code sections', category: 'Editor' },
  { id: 'editor:unfold_all', label: 'Unfold All', description: 'Expand all code sections', category: 'Editor' },
  { id: 'editor:command_palette', label: 'Command Palette', description: 'Open this palette', shortcut: 'Ctrl+Shift+P', category: 'Editor' },
  
  { id: 'sftp:site_manager', label: 'SFTP Site Manager', description: 'Manage SFTP connections', category: 'SFTP' },
  { id: 'sftp:connect', label: 'SFTP Connect', description: 'Connect to remote server', category: 'SFTP' },
  { id: 'sftp:disconnect', label: 'SFTP Disconnect', description: 'Disconnect from server', category: 'SFTP' },
  
  { id: 'marks:create', label: 'Create Mark', description: 'Add bookmark at cursor', category: 'Marks' },
  { id: 'marks:clear', label: 'Clear All Marks', description: 'Remove all bookmarks', category: 'Marks' },
  { id: 'bookmark:create', label: 'Create Bookmark', description: 'Add bookmark at cursor', category: 'Bookmarks' },
  { id: 'bookmark:clear', label: 'Clear Bookmarks', description: 'Remove all bookmarks', category: 'Bookmarks' },
  
  { id: 'llm:chat', label: 'AI Chat', description: 'Open AI chat panel', shortcut: 'Ctrl+Shift+A', category: 'AI' },
  { id: 'llm:summarize', label: 'Summarize File', description: 'Generate file summary', category: 'AI' },
  { id: 'llm:settings', label: 'AI Settings', description: 'Configure AI provider', category: 'AI' },
  { id: 'llm:clear_history', label: 'Clear Chat History', description: 'Reset chat history', category: 'AI' },
]

export function CommandPalette({ isOpen, onClose }: CommandPaletteProps) {
  const [query, setQuery] = useState('')
  const [selectedIndex, setSelectedIndex] = useState(0)
  const [commands, setCommands] = useState<CommandItem[]>(BUILTIN_COMMANDS)
  const inputRef = useRef<HTMLInputElement>(null)
  const listRef = useRef<HTMLDivElement>(null)

  // Load dynamic commands from action registry
  useEffect(() => {
    if (isOpen) {
      const registeredActions = actionRegistry.getAllActions()
      const dynamicCommands = registeredActions
        .filter(id => !BUILTIN_COMMANDS.some(cmd => cmd.id === id))
        .map(id => {
          const parts = id.split(':')
          const module = parts[0] || 'unknown'
          const action = parts[1] || id
          return {
            id,
            label: `${action.charAt(0).toUpperCase() + action.slice(1).replace(/_/g, ' ')}`,
            description: `${module} action`,
            category: module.charAt(0).toUpperCase() + module.slice(1),
          }
        })
      
      setCommands([...BUILTIN_COMMANDS, ...dynamicCommands])
    }
  }, [isOpen])

  // Filter commands based on query
  const filteredCommands = useMemo(() => {
    if (!query.trim()) return commands
    
    const lowerQuery = query.toLowerCase()
    return commands.filter(cmd => 
      cmd.label.toLowerCase().includes(lowerQuery) ||
      (cmd.description?.toLowerCase().includes(lowerQuery) ?? false) ||
      (cmd.category?.toLowerCase().includes(lowerQuery) ?? false) ||
      cmd.id.toLowerCase().includes(lowerQuery)
    )
  }, [commands, query])

  // Reset selection when query changes
  useEffect(() => {
    setSelectedIndex(0)
  }, [query])

  // Focus input when opened
  useEffect(() => {
    if (isOpen) {
      setQuery('')
      setSelectedIndex(0)
      setTimeout(() => inputRef.current?.focus(), 100)
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

      if (e.key === 'ArrowDown') {
        e.preventDefault()
        setSelectedIndex(prev => 
          prev < filteredCommands.length - 1 ? prev + 1 : prev
        )
        scrollToSelected()
      } else if (e.key === 'ArrowUp') {
        e.preventDefault()
        setSelectedIndex(prev => prev > 0 ? prev - 1 : 0)
        scrollToSelected()
      } else if (e.key === 'Enter') {
        e.preventDefault()
        if (filteredCommands[selectedIndex]) {
          executeCommand(filteredCommands[selectedIndex])
        }
      } else if (e.key === 'PageDown') {
        e.preventDefault()
        setSelectedIndex(prev => 
          Math.min(prev + 10, filteredCommands.length - 1)
        )
        scrollToSelected()
      } else if (e.key === 'PageUp') {
        e.preventDefault()
        setSelectedIndex(prev => Math.max(prev - 10, 0))
        scrollToSelected()
      } else if (e.key === 'Home') {
        e.preventDefault()
        setSelectedIndex(0)
        scrollToSelected()
      } else if (e.key === 'End') {
        e.preventDefault()
        setSelectedIndex(filteredCommands.length - 1)
        scrollToSelected()
      }
    }

    window.addEventListener('keydown', handleKeyDown)
    return () => window.removeEventListener('keydown', handleKeyDown)
  }, [isOpen, filteredCommands, selectedIndex])

  const scrollToSelected = useCallback(() => {
    const element = listRef.current?.querySelector(`[data-index="${selectedIndex}"]`)
    element?.scrollIntoView({ block: 'nearest', behavior: 'smooth' })
  }, [selectedIndex])

  const executeCommand = async (command: CommandItem) => {
    onClose()
    
    try {
      // Try to execute via action registry first
      if (actionRegistry.has(command.id)) {
        await actionRegistry.execute(command.id)
      } else {
        // Dispatch custom event for built-in commands
        window.dispatchEvent(new CustomEvent('sak:command', { 
          detail: { command: command.id }
        }))
      }
    } catch (err) {
      console.error(`Failed to execute command ${command.id}:`, err)
    }
  }

  // Group commands by category for display
  const groupedCommands = useMemo(() => {
    const groups: Record<string, CommandItem[]> = {}
    filteredCommands.forEach(cmd => {
      const cat = cmd.category || 'Other'
      if (!groups[cat]) groups[cat] = []
      groups[cat].push(cmd)
    })
    return groups
  }, [filteredCommands])

  // Flat list for keyboard navigation
  const flatCommands = useMemo(() => {
    return filteredCommands
  }, [filteredCommands])

  const highlightMatch = (text: string, queryText: string): React.ReactNode => {
    if (!queryText) return text
    const parts = text.split(new RegExp(`(${queryText.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')})`, 'gi'))
    return parts.map((part, i) => 
      part.toLowerCase() === queryText.toLowerCase() 
        ? <mark key={i}>{part}</mark> 
        : part
    )
  }

  if (!isOpen) return null

  return (
    <div className="command-palette-overlay" onClick={onClose}>
      <div className="command-palette" onClick={e => e.stopPropagation()}>
        <div className="command-palette-header">
          <div className="command-palette-input-wrapper">
            <span className="command-palette-icon">{'>'}_</span>
            <input
              ref={inputRef}
              type="text"
              value={query}
              onChange={e => setQuery(e.target.value)}
              placeholder="Type a command or search..."
              className="command-palette-input"
              spellCheck={false}
            />
            {query && (
              <button 
                className="command-palette-clear"
                onClick={() => { setQuery(''); inputRef.current?.focus(); }}
              >
                ×
              </button>
            )}
          </div>
          <div className="command-palette-count">
            {filteredCommands.length} command{filteredCommands.length !== 1 ? 's' : ''}
          </div>
        </div>

        <div className="command-palette-list" ref={listRef}>
          {filteredCommands.length === 0 ? (
            <div className="command-palette-empty">
              <p>No commands found</p>
              <p className="hint">Try a different search term</p>
            </div>
          ) : query ? (
            // Flat list when searching
            flatCommands.map((command, index) => (
              <div
                key={command.id}
                data-index={index}
                className={`command-item ${index === selectedIndex ? 'selected' : ''}`}
                onClick={() => executeCommand(command)}
                onMouseEnter={() => setSelectedIndex(index)}
              >
                <div className="command-item-main">
                  <div className="command-item-label">
                    {highlightMatch(command.label, query)}
                  </div>
                  <div className="command-item-description">
                    {highlightMatch(command.description || '', query)}
                  </div>
                </div>
                <div className="command-item-meta">
                  {command.category && (
                    <span className="command-category">{command.category}</span>
                  )}
                  {command.shortcut && (
                    <kbd className="command-shortcut">{command.shortcut}</kbd>
                  )}
                </div>
              </div>
            ))
          ) : (
            // Grouped list when not searching
            Object.entries(groupedCommands).map(([category, cmds]) => (
              <div key={category} className="command-group">
                <div className="command-group-header">{category}</div>
                {cmds.map((command) => {
                  const globalIndex = flatCommands.findIndex(c => c.id === command.id)
                  return (
                    <div
                      key={command.id}
                      data-index={globalIndex}
                      className={`command-item ${globalIndex === selectedIndex ? 'selected' : ''}`}
                      onClick={() => executeCommand(command)}
                      onMouseEnter={() => setSelectedIndex(globalIndex)}
                    >
                      <div className="command-item-main">
                        <div className="command-item-label">{command.label}</div>
                        <div className="command-item-description">{command.description}</div>
                      </div>
                      <div className="command-item-meta">
                        {command.shortcut && (
                          <kbd className="command-shortcut">{command.shortcut}</kbd>
                        )}
                      </div>
                    </div>
                  )
                })}
              </div>
            ))
          )}
        </div>

        <div className="command-palette-footer">
          <div className="keyboard-hints">
            <span><kbd>↑</kbd> <kbd>↓</kbd> Navigate</span>
            <span><kbd>Enter</kbd> Execute</span>
            <span><kbd>Esc</kbd> Close</span>
          </div>
        </div>
      </div>
    </div>
  )
}

export default CommandPalette
