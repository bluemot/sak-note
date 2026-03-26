//! Session Manager
//!
//! Save and restore editor sessions (open files, cursor positions, etc.)

use serde::{Serialize, Deserialize};
use std::path::PathBuf;
use std::fs;
use directories::ProjectDirs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionFile {
    pub path: String,
    pub cursor_line: usize,
    pub cursor_column: usize,
    pub scroll_position: usize,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub name: String,
    pub created_at: u64,
    pub modified_at: u64,
    pub files: Vec<SessionFile>,
}

pub struct SessionManager;

impl SessionManager {
    fn sessions_dir() -> Option<PathBuf> {
        ProjectDirs::from("com", "sak", "editor")
            .map(|dirs| dirs.data_dir().join("sessions"))
    }

    pub fn ensure_dir() -> Result<(), Box<dyn std::error::Error>> {
        if let Some(dir) = Self::sessions_dir() {
            fs::create_dir_all(&dir)?;
        }
        Ok(())
    }

    fn session_path(name: &str) -> Option<PathBuf> {
        Self::sessions_dir().map(|dir| dir.join(format!("{}.json", name)))
    }

    pub fn save_session(session: &Session) -> Result<(), Box<dyn std::error::Error>> {
        Self::ensure_dir()?;
        let path = Self::session_path(&session.name)
            .ok_or("Could not determine session path")?;
        
        let content = serde_json::to_string_pretty(session)?;
        fs::write(&path, content)?;
        Ok(())
    }

    pub fn load_session(name: &str) -> Result<Session, Box<dyn std::error::Error>> {
        let path = Self::session_path(name)
            .ok_or("Could not determine session path")?;
        
        if !path.exists() {
            return Err("Session not found".into());
        }

        let content = fs::read_to_string(&path)?;
        let session: Session = serde_json::from_str(&content)?;
        Ok(session)
    }

    pub fn list_sessions() -> Result<Vec<Session>, Box<dyn std::error::Error>> {
        let dir = Self::sessions_dir()
            .ok_or("Could not determine sessions directory")?;
        
        if !dir.exists() {
            return Ok(vec![]);
        }

        let mut sessions = vec![];
        for entry in fs::read_dir(&dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().map(|e| e == "json").unwrap_or(false) {
                if let Ok(content) = fs::read_to_string(&path) {
                    if let Ok(session) = serde_json::from_str::<Session>(&content) {
                        sessions.push(session);
                    }
                }
            }
        }
        
        // Sort by modified date (newest first)
        sessions.sort_by(|a, b| b.modified_at.cmp(&a.modified_at));
        Ok(sessions)
    }

    pub fn delete_session(name: &str) -> Result<(), Box<dyn std::error::Error>> {
        let path = Self::session_path(name)
            .ok_or("Could not determine session path")?;
        
        if path.exists() {
            fs::remove_file(&path)?;
        }
        Ok(())
    }
}

// Tauri Commands

#[tauri::command]
pub fn session_save(session: Session) -> Result<(), String> {
    SessionManager::save_session(&session).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn session_load(name: String) -> Result<Session, String> {
    SessionManager::load_session(&name).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn session_list() -> Result<Vec<Session>, String> {
    SessionManager::list_sessions().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn session_delete(name: String) -> Result<(), String> {
    SessionManager::delete_session(&name).map_err(|e| e.to_string())
}
