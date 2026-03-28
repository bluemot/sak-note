#!/bin/bash
# SAK Editor E2E Test Framework
# Provides utility functions for UI automation testing

set -o pipefail

# Global variables
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
REPORT_DIR="/tmp/sak-e2e"
REPORT_FILE="$REPORT_DIR/test-report.json"
APP_NAME="SAK Editor"
WINDOW_ID=""
TEST_RESULTS=()
CURRENT_CATEGORY=""
CURRENT_TEST=""
TEST_START_TIME=0
CATEGORY_START_TIME=0
PASSED_COUNT=0
FAILED_COUNT=0
TOTAL_COUNT=0
SCREENSHOTS=()
FAILURES=()

# Ensure report directory exists
mkdir -p "$REPORT_DIR"

# Colors for terminal output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# ============================================================================
# Core Test Functions
# ============================================================================

describe() {
    CURRENT_CATEGORY="$1"
    CATEGORY_START_TIME=$(date +%s)
    echo -e "\n${BLUE}==== $1 ====${NC}"
}

test() {
    CURRENT_TEST="$1"
    TEST_START_TIME=$(date +%s)
    TOTAL_COUNT=$((TOTAL_COUNT + 1))
    echo -e "\n  ${YELLOW}▶ $1${NC}"
}

pass() {
    local duration=$(($(date +%s) - TEST_START_TIME))
    PASSED_COUNT=$((PASSED_COUNT + 1))
    echo -e "    ${GREEN}✓ PASSED${NC} (${duration}s)"
    TEST_RESULTS+=("{\"category\":\"$CURRENT_CATEGORY\",\"test\":\"$CURRENT_TEST\",\"status\":\"passed\",\"duration\":$duration}")
}

fail() {
    local message="$1"
    local duration=$(($(date +%s) - TEST_START_TIME))
    FAILED_COUNT=$((FAILED_COUNT + 1))
    echo -e "    ${RED}✗ FAILED: $message${NC} (${duration}s)"
    TEST_RESULTS+=("{\"category\":\"$CURRENT_CATEGORY\",\"test\":\"$CURRENT_TEST\",\"status\":\"failed\",\"error\":\"$message\",\"duration\":$duration}")
    FAILURES+=("{\"test\":\"$CURRENT_TEST\",\"error\":\"$message\"}")
    
    # Take screenshot on failure
    take_screenshot "FAIL-$(echo "$CURRENT_TEST" | tr ' ' '-')"
}

skip() {
    local message="$1"
    echo -e "    ${YELLOW}⊘ SKIPPED: $message${NC}"
    TEST_RESULTS+=("{\"category\":\"$CURRENT_CATEGORY\",\"test\":\"$CURRENT_TEST\",\"status\":\"skipped\",\"reason\":\"$message\"}")
}

# ============================================================================
# Assertion Functions
# ============================================================================

assert_equals() {
    local expected="$1"
    local actual="$2"
    local message="$3"
    
    if [ "$expected" = "$actual" ]; then
        return 0
    else
        fail "$message: expected '$expected', got '$actual'"
        return 1
    fi
}

assert_not_empty() {
    local value="$1"
    local message="$2"
    
    if [ -n "$value" ]; then
        return 0
    else
        fail "$message"
        return 1
    fi
}

assert_true() {
    local condition="$1"
    local message="$2"
    
    if [ "$condition" -eq 1 ] 2>/dev/null || [ "$condition" = "true" ]; then
        return 0
    else
        fail "$message"
        return 1
    fi
}

assert_exists() {
    local path="$1"
    local message="$2"
    
    if [ -e "$path" ]; then
        return 0
    else
        fail "$message: path '$path' does not exist"
        return 1
    fi
}

# ============================================================================
# Window Management Functions
# ============================================================================

get_window_id() {
    xdotool search --name "$APP_NAME" 2>/dev/null | head -1
}

focus_window() {
    local timeout=10
    local elapsed=0
    
    echo "    Focusing window..."
    
    while [ $elapsed -lt $timeout ]; do
        WINDOW_ID=$(get_window_id)
        if [ -n "$WINDOW_ID" ]; then
            xdotool windowactivate "$WINDOW_ID" 2>/dev/null
            xdotool windowfocus "$WINDOW_ID" 2>/dev/null
            sleep 0.5
            return 0
        fi
        sleep 1
        elapsed=$((elapsed + 1))
    done
    
    fail "Could not find or focus SAK Editor window"
    return 1
}

wait_for_window() {
    local name="$1"
    local timeout="${2:-5}"
    local elapsed=0
    
    while [ $elapsed -lt $timeout ]; do
        local wid=$(xdotool search --name "$name" 2>/dev/null | head -1)
        if [ -n "$wid" ]; then
            echo "$wid"
            return 0
        fi
        sleep 1
        elapsed=$((elapsed + 1))
    done
    
    return 1
}

wait_for_window_close() {
    local name="$1"
    local timeout="${2:-5}"
    local elapsed=0
    
    while [ $elapsed -lt $timeout ]; do
        local wid=$(xdotool search --name "$name" 2>/dev/null | head -1)
        if [ -z "$wid" ]; then
            return 0
        fi
        sleep 1
        elapsed=$((elapsed + 1))
    done
    
    return 1
}

# ============================================================================
# UI Interaction Functions
# ============================================================================

send_key() {
    local key="$1"
    xdotool key "$key" 2>/dev/null
}

send_keys() {
    local keys="$1"
    xdotool key "$keys" 2>/dev/null
}

type_text() {
    local text="$1"
    xdotool type --delay 10 "$text" 2>/dev/null
}

click_at() {
    local x="$1"
    local y="$2"
    xdotool mousemove --window "$WINDOW_ID" "$x" "$y" 2>/dev/null
    xdotool click 1 2>/dev/null
}

cleanup_dialogs() {
    # Close any open dialogs with Escape key
    send_key "Escape"
    sleep 0.5
    send_key "Escape"
    sleep 0.3
}

# ============================================================================
# Screenshot Functions
# ============================================================================

take_screenshot() {
    local name="$1"
    local filename="$REPORT_DIR/${name}-$(date +%s).png"
    
    if [ -n "$WINDOW_ID" ]; then
        import -window "$WINDOW_ID" "$filename" 2>/dev/null || \
        xwd -id "$WINDOW_ID" -out "$filename.xwd" 2>/dev/null && \
        convert "$filename.xwd" "$filename" 2>/dev/null && rm "$filename.xwd" 2>/dev/null
    else
        # Fallback to full screen
        import -window root "$filename" 2>/dev/null || \
        gnome-screenshot -f "$filename" 2>/dev/null || \
        import "$filename" 2>/dev/null
    fi
    
    if [ -f "$filename" ]; then
        SCREENSHOTS+=("$filename")
        echo "    Screenshot saved: $filename"
    fi
}

# ============================================================================
# Verification Functions
# ============================================================================

check_window_exists() {
    local name="$1"
    local wid=$(xdotool search --name "$name" 2>/dev/null | head -1)
    [ -n "$wid" ]
}

check_element_exists() {
    # This is a placeholder - in a real implementation,
    # you might use OCR or specific window properties
    # For now, we rely on dialog window detection
    return 0
}

# ============================================================================
# Report Generation
# ============================================================================

generate_report() {
    local end_time=$(date +%s)
    local start_time="${1:-$CATEGORY_START_TIME}"
    local duration=$((end_time - start_time))
    local timestamp=$(date -Iseconds)
    
    # Build JSON array of screenshots
    local screenshots_json="["
    local first=true
    for screenshot in "${SCREENSHOTS[@]}"; do
        if [ "$first" = true ]; then
            first=false
        else
            screenshots_json+=","
        fi
        screenshots_json+="\"$screenshot\""
    done
    screenshots_json+="]"
    
    # Build JSON array of failures
    local failures_json="["
    first=true
    for failure in "${FAILURES[@]}"; do
        if [ "$first" = true ]; then
            first=false
        else
            failures_json+=","
        fi
        failures_json+="$failure"
    done
    failures_json+="]"
    
    cat > "$REPORT_FILE" << EOF
{
  "timestamp": "$timestamp",
  "total": $TOTAL_COUNT,
  "passed": $PASSED_COUNT,
  "failed": $FAILED_COUNT,
  "duration_seconds": $duration,
  "categories": {
    "file": { "total": 0, "passed": 0, "failed": 0 },
    "edit": { "total": 0, "passed": 0, "failed": 0 },
    "view": { "total": 0, "passed": 0, "failed": 0 },
    "navigation": { "total": 0, "passed": 0, "failed": 0 }
  },
  "failures": $failures_json,
  "screenshots": $screenshots_json
}
EOF
    
    echo -e "\n${BLUE}==== Test Report ====${NC}"
    echo "Total: $TOTAL_COUNT"
    echo -e "${GREEN}Passed: $PASSED_COUNT${NC}"
    echo -e "${RED}Failed: $FAILED_COUNT${NC}"
    echo "Duration: ${duration}s"
    echo "Report: $REPORT_FILE"
}

print_summary() {
    echo -e "\n${BLUE}========================================${NC}"
    echo -e "${BLUE}       SAK Editor E2E Test Summary       ${NC}"
    echo -e "${BLUE}========================================${NC}"
    echo -e "Total Tests: $TOTAL_COUNT"
    echo -e "${GREEN}Passed: $PASSED_COUNT${NC}"
    echo -e "${RED}Failed: $FAILED_COUNT${NC}"
    
    if [ ${#FAILURES[@]} -gt 0 ]; then
        echo -e "\n${RED}Failed Tests:${NC}"
        for failure in "${FAILURES[@]}"; do
            echo "  - $(echo "$failure" | grep -o '"test":"[^"]*"' | cut -d'"' -f4)"
        done
    fi
}

# ============================================================================
# Test Runner
# ============================================================================

run_tests() {
    # This function is called at the end of each test file
    # It signals completion of that category
    echo ""
}

# ============================================================================
# Utility Functions
# ============================================================================

log() {
    echo "    $1"
}

wait_for() {
    local seconds="$1"
    sleep "$seconds"
}

# Check if required tools are available
check_prerequisites() {
    local missing=()
    
    if ! command -v xdotool &> /dev/null; then
        missing+=("xdotool")
    fi
    
    if ! command -v import &> /dev/null; then
        missing+=("imagemagick (import)")
    fi
    
    if [ ${#missing[@]} -gt 0 ]; then
        echo -e "${RED}Error: Missing required tools:${NC}"
        for tool in "${missing[@]}"; do
            echo "  - $tool"
        done
        echo "Install with: sudo apt-get install xdotool imagemagick"
        exit 1
    fi
    
    # Check if X11 is available
    if [ -z "$DISPLAY" ]; then
        echo -e "${RED}Error: No X11 display available. Tests require a graphical environment.${NC}"
        exit 1
    fi
}

# Initialize framework
check_prerequisites
