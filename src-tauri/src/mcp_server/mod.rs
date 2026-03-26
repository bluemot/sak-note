//! Model Context Protocol (MCP) Server
//!
//! Allows LLMs to discover and operate all modules through JSON-RPC interface
//!
//! Usage for LLM:
//! 1. Call `list_tools` to see all available operations
//! 2. Call `execute_tool` with natural language or structured parameters
//!
//! Example:
//! ```json
//! {
//!   "tool": "file_module::open",
//!   "params": { "path": "/project/main.rs" }
//! }
//! ```

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

pub mod tools;
pub mod handlers;

pub use tools::{Tool, ToolRegistry};
pub use handlers::ToolHandler;

/// MCP Server instance
pub struct MCPServer {
    registry: ToolRegistry,
    handlers: HashMap<String, Box<dyn ToolHandler>>,
}

impl MCPServer {
    pub fn new() -> Self {
        let mut server = Self {
            registry: ToolRegistry::new(),
            handlers: HashMap::new(),
        };
        server.register_default_tools();
        server
    }

    fn register_default_tools(&mut self) {
        // Register all module tools
        tools::register_file_module_tools(&mut self.registry);
        tools::register_llm_module_tools(&mut self.registry);
        tools::register_sftp_module_tools(&mut self.registry);
        tools::register_semantic_tools(&mut self.registry);
        tools::register_mark_module_tools(&mut self.registry);
        
        // Register handlers
        handlers::register_default_handlers(self);
    }

    /// Register a handler for a tool
    pub fn register_handler(
        &mut self,
        tool_name: &str,
        handler: Box<dyn ToolHandler>,
    ) {
        self.handlers.insert(tool_name.to_string(), handler);
    }

    /// List all available tools for LLM discovery
    pub fn list_tools(&self) -> Vec<&Tool> {
        self.registry.list()
    }

    /// Execute a tool by name
    pub async fn execute(&self, tool_name: &str, params: Value) -> Result<Value, String> {
        match self.handlers.get(tool_name) {
            Some(handler) => handler.execute(params).await,
            None => Err(format!("Tool '{}' not found", tool_name)),
        }
    }

    /// Get tool schema by name
    pub fn get_tool_schema(&self, name: &str) -> Option<&Tool> {
        self.registry.get(name)
    }
}

/// MCP Request from LLM
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "method")]
pub enum MCPRequest {
    #[serde(rename = "list_tools")]
    ListTools,
    
    #[serde(rename = "execute")]
    Execute {
        tool: String,
        params: Value,
    },
    
    #[serde(rename = "describe")]
    Describe { tool: String },
}

/// MCP Response to LLM
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum MCPResponse {
    #[serde(rename = "tools")]
    Tools { tools: Vec<Tool> },
    
    #[serde(rename = "result")]
    Result { result: Value },
    
    #[serde(rename = "error")]
    Error { code: i32, message: String },
    
    #[serde(rename = "description")]
    Description { tool: Tool },
}

impl MCPServer {
    /// Handle MCP request
    pub async fn handle_request(&self, request: MCPRequest) -> MCPResponse {
        match request {
            MCPRequest::ListTools => {
                MCPResponse::Tools {
                    tools: self.list_tools().into_iter().cloned().collect(),
                }
            }
            
            MCPRequest::Execute { tool, params } => {
                match self.execute(&tool, params).await {
                    Ok(result) => MCPResponse::Result { result },
                    Err(msg) => MCPResponse::Error {
                        code: -32600,
                        message: msg,
                    },
                }
            }
            
            MCPRequest::Describe { tool } => {
                match self.get_tool_schema(&tool) {
                    Some(t) => MCPResponse::Description { tool: t.clone() },
                    None => MCPResponse::Error {
                        code: -32601,
                        message: format!("Tool '{}' not found", tool),
                    },
                }
            }
        }
    }
}

/// Global MCP server instance
use std::sync::{Arc, Mutex};
use lazy_static::lazy_static;

lazy_static! {
    static ref MCP_SERVER: Arc<Mutex<MCPServer>> = 
        Arc::new(Mutex::new(MCPServer::new()));
}

/// Get global MCP server
pub fn get_mcp_server() -> Arc<Mutex<MCPServer>> {
    MCP_SERVER.clone()
}

/// Tauri command to expose MCP to frontend
#[tauri::command]
pub async fn mcp_handle_request(request: Value) -> Result<Value, String> {
    let req: MCPRequest = serde_json::from_value(request)
        .map_err(|e| format!("Invalid request: {}", e))?;
    
    let server = MCP_SERVER.lock().map_err(|e| e.to_string())?;
    let response = server.handle_request(req).await;
    
    serde_json::to_value(response)
        .map_err(|e| format!("Serialization error: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mcp_server_creation() {
        let server = MCPServer::new();
        let tools = server.list_tools();
        assert!(!tools.is_empty());
    }

    #[tokio::test]
    async fn test_list_tools_request() {
        let server = MCPServer::new();
        let request = MCPRequest::ListTools;
        let response = server.handle_request(request).await;
        
        match response {
            MCPResponse::Tools { tools } => {
                assert!(!tools.is_empty());
                // Verify tools have proper namespacing
                assert!(tools.iter().any(|t| t.name.starts_with("file_module::")));
                assert!(tools.iter().any(|t| t.name.starts_with("llm_module::")));
            }
            _ => panic!("Expected Tools response"),
        }
    }
}
