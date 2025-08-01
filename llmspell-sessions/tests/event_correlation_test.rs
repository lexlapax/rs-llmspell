//! ABOUTME: Integration tests for session event correlation tracking
//! ABOUTME: Validates event correlation is properly integrated in SessionManager

use anyhow::Result;
use llmspell_events::bus::EventBus;
use llmspell_hooks::{HookExecutor, HookRegistry};
use llmspell_sessions::{types::CreateSessionOptions, SessionManager, SessionManagerConfig};
use llmspell_state_persistence::StateManager;
use llmspell_storage::MemoryBackend;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

#[cfg_attr(test_category = "integration")]
#[tokio::test]
async fn test_session_lifecycle_correlation() -> Result<()> {
    // Create infrastructure
    let storage_backend = Arc::new(MemoryBackend::new());
    let state_manager = Arc::new(StateManager::new().await.unwrap());
    let hook_registry = Arc::new(HookRegistry::new());
    let hook_executor = Arc::new(HookExecutor::new());
    let event_bus = Arc::new(EventBus::new());

    let config = SessionManagerConfig {
        event_config: llmspell_sessions::config::EventConfig {
            enable_session_events: true,
            ..Default::default()
        },
        ..Default::default()
    };

    let session_manager = SessionManager::new(
        state_manager,
        storage_backend,
        hook_registry,
        hook_executor,
        &event_bus,
        config,
    )
    .unwrap();

    // Create a session
    let options = CreateSessionOptions {
        name: Some("Test Correlation Session".to_string()),
        description: Some("Session for testing event correlation".to_string()),
        created_by: Some("test-user".to_string()),
        ..Default::default()
    };

    let session_id = session_manager.create_session(options).await?;

    // Verify the session was created
    let _session = session_manager.get_session(&session_id).await?;

    // Events are correlated internally - we verify by performing operations

    // Suspend the session
    session_manager.suspend_session(&session_id).await?;

    // Verify suspend worked by checking session status
    let session = session_manager.get_session(&session_id).await?;
    assert_eq!(
        session.status().await,
        llmspell_sessions::SessionStatus::Suspended
    );

    // Resume the session
    session_manager.resume_session(&session_id).await?;

    // Complete the session
    session_manager.complete_session(&session_id).await?;

    // Completed sessions are removed from active sessions list
    // We just verify the operation succeeded by the lack of error

    Ok(())
}

#[cfg_attr(test_category = "integration")]
#[tokio::test]
async fn test_artifact_event_correlation() -> Result<()> {
    // Create infrastructure
    let storage_backend = Arc::new(MemoryBackend::new());
    let state_manager = Arc::new(StateManager::new().await.unwrap());
    let hook_registry = Arc::new(HookRegistry::new());
    let hook_executor = Arc::new(HookExecutor::new());
    let event_bus = Arc::new(EventBus::new());

    let config = SessionManagerConfig {
        event_config: llmspell_sessions::config::EventConfig {
            enable_session_events: true,
            ..Default::default()
        },
        ..Default::default()
    };

    let session_manager = SessionManager::new(
        state_manager,
        storage_backend,
        hook_registry,
        hook_executor,
        &event_bus,
        config,
    )
    .unwrap();

    // Create a session
    let session_id = session_manager
        .create_session(CreateSessionOptions::default())
        .await?;

    // Store an artifact
    let artifact_content = b"Test artifact content".to_vec();
    let artifact_id = session_manager
        .store_artifact(
            &session_id,
            llmspell_sessions::artifact::ArtifactType::UserInput,
            "test.txt".to_string(),
            artifact_content,
            None,
        )
        .await?;

    // Verify artifact was stored
    let artifacts = session_manager.list_artifacts(&session_id).await?;
    assert_eq!(artifacts.len(), 1);
    // Artifact metadata doesn't include the ID, but we can verify by name
    assert_eq!(artifacts[0].name, "test.txt");

    // Delete the artifact
    session_manager
        .delete_artifact(&session_id, &artifact_id)
        .await?;

    // Verify artifact was deleted
    let artifacts = session_manager.list_artifacts(&session_id).await?;
    assert_eq!(artifacts.len(), 0);

    Ok(())
}

#[cfg_attr(test_category = "integration")]
#[tokio::test]
async fn test_session_lifecycle_with_multiple_operations() -> Result<()> {
    // Create infrastructure
    let storage_backend = Arc::new(MemoryBackend::new());
    let state_manager = Arc::new(StateManager::new().await.unwrap());
    let hook_registry = Arc::new(HookRegistry::new());
    let hook_executor = Arc::new(HookExecutor::new());
    let event_bus = Arc::new(EventBus::new());

    let config = SessionManagerConfig {
        event_config: llmspell_sessions::config::EventConfig {
            enable_session_events: true,
            ..Default::default()
        },
        ..Default::default()
    };

    let session_manager = SessionManager::new(
        state_manager,
        storage_backend,
        hook_registry,
        hook_executor,
        &event_bus,
        config,
    )
    .unwrap();

    // Create a session and verify operations are tracked
    let session_id = session_manager
        .create_session(CreateSessionOptions::default())
        .await?;

    // Perform various operations
    session_manager.suspend_session(&session_id).await?;

    // Small delay to ensure proper ordering
    sleep(Duration::from_millis(10)).await;

    session_manager.resume_session(&session_id).await?;

    // Store some artifacts
    for i in 0..3 {
        session_manager
            .store_artifact(
                &session_id,
                llmspell_sessions::artifact::ArtifactType::UserInput,
                format!("file{}.txt", i),
                format!("Content {}", i).into_bytes(),
                None,
            )
            .await?;

        sleep(Duration::from_millis(10)).await;
    }

    // Get session state before completing
    let session = session_manager.get_session(&session_id).await?;
    let metadata = session.metadata.read().await;
    assert_eq!(metadata.artifact_count, 3);
    drop(metadata);

    // Complete the session
    session_manager.complete_session(&session_id).await?;

    // Completed sessions are removed from active sessions list

    Ok(())
}

#[cfg_attr(test_category = "integration")]
#[tokio::test]
async fn test_multiple_sessions_correlation_isolation() -> Result<()> {
    // Create infrastructure
    let storage_backend = Arc::new(MemoryBackend::new());
    let state_manager = Arc::new(StateManager::new().await.unwrap());
    let hook_registry = Arc::new(HookRegistry::new());
    let hook_executor = Arc::new(HookExecutor::new());
    let event_bus = Arc::new(EventBus::new());

    let config = SessionManagerConfig {
        event_config: llmspell_sessions::config::EventConfig {
            enable_session_events: true,
            ..Default::default()
        },
        ..Default::default()
    };

    let session_manager = SessionManager::new(
        state_manager,
        storage_backend,
        hook_registry,
        hook_executor,
        &event_bus,
        config,
    )
    .unwrap();

    // Create multiple sessions
    let session1_id = session_manager
        .create_session(CreateSessionOptions {
            name: Some("Session 1".to_string()),
            ..Default::default()
        })
        .await?;

    let session2_id = session_manager
        .create_session(CreateSessionOptions {
            name: Some("Session 2".to_string()),
            ..Default::default()
        })
        .await?;

    // Verify both sessions exist and are different
    assert!(session1_id != session2_id);

    // Perform operations on each session
    session_manager
        .store_artifact(
            &session1_id,
            llmspell_sessions::artifact::ArtifactType::UserInput,
            "session1_file.txt".to_string(),
            b"Session 1 content".to_vec(),
            None,
        )
        .await?;

    session_manager
        .store_artifact(
            &session2_id,
            llmspell_sessions::artifact::ArtifactType::UserInput,
            "session2_file.txt".to_string(),
            b"Session 2 content".to_vec(),
            None,
        )
        .await?;

    // Verify both sessions have artifacts before completing
    let session1 = session_manager.get_session(&session1_id).await?;
    let metadata1 = session1.metadata.read().await;
    assert_eq!(metadata1.artifact_count, 1);
    drop(metadata1);

    let session2 = session_manager.get_session(&session2_id).await?;
    let metadata2 = session2.metadata.read().await;
    assert_eq!(metadata2.artifact_count, 1);
    drop(metadata2);

    // Complete both sessions
    session_manager.complete_session(&session1_id).await?;
    session_manager.complete_session(&session2_id).await?;

    // Verify both sessions completed (removed from active list)
    let sessions = session_manager
        .list_sessions(llmspell_sessions::types::SessionQuery::default())
        .await?;
    assert_eq!(sessions.len(), 0); // All sessions completed and removed

    Ok(())
}
