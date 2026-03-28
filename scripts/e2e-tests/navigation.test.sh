#!/bin/bash
# Navigation E2E Tests
# Tests: N01-N02 from E2E_TEST_PLAN.md

source "$(dirname "$0")/../test-framework.sh"

describe "Navigation"

# -----------------------------------------------------------------------------
# N01: Next Bookmark (F2)
# -----------------------------------------------------------------------------
test "N01: Next Bookmark (F2)"
if focus_window; then
    log "Creating new file for bookmark test..."
    send_key "ctrl+n"
    sleep 0.5
    
    log "Adding some content..."
    type_text "Line 1 content
Line 2 content  
Line 3 content
Line 4 content
Line 5 content"
    sleep 0.5
    
    log "Sending F2 for Next Bookmark..."
    send_key "F2"
    sleep 0.5
    
    take_screenshot "N01-Next-Bookmark"
    
    log "Cleanup..."
    send_key "ctrl+w"
    sleep 0.3
    
    pass
else
    skip "SAK Editor window not available"
fi

# -----------------------------------------------------------------------------
# N02: Prev Bookmark (Shift+F2)
# -----------------------------------------------------------------------------
test "N02: Prev Bookmark (Shift+F2)"
if focus_window; then
    log "Creating new file for bookmark test..."
    send_key "ctrl+n"
    sleep 0.5
    
    log "Adding some content..."
    type_text "Line 1 content
Line 2 content
Line 3 content
Line 4 content
Line 5 content"
    sleep 0.5
    
    log "Sending Shift+F2 for Previous Bookmark..."
    send_key "shift+F2"
    sleep 0.5
    
    take_screenshot "N02-Prev-Bookmark"
    
    log "Cleanup..."
    send_key "ctrl+w"
    sleep 0.3
    
    pass
else
    skip "SAK Editor window not available"
fi

run_tests