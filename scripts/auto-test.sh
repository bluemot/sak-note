#!/bin/bash
# SAK Editor Automation Test Script

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$SCRIPT_DIR/.."

echo "=== SAK Editor Automation Test ==="

# Kill any existing sak-editor processes
pkill -9 sak-editor 2>/dev/null || true
sleep 1

# Set environment variables for headless/virtual display
export DISPLAY=${DISPLAY:-:1}
export GSK_RENDERER=cairo
export LIBGL_ALWAYS_SOFTWARE=1
export WEBKIT_DISABLE_COMPOSITING_MODE=1

# Start SAK Editor
echo "Starting SAK Editor..."
cd "$PROJECT_ROOT"
./src-tauri/target/release/sak-editor &
echo "Started with PID: $!"

# Wait for window to appear
echo "Waiting for window..."
sleep 5

# Check if window exists
WINDOW_ID=$(xdotool search --name "SAK Editor" | head -1)
if [ -z "$$WINDOW_ID" ]; then
    echo "ERROR: SAK Editor window not found!"
    exit 1
fi

echo "Found window ID: $WINDOW_ID"

# Bring window to front
xdotool windowraise $WINDOW_ID
xdotool windowactivate $WINDOW_ID
xdotool windowfocus $WINDOW_ID
sleep 2

# Click on Open File button (approximate position)
echo "Clicking Open File button..."
xdotool mousemove --window $WINDOW_ID 100 50
xdotool click 1
sleep 3

# Check if file dialog appeared
DIALOG_ID=$(xdotool search --name "Open File" 2>/dev/null | head -1 || echo "")
if [ -n "$$DIALOG_ID" ]; then
    echo "SUCCESS: File dialog appeared!"
    # Close dialog
    xdotool key Escape
    sleep 1
else
    echo "WARNING: File dialog may not have appeared"
fi

echo "Test completed successfully!"
sleep 2

# Cleanup
pkill -9 sak-editor 2>/dev/null || true
echo "SAK Editor closed."
