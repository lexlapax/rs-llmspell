//! Workflow state storage types
//!
//! Types for persistent workflow execution state with lifecycle tracking.
//! Supports workflow checkpointing, status updates, and resumption.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Workflow execution status
///
/// Tracks the lifecycle of a workflow from pending to terminal states.
///
/// # State Transitions
///
/// ```text
/// Pending → Running → Completed
///                  ↘ Failed
///                  ↘ Cancelled
/// ```
///
/// # Examples
///
/// ```
/// use llmspell_core::types::storage::WorkflowStatus;
///
/// let status = WorkflowStatus::Pending;
/// assert_eq!(status.is_terminal(), false);
///
/// let status = WorkflowStatus::Completed;
/// assert_eq!(status.is_terminal(), true);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkflowStatus {
    /// Workflow created but not yet started
    Pending,
    /// Workflow is actively executing
    Running,
    /// Workflow completed successfully
    Completed,
    /// Workflow failed with error
    Failed,
    /// Workflow was cancelled by user/system
    Cancelled,
}

impl WorkflowStatus {
    /// Check if status is terminal (cannot transition further)
    ///
    /// # Returns
    ///
    /// `true` for Completed, Failed, or Cancelled; `false` for Pending or Running
    #[must_use]
    pub const fn is_terminal(self) -> bool {
        matches!(self, Self::Completed | Self::Failed | Self::Cancelled)
    }

    /// Check if workflow is active (pending or running)
    #[must_use]
    pub const fn is_active(self) -> bool {
        matches!(self, Self::Pending | Self::Running)
    }
}

impl std::fmt::Display for WorkflowStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pending => write!(f, "pending"),
            Self::Running => write!(f, "running"),
            Self::Completed => write!(f, "completed"),
            Self::Failed => write!(f, "failed"),
            Self::Cancelled => write!(f, "cancelled"),
        }
    }
}

/// Persistent workflow execution state
///
/// Stores complete workflow state for checkpointing and resumption.
/// The `state_data` field contains workflow-specific state as JSON.
///
/// # Examples
///
/// ```
/// use llmspell_core::types::storage::{WorkflowState, WorkflowStatus};
/// use serde_json::json;
///
/// let state = WorkflowState {
///     workflow_id: "wf-123".to_string(),
///     workflow_name: "data-processing".to_string(),
///     status: WorkflowStatus::Running,
///     current_step: 2,
///     state_data: json!({"input": "data.csv", "processed_rows": 1000}),
///     started_at: Some(chrono::Utc::now()),
///     completed_at: None,
/// };
///
/// assert_eq!(state.status, WorkflowStatus::Running);
/// assert_eq!(state.current_step, 2);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowState {
    /// Unique workflow identifier
    pub workflow_id: String,

    /// Human-readable workflow name
    pub workflow_name: String,

    /// Current execution status
    pub status: WorkflowStatus,

    /// Current step index (0-based)
    pub current_step: usize,

    /// Workflow-specific state data (stored as JSONB in database)
    ///
    /// This field contains all workflow-specific context needed for resumption,
    /// such as input parameters, intermediate results, and execution metadata.
    pub state_data: serde_json::Value,

    /// Timestamp when workflow execution started
    ///
    /// Set when transitioning from Pending → Running
    pub started_at: Option<DateTime<Utc>>,

    /// Timestamp when workflow reached terminal state
    ///
    /// Set when transitioning to Completed, Failed, or Cancelled
    pub completed_at: Option<DateTime<Utc>>,
}

impl WorkflowState {
    /// Create a new workflow state in Pending status
    ///
    /// # Arguments
    ///
    /// * `workflow_id` - Unique identifier for the workflow
    /// * `workflow_name` - Human-readable name
    ///
    /// # Examples
    ///
    /// ```
    /// use llmspell_core::types::storage::WorkflowState;
    ///
    /// let state = WorkflowState::new("wf-123", "my-workflow");
    /// assert_eq!(state.workflow_id, "wf-123");
    /// assert_eq!(state.current_step, 0);
    /// ```
    #[must_use]
    pub fn new(workflow_id: impl Into<String>, workflow_name: impl Into<String>) -> Self {
        Self {
            workflow_id: workflow_id.into(),
            workflow_name: workflow_name.into(),
            status: WorkflowStatus::Pending,
            current_step: 0,
            state_data: serde_json::Value::Object(serde_json::Map::new()),
            started_at: None,
            completed_at: None,
        }
    }

    /// Update workflow status and set appropriate timestamps
    ///
    /// Automatically sets `started_at` when transitioning to Running,
    /// and `completed_at` when reaching terminal state.
    pub fn set_status(&mut self, new_status: WorkflowStatus) {
        let now = Utc::now();

        // Set started_at when transitioning to Running
        if new_status == WorkflowStatus::Running && self.status == WorkflowStatus::Pending {
            self.started_at = Some(now);
        }

        // Set completed_at when reaching terminal state
        if new_status.is_terminal() && !self.status.is_terminal() {
            self.completed_at = Some(now);
        }

        self.status = new_status;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workflow_status_transitions() {
        assert!(WorkflowStatus::Completed.is_terminal());
        assert!(WorkflowStatus::Failed.is_terminal());
        assert!(WorkflowStatus::Cancelled.is_terminal());
        assert!(!WorkflowStatus::Pending.is_terminal());
        assert!(!WorkflowStatus::Running.is_terminal());
    }

    #[test]
    fn test_workflow_status_active() {
        assert!(WorkflowStatus::Pending.is_active());
        assert!(WorkflowStatus::Running.is_active());
        assert!(!WorkflowStatus::Completed.is_active());
    }

    #[test]
    fn test_workflow_state_new() {
        let state = WorkflowState::new("wf-1", "test");
        assert_eq!(state.status, WorkflowStatus::Pending);
        assert_eq!(state.current_step, 0);
        assert!(state.started_at.is_none());
        assert!(state.completed_at.is_none());
    }

    #[test]
    fn test_workflow_state_status_transitions() {
        let mut state = WorkflowState::new("wf-1", "test");

        // Pending → Running should set started_at
        state.set_status(WorkflowStatus::Running);
        assert!(state.started_at.is_some());
        assert!(state.completed_at.is_none());

        // Running → Completed should set completed_at
        state.set_status(WorkflowStatus::Completed);
        assert!(state.started_at.is_some());
        assert!(state.completed_at.is_some());
    }
}
