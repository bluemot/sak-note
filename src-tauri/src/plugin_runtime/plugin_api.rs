//! Plugin API Types
//!
//! Defines the core types and interfaces for WASM plugins.

use serde::{Serialize, Deserialize};
use std::collections::HashMap;

/// Plugin manifest - loaded from plugin.json
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub entry_point: String,
    pub capabilities: Vec<PluginCapability>,
    pub permissions: Vec<PluginPermission>,
    #[serde(default)]
    pub ui_components: Vec<PluginUiComponent>,
}

/// Plugin metadata - runtime information about a loaded plugin
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMetadata {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    #[serde(skip)]
    pub loaded: bool,
    #[serde(skip)]
    pub enabled: bool,
}

/// Plugin capability - a feature provided by the plugin
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginCapability {
    pub id: String,
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub input_schema: Option<String>,
    #[serde(default)]
    pub output_schema: Option<String>,
}

/// Plugin permission - resource access permissions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PluginPermission {
    FileRead,
    FileWrite,
    Network,
    Environment,
    Command,
}

/// Plugin UI component - UI extension point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginUiComponent {
    pub id: String,
    pub name: String,
    pub position: String, // e.g., "sidebar", "toolbar", "context-menu"
    #[serde(default)]
    pub config: HashMap<String, serde_json::Value>,
}

/// Plugin error
#[derive(Debug, Clone)]
pub struct PluginError {
    pub code: String,
    pub message: String,
}

impl PluginError {
    pub fn new(code: &str, message: &str) -> Self {
        Self {
            code: code.to_string(),
            message: message.to_string(),
        }
    }
}

impl std::fmt::Display for PluginError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}", self.code, self.message)
    }
}

impl std::error::Error for PluginError {}

/// Editor event types that plugins can listen to
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EditorEvent {
    FileOpened { path: String },
    FileClosed { path: String },
    FileSaved { path: String },
    ContentChanged { path: String },
    SelectionChanged { path: String, start: usize, end: usize },
    Startup,
    Shutdown,
    Custom { name: String, data: serde_json::Value },
}

/// Plugin trait - defines the interface for plugins
pub trait Plugin: Send + Sync {
    /// Get plugin manifest
    fn manifest(&self) -> &PluginManifest;
    
    /// Initialize the plugin
    fn initialize(&mut self) -> Result<(), PluginError>;
    
    /// Shutdown the plugin
    fn shutdown(&mut self) -> Result<(), PluginError>;
    
    /// Handle an editor event
    fn on_event(&mut self, event: &EditorEvent) -> Result<(), PluginError>;
    
    /// Execute a capability
    fn execute_capability(&mut self, capability_id: &str, input: &str) -> Result<String, PluginError>;
}

/// Plugin load status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginLoadStatus {
    pub plugin_id: String,
    pub success: bool,
    pub message: String,
}

/// Plugin execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginExecutionResult {
    pub success: bool,
    pub output: Option<String>,
    pub error: Option<String>,
}
