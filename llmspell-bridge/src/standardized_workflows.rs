//! ABOUTME: Standardized workflow integration using llmspell-workflows factory
//! ABOUTME: Replaces ad-hoc workflow creation with factory-based approach

use llmspell_core::{traits::base_agent::BaseAgent, Result};
use llmspell_workflows::{
    adapters::{WorkflowInputAdapter, WorkflowOutputAdapter},
    factory::{DefaultWorkflowFactory, WorkflowFactory},
    types::{WorkflowConfig, WorkflowInput},
};
use std::sync::Arc;
use std::time::Duration;

/// Standardized workflow factory using llmspell-workflows
pub struct StandardizedWorkflowFactory {
    factory: Arc<DefaultWorkflowFactory>,
    registry: Option<Arc<super::ComponentRegistry>>,
}

impl StandardizedWorkflowFactory {
    #[must_use]
    pub fn new() -> Self {
        Self {
            factory: Arc::new(DefaultWorkflowFactory::new()),
            registry: None,
        }
    }

    #[must_use]
    pub fn new_with_registry(registry: Arc<super::ComponentRegistry>) -> Self {
        Self {
            factory: Arc::new(DefaultWorkflowFactory::new()),
            registry: Some(registry),
        }
    }

    /// Create a workflow from type string and JSON parameters
    /// This is the bridge-specific method that converts JSON params to workflow configuration
    ///
    /// # Errors
    ///
    /// Returns an error if workflow creation fails or parameters are invalid
    #[allow(clippy::too_many_lines)]
    pub async fn create_from_type_json(
        &self,
        workflow_type: &str,
        params: serde_json::Value,
    ) -> Result<Box<dyn super::workflows::WorkflowExecutor>> {
        // Extract common parameters
        let name = params
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or(workflow_type)
            .to_string();

        // Build workflow configuration
        let mut config = WorkflowConfig::default();

        if let Some(timeout_ms) = params.get("timeout").and_then(serde_json::Value::as_u64) {
            config.max_execution_time = Some(Duration::from_millis(timeout_ms));
        }

        if let Some(continue_on_error) = params
            .get("continue_on_error")
            .and_then(serde_json::Value::as_bool)
        {
            config.continue_on_error = continue_on_error;
        }

        if let Some(max_retries) = params
            .get("max_retry_attempts")
            .and_then(serde_json::Value::as_u64)
        {
            config.max_retry_attempts = u32::try_from(max_retries).unwrap_or(u32::MAX);
        }

        // Extract type-specific configuration
        let type_config = match workflow_type {
            "sequential" => {
                // Sequential needs steps passed in type_config for the factory
                let steps = params
                    .get("steps")
                    .cloned()
                    .unwrap_or_else(|| serde_json::json!([]));
                serde_json::json!({
                    "steps": steps
                })
            }
            "parallel" => {
                let mut parallel_config = serde_json::json!({
                    "max_concurrency": params.get("max_concurrency").and_then(serde_json::Value::as_u64).unwrap_or(4),
                    "fail_fast": params.get("fail_fast").and_then(serde_json::Value::as_bool).unwrap_or(false),
                    "continue_on_optional_failure": params.get("continue_on_optional_failure").and_then(serde_json::Value::as_bool).unwrap_or(true),
                });

                // Pass through branches array if provided - CRITICAL for parallel workflows
                if let Some(branches) = params.get("branches") {
                    parallel_config["branches"] = branches.clone();
                }

                parallel_config
            }
            "conditional" => {
                // Pass through branches and other conditional-specific config
                let mut conditional_config = serde_json::json!({});

                // Pass through branches array if provided
                if let Some(branches) = params.get("branches") {
                    conditional_config["branches"] = branches.clone();
                }

                // Pass through default branch if provided
                if let Some(default_branch) = params.get("default_branch") {
                    conditional_config["default_branch"] = default_branch.clone();
                }

                conditional_config
            }
            "loop" => {
                let mut loop_config = serde_json::json!({
                    "body": [],
                    "break_conditions": [],
                    "aggregation": params.get("aggregation").and_then(serde_json::Value::as_str).unwrap_or("collect_all"),
                    "continue_on_error": params.get("continue_on_error").and_then(serde_json::Value::as_bool).unwrap_or(false),
                });

                // Handle iterator configuration
                if let Some(iterator) = params.get("iterator") {
                    loop_config["iterator"] = iterator.clone();
                } else if let Some(collection) = params.get("collection") {
                    loop_config["iterator"] = serde_json::json!({
                        "type": "collection",
                        "items": collection
                    });
                } else if let Some(max_iterations) = params
                    .get("max_iterations")
                    .and_then(serde_json::Value::as_u64)
                {
                    loop_config["iterator"] = serde_json::json!({
                        "type": "range",
                        "start": 0,
                        "end": max_iterations,
                        "step": 1
                    });
                }

                loop_config
            }
            _ => serde_json::json!({}),
        };

        // For conditional workflows, bypass the broken factory and use working implementation
        if workflow_type == "conditional" {
            // Use working conditional workflow creation directly
            let mut conditional_params = serde_json::json!({
                "name": name.clone()
            });

            // Merge type_config (which contains branches) into params
            if let serde_json::Value::Object(type_config_obj) = type_config {
                if let serde_json::Value::Object(ref mut params_obj) = conditional_params {
                    for (key, value) in type_config_obj {
                        params_obj.insert(key, value);
                    }
                }
            }

            // Use the working create_conditional_workflow function that properly handles branches
            let conditional_workflow = super::workflows::create_conditional_workflow(
                &conditional_params,
                self.registry.clone(),
            )?;
            return Ok(Box::new(conditional_workflow));
        }

        // For parallel workflows, also bypass the factory and use working implementation
        if workflow_type == "parallel" {
            // Use working parallel workflow creation directly
            // Pass through the original params which already contains branches
            let parallel_workflow =
                super::workflows::create_parallel_workflow(&params, self.registry.clone())?;
            return Ok(Box::new(parallel_workflow));
        }

        // For sequential and loop workflows, bypass factory and use working implementation with registry
        if workflow_type == "sequential" {
            let sequential_workflow =
                super::workflows::create_sequential_workflow(&params, self.registry.clone())?;
            return Ok(Box::new(sequential_workflow));
        }

        if workflow_type == "loop" {
            let loop_workflow =
                super::workflows::create_loop_workflow(&params, self.registry.clone())?;
            return Ok(Box::new(loop_workflow));
        }

        // For other workflow types, use the standardized factory (should not reach here)
        let workflow = self
            .factory
            .create_from_type(workflow_type, name.clone(), config, type_config)
            .await?;

        // Wrap in bridge executor
        Ok(Box::new(StandardizedWorkflowExecutor {
            workflow,
            name,
            workflow_type: workflow_type.to_string(),
        }))
    }

    /// List available workflow types
    #[must_use]
    pub fn list_workflow_types(&self) -> Vec<String> {
        self.factory.list_workflow_types()
    }
}

impl Default for StandardizedWorkflowFactory {
    fn default() -> Self {
        Self::new()
    }
}

/// Generic workflow executor wrapper for bridge compatibility
struct StandardizedWorkflowExecutor {
    workflow: Arc<dyn BaseAgent + Send + Sync>,
    name: String,
    workflow_type: String,
}

#[async_trait::async_trait]
impl super::workflows::WorkflowExecutor for StandardizedWorkflowExecutor {
    async fn execute(&self, input: serde_json::Value) -> Result<serde_json::Value> {
        // Convert JSON input to WorkflowInput
        let workflow_input = WorkflowInput {
            input: input.clone(),
            context: std::collections::HashMap::new(),
            timeout: None,
        };

        // Convert to AgentInput
        let agent_input = WorkflowInputAdapter::to_agent_input(workflow_input);

        // Execute through BaseAgent interface
        let context = llmspell_core::execution_context::ExecutionContext::new();
        let agent_output = self.workflow.execute(agent_input, context).await?;

        // Convert output back to WorkflowOutput
        let workflow_output = WorkflowOutputAdapter::from_agent_output(
            agent_output,
            Duration::from_secs(0), // Duration will be tracked by the output
        );

        // Convert to JSON for bridge
        let result = serde_json::json!({
            "success": workflow_output.success,
            "output": workflow_output.output,
            "steps_executed": workflow_output.steps_executed,
            "steps_failed": workflow_output.steps_failed,
            "duration_ms": workflow_output.duration.as_millis(),
            "error": workflow_output.error,
        });

        Ok(result)
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn workflow_type(&self) -> &str {
        &self.workflow_type
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_standardized_factory_creation() {
        let factory = StandardizedWorkflowFactory::new();
        let types = factory.list_workflow_types();
        assert_eq!(types.len(), 4);
        assert!(types.contains(&"sequential".to_string()));
        assert!(types.contains(&"parallel".to_string()));
        assert!(types.contains(&"conditional".to_string()));
        assert!(types.contains(&"loop".to_string()));
    }

    #[tokio::test]
    async fn test_create_sequential_workflow() {
        let factory = StandardizedWorkflowFactory::new();
        let params = serde_json::json!({
            "name": "test_sequential",
            "timeout": 5000,
            "continue_on_error": false,
        });

        let workflow = factory
            .create_from_type_json("sequential", params)
            .await
            .unwrap();
        assert_eq!(workflow.name(), "test_sequential");
        assert_eq!(workflow.workflow_type(), "sequential");
    }

    #[tokio::test]
    async fn test_create_parallel_workflow() {
        let factory = StandardizedWorkflowFactory::new();
        let params = serde_json::json!({
            "name": "test_parallel",
            "branches": [
                {
                    "name": "branch1",
                    "steps": []
                },
                {
                    "name": "branch2",
                    "steps": []
                }
            ],
            "max_concurrency": 8,
            "fail_fast": true,
        });

        let workflow = factory
            .create_from_type_json("parallel", params)
            .await
            .unwrap();
        assert_eq!(workflow.name(), "test_parallel");
        assert_eq!(workflow.workflow_type(), "parallel");
    }
}
