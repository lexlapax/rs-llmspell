//! Comprehensive multi-tenancy isolation tests for SQLite backend (Phase 13c.2.10.4)
//!
//! Verifies tenant_id filtering works correctly across core storage layers:
//! - Agent state storage
//! - Workflow state storage
//! - KV store (generic key-value)
//!
//! Note:
//! - Vector storage tenant isolation is tested via MemoryManager in integration_sqlite_tests.rs
//! - Graph storage tenant isolation is tested in postgres_knowledge_graph_tests.rs::test_tenant_isolation

use chrono::Utc;
use llmspell_core::traits::storage::WorkflowStateStorage;
use llmspell_core::types::storage::{WorkflowState, WorkflowStatus};
use llmspell_storage::backends::sqlite::{
    SqliteAgentStateStorage, SqliteBackend, SqliteConfig, SqliteKVStorage,
    SqliteWorkflowStateStorage,
};
use llmspell_storage::StorageBackend;
use serde_json::json;
use std::sync::Arc;
use tempfile::TempDir;

/// Create test backend with migrations
async fn create_test_backend() -> (TempDir, Arc<SqliteBackend>) {
    let temp_dir = TempDir::new().expect("create temp dir");
    let db_path = temp_dir.path().join("multi_tenancy_test.db");

    let config = SqliteConfig::new(db_path.to_str().unwrap()).with_max_connections(10);
    let backend = Arc::new(SqliteBackend::new(config).await.expect("create backend"));

    backend.run_migrations().await.expect("run migrations");

    (temp_dir, backend)
}

#[tokio::test]
async fn test_comprehensive_multi_tenancy_isolation() {
    let (_temp_dir, backend) = create_test_backend().await;

    // Create storage instances for two tenants
    let tenant_a = "tenant-alpha";
    let tenant_b = "tenant-beta";

    // === 1. Agent State Storage ===
    let agent_a = SqliteAgentStateStorage::new(Arc::clone(&backend), tenant_a.to_string());
    let agent_b = SqliteAgentStateStorage::new(Arc::clone(&backend), tenant_b.to_string());

    // Save agent states for both tenants
    agent_a
        .set("agent:bot-1", b"{\"tenant\":\"A\"}".to_vec())
        .await
        .expect("set A");

    agent_b
        .set("agent:bot-1", b"{\"tenant\":\"B\"}".to_vec())
        .await
        .expect("set B");

    // Verify isolation
    let state_a = agent_a.get("agent:bot-1").await.expect("get A").unwrap();
    let state_b = agent_b.get("agent:bot-1").await.expect("get B").unwrap();

    assert_eq!(state_a, b"{\"tenant\":\"A\"}");
    assert_eq!(state_b, b"{\"tenant\":\"B\"}");

    // === 2. Workflow State Storage ===
    let workflow_a = SqliteWorkflowStateStorage::new(Arc::clone(&backend), tenant_a.to_string());
    let workflow_b = SqliteWorkflowStateStorage::new(Arc::clone(&backend), tenant_b.to_string());

    // Save workflow states for both tenants
    let wf_state_a = WorkflowState {
        workflow_id: "wf-1".to_string(),
        workflow_name: "Workflow A".to_string(),
        status: WorkflowStatus::Running,
        current_step: 1,
        state_data: json!({}),
        started_at: Some(Utc::now()),
        completed_at: None,
    };

    let wf_state_b = WorkflowState {
        workflow_id: "wf-1".to_string(),
        workflow_name: "Workflow B".to_string(),
        status: WorkflowStatus::Running,
        current_step: 2,
        state_data: json!({}),
        started_at: Some(Utc::now()),
        completed_at: None,
    };

    workflow_a
        .save_state("wf-1", &wf_state_a)
        .await
        .expect("save A");
    workflow_b
        .save_state("wf-1", &wf_state_b)
        .await
        .expect("save B");

    // Verify isolation
    let loaded_a = workflow_a
        .load_state("wf-1")
        .await
        .expect("load A")
        .unwrap();
    let loaded_b = workflow_b
        .load_state("wf-1")
        .await
        .expect("load B")
        .unwrap();

    assert_eq!(loaded_a.workflow_name, "Workflow A");
    assert_eq!(loaded_a.current_step, 1);
    assert_eq!(loaded_b.workflow_name, "Workflow B");
    assert_eq!(loaded_b.current_step, 2);

    // === 3. KV Store ===
    let kv_a = SqliteKVStorage::new(Arc::clone(&backend), tenant_a.to_string());
    let kv_b = SqliteKVStorage::new(Arc::clone(&backend), tenant_b.to_string());

    // Store values for both tenants
    kv_a.set("config", b"Config A".to_vec())
        .await
        .expect("set A");
    kv_b.set("config", b"Config B".to_vec())
        .await
        .expect("set B");

    // Verify isolation
    let val_a = kv_a.get("config").await.expect("get A").unwrap();
    let val_b = kv_b.get("config").await.expect("get B").unwrap();

    assert_eq!(val_a, b"Config A");
    assert_eq!(val_b, b"Config B");
}

#[tokio::test]
async fn test_cross_tenant_data_leak_prevention() {
    let (_temp_dir, backend) = create_test_backend().await;

    let tenant_a = "tenant-confidential";
    let tenant_b = "tenant-public";

    // Create KV stores for both tenants
    let kv_a = SqliteKVStorage::new(Arc::clone(&backend), tenant_a.to_string());
    let kv_b = SqliteKVStorage::new(Arc::clone(&backend), tenant_b.to_string());

    // Store sensitive data for tenant A
    kv_a.set("api_key", b"secret-key-12345".to_vec())
        .await
        .expect("store secret");

    // Tenant B should NOT be able to retrieve tenant A's data
    let leaked_data = kv_b.get("api_key").await.expect("query B");
    assert!(leaked_data.is_none(), "Cross-tenant data leak detected!");

    // Verify tenant A can still retrieve their own data
    let own_data = kv_a.get("api_key").await.expect("query A").unwrap();
    assert_eq!(own_data, b"secret-key-12345");
}

#[tokio::test]
async fn test_multi_tenant_list_operations() {
    let (_temp_dir, backend) = create_test_backend().await;

    let tenant_a = "tenant-list-a";
    let tenant_b = "tenant-list-b";

    // Create KV stores
    let kv_a = SqliteKVStorage::new(Arc::clone(&backend), tenant_a.to_string());
    let kv_b = SqliteKVStorage::new(Arc::clone(&backend), tenant_b.to_string());

    // Store multiple keys for both tenants
    for i in 1..=5 {
        kv_a.set(&format!("key-{}", i), format!("value-a-{}", i).into_bytes())
            .await
            .expect("set A");
        kv_b.set(&format!("key-{}", i), format!("value-b-{}", i).into_bytes())
            .await
            .expect("set B");
    }

    // List keys - each tenant should only see their own
    let keys_a = kv_a.list_keys("key-").await.expect("list A");
    let keys_b = kv_b.list_keys("key-").await.expect("list B");

    assert_eq!(keys_a.len(), 5);
    assert_eq!(keys_b.len(), 5);

    // Verify values are correct for each tenant
    for i in 1..=5 {
        let val_a = kv_a
            .get(&format!("key-{}", i))
            .await
            .expect("get A")
            .unwrap();
        let val_b = kv_b
            .get(&format!("key-{}", i))
            .await
            .expect("get B")
            .unwrap();

        assert_eq!(val_a, format!("value-a-{}", i).into_bytes());
        assert_eq!(val_b, format!("value-b-{}", i).into_bytes());
    }
}
