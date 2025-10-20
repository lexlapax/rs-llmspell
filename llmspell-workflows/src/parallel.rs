// ABOUTME: Parallel workflow implementation for concurrent branch execution
// ABOUTME: Implements fork-join pattern with fixed concurrency limits and fail-fast error handling

use crate::{
    error_handling::{ErrorAction, ErrorHandler},
    hooks::{WorkflowExecutionPhase, WorkflowExecutor, WorkflowHookContext},
    result::{WorkflowError, WorkflowResult, WorkflowType},
    state::StateManager,
    step_executor::StepExecutor,
    traits::{StepResult, WorkflowStep as TraitWorkflowStep},
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
use std::{
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::{Mutex, RwLock, Semaphore};
use tracing::{debug, error, info, instrument, warn};

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
    /// Create a new parallel branch
    pub fn new(name: String) -> Self {
        Self {
            name,
            description: String::new(),
            steps: Vec::new(),
            required: true,
            timeout: None,
        }
    }

    /// Set the description for this branch
    pub fn with_description(mut self, description: String) -> Self {
        self.description = description;
        self
    }

    /// Add a step to this branch
    pub fn add_step(mut self, step: TraitWorkflowStep) -> Self {
        self.steps.push(step);
        self
    }

    /// Mark this branch as optional (failures won't fail the workflow)
    pub fn optional(mut self) -> Self {
        self.required = false;
        self
    }

    /// Set a timeout for this branch
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
    /// Create a successful branch result
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

    /// Create a failed branch result
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
    /// Create a new parallel workflow result
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
        let core_config = CoreWorkflowConfig::new()
            .with_max_parallel(Some(config.max_concurrency))
            .with_continue_on_error(!config.fail_fast)
            .with_timeout(config.timeout.or(workflow_config.max_execution_time));

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
        let core_config = CoreWorkflowConfig::new()
            .with_max_parallel(Some(config.max_concurrency))
            .with_continue_on_error(!config.fail_fast)
            .with_timeout(config.timeout.or(workflow_config.max_execution_time));

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

    /// Create with registry for component lookup
    pub fn new_with_registry(
        name: String,
        branches: Vec<ParallelBranch>,
        config: ParallelConfig,
        workflow_config: WorkflowConfig,
        registry: Option<Arc<dyn ComponentLookup>>,
    ) -> Self {
        let error_strategy = workflow_config.default_error_strategy.clone();
        let error_handler = ErrorHandler::new(error_strategy);
        let state_manager = StateManager::new(workflow_config.clone());
        let step_executor = if let Some(reg) = registry {
            StepExecutor::new_with_registry(workflow_config.clone(), reg)
        } else {
            StepExecutor::new(workflow_config.clone())
        };

        let metadata = ComponentMetadata::new(name.clone(), "Parallel workflow".to_string());

        // Create core workflow config from our config
        let core_config = CoreWorkflowConfig::new()
            .with_max_parallel(Some(config.max_concurrency))
            .with_continue_on_error(!config.fail_fast)
            .with_timeout(config.timeout.or(workflow_config.max_execution_time));

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

    /// Create with both hooks and registry
    pub fn new_with_hooks_and_registry(
        name: String,
        branches: Vec<ParallelBranch>,
        config: ParallelConfig,
        workflow_config: WorkflowConfig,
        workflow_executor: Arc<WorkflowExecutor>,
        registry: Option<Arc<dyn ComponentLookup>>,
    ) -> Self {
        let error_strategy = workflow_config.default_error_strategy.clone();
        let error_handler = ErrorHandler::new(error_strategy);
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

        let metadata =
            ComponentMetadata::new(name.clone(), "Parallel workflow with hooks".to_string());

        // Create core workflow config from our config
        let core_config = CoreWorkflowConfig::new()
            .with_max_parallel(Some(config.max_concurrency))
            .with_continue_on_error(!config.fail_fast)
            .with_timeout(config.timeout.or(workflow_config.max_execution_time));

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
    #[allow(clippy::too_many_arguments)]
    #[instrument(level = "info", skip_all, fields(
        branch_name = %branch.name,
        step_count = branch.steps.len(),
        steps = branch.steps.len(),
        execution_id = ?execution_component_id
    ))]
    async fn execute_branch(
        branch: ParallelBranch,
        step_executor: Arc<StepExecutor>,
        state_manager: Arc<Mutex<StateManager>>,
        error_handler: Arc<ErrorHandler>,
        workflow_config: WorkflowConfig,
        workflow_metadata: Option<ComponentMetadata>,
        has_hooks: bool,
        execution_component_id: ComponentId,
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
            // CRITICAL: Use the workflow's execution_component_id, not a new one!
            workflow_state.execution_id = execution_component_id;
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

    // execute_with_state removed - functionality moved to execute_impl
}

#[async_trait]
impl BaseAgent for ParallelWorkflow {
    fn metadata(&self) -> &ComponentMetadata {
        &self.metadata
    }

    #[instrument(level = "info", skip(self, input, context), fields(
        workflow_name = %self.metadata.name,
        branch_count = self.branches.len(),
        max_concurrency = self.config.max_concurrency,
        input_size = input.text.len()
    ))]
    async fn execute_impl(
        &self,
        input: AgentInput,
        context: ExecutionContext,
    ) -> Result<AgentOutput> {
        // Convert AgentInput to workflow execution
        // The workflow will use the input text as an execution trigger

        // Validate input first
        self.validate_input(&input).await?;

        // Execute the workflow - always execute branches (state is optional like in sequential workflow)
        let (workflow_result, execution_id_for_outputs) = {
            // Start of workflow execution
            let start_time = Instant::now();
            // Generate ComponentId once and use it consistently
            let execution_component_id = ComponentId::new();
            let execution_id = execution_component_id.to_string();
            info!(
                "Starting parallel workflow: {} (execution: {}) with {} branches",
                self.name,
                execution_id,
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

            self.state_manager.start_execution().await?;

            // Execute parallel branches inline
            debug!("ParallelWorkflow::execute() starting with {} branches", self.branches.len());
            for (idx, branch) in self.branches.iter().enumerate() {
                debug!("  Branch {}: name='{}', {} steps, required={}",
                       idx, branch.name, branch.steps.len(), branch.required);
            }

            let semaphore = Arc::new(Semaphore::new(self.config.max_concurrency));
            let results = Arc::new(Mutex::new(Vec::<BranchResult>::new()));
            let should_stop = Arc::new(tokio::sync::RwLock::new(false));
            let state_keys = Arc::new(Mutex::new(Vec::<String>::new()));

            let mut branch_handles: Vec<tokio::task::JoinHandle<Result<BranchResult>>> = Vec::new();
            let mut steps_executed = 0usize;
            let mut steps_failed = 0usize;

            debug!("Starting branch execution loop over {} branches", self.branches.len());
            for branch in &self.branches {
                let branch = branch.clone();
                let semaphore = semaphore.clone();
                let _results = results.clone();
                let should_stop = should_stop.clone();
                let _state_keys = state_keys.clone();
                let step_executor = Arc::new(self.step_executor.clone());
                let state_manager = Arc::new(Mutex::new(self.state_manager.clone()));
                let error_handler = Arc::new(self.error_handler.clone());
                let workflow_config = self.workflow_config.clone();
                let _fail_fast = self.config.fail_fast;
                let workflow_executor = self.workflow_executor.clone();
                let metadata = self.metadata.clone();
                let _context_state = context.state.clone();
                let _exec_id = execution_id.clone();
                let exec_component_id = execution_component_id;

                let handle = tokio::spawn(async move {
                    // Check if we should stop before starting
                    if *should_stop.read().await {
                        return Ok(BranchResult {
                            branch_name: branch.name.clone(),
                            success: false,
                            step_results: Vec::new(),
                            duration: Duration::from_secs(0),
                            error: Some("Workflow stopped before branch could start".to_string()),
                            required: branch.required,
                        });
                    }

                    // Acquire semaphore permit
                    let _permit =
                        semaphore
                            .acquire()
                            .await
                            .map_err(|e| LLMSpellError::Component {
                                message: format!("Failed to acquire semaphore: {}", e),
                                source: None,
                            })?;

                    // Execute the branch
                    Ok(Self::execute_branch(
                        branch,
                        step_executor,
                        state_manager,
                        error_handler,
                        workflow_config,
                        Some(metadata),
                        workflow_executor.is_some(),
                        exec_component_id,
                    )
                    .await)
                });

                branch_handles.push(handle);
            }

            // Wait for all branches to complete
            let mut all_branch_results = Vec::new();
            for handle in branch_handles {
                match handle.await {
                    Ok(Ok(result)) => {
                        if result.success {
                            steps_executed += result.step_results.len();
                        } else {
                            steps_failed +=
                                result.step_results.iter().filter(|r| !r.success).count();
                        }
                        all_branch_results.push(result);
                    }
                    Ok(Err(e)) => {
                        error!("Branch execution failed: {}", e);
                        steps_failed += 1;
                    }
                    Err(e) => {
                        error!("Branch task panicked: {}", e);
                        steps_failed += 1;
                    }
                }
            }

            // Build workflow result
            let duration = start_time.elapsed();
            let success = all_branch_results
                .iter()
                .filter(|r| r.required && !r.success)
                .count()
                == 0;

            let result = if success {
                WorkflowResult::success(
                    execution_id.clone(),
                    WorkflowType::Parallel,
                    self.name.clone(),
                    state_keys.lock().await.clone(),
                    steps_executed,
                    duration,
                )
            } else {
                WorkflowResult::failure(
                    execution_id.clone(),
                    WorkflowType::Parallel,
                    self.name.clone(),
                    WorkflowError::StepExecutionFailed {
                        step_name: "parallel_execution".to_string(),
                        reason: format!(
                            "{} required branches failed",
                            all_branch_results
                                .iter()
                                .filter(|r| r.required && !r.success)
                                .count()
                        ),
                    },
                    state_keys.lock().await.clone(),
                    steps_executed,
                    steps_failed,
                    duration,
                )
            };

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

            // result already defined above

            // Store execution_id for output collection
            let exec_id = result.execution_id.clone();

            // Convert to legacy result for backward compatibility
            // This will be removed once all callers are updated
            let branch_results = vec![]; // Branch details are in state now

            let legacy_result = ParallelWorkflowResult {
                workflow_name: result.workflow_name,
                success: result.success,
                branch_results,
                duration: result.duration,
                successful_branches: if result.success {
                    self.branches.len()
                } else {
                    0
                },
                failed_branches: result.steps_failed,
                stopped_early: false,
                error: result.error.map(|e| e.to_string()),
            };

            (legacy_result, Some(exec_id))
        };

        // Convert ParallelWorkflowResult to AgentOutput
        let total_branches = workflow_result.successful_branches + workflow_result.failed_branches;
        let output_text = if workflow_result.success {
            format!(
                "Parallel workflow '{}' completed successfully. {} branches executed, {} succeeded, {} failed. Duration: {:?}",
                workflow_result.workflow_name,
                total_branches,
                workflow_result.successful_branches,
                workflow_result.failed_branches,
                workflow_result.duration
            )
        } else {
            format!(
                "Parallel workflow '{}' failed: {}. {} branches executed, {} succeeded, {} failed. Duration: {:?}",
                workflow_result.workflow_name,
                workflow_result.error.as_deref().unwrap_or("Unknown error"),
                total_branches,
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

        // Add execution_id to metadata
        if let Some(execution_id) = &execution_id_for_outputs {
            metadata
                .extra
                .insert("execution_id".to_string(), serde_json::json!(execution_id));

            // Collect agent outputs from state if available
            let mut agent_outputs = serde_json::Map::new();
            if let Some(ref state) = context.state {
                for branch in &self.branches {
                    for step in &branch.steps {
                        if let StepType::Agent { agent_id, .. } = &step.step_type {
                            let key =
                                format!("workflow:{}:agent:{}:output", execution_id, agent_id);
                            if let Ok(Some(output)) = state.read(&key).await {
                                agent_outputs.insert(agent_id.clone(), output);
                            }
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

        // If workflow failed, return an error so BaseAgent emits workflow.failed event
        if !workflow_result.success {
            return Err(LLMSpellError::Workflow {
                message: output_text.clone(),
                step: workflow_result
                    .error
                    .as_ref()
                    .map(|_| "parallel_execution".to_string()),
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
            WorkflowStatus::PartiallyCompleted => CoreWorkflowStatus::Completed,
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
    registry: Option<Arc<dyn ComponentLookup>>,
}

impl ParallelWorkflowBuilder {
    /// Create a new parallel workflow builder
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: String::new(),
            branches: Vec::new(),
            config: ParallelConfig::default(),
            workflow_config: WorkflowConfig::default(),
            workflow_executor: None,
            registry: None,
        }
    }

    /// Set the description for this parallel workflow
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = desc.into();
        self
    }

    /// Add a branch to the parallel workflow
    pub fn add_branch(mut self, branch: ParallelBranch) -> Self {
        self.branches.push(branch);
        self
    }

    /// Set the maximum concurrency for parallel execution
    pub fn with_max_concurrency(mut self, max: usize) -> Self {
        self.config.max_concurrency = max;
        self
    }

    /// Enable fail-fast mode (stop on first failure)
    pub fn fail_fast(mut self, enabled: bool) -> Self {
        self.config.fail_fast = enabled;
        self
    }

    /// Set a timeout for the parallel workflow
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.config.timeout = Some(timeout);
        self
    }

    /// Continue execution even if optional branches fail
    pub fn continue_on_optional_failure(mut self, enabled: bool) -> Self {
        self.config.continue_on_optional_failure = enabled;
        self
    }

    /// Set the workflow configuration
    pub fn with_workflow_config(mut self, config: WorkflowConfig) -> Self {
        self.workflow_config = config;
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

    /// Build the parallel workflow
    pub fn build(self) -> Result<ParallelWorkflow> {
        debug!("ParallelWorkflowBuilder::build() called with {} branches", self.branches.len());
        for (idx, branch) in self.branches.iter().enumerate() {
            debug!("  Branch {}: name='{}', {} steps", idx, branch.name, branch.steps.len());
        }

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

        debug!("Building parallel workflow with executor={}, registry={}",
               self.workflow_executor.is_some(), self.registry.is_some());

        match (self.workflow_executor, self.registry) {
            (Some(workflow_executor), Some(registry)) => {
                debug!("Creating ParallelWorkflow with hooks and registry");
                Ok(ParallelWorkflow::new_with_hooks_and_registry(
                    self.name,
                    self.branches,
                    self.config,
                    self.workflow_config,
                    workflow_executor,
                    Some(registry),
                ))
            }
            (Some(workflow_executor), None) => Ok(ParallelWorkflow::new_with_hooks(
                self.name,
                self.branches,
                self.config,
                self.workflow_config,
                workflow_executor,
            )),
            (None, Some(registry)) => Ok(ParallelWorkflow::new_with_registry(
                self.name,
                self.branches,
                self.config,
                self.workflow_config,
                Some(registry),
            )),
            (None, None) => Ok(ParallelWorkflow::new(
                self.name,
                self.branches,
                self.config,
                self.workflow_config,
            )),
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
