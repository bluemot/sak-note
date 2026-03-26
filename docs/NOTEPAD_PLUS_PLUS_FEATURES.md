# SAK Editor - Notepad++ Feature Implementation Plan

## Core Features (Priority 1)

### 1. Multi-Document Interface (Tabs)
- [x] Basic tab structure exists
- [ ] Multiple files open simultaneously
- [ ] Drag and drop tabs
- [ ] Tab close button
- [ ] Unsaved indicator on tabs
- [ ] Tab context menu (close others, close all, save)

### 2. Syntax Highlighting
- [x] Monaco Editor provides basic highlighting
- [ ] Extended language support: Rust, Python, Go, C++, etc.
- [ ] Custom color schemes (Catppuccin already set)
- [ ] Configurable font and size

### 3. Find/Replace
- [ ] Find in current file (Ctrl+F)
- [ ] Find and Replace (Ctrl+H)
- [ ] Find in all open files
- [ ] Regex support
- [ ] Match case option
- [ ] Whole word option
- [ ] Incremental search (search as you type)

### 4. Line Numbers
- [x] Monaco has line numbers
- [ ] Click to select entire line
- [ ] Current line highlight

### 5. Code Folding
- [ ] Collapse/expand code blocks
- [ ] Fold all / Unfold all
- [ ] Fold at level N

### 6. Bookmarks
- [ ] Toggle bookmark (Ctrl+F2)
- [ ] Next/Previous bookmark (F2/Shift+F2)
- [ ] Clear all bookmarks
- [ ] Bookmark margin

## Advanced Features (Priority 2)

### 7. Multi-View / Split View
- [ ] Clone document to another view
- [ ] Horizontal split
- [ ] Vertical split
- [ ] Synchronized scrolling

### 8. Document Map (Mini-Map)
- [ ] Overview of entire document
- [ ] Click to navigate
- [ ] Highlight current view area

### 9. Multi-Cursor Editing
- [ ] Ctrl+Click to place multiple cursors
- [ ] Alt+Drag for column selection
- [ ] Ctrl+D to select next occurrence
- [ ] Ctrl+Shift+L to split selection into lines

### 10. Column Mode (Block Selection)
- [ ] Alt+Drag for column selection
- [ ] Edit multiple lines simultaneously
- [ ] Paste in column mode

### 11. Macro Recording/Playback
- [ ] Start/Stop recording (Ctrl+Shift+R)
- [ ] Play macro (Ctrl+Shift+P)
- [ ] Save macro
- [ ] Multiple macro slots

### 12. Function List
- [ ] Panel showing all functions/methods
- [ ] Click to navigate
- [ ] Tree view of code structure
- [ ] Filter/Search in function list

### 13. Document Switcher
- [ ] Ctrl+Tab for MRU (Most Recently Used) switching
- [ ] List of open files
- [ ] Quick preview

## File Operations (Priority 1)

### 14. File Management
- [ ] Recent files list
- [ ] Save All
- [ ] Close All
- [ ] Close All Except Current
- [ ] Reload from disk
- [ ] Auto-save on focus lost

### 15. Session Management
- [ ] Save session (list of open files)
- [ ] Restore session on startup
- [ ] Backup on save

## Search Features (Priority 2)

### 16. Advanced Search
- [ ] Find in files (directory search)
- [ ] Mark search results
- [ ] Clear all marks
- [ ] Go to next/previous marked line
- [ ] Copy marked lines
- [ ] Remove marked lines

### 17. Go To
- [ ] Go to line (Ctrl+G)
- [ ] Go to offset
- [ ] Go to matching brace
- [ ] Go to previous/next function

## View Features (Priority 2)

### 18. View Modes
- [ ] Word wrap toggle
- [ ] Show all characters (whitespace, EOL, etc.)
- [ ] Zoom in/out (Ctrl++, Ctrl+-)
- [ ] Full-screen mode (F11)
- [ ] Post-It mode (distraction-free)

### 19. Panels
- [ ] Folder as Workspace (file explorer)
- [ ] Document list panel
- [ ] Clipboard history
- [ ] Character panel (special chars)

## Editing Features (Priority 2)

### 20. Advanced Editing
- [ ] Multi-line editing (Ctrl+Click)
- [ ] Smart indent
- [ ] Auto-indent on paste
- [ ] Trim trailing whitespace
- [ ] Convert tabs to spaces / spaces to tabs
- [ ] Change case (upper, lower, title)
- [ ] Line operations (sort, reverse, shuffle)
- [ ] Duplicate line
- [ ] Delete line
- [ ] Move line up/down

### 21. Insert Operations
- [ ] Insert date/time
- [ ] Insert special characters
- [ ] Insert from file

## Comparison Features (Priority 3)

### 22. File Comparison
- [ ] Compare two files
- [ ] Show differences inline
- [ ] Navigate to next/previous difference
- [ ] Synchronized scrolling
- [ ] Ignore whitespace option

## Implementation Priority

**Phase 1 (Core Editor):**
1. Multi-document tabs with close buttons
2. Find/Replace dialog with regex
3. Bookmarks system
4. Recent files
5. Go to line

**Phase 2 (Advanced Editing):**
6. Multi-cursor support
7. Column mode
8. Code folding
9. Document map

**Phase 3 (Productivity):**
10. Macro recording
11. Function list panel
12. File comparison
13. Folder workspace

## Technical Notes

- Monaco Editor supports many features natively
- Need to add Tauri commands for file operations
- State management for multiple open files
- UI components: Tabs, Find dialog, Panels
