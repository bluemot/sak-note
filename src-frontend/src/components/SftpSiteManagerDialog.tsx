import { useState, useEffect, useRef } from 'react'
import { invoke } from '@tauri-apps/api/core'
import './SftpSiteManagerDialog.css'

export interface SftpSite {
  id: string
  name: string
  host: string
  port: number
  username: string
  password?: string
}

interface SftpSiteManagerDialogProps {
  isOpen: boolean
  onClose: () => void
  onConnect: (site: SftpSite) => void
}

// Check if running inside Tauri
function isTauriAvailable(): boolean {
  return !!(window as any).__TAURI_INTERNALS__
}

export function SftpSiteManagerDialog({ isOpen, onClose, onConnect }: SftpSiteManagerDialogProps) {
  const [sites, setSites] = useState<SftpSite[]>([])
  const [selectedSite, setSelectedSite] = useState<SftpSite | null>(null)
  const [isEditing, setIsEditing] = useState(false)
  const [isAdding, setIsAdding] = useState(false)
  const [connectionStatus, setConnectionStatus] = useState<Record<string, 'connecting' | 'connected' | 'error'>>({})
  
  // Form state
  const [formData, setFormData] = useState<Partial<SftpSite>>({
    name: '',
    host: '',
    port: 22,
    username: '',
    password: '',
  })
  
  const nameInputRef = useRef<HTMLInputElement>(null)
  const listRef = useRef<HTMLDivElement>(null)
  const [focusedIndex, setFocusedIndex] = useState(-1)

  // Load sites on mount / when dialog opens
  useEffect(() => {
    if (isOpen) {
      loadSites()
      setSelectedSite(null)
      setFocusedIndex(-1)
    }
  }, [isOpen])

  // Focus management
  useEffect(() => {
    if (isOpen && (isAdding || isEditing)) {
      setTimeout(() => nameInputRef.current?.focus(), 100)
    }
  }, [isOpen, isAdding, isEditing])

  // Keyboard navigation
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (!isOpen) return
      
      if (e.key === 'Escape') {
        if (isAdding || isEditing) {
          handleCancelEdit()
        } else {
          onClose()
        }
        return
      }

      if (isAdding || isEditing) {
        if (e.key === 'Enter' && e.ctrlKey) {
          handleSaveSite()
        }
        return
      }

      if (e.key === 'ArrowDown') {
        e.preventDefault()
        setFocusedIndex(prev => {
          const newIndex = prev < sites.length - 1 ? prev + 1 : 0
          setSelectedSite(sites[newIndex] || null)
          return newIndex
        })
      } else if (e.key === 'ArrowUp') {
        e.preventDefault()
        setFocusedIndex(prev => {
          const newIndex = prev > 0 ? prev - 1 : sites.length - 1
          setSelectedSite(sites[newIndex] || null)
          return newIndex
        })
      } else if (e.key === 'Enter' && selectedSite) {
        e.preventDefault()
        handleConnect(selectedSite)
      } else if (e.key === 'Delete' && selectedSite) {
        e.preventDefault()
        handleDeleteSite(selectedSite.id)
      } else if (e.key === 'n' && e.ctrlKey) {
        e.preventDefault()
        handleAddNew()
      } else if (e.key === 'e' && e.ctrlKey && selectedSite) {
        e.preventDefault()
        handleEdit(selectedSite)
      }
    }

    window.addEventListener('keydown', handleKeyDown)
    return () => window.removeEventListener('keydown', handleKeyDown)
  }, [isOpen, sites, selectedSite, isAdding, isEditing])

  const loadSites = async () => {
    if (isTauriAvailable()) {
      try {
        const result = await invoke<{ sites: any[] }>('sftp_list_sites')
        const mapped: SftpSite[] = (result.sites || []).map((s: any) => ({
          id: s.id,
          name: s.name,
          host: s.hostname || s.host || '',
          port: s.port || 22,
          username: s.username || '',
          password: undefined, // never expose passwords from backend
        }))
        setSites(mapped)
      } catch (err) {
        console.error('[SftpSiteManager] Failed to load sites from backend, falling back to localStorage:', err)
        loadSitesFromLocalStorage()
      }
    } else {
      loadSitesFromLocalStorage()
    }
  }

  const loadSitesFromLocalStorage = () => {
    try {
      const stored = localStorage.getItem('sak-sftp-sites')
      if (stored) {
        setSites(JSON.parse(stored))
      }
    } catch (err) {
      console.error('Failed to load sites from localStorage:', err)
    }
  }

  const saveSitesToLocalStorage = (newSites: SftpSite[]) => {
    try {
      localStorage.setItem('sak-sftp-sites', JSON.stringify(newSites))
    } catch (err) {
      console.error('Failed to save sites to localStorage:', err)
    }
  }

  const handleAddNew = () => {
    setIsAdding(true)
    setIsEditing(false)
    setFormData({
      name: '',
      host: '',
      port: 22,
      username: '',
      password: '',
    })
  }

  const handleEdit = (site: SftpSite) => {
    setIsEditing(true)
    setIsAdding(false)
    setFormData({ ...site, password: '' })
    setSelectedSite(site)
  }

  const handleCancelEdit = () => {
    setIsAdding(false)
    setIsEditing(false)
    setFormData({ name: '', host: '', port: 22, username: '', password: '' })
  }

  const handleSaveSite = async () => {
    if (!formData.name || !formData.host || !formData.username) {
      return
    }

    if (isAdding) {
      const newSite: SftpSite = {
        id: crypto.randomUUID(),
        name: formData.name!,
        host: formData.host!,
        port: formData.port || 22,
        username: formData.username!,
        password: formData.password,
      }

      if (isTauriAvailable()) {
        try {
          await invoke('sftp_add_site', {
            request: {
              site: {
                id: newSite.id,
                name: newSite.name,
                hostname: newSite.host,
                port: newSite.port,
                username: newSite.username,
                default_path: '',
                group: '',
                notes: '',
              },
              password: newSite.password || null,
              ssh_key_path: null,
            }
          })
          // Reload from backend to get the canonical site data
          await loadSites()
        } catch (err) {
          console.error('[SftpSiteManager] Failed to add site via backend:', err)
          // Fallback: save locally
          const updated = [...sites, newSite]
          setSites(updated)
          saveSitesToLocalStorage(updated)
        }
      } else {
        const updated = [...sites, newSite]
        setSites(updated)
        saveSitesToLocalStorage(updated)
      }
    } else if (isEditing && selectedSite) {
      const updatedSite: SftpSite = {
        ...selectedSite,
        name: formData.name!,
        host: formData.host!,
        port: formData.port || 22,
        username: formData.username!,
        ...(formData.password ? { password: formData.password } : {}),
      }

      if (isTauriAvailable()) {
        try {
          await invoke('sftp_update_site', {
            site: {
              id: updatedSite.id,
              name: updatedSite.name,
              hostname: updatedSite.host,
              port: updatedSite.port,
              username: updatedSite.username,
              default_path: '',
              group: '',
              notes: '',
            },
            password: formData.password || null,
          })
          await loadSites()
        } catch (err) {
          console.error('[SftpSiteManager] Failed to update site via backend:', err)
          // Fallback: update locally
          const updated = sites.map(s => s.id === selectedSite.id ? updatedSite : s)
          setSites(updated)
          saveSitesToLocalStorage(updated)
        }
      } else {
        const updated = sites.map(s => s.id === selectedSite.id ? updatedSite : s)
        setSites(updated)
        saveSitesToLocalStorage(updated)
      }
    }

    handleCancelEdit()
  }

  const handleDeleteSite = async (siteId: string) => {
    if (!confirm('Are you sure you want to delete this site?')) return

    if (isTauriAvailable()) {
      try {
        await invoke('sftp_remove_site', { siteId })
        await loadSites()
      } catch (err) {
        console.error('[SftpSiteManager] Failed to delete site via backend:', err)
        // Fallback: delete locally
        const updated = sites.filter(s => s.id !== siteId)
        setSites(updated)
        saveSitesToLocalStorage(updated)
      }
    } else {
      const updated = sites.filter(s => s.id !== siteId)
      setSites(updated)
      saveSitesToLocalStorage(updated)
    }

    if (selectedSite?.id === siteId) {
      setSelectedSite(null)
      setFocusedIndex(-1)
    }
  }

  const handleConnect = async (site: SftpSite) => {
    setConnectionStatus(prev => ({ ...prev, [site.id]: 'connecting' }))
    
    try {
      if (isTauriAvailable()) {
        await invoke('sftp_connect_site', { siteId: site.id })
      } else {
        // Fallback: use legacy sftp_connect for browser dev mode
        await invoke('sftp_connect', {
          host: site.host,
          port: site.port,
          username: site.username,
          password: site.password,
        })
      }
      
      setConnectionStatus(prev => ({ ...prev, [site.id]: 'connected' }))
      onConnect(site)
      onClose()
    } catch (err) {
      console.error('Failed to connect:', err)
      setConnectionStatus(prev => ({ ...prev, [site.id]: 'error' }))
    }
  }

  const handleSelectSite = (site: SftpSite, index: number) => {
    setSelectedSite(site)
    setFocusedIndex(index)
  }

  if (!isOpen) return null

  return (
    <div className="sftp-dialog-overlay" onClick={onClose}>
      <div className="sftp-dialog" onClick={e => e.stopPropagation()}>
        <div className="sftp-dialog-header">
          <h3>SFTP Site Manager</h3>
          <button className="btn-close" onClick={onClose}>×</button>
        </div>

        <div className="sftp-dialog-body">
          {(isAdding || isEditing) ? (
            <div className="sftp-form">
              <div className="form-row">
                <label>Site Name *</label>
                <input
                  ref={nameInputRef}
                  type="text"
                  value={formData.name}
                  onChange={e => setFormData({ ...formData, name: e.target.value })}
                  placeholder="e.g., Production Server"
                />
              </div>

              <div className="form-row">
                <label>Host *</label>
                <input
                  type="text"
                  value={formData.host}
                  onChange={e => setFormData({ ...formData, host: e.target.value })}
                  placeholder="e.g., example.com"
                />
              </div>

              <div className="form-row form-row-half">
                <div>
                  <label>Port</label>
                  <input
                    type="number"
                    value={formData.port}
                    onChange={e => setFormData({ ...formData, port: parseInt(e.target.value) || 22 })}
                  />
                </div>
              </div>

              <div className="form-row">
                <label>Username *</label>
                <input
                  type="text"
                  value={formData.username}
                  onChange={e => setFormData({ ...formData, username: e.target.value })}
                  placeholder="e.g., root"
                />
              </div>

              <div className="form-row">
                <label>Password {isEditing ? '(leave blank to keep unchanged)' : ''}</label>
                <input
                  type="password"
                  value={formData.password || ''}
                  onChange={e => setFormData({ ...formData, password: e.target.value })}
                  placeholder={isEditing ? '••••••••' : ''}
                />
              </div>

              <div className="form-help">
                <kbd>Ctrl</kbd> + <kbd>Enter</kbd> to save
              </div>
            </div>
          ) : (
            <div className="sftp-sites-container">
              <div className="sftp-sites-toolbar">
                <button className="btn-toolbar" onClick={handleAddNew} title="Add New Site (Ctrl+N)">
                  + Add Site
                </button>
                <span className="sftp-sites-count">{sites.length} site(s)</span>
              </div>

              <div className="sftp-sites-list" ref={listRef}>
                {sites.length === 0 ? (
                  <div className="sftp-empty">
                    <p>No sites configured</p>
                    <p className="hint">Click "Add Site" to create one</p>
                  </div>
                ) : (
                  sites.map((site, index) => (
                    <div
                      key={site.id}
                      className={`site-item ${selectedSite?.id === site.id ? 'selected' : ''} ${focusedIndex === index ? 'focused' : ''}`}
                      onClick={() => handleSelectSite(site, index)}
                      onDoubleClick={() => handleConnect(site)}
                    >
                      <div className="site-info">
                        <div className="site-name">{site.name}</div>
                        <div className="site-details">
                          {site.username}@{site.host}:{site.port}
                        </div>
                      </div>
                      
                      <div className="site-status">
                        {connectionStatus[site.id] === 'connecting' && (
                          <span className="status connecting">Connecting...</span>
                        )}
                        {connectionStatus[site.id] === 'connected' && (
                          <span className="status connected">Connected</span>
                        )}
                        {connectionStatus[site.id] === 'error' && (
                          <span className="status error">Failed</span>
                        )}
                      </div>
                    </div>
                  ))
                )}
              </div>

              {selectedSite && (
                <div className="sftp-site-actions">
                  <button className="btn-action btn-connect" onClick={() => handleConnect(selectedSite)}>
                    Connect
                  </button>
                  <button className="btn-action btn-edit" onClick={() => handleEdit(selectedSite)}>
                    Edit
                  </button>
                  <button className="btn-action btn-delete" onClick={() => handleDeleteSite(selectedSite.id)}>
                    Delete
                  </button>
                </div>
              )}

              <div className="sftp-keyboard-hints">
                <span><kbd>↑</kbd><kbd>↓</kbd> Navigate</span>
                <span><kbd>Enter</kbd> Connect</span>
                <span><kbd>Ctrl</kbd><kbd>N</kbd> Add</span>
                <span><kbd>Ctrl</kbd><kbd>E</kbd> Edit</span>
                <span><kbd>Del</kbd> Delete</span>
                <span><kbd>Esc</kbd> Close</span>
              </div>
            </div>
          )}
        </div>

        <div className="sftp-dialog-footer">
          {(isAdding || isEditing) ? (
            <>
              <button className="btn-secondary" onClick={handleCancelEdit}>
                Cancel
              </button>
              <button
                className="btn-primary"
                onClick={handleSaveSite}
                disabled={!formData.name || !formData.host || !formData.username}
              >
                {isAdding ? 'Add Site' : 'Save Changes'}
              </button>
            </>
          ) : (
            <button className="btn-secondary" onClick={onClose}>
              Close
            </button>
          )}
        </div>
      </div>
    </div>
  )
}

export default SftpSiteManagerDialog