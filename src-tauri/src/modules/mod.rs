//! Modules for SAK Editor
//! 
//! Each module exposes JSON interfaces for LLM and internal use.

pub mod file_module;

/// Initialize and register all modules
pub fn init() {
    log::info!("Initializing modules...");
    
    // Register file module
    file_module::register();
    
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

// Search for pattern
{
  "module": "file",
  "capability": "search",
  "params": {"path": "...", "pattern": "TODO", "start_offset": 0}
}
```
"#.to_string()
}
