use std::fs::{File, OpenOptions};
use std::io::{self, Write, BufWriter};
use std::path::{Path, PathBuf};
use memmap2::Mmap;
use std::collections::BTreeMap;

pub const CHUNK_SIZE: usize = 64 * 1024; // 64KB chunks for optimal memory usage
pub const SEARCH_BUFFER_SIZE: usize = 256 * 1024; // 256KB search buffer

/// Represents a chunk of file content
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Chunk {
    pub id: usize,
    pub offset: usize,
    pub length: usize,
    pub data: Vec<u8>,
}

/// Chunk manager for large files using memory mapping (Read-only optimized)
pub struct ChunkManager {
    #[allow(dead_code)]
    file_path: String,
    file_size: u64,
    chunks: Vec<ChunkInfo>,
    #[allow(dead_code)]
    mmap: Option<Mmap>,
}

#[derive(Debug, Clone)]
struct ChunkInfo {
    #[allow(dead_code)]
    offset: usize,
    #[allow(dead_code)]
    length: usize,
}

#[allow(dead_code)]
impl ChunkManager {
    pub fn new<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        log::debug!("[ChunkManager::new] Opening file for memory mapping...");
        let file = File::open(&path)?;
        let metadata = file.metadata()?;
        let file_size = metadata.len();
        log::debug!("[ChunkManager::new] File opened: size={} bytes", file_size);

        // Memory map the file
        let mmap = unsafe { Mmap::map(&file)? };
        log::debug!("[ChunkManager::new] File memory mapped successfully");

        // Calculate chunks
        let num_chunks = ((file_size + CHUNK_SIZE as u64 - 1) / CHUNK_SIZE as u64) as usize;
        let mut chunks = Vec::with_capacity(num_chunks);
        log::debug!("[ChunkManager::new] Calculating {} chunks with size {} bytes", num_chunks, CHUNK_SIZE);

        for i in 0..num_chunks {
            let offset = i * CHUNK_SIZE;
            let length = if i == num_chunks - 1 {
                (file_size as usize) - offset
            } else {
                CHUNK_SIZE
            };
            chunks.push(ChunkInfo { offset, length });
        }
        log::info!("[ChunkManager::new] ChunkManager created: path={:?}, size={}, chunks={}", 
            path.as_ref(), file_size, num_chunks);

        Ok(ChunkManager {
            file_path: path.as_ref().to_string_lossy().to_string(),
            file_size,
            chunks,
            mmap: Some(mmap),
        })
    }

    pub fn file_size(&self) -> u64 { self.file_size }
    pub fn chunk_count(&self) -> usize { self.chunks.len() }
    pub fn file_path(&self) -> &str { &self.file_path }

    /// Count lines in the file by counting newlines in the mmap
    pub fn line_count(&self) -> usize {
        if let Some(ref mmap) = self.mmap {
            let count = 1 + mmap.iter().filter(|&&b| b == b'\n').count();
            count
        } else {
            0
        }
    }

    /// Get lines by line number range (0-based, inclusive start, exclusive end)
    pub fn get_lines(&self, start_line: usize, end_line: usize) -> Option<String> {
        if let Some(ref mmap) = self.mmap {
            let mut line_starts: Vec<usize> = vec![0];
            for (i, &byte) in mmap.iter().enumerate() {
                if byte == b'\n' {
                    line_starts.push(i + 1);
                    if line_starts.len() > end_line + 1 {
                        break;
                    }
                }
            }
            if line_starts.len() <= end_line + 1 {
                line_starts.push(mmap.len());
            }

            let start = start_line.min(line_starts.len() - 1);
            let end = end_line.min(line_starts.len() - 1);
            let byte_start = line_starts[start];
            let byte_end = line_starts[end + 1].min(mmap.len());
            let length = byte_end.saturating_sub(byte_start);

            Some(String::from_utf8_lossy(&mmap[byte_start..byte_start + length]).to_string())
        } else {
            None
        }
    }

    pub fn get_chunk(&self, chunk_id: usize) -> Option<Chunk> {
        log::debug!("[ChunkManager::get_chunk] Requesting chunk_id={}", chunk_id);
        if chunk_id >= self.chunks.len() { 
            log::warn!("[ChunkManager::get_chunk] Chunk {} out of bounds (total chunks: {})", 
                chunk_id, self.chunks.len());
            return None; 
        }
        let info = &self.chunks[chunk_id];
        if let Some(ref mmap) = self.mmap {
            let data = mmap[info.offset..info.offset + info.length].to_vec();
            log::debug!("[ChunkManager::get_chunk] Chunk {} retrieved: offset={}, length={}", 
                chunk_id, info.offset, info.length);
            Some(Chunk { id: chunk_id, offset: info.offset, length: info.length, data })
        } else { 
            log::error!("[ChunkManager::get_chunk] No memory map available");
            None 
        }
    }

    pub fn get_text_range(&self, start: usize, end: usize) -> Option<String> {
        log::debug!("[ChunkManager::get_text_range] start={}, end={}", start, end);
        if let Some(ref mmap) = self.mmap {
            let end = end.min(self.file_size as usize);
            let bytes = &mmap[start..end];
            let text = String::from_utf8_lossy(bytes).to_string();
            log::debug!("[ChunkManager::get_text_range] Retrieved {} bytes as text", text.len());
            Some(text)
        } else { 
            log::error!("[ChunkManager::get_text_range] No memory map available");
            None 
        }
    }

    pub fn get_bytes(&self, start: usize, length: usize) -> Option<Vec<u8>> {
        log::debug!("[ChunkManager::get_bytes] start={}, length={}", start, length);
        if let Some(ref mmap) = self.mmap {
            let end = (start + length).min(self.file_size as usize);
            let data = mmap[start..end].to_vec();
            log::debug!("[ChunkManager::get_bytes] Retrieved {} bytes", data.len());
            Some(data)
        } else { 
            log::error!("[ChunkManager::get_bytes] No memory map available");
            None 
        }
    }

    pub fn get_byte(&self, position: usize) -> Option<u8> {
        log::debug!("[ChunkManager::get_byte] position={}", position);
        if let Some(ref mmap) = self.mmap {
            let byte = mmap.get(position).copied();
            log::debug!("[ChunkManager::get_byte] Byte at position {}: {:?}", position, byte);
            byte
        } else { 
            log::error!("[ChunkManager::get_byte] No memory map available");
            None 
        }
    }

    pub fn get_hex_range(&self, start: usize, length: usize) -> Vec<(usize, Vec<u8>)> {
        log::debug!("[ChunkManager::get_hex_range] start={}, length={}", start, length);
        let mut result = Vec::new();
        if let Some(ref mmap) = self.mmap {
            let end = (start + length).min(self.file_size as usize);
            let mut current_offset = start;
            let mut current_row = Vec::new();
            for i in start..end {
                current_row.push(mmap[i]);
                if current_row.len() == 16 || i == end - 1 {
                    result.push((current_offset, current_row.clone()));
                    current_offset += current_row.len();
                    current_row.clear();
                }
            }
        }
        log::debug!("[ChunkManager::get_hex_range] Retrieved {} hex rows", result.len());
        result
    }
}

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

    /// Build from newline positions (for chunked scanning)
    pub fn build_from_newline_offsets(offsets: Vec<usize>, file_size: usize) -> Self {
        let mut line_starts = Vec::with_capacity(offsets.len() + 1);
        line_starts.push(0);
        for offset in offsets {
            line_starts.push(offset + 1); // Line starts after the newline
        }
        LineIndex {
            line_starts,
            file_size,
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
    /// Returns None if line is out of bounds
    pub fn line_offset(&self, line: usize) -> Option<usize> {
        self.line_starts.get(line).copied()
    }

    /// Get byte offset for the end of a line (start of next line, or file end)
    /// For the last line, returns file_size
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
    /// Returns (byte_start, byte_end)
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

/// Edit operation types
#[derive(Debug, Clone)]
pub enum EditOp {
    Insert { offset: usize, data: Vec<u8> },
    Delete { offset: usize, length: usize },
    Replace { offset: usize, length: usize, data: Vec<u8> },
}

/// Editable file manager with undo/redo support
pub struct EditableFileManager {
    file_path: PathBuf,
    file_size: u64,
    mmap: Option<Mmap>,
    // Edit journal for undo/redo
    edit_history: Vec<EditOp>,
    history_position: usize,
    // Unsaved changes flag
    has_changes: bool,
    // Modified regions (for display)
    modified_regions: BTreeMap<usize, usize>, // offset -> length
    // Pre-built line index for O(1) line lookups
    line_index: Option<LineIndex>,
}

impl EditableFileManager {
    pub fn new<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        log::info!("[EditableFileManager::new] Opening file for editing: {:?}", path.as_ref());
        let file = File::open(&path)?;
        let metadata = file.metadata()?;
        let file_size = metadata.len();
        log::info!("[EditableFileManager::new] File opened: size={} bytes", file_size);
        
        let mmap = unsafe { Mmap::map(&file)? };
        log::debug!("[EditableFileManager::new] File memory mapped successfully");

        log::info!("[EditableFileManager::new] EditableFileManager created: path={:?}, size={}", 
            path.as_ref(), file_size);

        Ok(EditableFileManager {
            file_path: path.as_ref().to_path_buf(),
            file_size,
            mmap: Some(mmap),
            edit_history: Vec::new(),
            history_position: 0,
            has_changes: false,
            modified_regions: BTreeMap::new(),
            line_index: None,
        })
    }

    #[allow(dead_code)]
    pub fn file_size(&self) -> u64 { self.file_size }
    pub fn has_changes(&self) -> bool { self.has_changes }
    pub fn can_undo(&self) -> bool { self.history_position > 0 }
    pub fn can_redo(&self) -> bool { self.history_position < self.edit_history.len() }

    /// Get current effective file size (considering edits)
    pub fn effective_size(&self) -> u64 {
        let mut size = self.file_size as i64;
        for op in &self.edit_history[..self.history_position] {
            match op {
                EditOp::Insert { data, .. } => size += data.len() as i64,
                EditOp::Delete { length, .. } => size -= *length as i64,
                EditOp::Replace { length, data, .. } => {
                    size -= *length as i64;
                    size += data.len() as i64;
                }
            }
        }
        let result = size.max(0) as u64;
        log::trace!("[EditableFileManager::effective_size] Original: {}, Effective: {}", 
            self.file_size, result);
        result
    }

    /// Apply edit operation
    pub fn apply_edit(&mut self, op: EditOp) {
        log::debug!("[EditableFileManager::apply_edit] Applying edit: {:?}", op);
        // Remove any redo history
        if self.history_position < self.edit_history.len() {
            self.edit_history.truncate(self.history_position);
        }

        // Add to history
        self.edit_history.push(op.clone());
        self.history_position += 1;
        self.has_changes = true;
        // Invalidate line index since file content changed
        self.line_index = None;

        // Track modified region
        match &op {
            EditOp::Insert { offset, data } => {
                self.modified_regions.insert(*offset, data.len());
            }
            EditOp::Delete { offset, length } => {
                self.modified_regions.insert(*offset, *length);
            }
            EditOp::Replace { offset, data, .. } => {
                self.modified_regions.insert(*offset, data.len());
            }
        }
        log::debug!("[EditableFileManager::apply_edit] Edit applied. History position: {}/{}, has_changes: {}", 
            self.history_position, self.edit_history.len(), self.has_changes);
    }

    #[allow(dead_code)]
    /// Get byte at logical position (considering edits)
    pub fn get_byte_at(&self, logical_offset: usize) -> Option<u8> {
        let physical = self.logical_to_physical(logical_offset);
        if let Some(ref mmap) = self.mmap {
            mmap.get(physical).copied()
        } else { None }
    }

    /// Get range of bytes (considering edits)
    pub fn get_range(&self, start: usize, length: usize) -> Vec<u8> {
        log::debug!("[EditableFileManager::get_range] start={}, length={}", start, length);

        // Fast path: no edits applied, read directly from mmap
        if self.history_position == 0 {
            if let Some(ref mmap) = self.mmap {
                let end = (start + length).min(mmap.len());
                if start >= mmap.len() {
                    return Vec::new();
                }
                return mmap[start..end].to_vec();
            }
        }

        // Slow path: apply edits to get the logical view
        let mut result = Vec::with_capacity(length);
        let end = start + length;

        // Build edit map: logical offset -> (physical offset or inserted data)
        let mut logical_cursor: usize = 0;
        let mut physical_cursor: usize = 0;

        // Collect all edits up to current history position, sorted by offset
        let mut edits: Vec<&EditOp> = self.edit_history[..self.history_position].iter().collect();
        edits.sort_by_key(|op| match op {
            EditOp::Insert { offset, .. } | EditOp::Delete { offset, .. } | EditOp::Replace { offset, .. } => *offset,
        });

        // Build logical to physical mapping
        let mut edit_index = 0;
        while logical_cursor < end && physical_cursor < self.file_size as usize {
            // Check if there's an edit at this position
            let edit_at = edits.get(edit_index).map(|op| match op {
                EditOp::Insert { offset, .. } | EditOp::Delete { offset, .. } | EditOp::Replace { offset, .. } => *offset,
            });

            if let Some(offset) = edit_at {
                if offset == physical_cursor {
                    // Apply the edit
                    match &edits[edit_index] {
                        EditOp::Insert { data, .. } => {
                            // Insert data affects logical view
                            for i in 0..data.len() {
                                let logical_pos = logical_cursor + i;
                                if logical_pos >= start && logical_pos < end {
                                    result.push(data[i]);
                                }
                            }
                            logical_cursor += data.len();
                            // Physical cursor doesn't move for inserts
                        }
                        EditOp::Delete { length, .. } => {
                            // Skip deleted bytes in physical view
                            physical_cursor += *length;
                        }
                        EditOp::Replace { length, data, .. } => {
                            // Replace: skip physical bytes, use new data
                            for i in 0..data.len() {
                                let logical_pos = logical_cursor + i;
                                if logical_pos >= start && logical_pos < end {
                                    result.push(data[i]);
                                }
                            }
                            logical_cursor += data.len();
                            physical_cursor += *length;
                        }
                    }
                    edit_index += 1;
                    continue;
                }
            }

            // No edit at this position, copy from mmap
            if logical_cursor >= start && logical_cursor < end {
                if let Some(ref mmap) = self.mmap {
                    result.push(mmap[physical_cursor]);
                }
            }
            logical_cursor += 1;
            physical_cursor += 1;
        }

        // Handle any remaining inserts at the end
        while edit_index < edits.len() {
            if let EditOp::Insert { data, .. } = &edits[edit_index] {
                for i in 0..data.len() {
                    let logical_pos = logical_cursor + i;
                    if logical_pos >= start && logical_pos < end {
                        result.push(data[i]);
                    }
                }
                logical_cursor += data.len();
            }
            edit_index += 1;
        }

        log::debug!("[EditableFileManager::get_range] Returning {} bytes", result.len());
        result
    }

    /// Get text range (considering edits)
    pub fn get_text(&self, start: usize, length: usize) -> String {
        log::debug!("[EditableFileManager::get_text] start={}, length={}", start, length);

        // Fast path: no edits, read directly from mmap
        if self.history_position == 0 {
            if let Some(ref mmap) = self.mmap {
                let end = (start + length).min(mmap.len());
                if start >= mmap.len() {
                    return String::new();
                }
                return String::from_utf8_lossy(&mmap[start..end]).to_string();
            }
        }

        let bytes = self.get_range(start, length);
        String::from_utf8_lossy(&bytes).to_string()
    }

    /// Count the number of lines in the file (considering edits)
    /// Build the line index from current file content for O(1) line lookups
    /// Scan the file for newline positions (read-only, can run under read lock)
    /// Returns the built LineIndex but does NOT store it
    pub fn scan_line_index(&self) -> Option<LineIndex> {
        let effective_size = self.effective_size() as usize;
        if effective_size == 0 {
            return None;
        }

        // Fast path: read directly from mmap when no edits
        if self.history_position == 0 {
            if let Some(ref mmap) = self.mmap {
                let start = std::time::Instant::now();
                let index = LineIndex::build(&mmap[..effective_size]);
                log::info!("[EditableFileManager::scan_line_index] Index built from mmap in {:?}: {} lines",
                    start.elapsed(), index.line_count());
                return Some(index);
            }
        }

        // Fallback: use get_range for files with edits
        log::info!("[EditableFileManager::scan_line_index] Using get_range fallback (has {} edits)", self.history_position);
        let mut newline_offsets = Vec::new();
        let mut offset = 0usize;
        while offset < effective_size {
            let chunk_size = SEARCH_BUFFER_SIZE.min(effective_size - offset);
            let chunk = self.get_range(offset, chunk_size);
            for (i, &byte) in chunk.iter().enumerate() {
                if byte == b'\n' {
                    newline_offsets.push(offset + i);
                }
            }
            offset += chunk_size;
        }

        let index = LineIndex::build_from_newline_offsets(newline_offsets, effective_size);
        log::info!("[EditableFileManager::scan_line_index] Index built: {} lines", index.line_count());
        Some(index)
    }

    /// Store a pre-built line index (write operation, needs write lock)
    pub fn set_line_index(&mut self, index: LineIndex) {
        log::info!("[EditableFileManager::set_line_index] Storing index: {} lines, {} bytes",
            index.line_count(), index.memory_usage());
        self.line_index = Some(index);
    }

    pub fn build_line_index(&mut self) {
        if let Some(index) = self.scan_line_index() {
            self.line_index = Some(index);
        }
    }

    /// Get lines using pre-built index (O(1) lookup instead of O(n) scan)
    pub fn get_lines_indexed(&self, start_line: usize, end_line: usize) -> Option<String> {
        if let Some(ref index) = self.line_index {
            let (byte_start, byte_end) = index.line_range(start_line, end_line)?;
            let length = byte_end.saturating_sub(byte_start);
            if length == 0 || byte_start >= index.file_size {
                return Some(String::new());
            }
            let text = self.get_text(byte_start, length);
            Some(text)
        } else {
            None
        }
    }

    /// Get total line count from pre-built index
    pub fn line_count_indexed(&self) -> Option<usize> {
        self.line_index.as_ref().map(|idx| idx.line_count())
    }

    /// Get byte offset for a specific line using pre-built index
    pub fn line_offset_indexed(&self, line: usize) -> Option<usize> {
        self.line_index.as_ref().and_then(|idx| idx.line_offset(line))
    }

    /// Invalidate the line index (call after edits)
    pub fn invalidate_line_index(&mut self) {
        self.line_index = None;
    }

    pub fn line_count(&self) -> usize {
        // Try indexed path first (O(1))
        if let Some(count) = self.line_count_indexed() {
            return count;
        }

        log::debug!("[EditableFileManager::line_count] Counting lines (fallback scan)");
        let effective_size = self.effective_size() as usize;
        if effective_size == 0 {
            return 0;
        }
        // Count newlines by scanning through chunks
        let mut count = 1usize; // A file with no newlines has 1 line
        let mut offset = 0usize;
        while offset < effective_size {
            let chunk_size = SEARCH_BUFFER_SIZE.min(effective_size - offset);
            let chunk = self.get_range(offset, chunk_size);
            count += chunk.iter().filter(|&&b| b == b'\n').count();
            offset += chunk_size;
        }
        log::debug!("[EditableFileManager::line_count] File has {} lines", count);
        count
    }

    /// Get lines by line number range (0-based, inclusive start, exclusive end)
    /// Returns the text for those lines including their newline terminators
    pub fn get_lines(&self, start_line: usize, end_line: usize) -> String {
        // Try indexed path first (O(1))
        if let Some(result) = self.get_lines_indexed(start_line, end_line) {
            return result;
        }

        // Fallback: scan from beginning (original implementation)
        log::debug!("[EditableFileManager::get_lines] start_line={}, end_line={} (fallback scan)", start_line, end_line);
        let effective_size = self.effective_size() as usize;
        if effective_size == 0 {
            return String::new();
        }

        // Scan through file to find line boundaries
        let mut line_starts: Vec<usize> = vec![0]; // Line 0 starts at offset 0
        let mut offset = 0usize;
        while offset < effective_size && line_starts.len() <= end_line + 1 {
            let chunk_size = SEARCH_BUFFER_SIZE.min(effective_size - offset);
            let chunk = self.get_range(offset, chunk_size);
            for (i, &byte) in chunk.iter().enumerate() {
                if byte == b'\n' {
                    line_starts.push(offset + i + 1);
                    if line_starts.len() > end_line + 1 {
                        break;
                    }
                }
            }
            offset += chunk_size;
        }
        // Add end-of-file as final line start if needed
        if line_starts.len() <= end_line + 1 {
            line_starts.push(effective_size);
        }

        // Clamp range
        let start = start_line.min(line_starts.len() - 1);
        let end = end_line.min(line_starts.len() - 1);
        let byte_start = line_starts[start];
        let byte_end = line_starts[end + 1].min(effective_size);
        let length = byte_end.saturating_sub(byte_start);

        let result = self.get_text(byte_start, length);
        log::debug!("[EditableFileManager::get_lines] Returning {} chars for lines {}-{}", result.len(), start_line, end_line);
        result
    }

    /// Search for pattern in file (including uncommitted edits)
    pub fn search(&self, pattern: &[u8], start_offset: usize) -> Vec<usize> {
        log::debug!("[EditableFileManager::search] pattern.len={}, start_offset={}", 
            pattern.len(), start_offset);
        let mut results = Vec::new();
        if pattern.is_empty() { return results; }

        let pattern_len = pattern.len();
        let effective_size = self.effective_size() as usize;

        // For small files or few edits, build virtual view and search
        // For large files with many edits, use optimized approach
        if effective_size < SEARCH_BUFFER_SIZE || self.history_position == 0 {
            log::debug!("[EditableFileManager::search] Using full buffer search (size={})", effective_size);
            // Build virtual buffer and search
            let virtual_content = self.get_range(0, effective_size);
            results = self.search_in_slice(&virtual_content, pattern, start_offset);
        } else {
            log::debug!("[EditableFileManager::search] Using chunked search");
            // Chunked search for large files
            let mut search_offset = start_offset;
            while search_offset < effective_size {
                let chunk_size = SEARCH_BUFFER_SIZE.min(effective_size - search_offset);
                let chunk = self.get_range(search_offset, chunk_size);

                // Search in this chunk (including overlap for patterns crossing chunks)
                let search_len = if search_offset + chunk_size < effective_size {
                    chunk.len() // Full chunk
                } else {
                    chunk.len().saturating_sub(pattern_len - 1) // Last chunk, don't overflow
                };

                let chunk_results = self.search_in_slice(&chunk[..search_len.min(chunk.len())],
                    pattern, 0);

                for offset in chunk_results {
                    results.push(search_offset + offset);
                }

                // Move to next chunk (with overlap)
                search_offset += chunk_size.saturating_sub(pattern_len - 1);

                // Prevent infinite loop on small chunks
                if chunk_size <= pattern_len {
                    break;
                }
            }
        }
        log::debug!("[EditableFileManager::search] Found {} matches", results.len());
        results
    }

    /// Search within a byte slice (Boyer-Moore-Horspool algorithm)
    fn search_in_slice(&self, haystack: &[u8], needle: &[u8], start_offset: usize) -> Vec<usize> {
        let mut results = Vec::new();
        if needle.is_empty() || needle.len() > haystack.len() { return results; }

        let needle_len = needle.len();
        let haystack_len = haystack.len();

        // Build skip table for Boyer-Moore-Horspool
        let mut skip_table = vec![needle_len; 256];
        for (i, &byte) in needle.iter().enumerate().take(needle_len - 1) {
            skip_table[byte as usize] = needle_len - 1 - i;
        }

        let mut i = start_offset;
        while i + needle_len <= haystack_len {
            let mut j = needle_len - 1;
            while j > 0 && haystack[i + j] == needle[j] {
                j -= 1;
            }

            if j == 0 && haystack[i] == needle[0] {
                results.push(i);
                i += needle_len; // Skip past this match
            } else {
                i += skip_table[haystack[i + needle_len - 1] as usize].max(1);
            }
        }

        results
    }

    /// Search for text (UTF-8)
    pub fn search_text(&self, text: &str, start_offset: usize) -> Vec<usize> {
        self.search(text.as_bytes(), start_offset)
    }

    /// Search all occurrences
    pub fn search_all(&self, pattern: &[u8]) -> Vec<usize> {
        self.search(pattern, 0)
    }

    /// Replace all occurrences of pattern
    pub fn replace_all(&mut self, pattern: &[u8], replacement: &[u8]) -> usize {
        let matches = self.search_all(pattern);
        let count = matches.len();
        log::info!("[EditableFileManager::replace_all] Found {} matches to replace", count);

        // Apply replacements in reverse order to maintain offsets
        for &offset in matches.iter().rev() {
            self.apply_edit(EditOp::Replace {
                offset,
                length: pattern.len(),
                data: replacement.to_vec(),
            });
        }

        count
    }

    /// Undo last operation
    pub fn undo(&mut self) -> bool {
        if self.history_position > 0 {
            self.history_position -= 1;
            log::debug!("[EditableFileManager::undo] Undo performed. New position: {}", self.history_position);
            true
        } else { 
            log::debug!("[EditableFileManager::undo] Cannot undo - at beginning of history");
            false 
        }
    }

    /// Redo last undone operation
    pub fn redo(&mut self) -> bool {
        if self.history_position < self.edit_history.len() {
            self.history_position += 1;
            log::debug!("[EditableFileManager::redo] Redo performed. New position: {}", self.history_position);
            true
        } else { 
            log::debug!("[EditableFileManager::redo] Cannot redo - at end of history");
            false 
        }
    }

    /// Save changes to disk
    pub fn save(&mut self) -> io::Result<()> {
        log::info!("[EditableFileManager::save] Starting save operation...");
        if !self.has_changes || self.edit_history.is_empty() {
            log::info!("[EditableFileManager::save] No changes to save");
            return Ok(());
        }

        // Create temp file
        let temp_path = self.file_path.with_extension("tmp");
        log::debug!("[EditableFileManager::save] Creating temp file: {:?}", temp_path);
        let temp_file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&temp_path)?;

        let mut writer = BufWriter::new(temp_file);

        if let Some(ref mmap) = self.mmap {
            // Apply all edits and write to temp file
            let mut current_offset: usize = 0;

            // Sort edits by offset
            let mut sorted_edits: Vec<_> = self.edit_history[..self.history_position].iter().collect();
            sorted_edits.sort_by_key(|op| match op {
                EditOp::Insert { offset, .. } | EditOp::Delete { offset, .. } | EditOp::Replace { offset, .. } => *offset,
            });

            log::debug!("[EditableFileManager::save] Applying {} edits", sorted_edits.len());

            // Write with edits applied
            for op in sorted_edits {
                match op {
                    EditOp::Insert { offset, data } => {
                        // Write original data up to insert point
                        if current_offset < *offset {
                            writer.write_all(&mmap[current_offset..*offset])?;
                            current_offset = *offset;
                        }
                        // Write inserted data
                        writer.write_all(data)?;
                    }
                    EditOp::Delete { offset, length } => {
                        // Write original data up to delete point
                        if current_offset < *offset {
                            writer.write_all(&mmap[current_offset..*offset])?;
                        }
                        // Skip deleted data
                        current_offset = *offset + *length;
                    }
                    EditOp::Replace { offset, length, data } => {
                        // Write original data up to replace point
                        if current_offset < *offset {
                            writer.write_all(&mmap[current_offset..*offset])?;
                        }
                        // Write replacement data
                        writer.write_all(data)?;
                        // Skip replaced data
                        current_offset = *offset + *length;
                    }
                }
            }

            // Write remaining data
            if current_offset < self.file_size as usize {
                writer.write_all(&mmap[current_offset..])?;
            }
        }

        writer.flush()?;
        drop(writer);
        log::debug!("[EditableFileManager::save] Temp file written successfully");

        // Atomically replace original file
        std::fs::rename(&temp_path, &self.file_path)?;
        log::info!("[EditableFileManager::save] File saved and replaced atomically");

        // Re-map the file
        let file = File::open(&self.file_path)?;
        let metadata = file.metadata()?;
        self.file_size = metadata.len();
        self.mmap = Some(unsafe { Mmap::map(&file)? });
        log::debug!("[EditableFileManager::save] File re-mapped. New size: {} bytes", self.file_size);

        // Clear history
        self.edit_history.clear();
        self.history_position = 0;
        self.has_changes = false;
        self.modified_regions.clear();
        log::info!("[EditableFileManager::save] Save operation completed successfully");

        Ok(())
    }

    /// Save as new file
    pub fn save_as<P: AsRef<Path>>(&self, new_path: P) -> io::Result<()> {
        log::info!("[EditableFileManager::save_as] Saving to new path: {:?}", new_path.as_ref());
        let new_file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(new_path)?;

        let mut writer = BufWriter::new(new_file);

        if let Some(ref mmap) = self.mmap {
            // Apply edits to output
            let mut current_offset: usize = 0;
            let mut sorted_edits: Vec<_> = self.edit_history[..self.history_position].iter().collect();
            sorted_edits.sort_by_key(|op| match op {
                EditOp::Insert { offset, .. } | EditOp::Delete { offset, .. } | EditOp::Replace { offset, .. } => *offset,
            });

            for op in sorted_edits {
                match op {
                    EditOp::Insert { offset, data } => {
                        if current_offset < *offset {
                            writer.write_all(&mmap[current_offset..*offset])?;
                            current_offset = *offset;
                        }
                        writer.write_all(data)?;
                    }
                    EditOp::Delete { offset, length } => {
                        if current_offset < *offset {
                            writer.write_all(&mmap[current_offset..*offset])?;
                        }
                        current_offset = *offset + *length;
                    }
                    EditOp::Replace { offset, length, data } => {
                        if current_offset < *offset {
                            writer.write_all(&mmap[current_offset..*offset])?;
                        }
                        writer.write_all(data)?;
                        current_offset = *offset + *length;
                    }
                }
            }

            if current_offset < self.file_size as usize {
                writer.write_all(&mmap[current_offset..])?;
            }
        }

        writer.flush()?;
        log::info!("[EditableFileManager::save_as] File saved successfully to new path");
        Ok(())
    }

    /// Helper: convert logical offset to physical offset
    fn logical_to_physical(&self, logical: usize) -> usize {
        // Simplified - full implementation would track edit mappings
        logical.min(self.file_size as usize)
    }

    /// Get modified regions
    pub fn get_modified_regions(&self) -> &BTreeMap<usize, usize> {
        &self.modified_regions
    }
}

/// Search result for UI
#[derive(Debug, Clone, serde::Serialize)]
pub struct SearchResult {
    pub offset: usize,
    pub length: usize,
    pub preview: String,
}

/// Search engine for file content
pub struct SearchEngine;

#[allow(dead_code)]
impl SearchEngine {
    /// Find all occurrences with context
    pub fn find_with_context(
        content: &[u8],
        pattern: &[u8],
        context_bytes: usize
    ) -> Vec<SearchResult> {
        let mut results = Vec::new();
        if pattern.is_empty() { return results; }

        // Simple search (can be optimized with BMH)
        let mut start = 0;
        while let Some(pos) = Self::find_in_slice(&content[start..], pattern) {
            let absolute_pos = start + pos;
            let context_start = absolute_pos.saturating_sub(context_bytes);
            let context_end = (absolute_pos + pattern.len() + context_bytes).min(content.len());

            let preview = String::from_utf8_lossy(&content[context_start..context_end]).to_string();

            results.push(SearchResult {
                offset: absolute_pos,
                length: pattern.len(),
                preview,
            });

            start = absolute_pos + 1;
        }

        results
    }

    fn find_in_slice(haystack: &[u8], needle: &[u8]) -> Option<usize> {
        if needle.len() > haystack.len() { return None; }
        haystack.windows(needle.len()).position(|window| window == needle)
    }
}

/// Batch edit operations
#[derive(Debug, Clone)]
pub struct BatchEdit {
    pub operations: Vec<EditOp>,
}

impl BatchEdit {
    pub fn new() -> Self {
        BatchEdit { operations: Vec::new() }
    }

    pub fn insert(&mut self, offset: usize, data: Vec<u8>) {
        self.operations.push(EditOp::Insert { offset, data });
    }

    pub fn delete(&mut self, offset: usize, length: usize) {
        self.operations.push(EditOp::Delete { offset, length });
    }

    pub fn replace(&mut self, offset: usize, length: usize, data: Vec<u8>) {
        self.operations.push(EditOp::Replace { offset, length, data });
    }

    pub fn apply_to(&self, manager: &mut EditableFileManager) {
        for op in &self.operations {
            manager.apply_edit(op.clone());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_line_index_basic() {
        // "hello\nworld\nfoo\n" = 3 lines
        let data = b"hello\nworld\nfoo\n";
        let index = LineIndex::build(data);

        assert_eq!(index.line_count(), 4); // 0: "hello\n", 1: "world\n", 2: "foo\n", 3: "" (empty trailing line)
        assert_eq!(index.line_offset(0), Some(0));   // "hello\n" starts at 0
        assert_eq!(index.line_offset(1), Some(6));   // "world\n" starts at 6
        assert_eq!(index.line_offset(2), Some(12));  // "foo\n" starts at 12
        assert_eq!(index.line_offset(3), Some(16));  // trailing empty line starts at 16
        assert_eq!(index.line_offset(100), None);    // out of bounds
    }

    #[test]
    fn test_line_index_empty_file() {
        let data = b"";
        let index = LineIndex::build(data);
        assert_eq!(index.line_count(), 0);
    }

    #[test]
    fn test_line_index_single_line_no_newline() {
        let data = b"hello";
        let index = LineIndex::build(data);
        assert_eq!(index.line_count(), 1);
        assert_eq!(index.line_offset(0), Some(0));
        assert_eq!(index.line_end_offset(0), Some(5));
    }

    #[test]
    fn test_line_index_single_line_with_newline() {
        let data = b"hello\n";
        let index = LineIndex::build(data);
        assert_eq!(index.line_count(), 2); // "hello\n" + empty trailing line
        assert_eq!(index.line_offset(0), Some(0));
        assert_eq!(index.line_offset(1), Some(6));
    }

    #[test]
    fn test_line_range() {
        // "aaa\nbbb\nccc\n" = 4 lines (0-2 have content, 3 is empty)
        let data = b"aaa\nbbb\nccc\n";
        let index = LineIndex::build(data);

        // Get byte range for lines 0-1 (should be "aaa\nbbb\n")
        let (start, end) = index.line_range(0, 2).unwrap();
        assert_eq!(start, 0);
        assert_eq!(end, 8);

        // Get byte range for lines 1-2 (should be "bbb\nccc\n")
        let (start2, end2) = index.line_range(1, 3).unwrap();
        assert_eq!(start2, 4);
        assert_eq!(end2, 12);
    }

    #[test]
    fn test_line_index_large_file() {
        // Simulate a 1MB file with ~10000 lines
        let line = b"This is a test line with some content\n";
        let num_lines = 10000;
        let mut data: Vec<u8> = Vec::with_capacity(line.len() * num_lines);
        for _ in 0..num_lines {
            data.extend_from_slice(line);
        }

        let start = std::time::Instant::now();
        let index = LineIndex::build(&data);
        let build_time = start.elapsed();

        assert_eq!(index.line_count(), num_lines + 1); // +1 for trailing empty line

        // Verify O(1) lookups
        let start = std::time::Instant::now();
        for i in 0..num_lines {
            let offset = index.line_offset(i).unwrap();
            assert_eq!(offset, i * line.len());
        }
        let lookup_time = start.elapsed();

        // Verify line_range is fast
        let start = std::time::Instant::now();
        for _ in 0..1000 {
            let _ = index.line_range(5000, 5100);
        }
        let range_time = start.elapsed();

        println!("Line index build time for {} lines: {:?}", num_lines, build_time);
        println!("10000 offset lookups: {:?}", lookup_time);
        println!("1000 range lookups: {:?}", range_time);

        // Build should be under 1 second for 10K lines
        assert!(build_time.as_millis() < 1000);
        // Lookups should be under 1ms total
        assert!(lookup_time.as_micros() < 1000);
    }

    #[test]
    fn test_line_index_from_newline_offsets() {
        // Build from newline positions instead of raw data
        let offsets = vec![5, 11, 20]; // newlines at positions 5, 11, 20
        let index = LineIndex::build_from_newline_offsets(offsets, 25);

        assert_eq!(index.line_count(), 4); // 3 newlines = 4 lines
        assert_eq!(index.line_offset(0), Some(0));   // starts at 0
        assert_eq!(index.line_offset(1), Some(6));    // after newline at 5
        assert_eq!(index.line_offset(2), Some(12));   // after newline at 11
        assert_eq!(index.line_offset(3), Some(21));   // after newline at 20
    }

    #[test]
    fn test_line_index_memory_usage() {
        let data = b"hello\nworld\nfoo\n";
        let index = LineIndex::build(data);
        // 4 line starts * size_of::<usize>()
        assert_eq!(index.memory_usage(), 4 * std::mem::size_of::<usize>());
    }

    #[test]
    fn test_line_index_end_offset() {
        let data = b"aaa\nbbb\n";
        let index = LineIndex::build(data);

        assert_eq!(index.line_end_offset(0), Some(4));   // end of "aaa\n" = start of line 1
        assert_eq!(index.line_end_offset(1), Some(8));   // end of "bbb\n" = file end
        assert_eq!(index.line_end_offset(2), None);      // out of bounds
    }

    #[test]
    fn test_editable_file_manager_line_index() {
        use std::io::Write;

        // Create a temp file with known content
        let temp_dir = std::env::temp_dir();
        let temp_path = temp_dir.join("sak_test_line_index.txt");
        let mut file = std::fs::File::create(&temp_path).unwrap();
        writeln!(file, "line zero").unwrap();
        writeln!(file, "line one").unwrap();
        writeln!(file, "line two").unwrap();
        writeln!(file, "line three").unwrap();

        // Create EditableFileManager and open the file
        let mut manager = EditableFileManager::new(temp_path.to_str().unwrap()).unwrap();

        // Build line index
        manager.build_line_index();

        // Verify line count
        let count = manager.line_count();
        assert_eq!(count, 5); // 4 content lines + 1 empty trailing line

        // Verify get_lines uses index
        let lines = manager.get_lines_indexed(0, 2);
        assert!(lines.is_some());
        let content = lines.unwrap();
        assert!(content.contains("line zero"));
        assert!(content.contains("line one"));

        // Verify line_offset_indexed
        let offset = manager.line_offset_indexed(0);
        assert_eq!(offset, Some(0));

        // Clean up
        let _ = std::fs::remove_file(&temp_path);
    }

    #[test]
    fn test_editable_file_manager_indexed_vs_scan() {
        use std::io::Write;

        // Create a temp file with many lines
        let temp_dir = std::env::temp_dir();
        let temp_path = temp_dir.join("sak_test_line_index_perf.txt");
        let mut file = std::fs::File::create(&temp_path).unwrap();
        for i in 0..5000 {
            writeln!(file, "This is line number {}", i).unwrap();
        }

        // Test without index (fallback scan)
        let manager_no_index = EditableFileManager::new(temp_path.to_str().unwrap()).unwrap();
        // Don't build index - force fallback

        let start = std::time::Instant::now();
        let count_scan = manager_no_index.line_count();
        let scan_time = start.elapsed();

        // Test with index
        let mut manager_indexed = EditableFileManager::new(temp_path.to_str().unwrap()).unwrap();
        manager_indexed.build_line_index();

        let start = std::time::Instant::now();
        let count_indexed = manager_indexed.line_count();
        let indexed_time = start.elapsed();

        assert_eq!(count_scan, count_indexed);
        println!("Line count (5000 lines): scan={:?}, indexed={:?}", scan_time, indexed_time);

        // Test get_lines performance
        let start = std::time::Instant::now();
        for _ in 0..100 {
            let _ = manager_no_index.get_lines(2500, 2510);
        }
        let get_lines_scan_time = start.elapsed();

        let start = std::time::Instant::now();
        for _ in 0..100 {
            let _ = manager_indexed.get_lines(2500, 2510);
        }
        let get_lines_indexed_time = start.elapsed();

        println!("get_lines x100 (2500-2510): scan={:?}, indexed={:?}", 
                 get_lines_scan_time, get_lines_indexed_time);

        // Indexed should be faster
        assert!(get_lines_indexed_time <= get_lines_scan_time);

        // Clean up
        let _ = std::fs::remove_file(&temp_path);
    }
}
