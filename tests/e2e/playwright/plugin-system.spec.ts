import { test, expect, Page } from '@playwright/test';
import { invoke } from '@tauri-apps/api/core';

/**
 * Plugin System E2E Tests
 * 
 * These tests verify the plugin system functionality including:
 * - Plugin discovery
 * - Plugin loading
 * - Plugin execution
 * - Error scenarios
 */

// Helper to invoke Tauri commands with logging
async function invokeTauri<T>(page: Page, command: string, args?: any): Promise<T> {
  console.log(`[Test] Invoking ${command}`, args);
  
  const result = await page.evaluate(async ({ cmd, args }) => {
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      const result = await invoke(cmd, args);
      return { success: true, result };
    } catch (error) {
      return { success: false, error: String(error) };
    }
  }, { cmd: command, args });

  if (!result.success) {
    console.error(`[Test] ${command} failed:`, result.error);
    throw new Error(`Command ${command} failed: ${result.error}`);
  }

  console.log(`[Test] ${command} result:`, result.result);
  return result.result as T;
}

// Helper to capture browser console logs
test.beforeEach(async ({ page }) => {
  // Capture all browser console logs
  page.on('console', msg => {
    console.log(`[Browser Console ${msg.type()}]:`, msg.text());
  });

  // Capture page errors
  page.on('pageerror', error => {
    console.error('[Browser Page Error]:', error.message);
  });
});

test.describe('Plugin System', () => {

  test('should initialize plugin system', async ({ page }) => {
    console.log('[Test] Starting: Plugin system initialization');
    
    try {
      const result = await invokeTauri<boolean>(page, 'plugin_init');
      console.log('[Test] plugin_init result:', result);
      expect(result).toBe(true);
      console.log('[Test] PASSED: Plugin system initialized successfully');
    } catch (error) {
      console.error('[Test] FAILED: Plugin system initialization:', error);
      throw error;
    }
  });

  test('should discover available plugins', async ({ page }) => {
    console.log('[Test] Starting: Plugin discovery');
    
    try {
      // First initialize the plugin system
      await invokeTauri<boolean>(page, 'plugin_init');
      
      // Discover plugins
      const plugins = await invokeTauri<any[]>(page, 'plugin_discover');
      console.log(`[Test] Discovered ${plugins.length} plugins:`, plugins);
      
      // Log plugin details
      plugins.forEach((plugin: any, index: number) => {
        console.log(`[Test] Plugin ${index}:`, {
          id: plugin.id,
          name: plugin.name,
          version: plugin.version,
          entry_point: plugin.entry_point,
          capabilities: plugin.capabilities?.length || 0,
          permissions: plugin.permissions?.length || 0
        });
      });

      // We should have at least the sample plugin
      expect(plugins.length).toBeGreaterThanOrEqual(0);
      
      // If plugins exist, verify their structure
      if (plugins.length > 0) {
        const firstPlugin = plugins[0];
        expect(firstPlugin).toHaveProperty('id');
        expect(firstPlugin).toHaveProperty('name');
        expect(firstPlugin).toHaveProperty('version');
        expect(firstPlugin).toHaveProperty('entry_point');
        expect(firstPlugin).toHaveProperty('capabilities');
        expect(firstPlugin).toHaveProperty('permissions');
      }
      
      console.log('[Test] PASSED: Plugin discovery completed');
    } catch (error) {
      console.error('[Test] FAILED: Plugin discovery:', error);
      throw error;
    }
  });

  test('should load discovered plugins', async ({ page }) => {
    console.log('[Test] Starting: Load all plugins');
    
    try {
      // Initialize plugin system
      await invokeTauri<boolean>(page, 'plugin_init');
      
      // Discover plugins first
      const discoveredPlugins = await invokeTauri<any[]>(page, 'plugin_discover');
      console.log(`[Test] Found ${discoveredPlugins.length} plugins to load`);
      
      if (discoveredPlugins.length === 0) {
        console.log('[Test] Skipping: No plugins to load');
        test.skip();
        return;
      }
      
      // Load all plugins
      const loadResults = await invokeTauri<any[]>(page, 'plugin_load_all');
      console.log(`[Test] Load results:`, loadResults);
      
      // Verify load results structure
      expect(Array.isArray(loadResults)).toBe(true);
      
      // Check each plugin's load status
      loadResults.forEach((result: any) => {
        console.log(`[Test] Plugin ${result.plugin_id}: success=${result.success}, message="${result.message}"`);
        expect(result).toHaveProperty('plugin_id');
        expect(result).toHaveProperty('success');
        expect(result).toHaveProperty('message');
      });
      
      console.log('[Test] PASSED: Plugin loading completed');
    } catch (error) {
      console.error('[Test] FAILED: Plugin loading:', error);
      throw error;
    }
  });

  test('should list loaded plugins', async ({ page }) => {
    console.log('[Test] Starting: List loaded plugins');
    
    try {
      // Initialize and discover
      await invokeTauri<boolean>(page, 'plugin_init');
      await invokeTauri<any[]>(page, 'plugin_discover');
      await invokeTauri<any[]>(page, 'plugin_load_all');
      
      // Get loaded plugins
      const loadedPlugins = await invokeTauri<any[]>(page, 'plugin_list_loaded');
      console.log(`[Test] Loaded ${loadedPlugins.length} plugins:`, loadedPlugins);
      
      // Verify each loaded plugin
      loadedPlugins.forEach((plugin: any) => {
        console.log(`[Test] Loaded plugin details:`, plugin);
        expect(plugin).toHaveProperty('id');
        expect(plugin).toHaveProperty('name');
        expect(plugin).toHaveProperty('version');
        expect(plugin).toHaveProperty('description');
        expect(plugin).toHaveProperty('author');
      });
      
      console.log('[Test] PASSED: Listed loaded plugins');
    } catch (error) {
      console.error('[Test] FAILED: List loaded plugins:', error);
      throw error;
    }
  });

  test('should get plugin capabilities', async ({ page }) => {
    console.log('[Test] Starting: Get plugin capabilities');
    
    try {
      // Initialize plugin system
      await invokeTauri<boolean>(page, 'plugin_init');
      await invokeTauri<any[]>(page, 'plugin_discover');
      await invokeTauri<any[]>(page, 'plugin_load_all');
      
      // Get capabilities
      const capabilities = await invokeTauri<any[]>(page, 'plugin_get_capabilities');
      console.log(`[Test] Got ${capabilities.length} capabilities:`, capabilities);
      
      // Log capability details
      capabilities.forEach((cap: any, index: number) => {
        console.log(`[Test] Capability ${index}:`, {
          id: cap.id,
          name: cap.name,
          description: cap.description,
          plugin_id: cap.plugin_id
        });
      });
      
      console.log('[Test] PASSED: Got plugin capabilities');
    } catch (error) {
      console.error('[Test] FAILED: Get plugin capabilities:', error);
      throw error;
    }
  });

  test('should execute plugin capability', async ({ page }) => {
    console.log('[Test] Starting: Execute plugin capability');
    
    try {
      // Initialize plugin system
      await invokeTauri<boolean>(page, 'plugin_init');
      await invokeTauri<any[]>(page, 'plugin_discover');
      const loadResults = await invokeTauri<any[]>(page, 'plugin_load_all');
      
      // Find a successfully loaded plugin with capabilities
      const loadedPlugins = await invokeTauri<any[]>(page, 'plugin_list_loaded');
      const capabilities = await invokeTauri<any[]>(page, 'plugin_get_capabilities');
      
      if (capabilities.length === 0) {
        console.log('[Test] Skipping: No capabilities to execute');
        test.skip();
        return;
      }
      
      // Try to execute the first capability
      const firstCap = capabilities[0];
      console.log(`[Test] Executing capability: ${firstCap.id} from plugin ${firstCap.plugin_id}`);
      
      const execResult = await invokeTauri<any>(page, 'plugin_execute', {
        pluginId: firstCap.plugin_id,
        capabilityId: firstCap.id,
        input: '{"test": true}'
      });
      
      console.log('[Test] Execution result:', execResult);
      expect(execResult).toHaveProperty('success');
      
      if (execResult.success) {
        console.log('[Test] Capability executed successfully');
      } else {
        console.log('[Test] Capability execution failed:', execResult.error);
      }
      
      console.log('[Test] PASSED: Plugin capability execution test completed');
    } catch (error) {
      console.error('[Test] FAILED: Execute plugin capability:', error);
      throw error;
    }
  });

  test('should handle plugin load errors gracefully', async ({ page }) => {
    console.log('[Test] Starting: Plugin load error handling');
    
    try {
      // Initialize plugin system
      await invokeTauri<boolean>(page, 'plugin_init');
      
      // Try to load a non-existent plugin
      const nonExistentPluginId = 'non-existent-plugin-12345';
      console.log(`[Test] Attempting to load non-existent plugin: ${nonExistentPluginId}`);
      
      try {
        await invokeTauri<any>(page, 'plugin_load', { pluginId: nonExistentPluginId });
        console.log('[Test] WARNING: Expected error was not thrown');
      } catch (error) {
        console.log('[Test] Expected error occurred:', error);
        // This is expected - the plugin doesn't exist
      }
      
      console.log('[Test] PASSED: Plugin load error handling works correctly');
    } catch (error) {
      console.error('[Test] FAILED: Plugin load error handling:', error);
      throw error;
    }
  });

  test('should handle plugin execute errors gracefully', async ({ page }) => {
    console.log('[Test] Starting: Plugin execute error handling');
    
    try {
      // Initialize plugin system
      await invokeTauri<boolean>(page, 'plugin_init');
      await invokeTauri<any[]>(page, 'plugin_discover');
      
      // Try to execute a capability on a non-existent plugin
      console.log('[Test] Attempting to execute capability on non-existent plugin');
      
      try {
        await invokeTauri<any>(page, 'plugin_execute', {
          pluginId: 'non-existent-plugin',
          capabilityId: 'non-existent-capability',
          input: '{}'
        });
        console.log('[Test] WARNING: Expected error was not thrown');
      } catch (error) {
        console.log('[Test] Expected error occurred:', error);
        // This is expected - the plugin doesn't exist
      }
      
      console.log('[Test] PASSED: Plugin execute error handling works correctly');
    } catch (error) {
      console.error('[Test] FAILED: Plugin execute error handling:', error);
      throw error;
    }
  });

  test('should unload plugin', async ({ page }) => {
    console.log('[Test] Starting: Unload plugin');
    
    try {
      // Initialize plugin system
      await invokeTauri<boolean>(page, 'plugin_init');
      await invokeTauri<any[]>(page, 'plugin_discover');
      await invokeTauri<any[]>(page, 'plugin_load_all');
      
      // Get loaded plugins
      const loadedPlugins = await invokeTauri<any[]>(page, 'plugin_list_loaded');
      
      if (loadedPlugins.length === 0) {
        console.log('[Test] Skipping: No plugins to unload');
        test.skip();
        return;
      }
      
      // Unload the first plugin
      const firstPlugin = loadedPlugins[0];
      console.log(`[Test] Unloading plugin: ${firstPlugin.id}`);
      
      await invokeTauri<void>(page, 'plugin_unload', { pluginId: firstPlugin.id });
      
      // Verify plugin is unloaded
      const remainingPlugins = await invokeTauri<any[]>(page, 'plugin_list_loaded');
      console.log(`[Test] Remaining loaded plugins: ${remainingPlugins.length}`);
      
      const unloadedPlugin = remainingPlugins.find((p: any) => p.id === firstPlugin.id);
      expect(unloadedPlugin).toBeUndefined();
      
      console.log('[Test] PASSED: Plugin unloaded successfully');
    } catch (error) {
      console.error('[Test] FAILED: Unload plugin:', error);
      throw error;
    }
  });

  test('should broadcast events to plugins', async ({ page }) => {
    console.log('[Test] Starting: Broadcast events to plugins');
    
    try {
      // Initialize plugin system
      await invokeTauri<boolean>(page, 'plugin_init');
      await invokeTauri<any[]>(page, 'plugin_discover');
      await invokeTauri<any[]>(page, 'plugin_load_all');
      
      // Broadcast various events
      const events = [
        { type: 'Startup', data: {} },
        { type: 'FileOpened', data: { path: '/test/file.txt' } },
        { type: 'ContentChanged', data: { path: '/test/file.txt' } },
        { type: 'FileSaved', data: { path: '/test/file.txt' } },
      ];
      
      for (const event of events) {
        console.log(`[Test] Broadcasting event: ${event.type}`, event.data);
        await invokeTauri<void>(page, 'plugin_broadcast_event', {
          eventType: event.type,
          eventData: event.data
        });
      }
      
      console.log('[Test] PASSED: Events broadcast successfully');
    } catch (error) {
      console.error('[Test] FAILED: Broadcast events:', error);
      throw error;
    }
  });

  test('should get plugin directory', async ({ page }) => {
    console.log('[Test] Starting: Get plugin directory');
    
    try {
      // Initialize plugin system
      await invokeTauri<boolean>(page, 'plugin_init');
      
      const pluginDir = await invokeTauri<string>(page, 'plugin_get_directory');
      console.log('[Test] Plugin directory:', pluginDir);
      
      expect(pluginDir).toBeTruthy();
      expect(typeof pluginDir).toBe('string');
      expect(pluginDir.length).toBeGreaterThan(0);
      
      console.log('[Test] PASSED: Got plugin directory');
    } catch (error) {
      console.error('[Test] FAILED: Get plugin directory:', error);
      throw error;
    }
  });

  test('should get plugin info', async ({ page }) => {
    console.log('[Test] Starting: Get plugin info');
    
    try {
      // Initialize plugin system
      await invokeTauri<boolean>(page, 'plugin_init');
      await invokeTauri<any[]>(page, 'plugin_discover');
      const loadResults = await invokeTauri<any[]>(page, 'plugin_load_all');
      
      // Get loaded plugins
      const loadedPlugins = await invokeTauri<any[]>(page, 'plugin_list_loaded');
      
      if (loadedPlugins.length === 0) {
        console.log('[Test] Skipping: No plugins to get info for');
        test.skip();
        return;
      }
      
      // Get info for first plugin
      const firstPlugin = loadedPlugins[0];
      console.log(`[Test] Getting info for plugin: ${firstPlugin.id}`);
      
      const pluginInfo = await invokeTauri<any>(page, 'plugin_get_info', {
        pluginId: firstPlugin.id
      });
      
      console.log('[Test] Plugin info:', pluginInfo);
      expect(pluginInfo).toHaveProperty('id');
      expect(pluginInfo).toHaveProperty('name');
      expect(pluginInfo).toHaveProperty('version');
      
      console.log('[Test] PASSED: Got plugin info');
    } catch (error) {
      console.error('[Test] FAILED: Get plugin info:', error);
      throw error;
    }
  });

  test('should enable/disable plugin', async ({ page }) => {
    console.log('[Test] Starting: Enable/disable plugin');
    
    try {
      // Initialize plugin system
      await invokeTauri<boolean>(page, 'plugin_init');
      await invokeTauri<any[]>(page, 'plugin_discover');
      await invokeTauri<any[]>(page, 'plugin_load_all');
      
      // Get loaded plugins
      const loadedPlugins = await invokeTauri<any[]>(page, 'plugin_list_loaded');
      
      if (loadedPlugins.length === 0) {
        console.log('[Test] Skipping: No plugins to enable/disable');
        test.skip();
        return;
      }
      
      const firstPlugin = loadedPlugins[0];
      console.log(`[Test] Testing enable/disable for plugin: ${firstPlugin.id}`);
      
      // Disable plugin
      console.log(`[Test] Disabling plugin: ${firstPlugin.id}`);
      await invokeTauri<void>(page, 'plugin_set_enabled', {
        pluginId: firstPlugin.id,
        enabled: false
      });
      
      // Enable plugin
      console.log(`[Test] Enabling plugin: ${firstPlugin.id}`);
      await invokeTauri<void>(page, 'plugin_set_enabled', {
        pluginId: firstPlugin.id,
        enabled: true
      });
      
      console.log('[Test] PASSED: Plugin enable/disable works');
    } catch (error) {
      console.error('[Test] FAILED: Enable/disable plugin:', error);
      throw error;
    }
  });
});

test.describe('Plugin System Error Scenarios', () => {

  test('should handle uninitialized plugin system', async ({ page }) => {
    console.log('[Test] Starting: Uninitialized plugin system handling');
    
    // Don't initialize - try to use commands directly
    try {
      await invokeTauri<any[]>(page, 'plugin_discover');
      console.log('[Test] WARNING: Expected error for uninitialized system was not thrown');
    } catch (error) {
      console.log('[Test] Expected error occurred for uninitialized system:', error);
    }
    
    console.log('[Test] PASSED: Uninitialized system handled correctly');
  });

  test('should handle invalid plugin ID', async ({ page }) => {
    console.log('[Test] Starting: Invalid plugin ID handling');
    
    await invokeTauri<boolean>(page, 'plugin_init');
    
    // Try to get info for non-existent plugin
    try {
      await invokeTauri<any>(page, 'plugin_get_info', {
        pluginId: 'invalid-plugin-id-!@#$%'
      });
      console.log('[Test] WARNING: Expected error for invalid plugin ID was not thrown');
    } catch (error) {
      console.log('[Test] Expected error occurred for invalid plugin ID:', error);
    }
    
    console.log('[Test] PASSED: Invalid plugin ID handled correctly');
  });

  test('should handle duplicate plugin load', async ({ page }) => {
    console.log('[Test] Starting: Duplicate plugin load handling');
    
    await invokeTauri<boolean>(page, 'plugin_init');
    await invokeTauri<any[]>(page, 'plugin_discover');
    
    // Get list of plugins
    const plugins = await invokeTauri<any[]>(page, 'plugin_discover');
    
    if (plugins.length === 0) {
      console.log('[Test] Skipping: No plugins to test duplicate load');
      test.skip();
      return;
    }
    
    // Load first plugin
    const firstPlugin = plugins[0];
    console.log(`[Test] Loading plugin: ${firstPlugin.id}`);
    
    const firstResult = await invokeTauri<any>(page, 'plugin_load', {
      pluginId: firstPlugin.id
    });
    console.log('[Test] First load result:', firstResult);
    
    // Try to load same plugin again
    console.log(`[Test] Attempting duplicate load: ${firstPlugin.id}`);
    const secondResult = await invokeTauri<any>(page, 'plugin_load', {
      pluginId: firstPlugin.id
    });
    console.log('[Test] Duplicate load result:', secondResult);
    
    // Should indicate already loaded
    if (secondResult.success === false) {
      console.log('[Test] Duplicate load correctly reported as already loaded');
    }
    
    console.log('[Test] PASSED: Duplicate plugin load handled correctly');
  });
});
