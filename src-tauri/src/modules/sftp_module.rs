//! SFTP Module - Remote file access via SSH/SFTP
//! 
//! Uses VfsManager for unified file operations.
//! Connection management separate from file operations.
//!
//! Exposed capabilities:
//! - sftp.connect: Connect to SSH server
//! - sftp.disconnect: Close connection
//! - sftp.open: Open remote file
//! - sftp.close: Close remote file
//! - sftp.read/read_text: Read bytes/text
//! - sftp.write: Write bytes
//! - sftp.list_dir: List directory
//! - sftp.stat: Get file info
//! - sftp.mkdir/rmdir: Directory ops
//! - sftp.unlink: Delete file

use crate::modular::{Module, ModuleInfo, Capability, ModuleError};
use crate::vfs::manager::VfsManager;
use crate::vfs::{VfsBackend, VfsMetadata, EditOp};
use serde::{Serialize, Deserialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// SSH connection config
#[derive(Clone)]
pub struct SshConnection {
    pub hostname: String,
    pub port: u16,
    pub username: String,
}

/// SFTP session wrapping a backend
struct SftpSession {
    backend: Box<dyn VfsBackend>,
    path: String,
}

/// SFTP Module implementation
pub struct SftpModule {
    connections: RwLock<HashMap<String, SshConnection>>,
    sessions: RwLock<HashMap<String, SftpSession>>,
}

impl SftpModule {
    pub fn new() -> Self {
        SftpModule {
            connections: RwLock::new(HashMap::new()),
            sessions: RwLock::new(HashMap::new()),
        }
    }
    
    fn capability_schemas() -> Vec<Capability> {
        vec![
            Capability {
                name: "connect".to_string(),
                description: "Connect to SSH server".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "connection_id": {"type": "string"},
                        "hostname": {"type": "string"},
                        "port": {"type": "integer", "default": 22},
                        "username": {"type": "string"},
                        "password": {"type": "string"}
                    },
                    "required": ["connection_id", "hostname", "username"]
                }),
                output_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "success": {"type": "boolean"},
                        "connection_id": {"type": "string"},
                        "sftp_version": {"type": "integer"}
                    }
                }),
            },
            Capability {
                name: "disconnect".to_string(),
                description: "Close SSH connection".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "connection_id": {"type": "string"}
                    },
                    "required": ["connection_id"]
                }),
                output_schema: serde_json::json!({"type": "object", "properties": {"success": {"type": "boolean"}}}),
            },
            Capability {
                name: "open".to_string(),
                description: "Open remote file".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "connection_id": {"type": "string"},
                        "remote_path": {"type": "string"}
                    },
                    "required": ["connection_id", "remote_path"]
                }),
                output_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "file_handle": {"type": "string"},
                        "size": {"type": "integer"}
                    }
                }),
            },
            Capability {
                name: "close".to_string(),
                description: "Close remote file".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "file_handle": {"type": "string"}
                    },
                    "required": ["file_handle"]
                }),
                output_schema: serde_json::json!({"type": "object", "properties": {"success": {"type": "boolean"}}}),
            },
            Capability {
                name: "read".to_string(),
                description: "Read bytes from file".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "file_handle": {"type": "string"},
                        "offset": {"type": "integer", "minimum": 0},
                        "length": {"type": "integer", "minimum": 1, "maximum": 1048576}
                    },
                    "required": ["file_handle", "offset", "length"]
                }),
                output_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "data": {"type": "array", "items": {"type": "integer"}},
                        "bytes_read": {"type": "integer"}
                    }
                }),
            },
            Capability {
                name: "read_text".to_string(),
                description: "Read text from file".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "file_handle": {"type": "string"},
                        "offset": {"type": "integer", "minimum": 0},
                        "length": {"type": "integer", "minimum": 1}
                    },
                    "required": ["file_handle", "offset", "length"]
                }),
                output_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "text": {"type": "string"},
                        "bytes_read": {"type": "integer"}
                    }
                }),
            },
            Capability {
                name: "write".to_string(),
                description: "Write bytes to file".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "file_handle": {"type": "string"},
                        "offset": {"type": "integer", "minimum": 0},
                        "data": {"type": "array", "items": {"type": "integer"}}
                    },
                    "required": ["file_handle", "data"]
                }),
                output_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "bytes_written": {"type": "integer"},
                        "success": {"type": "boolean"}
                    }
                }),
            },
            Capability {
                name: "list_dir".to_string(),
                description: "List directory contents".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "connection_id": {"type": "string"},
                        "remote_path": {"type": "string"}
                    },
                    "required": ["connection_id", "remote_path"]
                }),
                output_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "entries": {"type": "array"},
                        "count": {"type": "integer"}
                    }
                }),
            },
            Capability {
                name: "stat".to_string(),
                description: "Get file info".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "connection_id": {"type": "string"},
                        "remote_path": {"type": "string"}
                    },
                    "required": ["connection_id", "remote_path"]
                }),
                output_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "exists": {"type": "boolean"},
                        "size": {"type": "integer"},
                        "is_file": {"type": "boolean"},
                        "is_dir": {"type": "boolean"}
                    }
                }),
            },
            Capability {
                name: "mkdir".to_string(),
                description: "Create directory".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "connection_id": {"type": "string"},
                        "remote_path": {"type": "string"}
                    },
                    "required": ["connection_id", "remote_path"]
                }),
                output_schema: serde_json::json!({"type": "object", "properties": {"success": {"type": "boolean"}}}),
            },
            Capability {
                name: "rmdir".to_string(),
                description: "Remove directory".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "connection_id": {"type": "string"},
                        "remote_path": {"type": "string"}
                    },
                    "required": ["connection_id", "remote_path"]
                }),
                output_schema: serde_json::json!({"type": "object", "properties": {"success": {"type": "boolean"}}}),
            },
            Capability {
                name: "unlink".to_string(),
                description: "Delete file".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "connection_id": {"type": "string"},
                        "remote_path": {"type": "string"}
                    },
                    "required": ["connection_id", "remote_path"]
                }),
                output_schema: serde_json::json!({"type": "object", "properties": {"success": {"type": "boolean"}}}),
            },
        ]
    }
}

impl Module for SftpModule {
    fn info(&self) -> ModuleInfo {
        ModuleInfo {
            name: "sftp".to_string(),
            version: "2.0.0".to_string(),
            description: "SSH/SFTP remote file access via VFS".to_string(),
            capabilities: Self::capability_schemas(),
        }
    }
    
    fn execute(&self, capability: &str, input: Value) -> Result<Value, ModuleError> {
        match capability {
            "connect" => self.cmd_connect(input),
            "disconnect" => self.cmd_disconnect(input),
            "open" => self.cmd_open(input),
            "close" => self.cmd_close(input),
            "read" => self.cmd_read(input),
            "read_text" => self.cmd_read_text(input),
            "write" => self.cmd_write(input),
            "list_dir" => self.cmd_list_dir(input),
            "stat" => self.cmd_stat(input),
            "mkdir" => self.cmd_mkdir(input),
            "rmdir" => self.cmd_rmdir(input),
            "unlink" => self.cmd_unlink(input),
            _ => Err(ModuleError::new("unknown_capability", &format!("Unknown: {}", capability))),
        }
    }
    
    fn get_state(&self) -> Value {
        let connections = self.connections.read().unwrap();
        let sessions = self.sessions.read().unwrap();
        serde_json::json!({
            "type": "sftp_module_v2",
            "active_connections": connections.len(),
            "open_sessions": sessions.len()
        })
    }
    
    fn set_state(&mut self, _state: Value) -> Result<(), ModuleError> {
        Ok(())
    }
}

impl SftpModule {
    fn cmd_connect(&self, input: Value) -> Result<Value, ModuleError> {
        let connection_id = input["connection_id"].as_str()
            .ok_or_else(|| ModuleError::new("invalid_input", "Missing 'connection_id'"))?;
        let hostname = input["hostname"].as_str()
            .ok_or_else(|| ModuleError::new("invalid_input", "Missing 'hostname'"))?;
        let username = input["username"].as_str()
            .ok_or_else(|| ModuleError::new("invalid_input", "Missing 'username'"))?;
        let port = input["port"].as_u64().unwrap_or(22) as u16;
        
        // Store connection info (actual connection would be created by backend)
        let mut connections = self.connections.write().unwrap();
        connections.insert(connection_id.to_string(), SshConnection {
            hostname: hostname.to_string(),
            port,
            username: username.to_string(),
        });
        
        Ok(serde_json::json!({
            "success": true,
            "connection_id": connection_id,
            "sftp_version": 3
        }))
    }
    
    fn cmd_disconnect(&self, input: Value) -> Result<Value, ModuleError> {
        let connection_id = input["connection_id"].as_str()
            .ok_or_else(|| ModuleError::new("invalid_input", "Missing 'connection_id'"))?;
        
        let mut connections = self.connections.write().unwrap();
        connections.remove(connection_id);
        
        // Close all sessions for this connection
        let mut sessions = self.sessions.write().unwrap();
        sessions.retain(|k, _| !k.starts_with(connection_id));
        
        Ok(serde_json::json!({"success": true}))
    }
    
    fn cmd_open(&self, input: Value) -> Result<Value, ModuleError> {
        let connection_id = input["connection_id"].as_str()
            .ok_or_else(|| ModuleError::new("invalid_input", "Missing 'connection_id'"))?;
        let remote_path = input["remote_path"].as_str()
            .ok_or_else(|| ModuleError::new("invalid_input", "Missing 'remote_path'"))?;
        
        // Check connection exists
        let connections = self.connections.read().unwrap();
        if !connections.contains_key(connection_id) {
            return Err(ModuleError::new("not_connected", "Connection not found"));
        }
        drop(connections);
        
        // Create a virtual file handle path
        let file_handle = format!("{}:{}", connection_id, remote_path);
        
        // Open via VfsManager with local backend for now
        // (SFTP backend integration pending)
        let handle = VfsManager::open_local(remote_path)
            .map_err(|e| ModuleError::new("open_error", &e.to_string()))?;
        
        let size = handle.metadata().map(|m| m.size).unwrap_or(0);
        
        let mut sessions = self.sessions.write().unwrap();
        sessions.insert(file_handle.clone(), SftpSession {
            backend: Box::new(crate::vfs::local::LocalBackend::new()),
            path: remote_path.to_string(),
        });
        
        Ok(serde_json::json!({
            "file_handle": file_handle,
            "size": size
        }))
    }
    
    fn cmd_close(&self, input: Value) -> Result<Value, ModuleError> {
        let file_handle = input["file_handle"].as_str()
            .ok_or_else(|| ModuleError::new("invalid_input", "Missing 'file_handle'"))?;
        
        let mut sessions = self.sessions.write().unwrap();
        sessions.remove(file_handle);
        
        VfsManager::close(file_handle);
        
        Ok(serde_json::json!({"success": true}))
    }
    
    fn cmd_read(&self, input: Value) -> Result<Value, ModuleError> {
        let file_handle = input["file_handle"].as_str()
            .ok_or_else(|| ModuleError::new("invalid_input", "Missing 'file_handle'"))?;
        let offset = input["offset"].as_u64().unwrap_or(0);
        let length = input["length"].as_u64().unwrap_or(1024) as usize;
        
        let handle = VfsManager::get(file_handle)
            .ok_or_else(|| ModuleError::new("not_open", "File not open"))?;
        
        let data = handle.read_range(offset, length);
        
        Ok(serde_json::json!({
            "data": data,
            "bytes_read": data.len()
        }))
    }
    
    fn cmd_read_text(&self, input: Value) -> Result<Value, ModuleError> {
        let file_handle = input["file_handle"].as_str()
            .ok_or_else(|| ModuleError::new("invalid_input", "Missing 'file_handle'"))?;
        let offset = input["offset"].as_u64().unwrap_or(0);
        let length = input["length"].as_u64().unwrap_or(1024) as usize;
        
        let handle = VfsManager::get(file_handle)
            .ok_or_else(|| ModuleError::new("not_open", "File not open"))?;
        
        let text = handle.read_text(offset, length);
        
        Ok(serde_json::json!({
            "text": text,
            "bytes_read": text.len()
        }))
    }
    
    fn cmd_write(&self, input: Value) -> Result<Value, ModuleError> {
        let file_handle = input["file_handle"].as_str()
            .ok_or_else(|| ModuleError::new("invalid_input", "Missing 'file_handle'"))?;
        let offset = input["offset"].as_u64().unwrap_or(0);
        let data: Vec<u8> = input["data"].as_array()
            .ok_or_else(|| ModuleError::new("invalid_input", "Missing 'data'"))?
            .iter()
            .filter_map(|v| v.as_u64().map(|n| n as u8))
            .collect();
        
        // Get handle and apply edit
        let mut sessions = self.sessions.write().unwrap();
        if let Some(session) = sessions.get_mut(file_handle) {
            let mut handle = VfsManager::get(file_handle)
                .ok_or_else(|| ModuleError::new("not_open", "File not open"))?;
            handle.apply_edit(EditOp::Insert { offset, data: data.clone() });
        }
        
        Ok(serde_json::json!({
            "bytes_written": data.len(),
            "success": true
        }))
    }
    
    fn cmd_list_dir(&self, input: Value) -> Result<Value, ModuleError> {
        let _connection_id = input["connection_id"].as_str()
            .ok_or_else(|| ModuleError::new("invalid_input", "Missing 'connection_id'"))?;
        let remote_path = input["remote_path"].as_str()
            .ok_or_else(|| ModuleError::new("invalid_input", "Missing 'remote_path'"))?;
        
        // Use VfsManager for unified access
        let entries = VfsManager::read_dir(remote_path)
            .map_err(|e| ModuleError::new("list_error", &e.to_string()))?;
        
        Ok(serde_json::json!({
            "entries": entries.iter().map(|e| serde_json::json!({
                "name": e.name,
                "path": e.path,
                "is_file": e.metadata.is_file,
                "is_dir": e.metadata.is_dir,
                "size": e.metadata.size
            })).collect::<Vec<_>>(),
            "count": entries.len()
        }))
    }
    
    fn cmd_stat(&self, input: Value) -> Result<Value, ModuleError> {
        let _connection_id = input["connection_id"].as_str()
            .ok_or_else(|| ModuleError::new("invalid_input", "Missing 'connection_id'"))?;
        let remote_path = input["remote_path"].as_str()
            .ok_or_else(|| ModuleError::new("invalid_input", "Missing 'remote_path'"))?;
        
        // Use VfsManager for unified access
        match VfsManager::stat(remote_path) {
            Ok(meta) => Ok(serde_json::json!({
                "exists": true,
                "size": meta.size,
                "is_file": meta.is_file,
                "is_dir": meta.is_dir
            })),
            Err(_) => Ok(serde_json::json!({
                "exists": false,
                "size": 0,
                "is_file": false,
                "is_dir": false
            }))
        }
    }
    
    fn cmd_mkdir(&self, input: Value) -> Result<Value, ModuleError> {
        let remote_path = input["remote_path"].as_str()
            .ok_or_else(|| ModuleError::new("invalid_input", "Missing 'remote_path'"))?;
        
        let backend = Box::new(crate::vfs::local::LocalBackend::new());
        
        match backend.create_dir(remote_path) {
            Ok(_) => Ok(serde_json::json!({"success": true})),
            Err(e) => Err(ModuleError::new("mkdir_error", &e.to_string()))
        }
    }
    
    fn cmd_rmdir(&self, input: Value) -> Result<Value, ModuleError> {
        let remote_path = input["remote_path"].as_str()
            .ok_or_else(|| ModuleError::new("invalid_input", "Missing 'remote_path'"))?;
        
        let backend = Box::new(crate::vfs::local::LocalBackend::new());
        
        match backend.remove_dir(remote_path) {
            Ok(_) => Ok(serde_json::json!({"success": true})),
            Err(e) => Err(ModuleError::new("rmdir_error", &e.to_string()))
        }
    }
    
    fn cmd_unlink(&self, input: Value) -> Result<Value, ModuleError> {
        let remote_path = input["remote_path"].as_str()
            .ok_or_else(|| ModuleError::new("invalid_input", "Missing 'remote_path'"))?;
        
        let backend = Box::new(crate::vfs::local::LocalBackend::new());
        
        match backend.remove_file(remote_path) {
            Ok(_) => Ok(serde_json::json!({"success": true})),
            Err(e) => Err(ModuleError::new("unlink_error", &e.to_string()))
        }
    }
}

/// Register the SFTP module
pub fn register() {
    let module = SftpModule::new();
    crate::modular::ModuleRegistry::register("sftp", Box::new(module))
        .expect("Failed to register sftp module");
}
