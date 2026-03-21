use serde::{Serialize, Deserialize};
use std::fs::{File, OpenOptions};
use std::io::{self, Read, Write, Seek, SeekFrom};
use std::path::Path;
use crate::mark_engine::{Mark, MarkColor};

/// SAK file format - Human-readable JSON-based marks storage
/// 
/// File Structure:
/// ```
/// [Original file content - unchanged]
/// \n===SAK_MARKS_BEGIN===\n
/// {JSON marks data}
/// \n===SAK_MARKS_END===\n
/// ```
/// 
/// This format allows:
/// - Normal text editors to open and edit the original content
/// - SAK Editor to extract and manage marks
/// - LLMs to parse the JSON marks directly
/// - Easy manual inspection of marks

pub const SAK_MARKS_BEGIN: &str = "\n===SAK_MARKS_BEGIN===\n";
pub const SAK_MARKS_END: &str = "\n===SAK_MARKS_END===\n";

/// SAK file container
#[derive(Debug, Clone)]
pub struct SakFile {
    original_content: Vec<u8>,
    marks: Vec<Mark>,
    original_path: Option<String>,
}

impl SakFile {
    /// Create new SAK file
    pub fn new(original_content: Vec<u8>, marks: Vec<Mark>) -> Self {
        SakFile {
            original_content,
            marks,
            original_path: None,
        }
    }
    
    /// Create from existing file
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
    
    /// Save as .sak file (JSON format)
    pub fn save<P: AsRef<Path>>(&self, path: P) -> io::Result<()> {
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)?;
        
        // Write original content completely unchanged
        file.write_all(&self.original_content)?;
        
        // If original content doesn't end with newline, add one before marks
        let needs_newline = !self.original_content.ends_with(b"\n")
            && !self.original_content.is_empty();
        if needs_newline {
            file.write_all(b"\n")?;
        }
        
        // Write begin delimiter
        file.write_all(SAK_MARKS_BEGIN.as_bytes())?;
        
        // Create and write JSON marks data
        let sak_data = SakData {
            version: "1.0".to_string(),
            format: "sak-marks-json".to_string(),
            original_size: self.original_content.len(),
            marks_count: self.marks.len(),
            marks: self.marks.clone(),
            metadata: SakMetadata {
                created_at: current_timestamp(),
                editor: "SAK Editor".to_string(),
                editor_version: env!("CARGO_PKG_VERSION").to_string(),
            },
        };
        
        // Pretty print JSON for human readability
        let json = serde_json::to_string_pretty(&sak_data)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        
        file.write_all(json.as_bytes())?;
        
        // Write end delimiter
        file.write_all(SAK_MARKS_END.as_bytes())?;
        
        file.flush()?;
        Ok(())
    }
    
    /// Load from .sak file
    pub fn load<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let mut file = File::open(&path)?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;
        
        // Find begin delimiter
        let begin_pos = content.find(SAK_MARKS_BEGIN)
            .ok_or_else(|| io::Error::new(
                io::ErrorKind::InvalidData,
                "Not a valid SAK file: begin marker not found"
            ))?;
        
        // Find end delimiter
        let end_pos = content.rfind(SAK_MARKS_END)
            .ok_or_else(|| io::Error::new(
                io::ErrorKind::InvalidData,
                "Not a valid SAK file: end marker not found"
            ))?;
        
        // Extract original content (as bytes to preserve encoding)
        let original_str = &content[..begin_pos];
        // Remove trailing newline if it was added before marks
        let original_content = if original_str.ends_with('\n') && !original_str.is_empty() {
            original_str[..original_str.len()-1].as_bytes().to_vec()
        } else {
            original_str.as_bytes().to_vec()
        };
        
        // Extract JSON data
        let json_start = begin_pos + SAK_MARKS_BEGIN.len();
        let json_end = end_pos;
        let json_str = &content[json_start..json_end];
        
        // Parse JSON
        let sak_data: SakData = serde_json::from_str(json_str)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        
        // Verify original size
        if sak_data.original_size != original_content.len() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("SAK file corrupted: size mismatch (expected {}, got {})",
                    sak_data.original_size, original_content.len())
            ));
        }
        
        Ok(SakFile {
            original_content,
            marks: sak_data.marks,
            original_path: Some(path.as_ref().to_string_lossy().to_string()),
        })
    }
    
    /// Check if file is a SAK file
    pub fn is_sak_file<P: AsRef<Path>>(path: P) -> bool {
        if let Ok(mut file) = File::open(&path) {
            // Read up to 64KB from start to check for delimiter
            let mut buf = vec![0u8; 65536];
            match file.read(&mut buf) {
                Ok(n) if n > 0 => {
                    buf.truncate(n);
                    let content = String::from_utf8_lossy(&buf);
                    content.contains(SAK_MARKS_BEGIN)
                }
                _ => false,
            }
        } else {
            false
        }
    }
    
    /// Check if file has .sak extension
    pub fn has_sak_extension<P: AsRef<Path>>(path: P) -> bool {
        path.as_ref()
            .extension()
            .map(|ext| ext.to_string_lossy().to_lowercase() == "sak")
            .unwrap_or(false)
    }
    
    /// Extract marks without loading full content
    pub fn extract_marks<P: AsRef<Path>>(path: P) -> io::Result<Vec<Mark>> {
        let sak = Self::load(path)?;
        Ok(sak.marks)
    }
    
    /// Export as regular file (strip marks)
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
    
    // Getters
    pub fn content(&self) -> &[u8] { &self.original_content }
    pub fn marks(&self) -> &[Mark] { &self.marks }
    pub fn marks_mut(&mut self) -> &mut Vec<Mark> { &mut self.marks }
    pub fn set_marks(&mut self, marks: Vec<Mark>) { self.marks = marks; }
    
    pub fn info(&self) -> SakInfo {
        SakInfo {
            original_size: self.original_content.len(),
            marks_count: self.marks.len(),
            original_path: self.original_path.clone(),
        }
    }
    
    /// Generate LLM-friendly summary of marks
    pub fn to_llm_summary(&self) -> String {
        let mut summary = format!(
            "File: {}\nSize: {} bytes\nMarks: {}\n\n",
            self.original_path.as_deref().unwrap_or("unknown"),
            self.original_size(),
            self.marks.len()
        );
        
        for mark in &self.marks {
            summary.push_str(&format!(
                "- Range: {}-{}",
                mark.start, mark.end
            ));
            if let Some(ref label) = mark.label {
                summary.push_str(&format!(", Label: {}", label));
            }
            summary.push_str(&format!(", Color: {:?}\n", mark.color));
            if let Some(ref note) = mark.note {
                summary.push_str(&format!("  Note: {}\n", note));
            }
        }
        
        summary
    }
    
    fn original_size(&self) -> usize {
        self.original_content.len()
    }
}

/// JSON data structure for marks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SakData {
    pub version: String,
    pub format: String,
    pub original_size: usize,
    pub marks_count: usize,
    pub marks: Vec<Mark>,
    pub metadata: SakMetadata,
}

/// Metadata for SAK file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SakMetadata {
    pub created_at: u64,
    pub editor: String,
    pub editor_version: String,
}

/// File information
#[derive(Debug, Clone)]
pub struct SakInfo {
    pub original_size: usize,
    pub marks_count: usize,
    pub original_path: Option<String>,
}

/// Convert regular file to SAK
pub fn convert_to_sak<P: AsRef<Path>, Q: AsRef<Path>>(
    source: P,
    target: Q,
    marks: Vec<Mark>
) -> io::Result<()> {
    let sak = SakFile::from_file(source, marks)?;
    sak.save(target)
}

/// Convert SAK to regular file
pub fn convert_from_sak<P: AsRef<Path>, Q: AsRef<Path>>(
    source: P,
    target: Q
) -> io::Result<Vec<Mark>> {
    let sak = SakFile::load(source)?;
    sak.export_as_regular(target)?;
    Ok(sak.marks)
}

fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_sak_json_roundtrip() {
        let mut temp = NamedTempFile::new().unwrap();
        let content = "Hello, World!\nLine 2\nLine 3\n";
        temp.write_all(content.as_bytes()).unwrap();
        
        let marks = vec![
            Mark {
                id: "mark1".to_string(),
                start: 0,
                end: 5,
                color: MarkColor::Red,
                label: Some("Hello".to_string()),
                note: Some("First word".to_string()),
                created_at: 1234567890,
                updated_at: 1234567890,
            },
        ];
        
        // Convert to SAK
        let sak_path = temp.path().with_extension("sak");
        convert_to_sak(temp.path(), &sak_path, marks.clone()).unwrap();
        
        // Load and verify
        let sak = SakFile::load(&sak_path).unwrap();
        assert_eq!(String::from_utf8_lossy(sak.content()), content);
        assert_eq!(sak.marks().len(), 1);
        assert_eq!(sak.marks()[0].label, Some("Hello".to_string()));
        
        // Verify JSON is human-readable
        let sak_content = std::fs::read_to_string(&sak_path).unwrap();
        assert!(sak_content.contains("===SAK_MARKS_BEGIN==="));
        assert!(sak_content.contains("\"marks\":"));
        assert!(sak_content.contains("\"color\":\"red\""));
        assert!(sak_content.contains("===SAK_MARKS_END==="));
        
        // Cleanup
        std::fs::remove_file(&sak_path).ok();
    }
    
    #[test]
    fn test_sak_with_null_bytes() {
        let mut temp = NamedTempFile::new().unwrap();
        let mut content = vec![b'H', b'e', b'l', b'l', b'o', 0, 0, 0]; // With null bytes
        temp.write_all(&content).unwrap();
        
        let marks = vec![];
        
        let sak_path = temp.path().with_extension("sak");
        convert_to_sak(temp.path(), &sak_path, marks).unwrap();
        
        let sak = SakFile::load(&sak_path).unwrap();
        assert_eq!(sak.content(), &content);
        
        std::fs::remove_file(&sak_path).ok();
    }
    
    #[test]
    fn test_llm_summary() {
        let sak = SakFile::new(
            b"Hello World".to_vec(),
            vec![Mark {
                id: "m1".to_string(),
                start: 0,
                end: 5,
                color: MarkColor::Red,
                label: Some("Important".to_string()),
                note: Some("Note here".to_string()),
                created_at: 1234567890,
                updated_at: 1234567890,
            }],
        );
        
        let summary = sak.to_llm_summary();
        assert!(summary.contains("Marks:"));
        assert!(summary.contains("Important"));
        assert!(summary.contains("Note here"));
    }
}
