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
        Self {
            active_file: None,
            settings: std::collections::HashMap::new(),
        }
    }
    
    /// Get the global bridge instance
    pub fn global() -> Arc<Mutex<PluginBridge>> {
        BRIDGE.clone()
    }
    
    /// Set active file
    pub fn set_active_file(&mut self, path: String) {
        self.active_file = Some(path);
    }
    
    /// Get active file
    pub fn get_active_file(&self) -> Option<&str> {
        self.active_file.as_deref()
    }
    
    /// Handle bridge request
    pub fn handle_request(&self, request: &BridgeRequest) -> BridgeResponse {
        match request {
            BridgeRequest::GetFileContent { path } => {
                if let Some(manager) = FileEngine::get_editable(path) {
                    match manager.read() {
                        Ok(guard) => {
                            let content = guard.get_text(0, guard.effective_size() as usize);
                            BridgeResponse::FileContent {
                                content,
                                path: path.clone(),
                            }
                        }
                        Err(e) => BridgeResponse::Error {
                            code: "lock_error".to_string(),
                            message: e.to_string(),
                        }
                    }
                } else {
                    BridgeResponse::Error {
                        code: "file_not_open".to_string(),
                        message: format!("File not open: {}", path),
                    }
                }
            }
            
            BridgeRequest::SetFileContent { path, content } => {
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
                            BridgeResponse::FileContent {
                                content: content.clone(),
                                path: path.clone(),
                            }
                        }
                        Err(e) => BridgeResponse::Error {
                            code: "lock_error".to_string(),
                            message: e.to_string(),
                        }
                    }
                } else {
                    BridgeResponse::Error {
                        code: "file_not_open".to_string(),
                        message: format!("File not open: {}", path),
                    }
                }
            }
            
            BridgeRequest::GetFileInfo { path } => {
                if let Some(info) = FileEngine::get_file_info(path) {
                    BridgeResponse::FileInfo {
                        size: info.size,
                        editable: info.editable,
                    }
                } else {
                    BridgeResponse::Error {
                        code: "file_not_found".to_string(),
                        message: format!("File not found: {}", path),
                    }
                }
            }
            
            BridgeRequest::GetSelection { path } => {
                // For now, return full file range
                if let Some(info) = FileEngine::get_file_info(path) {
                    BridgeResponse::Selection {
                        start: 0,
                        end: info.size as usize,
                    }
                } else {
                    BridgeResponse::Error {
                        code: "file_not_found".to_string(),
                        message: format!("File not found: {}", path),
                    }
                }
            }
            
            BridgeRequest::ShowNotification { title, message } => {
                // In a real implementation, this would show a UI notification
                log::info!("[{}] {}", title, message);
                BridgeResponse::NotificationSent
            }
            
            BridgeRequest::GetSetting { key } => {
                BridgeResponse::SettingValue {
                    value: self.settings.get(key).cloned(),
                }
            }
            
            BridgeRequest::SetSetting { key, value } => {
                // Settings are per-plugin and should be handled differently
                BridgeResponse::SettingSaved
            }
            
            BridgeRequest::ExecuteCommand { command, args } => {
                // Execute editor command
                log::info!("Plugin executing command: {} {:?}", command, args);
                BridgeResponse::CommandExecuted {
                    result: "ok".to_string(),
                }
            }
            
            _ => BridgeResponse::Error {
                code: "not_implemented".to_string(),
                message: "Request type not implemented".to_string(),
            }
        }
    }
    
    /// Get a setting value
    pub fn get_setting(&self, key: &str) -> Option<&str> {
        self.settings.get(key).map(|s| s.as_str())
    }
    
    /// Set a setting value
    pub fn set_setting(&mut self, key: String, value: String) {
        self.settings.insert(key, value);
    }
}

/// Host functions for WASM plugins
/// These are called by WASM code to interact with the editor

/// Log a message from a plugin
pub fn host_log(plugin_id: &str, message: &str) {
    log::info!("[Plugin {}] {}", plugin_id, message);
}

/// Get file content for a plugin
pub fn host_get_file_content(plugin_id: &str, path: &str) -> Result<String, String> {
    let request = BridgeRequest::GetFileContent {
        path: path.to_string(),
    };
    
    let bridge = PluginBridge::global();
    let bridge_guard = bridge.lock().map_err(|e| e.to_string())?;
    
    match bridge_guard.handle_request(&request) {
        BridgeResponse::FileContent { content, .. } => Ok(content),
        BridgeResponse::Error { message, .. } => Err(message),
        _ => Err("Unexpected response".to_string()),
    }
}

/// Set file content from a plugin
pub fn host_set_file_content(plugin_id: &str, path: &str, content: &str) -> Result<(), String> {
    let request = BridgeRequest::SetFileContent {
        path: path.to_string(),
        content: content.to_string(),
    };
    
    let bridge = PluginBridge::global();
    let bridge_guard = bridge.lock().map_err(|e| e.to_string())?;
    
    match bridge_guard.handle_request(&request) {
        BridgeResponse::FileContent { .. } => Ok(()),
        BridgeResponse::Error { message, .. } => Err(message),
        _ => Err("Unexpected response".to_string()),
    }
}

/// Show notification from a plugin
pub fn host_show_notification(plugin_id: &str, title: &str, message: &str) -> Result<(), String> {
    let request = BridgeRequest::ShowNotification {
        title: title.to_string(),
        message: message.to_string(),
    };
    
    let bridge = PluginBridge::global();
    let bridge_guard = bridge.lock().map_err(|e| e.to_string())?;
    
    match bridge_guard.handle_request(&request) {
        BridgeResponse::NotificationSent => Ok(()),
        BridgeResponse::Error { message, .. } => Err(message),
        _ => Err("Unexpected response".to_string()),
    }
}