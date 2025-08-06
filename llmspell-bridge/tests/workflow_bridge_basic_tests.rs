//! ABOUTME: Basic integration tests for workflow bridge functionality
//! ABOUTME: Tests script-to-workflow communication without tool dependencies

use llmspell_bridge::workflows::WorkflowBridge;
use llmspell_bridge::ComponentRegistry;
use std::sync::Arc;
#[tokio::test]
async fn test_workflow_bridge_creation() {
    let registry = Arc::new(ComponentRegistry::new());
    let bridge = WorkflowBridge::new(registry);

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
    let bridge = WorkflowBridge::new(registry);

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
    let bridge = WorkflowBridge::new(registry);

    let params = serde_json::json!({
        "name": "test_sequential",
        "steps": []  // Empty steps for now
    });

    let workflow_id = bridge.create_workflow("sequential", params).await.unwrap();
    assert!(workflow_id.starts_with("workflow_"));

    // Test listing active workflows
    let active = bridge.list_active_workflows().await;
    assert_eq!(active.len(), 1);
    assert_eq!(active[0].1, "sequential");
}
#[tokio::test]
async fn test_workflow_metrics() {
    let registry = Arc::new(ComponentRegistry::new());
    let bridge = WorkflowBridge::new(registry);

    // Get initial metrics
    let metrics = bridge.get_bridge_metrics().await;
    assert_eq!(metrics["workflows_created"], 0);
    assert_eq!(metrics["workflow_executions"], 0);

    // Create a workflow
    let params = serde_json::json!({
        "name": "metrics_test",
        "steps": []
    });
    let _ = bridge.create_workflow("sequential", params).await.unwrap();

    // Check updated metrics
    let metrics = bridge.get_bridge_metrics().await;
    assert_eq!(metrics["workflows_created"], 1);
    assert_eq!(metrics["active_workflows"], 1);
}
#[tokio::test]
async fn test_workflow_removal() {
    let registry = Arc::new(ComponentRegistry::new());
    let bridge = WorkflowBridge::new(registry);

    let params = serde_json::json!({
        "name": "removal_test",
        "steps": []
    });

    let workflow_id = bridge.create_workflow("sequential", params).await.unwrap();

    // Verify workflow exists
    let active = bridge.list_active_workflows().await;
    assert_eq!(active.len(), 1);

    // Remove workflow
    bridge.remove_workflow(&workflow_id).await.unwrap();

    // Verify workflow is gone
    let active = bridge.list_active_workflows().await;
    assert_eq!(active.len(), 0);

    // Try to remove non-existent workflow
    assert!(bridge.remove_workflow("non_existent").await.is_err());
}
#[tokio::test]
async fn test_workflow_discovery() {
    let registry = Arc::new(ComponentRegistry::new());
    let bridge = WorkflowBridge::new(registry);

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
    let bridge = WorkflowBridge::new(registry);

    let params = serde_json::json!({
        "name": "parallel_test",
        "branches": [{
            "name": "branch1",
            "steps": []
        }],
        "max_concurrency": 2
    });

    let workflow_id = bridge.create_workflow("parallel", params).await.unwrap();
    assert!(workflow_id.starts_with("workflow_"));
}
#[tokio::test]
async fn test_conditional_workflow_creation() {
    let registry = Arc::new(ComponentRegistry::new());
    let bridge = WorkflowBridge::new(registry);

    let params = serde_json::json!({
        "name": "conditional_test",
        "condition": "true",
        "then_branch": {},
        "else_branch": {}
    });

    let workflow_id = bridge.create_workflow("conditional", params).await.unwrap();
    assert!(workflow_id.starts_with("workflow_"));
}
#[tokio::test]
async fn test_loop_workflow_creation() {
    let registry = Arc::new(ComponentRegistry::new());
    let bridge = WorkflowBridge::new(registry);

    let params = serde_json::json!({
        "name": "loop_test",
        "iterator": {
            "type": "range",
            "start": 0,
            "end": 3,
            "step": 1
        },
        "body": [{
            "name": "loop_step",
            "tool": "mock_tool"
        }]
    });

    let workflow_id = bridge.create_workflow("loop", params).await.unwrap();
    assert!(workflow_id.starts_with("workflow_"));
}
