use serde::{Serialize, Deserialize};

// Re-export chrono for timestamp logging
pub use chrono;

mod file_engine;
mod mark_engine;
mod sak_format;
mod modular;
mod modules;
mod vfs;
mod semantic;
mod mcp_server;
mod bookmark_engine;
mod bookmark_commands;
mod sftp_site_manager;
mod sftp_commands;
mod recent_files;
mod line_operations;
mod ui_commands;
mod session_manager;
mod print_manager;
mod find_in_files;
mod plugin_runtime;

use bookmark_commands::{bookmark_toggle, bookmark_get_all, bookmark_next, bookmark_prev, bookmark_remove, bookmark_clear, bookmark_update_label};
use sftp_commands::{sftp_list_sites, sftp_add_site, sftp_update_site, sftp_remove_site, sftp_connect_site, sftp_test_connection};
use ui_commands::{ui_goto_line, ui_get_editor_state, ui_execute_line_operation, ui_execute_text_operation};

use file_engine::{FileEngine, EditOp, FileInfo, CHUNK_SIZE};
use mark_engine::{MarkEngine, MarkColor, Mark, MarkUpdate, MarkExport};

// ============== Request/Response Types ==============

#[derive(Debug, Serialize)]
struct OpenFileResponse {
    path: String,
    size: u64,
    chunks: usize,
    chunk_size: usize,
    editable: bool,
    has_changes: bool,
}

#[derive(Debug, Deserialize)]
struct GetChunkRequest {
    path: String,
    chunk_id: usize,
}

#[derive(Debug, Serialize)]
struct ChunkResponse {
    id: usize,
    offset: usize,
    length: usize,
    text: String,
}

#[derive(Debug, Deserialize)]
struct GetTextRequest {
    path: String,
    start: usize,
    end: usize,
}

#[derive(Debug, Deserialize)]
struct GetLinesRequest {
    path: String,
    start_line: usize,
    end_line: usize,
}

#[derive(Debug, Deserialize)]
struct GetHexRequest {
    path: String,
    start: usize,
    length: usize,
}

#[derive(Debug, Serialize)]
struct HexRow {
    offset: usize,
    hex: String,
    ascii: String,
}

#[derive(Debug, Serialize)]
struct HexResponse {
    rows: Vec<HexRow>,
}

#[derive(Debug, Deserialize)]
struct InsertRequest {
    path: String,
    offset: usize,
    data: Vec<u8>,
}

#[derive(Debug, Deserialize)]
struct DeleteRequest {
    path: String,
    offset: usize,
    length: usize,
}

#[derive(Debug, Deserialize)]
struct ReplaceRequest {
    path: String,
    offset: usize,
    length: usize,
    data: Vec<u8>,
}

#[derive(Debug, Deserialize)]
struct SearchRequest {
    path: String,
    pattern: String,
    is_hex: bool,
    start_offset: Option<usize>,
}

#[derive(Debug, Serialize)]
struct SearchResponse {
    offset: usize,
    length: usize,
    preview: String,
}

#[derive(Debug, Serialize)]
struct SearchResultResponse {
    results: Vec<SearchResponse>,
    total: usize,
}

#[derive(Debug, Serialize)]
struct EditStatus {
    has_changes: bool,
    can_undo: bool,
    can_redo: bool,
    effective_size: u64,
}

#[derive(Debug, Deserialize)]
struct SaveAsRequest {
    source_path: String,
    target_path: String,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum EditRequest {
    Insert { offset: usize, data: Vec<u8> },
    Delete { offset: usize, length: usize },
    Replace { offset: usize, length: usize, data: Vec<u8> },
}

#[derive(Debug, Deserialize)]
struct ApplyEditRequest {
    path: String,
    edit: EditRequest,
}

// ============== Mark Request/Response Types ==============

#[derive(Debug, Deserialize)]
struct CreateMarkRequest {
    path: String,
    start: usize,
    end: usize,
    color: MarkColor,
    label: Option<String>,
    note: Option<String>,
}

#[derive(Debug, Deserialize)]
struct UpdateMarkRequest {
    path: String,
    id: String,
    updates: MarkUpdate,
}

#[derive(Debug, Deserialize)]
struct DeleteMarkRequest {
    path: String,
    id: String,
}

#[derive(Debug, Deserialize)]
struct GetMarksRequest {
    path: String,
    start: Option<usize>,
    end: Option<usize>,
}

#[derive(Debug, Serialize)]
struct MarksResponse {
    marks: Vec<Mark>,
}

#[derive(Debug, Serialize)]
struct MarkCountResponse {
    count: usize,
}

// ============== Tauri Commands ==============

/// Open file command - entry point for file open workflow from frontend
#[tauri::command]
async fn open_file(path: String) -> Result<OpenFileResponse, String> {
    let start_time = std::time::Instant::now();
    let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
    
    log::info!("[lib::open_file] === OPEN FILE COMMAND STARTED ===");
    log::info!("[lib::open_file] Received path: {}", path);
    log::debug!("[lib::open_file] Timestamp: {}", timestamp);
    
    // Open as editable by default
    log::debug!("[lib::open_file] Calling FileEngine::open_for_edit()");
    match FileEngine::open_for_edit(&path) {
        Ok(manager) => {
            log::debug!("[lib::open_file] FileEngine::open_for_edit() returned Ok, acquiring read lock");
            let guard = manager.read().map_err(|e| {
                log::error!("[lib::open_file] Failed to acquire read lock: {}", e);
                format!("Lock error: {}", e)
            })?;
            
            let effective_size = guard.effective_size();
            let chunks = ((effective_size + CHUNK_SIZE as u64 - 1) / CHUNK_SIZE as u64) as usize;
            let has_changes = guard.has_changes();
            
            log::info!("[lib::open_file] File opened successfully:");
            log::info!("[lib::open_file]   Path: {}", path);
            log::info!("[lib::open_file]   Size: {} bytes", effective_size);
            log::info!("[lib::open_file]   Chunks: {}", chunks);
            log::info!("[lib::open_file]   Chunk size: {} bytes", CHUNK_SIZE);
            log::info!("[lib::open_file]   Has changes: {}", has_changes);
            
            let elapsed = start_time.elapsed();
            log::info!("[lib::open_file] === OPEN FILE COMMAND COMPLETED in {:?} ===", elapsed);
            
            Ok(OpenFileResponse {
                path: path.clone(),
                size: effective_size,
                chunks,
                chunk_size: CHUNK_SIZE,
                editable: true,
                has_changes,
            })
        }
        Err(e) => {
            let elapsed = start_time.elapsed();
            log::error!("[lib::open_file] Failed to open file after {:?}: {}", elapsed, e);
            log::error!("[lib::open_file] Path: {}", path);
            Err(format!("Failed to open file: {}", e))
        }
    }
}

/// Get text command - retrieves text content from file
#[tauri::command]
async fn get_text(req: GetTextRequest) -> Result<String, String> {
    let start_time = std::time::Instant::now();
    let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
    
    log::info!("[lib::get_text] === GET TEXT COMMAND STARTED ===");
    log::info!("[lib::get_text] Request: path={}, start={}, end={}", req.path, req.start, req.end);
    log::debug!("[lib::get_text] Timestamp: {}", timestamp);
    
    log::debug!("[lib::get_text] Looking up editable file manager for path: {}", req.path);
    match FileEngine::get_editable(&req.path) {
        Some(manager) => {
            log::debug!("[lib::get_text] File manager found, acquiring read lock");
            let guard = manager.read().map_err(|e| {
                log::error!("[lib::get_text] Failed to acquire read lock: {}", e);
                format!("Lock error: {}", e)
            })?;
            
            let length = req.end.saturating_sub(req.start);
            log::debug!("[lib::get_text] Calling get_text() with offset={}, length={}", req.start, length);
            
            let text = guard.get_text(req.start, length);
            let text_len = text.len();
            
            let elapsed = start_time.elapsed();
            log::info!("[lib::get_text] Text retrieved successfully: {} bytes in {:?}", text_len, elapsed);
            log::debug!("[lib::get_text] Preview (first 100 chars): {}", 
                if text_len > 100 { &text[..100] } else { &text });
            log::info!("[lib::get_text] === GET TEXT COMMAND COMPLETED ===");
            
            Ok(text)
        }
        None => {
            log::error!("[lib::get_text] File not open in editable mode: {}", req.path);
            Err("File not open".to_string())
        }
    }
}

/// Get lines by line number range (0-based, start inclusive, end exclusive)
#[tauri::command]
async fn get_lines(req: GetLinesRequest) -> Result<String, String> {
    let start_time = std::time::Instant::now();
    log::info!("[lib::get_lines] Request: path={}, start_line={}, end_line={}", req.path, req.start_line, req.end_line);

    // Try editable first
    match FileEngine::get_editable(&req.path) {
        Some(manager) => {
            let guard = manager.read().map_err(|e| format!("Lock error: {}", e))?;
            let text = guard.get_lines(req.start_line, req.end_line);
            let elapsed = start_time.elapsed();
            log::info!("[lib::get_lines] Got {} chars in {:?}", text.len(), elapsed);
            Ok(text)
        }
        None => {
            // Try read-only cache
            match FileEngine::get_file(&req.path) {
                Some(chunk_mgr) => {
                    match chunk_mgr.get_lines(req.start_line, req.end_line) {
                        Some(text) => {
                            let elapsed = start_time.elapsed();
                            log::info!("[lib::get_lines] Got {} chars in {:?}", text.len(), elapsed);
                            Ok(text)
                        }
                        None => Err("Failed to get lines from file".to_string()),
                    }
                }
                None => Err("File not open".to_string()),
            }
        }
    }
}

/// Get the number of lines in a file
#[tauri::command]
async fn line_count(path: String) -> Result<usize, String> {
    log::info!("[lib::line_count] Request: path={}", path);

    // Try editable first
    match FileEngine::get_editable(&path) {
        Some(manager) => {
            let guard = manager.read().map_err(|e| format!("Lock error: {}", e))?;
            Ok(guard.line_count())
        }
        None => {
            // Try read-only cache
            match FileEngine::get_file(&path) {
                Some(chunk_mgr) => Ok(chunk_mgr.line_count()),
                None => Err("File not open".to_string()),
            }
        }
    }
}

/// Build line index for O(1) line lookups
#[tauri::command]
async fn build_line_index(path: String) -> Result<FileInfo, String> {
    log::info!("[lib::build_line_index] Building line index for: {}", path);

    let manager = FileEngine::get_editable(&path)
        .ok_or_else(|| "File not open".to_string())?;
    {
        let mut guard = manager.write().map_err(|e| e.to_string())?;
        guard.build_line_index();
    }

    // Return updated file info with line count from index
    FileEngine::get_file_info(&path)
        .ok_or_else(|| "Failed to get file info after building index".to_string())
}

/// Get byte offset for a specific line using pre-built index
#[tauri::command]
async fn get_line_offset(path: String, line: usize) -> Result<usize, String> {
    let manager = FileEngine::get_editable(&path)
        .ok_or_else(|| "File not open".to_string())?;
    let guard = manager.read().map_err(|e| e.to_string())?;

    guard.line_offset_indexed(line)
        .ok_or_else(|| "Line index not available or line out of bounds".to_string())
}

/// Get chunk command - retrieves a specific chunk from file
#[tauri::command]
async fn get_chunk(req: GetChunkRequest) -> Result<ChunkResponse, String> {
    let start_time = std::time::Instant::now();
    log::debug!("[lib::get_chunk] === GET CHUNK COMMAND STARTED ===");
    log::debug!("[lib::get_chunk] Request: path={}, chunk_id={}", req.path, req.chunk_id);
    
    match FileEngine::get_editable(&req.path) {
        Some(manager) => {
            log::debug!("[lib::get_chunk] File manager found, acquiring read lock");
            let guard = manager.read().map_err(|e| {
                log::error!("[lib::get_chunk] Failed to acquire read lock: {}", e);
                format!("Lock error: {}", e)
            })?;
            
            let offset = req.chunk_id * CHUNK_SIZE;
            let length = CHUNK_SIZE.min((guard.effective_size() as usize).saturating_sub(offset));
            log::debug!("[lib::get_chunk] Calculated: offset={}, length={}", offset, length);
            
            if length == 0 {
                log::warn!("[lib::get_chunk] Chunk out of bounds: chunk_id={}, offset={}", req.chunk_id, offset);
                return Err("Chunk out of bounds".to_string());
            }
            
            log::debug!("[lib::get_chunk] Calling get_range()");
            let data = guard.get_range(offset, length);
            let text = String::from_utf8_lossy(&data).to_string();
            
            let elapsed = start_time.elapsed();
            log::debug!("[lib::get_chunk] Chunk retrieved: {} bytes in {:?}", data.len(), elapsed);
            log::debug!("[lib::get_chunk] === GET CHUNK COMMAND COMPLETED ===");
            
            Ok(ChunkResponse {
                id: req.chunk_id,
                offset,
                length,
                text,
            })
        }
        None => {
            log::error!("[lib::get_chunk] File not open: {}", req.path);
            Err("File not open".to_string())
        }
    }
}

#[tauri::command]
async fn get_hex_view(req: GetHexRequest) -> Result<HexResponse, String> {
    if let Some(manager) = FileEngine::get_editable(&req.path) {
        let guard = manager.read().map_err(|e| format!("Lock error: {}", e))?;
        let data = guard.get_range(req.start, req.length);
        
        let rows: Vec<HexRow> = data
            .chunks(16)
            .enumerate()
            .map(|(idx, bytes)| {
                let offset = req.start + idx * 16;
                let hex = bytes
                    .iter()
                    .map(|b| format!("{:02x}", b))
                    .collect::<Vec<_>>()
                    .join(" ");
                let ascii: String = bytes
                    .iter()
                    .map(|b| {
                        if b.is_ascii_graphic() || *b == b' ' {
                            *b as char
                        } else {
                            '.'
                        }
                    })
                    .collect();
                HexRow { offset, hex, ascii }
            })
            .collect();
            
        Ok(HexResponse { rows })
    } else {
        Err("File not open".to_string())
    }
}

// ============== Edit Commands ==============

#[tauri::command]
async fn insert_bytes(req: InsertRequest) -> Result<(), String> {
    if let Some(manager) = FileEngine::get_editable(&req.path) {
        let mut guard = manager.write().map_err(|e| format!("Lock error: {}", e))?;
        guard.apply_edit(EditOp::Insert { 
            offset: req.offset, 
            data: req.data 
        });
        Ok(())
    } else {
        Err("File not open in editable mode".to_string())
    }
}

#[tauri::command]
async fn delete_bytes(req: DeleteRequest) -> Result<(), String> {
    if let Some(manager) = FileEngine::get_editable(&req.path) {
        let mut guard = manager.write().map_err(|e| format!("Lock error: {}", e))?;
        guard.apply_edit(EditOp::Delete { 
            offset: req.offset, 
            length: req.length 
        });
        Ok(())
    } else {
        Err("File not open in editable mode".to_string())
    }
}

#[tauri::command]
async fn replace_bytes(req: ReplaceRequest) -> Result<(), String> {
    if let Some(manager) = FileEngine::get_editable(&req.path) {
        let mut guard = manager.write().map_err(|e| format!("Lock error: {}", e))?;
        guard.apply_edit(EditOp::Replace { 
            offset: req.offset, 
            length: req.length, 
            data: req.data 
        });
        Ok(())
    } else {
        Err("File not open in editable mode".to_string())
    }
}

#[tauri::command]
async fn apply_edit(req: ApplyEditRequest) -> Result<(), String> {
    if let Some(manager) = FileEngine::get_editable(&req.path) {
        let mut guard = manager.write().map_err(|e| format!("Lock error: {}", e))?;
        let op = match req.edit {
            EditRequest::Insert { offset, data } => EditOp::Insert { offset, data },
            EditRequest::Delete { offset, length } => EditOp::Delete { offset, length },
            EditRequest::Replace { offset, length, data } => EditOp::Replace { offset, length, data },
        };
        guard.apply_edit(op);
        Ok(())
    } else {
        Err("File not open in editable mode".to_string())
    }
}

// ============== Undo/Redo Commands ==============

#[tauri::command]
async fn undo(path: String) -> Result<bool, String> {
    if let Some(manager) = FileEngine::get_editable(&path) {
        let mut guard = manager.write().map_err(|e| format!("Lock error: {}", e))?;
        Ok(guard.undo())
    } else {
        Err("File not open in editable mode".to_string())
    }
}

#[tauri::command]
async fn redo(path: String) -> Result<bool, String> {
    if let Some(manager) = FileEngine::get_editable(&path) {
        let mut guard = manager.write().map_err(|e| format!("Lock error: {}", e))?;
        Ok(guard.redo())
    } else {
        Err("File not open in editable mode".to_string())
    }
}

#[tauri::command]
async fn get_edit_status(path: String) -> Result<EditStatus, String> {
    if let Some(manager) = FileEngine::get_editable(&path) {
        let guard = manager.read().map_err(|e| format!("Lock error: {}", e))?;
        Ok(EditStatus {
            has_changes: guard.has_changes(),
            can_undo: guard.can_undo(),
            can_redo: guard.can_redo(),
            effective_size: guard.effective_size(),
        })
    } else {
        Err("File not open in editable mode".to_string())
    }
}

// ============== Search Commands ==============

#[tauri::command]
async fn search(req: SearchRequest) -> Result<SearchResultResponse, String> {
    if let Some(manager) = FileEngine::get_editable(&req.path) {
        let guard = manager.read().map_err(|e| format!("Lock error: {}", e))?;
        
        let results = if req.is_hex {
            // Parse hex pattern
            let pattern: Vec<u8> = req.pattern
                .split_whitespace()
                .map(|s| u8::from_str_radix(s, 16).unwrap_or(0))
                .collect();
            guard.search(&pattern, req.start_offset.unwrap_or(0))
        } else {
            guard.search_text(&req.pattern, req.start_offset.unwrap_or(0))
        };
        
        let search_results: Vec<SearchResponse> = results
            .iter()
            .map(|&offset| {
                let preview_data = guard.get_range(offset.saturating_sub(16), req.pattern.len() + 32);
                let preview = String::from_utf8_lossy(&preview_data).to_string();
                SearchResponse {
                    offset,
                    length: req.pattern.len(),
                    preview,
                }
            })
            .collect();
            
        Ok(SearchResultResponse {
            total: search_results.len(),
            results: search_results,
        })
    } else {
        Err("File not open".to_string())
    }
}

#[tauri::command]
async fn replace_all(path: String, pattern: String, replacement: String, is_hex: bool) -> Result<usize, String> {
    if let Some(manager) = FileEngine::get_editable(&path) {
        let mut guard = manager.write().map_err(|e| format!("Lock error: {}", e))?;
        
        let pattern_bytes = if is_hex {
            pattern.split_whitespace()
                .map(|s| u8::from_str_radix(s, 16).unwrap_or(0))
                .collect::<Vec<u8>>()
        } else {
            pattern.into_bytes()
        };
        
        let replacement_bytes = if is_hex {
            replacement.split_whitespace()
                .map(|s| u8::from_str_radix(s, 16).unwrap_or(0))
                .collect::<Vec<u8>>()
        } else {
            replacement.into_bytes()
        };
        
        let count = guard.replace_all(&pattern_bytes, &replacement_bytes);
        Ok(count)
    } else {
        Err("File not open in editable mode".to_string())
    }
}

// ============== Save Commands ==============

#[tauri::command]
async fn save_file(path: String) -> Result<(), String> {
    if let Some(manager) = FileEngine::get_editable(&path) {
        let mut guard = manager.write().map_err(|e| format!("Lock error: {}", e))?;
        guard.save().map_err(|e| format!("Save failed: {}", e))?;
        Ok(())
    } else {
        Err("File not open in editable mode".to_string())
    }
}

#[tauri::command]
async fn save_as(req: SaveAsRequest) -> Result<(), String> {
    if let Some(manager) = FileEngine::get_editable(&req.source_path) {
        let guard = manager.read().map_err(|e| format!("Lock error: {}", e))?;
        guard.save_as(&req.target_path).map_err(|e| format!("Save as failed: {}", e))?;
        Ok(())
    } else {
        Err("File not open".to_string())
    }
}

#[tauri::command]
async fn close_file(path: String) -> Result<(), String> {
    FileEngine::close_editable(&path);
    FileEngine::close_file(&path);
    Ok(())
}

#[tauri::command]
async fn get_file_info(path: String) -> Result<FileInfo, String> {
    match FileEngine::get_file_info(&path) {
        Some(info) => Ok(info),
        None => Err("File not found in cache".to_string()),
    }
}

// ============== Mark Commands ==============

#[tauri::command]
async fn create_mark(req: CreateMarkRequest) -> Result<Mark, String> {
    let mut marks = MarkEngine::get_or_create(&req.path);
    
    let mark = Mark {
        id: String::new(), // Will be generated
        start: req.start,
        end: req.end,
        color: req.color,
        label: req.label,
        note: req.note,
        created_at: current_timestamp(),
        updated_at: current_timestamp(),
    };
    
    marks.add_mark(mark).map_err(|e| e)
}

#[tauri::command]
async fn update_mark(req: UpdateMarkRequest) -> Result<Mark, String> {
    let mut marks = MarkEngine::get_or_create(&req.path);
    marks.update_mark(&req.id, req.updates).map_err(|e| e)
}

#[tauri::command]
async fn delete_mark(req: DeleteMarkRequest) -> Result<Mark, String> {
    let mut marks = MarkEngine::get_or_create(&req.path);
    marks.delete_mark(&req.id)
        .ok_or_else(|| "Mark not found".to_string())
}

#[tauri::command]
async fn get_marks(req: GetMarksRequest) -> Result<MarksResponse, String> {
    let marks = MarkEngine::get_or_create(&req.path);
    
    let marks_vec: Vec<Mark> = if let (Some(start), Some(end)) = (req.start, req.end) {
        marks.get_marks_in_range(start, end)
            .into_iter()
            .cloned()
            .collect()
    } else {
        marks.get_all_marks()
            .into_iter()
            .cloned()
            .collect()
    };
    
    Ok(MarksResponse { marks: marks_vec })
}

#[tauri::command]
async fn get_marks_at(path: String, offset: usize) -> Result<MarksResponse, String> {
    let marks = MarkEngine::get_or_create(&path);
    let marks_vec: Vec<Mark> = marks.get_marks_at(offset)
        .into_iter()
        .cloned()
        .collect();
    
    Ok(MarksResponse { marks: marks_vec })
}

#[tauri::command]
async fn clear_marks(path: String) -> Result<(), String> {
    let mut marks = MarkEngine::get_or_create(&path);
    marks.clear_all();
    Ok(())
}

#[tauri::command]
async fn delete_marks_by_color(path: String, color: MarkColor) -> Result<usize, String> {
    let mut marks = MarkEngine::get_or_create(&path);
    Ok(marks.delete_by_color(color))
}

#[tauri::command]
async fn get_mark_count(path: String) -> Result<MarkCountResponse, String> {
    let marks = MarkEngine::get_or_create(&path);
    Ok(MarkCountResponse { count: marks.count() })
}

#[tauri::command]
async fn export_marks(path: String) -> Result<MarkExport, String> {
    let marks = MarkEngine::get_or_create(&path);
    Ok(marks.export())
}

/// Helper: get current timestamp
fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// Execute a module capability
#[tauri::command]
async fn execute_module(
    module: String,
    capability: String,
    input: serde_json::Value,
) -> Result<serde_json::Value, String> {
    modular::execute_module(&module, &capability, input)
        .map_err(|e| e.to_string())
}

pub fn run() {
    log::info!("[lib] SAK Editor run() starting...");
    
    // Initialize plugin system
    log::info!("[lib] Initializing plugin system...");
    if let Err(e) = plugin_runtime::manager::init_plugin_manager() {
        log::error!("[lib] Failed to initialize plugin manager: {}", e);
    } else {
        log::info!("[lib] Plugin manager initialized successfully");
    }
    
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            // File operations
            open_file,
            get_chunk,
            get_text,
            get_lines,
            line_count,
            get_hex_view,
            close_file,
            get_file_info,
            // Line index commands
            build_line_index,
            get_line_offset,
            // Edit operations
            insert_bytes,
            delete_bytes,
            replace_bytes,
            apply_edit,
            // Undo/redo
            undo,
            redo,
            get_edit_status,
            // Search
            search,
            replace_all,
            // Save
            save_file,
            save_as,
            // Marks
            create_mark,
            update_mark,
            delete_mark,
            get_marks,
            get_marks_at,
            clear_marks,
            delete_marks_by_color,
            get_mark_count,
            export_marks,
            // Module execution
            execute_module,
            // Semantic LLM-friendly commands
            semantic::commands::semantic_parse_file,
            semantic::commands::semantic_query,
            semantic::commands::semantic_export_llm,
            semantic::commands::semantic_edit,
            semantic::commands::semantic_parse_edit_request,
            semantic::commands::semantic_conversation_start,
            semantic::commands::semantic_conversation_send,
            semantic::commands::semantic_conversation_history,
            // Bookmark commands
            bookmark_toggle,
            bookmark_get_all,
            bookmark_next,
            bookmark_prev,
            bookmark_remove,
            bookmark_clear,
            bookmark_update_label,
            // SFTP Site Manager commands
            sftp_list_sites,
            sftp_add_site,
            sftp_update_site,
            sftp_remove_site,
            sftp_connect_site,
            sftp_test_connection,
            // Recent files commands
            recent_files::file_get_recent_files,
            recent_files::file_clear_recent_files,
            recent_files::file_add_recent_file,
            // Line operations commands
            line_operations::edit_duplicate_line,
            line_operations::edit_move_line_up,
            line_operations::edit_move_line_down,
            line_operations::edit_delete_line,
            line_operations::edit_join_lines,
            line_operations::edit_split_line,
            line_operations::edit_trim_trailing,
            line_operations::edit_trim_leading,
            line_operations::edit_trim_all,
            line_operations::edit_to_uppercase,
            line_operations::edit_to_lowercase,
            line_operations::edit_sort_lines,
            line_operations::edit_toggle_comment,
            // Go to line
            ui_goto_line,
            // LLM-friendly UI commands
            ui_get_editor_state,
            ui_execute_line_operation,
            ui_execute_text_operation,
            // Session management
            session_manager::session_save,
            session_manager::session_load,
            session_manager::session_list,
            session_manager::session_delete,
            // Print
            print_manager::file_print,
            print_manager::file_export_pdf,
            // Find in Files
            find_in_files::find_in_files,
            // Plugin management commands
            plugin_runtime::commands::plugin_init,
            plugin_runtime::commands::plugin_discover,
            plugin_runtime::commands::plugin_load_all,
            plugin_runtime::commands::plugin_load,
            plugin_runtime::commands::plugin_unload,
            plugin_runtime::commands::plugin_list_loaded,
            plugin_runtime::commands::plugin_get_info,
            plugin_runtime::commands::plugin_execute,
            plugin_runtime::commands::plugin_set_enabled,
            plugin_runtime::commands::plugin_get_capabilities,
            plugin_runtime::commands::plugin_broadcast_event,
            plugin_runtime::commands::plugin_get_directory,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
