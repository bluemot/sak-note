#!/bin/bash
# SAK Editor - Complete Test Suite
# Runs all unit and integration tests

# set -e  # Allow individual test failures

echo "=========================================="
echo "SAK Editor - Complete Test Suite"
echo "=========================================="
echo ""

PASSED=0
FAILED=0

# Track test results
run_test_suite() {
    local name="$1"
    local script="$2"
    
    echo "Running: $name"
    echo "------------------------------------------"
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
        ((FAILED++))
    fi
    echo ""
}

# Get script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(dirname "$(dirname "$SCRIPT_DIR")")"

# 1. Run VFS Core Unit Tests
run_test_suite "VFS Core Tests" "$SCRIPT_DIR/../unit/test_vfs_core.sh"

# 2. Check Rust compilation (if cargo available)
echo "Running: Rust Compilation Check"
echo "------------------------------------------"
if command -v cargo &> /dev/null; then
    cd "$ROOT_DIR/src-tauri"
    if cargo check 2>&1 | grep -q "error"; then
        echo "[FAILED] Rust compilation errors found"
        ((FAILED++))
    else
        echo "[OK] Rust compilation check passed"
        ((PASSED++))
    fi
else
    echo "[SKIP] Rust not installed, skipping compilation check"
fi
echo ""

# 3. Check frontend build (if npm available)
echo "Running: Frontend Build Check"
echo "------------------------------------------"
if command -v npm &> /dev/null; then
    cd "$ROOT_DIR/src-frontend"
    if npm run build &> /tmp/npm_build.log; then
        echo "[OK] Frontend build successful"
        ((PASSED++))
    else
        echo "[FAILED] Frontend build failed"
        echo "See /tmp/npm_build.log for details"
        ((FAILED++))
    fi
else
    echo "[SKIP] npm not installed, skipping frontend build check"
fi
echo ""

# 4. Configuration validation
echo "Running: Configuration Validation"
echo "------------------------------------------"
if [ -f "$ROOT_DIR/src-tauri/tauri.conf.json" ]; then
    if python3 -c "import json; json.load(open('$ROOT_DIR/src-tauri/tauri.conf.json'))" 2>/dev/null; then
        echo "[OK] tauri.conf.json is valid JSON"
        ((PASSED++))
    else
        echo "[FAILED] tauri.conf.json is invalid JSON"
        ((FAILED++))
    fi
else
    echo "[FAILED] tauri.conf.json not found"
    ((FAILED++))
fi

if [ -f "$ROOT_DIR/src-frontend/package.json" ]; then
    if python3 -c "import json; json.load(open('$ROOT_DIR/src-frontend/package.json'))" 2>/dev/null; then
        echo "[OK] package.json is valid JSON"
        ((PASSED++))
    else
        echo "[FAILED] package.json is invalid JSON"
        ((FAILED++))
    fi
else
    echo "[FAILED] package.json not found"
    ((FAILED++))
fi
echo ""

# Summary
echo "=========================================="
echo "Test Summary"
echo "=========================================="
echo "Passed: $PASSED"
echo "Failed: $FAILED"
echo "Total:  $((PASSED + FAILED))"
echo ""

if [ $FAILED -eq 0 ]; then
    echo "All tests passed!"
    exit 0
else
    echo "Some tests failed!"
    exit 1
fi
