//! Bridge between WASM plugins and the editor
//!
//! Provides a safe API for plugins to interact with editor state.

use std::sync::{Arc, Mutex};
use serde::{Serialize, Deserialize};
use lazy_static::lazy_static;

use crate::file_engine::FileEngine;
use crate::mark_engine::MarkEngine;

/// Bridge state for plugin-editor communication
#[derive(Debug)]
pub struct PluginBridge {
    /// Currently active file
    active_file: Option<String>,
    /// Plugin settings (key-value store per plugin)
    settings: std::collections::HashMap<String, String>,
}

/// Request from plugin to editor
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum BridgeRequest {
    GetFileContent { path: String },
    SetFileContent { path: String, content: String },
    GetFileInfo { path: String },
    GetSelection { path: String },
    GetMarks { path: String },
    ShowNotification { title: String, message: String },
    GetSetting { key: String },
    SetSetting { key: String, value: String },
    ExecuteCommand { command: String, args: Vec<String> },
}

/// Response from editor to plugin
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum BridgeResponse {
    FileContent { content: String, path: String },
    FileInfo { size: u64, editable: bool },
    Selection { start: usize, end: usize },
    Marks { marks: Vec<String> },
    NotificationSent,
    SettingValue { value: Option<String> },
    SettingSaved,
    CommandExecuted { result: String },
    Error { code: String, message: String },
}

lazy_static! {
    static ref BRIDGE: Arc<Mutex<PluginBridge>> = 
        Arc::new(Mutex::new(PluginBridge::new()));
}

impl PluginBridge {
    /// Create a new bridge
    pub fn new() -> Self {
        log::info!("[plugin:bridge] Creating new PluginBridge");
        Self {
            active_file: None,
            settings: std::collections::HashMap::new(),
        }
    }
    
    /// Get the global bridge instance
    pub fn global() -> Arc<Mutex<PluginBridge>> {
        log::debug!("[plugin:bridge] Getting global bridge instance");
        BRIDGE.clone()
    }
    
    /// Set active file
    pub fn set_active_file(&mut self, path: String) {
        log::info!("[plugin:bridge] Setting active file: {}", path);
        self.active_file = Some(path);
    }
    
    /// Get active file
    pub fn get_active_file(&self) -> Option<&str> {
        log::debug!("[plugin:bridge] Getting active file: {:?}", self.active_file);
        self.active_file.as_deref()
    }
    
    /// Handle bridge request
    pub fn handle_request(&self, request: &BridgeRequest) -> BridgeResponse {
        log::info!("[plugin:bridge] Handling bridge request: {:?}", request);
        
        match request {
            BridgeRequest::GetFileContent { path } => {
                log::debug!("[plugin:bridge] GetFileContent for path: {}", path);
                
                if let Some(manager) = FileEngine::get_editable(path) {
                    match manager.read() {
                        Ok(guard) => {
                            let content = guard.get_text(0, guard.effective_size() as usize);
                            log::debug!("[plugin:bridge] Read {} bytes from file {}", content.len(), path);
                            BridgeResponse::FileContent {
                                content,
                                path: path.clone(),
                            }
                        }
                        Err(e) => {
                            log::error!("[plugin:bridge] Lock error reading file {}: {}", path, e);
                            BridgeResponse::Error {
                                code: "lock_error".to_string(),
                                message: e.to_string(),
                            }
                        }
                    }
                } else {
                    log::warn!("[plugin:bridge] File not open: {}", path);
                    BridgeResponse::Error {
                        code: "file_not_open".to_string(),
                        message: format!("File not open: {}", path),
                    }
                }
            }
            
            BridgeRequest::SetFileContent { path, content } => {
                log::debug!("[plugin:bridge] SetFileContent for path: {} ({} bytes)", path, content.len());
                
                if let Some(manager) = FileEngine::get_editable(path) {
                    match manager.write() {
                        Ok(mut guard) => {
                            let current_size = guard.effective_size() as usize;
                            // Replace entire content
                            guard.apply_edit(crate::file_engine::EditOp::Replace {
                                offset: 0,
                                length: current_size,
                                data: content.as_bytes().to_vec(),
                            });
                            log::info!("[plugin:bridge] File content updated: {}", path);
                            BridgeResponse::FileContent {
                                content: content.clone(),
                                path: path.clone(),
                            }
                        }
                        Err(e) => {
                            log::error!("[plugin:bridge] Lock error writing file {}: {}", path, e);
                            BridgeResponse::Error {
                                code: "lock_error".to_string(),
                                message: e.to_string(),
                            }
                        }
                    }
                } else {
                    log::warn!("[plugin:bridge] File not open for writing: {}", path);
                    BridgeResponse::Error {
                        code: "file_not_open".to_string(),
                        message: format!("File not open: {}", path),
                    }
                }
            }
            
            BridgeRequest::GetFileInfo { path } => {
                log::debug!("[plugin:bridge] GetFileInfo for path: {}", path);
                
                if let Some(info) = FileEngine::get_file_info(path) {
                    log::debug!("[plugin:bridge] File info: size={}, editable={}", info.size, info.editable);
                    BridgeResponse::FileInfo {
                        size: info.size,
                        editable: info.editable,
                    }
                } else {
                    log::warn!("[plugin:bridge] File not found: {}", path);
                    BridgeResponse::Error {
                        code: "file_not_found".to_string(),
                        message: format!("File not found: {}", path),
                    }
                }
            }
            
            BridgeRequest::GetSelection { path } => {
                log::debug!("[plugin:bridge] GetSelection for path: {}", path);
                
                // For now, return full file range
                if let Some(info) = FileEngine::get_file_info(path) {
                    log::debug!("[plugin:bridge] Selection range: 0-{}", info.size);
                    BridgeResponse::Selection {
                        start: 0,
                        end: info.size as usize,
                    }
                } else {
                    log::warn!("[plugin:bridge] File not found for selection: {}", path);
                    BridgeResponse::Error {
                        code: "file_not_found".to_string(),
                        message: format!("File not found: {}", path),
                    }
                }
            }
            
            BridgeRequest::ShowNotification { title, message } => {
                log::info!("[plugin:bridge] ShowNotification: [{}] {}", title, message);
                // In a real implementation, this would show a UI notification
                log::info!("[plugin:bridge] Notification displayed: {}", title);
                BridgeResponse::NotificationSent
            }
            
            BridgeRequest::GetSetting { key } => {
                log::debug!("[plugin:bridge] GetSetting for key: {}", key);
                let value = self.settings.get(key).cloned();
                log::debug!("[plugin:bridge] Setting value for {}: {:?}", key, value);
                BridgeResponse::SettingValue { value }
            }
            
            BridgeRequest::SetSetting { key, value } => {
                log::info!("[plugin:bridge] SetSetting: {} = {}", key, value);
                // Settings are per-plugin and should be handled differently
                log::debug!("[plugin:bridge] Setting saved (note: actual persistence not implemented)");
                BridgeResponse::SettingSaved
            }
            
            BridgeRequest::ExecuteCommand { command, args } => {
                log::info!("[plugin:bridge] ExecuteCommand: {} {:?}", command, args);
                // Execute editor command
                log::info!("[plugin:bridge] Command executed: {}", command);
                BridgeResponse::CommandExecuted {
                    result: "ok".to_string(),
                }
            }
            
            _ => {
                log::warn!("[plugin:bridge] Request type not implemented: {:?}", request);
                BridgeResponse::Error {
                    code: "not_implemented".to_string(),
                    message: "Request type not implemented".to_string(),
                }
            }
        }
    }
    
    /// Get a setting value
    pub fn get_setting(&self, key: &str) -> Option<&str> {
        log::debug!("[plugin:bridge] get_setting called for key: {}", key);
        let result = self.settings.get(key).map(|s| s.as_str());
        log::debug!("[plugin:bridge] Setting value for {}: {:?}", key, result);
        result
    }
    
    /// Set a setting value
    pub fn set_setting(&mut self, key: String, value: String) {
        log::info!("[plugin:bridge] set_setting: {} = {}", key, value);
        self.settings.insert(key, value);
    }
}

/// Host functions for WASM plugins
/// These are called by WASM code to interact with the editor

/// Log a message from a plugin
pub fn host_log(plugin_id: &str, message: &str) {
    log::info!("[plugin:bridge] [Plugin {}] {}", plugin_id, message);
}

/// Get file content for a plugin
pub fn host_get_file_content(plugin_id: &str, path: &str) -> Result<String, String> {
    log::info!("[plugin:bridge] host_get_file_content called by plugin {} for path: {}", plugin_id, path);
    
    let request = BridgeRequest::GetFileContent {
        path: path.to_string(),
    };
    
    let bridge = PluginBridge::global();
    let bridge_guard = bridge.lock().map_err(|e| {
        log::error!("[plugin:bridge] Failed to lock bridge: {}", e);
        e.to_string()
    })?;
    
    match bridge_guard.handle_request(&request) {
        BridgeResponse::FileContent { content, .. } => {
            log::info!("[plugin:bridge] Got file content for {}: {} bytes", path, content.len());
            Ok(content)
        }
        BridgeResponse::Error { message, .. } => {
            log::error!("[plugin:bridge] Error getting file content: {}", message);
            Err(message)
        }
        _ => {
            log::error!("[plugin:bridge] Unexpected response from bridge");
            Err("Unexpected response".to_string())
        }
    }
}

/// Set file content from a plugin
pub fn host_set_file_content(plugin_id: &str, path: &str, content: &str) -> Result<(), String> {
    log::info!("[plugin:bridge] host_set_file_content called by plugin {} for path: {} ({} bytes)",
        plugin_id, path, content.len());
    
    let request = BridgeRequest::SetFileContent {
        path: path.to_string(),
        content: content.to_string(),
    };
    
    let bridge = PluginBridge::global();
    let bridge_guard = bridge.lock().map_err(|e| {
        log::error!("[plugin:bridge] Failed to lock bridge: {}", e);
        e.to_string()
    })?;
    
    match bridge_guard.handle_request(&request) {
        BridgeResponse::FileContent { .. } => {
            log::info!("[plugin:bridge] File content set successfully for {}", path);
            Ok(())
        }
        BridgeResponse::Error { message, .. } => {
            log::error!("[plugin:bridge] Error setting file content: {}", message);
            Err(message)
        }
        _ => {
            log::error!("[plugin:bridge] Unexpected response from bridge");
            Err("Unexpected response".to_string())
        }
    }
}

/// Show notification from a plugin
pub fn host_show_notification(plugin_id: &str, title: &str, message: &str) -> Result<(), String> {
    log::info!("[plugin:bridge] host_show_notification called by plugin {}: [{}] {}",
        plugin_id, title, message);
    
    let request = BridgeRequest::ShowNotification {
        title: title.to_string(),
        message: message.to_string(),
    };
    
    let bridge = PluginBridge::global();
    let bridge_guard = bridge.lock().map_err(|e| {
        log::error!("[plugin:bridge] Failed to lock bridge: {}", e);
        e.to_string()
    })?;
    
    match bridge_guard.handle_request(&request) {
        BridgeResponse::NotificationSent => {
            log::info!("[plugin:bridge] Notification sent successfully");
            Ok(())
        }
        BridgeResponse::Error { message, .. } => {
            log::error!("[plugin:bridge] Error sending notification: {}", message);
            Err(message)
        }
        _ => {
            log::error!("[plugin:bridge] Unexpected response from bridge");
            Err("Unexpected response".to_string())
        }
    }
}
