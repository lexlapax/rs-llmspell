//! ABOUTME: Integration tests for WorkflowBridge with standardized factory
//! ABOUTME: Tests end-to-end workflow lifecycle through the bridge

use llmspell_bridge::{workflows::WorkflowBridge, ComponentRegistry};
use serde_json::json;
use std::sync::Arc;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[cfg_attr(feature = "integration-tests", ignore = "integration")]
    #[cfg_attr(feature = "bridge-tests", ignore = "bridge")]
    #[cfg_attr(feature = "workflow-tests", ignore = "workflow")]
    #[ignore = "Workflow execution without steps not implemented - placeholder test"]
    async fn test_workflow_lifecycle_through_bridge() {
        let registry = Arc::new(ComponentRegistry::new());
        let bridge = WorkflowBridge::new(registry);

        // Create a sequential workflow
        let params = json!({
            "name": "test_sequential",
            "steps": []
        });

        let workflow_id = bridge.create_workflow("sequential", params).await.unwrap();

        assert!(workflow_id.starts_with("workflow_"));

        // List active workflows
        let active = bridge.list_active_workflows().await;
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].1, "sequential");

        // Execute workflow
        let input = json!({
            "test": "data"
        });

        let result = bridge.execute_workflow(&workflow_id, input).await.unwrap();
        assert!(result["success"].as_bool().unwrap_or(false));

        // Get execution history
        let history = bridge.get_execution_history().await;
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].workflow_id, workflow_id);
        assert!(history[0].success);

        // Remove workflow
        bridge.remove_workflow(&workflow_id).await.unwrap();
        let active = bridge.list_active_workflows().await;
        assert_eq!(active.len(), 0);
    }

    #[tokio::test]
    #[cfg_attr(feature = "integration-tests", ignore = "integration")]
    #[cfg_attr(feature = "bridge-tests", ignore = "bridge")]
    #[cfg_attr(feature = "workflow-tests", ignore = "workflow")]
    #[ignore = "Workflow oneshot execution not implemented - placeholder test"]
    async fn test_oneshot_workflow_execution() {
        let registry = Arc::new(ComponentRegistry::new());
        let bridge = WorkflowBridge::new(registry);

        let params = json!({
            "name": "oneshot_parallel",
            "max_concurrency": 2,
            "branches": []
        });

        let input = json!({
            "data": "test"
        });

        let result = bridge
            .execute_workflow_oneshot("parallel", params, input)
            .await
            .unwrap();

        assert!(result["success"].as_bool().unwrap_or(false));

        // Workflow should not be in active list
        let active = bridge.list_active_workflows().await;
        assert_eq!(active.len(), 0);
    }

    #[tokio::test]
    #[cfg_attr(feature = "integration-tests", ignore = "integration")]
    #[cfg_attr(feature = "bridge-tests", ignore = "bridge")]
    #[cfg_attr(feature = "workflow-tests", ignore = "workflow")]
    #[ignore = "Workflow metrics tracking not implemented - placeholder test"]
    async fn test_workflow_metrics() {
        let registry = Arc::new(ComponentRegistry::new());
        let bridge = WorkflowBridge::new(registry);

        // Get initial metrics
        let metrics = bridge.get_bridge_metrics().await;
        assert_eq!(metrics["workflows_created"], 0);
        assert_eq!(metrics["workflow_executions"], 0);

        // Create and execute workflow
        let workflow_id = bridge
            .create_workflow("sequential", json!({"name": "metrics_test"}))
            .await
            .unwrap();

        bridge
            .execute_workflow(&workflow_id, json!({}))
            .await
            .unwrap();

        // Check updated metrics
        let metrics = bridge.get_bridge_metrics().await;
        assert_eq!(metrics["workflows_created"], 1);
        assert_eq!(metrics["workflow_executions"], 1);
        assert_eq!(metrics["successful_executions"], 1);
        assert_eq!(metrics["failed_executions"], 0);
        assert!(metrics["avg_execution_time_ms"].as_u64().unwrap() > 0);

        // Check performance metrics
        let perf = metrics["performance"].as_object().unwrap();
        assert!(perf.contains_key("average_operation_ms"));
        assert!(perf.contains_key("p99_operation_ms"));
        assert!(perf.contains_key("within_bounds"));

        bridge.remove_workflow(&workflow_id).await.unwrap();
    }

    #[tokio::test]
    #[cfg_attr(feature = "integration-tests", ignore = "integration")]
    #[cfg_attr(feature = "bridge-tests", ignore = "bridge")]
    #[cfg_attr(feature = "workflow-tests", ignore = "workflow")]
    async fn test_workflow_type_discovery() {
        let registry = Arc::new(ComponentRegistry::new());
        let bridge = WorkflowBridge::new(registry);

        // List available workflow types
        let types = bridge.list_workflow_types();
        assert_eq!(types.len(), 4);
        assert!(types.contains(&"sequential".to_string()));
        assert!(types.contains(&"parallel".to_string()));
        assert!(types.contains(&"conditional".to_string()));
        assert!(types.contains(&"loop".to_string()));

        // Get info for a specific type
        let info = bridge.get_workflow_info("sequential").unwrap();
        assert_eq!(info.workflow_type, "sequential");
        assert!(info.required_params.contains(&"steps".to_string()));

        // Get all workflow info
        let all_info = bridge.get_all_workflow_info();
        assert_eq!(all_info.len(), 4);
    }

    #[tokio::test]
    #[cfg_attr(feature = "integration-tests", ignore = "integration")]
    #[cfg_attr(feature = "bridge-tests", ignore = "bridge")]
    #[cfg_attr(feature = "workflow-tests", ignore = "workflow")]
    async fn test_workflow_error_handling() {
        let registry = Arc::new(ComponentRegistry::new());
        let bridge = WorkflowBridge::new(registry);

        // Try to execute non-existent workflow
        let result = bridge.execute_workflow("non_existent_id", json!({})).await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("No active workflow"));

        // Try to remove non-existent workflow
        let result = bridge.remove_workflow("non_existent_id").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    #[cfg_attr(feature = "integration-tests", ignore = "integration")]
    #[cfg_attr(feature = "bridge-tests", ignore = "bridge")]
    #[cfg_attr(feature = "workflow-tests", ignore = "workflow")]
    #[ignore = "Workflow execution history not implemented - placeholder test"]
    async fn test_execution_history_management() {
        let registry = Arc::new(ComponentRegistry::new());
        let bridge = WorkflowBridge::new(registry);

        // Execute multiple workflows
        for i in 0..5 {
            let params = json!({
                "name": format!("test_workflow_{}", i)
            });
            bridge
                .execute_workflow_oneshot("sequential", params, json!({}))
                .await
                .unwrap();
        }

        // Check history
        let history = bridge.get_execution_history().await;
        assert_eq!(history.len(), 5);

        // All should be successful
        for record in &history {
            assert!(record.success);
            assert!(record.end_time.is_some());
            assert!(record.duration_ms.is_some());
        }

        // Clear history
        bridge.clear_execution_history().await;
        let history = bridge.get_execution_history().await;
        assert_eq!(history.len(), 0);
    }

    #[tokio::test]
    #[cfg_attr(feature = "integration-tests", ignore = "integration")]
    #[cfg_attr(feature = "bridge-tests", ignore = "bridge")]
    #[cfg_attr(feature = "workflow-tests", ignore = "workflow")]
    #[ignore = "Workflow concurrent execution not implemented - placeholder test"]
    async fn test_concurrent_workflow_execution() {
        let registry = Arc::new(ComponentRegistry::new());
        let bridge = Arc::new(WorkflowBridge::new(registry));

        // Create multiple workflows
        let mut handles = vec![];

        for i in 0..3 {
            let bridge_clone = bridge.clone();
            let handle = tokio::spawn(async move {
                let params = json!({
                    "name": format!("concurrent_{}", i)
                });

                let workflow_id = bridge_clone
                    .create_workflow("sequential", params)
                    .await
                    .unwrap();

                let result = bridge_clone
                    .execute_workflow(&workflow_id, json!({"index": i}))
                    .await
                    .unwrap();

                bridge_clone.remove_workflow(&workflow_id).await.unwrap();
                result
            });
            handles.push(handle);
        }

        // Wait for all executions
        for handle in handles {
            let result = handle.await.unwrap();
            assert!(result["success"].as_bool().unwrap_or(false));
        }

        // Check metrics
        let metrics = bridge.get_bridge_metrics().await;
        assert_eq!(metrics["workflows_created"], 3);
        assert_eq!(metrics["workflow_executions"], 3);
        assert_eq!(metrics["successful_executions"], 3);
    }
}
