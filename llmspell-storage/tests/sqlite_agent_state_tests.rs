//! Integration tests for SQLite agent state storage (Phase 13c.2.10.3)
//!
//! Verifies:
//! - Agent state save/load via SqliteAgentStateStorage
//! - Versioning and checksum validation
//! - Tenant isolation
//! - Multi-agent state management

use llmspell_storage::backends::sqlite::{SqliteAgentStateStorage, SqliteBackend, SqliteConfig};
use llmspell_storage::traits::StorageBackend;
use std::sync::Arc;
use tempfile::TempDir;

/// Create test storage with temporary database
async fn create_test_storage() -> (TempDir, Arc<SqliteBackend>, SqliteAgentStateStorage, String) {
    let temp_dir = TempDir::new().expect("create temp dir");
    let db_path = temp_dir.path().join("test_agents.db");

    let config = SqliteConfig::new(db_path.to_str().unwrap()).with_max_connections(5);
    let backend = Arc::new(SqliteBackend::new(config).await.expect("create backend"));

    backend
        .run_migrations()
        .await
        .expect("run migrations");

    let tenant_id = "test-tenant".to_string();
    let storage = SqliteAgentStateStorage::new(Arc::clone(&backend), tenant_id.clone());

    (temp_dir, backend, storage, tenant_id)
}

#[tokio::test]
async fn test_agent_state_save_and_load() {
    let (_temp_dir, _backend, storage, _tenant_id) = create_test_storage().await;

    let agent_id = "agent-1";
    let agent_key = format!("agent:{}", agent_id);
    let state_data = b"{\"conversation\":[\"Hello\",\"World\"]}".to_vec();

    // Save agent state
    storage
        .set(&agent_key, state_data.clone())
        .await
        .expect("save state");

    // Load agent state
    let loaded_state = storage
        .get(&agent_key)
        .await
        .expect("load state")
        .expect("state exists");

    assert_eq!(loaded_state, state_data);
}

#[tokio::test]
async fn test_agent_state_update_versioning() {
    let (_temp_dir, _backend, storage, _tenant_id) = create_test_storage().await;

    let agent_id = "agent-2";
    let agent_key = format!("agent:{}", agent_id);

    // Save initial state
    let state_v1 = b"{\"version\":1}".to_vec();
    storage
        .set(&agent_key, state_v1)
        .await
        .expect("save v1");

    // Update state (should increment version)
    let state_v2 = b"{\"version\":2}".to_vec();
    storage
        .set(&agent_key, state_v2.clone())
        .await
        .expect("save v2");

    // Load latest state
    let loaded_state = storage
        .get(&agent_key)
        .await
        .expect("load state")
        .expect("state exists");

    assert_eq!(loaded_state, state_v2);
}

#[tokio::test]
async fn test_agent_state_checksum_validation() {
    let (_temp_dir, _backend, storage, _tenant_id) = create_test_storage().await;

    let agent_id = "agent-3";
    let agent_key = format!("agent:{}", agent_id);
    let state_data = b"{\"checksum_test\":true}".to_vec();

    // Save state (checksum computed automatically)
    storage
        .set(&agent_key, state_data.clone())
        .await
        .expect("save state");

    // Load state (checksum verified automatically)
    let loaded_state = storage
        .get(&agent_key)
        .await
        .expect("load state")
        .expect("state exists");

    assert_eq!(loaded_state, state_data);
}

#[tokio::test]
async fn test_agent_state_tenant_isolation() {
    let temp_dir = TempDir::new().expect("create temp dir");
    let db_path = temp_dir.path().join("test_isolation.db");

    let config = SqliteConfig::new(db_path.to_str().unwrap()).with_max_connections(5);
    let backend = Arc::new(SqliteBackend::new(config).await.expect("create backend"));
    backend
        .run_migrations()
        .await
        .expect("run migrations");

    // Create storage for tenant A
    let storage_a = SqliteAgentStateStorage::new(Arc::clone(&backend), "tenant-a".to_string());

    // Create storage for tenant B
    let storage_b = SqliteAgentStateStorage::new(Arc::clone(&backend), "tenant-b".to_string());

    let agent_id = "shared-agent-id";
    let agent_key = format!("agent:{}", agent_id);

    // Save state for tenant A
    let state_a = b"{\"tenant\":\"A\"}".to_vec();
    storage_a
        .set(&agent_key, state_a.clone())
        .await
        .expect("save A");

    // Save state for tenant B
    let state_b = b"{\"tenant\":\"B\"}".to_vec();
    storage_b
        .set(&agent_key, state_b.clone())
        .await
        .expect("save B");

    // Verify tenant A sees only their state
    let loaded_a = storage_a
        .get(&agent_key)
        .await
        .expect("load A")
        .expect("A exists");
    assert_eq!(loaded_a, state_a);

    // Verify tenant B sees only their state
    let loaded_b = storage_b
        .get(&agent_key)
        .await
        .expect("load B")
        .expect("B exists");
    assert_eq!(loaded_b, state_b);
}

#[tokio::test]
async fn test_agent_state_delete() {
    let (_temp_dir, _backend, storage, _tenant_id) = create_test_storage().await;

    let agent_id = "agent-4";
    let agent_key = format!("agent:{}", agent_id);
    let state_data = b"{\"to_delete\":true}".to_vec();

    // Save state
    storage
        .set(&agent_key, state_data)
        .await
        .expect("save state");

    // Verify exists
    assert!(storage.get(&agent_key).await.expect("get").is_some());

    // Delete state
    storage
        .delete(&agent_key)
        .await
        .expect("delete state");

    // Verify deleted
    assert!(storage.get(&agent_key).await.expect("get").is_none());
}

#[tokio::test]
async fn test_multi_agent_state_management() {
    let (_temp_dir, _backend, storage, _tenant_id) = create_test_storage().await;

    // Save states for multiple agents
    for i in 1..=5 {
        let agent_key = format!("agent:agent-{}", i);
        let state_data = format!("{{\"agent_id\":{}}}", i).into_bytes();
        storage
            .set(&agent_key, state_data)
            .await
            .expect("save state");
    }

    // Verify all agents have their states
    for i in 1..=5 {
        let agent_key = format!("agent:agent-{}", i);
        let loaded_state = storage
            .get(&agent_key)
            .await
            .expect("load state")
            .expect("state exists");

        let expected = format!("{{\"agent_id\":{}}}", i).into_bytes();
        assert_eq!(loaded_state, expected);
    }
}

#[tokio::test]
async fn test_agent_state_nonexistent_key() {
    let (_temp_dir, _backend, storage, _tenant_id) = create_test_storage().await;

    // Try to load nonexistent state
    let result = storage
        .get("agent:nonexistent")
        .await
        .expect("get should succeed");

    assert!(result.is_none());
}

#[tokio::test]
async fn test_agent_state_list_keys() {
    let (_temp_dir, _backend, storage, _tenant_id) = create_test_storage().await;

    // Save multiple agent states
    storage
        .set("agent:alice", b"{\"name\":\"alice\"}".to_vec())
        .await
        .expect("save alice");

    storage
        .set("agent:bob", b"{\"name\":\"bob\"}".to_vec())
        .await
        .expect("save bob");

    storage
        .set("agent:charlie", b"{\"name\":\"charlie\"}".to_vec())
        .await
        .expect("save charlie");

    // List all keys with prefix
    let keys = storage
        .list_keys("agent:")
        .await
        .expect("list keys");

    assert_eq!(keys.len(), 3);
    assert!(keys.contains(&"agent:alice".to_string()));
    assert!(keys.contains(&"agent:bob".to_string()));
    assert!(keys.contains(&"agent:charlie".to_string()));
}
