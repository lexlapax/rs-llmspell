// ABOUTME: Security tests for multi-agent state isolation
// ABOUTME: Validates isolation boundaries and permission enforcement

use llmspell_agents::state::{
    IsolationBoundary, SharedScopeConfig, SharingPattern, StateIsolationManager, StateMessage,
    StateOperation, StatePermission, StateScope, StateSharingManager,
};
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::time::timeout;

// Mock state manager for testing
struct MockStateManager;
#[tokio::test]
async fn test_strict_isolation_prevents_cross_agent_access() {
    let state_manager = Arc::new(MockStateManager);
    let isolation_manager = StateIsolationManager::new(state_manager.clone());

    // Set up agents with strict isolation
    isolation_manager.set_agent_boundary("agent1", IsolationBoundary::Strict);
    isolation_manager.set_agent_boundary("agent2", IsolationBoundary::Strict);

    // Agent1 stores data
    // TODO: When StateManager is properly integrated, uncomment this:
    // state_manager
    //     .set_scoped(
    //         StateScope::Agent("agent1".to_string()),
    //         "secret",
    //         json!("agent1_secret_data"),
    //     )
    //     .await
    //     .unwrap();

    // Verify agent1 can access its own data
    assert!(isolation_manager
        .check_access(
            "agent1",
            &StateScope::Agent("agent1".to_string()),
            StateOperation::Read
        )
        .unwrap());

    // Verify agent2 cannot access agent1's data
    assert!(!isolation_manager
        .check_access(
            "agent2",
            &StateScope::Agent("agent1".to_string()),
            StateOperation::Read
        )
        .unwrap());

    // Verify neither agent can access global scope under strict isolation
    assert!(!isolation_manager
        .check_access("agent1", &StateScope::Global, StateOperation::Read)
        .unwrap());
    assert!(!isolation_manager
        .check_access("agent2", &StateScope::Global, StateOperation::Write)
        .unwrap());
}
#[tokio::test]
async fn test_shared_scope_controlled_access() {
    let state_manager = Arc::new(MockStateManager);
    let isolation_manager = StateIsolationManager::new(state_manager.clone());

    // Create shared scope with specific permissions
    let mut permissions = HashMap::new();
    permissions.insert("reader".to_string(), vec![StatePermission::Read]);
    permissions.insert(
        "writer".to_string(),
        vec![StatePermission::Read, StatePermission::Write],
    );
    permissions.insert(
        "admin".to_string(),
        vec![
            StatePermission::Read,
            StatePermission::Write,
            StatePermission::Delete,
        ],
    );

    let shared_config = SharedScopeConfig {
        scope_id: "shared_data".to_string(),
        owner_agent_id: Some("admin".to_string()),
        allowed_agents: vec![
            "reader".to_string(),
            "writer".to_string(),
            "admin".to_string(),
        ],
        permissions,
        created_at: SystemTime::now(),
        expires_at: None,
    };

    isolation_manager
        .create_shared_scope("shared_data", Some("admin".to_string()), shared_config)
        .unwrap();

    // Set agent boundaries
    isolation_manager.set_agent_boundary("reader", IsolationBoundary::ReadOnlyShared);
    isolation_manager.set_agent_boundary("writer", IsolationBoundary::SharedAccess);
    isolation_manager.set_agent_boundary("admin", IsolationBoundary::SharedAccess);

    let shared_scope = StateScope::Custom("shared:shared_data".to_string());

    // Test read permissions
    assert!(isolation_manager
        .check_access("reader", &shared_scope, StateOperation::Read)
        .unwrap());
    assert!(isolation_manager
        .check_access("writer", &shared_scope, StateOperation::Read)
        .unwrap());
    assert!(isolation_manager
        .check_access("admin", &shared_scope, StateOperation::Read)
        .unwrap());

    // Test write permissions
    assert!(!isolation_manager
        .check_access("reader", &shared_scope, StateOperation::Write)
        .unwrap());
    assert!(isolation_manager
        .check_access("writer", &shared_scope, StateOperation::Write)
        .unwrap());
    assert!(isolation_manager
        .check_access("admin", &shared_scope, StateOperation::Write)
        .unwrap());

    // Test delete permissions
    assert!(!isolation_manager
        .check_access("reader", &shared_scope, StateOperation::Delete)
        .unwrap());
    assert!(!isolation_manager
        .check_access("writer", &shared_scope, StateOperation::Delete)
        .unwrap());
    assert!(isolation_manager
        .check_access("admin", &shared_scope, StateOperation::Delete)
        .unwrap());
}
#[tokio::test]
async fn test_audit_logging_tracks_access_attempts() {
    let state_manager = Arc::new(MockStateManager);
    let isolation_manager = StateIsolationManager::new(state_manager.clone());

    // Set strict isolation
    isolation_manager.set_agent_boundary("attacker", IsolationBoundary::Strict);
    isolation_manager.set_agent_boundary("victim", IsolationBoundary::Strict);

    // Simulate access attempts
    let victim_scope = StateScope::Agent("victim".to_string());

    // Denied access attempt
    let _ = isolation_manager.check_access("attacker", &victim_scope, StateOperation::Read);

    // Allowed access attempt
    let _ = isolation_manager.check_access("victim", &victim_scope, StateOperation::Write);

    // Check audit log
    let audit_log = isolation_manager.get_audit_log(Some(10));
    assert!(audit_log.len() >= 2);

    // Find the denied access
    let denied_entries: Vec<_> = audit_log.iter().filter(|e| !e.allowed).collect();
    assert!(!denied_entries.is_empty());
    assert_eq!(denied_entries[0].agent_id, "attacker");

    // Find the allowed access
    let allowed_entries: Vec<_> = audit_log.iter().filter(|e| e.allowed).collect();
    assert!(!allowed_entries.is_empty());
}
#[tokio::test]
async fn test_state_leakage_prevention() {
    let state_manager = Arc::new(MockStateManager);
    let isolation_manager = StateIsolationManager::new(state_manager.clone());

    // Create multiple agents with different data
    let agents = vec!["agent_a", "agent_b", "agent_c"];

    for agent in &agents {
        isolation_manager.set_agent_boundary(agent, IsolationBoundary::Strict);

        // Each agent stores sensitive data
        // TODO: When StateManager is properly integrated, uncomment this:
        // state_manager
        //     .set_scoped(
        //         StateScope::Agent(agent.to_string()),
        //         "api_key",
        //         json!(format!("secret_key_for_{}", agent)),
        //     )
        //     .await
        //     .unwrap();
    }

    // Verify no cross-contamination
    for i in 0..agents.len() {
        for j in 0..agents.len() {
            if i != j {
                // Agent i should not be able to access agent j's scope
                assert!(!isolation_manager
                    .check_access(
                        agents[i],
                        &StateScope::Agent(agents[j].to_string()),
                        StateOperation::Read,
                    )
                    .unwrap());
            }
        }
    }

    // Verify each agent can only see their own data
    // TODO: When StateManager is properly integrated, uncomment this:
    // for agent in &agents {
    //     let data = state_manager
    //         .get_scoped(StateScope::Agent(agent.to_string()), "api_key")
    //         .await
    //         .unwrap();
    //     assert_eq!(
    //         data,
    //         Some(json!(format!("secret_key_for_{}", agent)))
    //     );
    // }
}
#[tokio::test]
async fn test_performance_isolation_overhead() {
    let state_manager = Arc::new(MockStateManager);
    let isolation_manager = StateIsolationManager::new(state_manager.clone());

    // Set up test agent
    isolation_manager.set_agent_boundary("perf_test", IsolationBoundary::Strict);
    let scope = StateScope::Agent("perf_test".to_string());

    // Measure access check performance
    let start = std::time::Instant::now();

    for _ in 0..1000 {
        let _ = isolation_manager.check_access("perf_test", &scope, StateOperation::Read);
    }

    let duration = start.elapsed();
    let per_check = duration / 1000;

    // Verify isolation checks are under 1ms
    assert!(
        per_check < Duration::from_millis(1),
        "Isolation check took {:?} (>1ms threshold)",
        per_check
    );
}
#[tokio::test]
async fn test_permission_revocation() {
    let state_manager = Arc::new(MockStateManager);
    let isolation_manager = StateIsolationManager::new(state_manager.clone());

    let scope = StateScope::Custom("temp_project".to_string());

    // Grant permission
    isolation_manager.grant_permission("contractor", scope.clone(), StatePermission::Write);

    // Verify access
    assert!(isolation_manager
        .check_access("contractor", &scope, StateOperation::Write)
        .unwrap());

    // Revoke permission
    isolation_manager.revoke_permissions("contractor", scope.clone());

    // Verify access is revoked
    assert!(!isolation_manager
        .check_access("contractor", &scope, StateOperation::Write)
        .unwrap());
}
#[tokio::test]
async fn test_broadcast_channel_isolation() {
    let state_manager = Arc::new(MockStateManager);
    let sharing_manager = StateSharingManager::new(state_manager);

    // Create broadcast channel
    sharing_manager
        .create_channel(
            "announcements",
            SharingPattern::Broadcast,
            "broadcaster",
            None,
        )
        .unwrap();

    // Subscribe listeners
    sharing_manager
        .subscribe_agent("listener1", "announcements")
        .unwrap();
    sharing_manager
        .subscribe_agent("listener2", "announcements")
        .unwrap();

    // Only broadcaster can send
    let result = sharing_manager
        .publish_message(
            "announcements",
            "broadcaster",
            "notice",
            json!({"message": "Important update"}),
            None,
        )
        .await;
    assert!(result.is_ok());

    // Listeners cannot broadcast
    let result = sharing_manager
        .publish_message(
            "announcements",
            "listener1",
            "notice",
            json!({"message": "Unauthorized broadcast"}),
            None,
        )
        .await;
    assert!(result.is_err());
}
#[tokio::test]
async fn test_pipeline_ordered_access() {
    let state_manager = Arc::new(MockStateManager);
    let sharing_manager = StateSharingManager::new(state_manager);

    // Create pipeline
    let stages = vec![
        "validator".to_string(),
        "processor".to_string(),
        "outputter".to_string(),
    ];
    sharing_manager
        .create_pipeline("data_pipeline", stages.clone())
        .unwrap();

    // Process through pipeline in order
    let next = sharing_manager
        .process_pipeline_stage("data_pipeline", "validator", json!({"status": "validated"}))
        .await
        .unwrap();
    assert_eq!(next, Some("processor".to_string()));

    let next = sharing_manager
        .process_pipeline_stage("data_pipeline", "processor", json!({"status": "processed"}))
        .await
        .unwrap();
    assert_eq!(next, Some("outputter".to_string()));

    let next = sharing_manager
        .process_pipeline_stage("data_pipeline", "outputter", json!({"status": "complete"}))
        .await
        .unwrap();
    assert_eq!(next, None);
}
#[tokio::test]
async fn test_concurrent_access_safety() {
    let state_manager = Arc::new(MockStateManager);
    let isolation_manager = Arc::new(StateIsolationManager::new(state_manager.clone()));

    // Create shared scope
    let config = SharedScopeConfig {
        scope_id: "concurrent_test".to_string(),
        owner_agent_id: None,
        allowed_agents: vec!["agent1".to_string(), "agent2".to_string()],
        permissions: {
            let mut perms = HashMap::new();
            perms.insert("agent1".to_string(), vec![StatePermission::Write]);
            perms.insert("agent2".to_string(), vec![StatePermission::Write]);
            perms
        },
        created_at: SystemTime::now(),
        expires_at: None,
    };

    isolation_manager
        .create_shared_scope("concurrent_test", None, config)
        .unwrap();

    let scope = StateScope::Custom("shared:concurrent_test".to_string());

    // Spawn concurrent tasks
    let mut handles = vec![];

    for i in 0..10 {
        let agent_id = if i % 2 == 0 { "agent1" } else { "agent2" };
        let isolation_mgr = isolation_manager.clone();
        let _state_mgr = state_manager.clone();
        let scope_clone = scope.clone();

        let handle = tokio::spawn(async move {
            // Check access
            let allowed = isolation_mgr
                .check_access(agent_id, &scope_clone, StateOperation::Write)
                .unwrap();
            assert!(allowed);

            // Perform write
            // TODO: When StateManager is properly integrated, uncomment this:
            // _state_mgr
            //     .set_scoped(scope_clone, &format!("key_{}", i), json!(i))
            //     .await
            //     .unwrap();
        });

        handles.push(handle);
    }

    // Wait for all tasks
    for handle in handles {
        handle.await.unwrap();
    }

    // Verify all writes succeeded
    // TODO: When StateManager is properly integrated, uncomment this:
    // for i in 0..10 {
    //     let value = state_manager
    //         .get_scoped(scope.clone(), &format!("key_{}", i))
    //         .await
    //         .unwrap();
    //     assert_eq!(value, Some(json!(i)));
    // }
}
#[tokio::test]
async fn test_custom_isolation_policy() {
    let state_manager = Arc::new(MockStateManager);
    let isolation_manager = StateIsolationManager::new(state_manager.clone());

    // Set custom policies
    isolation_manager
        .set_agent_boundary("reader", IsolationBoundary::Custom("read-all".to_string()));
    isolation_manager.set_agent_boundary(
        "writer",
        IsolationBoundary::Custom("write-shared".to_string()),
    );

    // Test read-all policy
    assert!(isolation_manager
        .check_access("reader", &StateScope::Global, StateOperation::Read)
        .unwrap());
    assert!(!isolation_manager
        .check_access("reader", &StateScope::Global, StateOperation::Write)
        .unwrap());

    // Create shared scope for write-shared policy test
    let config = SharedScopeConfig {
        scope_id: "writable".to_string(),
        owner_agent_id: None,
        allowed_agents: vec!["writer".to_string()],
        permissions: HashMap::new(),
        created_at: SystemTime::now(),
        expires_at: None,
    };
    isolation_manager
        .create_shared_scope("writable", None, config)
        .unwrap();

    // Test write-shared policy
    let shared_scope = StateScope::Custom("shared:writable".to_string());
    assert!(isolation_manager
        .check_access("writer", &shared_scope, StateOperation::Write)
        .unwrap());
    assert!(!isolation_manager
        .check_access(
            "writer",
            &StateScope::Agent("other".to_string()),
            StateOperation::Write
        )
        .unwrap());
}
