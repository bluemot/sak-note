use serde::{Serialize, Deserialize};
use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use dashmap::DashMap;
use once_cell::sync::Lazy;

/// Global mark storage per file
static MARK_STORE: Lazy<DashMap<String, FileMarks>> = Lazy::new(|| DashMap::new());

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
            MarkColor::Green => "#1dd1a1",
            MarkColor::Cyan => "#00d2d3",
            MarkColor::Blue => "#54a0ff",
            MarkColor::Purple => "#5f27cd",
            MarkColor::Pink => "#ff9ff3",
            MarkColor::Gray => "#8395a7",
        }
    }
    
    pub fn to_rgba(&self, alpha: f32) -> String {
        let hex = self.to_hex();
        let r = u8::from_str_radix(&hex[1..3], 16).unwrap_or(0);
        let g = u8::from_str_radix(&hex[3..5], 16).unwrap_or(0);
        let b = u8::from_str_radix(&hex[5..7], 16).unwrap_or(0);
        format!("rgba({}, {}, {}, {})", r, g, b, alpha)
    }
}

/// A single mark/highlight
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mark {
    pub id: String,
    pub start: usize,       // Byte offset start
    pub end: usize,         // Byte offset end (exclusive)
    pub color: MarkColor,
    pub label: Option<String>,
    pub note: Option<String>,
    pub created_at: u64,    // Unix timestamp
    pub updated_at: u64,
}

/// Marks for a specific file
#[derive(Debug, Clone, Default)]
pub struct FileMarks {
    path: PathBuf,
    marks: BTreeMap<String, Mark>,  // id -> mark
    by_offset: BTreeMap<usize, Vec<String>>, // start_offset -> mark ids (for fast lookup)
}

impl FileMarks {
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        FileMarks {
            path: path.as_ref().to_path_buf(),
            marks: BTreeMap::new(),
            by_offset: BTreeMap::new(),
        }
    }
    
    /// Add a new mark
    pub fn add_mark(&mut self, mut mark: Mark) -> Result<Mark, String> {
        // Validate range
        if mark.start >= mark.end {
            return Err("Invalid range: start must be less than end".to_string());
        }
        
        // Generate ID if not provided
        if mark.id.is_empty() {
            mark.id = format!("mark_{}_{}", mark.start, mark.created_at);
        }
        
        // Update timestamp
        mark.updated_at = current_timestamp();
        
        // Store
        let id = mark.id.clone();
        self.by_offset.entry(mark.start).or_default().push(id.clone());
        self.marks.insert(id, mark.clone());
        
        Ok(mark)
    }
    
    /// Delete a mark by ID
    pub fn delete_mark(&mut self, id: &str) -> Option<Mark> {
        let mark = self.marks.remove(id)?;
        
        // Remove from index
        if let Some(ids) = self.by_offset.get_mut(&mark.start) {
            ids.retain(|x| x != id);
            if ids.is_empty() {
                self.by_offset.remove(&mark.start);
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
        }
        
        // Apply updates
        if let Some(start) = updates.start {
            mark.start = start;
        }
        if let Some(end) = updates.end {
            mark.end = end;
        }
        if let Some(color) = updates.color {
            mark.color = color;
        }
        if let Some(ref label) = updates.label {
            mark.label = Some(label.clone());
        }
        if let Some(ref note) = updates.note {
            mark.note = Some(note.clone());
        }
        if updates.clear_label {
            mark.label = None;
        }
        if updates.clear_note {
            mark.note = None;
        }
        
        mark.updated_at = current_timestamp();
        
        // Validate
        if mark.start >= mark.end {
            return Err("Invalid range after update".to_string());
        }
        
        // Update storage
        self.by_offset.entry(mark.start).or_default().push(id.to_string());
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
    
    /// Get marks in a range
    pub fn get_marks_in_range(&self, start: usize, end: usize) -> Vec<&Mark> {
        self.by_offset
            .range(..=end)
            .flat_map(|(_, ids)| ids.iter())
            .filter_map(|id| self.marks.get(id))
            .filter(|m| m.end > start)  // Overlapping marks
            .collect()
    }
    
    /// Get marks at specific position
    pub fn get_marks_at(&self, offset: usize) -> Vec<&Mark> {
        self.marks
            .values()
            .filter(|m| m.start <= offset && m.end > offset)
            .collect()
    }
    
    /// Clear all marks
    pub fn clear_all(&mut self) {
        self.marks.clear();
        self.by_offset.clear();
    }
    
    /// Delete marks by color
    pub fn delete_by_color(&mut self, color: MarkColor) -> usize {
        let to_delete: Vec<String> = self.marks
            .iter()
            .filter(|(_, m)| m.color == color)
            .map(|(id, _)| id.clone())
            .collect();
        
        let count = to_delete.len();
        for id in to_delete {
            self.delete_mark(&id);
        }
        count
    }
    
    /// Get mark count
    pub fn count(&self) -> usize {
        self.marks.len()
    }
    
    /// Export to serializable format
    pub fn export(&self) -> MarkExport {
        MarkExport {
            path: self.path.to_string_lossy().to_string(),
            marks: self.marks.values().cloned().collect(),
        }
    }
    
    /// Import from serializable format
    pub fn import(&mut self, export: MarkExport) -> Result<(), String> {
        self.clear_all();
        for mark in export.marks {
            self.add_mark(mark)?;
        }
        Ok(())
    }
    
    /// Load marks from persistence (placeholder for future SQLite integration)
    pub fn load_from_disk(&mut self) -> Result<(), String> {
        // TODO: Implement SQLite loading
        Ok(())
    }
    
    /// Save marks to persistence (placeholder for future SQLite integration)
    pub fn save_to_disk(&self) -> Result<(), String> {
        // TODO: Implement SQLite saving
        Ok(())
    }
}

/// Update structure for partial updates
#[derive(Debug, Clone, Default, Deserialize)]
pub struct MarkUpdate {
    pub start: Option<usize>,
    pub end: Option<usize>,
    pub color: Option<MarkColor>,
    pub label: Option<String>,
    pub note: Option<String>,
    #[serde(default)]
    pub clear_label: bool,
    #[serde(default)]
    pub clear_note: bool,
}

/// Export format for persistence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarkExport {
    pub path: String,
    pub marks: Vec<Mark>,
}

/// Mark engine for global operations
pub struct MarkEngine;

impl MarkEngine {
    /// Get or create marks for a file
    pub fn get_or_create<P: AsRef<Path>>(path: P) -> dashmap::mapref::one::RefMut<String, FileMarks> {
        let path_str = path.as_ref().to_string_lossy().to_string();
        
        MARK_STORE.entry(path_str.clone()).or_insert_with(|| {
            FileMarks::new(&path_str)
        })
    }
    
    /// Get marks for a file (if exists)
    pub fn get<P: AsRef<Path>>(path: P) -> Option<dashmap::mapref::one::Ref<String, FileMarks>> {
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
            .map(|entry| {
                (entry.key().clone(), entry.export())
            })
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

/// Color preset management
#[derive(Debug, Clone, Default)]
pub struct ColorPresets {
    presets: HashMap<String, MarkColor>,
}

impl ColorPresets {
    pub fn new() -> Self {
        ColorPresets {
            presets: HashMap::new(),
        }
    }
    
    pub fn set(&mut self, name: &str, color: MarkColor) {
        self.presets.insert(name.to_string(), color);
    }
    
    pub fn get(&self, name: &str) -> Option<MarkColor> {
        self.presets.get(name).copied()
    }
    
    pub fn remove(&mut self, name: &str) -> Option<MarkColor> {
        self.presets.remove(name)
    }
    
    pub fn all(&self) -> &HashMap<String, MarkColor> {
        &self.presets
    }
}

/// Quick mark with keyboard shortcuts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuickMarkConfig {
    pub key: String,           // e.g., "1", "2", "r", "y"
    pub color: MarkColor,
    pub label: Option<String>,
}

/// Complete marks configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarksConfig {
    pub quick_marks: Vec<QuickMarkConfig>,
    pub default_color: MarkColor,
    pub show_in_gutter: bool,
    pub show_in_minimap: bool,
}

impl Default for MarksConfig {
    fn default() -> Self {
        MarksConfig {
            quick_marks: vec![
                QuickMarkConfig { key: "1".to_string(), color: MarkColor::Red, label: Some("Important".to_string()) },
                QuickMarkConfig { key: "2".to_string(), color: MarkColor::Yellow, label: Some("Review".to_string()) },
                QuickMarkConfig { key: "3".to_string(), color: MarkColor::Green, label: Some("OK".to_string()) },
                QuickMarkConfig { key: "4".to_string(), color: MarkColor::Blue, label: Some("Reference".to_string()) },
            ],
            default_color: MarkColor::Yellow,
            show_in_gutter: true,
            show_in_minimap: true,
        }
    }
}
