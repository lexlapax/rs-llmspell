// ABOUTME: Loop workflow implementation for iterative processing
// ABOUTME: Supports collection, range, and while-condition iterations with break conditions

use crate::{
    error_handling::{ErrorAction, ErrorHandler},
    hooks::{WorkflowExecutionPhase, WorkflowExecutor, WorkflowHookContext},
    result::{WorkflowError, WorkflowResult, WorkflowType},
    state::StateManager,
    step_executor::StepExecutor,
    traits::{ErrorStrategy, StepResult, WorkflowStep as TraitWorkflowStep},
    types::{StepExecutionContext, WorkflowConfig, WorkflowState},
    StepType,
};
use async_trait::async_trait;
use llmspell_core::{
    execution_context::ExecutionContext,
    traits::base_agent::BaseAgent,
    traits::workflow::{
        Config as CoreWorkflowConfig, Status as CoreWorkflowStatus, StepResult as CoreStepResult,
        Workflow, WorkflowStep as CoreWorkflowStep,
    },
    types::{AgentInput, AgentOutput},
    ComponentId, ComponentLookup, ComponentMetadata, LLMSpellError, Result,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::RwLock;
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

impl LoopConfig {
    /// Create a new builder for LoopConfig
    pub fn builder() -> LoopConfigBuilder {
        LoopConfigBuilder::new()
    }
}

/// Builder for LoopConfig
pub struct LoopConfigBuilder {
    iterator: Option<LoopIterator>,
    body: Vec<TraitWorkflowStep>,
    break_conditions: Vec<BreakCondition>,
    aggregation: ResultAggregation,
    continue_on_error: bool,
    timeout: Option<Duration>,
    iteration_delay: Option<Duration>,
}

impl LoopConfigBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            iterator: None,
            body: Vec::new(),
            break_conditions: Vec::new(),
            aggregation: ResultAggregation::CollectAll,
            continue_on_error: false,
            timeout: None,
            iteration_delay: None,
        }
    }

    /// Set the iterator configuration
    pub fn iterator(mut self, iterator: LoopIterator) -> Self {
        self.iterator = Some(iterator);
        self
    }

    /// Add a step to the loop body
    pub fn add_step(mut self, step: TraitWorkflowStep) -> Self {
        self.body.push(step);
        self
    }

    /// Set all body steps at once
    pub fn body(mut self, steps: Vec<TraitWorkflowStep>) -> Self {
        self.body = steps;
        self
    }

    /// Add a break condition
    pub fn add_break_condition(mut self, condition: BreakCondition) -> Self {
        self.break_conditions.push(condition);
        self
    }

    /// Set the result aggregation strategy
    pub fn aggregation(mut self, aggregation: ResultAggregation) -> Self {
        self.aggregation = aggregation;
        self
    }

    /// Set whether to continue on iteration errors
    pub fn continue_on_error(mut self, enabled: bool) -> Self {
        self.continue_on_error = enabled;
        self
    }

    /// Set timeout for the entire loop
    pub fn timeout(mut self, timeout: Option<Duration>) -> Self {
        self.timeout = timeout;
        self
    }

    /// Set delay between iterations
    pub fn iteration_delay(mut self, delay: Option<Duration>) -> Self {
        self.iteration_delay = delay;
        self
    }

    /// Build the final LoopConfig with validation
    pub fn build(self) -> Result<LoopConfig> {
        let iterator = self.iterator.ok_or_else(|| LLMSpellError::Validation {
            message: "LoopConfig requires an iterator".to_string(),
            field: Some("iterator".to_string()),
        })?;

        if self.body.is_empty() {
            return Err(LLMSpellError::Validation {
                message: "LoopConfig requires at least one body step".to_string(),
                field: Some("body".to_string()),
            });
        }

        Ok(LoopConfig {
            iterator,
            body: self.body,
            break_conditions: self.break_conditions,
            aggregation: self.aggregation,
            continue_on_error: self.continue_on_error,
            timeout: self.timeout,
            iteration_delay: self.iteration_delay,
        })
    }
}

impl Default for LoopConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
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
    /// Core workflow configuration for Workflow trait
    core_config: CoreWorkflowConfig,
    /// Core workflow steps for Workflow trait
    core_steps: Arc<RwLock<Vec<CoreWorkflowStep>>>,
    /// Core workflow results for Workflow trait
    core_results: Arc<RwLock<Vec<CoreStepResult>>>,
}

impl LoopWorkflow {
    /// Create a new loop workflow
    pub fn new(name: String, config: LoopConfig, workflow_config: WorkflowConfig) -> Self {
        let error_strategy = workflow_config.default_error_strategy.clone();
        let error_handler = ErrorHandler::new(error_strategy.clone());
        let state_manager = StateManager::new(workflow_config.clone());
        let step_executor = StepExecutor::new(workflow_config.clone());

        let metadata = ComponentMetadata::new(name.clone(), "Loop workflow".to_string());

        // Create core workflow config from our config
        let core_config = CoreWorkflowConfig::new()
            .with_max_parallel(Some(1)) // Loop execution is sequential by nature
            .with_continue_on_error(config.continue_on_error)
            .with_timeout(config.timeout.or(workflow_config.max_execution_time));

        Self {
            name,
            config,
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
        config: LoopConfig,
        workflow_config: WorkflowConfig,
        workflow_executor: Arc<WorkflowExecutor>,
    ) -> Self {
        let error_strategy = workflow_config.default_error_strategy.clone();
        let error_handler = ErrorHandler::new(error_strategy.clone());
        let state_manager =
            StateManager::new_with_hooks(workflow_config.clone(), workflow_executor.clone());
        let step_executor =
            StepExecutor::new_with_hooks(workflow_config.clone(), workflow_executor.clone());

        let metadata = ComponentMetadata::new(name.clone(), "Loop workflow with hooks".to_string());

        // Create core workflow config from our config
        let core_config = CoreWorkflowConfig::new()
            .with_max_parallel(Some(1)) // Loop execution is sequential by nature
            .with_continue_on_error(config.continue_on_error)
            .with_timeout(config.timeout.or(workflow_config.max_execution_time));

        Self {
            name,
            config,
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

    /// Create with registry for component lookup
    pub fn new_with_registry(
        name: String,
        config: LoopConfig,
        workflow_config: WorkflowConfig,
        registry: Option<Arc<dyn ComponentLookup>>,
    ) -> Self {
        let error_strategy = workflow_config.default_error_strategy.clone();
        let error_handler = ErrorHandler::new(error_strategy.clone());
        let state_manager = StateManager::new(workflow_config.clone());
        let step_executor = if let Some(reg) = registry {
            StepExecutor::new_with_registry(workflow_config.clone(), reg)
        } else {
            StepExecutor::new(workflow_config.clone())
        };

        let metadata = ComponentMetadata::new(name.clone(), "Loop workflow".to_string());

        // Create core workflow config from our config
        let core_config = CoreWorkflowConfig::new()
            .with_max_parallel(Some(1)) // Loop execution is sequential by nature
            .with_continue_on_error(config.continue_on_error)
            .with_timeout(config.timeout.or(workflow_config.max_execution_time));

        Self {
            name,
            config,
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

    /// Create with both hooks and registry
    pub fn new_with_hooks_and_registry(
        name: String,
        config: LoopConfig,
        workflow_config: WorkflowConfig,
        workflow_executor: Arc<WorkflowExecutor>,
        registry: Option<Arc<dyn ComponentLookup>>,
    ) -> Self {
        let error_strategy = workflow_config.default_error_strategy.clone();
        let error_handler = ErrorHandler::new(error_strategy.clone());
        let state_manager =
            StateManager::new_with_hooks(workflow_config.clone(), workflow_executor.clone());
        let step_executor = if let Some(reg) = registry {
            StepExecutor::new_with_hooks_and_registry(
                workflow_config.clone(),
                workflow_executor.clone(),
                reg,
            )
        } else {
            StepExecutor::new_with_hooks(workflow_config.clone(), workflow_executor.clone())
        };

        let metadata = ComponentMetadata::new(name.clone(), "Loop workflow with hooks".to_string());

        // Create core workflow config from our config
        let core_config = CoreWorkflowConfig::new()
            .with_max_parallel(Some(1)) // Loop execution is sequential by nature
            .with_continue_on_error(config.continue_on_error)
            .with_timeout(config.timeout.or(workflow_config.max_execution_time));

        Self {
            name,
            config,
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
    async fn execute_iteration(
        &self,
        iteration: usize,
        value: Value,
        execution_component_id: ComponentId,
    ) -> Result<Vec<StepResult>> {
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
            // CRITICAL: Use the workflow's execution_component_id, not a new one!
            workflow_state.execution_id = execution_component_id;
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
                        }
                        ErrorAction::RetryStep => {
                            // Already handled by execute_step_with_retry
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

    /// Execute the loop workflow with state-based output
    ///
    /// This method executes the loop workflow and writes all outputs directly to state.
    /// Each iteration's outputs are stored with keys following the pattern:
    /// - `workflow:{execution_id}:iteration_{n}:{step_name}` for individual step outputs
    /// - `workflow:{execution_id}:aggregated` for aggregated results (if applicable)
    ///
    /// # Arguments
    ///
    /// * `context` - The execution context containing state access
    ///
    /// # Returns
    ///
    /// A `WorkflowResult` containing only execution metadata, with state keys
    /// indicating where the actual outputs have been written.
    pub async fn execute_with_state(&self, context: &ExecutionContext) -> Result<WorkflowResult> {
        let start_time = Instant::now();
        // Generate ComponentId once and use it consistently
        let execution_component_id = ComponentId::new();
        let execution_id = format!("loop_{}", execution_component_id);

        // Get state access or return error
        let state = context
            .state
            .as_ref()
            .ok_or_else(|| LLMSpellError::Workflow {
                message: "State access not available in context".to_string(),
                step: None,
                source: None,
            })?;

        info!(
            "Starting state-based loop workflow: {} with execution_id: {}",
            self.name, execution_id
        );

        // Initialize workflow state
        self.state_manager.start_execution().await?;

        // Track state keys where we write outputs
        let mut state_keys = Vec::new();

        // Generate iterator values
        let iterator_values = self.generate_iterator_values().await?;
        let total_iterations = iterator_values.len();

        let mut completed_iterations = 0;
        let mut failed_iterations = 0;
        let mut skipped_iterations = 0;
        let mut aggregated_results = HashMap::new();
        let mut break_reason = None;
        let mut last_error = None;

        for (iteration, value) in iterator_values {
            debug!(
                "Loop workflow '{}' iteration {}: Processing",
                self.name, iteration
            );

            // Add iteration context to workflow state
            self.state_manager
                .set_shared_data("iteration".to_string(), Value::Number(iteration.into()))
                .await?;
            self.state_manager
                .set_shared_data("iteration_value".to_string(), value.clone())
                .await?;
            self.state_manager
                .set_shared_data(
                    "total_iterations".to_string(),
                    Value::Number(total_iterations.into()),
                )
                .await?;

            // Get current workflow state for evaluation
            let workflow_state = self.state_manager.get_state_snapshot().await?;

            // Check break conditions
            for break_condition in &self.config.break_conditions {
                if self
                    .evaluate_condition(&break_condition.expression, &workflow_state, iteration)
                    .await?
                {
                    let reason = break_condition.message.clone().unwrap_or_else(|| {
                        format!("Break condition met: {}", break_condition.expression)
                    });
                    debug!(
                        "Loop workflow '{}' breaking at iteration {}: {}",
                        self.name, iteration, reason
                    );
                    break_reason = Some(reason);
                    skipped_iterations = total_iterations.saturating_sub(iteration + 1);
                    break;
                }
            }

            if break_reason.is_some() {
                break;
            }

            // Check timeout if configured
            if let Some(timeout) = self.config.timeout {
                if start_time.elapsed() > timeout {
                    let error = WorkflowError::Timeout {
                        duration: timeout,
                        message: format!("Loop workflow timed out after {} iterations", iteration),
                    };
                    last_error = Some(error.clone());
                    failed_iterations += 1;
                    break;
                }
            }

            // Execute loop body steps
            let mut iteration_succeeded = true;
            let mut last_step_output = value.clone();

            for (step_idx, step) in self.config.body.iter().enumerate() {
                let step_name = format!("step_{}", step_idx);
                let state_key =
                    WorkflowResult::generate_iteration_key(&execution_id, iteration, &step_name);

                // Create execution context for the step
                let step_context =
                    StepExecutionContext::new(workflow_state.clone(), self.config.timeout);

                match self.step_executor.execute_step(step, step_context).await {
                    Ok(result) => {
                        // Write step output to state
                        let output_value =
                            serde_json::to_value(&result.output).unwrap_or_else(|_| {
                                Value::String("Failed to serialize output".to_string())
                            });

                        if let Err(e) = state.write(&state_key, output_value.clone()).await {
                            warn!("Failed to write step output to state: {}", e);
                            if !self.config.continue_on_error {
                                last_error = Some(WorkflowError::StateAccessFailed {
                                    operation: "write".to_string(),
                                    error: e.to_string(),
                                });
                                iteration_succeeded = false;
                                break;
                            }
                        } else {
                            state_keys.push(state_key.clone());
                            debug!(
                                "Loop workflow '{}' iteration {} step {} output written to: {}",
                                self.name, iteration, step_idx, state_key
                            );
                        }

                        // Update last step output for aggregation
                        last_step_output = output_value;

                        // Update workflow state with step result
                        self.state_manager
                            .record_step_result(result.clone())
                            .await?;
                    }
                    Err(e) => {
                        warn!(
                            "Loop workflow '{}' iteration {} step {} failed: {}",
                            self.name, iteration, step_idx, e
                        );

                        if !self.config.continue_on_error {
                            last_error = Some(WorkflowError::StepExecutionFailed {
                                step_name: step_name.clone(),
                                reason: e.to_string(),
                            });
                            iteration_succeeded = false;
                            break;
                        }
                    }
                }
            }

            if iteration_succeeded {
                completed_iterations += 1;

                // Aggregate results based on strategy
                match &self.config.aggregation {
                    ResultAggregation::CollectAll => {
                        aggregated_results
                            .insert(format!("iteration_{}", iteration), last_step_output);
                    }
                    ResultAggregation::LastOnly => {
                        aggregated_results.clear();
                        aggregated_results.insert("last".to_string(), last_step_output);
                    }
                    ResultAggregation::FirstN(n) if completed_iterations <= *n => {
                        aggregated_results
                            .insert(format!("iteration_{}", iteration), last_step_output);
                    }
                    ResultAggregation::LastN(n) => {
                        // Keep only last N results
                        if aggregated_results.len() >= *n {
                            // Remove oldest
                            let oldest_key = format!("iteration_{}", iteration.saturating_sub(*n));
                            aggregated_results.remove(&oldest_key);
                        }
                        aggregated_results
                            .insert(format!("iteration_{}", iteration), last_step_output);
                    }
                    ResultAggregation::None => {
                        // No aggregation
                    }
                    _ => {}
                }
            } else {
                failed_iterations += 1;
                if !self.config.continue_on_error {
                    break;
                }
            }

            // Apply iteration delay if configured
            if let Some(delay) = self.config.iteration_delay {
                tokio::time::sleep(delay).await;
            }
        }

        // Write aggregated results to state if we have any
        if !aggregated_results.is_empty() {
            let aggregated_key = WorkflowResult::generate_aggregated_key(&execution_id);
            let aggregated_value = Value::Object(aggregated_results.into_iter().collect());

            if let Err(e) = state.write(&aggregated_key, aggregated_value).await {
                warn!("Failed to write aggregated results to state: {}", e);
            } else {
                state_keys.push(aggregated_key.clone());
                debug!(
                    "Loop workflow '{}' aggregated results written to: {}",
                    self.name, aggregated_key
                );
            }
        }

        let duration = start_time.elapsed();

        // Determine overall success
        let success = failed_iterations == 0 && last_error.is_none();

        if success {
            if let Some(reason) = break_reason {
                info!(
                    "Loop workflow '{}' completed with early break: {} ({}/{} iterations, {:?})",
                    self.name, reason, completed_iterations, total_iterations, duration
                );
            } else {
                info!(
                    "Loop workflow '{}' completed successfully: {}/{} iterations in {:?}",
                    self.name, completed_iterations, total_iterations, duration
                );
            }

            Ok(WorkflowResult::partial(
                execution_id,
                WorkflowType::Loop,
                self.name.clone(),
                state_keys,
                completed_iterations,
                failed_iterations,
                skipped_iterations,
                duration,
                None,
            ))
        } else {
            warn!(
                "Loop workflow '{}' failed: {}/{} iterations completed, {} failed in {:?}",
                self.name, completed_iterations, total_iterations, failed_iterations, duration
            );

            Ok(WorkflowResult::failure(
                execution_id,
                WorkflowType::Loop,
                self.name.clone(),
                last_error.unwrap_or_else(|| WorkflowError::General {
                    message: "Loop workflow failed".to_string(),
                }),
                state_keys,
                completed_iterations,
                failed_iterations,
                duration,
            ))
        }
    }

    /// Execute the loop workflow
    pub async fn execute_workflow(&self) -> Result<LoopWorkflowResult> {
        let start_time = Instant::now();
        // Generate ComponentId once and use it consistently
        let execution_component_id = ComponentId::new();
        let execution_id = execution_component_id.to_string();
        info!(
            "Starting loop workflow: {} (execution: {})",
            self.name, execution_id
        );

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
            // CRITICAL: Use the workflow's execution_component_id, not a new one!
            workflow_state.execution_id = execution_component_id;
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
            match self
                .execute_iteration(iteration, value.clone(), execution_component_id)
                .await
            {
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

#[async_trait]
impl BaseAgent for LoopWorkflow {
    fn metadata(&self) -> &ComponentMetadata {
        &self.metadata
    }

    async fn execute(&self, input: AgentInput, context: ExecutionContext) -> Result<AgentOutput> {
        // Convert AgentInput to workflow execution
        // The workflow will use the input text as an execution trigger

        // Validate input first
        self.validate_input(&input).await?;

        // Check if we should use state-based execution
        let (workflow_result, output_text, execution_id_for_outputs) = if context.state.is_some() {
            // Use state-based execution
            let result = self.execute_with_state(&context).await?;

            // Store execution_id for output collection
            let exec_id = result.execution_id.clone();

            let text = if result.success {
                let break_info = if result.steps_skipped > 0 {
                    format!(" ({} iterations skipped)", result.steps_skipped)
                } else {
                    String::new()
                };

                format!(
                    "Loop workflow '{}' completed successfully. {} iterations executed, {} failed{}. Duration: {:?}. Outputs written to {} state keys.",
                    result.workflow_name,
                    result.steps_executed,
                    result.steps_failed,
                    break_info,
                    result.duration,
                    result.state_keys.len()
                )
            } else {
                format!(
                    "Loop workflow '{}' failed: {}. {} iterations executed, {} failed. Duration: {:?}",
                    result.workflow_name,
                    result.error.as_ref().map(|e| e.to_string()).unwrap_or_else(|| "Unknown error".to_string()),
                    result.steps_executed,
                    result.steps_failed,
                    result.duration
                )
            };

            // Convert WorkflowResult to legacy format for compatibility
            let legacy_result = LoopWorkflowResult {
                workflow_name: result.workflow_name.clone(),
                success: result.success,
                total_iterations: result.steps_executed + result.steps_skipped,
                completed_iterations: result.steps_executed,
                aggregated_results: HashMap::new(), // Not used in state-based execution
                break_reason: if result.steps_skipped > 0 {
                    Some("Early termination".to_string())
                } else {
                    None
                },
                duration: result.duration,
                error: result.error.as_ref().map(|e| e.to_string()),
            };

            (legacy_result, text, Some(exec_id))
        } else {
            // Use legacy execution
            let workflow_result = self.execute_workflow().await?;

            let text = if workflow_result.success {
                let break_info = if let Some(reason) = &workflow_result.break_reason {
                    format!(" (broke early: {})", reason)
                } else {
                    String::new()
                };

                format!(
                    "Loop workflow '{}' completed successfully. {} of {} iterations completed{}. Duration: {:?}",
                    workflow_result.workflow_name,
                    workflow_result.completed_iterations,
                    workflow_result.total_iterations,
                    break_info,
                    workflow_result.duration
                )
            } else {
                format!(
                    "Loop workflow '{}' failed: {}. {} of {} iterations completed. Duration: {:?}",
                    workflow_result.workflow_name,
                    workflow_result.error.as_deref().unwrap_or("Unknown error"),
                    workflow_result.completed_iterations,
                    workflow_result.total_iterations,
                    workflow_result.duration
                )
            };

            (workflow_result, text, None)
        };

        // Build AgentOutput with execution metadata
        #[allow(clippy::cast_possible_truncation)]
        let execution_time_ms = workflow_result.duration.as_millis() as u64;
        let mut metadata = llmspell_core::types::OutputMetadata {
            execution_time_ms: Some(execution_time_ms),
            ..Default::default()
        };
        metadata
            .extra
            .insert("workflow_type".to_string(), serde_json::json!("loop"));
        metadata.extra.insert(
            "workflow_name".to_string(),
            serde_json::json!(workflow_result.workflow_name),
        );
        metadata.extra.insert(
            "total_iterations".to_string(),
            serde_json::json!(workflow_result.total_iterations),
        );
        metadata.extra.insert(
            "completed_iterations".to_string(),
            serde_json::json!(workflow_result.completed_iterations),
        );
        metadata.extra.insert(
            "break_reason".to_string(),
            serde_json::json!(workflow_result.break_reason),
        );
        metadata.extra.insert(
            "aggregated_results_count".to_string(),
            serde_json::json!(workflow_result.aggregated_results.len()),
        );

        // Add iterator type information
        let iterator_type = match &self.config.iterator {
            LoopIterator::Collection { values } => {
                metadata
                    .extra
                    .insert("iterator_type".to_string(), serde_json::json!("collection"));
                metadata.extra.insert(
                    "collection_size".to_string(),
                    serde_json::json!(values.len()),
                );
                "collection"
            }
            LoopIterator::Range { start, end, step } => {
                metadata
                    .extra
                    .insert("iterator_type".to_string(), serde_json::json!("range"));
                metadata
                    .extra
                    .insert("range_start".to_string(), serde_json::json!(start));
                metadata
                    .extra
                    .insert("range_end".to_string(), serde_json::json!(end));
                metadata
                    .extra
                    .insert("range_step".to_string(), serde_json::json!(step));
                "range"
            }
            LoopIterator::WhileCondition {
                condition,
                max_iterations,
            } => {
                metadata.extra.insert(
                    "iterator_type".to_string(),
                    serde_json::json!("while_condition"),
                );
                metadata
                    .extra
                    .insert("condition".to_string(), serde_json::json!(condition));
                metadata.extra.insert(
                    "max_iterations".to_string(),
                    serde_json::json!(max_iterations),
                );
                "while_condition"
            }
        };
        metadata.extra.insert(
            "iterator_type".to_string(),
            serde_json::json!(iterator_type),
        );
        metadata.extra.insert(
            "continue_on_error".to_string(),
            serde_json::json!(self.config.continue_on_error),
        );

        // Add execution_id to metadata
        if let Some(execution_id) = &execution_id_for_outputs {
            metadata
                .extra
                .insert("execution_id".to_string(), serde_json::json!(execution_id));
            metadata
                .extra
                .insert("workflow_id".to_string(), serde_json::json!(execution_id));

            // Collect agent outputs from state if available
            let mut agent_outputs = serde_json::Map::new();
            if let Some(ref state) = context.state {
                for step in &self.config.body {
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

        // Validate that we have steps to execute in the loop body
        if self.config.body.is_empty() {
            return Err(LLMSpellError::Validation {
                message: "Cannot execute loop workflow without steps in body".to_string(),
                field: Some("body".to_string()),
            });
        }

        // Validate iterator configuration
        match &self.config.iterator {
            LoopIterator::Collection { values } => {
                if values.is_empty() {
                    return Err(LLMSpellError::Validation {
                        message: "Collection iterator cannot be empty".to_string(),
                        field: Some("iterator.values".to_string()),
                    });
                }
            }
            LoopIterator::Range { start, end, step } => {
                if *step == 0 {
                    return Err(LLMSpellError::Validation {
                        message: "Range step cannot be zero".to_string(),
                        field: Some("iterator.step".to_string()),
                    });
                }
                if (*step > 0 && start >= end) || (*step < 0 && start <= end) {
                    return Err(LLMSpellError::Validation {
                        message: "Range configuration will not iterate".to_string(),
                        field: Some("iterator".to_string()),
                    });
                }
            }
            LoopIterator::WhileCondition {
                condition,
                max_iterations,
            } => {
                if condition.is_empty() {
                    return Err(LLMSpellError::Validation {
                        message: "While condition cannot be empty".to_string(),
                        field: Some("iterator.condition".to_string()),
                    });
                }
                if *max_iterations == 0 {
                    return Err(LLMSpellError::Validation {
                        message: "Max iterations must be at least 1".to_string(),
                        field: Some("iterator.max_iterations".to_string()),
                    });
                }
            }
        }

        Ok(())
    }

    async fn handle_error(&self, error: LLMSpellError) -> Result<AgentOutput> {
        // Handle workflow-specific errors gracefully
        let error_text = match &error {
            LLMSpellError::Workflow { message, step, .. } => {
                if let Some(step_name) = step {
                    format!("Loop workflow error in step '{}': {}", step_name, message)
                } else {
                    format!("Loop workflow error: {}", message)
                }
            }
            LLMSpellError::Validation { message, field } => {
                if let Some(field_name) = field {
                    format!("Validation error in field '{}': {}", field_name, message)
                } else {
                    format!("Validation error: {}", message)
                }
            }
            _ => format!("Loop workflow error: {}", error),
        };

        let mut metadata = llmspell_core::types::OutputMetadata::default();
        metadata.extra.insert(
            "error_type".to_string(),
            serde_json::json!("workflow_error"),
        );
        metadata
            .extra
            .insert("workflow_type".to_string(), serde_json::json!("loop"));
        metadata
            .extra
            .insert("workflow_name".to_string(), serde_json::json!(self.name));
        metadata.extra.insert(
            "body_steps_count".to_string(),
            serde_json::json!(self.config.body.len()),
        );

        Ok(AgentOutput::text(error_text).with_metadata(metadata))
    }
}

#[async_trait]
impl Workflow for LoopWorkflow {
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
        use crate::traits::WorkflowStatus;
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
    registry: Option<Arc<dyn ComponentLookup>>,
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
            registry: None,
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

    /// Set the component registry for component lookup
    pub fn with_registry(mut self, registry: Arc<dyn ComponentLookup>) -> Self {
        self.registry = Some(registry);
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

        match (self.workflow_executor, self.registry) {
            (Some(executor), Some(registry)) => Ok(LoopWorkflow::new_with_hooks_and_registry(
                self.name,
                config,
                self.workflow_config,
                executor,
                Some(registry),
            )),
            (Some(executor), None) => Ok(LoopWorkflow::new_with_hooks(
                self.name,
                config,
                self.workflow_config,
                executor,
            )),
            (None, Some(registry)) => Ok(LoopWorkflow::new_with_registry(
                self.name,
                config,
                self.workflow_config,
                Some(registry),
            )),
            (None, None) => Ok(LoopWorkflow::new(self.name, config, self.workflow_config)),
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
