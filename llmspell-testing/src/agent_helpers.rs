//! ABOUTME: Test helpers for agent creation and testing
//! ABOUTME: Provides utilities for creating test agents with various providers

//! Agent testing helpers.
//!
//! This module provides common test utilities for testing agents
//! including provider-specific agent creation and mock agent setup.
//!
//! # Examples
//!
//! ```rust,no_run
//! use llmspell_testing::agent_helpers::{
//!     create_mock_provider_agent,
//!     create_test_agent_config,
//!     AgentTestBuilder,
//! };
//!
//! # async fn test_example() {
//! // Create a mock provider agent
//! let agent = create_mock_provider_agent("openai", "gpt-4").await;
//!
//! // Use the builder for more control
//! let agent = AgentTestBuilder::new("test-agent")
//!     .with_provider("anthropic")
//!     .with_model("claude-3")
//!     .with_system_prompt("You are a test assistant")
//!     .build()
//!     .await
//!     .unwrap();
//! # }
//! ```

use llmspell_core::{
    execution_context::ExecutionContext,
    traits::agent::{Agent, AgentConfig, ConversationMessage},
    types::{AgentInput, AgentOutput},
    ComponentMetadata, LLMSpellError,
};
use serde_json::json;

/// Test agent implementation that simulates provider behavior
pub struct TestProviderAgent {
    metadata: ComponentMetadata,
    config: AgentConfig,
    _provider: String,
    _model: String,
    responses: Vec<String>,
    response_index: std::sync::Mutex<usize>,
}

impl TestProviderAgent {
    /// Create a new test provider agent
    pub fn new(provider: &str, model: &str) -> Self {
        let metadata = ComponentMetadata::new(
            format!("{}-{}-test", provider, model),
            format!("Test {} agent with model {}", provider, model),
        );

        Self {
            metadata,
            config: AgentConfig::default(),
            _provider: provider.to_string(),
            _model: model.to_string(),
            responses: vec!["Test response".to_string()],
            response_index: std::sync::Mutex::new(0),
        }
    }

    /// Add a response to the agent's response queue
    pub fn add_response(&mut self, response: String) {
        // Clear default response if this is the first custom response
        if self.responses.len() == 1 && self.responses[0] == "Test response" {
            self.responses.clear();
        }
        self.responses.push(response);
    }

    /// Set the agent configuration
    pub fn with_config(mut self, config: AgentConfig) -> Self {
        self.config = config;
        self
    }
}

#[async_trait::async_trait]
impl llmspell_core::traits::base_agent::BaseAgent for TestProviderAgent {
    fn metadata(&self) -> &ComponentMetadata {
        &self.metadata
    }

    async fn execute_impl(
        &self,
        _input: AgentInput,
        _context: ExecutionContext,
    ) -> Result<AgentOutput, LLMSpellError> {
        let mut index = self.response_index.lock().unwrap();
        let response = if *index < self.responses.len() {
            self.responses[*index].clone()
        } else {
            // Cycle through responses
            self.responses[*index % self.responses.len()].clone()
        };
        *index += 1;

        Ok(AgentOutput::text(response))
    }

    async fn validate_input(&self, _input: &AgentInput) -> Result<(), LLMSpellError> {
        // Test agent accepts all inputs
        Ok(())
    }

    async fn handle_error(&self, error: LLMSpellError) -> Result<AgentOutput, LLMSpellError> {
        Ok(AgentOutput::text(format!("Error handled: {}", error)))
    }
}

#[async_trait::async_trait]
impl Agent for TestProviderAgent {
    fn config(&self) -> &AgentConfig {
        &self.config
    }

    async fn get_conversation(&self) -> Result<Vec<ConversationMessage>, LLMSpellError> {
        // Return empty conversation for test agent
        Ok(Vec::new())
    }

    async fn add_message(&self, _message: ConversationMessage) -> Result<(), LLMSpellError> {
        // Test agent doesn't store messages
        Ok(())
    }

    async fn clear_conversation(&self) -> Result<(), LLMSpellError> {
        // Test agent has no conversation to clear
        Ok(())
    }
}

/// Builder for creating test agents
pub struct AgentTestBuilder {
    name: String,
    provider: Option<String>,
    model: Option<String>,
    config: AgentConfig,
    responses: Vec<String>,
    metadata_overrides: Option<ComponentMetadata>,
}

impl AgentTestBuilder {
    /// Create a new agent test builder
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            provider: None,
            model: None,
            config: AgentConfig::default(),
            responses: Vec::new(),
            metadata_overrides: None,
        }
    }

    /// Set the provider
    pub fn with_provider(mut self, provider: &str) -> Self {
        self.provider = Some(provider.to_string());
        self
    }

    /// Set the model
    pub fn with_model(mut self, model: &str) -> Self {
        self.model = Some(model.to_string());
        self
    }

    /// Set the system prompt
    pub fn with_system_prompt(mut self, prompt: &str) -> Self {
        self.config.system_prompt = Some(prompt.to_string());
        self
    }

    /// Set the temperature
    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.config.temperature = Some(temperature);
        self
    }

    /// Set max tokens
    pub fn with_max_tokens(mut self, max_tokens: usize) -> Self {
        self.config.max_tokens = Some(max_tokens);
        self
    }

    /// Add a response
    pub fn with_response(mut self, response: &str) -> Self {
        self.responses.push(response.to_string());
        self
    }

    /// Add multiple responses
    pub fn with_responses(mut self, responses: Vec<String>) -> Self {
        self.responses.extend(responses);
        self
    }

    /// Override metadata
    pub fn with_metadata(mut self, metadata: ComponentMetadata) -> Self {
        self.metadata_overrides = Some(metadata);
        self
    }

    /// Build the test agent
    pub async fn build(self) -> Result<TestProviderAgent, LLMSpellError> {
        let provider = self.provider.unwrap_or_else(|| "mock".to_string());
        let model = self.model.unwrap_or_else(|| "test-model".to_string());

        let mut agent = TestProviderAgent::new(&provider, &model);
        agent.config = self.config;

        if let Some(metadata) = self.metadata_overrides {
            agent.metadata = metadata;
        } else {
            agent.metadata =
                ComponentMetadata::new(self.name.clone(), format!("Test agent: {}", self.name));
        }

        if self.responses.is_empty() {
            agent.responses = vec![format!("Response from {} agent", self.name)];
        } else {
            agent.responses = self.responses;
        }

        Ok(agent)
    }
}

/// Create a mock provider agent with default settings
pub async fn create_mock_provider_agent(provider: &str, model: &str) -> TestProviderAgent {
    TestProviderAgent::new(provider, model)
}

/// Create a test agent configuration
pub fn create_test_agent_config() -> AgentConfig {
    AgentConfig {
        max_conversation_length: Some(10),
        system_prompt: Some("You are a helpful test assistant.".to_string()),
        temperature: Some(0.7),
        max_tokens: Some(1000),
    }
}

/// Provider-specific agent creators
pub mod providers {
    use super::*;

    /// Create a test OpenAI agent
    pub async fn create_openai_agent(model: &str) -> Result<TestProviderAgent, LLMSpellError> {
        AgentTestBuilder::new("openai-agent")
            .with_provider("openai")
            .with_model(model)
            .with_system_prompt("You are a helpful OpenAI assistant.")
            .with_temperature(0.7)
            .with_max_tokens(2048usize)
            .build()
            .await
    }

    /// Create a test Anthropic agent
    pub async fn create_anthropic_agent(model: &str) -> Result<TestProviderAgent, LLMSpellError> {
        AgentTestBuilder::new("anthropic-agent")
            .with_provider("anthropic")
            .with_model(model)
            .with_system_prompt("You are Claude, a helpful AI assistant.")
            .with_temperature(0.5)
            .with_max_tokens(4096usize)
            .build()
            .await
    }

    /// Create a test Gemini agent
    pub async fn create_gemini_agent(model: &str) -> Result<TestProviderAgent, LLMSpellError> {
        AgentTestBuilder::new("gemini-agent")
            .with_provider("gemini")
            .with_model(model)
            .with_system_prompt("You are Gemini, a helpful AI assistant.")
            .with_temperature(0.6)
            .with_max_tokens(2048usize)
            .build()
            .await
    }

    /// Create a test local/Ollama agent
    pub async fn create_local_agent(model: &str) -> Result<TestProviderAgent, LLMSpellError> {
        AgentTestBuilder::new("local-agent")
            .with_provider("ollama")
            .with_model(model)
            .with_system_prompt("You are a local AI assistant.")
            .with_temperature(0.8)
            .with_max_tokens(1024usize)
            .build()
            .await
    }
}

/// Test conversation builders
pub mod conversations {
    use super::*;
    use llmspell_core::traits::agent::MessageRole;

    /// Build a simple conversation
    pub fn simple_conversation() -> Vec<ConversationMessage> {
        vec![
            ConversationMessage::system("You are a helpful assistant.".to_string()),
            ConversationMessage::user("Hello!".to_string()),
            ConversationMessage::assistant("Hello! How can I help you today?".to_string()),
        ]
    }

    /// Build a conversation with context
    pub fn conversation_with_context(context: &str) -> Vec<ConversationMessage> {
        vec![
            ConversationMessage::system(format!(
                "You are a helpful assistant. Context: {}",
                context
            )),
            ConversationMessage::user("What's the context?".to_string()),
            ConversationMessage::assistant(format!("The context is: {}", context)),
        ]
    }

    /// Build a multi-turn conversation
    pub fn multi_turn_conversation(turns: usize) -> Vec<ConversationMessage> {
        let mut messages = vec![ConversationMessage::system(
            "You are a helpful assistant.".to_string(),
        )];

        for i in 0..turns {
            messages.push(ConversationMessage::user(format!("Question {}", i + 1)));
            messages.push(ConversationMessage::assistant(format!(
                "Answer to question {}",
                i + 1
            )));
        }

        messages
    }

    /// Build a conversation from raw messages
    pub fn from_raw_messages(messages: Vec<(MessageRole, String)>) -> Vec<ConversationMessage> {
        messages
            .into_iter()
            .map(|(role, content)| ConversationMessage {
                role,
                content,
                timestamp: chrono::Utc::now(),
            })
            .collect()
    }
}

/// Agent test scenarios
pub mod scenarios {
    use super::*;

    /// Test error handling scenario
    pub async fn test_agent_error_handling(
        agent: &impl Agent,
    ) -> Result<Vec<AgentOutput>, LLMSpellError> {
        let test_cases = vec![
            AgentInput::text(""),                // Empty input
            AgentInput::text("a".repeat(10000)), // Very long input
            create_malformed_input(),            // Malformed input
        ];

        let mut results = Vec::new();
        for input in test_cases {
            match agent.execute(input, ExecutionContext::default()).await {
                Ok(output) => results.push(output),
                Err(_) => results.push(AgentOutput::text("Error handled")),
            }
        }

        Ok(results)
    }

    /// Test conversation memory
    pub async fn test_conversation_memory(agent: &impl Agent) -> Result<bool, LLMSpellError> {
        let _history = conversations::simple_conversation();
        let input = AgentInput::text("What did we just talk about?");

        // For test purposes, just check if agent can execute
        let output = agent.execute(input, ExecutionContext::default()).await?;

        // Check if we got a response
        Ok(!output.text.is_empty())
    }

    /// Create malformed input for testing
    fn create_malformed_input() -> AgentInput {
        AgentInput::text("Test")
            .with_parameter("invalid_json", json!(f64::NAN))
            .with_parameter("circular_ref", json!(null))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use llmspell_core::traits::base_agent::BaseAgent;

    #[tokio::test]
    async fn test_agent_builder() {
        let agent = AgentTestBuilder::new("test-agent")
            .with_provider("openai")
            .with_model("gpt-4")
            .with_system_prompt("Test prompt")
            .with_response("Test response")
            .build()
            .await
            .unwrap();

        assert_eq!(agent._provider, "openai");
        assert_eq!(agent._model, "gpt-4");
        assert_eq!(agent.config.system_prompt, Some("Test prompt".to_string()));
    }

    #[tokio::test]
    async fn test_provider_agents() {
        let openai = providers::create_openai_agent("gpt-4").await.unwrap();
        assert_eq!(openai._provider, "openai");

        let anthropic = providers::create_anthropic_agent("claude-3").await.unwrap();
        assert_eq!(anthropic._provider, "anthropic");
    }

    #[tokio::test]
    async fn test_conversation_builders() {
        let simple = conversations::simple_conversation();
        assert_eq!(simple.len(), 3);

        let multi_turn = conversations::multi_turn_conversation(5);
        assert_eq!(multi_turn.len(), 11); // 1 system + 5 * (user + assistant)
    }

    #[tokio::test]
    async fn test_agent_responses() {
        let mut agent = TestProviderAgent::new("test", "model");
        agent.add_response("Response 1".to_string());
        agent.add_response("Response 2".to_string());

        let input = AgentInput::text("Test");
        let context = ExecutionContext::default();

        let output1 = agent.execute(input.clone(), context.clone()).await.unwrap();
        let output2 = agent.execute(input.clone(), context.clone()).await.unwrap();

        assert_eq!(output1.text, "Response 1");
        assert_eq!(output2.text, "Response 2");
    }
}
