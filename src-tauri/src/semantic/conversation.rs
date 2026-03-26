//! Conversation Management for LLM Interaction
//!
//! Manages chat history and context for code understanding

#![allow(dead_code)]

use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::time::SystemTime;
use uuid::Uuid;

/// A conversation session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conversation {
    pub id: String,
    pub created_at: SystemTime,
    pub updated_at: SystemTime,
    pub messages: Vec<ConversationMessage>,
    pub file_context: Option<String>,
    pub language: Option<String>,
}

/// A message in a conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationMessage {
    pub role: String,  // "user", "assistant", "system"
    pub content: String,
    pub timestamp: SystemTime,
}

impl Conversation {
    pub fn new() -> Self {
        let now = SystemTime::now();
        Self {
            id: Uuid::new_v4().to_string(),
            created_at: now,
            updated_at: now,
            messages: Vec::new(),
            file_context: None,
            language: None,
        }
    }

    pub fn with_file_context(mut self, path: String, language: String) -> Self {
        self.file_context = Some(path);
        self.language = Some(language);
        self
    }

    pub fn add_message(&mut self, message: ConversationMessage) {
        self.messages.push(message);
        self.updated_at = SystemTime::now();
    }

    pub fn get_last_n_messages(&self, n: usize) -> &[ConversationMessage] {
        let start = self.messages.len().saturating_sub(n);
        &self.messages[start..]
    }

    pub fn to_llm_prompt(&self) -> String {
        let mut prompt = String::new();
        
        if let Some(ref file) = self.file_context {
            prompt.push_str(&format!("Context: Working on file {}\n", file));
        }
        
        prompt.push_str("Conversation:\n");
        for msg in &self.messages {
            prompt.push_str(&format!("{}: {}\n", msg.role, msg.content));
        }
        
        prompt
    }
}

/// Manages multiple conversations
pub struct ConversationManager {
    conversations: HashMap<String, Conversation>,
}

impl ConversationManager {
    pub fn new() -> Self {
        Self {
            conversations: HashMap::new(),
        }
    }

    pub fn add_conversation(&mut self, conversation: Conversation) {
        self.conversations.insert(conversation.id.clone(), conversation);
    }

    pub fn get_conversation(&self, id: &str) -> Option<&Conversation> {
        self.conversations.get(id)
    }

    pub fn get_conversation_mut(&mut self, id: &str) -> Option<&mut Conversation> {
        self.conversations.get_mut(id)
    }

    pub fn remove_conversation(&mut self, id: &str) -> Option<Conversation> {
        self.conversations.remove(id)
    }

    pub fn list_conversations(&self) -> Vec<&Conversation> {
        self.conversations.values().collect()
    }

    pub fn cleanup_old_conversations(&mut self, max_age: std::time::Duration) {
        let now = SystemTime::now();
        self.conversations.retain(|_, conv| {
            now.duration_since(conv.updated_at)
                .map(|age| age < max_age)
                .unwrap_or(true)
        });
    }
}

/// Intent recognition for user messages
pub struct IntentRecognizer;

#[derive(Debug, Clone, PartialEq)]
pub enum UserIntent {
    QueryCode,      // "find function X", "where is Y"
    EditCode,       // "add field", "rename X to Y"
    ExplainCode,    // "what does this do", "explain X"
    GenerateCode,   // "create function X", "implement Y"
    Navigate,       // "go to X", "show me Y"
    GeneralChat,    // "hello", "help"
}

impl IntentRecognizer {
    pub fn recognize(message: &str) -> UserIntent {
        let lower = message.to_lowercase();
        
        if Self::is_query(&lower) {
            UserIntent::QueryCode
        } else if Self::is_edit(&lower) {
            UserIntent::EditCode
        } else if Self::is_explain(&lower) {
            UserIntent::ExplainCode
        } else if Self::is_generate(&lower) {
            UserIntent::GenerateCode
        } else if Self::is_navigate(&lower) {
            UserIntent::Navigate
        } else {
            UserIntent::GeneralChat
        }
    }

    fn is_query(msg: &str) -> bool {
        msg.contains("find") 
            || msg.contains("where")
            || msg.contains("show")
            || msg.contains("list")
            || msg.contains("search")
    }

    fn is_edit(msg: &str) -> bool {
        msg.contains("add")
            || msg.contains("remove")
            || msg.contains("delete")
            || msg.contains("rename")
            || msg.contains("change")
            || msg.contains("update")
            || msg.contains("edit")
            || msg.contains("extract")
            || msg.contains("refactor")
    }

    fn is_explain(msg: &str) -> bool {
        msg.contains("explain")
            || msg.contains("what")
            || msg.contains("how")
            || msg.contains("why")
            || msg.contains("what does")
            || msg.contains("describe")
    }

    fn is_generate(msg: &str) -> bool {
        msg.contains("create")
            || msg.contains("generate")
            || msg.contains("implement")
            || msg.contains("write")
            || msg.contains("make")
    }

    fn is_navigate(msg: &str) -> bool {
        msg.contains("go to")
            || msg.contains("jump to")
            || msg.contains("open")
            || msg.contains("navigate")
    }
}

/// Response templates for different intents
pub struct ResponseTemplates;

impl ResponseTemplates {
    pub fn for_intent(intent: UserIntent) -> &'static str {
        match intent {
            UserIntent::QueryCode => {
                "I'll help you find that in the code. Let me search for it."
            }
            UserIntent::EditCode => {
                "I can help you edit the code. Let me understand what you want to change."
            }
            UserIntent::ExplainCode => {
                "I'll explain this code for you. Here's what it does:"
            }
            UserIntent::GenerateCode => {
                "I'll help you create that. Let me generate the code:"
            }
            UserIntent::Navigate => {
                "I'll take you there."
            }
            UserIntent::GeneralChat => {
                "I'm here to help! What would you like to do with your code?"
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_conversation_creation() {
        let conv = Conversation::new();
        assert!(!conv.id.is_empty());
        assert!(conv.messages.is_empty());
    }

    #[test]
    fn test_conversation_with_context() {
        let conv = Conversation::new()
            .with_file_context("/test/main.rs".to_string(), "rust".to_string());
        
        assert_eq!(conv.file_context, Some("/test/main.rs".to_string()));
        assert_eq!(conv.language, Some("rust".to_string()));
    }

    #[test]
    fn test_add_message() {
        let mut conv = Conversation::new();
        conv.add_message(ConversationMessage {
            role: "user".to_string(),
            content: "Hello".to_string(),
            timestamp: SystemTime::now(),
        });
        
        assert_eq!(conv.messages.len(), 1);
        assert_eq!(conv.messages[0].content, "Hello");
    }

    #[test]
    fn test_intent_recognition() {
        assert_eq!(IntentRecognizer::recognize("find function main"), UserIntent::QueryCode);
        assert_eq!(IntentRecognizer::recognize("add field email"), UserIntent::EditCode);
        assert_eq!(IntentRecognizer::recognize("explain this code"), UserIntent::ExplainCode);
        assert_eq!(IntentRecognizer::recognize("create a new function"), UserIntent::GenerateCode);
        assert_eq!(IntentRecognizer::recognize("go to line 42"), UserIntent::Navigate);
        assert_eq!(IntentRecognizer::recognize("hello"), UserIntent::GeneralChat);
    }

    #[test]
    fn test_conversation_manager() {
        let mut manager = ConversationManager::new();
        let conv = Conversation::new();
        let id = conv.id.clone();
        
        manager.add_conversation(conv);
        assert!(manager.get_conversation(&id).is_some());
        
        let removed = manager.remove_conversation(&id);
        assert!(removed.is_some());
        assert!(manager.get_conversation(&id).is_none());
    }
}
