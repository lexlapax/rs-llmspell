//! State tracing instrumentation tests

use anyhow::Result;
use llmspell_kernel::state::{
    manager::StateManager,
    persistence::{FilePersistence, KernelStateSnapshot, MemoryPersistence, StatePersistence},
    types::{DebugState, ExecutionState, SessionState},
    StateScope,
};
use serde_json::json;
use tempfile::TempDir;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

/// Initialize test tracing subscriber
fn init_test_tracing() {
    let _ = tracing_subscriber::registry()
        .with(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("debug,llmspell_kernel=trace")),
        )
        .with(fmt::layer().with_test_writer())
        .try_init();
}

#[tokio::test]
async fn test_persistence_operations_tracing() -> Result<()> {
    init_test_tracing();

    // Test file persistence
    let temp_dir = TempDir::new()?;
    let file_persistence = FilePersistence::new(temp_dir.path().to_path_buf())?;

    let snapshot = KernelStateSnapshot::new(
        ExecutionState::default(),
        SessionState::default(),
        DebugState::default(),
    );

    // Test save_state tracing
    tracing::info_span!("test_save_state")
        .in_scope(|| async {
            file_persistence.save_state(&snapshot).await.unwrap();
        })
        .await;

    // Test load_state tracing
    tracing::info_span!("test_load_state")
        .in_scope(|| async {
            let loaded = file_persistence.load_state().await.unwrap();
            assert!(loaded.is_some());
        })
        .await;

    // Test state_exists tracing
    tracing::info_span!("test_state_exists")
        .in_scope(|| async {
            let exists = file_persistence.state_exists().await.unwrap();
            assert!(exists);
        })
        .await;

    // Test save_snapshot tracing
    tracing::info_span!("test_save_snapshot")
        .in_scope(|| async {
            file_persistence
                .save_snapshot("test_snapshot", &snapshot)
                .await
                .unwrap();
        })
        .await;

    // Test load_snapshot tracing
    tracing::info_span!("test_load_snapshot")
        .in_scope(|| async {
            let loaded = file_persistence
                .load_snapshot("test_snapshot")
                .await
                .unwrap();
            assert!(loaded.is_some());
        })
        .await;

    // Test list_snapshots tracing
    tracing::info_span!("test_list_snapshots")
        .in_scope(|| async {
            let snapshots = file_persistence.list_snapshots().await.unwrap();
            assert_eq!(snapshots.len(), 1);
        })
        .await;

    // Test delete_state tracing
    tracing::info_span!("test_delete_state")
        .in_scope(|| async {
            file_persistence.delete_state().await.unwrap();
        })
        .await;

    Ok(())
}

#[tokio::test]
async fn test_memory_persistence_tracing() -> Result<()> {
    init_test_tracing();

    let memory_persistence = MemoryPersistence::new();

    let snapshot = KernelStateSnapshot::new(
        ExecutionState::default(),
        SessionState::default(),
        DebugState::default(),
    );

    // Test all memory persistence operations with tracing
    tracing::info_span!("memory_persistence_ops")
        .in_scope(|| async {
            memory_persistence.save_state(&snapshot).await.unwrap();
            assert!(memory_persistence.state_exists().await.unwrap());

            let loaded = memory_persistence.load_state().await.unwrap();
            assert!(loaded.is_some());

            memory_persistence
                .save_snapshot("snapshot1", &snapshot)
                .await
                .unwrap();
            memory_persistence
                .save_snapshot("snapshot2", &snapshot)
                .await
                .unwrap();

            let snapshots = memory_persistence.list_snapshots().await.unwrap();
            assert_eq!(snapshots.len(), 2);

            let loaded_snapshot = memory_persistence.load_snapshot("snapshot1").await.unwrap();
            assert!(loaded_snapshot.is_some());

            memory_persistence.delete_state().await.unwrap();
            assert!(!memory_persistence.state_exists().await.unwrap());
        })
        .await;

    Ok(())
}

#[tokio::test]
async fn test_state_manager_tracing() -> Result<()> {
    init_test_tracing();

    let state_manager = StateManager::new().await?;

    // Test set operations tracing
    tracing::info_span!("test_set_operations")
        .in_scope(|| async {
            state_manager
                .set(StateScope::Global, "test_key", json!("test_value"))
                .await
                .unwrap();

            state_manager
                .set_with_class(
                    StateScope::Agent("agent1".to_string()),
                    "agent_key",
                    json!({"data": "agent_data"}),
                    None,
                )
                .await
                .unwrap();

            state_manager
                .set_with_hooks(
                    StateScope::Session("session1".to_string()),
                    "session_key",
                    json!(42),
                )
                .await
                .unwrap();
        })
        .await;

    // Test get operations tracing
    tracing::info_span!("test_get_operations")
        .in_scope(|| async {
            let value = state_manager
                .get(StateScope::Global, "test_key")
                .await
                .unwrap();
            assert!(value.is_some());

            let value = state_manager
                .get_with_class(StateScope::Agent("agent1".to_string()), "agent_key", None)
                .await
                .unwrap();
            assert!(value.is_some());
        })
        .await;

    // Test delete operations tracing
    tracing::info_span!("test_delete_operations")
        .in_scope(|| async {
            let deleted = state_manager
                .delete(StateScope::Global, "test_key")
                .await
                .unwrap();
            assert!(deleted);
        })
        .await;

    // Test list_keys tracing
    tracing::info_span!("test_list_keys")
        .in_scope(|| async {
            state_manager
                .set(StateScope::Global, "key1", json!("value1"))
                .await
                .unwrap();
            state_manager
                .set(StateScope::Global, "key2", json!("value2"))
                .await
                .unwrap();

            let keys = state_manager.list_keys(StateScope::Global).await.unwrap();
            assert!(keys.len() >= 2);
        })
        .await;

    // Test clear_scope tracing
    tracing::info_span!("test_clear_scope")
        .in_scope(|| async {
            state_manager.clear_scope(StateScope::Global).await.unwrap();
            let keys = state_manager.list_keys(StateScope::Global).await.unwrap();
            assert_eq!(keys.len(), 0);
        })
        .await;

    Ok(())
}

#[tokio::test]
async fn test_agent_state_operations_tracing() -> Result<()> {
    init_test_tracing();

    let state_manager = StateManager::new().await?;

    // Create test agent state
    use llmspell_kernel::state::agent_state::{
        AgentMetadata, AgentStateData, ExecutionState, ToolUsageStats,
    };
    use std::collections::HashMap;

    let agent_state = llmspell_kernel::state::agent_state::PersistentAgentState {
        agent_id: "test_agent".to_string(),
        agent_type: "assistant".to_string(),
        state: AgentStateData {
            conversation_history: vec![],
            context_variables: HashMap::new(),
            tool_usage_stats: ToolUsageStats::default(),
            execution_state: ExecutionState::Idle,
            custom_data: HashMap::new(),
        },
        metadata: AgentMetadata {
            name: "Test Agent".to_string(),
            description: Some("Test agent for tracing".to_string()),
            version: "1.0.0".to_string(),
            capabilities: vec![],
            provider_config: None,
            tags: vec![],
        },
        creation_time: std::time::SystemTime::now(),
        last_modified: std::time::SystemTime::now(),
        schema_version: 1,
        hook_registrations: vec![],
        last_hook_execution: None,
        correlation_context: None,
    };

    // Test save_agent_state tracing
    tracing::info_span!("test_save_agent_state")
        .in_scope(|| async {
            state_manager.save_agent_state(&agent_state).await.unwrap();
        })
        .await;

    // Test load_agent_state tracing
    tracing::info_span!("test_load_agent_state")
        .in_scope(|| async {
            let loaded = state_manager.load_agent_state("test_agent").await.unwrap();
            assert!(loaded.is_some());
        })
        .await;

    // Test load_agent_state_fast tracing
    tracing::info_span!("test_load_agent_state_fast")
        .in_scope(|| async {
            let loaded = state_manager
                .load_agent_state_fast("test_agent")
                .await
                .unwrap();
            assert!(loaded.is_some());
        })
        .await;

    // Test get_agent_metadata tracing
    tracing::info_span!("test_get_agent_metadata")
        .in_scope(|| async {
            let metadata = state_manager
                .get_agent_metadata("test_agent")
                .await
                .unwrap();
            assert!(metadata.is_some());
        })
        .await;

    // Test list_agent_states tracing
    tracing::info_span!("test_list_agent_states")
        .in_scope(|| async {
            let agents = state_manager.list_agent_states().await.unwrap();
            assert!(agents.contains(&"test_agent".to_string()));
        })
        .await;

    // Test delete_agent_state tracing
    tracing::info_span!("test_delete_agent_state")
        .in_scope(|| async {
            let deleted = state_manager
                .delete_agent_state("test_agent")
                .await
                .unwrap();
            assert!(deleted);
        })
        .await;

    Ok(())
}

#[tokio::test]
async fn test_scoped_operations_tracing() -> Result<()> {
    init_test_tracing();

    let state_manager = StateManager::new().await?;
    let scope = StateScope::Agent("test_agent".to_string());

    // Test scoped set/get/delete operations
    tracing::info_span!("test_scoped_ops")
        .in_scope(|| async {
            state_manager
                .set_scoped(scope.clone(), "scoped_key", json!("scoped_value"))
                .await
                .unwrap();

            let value = state_manager
                .get_scoped(scope.clone(), "scoped_key")
                .await
                .unwrap();
            assert!(value.is_some());

            let deleted = state_manager
                .delete_scoped(scope.clone(), "scoped_key")
                .await
                .unwrap();
            assert!(deleted);
        })
        .await;

    Ok(())
}

#[tokio::test]
async fn test_hook_persistence_tracing() -> Result<()> {
    init_test_tracing();

    let state_manager = StateManager::new().await?;

    // Test hook execution persistence tracing
    tracing::info_span!("test_hook_persistence")
        .in_scope(|| async {
            let correlation_id = uuid::Uuid::new_v4();

            // This would normally be called internally during hook execution
            // Here we're just testing that the tracing instrumentation works
            let executions = state_manager
                .replay_manager()
                .get_hook_executions_by_correlation(correlation_id)
                .await
                .unwrap();

            assert_eq!(executions.len(), 0);
        })
        .await;

    Ok(())
}

#[tokio::test]
async fn test_tracing_performance_overhead() -> Result<()> {
    // Test that tracing doesn't add significant overhead (<2%)
    let state_manager = StateManager::new().await?;

    // Measure time without tracing span
    let start = std::time::Instant::now();
    for i in 0..100 {
        state_manager
            .set(StateScope::Global, &format!("key_{}", i), json!(i))
            .await?;
    }
    let duration_without = start.elapsed();

    // Measure time with tracing span
    let start = std::time::Instant::now();
    tracing::info_span!("perf_test")
        .in_scope(|| async {
            for i in 0..100 {
                state_manager
                    .set(StateScope::Global, &format!("key2_{}", i), json!(i))
                    .await
                    .unwrap();
            }
        })
        .await;
    let duration_with = start.elapsed();

    // Check overhead is less than 2%
    let overhead_percent = ((duration_with.as_nanos() as f64 - duration_without.as_nanos() as f64)
        / duration_without.as_nanos() as f64)
        * 100.0;

    println!("Tracing overhead: {:.2}%", overhead_percent);
    assert!(
        overhead_percent < 2.0,
        "Tracing overhead too high: {:.2}%",
        overhead_percent
    );

    Ok(())
}
