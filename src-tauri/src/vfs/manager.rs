//! VFS Manager - Unified file operations with journaling
//! 
//! Supports both local and remote (SFTP) backends.
//! Backends can be registered for specific path patterns.

use crate::vfs::{VfsBackend, VfsMetadata, EditJournal, EditOp, Piece};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::io;

/// VFS file handle with journaling
#[derive(Clone)]
pub struct VfsFileHandle {
    backend: Arc<dyn VfsBackend>,
    path: String,
    journal: EditJournal,
}

impl VfsFileHandle {
    pub fn new(backend: Arc<dyn VfsBackend>, path: &str) -> Self {
        VfsFileHandle {
            backend,
            path: path.to_string(),
            journal: EditJournal::new(),
        }
    }
    
    pub fn with_backend(backend: Arc<dyn VfsBackend>, path: String) -> Self {
        VfsFileHandle {
            backend,
            path,
            journal: EditJournal::new(),
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
        self.journal.is_dirty()
    }
    
    /// Read range of bytes (using piece table)
    pub fn read_range(&self, start: u64, length: usize) -> Vec<u8> {
        let meta = match self.backend.metadata(&self.path) {
            Ok(m) => m,
            Err(_) => return Vec::new(),
        };
        
        let pieces = self.journal.build_piece_table(meta.size);
        let mut result = Vec::with_capacity(length);
        let mut current_pos = 0;
        let target_end = start + length as u64;
        
        // Open file once for all Original pieces in this range
        let mut file = match self.backend.open_read(&self.path) {
            Ok(f) => f,
            Err(_) => return Vec::new(),
        };

        for piece in pieces {
            let piece_len = piece.length();
            if current_pos + piece_len > start && current_pos < target_end {
                let overlap_start = start.max(current_pos);
                let overlap_end = target_end.min(current_pos + piece_len);
                let overlap_len = overlap_end - overlap_start;
                
                let piece_offset = overlap_start - current_pos;
                
                match piece {
                    Piece::Original { offset, .. } => {
                        let mut buf = vec![0u8; overlap_len as usize];
                        if file.read_at(offset + piece_offset, &mut buf).is_ok() {
                            result.extend_from_slice(&buf);
                        }
                    }
                    Piece::Added { data } => {
                        let data_start = piece_offset as usize;
                        let data_end = (piece_offset + overlap_len) as usize;
                        result.extend_from_slice(&data[data_start..data_end]);
                    }
                }
            }
            current_pos += piece_len;
            if current_pos >= target_end { break; }
        }
        result
    }
    
    /// Read text (UTF-8)
    pub fn read_text(&self, start: u64, length: usize) -> String {
        let bytes = self.read_range(start, length);
        String::from_utf8_lossy(&bytes).to_string()
    }
    
    /// Apply an edit operation
    pub fn apply_edit(&mut self, op: EditOp) -> io::Result<()> {
        self.journal.add_edit(op)
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
    pub fn save(&mut self) -> io::Result<()> {
        if !self.has_changes() {
            return Ok(());
        }
        
        let meta = self.backend.metadata(&self.path)?;
        let pieces = self.journal.build_piece_table(meta.size);
        
        let mut file = self.backend.open_write(&self.path)?;
        let mut current_offset = 0;
        
        for piece in &pieces {
            match piece {
                Piece::Original { offset, length } => {
                    let mut buf = vec![0u8; *length as usize];
                    if let Ok(mut reader) = self.backend.open_read(&self.path) {
                        reader.read_at(*offset, &mut buf)?;
                        file.write_at(current_offset, &buf)?;
                    }
                }
                Piece::Added { data } => {
                    file.write_at(current_offset, data)?;
                }
            }
            current_offset += piece.length();
        }
        
        file.sync()?;
        self.journal.set_dirty(false);
        self.journal.clear();
        Ok(())
    }
    
    /// Get file metadata
    pub fn metadata(&self) -> io::Result<VfsMetadata> {
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
    local_backend: Arc<dyn VfsBackend>,
}

lazy_static::lazy_static! {
    static ref GLOBAL_MANAGER: VfsManager = VfsManager::new();
}

impl VfsManager {
    pub fn new() -> Self {
        VfsManager {
            handles: RwLock::new(HashMap::new()),
            local_backend: Arc::new(crate::vfs::local::LocalBackend::new()),
        }
    }
    
    /// Get global VFS manager
    pub fn global() -> &'static VfsManager {
        &GLOBAL_MANAGER
    }
    
    /// Open a file with local backend
    pub fn open_local(path: &str) -> io::Result<VfsFileHandle> {
        let manager = Self::global();
        
        // Check if already open
        {
            let handles = manager.handles.read().map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
            if let Some(handle) = handles.get(path) {
                return Ok(handle.clone());
            }
        }
        
        let handle = VfsFileHandle::with_backend(manager.local_backend.clone(), path.to_string());
        
        let mut handles = manager.handles.write().map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
        handles.insert(path.to_string(), handle.clone());
        
        Ok(handle)
    }
    
    /// Open a file with a specific backend
    pub fn open_with_backend(backend: Arc<dyn VfsBackend>, path: &str) -> io::Result<VfsFileHandle> {
        let manager = Self::global();
        let handle = VfsFileHandle::with_backend(backend, path.to_string());
        
        let mut handles = manager.handles.write().map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
        handles.insert(path.to_string(), handle.clone());
        
        Ok(handle)
    }
    
    /// Get a file handle
    pub fn get(path: &str) -> Option<VfsFileHandle> {
        let manager = Self::global();
        let handles = manager.handles.read().ok()?;
        handles.get(path).cloned()
    }
    
    /// Close a file
    pub fn close(path: &str) {
        let manager = Self::global();
        if let Ok(mut handles) = manager.handles.write() {
            handles.remove(path);
        }
    }
    
    /// Check if file is open
    pub fn is_open(path: &str) -> bool {
        let manager = Self::global();
        if let Ok(handles) = manager.handles.read() {
            handles.contains_key(path)
        } else {
            false
        }
    }
    
    /// List all open files
    pub fn list_open_files() -> Vec<String> {
        let manager = Self::global();
        if let Ok(handles) = manager.handles.read() {
            handles.keys().cloned().collect()
        } else {
            Vec::new()
        }
    }
}
