//! Unit tests for trait behavior using mocks
//!
//! These tests verify the expected behavior of trait methods

use llmspell_core::{
    traits::{
        agent::{Agent, AgentConfig, ConversationMessage, MessageRole},
        base_agent::BaseAgent,
        tool::{ParameterDef, ParameterType, SecurityLevel, Tool, ToolCategory, ToolSchema},
        workflow::{RetryPolicy, Status as WorkflowStatus, Workflow, WorkflowStep},
    },
    types::{AgentInput, AgentOutput},
    ComponentId, ExecutionContext, LLMSpellError,
};
use llmspell_testing::mocks::*;
use mockall::predicate::*;
#[tokio::test]
async fn test_base_agent_mock_behavior() {
    let mut mock = MockBaseAgent::new();

    // Note: We can't mock metadata() as it returns a reference
    // Instead we'll just test the other methods

    mock.expect_validate_input()
        .with(always())
        .times(1)
        .returning(|_| Ok(()));

    mock.expect_execute()
        .with(always(), always())
        .times(1)
        .returning(|input, _| Ok(AgentOutput::text(format!("Mocked: {}", input.text))));

    // Test the mock
    let input = AgentInput::text("test input");
    let context = ExecutionContext::with_conversation("test-session".to_string());

    assert!(mock.validate_input(&input).await.is_ok());
    let result = mock.execute(input, context).await.unwrap();
    assert_eq!(result.text, "Mocked: test input");
}
#[tokio::test]
async fn test_agent_mock_conversation_management() {
    let mut mock = MockAgent::new();

    // Setup conversation expectations
    let messages = vec![
        ConversationMessage::user("Hello".to_string()),
        ConversationMessage::assistant("Hi there!".to_string()),
    ];

    mock.expect_get_conversation()
        .times(1)
        .returning(move || Ok(messages.clone()));

    mock.expect_add_message()
        .withf(|msg| msg.role == MessageRole::System && msg.content == "System prompt")
        .times(1)
        .returning(|_| Ok(()));

    mock.expect_clear_conversation()
        .times(1)
        .returning(|| Ok(()));

    // Test conversation methods
    let conv = mock.get_conversation().await.unwrap();
    assert_eq!(conv.len(), 2);
    assert_eq!(conv[0].content, "Hello");

    mock.add_message(ConversationMessage::system("System prompt".to_string()))
        .await
        .unwrap();
    mock.clear_conversation().await.unwrap();
}
#[tokio::test]
async fn test_tool_mock_schema_validation() {
    let mut mock = MockTool::new();

    // Setup schema
    let schema = ToolSchema::new("test_tool".to_string(), "A test tool".to_string())
        .with_parameter(ParameterDef {
            name: "input".to_string(),
            param_type: ParameterType::String,
            description: "Input parameter".to_string(),
            required: true,
            default: None,
        });

    mock.expect_schema()
        .times(1)
        .returning(move || schema.clone());

    mock.expect_validate_parameters()
        .with(eq(serde_json::json!({"input": "test"})))
        .times(1)
        .returning(|_| Ok(()));

    mock.expect_validate_parameters()
        .with(eq(serde_json::json!({})))
        .times(1)
        .returning(|_| {
            Err(LLMSpellError::Validation {
                message: "Missing required parameter: input".to_string(),
                field: Some("input".to_string()),
            })
        });

    // Test schema and validation
    let tool_schema = mock.schema();
    assert_eq!(tool_schema.name, "test_tool");
    assert_eq!(tool_schema.required_parameters(), vec!["input"]);

    assert!(mock
        .validate_parameters(&serde_json::json!({"input": "test"}))
        .await
        .is_ok());
    assert!(mock
        .validate_parameters(&serde_json::json!({}))
        .await
        .is_err());
}
#[tokio::test]
async fn test_workflow_mock_step_execution() {
    let mut mock = MockWorkflow::new();

    // Setup workflow steps
    let component_id1 = ComponentId::from_name("component1");
    let component_id2 = ComponentId::from_name("component2");
    let component_id3 = ComponentId::from_name("component3");

    let steps = vec![
        WorkflowStep::new("step1".to_string(), component_id1),
        WorkflowStep::new("step2".to_string(), component_id2),
        WorkflowStep::new("step3".to_string(), component_id3),
    ];

    let steps_for_mock = steps.clone();
    let steps_for_test = steps.clone();

    mock.expect_add_step()
        .with(always())
        .times(3)
        .returning(|_| Ok(()));

    mock.expect_get_steps()
        .times(1)
        .returning(move || Ok(steps_for_mock.clone()));

    mock.expect_status()
        .times(1)
        .returning(|| Ok(WorkflowStatus::Running));

    // Test workflow execution
    for step in &steps_for_test {
        mock.add_step(step.clone()).await.unwrap();
    }

    let workflow_steps = mock.get_steps().await.unwrap();
    assert_eq!(workflow_steps.len(), 3);

    let status = mock.status().await.unwrap();
    assert_eq!(status, WorkflowStatus::Running);
}
#[test]
fn test_tool_category_enum_variants() {
    // Test all category variants
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
        // Ensure Display trait works
        let _ = category.to_string();

        // Ensure Clone works
        let _ = category.clone();
    }
}
#[test]
fn test_security_level_ordering() {
    assert!(SecurityLevel::Safe < SecurityLevel::Restricted);
    assert!(SecurityLevel::Restricted < SecurityLevel::Privileged);

    // Test allows method
    assert!(SecurityLevel::Privileged.allows(&SecurityLevel::Safe));
    assert!(SecurityLevel::Privileged.allows(&SecurityLevel::Restricted));
    assert!(SecurityLevel::Privileged.allows(&SecurityLevel::Privileged));

    assert!(!SecurityLevel::Safe.allows(&SecurityLevel::Restricted));
    assert!(!SecurityLevel::Safe.allows(&SecurityLevel::Privileged));
}
#[test]
fn test_message_role_variants() {
    let roles = vec![
        MessageRole::System,
        MessageRole::User,
        MessageRole::Assistant,
    ];

    for role in roles {
        // Test Display
        let _ = role.to_string();

        // Test Clone
        let _ = role.clone();

        // Test PartialEq
        assert_eq!(role, role);
    }
}
#[test]
fn test_workflow_status_transitions() {
    // Test all status variants
    let statuses = vec![
        WorkflowStatus::Pending,
        WorkflowStatus::Running,
        WorkflowStatus::Completed,
        WorkflowStatus::Failed,
        WorkflowStatus::Cancelled,
    ];

    for status in statuses {
        // Ensure Display works
        let _ = format!("{:?}", status);

        // Ensure Clone works
        let _ = status.clone();
    }
}
#[test]
fn test_retry_policy_configuration() {
    let default_policy = RetryPolicy::default();
    assert_eq!(default_policy.max_attempts, 3);
    assert_eq!(default_policy.backoff_seconds, 1);
    assert!(default_policy.exponential_backoff);

    let custom_policy = RetryPolicy::new(5, 2, false);
    assert_eq!(custom_policy.max_attempts, 5);
    assert_eq!(custom_policy.backoff_seconds, 2);
    assert!(!custom_policy.exponential_backoff);
}
#[test]
fn test_execution_context_builder() {
    let mut context = ExecutionContext::with_conversation("test-session".to_string());
    context.user_id = Some("user-123".to_string());
    let context = context
        .with_data("KEY1".to_string(), serde_json::json!("value1"))
        .with_data("KEY2".to_string(), serde_json::json!("value2"));

    assert_eq!(context.conversation_id, Some("test-session".to_string()));
    assert_eq!(context.user_id, Some("user-123".to_string()));
    assert_eq!(context.data.get("KEY1"), Some(&serde_json::json!("value1")));
    assert_eq!(context.data.get("KEY2"), Some(&serde_json::json!("value2")));
    assert_eq!(context.data.get("KEY3"), None);
}
#[test]
fn test_agent_config_defaults() {
    let config = AgentConfig::default();

    // Verify default values are sensible
    assert_eq!(config.max_tokens, Some(2000));
    assert_eq!(config.temperature, Some(0.7));
    assert_eq!(config.max_conversation_length, Some(100));
    assert_eq!(config.system_prompt, None);
}
#[test]
fn test_parameter_type_variants() {
    let types = vec![
        ParameterType::String,
        ParameterType::Number,
        ParameterType::Boolean,
        ParameterType::Array,
        ParameterType::Object,
        ParameterType::Null,
    ];

    // Ensure all variants work
    for param_type in types {
        let _ = format!("{:?}", param_type);
        let _ = param_type.clone();
    }
}
#[test]
fn test_conversation_message_creation() {
    let user_msg = ConversationMessage::user("Hello".to_string());
    assert_eq!(user_msg.role, MessageRole::User);
    assert_eq!(user_msg.content, "Hello");

    let assistant_msg = ConversationMessage::assistant("Hi there!".to_string());
    assert_eq!(assistant_msg.role, MessageRole::Assistant);
    assert_eq!(assistant_msg.content, "Hi there!");

    let system_msg = ConversationMessage::system("You are helpful".to_string());
    assert_eq!(system_msg.role, MessageRole::System);
    assert_eq!(system_msg.content, "You are helpful");
}
