//! Print Manager
//!
//! Handles printing document content

use serde::{Serialize, Deserialize};
use std::process::Command;

#[derive(Debug, Serialize, Deserialize)]
pub struct PrintOptions {
    pub path: String,
    pub content: String,
    pub line_numbers: bool,
    pub color_print: bool,
    pub copies: u32,
}

pub struct PrintManager;

impl PrintManager {
    /// Print content to system printer
    pub fn print(options: PrintOptions) -> Result<String, Box<dyn std::error::Error>> {
        // For Linux, use lpr command
        #[cfg(target_os = "linux")]
        {
            let mut formatted_content = String::new();
            
            // Add line numbers if requested
            if options.line_numbers {
                for (i, line) in options.content.lines().enumerate() {
                    formatted_content.push_str(&format!("{:4} | {}\n", i + 1, line));
                }
            } else {
                formatted_content = options.content;
            }
            
            // Create a temporary file
            let temp_path = std::env::temp_dir().join("sak_print.txt");
            std::fs::write(&temp_path, &formatted_content)?;
            
            // Use lpr to print
            let output = Command::new("lpr")
                .arg("-#")
                .arg(options.copies.to_string())
                .arg(temp_path.to_str().unwrap())
                .output()?;
            
            if output.status.success() {
                Ok("Document sent to printer".to_string())
            } else {
                Err(format!("Print failed: {}", String::from_utf8_lossy(&output.stderr)).into())
            }
        }
        
        #[cfg(target_os = "windows")]
        {
            // Use notepad /p on Windows
            let temp_path = std::env::temp_dir().join("sak_print.txt");
            std::fs::write(&temp_path, &options.content)?;
            
            Command::new("notepad")
                .arg("/p")
                .arg(temp_path.to_str().unwrap())
                .spawn()?;
            
            Ok("Document sent to printer".to_string())
        }
        
        #[cfg(target_os = "macos")]
        {
            let temp_path = std::env::temp_dir().join("sak_print.txt");
            std::fs::write(&temp_path, &options.content)?;
            
            Command::new("lp")
                .arg("-n")
                .arg(options.copies.to_string())
                .arg(temp_path.to_str().unwrap())
                .output()?;
            
            Ok("Document sent to printer".to_string())
        }
    }
    
    /// Export to PDF
    pub fn export_pdf(path: &str, content: &str) -> Result<String, Box<dyn std::error::Error>> {
        let pdf_path = path.replace(std::path::Path::new(path).extension()
            .map(|e| e.to_str().unwrap_or(""))
            .unwrap_or(""), "pdf");
        
        // For now, just save as text (PDF generation requires additional libraries)
        std::fs::write(&pdf_path, content)?;
        
        Ok(pdf_path)
    }
}

// Tauri Commands

#[tauri::command]
pub fn file_print(options: PrintOptions) -> Result<String, String> {
    PrintManager::print(options).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn file_export_pdf(path: String, content: String) -> Result<String, String> {
    PrintManager::export_pdf(&path, &content).map_err(|e| e.to_string())
}