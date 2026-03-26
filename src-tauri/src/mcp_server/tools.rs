//! Tool Definitions for All Modules
//!
//! Each module exposes operations as MCP Tools for LLM discovery

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Tool definition for LLM
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    pub name: String,
    pub description: String,
    pub parameters: ToolParameters,
    pub returns: ToolReturns,
    pub examples: Vec<ToolExample>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolParameters {
    pub type_: String,
    pub properties: HashMap<String, ToolProperty>,
    pub required: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolProperty {
    pub type_: String,
    pub description: String,
    pub enum_values: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolReturns {
    pub description: String,
    pub schema: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolExample {
    pub description: String,
    pub request: serde_json::Value,
    pub response: serde_json::Value,
}

/// Tool registry
pub struct ToolRegistry {
    tools: HashMap<String, Tool>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
        }
    }

    pub fn register(&mut self, tool: Tool) {
        self.tools.insert(tool.name.clone(), tool);
    }

    pub fn get(&self, name: &str) -> Option<&Tool> {
        self.tools.get(name)
    }

    pub fn list(&self) -> Vec<&Tool> {
        self.tools.values().collect()
    }
}

/// Register file_module tools
pub fn register_file_module_tools(registry: &mut ToolRegistry) {
    // file_module::open
    registry.register(Tool {
        name: "file_module::open".to_string(),
        description: "Open a file and return file info with chunks".to_string(),
        parameters: ToolParameters {
            type_: "object".to_string(),
            properties: HashMap::from([
                ("path".to_string(), ToolProperty {
                    type_: "string".to_string(),
                    description: "Absolute path to the file".to_string(),
                    enum_values: None,
                }),
            ]),
            required: vec!["path".to_string()],
        },
        returns: ToolReturns {
            description: "File info including size, chunks, chunk_size".to_string(),
            schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {"type": "string"},
                    "size": {"type": "number"},
                    "chunks": {"type": "number"},
                    "chunk_size": {"type": "number"},
                }
            }),
        },
        examples: vec![
            ToolExample {
                description: "Open main.rs".to_string(),
                request: serde_json::json!({"path": "/project/src/main.rs"}),
                response: serde_json::json!({
                    "path": "/project/src/main.rs",
                    "size": 1024,
                    "chunks": 2,
                    "chunk_size": 65536
                }),
            },
        ],
    });

    // file_module::read_text
    registry.register(Tool {
        name: "file_module::read_text".to_string(),
        description: "Read a chunk as UTF-8 text".to_string(),
        parameters: ToolParameters {
            type_: "object".to_string(),
            properties: HashMap::from([
                ("path".to_string(), ToolProperty {
                    type_: "string".to_string(),
                    description: "File path".to_string(),
                    enum_values: None,
                }),
                ("chunk_id".to_string(), ToolProperty {
                    type_: "number".to_string(),
                    description: "Chunk index (0-based)".to_string(),
                    enum_values: None,
                }),
            ]),
            required: vec!["path".to_string(), "chunk_id".to_string()],
        },
        returns: ToolReturns {
            description: "Text content of the chunk".to_string(),
            schema: serde_json::json!({"type": "string"}),
        },
        examples: vec![],
    });

    // file_module::get_hex
    registry.register(Tool {
        name: "file_module::get_hex".to_string(),
        description: "Get hex view of file content".to_string(),
        parameters: ToolParameters {
            type_: "object".to_string(),
            properties: HashMap::from([
                ("path".to_string(), ToolProperty {
                    type_: "string".to_string(),
                    description: "File path".to_string(),
                    enum_values: None,
                }),
                ("offset".to_string(), ToolProperty {
                    type_: "number".to_string(),
                    description: "Byte offset".to_string(),
                    enum_values: None,
                }),
                ("length".to_string(), ToolProperty {
                    type_: "number".to_string(),
                    description: "Number of bytes".to_string(),
                    enum_values: None,
                }),
            ]),
            required: vec!["path".to_string()],
        },
        returns: ToolReturns {
            description: "Hex view rows".to_string(),
            schema: serde_json::json!({"type": "array"}),
        },
        examples: vec![],
    });

    // file_module::edit
    registry.register(Tool {
        name: "file_module::edit".to_string(),
        description: "Edit file content".to_string(),
        parameters: ToolParameters {
            type_: "object".to_string(),
            properties: HashMap::from([
                ("path".to_string(), ToolProperty {
                    type_: "string".to_string(),
                    description: "File path".to_string(),
                    enum_values: None,
                }),
                ("operation".to_string(), ToolProperty {
                    type_: "string".to_string(),
                    description: "Edit operation: insert, delete, replace".to_string(),
                    enum_values: Some(vec![
                        "insert".to_string(),
                        "delete".to_string(),
                        "replace".to_string(),
                    ]),
                }),
                ("offset".to_string(), ToolProperty {
                    type_: "number".to_string(),
                    description: "Byte offset".to_string(),
                    enum_values: None,
                }),
                ("data".to_string(), ToolProperty {
                    type_: "string".to_string(),
                    description: "Data to insert/replace (base64)".to_string(),
                    enum_values: None,
                }),
            ]),
            required: vec!["path".to_string(), "operation".to_string()],
        },
        returns: ToolReturns {
            description: "Edit result".to_string(),
            schema: serde_json::json!({"type": "object"}),
        },
        examples: vec![],
    });

    // file_module::save
    registry.register(Tool {
        name: "file_module::save".to_string(),
        description: "Save changes to file".to_string(),
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
            description: "Success status".to_string(),
            schema: serde_json::json!({"type": "boolean"}),
        },
        examples: vec![],
    });

    // file_module::undo
    registry.register(Tool {
        name: "file_module::undo".to_string(),
        description: "Undo last edit".to_string(),
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
            description: "Undo result".to_string(),
            schema: serde_json::json!({"type": "object"}),
        },
        examples: vec![],
    });
}

/// Register llm_module tools
pub fn register_llm_module_tools(registry: &mut ToolRegistry) {
    // llm_module::chat
    registry.register(Tool {
        name: "llm_module::chat".to_string(),
        description: "Send message to LLM and get response".to_string(),
        parameters: ToolParameters {
            type_: "object".to_string(),
            properties: HashMap::from([
                ("message".to_string(), ToolProperty {
                    type_: "string".to_string(),
                    description: "Message text".to_string(),
                    enum_values: None,
                }),
                ("context".to_string(), ToolProperty {
                    type_: "string".to_string(),
                    description: "Optional file context".to_string(),
                    enum_values: None,
                }),
            ]),
            required: vec!["message".to_string()],
        },
        returns: ToolReturns {
            description: "LLM response".to_string(),
            schema: serde_json::json!({"type": "string"}),
        },
        examples: vec![],
    });

    // llm_module::summarize
    registry.register(Tool {
        name: "llm_module::summarize".to_string(),
        description: "Summarize file content".to_string(),
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
            description: "Summary text".to_string(),
            schema: serde_json::json!({"type": "string"}),
        },
        examples: vec![],
    });
}

/// Register sftp_module tools
pub fn register_sftp_module_tools(registry: &mut ToolRegistry) {
    // sftp_module::connect
    registry.register(Tool {
        name: "sftp_module::connect".to_string(),
        description: "Connect to remote SFTP server".to_string(),
        parameters: ToolParameters {
            type_: "object".to_string(),
            properties: HashMap::from([
                ("host".to_string(), ToolProperty {
                    type_: "string".to_string(),
                    description: "Hostname or IP".to_string(),
                    enum_values: None,
                }),
                ("port".to_string(), ToolProperty {
                    type_: "number".to_string(),
                    description: "Port (default 22)".to_string(),
                    enum_values: None,
                }),
                ("username".to_string(), ToolProperty {
                    type_: "string".to_string(),
                    description: "Username".to_string(),
                    enum_values: None,
                }),
                ("password".to_string(), ToolProperty {
                    type_: "string".to_string(),
                    description: "Password".to_string(),
                    enum_values: None,
                }),
            ]),
            required: vec!["host".to_string(), "username".to_string()],
        },
        returns: ToolReturns {
            description: "Connection handle".to_string(),
            schema: serde_json::json!({"type": "string"}),
        },
        examples: vec![],
    });

    // sftp_module::list_dir
    registry.register(Tool {
        name: "sftp_module::list_dir".to_string(),
        description: "List remote directory".to_string(),
        parameters: ToolParameters {
            type_: "object".to_string(),
            properties: HashMap::from([
                ("connection".to_string(), ToolProperty {
                    type_: "string".to_string(),
                    description: "Connection handle".to_string(),
                    enum_values: None,
                }),
                ("path".to_string(), ToolProperty {
                    type_: "string".to_string(),
                    description: "Remote path".to_string(),
                    enum_values: None,
                }),
            ]),
            required: vec!["connection".to_string(), "path".to_string()],
        },
        returns: ToolReturns {
            description: "Directory entries".to_string(),
            schema: serde_json::json!({"type": "array"}),
        },
        examples: vec![],
    });
}

/// Register semantic module tools
pub fn register_semantic_tools(registry: &mut ToolRegistry) {
    // semantic::parse_file
    registry.register(Tool {
        name: "semantic::parse_file".to_string(),
        description: "Parse file into semantic blocks (functions, structs, imports)".to_string(),
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
            description: "Semantic blocks with metadata".to_string(),
            schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {"type": "string"},
                    "language": {"type": "string"},
                    "block_count": {"type": "number"},
                    "blocks": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "properties": {
                                "id": {"type": "string"},
                                "type": {"type": "string"},
                                "name": {"type": "string"},
                                "line_start": {"type": "number"},
                                "line_end": {"type": "number"},
                            }
                        }
                    }
                }
            }),
        },
        examples: vec![
            ToolExample {
                description: "Parse main.rs".to_string(),
                request: serde_json::json!({"path": "/project/src/main.rs"}),
                response: serde_json::json!({
                    "path": "/project/src/main.rs",
                    "language": "rust",
                    "block_count": 3,
                    "blocks": [
                        {"id": "1", "type": "Function", "name": "main", "line_start": 1, "line_end": 5},
                    ]
                }),
            },
        ],
    });

    // semantic::query
    registry.register(Tool {
        name: "semantic::query".to_string(),
        description: "Query code using natural language. Examples: 'function called main', 'import from std', 'struct named User'".to_string(),
        parameters: ToolParameters {
            type_: "object".to_string(),
            properties: HashMap::from([
                ("path".to_string(), ToolProperty {
                    type_: "string".to_string(),
                    description: "File path".to_string(),
                    enum_values: None,
                }),
                ("query".to_string(), ToolProperty {
                    type_: "string".to_string(),
                    description: "Natural language query".to_string(),
                    enum_values: None,
                }),
            ]),
            required: vec!["path".to_string(), "query".to_string()],
        },
        returns: ToolReturns {
            description: "Matching blocks".to_string(),
            schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "total": {"type": "number"},
                    "results": {"type": "array"},
                }
            }),
        },
        examples: vec![
            ToolExample {
                description: "Find authentication function".to_string(),
                request: serde_json::json!({"path": "/project/main.rs", "query": "function handling authentication"}),
                response: serde_json::json!({
                    "total": 1,
                    "results": [{"name": "authenticate_user", "line_start": 42}]
                }),
            },
        ],
    });

    // semantic::intelligent_mark
    registry.register(Tool {
        name: "semantic::intelligent_mark".to_string(),
        description: "Automatically analyze and mark important code sections with colors".to_string(),
        parameters: ToolParameters {
            type_: "object".to_string(),
            properties: HashMap::from([
                ("path".to_string(), ToolProperty {
                    type_: "string".to_string(),
                    description: "File path".to_string(),
                    enum_values: None,
                }),
                ("auto_navigate".to_string(), ToolProperty {
                    type_: "boolean".to_string(),
                    description: "Jump to first critical mark".to_string(),
                    enum_values: None,
                }),
            ]),
            required: vec!["path".to_string()],
        },
        returns: ToolReturns {
            description: "Analysis with marks".to_string(),
            schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "summary": {"type": "string"},
                    "marks_added": {"type": "number"},
                    "critical_count": {"type": "number"},
                    "marked_sections": {"type": "array"},
                }
            }),
        },
        examples: vec![],
    });
}

/// Register marks_module tools
pub fn register_mark_module_tools(registry: &mut ToolRegistry) {
    // marks_module::create
    registry.register(Tool {
        name: "marks_module::create".to_string(),
        description: "Create a color mark at specific line".to_string(),
        parameters: ToolParameters {
            type_: "object".to_string(),
            properties: HashMap::from([
                ("path".to_string(), ToolProperty {
                    type_: "string".to_string(),
                    description: "File path".to_string(),
                    enum_values: None,
                }),
                ("line".to_string(), ToolProperty {
                    type_: "number".to_string(),
                    description: "Line number (1-based)".to_string(),
                    enum_values: None,
                }),
                ("color".to_string(), ToolProperty {
                    type_: "string".to_string(),
                    description: "Color: red, orange, yellow, green, blue, purple".to_string(),
                    enum_values: Some(vec![
                        "red".to_string(),
                        "orange".to_string(),
                        "yellow".to_string(),
                        "green".to_string(),
                        "blue".to_string(),
                        "purple".to_string(),
                    ]),
                }),
                ("label".to_string(), ToolProperty {
                    type_: "string".to_string(),
                    description: "Optional label".to_string(),
                    enum_values: None,
                }),
            ]),
            required: vec!["path".to_string(), "line".to_string()],
        },
        returns: ToolReturns {
            description: "Mark ID".to_string(),
            schema: serde_json::json!({"type": "string"}),
        },
        examples: vec![
            ToolExample {
                description: "Mark line 42 as critical".to_string(),
                request: serde_json::json!({
                    "path": "/project/main.rs",
                    "line": 42,
                    "color": "red",
                    "label": "Security issue"
                }),
                response: serde_json::json!({"mark_id": "mark-123"}),
            },
        ],
    });

    // marks_module::clear
    registry.register(Tool {
        name: "marks_module::clear".to_string(),
        description: "Clear all marks from file".to_string(),
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
            description: "Cleared count".to_string(),
            schema: serde_json::json!({"type": "number"}),
        },
        examples: vec![],
    });

    // marks_module::navigate
    registry.register(Tool {
        name: "marks_module::navigate".to_string(),
        description: "Navigate to a specific mark or line".to_string(),
        parameters: ToolParameters {
            type_: "object".to_string(),
            properties: HashMap::from([
                ("path".to_string(), ToolProperty {
                    type_: "string".to_string(),
                    description: "File path".to_string(),
                    enum_values: None,
                }),
                ("line".to_string(), ToolProperty {
                    type_: "number".to_string(),
                    description: "Line to navigate to".to_string(),
                    enum_values: None,
                }),
            ]),
            required: vec!["path".to_string(), "line".to_string()],
        },
        returns: ToolReturns {
            description: "Navigation result".to_string(),
            schema: serde_json::json!({"type": "object"}),
        },
        examples: vec![],
    });
}
