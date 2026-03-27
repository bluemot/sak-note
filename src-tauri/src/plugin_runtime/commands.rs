//! Plugin Commands
//!
//! Tauri commands for managing WASM plugins from the frontend.

use serde::{Serialize, Deserialize};
use serde_json::Value;
use std::sync::Mutex;
use lazy_static::lazy_static;

use crate::plugin_runtime::{
    manager::PluginManager,
    plugin_api::{PluginManifest, PluginMetadata, PluginCapability, EditorEvent, PluginLoadStatus, PluginExecutionResult},
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
    log::info!("[plugin:commands] plugin_init called");
    
    let manager = PluginManager::new()
        .map_err(|e| {
            log::error!("[plugin:commands] Failed to create PluginManager: {}", e);
            PluginCommandError::from(e)
        })?;
    
    let mut global = PLUGIN_MANAGER.lock().map_err(|e| {
        log::error!("[plugin:commands] Failed to lock PLUGIN_MANAGER: {}", e);
        PluginCommandError::from(e.to_string())
    })?;
    *global = Some(manager);
    
    log::info!("[plugin:commands] plugin_init completed successfully");
    Ok(true)
}

/// Discover plugins in the plugin directory
#[tauri::command]
pub fn plugin_discover() -> Result<Vec<PluginManifest>, PluginCommandError> {
    log::info!("[plugin:commands] plugin_discover called");
    
    let mut manager_guard = PLUGIN_MANAGER.lock().map_err(|e| {
        log::error!("[plugin:commands] Failed to lock PLUGIN_MANAGER: {}", e);
        PluginCommandError::from(e.to_string())
    })?;
    
    let manager = manager_guard.as_mut()
        .ok_or_else(|| {
            log::error!("[plugin:commands] Plugin system not initialized");
            PluginCommandError::from("Plugin system not initialized".to_string())
        })?;
    
    let discovered = manager.discover_plugins()
        .map_err(|e| {
            log::error!("[plugin:commands] Failed to discover plugins: {}", e);
            PluginCommandError::from(e.to_string())
        })?;
    
    let manifests: Vec<PluginManifest> = discovered.into_iter().map(|p| p.manifest).collect();
    log::info!("[plugin:commands] plugin_discover found {} plugins", manifests.len());
    
    for manifest in &manifests {
        log::debug!("[plugin:commands]   - Discovered plugin: {} ({}) v{}", manifest.name, manifest.id, manifest.version);
    }
    
    Ok(manifests)
}

/// Load all discovered plugins
#[tauri::command]
pub fn plugin_load_all() -> Result<Vec<PluginLoadStatus>, PluginCommandError> {
    log::info!("[plugin:commands] plugin_load_all called");
    
    let mut manager_guard = PLUGIN_MANAGER.lock().map_err(|e| {
        log::error!("[plugin:commands] Failed to lock PLUGIN_MANAGER: {}", e);
        PluginCommandError::from(e.to_string())
    })?;
    
    let manager = manager_guard.as_mut()
        .ok_or_else(|| {
            log::error!("[plugin:commands] Plugin system not initialized");
            PluginCommandError::from("Plugin system not initialized".to_string())
        })?;
    
    let results = manager.load_all()
        .map_err(|e| {
            log::error!("[plugin:commands] Failed to load all plugins: {}", e);
            PluginCommandError::from(e.to_string())
        })?;
    
    log::info!("[plugin:commands] plugin_load_all completed: {} plugins processed", results.len());
    
    for result in &results {
        if result.success {
            log::info!("[plugin:commands]   - Successfully loaded: {} - {}", result.plugin_id, result.message);
        } else {
            log::warn!("[plugin:commands]   - Failed to load: {} - {}", result.plugin_id, result.message);
        }
    }
    
    Ok(results)
}

/// Load a specific plugin
#[tauri::command]
pub fn plugin_load(plugin_id: String) -> Result<PluginLoadStatus, PluginCommandError> {
    log::info!("[plugin:commands] plugin_load called with plugin_id: {}", plugin_id);
    
    let mut manager_guard = PLUGIN_MANAGER.lock().map_err(|e| {
        log::error!("[plugin:commands] Failed to lock PLUGIN_MANAGER: {}", e);
        PluginCommandError::from(e.to_string())
    })?;
    
    let manager = manager_guard.as_mut()
        .ok_or_else(|| {
            log::error!("[plugin:commands] Plugin system not initialized");
            PluginCommandError::from("Plugin system not initialized".to_string())
        })?;
    
    // First discover to find the plugin
    let discovered = manager.discover_plugins()
        .map_err(|e| {
            log::error!("[plugin:commands] Failed to discover plugins: {}", e);
            PluginCommandError::from(e.to_string())
        })?;
    
    let plugin = discovered.into_iter()
        .find(|p| p.manifest.id == plugin_id)
        .ok_or_else(|| {
            log::error!("[plugin:commands] Plugin '{}' not found during discovery", plugin_id);
            PluginCommandError::from(format!("Plugin '{}' not found", plugin_id))
        })?;
    
    log::debug!("[plugin:commands] Found plugin '{}' at {:?}", plugin_id, plugin.manifest_path);
    
    let status = manager.load_plugin(plugin);
    
    if status.success {
        log::info!("[plugin:commands] plugin_load completed successfully for {}: {}", plugin_id, status.message);
    } else {
        log::error!("[plugin:commands] plugin_load failed for {}: {}", plugin_id, status.message);
    }
    
    Ok(status)
}

/// Unload a plugin
#[tauri::command]
pub fn plugin_unload(plugin_id: String) -> Result<(), PluginCommandError> {
    log::info!("[plugin:commands] plugin_unload called with plugin_id: {}", plugin_id);
    
    let mut manager_guard = PLUGIN_MANAGER.lock().map_err(|e| {
        log::error!("[plugin:commands] Failed to lock PLUGIN_MANAGER: {}", e);
        PluginCommandError::from(e.to_string())
    })?;
    
    let manager = manager_guard.as_mut()
        .ok_or_else(|| {
            log::error!("[plugin:commands] Plugin system not initialized");
            PluginCommandError::from("Plugin system not initialized".to_string())
        })?;
    
    manager.unload_plugin(&plugin_id)
        .map_err(|e| {
            log::error!("[plugin:commands] Failed to unload plugin {}: {}", plugin_id, e);
            PluginCommandError::from(e.to_string())
        })?;
    
    log::info!("[plugin:commands] plugin_unload completed successfully for {}", plugin_id);
    Ok(())
}

/// Get all loaded plugins
#[tauri::command]
pub fn plugin_list_loaded() -> Result<Vec<PluginMetadata>, PluginCommandError> {
    log::info!("[plugin:commands] plugin_list_loaded called");
    
    let manager_guard = PLUGIN_MANAGER.lock().map_err(|e| {
        log::error!("[plugin:commands] Failed to lock PLUGIN_MANAGER: {}", e);
        PluginCommandError::from(e.to_string())
    })?;
    
    let manager = manager_guard.as_ref()
        .ok_or_else(|| {
            log::error!("[plugin:commands] Plugin system not initialized");
            PluginCommandError::from("Plugin system not initialized".to_string())
        })?;
    
    let plugins: Vec<PluginMetadata> = manager.get_loaded_plugins()
        .into_iter()
        .cloned()
        .collect();
    
    log::info!("[plugin:commands] plugin_list_loaded returning {} loaded plugins", plugins.len());
    
    for plugin in &plugins {
        log::debug!("[plugin:commands]   - Loaded plugin: {} ({}) enabled={}", plugin.name, plugin.id, plugin.enabled);
    }
    
    Ok(plugins)
}

/// Get plugin info
#[tauri::command]
pub fn plugin_get_info(plugin_id: String) -> Result<PluginMetadata, PluginCommandError> {
    log::info!("[plugin:commands] plugin_get_info called with plugin_id: {}", plugin_id);
    
    let manager_guard = PLUGIN_MANAGER.lock().map_err(|e| {
        log::error!("[plugin:commands] Failed to lock PLUGIN_MANAGER: {}", e);
        PluginCommandError::from(e.to_string())
    })?;
    
    let manager = manager_guard.as_ref()
        .ok_or_else(|| {
            log::error!("[plugin:commands] Plugin system not initialized");
            PluginCommandError::from("Plugin system not initialized".to_string())
        })?;
    
    let metadata = manager.get_plugin(&plugin_id)
        .cloned()
        .ok_or_else(|| {
            log::error!("[plugin:commands] Plugin '{}' not found", plugin_id);
            PluginCommandError::from(format!("Plugin '{}' not found", plugin_id))
        })?;
    
    log::debug!("[plugin:commands] plugin_get_info returning: {} v{}", metadata.name, metadata.version);
    Ok(metadata)
}

/// Execute a plugin capability
#[tauri::command]
pub fn plugin_execute(
    plugin_id: String,
    capability_id: String,
    input: Option<String>,
) -> Result<PluginExecutionResult, PluginCommandError> {
    let input_str = input.as_deref().unwrap_or_default();
    log::info!("[plugin:commands] plugin_execute called: plugin_id={}, capability_id={}, input_len={}",
        plugin_id, capability_id, input_str.len());
    log::debug!("[plugin:commands] plugin_execute input: {}", input_str);
    
    let mut manager_guard = PLUGIN_MANAGER.lock().map_err(|e| {
        log::error!("[plugin:commands] Failed to lock PLUGIN_MANAGER: {}", e);
        PluginCommandError::from(e.to_string())
    })?;
    
    let manager = manager_guard.as_mut()
        .ok_or_else(|| {
            log::error!("[plugin:commands] Plugin system not initialized");
            PluginCommandError::from("Plugin system not initialized".to_string())
        })?;
    
    let input = input.unwrap_or_default();
    
    match manager.execute_capability(&plugin_id, &capability_id, &input) {
        Ok(output) => {
            log::info!("[plugin:commands] plugin_execute completed successfully for {}.{}", plugin_id, capability_id);
            log::debug!("[plugin:commands] plugin_execute output: {}", output);
            Ok(PluginExecutionResult {
                success: true,
                output: Some(output),
                error: None,
            })
        }
        Err(e) => {
            log::error!("[plugin:commands] plugin_execute failed for {}.{}: {}", plugin_id, capability_id, e);
            Ok(PluginExecutionResult {
                success: false,
                output: None,
                error: Some(e.to_string()),
            })
        }
    }
}

/// Enable/disable a plugin
#[tauri::command]
pub fn plugin_set_enabled(
    plugin_id: String,
    enabled: bool,
) -> Result<(), PluginCommandError> {
    log::info!("[plugin:commands] plugin_set_enabled called: plugin_id={}, enabled={}", plugin_id, enabled);
    
    let mut manager_guard = PLUGIN_MANAGER.lock().map_err(|e| {
        log::error!("[plugin:commands] Failed to lock PLUGIN_MANAGER: {}", e);
        PluginCommandError::from(e.to_string())
    })?;
    
    let manager = manager_guard.as_mut()
        .ok_or_else(|| {
            log::error!("[plugin:commands] Plugin system not initialized");
            PluginCommandError::from("Plugin system not initialized".to_string())
        })?;
    
    manager.set_plugin_enabled(&plugin_id, enabled)
        .map_err(|e| {
            log::error!("[plugin:commands] Failed to set plugin enabled status for {}: {}", plugin_id, e);
            PluginCommandError::from(e.to_string())
        })?;
    
    log::info!("[plugin:commands] plugin_set_enabled completed successfully for {}", plugin_id);
    Ok(())
}

/// Get all capabilities from all loaded plugins
#[tauri::command]
pub fn plugin_get_capabilities() -> Result<Vec<PluginCapabilityInfo>, PluginCommandError> {
    log::info!("[plugin:commands] plugin_get_capabilities called");
    
    let manager_guard = PLUGIN_MANAGER.lock().map_err(|e| {
        log::error!("[plugin:commands] Failed to lock PLUGIN_MANAGER: {}", e);
        PluginCommandError::from(e.to_string())
    })?;
    
    let manager = manager_guard.as_ref()
        .ok_or_else(|| {
            log::error!("[plugin:commands] Plugin system not initialized");
            PluginCommandError::from("Plugin system not initialized".to_string())
        })?;
    
    let capabilities = manager.get_all_capabilities()
        .into_iter()
        .map(|(cap, plugin_id)| PluginCapabilityInfo {
            capability: cap,
            plugin_id,
        })
        .collect();
    
    log::info!("[plugin:commands] plugin_get_capabilities returning capabilities");
    Ok(capabilities)
}

/// Broadcast an editor event to all plugins
#[tauri::command]
pub fn plugin_broadcast_event(event_type: String, event_data: Value) -> Result<(), PluginCommandError> {
    log::info!("[plugin:commands] plugin_broadcast_event called: event_type={}", event_type);
    log::debug!("[plugin:commands] plugin_broadcast_event data: {}", event_data);
    
    let mut manager_guard = PLUGIN_MANAGER.lock().map_err(|e| {
        log::error!("[plugin:commands] Failed to lock PLUGIN_MANAGER: {}", e);
        PluginCommandError::from(e.to_string())
    })?;
    
    let manager = manager_guard.as_mut()
        .ok_or_else(|| {
            log::error!("[plugin:commands] Plugin system not initialized");
            PluginCommandError::from("Plugin system not initialized".to_string())
        })?;
    
    // Parse event from JSON
    let event = parse_event(&event_type, event_data)
        .map_err(|e| {
            log::error!("[plugin:commands] Failed to parse event '{}': {}", event_type, e);
            PluginCommandError::from(format!("Failed to parse event: {}", e))
        })?;
    
    manager.broadcast_event(&event);
    
    log::info!("[plugin:commands] plugin_broadcast_event completed for {}", event_type);
    Ok(())
}

/// Get plugin directory path
#[tauri::command]
pub fn plugin_get_directory() -> Result<String, PluginCommandError> {
    log::info!("[plugin:commands] plugin_get_directory called");
    
    let manager_guard = PLUGIN_MANAGER.lock().map_err(|e| {
        log::error!("[plugin:commands] Failed to lock PLUGIN_MANAGER: {}", e);
        PluginCommandError::from(e.to_string())
    })?;
    
    let manager = manager_guard.as_ref()
        .ok_or_else(|| {
            log::error!("[plugin:commands] Plugin system not initialized");
            PluginCommandError::from("Plugin system not initialized".to_string())
        })?;
    
    let path = manager.plugin_dir.to_string_lossy().to_string();
    log::info!("[plugin:commands] plugin_get_directory returning: {}", path);
    Ok(path)
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
    log::debug!("[plugin:commands] parse_event called: event_type={}", event_type);
    
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
    
    log::debug!("[plugin:commands] parse_event completed: {:?}", event);
    Ok(event)
}
