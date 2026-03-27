import { invoke } from '@tauri-apps/api/core';

/**
 * Plugin Service
 * 
 * Provides a TypeScript API for interacting with the plugin system.
 * All commands include comprehensive logging for debugging.
 */

export interface PluginManifest {
  id: string;
  name: string;
  version: string;
  description: string;
  author: string;
  entry_point: string;
  capabilities: PluginCapability[];
  permissions: PluginPermission[];
  ui_components?: PluginUiComponent[];
}

export interface PluginCapability {
  id: string;
  name: string;
  description: string;
  input_schema?: string;
  output_schema?: string;
}

export type PluginPermission = 'FileRead' | 'FileWrite' | 'Network' | 'Environment' | 'Command';

export interface PluginUiComponent {
  id: string;
  name: string;
  position: string;
  config?: Record<string, any>;
}

export interface PluginMetadata {
  id: string;
  name: string;
  version: string;
  description: string;
  author: string;
}

export interface PluginLoadStatus {
  plugin_id: string;
  success: boolean;
  message: string;
}

export interface PluginExecutionResult {
  success: boolean;
  output?: string;
  error?: string;
}

export interface PluginCapabilityInfo extends PluginCapability {
  plugin_id: string;
}

export type EditorEventType = 
  | 'FileOpened' 
  | 'FileClosed' 
  | 'FileSaved' 
  | 'ContentChanged' 
  | 'SelectionChanged' 
  | 'Startup' 
  | 'Shutdown'
  | string; // Custom events

/**
 * Initialize the plugin system
 */
export async function initPluginSystem(): Promise<boolean> {
  console.log('[PluginService] Initializing plugin system...');
  try {
    const result = await invoke<boolean>('plugin_init');
    console.log('[PluginService] Plugin system initialized:', result);
    return result;
  } catch (error) {
    console.error('[PluginService] Failed to initialize plugin system:', error);
    throw error;
  }
}

/**
 * Discover available plugins in the plugin directory
 */
export async function discoverPlugins(): Promise<PluginManifest[]> {
  console.log('[PluginService] Discovering plugins...');
  try {
    const plugins = await invoke<PluginManifest[]>('plugin_discover');
    console.log(`[PluginService] Discovered ${plugins.length} plugins:`, plugins.map(p => p.id));
    return plugins;
  } catch (error) {
    console.error('[PluginService] Failed to discover plugins:', error);
    throw error;
  }
}

/**
 * Load all discovered plugins
 */
export async function loadAllPlugins(): Promise<PluginLoadStatus[]> {
  console.log('[PluginService] Loading all plugins...');
  try {
    const results = await invoke<PluginLoadStatus[]>('plugin_load_all');
    const successCount = results.filter(r => r.success).length;
    console.log(`[PluginService] Loaded ${successCount}/${results.length} plugins`);
    
    results.forEach(result => {
      if (result.success) {
        console.log(`[PluginService]   [OK] ${result.plugin_id}: ${result.message}`);
      } else {
        console.warn(`[PluginService]   [FAIL] ${result.plugin_id}: ${result.message}`);
      }
    });
    
    return results;
  } catch (error) {
    console.error('[PluginService] Failed to load all plugins:', error);
    throw error;
  }
}

/**
 * Load a specific plugin by ID
 */
export async function loadPlugin(pluginId: string): Promise<PluginLoadStatus> {
  console.log(`[PluginService] Loading plugin: ${pluginId}`);
  try {
    const result = await invoke<PluginLoadStatus>('plugin_load', { pluginId });
    if (result.success) {
      console.log(`[PluginService] Plugin ${pluginId} loaded successfully:`, result.message);
    } else {
      console.warn(`[PluginService] Plugin ${pluginId} failed to load:`, result.message);
    }
    return result;
  } catch (error) {
    console.error(`[PluginService] Failed to load plugin ${pluginId}:`, error);
    throw error;
  }
}

/**
 * Unload a plugin by ID
 */
export async function unloadPlugin(pluginId: string): Promise<void> {
  console.log(`[PluginService] Unloading plugin: ${pluginId}`);
  try {
    await invoke('plugin_unload', { pluginId });
    console.log(`[PluginService] Plugin ${pluginId} unloaded successfully`);
  } catch (error) {
    console.error(`[PluginService] Failed to unload plugin ${pluginId}:`, error);
    throw error;
  }
}

/**
 * Get list of all loaded plugins
 */
export async function getLoadedPlugins(): Promise<PluginMetadata[]> {
  console.log('[PluginService] Getting loaded plugins...');
  try {
    const plugins = await invoke<PluginMetadata[]>('plugin_list_loaded');
    console.log(`[PluginService] Found ${plugins.length} loaded plugins:`, plugins.map(p => `${p.name} (${p.id})`));
    return plugins;
  } catch (error) {
    console.error('[PluginService] Failed to get loaded plugins:', error);
    throw error;
  }
}

/**
 * Get detailed info about a specific plugin
 */
export async function getPluginInfo(pluginId: string): Promise<PluginMetadata> {
  console.log(`[PluginService] Getting info for plugin: ${pluginId}`);
  try {
    const info = await invoke<PluginMetadata>('plugin_get_info', { pluginId });
    console.log(`[PluginService] Plugin ${pluginId} info:`, info);
    return info;
  } catch (error) {
    console.error(`[PluginService] Failed to get plugin info for ${pluginId}:`, error);
    throw error;
  }
}

/**
 * Execute a plugin capability
 */
export async function executeCapability(
  pluginId: string, 
  capabilityId: string, 
  input?: string
): Promise<PluginExecutionResult> {
  console.log(`[PluginService] Executing capability: ${pluginId}.${capabilityId}`);
  console.log(`[PluginService] Input:`, input);
  
  try {
    const result = await invoke<PluginExecutionResult>('plugin_execute', {
      pluginId,
      capabilityId,
      input: input || '{}'
    });
    
    if (result.success) {
      console.log(`[PluginService] Capability ${capabilityId} executed successfully:`, result.output);
    } else {
      console.error(`[PluginService] Capability ${capabilityId} failed:`, result.error);
    }
    
    return result;
  } catch (error) {
    console.error(`[PluginService] Failed to execute capability ${capabilityId}:`, error);
    throw error;
  }
}

/**
 * Enable or disable a plugin
 */
export async function setPluginEnabled(pluginId: string, enabled: boolean): Promise<void> {
  console.log(`[PluginService] Setting plugin ${pluginId} enabled: ${enabled}`);
  try {
    await invoke('plugin_set_enabled', { pluginId, enabled });
    console.log(`[PluginService] Plugin ${pluginId} ${enabled ? 'enabled' : 'disabled'}`);
  } catch (error) {
    console.error(`[PluginService] Failed to set plugin ${pluginId} enabled status:`, error);
    throw error;
  }
}

/**
 * Get all capabilities from all loaded plugins
 */
export async function getAllCapabilities(): Promise<PluginCapabilityInfo[]> {
  console.log('[PluginService] Getting all capabilities...');
  try {
    const capabilities = await invoke<PluginCapabilityInfo[]>('plugin_get_capabilities');
    console.log(`[PluginService] Found ${capabilities.length} capabilities from all plugins`);
    capabilities.forEach(cap => {
      console.log(`[PluginService]   - ${cap.id} from ${cap.plugin_id}`);
    });
    return capabilities;
  } catch (error) {
    console.error('[PluginService] Failed to get capabilities:', error);
    throw error;
  }
}

/**
 * Broadcast an event to all plugins
 */
export async function broadcastEvent(
  eventType: EditorEventType, 
  eventData: Record<string, any>
): Promise<void> {
  console.log(`[PluginService] Broadcasting event: ${eventType}`, eventData);
  try {
    await invoke('plugin_broadcast_event', { eventType, eventData });
    console.log(`[PluginService] Event ${eventType} broadcasted successfully`);
  } catch (error) {
    console.error(`[PluginService] Failed to broadcast event ${eventType}:`, error);
    throw error;
  }
}

/**
 * Get the plugin directory path
 */
export async function getPluginDirectory(): Promise<string> {
  console.log('[PluginService] Getting plugin directory...');
  try {
    const directory = await invoke<string>('plugin_get_directory');
    console.log(`[PluginService] Plugin directory: ${directory}`);
    return directory;
  } catch (error) {
    console.error('[PluginService] Failed to get plugin directory:', error);
    throw error;
  }
}

/**
 * Initialize and load all plugins (convenience function)
 */
export async function initializeAndLoadPlugins(): Promise<{
  initialized: boolean;
  discovered: PluginManifest[];
  loaded: PluginLoadStatus[];
}> {
  console.log('[PluginService] Starting full plugin initialization...');
  
  try {
    // Initialize plugin system
    const initialized = await initPluginSystem();
    
    // Discover plugins
    const discovered = await discoverPlugins();
    
    // Load all plugins
    const loaded = await loadAllPlugins();
    
    console.log('[PluginService] Plugin initialization complete:', {
      initialized,
      discovered: discovered.length,
      loaded: loaded.filter(l => l.success).length
    });
    
    return { initialized, discovered, loaded };
  } catch (error) {
    console.error('[PluginService] Plugin initialization failed:', error);
    throw error;
  }
}
