//! MCP Tool Handlers
//!
//! Connects MCP tool definitions to actual module implementations

use async_trait::async_trait;
use serde_json::Value;
use crate::modules::file_module::FileModule;
use crate::modules::llm_module::LlmModule;
use crate::modules::sftp_module::SftpModule;
use crate::modules::marks_module::MarksModule;
use crate::semantic::intelligent_marks::{IntelligentMarkEngine, ApplyIntelligentMarksResult, NavigationTarget};
use crate::semantic::SemanticDocument;
use crate::semantic::parser::CodeParser;
use crate::semantic::query::QueryEngine;
use crate::semantic::bridge::LLMBridge;
use crate::mark_engine::{MarkColor, Mark};
use std::sync::{Arc, Mutex};

/// Tool handler trait
#[async_trait]
pub trait ToolHandler: Send + Sync {
    async fn execute(&self, params: Value) -> Result<Value, String>;
}

/// Register all default handlers
pub fn register_default_handlers(server: &mut super::MCPServer) {
    // File module handlers
    server.register_handler("file_module::open", Box::new(FileOpenHandler));
    server.register_handler("file_module::read_text", Box::new(FileReadTextHandler));
    server.register_handler("file_module::get_hex", Box::new(FileGetHexHandler));
    server.register_handler("file_module::edit", Box::new(FileEditHandler));
    server.register_handler("file_module::save", Box::new(FileSaveHandler));
    server.register_handler("file_module::undo", Box::new(FileUndoHandler));
    
    // LLM module handlers
    server.register_handler("llm_module::chat", Box::new(LlmChatHandler));
    server.register_handler("llm_module::summarize", Box::new(LlmSummarizeHandler));
    
    // SFTP module handlers
    server.register_handler("sftp_module::connect", Box::new(SftpConnectHandler));
    server.register_handler("sftp_module::list_dir", Box::new(SftpListDirHandler));
    
    // Semantic module handlers
    server.register_handler("semantic::parse_file", Box::new(SemanticParseHandler));
    server.register_handler("semantic::query", Box::new(SemanticQueryHandler));
    server.register_handler("semantic::intelligent_mark", Box::new(SemanticIntelligentMarkHandler));
    server.register_handler("semantic::export_llm", Box::new(SemanticExportHandler));
    
    // Marks module handlers
    server.register_handler("marks_module::create", Box::new(MarksCreateHandler));
    server.register_handler("marks_module::clear", Box::new(MarksClearHandler));
    server.register_handler("marks_module::navigate", Box::new(MarksNavigateHandler));
}

// ============== File Module Handlers ==============

struct FileOpenHandler;

#[async_trait]
impl ToolHandler for FileOpenHandler {
    async fn execute(&self, params: Value) -> Result<Value, String> {
        let path = params["path"].as_str()
            .ok_or("path parameter required")?;
        
        // Call actual file_module::open via modular system
        let result = crate::modular::execute_module(
            "file",
            "open",
            serde_json::json!({ "path": path })
        ).map_err(|e| e.to_string())?;
        
        Ok(result)
    }
}

struct FileReadTextHandler;

#[async_trait]
impl ToolHandler for FileReadTextHandler {
    async fn execute(&self, params: Value) -> Result<Value, String> {
        let path = params["path"].as_str().ok_or("path required")?;
        let chunk_id = params["chunk_id"].as_u64().unwrap_or(0) as usize;
        
        let result = crate::modular::execute_module(
            "file",
            "read_text",
            serde_json::json!({ "path": path, "chunk_id": chunk_id })
        ).map_err(|e| e.to_string())?;
        
        Ok(result)
    }
}

struct FileGetHexHandler;

#[async_trait]
impl ToolHandler for FileGetHexHandler {
    async fn execute(&self, params: Value) -> Result<Value, String> {
        let path = params["path"].as_str().ok_or("path required")?;
        let offset = params["offset"].as_u64().unwrap_or(0) as usize;
        let length = params["length"].as_u64().unwrap_or(256) as usize;
        
        let result = crate::modular::execute_module(
            "file",
            "get_hex",
            serde_json::json!({ "path": path, "offset": offset, "length": length })
        ).map_err(|e| e.to_string())?;
        
        Ok(result)
    }
}

struct FileEditHandler;

#[async_trait]
impl ToolHandler for FileEditHandler {
    async fn execute(&self, params: Value) -> Result<Value, String> {
        let path = params["path"].as_str().ok_or("path required")?;
        let operation = params["operation"].as_str().ok_or("operation required")?;
        
        let result = match operation {
            "insert" => {
                let offset = params["offset"].as_u64().ok_or("offset required")? as usize;
                let data = params["data"].as_str().ok_or("data required")?;
                crate::modular::execute_module(
                    "file",
                    "insert",
                    serde_json::json!({ "path": path, "offset": offset, "data": data })
                )
            }
            "delete" => {
                let offset = params["offset"].as_u64().ok_or("offset required")? as usize;
                let length = params["length"].as_u64().ok_or("length required")? as usize;
                crate::modular::execute_module(
                    "file",
                    "delete",
                    serde_json::json!({ "path": path, "offset": offset, "length": length })
                )
            }
            "replace" => {
                let offset = params["offset"].as_u64().ok_or("offset required")? as usize;
                let data = params["data"].as_str().ok_or("data required")?;
                crate::modular::execute_module(
                    "file",
                    "replace",
                    serde_json::json!({ "path": path, "offset": offset, "data": data })
                )
            }
            _ => return Err(format!("Unknown operation: {}", operation)),
        }.map_err(|e| e.to_string())?;
        
        Ok(result)
    }
}

struct FileSaveHandler;

#[async_trait]
impl ToolHandler for FileSaveHandler {
    async fn execute(&self, params: Value) -> Result<Value, String> {
        let path = params["path"].as_str().ok_or("path required")?;
        
        let result = crate::modular::execute_module(
            "file",
            "save",
            serde_json::json!({ "path": path })
        ).map_err(|e| e.to_string())?;
        
        Ok(result)
    }
}

struct FileUndoHandler;

#[async_trait]
impl ToolHandler for FileUndoHandler {
    async fn execute(&self, params: Value) -> Result<Value, String> {
        let path = params["path"].as_str().ok_or("path required")?;
        
        let result = crate::modular::execute_module(
            "file",
            "undo",
            serde_json::json!({ "path": path })
        ).map_err(|e| e.to_string())?;
        
        Ok(result)
    }
}

// ============== LLM Module Handlers ==============

struct LlmChatHandler;

#[async_trait]
impl ToolHandler for LlmChatHandler {
    async fn execute(&self, params: Value) -> Result<Value, String> {
        let message = params["message"].as_str().ok_or("message required")?;
        let context = params.get("context").and_then(|v| v.as_str());
        
        let result = crate::modular::execute_module(
            "llm",
            "chat",
            serde_json::json!({ 
                "message": message,
                "context": context 
            })
        ).map_err(|e| e.to_string())?;
        
        Ok(result)
    }
}

struct LlmSummarizeHandler;

#[async_trait]
impl ToolHandler for LlmSummarizeHandler {
    async fn execute(&self, params: Value) -> Result<Value, String> {
        let path = params["path"].as_str().ok_or("path required")?;
        
        // First read the file content
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read file: {}", e))?;
        
        let result = crate::modular::execute_module(
            "llm",
            "summarize",
            serde_json::json!({ "content": content })
        ).map_err(|e| e.to_string())?;
        
        Ok(result)
    }
}

// ============== SFTP Module Handlers ==============

struct SftpConnectHandler;

#[async_trait]
impl ToolHandler for SftpConnectHandler {
    async fn execute(&self, params: Value) -> Result<Value, String> {
        let host = params["host"].as_str().ok_or("host required")?;
        let port = params["port"].as_u64().unwrap_or(22) as u16;
        let username = params["username"].as_str().ok_or("username required")?;
        let password = params.get("password").and_then(|v| v.as_str());
        
        let result = crate::modular::execute_module(
            "sftp",
            "connect",
            serde_json::json!({
                "host": host,
                "port": port,
                "username": username,
                "password": password
            })
        ).map_err(|e| e.to_string())?;
        
        Ok(result)
    }
}

struct SftpListDirHandler;

#[async_trait]
impl ToolHandler for SftpListDirHandler {
    async fn execute(&self, params: Value) -> Result<Value, String> {
        let connection = params["connection"].as_str().ok_or("connection handle required")?;
        let path = params["path"].as_str().ok_or("path required")?;
        
        let result = crate::modular::execute_module(
            "sftp",
            "list_dir",
            serde_json::json!({
                "connection": connection,
                "path": path
            })
        ).map_err(|e| e.to_string())?;
        
        Ok(result)
    }
}

// ============== Semantic Module Handlers ==============

struct SemanticParseHandler;

#[async_trait]
impl ToolHandler for SemanticParseHandler {
    async fn execute(&self, params: Value) -> Result<Value, String> {
        let path = params["path"].as_str().ok_or("path required")?;
        
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read file: {}", e))?;
        
        let language = crate::semantic::parser::CodeParser::detect_language(path);
        let parser = crate::semantic::parser::CodeParser::new(&language);
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
}

struct SemanticQueryHandler;

#[async_trait]
impl ToolHandler for SemanticQueryHandler {
    async fn execute(&self, params: Value) -> Result<Value, String> {
        let path = params["path"].as_str().ok_or("path required")?;
        let query = params["query"].as_str().ok_or("query required")?;
        
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read file: {}", e))?;
        
        let language = crate::semantic::parser::CodeParser::detect_language(path);
        let parser = crate::semantic::parser::CodeParser::new(&language);
        let result = parser.parse(&content);
        
        let doc = SemanticDocument {
            path: path.to_string(),
            language,
            blocks: result.blocks,
            relationships: vec![],
            summary: String::new(),
        };
        
        let engine = QueryEngine::new(&doc);
        let matches = engine.execute(query);
        
        Ok(serde_json::json!({
            "success": true,
            "query": query,
            "total": matches.len(),
            "results": matches.iter().map(|b| {
                serde_json::json!({
                    "id": b.id.0,
                    "type": format!("{:?}", b.block_type),
                    "name": b.name,
                    "line_start": b.location.line_start,
                    "line_end": b.location.line_end,
                })
            }).collect::<Vec<_>>(),
        }))
    }
}

struct SemanticIntelligentMarkHandler;

#[async_trait]
impl ToolHandler for SemanticIntelligentMarkHandler {
    async fn execute(&self, params: Value) -> Result<Value, String> {
        let path = params["path"].as_str().ok_or("path required")?;
        let auto_navigate = params["auto_navigate"].as_bool().unwrap_or(false);
        
        // Parse file into semantic blocks
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read file: {}", e))?;
        
        let language = crate::semantic::parser::CodeParser::detect_language(path);
        let parser = crate::semantic::parser::CodeParser::new(&language);
        let result = parser.parse(&content);
        
        let doc = SemanticDocument {
            path: path.to_string(),
            language,
            blocks: result.blocks,
            relationships: vec![],
            summary: format!("{} file", path),
        };
        
        // Run intelligent analysis
        let analysis = IntelligentMarkEngine::analyze(&doc);
        
        // Create marks via marks_module
        let mut marks_added = 0;
        let mut critical_count = 0;
        let mut first_critical: Option<NavigationTarget> = None;
        
        for section in &analysis.marked_sections {
            let color = section.importance.to_mark_color();
            
            // Create mark for each line in range
            for line in section.line_start..=section.line_end {
                let _ = crate::modular::execute_module(
                    "marks",
                    "create",
                    serde_json::json!({
                        "path": path,
                        "line": line,
                        "color": format!("{:?}", color).to_lowercase(),
                        "label": format!("{}: {}", section.importance.to_mark_color(), section.reason)
                    })
                );
                marks_added += 1;
            }
            
            if section.importance == crate::semantic::intelligent_marks::ImportanceLevel::Critical {
                critical_count += 1;
                if first_critical.is_none() {
                    first_critical = Some(NavigationTarget {
                        file_path: path.to_string(),
                        line: section.line_start,
                        reason: section.reason.clone(),
                    });
                }
            }
        }
        
        // Auto-navigate if requested
        let navigation_target = if auto_navigate {
            if let Some(ref target) = first_critical {
                let _ = crate::modular::execute_module(
                    "marks",
                    "navigate",
                    serde_json::json!({
                        "path": target.file_path,
                        "line": target.line
                    })
                );
            }
            first_critical
        } else {
            None
        };
        
        Ok(serde_json::json!({
            "success": true,
            "marks_added": marks_added,
            "critical_count": critical_count,
            "summary": analysis.summary,
            "key_insights": analysis.key_insights,
            "navigation_target": navigation_target.map(|t| {
                serde_json::json!({
                    "file_path": t.file_path,
                    "line": t.line,
                    "reason": t.reason
                })
            }),
        }))
    }
}

struct SemanticExportHandler;

#[async_trait]
impl ToolHandler for SemanticExportHandler {
    async fn execute(&self, params: Value) -> Result<Value, String> {
        let path = params["path"].as_str().ok_or("path required")?;
        let format = params["format"].as_str().unwrap_or("compact");
        
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read file: {}", e))?;
        
        let language = crate::semantic::parser::CodeParser::detect_language(path);
        let parser = crate::semantic::parser::CodeParser::new(&language);
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
}

// ============== Marks Module Handlers ==============

struct MarksCreateHandler;

#[async_trait]
impl ToolHandler for MarksCreateHandler {
    async fn execute(&self, params: Value) -> Result<Value, String> {
        let path = params["path"].as_str().ok_or("path required")?;
        let line = params["line"].as_u64().ok_or("line required")? as u32;
        let color_str = params["color"].as_str().unwrap_or("yellow");
        let label = params.get("label").and_then(|v| v.as_str());
        
        let color = match color_str {
            "red" => MarkColor::Red,
            "orange" => MarkColor::Orange,
            "yellow" => MarkColor::Yellow,
            "green" => MarkColor::Green,
            "blue" => MarkColor::Blue,
            "purple" => MarkColor::Purple,
            _ => MarkColor::Yellow,
        };
        
        let result = crate::modular::execute_module(
            "marks",
            "create",
            serde_json::json!({
                "path": path,
                "line": line,
                "color": color_str,
                "label": label
            })
        ).map_err(|e| e.to_string())?;
        
        Ok(result)
    }
}

struct MarksClearHandler;

#[async_trait]
impl ToolHandler for MarksClearHandler {
    async fn execute(&self, params: Value) -> Result<Value, String> {
        let path = params["path"].as_str().ok_or("path required")?;
        
        let result = crate::modular::execute_module(
            "marks",
            "clear",
            serde_json::json!({ "path": path })
        ).map_err(|e| e.to_string())?;
        
        Ok(result)
    }
}

struct MarksNavigateHandler;

#[async_trait]
impl ToolHandler for MarksNavigateHandler {
    async fn execute(&self, params: Value) -> Result<Value, String> {
        let path = params["path"].as_str().ok_or("path required")?;
        let line = params["line"].as_u64().ok_or("line required")? as u32;
        
        // This triggers UI navigation
        // In real implementation, this would emit an event or call into the frontend
        let result = crate::modular::execute_module(
            "marks",
            "navigate",
            serde_json::json!({
                "path": path,
                "line": line
            })
        ).map_err(|e| e.to_string())?;
        
        Ok(serde_json::json!({
            "success": true,
            "navigated_to": {
                "path": path,
                "line": line
            },
            "message": "Editor should navigate to this location"
        }))
    }
}
