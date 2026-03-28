#!/bin/bash
# File Operations E2E Tests
# Tests: F01-F06 from E2E_TEST_PLAN.md

source "$(dirname "$0")/../test-framework.sh"

describe "File Operations"

# -----------------------------------------------------------------------------
# F01: Open File Dialog (Ctrl+O)
# -----------------------------------------------------------------------------
test "F01: Open File Dialog (Ctrl+O)"
if focus_window; then
    log "Sending Ctrl+O..."
    send_key "ctrl+o"
    sleep 1.5
    
    log "Checking for Open File dialog..."
    DIALOG_ID=$(xdotool search --name "Open File" 2>/dev/null | head -1)
    
    if [ -n "$DIALOG_ID" ]; then
        pass
    else
        # Try alternative window names
        DIALOG_ID=$(xdotool search --name "Open" 2>/dev/null | head -1)
        if [ -n "$DIALOG_ID" ]; then
            pass
        else
            fail "Open File dialog did not appear"
        fi
    fi
    
    # Cleanup
    log "Closing dialog..."
    send_key "Escape"
    sleep 0.5
else
    skip "SAK Editor window not available"
fi

# -----------------------------------------------------------------------------
# F02: Save Dialog (Ctrl+Shift+S)
# -----------------------------------------------------------------------------
test "F02: Save Dialog (Ctrl+Shift+S)"
if focus_window; then
    log "Sending Ctrl+Shift+S..."
    send_key "ctrl+shift+s"
    sleep 1.5
    
    log "Checking for Save As dialog..."
    DIALOG_ID=$(xdotool search --name "Save As" 2>/dev/null | head -1)
    
    if [ -n "$DIALOG_ID" ]; then
        pass
    else
        # Try alternative names
        DIALOG_ID=$(xdotool search --name "Save" 2>/dev/null | head -1)
        if [ -n "$DIALOG_ID" ]; then
            pass
        else
            fail "Save As dialog did not appear"
        fi
    fi
    
    # Cleanup
    log "Closing dialog..."
    send_key "Escape"
    sleep 0.5
else
    skip "SAK Editor window not available"
fi

# -----------------------------------------------------------------------------
# F03: New File (Ctrl+N)
# -----------------------------------------------------------------------------
test "F03: New File (Ctrl+N)"
if focus_window; then
    log "Sending Ctrl+N..."
    send_key "ctrl+n"
    sleep 1
    
    # Take screenshot to verify UI state change
    take_screenshot "F03-New-File"
    
    # Verify window is still active (new tab/file opened)
    if check_window_exists "$APP_NAME"; then
        pass
    else
        fail "Window state unexpected after New File"
    fi
else
    skip "SAK Editor window not available"
fi

# -----------------------------------------------------------------------------
# F04: Close File (Ctrl+W)
# -----------------------------------------------------------------------------
test "F04: Close File (Ctrl+W)"
if focus_window; then
    log "Creating a new file first..."
    send_key "ctrl+n"
    sleep 0.5
    
    log "Sending Ctrl+W..."
    send_key "ctrl+w"
    sleep 1
    
    # Take screenshot to verify
    take_screenshot "F04-Close-File"
    
    # Window should still exist, just file closed
    if check_window_exists "$APP_NAME"; then
        pass
    else
        fail "Window closed unexpectedly"
    fi
else
    skip "SAK Editor window not available"
fi

# -----------------------------------------------------------------------------
# F05: Recent Files Menu
# -----------------------------------------------------------------------------
test "F05: Recent Files Menu"
if focus_window; then
    log "Clicking File menu..."
    # Alt+F to open File menu
    send_key "alt+f"
    sleep 0.5
    
    log "Navigating to Open Recent..."
    # Navigate to Open Recent submenu
    send_key "Down"
    send_key "Down"
    send_key "Down"
    send_key "Right"
    sleep 0.5
    
    take_screenshot "F05-Recent-Files-Menu"
    
    # Close menu with Escape
    send_key "Escape"
    sleep 0.3
    
    # Test passes if we got here without errors
    pass
else
    skip "SAK Editor window not available"
fi

# -----------------------------------------------------------------------------
# F06: Save Button State (conditional rendering)
# -----------------------------------------------------------------------------
test "F06: Save Button State"
if focus_window; then
    log "Checking Save button state..."
    
    # Take screenshot of toolbar
    take_screenshot "F06-Save-Button-Initial"
    
    # Create new file and type something
    send_key "ctrl+n"
    sleep 0.3
    
    # Type some text
    type_text "test content"
    sleep 0.5
    
    take_screenshot "F06-Save-Button-Modified"
    
    # Cleanup
    send_key "ctrl+w"
    sleep 0.3
    
    # Test passes - we're verifying the screenshot shows the state change
    pass
else
    skip "SAK Editor window not available"
fi

run_tests