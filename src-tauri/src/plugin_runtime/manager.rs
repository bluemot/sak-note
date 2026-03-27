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
        log::info!("[plugin:manager] Creating new PluginManager");
        
        // Get plugin directory
        let plugin_dir = get_plugin_dir();
        log::info!("[plugin:manager] Plugin directory: {:?}", plugin_dir);
        
        // Create directory if it doesn't exist
        if let Err(e) = std::fs::create_dir_all(&plugin_dir) {
            log::error!("[plugin:manager] Failed to create plugin directory: {}", e);
            return Err(format!("Failed to create plugin directory: {}", e));
        }
        log::debug!("[plugin:manager] Plugin directory ensured at {:?}", plugin_dir);
        
        // Create WASM engine
        let engine_config = WasmEngineConfig {
            allow_fs: true,
            allow_network: false,
            allow_env: false,
            plugin_dir: plugin_dir.clone(),
            preopened_dirs: vec![],
        };
        
        log::debug!("[plugin:manager] Creating WASM engine with config: allow_fs={}, allow_network={}, allow_env={}",
            engine_config.allow_fs, engine_config.allow_network, engine_config.allow_env);
        
        let engine = WasmEngine::new(engine_config)
            .map_err(|e| {
                log::error!("[plugin:manager] Failed to create WASM engine: {}", e);
                format!("Failed to create WASM engine: {}", e)
            })?;
        
        log::info!("[plugin:manager] PluginManager created successfully");
        
        Ok(Self {
            loaded_plugins: HashMap::new(),
            plugin_dir,
            engine,
        })
    }
    
    /// Discover all plugins in the plugin directory
    pub fn discover_plugins(&mut self) -> Result<Vec<DiscoveredPlugin>, PluginError> {
        log::info!("[plugin:manager] Starting plugin discovery in {:?}", self.plugin_dir);
        
        let mut discovered = Vec::new();
        
        if !self.plugin_dir.exists() {
            log::warn!("[plugin:manager] Plugin directory does not exist: {:?}", self.plugin_dir);
            return Ok(discovered);
        }
        
        let mut manifest_files_found = 0;
        let mut wasm_files_found = 0;
        let mut errors = 0;
        
        // Walk plugin directory
        for entry in WalkDir::new(&self.plugin_dir)
            .max_depth(3)
            .follow_links(false)
        {
            let entry = match entry {
                Ok(e) => e,
                Err(e) => {
                    log::warn!("[plugin:manager] Error walking directory: {}", e);
                    errors += 1;
                    continue;
                }
            };
            
            // Look for plugin.json files
            if entry.file_name() == "plugin.json" {
                manifest_files_found += 1;
                let manifest_path = entry.path();
                log::debug!("[plugin:manager] Found manifest file: {:?}", manifest_path);
                
                // Try to parse manifest
                match self.load_manifest(manifest_path) {
                    Ok(manifest) => {
                        log::debug!("[plugin:manager] Parsed manifest: id={}, name={}, version={}, entry_point={}",
                            manifest.id, manifest.name, manifest.version, manifest.entry_point);
                        
                        // Find corresponding WASM file
                        let parent = manifest_path.parent()
                            .ok_or_else(|| PluginError::new("invalid_path", "Manifest has no parent directory"))?;
                        let wasm_path = parent.join(&manifest.entry_point);
                        
                        if wasm_path.exists() {
                            log::info!("[plugin:manager] Found valid plugin: {} ({}) at {:?}",
                                manifest.name, manifest.id, wasm_path);
                            wasm_files_found += 1;
                            discovered.push(DiscoveredPlugin {
                                manifest,
                                wasm_path,
                                manifest_path: manifest_path.to_path_buf(),
                            });
                        } else {
                            log::warn!("[plugin:manager] Plugin '{}' references non-existent WASM file: {:?}",
                                manifest.name, wasm_path);
                        }
                    }
                    Err(e) => {
                        log::warn!("[plugin:manager] Failed to load manifest at {:?}: {}", manifest_path, e);
                        errors += 1;
                    }
                }
            }
        }
        
        log::info!("[plugin:manager] Plugin discovery complete: {} manifests found, {} valid plugins, {} errors",
            manifest_files_found, discovered.len(), errors);
        
        Ok(discovered)
    }
    
    /// Load a plugin manifest
    fn load_manifest(&self, path: &std::path::Path) -> Result<PluginManifest, PluginError> {
        log::debug!("[plugin:manager] Loading manifest from {:?}", path);
        
        let content = std::fs::read_to_string(path)
            .map_err(|e| {
                log::error!("[plugin:manager] Failed to read manifest file: {}", e);
                PluginError::new("read_failed", &format!("Failed to read manifest: {}", e))
            })?;
        
        log::debug!("[plugin:manager] Manifest file content length: {} bytes", content.len());
        
        let manifest: PluginManifest = serde_json::from_str(&content)
            .map_err(|e| {
                log::error!("[plugin:manager] Failed to parse manifest JSON: {}", e);
                PluginError::new("parse_failed", &format!("Failed to parse manifest: {}", e))
            })?;
        
        log::debug!("[plugin:manager] Successfully parsed manifest: {} ({}) v{}",
            manifest.name, manifest.id, manifest.version);
        
        Ok(manifest)
    }
    
    /// Load all discovered plugins
    pub fn load_all(&mut self) -> Result<Vec<PluginLoadStatus>, PluginError> {
        log::info!("[plugin:manager] Starting load_all");
        
        let discovered = self.discover_plugins()?;
        let mut results = Vec::new();
        
        log::info!("[plugin:manager] Loading {} discovered plugins", discovered.len());
        
        for plugin in discovered {
            let status = self.load_plugin(plugin);
            results.push(status);
        }
        
        let success_count = results.iter().filter(|r| r.success).count();
        log::info!("[plugin:manager] load_all complete: {}/{} plugins loaded successfully", success_count, results.len());
        
        Ok(results)
    }
    
    /// Load a specific plugin
    pub fn load_plugin(&mut self, discovered: DiscoveredPlugin) -> PluginLoadStatus {
        let plugin_id = discovered.manifest.id.clone();
        log::info!("[plugin:manager] Loading plugin: {} ({}) from {:?}",
            discovered.manifest.name, plugin_id, discovered.wasm_path);
        
        // Check if already loaded
        if self.loaded_plugins.contains_key(&plugin_id) {
            log::warn!("[plugin:manager] Plugin {} is already loaded, skipping", plugin_id);
            return PluginLoadStatus {
                plugin_id: plugin_id.clone(),
                success: false,
                message: "Plugin already loaded".to_string(),
            };
        }
        
        log::debug!("[plugin:manager] Plugin {} not already loaded, proceeding with WASM engine", plugin_id);
        
        // Load using WASM engine
        match self.engine.load_plugin(discovered.manifest, discovered.wasm_path) {
            Ok(plugin) => {
                let name = plugin.manifest.name.clone();
                let caps_count = plugin.manifest.capabilities.len();
                self.loaded_plugins.insert(plugin_id.clone(), plugin);
                
                log::info!("[plugin:manager] Successfully loaded plugin '{}' ({}): {} capabilities",
                    name, plugin_id, caps_count);
                log::debug!("[plugin:manager] Plugin {} has permissions: {:?}",
                    plugin_id, self.loaded_plugins.get(&plugin_id).map(|p| &p.manifest.permissions));
                
                PluginLoadStatus {
                    plugin_id,
                    success: true,
                    message: format!("Loaded plugin '{}' with {} capabilities", name, caps_count),
                }
            }
            Err(e) => {
                log::error!("[plugin:manager] Failed to load plugin {}: {}", plugin_id, e);
                
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
        log::info!("[plugin:manager] Unloading plugin: {}", plugin_id);
        
        if let Some(mut plugin) = self.loaded_plugins.remove(plugin_id) {
            log::debug!("[plugin:manager] Found plugin {}, calling shutdown", plugin_id);
            plugin.shutdown()?;
            log::info!("[plugin:manager] Successfully unloaded plugin {}", plugin_id);
        } else {
            log::warn!("[plugin:manager] Plugin {} was not loaded, nothing to unload", plugin_id);
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
        log::info!("[plugin:manager] execute_capability: plugin_id={}, capability_id={}",
            plugin_id, capability_id);
        log::debug!("[plugin:manager] execute_capability input length: {} bytes", input.len());
        
        let plugin = self.loaded_plugins.get_mut(plugin_id)
            .ok_or_else(|| {
                log::error!("[plugin:manager] Plugin '{}' not found for capability execution", plugin_id);
                PluginError::new("plugin_not_found", &format!("Plugin '{}' not found", plugin_id))
            })?;
        
        log::debug!("[plugin:manager] Found plugin {}, executing capability {}", plugin_id, capability_id);
        
        plugin.execute_capability(capability_id, input)
    }
    
    /// Get all loaded plugins
    pub fn get_loaded_plugins(&self) -> Vec<&PluginMetadata> {
        let count = self.loaded_plugins.len();
        log::debug!("[plugin:manager] get_loaded_plugins called, returning {} plugins", count);
        self.loaded_plugins.values()
            .map(|p| &p.metadata)
            .collect()
    }
    
    /// Get plugin by ID
    pub fn get_plugin(&self, plugin_id: &str) -> Option<&PluginMetadata> {
        log::debug!("[plugin:manager] get_plugin called for: {}", plugin_id);
        let result = self.loaded_plugins.get(plugin_id)
            .map(|p| &p.metadata);
        
        match &result {
            Some(meta) => log::debug!("[plugin:manager] Found plugin {}: {} v{}", plugin_id, meta.name, meta.version),
            None => log::debug!("[plugin:manager] Plugin {} not found", plugin_id),
        }
        
        result
    }
    
    /// Broadcast an editor event to all plugins
    pub fn broadcast_event(&mut self, event: &EditorEvent) {
        let plugin_count = self.loaded_plugins.len();
        log::info!("[plugin:manager] Broadcasting event {:?} to {} plugins", event, plugin_count);
        
        let mut success_count = 0;
        let mut error_count = 0;
        
        for (plugin_id, plugin) in &mut self.loaded_plugins {
            log::debug!("[plugin:manager] Sending event to plugin {}", plugin_id);
            match plugin.on_event(event) {
                Ok(()) => {
                    log::debug!("[plugin:manager] Plugin {} handled event successfully", plugin_id);
                    success_count += 1;
                }
                Err(e) => {
                    log::warn!("[plugin:manager] Plugin {} failed to handle event: {}", plugin_id, e);
                    error_count += 1;
                }
            }
        }
        
        log::info!("[plugin:manager] Event broadcast complete: {} success, {} errors", success_count, error_count);
    }
    
    /// Get all capabilities from all loaded plugins
    pub fn get_all_capabilities(&self) -> Vec<(PluginCapability, String)> {
        let mut capabilities = Vec::new();
        
        log::debug!("[plugin:manager] get_all_capabilities called for {} plugins", self.loaded_plugins.len());
        
        for (plugin_id, plugin) in &self.loaded_plugins {
            let caps_count = plugin.manifest.capabilities.len();
            log::debug!("[plugin:manager] Plugin {} has {} capabilities", plugin_id, caps_count);
            
            for cap in &plugin.manifest.capabilities {
                log::debug!("[plugin:manager]   - Capability: {} from {}", cap.id, plugin_id);
                capabilities.push((cap.clone(), plugin_id.clone()));
            }
        }
        
        log::info!("[plugin:manager] get_all_capabilities returning {} total capabilities", capabilities.len());
        capabilities
    }
    
    /// Enable/disable a plugin
    pub fn set_plugin_enabled(&mut self, plugin_id: &str, enabled: bool) -> Result<(), PluginError> {
        log::info!("[plugin:manager] set_plugin_enabled: plugin_id={}, enabled={}", plugin_id, enabled);
        
        if let Some(plugin) = self.loaded_plugins.get_mut(plugin_id) {
            plugin.metadata.enabled = enabled;
            log::info!("[plugin:manager] Plugin {} enabled status set to {}", plugin_id, enabled);
            Ok(())
        } else {
            log::error!("[plugin:manager] Cannot set enabled status: Plugin {} not found", plugin_id);
            Err(PluginError::new("plugin_not_found", &format!("Plugin '{}' not found", plugin_id)))
        }
    }
}

// PluginLoadStatus is now in plugin_api.rs

/// Get the default plugin directory
fn get_plugin_dir() -> PathBuf {
    log::debug!("[plugin:manager] Getting plugin directory");
    
    let dir = if let Some(base) = BaseDirs::new() {
        let path = base.config_dir()
            .join("sak-editor")
            .join("plugins");
        log::debug!("[plugin:manager] Using BaseDirs config path: {:?}", path);
        path
    } else {
        // Fallback to home directory
        let fallback = PathBuf::from(std::env::var("HOME").unwrap_or_else(|_| ".".to_string()))
            .join(".config")
            .join("sak-editor")
            .join("plugins");
        log::debug!("[plugin:manager] Using fallback home directory path: {:?}", fallback);
        fallback
    };
    
    log::info!("[plugin:manager] Resolved plugin directory: {:?}", dir);
    dir
}

/// Global plugin manager instance
use lazy_static::lazy_static;
lazy_static! {
    static ref PLUGIN_MANAGER: Arc<Mutex<Option<PluginManager>>> = 
        Arc::new(Mutex::new(None));
}

/// Initialize the global plugin manager
pub fn init_plugin_manager() -> Result<(), String> {
    log::info!("[plugin:manager] Initializing global plugin manager");
    
    let manager = PluginManager::new()?;
    
    let mut global = PLUGIN_MANAGER.lock().map_err(|e| {
        log::error!("[plugin:manager] Failed to lock global PLUGIN_MANAGER: {}", e);
        e.to_string()
    })?;
    *global = Some(manager);
    
    log::info!("[plugin:manager] Global plugin manager initialized successfully");
    Ok(())
}

/// Get the global plugin manager
pub fn get_plugin_manager() -> Result<Arc<Mutex<Option<PluginManager>>>, String> {
    log::debug!("[plugin:manager] get_plugin_manager called");
    Ok(PLUGIN_MANAGER.clone())
}
