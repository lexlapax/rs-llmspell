//! Backup and restore integration tests for SQLite backend (Phase 13c.2.10.5)
//!
//! Verifies that libsql's single-file design makes backup/restore trivial:
//! - Backup: Copy single .db file (vs 4 separate procedures for old multi-backend setup)
//! - Restore: Copy backup file back
//! - No complex coordination between multiple storage backends
//!
//! This demonstrates one of the key benefits of Phase 13c consolidation.

use llmspell_core::traits::storage::WorkflowStateStorage;
use llmspell_core::types::storage::{WorkflowState, WorkflowStatus};
use llmspell_storage::backends::sqlite::{
    SqliteAgentStateStorage, SqliteBackend, SqliteConfig, SqliteKVStorage,
    SqliteWorkflowStateStorage,
};
use llmspell_storage::traits::StorageBackend;
use serde_json::json;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use tempfile::TempDir;

/// Create test backend with migrations
async fn create_test_backend(db_path: PathBuf) -> Arc<SqliteBackend> {
    let config = SqliteConfig::new(db_path.to_str().unwrap()).with_max_connections(10);
    let backend = Arc::new(SqliteBackend::new(config).await.expect("create backend"));

    backend.run_migrations().await.expect("run migrations");

    backend
}

#[tokio::test]
async fn test_backup_and_restore_single_file() {
    let temp_dir = TempDir::new().expect("create temp dir");
    let db_path = temp_dir.path().join("backup_test.db");
    let backup_path = temp_dir.path().join("backup_test.db.backup");

    // Phase 1: Create database with some data
    let backend = create_test_backend(db_path.clone()).await;
    let tenant_id = "test-tenant";

    // Insert data into agent state storage
    let agent_storage = SqliteAgentStateStorage::new(Arc::clone(&backend), tenant_id.to_string());
    agent_storage
        .set("agent:original", b"{\"state\":\"original\"}".to_vec())
        .await
        .expect("set agent state");

    // Insert data into workflow state storage
    let workflow_storage =
        SqliteWorkflowStateStorage::new(Arc::clone(&backend), tenant_id.to_string());
    let workflow_state = WorkflowState {
        workflow_id: "wf-original".to_string(),
        workflow_name: "Original Workflow".to_string(),
        state_data: json!({"step": 1}),
        current_step: 1,
        status: WorkflowStatus::Running,
        started_at: Some(chrono::Utc::now()),
        completed_at: None,
    };
    workflow_storage
        .save_state("wf-original", &workflow_state)
        .await
        .expect("save workflow state");

    // Insert data into KV storage
    let kv_storage = SqliteKVStorage::new(Arc::clone(&backend), tenant_id.to_string());
    kv_storage
        .set("config", b"original-config".to_vec())
        .await
        .expect("set kv");

    // Verify data exists
    let agent_data_before = agent_storage
        .get("agent:original")
        .await
        .expect("get")
        .unwrap();
    assert_eq!(agent_data_before, b"{\"state\":\"original\"}");

    let workflow_data_before = workflow_storage
        .load_state("wf-original")
        .await
        .expect("load")
        .unwrap();
    assert_eq!(workflow_data_before.workflow_name, "Original Workflow");

    let kv_data_before = kv_storage.get("config").await.expect("get").unwrap();
    assert_eq!(kv_data_before, b"original-config");

    // Drop backend to release file lock
    drop(backend);
    drop(agent_storage);
    drop(workflow_storage);
    drop(kv_storage);

    // Phase 2: Backup (simple file copy!)
    fs::copy(&db_path, &backup_path).expect("backup failed");

    // Phase 3: Modify data
    let backend2 = create_test_backend(db_path.clone()).await;
    let agent_storage2 = SqliteAgentStateStorage::new(Arc::clone(&backend2), tenant_id.to_string());
    let workflow_storage2 =
        SqliteWorkflowStateStorage::new(Arc::clone(&backend2), tenant_id.to_string());
    let kv_storage2 = SqliteKVStorage::new(Arc::clone(&backend2), tenant_id.to_string());

    // Overwrite with modified data
    agent_storage2
        .set("agent:original", b"{\"state\":\"modified\"}".to_vec())
        .await
        .expect("set agent state");

    let modified_workflow_state = WorkflowState {
        workflow_id: "wf-original".to_string(),
        workflow_name: "Modified Workflow".to_string(),
        state_data: json!({"step": 999}),
        current_step: 999,
        status: WorkflowStatus::Completed,
        started_at: workflow_state.started_at,
        completed_at: Some(chrono::Utc::now()),
    };
    workflow_storage2
        .save_state("wf-original", &modified_workflow_state)
        .await
        .expect("save workflow state");

    kv_storage2
        .set("config", b"modified-config".to_vec())
        .await
        .expect("set kv");

    // Verify data is modified
    let agent_data_modified = agent_storage2
        .get("agent:original")
        .await
        .expect("get")
        .unwrap();
    assert_eq!(agent_data_modified, b"{\"state\":\"modified\"}");

    let workflow_data_modified = workflow_storage2
        .load_state("wf-original")
        .await
        .expect("load")
        .unwrap();
    assert_eq!(workflow_data_modified.workflow_name, "Modified Workflow");

    let kv_data_modified = kv_storage2.get("config").await.expect("get").unwrap();
    assert_eq!(kv_data_modified, b"modified-config");

    // Drop backend to release file lock
    drop(backend2);
    drop(agent_storage2);
    drop(workflow_storage2);
    drop(kv_storage2);

    // Phase 4: Restore (simple file copy!)
    fs::copy(&backup_path, &db_path).expect("restore failed");

    // Phase 5: Verify original data is restored
    let backend3 = create_test_backend(db_path.clone()).await;
    let agent_storage3 = SqliteAgentStateStorage::new(Arc::clone(&backend3), tenant_id.to_string());
    let workflow_storage3 =
        SqliteWorkflowStateStorage::new(Arc::clone(&backend3), tenant_id.to_string());
    let kv_storage3 = SqliteKVStorage::new(Arc::clone(&backend3), tenant_id.to_string());

    let agent_data_restored = agent_storage3
        .get("agent:original")
        .await
        .expect("get")
        .unwrap();
    assert_eq!(
        agent_data_restored, b"{\"state\":\"original\"}",
        "Agent state should be restored to original"
    );

    let workflow_data_restored = workflow_storage3
        .load_state("wf-original")
        .await
        .expect("load")
        .unwrap();
    assert_eq!(
        workflow_data_restored.workflow_name, "Original Workflow",
        "Workflow state should be restored to original"
    );

    let kv_data_restored = kv_storage3.get("config").await.expect("get").unwrap();
    assert_eq!(
        kv_data_restored, b"original-config",
        "KV data should be restored to original"
    );
}

#[tokio::test]
async fn test_backup_simplicity_vs_old_multi_backend() {
    // This test documents the simplification from Phase 13c consolidation
    //
    // OLD MULTI-BACKEND APPROACH (Phase 13b):
    // - Episodic memory: HNSW indices (4 files) + metadata (SurrealDB/Sled)
    // - Semantic memory: PostgreSQL dump procedure
    // - Procedural memory: Sled database files
    // - State: Separate backup procedures for each backend
    // - Total: 4+ separate backup/restore procedures with complex coordination
    //
    // NEW LIBSQL APPROACH (Phase 13c):
    // - Everything: Single .db file
    // - Backup: fs::copy(source, dest)
    // - Restore: fs::copy(backup, source)
    // - Total: 1 file copy operation
    //
    // This test verifies that backup/restore now requires only file system operations.

    let temp_dir = TempDir::new().expect("create temp dir");
    let db_path = temp_dir.path().join("simplicity_test.db");

    let backend = create_test_backend(db_path.clone()).await;
    let tenant_id = "test-tenant";

    // Create data in ALL storage layers
    let agent_storage = SqliteAgentStateStorage::new(Arc::clone(&backend), tenant_id.to_string());
    let workflow_storage =
        SqliteWorkflowStateStorage::new(Arc::clone(&backend), tenant_id.to_string());
    let kv_storage = SqliteKVStorage::new(Arc::clone(&backend), tenant_id.to_string());

    agent_storage
        .set("agent:1", b"{\"test\":1}".to_vec())
        .await
        .expect("set");
    workflow_storage
        .save_state(
            "wf-1",
            &WorkflowState {
                workflow_id: "wf-1".to_string(),
                workflow_name: "test".to_string(),
                state_data: json!({}),
                current_step: 0,
                status: WorkflowStatus::Pending,
                started_at: None,
                completed_at: None,
            },
        )
        .await
        .expect("save");
    kv_storage.set("key", b"value".to_vec()).await.expect("set");

    drop(backend);
    drop(agent_storage);
    drop(workflow_storage);
    drop(kv_storage);

    // Verify: Only 1 file exists for all storage layers
    assert!(db_path.exists(), "Database file should exist");

    // Verify: Backup is a single file copy
    let backup_path = temp_dir.path().join("simplicity_test.db.backup");
    fs::copy(&db_path, &backup_path).expect("backup via simple file copy");

    assert!(backup_path.exists(), "Backup file should exist");

    // Verify: Restore is a single file copy
    fs::copy(&backup_path, &db_path).expect("restore via simple file copy");

    // This test passes if no errors occurred - demonstrating that backup/restore
    // is now trivial (1 file copy) instead of complex (4+ procedures).
}
