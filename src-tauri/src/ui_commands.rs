//! UI Commands for LLM Integration
//!
//! Exposes editor operations to LLM via Tauri commands

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Editor state for LLM
#[derive(Debug, Serialize)]
pub struct EditorState {
    pub current_file: Option<String>,
    pub current_line: usize,
    pub total_lines: usize,
    pub cursor_position: CursorPosition,
    pub selection: Option<Selection>,
    pub is_dirty: bool,
    pub view_mode: String,
}

#[derive(Debug, Serialize)]
pub struct CursorPosition {
    pub line: usize,
    pub column: usize,
    pub offset: usize,
}

#[derive(Debug, Serialize)]
pub struct Selection {
    pub start_line: usize,
    pub start_column: usize,
    pub end_line: usize,
    pub end_column: usize,
    pub text: String,
}

/// Get current editor state for LLM
#[tauri::command]
pub fn ui_get_editor_state() -> Result<EditorState, String> {
    // This will be populated from frontend state
    // For now return placeholder
    Ok(EditorState {
        current_file: None,
        current_line: 1,
        total_lines: 0,
        cursor_position: CursorPosition {
            line: 1,
            column: 1,
            offset: 0,
        },
        selection: None,
        is_dirty: false,
        view_mode: "normal".to_string(),
    })
}

/// Go to specific line
#[tauri::command]
pub fn ui_goto_line(line: usize, offset: Option<usize>) -> Result<Value, String> {
    Ok(serde_json::json!({
        "success": true,
        "line": line,
        "offset": offset,
        "action": "goto_line"
    }))
}

/// Line operation request
#[derive(Debug, Deserialize)]
pub struct LineOperationRequest {
    pub operation: String,  // "duplicate", "move_up", "move_down", "delete", "join", "split"
    pub line_number: usize,
    pub content: String,
    pub end_line: Option<usize>,
    pub column: Option<usize>,
}

/// Execute line operation
#[tauri::command]
pub fn ui_execute_line_operation(request: LineOperationRequest) -> Result<Value, String> {
    match request.operation.as_str() {
        "duplicate" => {
            crate::line_operations::edit_duplicate_line(request.content, request.line_number)
        }
        "move_up" => {
            crate::line_operations::edit_move_line_up(request.content, request.line_number)
        }
        "move_down" => {
            crate::line_operations::edit_move_line_down(request.content, request.line_number)
        }
        "delete" => {
            crate::line_operations::edit_delete_line(request.content, request.line_number)
        }
        "join" => {
            let end_line = request.end_line.unwrap_or(request.line_number);
            crate::line_operations::edit_join_lines(request.content, request.line_number, end_line)
        }
        "split" => {
            let column = request.column.unwrap_or(0);
            crate::line_operations::edit_split_line(request.content, request.line_number, column)
        }
        _ => Err(format!("Unknown operation: {}", request.operation)),
    }
}

/// Text operation request
#[derive(Debug, Deserialize)]
pub struct TextOperationRequest {
    pub operation: String,  // "uppercase", "lowercase", "trim_trailing", "trim_leading", "trim_all", "sort"
    pub content: String,
    pub start: Option<usize>,
    pub end: Option<usize>,
    pub start_line: Option<usize>,
    pub end_line: Option<usize>,
    pub ascending: Option<bool>,
    pub comment_prefix: Option<String>,
}

/// Execute text operation
#[tauri::command]
pub fn ui_execute_text_operation(request: TextOperationRequest) -> Result<Value, String> {
    match request.operation.as_str() {
        "uppercase" => {
            let start = request.start.unwrap_or(0);
            let end = request.end.unwrap_or(request.content.len());
            crate::line_operations::edit_to_uppercase(request.content, start, end)
        }
        "lowercase" => {
            let start = request.start.unwrap_or(0);
            let end = request.end.unwrap_or(request.content.len());
            crate::line_operations::edit_to_lowercase(request.content, start, end)
        }
        "trim_trailing" => {
            crate::line_operations::edit_trim_trailing(request.content)
        }
        "trim_leading" => {
            crate::line_operations::edit_trim_leading(request.content)
        }
        "trim_all" => {
            crate::line_operations::edit_trim_all(request.content)
        }
        "sort" => {
            let start_line = request.start_line.unwrap_or(1);
            let end_line = request.end_line.unwrap_or(request.content.lines().count());
            let ascending = request.ascending.unwrap_or(true);
            crate::line_operations::edit_sort_lines(request.content, start_line, end_line, ascending)
        }
        "toggle_comment" => {
            let start_line = request.start_line.unwrap_or(1);
            let end_line = request.end_line.unwrap_or(request.content.lines().count());
            let prefix = request.comment_prefix.unwrap_or_else(|| "//".to_string());
            crate::line_operations::edit_toggle_comment(request.content, start_line, end_line, prefix)
        }
        _ => Err(format!("Unknown operation: {}", request.operation)),
    }
}
