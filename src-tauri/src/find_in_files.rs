//! Find in Files
//!
//! Search across multiple files in directory

use serde::{Serialize, Deserialize};
use walkdir::WalkDir;
use regex::RegexBuilder;
use std::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub path: String,
    pub line: usize,
    pub column: usize,
    pub text: String,
    pub context: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FindInFilesOptions {
    pub query: String,
    pub directory: Option<String>,
    pub filters: Option<String>,
    pub case_sensitive: bool,
    pub regex: bool,
    pub recursive: bool,
}

pub struct FindInFiles;

impl FindInFiles {
    pub fn search(options: FindInFilesOptions) -> Result<Vec<SearchResult>, Box<dyn std::error::Error>> {
        let mut results = vec![];
        
        let dir = options.directory.clone().unwrap_or_else(|| ".".to_string());
        
        // Parse filters
        let filters: Vec<&str> = options.filters
            .as_ref()
            .map(|f| f.split(',').collect())
            .unwrap_or_else(|| vec!["*"]);
        
        // Build regex for file filtering
        for entry in WalkDir::new(&dir)
            .max_depth(if options.recursive { usize::MAX } else { 1 })
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {
            let path = entry.path();
            let path_str = path.to_string_lossy();
            
            // Check file extension filter
            if !Self::matches_filter(&path_str, &filters) {
                continue;
            }
            
            // Search in file
            match Self::search_file(&path_str, &options) {
                Ok(file_results) => results.extend(file_results),
                Err(_) => continue,
            }
        }
        
        Ok(results)
    }
    
    fn matches_filter(path: &str, filters: &[&str]) -> bool {
        if filters.contains(&"*") {
            return true;
        }
        
        let ext = path.split('.').last().unwrap_or("");
        filters.iter().any(|f| {
            let pattern = f.trim();
            if pattern.starts_with("*.") {
                ext == &pattern[2..]
            } else {
                path.contains(pattern)
            }
        })
    }
    
    fn search_file(path: &str, options: &FindInFilesOptions) -> Result<Vec<SearchResult>, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let mut results = vec![];
        
        if options.regex {
            // Regex search
            let regex = RegexBuilder::new(&options.query)
                .case_insensitive(!options.case_sensitive)
                .build()?;
            
            for (line_num, line) in content.lines().enumerate() {
                if regex.is_match(line) {
                    // Find column
                    let mat = regex.find(line).unwrap();
                    results.push(SearchResult {
                        path: path.to_string(),
                        line: line_num + 1,
                        column: mat.start() + 1,
                        text: line.to_string(),
                        context: line.to_string(),
                    });
                }
            }
        } else {
            // Simple string search
            for (line_num, line) in content.lines().enumerate() {
                let matches = if options.case_sensitive {
                    line.contains(&options.query)
                } else {
                    line.to_lowercase().contains(&options.query.to_lowercase())
                };
                
                if matches {
                    let col = if options.case_sensitive {
                        line.find(&options.query).unwrap_or(0)
                    } else {
                        line.to_lowercase().find(&options.query.to_lowercase()).unwrap_or(0)
                    };
                    
                    results.push(SearchResult {
                        path: path.to_string(),
                        line: line_num + 1,
                        column: col + 1,
                        text: line.to_string(),
                        context: line.to_string(),
                    });
                }
            }
        }
        
        Ok(results)
    }
}

// Tauri Command

#[tauri::command]
pub fn find_in_files(
    query: String,
    directory: Option<String>,
    filters: Option<String>,
    case_sensitive: bool,
    regex: bool,
    recursive: bool
) -> Result<Vec<SearchResult>, String> {
    let options = FindInFilesOptions {
        query,
        directory,
        filters,
        case_sensitive,
        regex,
        recursive,
    };
    
    FindInFiles::search(options).map_err(|e| e.to_string())
}