//! ABOUTME: Basic agent implementation
//! ABOUTME: Simple agent with minimal functionality for testing and examples

use crate::factory::AgentConfig;
use anyhow::Result;
use async_trait::async_trait;
use llmspell_core::{
    traits::{
        agent::{Agent, AgentConfig as CoreAgentConfig, ConversationMessage},
        base_agent::BaseAgent,
    },
    types::{AgentInput, AgentOutput, ExecutionContext},
    ComponentMetadata, LLMSpellError,
};
use std::sync::{Arc, Mutex};

/// Basic agent implementation
pub struct BasicAgent {
    metadata: ComponentMetadata,
    config: AgentConfig,
    core_config: CoreAgentConfig,
    conversation: Arc<Mutex<Vec<ConversationMessage>>>,
}

impl BasicAgent {
    /// Create a new basic agent
    pub fn new(config: AgentConfig) -> Result<Self> {
        let metadata = ComponentMetadata::new(config.name.clone(), config.description.clone());

        let core_config = CoreAgentConfig {
            max_conversation_length: Some(100),
            system_prompt: Some(format!("You are {}, a basic agent.", config.name)),
            temperature: None,
            max_tokens: None,
        };

        Ok(Self {
            metadata,
            config,
            core_config,
            conversation: Arc::new(Mutex::new(Vec::new())),
        })
    }

    /// Get configuration
    pub fn get_config(&self) -> &AgentConfig {
        &self.config
    }
}

#[async_trait]
impl BaseAgent for BasicAgent {
    fn metadata(&self) -> &ComponentMetadata {
        &self.metadata
    }

    async fn execute(
        &self,
        input: AgentInput,
        _context: ExecutionContext,
    ) -> Result<AgentOutput, LLMSpellError> {
        // Basic implementation: echo the input
        let response = format!(
            "BasicAgent '{}' received: {}",
            self.metadata.name, input.text
        );

        // Add to conversation history
        if let Ok(mut conv) = self.conversation.lock() {
            conv.push(ConversationMessage::user(input.text.clone()));
            conv.push(ConversationMessage::assistant(response.clone()));
        }

        Ok(AgentOutput::text(response))
    }

    async fn validate_input(&self, input: &AgentInput) -> Result<(), LLMSpellError> {
        if input.text.is_empty() {
            return Err(LLMSpellError::Validation {
                message: "Input text cannot be empty".to_string(),
                field: Some("text".to_string()),
            });
        }

        // Check resource limits
        if input.text.len() > (self.config.resource_limits.max_memory_mb as usize * 1024) {
            return Err(LLMSpellError::Validation {
                message: "Input text exceeds memory limit".to_string(),
                field: Some("text".to_string()),
            });
        }

        Ok(())
    }

    async fn handle_error(&self, error: LLMSpellError) -> Result<AgentOutput, LLMSpellError> {
        Ok(AgentOutput::text(format!(
            "BasicAgent '{}' encountered error: {}",
            self.metadata.name, error
        )))
    }
}

#[async_trait]
impl Agent for BasicAgent {
    fn config(&self) -> &CoreAgentConfig {
        &self.core_config
    }

    async fn get_conversation(&self) -> Result<Vec<ConversationMessage>, LLMSpellError> {
        self.conversation
            .lock()
            .map(|conv| conv.clone())
            .map_err(|_| LLMSpellError::Component {
                message: "Failed to lock conversation".to_string(),
                source: None,
            })
    }

    async fn add_message(&mut self, message: ConversationMessage) -> Result<(), LLMSpellError> {
        self.conversation
            .lock()
            .map(|mut conv| conv.push(message))
            .map_err(|_| LLMSpellError::Component {
                message: "Failed to lock conversation".to_string(),
                source: None,
            })
    }

    async fn clear_conversation(&mut self) -> Result<(), LLMSpellError> {
        self.conversation
            .lock()
            .map(|mut conv| conv.clear())
            .map_err(|_| LLMSpellError::Component {
                message: "Failed to lock conversation".to_string(),
                source: None,
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::builder::AgentBuilder;

    #[tokio::test]
    async fn test_basic_agent_creation() {
        let config = AgentBuilder::basic("test-agent")
            .description("Test agent")
            .build()
            .unwrap();

        let agent = BasicAgent::new(config).unwrap();
        assert_eq!(agent.metadata().name, "test-agent");
    }

    #[tokio::test]
    async fn test_basic_agent_execution() {
        let config = AgentBuilder::basic("test-agent").build().unwrap();
        let agent = BasicAgent::new(config).unwrap();

        let input = AgentInput::text("Hello, agent!");
        let context = ExecutionContext::default();

        let output = agent.execute(input, context).await.unwrap();
        assert!(output
            .text
            .contains("BasicAgent 'test-agent' received: Hello, agent!"));
    }

    #[tokio::test]
    async fn test_basic_agent_conversation() {
        let config = AgentBuilder::basic("test-agent").build().unwrap();
        let mut agent = BasicAgent::new(config).unwrap();

        // Add messages
        agent
            .add_message(ConversationMessage::user("Hello".to_string()))
            .await
            .unwrap();
        agent
            .add_message(ConversationMessage::assistant("Hi there!".to_string()))
            .await
            .unwrap();

        // Get conversation
        let conv = agent.get_conversation().await.unwrap();
        assert_eq!(conv.len(), 2);

        // Clear conversation
        agent.clear_conversation().await.unwrap();
        let conv = agent.get_conversation().await.unwrap();
        assert_eq!(conv.len(), 0);
    }

    #[tokio::test]
    async fn test_basic_agent_validation() {
        let config = AgentBuilder::basic("test-agent").build().unwrap();
        let agent = BasicAgent::new(config).unwrap();

        // Empty input should fail
        let empty_input = AgentInput::text("");
        let result = agent.validate_input(&empty_input).await;
        assert!(result.is_err());

        // Valid input should pass
        let valid_input = AgentInput::text("Valid input");
        let result = agent.validate_input(&valid_input).await;
        assert!(result.is_ok());
    }
}
