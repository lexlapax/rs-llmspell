// ABOUTME: Test automatic cleanup during backup creation
// ABOUTME: Verifies that max_backups limit is enforced automatically

#[cfg(test)]
mod test {
    use crate::state::backup::*;
    use crate::state::config::BackupConfig;
    use crate::state::{PersistenceConfig, StorageBackendType};
    use crate::state::{StateManager, StateScope};
    use serde_json::json;
    use std::sync::Arc;
    use std::time::Duration;
    use tempfile::TempDir;
    #[tokio::test]
    async fn test_automatic_cleanup_during_creation() {
        // Create state manager
        let state_manager = Arc::new(
            StateManager::with_backend(StorageBackendType::Memory, PersistenceConfig::default())
                .await
                .expect("Failed to create state manager"),
        );

        // Create backup manager with max_backups=2
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let config = BackupConfig {
            backup_dir: temp_dir.path().to_path_buf(),
            compression_enabled: false,
            compression_type: CompressionType::None,
            compression_level: 3,
            encryption_enabled: false,
            max_backups: Some(2), // Keep only 2 backups
            max_backup_age: None,
            incremental_enabled: false,
            full_backup_interval: Duration::from_secs(3600),
        };

        let backup_manager = Arc::new(
            BackupManager::new(config, state_manager.clone())
                .expect("Failed to create backup manager"),
        );

        // Create 3 backups
        for i in 0..3 {
            state_manager
                .set(StateScope::Global, &format!("key_{i}"), json!(i))
                .await
                .expect("Failed to set state");

            backup_manager
                .create_backup(false)
                .await
                .expect("Failed to create backup");

            // Small delay to ensure different timestamps
            tokio::time::sleep(Duration::from_millis(50)).await;
        }

        // List backups - should only have 2 (automatic cleanup)
        let backups = backup_manager
            .list_backups()
            .await
            .expect("Failed to list backups");

        assert_eq!(
            backups.len(),
            2,
            "Automatic cleanup should maintain max_backups=2"
        );

        // Verify the two newest backups were kept
        // (backups are sorted newest first)
        assert!(backups[0].created_at > backups[1].created_at);
    }
}
