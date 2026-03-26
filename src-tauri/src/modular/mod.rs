//! Modular system for SAK Editor
//! 
//! Each module exposes its capabilities through JSON interfaces,
//! allowing both internal components and LLMs to interact with them.

#![allow(dead_code)]

use serde::{Serialize, Deserialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::RwLock;
use once_cell::sync::Lazy;

/// Global module registry
static MODULE_REGISTRY: Lazy<RwLock<HashMap<String, Box<dyn Module>>>> = 
    Lazy::new(|| RwLock::new(HashMap::new()));

/// Module capability descriptor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleInfo {
    pub name: String,
    pub version: String,
    pub description: String,
    pub capabilities: Vec<Capability>,
}

/// Individual capability exposed by a module
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Capability {
    pub name: String,
    pub description: String,
    pub input_schema: Value,  // JSON Schema
    pub output_schema: Value, // JSON Schema
}

/// Module trait - all modules implement this
pub trait Module: Send + Sync {
    /// Get module information
    fn info(&self) -> ModuleInfo;
    
    /// Execute a capability with JSON input/output
    fn execute(&self, capability: &str, input: Value) -> Result<Value, ModuleError>;
    
    /// Get module state as JSON
    fn get_state(&self) -> Value;
    
    /// Set module state from JSON
    fn set_state(&mut self, state: Value) -> Result<(), ModuleError>;
}

/// Module execution error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleError {
    pub code: String,
    pub message: String,
    pub details: Option<Value>,
}

impl std::fmt::Display for ModuleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}

impl ModuleError {
    pub fn new(code: &str, message: &str) -> Self {
        ModuleError {
            code: code.to_string(),
            message: message.to_string(),
            details: None,
        }
    }
    
    pub fn with_details(mut self, details: Value) -> Self {
        self.details = Some(details);
        self
    }
}

/// Module registry for managing modules
pub struct ModuleRegistry;

impl ModuleRegistry {
    /// Register a module
    pub fn register(name: &str, module: Box<dyn Module>) -> Result<(), String> {
        let mut registry = MODULE_REGISTRY.write().map_err(|e| e.to_string())?;
        registry.insert(name.to_string(), module);
        Ok(())
    }
    
    /// Get module info
    pub fn get_info(name: &str) -> Option<ModuleInfo> {
        let registry = MODULE_REGISTRY.read().ok()?;
        registry.get(name).map(|m| m.info())
    }
    
    /// List all registered modules
    pub fn list_modules() -> Vec<ModuleInfo> {
        let registry = MODULE_REGISTRY.read().unwrap();
        registry.values().map(|m| m.info()).collect()
    }
    
    /// Execute capability on a module
    pub fn execute(module: &str, capability: &str, input: Value) -> Result<Value, ModuleError> {
        let registry = MODULE_REGISTRY.read().map_err(|e| 
            ModuleError::new("registry_lock", &e.to_string())
        )?;
        
        let module = registry.get(module).ok_or_else(|| 
            ModuleError::new("module_not_found", &format!("Module '{}' not found", module))
        )?;
        
        module.execute(capability, input)
    }
    
    /// Get all capabilities across all modules
    pub fn get_all_capabilities() -> HashMap<String, Vec<Capability>> {
        let registry = MODULE_REGISTRY.read().unwrap();
        
        registry.iter()
            .map(|(name, module)| {
                (name.clone(), module.info().capabilities)
            })
            .collect()
    }
}

/// JSON-RPC style request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleRequest {
    pub module: String,
    pub capability: String,
    pub params: Value,
    pub id: Option<String>,
}

/// JSON-RPC style response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleResponse {
    pub result: Option<Value>,
    pub error: Option<ModuleError>,
    pub id: Option<String>,
}

impl ModuleResponse {
    pub fn success(result: Value, id: Option<String>) -> Self {
        ModuleResponse {
            result: Some(result),
            error: None,
            id,
        }
    }
    
    pub fn error(error: ModuleError, id: Option<String>) -> Self {
        ModuleResponse {
            result: None,
            error: Some(error),
            id,
        }
    }
}

/// Process a module request
pub fn process_request(request: ModuleRequest) -> ModuleResponse {
    match ModuleRegistry::execute(&request.module, &request.capability, request.params) {
        Ok(result) => ModuleResponse::success(result, request.id),
        Err(e) => ModuleResponse::error(e, request.id),
    }
}

/// Execute a module capability (public API)
pub fn execute_module(module: &str, capability: &str, input: Value) -> Result<Value, ModuleError> {
    ModuleRegistry::execute(module, capability, input)
}

/// Helper macro to define module capabilities
#[macro_export]
macro_rules! define_capability {
    ($name:expr, $desc:expr, $input:expr, $output:expr) => {
        Capability {
            name: $name.to_string(),
            description: $desc.to_string(),
            input_schema: serde_json::json!($input),
            output_schema: serde_json::json!($output),
        }
    };
}

/// Initialize all modules
pub fn init_modules() {
    // This will be called at startup to register all modules
    log::info!("Initializing modular system...");
}
