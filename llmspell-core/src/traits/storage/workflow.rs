//! Workflow state persistence trait
//!
//! Provides trait abstraction for workflow state storage with lifecycle tracking.

use crate::types::storage::{WorkflowState, WorkflowStatus};
use anyhow::Result;
use async_trait::async_trait;

/// Workflow state persistence trait
///
/// Manages persistent storage for workflow execution state with lifecycle tracking.
/// Supports workflow checkpointing, status updates, and resumption across restarts.
///
/// # Lifecycle
///
/// Workflows transition through states: Pending → Running → Terminal (Completed/Failed/Cancelled).
/// The trait provides methods to save complete state, load for resumption, and update status.
///
/// # Implementation Notes
///
/// - Implementations should ensure atomic updates for status transitions
/// - State data is stored as JSON for flexibility
/// - Workflow IDs must be unique within a tenant
/// - Terminal workflows can be cleaned up via `delete_state()`
///
/// # Examples
///
/// ```no_run
/// use llmspell_core::traits::storage::WorkflowStateStorage;
/// use llmspell_core::types::storage::{WorkflowState, WorkflowStatus};
/// # use anyhow::Result;
///
/// # async fn example(storage: &dyn WorkflowStateStorage) -> Result<()> {
/// // Create and save new workflow
/// let mut state = WorkflowState::new("wf-123", "data-processing");
/// storage.save_state(&state.workflow_id, &state).await?;
///
/// // Update status to running
/// storage.update_status("wf-123", WorkflowStatus::Running).await?;
///
/// // Load state for resumption
/// let loaded = storage.load_state("wf-123").await?;
/// assert!(loaded.is_some());
///
/// // List all running workflows
/// let running = storage.list_workflows(Some(WorkflowStatus::Running)).await?;
/// # Ok(())
/// # }
/// ```
#[async_trait]
pub trait WorkflowStateStorage: Send + Sync {
    /// Save complete workflow state
    ///
    /// Stores or updates the full workflow state including current step,
    /// status, and workflow-specific data. Use this for checkpointing.
    ///
    /// # Arguments
    ///
    /// * `workflow_id` - Unique workflow identifier
    /// * `state` - Complete workflow state to persist
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, `Err` if storage operation fails
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use llmspell_core::traits::storage::WorkflowStateStorage;
    /// # use llmspell_core::types::storage::WorkflowState;
    /// # use anyhow::Result;
    /// # async fn example(storage: &dyn WorkflowStateStorage) -> Result<()> {
    /// let state = WorkflowState::new("wf-123", "my-workflow");
    /// storage.save_state(&state.workflow_id, &state).await?;
    /// # Ok(())
    /// # }
    /// ```
    async fn save_state(&self, workflow_id: &str, state: &WorkflowState) -> Result<()>;

    /// Load workflow state by ID
    ///
    /// Retrieves the complete workflow state for resumption or inspection.
    /// Returns `None` if workflow ID not found.
    ///
    /// # Arguments
    ///
    /// * `workflow_id` - Unique workflow identifier
    ///
    /// # Returns
    ///
    /// - `Ok(Some(state))` if workflow found
    /// - `Ok(None)` if workflow not found
    /// - `Err` if storage operation fails
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use llmspell_core::traits::storage::WorkflowStateStorage;
    /// # use anyhow::Result;
    /// # async fn example(storage: &dyn WorkflowStateStorage) -> Result<()> {
    /// match storage.load_state("wf-123").await? {
    ///     Some(state) => println!("Found workflow at step {}", state.current_step),
    ///     None => println!("Workflow not found"),
    /// }
    /// # Ok(())
    /// # }
    /// ```
    async fn load_state(&self, workflow_id: &str) -> Result<Option<WorkflowState>>;

    /// Update workflow status
    ///
    /// Updates only the status field of a workflow. More efficient than
    /// full state save when only status changes. Automatically sets
    /// `started_at` and `completed_at` timestamps.
    ///
    /// # Arguments
    ///
    /// * `workflow_id` - Unique workflow identifier
    /// * `status` - New status to set
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, `Err` if workflow not found or update fails
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use llmspell_core::traits::storage::WorkflowStateStorage;
    /// # use llmspell_core::types::storage::WorkflowStatus;
    /// # use anyhow::Result;
    /// # async fn example(storage: &dyn WorkflowStateStorage) -> Result<()> {
    /// // Mark workflow as running
    /// storage.update_status("wf-123", WorkflowStatus::Running).await?;
    ///
    /// // Mark as completed
    /// storage.update_status("wf-123", WorkflowStatus::Completed).await?;
    /// # Ok(())
    /// # }
    /// ```
    async fn update_status(&self, workflow_id: &str, status: WorkflowStatus) -> Result<()>;

    /// List workflows matching optional status filter
    ///
    /// Returns workflow IDs for all workflows, or only those matching
    /// the specified status filter.
    ///
    /// # Arguments
    ///
    /// * `status_filter` - Optional status to filter by (None = all workflows)
    ///
    /// # Returns
    ///
    /// Vector of workflow IDs matching the filter
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use llmspell_core::traits::storage::WorkflowStateStorage;
    /// # use llmspell_core::types::storage::WorkflowStatus;
    /// # use anyhow::Result;
    /// # async fn example(storage: &dyn WorkflowStateStorage) -> Result<()> {
    /// // Get all workflows
    /// let all = storage.list_workflows(None).await?;
    ///
    /// // Get only running workflows
    /// let running = storage.list_workflows(Some(WorkflowStatus::Running)).await?;
    /// # Ok(())
    /// # }
    /// ```
    async fn list_workflows(&self, status_filter: Option<WorkflowStatus>) -> Result<Vec<String>>;

    /// Delete workflow state
    ///
    /// Removes workflow state from storage. Typically used for cleanup
    /// after terminal workflows (Completed/Failed/Cancelled).
    ///
    /// # Arguments
    ///
    /// * `workflow_id` - Unique workflow identifier to delete
    ///
    /// # Returns
    ///
    /// `Ok(())` on success (even if workflow not found), `Err` if delete fails
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use llmspell_core::traits::storage::WorkflowStateStorage;
    /// # use anyhow::Result;
    /// # async fn example(storage: &dyn WorkflowStateStorage) -> Result<()> {
    /// // Clean up completed workflow
    /// storage.delete_state("wf-123").await?;
    /// # Ok(())
    /// # }
    /// ```
    async fn delete_state(&self, workflow_id: &str) -> Result<()>;
}
