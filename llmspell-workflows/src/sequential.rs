//! ABOUTME: Sequential workflow implementation for basic step-by-step execution
//! ABOUTME: Executes workflow steps in sequence with error handling and state management

use super::error_handling::{ErrorAction, ErrorHandler};
use super::hooks::{WorkflowExecutionPhase, WorkflowExecutor, WorkflowHookContext};
use super::result::{WorkflowError, WorkflowResult, WorkflowType};
use super::state::StateManager;
use super::step_executor::StepExecutor;
use super::traits::{ErrorStrategy, StepType, WorkflowStatus, WorkflowStep};
use super::types::{StepExecutionContext, WorkflowConfig};
use async_trait::async_trait;
use llmspell_core::{
    execution_context::ExecutionContext,
    traits::base_agent::BaseAgent,
    traits::component_lookup::ComponentLookup,
    traits::workflow::{
        Config as CoreWorkflowConfig, Status as CoreWorkflowStatus, StepResult as CoreStepResult,
        Workflow, WorkflowStep as CoreWorkflowStep,
    },
    types::{AgentInput, AgentOutput},
    ComponentId, ComponentMetadata, LLMSpellError, Result,
};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;
use tracing::{debug, error, info, instrument, warn};

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
    /// Optional template executor for template step execution
    template_executor: Option<Arc<dyn llmspell_core::traits::template_executor::TemplateExecutor>>,
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
        Self::new_with_registry(name, config, None)
    }

    /// Create a new sequential workflow with registry for component lookup
    pub fn new_with_registry(
        name: String,
        config: WorkflowConfig,
        registry: Option<Arc<dyn ComponentLookup>>,
    ) -> Self {
        let error_strategy = config.default_error_strategy.clone();
        let error_handler = ErrorHandler::new(error_strategy.clone());
        let state_manager = StateManager::new(config.clone());
        let step_executor = if let Some(reg) = registry {
            StepExecutor::new_with_registry(config.clone(), reg)
        } else {
            StepExecutor::new(config.clone())
        };

        let metadata = ComponentMetadata::new(name.clone(), "Sequential workflow".to_string());

        // Create core workflow config from our config
        let core_config = CoreWorkflowConfig::new()
            .with_max_parallel(Some(1)) // Sequential execution
            .with_continue_on_error(matches!(error_strategy, ErrorStrategy::Continue))
            .with_timeout(config.max_execution_time);

        Self {
            name,
            steps: Vec::new(),
            state_manager,
            step_executor,
            error_handler,
            error_strategy,
            workflow_executor: None,
            template_executor: None,
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
        Self::new_with_hooks_and_registry(name, config, workflow_executor, None)
    }

    /// Create with hook integration and registry
    pub fn new_with_hooks_and_registry(
        name: String,
        config: WorkflowConfig,
        workflow_executor: Arc<WorkflowExecutor>,
        registry: Option<Arc<dyn ComponentLookup>>,
    ) -> Self {
        let error_strategy = config.default_error_strategy.clone();
        let error_handler = ErrorHandler::new(error_strategy.clone());
        let state_manager = StateManager::new_with_hooks(config.clone(), workflow_executor.clone());
        let step_executor = if let Some(reg) = registry {
            StepExecutor::new_with_hooks_and_registry(
                config.clone(),
                workflow_executor.clone(),
                reg,
            )
        } else {
            StepExecutor::new_with_hooks(config.clone(), workflow_executor.clone())
        };

        let metadata =
            ComponentMetadata::new(name.clone(), "Sequential workflow with hooks".to_string());

        // Create core workflow config from our config
        let core_config = CoreWorkflowConfig::new()
            .with_max_parallel(Some(1)) // Sequential execution
            .with_continue_on_error(matches!(error_strategy, ErrorStrategy::Continue))
            .with_timeout(config.max_execution_time);

        Self {
            name,
            steps: Vec::new(),
            state_manager,
            step_executor,
            error_handler,
            error_strategy,
            workflow_executor: Some(workflow_executor),
            template_executor: None,
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

    /// Execute the workflow with state-based outputs
    ///
    /// This is the new state-based execution method that writes outputs to state
    /// and returns only metadata in the WorkflowResult.
    // execute_with_state removed - functionality moved to execute_impl
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

    #[instrument(level = "info", skip(self, input, context), fields(
        workflow_name = %self.metadata.name,
        step_count = self.steps.len(),
        input_size = input.text.len()
    ))]
    async fn execute_impl(
        &self,
        input: AgentInput,
        context: ExecutionContext,
    ) -> Result<AgentOutput> {
        // Validate input
        self.validate_input(&input).await?;

        // Debug: Check state availability
        debug!(
            "SequentialWorkflow::execute - State available in context: {}",
            context.state.is_some()
        );

        // Execute workflow with state (inlined from execute_with_state)
        let start_time = Instant::now();
        // Generate ComponentId once and use it consistently
        let execution_component_id = ComponentId::new();
        let execution_id = execution_component_id.to_string();

        // Debug: Double-check state availability
        debug!(
            "execute_impl - State in context: {}, type: {:?}",
            context.state.is_some(),
            context.state.as_ref().map(|_| "StateAccess")
        );

        info!(
            "Starting sequential workflow: {} (execution: {}) - State available: {}",
            self.name,
            execution_id,
            context.state.is_some()
        );

        // Note: workflow.started event is now emitted by execute() wrapper in BaseAgent

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

        let mut state_keys = Vec::new();
        let mut steps_executed = 0usize;
        let mut steps_failed = 0usize;
        let mut steps_skipped = 0usize;

        for (index, step) in self.steps.iter().enumerate() {
            // Check for execution timeout
            if self.state_manager.check_execution_timeout().await? {
                error!("Workflow '{}' exceeded maximum execution time", self.name);
                self.state_manager.complete_execution(false).await?;

                // Note: workflow.failed event is now emitted by execute() wrapper in BaseAgent

                return Err(LLMSpellError::Workflow {
                    message: format!("Workflow '{}' exceeded maximum execution time", self.name),
                    step: None,
                    source: None,
                });
            }

            debug!(
                "Executing step {} of {}: {}",
                index + 1,
                self.steps.len(),
                step.name
            );

            // Create execution context for step
            let shared_data = self.state_manager.get_all_shared_data().await?;
            let mut workflow_state = crate::types::WorkflowState::new();
            // CRITICAL: Use the workflow's execution_component_id, not a new one!
            workflow_state.execution_id = execution_component_id;
            workflow_state.shared_data = shared_data;
            workflow_state.current_step = index;
            let mut step_context = StepExecutionContext::new(workflow_state.clone(), None);

            // Pass events to step context if available
            if let Some(ref events) = context.events {
                step_context = step_context.with_events(events.clone());
            }

            // Pass state to step context if available
            if let Some(ref state) = context.state {
                info!(
                    "Passing state to StepExecutionContext for step {} - state exists: true",
                    step.name
                );
                step_context = step_context.with_state(state.clone());
                info!(
                    "After with_state: step_context.state is_some: {}",
                    step_context.state.is_some()
                );
            } else {
                warn!(
                    "No state available in ExecutionContext for step {} - state is None!",
                    step.name
                );
            }

            // Pass template_executor to step context if available
            if let Some(ref template_executor) = self.template_executor {
                step_context = step_context.with_template_executor(template_executor.clone());
            }

            // Execute step with retry logic
            let step_result = if self.workflow_executor.is_some() {
                self.step_executor
                    .execute_step_with_retry_and_metadata(
                        step,
                        step_context,
                        &self.error_strategy,
                        Some(self.metadata.clone()),
                        Some("sequential".to_string()),
                    )
                    .await?
            } else {
                self.step_executor
                    .execute_step_with_retry(step, step_context, &self.error_strategy)
                    .await?
            };

            // Record the result
            self.state_manager
                .record_step_result(step_result.clone())
                .await?;

            if step_result.success {
                steps_executed += 1;

                // Write step output to state if state is available
                if let Some(ref state) = context.state {
                    let state_key = WorkflowResult::generate_state_key(&execution_id, &step.name);
                    let output_value = serde_json::json!({
                        "step_name": step.name,
                        "step_id": step_result.step_id.to_string(),
                        "output": step_result.output,
                        "duration_ms": step_result.duration.as_millis(),
                        "retry_count": step_result.retry_count,
                    });

                    state.write(&state_key, output_value).await.map_err(|e| {
                        LLMSpellError::Component {
                            message: format!("Failed to write step output to state: {}", e),
                            source: None,
                        }
                    })?;

                    state_keys.push(state_key);
                    debug!("Wrote step output to state for step: {}", step.name);
                }

                self.state_manager.advance_step().await?;
            } else {
                steps_failed += 1;

                // Handle the failure based on error strategy
                let error_action = self
                    .error_handler
                    .handle_step_failure(&step_result, Some(&self.error_strategy))
                    .await?;

                match error_action {
                    ErrorAction::StopWorkflow => {
                        warn!("Stopping workflow '{}' due to step failure", self.name);
                        self.state_manager.complete_execution(false).await?;

                        // Note: workflow.failed event is now emitted by execute() wrapper in BaseAgent

                        return Err(LLMSpellError::Workflow {
                            message: format!(
                                "Step '{}' failed: {}",
                                step.name,
                                step_result
                                    .error
                                    .unwrap_or_else(|| "Unknown error".to_string())
                            ),
                            step: Some(step.name.clone()),
                            source: None,
                        });
                    }
                    ErrorAction::ContinueToNext => {
                        warn!(
                            "Continuing to next step after failure in step: {}",
                            step.name
                        );
                        steps_skipped += 1;
                        self.state_manager.advance_step().await?;
                    }
                    ErrorAction::RetryStep => {
                        // This is handled by execute_step_with_retry, so if we're here,
                        // all retries have been exhausted and we should continue based on strategy
                        if matches!(self.error_strategy, ErrorStrategy::Continue) {
                            warn!("All retries exhausted for step {}, continuing", step.name);
                            steps_skipped += 1;
                            self.state_manager.advance_step().await?;
                        } else {
                            warn!(
                                "All retries exhausted for step {}, stopping workflow",
                                step.name
                            );
                            self.state_manager.complete_execution(false).await?;

                            // Note: workflow.failed event is now emitted by execute() wrapper in BaseAgent

                            return Err(LLMSpellError::Workflow {
                                message: format!("All retries exhausted for step {}", step.name),
                                step: Some(step.name.clone()),
                                source: None,
                            });
                        }
                    }
                }
            }
        }

        // All steps completed
        let duration = start_time.elapsed();
        self.state_manager.complete_execution(true).await?;

        // Note: workflow.completed event is now emitted by execute() wrapper in BaseAgent

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

        // Return metadata-only result
        let result = if steps_failed > 0 {
            WorkflowResult::partial(
                execution_id,
                WorkflowType::Sequential,
                self.name.clone(),
                state_keys,
                steps_executed,
                steps_failed,
                steps_skipped,
                duration,
                None,
            )
        } else {
            WorkflowResult::success(
                execution_id,
                WorkflowType::Sequential,
                self.name.clone(),
                state_keys,
                steps_executed,
                duration,
            )
        };

        // Store execution_id for output collection (cloned from result after move)
        let execution_id = result.execution_id.clone();

        // Build output text
        let output_text = if result.success {
            format!(
                "Sequential workflow '{}' completed successfully. {} steps executed. Duration: {:?}",
                result.workflow_name,
                result.steps_executed,
                result.duration
            )
        } else {
            format!(
                "Sequential workflow '{}' failed: {}. {} steps executed, {} failed. Duration: {:?}",
                result.workflow_name,
                result
                    .error
                    .as_ref()
                    .map(|e| e.to_string())
                    .unwrap_or_else(|| "Unknown error".to_string()),
                result.steps_executed,
                result.steps_failed,
                result.duration
            )
        };

        // Build metadata
        let mut metadata = llmspell_core::types::OutputMetadata {
            #[allow(clippy::cast_possible_truncation)]
            execution_time_ms: Some(result.duration.as_millis() as u64),
            ..Default::default()
        };

        metadata
            .extra
            .insert("workflow_type".to_string(), serde_json::json!("sequential"));
        metadata.extra.insert(
            "workflow_name".to_string(),
            serde_json::json!(result.workflow_name),
        );
        metadata.extra.insert(
            "execution_id".to_string(),
            serde_json::json!(result.execution_id),
        );
        metadata.extra.insert(
            "total_steps".to_string(),
            serde_json::json!(self.steps.len()),
        );
        metadata.extra.insert(
            "successful_steps".to_string(),
            serde_json::json!(result.steps_executed.saturating_sub(result.steps_failed)),
        );
        metadata.extra.insert(
            "failed_steps".to_string(),
            serde_json::json!(result.steps_failed),
        );

        #[allow(clippy::cast_precision_loss)]
        let success_rate = if result.steps_executed > 0 {
            ((result.steps_executed - result.steps_failed) as f64 / result.steps_executed as f64)
                * 100.0
        } else {
            0.0
        };
        metadata
            .extra
            .insert("success_rate".to_string(), serde_json::json!(success_rate));

        // Collect agent outputs from state if available (matching parallel/loop/conditional)
        let mut agent_outputs = serde_json::Map::new();
        if let Some(ref state) = context.state {
            for step in &self.steps {
                if let StepType::Agent { agent_id, .. } = &step.step_type {
                    let key = format!("workflow:{}:agent:{}:output", execution_id, agent_id);
                    if let Ok(Some(output)) = state.read(&key).await {
                        agent_outputs.insert(agent_id.clone(), output);
                    }
                }
            }
        }

        if !agent_outputs.is_empty() {
            metadata.extra.insert(
                "agent_outputs".to_string(),
                serde_json::Value::Object(agent_outputs),
            );
        }

        // If workflow failed, return an error so BaseAgent emits workflow.failed event
        if !result.success {
            return Err(LLMSpellError::Workflow {
                message: output_text.clone(),
                step: result.error.as_ref().and_then(|e| {
                    if let WorkflowError::StepExecutionFailed { step_name, .. } = e {
                        Some(step_name.clone())
                    } else {
                        None
                    }
                }),
                source: None,
            });
        }

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
        let status = self.state_manager.get_status().await?;

        // Convert our WorkflowStatus to CoreWorkflowStatus
        let core_status = match status {
            WorkflowStatus::Pending => CoreWorkflowStatus::Pending,
            WorkflowStatus::Running => CoreWorkflowStatus::Running,
            WorkflowStatus::Completed => CoreWorkflowStatus::Completed,
            WorkflowStatus::Failed => CoreWorkflowStatus::Failed,
            WorkflowStatus::Cancelled => CoreWorkflowStatus::Cancelled,
            WorkflowStatus::PartiallyCompleted => CoreWorkflowStatus::Completed,
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
    template_executor: Option<Arc<dyn llmspell_core::traits::template_executor::TemplateExecutor>>,
    registry: Option<Arc<dyn ComponentLookup>>,
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
            template_executor: None,
            registry: None,
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

    /// Add a template execution step to the workflow
    ///
    /// Convenience method for adding a `StepType::Template` step without manual construction.
    ///
    /// # Example
    ///
    /// ```
    /// use llmspell_workflows::sequential::SequentialWorkflowBuilder;
    /// use serde_json::json;
    ///
    /// let workflow = SequentialWorkflowBuilder::new("research-chat".to_string())
    ///     .add_template_step(
    ///         "research".to_string(),
    ///         "research-assistant".to_string(),
    ///         json!({
    ///             "topic": "Rust async programming",
    ///             "max_sources": 10,
    ///             "session_id": "test-session",
    ///         }),
    ///     )
    ///     .add_template_step(
    ///         "chat".to_string(),
    ///         "interactive-chat".to_string(),
    ///         json!({
    ///             "message": "Summarize the findings",
    ///             "session_id": "test-session",
    ///         }),
    ///     )
    ///     .build();
    /// ```
    pub fn add_template_step(
        mut self,
        name: String,
        template_id: String,
        params: serde_json::Value,
    ) -> Self {
        let step = WorkflowStep::new(name, StepType::Template { template_id, params });
        self.steps.push(step);
        self
    }

    /// Enable hook integration with a WorkflowExecutor
    pub fn with_hooks(mut self, workflow_executor: Arc<WorkflowExecutor>) -> Self {
        self.workflow_executor = Some(workflow_executor);
        self
    }

    /// Set the component registry for component lookup
    pub fn with_registry(mut self, registry: Arc<dyn ComponentLookup>) -> Self {
        self.registry = Some(registry);
        self
    }

    /// Set the template executor for template step execution
    pub fn with_template_executor(
        mut self,
        template_executor: Arc<dyn llmspell_core::traits::template_executor::TemplateExecutor>,
    ) -> Self {
        self.template_executor = Some(template_executor);
        self
    }

    /// Build the sequential workflow
    pub fn build(mut self) -> SequentialWorkflow {
        // Apply error strategy if provided
        if let Some(strategy) = self.error_strategy {
            self.config.default_error_strategy = strategy;
        }

        let mut workflow = match (self.workflow_executor, self.registry) {
            (Some(executor), Some(registry)) => SequentialWorkflow::new_with_hooks_and_registry(
                self.name,
                self.config,
                executor,
                Some(registry),
            ),
            (Some(executor), None) => {
                SequentialWorkflow::new_with_hooks(self.name, self.config, executor)
            }
            (None, Some(registry)) => {
                SequentialWorkflow::new_with_registry(self.name, self.config, Some(registry))
            }
            (None, None) => SequentialWorkflow::new(self.name, self.config),
        };
        workflow.add_steps(self.steps);
        workflow.template_executor = self.template_executor;
        workflow
    }
}

// Legacy SequentialWorkflowResult removed - using WorkflowResult from types module

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
    async fn test_add_template_step_builder() {
        use serde_json::json;

        // Build workflow with template steps using convenience method
        let workflow = SequentialWorkflow::builder("research-chat".to_string())
            .add_template_step(
                "research".to_string(),
                "research-assistant".to_string(),
                json!({
                    "topic": "Rust async",
                    "max_sources": 10,
                }),
            )
            .add_template_step(
                "chat".to_string(),
                "interactive-chat".to_string(),
                json!({
                    "message": "Summarize findings",
                }),
            )
            .build();

        // Verify workflow created correctly
        assert_eq!(workflow.name(), "research-chat");
        assert_eq!(workflow.step_count(), 2);

        // Verify first step is Template type with correct params
        let steps = &workflow.steps;
        match &steps[0].step_type {
            StepType::Template {
                template_id,
                params,
            } => {
                assert_eq!(template_id, "research-assistant");
                assert_eq!(params["topic"], "Rust async");
                assert_eq!(params["max_sources"], 10);
            }
            _ => panic!("Expected Template step type"),
        }

        // Verify second step is Template type
        match &steps[1].step_type {
            StepType::Template {
                template_id,
                params,
            } => {
                assert_eq!(template_id, "interactive-chat");
                assert_eq!(params["message"], "Summarize findings");
            }
            _ => panic!("Expected Template step type"),
        }
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

        let _workflow = SequentialWorkflow::builder("test_workflow".to_string())
            .add_step(step1)
            .add_step(step2)
            .build();

        // Test removed - execute_workflow no longer exists
        // TODO: Update test to use BaseAgent::execute with ExecutionContext
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

        let _workflow = SequentialWorkflow::builder("test_workflow".to_string())
            .add_step(step1)
            .add_step(step2)
            .with_error_strategy(ErrorStrategy::FailFast)
            .build();

        // Test removed - execute_workflow no longer exists
        // TODO: Update test to use BaseAgent::execute with ExecutionContext
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

        let _workflow = SequentialWorkflow::builder("test_workflow".to_string())
            .add_step(step1)
            .add_step(step2)
            .add_step(step3)
            .with_error_strategy(ErrorStrategy::Continue)
            .build();

        // Test removed - execute_workflow no longer exists
        // TODO: Update test to use BaseAgent::execute with ExecutionContext
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
        let status = workflow.state_manager.get_status().await.unwrap();
        assert_eq!(status, WorkflowStatus::Pending);

        // Reset should work
        workflow.reset().await.unwrap();
        let status = workflow.state_manager.get_status().await.unwrap();
        assert_eq!(status, WorkflowStatus::Pending);
    }
}
