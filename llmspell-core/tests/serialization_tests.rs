//! Serialization and deserialization tests
//!
//! These tests verify that all types can be correctly serialized and deserialized

use llmspell_core::{
    traits::{
        agent::{AgentConfig, ConversationMessage, MessageRole},
        base_agent::{AgentInput, AgentOutput, ExecutionContext},
        tool::{ParameterDef, ParameterType, SecurityLevel, ToolCategory, ToolSchema},
        workflow::{RetryPolicy, StepResult, WorkflowStatus, WorkflowStep},
    },
    ComponentId, ComponentMetadata, Version,
};
use serde_json;

#[test]
fn test_component_id_json_roundtrip() {
    let id = ComponentId::from_name("test-component");

    let json = serde_json::to_string(&id).unwrap();
    let deserialized: ComponentId = serde_json::from_str(&json).unwrap();

    assert_eq!(id, deserialized);
}

#[test]
fn test_version_json_roundtrip() {
    let version = Version::new(1, 2, 3);

    let json = serde_json::to_string(&version).unwrap();
    let deserialized: Version = serde_json::from_str(&json).unwrap();

    assert_eq!(version, deserialized);
    assert_eq!(deserialized.major, 1);
    assert_eq!(deserialized.minor, 2);
    assert_eq!(deserialized.patch, 3);
}

#[test]
fn test_component_metadata_json_roundtrip() {
    let metadata = ComponentMetadata::new(
        "test-component".to_string(),
        "Test component description".to_string(),
    );

    let json = serde_json::to_string(&metadata).unwrap();
    let deserialized: ComponentMetadata = serde_json::from_str(&json).unwrap();

    assert_eq!(metadata.id, deserialized.id);
    assert_eq!(metadata.name, deserialized.name);
    assert_eq!(metadata.description, deserialized.description);
    assert_eq!(metadata.version, deserialized.version);
}

#[test]
fn test_agent_input_json_roundtrip() {
    let input = AgentInput::new("test prompt".to_string())
        .with_context("key1".to_string(), serde_json::json!("value1"))
        .with_context("key2".to_string(), serde_json::json!(42))
        .with_context(
            "nested".to_string(),
            serde_json::json!({
                "inner": "value",
                "count": 10
            }),
        );

    let json = serde_json::to_string(&input).unwrap();
    let deserialized: AgentInput = serde_json::from_str(&json).unwrap();

    assert_eq!(input.prompt, deserialized.prompt);
    assert_eq!(input.context, deserialized.context);
}

#[test]
fn test_agent_output_json_roundtrip() {
    let output = AgentOutput::new("result content".to_string())
        .with_metadata("confidence".to_string(), serde_json::json!(0.95))
        .with_metadata("tokens".to_string(), serde_json::json!(150))
        .with_metadata("model".to_string(), serde_json::json!("gpt-4"));

    let json = serde_json::to_string(&output).unwrap();
    let deserialized: AgentOutput = serde_json::from_str(&json).unwrap();

    assert_eq!(output.content, deserialized.content);
    assert_eq!(output.metadata, deserialized.metadata);
}

#[test]
fn test_execution_context_json_roundtrip() {
    let context = ExecutionContext::new("session-123".to_string())
        .with_user_id("user-456".to_string())
        .with_env("ENV_VAR".to_string(), "value".to_string());

    let json = serde_json::to_string(&context).unwrap();
    let deserialized: ExecutionContext = serde_json::from_str(&json).unwrap();

    assert_eq!(context.session_id, deserialized.session_id);
    assert_eq!(context.user_id, deserialized.user_id);
    assert_eq!(context.environment, deserialized.environment);
}

#[test]
fn test_conversation_message_json_roundtrip() {
    let messages = vec![
        ConversationMessage::system("System prompt".to_string()),
        ConversationMessage::user("User message".to_string()),
        ConversationMessage::assistant("Assistant response".to_string()),
    ];

    for msg in messages {
        let json = serde_json::to_string(&msg).unwrap();
        let deserialized: ConversationMessage = serde_json::from_str(&json).unwrap();

        assert_eq!(msg.role, deserialized.role);
        assert_eq!(msg.content, deserialized.content);
        // Timestamps might differ slightly, so we just check they exist
        assert!(deserialized.timestamp.timestamp() > 0);
    }
}

#[test]
fn test_message_role_json_roundtrip() {
    let roles = vec![
        MessageRole::System,
        MessageRole::User,
        MessageRole::Assistant,
    ];

    for role in roles {
        let json = serde_json::to_string(&role).unwrap();
        let deserialized: MessageRole = serde_json::from_str(&json).unwrap();
        assert_eq!(role, deserialized);
    }
}

#[test]
fn test_agent_config_json_roundtrip() {
    let config = AgentConfig {
        max_conversation_length: Some(100),
        system_prompt: Some("You are helpful".to_string()),
        temperature: Some(0.7),
        max_tokens: Some(2000),
    };

    let json = serde_json::to_string(&config).unwrap();
    let deserialized: AgentConfig = serde_json::from_str(&json).unwrap();

    assert_eq!(
        config.max_conversation_length,
        deserialized.max_conversation_length
    );
    assert_eq!(config.system_prompt, deserialized.system_prompt);
    assert_eq!(config.temperature, deserialized.temperature);
    assert_eq!(config.max_tokens, deserialized.max_tokens);
}

#[test]
fn test_tool_schema_json_roundtrip() {
    let schema = ToolSchema::new("test_tool".to_string(), "A test tool".to_string())
        .with_parameter(ParameterDef {
            name: "param1".to_string(),
            param_type: ParameterType::String,
            description: "First parameter".to_string(),
            required: true,
            default: None,
        })
        .with_parameter(ParameterDef {
            name: "param2".to_string(),
            param_type: ParameterType::Number,
            description: "Second parameter".to_string(),
            required: false,
            default: Some(serde_json::json!(42)),
        });

    let json = serde_json::to_string(&schema).unwrap();
    let deserialized: ToolSchema = serde_json::from_str(&json).unwrap();

    assert_eq!(schema.name, deserialized.name);
    assert_eq!(schema.description, deserialized.description);
    assert_eq!(schema.parameters.len(), deserialized.parameters.len());
}

#[test]
fn test_tool_category_json_roundtrip() {
    let categories = vec![
        ToolCategory::Filesystem,
        ToolCategory::Web,
        ToolCategory::Analysis,
        ToolCategory::Data,
        ToolCategory::System,
        ToolCategory::Utility,
        ToolCategory::Custom("MyCategory".to_string()),
    ];

    for category in categories {
        let json = serde_json::to_string(&category).unwrap();
        let deserialized: ToolCategory = serde_json::from_str(&json).unwrap();
        assert_eq!(category, deserialized);
    }
}

#[test]
fn test_security_level_json_roundtrip() {
    let levels = vec![
        SecurityLevel::Safe,
        SecurityLevel::Restricted,
        SecurityLevel::Privileged,
    ];

    for level in levels {
        let json = serde_json::to_string(&level).unwrap();
        let deserialized: SecurityLevel = serde_json::from_str(&json).unwrap();
        assert_eq!(level, deserialized);
    }
}

#[test]
fn test_parameter_type_json_roundtrip() {
    let types = vec![
        ParameterType::String,
        ParameterType::Number,
        ParameterType::Boolean,
        ParameterType::Array,
        ParameterType::Object,
        ParameterType::Null,
    ];

    for param_type in types {
        let json = serde_json::to_string(&param_type).unwrap();
        let deserialized: ParameterType = serde_json::from_str(&json).unwrap();
        assert_eq!(param_type, deserialized);
    }
}

#[test]
fn test_workflow_step_json_roundtrip() {
    let component_id = ComponentId::from_name("process-component");
    let dep_id = ComponentId::from_name("init-component");

    let step = WorkflowStep::new("process_data".to_string(), component_id)
        .with_dependency(dep_id)
        .with_retry(RetryPolicy::default())
        .with_timeout(std::time::Duration::from_secs(300));

    let json = serde_json::to_string(&step).unwrap();
    let deserialized: WorkflowStep = serde_json::from_str(&json).unwrap();

    assert_eq!(step.id, deserialized.id);
    assert_eq!(step.name, deserialized.name);
    assert_eq!(step.component_id, deserialized.component_id);
    assert_eq!(step.dependencies, deserialized.dependencies);
}

#[test]
fn test_step_result_json_roundtrip() {
    let step_id = ComponentId::from_name("test-step");
    let output = AgentOutput::new("Completed successfully".to_string())
        .with_metadata("records".to_string(), serde_json::json!(100));

    let success_result = StepResult::success(step_id, output, std::time::Duration::from_secs(1));

    let json = serde_json::to_string(&success_result).unwrap();
    let deserialized: StepResult = serde_json::from_str(&json).unwrap();

    assert_eq!(success_result.success, deserialized.success);
    assert_eq!(success_result.step_id, deserialized.step_id);
    assert!(deserialized.error.is_none());

    let error_result = StepResult::failure(
        step_id,
        "Processing failed".to_string(),
        std::time::Duration::from_secs(1),
        2,
    );

    let json = serde_json::to_string(&error_result).unwrap();
    let deserialized: StepResult = serde_json::from_str(&json).unwrap();

    assert!(!deserialized.success);
    assert_eq!(deserialized.error, Some("Processing failed".to_string()));
    assert_eq!(deserialized.retry_count, 2);
}

#[test]
fn test_workflow_status_json_roundtrip() {
    let statuses = vec![
        WorkflowStatus::Pending,
        WorkflowStatus::Running,
        WorkflowStatus::Completed,
        WorkflowStatus::Failed,
        WorkflowStatus::Cancelled,
    ];

    for status in statuses {
        let json = serde_json::to_string(&status).unwrap();
        let deserialized: WorkflowStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(status, deserialized);
    }
}

#[test]
fn test_retry_policy_json_roundtrip() {
    let policies = vec![
        RetryPolicy::default(),
        RetryPolicy {
            max_attempts: 5,
            backoff_seconds: 2,
            exponential_backoff: false,
        },
        RetryPolicy {
            max_attempts: 10,
            backoff_seconds: 5,
            exponential_backoff: true,
        },
    ];

    for policy in policies {
        let json = serde_json::to_string(&policy).unwrap();
        let deserialized: RetryPolicy = serde_json::from_str(&json).unwrap();

        assert_eq!(policy.max_attempts, deserialized.max_attempts);
        assert_eq!(policy.backoff_seconds, deserialized.backoff_seconds);
        assert_eq!(policy.exponential_backoff, deserialized.exponential_backoff);
    }
}

#[test]
fn test_complex_nested_serialization() {
    // Test deeply nested structures
    let input = AgentInput::new("complex test".to_string()).with_context(
        "nested".to_string(),
        serde_json::json!({
            "level1": {
                "level2": {
                    "level3": {
                        "data": [1, 2, 3],
                        "flag": true,
                        "value": null
                    }
                }
            }
        }),
    );

    let json = serde_json::to_string(&input).unwrap();
    let deserialized: AgentInput = serde_json::from_str(&json).unwrap();

    let nested = deserialized.get_context("nested").unwrap();
    let level3 = &nested["level1"]["level2"]["level3"];
    assert_eq!(level3["data"], serde_json::json!([1, 2, 3]));
    assert_eq!(level3["flag"], serde_json::json!(true));
    assert_eq!(level3["value"], serde_json::json!(null));
}
