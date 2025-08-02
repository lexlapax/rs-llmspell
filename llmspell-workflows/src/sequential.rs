//! ABOUTME: Sequential workflow implementation for basic step-by-step execution
//! ABOUTME: Executes workflow steps in sequence with error handling and state management

use super::error_handling::{ErrorAction, ErrorHandler};
use super::hooks::{WorkflowExecutionPhase, WorkflowExecutor, WorkflowHookContext};
use super::state::{ExecutionStats, StateManager};
use super::step_executor::StepExecutor;
use super::traits::{ErrorStrategy, StepResult, WorkflowStatus, WorkflowStep};
use super::types::{StepExecutionContext, WorkflowConfig};
use async_trait::async_trait;
use llmspell_core::{
    execution_context::ExecutionContext,
    traits::base_agent::BaseAgent,
    traits::workflow::{
        StepResult as CoreStepResult, Workflow, WorkflowConfig as CoreWorkflowConfig,
        WorkflowStatus as CoreWorkflowStatus, WorkflowStep as CoreWorkflowStep,
    },
    types::{AgentInput, AgentOutput},
    ComponentId, ComponentMetadata, LLMSpellError, Result,
};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

/// Sequential workflow that executes steps one after another
pub struct SequentialWorkflow {
    name: String,
    steps: Vec<WorkflowStep>,
    state_manager: StateManager,
    step_executor: StepExecutor,
    error_handler: ErrorHandler,
    error_strategy: ErrorStrategy,
    /// Optional workflow executor for hook integration
    workflow_executor: Option<Arc<WorkflowExecutor>>,
    /// Workflow metadata
    metadata: ComponentMetadata,
    /// Core workflow configuration for Workflow trait
    core_config: CoreWorkflowConfig,
    /// Core workflow steps for Workflow trait
    core_steps: Arc<RwLock<Vec<CoreWorkflowStep>>>,
    /// Core workflow results for Workflow trait
    core_results: Arc<RwLock<Vec<CoreStepResult>>>,
}

impl SequentialWorkflow {
    /// Create a new sequential workflow
    pub fn new(name: String, config: WorkflowConfig) -> Self {
        let error_strategy = config.default_error_strategy.clone();
        let error_handler = ErrorHandler::new(error_strategy.clone());
        let state_manager = StateManager::new(config.clone());
        let step_executor = StepExecutor::new(config.clone());

        let metadata = ComponentMetadata::new(name.clone(), "Sequential workflow".to_string());

        // Create core workflow config from our config
        let core_config = CoreWorkflowConfig {
            max_parallel: Some(1), // Sequential execution
            continue_on_error: matches!(error_strategy, ErrorStrategy::Continue),
            timeout: config.max_execution_time,
        };

        Self {
            name,
            steps: Vec::new(),
            state_manager,
            step_executor,
            error_handler,
            error_strategy,
            workflow_executor: None,
            metadata,
            core_config,
            core_steps: Arc::new(RwLock::new(Vec::new())),
            core_results: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Create with hook integration
    pub fn new_with_hooks(
        name: String,
        config: WorkflowConfig,
        workflow_executor: Arc<WorkflowExecutor>,
    ) -> Self {
        let error_strategy = config.default_error_strategy.clone();
        let error_handler = ErrorHandler::new(error_strategy.clone());
        let state_manager = StateManager::new_with_hooks(config.clone(), workflow_executor.clone());
        let step_executor = StepExecutor::new_with_hooks(config.clone(), workflow_executor.clone());

        let metadata =
            ComponentMetadata::new(name.clone(), "Sequential workflow with hooks".to_string());

        // Create core workflow config from our config
        let core_config = CoreWorkflowConfig {
            max_parallel: Some(1), // Sequential execution
            continue_on_error: matches!(error_strategy, ErrorStrategy::Continue),
            timeout: config.max_execution_time,
        };

        Self {
            name,
            steps: Vec::new(),
            state_manager,
            step_executor,
            error_handler,
            error_strategy,
            workflow_executor: Some(workflow_executor),
            metadata,
            core_config,
            core_steps: Arc::new(RwLock::new(Vec::new())),
            core_results: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Create a new sequential workflow with builder pattern
    pub fn builder(name: String) -> SequentialWorkflowBuilder {
        SequentialWorkflowBuilder::new(name)
    }

    /// Add a step to the workflow
    pub fn add_step(&mut self, step: WorkflowStep) {
        self.steps.push(step);
    }

    /// Add multiple steps to the workflow
    pub fn add_steps(&mut self, steps: Vec<WorkflowStep>) {
        self.steps.extend(steps);
    }

    /// Get workflow name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get number of steps
    pub fn step_count(&self) -> usize {
        self.steps.len()
    }

    /// Execute the workflow
    pub async fn execute_workflow(&self) -> Result<SequentialWorkflowResult> {
        let start_time = Instant::now();
        info!("Starting sequential workflow: {}", self.name);

        // Execute workflow start hooks
        if let Some(workflow_executor) = &self.workflow_executor {
            let component_id = llmspell_hooks::ComponentId::new(
                llmspell_hooks::ComponentType::Workflow,
                format!("workflow_{}", self.name),
            );
            let workflow_state = self.state_manager.get_state_snapshot().await?;
            let hook_ctx = WorkflowHookContext::new(
                component_id,
                self.metadata.clone(),
                workflow_state,
                "sequential".to_string(),
                WorkflowExecutionPhase::WorkflowStart,
            );
            let _ = workflow_executor.execute_workflow_hooks(hook_ctx).await;
        }

        // Start execution tracking
        self.state_manager.start_execution().await?;

        let mut successful_steps = Vec::new();
        let mut failed_steps = Vec::new();

        for (index, step) in self.steps.iter().enumerate() {
            // Check for execution timeout
            if self.state_manager.check_execution_timeout().await? {
                error!("Workflow '{}' exceeded maximum execution time", self.name);
                self.state_manager.complete_execution(false).await?;
                return Ok(SequentialWorkflowResult::timeout(
                    self.name.clone(),
                    successful_steps,
                    failed_steps,
                    start_time.elapsed(),
                ));
            }
            debug!(
                "Executing step {} of {}: {}",
                index + 1,
                self.steps.len(),
                step.name
            );

            // Create execution context
            let shared_data = self.state_manager.get_all_shared_data().await?;
            let mut workflow_state = crate::types::WorkflowState::new();
            workflow_state.shared_data = shared_data;
            workflow_state.current_step = index;
            let context = StepExecutionContext::new(workflow_state.clone(), None);

            // Execute step with retry logic (with workflow metadata if hooks are enabled)
            let step_result = if self.workflow_executor.is_some() {
                self.step_executor
                    .execute_step_with_retry_and_metadata(
                        step,
                        context,
                        &self.error_strategy,
                        Some(self.metadata.clone()),
                        Some("sequential".to_string()),
                    )
                    .await?
            } else {
                self.step_executor
                    .execute_step_with_retry(step, context, &self.error_strategy)
                    .await?
            };

            // Record the result
            self.state_manager
                .record_step_result(step_result.clone())
                .await?;

            if step_result.success {
                successful_steps.push(step_result);
                self.state_manager.advance_step().await?;
            } else {
                failed_steps.push(step_result.clone());

                // Handle the failure based on error strategy
                let error_action = self
                    .error_handler
                    .handle_step_failure(&step_result, Some(&self.error_strategy))
                    .await?;

                match error_action {
                    ErrorAction::StopWorkflow => {
                        warn!("Stopping workflow '{}' due to step failure", self.name);
                        self.state_manager.complete_execution(false).await?;
                        return Ok(SequentialWorkflowResult::failure(
                            self.name.clone(),
                            successful_steps,
                            failed_steps,
                            start_time.elapsed(),
                            format!("Workflow stopped at step {}: {}", index + 1, step.name),
                        ));
                    }
                    ErrorAction::ContinueToNext => {
                        warn!(
                            "Continuing to next step after failure in step: {}",
                            step.name
                        );
                        self.state_manager.advance_step().await?;
                        continue;
                    }
                    ErrorAction::RetryStep => {
                        // This is handled by execute_step_with_retry, so if we're here,
                        // all retries have been exhausted and we should continue based on strategy
                        if matches!(self.error_strategy, ErrorStrategy::Continue) {
                            warn!("All retries exhausted for step {}, continuing", step.name);
                            self.state_manager.advance_step().await?;
                            continue;
                        } else {
                            warn!(
                                "All retries exhausted for step {}, stopping workflow",
                                step.name
                            );
                            self.state_manager.complete_execution(false).await?;
                            return Ok(SequentialWorkflowResult::failure(
                                self.name.clone(),
                                successful_steps,
                                failed_steps,
                                start_time.elapsed(),
                                format!(
                                    "Workflow stopped after retries exhausted at step {}: {}",
                                    index + 1,
                                    step.name
                                ),
                            ));
                        }
                    }
                }
            }
        }

        // All steps completed successfully
        let duration = start_time.elapsed();
        self.state_manager.complete_execution(true).await?;

        // Execute workflow completion hooks
        if let Some(workflow_executor) = &self.workflow_executor {
            let component_id = llmspell_hooks::ComponentId::new(
                llmspell_hooks::ComponentType::Workflow,
                format!("workflow_{}", self.name),
            );
            let workflow_state = self.state_manager.get_state_snapshot().await?;
            let hook_ctx = WorkflowHookContext::new(
                component_id,
                self.metadata.clone(),
                workflow_state,
                "sequential".to_string(),
                WorkflowExecutionPhase::WorkflowComplete,
            );
            let _ = workflow_executor.execute_workflow_hooks(hook_ctx).await;
        }

        info!(
            "Sequential workflow '{}' completed successfully in {:?}",
            self.name, duration
        );

        Ok(SequentialWorkflowResult::success(
            self.name.clone(),
            successful_steps,
            failed_steps,
            duration,
        ))
    }

    /// Get current execution status
    pub async fn get_status(&self) -> Result<WorkflowStatus> {
        self.state_manager.get_status().await
    }

    /// Get execution statistics
    pub async fn get_execution_stats(&self) -> Result<ExecutionStats> {
        self.state_manager.get_execution_stats().await
    }

    /// Cancel the workflow execution
    pub async fn cancel(&self) -> Result<()> {
        warn!("Cancelling sequential workflow: {}", self.name);
        self.state_manager.cancel_execution().await
    }

    /// Reset the workflow to initial state
    pub async fn reset(&self) -> Result<()> {
        debug!("Resetting sequential workflow: {}", self.name);
        self.state_manager.reset().await
    }

    /// Get shared data value
    pub async fn get_shared_data(&self, key: &str) -> Result<Option<serde_json::Value>> {
        self.state_manager.get_shared_data(key).await
    }

    /// Set shared data value
    pub async fn set_shared_data(&self, key: String, value: serde_json::Value) -> Result<()> {
        self.state_manager.set_shared_data(key, value).await
    }
}

#[async_trait]
impl BaseAgent for SequentialWorkflow {
    fn metadata(&self) -> &ComponentMetadata {
        &self.metadata
    }

    async fn execute(&self, input: AgentInput, _context: ExecutionContext) -> Result<AgentOutput> {
        // Convert AgentInput to workflow execution
        // The workflow will use the input text as an execution trigger
        // and parameters for workflow configuration

        // Validate input first
        self.validate_input(&input).await?;

        // Execute the workflow using existing implementation
        let workflow_result = self.execute_workflow().await?;

        // Convert SequentialWorkflowResult to AgentOutput
        let output_text = if workflow_result.success {
            format!(
                "Sequential workflow '{}' completed successfully. {} steps succeeded, {} steps failed. Duration: {:?}",
                workflow_result.workflow_name,
                workflow_result.successful_steps.len(),
                workflow_result.failed_steps.len(),
                workflow_result.duration
            )
        } else {
            format!(
                "Sequential workflow '{}' failed: {}. {} steps succeeded, {} steps failed. Duration: {:?}",
                workflow_result.workflow_name,
                workflow_result.error_message.as_deref().unwrap_or("Unknown error"),
                workflow_result.successful_steps.len(),
                workflow_result.failed_steps.len(),
                workflow_result.duration
            )
        };

        // Build AgentOutput with execution metadata
        let mut metadata = llmspell_core::types::OutputMetadata {
            execution_time_ms: Some(workflow_result.duration.as_millis() as u64),
            ..Default::default()
        };
        metadata
            .extra
            .insert("workflow_type".to_string(), serde_json::json!("sequential"));
        metadata.extra.insert(
            "workflow_name".to_string(),
            serde_json::json!(workflow_result.workflow_name),
        );
        metadata.extra.insert(
            "total_steps".to_string(),
            serde_json::json!(workflow_result.total_steps()),
        );
        metadata.extra.insert(
            "successful_steps".to_string(),
            serde_json::json!(workflow_result.successful_steps.len()),
        );
        metadata.extra.insert(
            "failed_steps".to_string(),
            serde_json::json!(workflow_result.failed_steps.len()),
        );
        metadata.extra.insert(
            "success_rate".to_string(),
            serde_json::json!(workflow_result.success_rate()),
        );

        Ok(AgentOutput::text(output_text).with_metadata(metadata))
    }

    async fn validate_input(&self, input: &AgentInput) -> Result<()> {
        // Basic validation - workflow can accept any non-empty text input
        if input.text.is_empty() {
            return Err(LLMSpellError::Validation {
                message: "Workflow input text cannot be empty".to_string(),
                field: Some("text".to_string()),
            });
        }

        // Validate that we have steps to execute
        if self.steps.is_empty() {
            return Err(LLMSpellError::Validation {
                message: "Cannot execute workflow without steps".to_string(),
                field: Some("steps".to_string()),
            });
        }

        Ok(())
    }

    async fn handle_error(&self, error: LLMSpellError) -> Result<AgentOutput> {
        // Handle workflow-specific errors gracefully
        let error_text = match &error {
            LLMSpellError::Workflow { message, step, .. } => {
                if let Some(step_name) = step {
                    format!("Workflow error in step '{}': {}", step_name, message)
                } else {
                    format!("Workflow error: {}", message)
                }
            }
            LLMSpellError::Validation { message, field } => {
                if let Some(field_name) = field {
                    format!("Validation error in field '{}': {}", field_name, message)
                } else {
                    format!("Validation error: {}", message)
                }
            }
            _ => format!("Sequential workflow error: {}", error),
        };

        let mut metadata = llmspell_core::types::OutputMetadata::default();
        metadata.extra.insert(
            "error_type".to_string(),
            serde_json::json!("workflow_error"),
        );
        metadata
            .extra
            .insert("workflow_type".to_string(), serde_json::json!("sequential"));
        metadata
            .extra
            .insert("workflow_name".to_string(), serde_json::json!(self.name));

        Ok(AgentOutput::text(error_text).with_metadata(metadata))
    }
}

#[async_trait]
impl Workflow for SequentialWorkflow {
    fn config(&self) -> &CoreWorkflowConfig {
        &self.core_config
    }

    async fn add_step(&self, step: CoreWorkflowStep) -> Result<()> {
        let mut steps = self.core_steps.write().await;
        steps.push(step);
        Ok(())
    }

    async fn remove_step(&self, step_id: ComponentId) -> Result<()> {
        let mut steps = self.core_steps.write().await;
        steps.retain(|s| s.id != step_id);
        Ok(())
    }

    async fn get_steps(&self) -> Result<Vec<CoreWorkflowStep>> {
        let steps = self.core_steps.read().await;
        Ok(steps.clone())
    }

    async fn status(&self) -> Result<CoreWorkflowStatus> {
        let status = self.get_status().await?;

        // Convert our WorkflowStatus to CoreWorkflowStatus
        let core_status = match status {
            WorkflowStatus::Pending => CoreWorkflowStatus::Pending,
            WorkflowStatus::Running => CoreWorkflowStatus::Running,
            WorkflowStatus::Completed => CoreWorkflowStatus::Completed,
            WorkflowStatus::Failed => CoreWorkflowStatus::Failed,
            WorkflowStatus::Cancelled => CoreWorkflowStatus::Cancelled,
        };

        Ok(core_status)
    }

    async fn get_results(&self) -> Result<Vec<CoreStepResult>> {
        let results = self.core_results.read().await;
        Ok(results.clone())
    }
}

/// Builder for creating sequential workflows
pub struct SequentialWorkflowBuilder {
    name: String,
    config: WorkflowConfig,
    steps: Vec<WorkflowStep>,
    error_strategy: Option<ErrorStrategy>,
    workflow_executor: Option<Arc<WorkflowExecutor>>,
}

impl SequentialWorkflowBuilder {
    /// Create a new builder
    pub fn new(name: String) -> Self {
        Self {
            name,
            config: WorkflowConfig::default(),
            steps: Vec::new(),
            error_strategy: None,
            workflow_executor: None,
        }
    }

    /// Set the workflow configuration
    pub fn with_config(mut self, config: WorkflowConfig) -> Self {
        self.config = config;
        self
    }

    /// Set the error strategy
    pub fn with_error_strategy(mut self, strategy: ErrorStrategy) -> Self {
        self.error_strategy = Some(strategy);
        self
    }

    /// Add a step to the workflow
    pub fn add_step(mut self, step: WorkflowStep) -> Self {
        self.steps.push(step);
        self
    }

    /// Add multiple steps to the workflow
    pub fn add_steps(mut self, steps: Vec<WorkflowStep>) -> Self {
        self.steps.extend(steps);
        self
    }

    /// Enable hook integration with a WorkflowExecutor
    pub fn with_hooks(mut self, workflow_executor: Arc<WorkflowExecutor>) -> Self {
        self.workflow_executor = Some(workflow_executor);
        self
    }

    /// Build the sequential workflow
    pub fn build(mut self) -> SequentialWorkflow {
        // Apply error strategy if provided
        if let Some(strategy) = self.error_strategy {
            self.config.default_error_strategy = strategy;
        }

        let mut workflow = if let Some(workflow_executor) = self.workflow_executor {
            SequentialWorkflow::new_with_hooks(self.name, self.config, workflow_executor)
        } else {
            SequentialWorkflow::new(self.name, self.config)
        };
        workflow.add_steps(self.steps);
        workflow
    }
}

/// Result of sequential workflow execution
#[derive(Debug, Clone)]
pub struct SequentialWorkflowResult {
    pub workflow_name: String,
    pub success: bool,
    pub successful_steps: Vec<StepResult>,
    pub failed_steps: Vec<StepResult>,
    pub duration: Duration,
    pub error_message: Option<String>,
}

impl SequentialWorkflowResult {
    /// Create a successful result
    pub fn success(
        workflow_name: String,
        successful_steps: Vec<StepResult>,
        failed_steps: Vec<StepResult>,
        duration: Duration,
    ) -> Self {
        Self {
            workflow_name,
            success: true,
            successful_steps,
            failed_steps,
            duration,
            error_message: None,
        }
    }

    /// Create a failed result
    pub fn failure(
        workflow_name: String,
        successful_steps: Vec<StepResult>,
        failed_steps: Vec<StepResult>,
        duration: Duration,
        error_message: String,
    ) -> Self {
        Self {
            workflow_name,
            success: false,
            successful_steps,
            failed_steps,
            duration,
            error_message: Some(error_message),
        }
    }

    /// Create a timeout result
    pub fn timeout(
        workflow_name: String,
        successful_steps: Vec<StepResult>,
        failed_steps: Vec<StepResult>,
        duration: Duration,
    ) -> Self {
        Self {
            workflow_name,
            success: false,
            successful_steps,
            failed_steps,
            duration,
            error_message: Some("Workflow execution timed out".to_string()),
        }
    }

    /// Get total number of steps
    pub fn total_steps(&self) -> usize {
        self.successful_steps.len() + self.failed_steps.len()
    }

    /// Get success rate as percentage
    pub fn success_rate(&self) -> f64 {
        if self.total_steps() == 0 {
            0.0
        } else {
            (self.successful_steps.len() as f64 / self.total_steps() as f64) * 100.0
        }
    }

    /// Generate a human-readable report
    pub fn generate_report(&self) -> String {
        format!(
            "Sequential Workflow '{}' Report:\n\
            - Success: {}\n\
            - Duration: {:?}\n\
            - Total Steps: {}\n\
            - Successful Steps: {}\n\
            - Failed Steps: {}\n\
            - Success Rate: {:.1}%\n\
            - Error: {}",
            self.workflow_name,
            if self.success { "✓" } else { "✗" },
            self.duration,
            self.total_steps(),
            self.successful_steps.len(),
            self.failed_steps.len(),
            self.success_rate(),
            self.error_message.as_deref().unwrap_or("None")
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::StepType;
    #[tokio::test]
    async fn test_sequential_workflow_creation() {
        let workflow =
            SequentialWorkflow::new("test_workflow".to_string(), WorkflowConfig::default());
        assert_eq!(workflow.name(), "test_workflow");
        assert_eq!(workflow.step_count(), 0);
    }
    #[tokio::test]
    async fn test_sequential_workflow_builder() {
        let step = WorkflowStep::new(
            "test_step".to_string(),
            StepType::Tool {
                tool_name: "calculator".to_string(),
                parameters: serde_json::json!({"expression": "2 + 2"}),
            },
        );

        let workflow = SequentialWorkflow::builder("test_workflow".to_string())
            .add_step(step)
            .with_error_strategy(ErrorStrategy::Continue)
            .build();

        assert_eq!(workflow.name(), "test_workflow");
        assert_eq!(workflow.step_count(), 1);
    }
    #[tokio::test]
    async fn test_sequential_workflow_execution_success() {
        let step1 = WorkflowStep::new(
            "step1".to_string(),
            StepType::Tool {
                tool_name: "calculator".to_string(),
                parameters: serde_json::json!({"expression": "2 + 2"}),
            },
        );

        let step2 = WorkflowStep::new(
            "step2".to_string(),
            StepType::Tool {
                tool_name: "json_processor".to_string(),
                parameters: serde_json::json!({"input": {"data": "test"}}),
            },
        );

        let workflow = SequentialWorkflow::builder("test_workflow".to_string())
            .add_step(step1)
            .add_step(step2)
            .build();

        let result = workflow.execute_workflow().await.unwrap();
        assert!(result.success);
        assert_eq!(result.successful_steps.len(), 2);
        assert_eq!(result.failed_steps.len(), 0);
        assert_eq!(result.total_steps(), 2);
        assert_eq!(result.success_rate(), 100.0);
    }
    #[tokio::test]
    async fn test_sequential_workflow_execution_with_failure() {
        let step1 = WorkflowStep::new(
            "step1".to_string(),
            StepType::Tool {
                tool_name: "calculator".to_string(),
                parameters: serde_json::json!({"expression": "2 + 2"}),
            },
        );

        // This step will fail due to empty tool name
        let step2 = WorkflowStep::new(
            "step2".to_string(),
            StepType::Tool {
                tool_name: "".to_string(), // Empty tool name causes failure
                parameters: serde_json::json!({}),
            },
        );

        let workflow = SequentialWorkflow::builder("test_workflow".to_string())
            .add_step(step1)
            .add_step(step2)
            .with_error_strategy(ErrorStrategy::FailFast)
            .build();

        let result = workflow.execute_workflow().await.unwrap();
        assert!(!result.success);
        assert_eq!(result.successful_steps.len(), 1);
        assert_eq!(result.failed_steps.len(), 1);
        assert!(result.error_message.is_some());
    }
    #[tokio::test]
    async fn test_sequential_workflow_continue_on_error() {
        let step1 = WorkflowStep::new(
            "step1".to_string(),
            StepType::Tool {
                tool_name: "calculator".to_string(),
                parameters: serde_json::json!({"expression": "2 + 2"}),
            },
        );

        // This step will fail but workflow should continue
        let step2 = WorkflowStep::new(
            "step2".to_string(),
            StepType::Tool {
                tool_name: "".to_string(), // Empty tool name causes failure
                parameters: serde_json::json!({}),
            },
        );

        let step3 = WorkflowStep::new(
            "step3".to_string(),
            StepType::Tool {
                tool_name: "json_processor".to_string(),
                parameters: serde_json::json!({"input": {"data": "test"}}),
            },
        );

        let workflow = SequentialWorkflow::builder("test_workflow".to_string())
            .add_step(step1)
            .add_step(step2)
            .add_step(step3)
            .with_error_strategy(ErrorStrategy::Continue)
            .build();

        let result = workflow.execute_workflow().await.unwrap();
        assert!(result.success); // Should succeed because we continue on error
        assert_eq!(result.successful_steps.len(), 2);
        assert_eq!(result.failed_steps.len(), 1);
        assert_eq!(result.total_steps(), 3);
    }
    #[tokio::test]
    async fn test_sequential_workflow_shared_data() {
        let workflow =
            SequentialWorkflow::new("test_workflow".to_string(), WorkflowConfig::default());

        // Set shared data
        let test_value = serde_json::json!({"key": "value"});
        workflow
            .set_shared_data("test_data".to_string(), test_value.clone())
            .await
            .unwrap();

        // Get shared data
        let retrieved = workflow.get_shared_data("test_data").await.unwrap();
        assert_eq!(retrieved, Some(test_value));

        // Get non-existent data
        let missing = workflow.get_shared_data("missing").await.unwrap();
        assert_eq!(missing, None);
    }
    #[tokio::test]
    async fn test_sequential_workflow_status_tracking() {
        let workflow =
            SequentialWorkflow::new("test_workflow".to_string(), WorkflowConfig::default());

        // Initial status should be pending
        let status = workflow.get_status().await.unwrap();
        assert_eq!(status, WorkflowStatus::Pending);

        // Reset should work
        workflow.reset().await.unwrap();
        let status = workflow.get_status().await.unwrap();
        assert_eq!(status, WorkflowStatus::Pending);
    }
}
