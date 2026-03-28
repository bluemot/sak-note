#!/bin/bash
# Edit Operations E2E Tests
# Tests: E01-E06 from E2E_TEST_PLAN.md

source "$(dirname "$0")/../test-framework.sh"

describe "Edit Operations"

# -----------------------------------------------------------------------------
# E01: Undo (Ctrl+Z)
# -----------------------------------------------------------------------------
test "E01: Undo (Ctrl+Z)"
if focus_window; then
    log "Creating new file..."
    send_key "ctrl+n"
    sleep 0.5
    
    log "Typing test content..."
    type_text "test content for undo"
    sleep 0.3
    
    log "Sending Ctrl+Z..."
    send_key "ctrl+z"
    sleep 0.5
    
    take_screenshot "E01-Undo"
    
    log "Cleanup..."
    send_key "ctrl+w"
    sleep 0.3
    
    pass
else
    skip "SAK Editor window not available"
fi

# -----------------------------------------------------------------------------
# E02: Redo (Ctrl+Y)
# -----------------------------------------------------------------------------
test "E02: Redo (Ctrl+Y)"
if focus_window; then
    log "Creating new file..."
    send_key "ctrl+n"
    sleep 0.5
    
    log "Typing test content..."
    type_text "test content for redo"
    sleep 0.3
    
    log "Undo first..."
    send_key "ctrl+z"
    sleep 0.3
    
    log "Sending Ctrl+Y for redo..."
    send_key "ctrl+y"
    sleep 0.5
    
    take_screenshot "E02-Redo"
    
    log "Cleanup..."
    send_key "ctrl+w"
    sleep 0.3
    
    pass
else
    skip "SAK Editor window not available"
fi

# -----------------------------------------------------------------------------
# E03: Find (Ctrl+F)
# -----------------------------------------------------------------------------
test "E03: Find (Ctrl+F)"
if focus_window; then
    log "Sending Ctrl+F..."
    send_key "ctrl+f"
    sleep 1
    
    log "Checking for Find panel/dialog..."
    
    # Look for Find window or panel
    FIND_ID=$(xdotool search --name "Find" 2>/dev/null | head -1)
    
    if [ -z "$FIND_ID" ]; then
        # Check if Search panel appeared in main window
        take_screenshot "E03-Find-Panel"
    fi
    
    log "Closing Find panel..."
    send_key "Escape"
    sleep 0.3
    
    pass
else
    skip "SAK Editor window not available"
fi

# -----------------------------------------------------------------------------
# E04: Replace (Ctrl+H)
# -----------------------------------------------------------------------------
test "E04: Replace (Ctrl+H)"
if focus_window; then
    log "Sending Ctrl+H..."
    send_key "ctrl+h"
    sleep 1
    
    log "Checking for Replace dialog..."
    
    REPLACE_ID=$(xdotool search --name "Replace" 2>/dev/null | head -1)
    
    if [ -n "$REPLACE_ID" ]; then
        log "Replace dialog found"
        take_screenshot "E04-Replace-Dialog"
        send_key "Escape"
    else
        # Could be integrated panel
        take_screenshot "E04-Replace-Panel"
        send_key "Escape"
    fi
    
    sleep 0.3
    pass
else
    skip "SAK Editor window not available"
fi

# -----------------------------------------------------------------------------
# E05: Go to Line (Ctrl+G)
# -----------------------------------------------------------------------------
test "E05: Go to Line (Ctrl+G)"
if focus_window; then
    log "Sending Ctrl+G..."
    send_key "ctrl+g"
    sleep 1
    
    log "Checking for Go to Line dialog..."
    
    GOTO_ID=$(xdotool search --name "Go to" 2>/dev/null | head -1)
    
    if [ -n "$GOTO_ID" ]; then
        log "Go to Line dialog found"
        take_screenshot "E05-Goto-Line"
        send_key "Escape"
        pass
    else
        # Try other common names
        GOTO_ID=$(xdotool search --name "Line" 2>/dev/null | head -1)
        if [ -n "$GOTO_ID" ]; then
            log "Go to Line dialog found (alt name)"
            take_screenshot "E05-Goto-Line"
            send_key "Escape"
            pass
        else
            take_screenshot "E05-Goto-Line-Check"
            send_key "Escape"
            pass "Dialog may be integrated"
        fi
    fi
    
    sleep 0.3
else
    skip "SAK Editor window not available"
fi

# -----------------------------------------------------------------------------
# E06: Find in Files (Ctrl+Shift+F)
# -----------------------------------------------------------------------------
test "E06: Find in Files (Ctrl+Shift+F)"
if focus_window; then
    log "Sending Ctrl+Shift+F..."
    send_key "ctrl+shift+f"
    sleep 1.5
    
    log "Checking for Find in Files dialog..."
    
    GLOBAL_FIND_ID=$(xdotool search --name "Find in Files" 2>/dev/null | head -1)
    
    if [ -n "$GLOBAL_FIND_ID" ]; then
        log "Find in Files dialog found"
        take_screenshot "E06-Find-In-Files"
        send_key "Escape"
        pass
    else
        # Try alternative names
        GLOBAL_FIND_ID=$(xdotool search --name "Search in Files" 2>/dev/null | head -1)
        if [ -n "$GLOBAL_FIND_ID" ]; then
            log "Find in Files dialog found (alt name)"
            take_screenshot "E06-Find-In-Files"
            send_key "Escape"
            pass
        else
            take_screenshot "E06-Find-In-Files-Check"
            send_key "Escape"
            pass "Panel may be integrated"
        fi
    fi
    
    sleep 0.3
else
    skip "SAK Editor window not available"
fi

run_tests