//! Modules for SAK Editor
//! 
//! Each module exposes JSON interfaces for LLM and internal use.

pub mod file_module;
pub mod marks_module;
pub mod llm_module;

/// Initialize and register all modules
pub fn init() {
    log::info!("Initializing modules...");
    
    // Register file module
    file_module::register();
    
    // Register marks module
    marks_module::register();
    
    // Register llm module
    llm_module::register();
    
    log::info!("Modules initialized");
}

/// Get module documentation for LLM
pub fn get_llm_documentation() -> String {
    r#"# SAK Editor Module API

## File Module (`file`)

Large file handling with memory-mapped I/O and edit journaling.

### Capabilities

#### `file.open`
Open a file for editing.
- Input: `{"path": "/path/to/file"}`
- Output: `{"path": "...", "size": 1234, "chunks": 20, "editable": true}`

#### `file.close`
Close a file.
- Input: `{"path": "/path/to/file"}`
- Output: `null`

#### `file.read`
Read raw bytes from file.
- Input: `{"path": "...", "offset": 0, "length": 1024}`
- Output: `{"data": [72, 101, 108, 108, 111], "offset": 0, "length": 5}`

#### `file.read_text`
Read text from file (UTF-8).
- Input: `{"path": "...", "offset": 0, "length": 1024}`
- Output: `{"text": "Hello", "offset": 0, "length": 5}`

#### `file.insert`
Insert bytes at position.
- Input: `{"path": "...", "offset": 10, "data": [65, 66, 67]}`
- Output: `{"success": true}`

#### `file.delete`
Delete bytes at position.
- Input: `{"path": "...", "offset": 10, "length": 5}`
- Output: `{"success": true}`

#### `file.replace`
Replace bytes at position.
- Input: `{"path": "...", "offset": 10, "length": 5, "data": [65, 66]}`
- Output: `{"success": true}`

#### `file.save`
Save changes to file.
- Input: `{"path": "..."}`
- Output: `{"success": true}`

#### `file.save_as`
Save to new file.
- Input: `{"source_path": "...", "target_path": "..."}`
- Output: `{"success": true}`

#### `file.undo`
Undo last operation.
- Input: `{"path": "..."}`
- Output: `{"success": true, "can_undo": false, "can_redo": true}`

#### `file.redo`
Redo last undone operation.
- Input: `{"path": "..."}`
- Output: `{"success": true, "can_undo": true, "can_redo": false}`

#### `file.search`
Search for pattern in file.
- Input: `{"path": "...", "pattern": "hello", "is_hex": false, "start_offset": 0}`
- Output: `{"results": [{"offset": 0, "length": 5, "preview": "hello world"}], "total": 1}`

#### `file.get_info`
Get file information.
- Input: `{"path": "..."}`
- Output: `{"path": "...", "size": 1234, "effective_size": 1234, "has_changes": false, "can_undo": false, "can_redo": false}`

#### `file.get_hex`
Get hex representation of bytes.
- Input: `{"path": "...", "offset": 0, "length": 256}`
- Output: `{"rows": [{"offset": 0, "hex": "48 65 6c 6c 6f", "ascii": "Hello"}]}`

## Marks Module (`marks`)

Color highlighting and annotation system for files.

### Capabilities

#### `marks.create`
Create a new color mark.
- Input: `{"path": "...", "start": 0, "end": 10, "color": "red", "label": "Important", "note": "Key section"}`
- Output: `{"id": "mark_0_1234567890", "start": 0, "end": 10, "color": "red", "label": "Important", "note": "Key section", "created_at": 1234567890, "updated_at": 1234567890}`

#### `marks.update`
Update an existing mark.
- Input: `{"path": "...", "id": "mark_0_1234567890", "updates": {"color": "yellow", "label": "Review"}}`
- Output: Updated mark object

#### `marks.delete`
Delete a mark by ID.
- Input: `{"path": "...", "id": "mark_0_1234567890"}`
- Output: `{"success": true, "deleted_id": "mark_0_1234567890"}`

#### `marks.get`
Get all marks or marks in range.
- Input: `{"path": "...", "start": 0, "end": 1000}` (range optional)
- Output: `{"marks": [...], "count": 5}`

#### `marks.get_at`
Get marks at specific position.
- Input: `{"path": "...", "offset": 50}`
- Output: `{"marks": [...], "count": 2}`

#### `marks.clear`
Clear all marks.
- Input: `{"path": "..."}`
- Output: `{"success": true, "cleared_count": 10}`

#### `marks.delete_by_color`
Delete marks by color.
- Input: `{"path": "...", "color": "red"}`
- Output: `{"deleted_count": 3}`

#### `marks.count`
Get mark count with breakdown by color.
- Input: `{"path": "..."}`
- Output: `{"count": 10, "by_color": {"red": 3, "yellow": 5, "green": 2}}`

#### `marks.export`
Export marks to JSON.
- Input: `{"path": "..."}`
- Output: `{"path": "...", "marks_count": 10, "marks": [...]}`

#### `marks.import`
Import marks from JSON.
- Input: `{"path": "...", "data": {"marks": [...]}}`
- Output: `{"success": true, "imported_count": 10}`

#### `llm.ask_about_file`
Ask questions about file content.
- Input: `{"file_path": "...", "question": "What does this function do?", "context_id": "..."}`
- Output: `{"response": "...", "relevant_sections": ["lines 10-20"]}`

#### `llm.generate`
Generate text from prompt.
- Input: `{"prompt": "Write a README", "template": "doc"}`
- Output: `{"generated": "...", "template_used": "doc"}`

## Usage Example

```json
// Open a file
{
  "module": "file",
  "capability": "open",
  "params": {"path": "/home/user/document.txt"}
}

// Read first 1KB
{
  "module": "file",
  "capability": "read_text",
  "params": {"path": "/home/user/document.txt", "offset": 0, "length": 1024}
}

// Create a mark
{
  "module": "marks",
  "capability": "create",
  "params": {"path": "...", "start": 0, "end": 10, "color": "yellow", "label": "Important"}
}

// Get all marks
{
  "module": "marks",
  "capability": "get",
  "params": {"path": "..."}
}

// Search for pattern
{
  "module": "file",
  "capability": "search",
  "params": {"path": "...", "pattern": "TODO", "start_offset": 0}
}
```
"#.to_string()
}
