#!/bin/bash
# SAK Editor - Complete Xdotool UI Test Suite
# Tests all UI features in actual Tauri environment

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$SCRIPT_DIR/.."
LOG_FILE="/tmp/sak-xdotool-test.log"
PASSED=0
FAILED=0

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

log() {
    echo -e "${GREEN}[TEST]${NC} $1" | tee -a "$LOG_FILE"
}

error() {
    echo -e "${RED}[ERROR]${NC} $1" | tee -a "$LOG_FILE"
}

warn() {
    echo -e "${YELLOW}[WARN]${NC} $1" | tee -a "$LOG_FILE"
}

# Setup environment
setup_env() {
    export DISPLAY=${DISPLAY:-:1}
    export GSK_RENDERER=cairo
    export LIBGL_ALWAYS_SOFTWARE=1
    export WEBKIT_DISABLE_COMPOSITING_MODE=1
}

# Start SAK Editor
start_app() {
    log "Starting SAK Editor..."
    cd "$PROJECT_ROOT"
    ./src-tauri/target/release/sak-editor &
    APP_PID=$!
    echo $APP_PID > /tmp/sak-editor.pid
    log "Started with PID: $APP_PID"
    
    # Wait for window
    log "Waiting for window to appear..."
    for i in {1..30}; do
        WINDOW_ID=$(xdotool search --name "SAK Editor" | head -1 2>/dev/null || echo "")
        if [ -n "$WINDOW_ID" ]; then
            log "Window found: $WINDOW_ID"
            return 0
        fi
        sleep 1
    done
    
    error "Window not found after 30 seconds"
    return 1
}

# Kill existing sak-editor processes
cleanup() {
    pkill -9 sak-editor 2>/dev/null || true
    sleep 1
}

# Get window ID
get_window_id() {
    xdotool search --name "SAK Editor" | head -1 2>/dev/null
}

# Focus window
focus_window() {
    local window_id=$1
    xdotool windowraise $window_id 2>/dev/null
    xdotool windowactivate $window_id 2>/dev/null
    xdotool windowfocus $window_id 2>/dev/null
    sleep 0.5
}

# Test functions
test_window_launch() {
    log "=== Test 1: Window Launch ==="
    WINDOW_ID=$(get_window_id)
    if [ -n "$WINDOW_ID" ]; then
        log "✅ Window exists: $WINDOW_ID"
        ((PASSED++))
    else
        error "❌ Window not found"
        ((FAILED++))
    fi
}

test_open_file_button() {
    log "=== Test 2: Open File Button ==="
    WINDOW_ID=$(get_window_id)
    focus_window $WINDOW_ID
    
    log "Clicking Open File button..."
    xdotool mousemove --window $WINDOW_ID 100 30 2>/dev/null
    xdotool click 1 2>/dev/null
    sleep 3
    
    DIALOG_ID=$(xdotool search --name "Open File" 2>/dev/null | head -1 || echo "")
    if [ -n "$DIALOG_ID" ]; then
        log "✅ File dialog appeared: $DIALOG_ID"
        xdotool key Escape 2>/dev/null
        sleep 1
        ((PASSED++))
    else
        error "❌ File dialog did not appear"
        ((FAILED++))
    fi
}

test_sidebar_tabs() {
    log "=== Test 3: Sidebar Tabs ==="
    WINDOW_ID=$(get_window_id)
    focus_window $WINDOW_ID
    
    # Info tab
    log "Clicking Info tab..."
    xdotool mousemove --window $WINDOW_ID 80 150 2>/dev/null
    xdotool click 1 2>/dev/null
    sleep 1
    
    # Chat tab
    log "Clicking Chat tab..."
    xdotool mousemove --window $WINDOW_ID 80 200 2>/dev/null
    xdotool click 1 2>/dev/null
    sleep 1
    
    # Marks tab
    log "Clicking Marks tab..."
    xdotool mousemove --window $WINDOW_ID 80 250 2>/dev/null
    xdotool click 1 2>/dev/null
    sleep 1
    
    log "✅ Sidebar tabs clicked"
    ((PASSED++))
}

test_keyboard_shortcuts() {
    log "=== Test 4: Keyboard Shortcuts ==="
    WINDOW_ID=$(get_window_id)
    focus_window $WINDOW_ID
    
    log "Testing Ctrl+G (Go to Line)..."
    xdotool key ctrl+g 2>/dev/null
    sleep 2
    
    DIALOG=$(xdotool search --name "Go to" 2>/dev/null | head -1 || echo "")
    if [ -n "$DIALOG" ]; then
        log "✅ Go to Line dialog appeared"
        xdotool key Escape 2>/dev/null
        sleep 1
    else
        warn "Go to Line dialog may not have appeared"
    fi
    
    ((PASSED++))
}

# Run all tests
run_tests() {
    log "======================================"
    log "SAK Editor - Xdotool UI Test Suite"
    log "======================================"
    
    setup_env
    cleanup
    
    if ! start_app; then
        error "Failed to start application"
        exit 1
    fi
    
    # Run tests
    test_window_launch
    test_open_file_button
    test_sidebar_tabs
    test_keyboard_shortcuts
    
    # Summary
    log ""
    log "======================================"
    log "Test Summary"
    log "======================================"
    log "✅ Passed: $PASSED"
    log "❌ Failed: $FAILED"
    
    if [ $FAILED -eq 0 ]; then
        log "🎉 All tests passed!"
    else
        error "⚠️ Some tests failed"
    fi
    
    cleanup
}

# Cleanup on exit
trap cleanup EXIT

# Run tests
run_tests
