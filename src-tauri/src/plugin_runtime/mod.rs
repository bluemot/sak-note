//! WASM Plugin Runtime System
//!
//! Provides a WebAssembly-based plugin system for SAK Editor.
//! Plugins are compiled to WASM and can interact with the editor
//! through a well-defined API using WASI for filesystem access.

#![allow(dead_code)]

use serde::{Serialize, Deserialize};

pub mod manager;
pub mod wasm_engine;
pub mod plugin_api;
pub mod bridge;
pub mod commands;

pub use manager::PluginManager;
pub use wasm_engine::WasmEngine;
pub use plugin_api::{Plugin, PluginManifest, PluginCapability, PluginUiComponent};

/// Plugin system configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfig {
    pub enabled: bool,
    pub auto_load: bool,
    pub plugin_dir: String,
}

impl Default for PluginConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            auto_load: true,
            plugin_dir: "~/.config/sak-editor/plugins".to_string(),
        }
    }
}

/// Plugin load result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginLoadResult {
    pub success: bool,
    pub plugin_id: String,
    pub message: String,
}

/// Plugin execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginExecutionResult {
    pub success: bool,
    pub output: Option<String>,
    pub error: Option<String>,
}

/// Initialize the plugin system
pub fn init() -> Result<PluginManager, String> {
    log::info!("[plugin:init] Initializing plugin system...");
    
    let mut manager = PluginManager::new()
        .map_err(|e| {
            log::error!("[plugin:init] Failed to create PluginManager: {}", e);
            e
        })?;
    
    log::info!("[plugin:init] PluginManager created, discovering plugins...");
    
    // Discover and load plugins
    match manager.discover_plugins() {
        Ok(discovered) => {
            log::info!("[plugin:init] Discovered {} plugins", discovered.len());
            for plugin in &discovered {
                log::info!("[plugin:init]   - {} v{} at {:?}", plugin.manifest.id, plugin.manifest.version, plugin.wasm_path);
            }
        }
        Err(e) => {
            log::error!("[plugin:init] Failed to discover plugins: {}", e);
        }
    }
    
    log::info!("[plugin:init] Plugin system initialized with {} plugins loaded", manager.loaded_plugins.len());
    
    Ok(manager)
}
