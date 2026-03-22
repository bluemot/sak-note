//! File Module - Large file handling with JSON interface
//! 
//! Now uses VfsManager for unified local/SFTP operations.
//! 
//! Exposed capabilities:
//! - file.open: Open a file for reading/editing
//! - file.close: Close a file
//! - file.read: Read bytes from file
//! - file.read_text: Read text from file
//! - file.insert: Insert bytes
//! - file.delete: Delete bytes
//! - file.replace: Replace bytes
//! - file.save: Save changes
//! - file.save_as: Save to new file
//! - file.undo: Undo last operation
//! - file.redo: Redo last undone operation
//! - file.get_info: Get file information
//! - file.get_hex: Get hex view

use crate::modular::{Module, ModuleInfo, Capability, ModuleError};
use crate::vfs::manager::{VfsManager, VfsFileHandle};
use crate::vfs::EditOp;
use serde::{Serialize, Deserialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Mutex;

/// File info response
#[derive(Debug, Serialize)]
struct FileInfoResponse {
    path: String,
    size: u64,
    effective_size: u64,
    has_changes: bool,
    can_undo: bool,
    can_redo: bool,
}

/// File module implementation using VfsManager
pub struct FileModule {
    handles: Mutex<HashMap<String, VfsFileHandle>>,
}

impl FileModule {
    pub fn new() -> Self {
        FileModule {
            handles: Mutex::new(HashMap::new()),
        }
    }
    
    fn capability_schemas() -> Vec<Capability> {
        vec![
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
                        "effective_size": {"type": "integer"},
                        "has_changes": {"type": "boolean"}
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
            Capability {
                name: "insert".to_string(),
                description: "Insert bytes at position".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "path": {"type": "string"},
                        "offset": {"type": "integer", "minimum": 0},
                        "data": {"type": "array", "items": {"type": "integer"}}
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
                    "required": ["path", "offset", "data"]
                }),
                output_schema: serde_json::json!({"type": "object", "properties": {"success": {"type": "boolean"}}}),
            },
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
            Capability {
                name: "get_hex".to_string(),
                description: "Get hex representation of bytes".to_string(),
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
            version: "2.0.0".to_string(),
            description: "Large file handling via VFS (local/SFTP)".to_string(),
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
            "undo" => self.cmd_undo(input),
            "redo" => self.cmd_redo(input),
            "get_info" => self.cmd_get_info(input),
            "get_hex" => self.cmd_get_hex(input),
            _ => Err(ModuleError::new("unknown_capability", &format!("Unknown: {}", capability))),
        }
    }
    
    fn get_state(&self) -> Value {
        let handles = self.handles.lock().unwrap();
        serde_json::json!({
            "type": "file_module_v2",
            "open_files": handles.len(),
            "files": handles.keys().collect::<Vec<_>>()
        })
    }
    
    fn set_state(&mut self, _state: Value) -> Result<(), ModuleError> {
        Ok(())
    }
}

impl FileModule {
    fn get_handle(&self, path: &str) -> Result<std::sync::MutexGuard<'_, HashMap<String, VfsFileHandle>>, ModuleError> {
        self.handles.lock().map_err(|e| ModuleError::new("lock_error", &e.to_string()))
    }
    
    fn cmd_open(&self, input: Value) -> Result<Value, ModuleError> {
        let path = input["path"].as_str()
            .ok_or_else(|| ModuleError::new("invalid_input", "Missing 'path'"))?;
        
        // Open via VfsManager
        let handle = VfsManager::open(path)
            .map_err(|e| ModuleError::new("open_error", &e.to_string()))?;
        
        let mut handles = self.get_handle(path)?;
        handles.insert(path.to_string(), handle);
        
        let handle = handles.get(path).unwrap();
        
        Ok(serde_json::json!({
            "path": path,
            "size": handle.metadata().map(|m| m.size).unwrap_or(0),
            "effective_size": handle.effective_size(),
            "has_changes": handle.has_changes()
        }))
    }
    
    fn cmd_close(&self, input: Value) -> Result<Value, ModuleError> {
        let path = input["path"].as_str()
            .ok_or_else(|| ModuleError::new("invalid_input", "Missing 'path'"))?;
        
        let mut handles = self.get_handle(path)?;
        handles.remove(path);
        VfsManager::close(path);
        
        Ok(serde_json::Value::Null)
    }
    
    fn cmd_read(&self, input: Value) -> Result<Value, ModuleError> {
        let path = input["path"].as_str()
            .ok_or_else(|| ModuleError::new("invalid_input", "Missing 'path'"))?;
        let offset = input["offset"].as_u64().unwrap_or(0);
        let length = input["length"].as_u64().unwrap_or(1024) as usize;
        
        let mut handles = self.get_handle(path)?;
        let handle = handles.get_mut(path)
            .ok_or_else(|| ModuleError::new("not_open", "File not open"))?;
        
        let data = handle.read_range(offset, length);
        
        Ok(serde_json::json!({
            "data": data,
            "offset": offset,
            "length": data.len()
        }))
    }
    
    fn cmd_read_text(&self, input: Value) -> Result<Value, ModuleError> {
        let path = input["path"].as_str()
            .ok_or_else(|| ModuleError::new("invalid_input", "Missing 'path'"))?;
        let offset = input["offset"].as_u64().unwrap_or(0);
        let length = input["length"].as_u64().unwrap_or(1024) as usize;
        
        let mut handles = self.get_handle(path)?;
        let handle = handles.get_mut(path)
            .ok_or_else(|| ModuleError::new("not_open", "File not open"))?;
        
        let text = handle.read_text(offset, length);
        
        Ok(serde_json::json!({
            "text": text,
            "offset": offset,
            "length": text.len()
        }))
    }
    
    fn cmd_insert(&self, input: Value) -> Result<Value, ModuleError> {
        let path = input["path"].as_str()
            .ok_or_else(|| ModuleError::new("invalid_input", "Missing 'path'"))?;
        let offset = input["offset"].as_u64().unwrap_or(0);
        let data: Vec<u8> = input["data"].as_array()
            .ok_or_else(|| ModuleError::new("invalid_input", "Missing 'data'"))?
            .iter()
            .filter_map(|v| v.as_u64().map(|n| n as u8))
            .collect();
        
        let mut handles = self.get_handle(path)?;
        let handle = handles.get_mut(path)
            .ok_or_else(|| ModuleError::new("not_open", "File not open"))?;
        
        handle.apply_edit(EditOp::Insert { offset, data });
        
        Ok(serde_json::json!({"success": true}))
    }
    
    fn cmd_delete(&self, input: Value) -> Result<Value, ModuleError> {
        let path = input["path"].as_str()
            .ok_or_else(|| ModuleError::new("invalid_input", "Missing 'path'"))?;
        let offset = input["offset"].as_u64().unwrap_or(0);
        let length = input["length"].as_u64().unwrap_or(1);
        
        let mut handles = self.get_handle(path)?;
        let handle = handles.get_mut(path)
            .ok_or_else(|| ModuleError::new("not_open", "File not open"))?;
        
        handle.apply_edit(EditOp::Delete { offset, length });
        
        Ok(serde_json::json!({"success": true}))
    }
    
    fn cmd_replace(&self, input: Value) -> Result<Value, ModuleError> {
        let path = input["path"].as_str()
            .ok_or_else(|| ModuleError::new("invalid_input", "Missing 'path'"))?;
        let offset = input["offset"].as_u64().unwrap_or(0);
        let length = input["length"].as_u64().unwrap_or(0);
        let data: Vec<u8> = input["data"].as_array()
            .ok_or_else(|| ModuleError::new("invalid_input", "Missing 'data'"))?
            .iter()
            .filter_map(|v| v.as_u64().map(|n| n as u8))
            .collect();
        
        let mut handles = self.get_handle(path)?;
        let handle = handles.get_mut(path)
            .ok_or_else(|| ModuleError::new("not_open", "File not open"))?;
        
        // Replace = delete then insert
        if length > 0 {
            handle.apply_edit(EditOp::Delete { offset, length });
        }
        handle.apply_edit(EditOp::Insert { offset, data });
        
        Ok(serde_json::json!({"success": true}))
    }
    
    fn cmd_save(&self, input: Value) -> Result<Value, ModuleError> {
        let path = input["path"].as_str()
            .ok_or_else(|| ModuleError::new("invalid_input", "Missing 'path'"))?;
        
        let mut handles = self.get_handle(path)?;
        let handle = handles.get_mut(path)
            .ok_or_else(|| ModuleError::new("not_open", "File not open"))?;
        
        handle.save()
            .map_err(|e| ModuleError::new("save_error", &e.to_string()))?;
        
        Ok(serde_json::json!({"success": true}))
    }
    
    fn cmd_undo(&self, input: Value) -> Result<Value, ModuleError> {
        let path = input["path"].as_str()
            .ok_or_else(|| ModuleError::new("invalid_input", "Missing 'path'"))?;
        
        let mut handles = self.get_handle(path)?;
        let handle = handles.get_mut(path)
            .ok_or_else(|| ModuleError::new("not_open", "File not open"))?;
        
        let success = handle.undo();
        
        Ok(serde_json::json!({
            "success": success,
            "can_undo": handle.can_undo(),
            "can_redo": handle.can_redo()
        }))
    }
    
    fn cmd_redo(&self, input: Value) -> Result<Value, ModuleError> {
        let path = input["path"].as_str()
            .ok_or_else(|| ModuleError::new("invalid_input", "Missing 'path'"))?;
        
        let mut handles = self.get_handle(path)?;
        let handle = handles.get_mut(path)
            .ok_or_else(|| ModuleError::new("not_open", "File not open"))?;
        
        let success = handle.redo();
        
        Ok(serde_json::json!({
            "success": success,
            "can_undo": handle.can_undo(),
            "can_redo": handle.can_redo()
        }))
    }
    
    fn cmd_get_info(&self, input: Value) -> Result<Value, ModuleError> {
        let path = input["path"].as_str()
            .ok_or_else(|| ModuleError::new("invalid_input", "Missing 'path'"))?;
        
        let mut handles = self.get_handle(path)?;
        let handle = handles.get_mut(path)
            .ok_or_else(|| ModuleError::new("not_open", "File not open"))?;
        
        let size = handle.metadata().map(|m| m.size).unwrap_or(0);
        
        Ok(serde_json::json!({
            "path": path,
            "size": size,
            "effective_size": handle.effective_size(),
            "has_changes": handle.has_changes(),
            "can_undo": handle.can_undo(),
            "can_redo": handle.can_redo()
        }))
    }
    
    fn cmd_get_hex(&self, input: Value) -> Result<Value, ModuleError> {
        let path = input["path"].as_str()
            .ok_or_else(|| ModuleError::new("invalid_input", "Missing 'path'"))?;
        let offset = input["offset"].as_u64().unwrap_or(0);
        let length = input["length"].as_u64().unwrap_or(256) as usize;
        
        let mut handles = self.get_handle(path)?;
        let handle = handles.get_mut(path)
            .ok_or_else(|| ModuleError::new("not_open", "File not open"))?;
        
        let data = handle.read_range(offset, length);
        
        let rows: Vec<serde_json::Value> = data
            .chunks(16)
            .enumerate()
            .map(|(idx, bytes)| {
                let row_offset = offset as usize + idx * 16;
                let hex = bytes
                    .iter()
                    .map(|b| format!("{:02x}", b))
                    .collect::<Vec<_>>()
                    .join(" ");
                let ascii: String = bytes
                    .iter()
                    .map(|b| {
                        if b.is_ascii_graphic() || *b == b' ' {
                            *b as char
                        } else {
                            '.'
                        }
                    })
                    .collect();
                serde_json::json!({
                    "offset": row_offset,
                    "hex": hex,
                    "ascii": ascii
                })
            })
            .collect();
        
        Ok(serde_json::json!({"rows": rows}))
    }
}

/// Register the file module
pub fn register() {
    let module = FileModule::new();
    crate::modular::ModuleRegistry::register("file", Box::new(module))
        .expect("Failed to register file module");
}
