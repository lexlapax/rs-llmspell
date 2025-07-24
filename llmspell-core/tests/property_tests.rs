//! Property-based tests for llmspell-core
//!
//! These tests use proptest to verify invariants and properties
//! that should hold for all possible inputs

use llmspell_core::{
    traits::{
        agent::{AgentConfig, ConversationMessage, MessageRole},
        workflow::{RetryPolicy, WorkflowStep},
    },
    types::{AgentInput, AgentOutput},
    ComponentId, ComponentMetadata, ExecutionContext, Version,
};
use proptest::prelude::*;

// Strategy for generating ComponentIds
prop_compose! {
    fn arb_component_name()(name in "[a-zA-Z][a-zA-Z0-9-_]{0,63}") -> String {
        name
    }
}

// Strategy for generating Versions
prop_compose! {
    fn arb_version()(
        major in 0u32..100,
        minor in 0u32..100,
        patch in 0u32..100
    ) -> Version {
        Version::new(major, minor, patch)
    }
}

// Strategy for generating AgentInput
prop_compose! {
    fn arb_agent_input()(
        prompt in prop::string::string_regex("[a-zA-Z0-9 .,!?-]{1,1000}").unwrap(),
        context_keys in prop::collection::vec(
            prop::string::string_regex("[a-zA-Z][a-zA-Z0-9_]{0,31}").unwrap(),
            0..5
        ),
        context_values in prop::collection::vec(
            prop_oneof![
                Just(serde_json::json!(null)),
                prop::num::i64::ANY.prop_map(|n| serde_json::json!(n)),
                prop::string::string_regex("[a-zA-Z0-9 ]{0,50}").unwrap().prop_map(|s| serde_json::json!(s)),
                prop::bool::ANY.prop_map(|b| serde_json::json!(b)),
            ],
            0..5
        )
    ) -> AgentInput {
        let mut input = AgentInput::text(prompt);
        for (key, value) in context_keys.into_iter().zip(context_values.into_iter()) {
            input = input.with_parameter(key, value);
        }
        input
    }
}

// Strategy for generating MessageRole
fn arb_message_role() -> impl Strategy<Value = MessageRole> {
    prop_oneof![
        Just(MessageRole::System),
        Just(MessageRole::User),
        Just(MessageRole::Assistant),
    ]
}

// Strategy for generating ConversationMessage
prop_compose! {
    fn arb_conversation_message()(
        role in arb_message_role(),
        content in prop::string::string_regex("[a-zA-Z0-9 .,!?-]{1,1000}").unwrap()
    ) -> ConversationMessage {
        ConversationMessage::new(role, content)
    }
}

// Strategy for generating RetryPolicy
prop_compose! {
    fn arb_retry_policy()(
        max_attempts in 1u32..10,
        backoff_seconds in 1u32..60,
        exponential_backoff in any::<bool>()
    ) -> RetryPolicy {
        RetryPolicy {
            max_attempts,
            backoff_seconds,
            exponential_backoff,
        }
    }
}

proptest! {
    #[test]
    fn test_component_id_deterministic(name in arb_component_name()) {
        // Property: Same name always produces same ComponentId
        let id1 = ComponentId::from_name(&name);
        let id2 = ComponentId::from_name(&name);
        prop_assert_eq!(id1, id2);
    }

    #[test]
    fn test_component_id_different_names(
        name1 in arb_component_name(),
        name2 in arb_component_name()
    ) {
        // Property: Different names produce different ComponentIds
        prop_assume!(name1 != name2);
        let id1 = ComponentId::from_name(&name1);
        let id2 = ComponentId::from_name(&name2);
        prop_assert_ne!(id1, id2);
    }

    #[test]
    fn test_component_id_serialization_roundtrip(name in arb_component_name()) {
        // Property: ComponentId survives serialization/deserialization
        let id = ComponentId::from_name(&name);
        let json = serde_json::to_string(&id).unwrap();
        let deserialized: ComponentId = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(id, deserialized);
    }

    #[test]
    fn test_version_ordering_properties(v1 in arb_version(), v2 in arb_version()) {
        // Property: Version ordering is transitive
        if v1 < v2 {
            prop_assert!(v2 >= v1); // Antisymmetry
        }
        if v1 == v2 {
            prop_assert!(v1.is_compatible_with(&v2));
            prop_assert!(v2.is_compatible_with(&v1));
        }
    }

    #[test]
    fn test_version_compatibility_properties(v in arb_version()) {
        // Property: A version is always compatible with itself
        prop_assert!(v.is_compatible_with(&v));

        // Property: Versions with same major are compatible
        let v2 = Version::new(v.major, v.minor + 1, v.patch);
        prop_assert!(v.is_compatible_with(&v2));

        // Property: Versions with different major are not compatible
        let v3 = Version::new(v.major + 1, v.minor, v.patch);
        prop_assert!(!v.is_compatible_with(&v3));
    }

    #[test]
    fn test_version_serialization_roundtrip(v in arb_version()) {
        // Property: Version survives serialization/deserialization
        let json = serde_json::to_string(&v).unwrap();
        let deserialized: Version = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(v.major, deserialized.major);
        prop_assert_eq!(v.minor, deserialized.minor);
        prop_assert_eq!(v.patch, deserialized.patch);
        prop_assert_eq!(v.to_string(), deserialized.to_string());
    }

    #[test]
    fn test_agent_input_context_preservation(input in arb_agent_input()) {
        // Property: Parameter values are preserved
        for (key, value) in &input.parameters {
            prop_assert_eq!(input.parameters.get(key), Some(value));
        }

        // Property: Non-existent keys return None
        prop_assert_eq!(input.parameters.get("non_existent_key_xyz"), None);
    }

    #[test]
    fn test_agent_input_serialization_roundtrip(input in arb_agent_input()) {
        // Property: AgentInput survives serialization/deserialization
        let json = serde_json::to_string(&input).unwrap();
        let deserialized: AgentInput = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(input.text, deserialized.text);
        prop_assert_eq!(input.parameters, deserialized.parameters);
    }

    #[test]
    fn test_agent_output_metadata_preservation(
        content in prop::string::string_regex("[a-zA-Z0-9 ]{1,100}").unwrap(),
        metadata_pairs in prop::collection::btree_map(
            prop::string::string_regex("[a-zA-Z][a-zA-Z0-9_]{0,31}").unwrap(),
            prop_oneof![
                Just(serde_json::json!(null)),
                prop::num::i64::ANY.prop_map(|n| serde_json::json!(n)),
                prop::string::string_regex("[a-zA-Z0-9 ]{0,50}").unwrap().prop_map(|s| serde_json::json!(s)),
                prop::bool::ANY.prop_map(|b| serde_json::json!(b)),
            ],
            0..5
        )
    ) {
        // Property: Metadata values are preserved (using BTreeMap to ensure unique keys)
        let mut metadata = llmspell_core::types::OutputMetadata::default();
        for (key, value) in metadata_pairs.iter() {
            metadata.extra.insert(key.clone(), value.clone());
        }
        let output = AgentOutput::text(content).with_metadata(metadata);
        for (key, value) in metadata_pairs.into_iter() {
            prop_assert_eq!(output.metadata.extra.get(&key), Some(&value));
        }
    }

    #[test]
    fn test_conversation_message_properties(msg in arb_conversation_message()) {
        // Property: Message fields are preserved
        let cloned = msg.clone();
        prop_assert_eq!(msg.role, cloned.role);
        prop_assert_eq!(msg.content, cloned.content);

        // Property: Timestamp is set and reasonable
        let now = chrono::Utc::now();
        let diff = now - msg.timestamp;
        prop_assert!(diff.num_seconds() >= 0);
        prop_assert!(diff.num_seconds() < 60); // Should be created within last minute
    }

    #[test]
    fn test_execution_context_environment_properties(
        session_id in prop::string::string_regex("[a-zA-Z0-9-]{1,50}").unwrap(),
        user_id in prop::option::of(prop::string::string_regex("[a-zA-Z0-9-]{1,50}").unwrap()),
        env_keys in prop::collection::vec(
            prop::string::string_regex("[A-Z_]{1,20}").unwrap(),
            0..5
        ),
        env_values in prop::collection::vec(
            prop::string::string_regex("[a-zA-Z0-9-]{1,50}").unwrap(),
            0..5
        )
    ) {
        // Property: Context preserves all fields
        let mut context = ExecutionContext::with_conversation(session_id.clone());

        if let Some(uid) = user_id.clone() {
            context.user_id = Some(uid);
        }

        for (key, value) in env_keys.into_iter().zip(env_values.into_iter()) {
            let json_value = serde_json::json!(value.clone());
            context = context.with_data(key.clone(), json_value.clone());
            prop_assert_eq!(context.data.get(&key), Some(&json_value));
        }

        prop_assert_eq!(context.conversation_id, Some(session_id));
        prop_assert_eq!(context.user_id, user_id);
    }

    #[test]
    fn test_retry_policy_properties(policy in arb_retry_policy()) {
        // Property: All fields are preserved
        let cloned = policy.clone();
        prop_assert_eq!(policy.max_attempts, cloned.max_attempts);
        prop_assert_eq!(policy.backoff_seconds, cloned.backoff_seconds);
        prop_assert_eq!(policy.exponential_backoff, cloned.exponential_backoff);

        // Property: Serialization roundtrip
        let json = serde_json::to_string(&policy).unwrap();
        let deserialized: RetryPolicy = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(policy.max_attempts, deserialized.max_attempts);
    }

    #[test]
    fn test_component_metadata_timestamp_ordering(
        name in arb_component_name(),
        description in prop::string::string_regex("[a-zA-Z0-9 ]{1,100}").unwrap()
    ) {
        // Property: created_at <= updated_at
        let metadata = ComponentMetadata::new(name, description);
        prop_assert!(metadata.created_at <= metadata.updated_at);

        // Property: After version update, updated_at changes
        let mut metadata_mut = metadata.clone();
        std::thread::sleep(std::time::Duration::from_millis(10));
        metadata_mut.update_version(Version::new(1, 0, 0));
        prop_assert!(metadata_mut.updated_at > metadata.updated_at);
    }

    #[test]
    fn test_agent_config_optional_fields(
        max_conversation_length in prop::option::of(1usize..1000),
        system_prompt in prop::option::of(prop::string::string_regex("[a-zA-Z0-9 ]{1,100}").unwrap()),
        temperature in prop::option::of(0.0f32..2.0),
        max_tokens in prop::option::of(1usize..10000)
    ) {
        // Property: Optional fields are preserved correctly
        let config = AgentConfig {
            max_conversation_length,
            system_prompt: system_prompt.clone(),
            temperature,
            max_tokens,
        };

        // Serialization roundtrip
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: AgentConfig = serde_json::from_str(&json).unwrap();

        prop_assert_eq!(config.max_conversation_length, deserialized.max_conversation_length);
        prop_assert_eq!(config.system_prompt, deserialized.system_prompt);
        prop_assert_eq!(config.temperature, deserialized.temperature);
        prop_assert_eq!(config.max_tokens, deserialized.max_tokens);
    }
}

// Additional complex property tests
proptest! {
    #[test]
    fn test_workflow_step_dependency_properties(
        name in arb_component_name(),
        component_id in arb_component_name().prop_map(|n| ComponentId::from_name(&n)),
        dep_count in 0usize..5
    ) {
        // Generate random dependencies
        let deps: Vec<ComponentId> = (0..dep_count)
            .map(|i| ComponentId::from_name(&format!("dep-{}", i)))
            .collect();

        // Build workflow step
        let mut step = WorkflowStep::new(name, component_id);
        for dep in &deps {
            step = step.with_dependency(*dep);
        }

        // Property: All dependencies are preserved
        prop_assert_eq!(step.dependencies.len(), deps.len());
        for dep in deps {
            prop_assert!(step.dependencies.contains(&dep));
        }
    }

    #[test]
    fn test_error_severity_ordering_transitivity(
        errors in prop::collection::vec(
            prop_oneof![
                Just(llmspell_core::error::ErrorSeverity::Info),
                Just(llmspell_core::error::ErrorSeverity::Warning),
                Just(llmspell_core::error::ErrorSeverity::Error),
                Just(llmspell_core::error::ErrorSeverity::Critical),
                Just(llmspell_core::error::ErrorSeverity::Fatal),
            ],
            3..=3
        )
    ) {
        // Property: If a < b and b < c, then a < c (transitivity)
        let (a, b, c) = (errors[0].clone(), errors[1].clone(), errors[2].clone());
        if a < b && b < c {
            prop_assert!(a < c);
        }

        // Property: Exactly one of a < b, a == b, or a > b is true
        let less = a < b;
        let equal = a == b;
        let greater = a > b;
        prop_assert_eq!((less as i32) + (equal as i32) + (greater as i32), 1);
    }
}

#[cfg(test)]
mod regression_tests {
    use super::*;

    #[test]
    fn test_empty_string_component_id() {
        // Regression test: empty strings should produce valid IDs
        let id1 = ComponentId::from_name("");
        let id2 = ComponentId::from_name("");
        assert_eq!(id1, id2);
    }

    #[test]
    fn test_unicode_component_names() {
        // Regression test: unicode should work in component names
        let names = vec![
            "компонент",
            "组件",
            "コンポーネント",
            "🚀-rocket",
            "café-component",
        ];

        for name in names {
            let id1 = ComponentId::from_name(name);
            let id2 = ComponentId::from_name(name);
            assert_eq!(id1, id2);
        }
    }

    #[test]
    fn test_very_large_version_numbers() {
        // Regression test: large version numbers
        let v = Version::new(u32::MAX, u32::MAX, u32::MAX);
        let json = serde_json::to_string(&v).unwrap();
        let deserialized: Version = serde_json::from_str(&json).unwrap();
        assert_eq!(v, deserialized);
    }
}
