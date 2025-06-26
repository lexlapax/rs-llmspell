//! ABOUTME: Agent trait for LLM-powered components
//! ABOUTME: Extends BaseAgent with conversation management and LLM provider integration

use super::base_agent::BaseAgent;
use crate::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Conversation message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationMessage {
    pub role: String,
    pub content: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Agent trait for LLM-powered components
#[async_trait]
pub trait Agent: BaseAgent {
    /// Get conversation history
    async fn get_conversation(&self) -> Result<Vec<ConversationMessage>>;
    
    /// Add message to conversation
    async fn add_message(&mut self, message: ConversationMessage) -> Result<()>;
    
    /// Clear conversation history
    async fn clear_conversation(&mut self) -> Result<()>;
}