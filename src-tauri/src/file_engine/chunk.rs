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
        let file = File::open(&path)?;
        let metadata = file.metadata()?;
        let file_size = metadata.len();

        // Memory map the file
        let mmap = unsafe { Mmap::map(&file)? };

        // Calculate chunks
        let num_chunks = ((file_size + CHUNK_SIZE as u64 - 1) / CHUNK_SIZE as u64) as usize;
        let mut chunks = Vec::with_capacity(num_chunks);

        for i in 0..num_chunks {
            let offset = i * CHUNK_SIZE;
            let length = if i == num_chunks - 1 {
                (file_size as usize) - offset
            } else {
                CHUNK_SIZE
            };
            chunks.push(ChunkInfo { offset, length });
        }

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

    pub fn get_chunk(&self, chunk_id: usize) -> Option<Chunk> {
        if chunk_id >= self.chunks.len() { return None; }
        let info = &self.chunks[chunk_id];
        if let Some(ref mmap) = self.mmap {
            let data = mmap[info.offset..info.offset + info.length].to_vec();
            Some(Chunk { id: chunk_id, offset: info.offset, length: info.length, data })
        } else { None }
    }

    pub fn get_text_range(&self, start: usize, end: usize) -> Option<String> {
        if let Some(ref mmap) = self.mmap {
            let end = end.min(self.file_size as usize);
            let bytes = &mmap[start..end];
            Some(String::from_utf8_lossy(bytes).to_string())
        } else { None }
    }

    pub fn get_bytes(&self, start: usize, length: usize) -> Option<Vec<u8>> {
        if let Some(ref mmap) = self.mmap {
            let end = (start + length).min(self.file_size as usize);
            Some(mmap[start..end].to_vec())
        } else { None }
    }

    pub fn get_byte(&self, position: usize) -> Option<u8> {
        if let Some(ref mmap) = self.mmap {
            mmap.get(position).copied()
        } else { None }
    }

    pub fn get_hex_range(&self, start: usize, length: usize) -> Vec<(usize, Vec<u8>)> {
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
        result
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
}

impl EditableFileManager {
    pub fn new<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let file = File::open(&path)?;
        let metadata = file.metadata()?;
        let file_size = metadata.len();
        let mmap = unsafe { Mmap::map(&file)? };

        Ok(EditableFileManager {
            file_path: path.as_ref().to_path_buf(),
            file_size,
            mmap: Some(mmap),
            edit_history: Vec::new(),
            history_position: 0,
            has_changes: false,
            modified_regions: BTreeMap::new(),
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
        size.max(0) as u64
    }

    /// Apply edit operation
    pub fn apply_edit(&mut self, op: EditOp) {
        // Remove any redo history
        if self.history_position < self.edit_history.len() {
            self.edit_history.truncate(self.history_position);
        }

        // Add to history
        self.edit_history.push(op.clone());
        self.history_position += 1;
        self.has_changes = true;

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
        // Apply edits to get the logical view
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

        result
    }

    /// Get text range (considering edits)
    pub fn get_text(&self, start: usize, length: usize) -> String {
        let bytes = self.get_range(start, length);
        String::from_utf8_lossy(&bytes).to_string()
    }

    /// Search for pattern in file (including uncommitted edits)
    pub fn search(&self, pattern: &[u8], start_offset: usize) -> Vec<usize> {
        let mut results = Vec::new();
        if pattern.is_empty() { return results; }

        let pattern_len = pattern.len();
        let effective_size = self.effective_size() as usize;

        // For small files or few edits, build virtual view and search
        // For large files with many edits, use optimized approach
        if effective_size < SEARCH_BUFFER_SIZE || self.history_position == 0 {
            // Build virtual buffer and search
            let virtual_content = self.get_range(0, effective_size);
            results = self.search_in_slice(&virtual_content, pattern, start_offset);
        } else {
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
            true
        } else { false }
    }

    /// Redo last undone operation
    pub fn redo(&mut self) -> bool {
        if self.history_position < self.edit_history.len() {
            self.history_position += 1;
            true
        } else { false }
    }

    /// Save changes to disk
    pub fn save(&mut self) -> io::Result<()> {
        if !self.has_changes || self.edit_history.is_empty() {
            return Ok(());
        }

        // Create temp file
        let temp_path = self.file_path.with_extension("tmp");
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

        // Atomically replace original file
        std::fs::rename(&temp_path, &self.file_path)?;

        // Re-map the file
        let file = File::open(&self.file_path)?;
        let metadata = file.metadata()?;
        self.file_size = metadata.len();
        self.mmap = Some(unsafe { Mmap::map(&file)? });

        // Clear history
        self.edit_history.clear();
        self.history_position = 0;
        self.has_changes = false;
        self.modified_regions.clear();

        Ok(())
    }

    /// Save as new file
    pub fn save_as<P: AsRef<Path>>(&self, new_path: P) -> io::Result<()> {
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
