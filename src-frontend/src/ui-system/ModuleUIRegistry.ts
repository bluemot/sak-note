/**
 * Module UI Registry
 * 
 * JSON-driven UI system - modules define their UI needs via configuration
 * Frontend renders components based on module definitions
 * 
 * Usage: Modules register UI components through JSON schema
 * {
 *   "module": "sftp",
 *   "components": [
 *     {
 *       "type": "sidebar_panel",
 *       "slot": "sidebar.tabs",
 *       "id": "sftp-sites",
 *       "title": "🌐 SFTP Sites",
 *       "icon": "globe",
 *       "component": "SftpSitePanel"
 *     },
 *     {
 *       "type": "toolbar_button",
 *       "slot": "toolbar.main",
 *       "id": "sftp-connect",
 *       "title": "Connect",
 *       "icon": "plug",
 *       "action": "sftp:connect_dialog"
 *     }
 *   ]
 * }
 */

import { ComponentType, LazyExoticComponent, lazy } from 'react'

// UI Slot definitions - where components can be placed
export type UISlot = 
  | 'toolbar.main'        // Main toolbar
  | 'toolbar.search'        // Search toolbar
  | 'sidebar.tabs'         // Sidebar tab panels
  | 'sidebar.tools'        // Sidebar tool buttons
  | 'editor.contextMenu'   // Editor right-click menu
  | 'editor.statusBar'     // Editor status bar
  | 'menu.file'           // File menu
  | 'menu.edit'           // Edit menu
  | 'menu.view'           // View menu
  | 'menu.tools'          // Tools menu
  | 'dialog.newFile'      // New file dialog
  | 'statusBar.main'      // Main status bar
  | 'tabBar.contextMenu'  // Tab right-click menu

// Component types
export type UIComponentType =
  | 'toolbar_button'
  | 'toolbar_toggle'
  | 'toolbar_dropdown'
  | 'sidebar_panel'
  | 'sidebar_button'
  | 'menu_item'
  | 'menu_submenu'
  | 'editor_action'
  | 'status_item'
  | 'dialog'
  | 'tab_action'

// Base component definition
export interface UIComponentDefinition {
  id: string
  type: UIComponentType
  slot: UISlot
  module: string
  
  // Display
  title: string
  tooltip?: string
  icon?: string
  
  // Component reference
  component?: string  // React component name
  componentPath?: string // Lazy load path
  
  // Actions
  action?: string     // Action ID like "sftp:connect"
  command?: string    // Tauri command to invoke
  shortcut?: string   // Keyboard shortcut
  
  // Conditions
  when?: string      // Condition expression
  visible?: boolean
  enabled?: boolean
  
  // Styling
  className?: string
  style?: Record<string, string>
  
  // Position
  order?: number     // Position in slot (lower = first)
  group?: string      // Grouping
  
  // Data
  props?: Record<string, any>
}

// Module UI registration
export interface ModuleUIRegistration {
  module: string
  version: string
  components: UIComponentDefinition[]
  menus?: MenuDefinition[]
  shortcuts?: ShortcutDefinition[]
}

// Menu structure
export interface MenuDefinition {
  id: string
  slot: UISlot
  title: string
  items: MenuItemDefinition[]
}

export interface MenuItemDefinition {
  id: string
  type: 'item' | 'separator' | 'submenu'
  title?: string
  shortcut?: string
  action?: string
  command?: string
  icon?: string
  enabled?: boolean
  when?: string
  submenu?: MenuItemDefinition[]
}

// Keyboard shortcuts
export interface ShortcutDefinition {
  key: string
  modifiers?: ('Ctrl' | 'Shift' | 'Alt' | 'Cmd')[]
  action: string
  command?: string
  when?: string
}

// Component registry
class ModuleUIRegistry {
  private components: Map<string, UIComponentDefinition> = new Map()
  private modules: Map<string, ModuleUIRegistration> = new Map()
  private componentCache: Map<string, LazyExoticComponent<ComponentType<any>>> = new Map()
  
  // Register a module's UI
  registerModule(registration: ModuleUIRegistration): void {
    this.modules.set(registration.module, registration)
    
    registration.components.forEach(component => {
      this.components.set(`${registration.module}:${component.id}`, component)
    })
    
    console.log(`[UI Registry] Registered module: ${registration.module} with ${registration.components.length} components`)
  }
  
  // Unregister module
  unregisterModule(moduleId: string): void {
    const registration = this.modules.get(moduleId)
    if (registration) {
      registration.components.forEach(component => {
        this.components.delete(`${moduleId}:${component.id}`)
      })
      this.modules.delete(moduleId)
    }
  }
  
  // Get components for a specific slot
  getComponentsForSlot(slot: UISlot): UIComponentDefinition[] {
    const results: UIComponentDefinition[] = []
    
    this.components.forEach(component => {
      if (component.slot === slot && component.visible !== false) {
        results.push(component)
      }
    })
    
    // Sort by order
    return results.sort((a, b) => (a.order || 999) - (b.order || 999))
  }
  
  // Get component by ID
  getComponent(moduleId: string, componentId: string): UIComponentDefinition | undefined {
    return this.components.get(`${moduleId}:${componentId}`)
  }
  
  // Lazy load React component
  loadComponent(componentPath: string): LazyExoticComponent<ComponentType<any>> {
    if (!this.componentCache.has(componentPath)) {
      const component = lazy(() => import(/* @vite-ignore */ componentPath))
      this.componentCache.set(componentPath, component)
    }
    return this.componentCache.get(componentPath)!
  }
  
  // Execute action
  async executeAction(actionId: string, params?: any): Promise<any> {
    const [module, action] = actionId.split(':')
    
    // Call Tauri command
    if (window.__TAURI__) {
      const { invoke } = await import('@tauri-apps/api/core')
      return invoke('ui_execute_action', {
        module,
        action,
        params
      })
    }
    
    // Fallback for web
    console.log(`[Action] ${actionId}`, params)
    return Promise.resolve({ success: true })
  }
  
  // Get all registered modules
  getModules(): ModuleUIRegistration[] {
    return Array.from(this.modules.values())
  }
  
  // Get shortcuts
  getShortcuts(): ShortcutDefinition[] {
    const shortcuts: ShortcutDefinition[] = []
    this.modules.forEach(module => {
      if (module.shortcuts) {
        shortcuts.push(...module.shortcuts)
      }
    })
    return shortcuts
  }
  
  // Register from JSON (for dynamic loading)
  async registerFromJSON(json: string | object): Promise<void> {
    const registration = typeof json === 'string' ? JSON.parse(json) : json
    this.registerModule(registration as ModuleUIRegistration)
  }
}

// Global registry instance
export const uiRegistry = new ModuleUIRegistry()

// React hooks for components
export function useSlotComponents(slot: UISlot): UIComponentDefinition[] {
  const [components, setComponents] = useState<UIComponentDefinition[]>([])
  
  useEffect(() => {
    setComponents(uiRegistry.getComponentsForSlot(slot))
    
    // Listen for registry changes
    const handleRegistryChange = () => {
      setComponents(uiRegistry.getComponentsForSlot(slot))
    }
    
    window.addEventListener('ui-registry-change', handleRegistryChange)
    return () => window.removeEventListener('ui-registry-change', handleRegistryChange)
  }, [slot])
  
  return components
}

// Hook for executing actions
export function useUIAction() {
  return useCallback(async (actionId: string, params?: any) => {
    return uiRegistry.executeAction(actionId, params)
  }, [])
}

// Built-in module registrations
export function registerBuiltInModules(): void {
  // File Module UI
  uiRegistry.registerModule({
    module: 'file',
    version: '1.0',
    components: [
      {
        id: 'open',
        type: 'toolbar_button',
        slot: 'toolbar.main',
        module: 'file',
        title: 'Open',
        icon: 'folder-open',
        action: 'file:open_dialog',
        shortcut: 'Ctrl+O',
        order: 10
      },
      {
        id: 'save',
        type: 'toolbar_button',
        slot: 'toolbar.main',
        module: 'file',
        title: 'Save',
        icon: 'save',
        action: 'file:save',
        shortcut: 'Ctrl+S',
        order: 20
      },
      {
        id: 'recent-files',
        type: 'menu_item',
        slot: 'menu.file',
        module: 'file',
        title: 'Open Recent',
        action: 'file:recent_menu'
      }
    ],
    menus: [
      {
        id: 'file-menu',
        slot: 'menu.file',
        title: 'File',
        items: [
          { id: 'new', type: 'item', title: 'New File', shortcut: 'Ctrl+N', action: 'file:new' },
          { id: 'sep1', type: 'separator' },
          { id: 'open', type: 'item', title: 'Open...', shortcut: 'Ctrl+O', action: 'file:open' },
          { id: 'open-recent', type: 'item', title: 'Open Recent', action: 'file:recent' },
          { id: 'sep2', type: 'separator' },
          { id: 'save', type: 'item', title: 'Save', shortcut: 'Ctrl+S', action: 'file:save' },
          { id: 'save-as', type: 'item', title: 'Save As...', shortcut: 'Ctrl+Shift+S', action: 'file:save_as' },
        ]
      }
    ],
    shortcuts: [
      { key: 'o', modifiers: ['Ctrl'], action: 'file:open' },
      { key: 's', modifiers: ['Ctrl'], action: 'file:save' },
      { key: 's', modifiers: ['Ctrl', 'Shift'], action: 'file:save_as' },
      { key: 'n', modifiers: ['Ctrl'], action: 'file:new' },
    ]
  })
  
  // Edit Module UI
  uiRegistry.registerModule({
    module: 'edit',
    version: '1.0',
    components: [
      {
        id: 'undo',
        type: 'toolbar_button',
        slot: 'toolbar.main',
        module: 'edit',
        title: 'Undo',
        icon: 'rotate-left',
        action: 'edit:undo',
        shortcut: 'Ctrl+Z',
        order: 30
      },
      {
        id: 'redo',
        type: 'toolbar_button',
        slot: 'toolbar.main',
        module: 'edit',
        title: 'Redo',
        icon: 'rotate-right',
        action: 'edit:redo',
        shortcut: 'Ctrl+Y',
        order: 31
      },
      {
        id: 'find',
        type: 'toolbar_button',
        slot: 'toolbar.search',
        module: 'edit',
        title: 'Find',
        icon: 'search',
        action: 'edit:find',
        shortcut: 'Ctrl+F',
        order: 10
      },
      {
        id: 'replace',
        type: 'toolbar_button',
        slot: 'toolbar.search',
        module: 'edit',
        title: 'Replace',
        icon: 'replace',
        action: 'edit:replace',
        shortcut: 'Ctrl+H',
        order: 20
      }
    ],
    shortcuts: [
      { key: 'z', modifiers: ['Ctrl'], action: 'edit:undo' },
      { key: 'y', modifiers: ['Ctrl'], action: 'edit:redo' },
      { key: 'f', modifiers: ['Ctrl'], action: 'edit:find' },
      { key: 'h', modifiers: ['Ctrl'], action: 'edit:replace' },
      { key: 'g', modifiers: ['Ctrl'], action: 'edit:goto_line' },
    ]
  })
  
  // Bookmark Module UI
  uiRegistry.registerModule({
    module: 'bookmark',
    version: '1.0',
    components: [
      {
        id: 'toggle',
        type: 'toolbar_button',
        slot: 'toolbar.main',
        module: 'bookmark',
        title: 'Toggle Bookmark',
        icon: 'bookmark',
        action: 'bookmark:toggle',
        shortcut: 'Ctrl+F2',
        order: 50
      },
      {
        id: 'next',
        type: 'toolbar_button',
        slot: 'toolbar.main',
        module: 'bookmark',
        title: 'Next Bookmark',
        icon: 'arrow-down',
        action: 'bookmark:next',
        shortcut: 'F2',
        order: 51
      },
      {
        id: 'prev',
        type: 'toolbar_button',
        slot: 'toolbar.main',
        module: 'bookmark',
        title: 'Prev Bookmark',
        icon: 'arrow-up',
        action: 'bookmark:prev',
        shortcut: 'Shift+F2',
        order: 52
      },
      {
        id: 'panel',
        type: 'sidebar_panel',
        slot: 'sidebar.tabs',
        module: 'bookmark',
        title: '📑 Bookmarks',
        icon: 'bookmark',
        component: 'BookmarkPanel',
        order: 40
      }
    ],
    shortcuts: [
      { key: 'F2', modifiers: [], action: 'bookmark:next' },
      { key: 'F2', modifiers: ['Shift'], action: 'bookmark:prev' },
      { key: 'F2', modifiers: ['Ctrl'], action: 'bookmark:toggle' },
    ]
  })
  
  // SFTP Module UI
  uiRegistry.registerModule({
    module: 'sftp',
    version: '1.0',
    components: [
      {
        id: 'connect',
        type: 'toolbar_button',
        slot: 'toolbar.main',
        module: 'sftp',
        title: 'SFTP',
        icon: 'globe',
        action: 'sftp:connect_dialog',
        order: 100
      },
      {
        id: 'sites-panel',
        type: 'sidebar_panel',
        slot: 'sidebar.tabs',
        module: 'sftp',
        title: '🌐 SFTP Sites',
        icon: 'globe',
        component: 'SftpSitePanel',
        order: 50
      }
    ]
  })
  
  // Semantic Analysis Module UI
  uiRegistry.registerModule({
    module: 'semantic',
    version: '1.0',
    components: [
      {
        id: 'analyze',
        type: 'toolbar_button',
        slot: 'toolbar.main',
        module: 'semantic',
        title: 'Analyze',
        icon: 'brain',
        action: 'semantic:intelligent_mark',
        order: 60
      },
      {
        id: 'panel',
        type: 'sidebar_panel',
        slot: 'sidebar.tabs',
        module: 'semantic',
        title: '🔍 Semantic',
        icon: 'brain',
        component: 'SemanticPanel',
        order: 30
      }
    ]
  })
}

// Export for use
export default uiRegistry