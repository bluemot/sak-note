//! Sample SAK Editor Plugin
//!
//! Demonstrates the WASM plugin API with basic editor operations.

use wasm_bindgen::prelude::*;
use serde::{Serialize, Deserialize};

// Import host functions provided by the editor
extern "C" {
    /// Log a message to the editor console
    fn sak_log(ptr: *const u8, len: usize);
    
    /// Get editor content for a file
    fn sak_get_content(
        path_ptr: *const u8,
        path_len: usize,
        out_ptr: *mut u8,
        out_len: usize,
    ) -> i32;
    
    /// Set editor content for a file
    fn sak_set_content(
        path_ptr: *const u8,
        path_len: usize,
        content_ptr: *const u8,
        content_len: usize,
    ) -> i32;
    
    /// Show a notification
    fn sak_show_notification(
        title_ptr: *const u8,
        title_len: usize,
        msg_ptr: *const u8,
        msg_len: usize,
    );
    
    /// Get a plugin setting
    fn sak_get_setting(
        key_ptr: *const u8,
        key_len: usize,
        out_ptr: *mut u8,
        out_len: usize,
    ) -> i32;
    
    /// Set a plugin setting
    fn sak_set_setting(
        key_ptr: *const u8,
        key_len: usize,
        value_ptr: *const u8,
        value_len: usize,
    ) -> i32;
}

/// Plugin initialization - called when plugin is loaded
#[wasm_bindgen]
pub fn __initialize() {
    log("Sample plugin initialized!");
}

/// Plugin shutdown - called when plugin is unloaded
#[wasm_bindgen]
pub fn __shutdown() {
    log("Sample plugin shutting down...");
}

/// Handle editor events
#[wasm_bindgen]
pub fn __on_event(event_json_ptr: *const u8, event_json_len: usize) {
    let event_json = unsafe {
        std::str::from_utf8_unchecked(std::slice::from_raw_parts(event_json_ptr, event_json_len))
    };
    
    log(&format!("Received event: {}", event_json));
    
    // Parse the event and handle specific types
    if let Ok(event) = serde_json::from_str::<EditorEvent>(event_json) {
        match event {
            EditorEvent::FileOpened { path } => {
                log(&format!("File opened: {}", path));
            }
            EditorEvent::FileSaved { path } => {
                log(&format!("File saved: {}", path));
                show_notification("File Saved", &format!("{} has been saved.", path));
            }
            _ => {}
        }
    }
}

/// Execute a capability by ID
#[wasm_bindgen]
pub fn __capability_uppercase(input_ptr: *const u8, input_len: usize) -> i32 {
    let input = unsafe {
        std::str::from_utf8_unchecked(std::slice::from_raw_parts(input_ptr, input_len))
    };
    
    // Parse input as JSON
    let request: UppercaseRequest = match serde_json::from_str(input) {
        Ok(r) => r,
        Err(_) => return 0,
    };
    
    // Convert to uppercase
    let result = UppercaseResult {
        result: request.text.to_uppercase(),
    };
    
    log(&format!("Uppercased: {} -> {}", request.text, result.result));
    
    // Return success (in a real implementation, we'd write to output buffer)
    1
}

/// Word count capability
#[wasm_bindgen]
pub fn __capability_word_count(input_ptr: *const u8, input_len: usize) -> i32 {
    let input = unsafe {
        std::str::from_utf8_unchecked(std::slice::from_raw_parts(input_ptr, input_len))
    };
    
    // Parse input
    let request: WordCountRequest = match serde_json::from_str(input) {
        Ok(r) => r,
        Err(_) => return 0,
    };
    
    // Count words
    let words: Vec<&str> = request.text.split_whitespace().collect();
    let result = WordCountResult {
        word_count: words.len(),
        char_count: request.text.len(),
        char_count_no_spaces: request.text.chars().filter(|c| !c.is_whitespace()).count(),
    };
    
    log(&format!(
        "Word count: {} words, {} chars",
        result.word_count, result.char_count
    ));
    
    // Return success
    1
}

/// Line sort capability - sort lines alphabetically
#[wasm_bindgen]
pub fn __capability_sort_lines(input_ptr: *const u8, input_len: usize) -> i32 {
    let input = unsafe {
        std::str::from_utf8_unchecked(std::slice::from_raw_parts(input_ptr, input_len))
    };
    
    let request: SortLinesRequest = match serde_json::from_str(input) {
        Ok(r) => r,
        Err(_) => return 0,
    };
    
    let mut lines: Vec<String> = request.text.lines().map(|s| s.to_string()).collect();
    
    match request.sort_type.as_str() {
        "ascending" => lines.sort(),
        "descending" => {
            lines.sort();
            lines.reverse();
        }
        "length_asc" => lines.sort_by_key(|s| s.len()),
        "length_desc" => lines.sort_by_key(|s| std::cmp::Reverse(s.len())),
        _ => lines.sort(),
    }
    
    let result = SortLinesResult {
        result: lines.join("\n"),
        line_count: lines.len(),
    };
    
    log(&format!("Sorted {} lines", result.line_count));
    
    1
}

/// Helper: Log a message
fn log(message: &str) {
    unsafe {
        sak_log(message.as_ptr(), message.len());
    }
}

/// Helper: Show notification
fn show_notification(title: &str, message: &str) {
    unsafe {
        sak_show_notification(
            title.as_ptr(),
            title.len(),
            message.as_ptr(),
            message.len(),
        );
    }
}

// Request/Response types

#[derive(Serialize, Deserialize)]
struct UppercaseRequest {
    text: String,
}

#[derive(Serialize, Deserialize)]
struct UppercaseResult {
    result: String,
}

#[derive(Serialize, Deserialize)]
struct WordCountRequest {
    text: String,
}

#[derive(Serialize, Deserialize)]
struct WordCountResult {
    word_count: usize,
    char_count: usize,
    char_count_no_spaces: usize,
}

#[derive(Serialize, Deserialize)]
struct SortLinesRequest {
    text: String,
    sort_type: String, // "ascending", "descending", "length_asc", "length_desc"
}

#[derive(Serialize, Deserialize)]
struct SortLinesResult {
    result: String,
    line_count: usize,
}

/// Editor events
#[derive(Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
enum EditorEvent {
    #[serde(rename = "FileOpened")]
    FileOpened { path: String },
    #[serde(rename = "FileClosed")]
    FileClosed { path: String },
    #[serde(rename = "FileSaved")]
    FileSaved { path: String },
    #[serde(rename = "ContentChanged")]
    ContentChanged { path: String },
    #[serde(rename = "SelectionChanged")]
    SelectionChanged { path: String, start: usize, end: usize },
    #[serde(rename = "CursorMoved")]
    CursorMoved { path: String, offset: usize },
    #[serde(rename = "Startup")]
    Startup,
    #[serde(rename = "Shutdown")]
    Shutdown,
}