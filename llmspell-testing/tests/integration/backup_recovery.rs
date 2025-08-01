// ABOUTME: Integration tests for backup and recovery functionality
// ABOUTME: Validates end-to-end backup operations, disaster recovery scenarios, and data integrity

use llmspell_state_persistence::{
    backup::{
        BackupConfig, BackupManager, BackupStatus,
        CompressionType, RestoreOptions,
    },
    config::PersistenceConfig,
    manager::StateManager,
    StateScope,
};
use serde_json::json;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tempfile::TempDir;
use tokio::time::sleep;

#[cfg(test)]
mod backup_recovery_tests {
    use super::*;

    /// Helper to create a test state manager with backup enabled
    async fn create_test_state_manager_with_backup(
    ) -> (Arc<StateManager>, Arc<BackupManager>, TempDir) {
        let temp_dir = tempfile::tempdir().unwrap();
        let backup_dir = temp_dir.path().join("backups");
        std::fs::create_dir_all(&backup_dir).unwrap();

        let persistence_config = PersistenceConfig {
            enabled: true,
            ..Default::default()
        };

        let backup_config = BackupConfig {
            backup_dir: backup_dir.clone(),
            compression_enabled: true,
            compression_type: CompressionType::Zstd,
            compression_level: 3,
            encryption_enabled: false,
            max_backups: Some(5),
            incremental_enabled: true,
            max_backup_age: Some(std::time::Duration::from_secs(7 * 24 * 3600)),
            ..Default::default()
        };

        let state_manager = Arc::new(
            StateManager::with_backend(
                llmspell_state_persistence::config::StorageBackendType::Memory,
                persistence_config,
            )
            .await
            .unwrap(),
        );

        let backup_manager =
            Arc::new(BackupManager::new(backup_config, state_manager.clone()).unwrap());

        (state_manager, backup_manager, temp_dir)
    }

    /// Helper to populate state manager with test data
    async fn populate_test_data(
        state_manager: &StateManager,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Add various types of state data
        state_manager
            .set(
                StateScope::Global,
                "user_settings",
                json!({
                    "theme": "dark",
                    "language": "en",
                    "notifications": true
                }),
            )
            .await?;

        state_manager
            .set(
                StateScope::Global,
                "session_data",
                json!({
                    "user_id": 12345,
                    "login_time": "2025-01-27T10:00:00Z",
                    "permissions": ["read", "write"]
                }),
            )
            .await?;

        state_manager
            .set(
                StateScope::Custom("agent_1".to_string()),
                "config",
                json!({
                    "model": "gpt-4",
                    "temperature": 0.7,
                    "max_tokens": 2000
                }),
            )
            .await?;

        state_manager
            .set(
                StateScope::Custom("agent_2".to_string()),
                "history",
                json!({
                    "conversations": [
                        {"role": "user", "content": "Hello"},
                        {"role": "assistant", "content": "Hi there!"}
                    ],
                    "total_tokens": 15
                }),
            )
            .await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_complete_backup_recovery_cycle() {
        let (state_manager, backup_manager, _temp_dir) =
            create_test_state_manager_with_backup().await;

        // Populate with test data
        populate_test_data(&state_manager).await.unwrap();

        // Create full backup
        let backup_status = backup_manager.create_backup(false).await.unwrap();
        assert!(!backup_status.is_incremental);
        assert!(backup_status.size_bytes > 0);
        assert!(backup_status.entry_count >= 4); // We added 4 state entries

        // Validate backup
        let validation = backup_manager
            .validate_backup(&backup_status.id)
            .await
            .unwrap();
        assert!(validation.is_valid);
        assert!(validation.checksum_valid);
        assert!(validation.integrity_valid);
        assert!(validation.errors.is_empty());

        // Modify state after backup
        state_manager
            .set(
                StateScope::Global,
                "new_data",
                json!({"after_backup": true}),
            )
            .await
            .unwrap();

        // Verify modification exists
        let modified_data = state_manager
            .get(StateScope::Global, "new_data")
            .await
            .unwrap();
        assert_eq!(modified_data, Some(json!({"after_backup": true})));

        // Restore from backup
        let restore_options = RestoreOptions {
            verify_checksums: true,
            backup_current: true,
            target_version: None,
            dry_run: false,
        };

        backup_manager
            .restore_backup(&backup_status.id, restore_options)
            .await
            .unwrap();

        // Verify original data is restored
        let user_settings = state_manager
            .get(StateScope::Global, "user_settings")
            .await
            .unwrap();
        assert_eq!(
            user_settings,
            Some(json!({
                "theme": "dark",
                "language": "en",
                "notifications": true
            }))
        );

        let session_data = state_manager
            .get(StateScope::Global, "session_data")
            .await
            .unwrap();
        assert_eq!(
            session_data,
            Some(json!({
                "user_id": 12345,
                "login_time": "2025-01-27T10:00:00Z",
                "permissions": ["read", "write"]
            }))
        );

        // Verify new data added after backup is gone (restored to backup state)
        let new_data = state_manager
            .get(StateScope::Global, "new_data")
            .await
            .unwrap();
        assert_eq!(new_data, None);
    }

    #[tokio::test]
    async fn test_incremental_backup_chain() {
        let (state_manager, backup_manager, _temp_dir) =
            create_test_state_manager_with_backup().await;

        // Initial data
        state_manager
            .set(StateScope::Global, "base_data", json!({"version": 1}))
            .await
            .unwrap();

        // Create full backup
        let full_backup = backup_manager.create_backup(false).await.unwrap();
        assert!(!full_backup.is_incremental);
        assert!(full_backup.parent_id.is_none());

        // Add more data
        state_manager
            .set(
                StateScope::Global,
                "incremental_data_1",
                json!({"version": 2}),
            )
            .await
            .unwrap();

        // Create incremental backup
        let incremental_backup_1 = backup_manager.create_backup(true).await.unwrap();
        assert!(incremental_backup_1.is_incremental);
        assert!(incremental_backup_1.parent_id.is_some());

        // Add more data
        state_manager
            .set(
                StateScope::Global,
                "incremental_data_2",
                json!({"version": 3}),
            )
            .await
            .unwrap();

        // Create another incremental backup
        let incremental_backup_2 = backup_manager.create_backup(true).await.unwrap();
        assert!(incremental_backup_2.is_incremental);
        assert!(incremental_backup_2.parent_id.is_some());

        // List all backups
        let backups = backup_manager.list_backups().await.unwrap();
        assert_eq!(backups.len(), 3);

        // Validate all backups
        for backup in &backups {
            let validation = backup_manager.validate_backup(&backup.id).await.unwrap();
            assert!(validation.is_valid, "Backup {} should be valid", backup.id);
        }

        // Restore from the latest incremental backup
        backup_manager
            .restore_backup(&incremental_backup_2.id, RestoreOptions::default())
            .await
            .unwrap();

        // Verify all data is restored correctly
        let base_data = state_manager
            .get(StateScope::Global, "base_data")
            .await
            .unwrap();
        assert_eq!(base_data, Some(json!({"version": 1})));

        let inc_data_1 = state_manager
            .get(StateScope::Global, "incremental_data_1")
            .await
            .unwrap();
        assert_eq!(inc_data_1, Some(json!({"version": 2})));

        let inc_data_2 = state_manager
            .get(StateScope::Global, "incremental_data_2")
            .await
            .unwrap();
        assert_eq!(inc_data_2, Some(json!({"version": 3})));
    }

    #[tokio::test]
    async fn test_disaster_recovery_simulation() {
        let (state_manager, backup_manager, temp_dir) =
            create_test_state_manager_with_backup().await;

        // Populate critical application data
        populate_test_data(&state_manager).await.unwrap();

        // Add more critical data
        state_manager
            .set(
                StateScope::Global,
                "critical_config",
                json!({
                    "database_url": "postgres://localhost:5432/myapp",
                    "api_keys": ["key1", "key2"],
                    "feature_flags": {"new_ui": true, "beta_features": false}
                }),
            )
            .await
            .unwrap();

        // Create backup of critical state
        let disaster_backup = backup_manager.create_backup(false).await.unwrap();

        // Simulate disaster - clear all state
        state_manager.clear_scope(StateScope::Global).await.unwrap();

        // Verify state is gone
        let user_settings = state_manager
            .get(StateScope::Global, "user_settings")
            .await
            .unwrap();
        assert_eq!(user_settings, None);

        // Simulate recovery process
        let recovery_start = SystemTime::now();

        // Restore from disaster backup
        backup_manager
            .restore_backup(
                &disaster_backup.id,
                RestoreOptions {
                    verify_checksums: true,
                    backup_current: false, // No need to backup empty state
                    target_version: None,
                    dry_run: false,
                },
            )
            .await
            .unwrap();

        let recovery_duration = recovery_start.elapsed().unwrap();

        // Verify all critical data is restored
        let user_settings = state_manager
            .get(StateScope::Global, "user_settings")
            .await
            .unwrap();
        assert!(user_settings.is_some());

        let critical_config = state_manager
            .get(StateScope::Global, "critical_config")
            .await
            .unwrap();
        assert_eq!(
            critical_config,
            Some(json!({
                "database_url": "postgres://localhost:5432/myapp",
                "api_keys": ["key1", "key2"],
                "feature_flags": {"new_ui": true, "beta_features": false}
            }))
        );

        let agent_1_config = state_manager
            .get(StateScope::Custom("agent_1".to_string()), "config")
            .await
            .unwrap();
        assert!(agent_1_config.is_some());

        // Verify recovery completed within acceptable time (should be fast for small datasets)
        assert!(
            recovery_duration < Duration::from_secs(5),
            "Recovery took too long: {:?}",
            recovery_duration
        );

        println!("Disaster recovery completed in {:?}", recovery_duration);
    }

    #[tokio::test]
    async fn test_backup_integrity_validation() {
        let (state_manager, backup_manager, temp_dir) =
            create_test_state_manager_with_backup().await;

        // Create test data with various data types
        state_manager
            .set(StateScope::Global, "string_data", json!("test string"))
            .await
            .unwrap();
        state_manager
            .set(StateScope::Global, "number_data", json!(42))
            .await
            .unwrap();
        state_manager
            .set(StateScope::Global, "boolean_data", json!(true))
            .await
            .unwrap();
        state_manager
            .set(StateScope::Global, "array_data", json!([1, 2, 3, 4, 5]))
            .await
            .unwrap();
        state_manager
            .set(
                StateScope::Global,
                "object_data",
                json!({
                    "nested": {"deep": {"value": "found"}},
                    "array": [{"id": 1}, {"id": 2}]
                }),
            )
            .await
            .unwrap();

        // Create backup
        let backup_status = backup_manager.create_backup(false).await.unwrap();

        // Validate backup integrity
        let validation = backup_manager
            .validate_backup(&backup_status.id)
            .await
            .unwrap();
        assert!(validation.is_valid);
        assert!(validation.checksum_valid);
        assert!(validation.integrity_valid);
        assert!(validation.errors.is_empty());
        assert!(validation.warnings.is_empty());

        println!(
            "Backup validation successful for backup: {}",
            backup_status.id
        );
        println!("Validation time: {:?}", validation.validated_at);
        println!("Backup size: {} bytes", backup_status.size_bytes);
        println!("Entry count: {}", backup_status.entry_count);
    }

    #[tokio::test]
    async fn test_partial_recovery_scenarios() {
        let (state_manager, backup_manager, _temp_dir) =
            create_test_state_manager_with_backup().await;

        // Create data in different scopes
        state_manager
            .set(
                StateScope::Global,
                "global_config",
                json!({"setting": "global"}),
            )
            .await
            .unwrap();
        state_manager
            .set(
                StateScope::Custom("service_a".to_string()),
                "config",
                json!({"service": "a"}),
            )
            .await
            .unwrap();
        state_manager
            .set(
                StateScope::Custom("service_b".to_string()),
                "config",
                json!({"service": "b"}),
            )
            .await
            .unwrap();
        state_manager
            .set(
                StateScope::Custom("service_c".to_string()),
                "config",
                json!({"service": "c"}),
            )
            .await
            .unwrap();

        // Create backup
        let backup_status = backup_manager.create_backup(false).await.unwrap();

        // Modify state after backup
        state_manager
            .set(
                StateScope::Custom("service_a".to_string()),
                "config",
                json!({"service": "a_modified"}),
            )
            .await
            .unwrap();
        state_manager
            .delete(StateScope::Custom("service_b".to_string()), "config")
            .await
            .unwrap();

        // Test dry run restore
        let dry_run_options = RestoreOptions {
            verify_checksums: true,
            backup_current: true,
            target_version: None,
            dry_run: true,
        };

        // Dry run should succeed without actually restoring
        backup_manager
            .restore_backup(&backup_status.id, dry_run_options)
            .await
            .unwrap();

        // Verify state is unchanged after dry run
        let service_a_config = state_manager
            .get(StateScope::Custom("service_a".to_string()), "config")
            .await
            .unwrap();
        assert_eq!(service_a_config, Some(json!({"service": "a_modified"})));

        let service_b_config = state_manager
            .get(StateScope::Custom("service_b".to_string()), "config")
            .await
            .unwrap();
        assert_eq!(service_b_config, None);

        // Now perform actual restore
        let restore_options = RestoreOptions {
            verify_checksums: true,
            backup_current: true,
            target_version: None,
            dry_run: false,
        };

        backup_manager
            .restore_backup(&backup_status.id, restore_options)
            .await
            .unwrap();

        // Verify state is restored
        let service_a_config = state_manager
            .get(StateScope::Custom("service_a".to_string()), "config")
            .await
            .unwrap();
        assert_eq!(service_a_config, Some(json!({"service": "a"})));

        let service_b_config = state_manager
            .get(StateScope::Custom("service_b".to_string()), "config")
            .await
            .unwrap();
        assert_eq!(service_b_config, Some(json!({"service": "b"})));
    }

    #[tokio::test]
    async fn test_backup_retention_and_cleanup() {
        let (state_manager, backup_manager, _temp_dir) =
            create_test_state_manager_with_backup().await;

        // Create multiple backups over time
        let mut backup_ids = Vec::new();

        for i in 0..7 {
            // Add some unique data for each backup
            state_manager
                .set(
                    StateScope::Global,
                    &format!("data_{}", i),
                    json!({"value": i}),
                )
                .await
                .unwrap();

            let backup_status = backup_manager.create_backup(false).await.unwrap();
            backup_ids.push(backup_status.id);

            // Small delay to ensure different timestamps
            sleep(Duration::from_millis(10)).await;
        }

        // Verify retention policies were applied automatically during backup creation
        // Since max_backups is 5, we should have at most 5 backups even though we created 7
        let backups = backup_manager.list_backups().await.unwrap();
        assert!(
            backups.len() <= 5,
            "Should have 5 or fewer backups due to automatic retention, but found {}",
            backups.len()
        );

        // Apply retention policies explicitly to verify they work
        let retention_report = backup_manager.apply_retention_policies().await.unwrap();

        // Verify backups are still within limit
        let backups_after_cleanup = backup_manager.list_backups().await.unwrap();
        assert!(
            backups_after_cleanup.len() <= 5,
            "Should have 5 or fewer backups after retention, but found {}",
            backups_after_cleanup.len()
        );

        // Verify newest backups are retained
        for backup in &backups_after_cleanup {
            let validation = backup_manager.validate_backup(&backup.id).await.unwrap();
            assert!(validation.is_valid, "Retained backup should be valid");
        }

        println!("Retention report: {:?}", retention_report);
        println!("Backups after cleanup: {}", backups_after_cleanup.len());
    }

    #[tokio::test]
    async fn test_backup_performance_impact() {
        let (state_manager, backup_manager, _temp_dir) =
            create_test_state_manager_with_backup().await;

        // Populate with larger dataset
        for i in 0..100 {
            state_manager.set(StateScope::Global, &format!("item_{}", i), json!({
                "id": i,
                "data": format!("Large data string for item {} with lots of text content to make the backup more realistic in size and complexity", i),
                "metadata": {
                    "created_at": "2025-01-27T10:00:00Z",
                    "tags": ["tag1", "tag2", "tag3"],
                    "properties": {
                        "nested": true,
                        "level": i % 10
                    }
                }
            })).await.unwrap();
        }

        // Measure backup creation time
        let backup_start = SystemTime::now();
        let backup_status = backup_manager.create_backup(false).await.unwrap();
        let backup_duration = backup_start.elapsed().unwrap();

        // Measure validation time
        let validation_start = SystemTime::now();
        let validation = backup_manager
            .validate_backup(&backup_status.id)
            .await
            .unwrap();
        let validation_duration = validation_start.elapsed().unwrap();

        // Measure restore time
        let restore_start = SystemTime::now();
        backup_manager
            .restore_backup(&backup_status.id, RestoreOptions::default())
            .await
            .unwrap();
        let restore_duration = restore_start.elapsed().unwrap();

        // Performance assertions
        assert!(
            backup_duration < Duration::from_secs(10),
            "Backup creation should complete within 10 seconds, took {:?}",
            backup_duration
        );

        assert!(
            validation_duration < Duration::from_secs(5),
            "Backup validation should complete within 5 seconds, took {:?}",
            validation_duration
        );

        assert!(
            restore_duration < Duration::from_secs(10),
            "Backup restore should complete within 10 seconds, took {:?}",
            restore_duration
        );

        assert!(validation.is_valid);
        assert_eq!(backup_status.entry_count, 100);

        println!("Performance metrics:");
        println!("  Backup creation: {:?}", backup_duration);
        println!("  Backup validation: {:?}", validation_duration);
        println!("  Backup restore: {:?}", restore_duration);
        println!("  Backup size: {} bytes", backup_status.size_bytes);
        println!("  Entries backed up: {}", backup_status.entry_count);
    }

    #[tokio::test]
    async fn test_concurrent_backup_operations() {
        let (state_manager, backup_manager, _temp_dir) =
            create_test_state_manager_with_backup().await;

        // Populate with test data
        populate_test_data(&state_manager).await.unwrap();

        // Launch multiple backup operations concurrently
        let mut backup_futures = Vec::new();

        for i in 0..3 {
            let backup_manager_clone = backup_manager.clone();
            let state_manager_clone = state_manager.clone();

            let future = tokio::spawn(async move {
                // Add unique data for each concurrent backup
                state_manager_clone
                    .set(
                        StateScope::Global,
                        &format!("concurrent_{}", i),
                        json!({"id": i}),
                    )
                    .await
                    .unwrap();

                // Create backup
                backup_manager_clone.create_backup(false).await
            });

            backup_futures.push(future);
        }

        // Wait for all backups to complete
        let backup_results: Result<Vec<_>, _> = futures::future::join_all(backup_futures)
            .await
            .into_iter()
            .collect();

        let backup_statuses: Vec<BackupStatus> = backup_results
            .unwrap()
            .into_iter()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        // Verify all backups succeeded
        assert_eq!(backup_statuses.len(), 3);

        // Verify all backups are valid
        for backup_status in &backup_statuses {
            let validation = backup_manager
                .validate_backup(&backup_status.id)
                .await
                .unwrap();
            assert!(
                validation.is_valid,
                "Concurrent backup {} should be valid",
                backup_status.id
            );
        }

        // Verify backups have unique IDs
        let mut ids = backup_statuses.iter().map(|b| &b.id).collect::<Vec<_>>();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), 3, "All backup IDs should be unique");

        println!("Concurrent backup test completed successfully");
        for (i, status) in backup_statuses.iter().enumerate() {
            println!(
                "  Backup {}: {} ({} bytes)",
                i, status.id, status.size_bytes
            );
        }
    }
}
