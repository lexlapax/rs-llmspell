// ABOUTME: Integration tests for backup recovery functionality
// ABOUTME: Validates point-in-time recovery, incremental chains, and error handling

#[cfg(test)]
mod tests {
    use super::{
        backup::{BackupManager, RestoreOptions},
        StateManager, StateScope,
    };
    use llmspell_config::{BackupConfig, CompressionType, PersistenceConfig, StorageBackendType};
    use serde_json::json;
    use std::sync::Arc;
    use std::time::Duration;
    use tempfile::TempDir;

    async fn create_test_infrastructure() -> (Arc<StateManager>, Arc<BackupManager>, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let backup_dir = temp_dir.path().join("backups");
        std::fs::create_dir_all(&backup_dir).unwrap();

        let state_manager = Arc::new(
            StateManager::with_backend(StorageBackendType::Memory, PersistenceConfig::default())
                .await
                .unwrap(),
        );

        let backup_config = BackupConfig {
            backup_dir: backup_dir.clone(),
            compression_enabled: true,
            compression_type: CompressionType::Zstd,
            compression_level: 3,
            encryption_enabled: false,
            max_backups: None,
            max_backup_age: None,
            incremental_enabled: true,
            full_backup_interval: Duration::from_secs(3600),
        };

        let backup_manager =
            Arc::new(BackupManager::new(backup_config, state_manager.clone()).unwrap());

        (state_manager, backup_manager, temp_dir)
    }
    #[tokio::test]
    async fn test_basic_backup_and_restore() {
        let (state_manager, backup_manager, _temp_dir) = create_test_infrastructure().await;

        // Add test data
        state_manager
            .set(StateScope::Global, "test_key1", json!({"value": "data1"}))
            .await
            .unwrap();
        state_manager
            .set(StateScope::Global, "test_key2", json!({"value": "data2"}))
            .await
            .unwrap();

        // Create backup
        let backup_status = backup_manager.create_backup(false).await.unwrap();
        assert_eq!(backup_status.entry_count, 2);
        assert!(!backup_status.is_incremental);

        // Clear state
        state_manager.clear_scope(StateScope::Global).await.unwrap();

        // Verify state is empty
        let value = state_manager
            .get(StateScope::Global, "test_key1")
            .await
            .unwrap();
        assert!(value.is_none());

        // Restore backup
        backup_manager
            .restore_backup(&backup_status.id, RestoreOptions::default())
            .await
            .unwrap();

        // Verify data is restored
        let restored1 = state_manager
            .get(StateScope::Global, "test_key1")
            .await
            .unwrap();
        assert_eq!(restored1, Some(json!({"value": "data1"})));

        let restored2 = state_manager
            .get(StateScope::Global, "test_key2")
            .await
            .unwrap();
        assert_eq!(restored2, Some(json!({"value": "data2"})));
    }
    #[tokio::test]
    async fn test_incremental_backup_chain() {
        let (state_manager, backup_manager, _temp_dir) = create_test_infrastructure().await;

        // Initial data
        state_manager
            .set(StateScope::Global, "base_key", json!({"version": 1}))
            .await
            .unwrap();

        // Full backup
        let full_backup = backup_manager.create_backup(false).await.unwrap();
        assert!(!full_backup.is_incremental);

        // Add more data
        state_manager
            .set(
                StateScope::Global,
                "incremental_key1",
                json!({"version": 2}),
            )
            .await
            .unwrap();

        // Incremental backup 1
        let inc_backup1 = backup_manager.create_backup(true).await.unwrap();
        assert!(inc_backup1.is_incremental);
        assert_eq!(inc_backup1.parent_id, Some(full_backup.id.clone()));

        // Add more data
        state_manager
            .set(
                StateScope::Global,
                "incremental_key2",
                json!({"version": 3}),
            )
            .await
            .unwrap();

        // Incremental backup 2
        let inc_backup2 = backup_manager.create_backup(true).await.unwrap();
        assert!(inc_backup2.is_incremental);
        assert_eq!(inc_backup2.parent_id, Some(inc_backup1.id.clone()));

        // Clear everything
        state_manager.clear_scope(StateScope::Global).await.unwrap();

        // Restore from last incremental (should restore full chain)
        backup_manager
            .restore_backup(&inc_backup2.id, RestoreOptions::default())
            .await
            .unwrap();

        // Verify all data is restored
        let base = state_manager
            .get(StateScope::Global, "base_key")
            .await
            .unwrap();
        assert_eq!(base, Some(json!({"version": 1})));

        let inc1 = state_manager
            .get(StateScope::Global, "incremental_key1")
            .await
            .unwrap();
        assert_eq!(inc1, Some(json!({"version": 2})));

        let inc2 = state_manager
            .get(StateScope::Global, "incremental_key2")
            .await
            .unwrap();
        assert_eq!(inc2, Some(json!({"version": 3})));
    }
    #[tokio::test]
    async fn test_restore_with_validation() {
        let (state_manager, backup_manager, _temp_dir) = create_test_infrastructure().await;

        // Add test data
        state_manager
            .set(StateScope::Global, "validate_key", json!({"test": true}))
            .await
            .unwrap();

        // Create backup
        let backup_status = backup_manager.create_backup(false).await.unwrap();

        // Validate backup
        let validation = backup_manager
            .validate_backup(&backup_status.id)
            .await
            .unwrap();
        assert!(validation.is_valid);
        assert!(validation.checksum_valid);
        assert!(validation.integrity_valid);

        // Restore with checksum verification
        let restore_options = RestoreOptions {
            verify_checksums: true,
            backup_current: false,
            target_version: None,
            dry_run: false,
        };

        backup_manager
            .restore_backup(&backup_status.id, restore_options)
            .await
            .unwrap();
    }
    #[tokio::test]
    async fn test_dry_run_restore() {
        let (state_manager, backup_manager, _temp_dir) = create_test_infrastructure().await;

        // Add test data
        state_manager
            .set(StateScope::Global, "dry_run_key", json!({"original": true}))
            .await
            .unwrap();

        // Create backup
        let backup_status = backup_manager.create_backup(false).await.unwrap();

        // Modify data
        state_manager
            .set(StateScope::Global, "dry_run_key", json!({"modified": true}))
            .await
            .unwrap();

        // Dry run restore
        let restore_options = RestoreOptions {
            verify_checksums: true,
            backup_current: false,
            target_version: None,
            dry_run: true,
        };

        backup_manager
            .restore_backup(&backup_status.id, restore_options)
            .await
            .unwrap();

        // Verify data was NOT restored (still modified)
        let value = state_manager
            .get(StateScope::Global, "dry_run_key")
            .await
            .unwrap();
        assert_eq!(value, Some(json!({"modified": true})));
    }
    #[tokio::test]
    async fn test_backup_before_restore() {
        let (state_manager, backup_manager, _temp_dir) = create_test_infrastructure().await;

        // Initial data
        state_manager
            .set(
                StateScope::Global,
                "backup_before_key",
                json!({"version": 1}),
            )
            .await
            .unwrap();

        // Create backup 1
        let backup1 = backup_manager.create_backup(false).await.unwrap();

        // Modify data
        state_manager
            .set(
                StateScope::Global,
                "backup_before_key",
                json!({"version": 2}),
            )
            .await
            .unwrap();

        // List backups before restore
        let backups_before = backup_manager.list_backups().await.unwrap();
        let count_before = backups_before.len();

        // Restore with backup_current enabled
        let restore_options = RestoreOptions {
            verify_checksums: false,
            backup_current: true,
            target_version: None,
            dry_run: false,
        };

        backup_manager
            .restore_backup(&backup1.id, restore_options)
            .await
            .unwrap();

        // Verify data was restored to version 1
        let value = state_manager
            .get(StateScope::Global, "backup_before_key")
            .await
            .unwrap();
        assert_eq!(value, Some(json!({"version": 1})));

        // Verify a new backup was created
        let backups_after = backup_manager.list_backups().await.unwrap();
        assert_eq!(backups_after.len(), count_before + 1);
    }
    #[tokio::test]
    async fn test_restore_nonexistent_backup() {
        let (_state_manager, backup_manager, _temp_dir) = create_test_infrastructure().await;

        // Try to restore non-existent backup
        let result = backup_manager
            .restore_backup("nonexistent_backup_id", RestoreOptions::default())
            .await;

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Backup not found"));
    }
    #[tokio::test]
    async fn test_concurrent_backup_operations() {
        let (state_manager, backup_manager, _temp_dir) = create_test_infrastructure().await;

        // Add test data
        for i in 0..10 {
            state_manager
                .set(
                    StateScope::Global,
                    &format!("concurrent_key_{}", i),
                    json!({"index": i}),
                )
                .await
                .unwrap();
        }

        // Create multiple backups concurrently
        let mut backup_handles = vec![];
        for i in 0..3 {
            let backup_mgr = backup_manager.clone();
            let handle = tokio::spawn(async move {
                tokio::time::sleep(Duration::from_millis(i * 10)).await;
                backup_mgr.create_backup(false).await
            });
            backup_handles.push(handle);
        }

        // Wait for all backups to complete
        let mut backup_ids = vec![];
        for handle in backup_handles {
            let result = handle.await.unwrap().unwrap();
            backup_ids.push(result.id);
        }

        // Verify all backups were created
        assert_eq!(backup_ids.len(), 3);

        // Verify each backup can be restored
        for backup_id in backup_ids {
            let validation = backup_manager.validate_backup(&backup_id).await.unwrap();
            assert!(validation.is_valid);
        }
    }
}
