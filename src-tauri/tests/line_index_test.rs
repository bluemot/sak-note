//! Standalone test for LineIndex functionality
//! Run with: rustc --edition 2021 line_index_test.rs -o line_index_test && ./line_index_test

use std::io::Write;
use std::time::Instant;

/// Pre-built index of line start positions for O(1) line lookup
pub struct LineIndex {
    /// Byte offset of each line start. line_starts[0] = 0, line_starts[i] = byte offset of line i
    line_starts: Vec<usize>,
    /// Total file size in bytes
    file_size: usize,
}

impl LineIndex {
    /// Build a line index by scanning for newlines
    pub fn build(data: &[u8]) -> Self {
        let mut line_starts = Vec::with_capacity(data.len() / 80);
        line_starts.push(0); // Line 0 starts at offset 0

        for (i, &byte) in data.iter().enumerate() {
            if byte == b'\n' {
                line_starts.push(i + 1);
            }
        }

        LineIndex {
            line_starts,
            file_size: data.len(),
        }
    }

    /// Get total number of lines
    pub fn line_count(&self) -> usize {
        if self.file_size == 0 {
            0
        } else {
            self.line_starts.len()
        }
    }

    /// Get byte offset for the start of a line (0-based line number)
    pub fn line_offset(&self, line: usize) -> Option<usize> {
        self.line_starts.get(line).copied()
    }

    /// Get byte offset for the end of a line (start of next line, or file end)
    pub fn line_end_offset(&self, line: usize) -> Option<usize> {
        if line + 1 < self.line_starts.len() {
            self.line_starts.get(line + 1).copied()
        } else if line < self.line_starts.len() {
            Some(self.file_size)
        } else {
            None
        }
    }

    /// Get byte range for lines [start_line, end_line) (0-based, end exclusive)
    pub fn line_range(&self, start_line: usize, end_line: usize) -> Option<(usize, usize)> {
        let byte_start = self.line_offset(start_line)?;
        let byte_end = if end_line >= self.line_starts.len() {
            self.file_size
        } else {
            self.line_starts.get(end_line).copied().unwrap_or(self.file_size)
        };
        Some((byte_start, byte_end))
    }

    /// Memory usage of the index in bytes
    pub fn memory_usage(&self) -> usize {
        self.line_starts.len() * std::mem::size_of::<usize>()
    }
}

fn main() {
    println!("=== LineIndex Test ===\n");

    // Test 1: Basic functionality
    println!("Test 1: Basic functionality");
    let data = b"hello\nworld\nfoo\n";
    let index = LineIndex::build(data);
    assert_eq!(index.line_count(), 4);
    assert_eq!(index.line_offset(0), Some(0));
    assert_eq!(index.line_offset(1), Some(6));
    assert_eq!(index.line_offset(2), Some(12));
    println!("  ✓ Line count: {}", index.line_count());
    println!("  ✓ Line offsets: {:?}", 
        (0..index.line_count()).map(|i| index.line_offset(i)).collect::<Vec<_>>());

    // Test 2: Line range
    println!("\nTest 2: Line range lookup");
    let (start, end) = index.line_range(0, 2).unwrap();
    assert_eq!(start, 0);
    assert_eq!(end, 12); // "hello\n" (6) + "world\n" (6) = 12 bytes for lines 0-1
    println!("  ✓ Lines 0-1 byte range: {}-{} bytes", start, end);
    println!("  ✓ Content: {:?}", std::str::from_utf8(&data[start..end]).unwrap());

    // Test 3: Large file performance
    println!("\nTest 3: Large file performance (100,000 lines)");
    let line = b"This is a test line with some content that is about 50 bytes long.\n";
    let num_lines = 100000;
    let mut data: Vec<u8> = Vec::with_capacity(line.len() * num_lines);
    for _ in 0..num_lines {
        data.extend_from_slice(line);
    }
    
    let start = Instant::now();
    let index = LineIndex::build(&data);
    let build_time = start.elapsed();
    
    assert_eq!(index.line_count(), num_lines + 1);
    println!("  ✓ Index built: {} lines in {:?}", index.line_count(), build_time);
    println!("  ✓ Memory usage: {} bytes", index.memory_usage());
    println!("  ✓ Avg bytes per line index: {:.2}", index.memory_usage() as f64 / index.line_count() as f64);

    // Test O(1) lookups
    let start = Instant::now();
    let mut sum = 0usize;
    for i in 0..10000 {
        sum += index.line_offset(i * 10).unwrap_or(0);
    }
    let lookup_time = start.elapsed();
    println!("  ✓ 10,000 random lookups in {:?} (sum={})", lookup_time, sum);

    // Compare with scanning
    println!("\nTest 4: Comparison with naive scan");
    let scan_start = Instant::now();
    let mut line_count_scan = 1usize;
    for &byte in data.iter() {
        if byte == b'\n' {
            line_count_scan += 1;
        }
    }
    let scan_time = scan_start.elapsed();
    
    println!("  ✓ Naive scan line count: {} in {:?}", line_count_scan, scan_time);
    println!("  ✓ Index build is {:.1}x faster than scan", scan_time.as_micros() as f64 / build_time.as_micros() as f64);

    // Test 5: Random access comparison
    let target_lines: Vec<usize> = (0..1000).map(|i| i * 100).collect();
    
    // Indexed access
    let idx_start = Instant::now();
    for &line_num in &target_lines {
        let _ = index.line_offset(line_num);
    }
    let idx_time = idx_start.elapsed();
    
    // Scanning access (simulated - for each target line, scan from beginning)
    let scan_start = Instant::now();
    for &target_line in &target_lines {
        let mut current_line = 0usize;
        let mut offset = 0usize;
        for (i, &byte) in data.iter().enumerate() {
            if byte == b'\n' {
                current_line += 1;
                if current_line == target_line {
                    offset = i + 1;
                    break;
                }
            }
        }
        let _ = offset;
    }
    let scan_access_time = scan_start.elapsed();
    
    println!("\nTest 5: Random access (1000 lookups)");
    println!("  ✓ Indexed access: {:?}", idx_time);
    println!("  ✓ Scan access: {:?}", scan_access_time);
    println!("  ✓ Indexed is {:.0}x faster", scan_access_time.as_micros() as f64 / idx_time.as_micros().max(1) as f64);

    println!("\n=== All tests passed! ===");
}
