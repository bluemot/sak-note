//! Complete Tool Registry for ALL Modules
//!
//! This exposes EVERY module capability to LLM through JSON interface

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

// ... (existing code)

/// Register ALL module tools including UI and VFS
pub fn register_all_module_tools(registry: &mut ToolRegistry) {
    // Existing modules
    register_file_module_tools(registry);
    register_llm_module_tools(registry);
    register_sftp_module_tools(registry);
    register_semantic_tools(registry);
    register_mark_module_tools(registry);
    
    // NEW: UI Control tools
    register_ui_tools(registry);
    
    // NEW: VFS tools
    register_vfs_tools(registry);
    
    // NEW: Line Operations tools
    register_line_operation_tools(registry);
}

/// UI Control Tools - LLM can control the editor interface
fn register_ui_tools(registry: &mut ToolRegistry) {
    // ui::open_file - Open file in editor (shows in UI)
    registry.register(Tool {
        name: "ui::open_file".to_string(),
        description: "Open a file in the editor UI. This will display the file content in the editor pane.".to_string(),
        parameters: ToolParameters {
            type_: "object".to_string(),
            properties: HashMap::from([
                ("path".to_string(), ToolProperty {
                    type_: "string".to_string(),
                    description: "Absolute path to file".to_string(),
                    enum_values: None,
                }),
                ("focus".to_string(), ToolProperty {
                    type_: "boolean".to_string(),
                    description: "Whether to focus the editor".to_string(),
                    enum_values: None,
                }),
            ]),
            required: vec!["path".to_string()],
        },
        returns: ToolReturns {
            description: "Success status and file info".to_string(),
            schema: serde_json::json!({"type": "object"}),
        },
        examples: vec![
            ToolExample {
                description: "Open main.rs in editor".to_string(),
                request: serde_json::json!({"path": "/project/src/main.rs", "focus": true}),
                response: serde_json::json!({"success": true, "file": "main.rs", "lines": 150}),
            },
        ],
    });

    // ui::navigate_to_line - Jump to specific line
    registry.register(Tool {
        name: "ui::navigate_to_line".to_string(),
        description: "Navigate editor to specific line number. Scrolls view and positions cursor.".to_string(),
        parameters: ToolParameters {
            type_: "object".to_string(),
            properties: HashMap::from([
                ("path".to_string(), ToolProperty {
                    type_: "string".to_string(),
                    description: "File path (must be open)".to_string(),
                    enum_values: None,
                }),
                ("line".to_string(), ToolProperty {
                    type_: "number".to_string(),
                    description: "Line number (1-based)".to_string(),
                    enum_values: None,
                }),
                ("column".to_string(), ToolProperty {
                    type_: "number".to_string(),
                    description: "Column number (optional, default 1)".to_string(),
                    enum_values: None,
                }),
                ("select".to_string(), ToolProperty {
                    type_: "boolean".to_string(),
                    description: "Whether to select the line".to_string(),
                    enum_values: None,
                }),
            ]),
            required: vec!["path".to_string(), "line".to_string()],
        },
        returns: ToolReturns {
            description: "Navigation result".to_string(),
            schema: serde_json::json!({"type": "object"}),
        },
        examples: vec![
            ToolExample {
                description: "Jump to line 42 and select it".to_string(),
                request: serde_json::json!({"path": "/project/main.rs", "line": 42, "select": true}),
                response: serde_json::json!({"success": true, "position": {"line": 42, "column": 1}}),
            },
        ],
    });

    // ui::get_open_files - List currently open files
    registry.register(Tool {
        name: "ui::get_open_files".to_string(),
        description: "Get list of currently open files in the editor.".to_string(),
        parameters: ToolParameters {
            type_: "object".to_string(),
            properties: HashMap::new(),
            required: vec![],
        },
        returns: ToolReturns {
            description: "List of open files with metadata".to_string(),
            schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "files": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "properties": {
                                "path": {"type": "string"},
                                "name": {"type": "string"},
                                "active": {"type": "boolean"},
                            }
                        }
                    }
                }
            }),
        },
        examples: vec![],
    });

    // ui::close_file - Close a file
    registry.register(Tool {
        name: "ui::close_file".to_string(),
        description: "Close a file in the editor.".to_string(),
        parameters: ToolParameters {
            type_: "object".to_string(),
            properties: HashMap::from([
                ("path".to_string(), ToolProperty {
                    type_: "string".to_string(),
                    description: "File path to close".to_string(),
                    enum_values: None,
                }),
            ]),
            required: vec!["path".to_string()],
        },
        returns: ToolReturns {
            description: "Success status".to_string(),
            schema: serde_json::json!({"type": "boolean"}),
        },
        examples: vec![],
    });

    // ui::show_panel - Show a sidebar panel
    registry.register(Tool {
        name: "ui::show_panel".to_string(),
        description: "Show a specific sidebar panel (info, semantic, chat, marks)".to_string(),
        parameters: ToolParameters {
            type_: "object".to_string(),
            properties: HashMap::from([
                ("panel".to_string(), ToolProperty {
                    type_: "string".to_string(),
                    description: "Panel name: info, semantic, chat, marks".to_string(),
                    enum_values: Some(vec![
                        "info".to_string(),
                        "semantic".to_string(),
                        "chat".to_string(),
                        "marks".to_string(),
                    ]),
                }),
            ]),
            required: vec!["panel".to_string()],
        },
        returns: ToolReturns {
            description: "Success status".to_string(),
            schema: serde_json::json!({"type": "boolean"}),
        },
        examples: vec![
            ToolExample {
                description: "Show semantic analysis panel".to_string(),
                request: serde_json::json!({"panel": "semantic"}),
                response: serde_json::json!({"success": true}),
            },
        ],
    });
}

/// VFS Tools - Direct VFS operations
fn register_vfs_tools(registry: &mut ToolRegistry) {
    // vfs::list_local - List local directory via VFS
    registry.register(Tool {
        name: "vfs::list_local".to_string(),
        description: "List directory contents using VFS.".to_string(),
        parameters: ToolParameters {
            type_: "object".to_string(),
            properties: HashMap::from([
                ("path".to_string(), ToolProperty {
                    type_: "string".to_string(),
                    description: "Directory path".to_string(),
                    enum_values: None,
                }),
            ]),
            required: vec!["path".to_string()],
        },
        returns: ToolReturns {
            description: "Directory entries".to_string(),
            schema: serde_json::json!({
                "type": "array",
                "items": {
                    "type": "object",
                    "properties": {
                        "name": {"type": "string"},
                        "is_file": {"type": "boolean"},
                        "size": {"type": "number"},
                    }
                }
            }),
        },
        examples: vec![],
    });

    // vfs::stat - Get file metadata via VFS
    registry.register(Tool {
        name: "vfs::stat".to_string(),
        description: "Get file/directory metadata via VFS.".to_string(),
        parameters: ToolParameters {
            type_: "object".to_string(),
            properties: HashMap::from([
                ("path".to_string(), ToolProperty {
                    type_: "string".to_string(),
                    description: "File or directory path".to_string(),
                    enum_values: None,
                }),
            ]),
            required: vec!["path".to_string()],
        },
        returns: ToolReturns {
            description: "Metadata including size, modified time, etc.".to_string(),
            schema: serde_json::json!({"type": "object"}),
        },
        examples: vec![],
    });

    // vfs::get_handle - Get VFS handle info
    registry.register(Tool {
        name: "vfs::get_handle".to_string(),
        description: "Get information about an open VFS file handle.".to_string(),
        parameters: ToolParameters {
            type_: "object".to_string(),
            properties: HashMap::from([
                ("path".to_string(), ToolProperty {
                    type_: "string".to_string(),
                    description: "File path".to_string(),
                    enum_values: None,
                }),
            ]),
            required: vec!["path".to_string()],
        },
        returns: ToolReturns {
            description: "Handle status".to_string(),
            schema: serde_json::json!({"type": "object"}),
        },
        examples: vec![],
    });

    // vfs::list_open - List all open VFS handles
    registry.register(Tool {
        name: "vfs::list_open".to_string(),
        description: "List all currently open VFS file handles.".to_string(),
        parameters: ToolParameters {
            type_: "object".to_string(),
            properties: HashMap::new(),
            required: vec![],
        },
        returns: ToolReturns {
            description: "List of open files".to_string(),
            schema: serde_json::json!({
                "type": "array",
                "items": {"type": "string"}
            }),
        },
        examples: vec![],
    });
}

/// Editor State Tools
fn register_editor_tools(registry: &mut ToolRegistry) {
    // editor::get_state - Get editor state
    registry.register(Tool {
        name: "editor::get_state".to_string(),
        description: "Get current editor state including active file, cursor position, view mode.".to_string(),
        parameters: ToolParameters {
            type_: "object".to_string(),
            properties: HashMap::new(),
            required: vec![],
        },
        returns: ToolReturns {
            description: "Editor state".to_string(),
            schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "active_file": {"type": "string"},
                    "view_mode": {"type": "string", "enum": ["text", "hex"]},
                    "cursor": {"type": "object"},
                    "selection": {"type": "object"},
                }
            }),
        },
        examples: vec![],
    });

    // editor::set_view_mode - Switch text/hex view
    registry.register(Tool {
        name: "editor::set_view_mode".to_string(),
        description: "Switch editor view mode between text and hex.".to_string(),
        parameters: ToolParameters {
            type_: "object".to_string(),
            properties: HashMap::from([
                ("mode".to_string(), ToolProperty {
                    type_: "string".to_string(),
                    description: "View mode: text or hex".to_string(),
                    enum_values: Some(vec!["text".to_string(), "hex".to_string()]),
                }),
            ]),
            required: vec!["mode".to_string()],
        },
        returns: ToolReturns {
            description: "Success status".to_string(),
            schema: serde_json::json!({"type": "boolean"}),
        },
        examples: vec![],
    });

    // editor::search - Search in current file
    registry.register(Tool {
        name: "editor::search".to_string(),
        description: "Search for text in the current file.".to_string(),
        parameters: ToolParameters {
            type_: "object".to_string(),
            properties: HashMap::from([
                ("query".to_string(), ToolProperty {
                    type_: "string".to_string(),
                    description: "Search query".to_string(),
                    enum_values: None,
                }),
                ("case_sensitive".to_string(), ToolProperty {
                    type_: "boolean".to_string(),
                    description: "Case sensitive search".to_string(),
                    enum_values: None,
                }),
            ]),
            required: vec!["query".to_string()],
        },
        returns: ToolReturns {
            description: "Search results".to_string(),
            schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "matches": {"type": "array"},
                    "count": {"type": "number"},
                }
            }),
        },
        examples: vec![],
    });
}

/// Line Operation Tools - LLM can manipulate text lines
fn register_line_operation_tools(registry: &mut ToolRegistry) {
    // edit::duplicate_line - Duplicate current line
    registry.register(Tool {
        name: "edit::duplicate_line".to_string(),
        description: "Duplicate the current line. Inserts a copy of the line below it.".to_string(),
        parameters: ToolParameters {
            type_: "object".to_string(),
            properties: HashMap::from([
                ("content".to_string(), ToolProperty {
                    type_: "string".to_string(),
                    description: "Full file content".to_string(),
                    enum_values: None,
                }),
                ("line_number".to_string(), ToolProperty {
                    type_: "number".to_string(),
                    description: "Line number to duplicate (1-based)".to_string(),
                    enum_values: None,
                }),
            ]),
            required: vec!["content".to_string(), "line_number".to_string()],
        },
        returns: ToolReturns {
            description: "Updated content with duplicated line".to_string(),
            schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "content": {"type": "string"},
                    "new_line_number": {"type": "number"},
                }
            }),
        },
        examples: vec![
            ToolExample {
                description: "Duplicate line 5".to_string(),
                request: serde_json::json!({"content": "line1\nline2\nline3", "line_number": 2}),
                response: serde_json::json!({"content": "line1\nline2\nline2\nline3", "new_line_number": 3}),
            },
        ],
    });

    // edit::move_line_up - Move line up
    registry.register(Tool {
        name: "edit::move_line_up".to_string(),
        description: "Move the current line up by one position.".to_string(),
        parameters: ToolParameters {
            type_: "object".to_string(),
            properties: HashMap::from([
                ("content".to_string(), ToolProperty {
                    type_: "string".to_string(),
                    description: "Full file content".to_string(),
                    enum_values: None,
                }),
                ("line_number".to_string(), ToolProperty {
                    type_: "number".to_string(),
                    description: "Line number to move (1-based)".to_string(),
                    enum_values: None,
                }),
            ]),
            required: vec!["content".to_string(), "line_number".to_string()],
        },
        returns: ToolReturns {
            description: "Updated content with line moved".to_string(),
            schema: serde_json::json!({"type": "object"}),
        },
        examples: vec![],
    });

    // edit::move_line_down - Move line down
    registry.register(Tool {
        name: "edit::move_line_down".to_string(),
        description: "Move the current line down by one position.".to_string(),
        parameters: ToolParameters {
            type_: "object".to_string(),
            properties: HashMap::from([
                ("content".to_string(), ToolProperty {
                    type_: "string".to_string(),
                    description: "Full file content".to_string(),
                    enum_values: None,
                }),
                ("line_number".to_string(), ToolProperty {
                    type_: "number".to_string(),
                    description: "Line number to move (1-based)".to_string(),
                    enum_values: None,
                }),
            ]),
            required: vec!["content".to_string(), "line_number".to_string()],
        },
        returns: ToolReturns {
            description: "Updated content with line moved".to_string(),
            schema: serde_json::json!({"type": "object"}),
        },
        examples: vec![],
    });

    // edit::delete_line - Delete line
    registry.register(Tool {
        name: "edit::delete_line".to_string(),
        description: "Delete the current line completely.".to_string(),
        parameters: ToolParameters {
            type_: "object".to_string(),
            properties: HashMap::from([
                ("content".to_string(), ToolProperty {
                    type_: "string".to_string(),
                    description: "Full file content".to_string(),
                    enum_values: None,
                }),
                ("line_number".to_string(), ToolProperty {
                    type_: "number".to_string(),
                    description: "Line number to delete (1-based)".to_string(),
                    enum_values: None,
                }),
            ]),
            required: vec!["content".to_string(), "line_number".to_string()],
        },
        returns: ToolReturns {
            description: "Updated content without deleted line".to_string(),
            schema: serde_json::json!({"type": "object"}),
        },
        examples: vec![],
    });

    // edit::sort_lines - Sort lines
    registry.register(Tool {
        name: "edit::sort_lines".to_string(),
        description: "Sort lines alphabetically or numerically.".to_string(),
        parameters: ToolParameters {
            type_: "object".to_string(),
            properties: HashMap::from([
                ("content".to_string(), ToolProperty {
                    type_: "string".to_string(),
                    description: "Full file content".to_string(),
                    enum_values: None,
                }),
                ("start_line".to_string(), ToolProperty {
                    type_: "number".to_string(),
                    description: "Start line number (1-based)".to_string(),
                    enum_values: None,
                }),
                ("end_line".to_string(), ToolProperty {
                    type_: "number".to_string(),
                    description: "End line number (1-based)".to_string(),
                    enum_values: None,
                }),
                ("ascending".to_string(), ToolProperty {
                    type_: "boolean".to_string(),
                    description: "Sort ascending (true) or descending (false)".to_string(),
                    enum_values: None,
                }),
            ]),
            required: vec!["content".to_string(), "start_line".to_string(), "end_line".to_string()],
        },
        returns: ToolReturns {
            description: "Updated content with sorted lines".to_string(),
            schema: serde_json::json!({"type": "object"}),
        },
        examples: vec![],
    });

    // edit::toggle_comment - Toggle line comment
    registry.register(Tool {
        name: "edit::toggle_comment".to_string(),
        description: "Toggle comment/uncomment for selected lines.".to_string(),
        parameters: ToolParameters {
            type_: "object".to_string(),
            properties: HashMap::from([
                ("content".to_string(), ToolProperty {
                    type_: "string".to_string(),
                    description: "Full file content".to_string(),
                    enum_values: None,
                }),
                ("start_line".to_string(), ToolProperty {
                    type_: "number".to_string(),
                    description: "Start line number (1-based)".to_string(),
                    enum_values: None,
                }),
                ("end_line".to_string(), ToolProperty {
                    type_: "number".to_string(),
                    description: "End line number (1-based)".to_string(),
                    enum_values: None,
                }),
                ("comment_prefix".to_string(), ToolProperty {
                    type_: "string".to_string(),
                    description: "Comment prefix (e.g., //, #)".to_string(),
                    enum_values: None,
                }),
            ]),
            required: vec!["content".to_string(), "start_line".to_string(), "end_line".to_string()],
        },
        returns: ToolReturns {
            description: "Updated content with comments toggled".to_string(),
            schema: serde_json::json!({"type": "object"}),
        },
        examples: vec![
            ToolExample {
                description: "Comment out lines 2-3 in Rust".to_string(),
                request: serde_json::json!({
                    "content": "fn main() {\n    println!(\"Hello\");\n    println!(\"World\");\n}",
                    "start_line": 2,
                    "end_line": 3,
                    "comment_prefix": "//"
                }),
                response: serde_json::json!({
                    "content": "fn main() {\n    // println!(\"Hello\");\n    // println!(\"World\");\n}",
                    "action": "commented"
                }),
            },
        ],
    });

    // edit::trim_whitespace - Trim whitespace
    registry.register(Tool {
        name: "edit::trim_whitespace".to_string(),
        description: "Trim whitespace from lines (trailing, leading, or all).".to_string(),
        parameters: ToolParameters {
            type_: "object".to_string(),
            properties: HashMap::from([
                ("content".to_string(), ToolProperty {
                    type_: "string".to_string(),
                    description: "Full file content".to_string(),
                    enum_values: None,
                }),
                ("mode".to_string(), ToolProperty {
                    type_: "string".to_string(),
                    description: "Trim mode: trailing, leading, or all".to_string(),
                    enum_values: Some(vec!["trailing".to_string(), "leading".to_string(), "all".to_string()]),
                }),
            ]),
            required: vec!["content".to_string(), "mode".to_string()],
        },
        returns: ToolReturns {
            description: "Updated content with trimmed whitespace".to_string(),
            schema: serde_json::json!({"type": "object"}),
        },
        examples: vec![],
    });

    // edit::change_case - Change text case
    registry.register(Tool {
        name: "edit::change_case".to_string(),
        description: "Change text case to uppercase or lowercase.".to_string(),
        parameters: ToolParameters {
            type_: "object".to_string(),
            properties: HashMap::from([
                ("content".to_string(), ToolProperty {
                    type_: "string".to_string(),
                    description: "Full file content".to_string(),
                    enum_values: None,
                }),
                ("start".to_string(), ToolProperty {
                    type_: "number".to_string(),
                    description: "Start position in content".to_string(),
                    enum_values: None,
                }),
                ("end".to_string(), ToolProperty {
                    type_: "number".to_string(),
                    description: "End position in content".to_string(),
                    enum_values: None,
                }),
                ("case".to_string(), ToolProperty {
                    type_: "string".to_string(),
                    description: "Target case: uppercase or lowercase".to_string(),
                    enum_values: Some(vec!["uppercase".to_string(), "lowercase".to_string()]),
                }),
            ]),
            required: vec!["content".to_string(), "start".to_string(), "end".to_string(), "case".to_string()],
        },
        returns: ToolReturns {
            description: "Updated content with changed case".to_string(),
            schema: serde_json::json!({"type": "object"}),
        },
        examples: vec![],
    });
}
