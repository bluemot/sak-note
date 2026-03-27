# SAK Editor - LLM Assistant Guide

## Overview

SAK Editor is a modern text editor similar to Notepad++, designed for LLM integration. It provides 30+ tools for AI assistants to control the editor.

## Core Capabilities

### 1. File Operations

**When user says**: "Open the file at ..."

**Use**: `file_open(path: string)`

**Example**:
```
User: "Open /home/user/main.rs"
Action: file_open("/home/user/main.rs")
```

**When user says**: "Show recent files"

**Use**: `file_get_recent_files()`

---

### 2. Navigation

**When user says**: "Go to line 50"

**Use**: `ui_goto_line(line: number, offset?: number)`

**Example**:
```
User: "Go to line 100"
Action: ui_goto_line(100)
```

**When user says**: "Jump to the main function"

**Use**: `semantic_query(path: string, query: string)` → Get line number → `ui_goto_line(line)`

**Example**:
```
User: "Jump to the main function"
Action:
  1. semantic_query("/home/user/main.rs", "find main function")
  2. Result: line 25
  3. ui_goto_line(25)
```

---

### 3. Line Operations

**When user says**: "Duplicate this line" / "Copy this line"

**Use**: `edit_duplicate_line(content: string, line_number: number)`

**Example**:
```
User: "Duplicate line 10"
Action: edit_duplicate_line(current_content, 10)
```

**When user says**: "Move line up" / "Move this line up"

**Use**: `edit_move_line_up(content: string, line_number: number)`

**When user says**: "Delete this line" / "Remove line 15"

**Use**: `edit_delete_line(content: string, line_number: number)`

---

### 4. Text Transformations

**When user says**: "Sort these lines" / "Sort lines 10 to 20"

**Use**: `edit_sort_lines(content: string, start_line: number, end_line: number, ascending: boolean)`

**Example**:
```
User: "Sort lines 5 to 15 alphabetically"
Action: edit_sort_lines(content, 5, 15, true)
```

**When user says**: "Comment out these lines" / "Add comments to lines 10-15"

**Use**: `edit_toggle_comment(content: string, start_line: number, end_line: number, comment_prefix: string)`

**Example**:
```
User: "Comment out lines 10 to 15 in Rust file"
Action: edit_toggle_comment(content, 10, 15, "//")
```

**When user says**: "Trim trailing whitespace"

**Use**: `edit_trim_whitespace(content: string, mode: "trailing" | "leading" | "all")`

**When user says**: "Convert to uppercase" / "Make this uppercase"

**Use**: `edit_to_uppercase(content: string, start: number, end: number)`

---

### 5. Bookmarks

**When user says**: "Add bookmark here" / "Toggle bookmark"

**Use**: `bookmark_toggle(path: string, line: number)`

**When user says**: "Next bookmark" / "Go to next bookmark"

**Use**: `bookmark_next(path: string, current_line: number)`

**When user says**: "Previous bookmark"

**Use**: `bookmark_prev(path: string, current_line: number)`

---

### 6. Search

**When user says**: "Find 'function' in all files"

**Use**: `find_in_files(query: string, directory?: string, filters?: string, case_sensitive?: boolean, regex?: boolean)`

**Example**:
```
User: "Search for 'TODO' in all Rust files"
Action: find_in_files("TODO", null, "*.rs", false, false)
```

---

### 7. Semantic Code Analysis

**When user says**: "What does this function do?" / "Explain this code"

**Use**: `semantic_parse_file(path: string)` → `semantic_query(path: string, query: string)`

**Example**:
```
User: "Explain the main function in this file"
Action:
  1. semantic_parse_file("/home/user/main.rs")
  2. semantic_query("/home/user/main.rs", "explain the main function")
```

**When user says**: "Find all functions that call this"

**Use**: `semantic_query(path: string, "find callers of function X")`

---

## Workflow Examples

### Example 1: Code Review
```
User: "Review this file and mark important lines"

Action:
1. file_open("/home/user/code.rs")
2. semantic_parse_file("/home/user/code.rs")
3. semantic_query("/home/user/code.rs", "find critical functions and potential bugs")
4. For each important line:
   - bookmark_toggle("/home/user/code.rs", line_number)
5. Report findings to user
```

### Example 2: Refactoring
```
User: "Sort all import statements"

Action:
1. Get current content
2. semantic_query("find all import lines")
3. edit_sort_lines(content, first_import_line, last_import_line, true)
4. Apply changes
```

### Example 3: Navigation
```
User: "Go to the function that handles errors"

Action:
1. semantic_query(current_file, "find error handling function")
2. Get line number from result
3. ui_goto_line(line_number)
```

---

## Available Shortcuts Reference

| Function | Shortcut |
|----------|----------|
| Go to line | Ctrl+G |
| Find | Ctrl+F |
| Replace | Ctrl+H |
| Find in files | Ctrl+Shift+F |
| Toggle bookmark | Ctrl+F2 |
| Next bookmark | F2 |
| Prev bookmark | Shift+F2 |
| Duplicate line | Ctrl+D |
| Move line up | Ctrl+Shift+↑ |
| Move line down | Ctrl+Shift+↓ |
| Delete line | Ctrl+L |
| Toggle comment | Ctrl+Q |
| Uppercase | Ctrl+Shift+U |
| Lowercase | Ctrl+U |

---

## Tips for LLM Assistants

1. **Always verify file paths** before operations
2. **Get content first** before editing (to preserve formatting)
3. **Use semantic queries** for natural language code understanding
4. **Check permissions** - plugins have VFS-level security
5. **Provide feedback** - show notifications for completed actions

## Error Handling

When a tool returns an error:
- Inform the user clearly
- Suggest alternative approaches
- Never crash the conversation

Example error handling:
```
Tool: file_open("/nonexistent/file.rs")
Result: Error: File not found

Response: "The file '/nonexistent/file.rs' doesn't exist. Would you like me to create it?"
```

---

## Quick Command Reference

```
file_open(path)
file_save(path)
file_get_recent_files()

ui_goto_line(line, offset?)
ui_get_editor_state()

edit_duplicate_line(content, line)
edit_move_line_up(content, line)
edit_move_line_down(content, line)
edit_delete_line(content, line)
edit_sort_lines(content, start, end, ascending)
edit_toggle_comment(content, start, end, prefix)
edit_trim_whitespace(content, mode)
edit_to_uppercase(content, start, end)
edit_to_lowercase(content, start, end)

bookmark_toggle(path, line)
bookmark_next(path, current_line)
bookmark_prev(path, current_line)

find_in_files(query, directory?, filters?, case_sensitive?, regex?)

semantic_parse_file(path)
semantic_query(path, query)
```
