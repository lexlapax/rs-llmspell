//! ABOUTME: Workflow trait for orchestration components
//! ABOUTME: Extends `BaseAgent` with step management and execution planning

use super::base_agent::BaseAgent;
use crate::types::AgentOutput;
use crate::{ComponentId, Result};
use async_trait::async_trait;
use core::time::Duration;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Workflow step definition.
///
/// Represents a single step in a workflow, including its dependencies,
/// retry policies, and timeout configuration. Steps are executed in
/// topological order based on their dependencies.
///
/// # Examples
///
/// ```
/// use llmspell_core::{ComponentId, traits::workflow::{WorkflowStep, RetryPolicy}};
/// use std::time::Duration;
///
/// let step1_id = ComponentId::new();
/// let step2_id = ComponentId::new();
/// let agent_id = ComponentId::new();
///
/// let step = WorkflowStep::new("analyze_data".to_string(), agent_id)
///     .with_dependency(step1_id)
///     .with_dependency(step2_id)
///     .with_retry(RetryPolicy::default())
///     .with_timeout(Duration::from_secs(300));
///
/// assert_eq!(step.name, "analyze_data");
/// assert_eq!(step.dependencies.len(), 2);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct WorkflowStep {
    pub component_id: ComponentId,
    pub dependencies: Vec<ComponentId>,
    pub id: ComponentId,
    pub name: String,
    pub retry_policy: Option<RetryPolicy>,
    pub timeout: Option<Duration>,
}

impl WorkflowStep {
    /// Create a new workflow step
    #[must_use]
    #[inline]
    pub fn new(name: String, component_id: ComponentId) -> Self {
        Self {
            component_id,
            dependencies: Vec::new(),
            id: ComponentId::new(),
            name,
            retry_policy: None,
            timeout: None,
        }
    }

    /// Create a workflow step with all fields
    #[must_use]
    #[inline]
    pub const fn with_all_fields(
        id: ComponentId,
        name: String,
        component_id: ComponentId,
        dependencies: Vec<ComponentId>,
        retry_policy: Option<RetryPolicy>,
        timeout: Option<Duration>,
    ) -> Self {
        Self {
            component_id,
            dependencies,
            id,
            name,
            retry_policy,
            timeout,
        }
    }

    /// Set dependencies
    #[must_use]
    #[inline]
    pub fn with_dependencies(mut self, dependencies: Vec<ComponentId>) -> Self {
        self.dependencies = dependencies;
        self
    }

    /// Add a dependency
    #[must_use]
    #[inline]
    pub fn with_dependency(mut self, dep: ComponentId) -> Self {
        self.dependencies.push(dep);
        self
    }

    /// Set retry policy
    #[must_use]
    #[inline]
    pub const fn with_retry(mut self, policy: RetryPolicy) -> Self {
        self.retry_policy = Some(policy);
        self
    }

    /// Set timeout
    #[must_use]
    #[inline]
    pub const fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }
}

/// Retry policy for workflow steps.
///
/// Defines how failed steps should be retried, including the number of attempts,
/// backoff strategy, and delay between retries.
///
/// # Examples
///
/// ```
/// use llmspell_core::traits::workflow::RetryPolicy;
///
/// // RetryPolicy is #[non_exhaustive], so we use Default and modify fields
/// let mut policy = RetryPolicy::default();
/// policy.max_attempts = 5;
/// policy.backoff_seconds = 2;
/// policy.exponential_backoff = true;
///
/// // Default policy
/// let default = RetryPolicy::default();
/// assert_eq!(default.max_attempts, 3);
/// assert!(default.exponential_backoff);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct RetryPolicy {
    pub backoff_seconds: u32,
    pub exponential_backoff: bool,
    pub max_attempts: u32,
}

impl Default for RetryPolicy {
    #[inline]
    fn default() -> Self {
        Self {
            backoff_seconds: 1,
            exponential_backoff: true,
            max_attempts: 3,
        }
    }
}

impl RetryPolicy {
    /// Create a new retry policy
    #[must_use]
    #[inline]
    pub const fn new(max_attempts: u32, backoff_seconds: u32, exponential_backoff: bool) -> Self {
        Self {
            backoff_seconds,
            exponential_backoff,
            max_attempts,
        }
    }
}

impl Config {
    /// Create a new workflow configuration
    #[must_use]
    #[inline]
    pub const fn new() -> Self {
        Self {
            continue_on_error: false,
            max_parallel: None,
            timeout: None,
        }
    }

    /// Set continue on error behavior
    #[must_use]
    #[inline]
    pub const fn with_continue_on_error(mut self, continue_on_error: bool) -> Self {
        self.continue_on_error = continue_on_error;
        self
    }

    /// Set maximum parallel executions
    #[must_use]
    #[inline]
    pub const fn with_max_parallel(mut self, max_parallel: Option<usize>) -> Self {
        self.max_parallel = max_parallel;
        self
    }

    /// Set global timeout
    #[must_use]
    #[inline]
    pub const fn with_timeout(mut self, timeout: Option<Duration>) -> Self {
        self.timeout = timeout;
        self
    }
}

/// Step execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct StepResult {
    pub duration: Duration,
    pub error: Option<String>,
    pub output: AgentOutput,
    pub retry_count: u32,
    pub step_id: ComponentId,
    pub success: bool,
}

/// Workflow execution status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[non_exhaustive]
pub enum Status {
    Cancelled,
    Completed,
    Failed,
    Pending,
    Running,
}

impl StepResult {
    /// Create a failed result
    #[must_use]
    #[inline]
    pub fn failure(
        step_id: ComponentId,
        error: String,
        duration: Duration,
        retry_count: u32,
    ) -> Self {
        Self {
            duration,
            error: Some(error),
            output: AgentOutput::text(String::new()),
            retry_count,
            step_id,
            success: false,
        }
    }

    /// Create a successful result
    #[must_use]
    #[inline]
    pub const fn success(step_id: ComponentId, output: AgentOutput, duration: Duration) -> Self {
        Self {
            duration,
            error: None,
            output,
            retry_count: 0,
            step_id,
            success: true,
        }
    }
}

/// Workflow configuration.
///
/// Controls workflow execution behavior including parallelism limits,
/// error handling strategy, and global timeouts.
///
/// # Examples
///
/// ```
/// use llmspell_core::traits::workflow::Config;
/// use std::time::Duration;
///
/// // Config is #[non_exhaustive], so we use Default and modify fields
/// let mut config = Config::default();
/// config.max_parallel = Some(4);
/// config.continue_on_error = true;
/// config.timeout = Some(Duration::from_secs(3600));
///
/// // Default configuration
/// let default = Config::default();
/// assert_eq!(default.max_parallel, None); // No limit
/// assert!(!default.continue_on_error); // Stop on error
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[non_exhaustive]
pub struct Config {
    /// Continue on step failure
    pub continue_on_error: bool,
    /// Maximum parallel executions
    pub max_parallel: Option<usize>,
    /// Global timeout for workflow
    pub timeout: Option<Duration>,
}

/// Workflow trait for orchestration components.
///
/// Extends `BaseAgent` to create workflows that orchestrate multiple components.
/// Workflows manage step execution order, handle dependencies, and coordinate
/// parallel execution while respecting configuration limits.
///
/// # Key Features
///
/// - **Dependency Management**: Steps execute in topological order
/// - **Parallel Execution**: Multiple independent steps can run concurrently
/// - **Error Handling**: Configurable continue-on-error behavior
/// - **Retry Logic**: Per-step retry policies with backoff
/// - **Circular Dependency Detection**: Validates workflow before execution
///
/// # Implementation Requirements
///
/// - Must detect circular dependencies during validation
/// - Should respect max_parallel configuration
/// - Must maintain step execution results
/// - Should handle step failures according to configuration
///
/// # Examples
///
/// ```ignore
/// use llmspell_core::{
///     ComponentId, ComponentMetadata, Result,
///     types::{AgentInput, AgentOutput, ExecutionContext},
///     traits::{
///         base_agent::BaseAgent,
///         workflow::{Workflow, Config, WorkflowStep, Status, StepResult}
///     }
/// };
/// use async_trait::async_trait;
///
/// struct DataPipeline {
///     metadata: ComponentMetadata,
///     config: Config,
///     steps: Vec<WorkflowStep>,
///     status: Status,
///     results: Vec<StepResult>,
/// }
///
/// #[async_trait]
/// impl Workflow for DataPipeline {
///     fn config(&self) -> &Config {
///         &self.config
///     }
///     
///     async fn add_step(&self, step: WorkflowStep) -> Result<()> {
///         // Validate no circular dependencies
///         // With interior mutability pattern
///         self.steps.lock().await.push(step);
///         self.validate().await?;
///         Ok(())
///     }
///     
///     async fn remove_step(&self, step_id: ComponentId) -> Result<()> {
///         // With interior mutability pattern
///         self.steps.lock().await.retain(|s| s.id != step_id);
///         Ok(())
///     }
///     
///     async fn get_steps(&self) -> Result<Vec<WorkflowStep>> {
///         Ok(self.steps.clone())
///     }
///     
///     async fn status(&self) -> Result<WorkflowStatus> {
///         Ok(self.status.clone())
///     }
///     
///     async fn get_results(&self) -> Result<Vec<StepResult>> {
///         Ok(self.results.clone())
///     }
/// }
///
/// # impl BaseAgent for DataPipeline {
/// #     fn metadata(&self) -> &ComponentMetadata { &self.metadata }
/// #     async fn execute(&self, input: AgentInput, context: ExecutionContext) -> Result<AgentOutput> {
/// #         Ok(AgentOutput::text("Workflow complete"))
/// #     }
/// #     async fn validate_input(&self, input: &AgentInput) -> Result<()> { Ok(()) }
/// #     async fn handle_error(&self, error: llmspell_core::LLMSpellError) -> Result<AgentOutput> {
/// #         Ok(AgentOutput::text("Error"))
/// #     }
/// # }
/// ```
#[async_trait]
pub trait Workflow: BaseAgent {
    /// Add step to workflow
    async fn add_step(&self, step: WorkflowStep) -> Result<()>;

    /// Get workflow configuration
    fn config(&self) -> &Config;

    /// Get execution plan (topologically sorted)
    #[inline]
    async fn plan_execution(&self) -> Result<Vec<WorkflowStep>> {
        let steps = match self.get_steps().await {
            Ok(steps) => steps,
            Err(err) => return Err(err),
        };

        // Build dependency graph
        let mut graph: HashMap<ComponentId, HashSet<ComponentId>> = HashMap::new();
        let mut in_degree: HashMap<ComponentId, usize> = HashMap::new();

        for step in &steps {
            graph.entry(step.id).or_default();
            in_degree.entry(step.id).or_insert(0);

            for dep in &step.dependencies {
                graph.entry(*dep).or_default().insert(step.id);
                if let Some(degree) = in_degree.get_mut(&step.id) {
                    *degree = degree.saturating_add(1);
                } else {
                    in_degree.insert(step.id, 1);
                }
            }
        }

        // Topological sort using Kahn's algorithm
        let mut queue: Vec<ComponentId> = in_degree
            .iter()
            .filter(|(_, &degree)| degree == 0)
            .map(|(id, _)| *id)
            .collect();

        let mut sorted = Vec::new();

        while let Some(node) = queue.pop() {
            if let Some(step) = steps.iter().find(|step_item| step_item.id == node) {
                sorted.push(step.clone());
            }

            if let Some(neighbors) = graph.get(&node) {
                let neighbors_vec: Vec<_> = neighbors.iter().copied().collect();
                for neighbor in neighbors_vec {
                    if let Some(degree) = in_degree.get_mut(&neighbor) {
                        *degree = degree.saturating_sub(1);
                        if *degree == 0 {
                            queue.push(neighbor);
                        }
                    }
                }
            }
        }

        // Check for cycles
        if sorted.len() != steps.len() {
            return Err(crate::LLMSpellError::Workflow {
                message: "Workflow contains circular dependencies".to_owned(),
                step: None,
                source: None,
            });
        }

        return Ok(sorted);
    }

    /// Get all steps
    async fn get_steps(&self) -> Result<Vec<WorkflowStep>>;

    /// Get step results
    async fn get_results(&self) -> Result<Vec<StepResult>>;

    /// Get result for specific step
    #[inline]
    async fn get_step_result(&self, step_id: ComponentId) -> Result<Option<StepResult>> {
        let results = match self.get_results().await {
            Ok(results) => results,
            Err(e) => return Err(e),
        };
        Ok(results
            .into_iter()
            .find(|result_item| result_item.step_id == step_id))
    }

    /// Remove step from workflow
    async fn remove_step(&self, step_id: ComponentId) -> Result<()>;

    /// Get workflow status
    async fn status(&self) -> Result<Status>;

    /// Check if workflow can execute (no cycles, all dependencies valid)
    #[inline]
    async fn validate(&self) -> Result<()> {
        // Check all dependencies exist first
        let steps = match self.get_steps().await {
            Ok(steps) => steps,
            Err(err) => return Err(err),
        };
        let step_ids: HashSet<ComponentId> = steps.iter().map(|step_item| step_item.id).collect();

        for step in &steps {
            for dep in &step.dependencies {
                if !step_ids.contains(dep) {
                    return Err(crate::LLMSpellError::Workflow {
                        message: format!("Step '{}' has invalid dependency: {:?}", step.name, dep),
                        step: Some(step.name.clone()),
                        source: None,
                    });
                }
            }
        }

        // Then check for cycles - plan execution will fail if there are cycles
        match self.plan_execution().await {
            Ok(_) => {}
            Err(e) => return Err(e),
        }

        return Ok(());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::AgentInput;
    use crate::ComponentMetadata;
    use crate::ExecutionContext;
    use std::sync::Arc;
    use tokio::sync::Mutex;
    #[test]
    fn test_workflow_step_builder() {
        let component_id = ComponentId::new();
        let dep_id = ComponentId::new();

        let step = WorkflowStep::new("test_step".to_string(), component_id)
            .with_dependency(dep_id)
            .with_retry(RetryPolicy::default())
            .with_timeout(Duration::from_secs(30));

        assert_eq!(step.name, "test_step");
        assert_eq!(step.component_id, component_id);
        assert_eq!(step.dependencies.len(), 1);
        assert_eq!(step.dependencies[0], dep_id);
        assert!(step.retry_policy.is_some());
        assert_eq!(step.timeout, Some(Duration::from_secs(30)));
    }
    #[test]
    fn test_retry_policy_default() {
        let policy = RetryPolicy::default();
        assert_eq!(policy.max_attempts, 3);
        assert_eq!(policy.backoff_seconds, 1);
        assert!(policy.exponential_backoff);
    }
    #[test]
    fn test_step_result_creation() {
        let step_id = ComponentId::new();
        let output = AgentOutput::text("Success");
        let duration = Duration::from_secs(1);

        // Test success result
        let success = StepResult::success(step_id, output.clone(), duration);
        assert!(success.success);
        assert_eq!(success.error, None);
        assert_eq!(success.retry_count, 0);

        // Test failure result
        let failure = StepResult::failure(step_id, "Error occurred".to_string(), duration, 2);
        assert!(!failure.success);
        assert_eq!(failure.error, Some("Error occurred".to_string()));
        assert_eq!(failure.retry_count, 2);
    }
    #[test]
    fn test_workflow_config_default() {
        let config = Config::default();
        assert_eq!(config.max_parallel, None);
        assert!(!config.continue_on_error);
        assert_eq!(config.timeout, None);
    }

    // Mock workflow implementation
    struct MockWorkflow {
        metadata: ComponentMetadata,
        config: Config,
        steps: Arc<Mutex<Vec<WorkflowStep>>>,
        status: Arc<Mutex<Status>>,
        results: Arc<Mutex<Vec<StepResult>>>,
    }

    impl MockWorkflow {
        fn new() -> Self {
            Self {
                metadata: ComponentMetadata::new(
                    "mock-workflow".to_string(),
                    "A mock workflow for testing".to_string(),
                ),
                config: Config::default(),
                steps: Arc::new(Mutex::new(Vec::new())),
                status: Arc::new(Mutex::new(Status::Pending)),
                results: Arc::new(Mutex::new(Vec::new())),
            }
        }
    }

    #[async_trait]
    impl BaseAgent for MockWorkflow {
        fn metadata(&self) -> &ComponentMetadata {
            &self.metadata
        }

        async fn execute_impl(
            &self,
            _input: AgentInput,
            _context: ExecutionContext,
        ) -> Result<AgentOutput> {
            // Execute workflow
            *self.status.lock().await = Status::Running;

            // Simulate execution
            let steps = self.steps.lock().await.clone();
            for step in steps {
                let result = StepResult::success(
                    step.id,
                    AgentOutput::text(format!("Executed {}", step.name)),
                    Duration::from_secs(1),
                );
                self.results.lock().await.push(result);
            }

            *self.status.lock().await = Status::Completed;
            Ok(AgentOutput::text("Workflow completed"))
        }

        async fn validate_input(&self, input: &AgentInput) -> Result<()> {
            if input.text.is_empty() {
                return Err(crate::LLMSpellError::Validation {
                    message: "Input text cannot be empty".to_string(),
                    field: Some("text".to_string()),
                });
            }
            Ok(())
        }

        async fn handle_error(&self, error: crate::LLMSpellError) -> Result<AgentOutput> {
            *self.status.lock().await = Status::Failed;
            Ok(AgentOutput::text(format!("Workflow error: {}", error)))
        }
    }

    #[async_trait]
    impl Workflow for MockWorkflow {
        fn config(&self) -> &Config {
            &self.config
        }

        async fn add_step(&self, step: WorkflowStep) -> Result<()> {
            self.steps.lock().await.push(step);
            Ok(())
        }

        async fn remove_step(&self, step_id: ComponentId) -> Result<()> {
            let mut steps = self.steps.lock().await;
            steps.retain(|s| s.id != step_id);
            Ok(())
        }

        async fn get_steps(&self) -> Result<Vec<WorkflowStep>> {
            Ok(self.steps.lock().await.clone())
        }

        async fn status(&self) -> Result<Status> {
            Ok(self.status.lock().await.clone())
        }

        async fn get_results(&self) -> Result<Vec<StepResult>> {
            Ok(self.results.lock().await.clone())
        }
    }
    #[tokio::test]
    async fn test_workflow_step_management() {
        let workflow = MockWorkflow::new();

        // Add steps
        let step1 = WorkflowStep::new("step1".to_string(), ComponentId::new());
        let step2 =
            WorkflowStep::new("step2".to_string(), ComponentId::new()).with_dependency(step1.id);

        workflow.add_step(step1.clone()).await.unwrap();
        workflow.add_step(step2.clone()).await.unwrap();

        let steps = workflow.get_steps().await.unwrap();
        assert_eq!(steps.len(), 2);

        // Remove step
        workflow.remove_step(step1.id).await.unwrap();
        let steps = workflow.get_steps().await.unwrap();
        assert_eq!(steps.len(), 1);
        assert_eq!(steps[0].id, step2.id);
    }
    #[tokio::test]
    async fn test_workflow_execution_planning() {
        let workflow = MockWorkflow::new();

        // Create steps with dependencies: step1 -> step2 -> step3
        let step1 = WorkflowStep::new("step1".to_string(), ComponentId::new());
        let step2 =
            WorkflowStep::new("step2".to_string(), ComponentId::new()).with_dependency(step1.id);
        let step3 =
            WorkflowStep::new("step3".to_string(), ComponentId::new()).with_dependency(step2.id);

        workflow.add_step(step3.clone()).await.unwrap();
        workflow.add_step(step1.clone()).await.unwrap();
        workflow.add_step(step2.clone()).await.unwrap();

        // Plan execution should order them correctly
        let plan = workflow.plan_execution().await.unwrap();
        assert_eq!(plan.len(), 3);
        assert_eq!(plan[0].id, step1.id);
        assert_eq!(plan[1].id, step2.id);
        assert_eq!(plan[2].id, step3.id);
    }
    #[tokio::test]
    async fn test_workflow_circular_dependency_detection() {
        let workflow = MockWorkflow::new();

        // Create circular dependency: step1 -> step2 -> step1
        let step1 = WorkflowStep::new("step1".to_string(), ComponentId::new());
        let step2 =
            WorkflowStep::new("step2".to_string(), ComponentId::new()).with_dependency(step1.id);
        let step1_circular = WorkflowStep {
            dependencies: vec![step2.id],
            ..step1.clone()
        };

        workflow.add_step(step1_circular).await.unwrap();
        workflow.add_step(step2).await.unwrap();

        // Planning should fail
        let result = workflow.plan_execution().await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("circular"));
    }
    #[tokio::test]
    async fn test_workflow_validation() {
        let workflow = MockWorkflow::new();

        // Valid workflow
        let step1 = WorkflowStep::new("step1".to_string(), ComponentId::new());
        let step2 =
            WorkflowStep::new("step2".to_string(), ComponentId::new()).with_dependency(step1.id);

        workflow.add_step(step1).await.unwrap();
        workflow.add_step(step2).await.unwrap();

        assert!(workflow.validate().await.is_ok());

        // Invalid workflow with missing dependency
        let invalid_dep = ComponentId::new();
        let step3 =
            WorkflowStep::new("step3".to_string(), ComponentId::new()).with_dependency(invalid_dep);

        workflow.add_step(step3).await.unwrap();

        let result = workflow.validate().await;
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(
            error_msg.contains("invalid dependency"),
            "Error message was: {}",
            error_msg
        );
    }
    #[tokio::test]
    async fn test_workflow_execution_and_results() {
        let workflow = MockWorkflow::new();

        // Add steps
        let step1 = WorkflowStep::new("step1".to_string(), ComponentId::new());
        workflow.add_step(step1.clone()).await.unwrap();

        // Execute workflow
        let input = AgentInput::text("Execute workflow");
        let context = ExecutionContext::with_conversation("session".to_string());

        let output = workflow.execute(input, context).await.unwrap();
        assert_eq!(output.text, "Workflow completed");

        // Check status and results
        assert_eq!(workflow.status().await.unwrap(), Status::Completed);

        let results = workflow.get_results().await.unwrap();
        assert_eq!(results.len(), 1);
        assert!(results[0].success);

        // Get specific step result
        let step_result = workflow.get_step_result(step1.id).await.unwrap();
        assert!(step_result.is_some());
    }
}
