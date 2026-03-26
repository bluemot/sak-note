//! Recent Files Manager
//!
//! Tracks recently opened files with persistence

#![allow(dead_code)]

use serde::{Serialize, Deserialize};
use std::path::PathBuf;
use std::fs;
use std::collections::VecDeque;
use directories::ProjectDirs;

const MAX_RECENT_FILES: usize = 20;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecentFile {
    pub path: String,
    pub name: String,
    pub last_opened: u64, // Unix timestamp
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RecentFilesManager {
    files: VecDeque<RecentFile>,
}

impl RecentFilesManager {
    pub fn new() -> Self {
        Self::load().unwrap_or_default()
    }

    fn config_path() -> Option<PathBuf> {
        ProjectDirs::from("com", "sak", "editor")
            .map(|dirs| dirs.config_dir().join("recent_files.json"))
    }

    fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let path = Self::config_path()
            .ok_or("Could not determine config path")?;
        
        if !path.exists() {
            return Ok(Self::default());
        }

        let content = fs::read_to_string(&path)?;
        let manager: Self = serde_json::from_str(&content)?;
        Ok(manager)
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let path = Self::config_path()
            .ok_or("Could not determine config path")?;
        
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let content = serde_json::to_string_pretty(self)?;
        fs::write(&path, content)?;
        Ok(())
    }

    pub fn add_file(&mut self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        // Remove if already exists
        self.files.retain(|f| f.path != path);

        // Get file name from path
        let name = PathBuf::from(path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Unknown")
            .to_string();

        // Add to front
        self.files.push_front(RecentFile {
            path: path.to_string(),
            name,
            last_opened: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        });

        // Trim to max size
        while self.files.len() > MAX_RECENT_FILES {
            self.files.pop_back();
        }

        self.save()
    }

    pub fn get_files(&self) -> Vec<RecentFile> {
        self.files.iter().cloned().collect()
    }

    pub fn clear(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.files.clear();
        self.save()
    }

    pub fn remove_file(&mut self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.files.retain(|f| f.path != path);
        self.save()
    }
}

// Tauri Commands

#[tauri::command]
pub fn file_get_recent_files() -> Result<Vec<RecentFile>, String> {
    let manager = RecentFilesManager::new();
    Ok(manager.get_files())
}

#[tauri::command]
pub fn file_clear_recent_files() -> Result<(), String> {
    let mut manager = RecentFilesManager::new();
    manager.clear().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn file_add_recent_file(path: String) -> Result<(), String> {
    let mut manager = RecentFilesManager::new();
    manager.add_file(&path).map_err(|e| e.to_string())
}
