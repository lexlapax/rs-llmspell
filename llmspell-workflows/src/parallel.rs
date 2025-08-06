// ABOUTME: Parallel workflow implementation for concurrent branch execution
// ABOUTME: Implements fork-join pattern with fixed concurrency limits and fail-fast error handling

use crate::{
    error_handling::{ErrorAction, ErrorHandler},
    hooks::{WorkflowExecutionPhase, WorkflowExecutor, WorkflowHookContext},
    state::StateManager,
    step_executor::StepExecutor,
    traits::{StepResult, WorkflowStep as TraitWorkflowStep},
    types::{StepExecutionContext, WorkflowConfig, WorkflowState},
};
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
use serde::{Deserialize, Serialize};
use std::{
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::{Mutex, RwLock, Semaphore};
use tracing::{debug, error, info, warn};

/// A branch in a parallel workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParallelBranch {
    /// Unique name for this branch
    pub name: String,
    /// Description of what this branch does
    pub description: String,
    /// Steps to execute in this branch
    pub steps: Vec<TraitWorkflowStep>,
    /// Whether this branch is required for workflow success
    pub required: bool,
    /// Maximum execution time for this branch
    pub timeout: Option<Duration>,
}

impl ParallelBranch {
    pub fn new(name: String) -> Self {
        Self {
            name,
            description: String::new(),
            steps: Vec::new(),
            required: true,
            timeout: None,
        }
    }

    pub fn with_description(mut self, description: String) -> Self {
        self.description = description;
        self
    }

    pub fn add_step(mut self, step: TraitWorkflowStep) -> Self {
        self.steps.push(step);
        self
    }

    pub fn optional(mut self) -> Self {
        self.required = false;
        self
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }
}

/// Configuration for parallel workflow execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParallelConfig {
    /// Maximum number of branches to execute concurrently
    pub max_concurrency: usize,
    /// Whether to fail fast on first error
    pub fail_fast: bool,
    /// Timeout for the entire parallel execution
    pub timeout: Option<Duration>,
    /// Whether to continue if optional branches fail
    pub continue_on_optional_failure: bool,
}

impl ParallelConfig {
    /// Create a new builder for ParallelConfig
    pub fn builder() -> ParallelConfigBuilder {
        ParallelConfigBuilder::new()
    }
}

impl Default for ParallelConfig {
    fn default() -> Self {
        Self {
            max_concurrency: 4,
            fail_fast: true,
            timeout: None,
            continue_on_optional_failure: true,
        }
    }
}

/// Builder for ParallelConfig
pub struct ParallelConfigBuilder {
    config: ParallelConfig,
}

impl ParallelConfigBuilder {
    /// Create a new builder with default configuration
    pub fn new() -> Self {
        Self {
            config: ParallelConfig::default(),
        }
    }

    /// Set maximum number of branches to execute concurrently
    pub fn max_concurrency(mut self, concurrency: usize) -> Self {
        self.config.max_concurrency = concurrency;
        self
    }

    /// Set whether to fail fast on first error
    pub fn fail_fast(mut self, enabled: bool) -> Self {
        self.config.fail_fast = enabled;
        self
    }

    /// Set timeout for the entire parallel execution
    pub fn timeout(mut self, timeout: Option<Duration>) -> Self {
        self.config.timeout = timeout;
        self
    }

    /// Set whether to continue if optional branches fail
    pub fn continue_on_optional_failure(mut self, enabled: bool) -> Self {
        self.config.continue_on_optional_failure = enabled;
        self
    }

    /// Build the final ParallelConfig with validation
    pub fn build(self) -> Result<ParallelConfig> {
        if self.config.max_concurrency == 0 {
            return Err(LLMSpellError::Validation {
                message: "max_concurrency must be greater than 0".to_string(),
                field: Some("max_concurrency".to_string()),
            });
        }
        Ok(self.config)
    }
}

impl Default for ParallelConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Result from a parallel branch execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BranchResult {
    /// Branch name
    pub branch_name: String,
    /// Whether the branch succeeded
    pub success: bool,
    /// Results from steps in the branch
    pub step_results: Vec<StepResult>,
    /// Total execution time
    pub duration: Duration,
    /// Error message if failed
    pub error: Option<String>,
    /// Whether this was a required branch
    pub required: bool,
}

impl BranchResult {
    pub fn success(
        branch_name: String,
        step_results: Vec<StepResult>,
        duration: Duration,
        required: bool,
    ) -> Self {
        Self {
            branch_name,
            success: true,
            step_results,
            duration,
            error: None,
            required,
        }
    }

    pub fn failure(
        branch_name: String,
        step_results: Vec<StepResult>,
        duration: Duration,
        error: String,
        required: bool,
    ) -> Self {
        Self {
            branch_name,
            success: false,
            step_results,
            duration,
            error: Some(error),
            required,
        }
    }
}

/// Result of parallel workflow execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParallelWorkflowResult {
    /// Workflow name
    pub workflow_name: String,
    /// Whether the workflow completed successfully
    pub success: bool,
    /// Results from all branches
    pub branch_results: Vec<BranchResult>,
    /// Total execution duration
    pub duration: Duration,
    /// Number of branches that succeeded
    pub successful_branches: usize,
    /// Number of branches that failed
    pub failed_branches: usize,
    /// Whether execution was stopped early due to fail-fast
    pub stopped_early: bool,
    /// Error message if workflow failed
    pub error: Option<String>,
}

impl ParallelWorkflowResult {
    pub fn new(
        workflow_name: String,
        branch_results: Vec<BranchResult>,
        duration: Duration,
        stopped_early: bool,
    ) -> Self {
        let successful_branches = branch_results.iter().filter(|r| r.success).count();
        let failed_branches = branch_results.iter().filter(|r| !r.success).count();

        // Workflow succeeds if all required branches succeed
        let required_failures = branch_results
            .iter()
            .filter(|r| r.required && !r.success)
            .count();

        // If we stopped early with no results, it's a failure
        let success = !branch_results.is_empty() && required_failures == 0;

        let error = if !success {
            Some(format!("{} required branches failed", required_failures))
        } else {
            None
        };

        Self {
            workflow_name,
            success,
            branch_results,
            duration,
            successful_branches,
            failed_branches,
            stopped_early,
            error,
        }
    }

    /// Generate a summary report
    pub fn generate_report(&self) -> String {
        let mut report = format!(
            "Parallel Workflow '{}' Report:\n\
            - Success: {}\n\
            - Duration: {:?}\n\
            - Total Branches: {}\n\
            - Successful: {}\n\
            - Failed: {}\n\
            - Stopped Early: {}\n",
            self.workflow_name,
            if self.success { "✓" } else { "✗" },
            self.duration,
            self.branch_results.len(),
            self.successful_branches,
            self.failed_branches,
            if self.stopped_early { "Yes" } else { "No" }
        );

        if !self.branch_results.is_empty() {
            report.push_str("\nBranch Results:\n");
            for result in &self.branch_results {
                report.push_str(&format!(
                    "  - {} ({}): {} in {:?}\n",
                    result.branch_name,
                    if result.required {
                        "required"
                    } else {
                        "optional"
                    },
                    if result.success { "✓" } else { "✗" },
                    result.duration
                ));
            }
        }

        if let Some(error) = &self.error {
            report.push_str(&format!("\nError: {}\n", error));
        }

        report
    }
}

/// Parallel workflow implementation
pub struct ParallelWorkflow {
    name: String,
    branches: Vec<ParallelBranch>,
    config: ParallelConfig,
    workflow_config: WorkflowConfig,
    state_manager: StateManager,
    step_executor: StepExecutor,
    error_handler: ErrorHandler,
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

impl ParallelWorkflow {
    /// Create a new parallel workflow
    pub fn new(
        name: String,
        branches: Vec<ParallelBranch>,
        config: ParallelConfig,
        workflow_config: WorkflowConfig,
    ) -> Self {
        let error_strategy = workflow_config.default_error_strategy.clone();
        let error_handler = ErrorHandler::new(error_strategy);
        let state_manager = StateManager::new(workflow_config.clone());
        let step_executor = StepExecutor::new(workflow_config.clone());

        let metadata = ComponentMetadata::new(name.clone(), "Parallel workflow".to_string());

        // Create core workflow config from our config
        let core_config = CoreWorkflowConfig {
            max_parallel: Some(config.max_concurrency),
            continue_on_error: !config.fail_fast,
            timeout: config.timeout.or(workflow_config.max_execution_time),
        };

        Self {
            name,
            branches,
            config,
            workflow_config,
            state_manager,
            step_executor,
            error_handler,
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
        branches: Vec<ParallelBranch>,
        config: ParallelConfig,
        workflow_config: WorkflowConfig,
        workflow_executor: Arc<WorkflowExecutor>,
    ) -> Self {
        let error_strategy = workflow_config.default_error_strategy.clone();
        let error_handler = ErrorHandler::new(error_strategy);
        let state_manager =
            StateManager::new_with_hooks(workflow_config.clone(), workflow_executor.clone());
        let step_executor =
            StepExecutor::new_with_hooks(workflow_config.clone(), workflow_executor.clone());

        let metadata =
            ComponentMetadata::new(name.clone(), "Parallel workflow with hooks".to_string());

        // Create core workflow config from our config
        let core_config = CoreWorkflowConfig {
            max_parallel: Some(config.max_concurrency),
            continue_on_error: !config.fail_fast,
            timeout: config.timeout.or(workflow_config.max_execution_time),
        };

        Self {
            name,
            branches,
            config,
            workflow_config,
            state_manager,
            step_executor,
            error_handler,
            workflow_executor: Some(workflow_executor),
            metadata,
            core_config,
            core_steps: Arc::new(RwLock::new(Vec::new())),
            core_results: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Create a new parallel workflow with builder pattern
    pub fn builder(name: String) -> ParallelWorkflowBuilder {
        ParallelWorkflowBuilder::new(name)
    }

    /// Get workflow name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get number of branches
    pub fn branch_count(&self) -> usize {
        self.branches.len()
    }

    /// Execute a single branch
    async fn execute_branch(
        branch: ParallelBranch,
        step_executor: Arc<StepExecutor>,
        state_manager: Arc<Mutex<StateManager>>,
        error_handler: Arc<ErrorHandler>,
        workflow_config: WorkflowConfig,
        workflow_metadata: Option<ComponentMetadata>,
        has_hooks: bool,
    ) -> BranchResult {
        let start_time = Instant::now();
        let branch_name = branch.name.clone();
        let required = branch.required;

        debug!("Starting parallel branch: {}", branch_name);

        let mut step_results = Vec::new();
        let mut branch_success = true;
        let mut branch_error = None;

        for (index, step) in branch.steps.iter().enumerate() {
            debug!(
                "Branch '{}' executing step {} of {}: {}",
                branch_name,
                index + 1,
                branch.steps.len(),
                step.name
            );

            // Create execution context
            let shared_data = state_manager
                .lock()
                .await
                .get_all_shared_data()
                .await
                .unwrap_or_default();
            let mut workflow_state = WorkflowState::new();
            workflow_state.shared_data = shared_data;
            workflow_state.current_step = index;
            let context = StepExecutionContext::new(workflow_state, branch.timeout);

            // Execute step (with workflow metadata if hooks are enabled)
            let step_result = if has_hooks && workflow_metadata.is_some() {
                step_executor
                    .execute_step_with_retry_and_metadata(
                        step,
                        context,
                        &workflow_config.default_error_strategy,
                        workflow_metadata.clone(),
                        Some("parallel".to_string()),
                    )
                    .await
            } else {
                step_executor
                    .execute_step_with_retry(step, context, &workflow_config.default_error_strategy)
                    .await
            };

            match step_result {
                Ok(result) => {
                    if result.success {
                        step_results.push(result);
                    } else {
                        step_results.push(result.clone());

                        // Handle failure
                        let error_action = error_handler
                            .handle_step_failure(
                                &result,
                                Some(&workflow_config.default_error_strategy),
                            )
                            .await
                            .unwrap_or(ErrorAction::StopWorkflow);

                        match error_action {
                            ErrorAction::StopWorkflow => {
                                branch_success = false;
                                branch_error = Some(format!(
                                    "Branch '{}' failed at step '{}': {}",
                                    branch_name,
                                    step.name,
                                    result.error.as_deref().unwrap_or("Unknown error")
                                ));
                                break;
                            }
                            ErrorAction::ContinueToNext => {
                                warn!(
                                    "Branch '{}' continuing after failure in step: {}",
                                    branch_name, step.name
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
                Err(e) => {
                    branch_success = false;
                    branch_error = Some(format!(
                        "Branch '{}' error at step '{}': {}",
                        branch_name, step.name, e
                    ));
                    break;
                }
            }

            // Check branch timeout
            if let Some(timeout) = branch.timeout {
                if start_time.elapsed() > timeout {
                    branch_success = false;
                    branch_error = Some(format!("Branch '{}' timed out", branch_name));
                    break;
                }
            }
        }

        let duration = start_time.elapsed();

        let result = if branch_success {
            BranchResult::success(branch_name.clone(), step_results, duration, required)
        } else {
            BranchResult::failure(
                branch_name.clone(),
                step_results,
                duration,
                branch_error.unwrap_or_else(|| "Unknown error".to_string()),
                required,
            )
        };

        debug!(
            "Branch '{}' completed with success={}",
            branch_name, result.success
        );
        result
    }

    /// Execute the parallel workflow
    pub async fn execute_workflow(&self) -> Result<ParallelWorkflowResult> {
        let start_time = Instant::now();
        info!(
            "Starting parallel workflow: {} with {} branches",
            self.name,
            self.branches.len()
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
                "parallel".to_string(),
                WorkflowExecutionPhase::WorkflowStart,
            );
            let _ = workflow_executor.execute_workflow_hooks(hook_ctx).await;
        }

        // Start execution tracking
        self.state_manager.start_execution().await?;

        // Create shared resources
        let step_executor = Arc::new(self.step_executor.clone());
        let state_manager = Arc::new(Mutex::new(self.state_manager.clone()));
        let error_handler = Arc::new(self.error_handler.clone());
        let semaphore = Arc::new(Semaphore::new(self.config.max_concurrency));

        // Atomic fail signal for fail-fast
        let fail_signal = Arc::new(tokio::sync::Mutex::new(false));

        let mut branch_handles = Vec::new();

        debug!("Spawning {} branches", self.branches.len());

        // Execute parallel fork hooks
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
                "parallel".to_string(),
                WorkflowExecutionPhase::ParallelFork,
            );
            hook_ctx = hook_ctx.with_pattern_context(
                "branch_count".to_string(),
                serde_json::Value::Number(self.branches.len().into()),
            );
            hook_ctx = hook_ctx.with_pattern_context(
                "max_concurrency".to_string(),
                serde_json::Value::Number(self.config.max_concurrency.into()),
            );
            let _ = workflow_executor.execute_workflow_hooks(hook_ctx).await;
        }

        let workflow_metadata = self.metadata.clone();
        let has_hooks = self.workflow_executor.is_some();

        for branch in self.branches.clone() {
            let step_executor = step_executor.clone();
            let state_manager = state_manager.clone();
            let error_handler = error_handler.clone();
            let workflow_config = self.workflow_config.clone();
            let semaphore = semaphore.clone();
            let fail_signal = fail_signal.clone();
            let fail_fast = self.config.fail_fast;
            let workflow_metadata = workflow_metadata.clone();

            let handle = tokio::spawn(async move {
                // Check if we should stop due to fail-fast
                if fail_fast && *fail_signal.lock().await {
                    return BranchResult::failure(
                        branch.name.clone(),
                        vec![],
                        Duration::from_secs(0),
                        "Skipped due to fail-fast".to_string(),
                        branch.required,
                    );
                }

                // Acquire semaphore permit for concurrency control
                let _permit = semaphore.acquire().await.unwrap();

                let result = Self::execute_branch(
                    branch.clone(),
                    step_executor,
                    state_manager,
                    error_handler,
                    workflow_config,
                    Some(workflow_metadata),
                    has_hooks,
                )
                .await;

                // Store the result in a shared location before signaling
                let should_signal = !result.success && result.required && fail_fast;

                if should_signal {
                    *fail_signal.lock().await = true;
                    // Don't signal immediately - let the result be collected first
                }

                result
            });

            branch_handles.push(handle);
        }

        // Set up timeout for the entire workflow
        let timeout_duration = self.config.timeout.unwrap_or(Duration::from_secs(3600));

        // Wait for all branches or timeout or fail-fast signal
        let mut stopped_early = false;

        debug!("Starting to wait for {} branches", branch_handles.len());
        debug!("Fail-fast enabled: {}", self.config.fail_fast);
        debug!("Timeout duration: {:?}", timeout_duration);

        // Wait for branches with timeout
        let fail_fast = self.config.fail_fast;
        let branches_future = async {
            let mut local_branch_results = Vec::new();
            let mut local_stopped_early = false;

            for handle in branch_handles {
                match handle.await {
                    Ok(result) => {
                        debug!("Branch completed: {}", result.branch_name);
                        let should_stop = fail_fast && !result.success && result.required;

                        local_branch_results.push(result);

                        if should_stop {
                            warn!("Required branch failed, stopping due to fail-fast");
                            local_stopped_early = true;
                            break;
                        }
                    }
                    Err(e) => {
                        error!("Branch execution panicked: {}", e);
                        local_branch_results.push(BranchResult::failure(
                            "unknown".to_string(),
                            vec![],
                            Duration::from_secs(0),
                            format!("Branch panicked: {}", e),
                            true,
                        ));
                        if fail_fast {
                            local_stopped_early = true;
                            break;
                        }
                    }
                }
            }
            (local_branch_results, local_stopped_early)
        };

        let (collected_results, early_stop) = tokio::select! {
            results = branches_future => {
                debug!("Branch processing completed");
                results
            }
            _ = tokio::time::sleep(timeout_duration) => {
                warn!("Parallel workflow '{}' timed out", self.name);
                (Vec::new(), true)
            }
        };

        let branch_results = collected_results;
        stopped_early = stopped_early || early_stop;

        // Execute parallel join hooks
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
                "parallel".to_string(),
                WorkflowExecutionPhase::ParallelJoin,
            );
            hook_ctx = hook_ctx.with_pattern_context(
                "completed_branches".to_string(),
                serde_json::Value::Number(branch_results.len().into()),
            );
            hook_ctx = hook_ctx.with_pattern_context(
                "stopped_early".to_string(),
                serde_json::Value::Bool(stopped_early),
            );
            let _ = workflow_executor.execute_workflow_hooks(hook_ctx).await;
        }

        // Complete execution tracking
        let all_required_succeeded = branch_results
            .iter()
            .filter(|r| r.required)
            .all(|r| r.success);

        self.state_manager
            .complete_execution(all_required_succeeded)
            .await?;

        let duration = start_time.elapsed();
        let result =
            ParallelWorkflowResult::new(self.name.clone(), branch_results, duration, stopped_early);

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
                "parallel".to_string(),
                WorkflowExecutionPhase::WorkflowComplete,
            );
            let _ = workflow_executor.execute_workflow_hooks(hook_ctx).await;
        }

        info!(
            "Parallel workflow '{}' completed: {} branches succeeded, {} failed",
            self.name, result.successful_branches, result.failed_branches
        );

        Ok(result)
    }
}

#[async_trait]
impl BaseAgent for ParallelWorkflow {
    fn metadata(&self) -> &ComponentMetadata {
        &self.metadata
    }

    async fn execute(&self, input: AgentInput, _context: ExecutionContext) -> Result<AgentOutput> {
        // Convert AgentInput to workflow execution
        // The workflow will use the input text as an execution trigger

        // Validate input first
        self.validate_input(&input).await?;

        // Execute the workflow using existing implementation
        let workflow_result = self.execute_workflow().await?;

        // Convert ParallelWorkflowResult to AgentOutput
        let output_text = if workflow_result.success {
            format!(
                "Parallel workflow '{}' completed successfully. {} branches executed, {} succeeded, {} failed. Duration: {:?}",
                workflow_result.workflow_name,
                workflow_result.branch_results.len(),
                workflow_result.successful_branches,
                workflow_result.failed_branches,
                workflow_result.duration
            )
        } else {
            format!(
                "Parallel workflow '{}' failed: {}. {} branches executed, {} succeeded, {} failed. Duration: {:?}",
                workflow_result.workflow_name,
                workflow_result.error.as_deref().unwrap_or("Unknown error"),
                workflow_result.branch_results.len(),
                workflow_result.successful_branches,
                workflow_result.failed_branches,
                workflow_result.duration
            )
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
            .insert("workflow_type".to_string(), serde_json::json!("parallel"));
        metadata.extra.insert(
            "workflow_name".to_string(),
            serde_json::json!(workflow_result.workflow_name),
        );
        metadata.extra.insert(
            "total_branches".to_string(),
            serde_json::json!(workflow_result.branch_results.len()),
        );
        metadata.extra.insert(
            "successful_branches".to_string(),
            serde_json::json!(workflow_result.successful_branches),
        );
        metadata.extra.insert(
            "failed_branches".to_string(),
            serde_json::json!(workflow_result.failed_branches),
        );
        metadata.extra.insert(
            "stopped_early".to_string(),
            serde_json::json!(workflow_result.stopped_early),
        );
        metadata.extra.insert(
            "max_concurrency".to_string(),
            serde_json::json!(self.config.max_concurrency),
        );
        metadata.extra.insert(
            "fail_fast".to_string(),
            serde_json::json!(self.config.fail_fast),
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

        // Validate that we have branches to execute
        if self.branches.is_empty() {
            return Err(LLMSpellError::Validation {
                message: "Cannot execute parallel workflow without branches".to_string(),
                field: Some("branches".to_string()),
            });
        }

        // Validate max concurrency
        if self.config.max_concurrency == 0 {
            return Err(LLMSpellError::Validation {
                message: "Max concurrency must be at least 1".to_string(),
                field: Some("max_concurrency".to_string()),
            });
        }

        Ok(())
    }

    async fn handle_error(&self, error: LLMSpellError) -> Result<AgentOutput> {
        // Handle workflow-specific errors gracefully
        let error_text = match &error {
            LLMSpellError::Workflow { message, step, .. } => {
                if let Some(step_name) = step {
                    format!(
                        "Parallel workflow error in step '{}': {}",
                        step_name, message
                    )
                } else {
                    format!("Parallel workflow error: {}", message)
                }
            }
            LLMSpellError::Validation { message, field } => {
                if let Some(field_name) = field {
                    format!("Validation error in field '{}': {}", field_name, message)
                } else {
                    format!("Validation error: {}", message)
                }
            }
            _ => format!("Parallel workflow error: {}", error),
        };

        let mut metadata = llmspell_core::types::OutputMetadata::default();
        metadata.extra.insert(
            "error_type".to_string(),
            serde_json::json!("workflow_error"),
        );
        metadata
            .extra
            .insert("workflow_type".to_string(), serde_json::json!("parallel"));
        metadata
            .extra
            .insert("workflow_name".to_string(), serde_json::json!(self.name));
        metadata.extra.insert(
            "branch_count".to_string(),
            serde_json::json!(self.branches.len()),
        );

        Ok(AgentOutput::text(error_text).with_metadata(metadata))
    }
}

#[async_trait]
impl Workflow for ParallelWorkflow {
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
        // Get the current workflow status
        let status = self.state_manager.get_status().await?;

        // Convert our WorkflowStatus to CoreWorkflowStatus
        use crate::traits::WorkflowStatus;
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

/// Builder for parallel workflows
pub struct ParallelWorkflowBuilder {
    name: String,
    description: String,
    branches: Vec<ParallelBranch>,
    config: ParallelConfig,
    workflow_config: WorkflowConfig,
    workflow_executor: Option<Arc<WorkflowExecutor>>,
}

impl ParallelWorkflowBuilder {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: String::new(),
            branches: Vec::new(),
            config: ParallelConfig::default(),
            workflow_config: WorkflowConfig::default(),
            workflow_executor: None,
        }
    }

    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = desc.into();
        self
    }

    pub fn add_branch(mut self, branch: ParallelBranch) -> Self {
        self.branches.push(branch);
        self
    }

    pub fn with_max_concurrency(mut self, max: usize) -> Self {
        self.config.max_concurrency = max;
        self
    }

    pub fn fail_fast(mut self, enabled: bool) -> Self {
        self.config.fail_fast = enabled;
        self
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.config.timeout = Some(timeout);
        self
    }

    pub fn continue_on_optional_failure(mut self, enabled: bool) -> Self {
        self.config.continue_on_optional_failure = enabled;
        self
    }

    pub fn with_workflow_config(mut self, config: WorkflowConfig) -> Self {
        self.workflow_config = config;
        self
    }

    /// Enable hook integration with a WorkflowExecutor
    pub fn with_hooks(mut self, workflow_executor: Arc<WorkflowExecutor>) -> Self {
        self.workflow_executor = Some(workflow_executor);
        self
    }

    pub fn build(self) -> Result<ParallelWorkflow> {
        if self.branches.is_empty() {
            return Err(llmspell_core::LLMSpellError::Configuration {
                message: "Parallel workflow must have at least one branch".to_string(),
                source: None,
            });
        }

        if self.config.max_concurrency == 0 {
            return Err(llmspell_core::LLMSpellError::Configuration {
                message: "Max concurrency must be at least 1".to_string(),
                source: None,
            });
        }

        if let Some(workflow_executor) = self.workflow_executor {
            Ok(ParallelWorkflow::new_with_hooks(
                self.name,
                self.branches,
                self.config,
                self.workflow_config,
                workflow_executor,
            ))
        } else {
            Ok(ParallelWorkflow::new(
                self.name,
                self.branches,
                self.config,
                self.workflow_config,
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::StepType;
    #[tokio::test]
    async fn test_parallel_builder() {
        let branch1 = ParallelBranch::new("branch1".to_string()).add_step(TraitWorkflowStep::new(
            "step1".to_string(),
            StepType::Tool {
                tool_name: "calculator".to_string(),
                parameters: serde_json::json!({"expression": "1+1"}),
            },
        ));

        let workflow = ParallelWorkflowBuilder::new("test_parallel")
            .description("Test parallel workflow")
            .add_branch(branch1)
            .with_max_concurrency(2)
            .build()
            .unwrap();

        assert_eq!(workflow.name(), "test_parallel");
        assert_eq!(workflow.branch_count(), 1);
    }
    #[tokio::test]
    async fn test_parallel_validation() {
        // Empty branches
        let result = ParallelWorkflowBuilder::new("test").build();

        assert!(result.is_err());

        // Zero concurrency
        let branch = ParallelBranch::new("branch".to_string());
        let result = ParallelWorkflowBuilder::new("test")
            .add_branch(branch)
            .with_max_concurrency(0)
            .build();

        assert!(result.is_err());
    }
}
