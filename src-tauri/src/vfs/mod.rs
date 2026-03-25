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
use std::sync::{Arc, RwLock};

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

/// Edit operations for journaling
#[derive(Debug, Clone)]
pub enum EditOp {
    Write { offset: u64, data: Vec<u8> },
    Insert { offset: u64, data: Vec<u8> },
    Delete { offset: u64, length: u64 },
}

/// Piece table entry
#[derive(Debug, Clone)]
pub enum Piece {
    Original { offset: u64, length: u64 },
    Added { data: Vec<u8> },
}

impl Piece {
    pub fn length(&self) -> u64 {
        match self {
            Piece::Original { length, .. } => *length,
            Piece::Added { data } => data.len() as u64,
        }
    }
}

#[derive(Debug, Default)]
struct EditJournalState {
    history: Vec<EditOp>,
    position: usize,
    is_dirty: bool,
}

/// Journal for tracking edits with piece table support
#[derive(Debug, Clone)]
pub struct EditJournal {
    state: Arc<RwLock<EditJournalState>>,
}

impl EditJournal {
    pub fn new() -> Self {
        EditJournal {
            state: Arc::new(RwLock::new(EditJournalState::default())),
        }
    }
    
    pub fn add_edit(&self, op: EditOp) -> io::Result<()> {
        let mut state = self.state.write().map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
        let pos = state.position;
        if pos < state.history.len() {
            state.history.truncate(pos);
        }
        state.history.push(op);
        state.position += 1;
        state.is_dirty = true;
        Ok(())
    }

    pub fn set_dirty(&self, dirty: bool) {
        if let Ok(mut state) = self.state.write() {
            state.is_dirty = dirty;
        }
    }

    pub fn is_dirty(&self) -> bool {
        self.state.read().map(|s| s.is_dirty || !s.history.is_empty()).unwrap_or(false)
    }
    
    pub fn undo(&self) -> bool {
        if let Ok(mut state) = self.state.write() {
            if state.position > 0 {
                state.position -= 1;
                return true;
            }
        }
        false
    }
    
    pub fn redo(&self) -> bool {
        if let Ok(mut state) = self.state.write() {
            if state.position < state.history.len() {
                state.position += 1;
                return true;
            }
        }
        false
    }
    
    /// Build piece table from history
    pub fn build_piece_table(&self, base_size: u64) -> Vec<Piece> {
        let state = match self.state.read() {
            Ok(s) => s,
            Err(_) => return vec![Piece::Original { offset: 0, length: base_size }],
        };
        
        let mut pieces = vec![Piece::Original { offset: 0, length: base_size }];
        
        for op in &state.history[..state.position] {
            match op {
                EditOp::Insert { offset, data } => {
                    self.insert_into_pieces(&mut pieces, *offset, data.clone());
                }
                EditOp::Delete { offset, length } => {
                    self.delete_from_pieces(&mut pieces, *offset, *length);
                }
                EditOp::Write { offset, data } => {
                    // Write is delete then insert
                    self.delete_from_pieces(&mut pieces, *offset, data.len() as u64);
                    self.insert_into_pieces(&mut pieces, *offset, data.clone());
                }
            }
        }
        pieces
    }
    
    fn insert_into_pieces(&self, pieces: &mut Vec<Piece>, offset: u64, data: Vec<u8>) {
        let mut current_pos = 0;
        let mut i = 0;
        while i < pieces.len() {
            let piece_len = pieces[i].length();
            if current_pos <= offset && offset < current_pos + piece_len {
                // Split piece i
                let split_off = offset - current_pos;
                let left_piece = match &pieces[i] {
                    Piece::Original { offset: o, .. } => Piece::Original { offset: *o, length: split_off },
                    Piece::Added { data: d } => Piece::Added { data: d[..split_off as usize].to_vec() },
                };
                let right_piece = match &pieces[i] {
                    Piece::Original { offset: o, length: l } => Piece::Original { offset: o + split_off, length: l - split_off },
                    Piece::Added { data: d } => Piece::Added { data: d[split_off as usize..].to_vec() },
                };
                
                pieces.remove(i);
                if right_piece.length() > 0 {
                    pieces.insert(i, right_piece);
                }
                pieces.insert(i, Piece::Added { data });
                if left_piece.length() > 0 {
                    pieces.insert(i, left_piece);
                }
                return;
            }
            current_pos += piece_len;
            i += 1;
        }
        // If offset is at the very end
        if offset == current_pos {
            pieces.push(Piece::Added { data });
        }
    }
    
    fn delete_from_pieces(&self, pieces: &mut Vec<Piece>, offset: u64, length: u64) {
        let mut current_pos = 0;
        let mut i = 0;
        let mut remaining_to_delete = length;
        
        while i < pieces.len() && remaining_to_delete > 0 {
            let piece_len = pieces[i].length();
            if current_pos + piece_len > offset {
                let overlap_start = offset.max(current_pos);
                let overlap_end = (offset + remaining_to_delete).min(current_pos + piece_len);
                let overlap_len = overlap_end - overlap_start;
                
                if overlap_len > 0 {
                    // Split or remove piece
                    let split_start = overlap_start - current_pos;
                    let split_end = overlap_end - current_pos;
                    
                    let piece = pieces.remove(i);
                    let mut replacements = Vec::new();
                    
                    if split_start > 0 {
                        replacements.push(match &piece {
                            Piece::Original { offset: o, .. } => Piece::Original { offset: *o, length: split_start },
                            Piece::Added { data: d } => Piece::Added { data: d[..split_start as usize].to_vec() },
                        });
                    }
                    
                    if split_end < piece_len {
                        replacements.push(match &piece {
                            Piece::Original { offset: o, length: l } => Piece::Original { offset: o + split_end, length: l - split_end },
                            Piece::Added { data: d } => Piece::Added { data: d[split_end as usize..].to_vec() },
                        });
                    }
                    
                    for (idx, r) in replacements.into_iter().enumerate() {
                        pieces.insert(i + idx, r);
                    }
                    
                    remaining_to_delete -= overlap_len;
                    // If we removed/split pieces, don't increment i yet as the new piece at i might also overlap
                    continue;
                }
            }
            current_pos += piece_len;
            i += 1;
        }
    }

    pub fn effective_size(&self, base_size: u64) -> u64 {
        self.build_piece_table(base_size).iter().map(|p| p.length()).sum()
    }
    
    pub fn logical_to_physical(&self, logical_offset: u64, base_size: u64) -> Option<u64> {
        let pieces = self.build_piece_table(base_size);
        let mut current_pos = 0;
        for piece in pieces {
            let piece_len = piece.length();
            if logical_offset >= current_pos && logical_offset < current_pos + piece_len {
                match piece {
                    Piece::Original { offset, .. } => return Some(offset + (logical_offset - current_pos)),
                    Piece::Added { .. } => return None, // Not in physical file
                }
            }
            current_pos += piece_len;
        }
        None
    }
    
    pub fn clear(&self) {
        if let Ok(mut state) = self.state.write() {
            state.history.clear();
            state.position = 0;
        }
    }
    
    pub fn can_undo(&self) -> bool {
        self.state.read().map(|s| s.position > 0).unwrap_or(false)
    }
    
    pub fn can_redo(&self) -> bool {
        self.state.read().map(|s| s.position < s.history.len()).unwrap_or(false)
    }
    
    pub fn has_edits(&self) -> bool {
        self.state.read().map(|s| !s.history.is_empty()).unwrap_or(false)
    }
}

/// Local file system backend
pub mod local;

/// Remote file system backend (SFTP)
pub mod remote;

/// VFS Manager for unified access
pub mod manager;

#[cfg(test)]
mod tests;
