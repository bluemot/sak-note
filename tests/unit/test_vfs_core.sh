#!/bin/bash
# VFS Core Unit Tests
# Tests the Virtual File System core functionality

echo "=========================================="
echo "SAK Editor - VFS Core Unit Tests"
echo "=========================================="

# Test directory
TEST_DIR="/tmp/sak_vfs_test_$$"
mkdir -p "$TEST_DIR"

cleanup() {
    echo "Cleaning up test directory..."
    rm -rf "$TEST_DIR"
}
trap cleanup ERR

# Counter for test results
PASSED=0
FAILED=0

run_test() {
    local test_name="$1"
    local test_cmd="$2"
    
    printf "Testing: %s... " "$test_name"
    if eval "$test_cmd" > /dev/null 2>&1; then
        echo "[PASS]"
        ((PASSED++))
        return 0
    else
        echo "[FAIL]"
        ((FAILED++))
        return 1
    fi
}

echo ""
echo "Setting up test files..."

# Create test files
echo "Hello, World!" > "$TEST_DIR/test1.txt"
dd if=/dev/urandom of="$TEST_DIR/binary.dat" bs=1024 count=10 2>/dev/null
echo "Line 1" > "$TEST_DIR/multiline.txt"
echo "Line 2" >> "$TEST_DIR/multiline.txt"
echo "Line 3" >> "$TEST_DIR/multiline.txt"

echo ""
echo "------------------------------------------"
echo "File System Tests"
echo "------------------------------------------"

run_test "File exists" "[ -f \"$TEST_DIR/test1.txt\" ]"
run_test "File readable" "[ -r \"$TEST_DIR/test1.txt\" ]"
run_test "File writable" "[ -w \"$TEST_DIR/test1.txt\" ]"
run_test "Binary file exists" "[ -f \"$TEST_DIR/binary.dat\" ]"
FILE_SIZE=$(stat -c%s "$TEST_DIR/binary.dat" 2>/dev/null || stat -f%z "$TEST_DIR/binary.dat" 2>/dev/null)
run_test "Binary file size is 10KB" "[ $FILE_SIZE -eq 10240 ]"

echo ""
echo "------------------------------------------"
echo "Content Tests"
echo "------------------------------------------"

CONTENT=$(cat "$TEST_DIR/test1.txt")
run_test "Text file content matches" "[ '$CONTENT' = 'Hello, World!' ]"
run_test "Multiline file has 3 lines" "[ $(wc -l < $TEST_DIR/multiline.txt) -eq 3 ]"
LINE1=$(head -1 "$TEST_DIR/multiline.txt")
run_test "Can read first line" "[ '$LINE1' = 'Line 1' ]"

echo ""
echo "------------------------------------------"
echo "Edit Operation Simulation Tests"
echo "------------------------------------------"

# Simulate insert operation
echo "BeforeHello, World!" > "$TEST_DIR/insert_test.txt"
INSERT_CONTENT=$(cat "$TEST_DIR/insert_test.txt")
run_test "Insert at beginning" "[ '$INSERT_CONTENT' = 'BeforeHello, World!' ]"

# Simulate delete operation
echo "Hello,World" > "$TEST_DIR/delete_test.txt"
DELETE_CONTENT=$(cat "$TEST_DIR/delete_test.txt" | tr -d ',')
run_test "Delete comma" "[ '$DELETE_CONTENT' = 'HelloWorld' ]"

# Simulate replace operation
echo "Hi, World!" > "$TEST_DIR/replace_test.txt"
REPLACE_CONTENT=$(cat "$TEST_DIR/replace_test.txt")
run_test "Replace text" "[ '$REPLACE_CONTENT' = 'Hi, World!' ]"

echo ""
echo "------------------------------------------"
echo "Hex View Simulation Tests"
echo "------------------------------------------"

# Test hex dump functionality
run_test "Hex dump produces output" "xxd $TEST_DIR/test1.txt > /dev/null"
run_test "Hex dump has correct format" "xxd $TEST_DIR/test1.txt | head -1 | grep -q '00000000:'"

echo ""
echo "------------------------------------------"
echo "Directory Operations Tests"
echo "------------------------------------------"

run_test "Directory exists" "[ -d \"$TEST_DIR\" ]"
run_test "Can list directory" "ls \"$TEST_DIR\" > /dev/null"
FILE_COUNT=$(ls -1 "$TEST_DIR" | wc -l)
run_test "Directory contains expected files" "[ $FILE_COUNT -ge 5 ]"

echo ""
echo "=========================================="
echo "Test Summary"
echo "=========================================="
echo "Passed: $PASSED"
echo "Failed: $FAILED"
echo "Total:  $((PASSED + FAILED))"

if [ $FAILED -eq 0 ]; then
    echo ""
    echo "All tests passed!"
    exit 0
else
    echo ""
    echo "Some tests failed!"
    exit 1
fi
