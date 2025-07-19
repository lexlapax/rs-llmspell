//! ABOUTME: Basic sequential workflow implementation
//! ABOUTME: Executes workflow steps in sequential order with memory-based state management

use super::error_handling::{BasicErrorHandler, ErrorAction};
use super::state::BasicStateManager;
use super::step_executor::BasicStepExecutor;
use super::traits::{
    BasicErrorStrategy, BasicStepResult, BasicStepType, BasicWorkflow, BasicWorkflowStatus,
    BasicWorkflowStep,
};
use super::types::{BasicWorkflowConfig, StepExecutionContext};
use async_trait::async_trait;
use llmspell_core::{ComponentId, LLMSpellError, Result};
use std::time::Instant;
use tracing::{debug, error, info, warn};

/// Basic sequential workflow that executes steps in order
///
/// This workflow pattern executes steps sequentially, maintaining order and
/// allowing each step to access the results of previous steps through shared state.
/// It uses memory-based state management and supports retry strategies and error handling.
///
/// # Features
/// - Sequential step execution
/// - Memory-based state management
/// - Step result sharing between steps
/// - Configurable error handling strategies
/// - Retry logic with exponential backoff
/// - Execution timeout management
/// - Comprehensive logging and monitoring
///
/// # Examples
///
/// ```rust
/// use llmspell_workflows::{
///     BasicWorkflow,
///     basic::{
///         BasicSequentialWorkflow, BasicWorkflowStep, BasicStepType, BasicWorkflowConfig
///     }
/// };
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let config = BasicWorkflowConfig::default();
///     let mut workflow = BasicSequentialWorkflow::new("data_pipeline".to_string(), config);
///     
///     // Add steps
///     let step1 = BasicWorkflowStep::new(
///         "load_data".to_string(),
///         BasicStepType::Tool {
///             tool_name: "file_operations".to_string(),
///             parameters: serde_json::json!({"operation": "read", "path": "data.json"}),
///         },
///     );
///     
///     workflow.add_step(step1).await?;
///     
///     // Execute workflow
///     let results = workflow.execute().await?;
///     println!("Workflow completed with {} steps", results.len());
///     
///     Ok(())
/// }
/// ```
pub struct BasicSequentialWorkflow {
    name: String,
    steps: Vec<BasicWorkflowStep>,
    state_manager: BasicStateManager,
    step_executor: BasicStepExecutor,
    error_handler: BasicErrorHandler,
    error_strategy: BasicErrorStrategy,
}

impl BasicSequentialWorkflow {
    /// Create a new basic sequential workflow
    pub fn new(name: String, config: BasicWorkflowConfig) -> Self {
        let error_strategy = if config.continue_on_error {
            BasicErrorStrategy::Continue
        } else {
            BasicErrorStrategy::FailFast
        };

        let state_manager = BasicStateManager::new(config.clone());
        let step_executor = BasicStepExecutor::new(config.clone());
        let error_handler = BasicErrorHandler::new(error_strategy.clone());

        Self {
            name,
            steps: Vec::new(),
            state_manager,
            step_executor,
            error_handler,
            error_strategy,
        }
    }

    /// Create a new workflow with custom error strategy
    pub fn with_error_strategy(
        name: String,
        config: BasicWorkflowConfig,
        error_strategy: BasicErrorStrategy,
    ) -> Self {
        let state_manager = BasicStateManager::new(config.clone());
        let step_executor = BasicStepExecutor::new(config.clone());
        let error_handler = BasicErrorHandler::new(error_strategy.clone());

        Self {
            name,
            steps: Vec::new(),
            state_manager,
            step_executor,
            error_handler,
            error_strategy,
        }
    }

    /// Get the number of steps in the workflow
    pub fn step_count(&self) -> usize {
        self.steps.len()
    }

    /// Check if workflow is empty (no steps)
    pub fn is_empty(&self) -> bool {
        self.steps.is_empty()
    }

    /// Get workflow configuration through state manager
    pub async fn get_config(&self) -> Result<BasicWorkflowConfig> {
        // For now, we'll return a default config - in a full implementation,
        // we'd store the config in the state manager
        Ok(BasicWorkflowConfig::default())
    }

    /// Execute workflow with detailed progress tracking
    async fn execute_with_tracking(&mut self) -> Result<Vec<BasicStepResult>> {
        info!("Starting execution of workflow '{}'", self.name);
        let execution_start = Instant::now();

        // Start execution tracking
        self.state_manager.start_execution().await?;

        let mut results = Vec::new();
        let total_steps = self.steps.len();

        for (index, step) in self.steps.iter().enumerate() {
            info!(
                "Executing step {}/{}: '{}' (id: {:?})",
                index + 1,
                total_steps,
                step.name,
                step.id
            );

            // Check for execution timeout before each step
            if self.state_manager.check_execution_timeout().await? {
                error!("Workflow execution timed out before step '{}'", step.name);
                self.state_manager.complete_execution(false).await?;
                return Err(LLMSpellError::Timeout {
                    message: "Workflow execution exceeded maximum time limit".to_string(),
                    duration_ms: None,
                });
            }

            // Create execution context with current workflow state
            let workflow_state = self.state_manager.get_state_snapshot().await?;
            let context = StepExecutionContext::new(workflow_state, step.timeout);

            // Execute step with retry logic
            let step_result = self
                .step_executor
                .execute_step_with_retry(step, context, &self.error_strategy)
                .await?;

            // Record the result
            self.state_manager
                .record_step_result(step_result.clone())
                .await?;
            results.push(step_result.clone());

            if step_result.success {
                info!(
                    "Step '{}' completed successfully in {:?}",
                    step.name, step_result.duration
                );
                self.state_manager.advance_step().await?;
            } else {
                warn!(
                    "Step '{}' failed: {}",
                    step.name,
                    step_result
                        .error
                        .as_ref()
                        .unwrap_or(&"Unknown error".to_string())
                );

                // Handle step failure based on error strategy
                let action = self
                    .error_handler
                    .handle_step_failure(&step_result, Some(&self.error_strategy))
                    .await?;

                match action {
                    ErrorAction::StopWorkflow => {
                        error!("Stopping workflow due to step failure: '{}'", step.name);
                        self.state_manager.complete_execution(false).await?;
                        break;
                    }
                    ErrorAction::ContinueToNext => {
                        warn!("Continuing to next step despite failure in '{}'", step.name);
                        self.state_manager.advance_step().await?;
                    }
                    ErrorAction::RetryStep => {
                        // This case is handled by execute_step_with_retry
                        warn!("Step '{}' will be retried", step.name);
                        self.state_manager.advance_step().await?;
                    }
                }
            }
        }

        let execution_duration = execution_start.elapsed();
        let successful_steps = results.iter().filter(|r| r.success).count();
        let workflow_success = successful_steps == results.len() && !results.is_empty();

        info!(
            "Workflow '{}' completed in {:?}: {}/{} steps successful",
            self.name,
            execution_duration,
            successful_steps,
            results.len()
        );

        // Complete execution tracking
        self.state_manager
            .complete_execution(workflow_success)
            .await?;

        Ok(results)
    }

    /// Get execution statistics
    pub async fn get_stats(&self) -> Result<super::state::ExecutionStats> {
        self.state_manager.get_execution_stats().await
    }

    /// Get current workflow state snapshot
    pub async fn get_state_snapshot(&self) -> Result<super::types::BasicWorkflowState> {
        self.state_manager.get_state_snapshot().await
    }

    /// Access shared data from workflow state
    pub async fn get_shared_data(&self, key: &str) -> Result<Option<serde_json::Value>> {
        self.state_manager.get_shared_data(key).await
    }

    /// Set shared data in workflow state
    pub async fn set_shared_data(&self, key: String, value: serde_json::Value) -> Result<()> {
        self.state_manager.set_shared_data(key, value).await
    }

    /// Get output from a specific step
    pub async fn get_step_output(&self, step_id: ComponentId) -> Result<Option<serde_json::Value>> {
        self.state_manager.get_step_output(step_id).await
    }
}

#[async_trait]
impl BasicWorkflow for BasicSequentialWorkflow {
    fn name(&self) -> &str {
        &self.name
    }

    async fn status(&self) -> Result<BasicWorkflowStatus> {
        self.state_manager.get_status().await
    }

    async fn add_step(&mut self, step: BasicWorkflowStep) -> Result<()> {
        debug!("Adding step '{}' to workflow '{}'", step.name, self.name);
        self.steps.push(step);
        Ok(())
    }

    async fn remove_step(&mut self, step_id: ComponentId) -> Result<()> {
        let initial_len = self.steps.len();
        self.steps.retain(|step| step.id != step_id);

        if self.steps.len() < initial_len {
            debug!("Removed step {:?} from workflow '{}'", step_id, self.name);
            Ok(())
        } else {
            Err(LLMSpellError::Workflow {
                message: format!("Step with id {:?} not found in workflow", step_id),
                step: None,
                source: None,
            })
        }
    }

    async fn get_steps(&self) -> Result<Vec<BasicWorkflowStep>> {
        Ok(self.steps.clone())
    }

    async fn execute(&mut self) -> Result<Vec<BasicStepResult>> {
        // Validate workflow before execution
        self.validate().await?;

        // Execute with detailed tracking
        self.execute_with_tracking().await
    }

    async fn get_results(&self) -> Result<Vec<BasicStepResult>> {
        self.state_manager.get_execution_history().await
    }

    async fn reset(&mut self) -> Result<()> {
        debug!("Resetting workflow '{}'", self.name);
        self.state_manager.reset().await
    }

    async fn validate(&self) -> Result<()> {
        if self.steps.is_empty() {
            return Err(LLMSpellError::Workflow {
                message: format!("Workflow '{}' has no steps", self.name),
                step: None,
                source: None,
            });
        }

        // Validate each step has required fields
        for (index, step) in self.steps.iter().enumerate() {
            if step.name.is_empty() {
                return Err(LLMSpellError::Workflow {
                    message: format!("Step at index {} has empty name", index),
                    step: Some(format!("step_{}", index)),
                    source: None,
                });
            }

            // Validate step type specific requirements
            match &step.step_type {
                BasicStepType::Tool { tool_name, .. } => {
                    if tool_name.is_empty() {
                        return Err(LLMSpellError::Workflow {
                            message: format!("Tool step '{}' has empty tool name", step.name),
                            step: Some(step.name.clone()),
                            source: None,
                        });
                    }
                }
                BasicStepType::Agent { input, .. } => {
                    if input.is_empty() {
                        return Err(LLMSpellError::Workflow {
                            message: format!("Agent step '{}' has empty input", step.name),
                            step: Some(step.name.clone()),
                            source: None,
                        });
                    }
                }
                BasicStepType::Custom { function_name, .. } => {
                    if function_name.is_empty() {
                        return Err(LLMSpellError::Workflow {
                            message: format!("Custom step '{}' has empty function name", step.name),
                            step: Some(step.name.clone()),
                            source: None,
                        });
                    }
                }
            }
        }

        debug!(
            "Workflow '{}' validation passed with {} steps",
            self.name,
            self.steps.len()
        );
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_basic_sequential_workflow_creation() {
        let config = BasicWorkflowConfig::default();
        let workflow = BasicSequentialWorkflow::new("test_workflow".to_string(), config);

        assert_eq!(workflow.name(), "test_workflow");
        assert_eq!(workflow.step_count(), 0);
        assert!(workflow.is_empty());
        assert_eq!(
            workflow.status().await.unwrap(),
            BasicWorkflowStatus::Pending
        );
    }

    #[tokio::test]
    async fn test_step_management() {
        let config = BasicWorkflowConfig::default();
        let mut workflow = BasicSequentialWorkflow::new("test_workflow".to_string(), config);

        // Add step
        let step = BasicWorkflowStep::new(
            "test_step".to_string(),
            BasicStepType::Tool {
                tool_name: "calculator".to_string(),
                parameters: serde_json::json!({"expression": "2 + 2"}),
            },
        );
        let step_id = step.id;

        workflow.add_step(step).await.unwrap();
        assert_eq!(workflow.step_count(), 1);
        assert!(!workflow.is_empty());

        // Get steps
        let steps = workflow.get_steps().await.unwrap();
        assert_eq!(steps.len(), 1);
        assert_eq!(steps[0].name, "test_step");

        // Remove step
        workflow.remove_step(step_id).await.unwrap();
        assert_eq!(workflow.step_count(), 0);
        assert!(workflow.is_empty());
    }

    #[tokio::test]
    async fn test_workflow_validation() {
        let config = BasicWorkflowConfig::default();
        let mut workflow = BasicSequentialWorkflow::new("test_workflow".to_string(), config);

        // Empty workflow should fail validation
        let result = workflow.validate().await;
        assert!(result.is_err());

        // Add valid step
        let step = BasicWorkflowStep::new(
            "valid_step".to_string(),
            BasicStepType::Tool {
                tool_name: "calculator".to_string(),
                parameters: serde_json::json!({}),
            },
        );

        workflow.add_step(step).await.unwrap();
        assert!(workflow.validate().await.is_ok());

        // Add invalid step (empty tool name)
        let invalid_step = BasicWorkflowStep::new(
            "invalid_step".to_string(),
            BasicStepType::Tool {
                tool_name: "".to_string(),
                parameters: serde_json::json!({}),
            },
        );

        workflow.add_step(invalid_step).await.unwrap();
        let result = workflow.validate().await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("empty tool name"));
    }

    #[tokio::test]
    async fn test_workflow_execution() {
        let mut config = BasicWorkflowConfig::default();
        config.continue_on_error = false; // Fail fast
        let mut workflow = BasicSequentialWorkflow::new("test_workflow".to_string(), config);

        // Add some steps
        let step1 = BasicWorkflowStep::new(
            "calculator_step".to_string(),
            BasicStepType::Tool {
                tool_name: "calculator".to_string(),
                parameters: serde_json::json!({"expression": "2 + 2"}),
            },
        );

        let step2 = BasicWorkflowStep::new(
            "json_step".to_string(),
            BasicStepType::Tool {
                tool_name: "json_processor".to_string(),
                parameters: serde_json::json!({"input": {"test": "data"}}),
            },
        );

        workflow.add_step(step1).await.unwrap();
        workflow.add_step(step2).await.unwrap();

        // Execute workflow
        let results = workflow.execute().await.unwrap();
        assert_eq!(results.len(), 2);
        assert!(results[0].success);
        assert!(results[1].success);

        // Check status
        assert_eq!(
            workflow.status().await.unwrap(),
            BasicWorkflowStatus::Completed
        );

        // Check results are accessible
        let all_results = workflow.get_results().await.unwrap();
        assert_eq!(all_results.len(), 2);
    }

    #[tokio::test]
    async fn test_workflow_execution_with_failure() {
        let mut config = BasicWorkflowConfig::default();
        config.continue_on_error = false; // Fail fast
        let mut workflow = BasicSequentialWorkflow::new("test_workflow".to_string(), config);

        // Add a step that will fail (empty tool name)
        let failing_step = BasicWorkflowStep::new(
            "failing_step".to_string(),
            BasicStepType::Tool {
                tool_name: "".to_string(), // This will cause failure
                parameters: serde_json::json!({}),
            },
        );

        workflow.add_step(failing_step).await.unwrap();

        // Validation should catch this
        let validation_result = workflow.validate().await;
        assert!(validation_result.is_err());
    }

    #[tokio::test]
    async fn test_workflow_state_management() {
        let config = BasicWorkflowConfig::default();
        let workflow = BasicSequentialWorkflow::new("test_workflow".to_string(), config);

        // Test shared data
        workflow
            .set_shared_data("test_key".to_string(), serde_json::json!("test_value"))
            .await
            .unwrap();
        let value = workflow.get_shared_data("test_key").await.unwrap();
        assert_eq!(value, Some(serde_json::json!("test_value")));

        // Test state snapshot
        let snapshot = workflow.get_state_snapshot().await.unwrap();
        assert_eq!(snapshot.current_step, 0);
        assert!(snapshot.shared_data.contains_key("test_key"));
    }

    #[tokio::test]
    async fn test_workflow_reset() {
        let config = BasicWorkflowConfig::default();
        let mut workflow = BasicSequentialWorkflow::new("test_workflow".to_string(), config);

        // Set some state
        workflow
            .set_shared_data("test".to_string(), serde_json::json!("value"))
            .await
            .unwrap();

        // Reset
        workflow.reset().await.unwrap();

        // Check state is reset
        assert_eq!(
            workflow.status().await.unwrap(),
            BasicWorkflowStatus::Pending
        );
        let shared_data = workflow.get_shared_data("test").await.unwrap();
        assert_eq!(shared_data, None);
    }
}
