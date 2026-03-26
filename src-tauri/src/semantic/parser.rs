//! Code Parser Module
//!
//! Parses code into semantic blocks for LLM understanding

use crate::semantic::blocks::*;
use regex::Regex;
use std::collections::HashMap;

/// Parse result containing blocks and any errors
#[derive(Debug, Clone)]
pub struct ParseResult {
    pub blocks: Vec<SemanticBlock>,
    pub errors: Vec<ParseError>,
}

#[derive(Debug, Clone)]
pub struct ParseError {
    pub line: usize,
    pub message: String,
}

impl ParseResult {
    pub fn new() -> Self {
        Self {
            blocks: Vec::new(),
            errors: Vec::new(),
        }
    }
}

/// Language-aware code parser
pub struct CodeParser {
    language: String,
}

impl CodeParser {
    pub fn new(language: impl Into<String>) -> Self {
        Self {
            language: language.into(),
        }
    }
    
    pub fn parse(&self, content: &str) -> ParseResult {
        match self.language.as_str() {
            "rust" => RustParser::new().parse(content),
            "typescript" | "javascript" | "ts" | "js" => TypeScriptParser::new().parse(content),
            "python" | "py" => PythonParser::new().parse(content),
            _ => GenericParser::new(&self.language).parse(content),
        }
    }
    
    pub fn detect_language(file_path: &str) -> String {
        let ext = file_path
            .split('.')
            .last()
            .unwrap_or("")
            .to_lowercase();
        
        match ext.as_str() {
            "rs" => "rust",
            "ts" => "typescript",
            "tsx" => "typescript",
            "js" => "javascript",
            "jsx" => "javascript",
            "py" => "python",
            "json" => "json",
            "md" | "markdown" => "markdown",
            "toml" => "toml",
            "yaml" | "yml" => "yaml",
            _ => "text",
        }
        .to_string()
    }
}

/// Rust-specific parser
pub struct RustParser;

impl RustParser {
    pub fn new() -> Self {
        Self
    }
    
    pub fn parse(&self, content: &str) -> ParseResult {
        let mut result = ParseResult::new();
        let lines: Vec<&str> = content.lines().collect();
        
        // Regex patterns for Rust
        let fn_regex = Regex::new(r"^(?:pub(?:\([^)]+\))?\s+)?(?:async\s+)?(?:unsafe\s+)?fn\s+(\w+)").unwrap();
        let struct_regex = Regex::new(r"^(?:pub\s+)?struct\s+(\w+)").unwrap();
        let impl_regex = Regex::new(r"^impl(?:\s+\w+)?\s+for\s+(\w+)").unwrap();
        let use_regex = Regex::new(r"^use\s+(.+);$").unwrap();
        let mod_regex = Regex::new(r"^(?:pub\s+)?mod\s+(\w+);").unwrap();
        let trait_regex = Regex::new(r"^(?:pub\s+)?trait\s+(\w+)").unwrap();
        let enum_regex = Regex::new(r"^(?:pub\s+)?enum\s+(\w+)").unwrap();
        
        let mut current_block: Option<SemanticBlock> = None;
        let mut brace_count = 0;
        let mut block_start = 0;
        
        for (i, line) in lines.iter().enumerate() {
            let line_num = i + 1;
            let trimmed = line.trim();
            
            // Count braces for block detection
            brace_count += line.matches('{').count();
            brace_count -= line.matches('}').count();
            
            // Detect imports
            if let Some(cap) = use_regex.captures(trimmed) {
                let block = SemanticBlock::new(
                    BlockType::Import,
                    line.to_string()
                )
                .with_name(&cap[1])
                .with_location(Location::new(line_num, 1))
                .with_purpose(format!("Import {}", &cap[1]));
                result.blocks.push(block);
                continue;
            }
            
            // Detect modules
            if let Some(cap) = mod_regex.captures(trimmed) {
                let block = SemanticBlock::new(
                    BlockType::Module,
                    line.to_string()
                )
                .with_name(&cap[1])
                .with_location(Location::new(line_num, 1))
                .with_purpose(format!("Module {}", &cap[1]));
                result.blocks.push(block);
                continue;
            }
            
            // Detect functions
            if let Some(cap) = fn_regex.captures(trimmed) {
                // Close previous block if any
                if let Some(mut block) = current_block.take() {
                    block.location = block.location.with_lines(block_start, line_num - 1);
                    result.blocks.push(block);
                }
                
                let fn_name = &cap[1];
                current_block = Some(
                    SemanticBlock::new(
                        BlockType::Function,
                        line.to_string()
                    )
                    .with_name(fn_name)
                    .with_location(Location::new(line_num, 1))
                    .with_signature(line.trim().to_string())
                    .with_purpose(format!("Function {}", fn_name))
                );
                block_start = line_num;
                continue;
            }
            
            // Detect structs
            if let Some(cap) = struct_regex.captures(trimmed) {
                let block = SemanticBlock::new(
                    BlockType::Struct,
                    line.to_string()
                )
                .with_name(&cap[1])
                .with_location(Location::new(line_num, 1))
                .with_signature(line.trim().to_string())
                .with_purpose(format!("Struct {}", &cap[1]));
                result.blocks.push(block);
                continue;
            }
            
            // Accumulate block content
            if let Some(ref mut block) = current_block {
                block.content.push('\n');
                block.content.push_str(line);
            }
            
            // Close block when braces balance
            if brace_count == 0 && current_block.is_some() {
                let mut block = current_block.take().unwrap();
                block.location = block.location.with_lines(block_start, line_num);
                result.blocks.push(block);
            }
        }
        
        // Handle unclosed block
        if let Some(mut block) = current_block {
            block.location = block.location.with_lines(block_start, lines.len());
            result.blocks.push(block);
        }
        
        result
    }
}

/// TypeScript/JavaScript parser
pub struct TypeScriptParser;

impl TypeScriptParser {
    pub fn new() -> Self {
        Self
    }
    
    pub fn parse(&self, content: &str) -> ParseResult {
        let mut result = ParseResult::new();
        let lines: Vec<&str> = content.lines().collect();
        
        // Regex patterns
        let import_regex = Regex::new(r"^import\s+(.+)\s+from\s+['\"](.+)['\"];?").unwrap();
        let fn_regex = Regex::new(r"^(?:export\s+)?(?:async\s+)?(?:function|const|let)\s+(\w+)\s*(?:=|:)?\s*(?:\([^)]*\)|async\s*\([^)]*\))\s*=>").unwrap();
        let class_regex = Regex::new(r"^(?:export\s+)?class\s+(\w+)").unwrap();
        let interface_regex = Regex::new(r"^(?:export\s+)?interface\s+(\w+)").unwrap();
        
        let mut current_block: Option<SemanticBlock> = None;
        let mut brace_count = 0;
        let mut block_start = 0;
        
        for (i, line) in lines.iter().enumerate() {
            let line_num = i + 1;
            let trimmed = line.trim();
            
            brace_count += line.matches('{').count();
            brace_count -= line.matches('}').count();
            
            // Detect imports
            if let Some(cap) = import_regex.captures(trimmed) {
                let block = SemanticBlock::new(
                    BlockType::Import,
                    line.to_string()
                )
                .with_name(&cap[2])  // module name
                .with_location(Location::new(line_num, 1))
                .with_purpose(format!("Import from {}", &cap[2]));
                result.blocks.push(block);
                continue;
            }
            
            // Detect functions
            if let Some(cap) = fn_regex.captures(trimmed) {
                if let Some(mut block) = current_block.take() {
                    block.location = block.location.with_lines(block_start, line_num - 1);
                    result.blocks.push(block);
                }
                
                current_block = Some(
                    SemanticBlock::new(
                        BlockType::Function,
                        line.to_string()
                    )
                    .with_name(&cap[1])
                    .with_location(Location::new(line_num, 1))
                    .with_signature(line.trim().to_string())
                );
                block_start = line_num;
                continue;
            }
            
            // Detect classes
            if let Some(cap) = class_regex.captures(trimmed) {
                let block = SemanticBlock::new(
                    BlockType::Class,
                    line.to_string()
                )
                .with_name(&cap[1])
                .with_location(Location::new(line_num, 1))
                .with_purpose(format!("Class {}", &cap[1]));
                result.blocks.push(block);
                continue;
            }
            
            // Detect interfaces
            if let Some(cap) = interface_regex.captures(trimmed) {
                let block = SemanticBlock::new(
                    BlockType::Interface,
                    line.to_string()
                )
                .with_name(&cap[1])
                .with_location(Location::new(line_num, 1))
                .with_purpose(format!("Interface {}", &cap[1]));
                result.blocks.push(block);
                continue;
            }
            
            if let Some(ref mut block) = current_block {
                block.content.push('\n');
                block.content.push_str(line);
            }
            
            if brace_count == 0 && current_block.is_some() {
                let mut block = current_block.take().unwrap();
                block.location = block.location.with_lines(block_start, line_num);
                result.blocks.push(block);
            }
        }
        
        if let Some(mut block) = current_block {
            block.location = block.location.with_lines(block_start, lines.len());
            result.blocks.push(block);
        }
        
        result
    }
}

/// Python parser
pub struct PythonParser;

impl PythonParser {
    pub fn new() -> Self {
        Self
    }
    
    pub fn parse(&self, content: &str) -> ParseResult {
        let mut result = ParseResult::new();
        let lines: Vec<&str> = content.lines().collect();
        
        let import_regex = Regex::new(r"^(?:from\s+(\S+)\s+)?import\s+(.+)").unwrap();
        let fn_regex = Regex::new(r"^(?:async\s+)?def\s+(\w+)\s*\(").unwrap();
        let class_regex = Regex::new(r"^class\s+(\w+)(?:\([^)]+\))?").unwrap();
        
        for (i, line) in lines.iter().enumerate() {
            let line_num = i + 1;
            let trimmed = line.trim();
            
            // Skip empty lines
            if trimmed.is_empty() {
                continue;
            }
            
            // Detect imports
            if let Some(cap) = import_regex.captures(trimmed) {
                let module = cap.get(1).map(|m| m.as_str()).unwrap_or("")
                let names = &cap[2];
                
                let block = SemanticBlock::new(
                    BlockType::Import,
                    line.to_string()
                )
                .with_name(names)
                .with_location(Location::new(line_num, 1))
                .with_purpose(format!("Import {} from {}", names, module));
                result.blocks.push(block);
                continue;
            }
            
            // Detect functions
            if let Some(cap) = fn_regex.captures(trimmed) {
                let block = SemanticBlock::new(
                    BlockType::Function,
                    line.to_string()
                )
                .with_name(&cap[1])
                .with_location(Location::new(line_num, 1))
                .with_signature(line.trim().to_string())
                .with_purpose(format!("Function {}", &cap[1]));
                result.blocks.push(block);
                continue;
            }
            
            // Detect classes
            if let Some(cap) = class_regex.captures(trimmed) {
                let block = SemanticBlock::new(
                    BlockType::Class,
                    line.to_string()
                )
                .with_name(&cap[1])
                .with_location(Location::new(line_num, 1))
                .with_purpose(format!("Class {}", &cap[1]));
                result.blocks.push(block);
                continue;
            }
        }
        
        result
    }
}

/// Generic parser for other languages
pub struct GenericParser {
    language: String,
}

impl GenericParser {
    pub fn new(language: &str) -> Self {
        Self {
            language: language.to_string(),
        }
    }
    
    pub fn parse(&self, content: &str) -> ParseResult {
        let mut result = ParseResult::new();
        
        // Simple line-based parsing
        let lines: Vec<&str> = content.lines().collect();
        let mut current_block = String::new();
        let mut block_start = 1;
        
        for (i, line) in lines.iter().enumerate() {
            let line_num = i + 1;
            
            // Split on empty lines or comments
            if line.trim().is_empty() || line.trim().starts_with("#") || line.trim().starts_with("//") {
                if !current_block.trim().is_empty() {
                    let block = SemanticBlock::new(
                        BlockType::Block,
                        current_block.clone()
                    )
                    .with_location(Location::new(block_start, 1).with_lines(block_start, line_num - 1))
                    .with_purpose(format!("{} block", self.language));
                    result.blocks.push(block);
                    current_block.clear();
                }
                block_start = line_num + 1;
            } else {
                current_block.push_str(line);
                current_block.push('\n');
            }
        }
        
        // Add remaining
        if !current_block.trim().is_empty() {
            let block = SemanticBlock::new(
                BlockType::Block,
                current_block
            )
            .with_location(Location::new(block_start, 1).with_lines(block_start, lines.len()))
            .with_purpose(format!("{} block", self.language));
            result.blocks.push(block);
        }
        
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_rust_parser() {
        let code = r#"
use std::fs::File;

pub fn main() {
    println!("Hello");
}

struct User {
    name: String,
}
"#;
        
        let parser = RustParser::new();
        let result = parser.parse(code);
        
        assert!(result.blocks.len() >= 3); // use, fn, struct
        
        let imports: Vec<_> = result.blocks.iter()
            .filter(|b| matches!(b.block_type, BlockType::Import))
            .collect();
        assert_eq!(imports.len(), 1);
        
        let functions: Vec<_> = result.blocks.iter()
            .filter(|b| matches!(b.block_type, BlockType::Function))
            .collect();
        assert_eq!(functions.len(), 1);
        assert_eq!(functions[0].name, "main");
    }
    
    #[test]
    fn test_typescript_parser() {
        let code = r#"
import React from 'react';

function App() {
    return <div>Hello</div>;
}

interface Props {
    name: string;
}
"#;
        
        let parser = TypeScriptParser::new();
        let result = parser.parse(code);
        
        assert!(result.blocks.len() >= 3);
        
        let imports: Vec<_> = result.blocks.iter()
            .filter(|b| matches!(b.block_type, BlockType::Import))
            .collect();
        assert_eq!(imports.len(), 1);
    }
    
    #[test]
    fn test_language_detection() {
        assert_eq!(CodeParser::detect_language("main.rs"), "rust");
        assert_eq!(CodeParser::detect_language("app.ts"), "typescript");
        assert_eq!(CodeParser::detect_language("script.py"), "python");
        assert_eq!(CodeParser::detect_language("config.json"), "json");
    }
}
