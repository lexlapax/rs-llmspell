//! ABOUTME: Common test fixtures and data for consistent testing
//! ABOUTME: Provides pre-configured test objects and sample data

//! Test fixtures and common test data.
//!
//! This module provides pre-configured test objects and sample data
//! that can be used across different test suites for consistency.
//!
//! # Examples
//!
//! ```rust
//! use llmspell_testing::fixtures::{
//!     sample_component_metadata,
//!     sample_agent_input,
//!     sample_workflow_steps,
//! };
//!
//! // Use pre-configured test data
//! let metadata = sample_component_metadata();
//! assert_eq!(metadata.name, "test-component");
//!
//! let input = sample_agent_input();
//! assert_eq!(input.prompt, "Test prompt");
//! ```

use llmspell_core::{
    traits::{
        agent::{AgentConfig, ConversationMessage},
        base_agent::{AgentInput, ExecutionContext},
        tool::ToolSchema,
        workflow::{RetryPolicy, WorkflowConfig, WorkflowStep},
    },
    ComponentId, ComponentMetadata, Version,
};

#[cfg(test)]
use llmspell_core::traits::agent::MessageRole;
use serde_json::json;
use std::time::Duration;

/// Sample ComponentMetadata for testing
pub fn sample_component_metadata() -> ComponentMetadata {
    let mut metadata = ComponentMetadata::new(
        "test-component".to_string(),
        "A test component for unit testing".to_string(),
    );
    metadata.version = Version {
        major: 1,
        minor: 0,
        patch: 0,
    };
    metadata
}

/// Sample ComponentMetadata variants for different scenarios
pub fn component_metadata_variants() -> Vec<ComponentMetadata> {
    vec![
        // Minimal metadata
        ComponentMetadata::new(
            "minimal-component".to_string(),
            "Minimal test component".to_string(),
        ),
        // Full metadata
        sample_component_metadata(),
        // Version 2.0 component
        {
            let mut metadata = ComponentMetadata::new(
                "v2-component".to_string(),
                "Version 2 test component".to_string(),
            );
            metadata.version = Version {
                major: 2,
                minor: 0,
                patch: 0,
            };
            metadata
        },
    ]
}

/// Sample AgentInput for testing
pub fn sample_agent_input() -> AgentInput {
    AgentInput::new("Test prompt".to_string())
        .with_context("user_id".to_string(), json!("test-user"))
        .with_context("session".to_string(), json!("test-session"))
}

/// Sample AgentInput variants
pub fn agent_input_variants() -> Vec<AgentInput> {
    vec![
        // Simple input
        AgentInput::new("Simple prompt".to_string()),
        // Input with context
        sample_agent_input(),
        // Complex input
        AgentInput::new("Complex prompt with lots of detail".to_string())
            .with_context("history".to_string(), json!(["previous", "messages"]))
            .with_context("temperature".to_string(), json!(0.7))
            .with_context("max_tokens".to_string(), json!(100))
            .with_context("priority".to_string(), json!("high"))
            .with_context("timestamp".to_string(), json!(1234567890)),
    ]
}

/// Sample ExecutionContext for testing
pub fn sample_execution_context() -> ExecutionContext {
    ExecutionContext::new("test-session-123".to_string())
        .with_user_id("test-user".to_string())
        .with_env("LLMSPELL_ENV".to_string(), "test".to_string())
}

/// Sample conversation for Agent testing
pub fn sample_conversation() -> Vec<ConversationMessage> {
    vec![
        ConversationMessage::system("You are a helpful assistant for testing.".to_string()),
        ConversationMessage::user("Hello, how are you?".to_string()),
        ConversationMessage::assistant(
            "I'm doing well, thank you! How can I help you today?".to_string(),
        ),
        ConversationMessage::user("Can you help me test something?".to_string()),
        ConversationMessage::assistant("Of course! I'd be happy to help you test.".to_string()),
    ]
}

/// Sample AgentConfig for testing
pub fn sample_agent_config() -> AgentConfig {
    AgentConfig {
        max_conversation_length: Some(100),
        system_prompt: Some("You are a test assistant.".to_string()),
        temperature: Some(0.7),
        max_tokens: Some(500),
    }
}

/// Sample ToolSchema for testing
pub fn sample_tool_schema() -> ToolSchema {
    use llmspell_core::traits::tool::{ParameterDef, ParameterType};

    ToolSchema::new(
        "process_data".to_string(),
        "Process data with various options".to_string(),
    )
    .with_parameter(ParameterDef {
        name: "input".to_string(),
        param_type: ParameterType::String,
        description: "The input to process".to_string(),
        required: true,
        default: None,
    })
    .with_parameter(ParameterDef {
        name: "format".to_string(),
        param_type: ParameterType::String,
        description: "Output format".to_string(),
        required: false,
        default: Some(json!("json")),
    })
    .with_parameter(ParameterDef {
        name: "verbose".to_string(),
        param_type: ParameterType::Boolean,
        description: "Verbose output".to_string(),
        required: false,
        default: Some(json!(false)),
    })
    .with_returns(ParameterType::Object)
}

/// Sample workflow steps for testing
pub fn sample_workflow_steps() -> Vec<WorkflowStep> {
    let step1_id = ComponentId::from_name("step-1");
    let step2_id = ComponentId::from_name("step-2");
    let step3_id = ComponentId::from_name("step-3");

    vec![
        WorkflowStep {
            id: step1_id,
            name: "Initialize".to_string(),
            component_id: ComponentId::from_name("init-agent"),
            dependencies: vec![],
            retry_policy: Some(RetryPolicy::default()),
            timeout: Some(Duration::from_secs(30)),
        },
        WorkflowStep {
            id: step2_id,
            name: "Process Data".to_string(),
            component_id: ComponentId::from_name("processor-agent"),
            dependencies: vec![step1_id],
            retry_policy: Some(RetryPolicy {
                max_attempts: 5,
                backoff_seconds: 2,
                exponential_backoff: true,
            }),
            timeout: Some(Duration::from_secs(120)),
        },
        WorkflowStep {
            id: step3_id,
            name: "Generate Report".to_string(),
            component_id: ComponentId::from_name("reporter-agent"),
            dependencies: vec![step2_id],
            retry_policy: None,
            timeout: Some(Duration::from_secs(60)),
        },
    ]
}

/// Sample WorkflowConfig for testing
pub fn sample_workflow_config() -> WorkflowConfig {
    WorkflowConfig {
        max_parallel: Some(3),
        continue_on_error: false,
        timeout: Some(Duration::from_secs(600)),
    }
}

/// Create test error scenarios
pub fn error_scenarios() -> Vec<llmspell_core::LLMSpellError> {
    use llmspell_core::LLMSpellError;

    vec![
        // Component error
        LLMSpellError::Component {
            message: "Component initialization failed".to_string(),
            source: None,
        },
        // Validation error
        LLMSpellError::Validation {
            message: "Invalid input format".to_string(),
            field: Some("email".to_string()),
        },
        // Network error (retryable)
        LLMSpellError::Network {
            message: "Connection timeout".to_string(),
            source: None,
        },
        // Storage error
        LLMSpellError::Storage {
            message: "Database connection failed".to_string(),
            operation: Some("read".to_string()),
            source: None,
        },
    ]
}

/// Create a test environment setup
pub fn setup_test_environment() -> std::collections::HashMap<String, String> {
    let mut env = std::collections::HashMap::new();
    env.insert("LLMSPELL_ENV".to_string(), "test".to_string());
    env.insert("LLMSPELL_LOG_LEVEL".to_string(), "debug".to_string());
    env.insert("LLMSPELL_LOG_FORMAT".to_string(), "json".to_string());
    env
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sample_fixtures() {
        // Test metadata fixture
        let metadata = sample_component_metadata();
        assert_eq!(metadata.name, "test-component");
        assert_eq!(metadata.version.major, 1);

        // Test input fixture
        let input = sample_agent_input();
        assert_eq!(input.prompt, "Test prompt");
        assert!(!input.context.is_empty());

        // Test conversation fixture
        let conversation = sample_conversation();
        assert_eq!(conversation.len(), 5);
        assert_eq!(conversation[0].role, MessageRole::System);

        // Test workflow steps
        let steps = sample_workflow_steps();
        assert_eq!(steps.len(), 3);
        assert_eq!(steps[1].dependencies.len(), 1);
    }

    #[test]
    fn test_fixture_variants() {
        let metadata_variants = component_metadata_variants();
        assert_eq!(metadata_variants.len(), 3);

        let input_variants = agent_input_variants();
        assert_eq!(input_variants.len(), 3);

        let errors = error_scenarios();
        assert!(errors.len() >= 4);
    }
}
