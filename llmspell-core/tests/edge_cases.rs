//! Edge case tests for llmspell-core
//! 
//! These tests verify behavior in boundary conditions and unusual scenarios

use llmspell_core::{
    ComponentId, Version, LLMSpellError,
    traits::{
        base_agent::{AgentInput, AgentOutput},
        agent::{MessageRole, ConversationMessage},
        tool::{ToolCategory, SecurityLevel, ParameterType},
    },
};

#[test]
fn test_component_id_edge_cases() {
    // Empty string should still produce valid ID
    let id1 = ComponentId::from_name("");
    let id2 = ComponentId::from_name("");
    assert_eq!(id1, id2);
    
    // Very long strings
    let long_name = "a".repeat(10000);
    let id = ComponentId::from_name(&long_name);
    let id2 = ComponentId::from_name(&long_name);
    assert_eq!(id, id2);
    
    // Unicode characters
    let unicode_name = "🚀🎯💡 Special-Characters_123 ñáéíóú";
    let id = ComponentId::from_name(unicode_name);
    let id2 = ComponentId::from_name(unicode_name);
    assert_eq!(id, id2);
    
    // Whitespace variations should produce different IDs
    let id1 = ComponentId::from_name("test");
    let id2 = ComponentId::from_name(" test");
    let id3 = ComponentId::from_name("test ");
    let id4 = ComponentId::from_name(" test ");
    assert_ne!(id1, id2);
    assert_ne!(id1, id3);
    assert_ne!(id1, id4);
}

#[test]
fn test_version_edge_cases() {
    // Maximum values
    let v = Version {
        major: u32::MAX,
        minor: u32::MAX,
        patch: u32::MAX,
    };
    assert_eq!(v.to_string(), format!("{}.{}.{}", u32::MAX, u32::MAX, u32::MAX));
    
    // Ordering edge cases
    let v1 = Version { major: 1, minor: 0, patch: 0 };
    let v2 = Version { major: 1, minor: 0, patch: 1 };
    let v3 = Version { major: 1, minor: 1, patch: 0 };
    let v4 = Version { major: 2, minor: 0, patch: 0 };
    
    assert!(v1 < v2);
    assert!(v2 < v3);
    assert!(v3 < v4);
    
    // Compatibility edge cases
    assert!(v1.is_compatible_with(&v2)); // Patch difference
    assert!(v1.is_compatible_with(&v3)); // Minor difference
    assert!(!v1.is_compatible_with(&v4)); // Major difference
    
    // Zero version
    let v0 = Version { major: 0, minor: 0, patch: 0 };
    assert_eq!(v0.to_string(), "0.0.0");
    assert!(v0.is_compatible_with(&v0));
}

#[test]
fn test_error_edge_cases() {
    // Very long error messages
    let long_message = "e".repeat(10000);
    let err = LLMSpellError::Component {
        message: long_message.clone(),
        source: None,
    };
    assert!(err.to_string().contains(&long_message[..100])); // Should contain at least start
    
    // Nested error sources
    let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
    let err1 = LLMSpellError::Storage {
        message: "Storage failed".to_string(),
        operation: Some("read".to_string()),
        source: Some(Box::new(io_err)),
    };
    
    let err2 = LLMSpellError::Component {
        message: "Component failed".to_string(),
        source: Some(Box::new(err1)),
    };
    
    // Should be able to chain through errors
    assert!(err2.to_string().contains("Component failed"));
    
    // Empty optional fields
    let err = LLMSpellError::Validation {
        message: "Validation failed".to_string(),
        field: None,
    };
    assert!(err.to_string().contains("Validation failed"));
}

#[test]
fn test_agent_input_edge_cases() {
    // Empty prompt
    let input = AgentInput::new("".to_string());
    assert_eq!(input.prompt, "");
    assert!(input.context.is_empty());
    
    // Very large context
    let mut input = AgentInput::new("test".to_string());
    for i in 0..1000 {
        input = input.with_context(format!("key{}", i), serde_json::json!(i));
    }
    assert_eq!(input.context.len(), 1000);
    
    // Overwriting context values
    let input = AgentInput::new("test".to_string())
        .with_context("key".to_string(), serde_json::json!("value1"))
        .with_context("key".to_string(), serde_json::json!("value2"));
    assert_eq!(input.get_context("key"), Some(&serde_json::json!("value2")));
    
    // Null and complex values in context
    let input = AgentInput::new("test".to_string())
        .with_context("null".to_string(), serde_json::json!(null))
        .with_context("array".to_string(), serde_json::json!([1, 2, 3]))
        .with_context("object".to_string(), serde_json::json!({"nested": {"deep": "value"}}));
    
    assert_eq!(input.get_context("null"), Some(&serde_json::json!(null)));
    assert_eq!(input.get_context("array"), Some(&serde_json::json!([1, 2, 3])));
}

#[test]
fn test_agent_output_edge_cases() {
    // Empty content
    let output = AgentOutput::new("".to_string());
    assert_eq!(output.content, "");
    assert!(output.metadata.is_empty());
    
    // Unicode content
    let output = AgentOutput::new("Hello 世界 🌍".to_string());
    assert_eq!(output.content, "Hello 世界 🌍");
    
    // Very large metadata
    let mut output = AgentOutput::new("result".to_string());
    for i in 0..1000 {
        output = output.with_metadata(format!("key{}", i), serde_json::json!(i));
    }
    assert_eq!(output.metadata.len(), 1000);
    
    // Overwriting metadata
    let output = AgentOutput::new("test".to_string())
        .with_metadata("key".to_string(), serde_json::json!(1))
        .with_metadata("key".to_string(), serde_json::json!(2));
    assert_eq!(output.get_metadata("key"), Some(&serde_json::json!(2)));
}

#[test]
fn test_conversation_message_edge_cases() {
    // Empty content
    let msg = ConversationMessage::new(MessageRole::User, "".to_string());
    assert_eq!(msg.content, "");
    
    // Very long content
    let long_content = "x".repeat(100000);
    let msg = ConversationMessage::new(MessageRole::Assistant, long_content.clone());
    assert_eq!(msg.content, long_content);
    
    // Unicode content
    let msg = ConversationMessage::new(MessageRole::System, "システムメッセージ 🤖".to_string());
    assert_eq!(msg.content, "システムメッセージ 🤖");
    
    // Timestamp ordering
    let msg1 = ConversationMessage::user("first".to_string());
    std::thread::sleep(std::time::Duration::from_millis(10));
    let msg2 = ConversationMessage::user("second".to_string());
    assert!(msg2.timestamp > msg1.timestamp);
}

#[test]
fn test_tool_category_edge_cases() {
    // Custom categories with special characters
    let category = ToolCategory::Custom("My-Special_Category 123!".to_string());
    assert_eq!(category.to_string(), "My-Special_Category 123!");
    
    // Empty custom category
    let category = ToolCategory::Custom("".to_string());
    assert_eq!(category.to_string(), "");
    
    // Very long custom category
    let long_name = "category".repeat(1000);
    let category = ToolCategory::Custom(long_name.clone());
    assert_eq!(category.to_string(), long_name);
}

#[test]
fn test_security_level_edge_cases() {
    // Ordering tests
    assert!(SecurityLevel::Safe < SecurityLevel::Restricted);
    assert!(SecurityLevel::Restricted < SecurityLevel::Privileged);
    assert!(SecurityLevel::Safe < SecurityLevel::Privileged);
    
    // allows() method edge cases
    assert!(SecurityLevel::Privileged.allows(&SecurityLevel::Safe));
    assert!(SecurityLevel::Privileged.allows(&SecurityLevel::Restricted));
    assert!(SecurityLevel::Privileged.allows(&SecurityLevel::Privileged));
    
    assert!(SecurityLevel::Restricted.allows(&SecurityLevel::Safe));
    assert!(SecurityLevel::Restricted.allows(&SecurityLevel::Restricted));
    assert!(!SecurityLevel::Restricted.allows(&SecurityLevel::Privileged));
    
    assert!(SecurityLevel::Safe.allows(&SecurityLevel::Safe));
    assert!(!SecurityLevel::Safe.allows(&SecurityLevel::Restricted));
    assert!(!SecurityLevel::Safe.allows(&SecurityLevel::Privileged));
}

#[test]
fn test_parameter_type_equality() {
    // Ensure all parameter types are distinct
    let types = vec![
        ParameterType::String,
        ParameterType::Number,
        ParameterType::Boolean,
        ParameterType::Array,
        ParameterType::Object,
        ParameterType::Null,
    ];
    
    for (i, t1) in types.iter().enumerate() {
        for (j, t2) in types.iter().enumerate() {
            if i == j {
                assert_eq!(t1, t2);
            } else {
                assert_ne!(t1, t2);
            }
        }
    }
}

#[test]
fn test_message_role_display_consistency() {
    // Ensure display strings are consistent
    assert_eq!(MessageRole::System.to_string(), "system");
    assert_eq!(MessageRole::User.to_string(), "user");
    assert_eq!(MessageRole::Assistant.to_string(), "assistant");
    
    // Case sensitivity
    assert_ne!(MessageRole::System.to_string(), "System");
    assert_ne!(MessageRole::User.to_string(), "USER");
    assert_ne!(MessageRole::Assistant.to_string(), "ASSISTANT");
}

#[test]
fn test_error_retryability_edge_cases() {
    // Network errors should always be retryable
    let err = LLMSpellError::Network {
        message: "Connection refused".to_string(),
        source: None,
    };
    assert!(err.is_retryable());
    
    // Timeout errors should always be retryable
    let err = LLMSpellError::Timeout {
        message: "Operation timed out".to_string(),
        duration_ms: Some(30000),
    };
    assert!(err.is_retryable());
    
    // Provider errors should be retryable
    let err = LLMSpellError::Provider {
        message: "Rate limit exceeded".to_string(),
        provider: Some("openai".to_string()),
        source: None,
    };
    assert!(err.is_retryable());
    
    // Resource errors should be retryable
    let err = LLMSpellError::Resource {
        message: "Memory limit exceeded".to_string(),
        resource_type: Some("memory".to_string()),
        source: None,
    };
    assert!(err.is_retryable());
    
    // Storage errors depend on operation
    let err = LLMSpellError::Storage {
        message: "Database error".to_string(),
        operation: Some("read".to_string()),
        source: None,
    };
    assert!(err.is_retryable());
    
    let err = LLMSpellError::Storage {
        message: "Database error".to_string(),
        operation: Some("delete".to_string()),
        source: None,
    };
    assert!(!err.is_retryable());
    
    // Validation errors should not be retryable
    let err = LLMSpellError::Validation {
        message: "Invalid input".to_string(),
        field: Some("email".to_string()),
    };
    assert!(!err.is_retryable());
}