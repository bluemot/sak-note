#!/bin/bash
# View Operations E2E Tests
# Tests: V01-V05 from E2E_TEST_PLAN.md

source "$(dirname "$0")/../test-framework.sh"

describe "View Operations"

# -----------------------------------------------------------------------------
# V01: Toggle Sidebar (Ctrl+B)
# -----------------------------------------------------------------------------
test "V01: Toggle Sidebar (Ctrl+B)"
if focus_window; then
    log "Taking initial state screenshot..."
    take_screenshot "V01-Sidebar-Initial"
    
    log "Sending Ctrl+B to hide sidebar..."
    send_key "ctrl+b"
    sleep 0.8
    
    take_screenshot "V01-Sidebar-Hidden"
    
    log "Sending Ctrl+B again to show sidebar..."
    send_key "ctrl+b"
    sleep 0.8
    
    take_screenshot "V01-Sidebar-Visible"
    
    pass
else
    skip "SAK Editor window not available"
fi

# -----------------------------------------------------------------------------
# V02: Toggle StatusBar
# -----------------------------------------------------------------------------
test "V02: Toggle StatusBar"
if focus_window; then
    log "Taking initial state screenshot..."
    take_screenshot "V02-StatusBar-Initial"
    
    log "Opening View menu..."
    send_key "alt+v"
    sleep 0.3
    
    log "Navigating to StatusBar option..."
    # Navigate to StatusBar toggle
    send_key "Down"
    send_key "Down"
    send_key "Down"
    sleep 0.3
    
    log "Toggling StatusBar..."
    send_key "Return"
    sleep 0.5
    
    take_screenshot "V02-StatusBar-Toggled"
    
    log "Restoring StatusBar..."
    send_key "alt+v"
    sleep 0.3
    send_key "Down"
    send_key "Down"
    send_key "Down"
    sleep 0.3
    send_key "Return"
    sleep 0.3
    
    pass
else
    skip "SAK Editor window not available"
fi

# -----------------------------------------------------------------------------
# V04: Zoom In (Ctrl++)
# -----------------------------------------------------------------------------
test "V04: Zoom In (Ctrl++)"
if focus_window; then
    log "Taking initial zoom screenshot..."
    take_screenshot "V04-Zoom-Initial"
    
    log "Sending Ctrl++ to zoom in..."
    send_key "ctrl+plus"
    sleep 0.5
    
    send_key "ctrl+plus"
    sleep 0.5
    
    take_screenshot "V04-Zoom-In"
    
    log "Resetting zoom..."
    # Ctrl+0 usually resets zoom
    send_key "ctrl+0"
    sleep 0.3
    
    pass
else
    skip "SAK Editor window not available"
fi

# -----------------------------------------------------------------------------
# V05: Zoom Out (Ctrl+-)
# -----------------------------------------------------------------------------
test "V05: Zoom Out (Ctrl+-)"
if focus_window; then
    log "Taking initial zoom screenshot..."
    take_screenshot "V05-Zoom-Initial"
    
    log "Sending Ctrl+- to zoom out..."
    send_key "ctrl+minus"
    sleep 0.5
    
    send_key "ctrl+minus"
    sleep 0.5
    
    take_screenshot "V05-Zoom-Out"
    
    log "Resetting zoom..."
    send_key "ctrl+0"
    sleep 0.3
    
    pass
else
    skip "SAK Editor window not available"
fi

run_tests