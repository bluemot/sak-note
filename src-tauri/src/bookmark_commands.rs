//! Bookmark Tauri Commands
//!
//! Exposes bookmark functionality to the frontend

use serde_json::Value;
use crate::bookmark_engine::BookmarkManager;
use std::sync::Mutex;
use lazy_static::lazy_static;

lazy_static! {
    static ref BOOKMARK_MANAGER: Mutex<BookmarkManager> = Mutex::new(BookmarkManager::new());
}

/// Toggle bookmark at current line
#[tauri::command]
pub fn bookmark_toggle(path: String, line: u32) -> Result<Value, String> {
    let mut manager = BOOKMARK_MANAGER.lock().map_err(|e| e.to_string())?;
    let added = manager.toggle(&path, line as usize)
        .map_err(|e| e.to_string())?;
    
    Ok(serde_json::json!({
        "added": added,
        "line": line
    }))
}

/// Get all bookmarks for a file
#[tauri::command]
pub fn bookmark_get_all(path: String) -> Result<Value, String> {
    let mut manager = BOOKMARK_MANAGER.lock().map_err(|e| e.to_string())?;
    let bookmarks = manager.get_bookmarks(&path)
        .map_err(|e| e.to_string())?;
    
    Ok(serde_json::json!({
        "bookmarks": bookmarks.iter().map(|b| {
            serde_json::json!({
                "line": b.line,
                "label": b.label,
                "note": b.note,
            })
        }).collect::<Vec<_>>()
    }))
}

/// Navigate to next bookmark
#[tauri::command]
pub fn bookmark_next(path: String, current_line: u32) -> Result<Value, String> {
    let mut manager = BOOKMARK_MANAGER.lock().map_err(|e| e.to_string())?;
    let bookmark = manager.next(&path, current_line as usize)
        .map_err(|e| e.to_string())?;
    
    Ok(serde_json::json!({
        "line": bookmark.map(|b| b.line)
    }))
}

/// Navigate to previous bookmark
#[tauri::command]
pub fn bookmark_prev(path: String, current_line: u32) -> Result<Value, String> {
    let mut manager = BOOKMARK_MANAGER.lock().map_err(|e| e.to_string())?;
    let bookmark = manager.prev(&path, current_line as usize)
        .map_err(|e| e.to_string())?;
    
    Ok(serde_json::json!({
        "line": bookmark.map(|b| b.line)
    }))
}

/// Remove bookmark at line
#[tauri::command]
pub fn bookmark_remove(path: String, line: u32) -> Result<Value, String> {
    let mut manager = BOOKMARK_MANAGER.lock().map_err(|e| e.to_string())?;
    let removed = manager.remove(&path, line as usize)
        .map_err(|e| e.to_string())?;
    
    Ok(serde_json::json!({
        "removed": removed
    }))
}

/// Clear all bookmarks for file
#[tauri::command]
pub fn bookmark_clear(path: String) -> Result<Value, String> {
    let mut manager = BOOKMARK_MANAGER.lock().map_err(|e| e.to_string())?;
    manager.clear(&path).map_err(|e| e.to_string())?;
    
    Ok(serde_json::json!({
        "success": true
    }))
}

/// Update bookmark label
#[tauri::command]
pub fn bookmark_update_label(path: String, line: u32, label: String) -> Result<Value, String> {
    let mut manager = BOOKMARK_MANAGER.lock().map_err(|e| e.to_string())?;
    let updated = manager.update_label(&path, line as usize, label)
        .map_err(|e| e.to_string())?;
    
    Ok(serde_json::json!({
        "updated": updated
    }))
}
