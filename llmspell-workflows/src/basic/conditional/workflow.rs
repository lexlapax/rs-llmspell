//! ABOUTME: Basic conditional workflow implementation
//! ABOUTME: Executes workflow branches based on condition evaluation with memory-based state management

use super::super::error_handling::{BasicErrorHandler, ErrorAction};
use super::super::state::BasicStateManager;
use super::super::step_executor::BasicStepExecutor;
use super::super::traits::{
    BasicErrorStrategy, BasicStepResult, BasicWorkflow, BasicWorkflowStatus, BasicWorkflowStep,
};
use super::super::types::{BasicWorkflowConfig, StepExecutionContext};
use super::conditions::BasicConditionEvaluator;
use super::types::{
    BranchExecutionResult, ConditionEvaluationContext, ConditionalBranch, ConditionalWorkflowConfig,
};
use async_trait::async_trait;
use llmspell_core::{ComponentId, LLMSpellError, Result};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tracing::{debug, error, info, warn};

/// Basic conditional workflow that executes branches based on condition evaluation
///
/// This workflow pattern evaluates conditions and executes the corresponding branch steps.
/// It supports multiple branch evaluation modes, default branches, and comprehensive error handling.
/// It uses memory-based state management and integrates with the existing step execution infrastructure.
///
/// # Features
/// - Condition-based branch selection
/// - Multiple evaluation modes (first match, all matches)
/// - Default branch support
/// - Memory-based state management
/// - Configurable timeout and retry handling
/// - Comprehensive logging and monitoring
/// - Integration with BasicStepExecutor
///
/// # Examples
///
/// ```rust
/// use llmspell_workflows::{
///     BasicWorkflow,
///     basic::{
///         BasicConditionalWorkflow, ConditionalBranch, BasicCondition,
///         ConditionalWorkflowConfig, BasicWorkflowConfig
///     }
/// };
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let workflow_config = BasicWorkflowConfig::default();
///     let conditional_config = ConditionalWorkflowConfig::default();
///     let mut workflow = BasicConditionalWorkflow::new(
///         "data_router".to_string(),
///         workflow_config,
///         conditional_config
///     );
///     
///     // Add conditional branch
///     let condition = BasicCondition::shared_data_equals(
///         "data_type".to_string(),
///         serde_json::json!("csv")
///     );
///     let branch = ConditionalBranch::new("process_csv".to_string(), condition);
///     workflow.add_branch(branch).await?;
///     
///     // Set shared data and execute
///     workflow.set_shared_data("data_type".to_string(), serde_json::json!("csv")).await?;
///     let results = workflow.execute().await?;
///     println!("Conditional workflow completed");
///     
///     Ok(())
/// }
/// ```
pub struct BasicConditionalWorkflow {
    name: String,
    branches: Vec<ConditionalBranch>,
    config: ConditionalWorkflowConfig,
    state_manager: BasicStateManager,
    step_executor: BasicStepExecutor,
    error_handler: BasicErrorHandler,
    condition_evaluator: BasicConditionEvaluator,
    error_strategy: BasicErrorStrategy,
}

impl BasicConditionalWorkflow {
    /// Create a new basic conditional workflow
    pub fn new(
        name: String,
        workflow_config: BasicWorkflowConfig,
        conditional_config: ConditionalWorkflowConfig,
    ) -> Self {
        let error_strategy = if workflow_config.continue_on_error {
            BasicErrorStrategy::Continue
        } else {
            BasicErrorStrategy::FailFast
        };

        let state_manager = BasicStateManager::new(workflow_config.clone());
        let step_executor = BasicStepExecutor::new(workflow_config.clone());
        let error_handler = BasicErrorHandler::new(error_strategy.clone());
        let condition_evaluator = BasicConditionEvaluator::new(Duration::from_millis(
            conditional_config.condition_evaluation_timeout_ms,
        ));

        Self {
            name,
            branches: Vec::new(),
            config: conditional_config,
            state_manager,
            step_executor,
            error_handler,
            condition_evaluator,
            error_strategy,
        }
    }

    /// Create a new workflow with custom error strategy
    pub fn with_error_strategy(
        name: String,
        workflow_config: BasicWorkflowConfig,
        conditional_config: ConditionalWorkflowConfig,
        error_strategy: BasicErrorStrategy,
    ) -> Self {
        let state_manager = BasicStateManager::new(workflow_config.clone());
        let step_executor = BasicStepExecutor::new(workflow_config.clone());
        let error_handler = BasicErrorHandler::new(error_strategy.clone());
        let condition_evaluator = BasicConditionEvaluator::new(Duration::from_millis(
            conditional_config.condition_evaluation_timeout_ms,
        ));

        Self {
            name,
            branches: Vec::new(),
            config: conditional_config,
            state_manager,
            step_executor,
            error_handler,
            condition_evaluator,
            error_strategy,
        }
    }

    /// Add a conditional branch to the workflow
    pub async fn add_branch(&mut self, branch: ConditionalBranch) -> Result<()> {
        debug!(
            "Adding branch '{}' to conditional workflow '{}'",
            branch.name, self.name
        );

        // Validate branch has at least one step
        if branch.steps.is_empty() {
            warn!("Branch '{}' has no steps", branch.name);
        }

        self.branches.push(branch);
        Ok(())
    }

    /// Remove a branch by ID
    pub async fn remove_branch(&mut self, branch_id: ComponentId) -> Result<()> {
        let initial_len = self.branches.len();
        self.branches.retain(|branch| branch.id != branch_id);

        if self.branches.len() < initial_len {
            debug!(
                "Removed branch {:?} from conditional workflow '{}'",
                branch_id, self.name
            );
            Ok(())
        } else {
            Err(LLMSpellError::Workflow {
                message: format!("Branch with id {:?} not found in workflow", branch_id),
                step: None,
                source: None,
            })
        }
    }

    /// Get all branches in the workflow
    pub async fn get_branches(&self) -> Result<Vec<ConditionalBranch>> {
        Ok(self.branches.clone())
    }

    /// Get the number of branches in the workflow
    pub fn branch_count(&self) -> usize {
        self.branches.len()
    }

    /// Check if workflow has any branches
    pub fn is_empty(&self) -> bool {
        self.branches.is_empty()
    }

    /// Execute workflow with conditional branch selection
    async fn execute_with_tracking(&mut self) -> Result<Vec<BasicStepResult>> {
        info!("Starting execution of conditional workflow '{}'", self.name);
        let execution_start = Instant::now();

        // Start execution tracking
        self.state_manager.start_execution().await?;

        // Build condition evaluation context
        let workflow_state = self.state_manager.get_state_snapshot().await?;
        let execution_history = self.state_manager.get_execution_history().await?;

        // Convert execution history to step results map
        let mut step_results = HashMap::new();
        for result in execution_history {
            step_results.insert(result.step_id, result);
        }

        let context = ConditionEvaluationContext::new(ComponentId::new())
            .with_shared_data(workflow_state.shared_data.clone())
            .with_step_results(step_results);

        // Evaluate branches and select which ones to execute
        let branches_to_execute = self.select_branches_to_execute(&context).await?;

        if branches_to_execute.is_empty() {
            warn!("No branches matched conditions in workflow '{}'", self.name);

            // Execute default branch if configured
            if self.config.execute_default_on_no_match {
                if let Some(default_branch) = self.find_default_branch() {
                    info!("Executing default branch: '{}'", default_branch.name);
                    let branch_result = self.execute_branch(default_branch, context).await?;
                    let execution_duration = execution_start.elapsed();

                    info!(
                        "Conditional workflow '{}' completed with default branch in {:?}",
                        self.name, execution_duration
                    );

                    self.state_manager
                        .complete_execution(branch_result.success)
                        .await?;
                    return Ok(branch_result.step_results);
                }
            }

            // No branches to execute and no default branch
            self.state_manager.complete_execution(true).await?;
            return Ok(Vec::new());
        }

        // Execute selected branches
        let mut all_results = Vec::new();
        let mut all_branches_succeeded = true;
        let branches_count = branches_to_execute.len();

        for branch in &branches_to_execute {
            info!("Executing branch: '{}'", branch.name);

            // Check for execution timeout before each branch
            if self.state_manager.check_execution_timeout().await? {
                error!(
                    "Workflow execution timed out before branch '{}'",
                    branch.name
                );
                self.state_manager.complete_execution(false).await?;
                return Err(LLMSpellError::Timeout {
                    message: "Conditional workflow execution exceeded maximum time limit"
                        .to_string(),
                    duration_ms: None,
                });
            }

            let branch_result = self.execute_branch(branch, context.clone()).await?;

            if !branch_result.success {
                all_branches_succeeded = false;

                // Handle branch failure based on error strategy
                let action = self
                    .error_handler
                    .handle_branch_failure(&branch_result, &self.error_strategy)
                    .await?;

                match action {
                    ErrorAction::StopWorkflow => {
                        error!(
                            "Stopping conditional workflow due to branch failure: '{}'",
                            branch_result.branch_name
                        );
                        all_results.extend(branch_result.step_results);
                        self.state_manager.complete_execution(false).await?;
                        return Ok(all_results);
                    }
                    ErrorAction::ContinueToNext => {
                        warn!(
                            "Continuing to next branch despite failure in '{}'",
                            branch_result.branch_name
                        );
                    }
                    ErrorAction::RetryStep => {
                        // For branches, we don't retry the whole branch - individual steps handle retries
                        warn!(
                            "Branch '{}' failed, continuing to next branch",
                            branch_result.branch_name
                        );
                    }
                }
            } else {
                info!(
                    "Branch '{}' completed successfully in {:?}",
                    branch_result.branch_name, branch_result.duration
                );
            }

            all_results.extend(branch_result.step_results);

            // If not executing all matching branches, stop after first successful execution
            if !self.config.execute_all_matching && branch_result.success {
                break;
            }
        }

        let execution_duration = execution_start.elapsed();
        let successful_steps = all_results.iter().filter(|r| r.success).count();

        info!(
            "Conditional workflow '{}' completed in {:?}: {}/{} steps successful across {} branches",
            self.name,
            execution_duration,
            successful_steps,
            all_results.len(),
            branches_count
        );

        // Complete execution tracking
        self.state_manager
            .complete_execution(all_branches_succeeded)
            .await?;

        Ok(all_results)
    }

    /// Select branches to execute based on condition evaluation
    async fn select_branches_to_execute(
        &self,
        context: &ConditionEvaluationContext,
    ) -> Result<Vec<&ConditionalBranch>> {
        let mut selected_branches = Vec::new();
        let mut evaluated_count = 0;

        for branch in &self.branches {
            // Check evaluation limits
            if evaluated_count >= self.config.max_branches_to_evaluate {
                warn!(
                    "Reached maximum branch evaluation limit: {}",
                    self.config.max_branches_to_evaluate
                );
                break;
            }

            // Skip default branches in normal evaluation
            if branch.is_default {
                continue;
            }

            debug!("Evaluating condition for branch: '{}'", branch.name);

            let condition_result = self
                .condition_evaluator
                .evaluate(&branch.condition, context)
                .await?;

            evaluated_count += 1;

            if condition_result.is_error() {
                warn!(
                    "Condition evaluation failed for branch '{}': {}",
                    branch.name,
                    condition_result.error.unwrap_or_default()
                );
                continue;
            }

            if condition_result.is_true {
                info!(
                    "Branch '{}' condition matched: {}",
                    branch.name, condition_result.description
                );
                selected_branches.push(branch);

                // Short-circuit if configured
                if self.config.short_circuit_evaluation && !self.config.execute_all_matching {
                    break;
                }
            } else {
                debug!(
                    "Branch '{}' condition not matched: {}",
                    branch.name, condition_result.description
                );
            }
        }

        Ok(selected_branches)
    }

    /// Execute a single branch
    async fn execute_branch(
        &self,
        branch: &ConditionalBranch,
        context: ConditionEvaluationContext,
    ) -> Result<BranchExecutionResult> {
        let branch_start = Instant::now();

        // Re-evaluate condition for recording purposes
        let condition_result = self
            .condition_evaluator
            .evaluate(&branch.condition, &context)
            .await?;

        let mut step_results = Vec::new();

        // Execute all steps in the branch
        for step in &branch.steps {
            // Create step execution context
            let workflow_state = self.state_manager.get_state_snapshot().await?;
            let step_context = StepExecutionContext::new(workflow_state, step.timeout);

            // Execute step with retry logic
            let step_result = self
                .step_executor
                .execute_step_with_retry(step, step_context, &self.error_strategy)
                .await?;

            // Record the result
            self.state_manager
                .record_step_result(step_result.clone())
                .await?;
            step_results.push(step_result.clone());

            if step_result.success {
                debug!(
                    "Step '{}' in branch '{}' completed successfully",
                    step.name, branch.name
                );
                self.state_manager.advance_step().await?;
            } else {
                warn!(
                    "Step '{}' in branch '{}' failed: {}",
                    step.name,
                    branch.name,
                    step_result
                        .error
                        .as_ref()
                        .unwrap_or(&"Unknown error".to_string())
                );

                // Handle step failure within branch
                let action = self
                    .error_handler
                    .handle_step_failure(&step_result, Some(&self.error_strategy))
                    .await?;

                match action {
                    ErrorAction::StopWorkflow => {
                        // Stop branch execution
                        break;
                    }
                    ErrorAction::ContinueToNext => {
                        // Continue to next step in branch
                        self.state_manager.advance_step().await?;
                    }
                    ErrorAction::RetryStep => {
                        // Retry is handled by execute_step_with_retry
                        self.state_manager.advance_step().await?;
                    }
                }
            }
        }

        let branch_duration = branch_start.elapsed();

        Ok(BranchExecutionResult::success(
            branch.id,
            branch.name.clone(),
            condition_result,
            step_results,
            branch_duration,
        ))
    }

    /// Find the default branch if one exists
    fn find_default_branch(&self) -> Option<&ConditionalBranch> {
        self.branches.iter().find(|branch| branch.is_default)
    }

    /// Get execution statistics
    pub async fn get_stats(&self) -> Result<super::super::state::ExecutionStats> {
        self.state_manager.get_execution_stats().await
    }

    /// Get current workflow state snapshot
    pub async fn get_state_snapshot(&self) -> Result<super::super::types::BasicWorkflowState> {
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
impl BasicWorkflow for BasicConditionalWorkflow {
    fn name(&self) -> &str {
        &self.name
    }

    async fn status(&self) -> Result<BasicWorkflowStatus> {
        self.state_manager.get_status().await
    }

    async fn add_step(&mut self, step: BasicWorkflowStep) -> Result<()> {
        // For conditional workflows, steps must be added to specific branches
        // This method adds to default branch or creates one if needed
        debug!(
            "Adding step '{}' to default branch in conditional workflow '{}'",
            step.name, self.name
        );

        // Find or create default branch
        if let Some(default_branch) = self.branches.iter_mut().find(|b| b.is_default) {
            default_branch.steps.push(step);
        } else {
            // Create new default branch
            let mut default_branch = ConditionalBranch::default("default".to_string());
            default_branch.steps.push(step);
            self.branches.push(default_branch);
        }

        Ok(())
    }

    async fn remove_step(&mut self, step_id: ComponentId) -> Result<()> {
        let mut removed = false;

        for branch in &mut self.branches {
            let initial_len = branch.steps.len();
            branch.steps.retain(|step| step.id != step_id);

            if branch.steps.len() < initial_len {
                removed = true;
                debug!(
                    "Removed step {:?} from branch '{}' in conditional workflow '{}'",
                    step_id, branch.name, self.name
                );
                break;
            }
        }

        if removed {
            Ok(())
        } else {
            Err(LLMSpellError::Workflow {
                message: format!("Step with id {:?} not found in any branch", step_id),
                step: None,
                source: None,
            })
        }
    }

    async fn get_steps(&self) -> Result<Vec<BasicWorkflowStep>> {
        // Return all steps from all branches
        let mut all_steps = Vec::new();
        for branch in &self.branches {
            all_steps.extend(branch.steps.clone());
        }
        Ok(all_steps)
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
        debug!("Resetting conditional workflow '{}'", self.name);
        self.state_manager.reset().await
    }

    async fn validate(&self) -> Result<()> {
        if self.branches.is_empty() {
            return Err(LLMSpellError::Workflow {
                message: format!("Conditional workflow '{}' has no branches", self.name),
                step: None,
                source: None,
            });
        }

        // Validate each branch
        for (index, branch) in self.branches.iter().enumerate() {
            if branch.name.is_empty() {
                return Err(LLMSpellError::Workflow {
                    message: format!("Branch at index {} has empty name", index),
                    step: Some(format!("branch_{}", index)),
                    source: None,
                });
            }

            // Validate branch has steps (allow empty for default branch)
            if branch.steps.is_empty() && !branch.is_default {
                warn!("Non-default branch '{}' has no steps", branch.name);
            }

            // Validate each step in the branch
            for (step_index, step) in branch.steps.iter().enumerate() {
                if step.name.is_empty() {
                    return Err(LLMSpellError::Workflow {
                        message: format!(
                            "Step at index {} in branch '{}' has empty name",
                            step_index, branch.name
                        ),
                        step: Some(format!("{}_{}", branch.name, step_index)),
                        source: None,
                    });
                }
            }
        }

        debug!(
            "Conditional workflow '{}' validation passed with {} branches",
            self.name,
            self.branches.len()
        );
        Ok(())
    }
}

// Extension trait for BasicErrorHandler to handle branch failures
impl BasicErrorHandler {
    /// Handle branch failure in conditional workflow
    pub async fn handle_branch_failure(
        &self,
        branch_result: &BranchExecutionResult,
        error_strategy: &BasicErrorStrategy,
    ) -> Result<ErrorAction> {
        warn!(
            "Branch '{}' failed with {} step failures",
            branch_result.branch_name,
            branch_result
                .step_results
                .iter()
                .filter(|r| !r.success)
                .count()
        );

        // For branch failures, we typically continue to next branch unless strategy says otherwise
        match error_strategy {
            BasicErrorStrategy::FailFast => Ok(ErrorAction::StopWorkflow),
            BasicErrorStrategy::Continue => Ok(ErrorAction::ContinueToNext),
            BasicErrorStrategy::Retry { .. } => {
                // For branches, we don't retry the whole branch
                Ok(ErrorAction::ContinueToNext)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::basic::conditional::types::BasicCondition;
    use crate::basic::traits::BasicStepType;

    #[tokio::test]
    async fn test_basic_conditional_workflow_creation() {
        let workflow_config = BasicWorkflowConfig::default();
        let conditional_config = ConditionalWorkflowConfig::default();
        let workflow = BasicConditionalWorkflow::new(
            "test_workflow".to_string(),
            workflow_config,
            conditional_config,
        );

        assert_eq!(workflow.name(), "test_workflow");
        assert_eq!(workflow.branch_count(), 0);
        assert!(workflow.is_empty());
        assert_eq!(
            workflow.status().await.unwrap(),
            BasicWorkflowStatus::Pending
        );
    }

    #[tokio::test]
    async fn test_branch_management() {
        let workflow_config = BasicWorkflowConfig::default();
        let conditional_config = ConditionalWorkflowConfig::default();
        let mut workflow = BasicConditionalWorkflow::new(
            "test_workflow".to_string(),
            workflow_config,
            conditional_config,
        );

        // Add branch
        let condition = BasicCondition::Always;
        let branch = ConditionalBranch::new("test_branch".to_string(), condition);
        let branch_id = branch.id;

        workflow.add_branch(branch).await.unwrap();
        assert_eq!(workflow.branch_count(), 1);
        assert!(!workflow.is_empty());

        // Get branches
        let branches = workflow.get_branches().await.unwrap();
        assert_eq!(branches.len(), 1);
        assert_eq!(branches[0].name, "test_branch");

        // Remove branch
        workflow.remove_branch(branch_id).await.unwrap();
        assert_eq!(workflow.branch_count(), 0);
        assert!(workflow.is_empty());
    }

    #[tokio::test]
    async fn test_conditional_workflow_validation() {
        let workflow_config = BasicWorkflowConfig::default();
        let conditional_config = ConditionalWorkflowConfig::default();
        let mut workflow = BasicConditionalWorkflow::new(
            "test_workflow".to_string(),
            workflow_config,
            conditional_config,
        );

        // Empty workflow should fail validation
        let result = workflow.validate().await;
        assert!(result.is_err());

        // Add valid branch
        let condition = BasicCondition::Always;
        let mut branch = ConditionalBranch::new("valid_branch".to_string(), condition);

        let step = BasicWorkflowStep::new(
            "valid_step".to_string(),
            BasicStepType::Tool {
                tool_name: "calculator".to_string(),
                parameters: serde_json::json!({}),
            },
        );
        branch = branch.with_step(step);

        workflow.add_branch(branch).await.unwrap();
        assert!(workflow.validate().await.is_ok());
    }

    #[tokio::test]
    async fn test_conditional_workflow_execution() {
        let workflow_config = BasicWorkflowConfig::default();
        let conditional_config = ConditionalWorkflowConfig::default();
        let mut workflow = BasicConditionalWorkflow::new(
            "test_workflow".to_string(),
            workflow_config,
            conditional_config,
        );

        // Add branch with always-true condition
        let condition = BasicCondition::Always;
        let step = BasicWorkflowStep::new(
            "calculator_step".to_string(),
            BasicStepType::Tool {
                tool_name: "calculator".to_string(),
                parameters: serde_json::json!({"expression": "2 + 2"}),
            },
        );
        let branch = ConditionalBranch::new("always_branch".to_string(), condition).with_step(step);

        workflow.add_branch(branch).await.unwrap();

        // Execute workflow
        let results = workflow.execute().await.unwrap();
        assert_eq!(results.len(), 1);
        assert!(results[0].success);

        // Check status
        assert_eq!(
            workflow.status().await.unwrap(),
            BasicWorkflowStatus::Completed
        );
    }

    #[tokio::test]
    async fn test_conditional_workflow_with_shared_data() {
        let workflow_config = BasicWorkflowConfig::default();
        let conditional_config = ConditionalWorkflowConfig::default();
        let mut workflow = BasicConditionalWorkflow::new(
            "test_workflow".to_string(),
            workflow_config,
            conditional_config,
        );

        // Set shared data
        workflow
            .set_shared_data("test_key".to_string(), serde_json::json!("expected_value"))
            .await
            .unwrap();

        // Add branch with shared data condition
        let condition = BasicCondition::shared_data_equals(
            "test_key".to_string(),
            serde_json::json!("expected_value"),
        );
        let step = BasicWorkflowStep::new(
            "data_processor".to_string(),
            BasicStepType::Tool {
                tool_name: "json_processor".to_string(),
                parameters: serde_json::json!({"input": {"test": "data"}}),
            },
        );
        let branch = ConditionalBranch::new("data_branch".to_string(), condition).with_step(step);

        workflow.add_branch(branch).await.unwrap();

        // Execute workflow
        let results = workflow.execute().await.unwrap();
        assert_eq!(results.len(), 1);
        assert!(results[0].success);
    }

    #[tokio::test]
    async fn test_conditional_workflow_default_branch() {
        let workflow_config = BasicWorkflowConfig::default();
        let conditional_config = ConditionalWorkflowConfig::default();
        let mut workflow = BasicConditionalWorkflow::new(
            "test_workflow".to_string(),
            workflow_config,
            conditional_config,
        );

        // Add branch with never-true condition
        let condition = BasicCondition::Never;
        let branch = ConditionalBranch::new("never_branch".to_string(), condition);
        workflow.add_branch(branch).await.unwrap();

        // Add default branch
        let step = BasicWorkflowStep::new(
            "default_step".to_string(),
            BasicStepType::Tool {
                tool_name: "calculator".to_string(),
                parameters: serde_json::json!({}),
            },
        );
        let default_branch =
            ConditionalBranch::default("default_branch".to_string()).with_step(step);
        workflow.add_branch(default_branch).await.unwrap();

        // Execute workflow - should execute default branch
        let results = workflow.execute().await.unwrap();
        assert_eq!(results.len(), 1);
        assert!(results[0].success);
    }
}
