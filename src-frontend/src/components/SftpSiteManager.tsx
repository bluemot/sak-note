import { useState, useEffect, useCallback } from 'react'
import { invoke } from '@tauri-apps/api/core'
import './SftpSiteManager.css'

interface SftpSite {
  id: string
  name: string
  hostname: string
  port: number
  username: string
  default_path?: string
  group?: string
  last_connected?: string
  notes?: string
}

interface SftpSiteManagerProps {
  onConnect: (site: SftpSite) => void
}

export function SftpSiteManager({ onConnect }: SftpSiteManagerProps) {
  const [sites, setSites] = useState<SftpSite[]>([])
  const [groups, setGroups] = useState<string[]>(['Default'])
  const [selectedGroup, setSelectedGroup] = useState<string>('All')
  const [showAddDialog, setShowAddDialog] = useState(false)
  const [editingSite, setEditingSite] = useState<SftpSite | null>(null)
  const [connectionStatus, setConnectionStatus] = useState<Record<string, string>>({})

  // Form state
  const [formData, setFormData] = useState<Partial<SftpSite>>({
    name: '',
    hostname: '',
    port: 22,
    username: '',
    default_path: '/home',
    group: 'Default',
    notes: '',
  })
  const [password, setPassword] = useState('')
  const [useSshKey, setUseSshKey] = useState(false)
  const [sshKeyPath, setSshKeyPath] = useState('')

  // Load sites on mount
  useEffect(() => {
    loadSites()
  }, [])

  const loadSites = async () => {
    try {
      const result = await invoke<{ sites: SftpSite[]; groups: string[] }>('sftp_list_sites')
      setSites(result.sites)
      setGroups(result.groups)
    } catch (err) {
      console.error('Failed to load sites:', err)
    }
  }

  const handleSaveSite = async () => {
    try {
      const siteData = {
        ...formData,
        id: editingSite?.id || crypto.randomUUID(),
      }

      if (editingSite) {
        await invoke('sftp_update_site', { site: siteData, password: password || undefined })
      } else {
        await invoke('sftp_add_site', { 
          site: siteData, 
          password: useSshKey ? undefined : password,
          sshKeyPath: useSshKey ? sshKeyPath : undefined 
        })
      }

      setShowAddDialog(false)
      setEditingSite(null)
      resetForm()
      loadSites()
    } catch (err) {
      console.error('Failed to save site:', err)
      alert('Failed to save site: ' + err)
    }
  }

  const handleDeleteSite = async (siteId: string) => {
    if (!confirm('Are you sure you want to delete this site?')) return
    
    try {
      await invoke('sftp_remove_site', { siteId })
      loadSites()
    } catch (err) {
      console.error('Failed to delete site:', err)
    }
  }

  const handleConnect = async (site: SftpSite) => {
    setConnectionStatus(prev => ({ ...prev, [site.id]: 'connecting' }))
    
    try {
      await invoke('sftp_connect_site', { siteId: site.id })
      setConnectionStatus(prev => ({ ...prev, [site.id]: 'connected' }))
      onConnect(site)
      
      // Update last connected
      loadSites()
    } catch (err) {
      console.error('Failed to connect:', err)
      setConnectionStatus(prev => ({ ...prev, [site.id]: 'error' }))
    }
  }

  const handleTestConnection = async (site: SftpSite) => {
    setConnectionStatus(prev => ({ ...prev, [site.id]: 'testing' }))
    
    try {
      const result = await invoke<{ success: boolean }>('sftp_test_connection', { site })
      setConnectionStatus(prev => ({ 
        ...prev, 
        [site.id]: result.success ? 'test_success' : 'test_failed' 
      }))
    } catch (err) {
      setConnectionStatus(prev => ({ ...prev, [site.id]: 'test_failed' }))
    }
  }

  const openEditDialog = (site: SftpSite) => {
    setEditingSite(site)
    setFormData(site)
    setPassword('')
    setShowAddDialog(true)
  }

  const resetForm = () => {
    setFormData({
      name: '',
      hostname: '',
      port: 22,
      username: '',
      default_path: '/home',
      group: 'Default',
      notes: '',
    })
    setPassword('')
    setUseSshKey(false)
    setSshKeyPath('')
  }

  const filteredSites = selectedGroup === 'All' 
    ? sites 
    : sites.filter(s => s.group === selectedGroup)

  const groupedSites = filteredSites.reduce((acc, site) => {
    const group = site.group || 'Default'
    if (!acc[group]) acc[group] = []
    acc[group].push(site)
    return acc
  }, {} as Record<string, SftpSite[]>)

  return (
    <div className="sftp-site-manager">
      <div className="sftp-header">
        <h3>🌐 SFTP Sites</h3>
        <button className="btn-add" onClick={() => { resetForm(); setShowAddDialog(true); }}>
          + Add Site
        </button>
      </div>

      {/* Group Filter */}
      <div className="sftp-groups">
        <button 
          className={`group-btn ${selectedGroup === 'All' ? 'active' : ''}`}
          onClick={() => setSelectedGroup('All')}
        >
          All ({sites.length})
        </button>
        {groups.map(group => (
          <button 
            key={group}
            className={`group-btn ${selectedGroup === group ? 'active' : ''}`}
            onClick={() => setSelectedGroup(group)}
          >
            {group} ({sites.filter(s => s.group === group).length})
          </button>
        ))}
      </div>

      {/* Sites List */}
      <div className="sftp-sites-list">
        {Object.entries(groupedSites).map(([group, groupSites]) => (
          <div key={group} className="site-group">
            <div className="group-header">{group}</div>
            {groupSites.map(site => (
              <div key={site.id} className="site-item">
                <div className="site-info">
                  <div className="site-name">{site.name}</div>
                  <div className="site-details">
                    {site.username}@{site.hostname}:{site.port}
                    {site.last_connected && (
                      <span className="last-connected">
                        • Last: {new Date(site.last_connected).toLocaleDateString()}
                      </span>
                    )}
                  </div>
                  {site.notes && <div className="site-notes">{site.notes}</div>}
                </div>
                
                <div className="site-actions">
                  {connectionStatus[site.id] === 'connecting' && (
                    <span className="status connecting">Connecting...</span>
                  )}
                  {connectionStatus[site.id] === 'connected' && (
                    <span className="status connected">✓ Connected</span>
                  )}
                  {connectionStatus[site.id] === 'error' && (
                    <span className="status error">✗ Failed</span>
                  )}
                  
                  <button 
                    className="btn-test"
                    onClick={() => handleTestConnection(site)}
                    disabled={connectionStatus[site.id] === 'testing'}
                  >
                    {connectionStatus[site.id] === 'testing' ? 'Testing...' : 'Test'}
                  </button>
                  
                  <button 
                    className="btn-connect"
                    onClick={() => handleConnect(site)}
                  >
                    Connect
                  </button>
                  
                  <button 
                    className="btn-edit"
                    onClick={() => openEditDialog(site)}
                  >
                    Edit
                  </button>
                  
                  <button 
                    className="btn-delete"
                    onClick={() => handleDeleteSite(site.id)}
                  >
                    🗑️
                  </button>
                </div>
              </div>
            ))}
          </div>
        ))}
        
        {filteredSites.length === 0 && (
          <div className="sftp-empty">
            <p>No SFTP sites configured</p>
            <p className="hint">Click "Add Site" to create one</p>
          </div>
        )}
      </div>

      {/* Add/Edit Dialog */}
      {showAddDialog && (
        <div className="sftp-dialog-overlay">
          <div className="sftp-dialog">
            <div className="sftp-dialog-header">
              <h3>{editingSite ? 'Edit Site' : 'Add New Site'}</h3>
              <button className="btn-close" onClick={() => setShowAddDialog(false)}>×</button>
            </div>
            
            <div className="sftp-dialog-body">
              <div className="form-row">
                <label>Site Name *</label>
                <input 
                  type="text" 
                  value={formData.name} 
                  onChange={e => setFormData({...formData, name: e.target.value})}
                  placeholder="e.g., Production Server"
                />
              </div>

              <div className="form-row">
                <label>Hostname *</label>
                <input 
                  type="text" 
                  value={formData.hostname} 
                  onChange={e => setFormData({...formData, hostname: e.target.value})}
                  placeholder="e.g., example.com or 192.168.1.1"
                />
              </div>

              <div className="form-row form-row-half">
                <div>
                  <label>Port</label>
                  <input 
                    type="number" 
                    value={formData.port} 
                    onChange={e => setFormData({...formData, port: parseInt(e.target.value) || 22})}
                  />
                </div>
                <div>
                  <label>Group</label>
                  <select 
                    value={formData.group}
                    onChange={e => setFormData({...formData, group: e.target.value})}
                  >
                    {groups.map(g => <option key={g} value={g}>{g}</option>)}
                  </select>
                </div>
              </div>

              <div className="form-row">
                <label>Username *</label>
                <input 
                  type="text" 
                  value={formData.username} 
                  onChange={e => setFormData({...formData, username: e.target.value})}
                  placeholder="e.g., root or ubuntu"
                />
              </div>

              <div className="form-row">
                <label className="checkbox-label">
                  <input 
                    type="checkbox" 
                    checked={useSshKey}
                    onChange={e => setUseSshKey(e.target.checked)}
                  />
                  Use SSH Key (instead of password)
                </label>
              </div>

              {useSshKey ? (
                <div className="form-row">
                  <label>SSH Key Path</label>
                  <input 
                    type="text" 
                    value={sshKeyPath} 
                    onChange={e => setSshKeyPath(e.target.value)}
                    placeholder="~/.ssh/id_rsa"
                  />
                </div>
              ) : (
                <div className="form-row">
                  <label>Password {editingSite ? '(leave blank to keep unchanged)' : ''}</label>
                  <input 
                    type="password" 
                    value={password} 
                    onChange={e => setPassword(e.target.value)}
                    placeholder={editingSite ? '••••••••' : ''}
                  />
                </div>
              )}

              <div className="form-row">
                <label>Default Path</label>
                <input 
                  type="text" 
                  value={formData.default_path} 
                  onChange={e => setFormData({...formData, default_path: e.target.value})}
                  placeholder="/home"
                />
              </div>

              <div className="form-row">
                <label>Notes</label>
                <textarea 
                  value={formData.notes} 
                  onChange={e => setFormData({...formData, notes: e.target.value})}
                  placeholder="Optional notes about this server..."
                  rows={3}
                />
              </div>
            </div>

            <div className="sftp-dialog-footer">
              <button className="btn-secondary" onClick={() => setShowAddDialog(false)}>
                Cancel
              </button>
              <button 
                className="btn-primary" 
                onClick={handleSaveSite}
                disabled={!formData.name || !formData.hostname || !formData.username}
              >
                {editingSite ? 'Save Changes' : 'Add Site'}
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  )
}

export default SftpSiteManager