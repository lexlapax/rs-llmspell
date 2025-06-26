//! ABOUTME: BaseAgent trait - foundation for all components
//! ABOUTME: Provides core functionality for agents, tools, and workflows

use crate::{ComponentMetadata, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Input for agent execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentInput {
    pub prompt: String,
    pub context: HashMap<String, serde_json::Value>,
}

impl AgentInput {
    /// Create new AgentInput with a prompt
    pub fn new(prompt: String) -> Self {
        Self {
            prompt,
            context: HashMap::new(),
        }
    }
    
    /// Add context value
    pub fn with_context(mut self, key: String, value: serde_json::Value) -> Self {
        self.context.insert(key, value);
        self
    }
    
    /// Get context value
    pub fn get_context(&self, key: &str) -> Option<&serde_json::Value> {
        self.context.get(key)
    }
}

/// Output from agent execution  
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentOutput {
    pub content: String,
    pub metadata: HashMap<String, serde_json::Value>,
}

impl AgentOutput {
    /// Create new AgentOutput with content
    pub fn new(content: String) -> Self {
        Self {
            content,
            metadata: HashMap::new(),
        }
    }
    
    /// Add metadata value
    pub fn with_metadata(mut self, key: String, value: serde_json::Value) -> Self {
        self.metadata.insert(key, value);
        self
    }
    
    /// Get metadata value
    pub fn get_metadata(&self, key: &str) -> Option<&serde_json::Value> {
        self.metadata.get(key)
    }
}

/// Execution context for components
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionContext {
    pub session_id: String,
    pub user_id: Option<String>,
    pub environment: HashMap<String, String>,
}

impl ExecutionContext {
    /// Create new ExecutionContext with session ID
    pub fn new(session_id: String) -> Self {
        Self {
            session_id,
            user_id: None,
            environment: HashMap::new(),
        }
    }
    
    /// Set user ID
    pub fn with_user_id(mut self, user_id: String) -> Self {
        self.user_id = Some(user_id);
        self
    }
    
    /// Add environment variable
    pub fn with_env(mut self, key: String, value: String) -> Self {
        self.environment.insert(key, value);
        self
    }
    
    /// Get environment variable
    pub fn get_env(&self, key: &str) -> Option<&String> {
        self.environment.get(key)
    }
}

/// Base trait for all components in the system
#[async_trait]
pub trait BaseAgent: Send + Sync {
    /// Get component metadata
    fn metadata(&self) -> &ComponentMetadata;
    
    /// Execute the component with given input
    async fn execute(&self, input: AgentInput, context: ExecutionContext) -> Result<AgentOutput>;
    
    /// Validate input before execution
    async fn validate_input(&self, input: &AgentInput) -> Result<()>;
    
    /// Handle execution errors
    async fn handle_error(&self, error: crate::LLMSpellError) -> Result<AgentOutput>;
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_agent_input_creation() {
        let prompt = "Test prompt".to_string();
        let input = AgentInput::new(prompt.clone());
        
        assert_eq!(input.prompt, prompt);
        assert!(input.context.is_empty());
    }
    
    #[test]
    fn test_agent_input_with_context() {
        let input = AgentInput::new("test".to_string())
            .with_context("key1".to_string(), serde_json::Value::String("value1".to_string()))
            .with_context("key2".to_string(), serde_json::Value::Number(42.into()));
        
        assert_eq!(input.context.len(), 2);
        assert_eq!(input.get_context("key1"), Some(&serde_json::Value::String("value1".to_string())));
        assert_eq!(input.get_context("key2"), Some(&serde_json::Value::Number(42.into())));
        assert_eq!(input.get_context("nonexistent"), None);
    }
    
    #[test]
    fn test_agent_input_serialization() {
        let input = AgentInput::new("test".to_string())
            .with_context("key".to_string(), serde_json::Value::String("value".to_string()));
        
        let json = serde_json::to_string(&input).unwrap();
        let deserialized: AgentInput = serde_json::from_str(&json).unwrap();
        
        assert_eq!(input.prompt, deserialized.prompt);
        assert_eq!(input.context, deserialized.context);
    }
    
    #[test]
    fn test_agent_output_creation() {
        let content = "Test output".to_string();
        let output = AgentOutput::new(content.clone());
        
        assert_eq!(output.content, content);
        assert!(output.metadata.is_empty());
    }
    
    #[test]
    fn test_agent_output_with_metadata() {
        let output = AgentOutput::new("test".to_string())
            .with_metadata("confidence".to_string(), serde_json::Value::Number(95.into()))
            .with_metadata("source".to_string(), serde_json::Value::String("model".to_string()));
        
        assert_eq!(output.metadata.len(), 2);
        assert_eq!(output.get_metadata("confidence"), Some(&serde_json::Value::Number(95.into())));
        assert_eq!(output.get_metadata("source"), Some(&serde_json::Value::String("model".to_string())));
        assert_eq!(output.get_metadata("nonexistent"), None);
    }
    
    #[test]
    fn test_agent_output_serialization() {
        let output = AgentOutput::new("test".to_string())
            .with_metadata("key".to_string(), serde_json::Value::String("value".to_string()));
        
        let json = serde_json::to_string(&output).unwrap();
        let deserialized: AgentOutput = serde_json::from_str(&json).unwrap();
        
        assert_eq!(output.content, deserialized.content);
        assert_eq!(output.metadata, deserialized.metadata);
    }
    
    #[test]
    fn test_execution_context_creation() {
        let session_id = "session-123".to_string();
        let context = ExecutionContext::new(session_id.clone());
        
        assert_eq!(context.session_id, session_id);
        assert_eq!(context.user_id, None);
        assert!(context.environment.is_empty());
    }
    
    #[test]
    fn test_execution_context_with_user() {
        let user_id = "user-456".to_string();
        let context = ExecutionContext::new("session".to_string())
            .with_user_id(user_id.clone());
        
        assert_eq!(context.user_id, Some(user_id));
    }
    
    #[test]
    fn test_execution_context_with_env() {
        let context = ExecutionContext::new("session".to_string())
            .with_env("VAR1".to_string(), "value1".to_string())
            .with_env("VAR2".to_string(), "value2".to_string());
        
        assert_eq!(context.environment.len(), 2);
        assert_eq!(context.get_env("VAR1"), Some(&"value1".to_string()));
        assert_eq!(context.get_env("VAR2"), Some(&"value2".to_string()));
        assert_eq!(context.get_env("NONEXISTENT"), None);
    }
    
    #[test]
    fn test_execution_context_serialization() {
        let context = ExecutionContext::new("session".to_string())
            .with_user_id("user".to_string())
            .with_env("KEY".to_string(), "value".to_string());
        
        let json = serde_json::to_string(&context).unwrap();
        let deserialized: ExecutionContext = serde_json::from_str(&json).unwrap();
        
        assert_eq!(context.session_id, deserialized.session_id);
        assert_eq!(context.user_id, deserialized.user_id);
        assert_eq!(context.environment, deserialized.environment);
    }
    
    // Mock implementation for testing
    struct MockAgent {
        metadata: ComponentMetadata,
    }
    
    impl MockAgent {
        fn new() -> Self {
            Self {
                metadata: ComponentMetadata::new(
                    "mock-agent".to_string(),
                    "A mock agent for testing".to_string(),
                ),
            }
        }
    }
    
    #[async_trait]
    impl BaseAgent for MockAgent {
        fn metadata(&self) -> &ComponentMetadata {
            &self.metadata
        }
        
        async fn execute(&self, input: AgentInput, _context: ExecutionContext) -> Result<AgentOutput> {
            Ok(AgentOutput::new(format!("Processed: {}", input.prompt)))
        }
        
        async fn validate_input(&self, input: &AgentInput) -> Result<()> {
            if input.prompt.is_empty() {
                return Err(crate::LLMSpellError::Validation {
                    message: "Prompt cannot be empty".to_string(),
                    field: Some("prompt".to_string()),
                });
            }
            Ok(())
        }
        
        async fn handle_error(&self, error: crate::LLMSpellError) -> Result<AgentOutput> {
            Ok(AgentOutput::new(format!("Error handled: {}", error)))
        }
    }
    
    #[tokio::test]
    async fn test_base_agent_implementation() {
        let agent = MockAgent::new();
        
        // Test metadata access
        let metadata = agent.metadata();
        assert_eq!(metadata.name, "mock-agent");
        assert_eq!(metadata.description, "A mock agent for testing");
        
        // Test successful execution
        let input = AgentInput::new("test prompt".to_string());
        let context = ExecutionContext::new("session".to_string());
        let result = agent.execute(input, context).await.unwrap();
        assert_eq!(result.content, "Processed: test prompt");
    }
    
    #[tokio::test]
    async fn test_base_agent_validation() {
        let agent = MockAgent::new();
        
        // Test valid input
        let valid_input = AgentInput::new("valid prompt".to_string());
        assert!(agent.validate_input(&valid_input).await.is_ok());
        
        // Test invalid input
        let invalid_input = AgentInput::new("".to_string());
        let validation_result = agent.validate_input(&invalid_input).await;
        assert!(validation_result.is_err());
        
        if let Err(crate::LLMSpellError::Validation { message, .. }) = validation_result {
            assert_eq!(message, "Prompt cannot be empty");
        } else {
            panic!("Expected validation error");
        }
    }
    
    #[tokio::test]
    async fn test_base_agent_error_handling() {
        let agent = MockAgent::new();
        
        let error = crate::LLMSpellError::Component {
            message: "Test error".to_string(),
            source: None,
        };
        
        let result = agent.handle_error(error).await.unwrap();
        assert!(result.content.contains("Error handled"));
        assert!(result.content.contains("Test error"));
    }
}