//! ABOUTME: Workflow traits for foundational workflow patterns
//! ABOUTME: Defines workflow interfaces for memory-based execution

use async_trait::async_trait;
use llmspell_core::{ComponentId, Result};
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Workflow step for workflow patterns.
///
/// Workflow step is designed for workflow patterns with memory-based state management.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowStep {
    /// Unique identifier for this step
    pub id: ComponentId,
    /// Human-readable name of the step
    pub name: String,
    /// Type of operation this step performs
    pub step_type: StepType,
    /// Optional timeout for step execution
    pub timeout: Option<Duration>,
    /// Number of retry attempts on failure
    pub retry_attempts: u32,
}

impl WorkflowStep {
    /// Create a new workflow step with the given name and type
    pub fn new(name: String, step_type: StepType) -> Self {
        Self {
            id: ComponentId::new(),
            name,
            step_type,
            timeout: None,
            retry_attempts: 0,
        }
    }

    /// Set the timeout for this step
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Set the number of retry attempts for this step
    pub fn with_retry(mut self, attempts: u32) -> Self {
        self.retry_attempts = attempts;
        self
    }
}

/// Types of workflow steps
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StepType {
    /// Execute a tool with given parameters
    Tool {
        /// Name of the tool to execute
        tool_name: String,
        /// Parameters to pass to the tool
        parameters: serde_json::Value,
    },
    /// Execute an agent with given input
    Agent {
        /// ID or name of the agent to execute
        // Changed from ComponentId to String to preserve original agent name for registry lookup
        agent_id: String,
        /// Input to pass to the agent
        input: String,
    },
    /// Execute a nested workflow
    Workflow {
        /// ID of the workflow to execute
        workflow_id: ComponentId,
        /// Input to pass to the workflow
        input: serde_json::Value,
    },
    /// Execute a template with given parameters
    Template {
        /// Template ID from registry (e.g., "research-assistant")
        template_id: String,
        /// Parameters to pass to the template
        params: serde_json::Value,
    },
}

/// Workflow execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepResult {
    /// ID of the step that produced this result
    pub step_id: ComponentId,
    /// Name of the step that produced this result
    pub step_name: String,
    /// Whether the step completed successfully
    pub success: bool,
    /// Output produced by the step
    pub output: String,
    /// Error message if the step failed
    pub error: Option<String>,
    /// How long the step took to execute
    pub duration: Duration,
    /// Number of times this step was retried
    pub retry_count: u32,
}

impl StepResult {
    /// Create a successful step result
    pub fn success(
        step_id: ComponentId,
        step_name: String,
        output: String,
        duration: Duration,
    ) -> Self {
        Self {
            step_id,
            step_name,
            success: true,
            output,
            error: None,
            duration,
            retry_count: 0,
        }
    }

    /// Create a failed step result
    pub fn failure(
        step_id: ComponentId,
        step_name: String,
        error: String,
        duration: Duration,
        retry_count: u32,
    ) -> Self {
        Self {
            step_id,
            step_name,
            success: false,
            output: String::new(),
            error: Some(error),
            duration,
            retry_count,
        }
    }
}

/// Workflow status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum WorkflowStatus {
    /// Workflow is queued but not started
    Pending,
    /// Workflow is currently executing
    Running,
    /// Workflow completed successfully
    Completed,
    /// Workflow failed
    Failed,
    /// Workflow was cancelled
    Cancelled,
    /// Some steps completed but workflow didn't finish
    PartiallyCompleted,
}

/// Error handling strategies for workflows
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ErrorStrategy {
    /// Stop execution on first error
    FailFast,
    /// Continue executing remaining steps
    Continue,
    /// Retry failed step with exponential backoff
    Retry {
        /// Maximum number of retry attempts
        max_attempts: u32,
        /// Initial backoff duration in milliseconds
        backoff_ms: u64,
    },
}

impl Default for ErrorStrategy {
    fn default() -> Self {
        Self::FailFast
    }
}

/// Workflow trait for workflow patterns.
///
/// This trait provides an interface for workflow execution with memory-based state management.
#[async_trait]
pub trait Workflow: Send + Sync {
    /// Get workflow name
    fn name(&self) -> &str;

    /// Get workflow status
    async fn status(&self) -> Result<WorkflowStatus>;

    /// Add a step to the workflow
    async fn add_step(&mut self, step: WorkflowStep) -> Result<()>;

    /// Remove a step from the workflow
    async fn remove_step(&mut self, step_id: ComponentId) -> Result<()>;

    /// Get all steps
    async fn get_steps(&self) -> Result<Vec<WorkflowStep>>;

    /// Execute the workflow
    async fn execute(&mut self) -> Result<Vec<StepResult>>;

    /// Get execution results
    async fn get_results(&self) -> Result<Vec<StepResult>>;

    /// Reset workflow to initial state
    async fn reset(&mut self) -> Result<()>;

    /// Validate workflow before execution
    async fn validate(&self) -> Result<()> {
        let steps = self.get_steps().await?;
        if steps.is_empty() {
            return Err(llmspell_core::LLMSpellError::Workflow {
                message: "Workflow has no steps".to_string(),
                step: None,
                source: None,
            });
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_step_type_template_serialization() {
        // Create a Template step type
        let step_type = StepType::Template {
            template_id: "research-assistant".to_string(),
            params: json!({
                "topic": "Rust async programming",
                "max_sources": 10,
                "session_id": "test-session-123"
            }),
        };

        // Serialize to JSON
        let serialized = serde_json::to_string(&step_type).expect("Failed to serialize");

        // Deserialize back
        let deserialized: StepType =
            serde_json::from_str(&serialized).expect("Failed to deserialize");

        // Verify it's a Template variant
        match deserialized {
            StepType::Template {
                template_id,
                params,
            } => {
                assert_eq!(template_id, "research-assistant");
                assert_eq!(params["topic"], "Rust async programming");
                assert_eq!(params["max_sources"], 10);
                assert_eq!(params["session_id"], "test-session-123");
            }
            _ => panic!("Expected Template variant"),
        }
    }

    #[test]
    fn test_workflow_step_with_template() {
        // Create a WorkflowStep with Template step type
        let step = WorkflowStep::new(
            "research".to_string(),
            StepType::Template {
                template_id: "research-assistant".to_string(),
                params: json!({"topic": "Rust"}),
            },
        );

        assert_eq!(step.name, "research");
        assert!(matches!(step.step_type, StepType::Template { .. }));

        // Verify serialization roundtrip
        let serialized = serde_json::to_string(&step).expect("Failed to serialize WorkflowStep");
        let deserialized: WorkflowStep =
            serde_json::from_str(&serialized).expect("Failed to deserialize WorkflowStep");

        assert_eq!(deserialized.name, "research");
        match deserialized.step_type {
            StepType::Template {
                template_id,
                params,
            } => {
                assert_eq!(template_id, "research-assistant");
                assert_eq!(params["topic"], "Rust");
            }
            _ => panic!("Expected Template variant"),
        }
    }
}
