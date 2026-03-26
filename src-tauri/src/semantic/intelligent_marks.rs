//! Intelligent Marking System
//! 
//! LLM analyzes code and automatically adds color highlights
//! Then controls the editor to navigate and display them

use serde::{Deserialize, Serialize};
use crate::semantic::SemanticDocument;
use crate::semantic::blocks::{BlockType, SemanticBlock};
use crate::mark_engine::{MarkColor, Mark, MarkUpdate};

/// Importance level for code elements
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ImportanceLevel {
    Critical,    // Red - Security issues, bugs
    High,        // Orange - Core logic, important functions
    Medium,      // Yellow - Worth noting, complex logic
    Low,         // Green - Documentation, comments
    Info,        // Blue - References, related code
}

impl ImportanceLevel {
    pub fn to_mark_color(&self) -> MarkColor {
        match self {
            ImportanceLevel::Critical => MarkColor::Red,
            ImportanceLevel::High => MarkColor::Orange,
            ImportanceLevel::Medium => MarkColor::Yellow,
            ImportanceLevel::Low => MarkColor::Green,
            ImportanceLevel::Info => MarkColor::Blue,
        }
    }
}

/// Analysis result from LLM
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeAnalysis {
    pub summary: String,
    pub key_insights: Vec<String>,
    pub marked_sections: Vec<MarkedSection>,
    pub suggestions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarkedSection {
    pub block_id: String,
    pub block_name: String,
    pub block_type: String,
    pub line_start: usize,
    pub line_end: usize,
    pub importance: ImportanceLevel,
    pub reason: String,
    pub note: String,  // LLM's explanation for this mark
}

/// Intelligent marking engine
pub struct IntelligentMarkEngine;

impl IntelligentMarkEngine {
    /// Analyze document and suggest marks
    pub fn analyze(document: &SemanticDocument) -> CodeAnalysis {
        let mut marked_sections = Vec::new();
        let mut key_insights = Vec::new();

        // Analyze each block
        for block in &document.blocks {
            if let Some((importance, reason)) = Self::analyze_block(block, document) {
                let note = Self::generate_note(block, importance, &reason);
                
                marked_sections.push(MarkedSection {
                    block_id: block.id.0.clone(),
                    block_name: block.name.clone(),
                    block_type: format!("{:?}", block.block_type),
                    line_start: block.location.line_start,
                    line_end: block.location.line_end,
                    importance,
                    reason: reason.clone(),
                    note,
                });

                if importance == ImportanceLevel::Critical || importance == ImportanceLevel::High {
                    key_insights.push(format!("{}: {}", block.name, reason));
                }
            }
        }

        CodeAnalysis {
            summary: Self::generate_summary(document, &marked_sections),
            key_insights,
            marked_sections,
            suggestions: Self::generate_suggestions(document),
        }
    }

    /// Analyze a single block
    fn analyze_block(block: &SemanticBlock, _doc: &SemanticDocument) -> Option<(ImportanceLevel, String)> {
        let block_type = &block.block_type;
        let name = &block.name;
        let content = &block.content;
        let purpose = block.purpose.as_deref().unwrap_or("");
        
        // Pattern-based analysis (simulating LLM intelligence)
        
        // Critical: Security-related
        if Self::contains_security_patterns(content) {
            return Some((ImportanceLevel::Critical, 
                "Security-sensitive code detected".to_string()));
        }
        
        // Critical: Unsafe operations
        if content.contains("unsafe") || content.contains("unwrap()") {
            return Some((ImportanceLevel::Critical,
                "Contains unsafe or unwrap operations".to_string()));
        }
        
        // High: Main entry points
        if name == "main" || purpose.to_lowercase().contains("entry") {
            return Some((ImportanceLevel::High,
                "Application entry point".to_string()));
        }
        
        // High: Authentication/Authorization
        if name.to_lowercase().contains("auth") || 
           name.to_lowercase().contains("login") ||
           name.to_lowercase().contains("verify") {
            return Some((ImportanceLevel::High,
                "Authentication/Security logic".to_string()));
        }
        
        // High: Core business logic
        if matches!(block_type, BlockType::Function | BlockType::Method) &&
           (purpose.to_lowercase().contains("core") ||
            purpose.to_lowercase().contains("main") ||
            content.len() > 100) {
            return Some((ImportanceLevel::High,
                "Core business logic".to_string()));
        }
        
        // Medium: Error handling
        if name.to_lowercase().contains("error") ||
           name.to_lowercase().contains("handle") ||
           content.contains("Result<") ||
           content.contains("Option<") {
            return Some((ImportanceLevel::Medium,
                "Error handling logic".to_string()));
        }
        
        // Medium: Complex logic
        if content.matches('{').count() > 5 || 
           content.contains("if") && content.contains("else") {
            return Some((ImportanceLevel::Medium,
                "Contains branching logic".to_string()));
        }
        
        // Info: Public APIs
        if content.contains("pub fn") || content.contains("pub struct") {
            return Some((ImportanceLevel::Info,
                "Public API surface".to_string()));
        }
        
        // Low: Data structures
        if matches!(block_type, BlockType::Struct | BlockType::Class) {
            return Some((ImportanceLevel::Low,
                "Data structure definition".to_string()));
        }
        
        None
    }

    fn contains_security_patterns(content: &str) -> bool {
        let patterns = [
            "password", "secret", "token", "api_key", "private_key",
            "SQL", "sql injection", "XSS", "sanitize",
            "hash", "encrypt", "decrypt",
        ];
        
        let content_lower = content.to_lowercase();
        patterns.iter().any(|p| content_lower.contains(p))
    }

    fn generate_note(block: &SemanticBlock, importance: ImportanceLevel, reason: &str) -> String {
        let emoji = match importance {
            ImportanceLevel::Critical => "🚨",
            ImportanceLevel::High => "⭐",
            ImportanceLevel::Medium => "📌",
            ImportanceLevel::Low => "📝",
            ImportanceLevel::Info => "ℹ️",
        };
        
        format!("{} {} ({}): {} - Lines {}-{}\nPurpose: {}",
            emoji,
            block.name,
            format!("{:?}", block.block_type),
            reason,
            block.location.line_start,
            block.location.line_end,
            block.purpose.as_deref().unwrap_or("No description")
        )
    }

    fn generate_summary(doc: &SemanticDocument, sections: &[MarkedSection]) -> String {
        let critical = sections.iter().filter(|s| s.importance == ImportanceLevel::Critical).count();
        let high = sections.iter().filter(|s| s.importance == ImportanceLevel::High).count();
        
        format!(
            "Analysis of {} ({} blocks): Found {} critical and {} high-importance sections.",
            doc.path,
            doc.blocks.len(),
            critical,
            high
        )
    }

    fn generate_suggestions(doc: &SemanticDocument) -> Vec<String> {
        let mut suggestions = Vec::new();
        
        // Check for missing documentation
        let undocumented: Vec<_> = doc.blocks.iter()
            .filter(|b| matches!(b.block_type, BlockType::Function | BlockType::Struct | BlockType::Class))
            .filter(|b| b.purpose.is_none() && b.documentation.is_none())
            .collect();
        
        if !undocumented.is_empty() {
            suggestions.push(format!(
                "Consider adding documentation to {} public items",
                undocumented.len()
            ));
        }
        
        // Check for error handling coverage
        let has_error_handling = doc.blocks.iter()
            .any(|b| b.name.to_lowercase().contains("error"));
        
        if !has_error_handling {
            suggestions.push("Consider adding dedicated error handling module".to_string());
        }
        
        suggestions
    }

    /// Convert analysis to editor marks
    pub fn to_editor_marks(analysis: &CodeAnalysis) -> Vec<Mark> {
        analysis.marked_sections.iter().map(|section| {
            Mark::new(
                section.line_start,
                section.line_end,
                section.importance.to_mark_color(),
            ).with_label(format!("{}: {}", 
                section.importance.to_mark_color(),
                section.reason
            ))
        }).collect()
    }
}

/// Command for LLM to apply intelligent marks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplyIntelligentMarksCommand {
    pub file_path: String,
    pub analysis_query: String,  // e.g., "Find all security-critical code"
    pub auto_navigate: bool,     // Whether to jump to first critical mark
}

/// Result of applying intelligent marks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplyIntelligentMarksResult {
    pub success: bool,
    pub marks_added: usize,
    pub critical_count: usize,
    pub navigation_target: Option<NavigationTarget>,
    pub summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NavigationTarget {
    pub file_path: String,
    pub line: usize,
    pub reason: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::semantic::blocks::*;

    fn create_test_document() -> SemanticDocument {
        let mut doc = SemanticDocument::new(
            "/test/auth.rs".to_string(),
            "rust".to_string()
        );

        // Add a security-sensitive function
        doc.blocks.push(SemanticBlock::new(
            BlockType::Function,
            "pub fn verify_password(password: &str) -> bool { password == SECRET }".to_string()
        )
        .with_name("verify_password")
        .with_location(Location::new(10, 1).with_lines(10, 15)));

        // Add a regular function
        doc.blocks.push(SemanticBlock::new(
            BlockType::Function,
            "pub fn format_name(name: &str) -> String { name.to_string() }".to_string()
        )
        .with_name("format_name")
        .with_location(Location::new(20, 1).with_lines(20, 22)));

        // Add main function
        doc.blocks.push(SemanticBlock::new(
            BlockType::Function,
            "fn main() { println!(\"Hello\"); }".to_string()
        )
        .with_name("main")
        .with_purpose("Entry point")
        .with_location(Location::new(1, 1).with_lines(1, 3)));

        doc
    }

    #[test]
    fn test_intelligent_analysis() {
        let doc = create_test_document();
        let analysis = IntelligentMarkEngine::analyze(&doc);

        // Should find the security function
        let security_marks: Vec<_> = analysis.marked_sections.iter()
            .filter(|s| s.importance == ImportanceLevel::Critical)
            .collect();
        
        assert!(!security_marks.is_empty());
        assert!(security_marks.iter().any(|s| s.block_name == "verify_password"));

        // Should find main
        let main_marks: Vec<_> = analysis.marked_sections.iter()
            .filter(|s| s.block_name == "main")
            .collect();
        
        assert_eq!(main_marks.len(), 1);
        assert_eq!(main_marks[0].importance, ImportanceLevel::High);
    }

    #[test]
    fn test_importance_to_color() {
        assert_eq!(ImportanceLevel::Critical.to_mark_color(), MarkColor::Red);
        assert_eq!(ImportanceLevel::High.to_mark_color(), MarkColor::Orange);
        assert_eq!(ImportanceLevel::Info.to_mark_color(), MarkColor::Blue);
    }
}
