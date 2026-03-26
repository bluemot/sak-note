//! Plugin Manager
//!
//! Discovers, loads, and manages WASM plugins from the plugins directory.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use walkdir::WalkDir;
use directories::BaseDirs;

use crate::plugin_runtime::{
    plugin_api::{
        PluginManifest, PluginError, PluginMetadata, PluginCapability,
        PluginPermission, EditorEvent, PluginLoadStatus
    },
    wasm_engine::{WasmEngine, WasmEngineConfig, WasmPlugin},
};

/// Plugin manager - central hub for plugin operations
pub struct PluginManager {
    pub loaded_plugins: HashMap<String, WasmPlugin>,
    pub plugin_dir: PathBuf,
    engine: WasmEngine,
}

/// Plugin discovery info
#[derive(Debug, Clone)]
pub struct DiscoveredPlugin {
    pub manifest: PluginManifest,
    pub wasm_path: PathBuf,
    pub manifest_path: PathBuf,
}

impl PluginManager {
    /// Create a new plugin manager
    pub fn new() -> Result<Self, String> {
        // Get plugin directory
        let plugin_dir = get_plugin_dir();
        
        // Create directory if it doesn't exist
        std::fs::create_dir_all(&plugin_dir)
            .map_err(|e| format!("Failed to create plugin directory: {}", e))?;
        
        // Create WASM engine
        let engine_config = WasmEngineConfig {
            allow_fs: true,
            allow_network: false,
            allow_env: false,
            plugin_dir: plugin_dir.clone(),
            preopened_dirs: vec![],
        };
        
        let engine = WasmEngine::new(engine_config)
            .map_err(|e| format!("Failed to create WASM engine: {}", e))?;
        
        Ok(Self {
            loaded_plugins: HashMap::new(),
            plugin_dir,
            engine,
        })
    }
    
    /// Discover all plugins in the plugin directory
    pub fn discover_plugins(&mut self) -> Result<Vec<DiscoveredPlugin>, PluginError> {
        let mut discovered = Vec::new();
        
        if !self.plugin_dir.exists() {
            return Ok(discovered);
        }
        
        // Walk plugin directory
        for entry in WalkDir::new(&self.plugin_dir)
            .max_depth(3)
            .follow_links(false)
        {
            let entry = match entry {
                Ok(e) => e,
                Err(_) => continue,
            };
            
            // Look for plugin.json files
            if entry.file_name() == "plugin.json" {
                let manifest_path = entry.path();
                
                // Try to parse manifest
                match self.load_manifest(manifest_path) {
                    Ok(manifest) => {
                        // Find corresponding WASM file
                        let parent = manifest_path.parent()
                            .ok_or_else(|| PluginError::new("invalid_path", "Manifest has no parent directory"))?;
                        let wasm_path = parent.join(&manifest.entry_point);
                        
                        if wasm_path.exists() {
                            discovered.push(DiscoveredPlugin {
                                manifest,
                                wasm_path,
                                manifest_path: manifest_path.to_path_buf(),
                            });
                        } else {
                            log::warn!(
                                "Plugin '{}' references non-existent WASM file: {:?}",
                                manifest.name, wasm_path
                            );
                        }
                    }
                    Err(e) => {
                        log::warn!("Failed to load manifest at {:?}: {}", manifest_path, e);
                    }
                }
            }
        }
        
        log::info!("Discovered {} plugins", discovered.len());
        Ok(discovered)
    }
    
    /// Load a plugin manifest
    fn load_manifest(&self, path: &std::path::Path) -> Result<PluginManifest, PluginError> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| PluginError::new("read_failed", &format!("Failed to read manifest: {}", e)))?;
        
        let manifest: PluginManifest = serde_json::from_str(&content)
            .map_err(|e| PluginError::new("parse_failed", &format!("Failed to parse manifest: {}", e)))?;
        
        Ok(manifest)
    }
    
    /// Load all discovered plugins
    pub fn load_all(&mut self) -> Result<Vec<PluginLoadStatus>, PluginError> {
        let discovered = self.discover_plugins()?;
        let mut results = Vec::new();
        
        for plugin in discovered {
            let status = self.load_plugin(plugin);
            results.push(status);
        }
        
        Ok(results)
    }
    
    /// Load a specific plugin
    pub fn load_plugin(&mut self, discovered: DiscoveredPlugin) -> PluginLoadStatus {
        let plugin_id = discovered.manifest.id.clone();
        
        // Check if already loaded
        if self.loaded_plugins.contains_key(&plugin_id) {
            return PluginLoadStatus {
                plugin_id: plugin_id.clone(),
                success: false,
                message: "Plugin already loaded".to_string(),
            };
        }
        
        // Load using WASM engine
        match self.engine.load_plugin(discovered.manifest, discovered.wasm_path) {
            Ok(plugin) => {
                let name = plugin.manifest.name.clone();
                self.loaded_plugins.insert(plugin_id.clone(), plugin);
                
                log::info!("Successfully loaded plugin '{}' ({})", name, plugin_id);
                
                PluginLoadStatus {
                    plugin_id,
                    success: true,
                    message: format!("Loaded plugin '{}'", name),
                }
            }
            Err(e) => {
                log::error!("Failed to load plugin {}: {}", plugin_id, e);
                
                PluginLoadStatus {
                    plugin_id,
                    success: false,
                    message: e.to_string(),
                }
            }
        }
    }
    
    /// Unload a plugin
    pub fn unload_plugin(&mut self, plugin_id: &str) -> Result<(), PluginError> {
        if let Some(mut plugin) = self.loaded_plugins.remove(plugin_id) {
            plugin.shutdown()?;
            log::info!("Unloaded plugin {}", plugin_id);
        }
        
        Ok(())
    }
    
    /// Execute a plugin capability
    pub fn execute_capability(
        &mut self,
        plugin_id: &str,
        capability_id: &str,
        input: &str,
    ) -> Result<String, PluginError> {
        let plugin = self.loaded_plugins.get_mut(plugin_id)
            .ok_or_else(|| PluginError::new("plugin_not_found", &format!("Plugin '{}' not found", plugin_id)))?;
        
        plugin.execute_capability(capability_id, input)
    }
    
    /// Get all loaded plugins
    pub fn get_loaded_plugins(&self) -> Vec<&PluginMetadata> {
        self.loaded_plugins.values()
            .map(|p| &p.metadata)
            .collect()
    }
    
    /// Get plugin by ID
    pub fn get_plugin(&self, plugin_id: &str) -> Option<&PluginMetadata> {
        self.loaded_plugins.get(plugin_id)
            .map(|p| &p.metadata)
    }
    
    /// Broadcast an editor event to all plugins
    pub fn broadcast_event(&mut self, event: &EditorEvent) {
        for (plugin_id, plugin) in &mut self.loaded_plugins {
            if let Err(e) = plugin.on_event(event) {
                log::warn!("Plugin {} failed to handle event: {}", plugin_id, e);
            }
        }
    }
    
    /// Get all capabilities from all loaded plugins
    pub fn get_all_capabilities(&self) -> Vec<(PluginCapability, String)> {
        let mut capabilities = Vec::new();
        
        for (plugin_id, plugin) in &self.loaded_plugins {
            for cap in &plugin.manifest.capabilities {
                capabilities.push((cap.clone(), plugin_id.clone()));
            }
        }
        
        capabilities
    }
    
    /// Enable/disable a plugin
    pub fn set_plugin_enabled(&mut self, plugin_id: &str, enabled: bool) -> Result<(), PluginError> {
        if let Some(plugin) = self.loaded_plugins.get_mut(plugin_id) {
            plugin.metadata.enabled = enabled;
            Ok(())
        } else {
            Err(PluginError::new("plugin_not_found", &format!("Plugin '{}' not found", plugin_id)))
        }
    }
}

// PluginLoadStatus is now in plugin_api.rs

/// Get the default plugin directory
fn get_plugin_dir() -> PathBuf {
    if let Some(base) = BaseDirs::new() {
        base.config_dir()
            .join("sak-editor")
            .join("plugins")
    } else {
        // Fallback to home directory
        PathBuf::from(std::env::var("HOME").unwrap_or_else(|_| ".".to_string()))
            .join(".config")
            .join("sak-editor")
            .join("plugins")
    }
}

/// Global plugin manager instance
use lazy_static::lazy_static;
lazy_static! {
    static ref PLUGIN_MANAGER: Arc<Mutex<Option<PluginManager>>> = 
        Arc::new(Mutex::new(None));
}

/// Initialize the global plugin manager
pub fn init_plugin_manager() -> Result<(), String> {
    let manager = PluginManager::new()?;
    
    let mut global = PLUGIN_MANAGER.lock().map_err(|e| e.to_string())?;
    *global = Some(manager);
    
    Ok(())
}

/// Get the global plugin manager
pub fn get_plugin_manager() -> Result<Arc<Mutex<Option<PluginManager>>>, String> {
    Ok(PLUGIN_MANAGER.clone())
}