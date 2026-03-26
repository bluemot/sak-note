//! Natural Language Query Engine
//!
//! Allows LLM to query code using natural language

use crate::semantic::blocks::*;
use crate::semantic::SemanticDocument;
use regex::Regex;

/// Query result containing matched blocks
#[derive(Debug, Clone)]
pub struct QueryResult {
    pub matches: Vec<QueryMatch>,
    pub total: usize,
    pub query_time_ms: u64,
}

#[derive(Debug, Clone)]
pub struct QueryMatch {
    pub block: SemanticBlock,
    pub relevance_score: f32,
    pub match_reason: String,
}

/// Natural language query engine
pub struct QueryEngine<'a> {
    document: &'a SemanticDocument,
}

impl<'a> QueryEngine<'a> {
    pub fn new(document: &'a SemanticDocument) -> Self {
        Self { document }
    }

    /// Execute natural language query
    pub fn execute(&self, query: &str) -> Vec<&'a SemanticBlock> {
        let normalized = query.to_lowercase();
        
        // Pattern matching for common queries
        if normalized.contains("function") || normalized.contains("fn") {
            self.find_functions(&normalized)
        } else if normalized.contains("import") || normalized.contains("use ") {
            self.find_imports(&normalized)
        } else if normalized.contains("struct") || normalized.contains("class") {
            self.find_types(&normalized)
        } else if normalized.contains("test") {
            self.find_tests()
        } else if normalized.contains("main") || normalized.contains("entry") {
            self.find_entry_points()
        } else {
            // Generic search by name or content
            self.search_by_keyword(&normalized)
        }
    }

    /// Find functions matching query
    fn find_functions(&self, query: &str) -> Vec<&'a SemanticBlock> {
        let name_filter = self.extract_name_from_query(query, &["function", "fn", "called", "named"]);
        
        self.document.blocks
            .iter()
            .filter(|b| matches!(b.block_type, BlockType::Function | BlockType::Method))
            .filter(|b| {
                if let Some(ref name) = name_filter {
                    b.name.to_lowercase().contains(name)
                } else {
                    true
                }
            })
            .filter(|b| {
                // Filter by purpose keywords
                if query.contains("handle") || query.contains("process") {
                    b.purpose.as_ref()
                        .map(|p| p.to_lowercase().contains("handle") || p.to_lowercase().contains("process"))
                        .unwrap_or(false) ||
                    b.name.to_lowercase().contains("handle") ||
                    b.name.to_lowercase().contains("process")
                } else {
                    true
                }
            })
            .collect()
    }

    /// Find imports matching query
    fn find_imports(&self, query: &str) -> Vec<&'a SemanticBlock> {
        let module_filter = self.extract_name_from_query(query, &["from", "import"]);
        
        self.document.blocks
            .iter()
            .filter(|b| matches!(b.block_type, BlockType::Import))
            .filter(|b| {
                if let Some(ref module) = module_filter {
                    b.name.to_lowercase().contains(module) ||
                    b.content.to_lowercase().contains(module)
                } else {
                    true
                }
            })
            .collect()
    }

    /// Find struct/class/enum definitions
    fn find_types(&self, query: &str) -> Vec<&'a SemanticBlock> {
        let name_filter = self.extract_name_from_query(query, &["struct", "class", "enum", "type", "called"]);
        
        self.document.blocks
            .iter()
            .filter(|b| matches!(b.block_type, 
                BlockType::Struct | BlockType::Class | BlockType::Enum | BlockType::Interface | BlockType::Trait))
            .filter(|b| {
                if let Some(ref name) = name_filter {
                    b.name.to_lowercase().contains(name)
                } else {
                    true
                }
            })
            .collect()
    }

    /// Find test blocks
    fn find_tests(&self) -> Vec<&'a SemanticBlock> {
        self.document.blocks
            .iter()
            .filter(|b| {
                matches!(b.block_type, BlockType::Test) ||
                b.name.to_lowercase().contains("test") ||
                b.name.starts_with("test_") ||
                b.tags.iter().any(|t| t == "test")
            })
            .collect()
    }

    /// Find entry points (main functions, etc.)
    fn find_entry_points(&self) -> Vec<&'a SemanticBlock> {
        self.document.blocks
            .iter()
            .filter(|b| {
                b.name == "main" ||
                b.name == "run" ||
                b.name == "start" ||
                matches!(b.block_type, BlockType::Constructor) ||
                b.purpose.as_ref().map(|p| 
                    p.to_lowercase().contains("entry") || 
                    p.to_lowercase().contains("start")
                ).unwrap_or(false)
            })
            .collect()
    }

    /// Generic keyword search
    fn search_by_keyword(&self, query: &str) -> Vec<&'a SemanticBlock> {
        let keywords: Vec<&str> = query.split_whitespace().collect();
        
        self.document.blocks
            .iter()
            .filter(|b| {
                let searchable = format!("{} {} {} {}", 
                    b.name.to_lowercase(),
                    b.content.to_lowercase(),
                    b.purpose.as_ref().unwrap_or(&String::new()).to_lowercase(),
                    b.documentation.as_ref().unwrap_or(&String::new()).to_lowercase()
                );
                
                keywords.iter().any(|kw| searchable.contains(kw))
            })
            .collect()
    }

    /// Extract name from query patterns like "function called X"
    fn extract_name_from_query(&self, query: &str, keywords: &[&str]) -> Option<String> {
        for keyword in keywords {
            let pattern = format!(r"{}\s+(?:called|named)?\s+(\w+)", keyword);
            let re = Regex::new(&pattern).ok()?;
            if let Some(cap) = re.captures(query) {
                return Some(cap[1].to_string());
            }
        }
        
        // Try to extract last word as name
        let words: Vec<&str> = query.split_whitespace().collect();
        words.last().map(|w| w.to_string())
    }

    /// Advanced query with scoring
    pub fn execute_scored(&self, query: &str) -> QueryResult {
        let start = std::time::Instant::now();
        
        let blocks = self.execute(query);
        let mut matches: Vec<QueryMatch> = blocks
            .into_iter()
            .map(|b| QueryMatch {
                block: b.clone(),
                relevance_score: self.calculate_relevance(b, query),
                match_reason: self.generate_match_reason(b, query),
            })
            .collect();
        
        // Sort by relevance
        matches.sort_by(|a, b| b.relevance_score.partial_cmp(&a.relevance_score).unwrap());
        
        let total = matches.len();
        let query_time_ms = start.elapsed().as_millis() as u64;
        
        QueryResult {
            total,
            matches,
            query_time_ms,
        }
    }

    fn calculate_relevance(&self, block: &SemanticBlock, query: &str) -> f32 {
        let query_lower = query.to_lowercase();
        let mut score: f32 = 0.0;
        
        // Name match is highest priority
        if block.name.to_lowercase().contains(&query_lower) {
            score += 1.0;
        }
        
        // Purpose/description match
        if let Some(ref purpose) = block.purpose {
            if purpose.to_lowercase().contains(&query_lower) {
                score += 0.8;
            }
        }
        
        // Content match
        if block.content.to_lowercase().contains(&query_lower) {
            score += 0.5;
        }
        
        // Documentation match
        if let Some(ref doc) = block.documentation {
            if doc.to_lowercase().contains(&query_lower) {
                score += 0.6;
            }
        }
        
        // Boost for specific block types based on query
        if query_lower.contains("function") && matches!(block.block_type, BlockType::Function) {
            score += 0.3;
        }
        if query_lower.contains("class") && matches!(block.block_type, BlockType::Class) {
            score += 0.3;
        }
        
        score = score.min(1.0);
        score
    }

    fn generate_match_reason(&self, block: &SemanticBlock, query: &str) -> String {
        let query_lower = query.to_lowercase();
        
        if block.name.to_lowercase().contains(&query_lower) {
            format!("Name matches '{}'", query)
        } else if block.purpose.as_ref().map(|p| p.to_lowercase().contains(&query_lower)).unwrap_or(false) {
            format!("Purpose: {}", block.purpose.as_ref().unwrap())
        } else if block.content.to_lowercase().contains(&query_lower) {
            "Content contains query".to_string()
        } else {
            format!("Type: {:?}", block.block_type)
        }
    }
}

/// Query suggestions for autocomplete
pub struct QuerySuggester;

impl QuerySuggester {
    pub fn suggest(query: &str) -> Vec<String> {
        let suggestions = vec![
            "function called ",
            "import from ",
            "struct named ",
            "class ",
            "test for ",
            "find ",
            "where is ",
            "how to ",
            "entry point",
            "main function",
            "all imports",
            "all functions",
            "all classes",
        ];
        
        suggestions
            .into_iter()
            .filter(|s| s.starts_with(&query.to_lowercase()))
            .map(|s| s.to_string())
            .collect()
    }
    
    pub fn examples() -> Vec<String> {
        vec![
            "function called main".to_string(),
            "import from std".to_string(),
            "struct named User".to_string(),
            "all tests".to_string(),
            "entry point".to_string(),
            "where is the login handler".to_string(),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_document() -> SemanticDocument {
        let mut doc = SemanticDocument::new(
            "/test/main.rs".to_string(),
            "rust".to_string()
        );
        
        // Add test blocks
        doc.blocks.push(SemanticBlock::new(
            BlockType::Function,
            "fn main() {}".to_string()
        ).with_name("main").with_purpose("Entry point"));
        
        doc.blocks.push(SemanticBlock::new(
            BlockType::Function,
            "fn handle_request() {}".to_string()
        ).with_name("handle_request").with_purpose("Handle HTTP requests"));
        
        doc.blocks.push(SemanticBlock::new(
            BlockType::Import,
            "use std::fs::File;".to_string()
        ).with_name("std::fs::File"));
        
        doc.blocks.push(SemanticBlock::new(
            BlockType::Struct,
            "struct User { name: String }".to_string()
        ).with_name("User").with_purpose("User data structure"));
        
        doc
    }

    #[test]
    fn test_query_function() {
        let doc = create_test_document();
        let engine = QueryEngine::new(&doc);
        
        let results = engine.execute("function called main");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "main");
    }

    #[test]
    fn test_query_imports() {
        let doc = create_test_document();
        let engine = QueryEngine::new(&doc);
        
        let results = engine.execute("import from std");
        assert_eq!(results.len(), 1);
        assert!(results[0].name.contains("std"));
    }

    #[test]
    fn test_query_entry_point() {
        let doc = create_test_document();
        let engine = QueryEngine::new(&doc);
        
        let results = engine.execute("entry point");
        assert!(!results.is_empty());
        assert!(results.iter().any(|b| b.name == "main"));
    }

    #[test]
    fn test_query_by_keyword() {
        let doc = create_test_document();
        let engine = QueryEngine::new(&doc);
        
        let results = engine.execute("handle");
        assert!(!results.is_empty());
        assert!(results.iter().any(|b| b.name == "handle_request"));
    }

    #[test]
    fn test_scored_query() {
        let doc = create_test_document();
        let engine = QueryEngine::new(&doc);
        
        let result = engine.execute_scored("main");
        assert!(!result.matches.is_empty());
        assert!(result.matches[0].relevance_score > 0.0);
    }
}
