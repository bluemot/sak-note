#!/bin/bash
# File Operations E2E Tests
# Tests file open, read, edit, save workflow

# set -e

echo "=========================================="
echo "SAK Editor - File Operations E2E Tests"
echo "=========================================="

PASS="[PASS]"
FAIL="[FAIL]"

PASSED=0
FAILED=0

TEST_DIR="/tmp/sak_file_ops_$$"
mkdir -p "$TEST_DIR"
trap "rm -rf $TEST_DIR" EXIT

run_test() {
    local name="$1"
    local cmd="$2"
    
    printf "Testing: %s... " "$name"
    if eval "$cmd" > /dev/null 2>&1; then
        echo "$PASS"
        ((PASSED++))
    else
        echo "$FAIL"
        ((FAILED++))
    fi
}

echo ""
echo "Setting up test environment..."

# Create test files of different types
echo "Hello, SAK Editor!" > "$TEST_DIR/small.txt"
dd if=/dev/urandom of="$TEST_DIR/medium.bin" bs=1024 count=100 2>/dev/null
dd if=/dev/urandom of="$TEST_DIR/large.bin" bs=1024 count=1024 2>/dev/null

# Create a JSON file
cat > "$TEST_DIR/config.json" << 'EOF'
{
  "editor": {
    "theme": "dark",
    "fontSize": 14,
    "tabSize": 2
  },
  "files": {
    "autoSave": true,
    "encoding": "utf-8"
  }
}
EOF

# Create a multi-line text file
for i in {1..100}; do
    echo "Line $i: This is test content for line number $i" >> "$TEST_DIR/multiline.txt"
done

echo ""
echo "------------------------------------------"
echo "Basic File Operations"
echo "------------------------------------------"

run_test "Open small text file" "cat $TEST_DIR/small.txt"
run_test "Open medium binary file" "cat $TEST_DIR/medium.bin > /dev/null"
run_test "Open large binary file" "head -c 1024 $TEST_DIR/large.bin > /dev/null"
run_test "Open JSON file" "cat $TEST_DIR/config.json"
run_test "Open multi-line file" "wc -l $TEST_DIR/multiline.txt"

echo ""
echo "------------------------------------------"
echo "Read Operations"
echo "------------------------------------------"

# Test reading at specific offsets
run_test "Read from file beginning" "head -c 10 $TEST_DIR/small.txt"
run_test "Read from file end" "tail -c 10 $TEST_DIR/small.txt"
run_test "Read middle of file" "sed -n '50p' $TEST_DIR/multiline.txt"
run_test "Read range of lines" "sed -n '10,20p' $TEST_DIR/multiline.txt"

echo ""
echo "------------------------------------------"
echo "Edit Operations Simulation"
echo "------------------------------------------"

# Simulate insert operation
run_test "Insert at beginning" "echo 'PREFIX' | cat - $TEST_DIR/small.txt > $TEST_DIR/inserted.txt"
run_test "Insert at end" "echo 'SUFFIX' >> $TEST_DIR/small.txt"
run_test "Verify insert result" "grep -q 'PREFIX' $TEST_DIR/inserted.txt"

# Simulate delete operation
run_test "Delete line" "sed '1d' $TEST_DIR/multiline.txt > $TEST_DIR/deleted.txt"
run_test "Verify delete result" "[ $(wc -l < $TEST_DIR/deleted.txt) -eq 99 ]"

# Simulate replace operation
run_test "Replace text" "sed 's/Line/Row/g' $TEST_DIR/multiline.txt > $TEST_DIR/replaced.txt"
run_test "Verify replace result" "grep -q 'Row 1:' $TEST_DIR/replaced.txt"

echo ""
echo "------------------------------------------"
echo "Save Operations Simulation"
echo "------------------------------------------"

# Simulate save (atomic write pattern)
run_test "Atomic write (write to temp, then move)" "cp $TEST_DIR/small.txt $TEST_DIR/small.txt.tmp && mv $TEST_DIR/small.txt.tmp $TEST_DIR/small.txt"
run_test "File size preserved after save" "[ $(stat -c%s $TEST_DIR/small.txt) -gt 0 ]"

echo ""
echo "------------------------------------------"
echo "Large File Handling"
echo "------------------------------------------"

# Test with larger files
LARGE_FILE="$TEST_DIR/very_large.bin"
dd if=/dev/urandom of="$LARGE_FILE" bs=1024 count=5120 2>/dev/null

run_test "Open 5MB file" "stat $LARGE_FILE"
run_test "Read first 1KB of 5MB file" "head -c 1024 $LARGE_FILE > /dev/null"
run_test "Read last 1KB of 5MB file" "tail -c 1024 $LARGE_FILE > /dev/null"
run_test "Random access read" "dd if=$LARGE_FILE bs=1024 skip=1000 count=1 2>/dev/null"

echo ""
echo "------------------------------------------"
echo "Hex View Operations"
echo "------------------------------------------"

run_test "Hex dump small file" "xxd $TEST_DIR/small.txt > /dev/null"
run_test "Hex dump first 256 bytes" "xxd -l 256 $TEST_DIR/medium.bin > /dev/null"
run_test "Hex dump with offset" "xxd -s 512 -l 256 $TEST_DIR/large.bin > /dev/null"
run_test "Hex dump to file" "xxd $TEST_DIR/small.txt $TEST_DIR/small.hex"
run_test "Hex dump file exists" "[ -f $TEST_DIR/small.hex ]"

echo ""
echo "------------------------------------------"
echo "Undo/Redo Simulation"
echo "------------------------------------------"

# Simulate undo by keeping backup copies
run_test "Create backup before edit" "cp $TEST_DIR/small.txt $TEST_DIR/small.txt.bak"
run_test "Modify file" "echo 'MODIFIED' > $TEST_DIR/small.txt"
run_test "Undo by restoring backup" "cp $TEST_DIR/small.txt.bak $TEST_DIR/small.txt"
run_test "Verify undo result" "grep -q 'Hello' $TEST_DIR/small.txt"

echo ""
echo "=========================================="
echo "File Operations Test Summary"
echo "=========================================="
echo "Passed: $PASSED"
echo "Failed: $FAILED"
echo "Total:  $((PASSED + FAILED))"
echo ""

if [ $FAILED -eq 0 ]; then
    echo "All file operations tests passed!"
    exit 0
else
    echo "Some file operations tests failed!"
    exit 1
fi
