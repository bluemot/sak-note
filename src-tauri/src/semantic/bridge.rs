//! LLM Bridge Module
//!
//! Converts between code and LLM-friendly formats

use crate::semantic::blocks::*;
use crate::semantic::SemanticDocument;
use serde::{Serialize, Deserialize};

/// Format for LLM consumption
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMFormat {
    pub format_version: String,
    pub summary: String,
    pub blocks: Vec<LLMBlock>,
    pub relationships: Vec<LLMRelationship>,
    pub metadata: LLMMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMBlock {
    pub id: String,
    pub block_type: String,
    pub name: String,
    pub signature: Option<String>,
    pub purpose: Option<String>,
    pub content_preview: String,
    pub line_range: (usize, usize),
    pub children: Vec<String>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMRelationship {
    pub source: String,
    pub target: String,
    pub kind: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMMetadata {
    pub language: String,
    pub file_path: String,
    pub total_lines: usize,
    pub block_count: usize,
}

/// Bridge between code and LLM formats
pub struct LLMBridge;

impl LLMBridge {
    /// Export semantic document to LLM format
    pub fn export(doc: &SemanticDocument) -> LLMFormat {
        let blocks: Vec<LLMBlock> = doc.blocks
            .iter()
            .map(|b| Self::block_to_llm(b))
            .collect();
        
        let relationships: Vec<LLMRelationship> = doc.relationships
            .iter()
            .map(|r| LLMRelationship {
                source: r.source.0.clone(),
                target: r.target.0.clone(),
                kind: format!("{:?}", r.kind),
            })
            .collect();
        
        LLMFormat {
            format_version: "1.0".to_string(),
            summary: doc.summary.clone(),
            blocks,
            relationships,
            metadata: LLMMetadata {
                language: doc.language.clone(),
                file_path: doc.path.clone(),
                total_lines: doc.blocks.iter().map(|b| b.location.line_end).max().unwrap_or(0),
                block_count: doc.blocks.len(),
            },
        }
    }

    fn block_to_llm(block: &SemanticBlock) -> LLMBlock {
        // Truncate content for preview
        let preview = if block.content.len() > 200 {
            format!("{}...", &block.content[..200])
        } else {
            block.content.clone()
        };
        
        LLMBlock {
            id: block.id.0.clone(),
            block_type: format!("{:?}", block.block_type),
            name: block.name.clone(),
            signature: block.signature.clone(),
            purpose: block.purpose.clone(),
            content_preview: preview,
            line_range: (block.location.line_start, block.location.line_end),
            children: block.children.iter().map(|c| c.0.clone()).collect(),
            tags: block.tags.clone(),
        }
    }

    /// Export to compact format for chat context
    pub fn export_compact(doc: &SemanticDocument) -> String {
        let mut output = format!("File: {}\n", doc.path);
        output.push_str(&format!("Language: {}\n", doc.language));
        output.push_str(&format!("Blocks: {}\n\n", doc.blocks.len()));
        
        for block in &doc.blocks {
            let line = format!(
                "[{}] {} `{}` (L{}-{}){}",
                format!("{:?}", block.block_type),
                block.name,
                block.signature.as_deref().unwrap_or(""),
                block.location.line_start,
                block.location.line_end,
                block.purpose.as_ref()
                    .map(|p| format!(" - {}", p))
                    .unwrap_or_default()
            );
            output.push_str(&line);
            output.push('\n');
        }
        
        output
    }

    /// Export to tree format
    pub fn export_tree(doc: &SemanticDocument) -> String {
        let mut output = format!("📄 {}\n", doc.path);
        
        for block in &doc.blocks {
            let icon = Self::block_icon(&block.block_type);
            let indent = if block.parent.is_some() { "  " } else { "" };
            
            output.push_str(&format!(
                "{}{} {} `{}`\n",
                indent,
                icon,
                format!("{:?}", block.block_type),
                block.name
            ));
        }
        
        output
    }

    fn block_icon(block_type: &BlockType) -> &'static str {
        match block_type {
            BlockType::File => "📄",
            BlockType::Function => "🔧",
            BlockType::Class => "🏗️",
            BlockType::Struct => "📦",
            BlockType::Interface => "🔌",
            BlockType::Trait => "🎭",
            BlockType::Import | BlockType::Module => "📥",
            BlockType::Test => "🧪",
            BlockType::Variable | BlockType::Field => "📋",
            BlockType::Return => "🔙",
            BlockType::LogicBranch => "🌿",
            BlockType::Loop => "🔄",
            _ => "📎",
        }
    }
}

/// Edit suggestions from LLM
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMEditRequest {
    pub intent: String,
    pub description: String,
    pub target: Option<String>,
    pub changes: Vec<LLMChange>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMChange {
    pub action: String,
    pub target_block: Option<String>,
    pub new_content: Option<String>,
    pub reference: Option<String>,
}

/// Parse LLM edit request into semantic edits
pub struct LLMEditParser;

impl LLMEditParser {
    /// Parse natural language edit request
    pub fn parse(request: &str) -> Result<LLMEditRequest, String> {
        // Pattern matching for common edit patterns
        let lower = request.to_lowercase();
        
        if lower.contains("add field") || lower.contains("add a field") {
            Self::parse_add_field(request)
        } else if lower.contains("extract function") || lower.contains("refactor") {
            Self::parse_extract_function(request)
        } else if lower.contains("add import") || lower.contains("import") {
            Self::parse_add_import(request)
        } else if lower.contains("rename") {
            Self::parse_rename(request)
        } else {
            Ok(LLMEditRequest {
                intent: "generic_edit".to_string(),
                description: request.to_string(),
                target: None,
                changes: vec![],
            })
        }
    }

    fn parse_add_field(request: &str) -> Result<LLMEditRequest, String> {
        // Pattern: "add field X of type Y to Z"
        let re = regex::Regex::new(r"add\s+(?:a\s+)?field\s+(\w+)\s+(?:of\s+)?type\s+(\w+)\s+(?:to|in)\s+(\w+)").unwrap();
        
        if let Some(cap) = re.captures(&request.to_lowercase()) {
            Ok(LLMEditRequest {
                intent: "add_field".to_string(),
                description: request.to_string(),
                target: Some(cap[3].to_string()),
                changes: vec![LLMChange {
                    action: "add_field".to_string(),
                    target_block: Some(cap[3].to_string()),
                    new_content: Some(format!("{}: {}", &cap[1], &cap[2])),
                    reference: None,
                }],
            })
        } else {
            Err("Could not parse add field request".to_string())
        }
    }

    fn parse_extract_function(request: &str) -> Result<LLMEditRequest, String> {
        // Pattern: "extract function X from Y"
        let re = regex::Regex::new(r"extract\s+(?:function|method)\s+(\w+)(?:\s+from)?").unwrap();
        
        if let Some(cap) = re.captures(&request.to_lowercase()) {
            Ok(LLMEditRequest {
                intent: "extract_function".to_string(),
                description: request.to_string(),
                target: Some(cap[1].to_string()),
                changes: vec![LLMChange {
                    action: "extract_function".to_string(),
                    target_block: None,
                    new_content: Some(cap[1].to_string()),
                    reference: None,
                }],
            })
        } else {
            Err("Could not parse extract function request".to_string())
        }
    }

    fn parse_add_import(request: &str) -> Result<LLMEditRequest, String> {
        // Pattern: "import X from Y"
        let re = regex::Regex::new(r#"import\s+(.+)\s+from\s+['"'](.+)['"']"#).unwrap();
        
        if let Some(cap) = re.captures(request) {
            Ok(LLMEditRequest {
                intent: "add_import".to_string(),
                description: request.to_string(),
                target: None,
                changes: vec![LLMChange {
                    action: "add_import".to_string(),
                    target_block: None,
                    new_content: Some(format!("import {} from '{}'", &cap[1], &cap[2])),
                    reference: None,
                }],
            })
        } else {
            Err("Could not parse import request".to_string())
        }
    }

    fn parse_rename(request: &str) -> Result<LLMEditRequest, String> {
        // Pattern: "rename X to Y"
        let re = regex::Regex::new(r"rename\s+(\w+)\s+to\s+(\w+)").unwrap();
        
        if let Some(cap) = re.captures(&request.to_lowercase()) {
            Ok(LLMEditRequest {
                intent: "rename".to_string(),
                description: request.to_string(),
                target: Some(cap[1].to_string()),
                changes: vec![LLMChange {
                    action: "rename".to_string(),
                    target_block: Some(cap[1].to_string()),
                    new_content: Some(cap[2].to_string()),
                    reference: None,
                }],
            })
        } else {
            Err("Could not parse rename request".to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_doc() -> SemanticDocument {
        let mut doc = SemanticDocument::new(
            "/test/user.rs".to_string(),
            "rust".to_string()
        );
        
        doc.blocks.push(SemanticBlock::new(
            BlockType::Struct,
            "struct User { name: String }".to_string()
        ).with_name("User").with_purpose("User data structure"));
        
        doc.summary = "User management module".to_string();
        doc
    }

    #[test]
    fn test_export_to_llm_format() {
        let doc = create_test_doc();
        let format = LLMBridge::export(&doc);
        
        assert_eq!(format.metadata.language, "rust");
        assert_eq!(format.blocks.len(), 1);
        assert_eq!(format.blocks[0].name, "User");
    }

    #[test]
    fn test_export_compact() {
        let doc = create_test_doc();
        let compact = LLMBridge::export_compact(&doc);
        
        assert!(compact.contains("User"));
        assert!(compact.contains("rust"));
    }

    #[test]
    fn test_export_tree() {
        let doc = create_test_doc();
        let tree = LLMBridge::export_tree(&doc);
        
        assert!(tree.contains("📦"));
    }

    #[test]
    fn test_parse_add_field() {
        let request = "add field email of type String to User";
        let result = LLMEditParser::parse(request).unwrap();
        
        assert_eq!(result.intent, "add_field");
        assert_eq!(result.target, Some("User".to_string()));
    }

    #[test]
    fn test_parse_extract_function() {
        let request = "extract function calculateTotal";
        let result = LLMEditParser::parse(request).unwrap();
        
        assert_eq!(result.intent, "extract_function");
    }

    #[test]
    fn test_parse_add_import() {
        let request = "import { useState } from 'react'";
        let result = LLMEditParser::parse(request).unwrap();
        
        assert_eq!(result.intent, "add_import");
    }

    #[test]
    fn test_parse_rename() {
        let request = "rename oldName to newName";
        let result = LLMEditParser::parse(request).unwrap();
        
        assert_eq!(result.intent, "rename");
        assert_eq!(result.target, Some("oldName".to_string()));
    }
}
