//! ABOUTME: Agent trait for LLM-powered components
//! ABOUTME: Extends BaseAgent with conversation management and LLM provider integration

use super::base_agent::BaseAgent;
use crate::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Role in a conversation.
///
/// Defines the role of a message in a conversation history.
/// Used for maintaining context in LLM interactions.
///
/// # Examples
///
/// ```
/// use llmspell_core::traits::agent::MessageRole;
///
/// assert_eq!(MessageRole::System.to_string(), "system");
/// assert_eq!(MessageRole::User.to_string(), "user");
/// assert_eq!(MessageRole::Assistant.to_string(), "assistant");
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum MessageRole {
    System,
    User,
    Assistant,
}

impl std::fmt::Display for MessageRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MessageRole::System => write!(f, "system"),
            MessageRole::User => write!(f, "user"),
            MessageRole::Assistant => write!(f, "assistant"),
        }
    }
}

/// Conversation message in an agent's history.
///
/// Represents a single message in a conversation, including the role,
/// content, and timestamp. Used to maintain conversation context for agents.
///
/// # Examples
///
/// ```
/// use llmspell_core::traits::agent::{ConversationMessage, MessageRole};
///
/// // Create messages using convenience methods
/// let system_msg = ConversationMessage::system("You are a helpful assistant".to_string());
/// let user_msg = ConversationMessage::user("Hello!".to_string());
/// let assistant_msg = ConversationMessage::assistant("Hi! How can I help?".to_string());
///
/// assert_eq!(system_msg.role, MessageRole::System);
/// assert_eq!(user_msg.role, MessageRole::User);
/// assert_eq!(assistant_msg.role, MessageRole::Assistant);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationMessage {
    pub role: MessageRole,
    pub content: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl ConversationMessage {
    /// Create a new conversation message
    pub fn new(role: MessageRole, content: String) -> Self {
        Self {
            role,
            content,
            timestamp: chrono::Utc::now(),
        }
    }

    /// Create a system message
    pub fn system(content: String) -> Self {
        Self::new(MessageRole::System, content)
    }

    /// Create a user message
    pub fn user(content: String) -> Self {
        Self::new(MessageRole::User, content)
    }

    /// Create an assistant message
    pub fn assistant(content: String) -> Self {
        Self::new(MessageRole::Assistant, content)
    }
}

/// Configuration for LLM-powered agents.
///
/// Controls agent behavior including conversation management, generation parameters,
/// and system prompts. All fields are optional to allow partial configuration.
///
/// # Examples
///
/// ```
/// use llmspell_core::traits::agent::AgentConfig;
///
/// let config = AgentConfig {
///     max_conversation_length: Some(50),
///     system_prompt: Some("You are a research assistant".to_string()),
///     temperature: Some(0.7),
///     max_tokens: Some(1000),
/// };
///
/// // Or use default configuration
/// let default_config = AgentConfig::default();
/// assert_eq!(default_config.max_conversation_length, Some(100));
/// assert_eq!(default_config.temperature, Some(0.7));
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    /// Maximum number of messages to retain in conversation history
    pub max_conversation_length: Option<usize>,
    /// System prompt for the agent
    pub system_prompt: Option<String>,
    /// Temperature setting for LLM generation
    pub temperature: Option<f32>,
    /// Maximum tokens to generate
    pub max_tokens: Option<usize>,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            max_conversation_length: Some(100),
            system_prompt: None,
            temperature: Some(0.7),
            max_tokens: Some(2000),
        }
    }
}

/// Agent trait for LLM-powered components.
///
/// Extends `BaseAgent` with conversation management and LLM-specific functionality.
/// Agents maintain conversation history, handle message trimming, and provide
/// configuration for LLM generation parameters.
///
/// # Implementation Requirements
///
/// - Must maintain conversation history thread-safely
/// - Should implement conversation trimming to respect max length
/// - Configuration should be accessible but may be immutable
/// - All conversation operations should be atomic
///
/// # Examples
///
/// ```ignore
/// use llmspell_core::{
///     ComponentMetadata, Result,
///     traits::{
///         base_agent::{BaseAgent, AgentInput, AgentOutput, ExecutionContext},
///         agent::{Agent, AgentConfig, ConversationMessage, MessageRole}
///     }
/// };
/// use async_trait::async_trait;
///
/// struct MyLLMAgent {
///     metadata: ComponentMetadata,
///     config: AgentConfig,
///     conversation: Vec<ConversationMessage>,
/// }
///
/// #[async_trait]
/// impl Agent for MyLLMAgent {
///     fn config(&self) -> &AgentConfig {
///         &self.config
///     }
///     
///     async fn get_conversation(&self) -> Result<Vec<ConversationMessage>> {
///         Ok(self.conversation.clone())
///     }
///     
///     async fn add_message(&mut self, message: ConversationMessage) -> Result<()> {
///         self.conversation.push(message);
///         // Trim if needed
///         self.trim_conversation().await?;
///         Ok(())
///     }
///     
///     async fn clear_conversation(&mut self) -> Result<()> {
///         self.conversation.clear();
///         Ok(())
///     }
/// }
///
/// # impl BaseAgent for MyLLMAgent {
/// #     fn metadata(&self) -> &ComponentMetadata { &self.metadata }
/// #     async fn execute(&self, input: AgentInput, context: ExecutionContext) -> Result<AgentOutput> {
/// #         Ok(AgentOutput::text("Response".to_string()))
/// #     }
/// #     async fn validate_input(&self, input: &AgentInput) -> Result<()> { Ok(()) }
/// #     async fn handle_error(&self, error: llmspell_core::LLMSpellError) -> Result<AgentOutput> {
/// #         Ok(AgentOutput::text("Error".to_string()))
/// #     }
/// # }
/// ```
#[async_trait]
pub trait Agent: BaseAgent {
    /// Get agent configuration
    fn config(&self) -> &AgentConfig;

    /// Get conversation history
    async fn get_conversation(&self) -> Result<Vec<ConversationMessage>>;

    /// Add message to conversation
    async fn add_message(&mut self, message: ConversationMessage) -> Result<()>;

    /// Clear conversation history
    async fn clear_conversation(&mut self) -> Result<()>;

    /// Get the current conversation length
    async fn conversation_length(&self) -> Result<usize> {
        Ok(self.get_conversation().await?.len())
    }

    /// Trim conversation to configured max length
    async fn trim_conversation(&mut self) -> Result<()> {
        if let Some(max_len) = self.config().max_conversation_length {
            let current_len = self.conversation_length().await?;
            if current_len > max_len {
                // Keep system messages and trim oldest user/assistant messages
                let conversation = self.get_conversation().await?;
                let system_messages: Vec<_> = conversation
                    .iter()
                    .filter(|msg| matches!(msg.role, MessageRole::System))
                    .cloned()
                    .collect();

                let other_messages: Vec<_> = conversation
                    .into_iter()
                    .filter(|msg| !matches!(msg.role, MessageRole::System))
                    .collect();

                let skip_count = other_messages
                    .len()
                    .saturating_sub(max_len - system_messages.len());

                self.clear_conversation().await?;

                // Re-add system messages
                for msg in system_messages {
                    self.add_message(msg).await?;
                }

                // Add trimmed other messages
                for msg in other_messages.into_iter().skip(skip_count) {
                    self.add_message(msg).await?;
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{AgentInput, AgentOutput};
    use crate::ComponentMetadata;
    use crate::ExecutionContext;
    use std::collections::VecDeque;

    #[test]
    fn test_message_role_display() {
        assert_eq!(MessageRole::System.to_string(), "system");
        assert_eq!(MessageRole::User.to_string(), "user");
        assert_eq!(MessageRole::Assistant.to_string(), "assistant");
    }

    #[test]
    fn test_conversation_message_creation() {
        let content = "Test message".to_string();
        let msg = ConversationMessage::new(MessageRole::User, content.clone());

        assert_eq!(msg.role, MessageRole::User);
        assert_eq!(msg.content, content);

        // Test helper methods
        let system_msg = ConversationMessage::system("System prompt".to_string());
        assert_eq!(system_msg.role, MessageRole::System);

        let user_msg = ConversationMessage::user("User input".to_string());
        assert_eq!(user_msg.role, MessageRole::User);

        let assistant_msg = ConversationMessage::assistant("Assistant response".to_string());
        assert_eq!(assistant_msg.role, MessageRole::Assistant);
    }

    #[test]
    fn test_conversation_message_serialization() {
        let msg = ConversationMessage::user("Test".to_string());

        let json = serde_json::to_string(&msg).unwrap();
        let deserialized: ConversationMessage = serde_json::from_str(&json).unwrap();

        assert_eq!(msg.role, deserialized.role);
        assert_eq!(msg.content, deserialized.content);
    }

    #[test]
    fn test_agent_config_default() {
        let config = AgentConfig::default();

        assert_eq!(config.max_conversation_length, Some(100));
        assert_eq!(config.system_prompt, None);
        assert_eq!(config.temperature, Some(0.7));
        assert_eq!(config.max_tokens, Some(2000));
    }

    #[test]
    fn test_agent_config_serialization() {
        let config = AgentConfig {
            system_prompt: Some("You are a helpful assistant".to_string()),
            temperature: Some(0.9),
            ..Default::default()
        };

        let json = serde_json::to_string(&config).unwrap();
        let deserialized: AgentConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(config.system_prompt, deserialized.system_prompt);
        assert_eq!(config.temperature, deserialized.temperature);
    }

    // Mock implementation for testing
    struct MockLLMAgent {
        metadata: ComponentMetadata,
        config: AgentConfig,
        conversation: VecDeque<ConversationMessage>,
    }

    impl MockLLMAgent {
        fn new() -> Self {
            let config = AgentConfig {
                system_prompt: Some("You are a test assistant".to_string()),
                ..Default::default()
            };

            Self {
                metadata: ComponentMetadata::new(
                    "mock-llm-agent".to_string(),
                    "A mock LLM agent for testing".to_string(),
                ),
                config,
                conversation: VecDeque::new(),
            }
        }
    }

    #[async_trait]
    impl BaseAgent for MockLLMAgent {
        fn metadata(&self) -> &ComponentMetadata {
            &self.metadata
        }

        async fn execute(
            &self,
            input: AgentInput,
            _context: ExecutionContext,
        ) -> Result<AgentOutput> {
            // Simple echo response
            Ok(AgentOutput::text(format!("Response to: {}", input.text)))
        }

        async fn validate_input(&self, input: &AgentInput) -> Result<()> {
            if input.text.is_empty() {
                return Err(crate::LLMSpellError::Validation {
                    message: "Prompt cannot be empty".to_string(),
                    field: Some("prompt".to_string()),
                });
            }
            Ok(())
        }

        async fn handle_error(&self, error: crate::LLMSpellError) -> Result<AgentOutput> {
            Ok(AgentOutput::text(format!("Error: {}", error)))
        }
    }

    #[async_trait]
    impl Agent for MockLLMAgent {
        fn config(&self) -> &AgentConfig {
            &self.config
        }

        async fn get_conversation(&self) -> Result<Vec<ConversationMessage>> {
            Ok(self.conversation.iter().cloned().collect())
        }

        async fn add_message(&mut self, message: ConversationMessage) -> Result<()> {
            self.conversation.push_back(message);
            Ok(())
        }

        async fn clear_conversation(&mut self) -> Result<()> {
            self.conversation.clear();
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_agent_conversation_management() {
        let mut agent = MockLLMAgent::new();

        // Test empty conversation
        assert_eq!(agent.conversation_length().await.unwrap(), 0);

        // Add messages
        agent
            .add_message(ConversationMessage::system("System prompt".to_string()))
            .await
            .unwrap();
        agent
            .add_message(ConversationMessage::user("Hello".to_string()))
            .await
            .unwrap();
        agent
            .add_message(ConversationMessage::assistant("Hi there!".to_string()))
            .await
            .unwrap();

        assert_eq!(agent.conversation_length().await.unwrap(), 3);

        // Get conversation
        let conversation = agent.get_conversation().await.unwrap();
        assert_eq!(conversation.len(), 3);
        assert_eq!(conversation[0].role, MessageRole::System);
        assert_eq!(conversation[1].role, MessageRole::User);
        assert_eq!(conversation[2].role, MessageRole::Assistant);

        // Clear conversation
        agent.clear_conversation().await.unwrap();
        assert_eq!(agent.conversation_length().await.unwrap(), 0);
    }

    #[tokio::test]
    async fn test_agent_conversation_trimming() {
        let mut agent = MockLLMAgent::new();
        agent.config.max_conversation_length = Some(5);

        // Add system message
        agent
            .add_message(ConversationMessage::system("System prompt".to_string()))
            .await
            .unwrap();

        // Add more messages than max length
        for i in 0..6 {
            agent
                .add_message(ConversationMessage::user(format!("Message {}", i)))
                .await
                .unwrap();
            agent
                .add_message(ConversationMessage::assistant(format!("Response {}", i)))
                .await
                .unwrap();
        }

        // Should have 13 messages (1 system + 12 others)
        assert_eq!(agent.conversation_length().await.unwrap(), 13);

        // Trim conversation
        agent.trim_conversation().await.unwrap();

        // Should keep system message and latest 4 messages
        assert_eq!(agent.conversation_length().await.unwrap(), 5);

        let conversation = agent.get_conversation().await.unwrap();
        assert_eq!(conversation[0].role, MessageRole::System);
        assert!(conversation[1].content.contains("Message 4")); // Should keep latest messages
    }

    #[tokio::test]
    async fn test_agent_config_usage() {
        let agent = MockLLMAgent::new();

        // Test config access
        let config = agent.config();
        assert_eq!(
            config.system_prompt,
            Some("You are a test assistant".to_string())
        );
        assert_eq!(config.temperature, Some(0.7));
        assert_eq!(config.max_tokens, Some(2000));
    }
}
