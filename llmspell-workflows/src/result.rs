//! ABOUTME: Unified workflow result structure for state-based outputs
//! ABOUTME: Provides consistent metadata-only results across all workflow types

use crate::traits::WorkflowStatus;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::time::Duration;

/// Workflow-specific error types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkflowError {
    /// A step in the workflow failed to execute
    StepExecutionFailed {
        /// Name of the step that failed
        step_name: String,
        /// Reason for the failure
        reason: String,
    },
    /// The workflow timed out
    Timeout {
        /// Duration before timeout occurred
        duration: Duration,
        /// Descriptive timeout message
        message: String,
    },
    /// A condition evaluation failed
    ConditionFailed {
        /// Name of the condition that failed
        condition_name: String,
        /// Error details from condition evaluation
        error: String,
    },
    /// State access failed
    StateAccessFailed {
        /// State operation that failed (e.g., "read", "write")
        operation: String,
        /// Error details from state access
        error: String,
    },
    /// Configuration error
    ConfigurationError {
        /// Configuration error message
        message: String,
    },
    /// General workflow error
    General {
        /// General error message
        message: String,
    },
}

impl fmt::Display for WorkflowError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WorkflowError::StepExecutionFailed { step_name, reason } => {
                write!(f, "Step '{}' failed: {}", step_name, reason)
            }
            WorkflowError::Timeout { duration, message } => {
                write!(f, "Workflow timed out after {:?}: {}", duration, message)
            }
            WorkflowError::ConditionFailed {
                condition_name,
                error,
            } => {
                write!(f, "Condition '{}' failed: {}", condition_name, error)
            }
            WorkflowError::StateAccessFailed { operation, error } => {
                write!(f, "State operation '{}' failed: {}", operation, error)
            }
            WorkflowError::ConfigurationError { message } => {
                write!(f, "Configuration error: {}", message)
            }
            WorkflowError::General { message } => {
                write!(f, "Workflow error: {}", message)
            }
        }
    }
}

impl std::error::Error for WorkflowError {}

/// Workflow type identifier for result categorization
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkflowType {
    /// Sequential workflow execution
    Sequential,
    /// Parallel workflow execution
    Parallel,
    /// Conditional workflow execution
    Conditional,
    /// Loop workflow execution
    Loop,
    /// Custom workflow type with name
    Custom(String),
}

impl std::fmt::Display for WorkflowType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WorkflowType::Sequential => write!(f, "sequential"),
            WorkflowType::Parallel => write!(f, "parallel"),
            WorkflowType::Conditional => write!(f, "conditional"),
            WorkflowType::Loop => write!(f, "loop"),
            WorkflowType::Custom(name) => write!(f, "custom:{}", name),
        }
    }
}

/// Unified workflow result containing only execution metadata
///
/// This structure represents the outcome of any workflow execution.
/// Actual data outputs are written to state during execution, and this
/// result contains the keys where outputs can be found.
///
/// # Design Philosophy
///
/// Following patterns from Google ADK, Temporal, and Airflow:
/// - Workflows are data processors, not data containers
/// - State is the primary data bus between components
/// - Results contain metadata for tracking and debugging
/// - Memory efficiency through state-based storage
///
/// # Example
///
/// ```ignore
/// let result = workflow.execute(input, context).await?;
///
/// if result.success {
///     // Access outputs from state using the provided keys
///     for key in &result.state_keys {
///         let output = context.state.read(key).await?;
///         // Process the output
///     }
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowResult {
    /// Unique execution ID for this workflow run
    pub execution_id: String,

    /// Type of workflow that was executed
    pub workflow_type: WorkflowType,

    /// Name of the workflow instance
    pub workflow_name: String,

    /// Overall success status
    pub success: bool,

    /// Current workflow status
    pub status: WorkflowStatus,

    /// Human-readable summary of the execution
    pub summary: String,

    /// State keys where outputs were written
    /// Format: "workflow:{execution_id}:{step_name}"
    pub state_keys: Vec<String>,

    /// Number of steps successfully executed
    pub steps_executed: usize,

    /// Number of steps that failed
    pub steps_failed: usize,

    /// Number of steps that were skipped
    pub steps_skipped: usize,

    /// Total execution duration
    pub duration: Duration,

    /// Error information if the workflow failed
    pub error: Option<WorkflowError>,

    /// Additional metadata for debugging
    #[serde(default)]
    pub metadata: serde_json::Map<String, serde_json::Value>,
}

impl WorkflowResult {
    /// Create a new successful workflow result
    pub fn success(
        execution_id: String,
        workflow_type: WorkflowType,
        workflow_name: String,
        state_keys: Vec<String>,
        steps_executed: usize,
        duration: Duration,
    ) -> Self {
        Self {
            execution_id,
            workflow_type: workflow_type.clone(),
            workflow_name: workflow_name.clone(),
            success: true,
            status: WorkflowStatus::Completed,
            summary: format!(
                "{} workflow '{}' completed successfully with {} steps",
                workflow_type, workflow_name, steps_executed
            ),
            state_keys,
            steps_executed,
            steps_failed: 0,
            steps_skipped: 0,
            duration,
            error: None,
            metadata: serde_json::Map::new(),
        }
    }

    /// Create a new failed workflow result
    #[allow(clippy::too_many_arguments)]
    pub fn failure(
        execution_id: String,
        workflow_type: WorkflowType,
        workflow_name: String,
        error: WorkflowError,
        state_keys: Vec<String>,
        steps_executed: usize,
        steps_failed: usize,
        duration: Duration,
    ) -> Self {
        Self {
            execution_id,
            workflow_type: workflow_type.clone(),
            workflow_name: workflow_name.clone(),
            success: false,
            status: WorkflowStatus::Failed,
            summary: format!(
                "{} workflow '{}' failed: {}",
                workflow_type, workflow_name, error
            ),
            state_keys,
            steps_executed,
            steps_failed,
            steps_skipped: 0,
            duration,
            error: Some(error),
            metadata: serde_json::Map::new(),
        }
    }

    /// Create a partially successful result
    #[allow(clippy::too_many_arguments)]
    pub fn partial(
        execution_id: String,
        workflow_type: WorkflowType,
        workflow_name: String,
        state_keys: Vec<String>,
        steps_executed: usize,
        steps_failed: usize,
        steps_skipped: usize,
        duration: Duration,
        error: Option<WorkflowError>,
    ) -> Self {
        let success = steps_failed == 0;
        let status = if success {
            WorkflowStatus::Completed
        } else if steps_executed > 0 {
            WorkflowStatus::PartiallyCompleted
        } else {
            WorkflowStatus::Failed
        };

        let summary = if success {
            format!(
                "{} workflow '{}' completed with {} steps executed, {} skipped",
                workflow_type, workflow_name, steps_executed, steps_skipped
            )
        } else {
            format!(
                "{} workflow '{}' partially completed: {} executed, {} failed, {} skipped",
                workflow_type, workflow_name, steps_executed, steps_failed, steps_skipped
            )
        };

        Self {
            execution_id,
            workflow_type,
            workflow_name,
            success,
            status,
            summary,
            state_keys,
            steps_executed,
            steps_failed,
            steps_skipped,
            duration,
            error,
            metadata: serde_json::Map::new(),
        }
    }

    /// Add metadata to the result
    pub fn with_metadata(mut self, key: String, value: serde_json::Value) -> Self {
        self.metadata.insert(key, value);
        self
    }

    /// Generate a state key for a workflow step
    ///
    /// Standard format: "workflow:{execution_id}:{step_name}"
    pub fn generate_state_key(execution_id: &str, step_name: &str) -> String {
        format!("workflow:{}:{}", execution_id, step_name)
    }

    /// Generate a state key for aggregated results
    ///
    /// Standard format: "workflow:{execution_id}:aggregated"
    pub fn generate_aggregated_key(execution_id: &str) -> String {
        format!("workflow:{}:aggregated", execution_id)
    }

    /// Generate a state key for a parallel branch
    ///
    /// Standard format: "workflow:{execution_id}:branch_{branch_name}:{step_name}"
    pub fn generate_branch_key(execution_id: &str, branch_name: &str, step_name: &str) -> String {
        format!(
            "workflow:{}:branch_{}:{}",
            execution_id, branch_name, step_name
        )
    }

    /// Generate a state key for a loop iteration
    ///
    /// Standard format: "workflow:{execution_id}:iteration_{n}:{step_name}"
    pub fn generate_iteration_key(execution_id: &str, iteration: usize, step_name: &str) -> String {
        format!(
            "workflow:{}:iteration_{}:{}",
            execution_id, iteration, step_name
        )
    }

    /// Get agent outputs collected during workflow execution
    ///
    /// Returns a reference to the collected agent outputs if any agents were executed.
    /// The map is keyed by agent ID and contains the JSON output from each agent execution.
    ///
    /// Agent outputs are automatically collected by all workflow types (Sequential, Parallel,
    /// Loop, Conditional) when agents are executed within the workflow and state is available.
    ///
    /// # Returns
    /// - `Some(&Map)` if agent outputs were collected during execution
    /// - `None` if no agents were executed, state was unavailable, or outputs weren't collected
    ///
    /// # Example
    /// ```ignore
    /// let result = workflow.execute(input, context).await?;
    /// if let Some(outputs) = result.agent_outputs() {
    ///     for (agent_id, output) in outputs {
    ///         println!("Agent {}: {:?}", agent_id, output);
    ///     }
    /// }
    /// ```
    pub fn agent_outputs(&self) -> Option<&serde_json::Map<String, serde_json::Value>> {
        self.metadata
            .get("agent_outputs")
            .and_then(|v| v.as_object())
    }

    /// Get output from a specific agent by ID
    ///
    /// Convenience method to retrieve output from a single agent without iterating
    /// through all outputs. This is useful when you know which agent's output you need.
    ///
    /// # Arguments
    /// * `agent_id` - The agent ID to look up (as specified in workflow step configuration)
    ///
    /// # Returns
    /// - `Some(&Value)` if the agent output exists
    /// - `None` if agent not found, no outputs collected, or state was unavailable
    ///
    /// # Example
    /// ```ignore
    /// let result = workflow.execute(input, context).await?;
    /// if let Some(output) = result.get_agent_output("requirements_analyst") {
    ///     let text = output.as_str().unwrap_or("N/A");
    ///     println!("Requirements: {}", text);
    /// }
    /// ```
    pub fn get_agent_output(&self, agent_id: &str) -> Option<&serde_json::Value> {
        self.agent_outputs()
            .and_then(|outputs| outputs.get(agent_id))
    }
}

/// Extension trait for backwards compatibility during migration
pub trait WorkflowResultExt {
    /// Convert legacy result to unified WorkflowResult
    fn to_unified_result(self) -> WorkflowResult;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workflow_result_success() {
        let result = WorkflowResult::success(
            "exec-123".to_string(),
            WorkflowType::Sequential,
            "data-pipeline".to_string(),
            vec!["workflow:exec-123:step1".to_string()],
            5,
            Duration::from_secs(10),
        );

        assert!(result.success);
        assert_eq!(result.status, WorkflowStatus::Completed);
        assert_eq!(result.steps_executed, 5);
        assert_eq!(result.steps_failed, 0);
        assert!(result.error.is_none());
    }

    #[test]
    fn test_workflow_result_failure() {
        let error = WorkflowError::StepExecutionFailed {
            step_name: "transform".to_string(),
            reason: "Invalid data format".to_string(),
        };

        let result = WorkflowResult::failure(
            "exec-456".to_string(),
            WorkflowType::Parallel,
            "batch-processor".to_string(),
            error.clone(),
            vec![],
            2,
            1,
            Duration::from_secs(5),
        );

        assert!(!result.success);
        assert_eq!(result.status, WorkflowStatus::Failed);
        assert_eq!(result.steps_executed, 2);
        assert_eq!(result.steps_failed, 1);
        assert!(result.error.is_some());
    }

    #[test]
    fn test_state_key_generation() {
        assert_eq!(
            WorkflowResult::generate_state_key("exec-123", "step1"),
            "workflow:exec-123:step1"
        );

        assert_eq!(
            WorkflowResult::generate_aggregated_key("exec-123"),
            "workflow:exec-123:aggregated"
        );

        assert_eq!(
            WorkflowResult::generate_branch_key("exec-123", "branch_a", "transform"),
            "workflow:exec-123:branch_branch_a:transform"
        );

        assert_eq!(
            WorkflowResult::generate_iteration_key("exec-123", 5, "process"),
            "workflow:exec-123:iteration_5:process"
        );
    }

    #[test]
    fn test_workflow_type_display() {
        assert_eq!(WorkflowType::Sequential.to_string(), "sequential");
        assert_eq!(WorkflowType::Parallel.to_string(), "parallel");
        assert_eq!(WorkflowType::Conditional.to_string(), "conditional");
        assert_eq!(WorkflowType::Loop.to_string(), "loop");
        assert_eq!(
            WorkflowType::Custom("etl".to_string()).to_string(),
            "custom:etl"
        );
    }

    #[test]
    fn test_agent_outputs_none_when_not_present() {
        let result = WorkflowResult::success(
            "exec-789".to_string(),
            WorkflowType::Sequential,
            "test-workflow".to_string(),
            vec![],
            3,
            Duration::from_secs(5),
        );

        assert!(result.agent_outputs().is_none());
    }

    #[test]
    fn test_agent_outputs_some_when_present() {
        let mut result = WorkflowResult::success(
            "exec-789".to_string(),
            WorkflowType::Sequential,
            "test-workflow".to_string(),
            vec![],
            3,
            Duration::from_secs(5),
        );

        // Simulate agent outputs being added (as done by workflow execution)
        let mut agent_outputs = serde_json::Map::new();
        agent_outputs.insert("agent1".to_string(), serde_json::json!({"text": "output1"}));
        agent_outputs.insert("agent2".to_string(), serde_json::json!({"text": "output2"}));
        result.metadata.insert(
            "agent_outputs".to_string(),
            serde_json::Value::Object(agent_outputs),
        );

        let outputs = result.agent_outputs();
        assert!(outputs.is_some());
        assert_eq!(outputs.unwrap().len(), 2);
        assert!(outputs.unwrap().contains_key("agent1"));
        assert!(outputs.unwrap().contains_key("agent2"));
    }

    #[test]
    fn test_get_agent_output_none_when_no_outputs() {
        let result = WorkflowResult::success(
            "exec-789".to_string(),
            WorkflowType::Sequential,
            "test-workflow".to_string(),
            vec![],
            3,
            Duration::from_secs(5),
        );

        assert!(result.get_agent_output("agent1").is_none());
    }

    #[test]
    fn test_get_agent_output_none_when_agent_not_found() {
        let mut result = WorkflowResult::success(
            "exec-789".to_string(),
            WorkflowType::Sequential,
            "test-workflow".to_string(),
            vec![],
            3,
            Duration::from_secs(5),
        );

        // Add agent outputs but not the one we're looking for
        let mut agent_outputs = serde_json::Map::new();
        agent_outputs.insert("agent1".to_string(), serde_json::json!({"text": "output1"}));
        result.metadata.insert(
            "agent_outputs".to_string(),
            serde_json::Value::Object(agent_outputs),
        );

        assert!(result.get_agent_output("agent2").is_none());
    }

    #[test]
    fn test_get_agent_output_some_when_found() {
        let mut result = WorkflowResult::success(
            "exec-789".to_string(),
            WorkflowType::Sequential,
            "test-workflow".to_string(),
            vec![],
            3,
            Duration::from_secs(5),
        );

        // Add agent outputs
        let mut agent_outputs = serde_json::Map::new();
        agent_outputs.insert(
            "requirements_agent".to_string(),
            serde_json::json!({"text": "Requirements gathered successfully"}),
        );
        agent_outputs.insert(
            "design_agent".to_string(),
            serde_json::json!({"text": "Design completed"}),
        );
        result.metadata.insert(
            "agent_outputs".to_string(),
            serde_json::Value::Object(agent_outputs),
        );

        let output = result.get_agent_output("requirements_agent");
        assert!(output.is_some());
        assert_eq!(
            output.unwrap().get("text").unwrap().as_str().unwrap(),
            "Requirements gathered successfully"
        );

        let output2 = result.get_agent_output("design_agent");
        assert!(output2.is_some());
        assert_eq!(
            output2.unwrap().get("text").unwrap().as_str().unwrap(),
            "Design completed"
        );
    }
}
