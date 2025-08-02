// ABOUTME: Test utilities for hook testing including context creation and common fixtures
// ABOUTME: Provides reusable helpers for hook unit and integration tests

use llmspell_hooks::{
    context::HookContext,
    types::{ComponentId, ComponentType, HookPoint},
};
use uuid::Uuid;

/// Create a test hook context with default values
pub fn create_test_hook_context() -> HookContext {
    create_test_hook_context_with_point(HookPoint::BeforeAgentExecution)
}

/// Create a test hook context with specific hook point
pub fn create_test_hook_context_with_point(point: HookPoint) -> HookContext {
    let component_id = ComponentId::new(ComponentType::Agent, "test-agent".to_string());
    HookContext::new(point, component_id)
}

/// Create a test hook context with custom component
pub fn create_test_hook_context_with_component(
    point: HookPoint,
    component_type: ComponentType,
    component_name: &str,
) -> HookContext {
    let component_id = ComponentId::new(component_type, component_name.to_string());
    HookContext::new(point, component_id)
}

/// Create a test hook context with correlation ID
pub fn create_test_hook_context_with_correlation(correlation_id: Uuid) -> HookContext {
    let component_id = ComponentId::new(ComponentType::Agent, "test-agent".to_string());
    let mut context = HookContext::new(HookPoint::BeforeAgentExecution, component_id);
    context.correlation_id = correlation_id;
    context
}

/// Create a test hook context with data
pub fn create_test_hook_context_with_data(
    data_key: &str,
    data_value: serde_json::Value,
) -> HookContext {
    let component_id = ComponentId::new(ComponentType::Agent, "test-agent".to_string());
    let mut context = HookContext::new(HookPoint::BeforeAgentExecution, component_id);
    context.insert_data(data_key.to_string(), data_value);
    context
}

/// Create a test hook context for tool execution
pub fn create_test_tool_hook_context(tool_name: &str) -> HookContext {
    let component_id = ComponentId::new(ComponentType::Tool, tool_name.to_string());
    HookContext::new(HookPoint::BeforeToolExecution, component_id)
}

/// Create a test hook context for workflow execution
pub fn create_test_workflow_hook_context(workflow_name: &str) -> HookContext {
    let component_id = ComponentId::new(ComponentType::Workflow, workflow_name.to_string());
    HookContext::new(HookPoint::BeforeWorkflowExecution, component_id)
}

/// Create a test hook context with error
pub fn create_test_error_hook_context(error_message: &str) -> HookContext {
    let component_id = ComponentId::new(ComponentType::Agent, "test-agent".to_string());
    let mut context = HookContext::new(HookPoint::AgentError, component_id);
    context.insert_data(
        "error".to_string(),
        serde_json::json!({
            "message": error_message,
            "type": "TestError"
        }),
    );
    context
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_hook_context() {
        let context = create_test_hook_context();
        assert_eq!(context.point, HookPoint::BeforeAgentExecution);
        assert_eq!(context.component_id.component_type, ComponentType::Agent);
        assert_eq!(context.component_id.name, "test-agent");
    }

    #[test]
    fn test_create_hook_context_with_point() {
        let context = create_test_hook_context_with_point(HookPoint::AfterToolExecution);
        assert_eq!(context.point, HookPoint::AfterToolExecution);
    }

    #[test]
    fn test_create_hook_context_with_component() {
        let context = create_test_hook_context_with_component(
            HookPoint::BeforeWorkflowExecution,
            ComponentType::Workflow,
            "test-workflow",
        );
        assert_eq!(context.point, HookPoint::BeforeWorkflowExecution);
        assert_eq!(context.component_id.component_type, ComponentType::Workflow);
        assert_eq!(context.component_id.name, "test-workflow");
    }

    #[test]
    fn test_create_hook_context_with_correlation() {
        let correlation_id = Uuid::new_v4();
        let context = create_test_hook_context_with_correlation(correlation_id);
        assert_eq!(context.correlation_id, correlation_id);
    }

    #[test]
    fn test_create_hook_context_with_data() {
        let data = serde_json::json!({ "key": "value" });
        let context = create_test_hook_context_with_data("test_data", data.clone());
        assert_eq!(context.get_data("test_data"), Some(&data));
    }

    #[test]
    fn test_create_tool_hook_context() {
        let context = create_test_tool_hook_context("test-tool");
        assert_eq!(context.point, HookPoint::BeforeToolExecution);
        assert_eq!(context.component_id.component_type, ComponentType::Tool);
        assert_eq!(context.component_id.name, "test-tool");
    }

    #[test]
    fn test_create_workflow_hook_context() {
        let context = create_test_workflow_hook_context("test-workflow");
        assert_eq!(context.point, HookPoint::BeforeWorkflowExecution);
        assert_eq!(context.component_id.component_type, ComponentType::Workflow);
        assert_eq!(context.component_id.name, "test-workflow");
    }

    #[test]
    fn test_create_error_hook_context() {
        let context = create_test_error_hook_context("Test error message");
        assert_eq!(context.point, HookPoint::AgentError);
        let error_data = context.get_data("error").unwrap();
        assert_eq!(error_data["message"], "Test error message");
        assert_eq!(error_data["type"], "TestError");
    }
}