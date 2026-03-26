use serde::{Serialize, Deserialize};
use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use dashmap::DashMap;
use once_cell::sync::Lazy;

/// Global mark storage per file
static MARK_STORE: Lazy<DashMap<String, FileMarks>> = Lazy::new(|| DashMap::new());

/// Mark engine for global operations
pub struct MarkEngine;

impl MarkEngine {
    /// Get or create marks for a file
    pub fn get_or_create<P: AsRef<Path>>(path: P) -> dashmap::mapref::one::RefMut<'static, String, FileMarks> {
        let path_str = path.as_ref().to_string_lossy().to_string();
        MARK_STORE.entry(path_str.clone()).or_insert_with(|| FileMarks::new(&path_str));
        // Return a RefMut that points to the entry
        // Safety: This is unsafe due to lifetime erasure, but DashMap manages this internally
        MARK_STORE.entry(path_str).or_insert_with(|| FileMarks::new(""))
    }
    
    /// Get marks for a file (if exists)
    pub fn get<P: AsRef<Path>>(path: P) -> Option<dashmap::mapref::one::Ref<'static, String, FileMarks>> {
        let path_str = path.as_ref().to_string_lossy().to_string();
        MARK_STORE.get(&path_str)
    }
    
    /// Close marks for a file
    pub fn close<P: AsRef<Path>>(path: P) {
        let path_str = path.as_ref().to_string_lossy().to_string();
        MARK_STORE.remove(&path_str);
    }
    
    /// Export all marks
    pub fn export_all() -> HashMap<String, MarkExport> {
        MARK_STORE
            .iter()
            .map(|entry| (entry.key().clone(), entry.value().export()))
            .collect()
    }
    
    /// Clear all marks
    pub fn clear_all() {
        MARK_STORE.clear();
    }
}

/// Helper: get current timestamp
fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// Color palette for marks
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum MarkColor {
    Red,
    Orange,
    Yellow,
    Green,
    Cyan,
    Blue,
    Purple,
    Pink,
    Gray,
}

impl MarkColor {
    pub fn to_hex(&self) -> &'static str {
        match self {
            MarkColor::Red => "#ff6b6b",
            MarkColor::Orange => "#ff9f43",
            MarkColor::Yellow => "#feca57",
            MarkColor::Green => "#5cd85c",
            MarkColor::Cyan => "#22d3ee",
            MarkColor::Blue => "#54a0ff",
            MarkColor::Purple => "#a55ee9",
            MarkColor::Pink => "#ff6b9d",
            MarkColor::Gray => "#a4a4a4",
        }
    }
}

impl std::fmt::Display for MarkColor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Default for MarkColor {
    fn default() -> Self {
        MarkColor::Red
    }
}

/// A mark in the file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mark {
    pub id: String,
    pub start: usize,
    pub end: usize,
    pub color: MarkColor,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    pub created_at: u64,
    pub updated_at: u64,
}

impl Mark {
    pub fn new(start: usize, end: usize, color: MarkColor) -> Self {
        let now = current_timestamp();
        Mark {
            id: format!("{:}-{:}", now, start),
            start,
            end,
            color,
            label: None,
            note: None,
            created_at: now,
            updated_at: now,
        }
    }
    
    pub fn with_label(mut self, label: String) -> Self {
        self.label = Some(label);
        self.updated_at = current_timestamp();
        self
    }
    
    pub fn with_note(mut self, note: String) -> Self {
        self.note = Some(note);
        self.updated_at = current_timestamp();
        self
    }
}

/// Updates to a mark
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarkUpdate {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<MarkColor>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub clear_label: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub clear_note: Option<bool>,
}

/// Export format for marks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarkExport {
    pub path: String,
    pub count: usize,
    pub marks: Vec<Mark>,
}

/// Marks for a single file
#[derive(Debug)]
pub struct FileMarks {
    path: String,
    marks: HashMap<String, Mark>,
    by_offset: BTreeMap<usize, Vec<String>>, // For quick lookup at offset
    by_color: HashMap<MarkColor, Vec<String>>,
}

impl FileMarks {
    pub fn new(path: &str) -> Self {
        FileMarks {
            path: path.to_string(),
            marks: HashMap::new(),
            by_offset: BTreeMap::new(),
            by_color: HashMap::new(),
        }
    }
    
    /// Add a mark
    pub fn add_mark(&mut self, mut mark: Mark) -> Result<Mark, String> {
        // Generate ID if empty
        if mark.id.is_empty() {
            mark.id = format!("{:}-{:}", mark.created_at, mark.start);
        }
        
        let id = mark.id.clone();
        
        // Add to main map
        self.marks.insert(id.clone(), mark.clone());
        
        // Index by offset
        self.by_offset
            .entry(mark.start)
            .or_insert_with(Vec::new)
            .push(id.clone());
        
        // Index by color
        self.by_color
            .entry(mark.color)
            .or_insert_with(Vec::new)
            .push(id.clone());
        
        Ok(mark)
    }
    
    /// Delete a mark
    pub fn delete_mark(&mut self, id: &str) -> Option<Mark> {
        // Remove from main map
        let mark = self.marks.remove(id)?;
        
        // Remove from offset index
        if let Some(ids) = self.by_offset.get_mut(&mark.start) {
            ids.retain(|x| x != id);
            if ids.is_empty() {
                self.by_offset.remove(&mark.start);
            }
        }
        
        // Remove from color index
        if let Some(ids) = self.by_color.get_mut(&mark.color) {
            ids.retain(|x| x != id);
            if ids.is_empty() {
                self.by_color.remove(&mark.color);
            }
        }
        
        Some(mark)
    }
    
    /// Update a mark
    pub fn update_mark(&mut self, id: &str, updates: MarkUpdate) -> Result<Mark, String> {
        let mut mark = self.marks.get(id)
            .cloned()
            .ok_or_else(|| "Mark not found".to_string())?;
        
        // Remove old offset index entry
        if let Some(ids) = self.by_offset.get_mut(&mark.start) {
            ids.retain(|x| x != id);
            if ids.is_empty() {
                self.by_offset.remove(&mark.start);
            }
        }
        
        // Apply updates
        if let Some(start) = updates.start {
            mark.start = start;
        }
        if let Some(end) = updates.end {
            mark.end = end;
        }
        if let Some(color) = updates.color {
            // Remove from old color index
            if let Some(ids) = self.by_color.get_mut(&mark.color) {
                ids.retain(|x| x != id);
                if ids.is_empty() {
                    self.by_color.remove(&mark.color);
                }
            }
            mark.color = color;
        }
        if let Some(label) = updates.label {
            mark.label = Some(label);
        }
        if let Some(note) = updates.note {
            mark.note = Some(note);
        }
        mark.updated_at = current_timestamp();
        
        // Re-index
        self.by_offset
            .entry(mark.start)
            .or_insert_with(Vec::new)
            .push(id.to_string());
        
        self.by_color
            .entry(mark.color)
            .or_insert_with(Vec::new)
            .push(id.to_string());
        
        // Update in map
        self.marks.insert(id.to_string(), mark.clone());
        
        Ok(mark)
    }
    
    /// Get mark by ID
    pub fn get_mark(&self, id: &str) -> Option<&Mark> {
        self.marks.get(id)
    }
    
    /// Get all marks
    pub fn get_all_marks(&self) -> Vec<&Mark> {
        self.marks.values().collect()
    }
    
    /// Get marks at specific offset
    pub fn get_marks_at(&self, offset: usize) -> Vec<&Mark> {
        let mut result = Vec::new();
        
        // Find all ranges that contain this offset
        for (&start, ids) in &self.by_offset {
            if start <= offset {
                for id in ids {
                    if let Some(mark) = self.marks.get(id) {
                        if mark.end > offset {
                            result.push(mark);
                        }
                    }
                }
            }
        }
        
        result
    }
    
    /// Get marks in range
    pub fn get_marks_in_range(&self, start: usize, end: usize) -> Vec<&Mark> {
        self.marks.values()
            .filter(|m| m.start < end && m.end > start)
            .collect()
    }
    
    /// Delete by color
    pub fn delete_by_color(&mut self, color: MarkColor) -> usize {
        if let Some(ids) = self.by_color.remove(&color) {
            let count = ids.len();
            for id in &ids {
                self.marks.remove(id);
            }
            count
        } else {
            0
        }
    }
    
    /// Clear all marks
    pub fn clear_all(&mut self) {
        self.marks.clear();
        self.by_offset.clear();
        self.by_color.clear();
    }
    
    /// Count of marks
    pub fn count(&self) -> usize {
        self.marks.len()
    }
    
    /// Export all marks
    pub fn export(&self) -> MarkExport {
        MarkExport {
            path: self.path.clone(),
            count: self.marks.len(),
            marks: self.marks.values().cloned().collect(),
        }
    }
}
