/**
 * Plugin Loader
 *
 * Dynamically loads WASM plugins from the plugins/ directory.
 * Supports plugin.json manifest format for registration.
 */

import { uiRegistry } from './ModuleUIRegistry';

// Plugin Manifest Interface
export interface PluginManifest {
  name: string;
  version: string;
  description?: string;
  author?: string;
  entry?: string; // Entry point (e.g., "plugin.wasm")
  ui?: {
    components?: Array<{
      type: string;
      slot: string;
      id: string;
      title: string;
      icon?: string;
      action?: string;
    }>;
    menus?: Array<{
      id: string;
      slot: string;
      title: string;
      items: Array<{
        id: string;
        type: 'item' | 'separator' | 'submenu';
        title?: string;
        action?: string;
      }>;
    }>;
    shortcuts?: Array<{
      key: string;
      modifiers?: string[];
      action: string;
    }>;
  };
  permissions?: string[];
  hooks?: {
    onStartup?: string;
    onShutdown?: string;
    onFileOpen?: string;
    onFileSave?: string;
    onFileClose?: string;
    onSelectionChange?: string;
    onCursorMove?: string;
  };
}

// Loaded Plugin State
export interface LoadedPlugin {
  id: string;
  manifest: PluginManifest;
  module: WebAssembly.Module | null;
  instance: WebAssembly.Instance | null;
  exports: Record<string, any>;
  isActive: boolean;
}

// Plugin Registry
const loadedPlugins: Map<string, LoadedPlugin> = new Map();

/**
 * Load a plugin from a given path
 * @param pluginPath Path to the plugin directory
 * @returns Promise resolving to the loaded plugin
 */
export async function loadPlugin(pluginPath: string): Promise<LoadedPlugin> {
  console.log(`[PluginLoader] Loading plugin from: ${pluginPath}`);

  try {
    // Read plugin.json manifest
    const manifestUrl = `${pluginPath}/plugin.json`;
    const manifestResponse = await fetch(manifestUrl);

    if (!manifestResponse.ok) {
      throw new Error(`Failed to load plugin manifest: ${manifestResponse.statusText}`);
    }

    const manifest: PluginManifest = await manifestResponse.json();
    const pluginId = `${manifest.name}@${manifest.version}`;

    console.log(`[PluginLoader] Loaded manifest for ${manifest.name} v${manifest.version}`);

    // Check if already loaded
    if (loadedPlugins.has(pluginId)) {
      console.log(`[PluginLoader] Plugin ${pluginId} already loaded`);
      return loadedPlugins.get(pluginId)!;
    }

    // Initialize plugin state
    const loadedPlugin: LoadedPlugin = {
      id: pluginId,
      manifest,
      module: null,
      instance: null,
      exports: {},
      isActive: true,
    };

    // Load WASM if entry point specified
    if (manifest.entry) {
      const wasmUrl = `${pluginPath}/${manifest.entry}`;
      try {
        const wasmResponse = await fetch(wasmUrl);

        if (wasmResponse.ok) {
          const wasmBuffer = await wasmResponse.arrayBuffer();
          loadedPlugin.module = await WebAssembly.compile(wasmBuffer);

          // Create import object with host functions
          const importObject = createImportObject(loadedPlugin);

          // Instantiate WASM module
          loadedPlugin.instance = await WebAssembly.instantiate(
            loadedPlugin.module,
            importObject
          );

          // Get exported functions
          loadedPlugin.exports = loadedPlugin.instance.exports as Record<string, any>;

          console.log(`[PluginLoader] WASM module loaded for ${pluginId}`);

          // Call initialization if available
          if (loadedPlugin.exports.init) {
            loadedPlugin.exports.init();
          }
        } else {
          console.warn(`[PluginLoader] WASM file not found: ${wasmUrl}`);
        }
      } catch (wasmError) {
        console.warn(`[PluginLoader] Failed to load WASM:`, wasmError);
      }
    }

    // Register UI components from manifest
    if (manifest.ui) {
      registerPluginUI(manifest);
    }

    // Store in registry
    loadedPlugins.set(pluginId, loadedPlugin);

    console.log(`[PluginLoader] Plugin ${pluginId} loaded successfully`);

    return loadedPlugin;
  } catch (error) {
    console.error(`[PluginLoader] Failed to load plugin from ${pluginPath}:`, error);
    throw error;
  }
}

/**
 * Create import object for WASM module
 */
function createImportObject(plugin: LoadedPlugin): WebAssembly.Imports {
  return {
    env: {
      // Memory
      memory: new WebAssembly.Memory({ initial: 256, maximum: 512 }),

      // Console output
      log: (ptr: number, len: number) => {
        const memory = plugin.instance?.exports.memory as WebAssembly.Memory;
        if (memory) {
          const bytes = new Uint8Array(memory.buffer, ptr, len);
          const text = new TextDecoder().decode(bytes);
          console.log(`[Plugin:${plugin.id}]`, text);
        }
      },

      // Error output
      error: (ptr: number, len: number) => {
        const memory = plugin.instance?.exports.memory as WebAssembly.Memory;
        if (memory) {
          const bytes = new Uint8Array(memory.buffer, ptr, len);
          const text = new TextDecoder().decode(bytes);
          console.error(`[Plugin:${plugin.id}]`, text);
        }
      },

      // Host API - invoke Tauri command
      invoke: async (commandPtr: number, commandLen: number, argsPtr: number, argsLen: number) => {
        const memory = plugin.instance?.exports.memory as WebAssembly.Memory;
        if (!memory) return;

        const commandBytes = new Uint8Array(memory.buffer, commandPtr, commandLen);
        const command = new TextDecoder().decode(commandBytes);

        const argsBytes = new Uint8Array(memory.buffer, argsPtr, argsLen);
        const args = JSON.parse(new TextDecoder().decode(argsBytes));

        try {
          if (window.__TAURI__) {
            const { invoke } = await import('@tauri-apps/api/core');
            const result = await invoke(command, args);
            return JSON.stringify(result);
          }
        } catch (err) {
          console.error(`[PluginLoader] Invoke error:`, err);
        }
        return null;
      },

      // Host API - emit event
      emit: (eventPtr: number, eventLen: number, dataPtr: number, dataLen: number) => {
        const memory = plugin.instance?.exports.memory as WebAssembly.Memory;
        if (!memory) return;

        const eventBytes = new Uint8Array(memory.buffer, eventPtr, eventLen);
        const eventName = new TextDecoder().decode(eventBytes);

        const dataBytes = new Uint8Array(memory.buffer, dataPtr, dataLen);
        const data = JSON.parse(new TextDecoder().decode(dataBytes));

        // Broadcast to plugin service
        console.log(`[Plugin:${plugin.id}] Event emitted:`, eventName, data);
      },

      // Host API - get editor state
      getState: () => {
        // Return current editor state
        return JSON.stringify({
          hasFileOpen: false,
          currentFile: null,
        });
      },

      // Host API - allocate memory (for returning data to host)
      // eslint-disable-next-line @typescript-eslint/no-unused-vars
      alloc: (_size: number) => {
        // This would be implemented by the plugin's allocator
        // For now, we rely on the plugin's own memory management
        return 0;
      },

      // Host API - free memory
      // eslint-disable-next-line @typescript-eslint/no-unused-vars
      free: (_ptr: number) => {
        // Memory management is plugin's responsibility
      },
    },
  };
}

/**
 * Register UI components from plugin manifest
 */
function registerPluginUI(manifest: PluginManifest): void {
  const { ui, name, version } = manifest;
  if (!ui) return;

  const registration = {
    module: name,
    version,
    components: ui.components?.map((c) => ({
      id: c.id,
      type: c.type as any,
      slot: c.slot as any,
      module: name,
      title: c.title,
      icon: c.icon,
      action: c.action,
      visible: true,
      enabled: true,
    })) || [],
    menus: ui.menus?.map((m) => ({
      id: m.id,
      slot: m.slot as any,
      title: m.title,
      items: m.items as any[],
    })) || [],
    shortcuts: ui.shortcuts?.map((s) => ({
      key: s.key,
      modifiers: s.modifiers as any,
      action: s.action,
    })) || [],
  };

  uiRegistry.registerModule(registration);
  console.log(`[PluginLoader] Registered UI for ${name}`);
}

/**
 * Unload a plugin by ID
 * @param pluginId The plugin ID (name@version)
 */
export async function unloadPlugin(pluginId: string): Promise<void> {
  const plugin = loadedPlugins.get(pluginId);
  if (!plugin) {
    console.warn(`[PluginLoader] Plugin ${pluginId} not found`);
    return;
  }

  // Call shutdown hook if available
  if (plugin.exports.shutdown) {
    try {
      plugin.exports.shutdown();
    } catch (err) {
      console.error(`[PluginLoader] Shutdown error for ${pluginId}:`, err);
    }
  }

  // Unregister UI
  uiRegistry.unregisterModule(plugin.manifest.name);

  // Remove from registry
  loadedPlugins.delete(pluginId);

  console.log(`[PluginLoader] Plugin ${pluginId} unloaded`);
}

/**
 * Get all loaded plugins
 */
export function getLoadedPlugins(): LoadedPlugin[] {
  return Array.from(loadedPlugins.values());
}

/**
 * Get a specific plugin by ID
 */
export function getPlugin(pluginId: string): LoadedPlugin | undefined {
  return loadedPlugins.get(pluginId);
}

/**
 * Load all plugins from a directory
 * @param pluginsDir Path to the plugins directory
 */
export async function loadAllPlugins(pluginsDir: string = '/plugins'): Promise<LoadedPlugin[]> {
  console.log(`[PluginLoader] Loading all plugins from ${pluginsDir}`);

  const results: LoadedPlugin[] = [];

  try {
    // Try to fetch plugins directory listing
    const response = await fetch(`${pluginsDir}/index.json`);

    if (response.ok) {
      const pluginList = await response.json();

      for (const pluginName of pluginList.plugins) {
        try {
          const plugin = await loadPlugin(`${pluginsDir}/${pluginName}`);
          results.push(plugin);
        } catch (err) {
          console.error(`[PluginLoader] Failed to load plugin ${pluginName}:`, err);
        }
      }
    } else {
      console.warn(`[PluginLoader] No plugins index found at ${pluginsDir}/index.json`);
    }
  } catch (err) {
    console.warn(`[PluginLoader] Could not load plugins from ${pluginsDir}:`, err);
  }

  return results;
}

/**
 * Execute a plugin action
 * @param pluginId Plugin ID
 * @param action Action name
 * @param params Action parameters
 */
export async function executePluginAction(
  pluginId: string,
  action: string,
  params?: any
): Promise<any> {
  const plugin = loadedPlugins.get(pluginId);
  if (!plugin) {
    throw new Error(`Plugin ${pluginId} not found`);
  }

  if (plugin.exports[action]) {
    try {
      const result = plugin.exports[action](params);
      return result;
    } catch (err) {
      console.error(`[PluginLoader] Action ${action} failed in ${pluginId}:`, err);
      throw err;
    }
  }

  throw new Error(`Action ${action} not found in plugin ${pluginId}`);
}

/**
 * Trigger a hook in all loaded plugins
 * @param hookName Hook name
 * @param data Hook data
 */
export async function triggerHook(hookName: string, data?: any): Promise<void> {
  for (const [pluginId, plugin] of loadedPlugins) {
    const hookMethod = plugin.manifest.hooks?.[hookName as keyof typeof plugin.manifest.hooks];
    if (hookMethod && plugin.exports[hookMethod]) {
      try {
        plugin.exports[hookMethod](data);
      } catch (err) {
        console.error(`[PluginLoader] Hook ${hookName} failed in ${pluginId}:`, err);
      }
    }
  }
}

export default {
  loadPlugin,
  unloadPlugin,
  loadAllPlugins,
  getLoadedPlugins,
  getPlugin,
  executePluginAction,
  triggerHook,
};
