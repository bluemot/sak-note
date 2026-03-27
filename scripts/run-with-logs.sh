#!/bin/bash
# Run SAK Editor with comprehensive logging

# Create log directory
LOG_DIR="$HOME/.config/sak-editor/logs"
mkdir -p "$LOG_DIR"
mkdir -p "$LOG_DIR/plugin"

# Set log level (debug, info, warn, error)
export SAK_LOG_LEVEL=${SAK_LOG_LEVEL:-info}
export RUST_LOG=${RUST_LOG:-sak_editor=info}

# Log file paths
BACKEND_LOG="$LOG_DIR/sak-editor.log"
ERROR_LOG="$LOG_DIR/sak-editor-error.log"
FRONTEND_LOG="$LOG_DIR/frontend.log"

# Clear old logs if they're too large (keep last run)
if [ -f "$BACKEND_LOG" ] && [ $(stat -c%s "$BACKEND_LOG") -gt 10485760 ]; then
    mv "$BACKEND_LOG" "$BACKEND_LOG.old"
    echo "[$(date)] Log rotated" > "$BACKEND_LOG"
fi

echo "======================================="
echo "SAK Editor - Starting with logging"
echo "Log directory: $LOG_DIR"
echo "Log level: $SAK_LOG_LEVEL"
echo "======================================="
echo ""
echo "View logs with: tail -f $BACKEND_LOG"
echo ""

# Find the executable
if [ -f "./src-tauri/target/release/sak-editor" ]; then
    EXEC="./src-tauri/target/release/sak-editor"
elif [ -f "./src-tauri/target/debug/sak-editor" ]; then
    EXEC="./src-tauri/target/debug/sak-editor"
else
    echo "Error: sak-editor executable not found!"
    echo "Please build with: npm run build"
    exit 1
fi

echo "Using executable: $EXEC"
echo ""

# Run with logging
cd "$(dirname "$0")/.."
echo "[$(date)] Starting SAK Editor" >> "$BACKEND_LOG"
echo "[$(date)] Log level: $SAK_LOG_LEVEL" >> "$BACKEND_LOG"

# Capture both stdout and stderr
"$EXEC" 2>&1 | tee -a "$BACKEND_LOG" | tee -a "$ERROR_LOG"

# Capture exit code
EXIT_CODE=${PIPESTATUS[0]}

echo ""
echo "======================================="
if [ $EXIT_CODE -eq 0 ]; then
    echo "SAK Editor exited normally"
else
    echo "SAK Editor exited with code: $EXIT_CODE"
    echo "Check error log: $ERROR_LOG"
fi
echo "======================================="

exit $EXIT_CODE
