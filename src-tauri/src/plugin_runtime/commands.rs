//! Plugin Commands
//!
//! Tauri commands for managing WASM plugins from the frontend.

use serde::{Serialize, Deserialize};
use serde_json::Value;
use std::sync::Mutex;
use lazy_static::lazy_static;

use crate::plugin_runtime::{
    manager::PluginManager,
    plugin_api::{PluginManifest, PluginMetadata, EditorEvent, PluginLoadStatus, PluginExecutionResult},
};

/// Plugin command error
#[derive(Debug, Serialize)]
pub struct PluginCommandError {
    pub code: String,
    pub message: String,
}

impl From<String> for PluginCommandError {
    fn from(msg: String) -> Self {
        Self {
            code: "error".to_string(),
            message: msg,
        }
    }
}

/// Global plugin manager
lazy_static! {
    static ref PLUGIN_MANAGER: Mutex<Option<PluginManager>> = Mutex::new(None);
}

/// Initialize the plugin system
#[tauri::command]
pub fn plugin_init() -> Result<bool, PluginCommandError> {
    let manager = PluginManager::new()
        .map_err(|e| PluginCommandError::from(e))?;
    
    let mut global = PLUGIN_MANAGER.lock().map_err(|e| PluginCommandError::from(e.to_string()))?;
    *global = Some(manager);
    
    Ok(true)
}

/// Discover plugins in the plugin directory
#[tauri::command]
pub fn plugin_discover() -> Result<Vec<PluginManifest>, PluginCommandError> {
    let mut manager_guard = PLUGIN_MANAGER.lock().map_err(|e| PluginCommandError::from(e.to_string()))?;
    
    let manager = manager_guard.as_mut()
        .ok_or_else(|| PluginCommandError::from("Plugin system not initialized".to_string()))?;
    
    let discovered = manager.discover_plugins()
        .map_err(|e| PluginCommandError::from(e.to_string()))?;
    
    Ok(discovered.into_iter().map(|p| p.manifest).collect())
}

/// Load all discovered plugins
#[tauri::command]
pub fn plugin_load_all() -> Result<Vec<PluginLoadStatus>, PluginCommandError> {
    let mut manager_guard = PLUGIN_MANAGER.lock().map_err(|e| PluginCommandError::from(e.to_string()))?;
    
    let manager = manager_guard.as_mut()
        .ok_or_else(|| PluginCommandError::from("Plugin system not initialized".to_string()))?;
    
    let results = manager.load_all()
        .map_err(|e| PluginCommandError::from(e.to_string()))?;
    
    Ok(results)
}

/// Load a specific plugin
#[tauri::command]
pub fn plugin_load(plugin_id: String) -> Result<PluginLoadStatus, PluginCommandError> {
    let mut manager_guard = PLUGIN_MANAGER.lock().map_err(|e| PluginCommandError::from(e.to_string()))?;
    
    let manager = manager_guard.as_mut()
        .ok_or_else(|| PluginCommandError::from("Plugin system not initialized".to_string()))?;
    
    // First discover to find the plugin
    let discovered = manager.discover_plugins()
        .map_err(|e| PluginCommandError::from(e.to_string()))?;
    
    let plugin = discovered.into_iter()
        .find(|p| p.manifest.id == plugin_id)
        .ok_or_else(|| PluginCommandError::from(format!("Plugin '{}' not found", plugin_id)))?;
    
    let status = manager.load_plugin(plugin);
    
    Ok(status)
}

/// Unload a plugin
#[tauri::command]
pub fn plugin_unload(plugin_id: String) -> Result<(), PluginCommandError> {
    let mut manager_guard = PLUGIN_MANAGER.lock().map_err(|e| PluginCommandError::from(e.to_string()))?;
    
    let manager = manager_guard.as_mut()
        .ok_or_else(|| PluginCommandError::from("Plugin system not initialized".to_string()))?;
    
    manager.unload_plugin(&plugin_id)
        .map_err(|e| PluginCommandError::from(e.to_string()))?;
    
    Ok(())
}

/// Get all loaded plugins
#[tauri::command]
pub fn plugin_list_loaded() -> Result<Vec<PluginMetadata>, PluginCommandError> {
    let manager_guard = PLUGIN_MANAGER.lock().map_err(|e| PluginCommandError::from(e.to_string()))?;
    
    let manager = manager_guard.as_ref()
        .ok_or_else(|| PluginCommandError::from("Plugin system not initialized".to_string()))?;
    
    Ok(manager.get_loaded_plugins()
        .into_iter()
        .cloned()
        .collect())
}

/// Get plugin info
#[tauri::command]
pub fn plugin_get_info(plugin_id: String) -> Result<PluginMetadata, PluginCommandError> {
    let manager_guard = PLUGIN_MANAGER.lock().map_err(|e| PluginCommandError::from(e.to_string()))?;
    
    let manager = manager_guard.as_ref()
        .ok_or_else(|| PluginCommandError::from("Plugin system not initialized".to_string()))?;
    
    manager.get_plugin(&plugin_id)
        .cloned()
        .ok_or_else(|| PluginCommandError::from(format!("Plugin '{}' not found", plugin_id)))
}

/// Execute a plugin capability
#[tauri::command]
pub fn plugin_execute(
    plugin_id: String,
    capability_id: String,
    input: Option<String>,
) -> Result<PluginExecutionResult, PluginCommandError> {
    let mut manager_guard = PLUGIN_MANAGER.lock().map_err(|e| PluginCommandError::from(e.to_string()))?;
    
    let manager = manager_guard.as_mut()
        .ok_or_else(|| PluginCommandError::from("Plugin system not initialized".to_string()))?;
    
    let input = input.unwrap_or_default();
    
    match manager.execute_capability(&plugin_id, &capability_id, &input) {
        Ok(output) => Ok(PluginExecutionResult {
            success: true,
            output: Some(output),
            error: None,
        }),
        Err(e) => Ok(PluginExecutionResult {
            success: false,
            output: None,
            error: Some(e.to_string()),
        }),
    }
}

/// Enable/disable a plugin
#[tauri::command]
pub fn plugin_set_enabled(
    plugin_id: String,
    enabled: bool,
) -> Result<(), PluginCommandError> {
    let mut manager_guard = PLUGIN_MANAGER.lock().map_err(|e| PluginCommandError::from(e.to_string()))?;
    
    let manager = manager_guard.as_mut()
        .ok_or_else(|| PluginCommandError::from("Plugin system not initialized".to_string()))?;
    
    manager.set_plugin_enabled(&plugin_id, enabled)
        .map_err(|e| PluginCommandError::from(e.to_string()))?;
    
    Ok(())
}

/// Get all capabilities from all loaded plugins
#[tauri::command]
pub fn plugin_get_capabilities() -> Result<Vec<PluginCapabilityInfo>, PluginCommandError> {
    let manager_guard = PLUGIN_MANAGER.lock().map_err(|e| PluginCommandError::from(e.to_string()))?;
    
    let manager = manager_guard.as_ref()
        .ok_or_else(|| PluginCommandError::from("Plugin system not initialized".to_string()))?;
    
    let capabilities = manager.get_all_capabilities()
        .into_iter()
        .map(|(cap, plugin_id)| PluginCapabilityInfo {
            capability: cap,
            plugin_id,
        })
        .collect();
    
    Ok(capabilities)
}

/// Broadcast an editor event to all plugins
#[tauri::command]
pub fn plugin_broadcast_event(event_type: String, event_data: Value) -> Result<(), PluginCommandError> {
    let mut manager_guard = PLUGIN_MANAGER.lock().map_err(|e| PluginCommandError::from(e.to_string()))?;
    
    let manager = manager_guard.as_mut()
        .ok_or_else(|| PluginCommandError::from("Plugin system not initialized".to_string()))?;
    
    // Parse event from JSON
    let event = parse_event(&event_type, event_data)
        .map_err(|e| PluginCommandError::from(format!("Failed to parse event: {}", e)))?;
    
    manager.broadcast_event(&event);
    
    Ok(())
}

/// Get plugin directory path
#[tauri::command]
pub fn plugin_get_directory() -> Result<String, PluginCommandError> {
    let manager_guard = PLUGIN_MANAGER.lock().map_err(|e| PluginCommandError::from(e.to_string()))?;
    
    let manager = manager_guard.as_ref()
        .ok_or_else(|| PluginCommandError::from("Plugin system not initialized".to_string()))?;
    
    Ok(manager.plugin_dir.to_string_lossy().to_string())
}



/// Plugin capability info with plugin ID
#[derive(Debug, Serialize, Deserialize)]
pub struct PluginCapabilityInfo {
    #[serde(flatten)]
    pub capability: crate::plugin_runtime::plugin_api::PluginCapability,
    pub plugin_id: String,
}

/// Parse event from JSON
fn parse_event(event_type: &str, data: Value) -> Result<EditorEvent, String> {
    let event = match event_type {
        "FileOpened" => {
            let path = data.get("path")
                .and_then(|v| v.as_str())
                .ok_or("path required")?;
            EditorEvent::FileOpened { path: path.to_string() }
        }
        "FileClosed" => {
            let path = data.get("path")
                .and_then(|v| v.as_str())
                .ok_or("path required")?;
            EditorEvent::FileClosed { path: path.to_string() }
        }
        "FileSaved" => {
            let path = data.get("path")
                .and_then(|v| v.as_str())
                .ok_or("path required")?;
            EditorEvent::FileSaved { path: path.to_string() }
        }
        "ContentChanged" => {
            let path = data.get("path")
                .and_then(|v| v.as_str())
                .ok_or("path required")?;
            EditorEvent::ContentChanged { path: path.to_string() }
        }
        "SelectionChanged" => {
            let path = data.get("path")
                .and_then(|v| v.as_str())
                .ok_or("path required")?;
            let start = data.get("start")
                .and_then(|v| v.as_u64())
                .unwrap_or(0) as usize;
            let end = data.get("end")
                .and_then(|v| v.as_u64())
                .unwrap_or(0) as usize;
            EditorEvent::SelectionChanged {
                path: path.to_string(),
                start,
                end,
            }
        }
        "Startup" => EditorEvent::Startup,
        "Shutdown" => EditorEvent::Shutdown,
        _ => EditorEvent::Custom {
            name: event_type.to_string(),
            data,
        },
    };
    
    Ok(event)
}