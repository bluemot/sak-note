//! File Module - Large file handling with JSON interface
//! 
//! Exposed capabilities:
//! - file.open: Open a file for reading/editing
//! - file.close: Close a file
//! - file.read: Read bytes from file
//! - file.read_text: Read text from file
//! - file.write: Write/insert bytes
//! - file.delete: Delete bytes
//! - file.replace: Replace bytes
//! - file.save: Save changes
//! - file.save_as: Save to new file
//! - file.undo: Undo last operation
//! - file.redo: Redo last undone operation
//! - file.search: Search for pattern
//! - file.get_info: Get file information
//! - file.get_hex: Get hex view

use crate::modular::{Module, ModuleInfo, Capability, ModuleError};
use crate::file_engine::{FileEngine, EditableFileManager, EditOp};
use serde::{Serialize, Deserialize};
use serde_json::Value;
use std::sync::Arc;
use std::sync::RwLock;

/// File module implementation
pub struct FileModule;

impl FileModule {
    pub fn new() -> Self {
        FileModule
    }
    
    fn capability_schemas() -> Vec<Capability> {
        vec![
            // File operations
            Capability {
                name: "open".to_string(),
                description: "Open a file for editing".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "path": {"type": "string", "description": "Absolute file path"}
                    },
                    "required": ["path"]
                }),
                output_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "path": {"type": "string"},
                        "size": {"type": "integer"},
                        "chunks": {"type": "integer"},
                        "editable": {"type": "boolean"}
                    }
                }),
            },
            Capability {
                name: "close".to_string(),
                description: "Close a file".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "path": {"type": "string"}
                    },
                    "required": ["path"]
                }),
                output_schema: serde_json::json!({"type": "null"}),
            },
            // Read operations
            Capability {
                name: "read".to_string(),
                description: "Read raw bytes from file".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "path": {"type": "string"},
                        "offset": {"type": "integer", "minimum": 0},
                        "length": {"type": "integer", "minimum": 1}
                    },
                    "required": ["path", "offset", "length"]
                }),
                output_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "data": {"type": "array", "items": {"type": "integer"}},
                        "offset": {"type": "integer"},
                        "length": {"type": "integer"}
                    }
                }),
            },
            Capability {
                name: "read_text".to_string(),
                description: "Read text from file (UTF-8)".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "path": {"type": "string"},
                        "offset": {"type": "integer", "minimum": 0},
                        "length": {"type": "integer", "minimum": 1}
                    },
                    "required": ["path", "offset", "length"]
                }),
                output_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "text": {"type": "string"},
                        "offset": {"type": "integer"},
                        "length": {"type": "integer"}
                    }
                }),
            },
            // Edit operations
            Capability {
                name: "insert".to_string(),
                description: "Insert bytes at position".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "path": {"type": "string"},
                        "offset": {"type": "integer", "minimum": 0},
                        "data": {"type": "array", "items": {"type": "integer"}, "description": "Bytes to insert"}
                    },
                    "required": ["path", "offset", "data"]
                }),
                output_schema: serde_json::json!({"type": "object", "properties": {"success": {"type": "boolean"}}}),
            },
            Capability {
                name: "delete".to_string(),
                description: "Delete bytes at position".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "path": {"type": "string"},
                        "offset": {"type": "integer", "minimum": 0},
                        "length": {"type": "integer", "minimum": 1}
                    },
                    "required": ["path", "offset", "length"]
                }),
                output_schema: serde_json::json!({"type": "object", "properties": {"success": {"type": "boolean"}}}),
            },
            Capability {
                name: "replace".to_string(),
                description: "Replace bytes at position".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "path": {"type": "string"},
                        "offset": {"type": "integer", "minimum": 0},
                        "length": {"type": "integer", "minimum": 0},
                        "data": {"type": "array", "items": {"type": "integer"}}
                    },
                    "required": ["path", "offset", "length", "data"]
                }),
                output_schema: serde_json::json!({"type": "object", "properties": {"success": {"type": "boolean"}}}),
            },
            // Save operations
            Capability {
                name: "save".to_string(),
                description: "Save changes to file".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "path": {"type": "string"}
                    },
                    "required": ["path"]
                }),
                output_schema: serde_json::json!({"type": "object", "properties": {"success": {"type": "boolean"}}}),
            },
            Capability {
                name: "save_as".to_string(),
                description: "Save to new file".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "source_path": {"type": "string"},
                        "target_path": {"type": "string"}
                    },
                    "required": ["source_path", "target_path"]
                }),
                output_schema: serde_json::json!({"type": "object", "properties": {"success": {"type": "boolean"}}}),
            },
            // Undo/Redo
            Capability {
                name: "undo".to_string(),
                description: "Undo last operation".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "path": {"type": "string"}
                    },
                    "required": ["path"]
                }),
                output_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "success": {"type": "boolean"},
                        "can_undo": {"type": "boolean"},
                        "can_redo": {"type": "boolean"}
                    }
                }),
            },
            Capability {
                name: "redo".to_string(),
                description: "Redo last undone operation".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "path": {"type": "string"}
                    },
                    "required": ["path"]
                }),
                output_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "success": {"type": "boolean"},
                        "can_undo": {"type": "boolean"},
                        "can_redo": {"type": "boolean"}
                    }
                }),
            },
            // Search
            Capability {
                name: "search".to_string(),
                description: "Search for pattern in file".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "path": {"type": "string"},
                        "pattern": {"type": "string", "description": "Text or hex pattern"},
                        "is_hex": {"type": "boolean", "default": false},
                        "start_offset": {"type": "integer", "default": 0}
                    },
                    "required": ["path", "pattern"]
                }),
                output_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "results": {
                            "type": "array",
                            "items": {
                                "type": "object",
                                "properties": {
                                    "offset": {"type": "integer"},
                                    "length": {"type": "integer"},
                                    "preview": {"type": "string"}
                                }
                            }
                        },
                        "total": {"type": "integer"}
                    }
                }),
            },
            // Info
            Capability {
                name: "get_info".to_string(),
                description: "Get file information".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "path": {"type": "string"}
                    },
                    "required": ["path"]
                }),
                output_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "path": {"type": "string"},
                        "size": {"type": "integer"},
                        "effective_size": {"type": "integer"},
                        "has_changes": {"type": "boolean"},
                        "can_undo": {"type": "boolean"},
                        "can_redo": {"type": "boolean"}
                    }
                }),
            },
            // Hex view
            Capability {
                name: "get_hex".to_string(),
                description: "Get hex representation of bytes".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "path": {"type": "string"},
                        "offset": {"type": "integer", "minimum": 0},
                        "length": {"type": "integer", "minimum": 1, "maximum": 4096}
                    },
                    "required": ["path", "offset", "length"]
                }),
                output_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "rows": {
                            "type": "array",
                            "items": {
                                "type": "object",
                                "properties": {
                                    "offset": {"type": "integer"},
                                    "hex": {"type": "string"},
                                    "ascii": {"type": "string"}
                                }
                            }
                        }
                    }
                }),
            },
        ]
    }
}

impl Module for FileModule {
    fn info(&self) -> ModuleInfo {
        ModuleInfo {
            name: "file".to_string(),
            version: "1.0.0".to_string(),
            description: "Large file handling with memory-mapped I/O and edit journaling".to_string(),
            capabilities: Self::capability_schemas(),
        }
    }
    
    fn execute(&self, capability: &str, input: Value) -> Result<Value, ModuleError> {
        match capability {
            "open" => self.cmd_open(input),
            "close" => self.cmd_close(input),
            "read" => self.cmd_read(input),
            "read_text" => self.cmd_read_text(input),
            "insert" => self.cmd_insert(input),
            "delete" => self.cmd_delete(input),
            "replace" => self.cmd_replace(input),
            "save" => self.cmd_save(input),
            "save_as" => self.cmd_save_as(input),
            "undo" => self.cmd_undo(input),
            "redo" => self.cmd_redo(input),
            "search" => self.cmd_search(input),
            "get_info" => self.cmd_get_info(input),
            "get_hex" => self.cmd_get_hex(input),
            _ => Err(ModuleError::new("unknown_capability", &format!("Unknown capability: {}", capability))),
        }
    }
    
    fn get_state(&self) -> Value {
        // Return list of open files
        let modules: Vec<String> = crate::modular::ModuleRegistry::list_modules()
            .iter()
            .map(|m| m.name.clone())
            .collect();
        serde_json::json!({
            "type": "file_module",
            "loaded": true
        })
    }
    
    fn set_state(&mut self, _state: Value) -> Result<(), ModuleError> {
        Ok(())
    }
}

impl FileModule {
    // Command implementations
    
    fn cmd_open(&self, input: Value) -> Result<Value, ModuleError> {
        let path = input["path"].as_str()
            .ok_or_else(|| ModuleError::new("invalid_input", "Missing 'path'"))?;
        
        match FileEngine::open_for_edit(path) {
            Ok(manager) => {
                let guard = manager.read().map_err(|e| 
                    ModuleError::new("lock_error", &e.to_string())
                )?;
                
                Ok(serde_json::json!({
                    "path": path,
                    "size": guard.effective_size(),
                    "chunks": ((guard.effective_size() + 65535) / 65536) as usize,
                    "editable": true,
                    "has_changes": guard.has_changes()
                }))
            }
            Err(e) => Err(ModuleError::new("open_failed", &e.to_string())),
        }
    }
    
    fn cmd_close(&self, input: Value) -> Result<Value, ModuleError> {
        let path = input["path"].as_str()
            .ok_or_else(|| ModuleError::new("invalid_input", "Missing 'path'"))?;
        
        FileEngine::close_editable(path);
        FileEngine::close_file(path);
        Ok(serde_json::json!(null))
    }
    
    fn cmd_read(&self, input: Value) -> Result<Value, ModuleError> {
        let path = input["path"].as_str()
            .ok_or_else(|| ModuleError::new("invalid_input", "Missing 'path'"))?;
        let offset = input["offset"].as_u64().unwrap_or(0) as usize;
        let length = input["length"].as_u64().unwrap_or(1024) as usize;
        
        if let Some(manager) = FileEngine::get_editable(path) {
            let guard = manager.read().map_err(|e| 
                ModuleError::new("lock_error", &e.to_string())
            )?;
            
            let data = guard.get_range(offset, length);
            Ok(serde_json::json!({
                "data": data,
                "offset": offset,
                "length": data.len()
            }))
        } else {
            Err(ModuleError::new("file_not_open", "File not open"))
        }
    }
    
    fn cmd_read_text(&self, input: Value) -> Result<Value, ModuleError> {
        let path = input["path"].as_str()
            .ok_or_else(|| ModuleError::new("invalid_input", "Missing 'path'"))?;
        let offset = input["offset"].as_u64().unwrap_or(0) as usize;
        let length = input["length"].as_u64().unwrap_or(1024) as usize;
        
        if let Some(manager) = FileEngine::get_editable(path) {
            let guard = manager.read().map_err(|e| 
                ModuleError::new("lock_error", &e.to_string())
            )?;
            
            let text = guard.get_text(offset, length);
            Ok(serde_json::json!({
                "text": text,
                "offset": offset,
                "length": text.len()
            }))
        } else {
            Err(ModuleError::new("file_not_open", "File not open"))
        }
    }
    
    fn cmd_insert(&self, input: Value) -> Result<Value, ModuleError> {
        let path = input["path"].as_str()
            .ok_or_else(|| ModuleError::new("invalid_input", "Missing 'path'"))?;
        let offset = input["offset"].as_u64().unwrap_or(0) as usize;
        let data: Vec<u8> = input["data"].as_array()
            .ok_or_else(|| ModuleError::new("invalid_input", "Missing 'data'"))?
            .iter()
            .filter_map(|v| v.as_u64().map(|n| n as u8))
            .collect();
        
        if let Some(manager) = FileEngine::get_editable(path) {
            let mut guard = manager.write().map_err(|e| 
                ModuleError::new("lock_error", &e.to_string())
            )?;
            
            guard.apply_edit(EditOp::Insert { offset, data });
            Ok(serde_json::json!({"success": true}))
        } else {
            Err(ModuleError::new("file_not_open", "File not open"))
        }
    }
    
    fn cmd_delete(&self, input: Value) -> Result<Value, ModuleError> {
        let path = input["path"].as_str()
            .ok_or_else(|| ModuleError::new("invalid_input", "Missing 'path'"))?;
        let offset = input["offset"].as_u64().unwrap_or(0) as usize;
        let length = input["length"].as_u64().unwrap_or(1) as usize;
        
        if let Some(manager) = FileEngine::get_editable(path) {
            let mut guard = manager.write().map_err(|e| 
                ModuleError::new("lock_error", &e.to_string())
            )?;
            
            guard.apply_edit(EditOp::Delete { offset, length });
            Ok(serde_json::json!({"success": true}))
        } else {
            Err(ModuleError::new("file_not_open", "File not open"))
        }
    }
    
    fn cmd_replace(&self, input: Value) -> Result<Value, ModuleError> {
        let path = input["path"].as_str()
            .ok_or_else(|| ModuleError::new("invalid_input", "Missing 'path'"))?;
        let offset = input["offset"].as_u64().unwrap_or(0) as usize;
        let length = input["length"].as_u64().unwrap_or(0) as usize;
        let data: Vec<u8> = input["data"].as_array()
            .ok_or_else(|| ModuleError::new("invalid_input", "Missing 'data'"))?
            .iter()
            .filter_map(|v| v.as_u64().map(|n| n as u8))
            .collect();
        
        if let Some(manager) = FileEngine::get_editable(path) {
            let mut guard = manager.write().map_err(|e| 
                ModuleError::new("lock_error", &e.to_string())
            )?;
            
            guard.apply_edit(EditOp::Replace { offset, length, data });
            Ok(serde_json::json!({"success": true}))
        } else {
            Err(ModuleError::new("file_not_open", "File not open"))
        }
    }
    
    fn cmd_save(&self, input: Value) -> Result<Value, ModuleError> {
        let path = input["path"].as_str()
            .ok_or_else(|| ModuleError::new("invalid_input", "Missing 'path'"))?;
        
        if let Some(manager) = FileEngine::get_editable(path) {
            let mut guard = manager.write().map_err(|e| 
                ModuleError::new("lock_error", &e.to_string())
            )?;
            
            match guard.save() {
                Ok(_) => Ok(serde_json::json!({"success": true})),
                Err(e) => Err(ModuleError::new("save_failed", &e.to_string())),
            }
        } else {
            Err(ModuleError::new("file_not_open", "File not open"))
        }
    }
    
    fn cmd_save_as(&self, input: Value) -> Result<Value, ModuleError> {
        let source = input["source_path"].as_str()
            .ok_or_else(|| ModuleError::new("invalid_input", "Missing 'source_path'"))?;
        let target = input["target_path"].as_str()
            .ok_or_else(|| ModuleError::new("invalid_input", "Missing 'target_path'"))?;
        
        if let Some(manager) = FileEngine::get_editable(source) {
            let guard = manager.read().map_err(|e| 
                ModuleError::new("lock_error", &e.to_string())
            )?;
            
            match guard.save_as(target) {
                Ok(_) => Ok(serde_json::json!({"success": true})),
                Err(e) => Err(ModuleError::new("save_failed", &e.to_string())),
            }
        } else {
            Err(ModuleError::new("file_not_open", "File not open"))
        }
    }
    
    fn cmd_undo(&self, input: Value) -> Result<Value, ModuleError> {
        let path = input["path"].as_str()
            .ok_or_else(|| ModuleError::new("invalid_input", "Missing 'path'"))?;
        
        if let Some(manager) = FileEngine::get_editable(path) {
            let mut guard = manager.write().map_err(|e| 
                ModuleError::new("lock_error", &e.to_string())
            )?;
            
            let success = guard.undo();
            Ok(serde_json::json!({
                "success": success,
                "can_undo": guard.can_undo(),
                "can_redo": guard.can_redo()
            }))
        } else {
            Err(ModuleError::new("file_not_open", "File not open"))
        }
    }
    
    fn cmd_redo(&self, input: Value) -> Result<Value, ModuleError> {
        let path = input["path"].as_str()
            .ok_or_else(|| ModuleError::new("invalid_input", "Missing 'path'"))?;
        
        if let Some(manager) = FileEngine::get_editable(path) {
            let mut guard = manager.write().map_err(|e| 
                ModuleError::new("lock_error", &e.to_string())
            )?;
            
            let success = guard.redo();
            Ok(serde_json::json!({
                "success": success,
                "can_undo": guard.can_undo(),
                "can_redo": guard.can_redo()
            }))
        } else {
            Err(ModuleError::new("file_not_open", "File not open"))
        }
    }
    
    fn cmd_search(&self, input: Value) -> Result<Value, ModuleError> {
        let path = input["path"].as_str()
            .ok_or_else(|| ModuleError::new("invalid_input", "Missing 'path'"))?;
        let pattern = input["pattern"].as_str()
            .ok_or_else(|| ModuleError::new("invalid_input", "Missing 'pattern'"))?;
        let is_hex = input["is_hex"].as_bool().unwrap_or(false);
        let start_offset = input["start_offset"].as_u64().unwrap_or(0) as usize;
        
        if let Some(manager) = FileEngine::get_editable(path) {
            let guard = manager.read().map_err(|e| 
                ModuleError::new("lock_error", &e.to_string())
            )?;
            
            let results = if is_hex {
                let pattern_bytes: Vec<u8> = pattern.split_whitespace()
                    .filter_map(|s| u8::from_str_radix(s, 16).ok())
                    .collect();
                guard.search(&pattern_bytes, start_offset)
            } else {
                guard.search_text(pattern, start_offset)
            };
            
            let search_results: Vec<Value> = results.iter().map(|&offset| {
                let preview_data = guard.get_range(offset.saturating_sub(16), pattern.len() + 32);
                let preview = String::from_utf8_lossy(&preview_data).to_string();
                serde_json::json!({
                    "offset": offset,
                    "length": pattern.len(),
                    "preview": preview
                })
            }).collect();
            
            Ok(serde_json::json!({
                "results": search_results,
                "total": search_results.len()
            }))
        } else {
            Err(ModuleError::new("file_not_open", "File not open"))
        }
    }
    
    fn cmd_get_info(&self, input: Value) -> Result<Value, ModuleError> {
        let path = input["path"].as_str()
            .ok_or_else(|| ModuleError::new("invalid_input", "Missing 'path'"))?;
        
        if let Some(manager) = FileEngine::get_editable(path) {
            let guard = manager.read().map_err(|e| 
                ModuleError::new("lock_error", &e.to_string())
            )?;
            
            Ok(serde_json::json!({
                "path": path,
                "size": guard.file_size(),
                "effective_size": guard.effective_size(),
                "has_changes": guard.has_changes(),
                "can_undo": guard.can_undo(),
                "can_redo": guard.can_redo()
            }))
        } else {
            Err(ModuleError::new("file_not_open", "File not open"))
        }
    }
    
    fn cmd_get_hex(&self, input: Value) -> Result<Value, ModuleError> {
        let path = input["path"].as_str()
            .ok_or_else(|| ModuleError::new("invalid_input", "Missing 'path'"))?;
        let offset = input["offset"].as_u64().unwrap_or(0) as usize;
        let length = input["length"].as_u64().unwrap_or(256) as usize;
        let length = length.min(4096); // Limit to 4KB
        
        if let Some(manager) = FileEngine::get_editable(path) {
            let guard = manager.read().map_err(|e| 
                ModuleError::new("lock_error", &e.to_string())
            )?;
            
            let data = guard.get_range(offset, length);
            let rows: Vec<Value> = data.chunks(16).enumerate().map(|(idx, bytes)| {
                let row_offset = offset + idx * 16;
                let hex = bytes.iter()
                    .map(|b| format!("{:02x}", b))
                    .collect::<Vec<_>>()
                    .join(" ");
                let ascii: String = bytes.iter()
                    .map(|b| if b.is_ascii_graphic() || *b == b' ' { *b as char } else { '.' })
                    .collect();
                serde_json::json!({
                    "offset": row_offset,
                    "hex": hex,
                    "ascii": ascii
                })
            }).collect();
            
            Ok(serde_json::json!({"rows": rows}))
        } else {
            Err(ModuleError::new("file_not_open", "File not open"))
        }
    }
}

/// Register the file module
pub fn register() {
    let module = FileModule::new();
    crate::modular::ModuleRegistry::register("file", Box::new(module))
        .expect("Failed to register file module");
}
