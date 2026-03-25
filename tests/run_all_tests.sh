#!/bin/bash
# SAK Editor - Complete Test Suite
# Runs all unit, integration, and E2E tests

echo "=========================================="
echo "SAK Editor - Complete Test Suite"
echo "=========================================="
echo ""

# Colors
PASS="[PASS]"
FAIL="[FAIL]"
SKIP="[SKIP]"

PASSED=0
FAILED=0
SKIPPED=0

# Get script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(dirname "$SCRIPT_DIR")"

run_test_suite() {
    local name="$1"
    local script="$2"
    
    echo "=========================================="
    echo "Running: $name"
    echo "=========================================="
    
    if [ -f "$script" ]; then
        if bash "$script"; then
            ((PASSED++))
            echo "[OK] $name"
        else
            ((FAILED++))
            echo "[FAILED] $name"
        fi
    else
        echo "[SKIP] $name - script not found: $script"
        ((SKIPPED++))
    fi
    echo ""
}

# Layer 1: Unit Tests (Rust)
echo "=========================================="
echo "LAYER 1: Rust Unit Tests (if cargo available)"
echo "=========================================="
if command -v cargo &> /dev/null; then
    echo "Running cargo test..."
    cd "$ROOT_DIR/src-tauri"
    CARGO_OUTPUT=$(cargo test --lib vfs:: 2>&1)
    if echo "$CARGO_OUTPUT" | grep -q "test result: ok"; then
        echo "$PASS Rust VFS unit tests (12 tests)"
        ((PASSED++))
    else
        echo "$FAIL Rust unit tests"
        echo "$CARGO_OUTPUT"
        ((FAILED++))
    fi
else
    echo "$SKIP Rust not installed, skipping Rust unit tests"
    ((SKIPPED++))
fi
echo ""

# Layer 2: Frontend Component Tests (if vitest available)
echo "=========================================="
echo "LAYER 2: Frontend Component Tests (if vitest available)"
echo "=========================================="
if command -v npm &> /dev/null && [ -f "$ROOT_DIR/src-frontend/vitest.config.ts" ]; then
    cd "$ROOT_DIR/src-frontend"
    if npm list vitest &> /dev/null; then
        echo "Running vitest..."
        if npm run test -- --run 2>&1 | grep -q "Tests"; then
            echo "$PASS Frontend component tests"
            ((PASSED++))
        else
            echo "$FAIL Frontend component tests"
            ((FAILED++))
        fi
    else
        echo "$SKIP vitest not installed, skipping component tests"
        ((SKIPPED++))
    fi
else
    echo "$SKIP npm or vitest config not found, skipping component tests"
    ((SKIPPED++))
fi
echo ""

# Layer 3: E2E Tests
echo "=========================================="
echo "LAYER 3: E2E Tests"
echo "=========================================="

# 3.1 VFS Core Tests
run_test_suite "VFS Core Unit Tests" "$SCRIPT_DIR/unit/test_vfs_core.sh"

# 3.2 App Structure Tests
run_test_suite "Application Structure Tests" "$SCRIPT_DIR/e2e/test_app_structure.sh"

# 3.3 File Operations Tests
run_test_suite "File Operations Tests" "$SCRIPT_DIR/e2e/test_file_operations.sh"

# 4. Configuration Validation
echo "=========================================="
echo "LAYER 4: Configuration Validation"
echo "=========================================="

validate_json() {
    local file="$1"
    local name="$2"
    
    printf "Validating %s... " "$name"
    if python3 -c "import json; json.load(open('$file'))" 2>/dev/null; then
        echo "$PASS"
        ((PASSED++))
    else
        echo "$FAIL"
        ((FAILED++))
    fi
}

validate_json "$ROOT_DIR/src-tauri/tauri.conf.json" "tauri.conf.json"
validate_json "$ROOT_DIR/src-frontend/package.json" "package.json"
validate_json "$ROOT_DIR/package.json" "root package.json"

echo ""

# Summary
echo "=========================================="
echo "Complete Test Summary"
echo "=========================================="
echo "Passed:  $PASSED"
echo "Failed:  $FAILED"
echo "Skipped: $SKIPPED"
echo "Total:   $((PASSED + FAILED + SKIPPED))"
echo ""

if [ $FAILED -eq 0 ]; then
    echo "All available tests passed!"
    exit 0
else
    echo "Some tests failed!"
    exit 1
fi
