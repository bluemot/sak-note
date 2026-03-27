#!/bin/bash
#
# SAK Editor Log Viewer Script
#
# This script helps view and filter logs from SAK Editor.
# Logs are stored in ~/.config/sak-editor/logs/
#
# Usage:
#   ./scripts/view-logs.sh [options]
#
# Options:
#   -f, --follow          Follow log file in real-time (like tail -f)
#   -n, --lines N         Show last N lines (default: 100)
#   -p, --plugin          Show only plugin-related logs
#   -r, --rust            Show only Rust backend logs
#   -e, --error           Show only error logs
#   -d, --debug           Show debug level and above
#   -i, --info            Show info level and above
#   -w, --warn            Show warning level and above
#   --clear               Clear all log files
#   --list                List all log files
#   -h, --help            Show this help message
#

LOG_DIR="$HOME/.config/sak-editor/logs"
LINES=100
FILTER=""
FOLLOW=false
LOG_LEVEL=""

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -f|--follow)
            FOLLOW=true
            shift
            ;;
        -n|--lines)
            LINES="$2"
            shift 2
            ;;
        -p|--plugin)
            FILTER="plugin:"
            shift
            ;;
        -r|--rust)
            FILTER="[plugin:"
            shift
            ;;
        -e|--error)
            LOG_LEVEL="ERROR"
            shift
            ;;
        -d|--debug)
            LOG_LEVEL="DEBUG"
            shift
            ;;
        -i|--info)
            LOG_LEVEL="INFO"
            shift
            ;;
        -w|--warn)
            LOG_LEVEL="WARN"
            shift
            ;;
        --clear)
            echo "Clearing log files..."
            if [ -d "$LOG_DIR" ]; then
                rm -f "$LOG_DIR"/*.log
                echo "Log files cleared."
            else
                echo "Log directory not found: $LOG_DIR"
            fi
            exit 0
            ;;
        --list)
            if [ -d "$LOG_DIR" ]; then
                echo "Log files in $LOG_DIR:"
                ls -lh "$LOG_DIR"
            else
                echo "Log directory not found: $LOG_DIR"
            fi
            exit 0
            ;;
        -h|--help)
            head -n 30 "$0" | tail -n 28
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            echo "Use -h or --help for usage information"
            exit 1
            ;;
    esac
done

# Create log directory if it doesn't exist
mkdir -p "$LOG_DIR"

# Find the most recent log file
LOG_FILE=$(ls -t "$LOG_DIR"/*.log 2>/dev/null | head -n 1)

if [ -z "$LOG_FILE" ]; then
    echo "No log files found in $LOG_DIR"
    echo ""
    echo "Log files will be created when the application runs."
    echo ""
    echo "To enable file logging in the backend, run:"
    echo "  export RUST_LOG=debug"
    echo "Or set the appropriate log level."
    exit 1
fi

echo "Viewing logs from: $LOG_FILE"
echo "=========================================="
echo ""

# Build the command
if [ -n "$FILTER" ]; then
    # Filter by content
    if [ "$FOLLOW" = true ]; then
        tail -f -n "$LINES" "$LOG_FILE" | grep --color=always "$FILTER"
    else
        grep --color=always "$FILTER" "$LOG_FILE" | tail -n "$LINES"
    fi
elif [ -n "$LOG_LEVEL" ]; then
    # Filter by log level
    case "$LOG_LEVEL" in
        ERROR)
            if [ "$FOLLOW" = true ]; then
                tail -f -n "$LINES" "$LOG_FILE" | grep -E "(ERROR|error)"
            else
                grep -E "(ERROR|error)" "$LOG_FILE" | tail -n "$LINES"
            fi
            ;;
        WARN)
            if [ "$FOLLOW" = true ]; then
                tail -f -n "$LINES" "$LOG_FILE" | grep -E "(WARN|WARNING|ERROR|error)"
            else
                grep -E "(WARN|WARNING|ERROR|error)" "$LOG_FILE" | tail -n "$LINES"
            fi
            ;;
        INFO)
            if [ "$FOLLOW" = true ]; then
                tail -f -n "$LINES" "$LOG_FILE" | grep -v -E "^(DEBUG|TRACE)"
            else
                tail -n "$LINES" "$LOG_FILE" | grep -v -E "^(DEBUG|TRACE)"
            fi
            ;;
        DEBUG)
            if [ "$FOLLOW" = true ]; then
                tail -f -n "$LINES" "$LOG_FILE" | grep -v "^TRACE"
            else
                tail -n "$LINES" "$LOG_FILE" | grep -v "^TRACE"
            fi
            ;;
    esac
else
    # No filter - show all
    if [ "$FOLLOW" = true ]; then
        tail -f -n "$LINES" "$LOG_FILE"
    else
        tail -n "$LINES" "$LOG_FILE"
    fi
fi
