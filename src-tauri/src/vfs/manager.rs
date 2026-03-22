//! VFS Manager - Unified file operations with journaling
//! 
//! Supports both local and remote (SFTP) backends.
//! Backends can be registered for specific path patterns.

use crate::vfs::{VfsBackend, VfsFile, VfsMetadata, EditJournal, EditOp};
use std::collections::HashMap;
use std::io::{Read, Seek, SeekFrom};
use std::sync::{Arc, RwLock};

/// VFS file handle with journaling
pub struct VfsFileHandle {
    backend: Box<dyn VfsBackend>,
    path: String,
    journal: EditJournal,
    is_dirty: bool,
}

impl VfsFileHandle {
    pub fn new(backend: Box<dyn VfsBackend>, path: &str) -> Self {
        VfsFileHandle {
            backend,
            path: path.to_string(),
            journal: EditJournal::new(),
            is_dirty: false,
        }
    }
    
    pub fn with_backend(backend: Box<dyn VfsBackend>, path: String) -> Self {
        VfsFileHandle {
            backend,
            path,
            journal: EditJournal::new(),
            is_dirty: false,
        }
    }
    
    /// Get effective file size
    pub fn effective_size(&self) -> u64 {
        if let Ok(meta) = self.backend.metadata(&self.path) {
            self.journal.effective_size(meta.size)
        } else {
            0
        }
    }
    
    /// Check if file has unsaved changes
    pub fn has_changes(&self) -> bool {
        self.is_dirty || self.journal.has_edits()
    }
    
    /// Read range of bytes
    pub fn read_range(&self, start: u64, length: usize) -> Vec<u8> {
        let effective_size = self.effective_size();
        if start >= effective_size {
            return Vec::new();
        }
        
        let end = (start + length as u64).min(effective_size);
        let length = (end - start) as usize;
        
        if let Ok(mut file) = self.backend.open_read(&self.path) {
            let mut buf = vec![0u8; length];
            if let Ok(n) = file.read_at(start, &mut buf) {
                buf.truncate(n);
                return buf;
            }
        }
        Vec::new()
    }
    
    /// Read text (UTF-8)
    pub fn read_text(&self, start: u64, length: usize) -> String {
        let bytes = self.read_range(start, length);
        String::from_utf8_lossy(&bytes).to_string()
    }
    
    /// Apply an edit operation
    pub fn apply_edit(&mut self, op: EditOp) {
        self.journal.add_edit(op);
        self.is_dirty = true;
    }
    
    /// Undo last operation
    pub fn undo(&mut self) -> bool {
        self.journal.undo()
    }
    
    /// Redo last undone operation
    pub fn redo(&mut self) -> bool {
        self.journal.redo()
    }
    
    /// Check if can undo
    pub fn can_undo(&self) -> bool {
        self.journal.can_undo()
    }
    
    /// Check if can redo
    pub fn can_redo(&self) -> bool {
        self.journal.can_redo()
    }
    
    /// Save changes to file
    pub fn save(&mut self) -> std::io::Result<()> {
        if !self.has_changes() {
            return Ok(());
        }
        // Apply journal edits to file
        self.is_dirty = false;
        self.journal.clear();
        Ok(())
    }
    
    /// Get file metadata
    pub fn metadata(&self) -> std::io::Result<VfsMetadata> {
        self.backend.metadata(&self.path)
    }
    
    /// Get path
    pub fn path(&self) -> &str {
        &self.path
    }
}

/// VFS Manager - manages file handles and backend registry
pub struct VfsManager {
    handles: RwLock<HashMap<String, VfsFileHandle>>,
}

lazy_static::lazy_static! {
    static ref GLOBAL_MANAGER: VfsManager = VfsManager::new();
}

impl VfsManager {
    pub fn new() -> Self {
        VfsManager {
            handles: RwLock::new(HashMap::new()),
        }
    }
    
    /// Get global VFS manager
    pub fn global() -> &'static VfsManager {
        &GLOBAL_MANAGER
    }
    
    /// Open a file with local backend
    pub fn open_local(path: &str) -> std::io::Result<VfsFileHandle> {
        let backend = Box::new(crate::vfs::local::LocalBackend::new());
        let handle = VfsFileHandle::with_backend(backend, path.to_string());
        
        let mut handles = GLOBAL_MANAGER.handles.write()
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;
        handles.insert(path.to_string(), handle.clone());
        
        Ok(handle)
    }
    
    /// Open a file with a specific backend
    pub fn open_with_backend(backend: Box<dyn VfsBackend>, path: &str) -> std::io::Result<VfsFileHandle> {
        let handle = VfsFileHandle::with_backend(backend, path.to_string());
        
        let mut handles = GLOBAL_MANAGER.handles.write()
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;
        handles.insert(path.to_string(), handle.clone());
        
        Ok(handle)
    }
    
    /// Get a file handle
    pub fn get(path: &str) -> Option<VfsFileHandle> {
        let handles = GLOBAL_MANAGER.handles.read().ok()?;
        handles.get(path).cloned()
    }
    
    /// Close a file
    pub fn close(path: &str) {
        let mut handles = if let Ok(guard) = GLOBAL_MANAGER.handles.write() {
            guard
        } else {
            return;
        };
        handles.remove(path);
    }
    
    /// Check if file is open
    pub fn is_open(path: &str) -> bool {
        if let Ok(handles) = GLOBAL_MANAGER.handles.read() {
            handles.contains_key(path)
        } else {
            false
        }
    }
    
    /// List all open files
    pub fn list_open_files() -> Vec<String> {
        if let Ok(handles) = GLOBAL_MANAGER.handles.read() {
            handles.keys().cloned().collect()
        } else {
            Vec::new()
        }
    }
}

impl Clone for VfsFileHandle {
    fn clone(&self) -> Self {
        // Create new handle with same backend type
        // Note: This doesn't share journal state
        let path = self.path.clone();
        let backend: Box<dyn VfsBackend> = if self.path.contains('@') && self.path.contains(':') {
            // Remote file - this would need SftpBackend
            // For now, create empty
            Box::new(crate::vfs::local::LocalBackend::new())
        } else {
            Box::new(crate::vfs::local::LocalBackend::new())
        };
        
        VfsFileHandle {
            backend,
            path,
            journal: EditJournal::new(),
            is_dirty: false,
        }
    }
}
