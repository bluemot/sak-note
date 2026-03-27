pub mod chunk;

use std::path::Path;
use std::sync::{Arc, RwLock};
use once_cell::sync::Lazy;
use dashmap::DashMap;

pub use chunk::{
    ChunkManager, EditableFileManager, EditOp, CHUNK_SIZE
};

/// Global file handle cache
static FILE_CACHE: Lazy<DashMap<String, Arc<ChunkManager>>> = Lazy::new(|| DashMap::new());

/// Global editable file cache
static EDITABLE_CACHE: Lazy<DashMap<String, Arc<RwLock<EditableFileManager>>>> = Lazy::new(|| DashMap::new());

/// File Engine for handling large files
pub struct FileEngine;

impl FileEngine {
    /// Open a file and create chunk manager
    #[allow(dead_code)]
    pub fn open_file<P: AsRef<Path>>(path: P) -> Result<Arc<ChunkManager>, std::io::Error> {
        let path_str = path.as_ref().to_string_lossy().to_string();
        log::info!("[FileEngine::open_file] Opening file: {}", path_str);
        
        // Check cache first
        log::debug!("[FileEngine::open_file] Checking FILE_CACHE for: {}", path_str);
        if let Some(cached) = FILE_CACHE.get(&path_str) {
            log::info!("[FileEngine::open_file] File found in cache: {}", path_str);
            return Ok(cached.clone());
        }
        log::debug!("[FileEngine::open_file] File not in cache, creating new ChunkManager");
        
        // Create new chunk manager
        let manager = Arc::new(ChunkManager::new(&path)?);
        let file_size = manager.file_size();
        let chunk_count = manager.chunk_count();
        FILE_CACHE.insert(path_str.clone(), manager.clone());
        
        log::info!("[FileEngine::open_file] File opened successfully:");
        log::info!("[FileEngine::open_file]   Path: {}", path_str);
        log::info!("[FileEngine::open_file]   Size: {} bytes", file_size);
        log::info!("[FileEngine::open_file]   Chunks: {}", chunk_count);
        log::debug!("[FileEngine::open_file] File added to FILE_CACHE");
        
        Ok(manager)
    }
    
    /// Open file for editing
    pub fn open_for_edit<P: AsRef<Path>>(path: P) -> Result<Arc<RwLock<EditableFileManager>>, std::io::Error> {
        let path_str = path.as_ref().to_string_lossy().to_string();
        log::info!("[FileEngine::open_for_edit] Opening file for editing: {}", path_str);
        
        // Check cache first
        log::debug!("[FileEngine::open_for_edit] Checking EDITABLE_CACHE for: {}", path_str);
        if let Some(cached) = EDITABLE_CACHE.get(&path_str) {
            log::info!("[FileEngine::open_for_edit] File found in EDITABLE_CACHE: {}", path_str);
            return Ok(cached.clone());
        }
        log::debug!("[FileEngine::open_for_edit] File not in cache, creating new EditableFileManager");
        
        // Create new editable manager
        let manager = Arc::new(RwLock::new(EditableFileManager::new(&path)?));
        EDITABLE_CACHE.insert(path_str.clone(), manager.clone());
        
        // Get file info for logging
        if let Ok(guard) = manager.read() {
            let size = guard.effective_size();
            log::info!("[FileEngine::open_for_edit] Editable file opened successfully:");
            log::info!("[FileEngine::open_for_edit]   Path: {}", path_str);
            log::info!("[FileEngine::open_for_edit]   Effective size: {} bytes", size);
            log::debug!("[FileEngine::open_for_edit] File added to EDITABLE_CACHE");
        }
        
        Ok(manager)
    }
    
    /// Get editable file manager
    pub fn get_editable(path: &str) -> Option<Arc<RwLock<EditableFileManager>>> {
        log::trace!("[FileEngine::get_editable] Looking up editable file: {}", path);
        let result = EDITABLE_CACHE.get(path).map(|m| {
            log::debug!("[FileEngine::get_editable] Found editable file in cache: {}", path);
            m.clone()
        });
        if result.is_none() {
            log::warn!("[FileEngine::get_editable] Editable file not found in cache: {}", path);
        }
        result
    }
    
    /// Close editable file
    pub fn close_editable(path: &str) {
        log::info!("[FileEngine::close_editable] Closing editable file: {}", path);
        EDITABLE_CACHE.remove(path);
        log::debug!("[FileEngine::close_editable] File removed from EDITABLE_CACHE");
    }
    
    /// Close file and remove from cache
    pub fn close_file(path: &str) {
        log::info!("[FileEngine::close_file] Closing file: {}", path);
        FILE_CACHE.remove(path);
        log::debug!("[FileEngine::close_file] File removed from FILE_CACHE");
    }
    
    /// Get file info
    pub fn get_file_info(path: &str) -> Option<FileInfo> {
        log::trace!("[FileEngine::get_file_info] Getting file info for: {}", path);
        // Try editable first
        if let Some(manager) = EDITABLE_CACHE.get(path) {
            log::debug!("[FileEngine::get_file_info] Found in EDITABLE_CACHE");
            if let Ok(guard) = manager.read() {
                let size = guard.effective_size();
                let chunks = ((size + CHUNK_SIZE as u64 - 1) / CHUNK_SIZE as u64) as usize;
                let has_changes = guard.has_changes();
                log::debug!("[FileEngine::get_file_info] File info: size={}, chunks={}, has_changes={}", 
                    size, chunks, has_changes);
                return Some(FileInfo {
                    path: path.to_string(),
                    size,
                    chunks,
                    editable: true,
                    has_changes,
                });
            } else {
                log::warn!("[FileEngine::get_file_info] Failed to acquire read lock");
            }
        }
        
        // Fall back to read-only
        log::debug!("[FileEngine::get_file_info] Checking FILE_CACHE");
        FILE_CACHE.get(path).map(|m| {
            let size = m.file_size();
            let chunks = m.chunk_count();
            log::debug!("[FileEngine::get_file_info] Found in FILE_CACHE: size={}, chunks={}", 
                size, chunks);
            FileInfo {
                path: path.to_string(),
                size,
                chunks,
                editable: false,
                has_changes: false,
            }
        })
    }
    
    /// Clear all cached files
    #[allow(dead_code)]
    pub fn clear_cache() {
        log::info!("[FileEngine::clear_cache] Clearing all file caches");
        let file_cache_size = FILE_CACHE.len();
        let editable_cache_size = EDITABLE_CACHE.len();
        FILE_CACHE.clear();
        EDITABLE_CACHE.clear();
        log::info!("[FileEngine::clear_cache] Cleared {} files from FILE_CACHE and {} from EDITABLE_CACHE", 
            file_cache_size, editable_cache_size);
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
