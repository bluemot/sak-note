pub mod chunk;

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, RwLock};
use once_cell::sync::Lazy;
use dashmap::DashMap;

pub use chunk::{
    Chunk, ChunkManager, EditableFileManager, EditOp, BatchEdit, 
    SearchEngine, SearchResult, CHUNK_SIZE, SEARCH_BUFFER_SIZE
};

/// Global file handle cache
static FILE_CACHE: Lazy<DashMap<String, Arc<ChunkManager>>> = Lazy::new(|| DashMap::new());

/// Global editable file cache
static EDITABLE_CACHE: Lazy<DashMap<String, Arc<RwLock<EditableFileManager>>>> = Lazy::new(|| DashMap::new());

/// File Engine for handling large files
pub struct FileEngine;

impl FileEngine {
    /// Open a file and create chunk manager
    pub fn open_file<P: AsRef<Path>>(path: P) -> Result<Arc<ChunkManager>, std::io::Error> {
        let path_str = path.as_ref().to_string_lossy().to_string();
        
        // Check cache first
        if let Some(cached) = FILE_CACHE.get(&path_str) {
            return Ok(cached.clone());
        }
        
        // Create new chunk manager
        let manager = Arc::new(ChunkManager::new(&path)?);
        FILE_CACHE.insert(path_str, manager.clone());
        
        Ok(manager)
    }
    
    /// Open file for editing
    pub fn open_for_edit<P: AsRef<Path>>(path: P) -> Result<Arc<RwLock<EditableFileManager>>, std::io::Error> {
        let path_str = path.as_ref().to_string_lossy().to_string();
        
        // Check cache first
        if let Some(cached) = EDITABLE_CACHE.get(&path_str) {
            return Ok(cached.clone());
        }
        
        // Create new editable manager
        let manager = Arc::new(RwLock::new(EditableFileManager::new(&path)?));
        EDITABLE_CACHE.insert(path_str, manager.clone());
        
        Ok(manager)
    }
    
    /// Get editable file manager
    pub fn get_editable(path: &str) -> Option<Arc<RwLock<EditableFileManager>>> {
        EDITABLE_CACHE.get(path).map(|m| m.clone())
    }
    
    /// Close editable file
    pub fn close_editable(path: &str) {
        EDITABLE_CACHE.remove(path);
    }
    
    /// Close file and remove from cache
    pub fn close_file(path: &str) {
        FILE_CACHE.remove(path);
    }
    
    /// Get file info
    pub fn get_file_info(path: &str) -> Option<FileInfo> {
        // Try editable first
        if let Some(manager) = EDITABLE_CACHE.get(path) {
            if let Ok(guard) = manager.read() {
                return Some(FileInfo {
                    path: path.to_string(),
                    size: guard.effective_size(),
                    chunks: ((guard.effective_size() + CHUNK_SIZE as u64 - 1) / CHUNK_SIZE as u64) as usize,
                    editable: true,
                    has_changes: guard.has_changes(),
                });
            }
        }
        
        // Fall back to read-only
        FILE_CACHE.get(path).map(|m| FileInfo {
            path: path.to_string(),
            size: m.file_size(),
            chunks: m.chunk_count(),
            editable: false,
            has_changes: false,
        })
    }
    
    /// Clear all cached files
    pub fn clear_cache() {
        FILE_CACHE.clear();
        EDITABLE_CACHE.clear();
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct FileInfo {
    pub path: String,
    pub size: u64,
    pub chunks: usize,
    pub editable: bool,
    pub has_changes: bool,
}
