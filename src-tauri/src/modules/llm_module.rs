//! LLM Module - Chat with LLMs via Ollama API
//! 
//! Supports both local and cloud Ollama endpoints.
//! 
//! Exposed capabilities:
//! - llm.chat: Send a message and get response
//! - llm.list_models: List available Ollama models
//! - llm.get_context/clear_context: Manage conversation history
//! - llm.set_system_prompt: Set system prompt
//! - llm.summarize: Summarize text content
//! - llm.ask_about_file: Ask questions about current file

use crate::modular::{Module, ModuleInfo, Capability, ModuleError};
use serde::{Serialize, Deserialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Ollama API endpoint - supports cloud (ollama.com) or local
const DEFAULT_OLLAMA_URL: &str = "https://ollama.com";

/// Chat message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

/// Ollama API response
#[derive(Debug, Deserialize)]
struct OllamaResponse {
    model: String,
    message: OllamaMessage,
    done: bool,
}

#[derive(Debug, Deserialize)]
struct OllamaMessage {
    role: String,
    content: String,
}

/// Ollama models list response
#[derive(Debug, Deserialize)]
struct OllamaModelsResponse {
    models: Vec<OllamaModel>,
}

#[derive(Debug, Deserialize)]
struct OllamaModel {
    name: String,
    #[serde(default)]
    size: Option<i64>,
    #[serde(default, rename = "modified_at")]
    modified_at: Option<String>,
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
    contexts: Mutex<HashMap<String, ConversationContext>>,
    default_model: Mutex<String>,
    api_url: Mutex<String>,
}

impl LlmModule {
    pub fn new() -> Self {
        LlmModule {
            contexts: Mutex::new(HashMap::new()),
            default_model: Mutex::new("kimi-k2.5:cloud".to_string()),
            api_url: Mutex::new(DEFAULT_OLLAMA_URL.to_string()),
        }
    }
    
    /// Set custom API URL (for cloud or custom server)
    pub fn set_api_url(&self, url: &str) {
        let mut api_url = self.api_url.lock().unwrap();
        *api_url = url.trim_end_matches('/').to_string();
    }
    
    fn capability_schemas() -> Vec<Capability> {
        vec![
            Capability {
                name: "chat".to_string(),
                description: "Send message to LLM and get response".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "message": {"type": "string", "description": "User message"},
                        "model": {"type": "string", "description": "Model name (optional, default: kimi-k2.5:cloud)"},
                        "context_id": {"type": "string", "description": "Context identifier (e.g., file path)", "default": "default"},
                        "stream": {"type": "boolean", "default": false}
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
            Capability {
                name: "list_models".to_string(),
                description: "List available Ollama models".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "api_url": {"type": "string", "description": "Optional custom API URL"}
                    }
                }),
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
                        },
                        "api_url": {"type": "string"}
                    }
                }),
            },
            Capability {
                name: "get_context".to_string(),
                description: "Get conversation context".to_string(),
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
                        "model": {"type": "string"},
                        "messages": {"type": "array"},
                        "message_count": {"type": "integer"},
                        "system_prompt": {"type": ["string", "null"]}
                    }
                }),
            },
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
                    "properties": {"success": {"type": "boolean"}}
                }),
            },
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
                    "properties": {"success": {"type": "boolean"}}
                }),
            },
            Capability {
                name: "summarize".to_string(),
                description: "Summarize text content".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "content": {"type": "string", "description": "Text to summarize"},
                        "max_length": {"type": "integer", "default": 200}
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
            Capability {
                name: "ask_about_file".to_string(),
                description: "Ask questions about file content".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "file_path": {"type": "string"},
                        "question": {"type": "string"},
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
            Capability {
                name: "generate".to_string(),
                description: "Generate text from prompt".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "prompt": {"type": "string"},
                        "template": {"type": "string", "description": "Template type: code, doc, explain"}
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
    
    fn get_or_create_context(&self, context_id: &str) -> ConversationContext {
        let mut contexts = self.contexts.lock().unwrap();
        let default_model = self.default_model.lock().unwrap().clone();
        
        contexts.entry(context_id.to_string()).or_insert_with(|| {
            ConversationContext {
                model: default_model.clone(),
                messages: Vec::new(),
                system_prompt: None,
            }
        }).clone()
    }
    
    fn save_context(&self, context_id: &str, context: ConversationContext) {
        let mut contexts = self.contexts.lock().unwrap();
        contexts.insert(context_id.to_string(), context);
    }
    
    async fn call_ollama_chat(&self, model: &str, messages: Vec<ChatMessage>) -> Result<String, ModuleError> {
        let api_url = self.api_url.lock().unwrap().clone();
        let client = reqwest::Client::new();
        
        let request_body = serde_json::json!({
            "model": model,
            "messages": messages,
            "stream": false
        });
        
        let url = format!("{}/api/chat", api_url);
        
        let response = client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&request_body)
            .timeout(std::time::Duration::from_secs(120))
            .send()
            .await
            .map_err(|e| ModuleError::new("connection_error", &format!("Failed to connect to Ollama: {}", e)))?;
        
        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(ModuleError::new("api_error", &format!("Ollama API error {}: {}", status, text)));
        }
        
        let result: OllamaResponse = response.json().await
            .map_err(|e| ModuleError::new("parse_error", &format!("Failed to parse response: {}", e)))?;
        
        Ok(result.message.content)
    }
}

impl Module for LlmModule {
    fn info(&self) -> ModuleInfo {
        ModuleInfo {
            name: "llm".to_string(),
            version: "1.0.0".to_string(),
            description: "LLM integration via Ollama API (cloud or local)".to_string(),
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
        let api_url = self.api_url.lock().unwrap();
        
        serde_json::json!({
            "type": "llm_module",
            "default_model": *default_model,
            "api_url": *api_url,
            "active_contexts": contexts.len()
        })
    }
    
    fn set_state(&mut self, state: Value) -> Result<(), ModuleError> {
        if let Some(model) = state["default_model"].as_str() {
            let mut default_model = self.default_model.lock().unwrap();
            *default_model = model.to_string();
        }
        if let Some(url) = state["api_url"].as_str() {
            let mut api_url = self.api_url.lock().unwrap();
            *api_url = url.to_string();
        }
        Ok(())
    }
}

impl LlmModule {
    fn cmd_chat(&self, input: Value) -> Result<Value, ModuleError> {
        let message = input["message"].as_str()
            .ok_or_else(|| ModuleError::new("invalid_input", "Missing 'message'"))?;
        let context_id = input["context_id"].as_str().unwrap_or("default");
        
        // Get context and extract all needed data before async calls
        let (model, system_prompt) = {
            let context = self.get_or_create_context(context_id);
            (context.model.clone(), context.system_prompt.clone())
        };
        
        // Build messages
        let mut messages: Vec<ChatMessage> = Vec::new();
        
        if let Some(ref system) = system_prompt {
            messages.push(ChatMessage {
                role: "system".to_string(),
                content: system.clone(),
            });
        }
        
        messages.push(ChatMessage {
            role: "user".to_string(),
            content: message.to_string(),
        });
        
        // Call Ollama synchronously
        let response_text = tauri::async_runtime::block_on(async {
            self.call_ollama_chat(&model, messages).await
        })?;
        
        // Update context with new messages
        {
            let mut contexts = self.contexts.lock().unwrap();
            if let Some(ctx) = contexts.get_mut(context_id) {
                ctx.messages.push(ChatMessage {
                    role: "user".to_string(),
                    content: message.to_string(),
                });
                ctx.messages.push(ChatMessage {
                    role: "assistant".to_string(),
                    content: response_text.clone(),
                });
            }
        }
        
        Ok(serde_json::json!({
            "response": response_text,
            "model": model,
            "done": true
        }))
    }
    
    fn cmd_list_models(&self, input: Value) -> Result<Value, ModuleError> {
        let api_url = input["api_url"].as_str()
            .map(String::from)
            .unwrap_or_else(|| self.api_url.lock().unwrap().clone());
        
        let client = reqwest::Client::new();
        let url = format!("{}/api/tags", api_url);
        
        let response = tauri::async_runtime::block_on(async {
            client.get(&url).timeout(std::time::Duration::from_secs(10)).send().await
        })
        .map_err(|e| ModuleError::new("connection_error", &format!("Failed to connect: {}", e)))?;
        
        if !response.status().is_success() {
            return Err(ModuleError::new("api_error", &format!("API error: {}", response.status())));
        }
        
        let result: OllamaModelsResponse = tauri::async_runtime::block_on(async {
            response.json::<OllamaModelsResponse>().await
        })
        .map_err(|e| ModuleError::new("parse_error", &e.to_string()))?;
        
        // Convert to JSON-friendly format
        let models: Vec<serde_json::Value> = result.models.into_iter().map(|m| {
            serde_json::json!({
                "name": m.name,
                "size": m.size,
                "modified": m.modified_at
            })
        }).collect();
        
        Ok(serde_json::json!({
            "models": models,
            "api_url": api_url
        }))
    }
    
    fn cmd_get_context(&self, input: Value) -> Result<Value, ModuleError> {
        let context_id = input["context_id"].as_str()
            .ok_or_else(|| ModuleError::new("invalid_input", "Missing 'context_id'"))?;
        
        let contexts = self.contexts.lock().unwrap();
        
        if let Some(context) = contexts.get(context_id) {
            Ok(serde_json::json!({
                "model": context.model,
                "messages": context.messages,
                "message_count": context.messages.len(),
                "system_prompt": context.system_prompt
            }))
        } else {
            Ok(serde_json::json!({
                "model": "",
                "messages": [],
                "message_count": 0,
                "system_prompt": null
            }))
        }
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
        
        let mut context = self.get_or_create_context(context_id);
        context.system_prompt = Some(prompt.to_string());
        context.messages.retain(|m| m.role != "system"); // Remove old system messages
        self.save_context(context_id, context);
        
        Ok(serde_json::json!({"success": true}))
    }
    
    fn cmd_summarize(&self, input: Value) -> Result<Value, ModuleError> {
        let content = input["content"].as_str()
            .ok_or_else(|| ModuleError::new("invalid_input", "Missing 'content'"))?;
        let max_length = input["max_length"].as_u64().unwrap_or(200) as usize;
        
        let default_model = self.default_model.lock().unwrap().clone();
        let model = input["model"].as_str().unwrap_or(&default_model);
        let api_url = self.api_url.lock().unwrap().clone();
        
        let prompt = format!(
            "Summarize the following text in {} characters or less:\n\n{}",
            max_length, content
        );
        
        let messages = vec![ChatMessage {
            role: "user".to_string(),
            content: prompt,
        }];
        
        let summary = tauri::async_runtime::block_on(async {
            self.call_ollama_chat(model, messages).await
        })?;
        
        Ok(serde_json::json!({
            "summary": summary,
            "original_length": content.len(),
            "summary_length": summary.len()
        }))
    }
    
    fn cmd_ask_about_file(&self, input: Value) -> Result<Value, ModuleError> {
        let file_path = input["file_path"].as_str()
            .ok_or_else(|| ModuleError::new("invalid_input", "Missing 'file_path'"))?;
        let question = input["question"].as_str()
            .ok_or_else(|| ModuleError::new("invalid_input", "Missing 'question'"))?;
        let context_id = input["context_id"].as_str().unwrap_or("default");
        
        let mut context = self.get_or_create_context(context_id);
        let model = context.model.clone();
        
        // Build context-aware question
        let content = format!(
            "You are analyzing file: {}\n\nUser question: {}\n\nProvide a helpful response based on the file content.",
            file_path, question
        );
        
        context.messages.push(ChatMessage {
            role: "user".to_string(),
            content,
        });
        
        let messages = context.messages.clone();
        
        let response = tauri::async_runtime::block_on(async {
            self.call_ollama_chat(&model, messages).await
        })?;
        
        context.messages.push(ChatMessage {
            role: "assistant".to_string(),
            content: response.clone(),
        });
        self.save_context(context_id, context);
        
        Ok(serde_json::json!({
            "response": response,
            "relevant_sections": []
        }))
    }
    
    fn cmd_generate(&self, input: Value) -> Result<Value, ModuleError> {
        let prompt = input["prompt"].as_str()
            .ok_or_else(|| ModuleError::new("invalid_input", "Missing 'prompt'"))?;
        let template = input["template"].as_str().unwrap_or("general");
        
        let default_model = self.default_model.lock().unwrap().clone();
        let model = input["model"].as_str().unwrap_or(&default_model);
        
        let formatted_prompt = match template {
            "code" => format!("Write code for the following:\n{}", prompt),
            "doc" => format!("Write documentation for:\n{}", prompt),
            "explain" => format!("Explain this:\n{}", prompt),
            _ => prompt.to_string(),
        };
        
        let messages = vec![ChatMessage {
            role: "user".to_string(),
            content: formatted_prompt,
        }];
        
        let generated = tauri::async_runtime::block_on(async {
            self.call_ollama_chat(model, messages).await
        })?;
        
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
