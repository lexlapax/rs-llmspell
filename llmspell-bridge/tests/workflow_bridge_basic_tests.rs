//! ABOUTME: Basic integration tests for workflow bridge functionality
//! ABOUTME: Tests script-to-workflow communication without tool dependencies

use llmspell_bridge::workflows::WorkflowBridge;
use llmspell_bridge::ComponentRegistry;
use llmspell_core::ComponentId;
use llmspell_workflows::{StepType, WorkflowConfig, WorkflowStep};
use std::sync::Arc;
#[tokio::test]
async fn test_workflow_bridge_creation() {
    let registry = Arc::new(ComponentRegistry::new());
    let bridge = WorkflowBridge::new(&registry, None, None);

    // Test listing workflow types
    let types = bridge.list_workflow_types();
    assert_eq!(types.len(), 4);
    assert!(types.contains(&"sequential".to_string()));
    assert!(types.contains(&"parallel".to_string()));
    assert!(types.contains(&"conditional".to_string()));
    assert!(types.contains(&"loop".to_string()));
}
#[tokio::test]
async fn test_workflow_info_retrieval() {
    let registry = Arc::new(ComponentRegistry::new());
    let bridge = WorkflowBridge::new(&registry, None, None);

    // Test getting specific workflow info
    let info = bridge.get_workflow_info("sequential").unwrap();
    assert_eq!(info.workflow_type, "sequential");
    assert!(info.description.contains("Execute steps"));
    assert!(info.required_params.contains(&"steps".to_string()));

    // Test getting all workflow info
    let all_info = bridge.get_all_workflow_info();
    assert_eq!(all_info.len(), 4);
}
#[tokio::test]
async fn test_sequential_workflow_creation() {
    let registry = Arc::new(ComponentRegistry::new());
    let bridge = WorkflowBridge::new(&registry, None, None);

    let name = "test_sequential".to_string();
    let steps = vec![]; // Empty steps for now
    let config = WorkflowConfig::default();

    let workflow_id = bridge
        .create_workflow("sequential", name, steps, config, None)
        .await
        .unwrap();
    assert!(workflow_id.starts_with("workflow_"));

    // Test listing active workflows
    let active = bridge.list_active_workflows().await;
    assert_eq!(active.len(), 1);
    assert_eq!(active[0].1, "sequential");
}
#[tokio::test]
async fn test_workflow_metrics() {
    let registry = Arc::new(ComponentRegistry::new());
    let bridge = WorkflowBridge::new(&registry, None, None);

    // Get initial metrics
    let metrics = bridge.get_bridge_metrics();
    assert_eq!(metrics["workflows_created"], 0);
    assert_eq!(metrics["workflow_executions"], 0);

    // Create a workflow
    let name = "metrics_test".to_string();
    let steps = vec![];
    let config = WorkflowConfig::default();
    let _ = bridge
        .create_workflow("sequential", name, steps, config, None)
        .await
        .unwrap();

    // Check updated metrics
    let metrics = bridge.get_bridge_metrics();
    assert_eq!(metrics["workflows_created"], 1);
    assert_eq!(metrics["registered_workflows"], 1);
}
#[tokio::test]
async fn test_workflow_removal() {
    let registry = Arc::new(ComponentRegistry::new());
    let bridge = WorkflowBridge::new(&registry, None, None);

    let name = "removal_test".to_string();
    let steps = vec![];
    let config = WorkflowConfig::default();

    let workflow_id = bridge
        .create_workflow("sequential", name, steps, config, None)
        .await
        .unwrap();

    // Verify workflow exists
    let active = bridge.list_active_workflows().await;
    assert_eq!(active.len(), 1);

    // Removal is not supported in unified registry architecture
    // Workflows are persistent components that remain available for reuse
    let removal_result = bridge.remove_workflow(&workflow_id);
    assert!(removal_result.is_err());
    assert!(removal_result
        .unwrap_err()
        .to_string()
        .contains("not supported"));

    // Workflow should still exist after attempted removal
    let active = bridge.list_active_workflows().await;
    assert_eq!(active.len(), 1);

    // Try to remove non-existent workflow (also returns error)
    assert!(bridge.remove_workflow("non_existent").is_err());
}
#[tokio::test]
async fn test_workflow_discovery() {
    let registry = Arc::new(ComponentRegistry::new());
    let bridge = WorkflowBridge::new(&registry, None, None);

    let type_infos = bridge.get_all_workflow_info();
    assert_eq!(type_infos.len(), 4);

    // Check each type has proper info
    for (type_name, info) in type_infos {
        assert!(!type_name.is_empty());
        assert!(!info.description.is_empty());
        assert!(info.workflow_type == type_name);
    }
}
#[tokio::test]
async fn test_parallel_workflow_creation() {
    let registry = Arc::new(ComponentRegistry::new());
    let bridge = WorkflowBridge::new(&registry, None, None);

    let name = "parallel_test".to_string();
    let steps = vec![]; // For parallel workflows, steps would be distributed across branches
    let config = WorkflowConfig {
        max_execution_time: Some(std::time::Duration::from_secs(60)),
        ..Default::default()
    };

    let workflow_id = bridge
        .create_workflow("parallel", name, steps, config, None)
        .await
        .unwrap();
    assert!(workflow_id.starts_with("workflow_"));
}
#[tokio::test]
async fn test_conditional_workflow_creation() {
    let registry = Arc::new(ComponentRegistry::new());
    let bridge = WorkflowBridge::new(&registry, None, None);

    let name = "conditional_test".to_string();
    let steps = vec![]; // For conditional workflows, steps would be in branches
    let config = WorkflowConfig::default();

    let workflow_id = bridge
        .create_workflow("conditional", name, steps, config, None)
        .await
        .unwrap();
    assert!(workflow_id.starts_with("workflow_"));
}
#[tokio::test]
async fn test_loop_workflow_creation() {
    let registry = Arc::new(ComponentRegistry::new());
    let bridge = WorkflowBridge::new(&registry, None, None);

    let name = "loop_test".to_string();
    let steps = vec![WorkflowStep {
        id: ComponentId::from_name("loop_step"),
        name: "loop_step".to_string(),
        step_type: StepType::Tool {
            tool_name: "mock_tool".to_string(),
            parameters: serde_json::Value::default(),
        },
        timeout: None,
        retry_attempts: 0,
    }];
    let config = WorkflowConfig::default();

    let workflow_id = bridge
        .create_workflow("loop", name, steps, config, None)
        .await
        .unwrap();
    assert!(workflow_id.starts_with("workflow_"));
}
