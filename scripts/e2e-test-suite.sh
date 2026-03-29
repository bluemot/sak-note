#!/bin/bash
# SAK Editor E2E Test Suite
# Usage:
#   ./scripts/e2e-test-suite.sh              # Run all tests
#   ./scripts/e2e-test-suite.sh --category=file   # Run only file tests
#   ./scripts/e2e-test-suite.sh --category=edit   # Run only edit tests
#   ./scripts/e2e-test-suite.sh --category=view     # Run only view tests
#   ./scripts/e2e-test-suite.sh --category=nav    # Run only navigation tests

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
TEST_DIR="$SCRIPT_DIR/e2e-tests"
REPORT_DIR="/tmp/sak-e2e"
REPORT_FILE="$REPORT_DIR/test-report.json"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Default: run all categories
CATEGORY="all"

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --category=*)
            CATEGORY="${1#*=}"
            shift
            ;;
        --help|-h)
            echo "SAK Editor E2E Test Suite"
            echo ""
            echo "Usage:"
            echo "  $0                    # Run all tests"
            echo "  $0 --category=file    # Run file operations tests"
            echo "  $0 --category=edit    # Run edit operations tests"
            echo "  $0 --category=view    # Run view operations tests"
            echo "  $0 --category=nav     # Run navigation tests"
            echo ""
            echo "Categories:"
            echo "  file - File Operations (F01-F06)"
            echo "  edit - Edit Operations (E01-E06)"
            echo "  view - View Operations (V01-V05)"
            echo "  nav  - Navigation (N01-N02)"
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            echo "Use --help for usage information"
            exit 1
            ;;
    esac
done

# Create report directory
mkdir -p "$REPORT_DIR"

# Test results storage
declare -A CATEGORY_TOTALS
declare -A CATEGORY_PASSED
declare -A CATEGORY_FAILED
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0
START_TIME=$(date +%s)
SCREENSHOTS=()
FAILURES=()

# Function to run a test file
run_test_file() {
    local file="$1"
    local cat_name="$2"
    
    echo -e "\n${BLUE}========================================${NC}"
    echo -e "${BLUE}Running: $cat_name${NC}"
    echo -e "${BLUE}========================================${NC}"
    
    if [ -f "$file" ]; then
        chmod +x "$file"
        # Source the test file in a subshell to capture results
        (
            cd "$PROJECT_DIR"
            source "$file"
        )
        
        # Extract results from the test run (test files update global vars via framework)
        return 0
    else
        echo -e "${RED}Test file not found: $file${NC}"
        return 1
    fi
}

# Print header
echo -e "${BLUE}"
echo "╔══════════════════════════════════════════╗"
echo "║     SAK Editor E2E Test Suite            ║"
echo "╚══════════════════════════════════════════╝"
echo -e "${NC}"
echo "Report directory: $REPORT_DIR"
echo ""

# Check prerequisites
echo "Checking prerequisites..."
if ! command -v xdotool &> /dev/null; then
    echo -e "${RED}Error: xdotool not found. Install with: sudo apt-get install xdotool${NC}"
    exit 1
fi

if ! command -v import &> /dev/null; then
    echo -e "${YELLOW}Warning: ImageMagick not found. Screenshots may not work.${NC}"
fi

if [ -z "$DISPLAY" ]; then
    echo -e "${RED}Error: No X11 display available. Tests require a graphical environment.${NC}"
    exit 1
fi

echo -e "${GREEN}Prerequisites OK${NC}"
echo ""

# Check if SAK Editor is already running, start if not
echo "Checking SAK Editor..."
WINDOW_ID=$(xdotool search --name "SAK Editor" 2>/dev/null | head -1 || echo "")
if [ -z "$WINDOW_ID" ]; then
    echo "Starting SAK Editor..."
    export DISPLAY=${DISPLAY:-:1}
    export GSK_RENDERER=cairo
    export LIBGL_ALWAYS_SOFTWARE=1
    export WEBKIT_DISABLE_COMPOSITING_MODE=1
    cd "$PROJECT_DIR"
    ./src-tauri/target/release/sak-editor &
    APP_PID=$!
    echo "Started with PID: $APP_PID"
    
    # Wait for window
    echo "Waiting for window..."
    for i in {1..30}; do
        WINDOW_ID=$(xdotool search --name "SAK Editor" 2>/dev/null | head -1 || echo "")
        if [ -n "$WINDOW_ID" ]; then
            echo "Window ready: $WINDOW_ID"
            break
        fi
        sleep 1
    done
    
    if [ -z "$WINDOW_ID" ]; then
        echo -e "${RED}Failed to start SAK Editor${NC}"
        exit 1
    fi
    sleep 2
else
    echo "SAK Editor already running: $WINDOW_ID"
fi
echo ""

# Initialize test framework
source "$SCRIPT_DIR/test-framework.sh"

# Run tests based on category
if [ "$CATEGORY" = "all" ] || [ "$CATEGORY" = "file" ]; then
    run_test_file "$TEST_DIR/file-operations.test.sh" "File Operations"
fi

if [ "$CATEGORY" = "all" ] || [ "$CATEGORY" = "edit" ]; then
    run_test_file "$TEST_DIR/edit-operations.test.sh" "Edit Operations"
fi

if [ "$CATEGORY" = "all" ] || [ "$CATEGORY" = "view" ]; then
    run_test_file "$TEST_DIR/view-operations.test.sh" "View Operations"
fi

if [ "$CATEGORY" = "all" ] || [ "$CATEGORY" = "nav" ] || [ "$CATEGORY" = "navigation" ]; then
    run_test_file "$TEST_DIR/navigation.test.sh" "Navigation"
fi

# Calculate duration
END_TIME=$(date +%s)
DURATION=$((END_TIME - START_TIME))

# Generate JSON report
echo -e "\n${BLUE}Generating test report...${NC}"

TIMESTAMP=$(date -Iseconds)

# Create JSON report
cat > "$REPORT_FILE" << EOF
{
  "timestamp": "$TIMESTAMP",
  "total": $TOTAL_COUNT,
  "passed": $PASSED_COUNT,
  "failed": $FAILED_COUNT,
  "duration_seconds": $DURATION,
  "categories": {
    "file": { "total": 6, "passed": 0, "failed": 0 },
    "edit": { "total": 6, "passed": 0, "failed": 0 },
    "view": { "total": 4, "passed": 0, "failed": 0 },
    "navigation": { "total": 2, "passed": 0, "failed": 0 }
  },
  "failures": [],
  "screenshots": []
}
EOF

echo "Report saved to: $REPORT_FILE"

# Print summary
echo -e "\n${BLUE}========================================${NC}"
echo -e "${BLUE}         Test Summary${NC}"
echo -e "${BLUE}========================================${NC}"
echo -e "Total Tests: $TOTAL_COUNT"
echo -e "${GREEN}Passed: $PASSED_COUNT${NC}"
echo -e "${RED}Failed: $FAILED_COUNT${NC}"
echo "Duration: ${DURATION}s"

if [ $FAILED_COUNT -eq 0 ]; then
    echo -e "\n${GREEN}✓ All tests passed!${NC}"
    exit 0
else
    echo -e "\n${RED}✗ Some tests failed${NC}"
    exit 1
fi
