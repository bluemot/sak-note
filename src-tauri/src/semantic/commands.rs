//! Tauri Commands for Semantic Module
//!
//! Exposes LLM-friendly operations to the frontend

use serde_json::Value;
use crate::semantic::{SemanticDocument, SemanticEdit};
use crate::semantic::blocks::BlockId;
use crate::semantic::parser::CodeParser;
use crate::semantic::query::QueryEngine;
use crate::semantic::bridge::{LLMBridge, LLMEditParser};
use crate::semantic::conversation::{Conversation, ConversationManager, ConversationMessage};
use std::sync::Mutex;
use lazy_static::lazy_static;

/// Global conversation manager
lazy_static! {
    static ref CONVERSATION_MANAGER: Mutex<ConversationManager> = 
        Mutex::new(ConversationManager::new());
}

/// Parse a file into semantic blocks
#[tauri::command]
pub fn semantic_parse_file(path: String) -> Result<Value, String> {
    // Read file content
    let content = std::fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read file: {}", e))?;
    
    // Detect language
    let language = CodeParser::detect_language(&path);
    
    // Parse into semantic blocks
    let parser = CodeParser::new(&language);
    let result = parser.parse(&content);
    
    // Create semantic document
    let mut doc = SemanticDocument::new(path.clone(), language);
    doc.blocks = result.blocks;
    
    // Generate summary
    doc.summary = format!(
        "{} file with {} blocks",
        doc.language,
        doc.blocks.len()
    );
    
    // Return as JSON
    Ok(serde_json::json!({
        "path": doc.path,
        "language": doc.language,
        "summary": doc.summary,
        "block_count": doc.blocks.len(),
        "blocks": doc.blocks.iter().map(|b| {
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
        "errors": result.errors.iter().map(|e| {
            serde_json::json!({
                "line": e.line,
                "message": &e.message,
            })
        }).collect::<Vec<_>>(),
    }))
}

/// Query code using natural language
#[tauri::command]
pub fn semantic_query(path: String, query: String) -> Result<Value, String> {
    // Get or create document
    let content = std::fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read file: {}", e))?;
    
    let language = CodeParser::detect_language(&path);
    let parser = CodeParser::new(&language);
    let result = parser.parse(&content);
    
    let doc = SemanticDocument {
        path: path.clone(),
        language,
        blocks: result.blocks,
        relationships: vec![],
        summary: String::new(),
    };
    
    // Execute query
    let engine = QueryEngine::new(&doc);
    let matches = engine.execute(&query);
    
    // Return results
    Ok(serde_json::json!({
        "query": query,
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
                "preview": if b.content.len() > 100 {
                    format!("{}...", &b.content[..100])
                } else {
                    b.content.clone()
                },
            })
        }).collect::<Vec<_>>(),
    }))
}

/// Export file to LLM-friendly format
#[tauri::command]
pub fn semantic_export_llm(path: String, format: String) -> Result<Value, String> {
    let content = std::fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read file: {}", e))?;
    
    let language = CodeParser::detect_language(&path);
    let parser = CodeParser::new(&language);
    let result = parser.parse(&content);
    
    let doc = SemanticDocument {
        path: path.clone(),
        language,
        blocks: result.blocks,
        relationships: vec![],
        summary: format!("{} file", path),
    };
    
    let output = match format.as_str() {
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
        "format": format,
        "content": output,
    }))
}

/// Apply semantic edit
#[tauri::command]
pub fn semantic_edit(path: String, edit: Value) -> Result<Value, String> {
    let content = std::fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read file: {}", e))?;
    
    let language = CodeParser::detect_language(&path);
    let parser = CodeParser::new(&language);
    let result = parser.parse(&content);
    
    let mut doc = SemanticDocument {
        path: path.clone(),
        language,
        blocks: result.blocks,
        relationships: vec![],
        summary: String::new(),
    };
    
    // Parse edit from JSON
    let edit_type = edit.get("type")
        .and_then(|v| v.as_str())
        .ok_or("Edit type required")?;
    
    let semantic_edit = match edit_type {
        "add_field" => {
            let parent = edit.get("parent")
                .and_then(|v| v.as_str())
                .ok_or("Parent required")?;
            let name = edit.get("name")
                .and_then(|v| v.as_str())
                .ok_or("Name required")?;
            let type_annotation = edit.get("type_annotation")
                .and_then(|v| v.as_str())
                .unwrap_or("String");
            
            SemanticEdit::AddField {
                parent: BlockId::new(parent),
                name: name.to_string(),
                type_annotation: type_annotation.to_string(),
                validation: None,
            }
        }
        _ => return Err(format!("Unknown edit type: {}", edit_type)),
    };
    
    // Apply edit
    doc.apply_edit(semantic_edit)?;
    
    Ok(serde_json::json!({
        "success": true,
        "message": "Edit applied successfully",
    }))
}

/// Parse natural language edit request
#[tauri::command]
pub fn semantic_parse_edit_request(request: String) -> Result<Value, String> {
    let edit_request = LLMEditParser::parse(&request)?;
    
    Ok(serde_json::json!({
        "intent": edit_request.intent,
        "description": edit_request.description,
        "target": edit_request.target,
        "changes": edit_request.changes.iter().map(|c| {
            serde_json::json!({
                "action": c.action,
                "target_block": c.target_block,
                "new_content": c.new_content,
            })
        }).collect::<Vec<_>>(),
    }))
}

/// Start a new conversation
#[tauri::command]
pub fn semantic_conversation_start() -> String {
    let mut manager = CONVERSATION_MANAGER.lock().unwrap();
    let conversation = Conversation::new();
    let id = conversation.id.clone();
    manager.add_conversation(conversation);
    id
}

/// Send message to conversation
#[tauri::command]
pub fn semantic_conversation_send(
    conversation_id: String,
    message: String,
    file_context: Option<String>,
) -> Result<Value, String> {
    let mut manager = CONVERSATION_MANAGER.lock().unwrap();
    
    // Parse file context if provided
    let context = if let Some(path) = file_context {
        let content = std::fs::read_to_string(&path).ok();
        let language = CodeParser::detect_language(&path);
        content.map(|c| (path, c, language))
    } else {
        None
    };
    
    // Generate response based on message first (borrows manager)
    let response = generate_llm_response(&message, context.as_ref());
    let suggestions = generate_suggestions(&message);
    
    // Now add messages - need mutable access
    let conversation = manager
        .get_conversation_mut(&conversation_id)
        .ok_or("Conversation not found")?;
    
    // Add user message
    conversation.add_message(ConversationMessage {
        role: "user".to_string(),
        content: message.clone(),
        timestamp: std::time::SystemTime::now(),
    });
    
    // Add assistant message
    conversation.add_message(ConversationMessage {
        role: "assistant".to_string(),
        content: response.clone(),
        timestamp: std::time::SystemTime::now(),
    });
    
    Ok(serde_json::json!({
        "conversation_id": conversation_id,
        "response": response,
        "suggestions": suggestions,
    }))
}

/// Get conversation history
#[tauri::command]
pub fn semantic_conversation_history(conversation_id: String) -> Result<Value, String> {
    let manager = CONVERSATION_MANAGER.lock().unwrap();
    
    let messages: Vec<serde_json::Value> = manager
        .get_conversation(&conversation_id)
        .ok_or("Conversation not found")?
        .messages
        .iter()
        .map(|m| {
            serde_json::json!({
                "role": m.role,
                "content": m.content,
            })
        }).collect();
    
    Ok(serde_json::json!({
        "messages": messages,
    }))
}

/// Helper: Generate LLM-like response
fn generate_llm_response(
    message: &str,
    _context: Option<&(String, String, String)>,
) -> String {
    let lower = message.to_lowercase();
    
    if lower.contains("hello") || lower.contains("hi") {
        "Hello! I'm your LLM-friendly code assistant. I can help you understand, query, and edit your code using natural language. Try asking me about your code!".to_string()
    } else if lower.contains("find") || lower.contains("where") {
        "I can help you find code. Try using the semantic_query command with natural language like 'find function called main' or 'where is the user struct'".to_string()
    } else if lower.contains("edit") || lower.contains("add") || lower.contains("change") {
        "I can help you edit code. Try describing what you want to change in natural language, like 'add field email of type String to User' or 'extract function calculateTotal'".to_string()
    } else if lower.contains("explain") || lower.contains("what") {
        "I can explain your code. First, parse the file using semantic_parse_file, then I can analyze the semantic blocks for you.".to_string()
    } else {
        "I'm here to help! I can:\n1. Parse your code into semantic blocks\n2. Query code using natural language\n3. Edit code with intent-based operations\n4. Export code in LLM-friendly formats\n\nWhat would you like to do?".to_string()
    }
}

/// Helper: Generate suggestions based on query
fn generate_suggestions(message: &str) -> Vec<String> {
    let lower = message.to_lowercase();
    
    if lower.contains("find") {
        vec![
            "Show all functions".to_string(),
            "Find imports from std".to_string(),
            "Where are the tests?".to_string(),
        ]
    } else if lower.contains("edit") {
        vec![
            "Add field to struct".to_string(),
            "Extract function".to_string(),
            "Rename variable".to_string(),
        ]
    } else {
        vec![
            "Parse this file".to_string(),
            "Export to JSON".to_string(),
            "Show block tree".to_string(),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_edit_request_add_field() {
        let request = "add field email of type String to User".to_string();
        let result = semantic_parse_edit_request(request).unwrap();
        
        assert_eq!(result.get("intent").unwrap().as_str().unwrap(), "add_field");
    }
}
