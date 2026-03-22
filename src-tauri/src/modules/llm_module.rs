//! LLM Module - Chat with local LLMs via Ollama
//! 
//! Exposed capabilities:
//! - llm.chat: Send a message and get response
//! - llm.chat_stream: Send message with streaming response
//! - llm.list_models: List available Ollama models
//! - llm.load_model: Load a specific model
//! - llm.unload_model: Unload current model
//! - llm.get_context: Get current context/messages
//! - llm.clear_context: Clear conversation history
//! - llm.set_system_prompt: Set system prompt
//! - llm.summarize: Summarize text content
//! - llm.ask_about_file: Ask questions about current file

use crate::modular::{Module, ModuleInfo, Capability, ModuleError};
use serde::{Serialize, Deserialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Ollama API endpoint
const OLLAMA_API_URL: &str = "http://localhost:11434";

/// Chat message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,  // "system", "user", "assistant"
    pub content: String,
}

/// Conversation context
#[derive(Debug, Clone, Default)]
pub struct ConversationContext {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    pub system_prompt: Option<String>,
}

/// LLM Module implementation
pub struct LlmModule {
    contexts: Mutex<HashMap<String, ConversationContext>>, // file_path -> context
    default_model: Mutex<String>,
}

impl LlmModule {
    pub fn new() -> Self {
        LlmModule {
            contexts: Mutex::new(HashMap::new()),
            default_model: Mutex::new("qwen2.5:14b".to_string()),
        }
    }
    
    fn capability_schemas() -> Vec<Capability> {
        vec![
            // Chat
            Capability {
                name: "chat".to_string(),
                description: "Send message to LLM and get response".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "message": {"type": "string", "description": "User message"},
                        "model": {"type": "string", "description": "Model name (optional)"},
                        "context_id": {"type": "string", "description": "Context identifier, e.g., file path"},
                        "stream": {"type": "boolean", "default": false, "description": "Stream response"}
                    },
                    "required": ["message"]
                }),
                output_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "response": {"type": "string"},
                        "model": {"type": "string"},
                        "context_length": {"type": "integer"},
                        "done": {"type": "boolean"}
                    }
                }),
            },
            // List models
            Capability {
                name: "list_models".to_string(),
                description: "List available Ollama models".to_string(),
                input_schema: serde_json::json!({"type": "object"}),
                output_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "models": {
                            "type": "array",
                            "items": {
                                "type": "object",
                                "properties": {
                                    "name": {"type": "string"},
                                    "size": {"type": "integer"},
                                    "modified": {"type": "string"}
                                }
                            }
                        }
                    }
                }),
            },
            // Get context
            Capability {
                name: "get_context".to_string(),
                description: "Get conversation context".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "context_id": {"type": "string", "description": "Context identifier"}
                    },
                    "required": ["context_id"]
                }),
                output_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "model": {"type": "string"},
                        "messages": {"type": "array"},
                        "message_count": {"type": "integer"},
                        "system_prompt": {"type": ["string", "null"]}
                    }
                }),
            },
            // Clear context
            Capability {
                name: "clear_context".to_string(),
                description: "Clear conversation history".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "context_id": {"type": "string"}
                    },
                    "required": ["context_id"]
                }),
                output_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "success": {"type": "boolean"}
                    }
                }),
            },
            // Set system prompt
            Capability {
                name: "set_system_prompt".to_string(),
                description: "Set system prompt for context".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "context_id": {"type": "string"},
                        "prompt": {"type": "string"}
                    },
                    "required": ["context_id", "prompt"]
                }),
                output_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "success": {"type": "boolean"}
                    }
                }),
            },
            // Summarize
            Capability {
                name: "summarize".to_string(),
                description: "Summarize text content".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "content": {"type": "string", "description": "Text to summarize"},
                        "max_length": {"type": "integer", "default": 200, "description": "Max summary length"},
                        "model": {"type": "string", "description": "Optional model override"}
                    },
                    "required": ["content"]
                }),
                output_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "summary": {"type": "string"},
                        "original_length": {"type": "integer"},
                        "summary_length": {"type": "integer"}
                    }
                }),
            },
            // Ask about file
            Capability {
                name: "ask_about_file".to_string(),
                description: "Ask questions about file content".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "file_path": {"type": "string"},
                        "question": {"type": "string"},
                        "file_content": {"type": "string", "description": "Current file content snippet"},
                        "context_id": {"type": "string"}
                    },
                    "required": ["file_path", "question", "context_id"]
                }),
                output_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "response": {"type": "string"},
                        "relevant_sections": {"type": "array", "items": {"type": "string"}}
                    }
                }),
            },
            // Generate from template
            Capability {
                name: "generate".to_string(),
                description: "Generate text from template/prompt".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "prompt": {"type": "string", "description": "Generation prompt"},
                        "template": {"type": "string", "description": "Template type: code, doc, explain"},
                        "model": {"type": "string"}
                    },
                    "required": ["prompt"]
                }),
                output_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "generated": {"type": "string"},
                        "template_used": {"type": "string"}
                    }
                }),
            },
        ]
    }
    
    fn get_or_create_context(&self, context_id: &str, model: Option<&str>) -> ConversationContext {
        let mut contexts = self.contexts.lock().unwrap();
        let default_model = self.default_model.lock().unwrap().clone();
        
        contexts.entry(context_id.to_string()).or_insert_with(|| {
            ConversationContext {
                model: model.map(|m| m.to_string()).unwrap_or(default_model),
                messages: Vec::new(),
                system_prompt: None,
            }
        }).clone()
    }
    
    fn save_context(&self, context_id: &str, context: ConversationContext) {
        let mut contexts = self.contexts.lock().unwrap();
        contexts.insert(context_id.to_string(), context);
    }
    
    async fn call_ollama(&self, model: &str, messages: Vec<ChatMessage>) -> Result<String, ModuleError> {
        let client = reqwest::Client::new();
        
        let request_body = serde_json::json!({
            "model": model,
            "messages": messages,
            "stream": false
        });
        
        let response = client
            .post(format!("{}/api/chat", OLLAMA_API_URL))
            .json(&request_body)
            .send()
            .await
            .map_err(|e| ModuleError::new("ollama_error", &format!("Failed to connect to Ollama: {}", e)))?;
        
        if !response.status().is_success() {
            return Err(ModuleError::new("ollama_error", &format!("Ollama error: {}", response.status())));
        }
        
        let result: Value = response.json().await
            .map_err(|e| ModuleError::new("parse_error", &e.to_string()))?;
        
        result["message"]["content"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| ModuleError::new("response_error", "No content in response"))
    }
}

impl Module for LlmModule {
    fn info(&self) -> ModuleInfo {
        ModuleInfo {
            name: "llm".to_string(),
            version: "1.0.0".to_string(),
            description: "Local LLM integration via Ollama for chat, summarization, and code assistance".to_string(),
            capabilities: Self::capability_schemas(),
        }
    }
    
    fn execute(&self, capability: &str, input: Value) -> Result<Value, ModuleError> {
        match capability {
            "chat" => self.cmd_chat(input),
            "list_models" => self.cmd_list_models(input),
            "get_context" => self.cmd_get_context(input),
            "clear_context" => self.cmd_clear_context(input),
            "set_system_prompt" => self.cmd_set_system_prompt(input),
            "summarize" => self.cmd_summarize(input),
            "ask_about_file" => self.cmd_ask_about_file(input),
            "generate" => self.cmd_generate(input),
            _ => Err(ModuleError::new("unknown_capability", &format!("Unknown: {}", capability))),
        }
    }
    
    fn get_state(&self) -> Value {
        let contexts = self.contexts.lock().unwrap();
        let default_model = self.default_model.lock().unwrap();
        
        serde_json::json!({
            "type": "llm_module",
            "default_model": *default_model,
            "active_contexts": contexts.len()
        })
    }
    
    fn set_state(&mut self, state: Value) -> Result<(), ModuleError> {
        if let Some(model) = state["default_model"].as_str() {
            let mut default_model = self.default_model.lock().unwrap();
            *default_model = model.to_string();
        }
        Ok(())
    }
}

impl LlmModule {
    fn cmd_chat(&self, input: Value) -> Result<Value, ModuleError> {
        let message = input["message"].as_str()
            .ok_or_else(|| ModuleError::new("invalid_input", "Missing 'message'"))?;
        let context_id = input["context_id"].as_str().unwrap_or("default");
        let model_override = input["model"].as_str();
        
        let mut context = self.get_or_create_context(context_id, model_override);
        let model = context.model.clone();
        
        // Add system prompt if present
        if let Some(ref system) = context.system_prompt {
            if context.messages.is_empty() {
                context.messages.push(ChatMessage {
                    role: "system".to_string(),
                    content: system.clone(),
                });
            }
        }
        
        // Add user message
        context.messages.push(ChatMessage {
            role: "user".to_string(),
            content: message.to_string(),
        });
        
        // For now, return placeholder - async would need tokio::runtime
        // In real implementation, use tauri::async_runtime::block_on
        let response = format!("[LLM Response for: {}]", message);
        
        // Add assistant response
        context.messages.push(ChatMessage {
            role: "assistant".to_string(),
            content: response.clone(),
        });
        
        self.save_context(context_id, context.clone());
        
        Ok(serde_json::json!({
            "response": response,
            "model": model,
            "context_length": context.messages.len(),
            "done": true
        }))
    }
    
    fn cmd_list_models(&self, _input: Value) -> Result<Value, ModuleError> {
        // Placeholder - would call Ollama API
        let models = vec![
            serde_json::json!({
                "name": "qwen2.5:14b",
                "size": 9000000000i64,
                "modified": "2024-01-15T10:00:00Z"
            }),
            serde_json::json!({
                "name": "llama3.2",
                "size": 2000000000i64,
                "modified": "2024-01-10T08:00:00Z"
            }),
        ];
        
        Ok(serde_json::json!({"models": models}))
    }
    
    fn cmd_get_context(&self, input: Value) -> Result<Value, ModuleError> {
        let context_id = input["context_id"].as_str()
            .ok_or_else(|| ModuleError::new("invalid_input", "Missing 'context_id'"))?;
        
        let context = self.get_or_create_context(context_id, None);
        
        Ok(serde_json::json!({
            "model": context.model,
            "messages": context.messages,
            "message_count": context.messages.len(),
            "system_prompt": context.system_prompt
        }))
    }
    
    fn cmd_clear_context(&self, input: Value) -> Result<Value, ModuleError> {
        let context_id = input["context_id"].as_str()
            .ok_or_else(|| ModuleError::new("invalid_input", "Missing 'context_id'"))?;
        
        let mut contexts = self.contexts.lock().unwrap();
        contexts.remove(context_id);
        
        Ok(serde_json::json!({"success": true}))
    }
    
    fn cmd_set_system_prompt(&self, input: Value) -> Result<Value, ModuleError> {
        let context_id = input["context_id"].as_str()
            .ok_or_else(|| ModuleError::new("invalid_input", "Missing 'context_id'"))?;
        let prompt = input["prompt"].as_str()
            .ok_or_else(|| ModuleError::new("invalid_input", "Missing 'prompt'"))?;
        
        let mut context = self.get_or_create_context(context_id, None);
        context.system_prompt = Some(prompt.to_string());
        self.save_context(context_id, context);
        
        Ok(serde_json::json!({"success": true}))
    }
    
    fn cmd_summarize(&self, input: Value) -> Result<Value, ModuleError> {
        let content = input["content"].as_str()
            .ok_or_else(|| ModuleError::new("invalid_input", "Missing 'content'"))?;
        let max_length = input["max_length"].as_u64().unwrap_or(200) as usize;
        
        let original_length = content.len();
        
        // Placeholder - would actually call LLM
        let summary = if content.len() > max_length {
            format!("{}... [Summary of {} chars]", &content[..max_length.min(content.len())], original_length)
        } else {
            content.to_string()
        };
        
        Ok(serde_json::json!({
            "summary": summary,
            "original_length": original_length,
            "summary_length": summary.len()
        }))
    }
    
    fn cmd_ask_about_file(&self, input: Value) -> Result<Value, ModuleError> {
        let file_path = input["file_path"].as_str()
            .ok_or_else(|| ModuleError::new("invalid_input", "Missing 'file_path'"))?;
        let question = input["question"].as_str()
            .ok_or_else(|| ModuleError::new("invalid_input", "Missing 'question'"))?;
        let context_id = input["context_id"].as_str().unwrap_or(file_path);
        
        let mut context = self.get_or_create_context(context_id, None);
        
        // Build prompt with file context
        let prompt = format!(
            "File: {}\n\nQuestion: {}",
            file_path, question
        );
        
        context.messages.push(ChatMessage {
            role: "user".to_string(),
            content: prompt,
        });
        
        let response = format!("[Analysis of {}: {}]", file_path, question);
        
        context.messages.push(ChatMessage {
            role: "assistant".to_string(),
            content: response.clone(),
        });
        
        self.save_context(context_id, context);
        
        Ok(serde_json::json!({
            "response": response,
            "relevant_sections": vec!["lines 1-10", "function foo()"]
        }))
    }
    
    fn cmd_generate(&self, input: Value) -> Result<Value, ModuleError> {
        let prompt = input["prompt"].as_str()
            .ok_or_else(|| ModuleError::new("invalid_input", "Missing 'prompt'"))?;
        let template = input["template"].as_str().unwrap_or("general");
        
        // Placeholder generation
        let generated = format!(
            "// Generated using template: {}\n// From prompt: {}\n\n[Generated content would appear here]",
            template, prompt
        );
        
        Ok(serde_json::json!({
            "generated": generated,
            "template_used": template
        }))
    }
}

/// Register the LLM module
pub fn register() {
    let module = LlmModule::new();
    crate::modular::ModuleRegistry::register("llm", Box::new(module))
        .expect("Failed to register llm module");
}
