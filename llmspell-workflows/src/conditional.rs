//! ABOUTME: Conditional workflow implementation for branch-based execution
//! ABOUTME: Executes workflow branches based on condition evaluation with fallback support

use super::conditions::{
    Condition, ConditionEvaluationContext, ConditionEvaluator, ConditionResult,
};
use super::error_handling::{ErrorAction, ErrorHandler};
use super::hooks::{WorkflowExecutionPhase, WorkflowExecutor, WorkflowHookContext};
use super::state::{ExecutionStats, StateManager};
use super::step_executor::StepExecutor;
use super::traits::{ErrorStrategy, StepResult, WorkflowStatus, WorkflowStep};
use super::types::{StepExecutionContext, WorkflowConfig};
use llmspell_core::{ComponentId, ComponentMetadata, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::{debug, error, info, warn};

/// Conditional workflow branch containing steps to execute when condition is met
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConditionalBranch {
    /// Unique identifier for this branch
    pub id: ComponentId,
    /// Human-readable name for the branch
    pub name: String,
    /// Condition that must be true to execute this branch
    pub condition: Condition,
    /// Steps to execute when condition is met
    pub steps: Vec<WorkflowStep>,
    /// Whether this is the default branch (executes if no other conditions match)
    pub is_default: bool,
}

impl ConditionalBranch {
    /// Create a new conditional branch
    pub fn new(name: String, condition: Condition) -> Self {
        Self {
            id: ComponentId::new(),
            name,
            condition,
            steps: Vec::new(),
            is_default: false,
        }
    }

    /// Create a default branch (executes when no conditions match)
    pub fn default(name: String) -> Self {
        Self {
            id: ComponentId::new(),
            name,
            condition: Condition::Always, // Always true condition
            steps: Vec::new(),
            is_default: true,
        }
    }

    /// Add a step to this branch
    pub fn with_step(mut self, step: WorkflowStep) -> Self {
        self.steps.push(step);
        self
    }

    /// Add multiple steps to this branch
    pub fn with_steps(mut self, steps: Vec<WorkflowStep>) -> Self {
        self.steps.extend(steps);
        self
    }
}

/// Branch execution result
#[derive(Debug, Clone)]
pub struct BranchExecutionResult {
    /// Branch that was executed
    pub branch_id: ComponentId,
    /// Branch name
    pub branch_name: String,
    /// Condition evaluation result
    pub condition_result: ConditionResult,
    /// Results from executed steps
    pub step_results: Vec<StepResult>,
    /// Whether the branch execution was successful
    pub success: bool,
    /// Total execution time for the branch
    pub duration: Duration,
}

impl BranchExecutionResult {
    /// Create a successful branch execution result
    pub fn success(
        branch_id: ComponentId,
        branch_name: String,
        condition_result: ConditionResult,
        step_results: Vec<StepResult>,
        duration: Duration,
    ) -> Self {
        let success = step_results.iter().all(|r| r.success);
        Self {
            branch_id,
            branch_name,
            condition_result,
            step_results,
            success,
            duration,
        }
    }
}

/// Configuration for conditional workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConditionalWorkflowConfig {
    /// Whether to execute all matching branches or just the first one
    pub execute_all_matching: bool,
    /// Whether to execute the default branch if no conditions match
    pub execute_default_on_no_match: bool,
    /// Maximum number of branches to evaluate (prevents infinite loops)
    pub max_branches_to_evaluate: usize,
    /// Timeout for condition evaluation
    pub condition_evaluation_timeout_ms: u64,
    /// Whether to short-circuit evaluation (stop on first true condition)
    pub short_circuit_evaluation: bool,
}

impl Default for ConditionalWorkflowConfig {
    fn default() -> Self {
        Self {
            execute_all_matching: false, // Execute only first matching branch
            execute_default_on_no_match: true,
            max_branches_to_evaluate: 100,
            condition_evaluation_timeout_ms: 1000, // 1 second
            short_circuit_evaluation: true,
        }
    }
}

/// Conditional workflow that executes branches based on condition evaluation
pub struct ConditionalWorkflow {
    name: String,
    branches: Vec<ConditionalBranch>,
    config: ConditionalWorkflowConfig,
    state_manager: StateManager,
    step_executor: StepExecutor,
    error_handler: ErrorHandler,
    condition_evaluator: ConditionEvaluator,
    error_strategy: ErrorStrategy,
    /// Optional workflow executor for hook integration
    workflow_executor: Option<Arc<WorkflowExecutor>>,
    /// Workflow metadata
    metadata: ComponentMetadata,
}

impl ConditionalWorkflow {
    /// Create a new conditional workflow
    pub fn new(name: String, workflow_config: WorkflowConfig) -> Self {
        let config = ConditionalWorkflowConfig::default();
        let error_strategy = workflow_config.default_error_strategy.clone();
        let error_handler = ErrorHandler::new(error_strategy.clone());
        let state_manager = StateManager::new(workflow_config.clone());
        let step_executor = StepExecutor::new(workflow_config);
        let condition_evaluator = ConditionEvaluator::new(Duration::from_millis(
            config.condition_evaluation_timeout_ms,
        ));

        let metadata = ComponentMetadata::new(name.clone(), "Conditional workflow".to_string());

        Self {
            name,
            branches: Vec::new(),
            config,
            state_manager,
            step_executor,
            error_handler,
            condition_evaluator,
            error_strategy,
            workflow_executor: None,
            metadata,
        }
    }

    /// Create with hook integration
    pub fn new_with_hooks(
        name: String,
        workflow_config: WorkflowConfig,
        workflow_executor: Arc<WorkflowExecutor>,
    ) -> Self {
        let config = ConditionalWorkflowConfig::default();
        let error_strategy = workflow_config.default_error_strategy.clone();
        let error_handler = ErrorHandler::new(error_strategy.clone());
        let state_manager =
            StateManager::new_with_hooks(workflow_config.clone(), workflow_executor.clone());
        let step_executor =
            StepExecutor::new_with_hooks(workflow_config, workflow_executor.clone());
        let condition_evaluator = ConditionEvaluator::new(Duration::from_millis(
            config.condition_evaluation_timeout_ms,
        ));

        let metadata =
            ComponentMetadata::new(name.clone(), "Conditional workflow with hooks".to_string());

        Self {
            name,
            branches: Vec::new(),
            config,
            state_manager,
            step_executor,
            error_handler,
            condition_evaluator,
            error_strategy,
            workflow_executor: Some(workflow_executor),
            metadata,
        }
    }

    /// Create a new conditional workflow with builder pattern
    pub fn builder(name: String) -> ConditionalWorkflowBuilder {
        ConditionalWorkflowBuilder::new(name)
    }

    /// Add a branch to the workflow
    pub fn add_branch(&mut self, branch: ConditionalBranch) {
        self.branches.push(branch);
    }

    /// Add multiple branches to the workflow
    pub fn add_branches(&mut self, branches: Vec<ConditionalBranch>) {
        self.branches.extend(branches);
    }

    /// Get workflow name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get number of branches
    pub fn branch_count(&self) -> usize {
        self.branches.len()
    }

    /// Execute the workflow
    pub async fn execute(&self) -> Result<ConditionalWorkflowResult> {
        let start_time = Instant::now();
        info!("Starting conditional workflow: {}", self.name);

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
                "conditional".to_string(),
                WorkflowExecutionPhase::WorkflowStart,
            );
            let _ = workflow_executor.execute_workflow_hooks(hook_ctx).await;
        }

        // Start execution tracking
        self.state_manager.start_execution().await?;

        let mut executed_branches = Vec::new();
        let mut matched_branches = Vec::new();

        // Create condition evaluation context
        let shared_data = self.state_manager.get_all_shared_data().await?;
        let execution_history = self.state_manager.get_execution_history().await?;

        let step_outputs: HashMap<ComponentId, serde_json::Value> = execution_history
            .iter()
            .filter(|r| r.success)
            .map(|r| (r.step_id, serde_json::json!(r.output)))
            .collect();

        let step_results: HashMap<ComponentId, StepResult> = execution_history
            .into_iter()
            .map(|r| (r.step_id, r))
            .collect();

        let context = ConditionEvaluationContext::new(ComponentId::new())
            .with_shared_data(shared_data)
            .with_step_outputs(step_outputs)
            .with_step_results(step_results);

        // Evaluate conditions and execute matching branches
        for (branches_evaluated, branch) in self.branches.iter().enumerate() {
            if branches_evaluated >= self.config.max_branches_to_evaluate {
                warn!(
                    "Maximum branch evaluation limit reached: {}",
                    self.config.max_branches_to_evaluate
                );
                break;
            }

            // Check for execution timeout
            if self.state_manager.check_execution_timeout().await? {
                error!("Workflow '{}' exceeded maximum execution time", self.name);
                self.state_manager.complete_execution(false).await?;
                return Ok(ConditionalWorkflowResult::timeout(
                    self.name.clone(),
                    executed_branches,
                    start_time.elapsed(),
                ));
            }

            debug!("Evaluating condition for branch: {}", branch.name);

            // Execute condition evaluation hooks
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
                    "conditional".to_string(),
                    WorkflowExecutionPhase::ConditionEvaluation,
                );
                hook_ctx = hook_ctx.with_pattern_context(
                    "branch_name".to_string(),
                    serde_json::Value::String(branch.name.clone()),
                );
                hook_ctx = hook_ctx.with_pattern_context(
                    "branch_index".to_string(),
                    serde_json::Value::Number(branches_evaluated.into()),
                );
                let _ = workflow_executor.execute_workflow_hooks(hook_ctx).await;
            }

            // Evaluate the condition
            let condition_result = self
                .condition_evaluator
                .evaluate(&branch.condition, &context)
                .await?;

            if condition_result.is_true {
                matched_branches.push(branch.clone());
                debug!("Condition matched for branch: {}", branch.name);

                // Execute branch selection hooks
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
                        "conditional".to_string(),
                        WorkflowExecutionPhase::BranchSelection,
                    );
                    hook_ctx = hook_ctx.with_pattern_context(
                        "selected_branch".to_string(),
                        serde_json::Value::String(branch.name.clone()),
                    );
                    hook_ctx = hook_ctx.with_pattern_context(
                        "condition_result".to_string(),
                        serde_json::json!({
                            "is_true": condition_result.is_true,
                            "error": condition_result.error,
                            "description": condition_result.description
                        }),
                    );
                    let _ = workflow_executor.execute_workflow_hooks(hook_ctx).await;
                }

                // Execute the branch
                let branch_result = self.execute_branch(branch, &context).await?;
                executed_branches.push(branch_result);

                // If short-circuit evaluation is enabled and we don't execute all matching branches
                if self.config.short_circuit_evaluation && !self.config.execute_all_matching {
                    debug!(
                        "Short-circuiting after first matching branch: {}",
                        branch.name
                    );
                    break;
                }
            } else {
                debug!(
                    "Condition not matched for branch: {} - {}",
                    branch.name, condition_result.description
                );
            }
        }

        // Execute default branch if no conditions matched and configured to do so
        if executed_branches.is_empty() && self.config.execute_default_on_no_match {
            if let Some(default_branch) = self.branches.iter().find(|b| b.is_default) {
                info!(
                    "No conditions matched, executing default branch: {}",
                    default_branch.name
                );
                let branch_result = self.execute_branch(default_branch, &context).await?;
                executed_branches.push(branch_result);
            } else {
                warn!("No conditions matched and no default branch available");
            }
        }

        let duration = start_time.elapsed();
        let success =
            !executed_branches.is_empty() && executed_branches.iter().all(|br| br.success);

        self.state_manager.complete_execution(success).await?;

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
                "conditional".to_string(),
                WorkflowExecutionPhase::WorkflowComplete,
            );
            let _ = workflow_executor.execute_workflow_hooks(hook_ctx).await;
        }

        if success {
            info!(
                "Conditional workflow '{}' completed successfully in {:?}",
                self.name, duration
            );
        } else {
            warn!(
                "Conditional workflow '{}' completed with failures in {:?}",
                self.name, duration
            );
        }

        Ok(ConditionalWorkflowResult {
            workflow_name: self.name.clone(),
            success,
            executed_branches,
            matched_branches: matched_branches.len(),
            total_branches: self.branches.len(),
            duration,
            error_message: if success {
                None
            } else {
                Some("One or more branch executions failed".to_string())
            },
        })
    }

    /// Execute a single branch
    async fn execute_branch(
        &self,
        branch: &ConditionalBranch,
        _context: &ConditionEvaluationContext,
    ) -> Result<BranchExecutionResult> {
        let start_time = Instant::now();
        info!("Executing branch: {}", branch.name);

        let mut step_results = Vec::new();

        for step in &branch.steps {
            debug!("Executing step in branch '{}': {}", branch.name, step.name);

            // Create execution context
            let shared_data = self.state_manager.get_all_shared_data().await?;
            let mut workflow_state = crate::types::WorkflowState::new();
            workflow_state.shared_data = shared_data;
            workflow_state.current_step = step_results.len();
            let execution_context = StepExecutionContext::new(workflow_state, None);

            // Execute step with retry logic (with workflow metadata if hooks are enabled)
            let step_result = if self.workflow_executor.is_some() {
                self.step_executor
                    .execute_step_with_retry_and_metadata(
                        step,
                        execution_context,
                        &self.error_strategy,
                        Some(self.metadata.clone()),
                        Some("conditional".to_string()),
                    )
                    .await?
            } else {
                self.step_executor
                    .execute_step_with_retry(step, execution_context, &self.error_strategy)
                    .await?
            };

            // Record the result
            self.state_manager
                .record_step_result(step_result.clone())
                .await?;
            step_results.push(step_result.clone());

            if !step_result.success {
                // Handle the failure based on error strategy
                let error_action = self
                    .error_handler
                    .handle_step_failure(&step_result, Some(&self.error_strategy))
                    .await?;

                match error_action {
                    ErrorAction::StopWorkflow => {
                        warn!("Stopping branch '{}' due to step failure", branch.name);
                        break;
                    }
                    ErrorAction::ContinueToNext => {
                        warn!(
                            "Continuing to next step after failure in step: {}",
                            step.name
                        );
                        continue;
                    }
                    ErrorAction::RetryStep => {
                        // This is handled by execute_step_with_retry, so if we're here,
                        // all retries have been exhausted
                        if matches!(self.error_strategy, ErrorStrategy::Continue) {
                            warn!("All retries exhausted for step {}, continuing", step.name);
                            continue;
                        } else {
                            warn!(
                                "All retries exhausted for step {}, stopping branch",
                                step.name
                            );
                            break;
                        }
                    }
                }
            }

            self.state_manager.advance_step().await?;
        }

        let duration = start_time.elapsed();
        let condition_result =
            ConditionResult::success_true(format!("Branch '{}' condition evaluation", branch.name));

        Ok(BranchExecutionResult::success(
            branch.id,
            branch.name.clone(),
            condition_result,
            step_results,
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
        warn!("Cancelling conditional workflow: {}", self.name);
        self.state_manager.cancel_execution().await
    }

    /// Reset the workflow to initial state
    pub async fn reset(&self) -> Result<()> {
        debug!("Resetting conditional workflow: {}", self.name);
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

/// Builder for creating conditional workflows
pub struct ConditionalWorkflowBuilder {
    name: String,
    workflow_config: WorkflowConfig,
    conditional_config: ConditionalWorkflowConfig,
    branches: Vec<ConditionalBranch>,
    error_strategy: Option<ErrorStrategy>,
    workflow_executor: Option<Arc<WorkflowExecutor>>,
}

impl ConditionalWorkflowBuilder {
    /// Create a new builder
    pub fn new(name: String) -> Self {
        Self {
            name,
            workflow_config: WorkflowConfig::default(),
            conditional_config: ConditionalWorkflowConfig::default(),
            branches: Vec::new(),
            error_strategy: None,
            workflow_executor: None,
        }
    }

    /// Set the workflow configuration
    pub fn with_workflow_config(mut self, config: WorkflowConfig) -> Self {
        self.workflow_config = config;
        self
    }

    /// Set the conditional configuration
    pub fn with_conditional_config(mut self, config: ConditionalWorkflowConfig) -> Self {
        self.conditional_config = config;
        self
    }

    /// Set the error strategy
    pub fn with_error_strategy(mut self, strategy: ErrorStrategy) -> Self {
        self.error_strategy = Some(strategy);
        self
    }

    /// Add a branch to the workflow
    pub fn add_branch(mut self, branch: ConditionalBranch) -> Self {
        self.branches.push(branch);
        self
    }

    /// Add multiple branches to the workflow
    pub fn add_branches(mut self, branches: Vec<ConditionalBranch>) -> Self {
        self.branches.extend(branches);
        self
    }

    /// Enable hook integration with a WorkflowExecutor
    pub fn with_hooks(mut self, workflow_executor: Arc<WorkflowExecutor>) -> Self {
        self.workflow_executor = Some(workflow_executor);
        self
    }

    /// Build the conditional workflow
    pub fn build(mut self) -> ConditionalWorkflow {
        // Apply error strategy if provided
        if let Some(strategy) = self.error_strategy {
            self.workflow_config.default_error_strategy = strategy;
        }

        let mut workflow = if let Some(workflow_executor) = self.workflow_executor {
            ConditionalWorkflow::new_with_hooks(self.name, self.workflow_config, workflow_executor)
        } else {
            ConditionalWorkflow::new(self.name, self.workflow_config)
        };
        workflow.config = self.conditional_config;
        workflow.add_branches(self.branches);
        workflow
    }
}

/// Result of conditional workflow execution
#[derive(Debug, Clone)]
pub struct ConditionalWorkflowResult {
    pub workflow_name: String,
    pub success: bool,
    pub executed_branches: Vec<BranchExecutionResult>,
    pub matched_branches: usize,
    pub total_branches: usize,
    pub duration: Duration,
    pub error_message: Option<String>,
}

impl ConditionalWorkflowResult {
    /// Create a timeout result
    pub fn timeout(
        workflow_name: String,
        executed_branches: Vec<BranchExecutionResult>,
        duration: Duration,
    ) -> Self {
        Self {
            workflow_name,
            success: false,
            executed_branches,
            matched_branches: 0,
            total_branches: 0,
            duration,
            error_message: Some("Workflow execution timed out".to_string()),
        }
    }

    /// Get total number of steps executed
    pub fn total_steps(&self) -> usize {
        self.executed_branches
            .iter()
            .map(|br| br.step_results.len())
            .sum()
    }

    /// Get successful steps count
    pub fn successful_steps(&self) -> usize {
        self.executed_branches
            .iter()
            .flat_map(|br| &br.step_results)
            .filter(|r| r.success)
            .count()
    }

    /// Get failed steps count
    pub fn failed_steps(&self) -> usize {
        self.total_steps() - self.successful_steps()
    }

    /// Get success rate as percentage
    pub fn success_rate(&self) -> f64 {
        if self.total_steps() == 0 {
            0.0
        } else {
            (self.successful_steps() as f64 / self.total_steps() as f64) * 100.0
        }
    }

    /// Generate a human-readable report
    pub fn generate_report(&self) -> String {
        format!(
            "Conditional Workflow '{}' Report:\n\
            - Success: {}\n\
            - Duration: {:?}\n\
            - Branches: {} matched / {} total\n\
            - Executed Branches: {}\n\
            - Total Steps: {}\n\
            - Successful Steps: {}\n\
            - Failed Steps: {}\n\
            - Success Rate: {:.1}%\n\
            - Error: {}",
            self.workflow_name,
            if self.success { "✓" } else { "✗" },
            self.duration,
            self.matched_branches,
            self.total_branches,
            self.executed_branches.len(),
            self.total_steps(),
            self.successful_steps(),
            self.failed_steps(),
            self.success_rate(),
            self.error_message.as_deref().unwrap_or("None")
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::StepType;

    #[cfg_attr(test_category = "unit")]
    #[tokio::test]
    async fn test_conditional_workflow_creation() {
        let workflow =
            ConditionalWorkflow::new("test_workflow".to_string(), WorkflowConfig::default());
        assert_eq!(workflow.name(), "test_workflow");
        assert_eq!(workflow.branch_count(), 0);
    }

    #[cfg_attr(test_category = "unit")]
    #[tokio::test]
    async fn test_conditional_branch_creation() {
        let condition = Condition::Always;
        let step = WorkflowStep::new(
            "test_step".to_string(),
            StepType::Tool {
                tool_name: "calculator".to_string(),
                parameters: serde_json::json!({"expression": "2 + 2"}),
            },
        );

        let branch = ConditionalBranch::new("test_branch".to_string(), condition).with_step(step);

        assert_eq!(branch.name, "test_branch");
        assert_eq!(branch.steps.len(), 1);
        assert!(!branch.is_default);
    }

    #[cfg_attr(test_category = "unit")]
    #[tokio::test]
    async fn test_conditional_workflow_builder() {
        let condition = Condition::Always;
        let step = WorkflowStep::new(
            "test_step".to_string(),
            StepType::Tool {
                tool_name: "calculator".to_string(),
                parameters: serde_json::json!({"expression": "2 + 2"}),
            },
        );

        let branch = ConditionalBranch::new("test_branch".to_string(), condition).with_step(step);

        let workflow = ConditionalWorkflow::builder("test_workflow".to_string())
            .add_branch(branch)
            .with_error_strategy(ErrorStrategy::Continue)
            .build();

        assert_eq!(workflow.name(), "test_workflow");
        assert_eq!(workflow.branch_count(), 1);
    }

    #[cfg_attr(test_category = "unit")]
    #[tokio::test]
    async fn test_conditional_workflow_execution_always_true() {
        let condition = Condition::Always;
        let step = WorkflowStep::new(
            "test_step".to_string(),
            StepType::Tool {
                tool_name: "calculator".to_string(),
                parameters: serde_json::json!({"expression": "2 + 2"}),
            },
        );

        let branch = ConditionalBranch::new("always_branch".to_string(), condition).with_step(step);

        let workflow = ConditionalWorkflow::builder("test_workflow".to_string())
            .add_branch(branch)
            .build();

        let result = workflow.execute().await.unwrap();
        assert!(result.success);
        assert_eq!(result.executed_branches.len(), 1);
        assert_eq!(result.matched_branches, 1);
        assert_eq!(result.total_steps(), 1);
        assert_eq!(result.successful_steps(), 1);
    }

    #[cfg_attr(test_category = "unit")]
    #[tokio::test]
    async fn test_conditional_workflow_execution_never_condition() {
        let condition = Condition::Never;
        let step = WorkflowStep::new(
            "test_step".to_string(),
            StepType::Tool {
                tool_name: "calculator".to_string(),
                parameters: serde_json::json!({"expression": "2 + 2"}),
            },
        );

        let branch = ConditionalBranch::new("never_branch".to_string(), condition).with_step(step);

        let workflow = ConditionalWorkflow::builder("test_workflow".to_string())
            .add_branch(branch)
            .build();

        let result = workflow.execute().await.unwrap();
        // Should fail because no branches executed and no default branch
        assert!(!result.success);
        assert_eq!(result.executed_branches.len(), 0);
        assert_eq!(result.matched_branches, 0);
        assert_eq!(result.total_steps(), 0);
    }

    #[cfg_attr(test_category = "unit")]
    #[tokio::test]
    async fn test_conditional_workflow_default_branch() {
        let condition = Condition::Never;
        let step1 = WorkflowStep::new(
            "test_step1".to_string(),
            StepType::Tool {
                tool_name: "calculator".to_string(),
                parameters: serde_json::json!({"expression": "2 + 2"}),
            },
        );

        let step2 = WorkflowStep::new(
            "test_step2".to_string(),
            StepType::Tool {
                tool_name: "json_processor".to_string(),
                parameters: serde_json::json!({"input": {"data": "test"}}),
            },
        );

        let never_branch =
            ConditionalBranch::new("never_branch".to_string(), condition).with_step(step1);

        let default_branch =
            ConditionalBranch::default("default_branch".to_string()).with_step(step2);

        let workflow = ConditionalWorkflow::builder("test_workflow".to_string())
            .add_branch(never_branch)
            .add_branch(default_branch)
            .build();

        let result = workflow.execute().await.unwrap();
        assert!(result.success);
        assert_eq!(result.executed_branches.len(), 1);
        assert_eq!(result.executed_branches[0].branch_name, "default_branch");
        assert_eq!(result.total_steps(), 1);
        assert_eq!(result.successful_steps(), 1);
    }

    #[cfg_attr(test_category = "unit")]
    #[tokio::test]
    async fn test_conditional_workflow_shared_data_condition() {
        // Set up shared data
        let workflow =
            ConditionalWorkflow::new("test_workflow".to_string(), WorkflowConfig::default());
        workflow
            .set_shared_data("test_key".to_string(), serde_json::json!("expected_value"))
            .await
            .unwrap();

        let condition = Condition::shared_data_equals(
            "test_key".to_string(),
            serde_json::json!("expected_value"),
        );

        let step = WorkflowStep::new(
            "test_step".to_string(),
            StepType::Tool {
                tool_name: "calculator".to_string(),
                parameters: serde_json::json!({"expression": "2 + 2"}),
            },
        );

        let branch = ConditionalBranch::new("data_branch".to_string(), condition).with_step(step);

        // Rebuild workflow with the branch
        let workflow = ConditionalWorkflow::builder("test_workflow".to_string())
            .add_branch(branch)
            .build();

        // Set the shared data again in the new workflow
        workflow
            .set_shared_data("test_key".to_string(), serde_json::json!("expected_value"))
            .await
            .unwrap();

        let result = workflow.execute().await.unwrap();
        assert!(result.success);
        assert_eq!(result.executed_branches.len(), 1);
        assert_eq!(result.total_steps(), 1);
    }
}
