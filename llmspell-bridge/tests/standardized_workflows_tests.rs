//! ABOUTME: Integration tests for standardized workflow factory in bridge
//! ABOUTME: Tests workflow creation and execution through the bridge interface

use llmspell_bridge::standardized_workflows::StandardizedWorkflowFactory;
use serde_json::json;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[cfg_attr(feature = "integration-tests", ignore = "integration")]
    #[cfg_attr(feature = "bridge-tests", ignore = "bridge")]
    #[cfg_attr(feature = "workflow-tests", ignore = "workflow")]
    async fn test_standardized_sequential_workflow() {
        let factory = StandardizedWorkflowFactory::new();

        let params = json!({
            "name": "bridge_sequential",
            "timeout": 10000,
            "continue_on_error": false,
            "max_retry_attempts": 2
        });

        let workflow = factory
            .create_from_type_json("sequential", params)
            .await
            .unwrap();

        assert_eq!(workflow.name(), "bridge_sequential");
        assert_eq!(workflow.workflow_type(), "sequential");

        // Test execution
        let input = json!({
            "test": "data"
        });

        let result = workflow.execute(input).await.unwrap();
        assert!(result["success"].as_bool().unwrap_or(false));
    }

    #[tokio::test]
    #[cfg_attr(feature = "integration-tests", ignore = "integration")]
    #[cfg_attr(feature = "bridge-tests", ignore = "bridge")]
    #[cfg_attr(feature = "workflow-tests", ignore = "workflow")]
    async fn test_standardized_parallel_workflow() {
        let factory = StandardizedWorkflowFactory::new();

        let params = json!({
            "name": "bridge_parallel",
            "max_concurrency": 4,
            "fail_fast": true,
            "continue_on_optional_failure": false,
            "timeout": 5000
        });

        let workflow = factory
            .create_from_type_json("parallel", params)
            .await
            .unwrap();

        assert_eq!(workflow.name(), "bridge_parallel");
        assert_eq!(workflow.workflow_type(), "parallel");

        let result = workflow.execute(json!({})).await.unwrap();
        assert!(result["success"].as_bool().unwrap_or(false));
        assert_eq!(result["steps_executed"], 0);
    }

    #[tokio::test]
    #[cfg_attr(feature = "integration-tests", ignore = "integration")]
    #[cfg_attr(feature = "bridge-tests", ignore = "bridge")]
    #[cfg_attr(feature = "workflow-tests", ignore = "workflow")]
    async fn test_standardized_conditional_workflow() {
        let factory = StandardizedWorkflowFactory::new();

        let params = json!({
            "name": "bridge_conditional",
            "timeout": 3000
        });

        let workflow = factory
            .create_from_type_json("conditional", params)
            .await
            .unwrap();

        assert_eq!(workflow.name(), "bridge_conditional");
        assert_eq!(workflow.workflow_type(), "conditional");

        let result = workflow
            .execute(json!({"condition": "test"}))
            .await
            .unwrap();
        assert!(result.is_object());
    }

    #[tokio::test]
    #[cfg_attr(feature = "integration-tests", ignore = "integration")]
    #[cfg_attr(feature = "bridge-tests", ignore = "bridge")]
    #[cfg_attr(feature = "workflow-tests", ignore = "workflow")]
    async fn test_standardized_loop_workflow() {
        let factory = StandardizedWorkflowFactory::new();

        // Test with collection iterator
        let params = json!({
            "name": "bridge_loop_collection",
            "collection": [1, 2, 3],
            "aggregation": "collect_all",
            "continue_on_error": true
        });

        let workflow = factory.create_from_type_json("loop", params).await.unwrap();

        assert_eq!(workflow.name(), "bridge_loop_collection");
        assert_eq!(workflow.workflow_type(), "loop");

        // Test with range iterator
        let params = json!({
            "name": "bridge_loop_range",
            "max_iterations": 5,
            "aggregation": "last_only"
        });

        let workflow = factory.create_from_type_json("loop", params).await.unwrap();

        assert_eq!(workflow.name(), "bridge_loop_range");

        // Test with explicit iterator
        let params = json!({
            "name": "bridge_loop_explicit",
            "iterator": {
                "type": "range",
                "start": 0,
                "end": 3,
                "step": 1
            }
        });

        let workflow = factory.create_from_type_json("loop", params).await.unwrap();

        assert_eq!(workflow.name(), "bridge_loop_explicit");
    }

    #[tokio::test]
    #[cfg_attr(feature = "unit-tests", ignore = "unit")]
    #[cfg_attr(feature = "bridge-tests", ignore = "bridge")]
    #[cfg_attr(feature = "workflow-tests", ignore = "workflow")]
    fn test_list_workflow_types() {
        let factory = StandardizedWorkflowFactory::new();
        let types = factory.list_workflow_types();

        assert_eq!(types.len(), 4);
        assert!(types.contains(&"sequential".to_string()));
        assert!(types.contains(&"parallel".to_string()));
        assert!(types.contains(&"conditional".to_string()));
        assert!(types.contains(&"loop".to_string()));
    }

    #[tokio::test]
    #[cfg_attr(feature = "integration-tests", ignore = "integration")]
    #[cfg_attr(feature = "bridge-tests", ignore = "bridge")]
    #[cfg_attr(feature = "workflow-tests", ignore = "workflow")]
    async fn test_workflow_with_default_name() {
        let factory = StandardizedWorkflowFactory::new();

        // Don't provide name, should use workflow type as default
        let params = json!({
            "timeout": 1000
        });

        let workflow = factory
            .create_from_type_json("sequential", params)
            .await
            .unwrap();

        assert_eq!(workflow.name(), "sequential");
    }

    #[tokio::test]
    #[cfg_attr(feature = "integration-tests", ignore = "integration")]
    #[cfg_attr(feature = "bridge-tests", ignore = "bridge")]
    #[cfg_attr(feature = "workflow-tests", ignore = "workflow")]
    async fn test_unknown_workflow_type() {
        let factory = StandardizedWorkflowFactory::new();

        let params = json!({
            "name": "unknown"
        });

        let result = factory.create_from_type_json("unknown_type", params).await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Unknown workflow type"));
    }

    #[tokio::test]
    #[cfg_attr(feature = "integration-tests", ignore = "integration")]
    #[cfg_attr(feature = "bridge-tests", ignore = "bridge")]
    #[cfg_attr(feature = "workflow-tests", ignore = "workflow")]
    async fn test_workflow_output_format() {
        let factory = StandardizedWorkflowFactory::new();

        let params = json!({
            "name": "output_test"
        });

        let workflow = factory
            .create_from_type_json("sequential", params)
            .await
            .unwrap();

        let result = workflow.execute(json!({})).await.unwrap();

        // Verify output format matches expected structure
        assert!(result.get("success").is_some());
        assert!(result.get("output").is_some());
        assert!(result.get("steps_executed").is_some());
        assert!(result.get("steps_failed").is_some());
        assert!(result.get("duration_ms").is_some());
        assert!(result.get("error").is_some());
    }
}
