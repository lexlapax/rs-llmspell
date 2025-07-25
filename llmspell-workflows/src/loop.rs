// ABOUTME: Loop workflow implementation for iterative processing
// ABOUTME: Supports collection, range, and while-condition iterations with break conditions

use crate::{
    error_handling::{ErrorAction, ErrorHandler},
    hooks::{WorkflowExecutionPhase, WorkflowExecutor, WorkflowHookContext},
    state::StateManager,
    step_executor::StepExecutor,
    traits::{ErrorStrategy, StepResult, WorkflowStep as TraitWorkflowStep},
    types::{StepExecutionContext, WorkflowConfig, WorkflowState},
};
use llmspell_core::{ComponentId, ComponentMetadata, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};
use tracing::{debug, info, warn};

/// Iterator types for loop workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum LoopIterator {
    /// Iterate over a collection of values
    Collection { values: Vec<Value> },
    /// Iterate over a numeric range
    Range { start: i64, end: i64, step: i64 },
    /// Iterate while a condition is true
    WhileCondition {
        /// Condition to evaluate - can reference loop variables
        condition: String,
        /// Maximum iterations to prevent infinite loops
        max_iterations: usize,
    },
}

/// Break condition for early loop termination
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BreakCondition {
    /// Condition expression that when true, breaks the loop
    pub expression: String,
    /// Optional message to include when breaking
    pub message: Option<String>,
}

/// Result aggregation strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResultAggregation {
    /// Collect all iteration results
    CollectAll,
    /// Keep only the last result
    LastOnly,
    /// Keep first N results
    FirstN(usize),
    /// Keep last N results
    LastN(usize),
    /// No aggregation
    None,
}

impl Default for ResultAggregation {
    fn default() -> Self {
        Self::CollectAll
    }
}

/// Configuration for loop workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoopConfig {
    /// Iterator configuration
    pub iterator: LoopIterator,
    /// Steps to execute in each iteration
    pub body: Vec<TraitWorkflowStep>,
    /// Optional break conditions
    pub break_conditions: Vec<BreakCondition>,
    /// Result aggregation strategy
    pub aggregation: ResultAggregation,
    /// Whether to continue on iteration errors
    pub continue_on_error: bool,
    /// Maximum execution time for the entire loop
    pub timeout: Option<Duration>,
    /// Delay between iterations
    pub iteration_delay: Option<Duration>,
}

/// Loop workflow result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoopWorkflowResult {
    /// Workflow name
    pub workflow_name: String,
    /// Whether the workflow completed successfully
    pub success: bool,
    /// Total number of iterations planned
    pub total_iterations: usize,
    /// Number of iterations completed
    pub completed_iterations: usize,
    /// Aggregated results based on strategy
    pub aggregated_results: HashMap<String, Value>,
    /// Reason for breaking if applicable
    pub break_reason: Option<String>,
    /// Total execution time
    pub duration: Duration,
    /// Error message if failed
    pub error: Option<String>,
}

impl LoopWorkflowResult {
    pub fn success(
        workflow_name: String,
        total_iterations: usize,
        completed_iterations: usize,
        aggregated_results: HashMap<String, Value>,
        break_reason: Option<String>,
        duration: Duration,
    ) -> Self {
        Self {
            workflow_name,
            success: true,
            total_iterations,
            completed_iterations,
            aggregated_results,
            break_reason,
            duration,
            error: None,
        }
    }

    pub fn failure(
        workflow_name: String,
        total_iterations: usize,
        completed_iterations: usize,
        aggregated_results: HashMap<String, Value>,
        duration: Duration,
        error: String,
    ) -> Self {
        Self {
            workflow_name,
            success: false,
            total_iterations,
            completed_iterations,
            aggregated_results,
            break_reason: None,
            duration,
            error: Some(error),
        }
    }
}

/// Loop workflow implementation
pub struct LoopWorkflow {
    name: String,
    config: LoopConfig,
    state_manager: StateManager,
    step_executor: StepExecutor,
    error_handler: ErrorHandler,
    error_strategy: ErrorStrategy,
    /// Optional workflow executor for hook integration
    workflow_executor: Option<Arc<WorkflowExecutor>>,
    /// Workflow metadata
    metadata: ComponentMetadata,
}

impl LoopWorkflow {
    /// Create a new loop workflow
    pub fn new(name: String, config: LoopConfig, workflow_config: WorkflowConfig) -> Self {
        let error_strategy = workflow_config.default_error_strategy.clone();
        let error_handler = ErrorHandler::new(error_strategy.clone());
        let state_manager = StateManager::new(workflow_config.clone());
        let step_executor = StepExecutor::new(workflow_config);

        let metadata = ComponentMetadata::new(name.clone(), "Loop workflow".to_string());

        Self {
            name,
            config,
            state_manager,
            step_executor,
            error_handler,
            error_strategy,
            workflow_executor: None,
            metadata,
        }
    }

    /// Create with hook integration
    pub fn new_with_hooks(
        name: String,
        config: LoopConfig,
        workflow_config: WorkflowConfig,
        workflow_executor: Arc<WorkflowExecutor>,
    ) -> Self {
        let error_strategy = workflow_config.default_error_strategy.clone();
        let error_handler = ErrorHandler::new(error_strategy.clone());
        let state_manager =
            StateManager::new_with_hooks(workflow_config.clone(), workflow_executor.clone());
        let step_executor =
            StepExecutor::new_with_hooks(workflow_config, workflow_executor.clone());

        let metadata = ComponentMetadata::new(name.clone(), "Loop workflow with hooks".to_string());

        Self {
            name,
            config,
            state_manager,
            step_executor,
            error_handler,
            error_strategy,
            workflow_executor: Some(workflow_executor),
            metadata,
        }
    }

    /// Create a new loop workflow with builder pattern
    pub fn builder(name: String) -> LoopWorkflowBuilder {
        LoopWorkflowBuilder::new(name)
    }

    /// Get workflow name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Evaluate break conditions
    async fn should_break(
        &self,
        state: &WorkflowState,
        iteration: usize,
    ) -> Result<Option<String>> {
        for condition in &self.config.break_conditions {
            if self
                .evaluate_condition(&condition.expression, state, iteration)
                .await?
            {
                let message = condition
                    .message
                    .clone()
                    .unwrap_or_else(|| format!("Break condition met: {}", condition.expression));
                return Ok(Some(message));
            }
        }
        Ok(None)
    }

    /// Evaluate a condition expression
    async fn evaluate_condition(
        &self,
        expression: &str,
        state: &WorkflowState,
        iteration: usize,
    ) -> Result<bool> {
        // Simple condition evaluation - in a real implementation, this would use
        // an expression evaluator library
        let shared_data = &state.shared_data;

        // Check for simple comparisons
        if expression.contains("==") {
            let parts: Vec<&str> = expression.split("==").collect();
            if parts.len() == 2 {
                let left = self.resolve_value(parts[0].trim(), shared_data, iteration);
                let right = self.resolve_value(parts[1].trim(), shared_data, iteration);
                return Ok(left == right);
            }
        }

        if expression.contains(">") {
            let parts: Vec<&str> = expression.split('>').collect();
            if parts.len() == 2 {
                let left_val = self.resolve_value(parts[0].trim(), shared_data, iteration);
                let right_val = self.resolve_value(parts[1].trim(), shared_data, iteration);

                if let (Ok(left), Ok(right)) = (left_val.parse::<i64>(), right_val.parse::<i64>()) {
                    return Ok(left > right);
                }
            }
        }

        // For now, return false for unsupported expressions
        warn!("Unsupported condition expression: {}", expression);
        Ok(false)
    }

    /// Resolve a value from variables or literals
    fn resolve_value(
        &self,
        value_str: &str,
        shared_data: &HashMap<String, Value>,
        iteration: usize,
    ) -> String {
        // Check if it's a variable reference
        if let Some(var_name) = value_str.strip_prefix('$') {
            if var_name == "iteration" || var_name == "loop_index" {
                return iteration.to_string();
            }
            if let Some(value) = shared_data.get(var_name) {
                // Handle JSON values properly
                match value {
                    Value::String(s) => return s.clone(),
                    Value::Number(n) => return n.to_string(),
                    Value::Bool(b) => return b.to_string(),
                    _ => return value.to_string(),
                }
            }
        }

        // Return as literal
        value_str.to_string()
    }

    /// Generate iterator values
    async fn generate_iterator_values(&self) -> Result<Vec<(usize, Value)>> {
        match &self.config.iterator {
            LoopIterator::Collection { values } => Ok(values
                .iter()
                .enumerate()
                .map(|(i, v)| (i, v.clone()))
                .collect()),
            LoopIterator::Range { start, end, step } => {
                if *step == 0 {
                    return Err(llmspell_core::LLMSpellError::Configuration {
                        message: "Range step cannot be zero".to_string(),
                        source: None,
                    });
                }

                let mut values = Vec::new();
                let mut current = *start;
                let mut index = 0;

                while (*step > 0 && current < *end) || (*step < 0 && current > *end) {
                    values.push((index, Value::Number(current.into())));
                    current += step;
                    index += 1;
                }

                Ok(values)
            }
            LoopIterator::WhileCondition { max_iterations, .. } => {
                // For while conditions, we generate placeholder values up to max_iterations
                // The actual loop will break based on condition evaluation
                Ok((0..*max_iterations)
                    .map(|i| (i, Value::Number(i.into())))
                    .collect())
            }
        }
    }

    /// Execute loop body for one iteration
    async fn execute_iteration(&self, iteration: usize, value: Value) -> Result<Vec<StepResult>> {
        // Set loop variables
        self.state_manager
            .set_shared_data("loop_index".to_string(), Value::Number(iteration.into()))
            .await?;
        self.state_manager
            .set_shared_data("loop_value".to_string(), value)
            .await?;
        self.state_manager
            .set_shared_data("iteration".to_string(), Value::Number(iteration.into()))
            .await?;

        let mut iteration_results = Vec::new();

        for (step_index, step) in self.config.body.iter().enumerate() {
            debug!(
                "Executing iteration {} step {} of {}: {}",
                iteration,
                step_index + 1,
                self.config.body.len(),
                step.name
            );

            // Create execution context
            let shared_data = self.state_manager.get_all_shared_data().await?;
            let mut workflow_state = WorkflowState::new();
            workflow_state.shared_data = shared_data;
            workflow_state.current_step = step_index;
            let context = StepExecutionContext::new(workflow_state, None);

            // Execute step with retry logic (with workflow metadata if hooks are enabled)
            let step_result = if self.workflow_executor.is_some() {
                self.step_executor
                    .execute_step_with_retry_and_metadata(
                        step,
                        context,
                        &self.error_strategy,
                        Some(self.metadata.clone()),
                        Some("loop".to_string()),
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
                iteration_results.push(step_result);
            } else {
                iteration_results.push(step_result.clone());

                if !self.config.continue_on_error {
                    // Handle the failure based on error strategy
                    let error_action = self
                        .error_handler
                        .handle_step_failure(&step_result, Some(&self.error_strategy))
                        .await?;

                    match error_action {
                        ErrorAction::StopWorkflow => {
                            return Err(llmspell_core::LLMSpellError::Workflow {
                                message: format!(
                                    "Loop stopped at iteration {} step {}: {}",
                                    iteration,
                                    step_index + 1,
                                    step.name
                                ),
                                step: Some(step.name.clone()),
                                source: None,
                            });
                        }
                        ErrorAction::ContinueToNext => {
                            warn!(
                                "Continuing after failure in iteration {} step: {}",
                                iteration, step.name
                            );
                            continue;
                        }
                        ErrorAction::RetryStep => {
                            // Already handled by execute_step_with_retry
                            continue;
                        }
                    }
                }
            }
        }

        Ok(iteration_results)
    }

    /// Aggregate results based on strategy
    fn aggregate_results(&self, all_results: Vec<Vec<StepResult>>) -> HashMap<String, Value> {
        match &self.config.aggregation {
            ResultAggregation::CollectAll => {
                let mut aggregated = HashMap::new();
                aggregated.insert(
                    "all_iterations".to_string(),
                    Value::Array(
                        all_results
                            .into_iter()
                            .map(|r| serde_json::to_value(r).unwrap_or(Value::Null))
                            .collect(),
                    ),
                );
                aggregated
            }
            ResultAggregation::LastOnly => {
                let mut aggregated = HashMap::new();
                if let Some(last) = all_results.into_iter().next_back() {
                    aggregated.insert(
                        "last_iteration".to_string(),
                        serde_json::to_value(last).unwrap_or(Value::Null),
                    );
                }
                aggregated
            }
            ResultAggregation::FirstN(n) => {
                let mut aggregated = HashMap::new();
                let first_n: Vec<_> = all_results.into_iter().take(*n).collect();
                aggregated.insert(
                    "iterations".to_string(),
                    Value::Array(
                        first_n
                            .into_iter()
                            .map(|r| serde_json::to_value(r).unwrap_or(Value::Null))
                            .collect(),
                    ),
                );
                aggregated
            }
            ResultAggregation::LastN(n) => {
                let mut aggregated = HashMap::new();
                let count = all_results.len();
                let skip = if count > *n { count - n } else { 0 };
                let last_n: Vec<_> = all_results.into_iter().skip(skip).collect();
                aggregated.insert(
                    "iterations".to_string(),
                    Value::Array(
                        last_n
                            .into_iter()
                            .map(|r| serde_json::to_value(r).unwrap_or(Value::Null))
                            .collect(),
                    ),
                );
                aggregated
            }
            ResultAggregation::None => HashMap::new(),
        }
    }

    /// Execute the loop workflow
    pub async fn execute(&self) -> Result<LoopWorkflowResult> {
        let start_time = Instant::now();
        info!("Starting loop workflow: {}", self.name);

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
                "loop".to_string(),
                WorkflowExecutionPhase::WorkflowStart,
            );
            let _ = workflow_executor.execute_workflow_hooks(hook_ctx).await;
        }

        // Start execution tracking
        self.state_manager.start_execution().await?;

        // Generate iterator values
        let iterator_values = self.generate_iterator_values().await?;
        let total_iterations = iterator_values.len();

        let mut all_results = Vec::new();
        let mut completed_iterations = 0;
        let mut break_reason = None;

        for (iteration, value) in iterator_values {
            // Execute loop iteration start hooks
            if let Some(workflow_executor) = &self.workflow_executor {
                let component_id = llmspell_hooks::ComponentId::new(
                    llmspell_hooks::ComponentType::Workflow,
                    format!("workflow_{}", self.name),
                );
                let workflow_state = self.state_manager.get_state_snapshot().await?;
                let mut hook_ctx = WorkflowHookContext::new(
                    component_id,
                    self.metadata.clone(),
                    workflow_state,
                    "loop".to_string(),
                    WorkflowExecutionPhase::LoopIterationStart,
                );
                hook_ctx = hook_ctx.with_pattern_context(
                    "iteration".to_string(),
                    serde_json::Value::Number(iteration.into()),
                );
                hook_ctx = hook_ctx.with_pattern_context("value".to_string(), value.clone());
                let _ = workflow_executor.execute_workflow_hooks(hook_ctx).await;
            }

            // Check timeout
            if let Some(timeout) = self.config.timeout {
                if start_time.elapsed() > timeout {
                    warn!(
                        "Loop workflow timeout after {} iterations",
                        completed_iterations
                    );
                    break_reason = Some("Workflow timeout exceeded".to_string());
                    break;
                }
            }

            // Check for execution timeout from state manager
            if self.state_manager.check_execution_timeout().await? {
                warn!(
                    "Loop workflow '{}' exceeded maximum execution time",
                    self.name
                );
                break_reason = Some("Maximum execution time exceeded".to_string());
                break;
            }

            // Get current state
            let shared_data = self.state_manager.get_all_shared_data().await?;
            let mut workflow_state = WorkflowState::new();
            workflow_state.shared_data = shared_data;

            // For while conditions, check if we should continue
            if let LoopIterator::WhileCondition { condition, .. } = &self.config.iterator {
                if !self
                    .evaluate_condition(condition, &workflow_state, iteration)
                    .await?
                {
                    debug!("While condition false at iteration {}", iteration);
                    break;
                }
            }

            // Check break conditions
            if let Some(reason) = self.should_break(&workflow_state, iteration).await? {
                info!("Breaking loop: {}", reason);
                break_reason = Some(reason);
                break;
            }

            // Execute iteration
            match self.execute_iteration(iteration, value.clone()).await {
                Ok(results) => {
                    all_results.push(results);
                    completed_iterations += 1;
                    self.state_manager.advance_step().await?;

                    // Execute loop iteration complete hooks
                    if let Some(workflow_executor) = &self.workflow_executor {
                        let component_id = llmspell_hooks::ComponentId::new(
                            llmspell_hooks::ComponentType::Workflow,
                            format!("workflow_{}", self.name),
                        );
                        let workflow_state = self.state_manager.get_state_snapshot().await?;
                        let mut hook_ctx = WorkflowHookContext::new(
                            component_id,
                            self.metadata.clone(),
                            workflow_state,
                            "loop".to_string(),
                            WorkflowExecutionPhase::LoopIterationComplete,
                        );
                        hook_ctx = hook_ctx.with_pattern_context(
                            "iteration".to_string(),
                            serde_json::Value::Number(iteration.into()),
                        );
                        hook_ctx = hook_ctx.with_pattern_context(
                            "completed_iterations".to_string(),
                            serde_json::Value::Number(completed_iterations.into()),
                        );
                        let _ = workflow_executor.execute_workflow_hooks(hook_ctx).await;
                    }
                }
                Err(e) => {
                    if self.config.continue_on_error {
                        warn!("Error in iteration {}: {}", iteration, e);
                        let error_result = StepResult::failure(
                            ComponentId::new(),
                            format!("iteration_{}", iteration),
                            e.to_string(),
                            start_time.elapsed(),
                            0,
                        );
                        all_results.push(vec![error_result]);
                        completed_iterations += 1;
                    } else {
                        self.state_manager.complete_execution(false).await?;
                        return Ok(LoopWorkflowResult::failure(
                            self.name.clone(),
                            total_iterations,
                            completed_iterations,
                            self.aggregate_results(all_results),
                            start_time.elapsed(),
                            e.to_string(),
                        ));
                    }
                }
            }

            // Apply iteration delay if configured
            if let Some(delay) = self.config.iteration_delay {
                tokio::time::sleep(delay).await;
            }
        }

        // Execute loop termination hooks if break occurred
        if break_reason.is_some() {
            if let Some(workflow_executor) = &self.workflow_executor {
                let component_id = llmspell_hooks::ComponentId::new(
                    llmspell_hooks::ComponentType::Workflow,
                    format!("workflow_{}", self.name),
                );
                let workflow_state = self.state_manager.get_state_snapshot().await?;
                let mut hook_ctx = WorkflowHookContext::new(
                    component_id,
                    self.metadata.clone(),
                    workflow_state,
                    "loop".to_string(),
                    WorkflowExecutionPhase::LoopTermination,
                );
                hook_ctx = hook_ctx.with_pattern_context(
                    "break_reason".to_string(),
                    serde_json::Value::String(break_reason.clone().unwrap_or_default()),
                );
                hook_ctx = hook_ctx.with_pattern_context(
                    "completed_iterations".to_string(),
                    serde_json::Value::Number(completed_iterations.into()),
                );
                let _ = workflow_executor.execute_workflow_hooks(hook_ctx).await;
            }
        }

        // Complete execution
        self.state_manager.complete_execution(true).await?;

        // Aggregate results
        let mut aggregated_results = self.aggregate_results(all_results);

        // Add loop metadata
        aggregated_results.insert(
            "loop_metadata".to_string(),
            serde_json::json!({
                "total_iterations": total_iterations,
                "completed_iterations": completed_iterations,
                "break_reason": break_reason,
                "duration_ms": start_time.elapsed().as_millis(),
            }),
        );

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
                "loop".to_string(),
                WorkflowExecutionPhase::WorkflowComplete,
            );
            let _ = workflow_executor.execute_workflow_hooks(hook_ctx).await;
        }

        Ok(LoopWorkflowResult::success(
            self.name.clone(),
            total_iterations,
            completed_iterations,
            aggregated_results,
            break_reason,
            start_time.elapsed(),
        ))
    }
}

/// Builder for loop workflows
pub struct LoopWorkflowBuilder {
    name: String,
    description: String,
    iterator: Option<LoopIterator>,
    body: Vec<TraitWorkflowStep>,
    break_conditions: Vec<BreakCondition>,
    aggregation: ResultAggregation,
    continue_on_error: bool,
    timeout: Option<Duration>,
    iteration_delay: Option<Duration>,
    workflow_config: WorkflowConfig,
    workflow_executor: Option<Arc<WorkflowExecutor>>,
}

impl LoopWorkflowBuilder {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: String::new(),
            iterator: None,
            body: Vec::new(),
            break_conditions: Vec::new(),
            aggregation: ResultAggregation::CollectAll,
            continue_on_error: false,
            timeout: None,
            iteration_delay: None,
            workflow_config: WorkflowConfig::default(),
            workflow_executor: None,
        }
    }

    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = desc.into();
        self
    }

    pub fn with_collection<T: Into<Value>>(mut self, values: Vec<T>) -> Self {
        self.iterator = Some(LoopIterator::Collection {
            values: values.into_iter().map(|v| v.into()).collect(),
        });
        self
    }

    pub fn with_range(mut self, start: i64, end: i64, step: i64) -> Self {
        self.iterator = Some(LoopIterator::Range { start, end, step });
        self
    }

    pub fn with_while_condition(
        mut self,
        condition: impl Into<String>,
        max_iterations: usize,
    ) -> Self {
        self.iterator = Some(LoopIterator::WhileCondition {
            condition: condition.into(),
            max_iterations,
        });
        self
    }

    pub fn add_step(mut self, step: TraitWorkflowStep) -> Self {
        self.body.push(step);
        self
    }

    pub fn add_break_condition(
        mut self,
        expression: impl Into<String>,
        message: Option<String>,
    ) -> Self {
        self.break_conditions.push(BreakCondition {
            expression: expression.into(),
            message,
        });
        self
    }

    pub fn with_aggregation(mut self, aggregation: ResultAggregation) -> Self {
        self.aggregation = aggregation;
        self
    }

    pub fn continue_on_error(mut self, continue_on_error: bool) -> Self {
        self.continue_on_error = continue_on_error;
        self
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    pub fn with_iteration_delay(mut self, delay: Duration) -> Self {
        self.iteration_delay = Some(delay);
        self
    }

    pub fn with_workflow_config(mut self, config: WorkflowConfig) -> Self {
        self.workflow_config = config;
        self
    }

    pub fn with_error_strategy(mut self, strategy: ErrorStrategy) -> Self {
        self.workflow_config.default_error_strategy = strategy;
        self
    }

    /// Enable hook integration with a WorkflowExecutor
    pub fn with_hooks(mut self, workflow_executor: Arc<WorkflowExecutor>) -> Self {
        self.workflow_executor = Some(workflow_executor);
        self
    }

    pub fn build(self) -> Result<LoopWorkflow> {
        let iterator =
            self.iterator
                .ok_or_else(|| llmspell_core::LLMSpellError::Configuration {
                    message: "Loop iterator not configured".to_string(),
                    source: None,
                })?;

        // Validate iterator configuration
        if let LoopIterator::Range {
            start: _,
            end: _,
            step,
        } = &iterator
        {
            if *step == 0 {
                return Err(llmspell_core::LLMSpellError::Configuration {
                    message: "Range step cannot be zero".to_string(),
                    source: None,
                });
            }
        }

        if self.body.is_empty() {
            return Err(llmspell_core::LLMSpellError::Configuration {
                message: "Loop body cannot be empty".to_string(),
                source: None,
            });
        }

        let config = LoopConfig {
            iterator,
            body: self.body,
            break_conditions: self.break_conditions,
            aggregation: self.aggregation,
            continue_on_error: self.continue_on_error,
            timeout: self.timeout,
            iteration_delay: self.iteration_delay,
        };

        if let Some(workflow_executor) = self.workflow_executor {
            Ok(LoopWorkflow::new_with_hooks(
                self.name,
                config,
                self.workflow_config,
                workflow_executor,
            ))
        } else {
            Ok(LoopWorkflow::new(self.name, config, self.workflow_config))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::StepType;

    #[tokio::test]
    async fn test_loop_builder() {
        let workflow = LoopWorkflowBuilder::new("test_loop")
            .description("Test loop workflow")
            .with_range(0, 10, 1)
            .add_step(TraitWorkflowStep::new(
                "print_value".to_string(),
                StepType::Tool {
                    tool_name: "mock_tool".to_string(),
                    parameters: serde_json::json!({}),
                },
            ))
            .build()
            .unwrap();

        assert_eq!(workflow.name(), "test_loop");
        assert_eq!(workflow.config.body.len(), 1);
    }

    #[tokio::test]
    async fn test_range_validation() {
        // Invalid step
        let result = LoopWorkflowBuilder::new("test")
            .with_range(0, 10, 0)
            .add_step(TraitWorkflowStep::new(
                "step".to_string(),
                StepType::Tool {
                    tool_name: "tool".to_string(),
                    parameters: serde_json::json!({}),
                },
            ))
            .build();

        assert!(result.is_err());
    }
}
