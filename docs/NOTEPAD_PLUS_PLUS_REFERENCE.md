# Notepad++ Feature Reference (from source code analysis)

## Menu Structure (from menuCmdID.h)

### File Menu (IDM_FILE)
- NEW, OPEN, CLOSE, CLOSEALL
- SAVE, SAVEALL, SAVEAS
- CLOSEALL_BUT_CURRENT, CLOSEALL_TOLEFT, CLOSEALL_TORIGHT
- PRINT, PRINTNOW
- EXIT
- LOADSESSION, SAVESESSION
- RELOAD, DELETE, RENAME
- OPEN_FOLDER, OPEN_CMD
- RESTORELASTCLOSEDFILE
- OPENFOLDERASWORKSPACE
- CLOSEALL_UNCHANGED

### Edit Menu (IDM_EDIT)
- CUT, COPY, PASTE, DELETE
- UNDO, REDO
- SELECTALL
- INS_TAB, RMV_TAB (indent/unindent)
- DUP_LINE (duplicate line)
- TRANSPOSE_LINE
- SPLIT_LINES, JOIN_LINES
- LINE_UP, LINE_DOWN (move lines)
- UPPERCASE, LOWERCASE
- BLOCK_COMMENT, STREAM_COMMENT
- TRIMTRAILING, TRIMLINEHEAD, TRIM_BOTH
- COLUMNMODE (Alt+drag selection)
- SORTLINES_LEXICOGRAPHIC_ASCENDING/DESCENDING
- SORTLINES_INTEGER_ASCENDING/DESCENDING

### Search Menu
- FIND, REPLACE, FINDINFILES
- FIND_NEXT, FIND_PREV
- GO_TO_LINE (Ctrl+G)
- BOOKMARKS (Toggle, Next, Prev, Clear)
- JUMP_TO_MATCHING_BRACE

## Dialog Classes

### FindReplaceDlg
Features:
- Find Normal / Extended / Regex modes
- Match whole word
- Match case
- Wrap around
- Direction (Up/Down)
- In selection
- Dot matches newline (regex)
- Purge search results
- Mark lines
- Find in: Current doc / All open docs / Files in dir / Selection

### GoToLineDlg
- Line number input
- Offset input

### ColumnEditorDlg
- Text to insert
- Numbering options

## Search Options (FindOption struct)
```cpp
struct FindOption {
    bool _isWholeWord = true;
    bool _isMatchCase = true;
    bool _isWrapAround = true;
    bool _whichDirection = DIR_DOWN;  // DIR_DOWN or DIR_UP
    SearchType _searchType = FindNormal;  // FindNormal, FindExtended, FindRegex
    bool _isInSelection = false;
    bool _dotMatchesNewline = false;
    std::wstring _str2Search;
    std::wstring _str4Replace;
};
```

## Keyboard Shortcuts Reference

### Essential
- Ctrl+N: New file
- Ctrl+O: Open file
- Ctrl+S: Save
- Ctrl+Shift+S: Save As
- Ctrl+W: Close
- Ctrl+Z: Undo
- Ctrl+Y: Redo
- Ctrl+X: Cut
- Ctrl+C: Copy
- Ctrl+V: Paste
- Ctrl+A: Select All

### Search
- Ctrl+F: Find
- Ctrl+H: Replace
- Ctrl+Shift+F: Find in Files
- Ctrl+G: Go to Line
- F3: Find Next
- Shift+F3: Find Previous
- Ctrl+F2: Toggle Bookmark
- F2: Next Bookmark
- Shift+F2: Previous Bookmark

### Edit
- Ctrl+D: Duplicate line
- Ctrl+Shift+Up: Move line up
- Ctrl+Shift+Down: Move line down
- Ctrl+L: Delete line
- Ctrl+T: Switch current/previous line
- Ctrl+U: Lowercase
- Ctrl+Shift+U: Uppercase
- Ctrl+Q: Block comment
- Ctrl+Shift+Q: Block uncomment
- Tab: Indent
- Shift+Tab: Unindent

### View
- Ctrl+Tab: Next document
- Ctrl+Shift+Tab: Previous document
- Ctrl+W: Close document
- Ctrl+Numpad+: Zoom in
- Ctrl+Numpad-: Zoom out
- Ctrl+0: Restore default zoom
- F11: Fullscreen
- F12: Post-it mode

## Features Not Yet in SAK Editor

### High Priority
1. **Go to Line** (Ctrl+G) - Simple dialog, jump to line number
2. **Recent Files** - Track and display recent files
3. **Line Operations** - Duplicate, delete, move up/down, transpose
4. **Text Case** - Uppercase, lowercase, title case
5. **Indent/Unindent** - Tab/Shift+Tab handling
6. **Comment/Uncomment** - Block comment with shortcuts
7. **Trim Whitespace** - Trim trailing, trim leading, trim all
8. **Join/Split Lines** - Merge lines or split at cursor

### Medium Priority
9. **Sort Lines** - Lexicographic, numeric, ascending/descending
10. **Multi-cursor** - Ctrl+Click for multiple cursors
11. **Column Mode** - Alt+drag for block selection
12. **Document Map** - Minimap overview
13. **Function List** - List of functions in file
14. **Folder as Workspace** - File tree panel
15. **Clone to Other View** - Split view

### Lower Priority
16. **Macro Recording** - Record and playback actions
17. **Find in Files** - Search across directory
18. **Mark Search Results** - Highlight all matches
19. **Clipboard History** - Multiple clipboard entries
20. **Character Panel** - Insert special characters

## Implementation Notes for SAK Editor

### Monaco Editor Built-in Support
- ✅ Find/Replace (Ctrl+F, Ctrl+H)
- ✅ Go to Line (Ctrl+G) - partial
- ✅ Multi-cursor (Ctrl+Click)
- ✅ Column selection (Alt+Shift+Drag)
- ✅ Code folding
- ✅ Minimap (Document Map)

### Need Custom Implementation
- Bookmarks (F2 navigation)
- Recent files list
- Line operations (duplicate, move, delete)
- Text case conversion
- Sort lines
- Comment/uncomment with language detection
- Trim whitespace
- Join/split lines
- Clipboard history
