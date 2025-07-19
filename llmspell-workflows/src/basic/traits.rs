//! ABOUTME: Basic workflow traits for foundational workflow patterns
//! ABOUTME: Defines simplified workflow interfaces for memory-based execution

use async_trait::async_trait;
use llmspell_core::{ComponentId, Result};
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Basic workflow step for simplified workflow patterns.
///
/// Unlike the full WorkflowStep from core, BasicWorkflowStep is designed
/// for simple sequential patterns without complex dependency management.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BasicWorkflowStep {
    pub id: ComponentId,
    pub name: String,
    pub step_type: BasicStepType,
    pub timeout: Option<Duration>,
    pub retry_attempts: u32,
}

impl BasicWorkflowStep {
    pub fn new(name: String, step_type: BasicStepType) -> Self {
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

/// Types of basic workflow steps
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BasicStepType {
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
}

/// Basic workflow execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BasicStepResult {
    pub step_id: ComponentId,
    pub step_name: String,
    pub success: bool,
    pub output: String,
    pub error: Option<String>,
    pub duration: Duration,
    pub retry_count: u32,
}

impl BasicStepResult {
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

/// Basic workflow status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum BasicWorkflowStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

/// Error handling strategies for basic workflows
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BasicErrorStrategy {
    /// Stop execution on first error
    FailFast,
    /// Continue executing remaining steps
    Continue,
    /// Retry failed step with exponential backoff
    Retry { max_attempts: u32, backoff_ms: u64 },
}

impl Default for BasicErrorStrategy {
    fn default() -> Self {
        Self::FailFast
    }
}

/// Basic workflow trait for simple workflow patterns.
///
/// This trait provides a simplified interface for basic workflow execution
/// without the complexity of dependency management and advanced features.
/// It's designed for memory-based state management and sequential execution.
#[async_trait]
pub trait BasicWorkflow: Send + Sync {
    /// Get workflow name
    fn name(&self) -> &str;

    /// Get workflow status
    async fn status(&self) -> Result<BasicWorkflowStatus>;

    /// Add a step to the workflow
    async fn add_step(&mut self, step: BasicWorkflowStep) -> Result<()>;

    /// Remove a step from the workflow
    async fn remove_step(&mut self, step_id: ComponentId) -> Result<()>;

    /// Get all steps
    async fn get_steps(&self) -> Result<Vec<BasicWorkflowStep>>;

    /// Execute the workflow
    async fn execute(&mut self) -> Result<Vec<BasicStepResult>>;

    /// Get execution results
    async fn get_results(&self) -> Result<Vec<BasicStepResult>>;

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
