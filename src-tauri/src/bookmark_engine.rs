//! Bookmark Engine
//!
//! Manages bookmarks (F2 navigation) like Notepad++
//! Bookmarks are persisted to .sakbookmarks files

#![allow(dead_code)]

use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs;
use std::io::{self, Write};

/// A bookmark in a file
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Bookmark {
    pub line: u32,
    pub label: Option<String>,
    pub note: Option<String>,
    pub created_at: String, // ISO 8601 timestamp
}

impl Bookmark {
    pub fn new(line: u32) -> Self {
        Self {
            line,
            label: None,
            note: None,
            created_at: chrono::Local::now().to_rfc3339(),
        }
    }

    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn with_note(mut self, note: impl Into<String>) -> Self {
        self.note = Some(note.into());
        self
    }
}

/// Bookmark storage for a file
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BookmarkStore {
    pub file_path: String,
    pub bookmarks: Vec<Bookmark>,
    pub version: String,
}

impl BookmarkStore {
    pub fn new(file_path: impl Into<String>) -> Self {
        Self {
            file_path: file_path.into(),
            bookmarks: Vec::new(),
            version: "1.0".to_string(),
        }
    }

    /// Add a bookmark
    pub fn add(&mut self, line: u32) -> bool {
        // Check if bookmark already exists at this line
        if self.bookmarks.iter().any(|b| b.line == line) {
            return false;
        }
        
        let mut bookmark = Bookmark::new(line);
        
        // Auto-generate label from line number
        bookmark.label = Some(format!("Line {}", line));
        
        self.bookmarks.push(bookmark);
        self.bookmarks.sort_by_key(|b| b.line);
        true
    }

    /// Remove a bookmark at specific line
    pub fn remove(&mut self, line: u32) -> bool {
        let initial_len = self.bookmarks.len();
        self.bookmarks.retain(|b| b.line != line);
        self.bookmarks.len() < initial_len
    }

    /// Toggle bookmark at line
    pub fn toggle(&mut self, line: u32) -> bool {
        if self.has_bookmark(line) {
            self.remove(line);
            false
        } else {
            self.add(line);
            true
        }
    }

    /// Check if line has bookmark
    pub fn has_bookmark(&self, line: u32) -> bool {
        self.bookmarks.iter().any(|b| b.line == line)
    }

    /// Get next bookmark from current line
    pub fn next_bookmark(&self, current_line: u32) -> Option<&Bookmark> {
        self.bookmarks
            .iter()
            .find(|b| b.line > current_line)
            .or_else(|| self.bookmarks.first()) // Wrap around
    }

    /// Get previous bookmark from current line
    pub fn prev_bookmark(&self, current_line: u32) -> Option<&Bookmark> {
        self.bookmarks
            .iter()
            .rev()
            .find(|b| b.line < current_line)
            .or_else(|| self.bookmarks.last()) // Wrap around
    }

    /// Clear all bookmarks
    pub fn clear(&mut self) {
        self.bookmarks.clear();
    }

    /// Get bookmark file path
    fn bookmark_file_path(&self) -> PathBuf {
        let original = Path::new(&self.file_path);
        let parent = original.parent().unwrap_or(Path::new("."));
        let filename = original.file_name().unwrap_or_default();
        let bookmark_filename = format!(".{:?}.sakbookmarks", filename);
        // Remove quotes from OsStr debug
        let _bookmark_filename = bookmark_filename.trim_matches('"');
        parent.join(format!(".{:?}.sakbookmarks", filename).trim_matches('"'))
    }

    /// Save bookmarks to file
    pub fn save(&self) -> io::Result<()> {
        let bookmark_path = self.bookmark_file_path();
        let json = serde_json::to_string_pretty(self)?;
        let mut file = fs::File::create(&bookmark_path)?;
        file.write_all(json.as_bytes())?;
        Ok(())
    }

    /// Load bookmarks from file
    pub fn load(file_path: impl Into<String>) -> io::Result<Self> {
        let file_path = file_path.into();
        let bookmark_path = Self::bookmark_path_for_file(&file_path);
        
        if !bookmark_path.exists() {
            return Ok(Self::new(file_path));
        }
        
        let content = fs::read_to_string(&bookmark_path)?;
        let mut store: BookmarkStore = serde_json::from_str(&content)?;
        store.file_path = file_path; // Ensure path matches
        Ok(store)
    }

    /// Get bookmark file path for a given file
    fn bookmark_path_for_file(file_path: &str) -> PathBuf {
        let path = Path::new(file_path);
        let parent = path.parent().unwrap_or(Path::new("."));
        let filename = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");
        parent.join(format!(".{filename}.sakbookmarks"))
    }

    /// Delete bookmark file
    pub fn delete_storage(&self) -> io::Result<()> {
        let bookmark_path = self.bookmark_file_path();
        if bookmark_path.exists() {
            fs::remove_file(bookmark_path)?;
        }
        Ok(())
    }
}

/// Global bookmark manager
pub struct BookmarkManager {
    stores: HashMap<String, BookmarkStore>,
}

impl BookmarkManager {
    pub fn new() -> Self {
        Self {
            stores: HashMap::new(),
        }
    }

    /// Get or load bookmark store for file
    pub fn get_store(&mut self, file_path: &str) -> io::Result<&mut BookmarkStore> {
        if !self.stores.contains_key(file_path) {
            let store = BookmarkStore::load(file_path)?;
            self.stores.insert(file_path.to_string(), store);
        }
        Ok(self.stores.get_mut(file_path).unwrap())
    }

    /// Toggle bookmark
    pub fn toggle(&mut self, file_path: &str, line: u32) -> io::Result<bool> {
        let store = self.get_store(file_path)?;
        let added = store.toggle(line);
        store.save()?;
        Ok(added)
    }

    /// Add bookmark
    pub fn add(&mut self, file_path: &str, line: u32) -> io::Result<bool> {
        let store = self.get_store(file_path)?;
        let added = store.add(line);
        if added {
            store.save()?;
        }
        Ok(added)
    }

    /// Remove bookmark
    pub fn remove(&mut self, file_path: &str, line: u32) -> io::Result<bool> {
        let store = self.get_store(file_path)?;
        let removed = store.remove(line);
        if removed {
            store.save()?;
        }
        Ok(removed)
    }

    /// Get all bookmarks for file
    pub fn get_bookmarks(&mut self, file_path: &str) -> io::Result<Vec<Bookmark>> {
        let store = self.get_store(file_path)?;
        Ok(store.bookmarks.clone())
    }

    /// Clear all bookmarks for file
    pub fn clear(&mut self, file_path: &str) -> io::Result<()> {
        let store = self.get_store(file_path)?;
        store.clear();
        store.save()?;
        Ok(())
    }

    /// Navigate to next bookmark
    pub fn next(&mut self, file_path: &str, current_line: u32) -> io::Result<Option<Bookmark>> {
        let store = self.get_store(file_path)?;
        Ok(store.next_bookmark(current_line).cloned())
    }

    /// Navigate to previous bookmark
    pub fn prev(&mut self, file_path: &str, current_line: u32) -> io::Result<Option<Bookmark>> {
        let store = self.get_store(file_path)?;
        Ok(store.prev_bookmark(current_line).cloned())
    }

    /// Update bookmark label
    pub fn update_label(&mut self, file_path: &str, line: u32, label: impl Into<String>) -> io::Result<bool> {
        let store = self.get_store(file_path)?;
        if let Some(bookmark) = store.bookmarks.iter_mut().find(|b| b.line == line) {
            bookmark.label = Some(label.into());
            store.save()?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Update bookmark note
    pub fn update_note(&mut self, file_path: &str, line: u32, note: impl Into<String>) -> io::Result<bool> {
        let store = self.get_store(file_path)?;
        if let Some(bookmark) = store.bookmarks.iter_mut().find(|b| b.line == line) {
            bookmark.note = Some(note.into());
            store.save()?;
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl Default for BookmarkManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bookmark_toggle() {
        let mut store = BookmarkStore::new("/test/file.rs");
        
        // Add bookmark
        assert!(store.toggle(10));
        assert!(store.has_bookmark(10));
        
        // Remove bookmark
        assert!(!store.toggle(10));
        assert!(!store.has_bookmark(10));
    }

    #[test]
    fn test_bookmark_navigation() {
        let mut store = BookmarkStore::new("/test/file.rs");
        store.add(10);
        store.add(20);
        store.add(30);
        
        // Next from line 15 should be 20
        let next = store.next_bookmark(15);
        assert_eq!(next.map(|b| b.line), Some(20));
        
        // Next from line 30 should wrap to 10
        let next = store.next_bookmark(30);
        assert_eq!(next.map(|b| b.line), Some(10));
        
        // Previous from line 25 should be 20
        let prev = store.prev_bookmark(25);
        assert_eq!(prev.map(|b| b.line), Some(20));
    }

    #[test]
    fn test_bookmark_persistence() {
        use std::io::Write;
        
        // Create temp file
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("test_bookmark_file.txt");
        let mut file = fs::File::create(&test_file).unwrap();
        file.write_all(b"test content").unwrap();
        
        let test_path = test_file.to_str().unwrap();
        
        // Create and save bookmarks
        {
            let mut store = BookmarkStore::new(test_path);
            store.add(10);
            store.add(20);
            store.save().unwrap();
        }
        
        // Load and verify
        {
            let store = BookmarkStore::load(test_path).unwrap();
            assert_eq!(store.bookmarks.len(), 2);
            assert!(store.has_bookmark(10));
            assert!(store.has_bookmark(20));
        }
        
        // Cleanup
        let _ = fs::remove_file(&test_file);
        let bookmark_file = BookmarkStore::new(test_path).bookmark_file_path();
        let _ = fs::remove_file(bookmark_file);
    }
}
