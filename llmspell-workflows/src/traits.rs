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
    pub id: ComponentId,
    pub name: String,
    pub step_type: StepType,
    pub timeout: Option<Duration>,
    pub retry_attempts: u32,
}

impl WorkflowStep {
    pub fn new(name: String, step_type: StepType) -> Self {
        Self {
            id: ComponentId::new(),
            name,
            step_type,
            timeout: None,
            retry_attempts: 0,
        }
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

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
        tool_name: String,
        parameters: serde_json::Value,
    },
    /// Execute an agent with given input
    Agent {
        agent_id: ComponentId,
        input: String,
    },
    /// Custom function execution
    Custom {
        function_name: String,
        parameters: serde_json::Value,
    },
    /// Execute a nested workflow
    Workflow {
        workflow_id: ComponentId,
        input: serde_json::Value,
    },
}

/// Workflow execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepResult {
    pub step_id: ComponentId,
    pub step_name: String,
    pub success: bool,
    pub output: String,
    pub error: Option<String>,
    pub duration: Duration,
    pub retry_count: u32,
}

impl StepResult {
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
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

/// Error handling strategies for workflows
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ErrorStrategy {
    /// Stop execution on first error
    FailFast,
    /// Continue executing remaining steps
    Continue,
    /// Retry failed step with exponential backoff
    Retry { max_attempts: u32, backoff_ms: u64 },
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
