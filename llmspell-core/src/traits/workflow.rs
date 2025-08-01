//! ABOUTME: Workflow trait for orchestration components
//! ABOUTME: Extends BaseAgent with step management and execution planning

use super::base_agent::BaseAgent;
use crate::types::AgentOutput;
use crate::{ComponentId, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::time::Duration;

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
pub struct WorkflowStep {
    pub id: ComponentId,
    pub name: String,
    pub component_id: ComponentId,
    pub dependencies: Vec<ComponentId>,
    pub retry_policy: Option<RetryPolicy>,
    pub timeout: Option<Duration>,
}

impl WorkflowStep {
    /// Create a new workflow step
    pub fn new(name: String, component_id: ComponentId) -> Self {
        Self {
            id: ComponentId::new(),
            name,
            component_id,
            dependencies: Vec::new(),
            retry_policy: None,
            timeout: None,
        }
    }

    /// Add a dependency
    pub fn with_dependency(mut self, dep: ComponentId) -> Self {
        self.dependencies.push(dep);
        self
    }

    /// Set retry policy
    pub fn with_retry(mut self, policy: RetryPolicy) -> Self {
        self.retry_policy = Some(policy);
        self
    }

    /// Set timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
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
/// let policy = RetryPolicy {
///     max_attempts: 5,
///     backoff_seconds: 2,
///     exponential_backoff: true,
/// };
///
/// // Default policy
/// let default = RetryPolicy::default();
/// assert_eq!(default.max_attempts, 3);
/// assert!(default.exponential_backoff);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryPolicy {
    pub max_attempts: u32,
    pub backoff_seconds: u32,
    pub exponential_backoff: bool,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            backoff_seconds: 1,
            exponential_backoff: true,
        }
    }
}

/// Step execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepResult {
    pub step_id: ComponentId,
    pub output: AgentOutput,
    pub duration: Duration,
    pub success: bool,
    pub error: Option<String>,
    pub retry_count: u32,
}

impl StepResult {
    /// Create a successful result
    pub fn success(step_id: ComponentId, output: AgentOutput, duration: Duration) -> Self {
        Self {
            step_id,
            output,
            duration,
            success: true,
            error: None,
            retry_count: 0,
        }
    }

    /// Create a failed result
    pub fn failure(
        step_id: ComponentId,
        error: String,
        duration: Duration,
        retry_count: u32,
    ) -> Self {
        Self {
            step_id,
            output: AgentOutput::text(String::new()),
            duration,
            success: false,
            error: Some(error),
            retry_count,
        }
    }
}

/// Workflow execution status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum WorkflowStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

/// Workflow configuration.
///
/// Controls workflow execution behavior including parallelism limits,
/// error handling strategy, and global timeouts.
///
/// # Examples
///
/// ```
/// use llmspell_core::traits::workflow::WorkflowConfig;
/// use std::time::Duration;
///
/// let config = WorkflowConfig {
///     max_parallel: Some(4),
///     continue_on_error: true,
///     timeout: Some(Duration::from_secs(3600)),
/// };
///
/// // Default configuration
/// let default = WorkflowConfig::default();
/// assert_eq!(default.max_parallel, None); // No limit
/// assert!(!default.continue_on_error); // Stop on error
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WorkflowConfig {
    /// Maximum parallel executions
    pub max_parallel: Option<usize>,
    /// Continue on step failure
    pub continue_on_error: bool,
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
///         workflow::{Workflow, WorkflowConfig, WorkflowStep, WorkflowStatus, StepResult}
///     }
/// };
/// use async_trait::async_trait;
///
/// struct DataPipeline {
///     metadata: ComponentMetadata,
///     config: WorkflowConfig,
///     steps: Vec<WorkflowStep>,
///     status: WorkflowStatus,
///     results: Vec<StepResult>,
/// }
///
/// #[async_trait]
/// impl Workflow for DataPipeline {
///     fn config(&self) -> &WorkflowConfig {
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
    /// Get workflow configuration
    fn config(&self) -> &WorkflowConfig;

    /// Add step to workflow
    async fn add_step(&self, step: WorkflowStep) -> Result<()>;

    /// Remove step from workflow
    async fn remove_step(&self, step_id: ComponentId) -> Result<()>;

    /// Get all steps
    async fn get_steps(&self) -> Result<Vec<WorkflowStep>>;

    /// Get execution plan (topologically sorted)
    async fn plan_execution(&self) -> Result<Vec<WorkflowStep>> {
        let steps = self.get_steps().await?;

        // Build dependency graph
        let mut graph: HashMap<ComponentId, HashSet<ComponentId>> = HashMap::new();
        let mut in_degree: HashMap<ComponentId, usize> = HashMap::new();

        for step in &steps {
            graph.entry(step.id).or_default();
            in_degree.entry(step.id).or_insert(0);

            for dep in &step.dependencies {
                graph.entry(*dep).or_default().insert(step.id);
                *in_degree.entry(step.id).or_default() += 1;
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
            if let Some(step) = steps.iter().find(|s| s.id == node) {
                sorted.push(step.clone());
            }

            if let Some(neighbors) = graph.get(&node) {
                for neighbor in neighbors {
                    if let Some(degree) = in_degree.get_mut(neighbor) {
                        *degree -= 1;
                        if *degree == 0 {
                            queue.push(*neighbor);
                        }
                    }
                }
            }
        }

        // Check for cycles
        if sorted.len() != steps.len() {
            return Err(crate::LLMSpellError::Workflow {
                message: "Workflow contains circular dependencies".to_string(),
                step: None,
                source: None,
            });
        }

        Ok(sorted)
    }

    /// Get workflow status
    async fn status(&self) -> Result<WorkflowStatus>;

    /// Get step results
    async fn get_results(&self) -> Result<Vec<StepResult>>;

    /// Get result for specific step
    async fn get_step_result(&self, step_id: ComponentId) -> Result<Option<StepResult>> {
        let results = self.get_results().await?;
        Ok(results.into_iter().find(|r| r.step_id == step_id))
    }

    /// Check if workflow can execute (no cycles, all dependencies valid)
    async fn validate(&self) -> Result<()> {
        // Check all dependencies exist first
        let steps = self.get_steps().await?;
        let step_ids: HashSet<ComponentId> = steps.iter().map(|s| s.id).collect();

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
        self.plan_execution().await?;

        Ok(())
    }
}

#[cfg(test)]
#[cfg_attr(test_category = "core")]
mod tests {
    use super::*;
    use crate::types::AgentInput;
    use crate::ComponentMetadata;
    use crate::ExecutionContext;
    use std::sync::Arc;
    use tokio::sync::Mutex;

    #[cfg_attr(test_category = "unit")]
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

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_retry_policy_default() {
        let policy = RetryPolicy::default();
        assert_eq!(policy.max_attempts, 3);
        assert_eq!(policy.backoff_seconds, 1);
        assert!(policy.exponential_backoff);
    }

    #[cfg_attr(test_category = "unit")]
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

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_workflow_config_default() {
        let config = WorkflowConfig::default();
        assert_eq!(config.max_parallel, None);
        assert!(!config.continue_on_error);
        assert_eq!(config.timeout, None);
    }

    // Mock workflow implementation
    struct MockWorkflow {
        metadata: ComponentMetadata,
        config: WorkflowConfig,
        steps: Arc<Mutex<Vec<WorkflowStep>>>,
        status: Arc<Mutex<WorkflowStatus>>,
        results: Arc<Mutex<Vec<StepResult>>>,
    }

    impl MockWorkflow {
        fn new() -> Self {
            Self {
                metadata: ComponentMetadata::new(
                    "mock-workflow".to_string(),
                    "A mock workflow for testing".to_string(),
                ),
                config: WorkflowConfig::default(),
                steps: Arc::new(Mutex::new(Vec::new())),
                status: Arc::new(Mutex::new(WorkflowStatus::Pending)),
                results: Arc::new(Mutex::new(Vec::new())),
            }
        }
    }

    #[async_trait]
    impl BaseAgent for MockWorkflow {
        fn metadata(&self) -> &ComponentMetadata {
            &self.metadata
        }

        async fn execute(
            &self,
            _input: AgentInput,
            _context: ExecutionContext,
        ) -> Result<AgentOutput> {
            // Execute workflow
            *self.status.lock().await = WorkflowStatus::Running;

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

            *self.status.lock().await = WorkflowStatus::Completed;
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
            *self.status.lock().await = WorkflowStatus::Failed;
            Ok(AgentOutput::text(format!("Workflow error: {}", error)))
        }
    }

    #[async_trait]
    impl Workflow for MockWorkflow {
        fn config(&self) -> &WorkflowConfig {
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

        async fn status(&self) -> Result<WorkflowStatus> {
            Ok(self.status.lock().await.clone())
        }

        async fn get_results(&self) -> Result<Vec<StepResult>> {
            Ok(self.results.lock().await.clone())
        }
    }

    #[cfg_attr(test_category = "unit")]
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

    #[cfg_attr(test_category = "unit")]
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

    #[cfg_attr(test_category = "unit")]
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

    #[cfg_attr(test_category = "unit")]
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

    #[cfg_attr(test_category = "unit")]
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
        assert_eq!(workflow.status().await.unwrap(), WorkflowStatus::Completed);

        let results = workflow.get_results().await.unwrap();
        assert_eq!(results.len(), 1);
        assert!(results[0].success);

        // Get specific step result
        let step_result = workflow.get_step_result(step1.id).await.unwrap();
        assert!(step_result.is_some());
    }
}
