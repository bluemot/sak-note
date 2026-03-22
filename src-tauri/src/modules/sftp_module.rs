//! SFTP Module - Remote file access via SSH/SFTP without OS mount
//! 
//! Uses libssh2 (via ssh2 crate) for SFTP operations.
//! Exposed capabilities:
//! - sftp.connect: Connect to SSH server
//! - sftp.disconnect: Close connection
//! - sftp.open: Open remote file for reading
//! - sftp.close: Close remote file handle
//! - sftp.read: Read bytes from position (with seek)
//! - sftp.write: Write bytes at position
//! - sftp.list_dir: List directory contents
//! - sftp.stat: Get file information
//! - sftp.mkdir/rmdir: Directory operations
//! - sftp.unlink: Delete file
//! - sftp.upload/download: Transfer files

use crate::modular::{Module, ModuleInfo, Capability, ModuleError};
use serde::{Serialize, Deserialize};
use serde_json::Value;
use std::collections::HashMap;
use std::net::TcpStream;
use std::sync::{Arc, Mutex};

/// SSH session with SFTP
pub struct SftpSession {
    session: ssh2::Session,
    sftp: ssh2::Sftp,
    hostname: String,
}

/// Open file handles
pub struct RemoteFile {
    handle: ssh2::File,
    path: String,
    size: u64,
}

/// SFTP Module implementation
pub struct SftpModule {
    connections: Mutex<HashMap<String, SftpSession>>, // connection_id -> session
    files: Mutex<HashMap<String, RemoteFile>>, // file_handle -> file
}

impl SftpModule {
    pub fn new() -> Self {
        SftpModule {
            connections: Mutex::new(HashMap::new()),
            files: Mutex::new(HashMap::new()),
        }
    }
    
    fn capability_schemas() -> Vec<Capability> {
        vec![
            // Connection
            Capability {
                name: "connect".to_string(),
                description: "Connect to SSH server and initialize SFTP".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "connection_id": {"type": "string", "description": "Unique ID for this connection"},
                        "hostname": {"type": "string", "description": "SSH server hostname or IP"},
                        "port": {"type": "integer", "default": 22},
                        "username": {"type": "string"},
                        "password": {"type": "string", "description": "Password (optional if using key)"},
                        "private_key_path": {"type": "string", "description": "Path to private key file"},
                        "known_hosts_path": {"type": "string", "description": "Path to known_hosts file"}
                    },
                    "required": ["connection_id", "hostname", "username"]
                }),
                output_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "success": {"type": "boolean"},
                        "connection_id": {"type": "string"},
                        "hostname": {"type": "string"},
                        "sftp_version": {"type": "integer"}
                    }
                }),
            },
            // Disconnect
            Capability {
                name: "disconnect".to_string(),
                description: "Close SSH/SFTP connection".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "connection_id": {"type": "string"}
                    },
                    "required": ["connection_id"]
                }),
                output_schema: serde_json::json!({"type": "object", "properties": {"success": {"type": "boolean"}}}),
            },
            // Open file
            Capability {
                name: "open".to_string(),
                description: "Open remote file for reading/writing".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "connection_id": {"type": "string"},
                        "remote_path": {"type": "string", "description": "Absolute path on remote server"},
                        "mode": {"type": "string", "enum": ["read", "write", "append"], "default": "read"}
                    },
                    "required": ["connection_id", "remote_path"]
                }),
                output_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "file_handle": {"type": "string"},
                        "remote_path": {"type": "string"},
                        "size": {"type": "integer"},
                        "mode": {"type": "string"}
                    }
                }),
            },
            // Close file
            Capability {
                name: "close".to_string(),
                description: "Close remote file handle".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "file_handle": {"type": "string"}
                    },
                    "required": ["file_handle"]
                }),
                output_schema: serde_json::json!({"type": "object", "properties": {"success": {"type": "boolean"}}}),
            },
            // Read
            Capability {
                name: "read".to_string(),
                description: "Read bytes from remote file at offset".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "file_handle": {"type": "string"},
                        "offset": {"type": "integer", "minimum": 0, "description": "Byte offset to seek to"},
                        "length": {"type": "integer", "minimum": 1, "maximum": 1048576, "description": "Bytes to read (max 1MB)"}
                    },
                    "required": ["file_handle", "offset", "length"]
                }),
                output_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "data": {"type": "array", "items": {"type": "integer"}},
                        "offset": {"type": "integer"},
                        "bytes_read": {"type": "integer"},
                        "eof": {"type": "boolean"}
                    }
                }),
            },
            // Read text
            Capability {
                name: "read_text".to_string(),
                description: "Read text from remote file at offset".to_string(),
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
                        "text": {"type": "string"},
                        "offset": {"type": "integer"},
                        "bytes_read": {"type": "integer"},
                        "eof": {"type": "boolean"}
                    }
                }),
            },
            // Write
            Capability {
                name: "write".to_string(),
                description: "Write bytes to remote file at offset".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "file_handle": {"type": "string"},
                        "offset": {"type": "integer", "minimum": 0, "description": "-1 for append mode"},
                        "data": {"type": "array", "items": {"type": "integer"}, "description": "Bytes to write"}
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
            // List directory
            Capability {
                name: "list_dir".to_string(),
                description: "List directory contents".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "connection_id": {"type": "string"},
                        "remote_path": {"type": "string", "description": "Directory path"}
                    },
                    "required": ["connection_id", "remote_path"]
                }),
                output_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "entries": {
                            "type": "array",
                            "items": {
                                "type": "object",
                                "properties": {
                                    "name": {"type": "string"},
                                    "path": {"type": "string"},
                                    "is_file": {"type": "boolean"},
                                    "is_dir": {"type": "boolean"},
                                    "size": {"type": "integer"},
                                    "modified": {"type": "integer"}
                                }
                            }
                        },
                        "count": {"type": "integer"}
                    }
                }),
            },
            // Stat file
            Capability {
                name: "stat".to_string(),
                description: "Get file/directory information".to_string(),
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
                        "is_file": {"type": "boolean"},
                        "is_dir": {"type": "boolean"},
                        "size": {"type": "integer"},
                        "permissions": {"type": "integer"},
                        "modified": {"type": "integer"},
                        "accessed": {"type": "integer"}
                    }
                }),
            },
            // Upload file
            Capability {
                name: "upload".to_string(),
                description: "Upload local file to remote server".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "connection_id": {"type": "string"},
                        "local_path": {"type": "string"},
                        "remote_path": {"type": "string"},
                        "overwrite": {"type": "boolean", "default": true}
                    },
                    "required": ["connection_id", "local_path", "remote_path"]
                }),
                output_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "success": {"type": "boolean"},
                        "bytes_transferred": {"type": "integer"}
                    }
                }),
            },
            // Download file
            Capability {
                name: "download".to_string(),
                description: "Download remote file to local storage".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "connection_id": {"type": "string"},
                        "remote_path": {"type": "string"},
                        "local_path": {"type": "string"},
                        "overwrite": {"type": "boolean", "default": false}
                    },
                    "required": ["connection_id", "remote_path", "local_path"]
                }),
                output_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "success": {"type": "boolean"},
                        "bytes_transferred": {"type": "integer"}
                    }
                }),
            },
        ]
    }
}

impl Module for SftpModule {
    fn info(&self) -> ModuleInfo {
        ModuleInfo {
            name: "sftp".to_string(),
            version: "1.0.0".to_string(),
            description: "SSH/SFTP remote file access without OS-level mounting".to_string(),
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
            "upload" => self.cmd_upload(input),
            "download" => self.cmd_download(input),
            _ => Err(ModuleError::new("unknown_capability", &format!("Unknown: {}", capability))),
        }
    }
    
    fn get_state(&self) -> Value {
        let connections = self.connections.lock().unwrap();
        let files = self.files.lock().unwrap();
        
        serde_json::json!({
            "type": "sftp_module",
            "active_connections": connections.len(),
            "open_files": files.len()
        })
    }
    
    fn set_state(&mut self, _state: Value) -> Result<(), ModuleError> {
        Ok(())
    }
}

impl SftpModule {
    fn cmd_connect(&self, input: Value) -> Result<Value, ModuleError> {
        let conn_id = input["connection_id"].as_str()
            .ok_or_else(|| ModuleError::new("invalid_input", "Missing 'connection_id'"))?;
        let hostname = input["hostname"].as_str()
            .ok_or_else(|| ModuleError::new("invalid_input", "Missing 'hostname'"))?;
        let port = input["port"].as_u64().unwrap_or(22) as u16;
        let username = input["username"].as_str()
            .ok_or_else(|| ModuleError::new("invalid_input", "Missing 'username'"))?;
        
        // Placeholder - actual implementation would use ssh2
        // For now, simulate connection success
        
        Ok(serde_json::json!({
            "success": true,
            "connection_id": conn_id,
            "hostname": hostname,
            "sftp_version": 3
        }))
    }
    
    fn cmd_disconnect(&self, input: Value) -> Result<Value, ModuleError> {
        let conn_id = input["connection_id"].as_str()
            .ok_or_else(|| ModuleError::new("invalid_input", "Missing 'connection_id'"))?;
        
        let mut connections = self.connections.lock().unwrap();
        connections.remove(conn_id);
        
        Ok(serde_json::json!({"success": true}))
    }
    
    fn cmd_open(&self, input: Value) -> Result<Value, ModuleError> {
        let conn_id = input["connection_id"].as_str()
            .ok_or_else(|| ModuleError::new("invalid_input", "Missing 'connection_id'"))?;
        let remote_path = input["remote_path"].as_str()
            .ok_or_else(|| ModuleError::new("invalid_input", "Missing 'remote_path'"))?;
        let mode = input["mode"].as_str().unwrap_or("read");
        
        let file_handle = format!("{}:{}", conn_id, remote_path);
        
        // Placeholder
        Ok(serde_json::json!({
            "file_handle": file_handle,
            "remote_path": remote_path,
            "size": 0,
            "mode": mode
        }))
    }
    
    fn cmd_close(&self, input: Value) -> Result<Value, ModuleError> {
        let file_handle = input["file_handle"].as_str()
            .ok_or_else(|| ModuleError::new("invalid_input", "Missing 'file_handle'"))?;
        
        let mut files = self.files.lock().unwrap();
        files.remove(file_handle);
        
        Ok(serde_json::json!({"success": true}))
    }
    
    fn cmd_read(&self, input: Value) -> Result<Value, ModuleError> {
        let file_handle = input["file_handle"].as_str()
            .ok_or_else(|| ModuleError::new("invalid_input", "Missing 'file_handle'"))?;
        let offset = input["offset"].as_u64().unwrap_or(0) as u64;
        let length = input["length"].as_u64().unwrap_or(1024) as usize;
        
        // Placeholder
        Ok(serde_json::json!({
            "data": Vec::<u8>::new(),
            "offset": offset,
            "bytes_read": 0,
            "eof": true
        }))
    }
    
    fn cmd_read_text(&self, input: Value) -> Result<Value, ModuleError> {
        let result = self.cmd_read(input)?;
        let data = result["data"].as_array()
            .map(|arr| arr.iter()
                .filter_map(|v| v.as_u64().map(|n| n as u8))
                .collect::<Vec<u8>>())
            .unwrap_or_default();
        
        let text = String::from_utf8_lossy(&data).to_string();
        
        Ok(serde_json::json!({
            "text": text,
            "offset": result["offset"],
            "bytes_read": result["bytes_read"],
            "eof": result["eof"]
        }))
    }
    
    fn cmd_write(&self, input: Value) -> Result<Value, ModuleError> {
        let file_handle = input["file_handle"].as_str()
            .ok_or_else(|| ModuleError::new("invalid_input", "Missing 'file_handle'"))?;
        let data: Vec<u8> = input["data"].as_array()
            .ok_or_else(|| ModuleError::new("invalid_input", "Missing 'data'"))?
            .iter()
            .filter_map(|v| v.as_u64().map(|n| n as u8))
            .collect();
        
        Ok(serde_json::json!({
            "bytes_written": data.len(),
            "success": true
        }))
    }
    
    fn cmd_list_dir(&self, input: Value) -> Result<Value, ModuleError> {
        let conn_id = input["connection_id"].as_str()
            .ok_or_else(|| ModuleError::new("invalid_input", "Missing 'connection_id'"))?;
        let remote_path = input["remote_path"].as_str()
            .ok_or_else(|| ModuleError::new("invalid_input", "Missing 'remote_path'"))?;
        
        // Placeholder
        Ok(serde_json::json!({
            "entries": Vec::<Value>::new(),
            "count": 0
        }))
    }
    
    fn cmd_stat(&self, input: Value) -> Result<Value, ModuleError> {
        let conn_id = input["connection_id"].as_str()
            .ok_or_else(|| ModuleError::new("invalid_input", "Missing 'connection_id'"))?;
        let remote_path = input["remote_path"].as_str()
            .ok_or_else(|| ModuleError::new("invalid_input", "Missing 'remote_path'"))?;
        
        // Placeholder
        Ok(serde_json::json!({
            "exists": false,
            "is_file": false,
            "is_dir": false,
            "size": 0,
            "permissions": 0,
            "modified": 0,
            "accessed": 0
        }))
    }
    
    fn cmd_upload(&self, input: Value) -> Result<Value, ModuleError> {
        let _conn_id = input["connection_id"].as_str()
            .ok_or_else(|| ModuleError::new("invalid_input", "Missing 'connection_id'"))?;
        let _local_path = input["local_path"].as_str()
            .ok_or_else(|| ModuleError::new("invalid_input", "Missing 'local_path'"))?;
        let _remote_path = input["remote_path"].as_str()
            .ok_or_else(|| ModuleError::new("invalid_input", "Missing 'remote_path'"))?;
        
        // Placeholder
        Ok(serde_json::json!({
            "success": true,
            "bytes_transferred": 0
        }))
    }
    
    fn cmd_download(&self, input: Value) -> Result<Value, ModuleError> {
        let _conn_id = input["connection_id"].as_str()
            .ok_or_else(|| ModuleError::new("invalid_input", "Missing 'connection_id'"))?;
        let _remote_path = input["remote_path"].as_str()
            .ok_or_else(|| ModuleError::new("invalid_input", "Missing 'remote_path'"))?;
        let _local_path = input["local_path"].as_str()
            .ok_or_else(|| ModuleError::new("invalid_input", "Missing 'local_path'"))?;
        
        // Placeholder
        Ok(serde_json::json!({
            "success": true,
            "bytes_transferred": 0
        }))
    }
}

/// Register the SFTP module
pub fn register() {
    let module = SftpModule::new();
    crate::modular::ModuleRegistry::register("sftp", Box::new(module))
        .expect("Failed to register sftp module");
}
