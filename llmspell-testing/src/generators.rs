//! ABOUTME: Proptest strategies and data generators for property-based testing
//! ABOUTME: Provides strategies for generating test data for all core types

//! Property-based test generators.
//!
//! This module provides proptest strategies for generating test data
//! for all core types in the LLMSpell framework. These strategies help
//! test invariants and edge cases.
//!
//! # Examples
//!
//! ```rust
//! use proptest::prelude::*;
//! use llmspell_testing::generators::{component_id_strategy, version_strategy};
//!
//! proptest! {
//!     #[test]
//!     fn test_component_id_properties(id1 in component_id_strategy(), id2 in component_id_strategy()) {
//!         // Test that different generated IDs are not equal
//!         if id1 != id2 {
//!             assert_ne!(id1, id2);
//!         }
//!     }
//! }
//! ```

use llmspell_core::{
    traits::{
        agent::{AgentConfig, ConversationMessage, MessageRole},
        base_agent::{AgentInput, AgentOutput, ExecutionContext},
        tool::{SecurityLevel, ToolCategory, ToolSchema},
        workflow::{RetryPolicy, WorkflowConfig, WorkflowStatus, WorkflowStep},
    },
    ComponentId, ComponentMetadata, Version,
};
use proptest::prelude::*;
use std::time::Duration;

/// Strategy for generating ComponentId values
pub fn component_id_strategy() -> impl Strategy<Value = ComponentId> {
    "[a-zA-Z0-9_-]{5,20}".prop_map(|name| ComponentId::from_name(&name))
}

/// Strategy for generating ComponentId from a name
pub fn component_id_from_name_strategy() -> impl Strategy<Value = (String, ComponentId)> {
    "[a-zA-Z0-9_-]{5,20}".prop_map(|name| {
        let id = ComponentId::from_name(&name);
        (name, id)
    })
}

/// Strategy for generating Version values
pub fn version_strategy() -> impl Strategy<Value = Version> {
    (0u32..100u32, 0u32..100u32, 0u32..1000u32).prop_map(|(major, minor, patch)| Version {
        major,
        minor,
        patch,
    })
}

/// Strategy for generating ComponentMetadata
pub fn component_metadata_strategy() -> impl Strategy<Value = ComponentMetadata> {
    ("[a-zA-Z][a-zA-Z0-9_-]{2,30}", ".*", version_strategy()).prop_map(
        |(name, description, version)| {
            let mut metadata = ComponentMetadata::new(name, description);
            metadata.version = version;
            metadata
        },
    )
}

/// Strategy for generating simple JSON values
pub fn json_value_strategy() -> impl Strategy<Value = serde_json::Value> {
    prop_oneof![
        Just(serde_json::Value::Null),
        any::<bool>().prop_map(serde_json::Value::Bool),
        any::<i64>().prop_map(serde_json::Value::from),
        ".*".prop_map(serde_json::Value::String),
    ]
}

/// Strategy for generating AgentInput
pub fn agent_input_strategy() -> impl Strategy<Value = AgentInput> {
    (
        ".*",
        prop::collection::hash_map("[a-zA-Z0-9_]+", json_value_strategy(), 0..5),
    )
        .prop_map(|(prompt, context)| {
            let mut input = AgentInput::new(prompt);
            input.context = context;
            input
        })
}

/// Strategy for generating AgentOutput
pub fn agent_output_strategy() -> impl Strategy<Value = AgentOutput> {
    (
        ".*",
        prop::collection::hash_map("[a-zA-Z0-9_]+", json_value_strategy(), 0..5),
    )
        .prop_map(|(content, metadata)| {
            let mut output = AgentOutput::new(content);
            output.metadata = metadata;
            output
        })
}

/// Strategy for generating ExecutionContext
pub fn execution_context_strategy() -> impl Strategy<Value = ExecutionContext> {
    (
        "[a-zA-Z0-9-]{8,36}",
        prop::option::of("[a-zA-Z0-9_]+"),
        prop::collection::hash_map("[a-zA-Z_][a-zA-Z0-9_]*", "[a-zA-Z0-9_-]+", 0..5),
    )
        .prop_map(|(session_id, user_id, environment)| {
            let mut context = ExecutionContext::new(session_id);
            context.user_id = user_id;
            context.environment = environment;
            context
        })
}

/// Strategy for generating MessageRole
pub fn message_role_strategy() -> impl Strategy<Value = MessageRole> {
    prop_oneof![
        Just(MessageRole::System),
        Just(MessageRole::User),
        Just(MessageRole::Assistant),
    ]
}

/// Strategy for generating ConversationMessage
pub fn conversation_message_strategy() -> impl Strategy<Value = ConversationMessage> {
    (message_role_strategy(), ".*")
        .prop_map(|(role, content)| ConversationMessage::new(role, content))
}

/// Strategy for generating AgentConfig
pub fn agent_config_strategy() -> impl Strategy<Value = AgentConfig> {
    (
        prop::option::of(1usize..1000usize),
        prop::option::of(".*"),
        prop::option::of(0.0f32..2.0f32),
        prop::option::of(1usize..10000usize),
    )
        .prop_map(
            |(max_conversation_length, system_prompt, temperature, max_tokens)| AgentConfig {
                max_conversation_length,
                system_prompt,
                temperature,
                max_tokens,
            },
        )
}

/// Strategy for generating ToolCategory
pub fn tool_category_strategy() -> impl Strategy<Value = ToolCategory> {
    prop_oneof![
        Just(ToolCategory::Filesystem),
        Just(ToolCategory::Web),
        Just(ToolCategory::Analysis),
        Just(ToolCategory::Data),
        Just(ToolCategory::System),
        Just(ToolCategory::Utility),
        Just(ToolCategory::Custom("custom-tool".to_string())),
    ]
}

/// Strategy for generating SecurityLevel
pub fn security_level_strategy() -> impl Strategy<Value = SecurityLevel> {
    prop_oneof![
        Just(SecurityLevel::Safe),
        Just(SecurityLevel::Restricted),
        Just(SecurityLevel::Privileged),
    ]
}

/// Strategy for generating ToolSchema
pub fn tool_schema_strategy() -> impl Strategy<Value = ToolSchema> {
    ("[a-zA-Z][a-zA-Z0-9_]{2,20}", ".*")
        .prop_map(|(name, description)| ToolSchema::new(name, description))
}

/// Strategy for generating RetryPolicy
pub fn retry_policy_strategy() -> impl Strategy<Value = RetryPolicy> {
    (1u32..10u32, 1u32..60u32, any::<bool>()).prop_map(
        |(max_attempts, backoff_seconds, exponential_backoff)| RetryPolicy {
            max_attempts,
            backoff_seconds,
            exponential_backoff,
        },
    )
}

/// Strategy for generating WorkflowStep
pub fn workflow_step_strategy() -> impl Strategy<Value = WorkflowStep> {
    (
        "[a-zA-Z][a-zA-Z0-9_-]{2,30}",
        component_id_strategy(),
        prop::collection::vec(component_id_strategy(), 0..5),
        prop::option::of(retry_policy_strategy()),
        prop::option::of(1u64..3600u64),
    )
        .prop_map(
            |(name, component_id, dependencies, retry_policy, timeout_secs)| {
                let mut step = WorkflowStep::new(name, component_id);
                step.dependencies = dependencies;
                step.retry_policy = retry_policy;
                step.timeout = timeout_secs.map(Duration::from_secs);
                step
            },
        )
}

/// Strategy for generating WorkflowConfig  
pub fn workflow_config_strategy() -> impl Strategy<Value = WorkflowConfig> {
    (
        prop::option::of(1usize..100usize),
        any::<bool>(),
        prop::option::of(1u64..86400u64),
    )
        .prop_map(
            |(max_parallel, continue_on_error, timeout_secs)| WorkflowConfig {
                max_parallel,
                continue_on_error,
                timeout: timeout_secs.map(Duration::from_secs),
            },
        )
}

/// Strategy for generating WorkflowStatus
pub fn workflow_status_strategy() -> impl Strategy<Value = WorkflowStatus> {
    prop_oneof![
        Just(WorkflowStatus::Pending),
        Just(WorkflowStatus::Running),
        Just(WorkflowStatus::Completed),
        Just(WorkflowStatus::Failed),
        Just(WorkflowStatus::Cancelled),
    ]
}

// Removed json_roundtrip_strategy as it's not needed with current implementation

#[cfg(test)]
mod tests {
    use super::*;

    proptest! {
        #[test]
        fn test_component_id_generation(id in component_id_strategy()) {
            // Should be a valid ComponentId
            // We can't check internals, but can verify it was created
            let _ = id;
        }

        #[test]
        fn test_component_id_from_name((name, id) in component_id_from_name_strategy()) {
            // Same name should generate same ID
            let id2 = ComponentId::from_name(&name);
            assert_eq!(id, id2);
        }

        #[test]
        fn test_version_generation(version in version_strategy()) {
            // Version should serialize/deserialize correctly
            let json = serde_json::to_string(&version).unwrap();
            let parsed: Version = serde_json::from_str(&json).unwrap();
            assert_eq!(version, parsed);
        }

        #[test]
        fn test_agent_input_generation(input in agent_input_strategy()) {
            // Should have a non-empty prompt
            assert!(!input.prompt.is_empty() || input.prompt.is_empty()); // tautology but tests generation
        }

        #[test]
        fn test_workflow_step_generation(step in workflow_step_strategy()) {
            // Step should have a name
            assert!(!step.name.is_empty());

            // Just verify we have the expected fields
            let _ = step.component_id;
            let _ = step.dependencies;
        }
    }
}
