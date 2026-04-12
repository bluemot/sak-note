// Performance test for large file operations
// rustc --edition 2021 perf_test.rs && ./perf_test

use std::fs::File;
use std::io::{Read, Write};
use std::time::Instant;

fn main() {
    let path = "/home/ubuntu/.openclaw/workspace/sak-editor/[Com COM42] (2025-04-24_172717).log";
    
    println!("=== Large File Performance Test ===");
    println!("File: {}", path);
    
    // Test 1: Open file + mmap
    let start = Instant::now();
    let file = File::open(path).unwrap();
    let metadata = file.metadata().unwrap();
    let file_size = metadata.len();
    println!("\nTest 1: File open + stat: {:?} (size: {} bytes)", start.elapsed(), file_size);
    
    // Test 2: Count lines by scanning
    let start = Instant::now();
    let mut file = File::open(path).unwrap();
    let mut buffer = vec![0u8; 1024 * 1024]; // 1MB buffer
    let mut line_count = 1usize;
    loop {
        let bytes_read = file.read(&mut buffer).unwrap();
        if bytes_read == 0 { break; }
        for &byte in &buffer[..bytes_read] {
            if byte == b'\n' { line_count += 1; }
        }
    }
    let scan_time = start.elapsed();
    println!("\nTest 2: Count lines (scan): {:?} ({} lines)", scan_time, line_count);
    
    // Test 3: Build line index (same as scanning but storing offsets)
    let start = Instant::now();
    let mut file = File::open(path).unwrap();
    let mut buffer = vec![0u8; 1024 * 1024];
    let mut line_starts: Vec<usize> = Vec::with_capacity(line_count);
    line_starts.push(0);
    let mut total_offset = 0usize;
    loop {
        let bytes_read = file.read(&mut buffer).unwrap();
        if bytes_read == 0 { break; }
        for (i, &byte) in buffer[..bytes_read].iter().enumerate() {
            if byte == b'\n' {
                line_starts.push(total_offset + i + 1);
            }
        }
        total_offset += bytes_read;
    }
    let index_build_time = start.elapsed();
    println!("\nTest 3: Build line index: {:?}", index_build_time);
    println!("  Lines: {}", line_starts.len());
    println!("  Memory: {} bytes ({:.1} MB)", line_starts.len() * 8, (line_starts.len() * 8) as f64 / 1024.0 / 1024.0);
    
    // Test 4: O(1) lookups
    let start = Instant::now();
    for _ in 0..10000 {
        let _ = line_starts[3000000]; // Mid-file lookup
    }
    let lookup_time = start.elapsed();
    println!("\nTest 4: 10000 O(1) lookups: {:?}", lookup_time);
    
    // Test 5: Read lines 3000000-3000500
    let start = Instant::now();
    let byte_start = line_starts[3000000];
    let byte_end = line_starts[3000500];
    let mut file = File::open(path).unwrap();
    use std::io::Seek;
    use std::io::SeekFrom;
    file.seek(SeekFrom::Start(byte_start as u64)).unwrap();
    let mut buf = vec![0u8; byte_end - byte_start];
    file.read_exact(&mut buf).unwrap();
    let text = String::from_utf8_lossy(&buf);
    let read_time = start.elapsed();
    println!("\nTest 5: Read lines 3000000-3000500 via index: {:?}", read_time);
    println!("  Bytes: {}-{} ({} bytes)", byte_start, byte_end, byte_end - byte_start);
    println!("  First 100 chars: {}...", &text[..100.min(text.len())]);
    
    // Test 6: Simulate EditableFileManager::build_line_index (chunked scan via get_range)
    // This is closer to how the actual backend works
    let start = Instant::now();
    let mut newline_offsets: Vec<usize> = Vec::with_capacity(line_count);
    let mut offset = 0usize;
    let chunk_size = 65536; // SEARCH_BUFFER_SIZE
    // Simulate: we can't use mmap here, so use file reads
    let mut file = File::open(path).unwrap();
    loop {
        let mut chunk = vec![0u8; chunk_size];
        let bytes_read = file.read(&mut chunk).unwrap();
        if bytes_read == 0 { break; }
        for (i, &byte) in chunk[..bytes_read].iter().enumerate() {
            if byte == b'\n' {
                newline_offsets.push(offset + i);
            }
        }
        offset += bytes_read;
    }
    let chunked_scan_time = start.elapsed();
    println!("\nTest 6: Chunked scan (simulating build_line_index): {:?}", chunked_scan_time);
    println!("  Newlines found: {}", newline_offsets.len());
    
    // Summary
    println!("\n=== Summary ===");
    println!("File size: {} bytes ({:.0} MB)", file_size, file_size as f64 / 1024.0 / 1024.0);
    println!("Total lines: {}", line_count);
    println!("Line count scan: {:?}", scan_time);
    println!("Index build: {:?}", index_build_time);
    println!("Chunked scan: {:?}", chunked_scan_time);
    println!("O(1) lookup (10K): {:?}", lookup_time);
    println!("Seek+read 500 lines: {:?}", read_time);
    println!("Index memory: {:.1} MB", (line_starts.len() * 8) as f64 / 1024.0 / 1024.0);
}