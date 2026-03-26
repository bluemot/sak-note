//! Plugin API Definitions
//!
//! Defines the interface that WASM plugins must implement
//! and the manifest format for plugin metadata.

use serde::{Serialize, Deserialize};

/// Plugin manifest (plugin.json)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub entry_point: String,        // Path to WASM file
    pub capabilities: Vec<PluginCapability>,
    pub ui_components: Vec<PluginUiComponent>,
    pub permissions: Vec<PluginPermission>,
    pub min_editor_version: Option<String>,
    pub max_editor_version: Option<String>,
}

/// Plugin capability definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginCapability {
    pub id: String,
    pub name: String,
    pub description: String,
    pub trigger: CapabilityTrigger,  // When/how this capability is triggered
    pub input_schema: serde_json::Value,   // JSON Schema for input
    pub output_schema: serde_json::Value,  // JSON Schema for output
}

/// How a capability is triggered
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum CapabilityTrigger {
    Command { command: String },           // Via command palette
    Keybinding { keys: Vec<String> },      // Keyboard shortcut
    Event { event: String },               // On specific event
    ContextMenu { selector: String },     // Right-click context menu
    Auto { on: String },                  // Automatic on condition
}

/// UI component definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginUiComponent {
    pub id: String,
    pub type_: UiComponentType,
    pub title: String,
    pub icon: Option<String>,
    pub position: UiPosition,
    pub width: Option<u32>,
    pub height: Option<u32>,
}

/// UI component types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UiComponentType {
    Panel,        // Side panel
    Tab,          // Editor tab
    Modal,        // Modal dialog
    StatusBar,    // Status bar item
    Toolbar,      // Toolbar button
    ContextMenu,  // Context menu entry
    Decoration,   // Editor decoration (gutter, inline, etc.)
}

/// UI position
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UiPosition {
    Left,
    Right,
    Bottom,
    Top,
    Center,
    Floating,
}

/// Plugin permissions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PluginPermission {
    FileRead,           // Read files
    FileWrite,          // Write files
    FileCreate,         // Create new files
    Network,            // Make network requests
    Shell,              // Execute shell commands
    Clipboard,          // Access clipboard
    EditorState,        // Read editor state
    EditorModify,       // Modify editor content
    Notifications,      // Show notifications
    Settings,           // Read/write settings
}

/// Plugin trait - implemented by loaded WASM plugins
pub trait Plugin: Send + Sync {
    /// Get plugin manifest
    fn manifest(&self) -> &PluginManifest;
    
    /// Initialize the plugin
    fn initialize(&mut self) -> Result<(), PluginError>;
    
    /// Execute a capability
    fn execute(&mut self, capability_id: &str, input: &str) -> Result<String, PluginError>;
    
    /// Handle editor event
    fn on_event(&mut self, event: &EditorEvent) -> Result<(), PluginError>;
    
    /// Shutdown the plugin
    fn shutdown(&mut self) -> Result<(), PluginError>;
}

/// Plugin error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginError {
    pub code: String,
    pub message: String,
    pub details: Option<String>,
}

impl PluginError {
    pub fn new(code: &str, message: &str) -> Self {
        Self {
            code: code.to_string(),
            message: message.to_string(),
            details: None,
        }
    }
    
    pub fn with_details(mut self, details: &str) -> Self {
        self.details = Some(details.to_string());
        self
    }
}

impl std::fmt::Display for PluginError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}", self.code, self.message)
    }
}

/// Editor events that plugins can listen to
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EditorEvent {
    FileOpened { path: String },
    FileClosed { path: String },
    FileSaved { path: String },
    SelectionChanged { path: String, start: usize, end: usize },
    ContentChanged { path: String },
    CursorMoved { path: String, offset: usize },
    MarkCreated { path: String, id: String },
    MarkDeleted { path: String, id: String },
    EditorFocused { path: String },
    EditorBlurred { path: String },
    Startup,
    Shutdown,
    Custom { name: String, data: serde_json::Value },
}

/// Plugin metadata for display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMetadata {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub loaded: bool,
    pub enabled: bool,
}