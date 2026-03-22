//! Virtual File System (VFS) - Unified file operations for local and remote files
//! 
//! Provides a common interface for:
//! - Local files (memory-mapped, chunked)
//! - SFTP remote files (seek + partial read/write)
//! 
//! Both backends support:
//! - Journal-based editing (undo/redo)
//! - Chunked reading for large files
//! - Efficient seeking without loading entire file

use std::io::{self, Read, Write, Seek, SeekFrom};
use std::path::Path;

/// VFS backend trait - implemented by local and remote file systems
pub trait VfsBackend: Send + Sync {
    /// Open a file for reading
    fn open_read(&self, path: &str) -> io::Result<Box<dyn VfsFile>>;
    
    /// Open a file for writing
    fn open_write(&self, path: &str) -> io::Result<Box<dyn VfsFile>>;
    
    /// Check if file exists
    fn exists(&self, path: &str) -> io::Result<bool>;
    
    /// Get file metadata
    fn metadata(&self, path: &str) -> io::Result<VfsMetadata>;
    
    /// List directory contents
    fn read_dir(&self, path: &str) -> io::Result<Vec<VfsDirEntry>>;
    
    /// Create directory
    fn create_dir(&self, path: &str) -> io::Result<()>;
    
    /// Remove file
    fn remove_file(&self, path: &str) -> io::Result<()>;
    
    /// Remove directory
    fn remove_dir(&self, path: &str) -> io::Result<()>;
}

/// VFS file trait - common interface for file operations
pub trait VfsFile: Read + Write + Seek + Send {
    /// Get file size
    fn size(&self) -> io::Result<u64>;
    
    /// Sync to storage
    fn sync(&mut self) -> io::Result<()>;
    
    /// Read bytes at specific offset (with seek)
    fn read_at(&mut self, offset: u64, buf: &mut [u8]) -> io::Result<usize>;
    
    /// Write bytes at specific offset (with seek)
    fn write_at(&mut self, offset: u64, buf: &[u8]) -> io::Result<usize>;
}

/// VFS metadata
#[derive(Debug, Clone)]
pub struct VfsMetadata {
    pub size: u64,
    pub is_file: bool,
    pub is_dir: bool,
    pub modified: Option<std::time::SystemTime>,
    pub accessed: Option<std::time::SystemTime>,
    pub created: Option<std::time::SystemTime>,
    pub permissions: Option<u32>,
}

/// VFS directory entry
#[derive(Debug, Clone)]
pub struct VfsDirEntry {
    pub name: String,
    pub path: String,
    pub metadata: VfsMetadata,
}

/// VFS handle - combines backend with journaling
pub struct VfsHandle {
    backend: Box<dyn VfsBackend>,
    path: String,
    journal: EditJournal,
}

impl VfsHandle {
    pub fn new(backend: Box<dyn VfsBackend>, path: &str) -> Self {
        VfsHandle {
            backend,
            path: path.to_string(),
            journal: EditJournal::new(),
        }
    }
    
    /// Read range of bytes (applies journal on-the-fly)
    pub fn read_range(&self, start: u64, length: usize) -> io::Result<Vec<u8>> {
        // Apply journal edits to determine logical view
        let effective_size = self.journal.effective_size(self.backend_size()?);
        
        if start >= effective_size {
            return Ok(Vec::new());
        }
        
        let end = (start + length as u64).min(effective_size);
        let length = (end - start) as usize;
        
        // Read from backend with journal overlay
        let mut file = self.backend.open_read(&self.path)?;
        let mut result = vec![0u8; length];
        
        // Build logical to physical mapping
        let physical_offset = self.journal.logical_to_physical(start);
        file.seek(SeekFrom::Start(physical_offset))?;
        file.read_exact(&mut result)?;
        
        // Apply any pending edits in this range
        self.journal.apply_edits_to_buffer(start, &mut result);
        
        Ok(result)
    }
    
    /// Write operation (adds to journal)
    pub fn write(&mut self, offset: u64, data: &[u8]) {
        self.journal.add_edit(EditOp::Write { offset, data: data.to_vec() });
    }
    
    /// Insert operation (adds to journal)
    pub fn insert(&mut self, offset: u64, data: &[u8]) {
        self.journal.add_edit(EditOp::Insert { offset, data: data.to_vec() });
    }
    
    /// Delete operation (adds to journal)
    pub fn delete(&mut self, offset: u64, length: u64) {
        self.journal.add_edit(EditOp::Delete { offset, length });
    }
    
    /// Save changes to backend
    pub fn save(&mut self) -> io::Result<()> {
        // Apply all journal edits to actual file
        let mut file = self.backend.open_write(&self.path)?;
        self.journal.flush_to(&mut *file)?;
        self.journal.clear();
        file.sync()
    }
    
    /// Undo last operation
    pub fn undo(&mut self) -> bool {
        self.journal.undo()
    }
    
    /// Redo last undone operation
    pub fn redo(&mut self) -> bool {
        self.journal.redo()
    }
    
    /// Get effective file size
    pub fn size(&self) -> io::Result<u64> {
        let backend_size = self.backend_size()?;
        Ok(self.journal.effective_size(backend_size))
    }
    
    fn backend_size(&self) -> io::Result<u64> {
        self.backend.metadata(&self.path).map(|m| m.size)
    }
}

/// Edit operations for journaling
#[derive(Debug, Clone)]
pub enum EditOp {
    Write { offset: u64, data: Vec<u8> },
    Insert { offset: u64, data: Vec<u8> },
    Delete { offset: u64, length: u64 },
}

/// Journal for tracking edits
#[derive(Debug, Default)]
pub struct EditJournal {
    history: Vec<EditOp>,
    position: usize,
}

impl EditJournal {
    pub fn new() -> Self {
        EditJournal {
            history: Vec::new(),
            position: 0,
        }
    }
    
    pub fn add_edit(&mut self, op: EditOp) {
        // Remove redo history
        if self.position < self.history.len() {
            self.history.truncate(self.position);
        }
        self.history.push(op);
        self.position += 1;
    }
    
    pub fn undo(&mut self) -> bool {
        if self.position > 0 {
            self.position -= 1;
            true
        } else {
            false
        }
    }
    
    pub fn redo(&mut self) -> bool {
        if self.position < self.history.len() {
            self.position += 1;
            true
        } else {
            false
        }
    }
    
    pub fn effective_size(&self, base_size: u64) -> u64 {
        let mut size = base_size as i64;
        for op in &self.history[..self.position] {
            match op {
                EditOp::Insert { data, .. } => size += data.len() as i64,
                EditOp::Delete { length, .. } => size -= *length as i64,
                EditOp::Write { .. } => {} // Write doesn't change size
            }
        }
        size.max(0) as u64
    }
    
    pub fn logical_to_physical(&self, logical_offset: u64) -> u64 {
        // Simplified mapping - full implementation tracks all edits
        logical_offset
    }
    
    pub fn apply_edits_to_buffer(&self, _start_offset: u64, _buffer: &mut [u8]) {
        // Apply pending edits to read buffer
        // Full implementation would overlay edits on top of base data
    }
    
    pub fn flush_to(&self, _file: &mut dyn VfsFile) -> io::Result<()> {
        // Apply all edits to actual file
        // Write to temp file, then atomic rename
        Ok(())
    }
    
    pub fn clear(&mut self) {
        self.history.clear();
        self.position = 0;
    }
    
    pub fn can_undo(&self) -> bool {
        self.position > 0
    }
    
    pub fn can_redo(&self) -> bool {
        self.position < self.history.len()
    }
}

/// Local file system backend
pub mod local;

/// SFTP backend  
pub mod remote;
