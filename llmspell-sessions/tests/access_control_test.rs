//! ABOUTME: Integration tests for artifact access control system
//! ABOUTME: Tests session-based permissions, cross-session sharing, and audit logging

#[cfg(test)]
mod tests {
    use llmspell_events::bus::EventBus;
    use llmspell_hooks::{HookExecutor, HookRegistry};
    use llmspell_sessions::{
        artifact::{access::Permission, ArtifactType},
        config::SessionManagerConfig,
        SessionManager,
    };
    use llmspell_state_persistence::StateManager;
    use llmspell_storage::MemoryBackend;
    use std::sync::Arc;

    /// Create a test session manager with default configuration
    async fn create_test_session_manager() -> SessionManager {
        let state_manager = Arc::new(StateManager::new().await.unwrap());
        let storage_backend = Arc::new(MemoryBackend::new());
        let hook_registry = Arc::new(HookRegistry::new());
        let hook_executor = Arc::new(HookExecutor::new());
        let event_bus = Arc::new(EventBus::new());
        let config = SessionManagerConfig::default();

        SessionManager::new(
            state_manager,
            storage_backend,
            hook_registry,
            hook_executor,
            &event_bus,
            config,
        )
        .unwrap()
    }

    #[cfg_attr(test_category = "integration")]
    #[tokio::test]
    async fn test_artifact_owner_permissions() {
        let manager = create_test_session_manager().await;

        // Create a session
        let session_id = manager.create_session(Default::default()).await.unwrap();

        // Store an artifact
        let artifact_id = manager
            .store_artifact(
                &session_id,
                ArtifactType::UserInput,
                "test.txt".to_string(),
                b"test content".to_vec(),
                None,
            )
            .await
            .unwrap();

        // Owner should be able to access their own artifact
        let artifact = manager
            .get_artifact(&session_id, &artifact_id)
            .await
            .unwrap();
        assert_eq!(artifact.metadata.name, "test.txt");

        // Owner should be able to delete their own artifact
        assert!(manager
            .delete_artifact(&session_id, &artifact_id)
            .await
            .is_ok());
    }

    #[cfg_attr(test_category = "integration")]
    #[tokio::test]
    async fn test_cross_session_access_denied() {
        let manager = create_test_session_manager().await;

        // Create two sessions
        let owner_session = manager.create_session(Default::default()).await.unwrap();
        let other_session = manager.create_session(Default::default()).await.unwrap();

        // Store an artifact in the first session
        let artifact_id = manager
            .store_artifact(
                &owner_session,
                ArtifactType::UserInput,
                "private.txt".to_string(),
                b"private content".to_vec(),
                None,
            )
            .await
            .unwrap();

        // Other session should not be able to access the artifact
        let result = manager.get_artifact(&other_session, &artifact_id).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("does not have read permission"));

        // Other session should not be able to delete the artifact
        let result = manager.delete_artifact(&other_session, &artifact_id).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("does not have delete permission"));
    }

    #[cfg_attr(test_category = "integration")]
    #[tokio::test]
    async fn test_permission_granting() {
        let manager = create_test_session_manager().await;

        // Create two sessions
        let owner_session = manager.create_session(Default::default()).await.unwrap();
        let user_session = manager.create_session(Default::default()).await.unwrap();

        // Store an artifact
        let artifact_id = manager
            .store_artifact(
                &owner_session,
                ArtifactType::UserInput,
                "shared.txt".to_string(),
                b"shared content".to_vec(),
                None,
            )
            .await
            .unwrap();

        // User should not have initial access
        assert!(manager
            .get_artifact(&user_session, &artifact_id)
            .await
            .is_err());

        // Owner grants read permission
        assert!(
            manager
                .grant_artifact_permission(
                    &owner_session,
                    &artifact_id,
                    user_session,
                    Permission::Read,
                )
                .await
                .is_ok()
        );

        // User should now be able to read
        let artifact = manager
            .get_artifact(&user_session, &artifact_id)
            .await
            .unwrap();
        assert_eq!(artifact.metadata.name, "shared.txt");

        // But user should not be able to delete (needs admin permission)
        assert!(manager
            .delete_artifact(&user_session, &artifact_id)
            .await
            .is_err());

        // Owner grants admin permission
        assert!(manager
            .grant_artifact_permission(
                &owner_session,
                &artifact_id,
                user_session,
                Permission::Admin,
            )
            .await
            .is_ok());

        // User should now be able to delete
        assert!(manager
            .delete_artifact(&user_session, &artifact_id)
            .await
            .is_ok());
    }

    #[cfg_attr(test_category = "integration")]
    #[tokio::test]
    async fn test_permission_revocation() {
        let manager = create_test_session_manager().await;

        // Create two sessions
        let owner_session = manager.create_session(Default::default()).await.unwrap();
        let user_session = manager.create_session(Default::default()).await.unwrap();

        // Store an artifact
        let artifact_id = manager
            .store_artifact(
                &owner_session,
                ArtifactType::UserInput,
                "revokable.txt".to_string(),
                b"revokable content".to_vec(),
                None,
            )
            .await
            .unwrap();

        // Grant read permission
        assert!(
            manager
                .grant_artifact_permission(
                    &owner_session,
                    &artifact_id,
                    user_session,
                    Permission::Read,
                )
                .await
                .is_ok()
        );

        // User should be able to read
        assert!(manager
            .get_artifact(&user_session, &artifact_id)
            .await
            .is_ok());

        // Owner revokes permission
        assert!(manager
            .revoke_artifact_permission(&owner_session, &artifact_id, &user_session,)
            .await
            .is_ok());

        // User should no longer be able to read
        assert!(manager
            .get_artifact(&user_session, &artifact_id)
            .await
            .is_err());
    }

    #[cfg_attr(test_category = "integration")]
    #[tokio::test]
    async fn test_acl_viewing() {
        let manager = create_test_session_manager().await;

        // Create sessions
        let owner_session = manager.create_session(Default::default()).await.unwrap();
        let user_session = manager.create_session(Default::default()).await.unwrap();
        let other_session = manager.create_session(Default::default()).await.unwrap();

        // Store an artifact
        let artifact_id = manager
            .store_artifact(
                &owner_session,
                ArtifactType::UserInput,
                "acl_test.txt".to_string(),
                b"acl test content".to_vec(),
                None,
            )
            .await
            .unwrap();

        // Grant permission to user
        assert!(manager
            .grant_artifact_permission(
                &owner_session,
                &artifact_id,
                user_session,
                Permission::Write,
            )
            .await
            .is_ok());

        // Owner should be able to view ACL
        let acl = manager
            .get_artifact_acl(&owner_session, &artifact_id)
            .await
            .unwrap();
        assert_eq!(acl.owner, owner_session);
        assert_eq!(acl.entries.len(), 1);
        assert_eq!(acl.entries[0].session_id, user_session);
        assert_eq!(acl.entries[0].permission, Permission::Write);

        // User with write permission should not be able to view ACL (needs admin)
        assert!(manager
            .get_artifact_acl(&user_session, &artifact_id)
            .await
            .is_err());

        // Other session should not be able to view ACL
        assert!(manager
            .get_artifact_acl(&other_session, &artifact_id)
            .await
            .is_err());
    }

    #[cfg_attr(test_category = "integration")]
    #[tokio::test]
    async fn test_audit_logging() {
        let manager = create_test_session_manager().await;

        // Create sessions
        let owner_session = manager.create_session(Default::default()).await.unwrap();
        let user_session = manager.create_session(Default::default()).await.unwrap();

        // Store an artifact
        let artifact_id = manager
            .store_artifact(
                &owner_session,
                ArtifactType::UserInput,
                "audit_test.txt".to_string(),
                b"audit test content".to_vec(),
                None,
            )
            .await
            .unwrap();

        // Make some access attempts
        let _ = manager.get_artifact(&owner_session, &artifact_id).await; // Should succeed
        let _ = manager.get_artifact(&user_session, &artifact_id).await; // Should fail

        // Owner should be able to view audit log
        let audit_log = manager
            .get_artifact_audit_log(&owner_session, &artifact_id)
            .await
            .unwrap();
        assert!(audit_log.len() >= 2);

        // Check that we have both granted and denied entries
        let granted_entries: Vec<_> = audit_log.iter().filter(|e| e.granted).collect();
        let denied_entries: Vec<_> = audit_log.iter().filter(|e| !e.granted).collect();

        assert!(!granted_entries.is_empty());
        assert!(!denied_entries.is_empty());
    }

    #[cfg_attr(test_category = "integration")]
    #[tokio::test]
    async fn test_session_isolation() {
        let manager = create_test_session_manager().await;

        // Create two sessions
        let session1 = manager.create_session(Default::default()).await.unwrap();
        let session2 = manager.create_session(Default::default()).await.unwrap();

        // Each session stores an artifact with the same name
        let artifact1 = manager
            .store_artifact(
                &session1,
                ArtifactType::UserInput,
                "common.txt".to_string(),
                b"content from session 1".to_vec(),
                None,
            )
            .await
            .unwrap();

        let artifact2 = manager
            .store_artifact(
                &session2,
                ArtifactType::UserInput,
                "common.txt".to_string(),
                b"content from session 2".to_vec(),
                None,
            )
            .await
            .unwrap();

        // Each session should only be able to access their own artifact
        let content1 = manager.get_artifact(&session1, &artifact1).await.unwrap();
        assert_eq!(content1.get_content().unwrap(), b"content from session 1");

        let content2 = manager.get_artifact(&session2, &artifact2).await.unwrap();
        assert_eq!(content2.get_content().unwrap(), b"content from session 2");

        // Cross-session access should fail
        assert!(manager.get_artifact(&session1, &artifact2).await.is_err());
        assert!(manager.get_artifact(&session2, &artifact1).await.is_err());

        // List artifacts should only show own artifacts
        let artifacts1 = manager.list_artifacts(&session1).await.unwrap();
        let artifacts2 = manager.list_artifacts(&session2).await.unwrap();

        assert_eq!(artifacts1.len(), 1);
        assert_eq!(artifacts2.len(), 1);
        assert_eq!(artifacts1[0].name, "common.txt");
        assert_eq!(artifacts2[0].name, "common.txt");
    }

    #[cfg_attr(test_category = "integration")]
    #[tokio::test]
    async fn test_non_admin_cannot_grant_permissions() {
        let manager = create_test_session_manager().await;

        // Create three sessions
        let owner_session = manager.create_session(Default::default()).await.unwrap();
        let user1_session = manager.create_session(Default::default()).await.unwrap();
        let user2_session = manager.create_session(Default::default()).await.unwrap();

        // Store an artifact
        let artifact_id = manager
            .store_artifact(
                &owner_session,
                ArtifactType::UserInput,
                "restricted.txt".to_string(),
                b"restricted content".to_vec(),
                None,
            )
            .await
            .unwrap();

        // Owner grants read permission to user1
        assert!(manager
            .grant_artifact_permission(
                &owner_session,
                &artifact_id,
                user1_session,
                Permission::Read,
            )
            .await
            .is_ok());

        // User1 (with read permission) should not be able to grant permissions to user2
        let result = manager
            .grant_artifact_permission(
                &user1_session,
                &artifact_id,
                user2_session,
                Permission::Read,
            )
            .await;

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("cannot grant permissions"));
    }

    #[cfg_attr(test_category = "integration")]
    #[tokio::test]
    async fn test_access_control_with_different_artifact_types() {
        let manager = create_test_session_manager().await;

        // Create sessions
        let session1 = manager.create_session(Default::default()).await.unwrap();
        let session2 = manager.create_session(Default::default()).await.unwrap();

        // Store different types of artifacts
        let user_input = manager
            .store_artifact(
                &session1,
                ArtifactType::UserInput,
                "user_data.txt".to_string(),
                b"user input content".to_vec(),
                None,
            )
            .await
            .unwrap();

        println!("User input artifact ID: {}", user_input.storage_key());

        let system_generated = manager
            .store_artifact(
                &session1,
                ArtifactType::SystemGenerated,
                "system_log.txt".to_string(),
                b"system log content".to_vec(),
                None,
            )
            .await
            .unwrap();

        println!(
            "System generated artifact ID: {}",
            system_generated.storage_key()
        );

        // Cross-session access should be denied for all artifact types
        assert!(manager.get_artifact(&session2, &user_input).await.is_err());
        assert!(manager
            .get_artifact(&session2, &system_generated)
            .await
            .is_err());

        // Owner access should work for all types
        assert!(manager.get_artifact(&session1, &user_input).await.is_ok());

        let result = manager.get_artifact(&session1, &system_generated).await;
        if let Err(e) = &result {
            println!("Error getting system_generated artifact: {}", e);
        }
        assert!(result.is_ok());
    }

    #[cfg_attr(test_category = "integration")]
    #[tokio::test]
    async fn test_access_control_performance_impact() {
        use std::time::Instant;

        let manager = create_test_session_manager().await;
        let session_id = manager.create_session(Default::default()).await.unwrap();

        // Store an artifact
        let artifact_id = manager
            .store_artifact(
                &session_id,
                ArtifactType::UserInput,
                "perf_test.txt".to_string(),
                b"performance test content".to_vec(),
                None,
            )
            .await
            .unwrap();

        // Measure access time (should be < 5ms for access control overhead)
        let start = Instant::now();
        for _ in 0..100 {
            let _ = manager
                .get_artifact(&session_id, &artifact_id)
                .await
                .unwrap();
        }
        let elapsed = start.elapsed();
        let avg_per_access = elapsed.as_micros() / 100;

        println!("Average access time: {} μs", avg_per_access);
        assert!(
            avg_per_access < 5000, // 5ms max per access
            "Access control overhead too high: {} μs per access",
            avg_per_access
        );
    }

    #[cfg_attr(test_category = "integration")]
    #[tokio::test]
    async fn test_security_edge_cases() {
        let manager = create_test_session_manager().await;

        // Create sessions
        let owner_session = manager.create_session(Default::default()).await.unwrap();
        let attacker_session = manager.create_session(Default::default()).await.unwrap();

        // Store an artifact
        let artifact_id = manager
            .store_artifact(
                &owner_session,
                ArtifactType::UserInput,
                "sensitive.txt".to_string(),
                b"sensitive data".to_vec(),
                None,
            )
            .await
            .unwrap();

        // Edge Case 1: Non-owner tries to grant permissions to themselves
        let result = manager
            .grant_artifact_permission(
                &attacker_session,
                &artifact_id,
                attacker_session,
                Permission::Admin,
            )
            .await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("cannot grant permissions"));

        // Edge Case 2: Non-owner tries to revoke owner's permissions
        let result = manager
            .revoke_artifact_permission(&attacker_session, &artifact_id, &owner_session)
            .await;
        assert!(result.is_err());

        // Edge Case 3: Non-admin tries to view ACL
        let result = manager
            .get_artifact_acl(&attacker_session, &artifact_id)
            .await;
        assert!(result.is_err());

        // Edge Case 4: Owner cannot revoke their own permissions (should fail)
        let result = manager
            .revoke_artifact_permission(&owner_session, &artifact_id, &owner_session)
            .await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Cannot revoke owner's permissions"));
    }

    #[cfg_attr(test_category = "integration")]
    #[tokio::test]
    async fn test_audit_log_completeness() {
        let manager = create_test_session_manager().await;

        // Create sessions
        let owner_session = manager.create_session(Default::default()).await.unwrap();
        let user_session = manager.create_session(Default::default()).await.unwrap();

        // Store an artifact
        let artifact_id = manager
            .store_artifact(
                &owner_session,
                ArtifactType::UserInput,
                "audit_test.txt".to_string(),
                b"audit test content".to_vec(),
                None,
            )
            .await
            .unwrap();

        // Perform various operations that should be audited
        let _ = manager.get_artifact(&owner_session, &artifact_id).await; // Should succeed
        let _ = manager.get_artifact(&user_session, &artifact_id).await; // Should fail
        let _ = manager.delete_artifact(&user_session, &artifact_id).await; // Should fail

        // Grant permission and perform more operations
        manager
            .grant_artifact_permission(&owner_session, &artifact_id, user_session, Permission::Read)
            .await
            .unwrap();

        let _ = manager.get_artifact(&user_session, &artifact_id).await; // Should succeed now
        let _ = manager.delete_artifact(&user_session, &artifact_id).await; // Should still fail

        // Check audit log completeness
        let audit_log = manager
            .get_artifact_audit_log(&owner_session, &artifact_id)
            .await
            .unwrap();

        // Should have multiple entries
        assert!(audit_log.len() >= 5);

        // Should have both granted and denied entries
        let granted_count = audit_log.iter().filter(|e| e.granted).count();
        let denied_count = audit_log.iter().filter(|e| !e.granted).count();

        assert!(granted_count >= 2); // Owner access + user access after permission grant
        assert!(denied_count >= 3); // Initial user access + delete attempts

        // Check that different access types are logged
        let access_types: std::collections::HashSet<_> = audit_log
            .iter()
            .map(|e| format!("{:?}", e.access_type))
            .collect();
        assert!(access_types.len() >= 2); // Should have Read and Delete at minimum
    }
}
