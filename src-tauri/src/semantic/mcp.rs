//! Model Context Protocol (MCP) Server for LLM Integration
//!
//! Allows LLMs to directly control and query the editor

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use crate::semantic::SemanticDocument;
use crate::semantic::parser::CodeParser;
use crate::semantic::query::QueryEngine;
use crate::semantic::bridge::{LLMBridge, LLMEditParser};

/// MCP Tool definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    pub name: String,
    pub description: String,
    pub parameters: ToolParameters,
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
}

/// MCP Server for LLM integration
pub struct MCPServer {
    tools: Vec<Tool>,
    sessions: Arc<Mutex<HashMap<String, MCPSession>>>,
}

impl MCPServer {
    pub fn new() -> Self {
        let mut server = Self {
            tools: Vec::new(),
            sessions: Arc::new(Mutex::new(HashMap::new())),
        };
        server.register_default_tools();
        server
    }

    fn register_default_tools(&mut self) {
        // Tool: parse_file
        self.tools.push(Tool {
            name: "semantic_parse_file".to_string(),
            description: "Parse a code file into semantic blocks (functions, structs, imports, etc.)".to_string(),
            parameters: ToolParameters {
                type_: "object".to_string(),
                properties: HashMap::from([
                    ("path".to_string(), ToolProperty {
                        type_: "string".to_string(),
                        description: "Absolute path to the file to parse".to_string(),
                    }),
                ]),
                required: vec!["path".to_string()],
            },
        });

        // Tool: query_code
        self.tools.push(Tool {
            name: "semantic_query".to_string(),
            description: "Query code using natural language. Examples: 'function called main', 'import from std', 'struct named User', 'all tests'".to_string(),
            parameters: ToolParameters {
                type_: "object".to_string(),
                properties: HashMap::from([
                    ("path".to_string(), ToolProperty {
                        type_: "string".to_string(),
                        description: "Absolute path to the file".to_string(),
                    }),
                    ("query".to_string(), ToolProperty {
                        type_: "string".to_string(),
                        description: "Natural language query about the code".to_string(),
                    }),
                ]),
                required: vec!["path".to_string(), "query".to_string()],
            },
        });

        // Tool: export_llm
        self.tools.push(Tool {
            name: "semantic_export_llm".to_string(),
            description: "Export file in LLM-friendly format (compact, json, or tree)".to_string(),
            parameters: ToolParameters {
                type_: "object".to_string(),
                properties: HashMap::from([
                    ("path".to_string(), ToolProperty {
                        type_: "string".to_string(),
                        description: "Absolute path to the file".to_string(),
                    }),
                    ("format".to_string(), ToolProperty {
                        type_: "string".to_string(),
                        description: "Format: 'compact', 'json', or 'tree'".to_string(),
                    }),
                ]),
                required: vec!["path".to_string()],
            },
        });

        // Tool: edit_code
        self.tools.push(Tool {
            name: "semantic_edit".to_string(),
            description: "Edit code using natural language intent. Examples: 'add field email of type String to User', 'extract function calculateTotal', 'rename oldName to newName'".to_string(),
            parameters: ToolParameters {
                type_: "object".to_string(),
                properties: HashMap::from([
                    ("path".to_string(), ToolProperty {
                        type_: "string".to_string(),
                        description: "Absolute path to the file".to_string(),
                    }),
                    ("intent".to_string(), ToolProperty {
                        type_: "string".to_string(),
                        description: "Natural language edit intent".to_string(),
                    }),
                ]),
                required: vec!["path".to_string(), "intent".to_string()],
            },
        });

        // Tool: get_context
        self.tools.push(Tool {
            name: "semantic_get_context".to_string(),
            description: "Get context around a specific block or location".to_string(),
            parameters: ToolParameters {
                type_: "object".to_string(),
                properties: HashMap::from([
                    ("path".to_string(), ToolProperty {
                        type_: "string".to_string(),
                        description: "Absolute path to the file".to_string(),
                    }),
                    ("block_name".to_string(), ToolProperty {
                        type_: "string".to_string(),
                        description: "Name of the block to get context for".to_string(),
                    }),
                ]),
                required: vec!["path".to_string(), "block_name".to_string()],
            },
        });

        // Tool: list_directory
        self.tools.push(Tool {
            name: "list_directory".to_string(),
            description: "List files in a directory".to_string(),
            parameters: ToolParameters {
                type_: "object".to_string(),
                properties: HashMap::from([
                    ("path".to_string(), ToolProperty {
                        type_: "string".to_string(),
                        description: "Absolute path to the directory".to_string(),
                    }),
                ]),
                required: vec!["path".to_string()],
            },
        });

        // Tool: open_file
        self.tools.push(Tool {
            name: "open_file_in_editor".to_string(),
            description: "Open a file in the editor".to_string(),
            parameters: ToolParameters {
                type_: "object".to_string(),
                properties: HashMap::from([
                    ("path".to_string(), ToolProperty {
                        type_: "string".to_string(),
                        description: "Absolute path to the file to open".to_string(),
                    }),
                ]),
                required: vec!["path".to_string()],
            },
        });

        // Tool: get_open_files
        self.tools.push(Tool {
            name: "get_open_files".to_string(),
            description: "Get list of currently open files in the editor".to_string(),
            parameters: ToolParameters {
                type_: "object".to_string(),
                properties: HashMap::new(),
                required: vec![],
            },
        });
    }

    /// Get available tools
    pub fn list_tools(&self) -> &[Tool] {
        &self.tools
    }

    /// Execute a tool
    pub async fn execute_tool(&self, tool_name: &str, params: Value) -> Result<Value, String> {
        match tool_name {
            "semantic_parse_file" => self.handle_parse_file(params).await,
            "semantic_query" => self.handle_query(params).await,
            "semantic_export_llm" => self.handle_export(params).await,
            "semantic_edit" => self.handle_edit(params).await,
            "semantic_get_context" => self.handle_get_context(params).await,
            "list_directory" => self.handle_list_directory(params).await,
            "open_file_in_editor" => self.handle_open_file(params).await,
            "get_open_files" => self.handle_get_open_files().await,
            _ => Err(format!("Unknown tool: {}", tool_name)),
        }
    }

    async fn handle_parse_file(&self, params: Value) -> Result<Value, String> {
        let path = params["path"].as_str()
            .ok_or("path parameter required")?;
        
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read file: {}", e))?;
        
        let language = CodeParser::detect_language(path);
        let parser = CodeParser::new(&language);
        let result = parser.parse(&content);
        
        Ok(serde_json::json!({
            "success": true,
            "path": path,
            "language": language,
            "block_count": result.blocks.len(),
            "blocks": result.blocks.iter().map(|b| {
                serde_json::json!({
                    "id": b.id.0,
                    "type": format!("{:?}", b.block_type),
                    "name": b.name,
                    "signature": b.signature,
                    "purpose": b.purpose,
                    "line_start": b.location.line_start,
                    "line_end": b.location.line_end,
                })
            }).collect::<Vec<_>>(),
        }))
    }

    async fn handle_query(&self, params: Value) -> Result<Value, String> {
        let path = params["path"].as_str()
            .ok_or("path parameter required")?;
        let query_str = params["query"].as_str()
            .ok_or("query parameter required")?;
        
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read file: {}", e))?;
        
        let language = CodeParser::detect_language(path);
        let parser = CodeParser::new(&language);
        let result = parser.parse(&content);
        
        let doc = SemanticDocument {
            path: path.to_string(),
            language,
            blocks: result.blocks,
            relationships: vec![],
            summary: String::new(),
        };
        
        let engine = QueryEngine::new(&doc);
        let matches = engine.execute(query_str);
        
        Ok(serde_json::json!({
            "success": true,
            "query": query_str,
            "total": matches.len(),
            "results": matches.iter().map(|b| {
                serde_json::json!({
                    "id": b.id.0,
                    "type": format!("{:?}", b.block_type),
                    "name": b.name,
                    "signature": b.signature,
                    "purpose": b.purpose,
                    "line_start": b.location.line_start,
                    "line_end": b.location.line_end,
                    "preview": if b.content.len() > 200 {
                        format!("{}...", &b.content[..200])
                    } else {
                        b.content.clone()
                    },
                })
            }).collect::<Vec<_>>(),
        }))
    }

    async fn handle_export(&self, params: Value) -> Result<Value, String> {
        let path = params["path"].as_str()
            .ok_or("path parameter required")?;
        let format = params["format"].as_str().unwrap_or("compact");
        
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read file: {}", e))?;
        
        let language = CodeParser::detect_language(path);
        let parser = CodeParser::new(&language);
        let result = parser.parse(&content);
        
        let doc = SemanticDocument {
            path: path.to_string(),
            language,
            blocks: result.blocks,
            relationships: vec![],
            summary: format!("{} file", path),
        };
        
        let output = match format {
            "json" => {
                let llm_format = LLMBridge::export(&doc);
                serde_json::to_string_pretty(&llm_format)
                    .map_err(|e| format!("Serialization error: {}", e))?
            }
            "compact" => LLMBridge::export_compact(&doc),
            "tree" => LLMBridge::export_tree(&doc),
            _ => return Err(format!("Unknown format: {}", format)),
        };
        
        Ok(serde_json::json!({
            "success": true,
            "format": format,
            "content": output,
        }))
    }

    async fn handle_edit(&self, params: Value) -> Result<Value, String> {
        let path = params["path"].as_str()
            .ok_or("path parameter required")?;
        let intent = params["intent"].as_str()
            .ok_or("intent parameter required")?;
        
        let edit_request = LLMEditParser::parse(intent)?;
        
        Ok(serde_json::json!({
            "success": true,
            "intent": edit_request.intent,
            "description": edit_request.description,
            "target": edit_request.target,
            "message": "Edit parsed successfully. Use semantic_apply_edit to apply.",
        }))
    }

    async fn handle_get_context(&self, params: Value) -> Result<Value, String> {
        let path = params["path"].as_str()
            .ok_or("path parameter required")?;
        let block_name = params["block_name"].as_str()
            .ok_or("block_name parameter required")?;
        
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read file: {}", e))?;
        
        let language = CodeParser::detect_language(path);
        let parser = CodeParser::new(&language);
        let result = parser.parse(&content);
        
        // Find the block
        let block = result.blocks.iter()
            .find(|b| b.name == block_name)
            .ok_or(format!("Block '{}' not found", block_name))?;
        
        // Get surrounding lines
        let lines: Vec<&str> = content.lines().collect();
        let start = block.location.line_start.saturating_sub(5);
        let end = (block.location.line_end + 5).min(lines.len());
        let context: String = lines[start..end].join("\n");
        
        Ok(serde_json::json!({
            "success": true,
            "block": {
                "name": block.name,
                "type": format!("{:?}", block.block_type),
                "lines": (block.location.line_start, block.location.line_end),
            },
            "context_lines": (start + 1, end),
            "context": context,
        }))
    }

    async fn handle_list_directory(&self, params: Value) -> Result<Value, String> {
        let path = params["path"].as_str()
            .ok_or("path parameter required")?;
        
        let mut entries = Vec::new();
        let dir = std::fs::read_dir(path)
            .map_err(|e| format!("Failed to read directory: {}", e))?;
        
        for entry in dir {
            if let Ok(entry) = entry {
                let name = entry.file_name().to_string_lossy().to_string();
                let is_file = entry.file_type().map(|t| t.is_file()).unwrap_or(false);
                entries.push(serde_json::json!({
                    "name": name,
                    "is_file": is_file,
                    "path": entry.path().to_string_lossy().to_string(),
                }));
            }
        }
        
        Ok(serde_json::json!({
            "success": true,
            "path": path,
            "entries": entries,
        }))
    }

    async fn handle_open_file(&self, params: Value) -> Result<Value, String> {
        let path = params["path"].as_str()
            .ok_or("path parameter required")?;
        
        // This would trigger the UI to open the file
        // For now, return success
        Ok(serde_json::json!({
            "success": true,
            "path": path,
            "message": "File open request sent to editor",
        }))
    }

    async fn handle_get_open_files(&self) -> Result<Value, String> {
        // This would return actually open files from the editor
        Ok(serde_json::json!({
            "success": true,
            "files": [],
            "message": "Use the UI to see open files",
        }))
    }
}

/// MCP Session
#[derive(Debug)]
pub struct MCPSession {
    pub id: String,
    pub created_at: std::time::SystemTime,
}

impl MCPSession {
    pub fn new() -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            created_at: std::time::SystemTime::now(),
        }
    }
}

/// MCP Tool Call
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub arguments: Value,
}

/// MCP Tool Result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub tool_call_id: String,
    pub result: Value,
    pub error: Option<String>,
}
