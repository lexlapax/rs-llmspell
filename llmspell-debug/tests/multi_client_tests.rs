//! Multi-client debugging integration tests for Task 9.2.2
//!
//! Tests concurrent session handling, resource isolation, and conflict resolution

use llmspell_debug::{
    Breakpoint, DebugCommand, DebugSessionManager, DebugState, ExecutionManager,
    InteractiveDebugger, SharedExecutionContext,
};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Test concurrent session creation by multiple clients
#[tokio::test]
async fn test_concurrent_session_creation() {
    let execution_manager = Arc::new(ExecutionManager::new());
    let session_manager = Arc::new(DebugSessionManager::new(execution_manager));

    // Create multiple sessions concurrently
    let mut handles = vec![];
    for i in 0..10 {
        let manager = session_manager.clone();
        let handle = tokio::spawn(async move {
            let client_id = format!("client_{i}");
            manager.create_session(client_id).await
        });
        handles.push(handle);
    }

    // Wait for all sessions to be created
    let mut session_ids = vec![];
    for handle in handles {
        let result = handle.await.unwrap();
        assert!(result.is_ok());
        session_ids.push(result.unwrap());
    }

    // Verify all sessions are unique
    let unique_count = session_ids
        .iter()
        .collect::<std::collections::HashSet<_>>()
        .len();
    assert_eq!(unique_count, 10, "All session IDs should be unique");

    // Verify all sessions exist
    let active_sessions = session_manager.list_sessions().await;
    assert_eq!(active_sessions.len(), 10);
}

/// Test session persistence and reconnection
#[tokio::test]
async fn test_session_persistence_and_reconnection() {
    let execution_manager = Arc::new(ExecutionManager::new());
    let session_manager = Arc::new(DebugSessionManager::new(execution_manager));

    // Create initial session
    let client_id = "persistent_client".to_string();
    let session_id = session_manager
        .create_session(client_id.clone())
        .await
        .unwrap();

    // Add some state to the session
    let breakpoint = Breakpoint::new("test.lua".to_string(), 10);
    session_manager
        .add_session_breakpoint(&session_id, breakpoint)
        .await
        .unwrap();

    // Simulate reconnection - should get the same session
    let reconnected_id = session_manager
        .create_session(client_id.clone())
        .await
        .unwrap();
    assert_eq!(
        session_id, reconnected_id,
        "Should reconnect to same session"
    );

    // Verify state is preserved
    let session = session_manager.get_session(&reconnected_id).await.unwrap();
    assert_eq!(session.breakpoints.len(), 1);
    assert_eq!(session.client_id, client_id);

    // Test explicit reconnection
    let reconnect_result = session_manager.reconnect_session(&client_id).await;
    assert!(reconnect_result.is_ok());
    assert_eq!(reconnect_result.unwrap(), session_id);
}

/// Test script conflict resolution
#[tokio::test]
async fn test_script_conflict_resolution() {
    let execution_manager = Arc::new(ExecutionManager::new());
    let session_manager = Arc::new(DebugSessionManager::new(execution_manager));

    // Create two sessions
    let session1 = session_manager
        .create_session("client1".to_string())
        .await
        .unwrap();
    let session2 = session_manager
        .create_session("client2".to_string())
        .await
        .unwrap();

    let script_path = PathBuf::from("/test/script.lua");

    // First session locks the script
    let result = session_manager
        .set_session_script(&session1, script_path.clone())
        .await;
    assert!(result.is_ok(), "First session should lock script");

    // Second session tries to lock the same script
    let result = session_manager
        .set_session_script(&session2, script_path.clone())
        .await;
    assert!(result.is_err(), "Second session should be denied");
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("already being debugged"));

    // Check lock status
    assert!(session_manager.is_script_locked(&script_path).await);
    let locked_by = session_manager.get_script_session(&script_path).await;
    assert_eq!(locked_by, Some(session1.clone()));

    // First session can update its own script path
    let result = session_manager
        .set_session_script(&session1, script_path.clone())
        .await;
    assert!(result.is_ok(), "Same session should be able to update");

    // Remove first session should release the lock
    session_manager.remove_session(&session1).await.unwrap();
    assert!(!session_manager.is_script_locked(&script_path).await);

    // Now second session can lock it
    let result = session_manager
        .set_session_script(&session2, script_path.clone())
        .await;
    assert!(result.is_ok(), "Second session should now be able to lock");
}

/// Test resource isolation between sessions
#[tokio::test]
async fn test_resource_isolation_between_sessions() {
    let execution_manager = Arc::new(ExecutionManager::new());
    let session_manager = Arc::new(DebugSessionManager::new(execution_manager));

    // Create two independent sessions
    let session1 = session_manager
        .create_session("client1".to_string())
        .await
        .unwrap();
    let session2 = session_manager
        .create_session("client2".to_string())
        .await
        .unwrap();

    // Set different scripts for each session
    let script1 = PathBuf::from("/test/script1.lua");
    let script2 = PathBuf::from("/test/script2.lua");

    session_manager
        .set_session_script(&session1, script1.clone())
        .await
        .unwrap();
    session_manager
        .set_session_script(&session2, script2.clone())
        .await
        .unwrap();

    // Add different breakpoints to each session
    let bp1 = Breakpoint::new(script1.to_string_lossy().to_string(), 10);
    let bp2 = Breakpoint::new(script2.to_string_lossy().to_string(), 20);

    session_manager
        .add_session_breakpoint(&session1, bp1)
        .await
        .unwrap();
    session_manager
        .add_session_breakpoint(&session2, bp2)
        .await
        .unwrap();

    // Verify isolation - each session has its own breakpoints
    let s1 = session_manager.get_session(&session1).await.unwrap();
    let s2 = session_manager.get_session(&session2).await.unwrap();

    assert_eq!(s1.breakpoints.len(), 1);
    assert_eq!(s2.breakpoints.len(), 1);
    assert_ne!(s1.breakpoints[0].line, s2.breakpoints[0].line);
    assert_ne!(s1.script_path, s2.script_path);

    // Verify script locks are independent
    assert!(session_manager.is_script_locked(&script1).await);
    assert!(session_manager.is_script_locked(&script2).await);
    assert_eq!(
        session_manager.get_script_session(&script1).await,
        Some(session1.clone())
    );
    assert_eq!(
        session_manager.get_script_session(&script2).await,
        Some(session2.clone())
    );
}

/// Test handling multiple debug commands concurrently
#[tokio::test]
async fn test_concurrent_debug_commands() {
    let execution_manager = Arc::new(ExecutionManager::new());
    let session_manager = Arc::new(DebugSessionManager::new(execution_manager));

    // Create multiple sessions
    let mut session_ids = vec![];
    for i in 0..5 {
        let id = session_manager
            .create_session(format!("client_{i}"))
            .await
            .unwrap();
        session_ids.push(id);
    }

    // Send concurrent debug commands to different sessions
    let mut handles = vec![];
    for (i, session_id) in session_ids.iter().enumerate() {
        let manager = session_manager.clone();
        let sid = session_id.clone();
        let handle = tokio::spawn(async move {
            // Each session sends different commands
            let commands = match i % 3 {
                0 => vec![DebugCommand::Continue, DebugCommand::Pause],
                1 => vec![DebugCommand::StepInto, DebugCommand::StepOver],
                _ => vec![DebugCommand::StepOut, DebugCommand::Continue],
            };

            for cmd in commands {
                let result = manager.handle_debug_command(&sid, cmd).await;
                assert!(result.is_ok(), "Command should succeed for session {sid}");
            }
        });
        handles.push(handle);
    }

    // Wait for all commands to complete
    for handle in handles {
        handle.await.unwrap();
    }

    // Verify all sessions still exist
    let active = session_manager.list_sessions().await;
    assert_eq!(active.len(), 5);
}

/// Test session cleanup with active locks
#[tokio::test]
async fn test_session_cleanup_with_locks() {
    let execution_manager = Arc::new(ExecutionManager::new());
    let session_manager = Arc::new(DebugSessionManager::new(execution_manager));

    // Create session and lock a script
    let session_id = session_manager
        .create_session("cleanup_client".to_string())
        .await
        .unwrap();

    let script_path = PathBuf::from("/test/cleanup.lua");
    session_manager
        .set_session_script(&session_id, script_path.clone())
        .await
        .unwrap();

    // Verify lock is active
    assert!(session_manager.is_script_locked(&script_path).await);

    // Remove session
    let removed = session_manager.remove_session(&session_id).await.unwrap();
    assert!(removed);

    // Verify lock is released
    assert!(!session_manager.is_script_locked(&script_path).await);

    // Verify persistent session mapping is cleaned up
    let reconnect_result = session_manager.reconnect_session("cleanup_client").await;
    assert!(reconnect_result.is_err());
}

/// Test changing script path within a session
#[tokio::test]
async fn test_changing_script_path() {
    let execution_manager = Arc::new(ExecutionManager::new());
    let session_manager = Arc::new(DebugSessionManager::new(execution_manager));

    let session_id = session_manager
        .create_session("switcher".to_string())
        .await
        .unwrap();

    let script1 = PathBuf::from("/test/first.lua");
    let script2 = PathBuf::from("/test/second.lua");

    // Set first script
    session_manager
        .set_session_script(&session_id, script1.clone())
        .await
        .unwrap();
    assert!(session_manager.is_script_locked(&script1).await);

    // Change to second script
    session_manager
        .set_session_script(&session_id, script2.clone())
        .await
        .unwrap();

    // Verify first script is unlocked and second is locked
    assert!(!session_manager.is_script_locked(&script1).await);
    assert!(session_manager.is_script_locked(&script2).await);
}

/// Test maximum concurrent sessions handling
#[tokio::test]
async fn test_maximum_concurrent_sessions() {
    let execution_manager = Arc::new(ExecutionManager::new());
    let session_manager = Arc::new(DebugSessionManager::new(execution_manager));

    // Create many sessions concurrently (simulating heavy load)
    let session_count = 100;
    let mut handles = vec![];

    for i in 0..session_count {
        let manager = session_manager.clone();
        let handle = tokio::spawn(async move {
            let client_id = format!("stress_client_{i}");
            let session_id = manager.create_session(client_id).await?;

            // Each session performs some operations
            let script = PathBuf::from(format!("/test/script_{i}.lua"));
            manager.set_session_script(&session_id, script).await?;

            let bp = Breakpoint::new(format!("script_{i}.lua"), u32::try_from(i).unwrap());
            manager.add_session_breakpoint(&session_id, bp).await?;

            Ok::<String, anyhow::Error>(session_id)
        });
        handles.push(handle);
    }

    // Collect results
    let mut success_count = 0;
    for handle in handles {
        if let Ok(Ok(_)) = handle.await {
            success_count += 1;
        }
    }

    // All sessions should be created successfully
    assert_eq!(success_count, session_count);

    // Verify session count
    let active = session_manager.list_sessions().await;
    assert_eq!(active.len(), session_count);
}

/// Test session state synchronization
#[tokio::test]
async fn test_session_state_synchronization() {
    let execution_manager = Arc::new(ExecutionManager::new());
    let shared_context = Arc::new(RwLock::new(SharedExecutionContext::new()));
    let debugger = InteractiveDebugger::new(execution_manager.clone(), shared_context);

    // Create a session through the interactive debugger
    let _session_id = debugger
        .create_debug_session("sync_client".to_string())
        .await
        .unwrap();

    // Set breakpoints through interactive debugger
    let bp_id = debugger
        .set_conditional_breakpoint("test.lua".to_string(), 42, "x > 10".to_string())
        .await
        .unwrap();

    // Verify breakpoint is synchronized with execution manager
    let breakpoints = execution_manager.get_breakpoints().await;
    assert!(breakpoints.iter().any(|bp| bp.id == bp_id));

    // Send debug commands and verify state updates
    debugger.continue_execution().await.unwrap();
    let state = debugger.get_debug_state().await;
    assert_eq!(state, DebugState::Running);
}
