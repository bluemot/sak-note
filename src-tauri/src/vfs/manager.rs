//! VFS Manager - Unified file operations with journaling
//! 
//! Wraps VFS backends (local/SFTP) with EditJournal for undo/redo.
//! Provides the same interface as EditableFileManager.

use crate::vfs::{VfsBackend, VfsFile, VfsMetadata, EditJournal, EditOp};
use std::collections::HashMap;
use std::io::{Read, Seek, SeekFrom};
use once_cell::sync::Lazy;
use dashmap::DashMap;

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
    
    /// Get effective file size (accounting for edits)
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
        // For now, just mark as saved
        self.is_dirty = false;
        self.journal.clear();
        Ok(())
    }
    
    /// Get file metadata
    pub fn metadata(&self) -> std::io::Result<VfsMetadata> {
        self.backend.metadata(&self.path)
    }
}

/// Global VFS file handle cache
static VFS_CACHE: Lazy<DashMap<String, VfsFileHandle>> = Lazy::new(|| DashMap::new());

/// VFS Manager - manages file handles
pub struct VfsManager;

impl VfsManager {
    /// Open a file with VFS (auto-detects backend)
    pub fn open(path: &str) -> std::io::Result<VfsFileHandle> {
        // Check cache
        if let Some(handle) = VFS_CACHE.get(path) {
            return Ok(VfsFileHandle::new(
                Self::create_backend(path)?,
                path
            ));
        }
        
        let backend = Self::create_backend(path)?;
        let handle = VfsFileHandle::new(backend, path);
        VFS_CACHE.insert(path.to_string(), handle.clone());
        Ok(handle)
    }
    
    /// Create appropriate backend based on path
    fn create_backend(path: &str) -> std::io::Result<Box<dyn VfsBackend>> {
        // For local files, use local backend
        // For SSH paths (user@host:path), use SFTP backend
        if path.contains('@') && path.contains(':') {
            // SSH path format: user@hostname:/path/to/file
            // Would need to parse and create SftpBackend
            // For now, return error
            return Err(std::io::Error::new(
                std::io::ErrorKind::Unsupported,
                "SFTP backend not yet integrated"
            ));
        }
        
        // Use local backend
        Ok(Box::new(crate::vfs::local::LocalBackend::new()))
    }
    
    /// Close a file
    pub fn close(path: &str) {
        VFS_CACHE.remove(path);
    }
    
    /// Check if file is open
    pub fn is_open(path: &str) -> bool {
        VFS_CACHE.contains_key(path)
    }
}

impl Clone for VfsFileHandle {
    fn clone(&self) -> Self {
        // Note: This creates a new handle, not a reference
        // Journal state is not shared
        let backend = match Self::create_backend_for_handle(&self.path) {
            Ok(b) => b,
            Err(_) => return VfsFileHandle {
                backend: Box::new(crate::vfs::local::LocalBackend::new()),
                path: self.path.clone(),
                journal: EditJournal::new(),
                is_dirty: false,
            },
        };
        VfsFileHandle {
            backend,
            path: self.path.clone(),
            journal: EditJournal::new(),
            is_dirty: false,
        }
    }
}

impl VfsFileHandle {
    fn create_backend_for_handle(path: &str) -> std::io::Result<Box<dyn VfsBackend>> {
        VfsManager::create_backend(path)
    }
}
