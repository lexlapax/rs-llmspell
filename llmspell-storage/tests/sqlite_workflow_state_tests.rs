//! Integration tests for SQLite workflow state storage (Phase 13c.2.10.3)
//!
//! Verifies:
//! - Workflow state save/load via SqliteWorkflowStateStorage
//! - Lifecycle tracking (Pending → Running → Completed/Failed)
//! - Tenant isolation
//! - Multi-workflow state management

use chrono::Utc;
use llmspell_core::traits::storage::WorkflowStateStorage;
use llmspell_core::types::storage::{WorkflowState, WorkflowStatus};
use llmspell_storage::backends::sqlite::{SqliteBackend, SqliteConfig, SqliteWorkflowStateStorage};
use serde_json::json;
use std::sync::Arc;
use tempfile::TempDir;

/// Create test storage with temporary database
async fn create_test_storage() -> (
    TempDir,
    Arc<SqliteBackend>,
    SqliteWorkflowStateStorage,
    String,
) {
    let temp_dir = TempDir::new().expect("create temp dir");
    let db_path = temp_dir.path().join("test_workflows.db");

    let config = SqliteConfig::new(db_path.to_str().unwrap()).with_max_connections(5);
    let backend = Arc::new(SqliteBackend::new(config).await.expect("create backend"));

    backend.run_migrations().await.expect("run migrations");

    let tenant_id = "test-tenant".to_string();
    let storage = SqliteWorkflowStateStorage::new(Arc::clone(&backend), tenant_id.clone());

    (temp_dir, backend, storage, tenant_id)
}

#[tokio::test]
async fn test_workflow_state_save_and_load() {
    let (_temp_dir, _backend, storage, _tenant_id) = create_test_storage().await;

    let workflow_id = "workflow-1";
    let state = WorkflowState {
        workflow_id: workflow_id.to_string(),
        workflow_name: "test_workflow".to_string(),
        state_data: json!({"step": 1, "data": "test"}),
        current_step: 1,
        status: WorkflowStatus::Running,
        started_at: Some(Utc::now()),
        completed_at: None,
    };

    // Save workflow state
    storage
        .save_state(workflow_id, &state)
        .await
        .expect("save state");

    // Load workflow state
    let loaded_state = storage
        .load_state(workflow_id)
        .await
        .expect("load state")
        .expect("state exists");

    assert_eq!(loaded_state.workflow_name, "test_workflow");
    assert_eq!(loaded_state.current_step, 1);
    assert_eq!(loaded_state.status, WorkflowStatus::Running);
    assert!(loaded_state.started_at.is_some());
    assert!(loaded_state.completed_at.is_none());
}

#[tokio::test]
async fn test_workflow_state_lifecycle_transitions() {
    let (_temp_dir, _backend, storage, _tenant_id) = create_test_storage().await;

    let workflow_id = "workflow-2";

    // Phase 1: Pending
    let state_pending = WorkflowState {
        workflow_id: workflow_id.to_string(),
        workflow_name: "lifecycle_test".to_string(),
        state_data: json!({"phase": "pending"}),
        current_step: 0,
        status: WorkflowStatus::Pending,
        started_at: None,
        completed_at: None,
    };

    storage
        .save_state(workflow_id, &state_pending)
        .await
        .expect("save pending");

    // Phase 2: Running
    let state_running = WorkflowState {
        workflow_id: workflow_id.to_string(),
        workflow_name: "lifecycle_test".to_string(),
        state_data: json!({"phase": "running"}),
        current_step: 1,
        status: WorkflowStatus::Running,
        started_at: Some(Utc::now()),
        completed_at: None,
    };

    storage
        .save_state(workflow_id, &state_running)
        .await
        .expect("save running");

    // Phase 3: Completed
    let state_completed = WorkflowState {
        workflow_id: workflow_id.to_string(),
        workflow_name: "lifecycle_test".to_string(),
        state_data: json!({"phase": "completed"}),
        current_step: 5,
        status: WorkflowStatus::Completed,
        started_at: state_running.started_at,
        completed_at: Some(Utc::now()),
    };

    storage
        .save_state(workflow_id, &state_completed)
        .await
        .expect("save completed");

    // Verify final state
    let loaded_state = storage
        .load_state(workflow_id)
        .await
        .expect("load state")
        .expect("state exists");

    assert_eq!(loaded_state.status, WorkflowStatus::Completed);
    assert_eq!(loaded_state.current_step, 5);
    assert!(loaded_state.started_at.is_some());
    assert!(loaded_state.completed_at.is_some());
}

#[tokio::test]
async fn test_workflow_state_tenant_isolation() {
    let temp_dir = TempDir::new().expect("create temp dir");
    let db_path = temp_dir.path().join("test_isolation.db");

    let config = SqliteConfig::new(db_path.to_str().unwrap()).with_max_connections(5);
    let backend = Arc::new(SqliteBackend::new(config).await.expect("create backend"));
    backend.run_migrations().await.expect("run migrations");

    // Create storage for tenant A
    let storage_a = SqliteWorkflowStateStorage::new(Arc::clone(&backend), "tenant-a".to_string());

    // Create storage for tenant B
    let storage_b = SqliteWorkflowStateStorage::new(Arc::clone(&backend), "tenant-b".to_string());

    let workflow_id = "shared-workflow-id";

    // Save state for tenant A
    let state_a = WorkflowState {
        workflow_id: workflow_id.to_string(),
        workflow_name: "tenant_a_workflow".to_string(),
        state_data: json!({"tenant": "A"}),
        current_step: 1,
        status: WorkflowStatus::Running,
        started_at: Some(Utc::now()),
        completed_at: None,
    };

    storage_a
        .save_state(workflow_id, &state_a)
        .await
        .expect("save A");

    // Save state for tenant B
    let state_b = WorkflowState {
        workflow_id: workflow_id.to_string(),
        workflow_name: "tenant_b_workflow".to_string(),
        state_data: json!({"tenant": "B"}),
        current_step: 2,
        status: WorkflowStatus::Running,
        started_at: Some(Utc::now()),
        completed_at: None,
    };

    storage_b
        .save_state(workflow_id, &state_b)
        .await
        .expect("save B");

    // Verify tenant A sees only their state
    let loaded_a = storage_a
        .load_state(workflow_id)
        .await
        .expect("load A")
        .expect("A exists");
    assert_eq!(loaded_a.workflow_name, "tenant_a_workflow");
    assert_eq!(loaded_a.current_step, 1);

    // Verify tenant B sees only their state
    let loaded_b = storage_b
        .load_state(workflow_id)
        .await
        .expect("load B")
        .expect("B exists");
    assert_eq!(loaded_b.workflow_name, "tenant_b_workflow");
    assert_eq!(loaded_b.current_step, 2);
}

#[tokio::test]
async fn test_workflow_state_delete() {
    let (_temp_dir, _backend, storage, _tenant_id) = create_test_storage().await;

    let workflow_id = "workflow-3";
    let state = WorkflowState {
        workflow_id: workflow_id.to_string(),
        workflow_name: "to_delete".to_string(),
        state_data: json!({"delete_me": true}),
        current_step: 0,
        status: WorkflowStatus::Pending,
        started_at: None,
        completed_at: None,
    };

    // Save state
    storage
        .save_state(workflow_id, &state)
        .await
        .expect("save state");

    // Verify exists
    assert!(storage
        .load_state(workflow_id)
        .await
        .expect("load")
        .is_some());

    // Delete state
    storage
        .delete_state(workflow_id)
        .await
        .expect("delete state");

    // Verify deleted
    assert!(storage
        .load_state(workflow_id)
        .await
        .expect("load")
        .is_none());
}

#[tokio::test]
async fn test_multi_workflow_state_management() {
    let (_temp_dir, _backend, storage, _tenant_id) = create_test_storage().await;

    // Save states for multiple workflows
    for i in 1..=5 {
        let workflow_id = format!("workflow-{}", i);
        let state = WorkflowState {
            workflow_id: workflow_id.clone(),
            workflow_name: format!("workflow_{}", i),
            state_data: json!({"workflow_num": i}),
            current_step: i,
            status: if i % 2 == 0 {
                WorkflowStatus::Completed
            } else {
                WorkflowStatus::Running
            },
            started_at: Some(Utc::now()),
            completed_at: if i % 2 == 0 { Some(Utc::now()) } else { None },
        };

        storage
            .save_state(&workflow_id, &state)
            .await
            .expect("save state");
    }

    // Verify all workflows have their states
    for i in 1..=5 {
        let workflow_id = format!("workflow-{}", i);
        let loaded_state = storage
            .load_state(&workflow_id)
            .await
            .expect("load state")
            .expect("state exists");

        assert_eq!(loaded_state.workflow_name, format!("workflow_{}", i));
        assert_eq!(loaded_state.current_step, i);
    }
}

#[tokio::test]
async fn test_workflow_state_nonexistent() {
    let (_temp_dir, _backend, storage, _tenant_id) = create_test_storage().await;

    // Try to load nonexistent workflow
    let result = storage
        .load_state("nonexistent-workflow")
        .await
        .expect("load should succeed");

    assert!(result.is_none());
}

#[tokio::test]
async fn test_workflow_state_update_progress() {
    let (_temp_dir, _backend, storage, _tenant_id) = create_test_storage().await;

    let workflow_id = "workflow-4";
    let started_at = Utc::now();

    // Save initial state
    let mut state = WorkflowState {
        workflow_id: workflow_id.to_string(),
        workflow_name: "progress_test".to_string(),
        state_data: json!({"progress": 0}),
        current_step: 0,
        status: WorkflowStatus::Running,
        started_at: Some(started_at),
        completed_at: None,
    };

    storage
        .save_state(workflow_id, &state)
        .await
        .expect("save initial");

    // Update progress through steps
    for step in 1..=5 {
        state.current_step = step;
        state.state_data = json!({"progress": step * 20});

        storage
            .save_state(workflow_id, &state)
            .await
            .expect("save progress");
    }

    // Mark as completed
    state.status = WorkflowStatus::Completed;
    state.completed_at = Some(Utc::now());

    storage
        .save_state(workflow_id, &state)
        .await
        .expect("save completed");

    // Verify final state
    let loaded_state = storage
        .load_state(workflow_id)
        .await
        .expect("load state")
        .expect("state exists");

    assert_eq!(loaded_state.current_step, 5);
    assert_eq!(loaded_state.status, WorkflowStatus::Completed);
    assert_eq!(loaded_state.state_data["progress"], 100);
    assert!(loaded_state.started_at.is_some());
    assert!(loaded_state.completed_at.is_some());
}

#[tokio::test]
async fn test_workflow_state_failure_handling() {
    let (_temp_dir, _backend, storage, _tenant_id) = create_test_storage().await;

    let workflow_id = "workflow-5";
    let started_at = Utc::now();

    // Start workflow
    let mut state = WorkflowState {
        workflow_id: workflow_id.to_string(),
        workflow_name: "failure_test".to_string(),
        state_data: json!({"step": 1}),
        current_step: 1,
        status: WorkflowStatus::Running,
        started_at: Some(started_at),
        completed_at: None,
    };

    storage
        .save_state(workflow_id, &state)
        .await
        .expect("save running");

    // Simulate failure
    state.status = WorkflowStatus::Failed;
    state.state_data = json!({"error": "Simulated failure", "failed_at_step": 1});
    state.completed_at = Some(Utc::now());

    storage
        .save_state(workflow_id, &state)
        .await
        .expect("save failed");

    // Verify failed state
    let loaded_state = storage
        .load_state(workflow_id)
        .await
        .expect("load state")
        .expect("state exists");

    assert_eq!(loaded_state.status, WorkflowStatus::Failed);
    assert_eq!(loaded_state.state_data["error"], "Simulated failure");
    assert!(loaded_state.completed_at.is_some());
}
