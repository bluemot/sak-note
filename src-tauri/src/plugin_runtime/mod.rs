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
    let mut manager = PluginManager::new()?;
    
    // Discover and load plugins
    manager.discover_plugins()
        .map_err(|e| format!("Failed to discover plugins: {}", e))?;
    
    log::info!("Plugin system initialized with {} plugins", manager.loaded_plugins.len());
    
    Ok(manager)
}