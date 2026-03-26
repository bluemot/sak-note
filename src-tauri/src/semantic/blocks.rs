//! Semantic Block Definitions
//!
//! Code is divided into meaningful units for LLM understanding

use serde::{Serialize, Deserialize};
use std::fmt;

/// Unique identifier for a semantic block
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BlockId(pub String);

impl BlockId {
    pub fn new(id: impl Into<String>) -> Self {
        BlockId(id.into())
    }
    
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for BlockId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Type of semantic block
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BlockType {
    File,           // Root of a file
    Module,         // Namespace/module
    Import,         // Import/using statement
    Function,       // Function or method
    Class,          // Class definition
    Interface,      // Interface definition
    Trait,          // Trait definition
    Struct,         // Struct definition
    Enum,           // Enum definition
    Field,          // Class/struct field
    Property,       // Property getter/setter
    Method,         // Class method
    Constructor,    // Constructor
    Variable,       // Variable declaration
    Constant,       // Constant declaration
    TypeAlias,      // Type alias
    Block,          // Generic code block
    Comment,        // Comment block
    Documentation,  // Documentation comment
    Test,           // Test function/block
    Config,         // Configuration section
    LogicBranch,    // if/else, match, switch
    Loop,           // for, while, loop
    ErrorHandling,  // try/catch, result handling
    Validation,     // Input validation
    Calculation,    // Computation logic
    Return,         // Return statement
}

/// Location in source code
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Location {
    pub line_start: usize,
    pub line_end: usize,
    pub column_start: usize,
    pub column_end: usize,
    pub byte_offset_start: usize,
    pub byte_offset_end: usize,
}

impl Location {
    pub fn new(line: usize, column: usize) -> Self {
        Self {
            line_start: line,
            line_end: line,
            column_start: column,
            column_end: column,
            byte_offset_start: 0,
            byte_offset_end: 0,
        }
    }
    
    pub fn with_lines(mut self, start: usize, end: usize) -> Self {
        self.line_start = start;
        self.line_end = end;
        self
    }
}

/// A semantic block representing a meaningful code unit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticBlock {
    pub id: BlockId,
    pub block_type: BlockType,
    pub name: String,
    pub content: String,
    pub location: Location,
    pub signature: Option<String>,
    pub documentation: Option<String>,
    pub purpose: Option<String>,  // Human-readable description
    pub tags: Vec<String>,
    pub metadata: BlockMetadata,
    pub children: Vec<BlockId>,
    pub parent: Option<BlockId>,
}

/// Metadata about a block
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BlockMetadata {
    pub visibility: Visibility,
    pub is_async: bool,
    pub is_static: bool,
    pub is_abstract: bool,
    pub is_deprecated: bool,
    pub access_modifiers: Vec<String>,
    pub annotations: Vec<String>,
    pub generic_params: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub enum Visibility {
    #[default]
    Private,
    Public,
    Protected,
    Internal,
}

impl SemanticBlock {
    pub fn new(block_type: BlockType, content: String) -> Self {
        let id = BlockId::new(uuid::Uuid::new_v4().to_string());
        Self {
            id,
            block_type,
            name: String::new(),
            content,
            location: Location::new(1, 1),
            signature: None,
            documentation: None,
            purpose: None,
            tags: Vec::new(),
            metadata: BlockMetadata::default(),
            children: Vec::new(),
            parent: None,
        }
    }
    
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }
    
    pub fn with_location(mut self, location: Location) -> Self {
        self.location = location;
        self
    }
    
    pub fn with_signature(mut self, signature: impl Into<String>) -> Self {
        self.signature = Some(signature.into());
        self
    }
    
    pub fn with_purpose(mut self, purpose: impl Into<String>) -> Self {
        self.purpose = Some(purpose.into());
        self
    }
    
    pub fn with_documentation(mut self, doc: impl Into<String>) -> Self {
        self.documentation = Some(doc.into());
        self
    }
    
    pub fn add_tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }
    
    pub fn add_child(&mut self, child_id: BlockId) {
        self.children.push(child_id);
    }
    
    /// Get a summary of this block for LLM
    pub fn to_llm_summary(&self) -> String {
        let mut summary = format!("{} `{}`", self.block_type_name(), self.name);
        
        if let Some(ref sig) = self.signature {
            summary.push_str(&format!("\n  Signature: {}", sig));
        }
        
        if let Some(ref purpose) = self.purpose {
            summary.push_str(&format!("\n  Purpose: {}", purpose));
        }
        
        summary.push_str(&format!("\n  Lines: {}-{}", 
            self.location.line_start, 
            self.location.line_end
        ));
        
        summary
    }
    
    fn block_type_name(&self) -> &str {
        match self.block_type {
            BlockType::File => "File",
            BlockType::Module => "Module",
            BlockType::Import => "Import",
            BlockType::Function => "Function",
            BlockType::Class => "Class",
            BlockType::Interface => "Interface",
            BlockType::Trait => "Trait",
            BlockType::Struct => "Struct",
            BlockType::Enum => "Enum",
            BlockType::Field => "Field",
            BlockType::Property => "Property",
            BlockType::Method => "Method",
            BlockType::Constructor => "Constructor",
            BlockType::Variable => "Variable",
            BlockType::Constant => "Constant",
            BlockType::TypeAlias => "Type Alias",
            BlockType::Block => "Block",
            BlockType::Comment => "Comment",
            BlockType::Documentation => "Documentation",
            BlockType::Test => "Test",
            BlockType::Config => "Config",
            BlockType::LogicBranch => "Logic Branch",
            BlockType::Loop => "Loop",
            BlockType::ErrorHandling => "Error Handling",
            BlockType::Validation => "Validation",
            BlockType::Calculation => "Calculation",
            BlockType::Return => "Return",
        }
    }
}

/// Parser for extracting semantic blocks from code
pub trait BlockParser {
    fn language(&self) -> &str;
    fn parse(&self, content: &str) -> Vec<SemanticBlock>;
}

/// Simple parser for any text file
pub struct TextParser;

impl BlockParser for TextParser {
    fn language(&self) -> &str {
        "text"
    }
    
    fn parse(&self, content: &str) -> Vec<SemanticBlock> {
        let mut blocks = Vec::new();
        
        // Split by paragraphs/sections
        let lines: Vec<&str> = content.lines().collect();
        let mut current_block = String::new();
        let mut line_num = 1;
        
        for line in &lines {
            if line.trim().is_empty() {
                if !current_block.trim().is_empty() {
                    let block = SemanticBlock::new(
                        BlockType::Block,
                        current_block.clone()
                    ).with_location(Location::new(line_num - current_block.lines().count(), 1));
                    blocks.push(block);
                    current_block.clear();
                }
            } else {
                current_block.push_str(line);
                current_block.push('\n');
            }
            line_num += 1;
        }
        
        // Add remaining block
        if !current_block.trim().is_empty() {
            let block = SemanticBlock::new(
                BlockType::Block,
                current_block
            ).with_location(Location::new(line_num - current_block.lines().count(), 1));
            blocks.push(block);
        }
        
        blocks
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_block_id_creation() {
        let id = BlockId::new("test-123");
        assert_eq!(id.as_str(), "test-123");
    }
    
    #[test]
    fn test_semantic_block_builder() {
        let block = SemanticBlock::new(BlockType::Function, "fn main() {}".to_string())
            .with_name("main")
            .with_signature("fn main() -> ()")
            .with_purpose("Entry point");
        
        assert_eq!(block.name, "main");
        assert_eq!(block.signature, Some("fn main() -> ()".to_string()));
        assert_eq!(block.purpose, Some("Entry point".to_string()));
    }
    
    #[test]
    fn test_text_parser() {
        let parser = TextParser;
        let content = "Line 1\nLine 2\n\nLine 3";
        let blocks = parser.parse(content);
        
        assert_eq!(blocks.len(), 2);
        assert!(blocks[0].content.contains("Line 1"));
        assert!(blocks[1].content.contains("Line 3"));
    }
}
