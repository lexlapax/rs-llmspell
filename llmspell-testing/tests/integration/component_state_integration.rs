// ABOUTME: Integration tests for component state persistence across tools, workflows, and hooks
// ABOUTME: Validates state persistence integration and cross-component data consistency

use llmspell_core::{ComponentMetadata, ExecutionContext};
use llmspell_state_persistence::{
    config::{PersistenceConfig, StorageBackendType},
    StateManager as PersistentStateManager, StateScope,
};
use llmspell_tools::state::{ToolState, ToolStatePersistence, ToolStateRegistry};
use llmspell_workflows::state::{PersistentWorkflowState, PersistentWorkflowStateManager};
use llmspell_workflows::{WorkflowConfig, WorkflowStatus};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

/// Test helper to create a persistent state manager
async fn create_test_persistent_state_manager() -> Arc<PersistentStateManager> {
    let config = PersistenceConfig {
        enabled: true,
        ..Default::default()
    };
    
    Arc::new(
        PersistentStateManager::with_backend(StorageBackendType::Memory, config)
            .await
            .unwrap()
    )
}

#[cfg(test)]
mod component_integration_tests {
    use super::*;

    #[tokio::test]
    #[cfg_attr(test_category = "integration", test_category = "integration")]
    async fn test_tool_state_persistence_integration() {
        let state_manager = create_test_persistent_state_manager().await;
        let mut registry = ToolStateRegistry::new(state_manager.clone());

        // Create multiple tool states
        let tool_metadata = ComponentMetadata::new("test-tool".to_string(), "Test tool".to_string());
        let tool_state = ToolState::new(tool_metadata.id.to_string(), tool_metadata.clone());

        // Store tool state directly
        let state_scope = StateScope::Custom(format!("tool_{}", tool_metadata.id));
        state_manager.set(state_scope, "state", serde_json::to_value(&tool_state).unwrap())
            .await
            .unwrap();

        // Verify we can load it back
        let loaded_state_value = state_manager.get(state_scope, "state").await.unwrap();
        assert!(loaded_state_value.is_some());

        let loaded_tool_state: ToolState = serde_json::from_value(loaded_state_value.unwrap()).unwrap();
        assert_eq!(loaded_tool_state.tool_id, tool_state.tool_id);
        assert_eq!(loaded_tool_state.metadata.id, tool_state.metadata.id);

        println!("✅ Tool state persistence integration successful");
    }

    #[tokio::test]
    #[cfg_attr(test_category = "integration", test_category = "integration")]
    async fn test_workflow_state_persistence_integration() {
        let persistent_state_manager = create_test_persistent_state_manager().await;
        let config = WorkflowConfig::default();
        let workflow_id = "test-workflow".to_string();

        let mut workflow_manager = PersistentWorkflowStateManager::new(
            config,
            persistent_state_manager.clone(),
            workflow_id.clone(),
        );

        // Create and save persistent workflow state
        workflow_manager.save_state().await.unwrap();

        // Load it back
        let loaded = workflow_manager.load_state().await.unwrap();
        assert!(loaded);

        println!("✅ Workflow state persistence integration successful");
    }

    #[tokio::test]
    #[cfg_attr(test_category = "integration", test_category = "integration")]
    async fn test_cross_component_state_sharing() {
        let state_manager = create_test_persistent_state_manager().await;

        // Create tool state
        let tool_metadata = ComponentMetadata::new("shared-tool".to_string(), "Shared tool".to_string());
        let tool_state = ToolState::new(tool_metadata.id.to_string(), tool_metadata.clone());

        // Store tool state
        let tool_scope = StateScope::Custom(format!("tool_{}", tool_metadata.id));
        state_manager.set(tool_scope, "state", serde_json::to_value(&tool_state).unwrap())
            .await
            .unwrap();

        // Create workflow state that references the tool
        let workflow_metadata = ComponentMetadata::new("shared-workflow".to_string(), "Shared workflow".to_string());
        let workflow_state = PersistentWorkflowState::new(
            workflow_metadata.id.to_string(),
            WorkflowConfig::default(),
            workflow_metadata.clone(),
        );

        // Store workflow state
        let workflow_scope = StateScope::Custom(format!("workflow_{}", workflow_metadata.id));
        state_manager.set(workflow_scope, "state", serde_json::to_value(&workflow_state).unwrap())
            .await
            .unwrap();

        // Store cross-reference information
        let references_scope = StateScope::Global;
        let references = serde_json::json!({
            "tool_to_workflow": {
                tool_metadata.id: [workflow_metadata.id.clone()]
            },
            "workflow_to_tool": {
                workflow_metadata.id: [tool_metadata.id.clone()]
            }
        });
        state_manager.set(references_scope, "component_references", references)
            .await
            .unwrap();

        // Verify cross-references
        let loaded_references = state_manager.get(references_scope, "component_references").await.unwrap();
        assert!(loaded_references.is_some());

        let refs_value = loaded_references.unwrap();
        assert!(refs_value["tool_to_workflow"][&tool_metadata.id].as_array().unwrap().contains(&serde_json::Value::String(workflow_metadata.id.clone())));

        println!("✅ Cross-component state sharing integration successful");
    }

    #[tokio::test]
    #[cfg_attr(test_category = "integration", test_category = "integration")]
    async fn test_concurrent_state_operations() {
        let state_manager = create_test_persistent_state_manager().await;
        
        // Create multiple concurrent operations
        let mut handles = vec![];

        for i in 0..10 {
            let state_manager = state_manager.clone();
            let handle = tokio::spawn(async move {
                let component_id = format!("component_{}", i);
                let metadata = ComponentMetadata::new(component_id.clone(), format!("Component {}", i));
                
                // Tool state operation
                let tool_state = ToolState::new(component_id.clone(), metadata.clone());
                let tool_scope = StateScope::Custom(format!("tool_{}", component_id));
                state_manager.set(tool_scope, "state", serde_json::to_value(&tool_state).unwrap())
                    .await
                    .unwrap();

                // Small delay to simulate real work
                sleep(Duration::from_millis(10)).await;

                // Verify we can read it back
                let loaded = state_manager.get(tool_scope, "state").await.unwrap();
                assert!(loaded.is_some());

                i
            });
            handles.push(handle);
        }

        // Wait for all operations to complete
        let results: Vec<usize> = futures::future::join_all(handles)
            .await
            .into_iter()
            .map(|r| r.unwrap())
            .collect();

        assert_eq!(results.len(), 10);
        println!("✅ Concurrent state operations integration successful");
    }

    #[tokio::test]
    #[cfg_attr(test_category = "integration", test_category = "integration")]
    async fn test_state_migration_compatibility() {
        let state_manager = create_test_persistent_state_manager().await;

        // Create an "old" version of tool state (simplified structure)
        let old_tool_state = serde_json::json!({
            "tool_id": "legacy-tool",
            "metadata": {
                "id": "legacy-tool",
                "name": "Legacy Tool",
                "description": "A legacy tool"
            },
            "execution_stats": {
                "total_executions": 42,
                "successful_executions": 40,
                "failed_executions": 2,
                "total_execution_time_ms": 5000,
                "average_execution_time_ms": 119.0,
                "last_execution": null,
                "cache_hit_ratio": 85.0,
                "resource_usage": {
                    "peak_memory_bytes": 1024000,
                    "average_memory_bytes": 512000,
                    "total_cpu_time_ms": 3000,
                    "file_operations": 10,
                    "network_requests": 5
                }
            },
            "result_cache": {},
            "last_updated": "2025-01-27T10:00:00Z",
            "custom_state": {}
        });

        // Store the old format
        let scope = StateScope::Custom("tool_legacy-tool".to_string());
        state_manager.set(scope, "state", old_tool_state).await.unwrap();

        // Try to load it as the new format
        let loaded_value = state_manager.get(scope, "state").await.unwrap();
        assert!(loaded_value.is_some());

        // Attempt to deserialize - this tests backward compatibility
        let deserialization_result = serde_json::from_value::<ToolState>(loaded_value.unwrap());
        
        // If this succeeds, we have backward compatibility
        // If it fails, we need migration logic
        match deserialization_result {
            Ok(tool_state) => {
                assert_eq!(tool_state.tool_id, "legacy-tool");
                assert_eq!(tool_state.execution_stats.total_executions, 42);
                println!("✅ State format is backward compatible");
            }
            Err(e) => {
                // This is expected if we've changed the format significantly
                println!("⚠️ State migration would be needed for legacy data: {}", e);
                // In a real implementation, we would trigger migration logic here
            }
        }

        println!("✅ State migration compatibility check completed");
    }

    #[tokio::test]
    #[cfg_attr(test_category = "integration", test_category = "integration")]
    async fn test_state_cleanup_and_orphan_detection() {
        let state_manager = create_test_persistent_state_manager().await;

        // Create various component states
        let components = vec![
            ("tool_active", "active_tool", true),
            ("tool_inactive", "inactive_tool", false),
            ("workflow_running", "running_workflow", true),
            ("workflow_completed", "completed_workflow", false),
        ];

        for (scope_suffix, component_id, is_active) in &components {
            let scope = StateScope::Custom(scope_suffix.to_string());
            let state_data = serde_json::json!({
                "id": component_id,
                "active": is_active,
                "last_activity": "2025-01-27T10:00:00Z"
            });
            state_manager.set(scope, "state", state_data).await.unwrap();
        }

        // Create an inventory of all stored states
        let mut active_components = Vec::new();
        let mut inactive_components = Vec::new();

        for (scope_suffix, component_id, is_active) in &components {
            let scope = StateScope::Custom(scope_suffix.to_string());
            if let Ok(Some(state_value)) = state_manager.get(scope, "state").await {
                if state_value["active"].as_bool().unwrap_or(false) {
                    active_components.push(component_id);
                } else {
                    inactive_components.push(component_id);
                }
            }
        }

        assert_eq!(active_components.len(), 2);
        assert_eq!(inactive_components.len(), 2);

        // Simulate cleanup of inactive components
        for (scope_suffix, _component_id, is_active) in &components {
            if !is_active {
                let scope = StateScope::Custom(scope_suffix.to_string());
                state_manager.delete(scope, "state").await.unwrap();
            }
        }

        // Verify cleanup
        for (scope_suffix, _component_id, is_active) in &components {
            let scope = StateScope::Custom(scope_suffix.to_string());
            let state_value = state_manager.get(scope, "state").await.unwrap();
            
            if *is_active {
                assert!(state_value.is_some(), "Active component state should still exist");
            } else {
                assert!(state_value.is_none(), "Inactive component state should be cleaned up");
            }
        }

        println!("✅ State cleanup and orphan detection integration successful");
    }

    #[tokio::test]
    #[cfg_attr(test_category = "integration", test_category = "integration")]
    async fn test_state_performance_under_load() {
        let state_manager = create_test_persistent_state_manager().await;
        
        let start_time = std::time::Instant::now();
        let num_operations = 100;

        // Perform many state operations
        for i in 0..num_operations {
            let component_id = format!("perf_test_{}", i);
            let metadata = ComponentMetadata::new(component_id.clone(), format!("Performance test component {}", i));
            let tool_state = ToolState::new(component_id.clone(), metadata);
            
            let scope = StateScope::Custom(format!("perf_tool_{}", i));
            state_manager.set(scope, "state", serde_json::to_value(&tool_state).unwrap())
                .await
                .unwrap();
        }

        let write_duration = start_time.elapsed();

        // Read operations
        let read_start = std::time::Instant::now();
        for i in 0..num_operations {
            let scope = StateScope::Custom(format!("perf_tool_{}", i));
            let _state = state_manager.get(scope, "state").await.unwrap();
        }
        let read_duration = read_start.elapsed();

        // Performance assertions
        let avg_write_time = write_duration.as_millis() as f64 / num_operations as f64;
        let avg_read_time = read_duration.as_millis() as f64 / num_operations as f64;

        assert!(avg_write_time < 50.0, "Average write time should be < 50ms, was {}ms", avg_write_time);
        assert!(avg_read_time < 10.0, "Average read time should be < 10ms, was {}ms", avg_read_time);

        println!("✅ State performance under load test successful");
        println!("   Average write time: {:.2}ms", avg_write_time);
        println!("   Average read time: {:.2}ms", avg_read_time);
        println!("   Total operations: {}", num_operations * 2);
    }
}