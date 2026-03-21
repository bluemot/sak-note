use serde::{Serialize, Deserialize};
use std::fs::{File, OpenOptions};
use std::io::{self, Read, Write, Seek, SeekFrom};
use std::path::Path;
use flate2::{Compression, write::ZlibEncoder, read::ZlibDecoder};
use crate::mark_engine::{Mark, MarkColor, MarkExport};

/// Magic header for SAK file format
pub const SAK_MAGIC: &[u8] = b"SAK_EDITOR_DATA_V1";
/// Marker to separate original file content from marks
pub const SAK_MARKS_DELIMITER: &[u8] = b"\n__SAK_MARKS_DATA__\n";

/// SAK file container - original content + compressed marks
#[derive(Debug, Clone)]
pub struct SakFile {
    original_content: Vec<u8>,
    marks: Vec<Mark>,
    original_path: Option<String>,
}

impl SakFile {
    /// Create new SAK file from original content and marks
    pub fn new(original_content: Vec<u8>, marks: Vec<Mark>) -> Self {
        SakFile {
            original_content,
            marks,
            original_path: None,
        }
    }
    
    /// Create SAK file from existing file + marks
    pub fn from_file<P: AsRef<Path>>(path: P, marks: Vec<Mark>) -> io::Result<Self> {
        let mut file = File::open(&path)?;
        let mut content = Vec::new();
        file.read_to_end(&mut content)?;
        
        Ok(SakFile {
            original_content: content,
            marks,
            original_path: Some(path.as_ref().to_string_lossy().to_string()),
        })
    }
    
    /// Save as .sak file
    pub fn save<P: AsRef<Path>>(&self, path: P) -> io::Result<()> {
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)?;
        
        // Write original content
        file.write_all(&self.original_content)?;
        
        // Write delimiter
        file.write_all(SAK_MARKS_DELIMITER)?;
        
        // Serialize and compress marks
        let export = SakExport {
            version: 1,
            magic: SAK_MAGIC.to_vec(),
            original_size: self.original_content.len(),
            marks_count: self.marks.len(),
            marks: self.marks.clone(),
            metadata: SakMetadata {
                created_at: current_timestamp(),
                editor_version: env!("CARGO_PKG_VERSION").to_string(),
            },
        };
        
        let json_data = serde_json::to_vec(&export)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        
        // Compress
        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::best());
        encoder.write_all(&json_data)?;
        let compressed = encoder.finish()?;
        
        // Write compressed length (8 bytes, big-endian for readability)
        file.write_all(&(u64::to_be_bytes(compressed.len() as u64)))?;
        
        // Write compressed data
        file.write_all(&compressed)?;
        
        // Write magic trailer
        file.write_all(SAK_MAGIC)?;
        
        file.flush()?;
        Ok(())
    }
    
    /// Load from .sak file
    pub fn load<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let mut file = File::open(&path)?;
        let file_size = file.metadata()?.len() as usize;
        
        // Read entire file
        let mut content = Vec::with_capacity(file_size);
        file.read_to_end(&mut content)?;
        
        // Find delimiter
        let delim_pos = find_subsequence(&content, SAK_MARKS_DELIMITER)
            .ok_or_else(|| io::Error::new(
                io::ErrorKind::InvalidData, 
                "Not a valid SAK file: delimiter not found"
            ))?;
        
        // Extract original content
        let original_content = content[..delim_pos].to_vec();
        
        // Extract marks data (after delimiter)
        let marks_start = delim_pos + SAK_MARKS_DELIMITER.len();
        let marks_data = &content[marks_start..];
        
        // Read compressed length (first 8 bytes)
        if marks_data.len() < 8 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "SAK file too short"
            ));
        }
        
        let compressed_len = u64::from_be_bytes([
            marks_data[0], marks_data[1], marks_data[2], marks_data[3],
            marks_data[4], marks_data[5], marks_data[6], marks_data[7],
        ]) as usize;
        
        // Check magic trailer
        let trailer_start = 8 + compressed_len;
        if trailer_start + SAK_MAGIC.len() > marks_data.len() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "SAK file corrupted: incomplete data"
            ));
        }
        
        if &marks_data[trailer_start..trailer_start + SAK_MAGIC.len()] != SAK_MAGIC {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "SAK file corrupted: invalid magic trailer"
            ));
        }
        
        // Decompress
        let compressed = &marks_data[8..trailer_start];
        let mut decoder = ZlibDecoder::new(compressed);
        let mut json_data = Vec::new();
        decoder.read_to_end(&mut json_data)?;
        
        // Deserialize
        let export: SakExport = serde_json::from_slice(&json_data)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        
        // Verify original size matches
        if export.original_size != original_content.len() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "SAK file corrupted: size mismatch"
            ));
        }
        
        Ok(SakFile {
            original_content,
            marks: export.marks,
            original_path: Some(path.as_ref().to_string_lossy().to_string()),
        })
    }
    
    /// Check if file is a SAK file
    pub fn is_sak_file<P: AsRef<Path>>(path: P) -> bool {
        if let Ok(mut file) = File::open(&path) {
            // Check file extension first
            if let Some(ext) = path.as_ref().extension() {
                if ext.to_string_lossy().to_lowercase() != "sak" {
                    return false;
                }
            }
            
            // Read last 32 bytes to check for magic
            let file_size = match file.metadata() {
                Ok(m) => m.len(),
                Err(_) => return false,
            };
            
            if file_size < SAK_MAGIC.len() as u64 {
                return false;
            }
            
            let seek_pos = file_size - SAK_MAGIC.len() as u64;
            if file.seek(SeekFrom::Start(seek_pos)).is_ok() {
                let mut buf = vec![0u8; SAK_MAGIC.len()];
                if file.read_exact(&mut buf).is_ok() {
                    return buf == SAK_MAGIC;
                }
            }
        }
        false
    }
    
    /// Extract marks from a SAK file (without loading full content)
    pub fn extract_marks<P: AsRef<Path>>(path: P) -> io::Result<Vec<Mark>> {
        let sak = Self::load(path)?;
        Ok(sak.marks)
    }
    
    /// Export as regular file (without marks)
    pub fn export_as_regular<P: AsRef<Path>>(&self, path: P) -> io::Result<()> {
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)?;
        
        file.write_all(&self.original_content)?;
        file.flush()?;
        Ok(())
    }
    
    /// Get original content
    pub fn content(&self) -> &[u8] {
        &self.original_content
    }
    
    /// Get marks
    pub fn marks(&self) -> &[Mark] {
        &self.marks
    }
    
    /// Get marks mutably
    pub fn marks_mut(&mut self) -> &mut Vec<Mark> {
        &mut self.marks
    }
    
    /// Set marks
    pub fn set_marks(&mut self, marks: Vec<Mark>) {
        self.marks = marks;
    }
    
    /// Get file info
    pub fn info(&self) -> SakInfo {
        SakInfo {
            original_size: self.original_content.len(),
            marks_count: self.marks.len(),
            original_path: self.original_path.clone(),
        }
    }
}

/// SAK export format (serialized to JSON)
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SakExport {
    version: u32,
    magic: Vec<u8>,
    original_size: usize,
    marks_count: usize,
    marks: Vec<Mark>,
    metadata: SakMetadata,
}

/// SAK metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SakMetadata {
    created_at: u64,
    editor_version: String,
}

/// SAK file information
#[derive(Debug, Clone)]
pub struct SakInfo {
    pub original_size: usize,
    pub marks_count: usize,
    pub original_path: Option<String>,
}

/// Find subsequence in bytes
fn find_subsequence(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() || haystack.len() < needle.len() {
        return None;
    }
    
    haystack.windows(needle.len())
        .position(|window| window == needle)
}

/// Helper: get current timestamp
fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// Convert regular file to SAK file
pub fn convert_to_sak<P: AsRef<Path>, Q: AsRef<Path>>(
    source: P, 
    target: Q,
    marks: Vec<Mark>
) -> io::Result<()> {
    let sak = SakFile::from_file(source, marks)?;
    sak.save(target)
}

/// Convert SAK file to regular file
pub fn convert_from_sak<P: AsRef<Path>, Q: AsRef<Path>>(
    source: P,
    target: Q
) -> io::Result<Vec<Mark>> {
    let sak = SakFile::load(source)?;
    sak.export_as_regular(target)?;
    Ok(sak.marks)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_sak_file_roundtrip() {
        // Create temp file
        let mut temp = NamedTempFile::new().unwrap();
        let content = b"Hello, World! This is test content.";
        temp.write_all(content).unwrap();
        
        // Create marks
        let marks = vec![
            Mark {
                id: "mark1".to_string(),
                start: 0,
                end: 5,
                color: MarkColor::Red,
                label: Some("Hello".to_string()),
                note: None,
                created_at: 1234567890,
                updated_at: 1234567890,
            },
            Mark {
                id: "mark2".to_string(),
                start: 7,
                end: 12,
                color: MarkColor::Yellow,
                label: Some("World".to_string()),
                note: Some("Important".to_string()),
                created_at: 1234567890,
                updated_at: 1234567890,
            },
        ];
        
        // Convert to SAK
        let sak_path = temp.path().with_extension("sak");
        convert_to_sak(temp.path(), &sak_path, marks.clone()).unwrap();
        
        // Verify it's a SAK file
        assert!(SakFile::is_sak_file(&sak_path));
        assert!(!SakFile::is_sak_file(temp.path()));
        
        // Load and verify
        let sak = SakFile::load(&sak_path).unwrap();
        assert_eq!(sak.content(), content);
        assert_eq!(sak.marks().len(), 2);
        assert_eq!(sak.marks()[0].label, Some("Hello".to_string()));
        assert_eq!(sak.marks()[1].color, MarkColor::Yellow);
        
        // Export back
        let mut out_temp = NamedTempFile::new().unwrap();
        sak.export_as_regular(out_temp.path()).unwrap();
        
        let mut output = Vec::new();
        File::open(out_temp.path()).unwrap().read_to_end(&mut output).unwrap();
        assert_eq!(output, content);
        
        // Cleanup
        std::fs::remove_file(&sak_path).ok();
    }
}
