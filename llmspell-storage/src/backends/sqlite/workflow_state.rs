// ABOUTME: SQLite workflow state storage (Phase 13c.2.6)
//! ABOUTME: Storage layer for workflow execution state with lifecycle tracking

use super::backend::SqliteBackend;
use super::error::SqliteError;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use llmspell_core::traits::storage::WorkflowStateStorage;
use llmspell_core::types::storage::{WorkflowState, WorkflowStatus};
use std::sync::Arc;

/// SQLite-backed workflow state storage
///
/// Stores workflow execution states with:
/// - Tenant isolation via application-level filtering
/// - Lifecycle tracking (Pending → Running → Terminal)
/// - Automatic timestamp management via triggers
/// - Status transition validation
///
/// # Performance Target
/// <10ms for state save, <5ms for state load (Task 13c.2.6)
///
/// # Architecture
/// This implements the `WorkflowStateStorage` trait from llmspell-core,
/// using the V8 workflow_states table with lifecycle triggers.
#[derive(Clone)]
pub struct SqliteWorkflowStateStorage {
    backend: Arc<SqliteBackend>,
    tenant_id: String,
}

impl SqliteWorkflowStateStorage {
    /// Create new SQLite workflow state storage
    ///
    /// # Arguments
    /// * `backend` - SQLite backend with connection pool
    /// * `tenant_id` - Tenant identifier for isolation
    ///
    /// # Example
    /// ```rust,no_run
    /// use llmspell_storage::backends::sqlite::{SqliteBackend, SqliteConfig};
    /// use llmspell_storage::backends::sqlite::SqliteWorkflowStateStorage;
    /// use std::sync::Arc;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let config = SqliteConfig::new("./llmspell.db");
    /// let backend = Arc::new(SqliteBackend::new(config).await?);
    /// let storage = SqliteWorkflowStateStorage::new(backend, "tenant-123".to_string());
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(backend: Arc<SqliteBackend>, tenant_id: String) -> Self {
        Self { backend, tenant_id }
    }

    /// Get tenant ID for queries
    fn get_tenant_id(&self) -> &str {
        &self.tenant_id
    }

    /// Convert database timestamp (Unix epoch seconds) to DateTime<Utc>
    fn timestamp_to_datetime(timestamp: i64) -> DateTime<Utc> {
        DateTime::from_timestamp(timestamp, 0).unwrap_or_else(Utc::now)
    }

    /// Convert DateTime<Utc> to database timestamp (Unix epoch seconds)
    fn datetime_to_timestamp(dt: DateTime<Utc>) -> i64 {
        dt.timestamp()
    }
}

#[async_trait]
impl WorkflowStateStorage for SqliteWorkflowStateStorage {
    async fn save_state(
        &self,
        workflow_id: &str,
        state: &WorkflowState,
    ) -> anyhow::Result<()> {
        let tenant_id = self.get_tenant_id();
        let conn = self.backend.get_connection().await?;

        // Serialize state_data to JSON string
        let state_data_json = serde_json::to_string(&state.state_data)
            .map_err(|e| SqliteError::Query(format!("Failed to serialize state_data: {}", e)))?;

        // Convert timestamps
        let started_at = state.started_at.map(Self::datetime_to_timestamp);
        let completed_at = state.completed_at.map(Self::datetime_to_timestamp);
        let now = Utc::now().timestamp();

        // Use UPSERT to handle both insert and update
        let stmt = conn
            .prepare(
                "INSERT INTO workflow_states
                 (tenant_id, workflow_id, workflow_name, state_data, current_step, status, started_at, completed_at, last_updated, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
                 ON CONFLICT(tenant_id, workflow_id) DO UPDATE SET
                   workflow_name = excluded.workflow_name,
                   state_data = excluded.state_data,
                   current_step = excluded.current_step,
                   status = excluded.status,
                   started_at = COALESCE(excluded.started_at, workflow_states.started_at),
                   completed_at = COALESCE(excluded.completed_at, workflow_states.completed_at),
                   last_updated = excluded.last_updated",
            )
            .await
            .map_err(|e| SqliteError::Query(format!("Failed to prepare workflow state save: {}", e)))?;

        stmt.execute(libsql::params![
            tenant_id,
            workflow_id,
            state.workflow_name.clone(),
            state_data_json,
            state.current_step as i64,
            state.status.to_string(),
            started_at,
            completed_at,
            now,
            now
        ])
        .await
        .map_err(|e| SqliteError::Query(format!("Failed to save workflow state: {}", e)))?;

        Ok(())
    }

    async fn load_state(&self, workflow_id: &str) -> anyhow::Result<Option<WorkflowState>> {
        let tenant_id = self.get_tenant_id();
        let conn = self.backend.get_connection().await?;

        let stmt = conn
            .prepare(
                "SELECT workflow_name, state_data, current_step, status, started_at, completed_at
                 FROM workflow_states
                 WHERE tenant_id = ?1 AND workflow_id = ?2",
            )
            .await
            .map_err(|e| SqliteError::Query(format!("Failed to prepare workflow state load: {}", e)))?;

        let mut rows = stmt
            .query(libsql::params![tenant_id, workflow_id])
            .await
            .map_err(|e| SqliteError::Query(format!("Failed to load workflow state: {}", e)))?;

        let row = match rows.next().await {
            Ok(Some(row)) => row,
            Ok(None) => return Ok(None),
            Err(e) => return Err(SqliteError::Query(format!("Failed to fetch row: {}", e)).into()),
        };

        // Extract fields
        let workflow_name: String = row
            .get(0)
            .map_err(|e| SqliteError::Query(format!("Failed to get workflow_name: {}", e)))?;

        let state_data_json: String = row
            .get(1)
            .map_err(|e| SqliteError::Query(format!("Failed to get state_data: {}", e)))?;

        let current_step: i64 = row
            .get(2)
            .map_err(|e| SqliteError::Query(format!("Failed to get current_step: {}", e)))?;

        let status_str: String = row
            .get(3)
            .map_err(|e| SqliteError::Query(format!("Failed to get status: {}", e)))?;

        let started_at: Option<i64> = row
            .get(4)
            .map_err(|e| SqliteError::Query(format!("Failed to get started_at: {}", e)))?;

        let completed_at: Option<i64> = row
            .get(5)
            .map_err(|e| SqliteError::Query(format!("Failed to get completed_at: {}", e)))?;

        // Parse status
        let status = match status_str.as_str() {
            "pending" => WorkflowStatus::Pending,
            "running" => WorkflowStatus::Running,
            "completed" => WorkflowStatus::Completed,
            "failed" => WorkflowStatus::Failed,
            "cancelled" => WorkflowStatus::Cancelled,
            _ => {
                return Err(SqliteError::Query(format!("Invalid status: {}", status_str)).into())
            }
        };

        // Deserialize state_data
        let state_data: serde_json::Value = serde_json::from_str(&state_data_json)
            .map_err(|e| SqliteError::Query(format!("Failed to deserialize state_data: {}", e)))?;

        Ok(Some(WorkflowState {
            workflow_id: workflow_id.to_string(),
            workflow_name,
            status,
            current_step: current_step as usize,
            state_data,
            started_at: started_at.map(Self::timestamp_to_datetime),
            completed_at: completed_at.map(Self::timestamp_to_datetime),
        }))
    }

    async fn update_status(
        &self,
        workflow_id: &str,
        status: WorkflowStatus,
    ) -> anyhow::Result<()> {
        let tenant_id = self.get_tenant_id();
        let conn = self.backend.get_connection().await?;
        let now = Utc::now().timestamp();

        // Update status with lifecycle timestamp handling
        // Note: Triggers will automatically set started_at and completed_at
        let stmt = conn
            .prepare(
                "UPDATE workflow_states
                 SET status = ?1, last_updated = ?2
                 WHERE tenant_id = ?3 AND workflow_id = ?4",
            )
            .await
            .map_err(|e| SqliteError::Query(format!("Failed to prepare status update: {}", e)))?;

        let rows_affected = stmt
            .execute(libsql::params![status.to_string(), now, tenant_id, workflow_id])
            .await
            .map_err(|e| SqliteError::Query(format!("Failed to update workflow status: {}", e)))?;

        if rows_affected == 0 {
            return Err(SqliteError::Query(format!(
                "Workflow not found: {}",
                workflow_id
            ))
            .into());
        }

        Ok(())
    }

    async fn list_workflows(
        &self,
        status_filter: Option<WorkflowStatus>,
    ) -> anyhow::Result<Vec<String>> {
        let tenant_id = self.get_tenant_id();
        let conn = self.backend.get_connection().await?;

        let (query, params): (String, Vec<libsql::Value>) = match status_filter {
            Some(status) => (
                "SELECT workflow_id FROM workflow_states WHERE tenant_id = ?1 AND status = ?2 ORDER BY created_at DESC".to_string(),
                vec![
                    tenant_id.to_string().into(),
                    status.to_string().into(),
                ],
            ),
            None => (
                "SELECT workflow_id FROM workflow_states WHERE tenant_id = ?1 ORDER BY created_at DESC".to_string(),
                vec![tenant_id.to_string().into()],
            ),
        };

        let stmt = conn
            .prepare(&query)
            .await
            .map_err(|e| SqliteError::Query(format!("Failed to prepare list query: {}", e)))?;

        let mut rows = stmt
            .query(libsql::params_from_iter(params))
            .await
            .map_err(|e| SqliteError::Query(format!("Failed to list workflows: {}", e)))?;

        let mut workflow_ids = Vec::new();
        while let Some(row) = rows
            .next()
            .await
            .map_err(|e| SqliteError::Query(format!("Failed to fetch row: {}", e)))?
        {
            let workflow_id: String = row
                .get(0)
                .map_err(|e| SqliteError::Query(format!("Failed to get workflow_id: {}", e)))?;
            workflow_ids.push(workflow_id);
        }

        Ok(workflow_ids)
    }

    async fn delete_state(&self, workflow_id: &str) -> anyhow::Result<()> {
        let tenant_id = self.get_tenant_id();
        let conn = self.backend.get_connection().await?;

        let stmt = conn
            .prepare("DELETE FROM workflow_states WHERE tenant_id = ?1 AND workflow_id = ?2")
            .await
            .map_err(|e| SqliteError::Query(format!("Failed to prepare delete: {}", e)))?;

        stmt.execute(libsql::params![tenant_id, workflow_id])
            .await
            .map_err(|e| SqliteError::Query(format!("Failed to delete workflow state: {}", e)))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backends::sqlite::SqliteConfig;
    use tempfile::TempDir;

    async fn create_test_storage() -> (TempDir, Arc<SqliteBackend>, SqliteWorkflowStateStorage, String) {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let config = SqliteConfig::new(db_path.to_str().unwrap()).with_max_connections(5);
        let backend = Arc::new(SqliteBackend::new(config).await.unwrap());

        // Run migrations manually (V1, V8 for workflow_state tests)
        let conn = backend.get_connection().await.unwrap();

        // V1: Initial setup
        conn.execute_batch(include_str!(
            "../../../migrations/sqlite/V1__initial_setup.sql"
        ))
        .await
        .unwrap();

        // V8: Workflow states
        conn.execute_batch(include_str!(
            "../../../migrations/sqlite/V8__workflow_states.sql"
        ))
        .await
        .unwrap();

        // Create unique tenant ID
        let tenant_id = format!("test-tenant-{}", uuid::Uuid::new_v4());

        // Set tenant context
        backend.set_tenant_context(&tenant_id).await.unwrap();

        let storage = SqliteWorkflowStateStorage::new(
            Arc::clone(&backend),
            tenant_id.clone(),
        );

        (temp_dir, backend, storage, tenant_id)
    }

    #[tokio::test]
    async fn test_save_and_load_workflow_state() {
        let (_temp, _backend, storage, _tenant_id) = create_test_storage().await;

        // Create workflow state
        let state = WorkflowState::new("wf-123", "test-workflow");

        // Save state
        storage
            .save_state("wf-123", &state)
            .await
            .expect("Failed to save state");

        // Load state
        let loaded = storage
            .load_state("wf-123")
            .await
            .expect("Failed to load state")
            .expect("State not found");

        assert_eq!(loaded.workflow_id, "wf-123");
        assert_eq!(loaded.workflow_name, "test-workflow");
        assert_eq!(loaded.status, WorkflowStatus::Pending);
        assert_eq!(loaded.current_step, 0);
    }

    #[tokio::test]
    async fn test_update_workflow_status() {
        let (_temp, _backend, storage, _tenant_id) = create_test_storage().await;

        // Create and save workflow
        let state = WorkflowState::new("wf-456", "status-test");
        storage.save_state("wf-456", &state).await.unwrap();

        // Update to running
        storage
            .update_status("wf-456", WorkflowStatus::Running)
            .await
            .expect("Failed to update status");

        // Verify status changed
        let loaded = storage.load_state("wf-456").await.unwrap().unwrap();
        assert_eq!(loaded.status, WorkflowStatus::Running);

        // Update to completed
        storage
            .update_status("wf-456", WorkflowStatus::Completed)
            .await
            .unwrap();

        let loaded = storage.load_state("wf-456").await.unwrap().unwrap();
        assert_eq!(loaded.status, WorkflowStatus::Completed);
    }

    #[tokio::test]
    async fn test_list_workflows_with_filter() {
        let (_temp, _backend, storage, _tenant_id) = create_test_storage().await;

        // Create multiple workflows with different statuses
        let mut state1 = WorkflowState::new("wf-1", "workflow1");
        state1.status = WorkflowStatus::Pending;
        storage.save_state("wf-1", &state1).await.unwrap();

        let mut state2 = WorkflowState::new("wf-2", "workflow2");
        state2.status = WorkflowStatus::Running;
        storage.save_state("wf-2", &state2).await.unwrap();

        let mut state3 = WorkflowState::new("wf-3", "workflow3");
        state3.status = WorkflowStatus::Running;
        storage.save_state("wf-3", &state3).await.unwrap();

        // List all workflows
        let all = storage.list_workflows(None).await.unwrap();
        assert_eq!(all.len(), 3);

        // List only running workflows
        let running = storage
            .list_workflows(Some(WorkflowStatus::Running))
            .await
            .unwrap();
        assert_eq!(running.len(), 2);
        assert!(running.contains(&"wf-2".to_string()));
        assert!(running.contains(&"wf-3".to_string()));

        // List only pending workflows
        let pending = storage
            .list_workflows(Some(WorkflowStatus::Pending))
            .await
            .unwrap();
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0], "wf-1");
    }

    #[tokio::test]
    async fn test_delete_workflow_state() {
        let (_temp, _backend, storage, _tenant_id) = create_test_storage().await;

        // Create and save workflow
        let state = WorkflowState::new("wf-del", "delete-test");
        storage.save_state("wf-del", &state).await.unwrap();

        // Verify it exists
        assert!(storage.load_state("wf-del").await.unwrap().is_some());

        // Delete it
        storage.delete_state("wf-del").await.unwrap();

        // Verify it's gone
        assert!(storage.load_state("wf-del").await.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_workflow_state_upsert() {
        let (_temp, _backend, storage, _tenant_id) = create_test_storage().await;

        // Save initial state
        let state1 = WorkflowState::new("wf-up", "upsert-test");
        storage.save_state("wf-up", &state1).await.unwrap();

        // Update with new data
        let mut state2 = WorkflowState::new("wf-up", "upsert-test-updated");
        state2.current_step = 5;
        storage.save_state("wf-up", &state2).await.unwrap();

        // Verify updated
        let loaded = storage.load_state("wf-up").await.unwrap().unwrap();
        assert_eq!(loaded.workflow_name, "upsert-test-updated");
        assert_eq!(loaded.current_step, 5);

        // Verify only one record exists
        let all = storage.list_workflows(None).await.unwrap();
        assert_eq!(all.len(), 1);
    }

    #[tokio::test]
    async fn test_tenant_isolation() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let config = SqliteConfig::new(db_path.to_str().unwrap()).with_max_connections(5);
        let backend = Arc::new(SqliteBackend::new(config).await.unwrap());

        // Run migrations manually
        let conn = backend.get_connection().await.unwrap();
        conn.execute_batch(include_str!(
            "../../../migrations/sqlite/V1__initial_setup.sql"
        ))
        .await
        .unwrap();
        conn.execute_batch(include_str!(
            "../../../migrations/sqlite/V8__workflow_states.sql"
        ))
        .await
        .unwrap();

        // Create storage for two different tenants
        let storage1 = SqliteWorkflowStateStorage::new(Arc::clone(&backend), "tenant-1".to_string());
        let storage2 = SqliteWorkflowStateStorage::new(Arc::clone(&backend), "tenant-2".to_string());

        // Save workflow for tenant 1
        let state = WorkflowState::new("wf-shared", "shared-id");
        storage1.save_state("wf-shared", &state).await.unwrap();

        // Tenant 2 should not see tenant 1's workflow
        assert!(storage2.load_state("wf-shared").await.unwrap().is_none());

        // Tenant 1 should see their own workflow
        assert!(storage1.load_state("wf-shared").await.unwrap().is_some());
    }
}
