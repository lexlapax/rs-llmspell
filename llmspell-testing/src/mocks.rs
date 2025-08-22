//! ABOUTME: Mock implementations of core traits for testing
//! ABOUTME: Provides configurable mocks for BaseAgent, Agent, Tool, and Workflow

//! Mock implementations for testing.
//!
//! This module provides mock implementations of the core traits
//! using mockall. These mocks can be configured with expectations
//! for unit testing.
//!
//! # Examples
//!
//! ```rust,no_run
//! use llmspell_testing::mocks::MockBaseAgent;
//! use llmspell_core::{
//!     ExecutionContext,
//!     traits::base_agent::BaseAgent,
//!     types::{AgentInput, AgentOutput}
//! };
//!
//! # async fn test_example() {
//! let mut mock = MockBaseAgent::new();
//! mock.expect_execute_impl()
//!     .times(1)
//!     .returning(|input, _| {
//!         Ok(AgentOutput::text(format!("Processed: {}", input.text)))
//!     });
//!
//! let input = AgentInput::text("test");
//! let context = ExecutionContext::new();
//! let result = mock.execute(input, context).await.unwrap();
//! assert_eq!(result.text, "Processed: test");
//! # }
//! ```

use async_trait::async_trait;
use llmspell_core::{
    execution_context::ExecutionContext,
    traits::{
        agent::{Agent, AgentConfig, ConversationMessage},
        base_agent::BaseAgent,
        tool::{SecurityLevel, Tool, ToolCategory, ToolSchema},
        workflow::{
            Config as WorkflowConfig, Status as WorkflowStatus, StepResult, Workflow, WorkflowStep,
        },
    },
    types::{AgentInput, AgentOutput, AgentStream, MediaType},
    ComponentId, ComponentMetadata, LLMSpellError, Result,
};
use mockall::*;

// Mock for BaseAgent trait
mock! {
    pub BaseAgent {}

    #[async_trait]
    impl BaseAgent for BaseAgent {
        fn metadata(&self) -> &ComponentMetadata;
        async fn execute_impl(&self, input: AgentInput, context: ExecutionContext) -> Result<AgentOutput>;
        async fn validate_input(&self, input: &AgentInput) -> Result<()>;
        async fn handle_error(&self, error: LLMSpellError) -> Result<AgentOutput>;
        async fn stream_execute(&self, input: AgentInput, context: ExecutionContext) -> Result<AgentStream>;
        fn supports_streaming(&self) -> bool;
        fn supports_multimodal(&self) -> bool;
        fn supported_media_types(&self) -> Vec<MediaType>;
    }
}

// Mock for Agent trait
mock! {
    pub Agent {}

    #[async_trait]
    impl BaseAgent for Agent {
        fn metadata(&self) -> &ComponentMetadata;
        async fn execute_impl(&self, input: AgentInput, context: ExecutionContext) -> Result<AgentOutput>;
        async fn validate_input(&self, input: &AgentInput) -> Result<()>;
        async fn handle_error(&self, error: LLMSpellError) -> Result<AgentOutput>;
        async fn stream_execute(&self, input: AgentInput, context: ExecutionContext) -> Result<AgentStream>;
        fn supports_streaming(&self) -> bool;
        fn supports_multimodal(&self) -> bool;
        fn supported_media_types(&self) -> Vec<MediaType>;
    }

    #[async_trait]
    impl Agent for Agent {
        fn config(&self) -> &AgentConfig;
        async fn get_conversation(&self) -> Result<Vec<ConversationMessage>>;
        async fn add_message(&self, message: ConversationMessage) -> Result<()>;
        async fn clear_conversation(&self) -> Result<()>;
        async fn conversation_length(&self) -> Result<usize>;
        async fn trim_conversation(&self) -> Result<()>;
    }
}

// Mock for Tool trait
mock! {
    pub Tool {}

    #[async_trait]
    impl BaseAgent for Tool {
        fn metadata(&self) -> &ComponentMetadata;
        async fn execute_impl(&self, input: AgentInput, context: ExecutionContext) -> Result<AgentOutput>;
        async fn validate_input(&self, input: &AgentInput) -> Result<()>;
        async fn handle_error(&self, error: LLMSpellError) -> Result<AgentOutput>;
        async fn stream_execute(&self, input: AgentInput, context: ExecutionContext) -> Result<AgentStream>;
        fn supports_streaming(&self) -> bool;
        fn supports_multimodal(&self) -> bool;
        fn supported_media_types(&self) -> Vec<MediaType>;
    }

    #[async_trait]
    impl Tool for Tool {
        fn schema(&self) -> ToolSchema;
        fn category(&self) -> ToolCategory;
        fn security_level(&self) -> SecurityLevel;
        async fn validate_parameters(&self, params: &serde_json::Value) -> Result<()>;
    }
}

// Mock for Workflow trait
mock! {
    pub Workflow {}

    #[async_trait]
    impl BaseAgent for Workflow {
        fn metadata(&self) -> &ComponentMetadata;
        async fn execute_impl(&self, input: AgentInput, context: ExecutionContext) -> Result<AgentOutput>;
        async fn validate_input(&self, input: &AgentInput) -> Result<()>;
        async fn handle_error(&self, error: LLMSpellError) -> Result<AgentOutput>;
        async fn stream_execute(&self, input: AgentInput, context: ExecutionContext) -> Result<AgentStream>;
        fn supports_streaming(&self) -> bool;
        fn supports_multimodal(&self) -> bool;
        fn supported_media_types(&self) -> Vec<MediaType>;
    }

    #[async_trait]
    impl Workflow for Workflow {
        fn config(&self) -> &WorkflowConfig;
        async fn add_step(&self, step: WorkflowStep) -> Result<()>;
        async fn remove_step(&self, step_id: ComponentId) -> Result<()>;
        async fn get_steps(&self) -> Result<Vec<WorkflowStep>>;
        async fn plan_execution(&self) -> Result<Vec<WorkflowStep>>;
        async fn status(&self) -> Result<WorkflowStatus>;
        async fn get_results(&self) -> Result<Vec<StepResult>>;
        async fn get_step_result(&self, step_id: ComponentId) -> Result<Option<StepResult>>;
        async fn validate(&self) -> Result<()>;
    }
}

/// Test helper to create a simple mock BaseAgent
pub fn create_simple_mock_agent() -> MockBaseAgent {
    let mut mock = MockBaseAgent::new();

    // Set default expectations
    mock.expect_validate_input().returning(|_| Ok(()));

    mock
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio;
    #[tokio::test]
    async fn test_mock_base_agent() {
        let mut mock = MockBaseAgent::new();

        // Set up metadata expectation (required by execute() for event emission)
        let metadata = ComponentMetadata::new("test-mock".to_string(), "Test Mock".to_string());
        mock.expect_metadata().return_const(metadata);

        mock.expect_execute_impl()
            .times(1)
            .returning(|input, _| Ok(AgentOutput::text(format!("Echo: {}", input.text))));

        let input = AgentInput::text("Hello");
        let context = ExecutionContext::with_conversation("test-session".to_string());

        let result = mock.execute(input, context).await.unwrap();
        assert_eq!(result.text, "Echo: Hello");
    }
    #[tokio::test]
    async fn test_simple_mock_helper() {
        let mock = create_simple_mock_agent();

        // Validate input should succeed
        let input = AgentInput::text("test");
        assert!(mock.validate_input(&input).await.is_ok());
    }
}
