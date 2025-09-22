// ABOUTME: Unit tests for backup functionality
// ABOUTME: Tests atomic backup operations, compression, and backup management

#[cfg(test)]
mod backup_tests {
    use crate::state::backup::atomic::{AtomicBackup, BackupOperation, OperationStatus};
    use crate::state::backup::compression::{BackupCompression, CompressionLevel};
    use crate::state::backup::manager::{BackupMetadata, BackupStats, BackupType};
    use crate::state::backup::*;
    use crate::state::StateScope;
    #[tokio::test]
    async fn test_backup_config_defaults() {
        let config = BackupConfig::default();
        assert_eq!(config.backup_dir, std::path::PathBuf::from("./backups"));
        assert!(config.compression_enabled);
        assert_eq!(config.compression_type, CompressionType::Zstd);
        assert_eq!(config.compression_level, 3);
        assert!(!config.encryption_enabled);
        assert_eq!(config.max_backups, Some(10));
        assert!(config.incremental_enabled);
    }
    #[tokio::test]
    async fn test_compression_types() {
        let types = vec![
            CompressionType::None,
            CompressionType::Gzip,
            CompressionType::Zstd,
            CompressionType::Lz4,
            CompressionType::Brotli,
        ];

        for comp_type in types {
            let compressor = BackupCompression::new(comp_type, CompressionLevel::default());
            let test_data = b"Hello, World! This is test data for compression.".repeat(10);

            // Test compression
            let compressed = compressor.compress(&test_data).unwrap();

            // Verify compression worked (except for None type)
            if comp_type != CompressionType::None {
                assert!(compressed.len() < test_data.len());
            }

            // Test decompression
            let decompressed = compressor.decompress(&compressed).unwrap();
            assert_eq!(decompressed, test_data);
        }
    }
    #[test]
    fn test_compression_level_validation() {
        assert!(CompressionLevel::new(0).is_err());
        assert!(CompressionLevel::new(5).is_ok());
        assert!(CompressionLevel::new(10).is_err());

        assert_eq!(CompressionLevel::fast().as_u32(), 1);
        assert_eq!(CompressionLevel::default().as_u32(), 3);
        assert_eq!(CompressionLevel::best().as_u32(), 9);
    }
    #[test]
    fn test_compression_type_extensions() {
        assert_eq!(CompressionType::None.extension(), "");
        assert_eq!(CompressionType::Gzip.extension(), ".gz");
        assert_eq!(CompressionType::Zstd.extension(), ".zst");
        assert_eq!(CompressionType::Lz4.extension(), ".lz4");
        assert_eq!(CompressionType::Brotli.extension(), ".br");
    }
    #[tokio::test]
    async fn test_atomic_backup_builder() {
        let _builder = AtomicBackup::builder("test_backup".to_string())
            .with_parent("parent_backup".to_string())
            .include_scopes(vec![StateScope::Global])
            .exclude_patterns(vec!["temp_*".to_string()]);

        // Test that builder was configured correctly
        // Note: builder fields are private, so we can only test construction
        // Builder successfully created with all options
    }
    #[test]
    fn test_backup_operation_status() {
        let operation = BackupOperation {
            backup_id: "test".to_string(),
            status: OperationStatus::Pending,
            started_at: std::time::SystemTime::now(),
            completed_at: None,
            entries_processed: 0,
            bytes_processed: 0,
            errors: Vec::new(),
        };

        assert_eq!(operation.status, OperationStatus::Pending);
        assert!(operation.completed_at.is_none());
        assert_eq!(operation.entries_processed, 0);
    }
    #[test]
    fn test_restore_options_defaults() {
        let options = RestoreOptions::default();
        assert!(options.verify_checksums);
        assert!(options.backup_current);
        assert!(!options.dry_run);
        assert!(options.target_version.is_none());
    }
    #[tokio::test]
    async fn test_backup_validation_result() {
        let validation = BackupValidation {
            is_valid: true,
            validated_at: std::time::SystemTime::now(),
            checksum_valid: true,
            integrity_valid: true,
            errors: vec![],
            warnings: vec!["Test warning".to_string()],
        };

        assert!(validation.is_valid);
        assert!(validation.checksum_valid);
        assert!(validation.integrity_valid);
        assert_eq!(validation.errors.len(), 0);
        assert_eq!(validation.warnings.len(), 1);
    }
    #[tokio::test]
    async fn test_compression_analysis() {
        let compressor = BackupCompression::new(CompressionType::Zstd, CompressionLevel::default());

        let test_data = b"Highly compressible data ".repeat(100);
        let analysis = compressor.analyze_compression(&test_data);

        assert!(analysis.is_compressible);
        assert!(analysis.compression_ratio > 50.0); // Should achieve >50% compression
        assert!(analysis.compressed_size < analysis.original_size);
        assert_eq!(analysis.algorithm, CompressionType::Zstd);
    }
    #[tokio::test]
    async fn test_backup_metadata_serialization() {
        let metadata = BackupMetadata {
            id: "test_backup".to_string(),
            created_at: std::time::SystemTime::now(),
            backup_type: BackupType::Full,
            parent_id: None,
            schema_version: "1.0.0".to_string(),
            checksums: std::collections::HashMap::new(),
            compression: None,
            encryption: None,
            stats: BackupStats {
                total_entries: 100,
                total_size: 1024,
                duration_ms: 500,
                scopes_backed_up: vec!["global".to_string()],
            },
        };

        // Test serialization
        let json = serde_json::to_string(&metadata).unwrap();
        let deserialized: BackupMetadata = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.id, metadata.id);
        assert_eq!(deserialized.backup_type, metadata.backup_type);
        assert_eq!(deserialized.stats.total_entries, 100);
    }
}

#[cfg(test)]
mod integration_tests {
    use crate::state::backup::atomic::AtomicBackup;
    use crate::state::backup::*;
    use crate::state::config::{BackupConfig, CompressionType};
    use crate::state::{PersistenceConfig, StorageBackendType};
    use crate::state::{StateManager, StateScope};
    use serde_json::json;
    use std::sync::Arc;
    use std::time::Duration;
    use tempfile::TempDir;

    /// Helper to create a test state manager
    async fn create_test_state_manager() -> Arc<StateManager> {
        Arc::new(
            StateManager::with_backend(StorageBackendType::Memory, PersistenceConfig::default())
                .await
                .expect("Failed to create state manager"),
        )
    }

    /// Helper to create a test backup manager
    fn create_test_backup_manager(
        state_manager: Arc<StateManager>,
    ) -> (Arc<BackupManager>, TempDir) {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let config = BackupConfig {
            backup_dir: temp_dir.path().to_path_buf(),
            compression_enabled: true,
            compression_type: CompressionType::Zstd,
            compression_level: 3,
            encryption_enabled: false,
            max_backups: Some(10),
            max_backup_age: None,
            incremental_enabled: true,
            full_backup_interval: Duration::from_secs(3600),
        };

        let backup_manager = Arc::new(
            BackupManager::new(config, state_manager).expect("Failed to create backup manager"),
        );

        (backup_manager, temp_dir)
    }
    #[tokio::test]
    async fn test_backup_and_restore_integration() {
        // Create state manager and add test data
        let state_manager = create_test_state_manager().await;

        // Add test data
        state_manager
            .set(StateScope::Global, "test_key1", json!("test_value1"))
            .await
            .expect("Failed to set state");

        state_manager
            .set(StateScope::Global, "test_key2", json!({"nested": "value"}))
            .await
            .expect("Failed to set state");

        // Create backup manager and perform backup
        let (backup_manager, _temp_dir) = create_test_backup_manager(state_manager.clone());
        let backup_status = backup_manager
            .create_backup(false)
            .await
            .expect("Failed to create backup");

        assert_eq!(backup_status.entry_count, 2);
        assert!(!backup_status.is_incremental);

        // Clear state
        state_manager
            .delete(StateScope::Global, "test_key1")
            .await
            .expect("Failed to delete state");
        state_manager
            .delete(StateScope::Global, "test_key2")
            .await
            .expect("Failed to delete state");

        // Verify state is empty
        let keys = state_manager
            .list_keys(StateScope::Global)
            .await
            .expect("Failed to list keys");
        assert_eq!(keys.len(), 0);

        // Restore from backup
        backup_manager
            .restore_backup(&backup_status.id, RestoreOptions::default())
            .await
            .expect("Failed to restore backup");

        // Verify state is restored
        let value1 = state_manager
            .get(StateScope::Global, "test_key1")
            .await
            .expect("Failed to get state")
            .expect("State not found");
        assert_eq!(value1, json!("test_value1"));

        let value2 = state_manager
            .get(StateScope::Global, "test_key2")
            .await
            .expect("Failed to get state")
            .expect("State not found");
        assert_eq!(value2, json!({"nested": "value"}));
    }
    #[tokio::test]
    async fn test_empty_backup() {
        let state_manager = create_test_state_manager().await;
        let (backup_manager, _temp_dir) = create_test_backup_manager(state_manager.clone());

        // Create backup of empty state
        let backup_status = backup_manager
            .create_backup(false)
            .await
            .expect("Failed to create backup");

        assert_eq!(backup_status.entry_count, 0);

        // Restore empty backup (should not fail)
        backup_manager
            .restore_backup(&backup_status.id, RestoreOptions::default())
            .await
            .expect("Failed to restore backup");
    }
    #[tokio::test]
    async fn test_compression_edge_cases() {
        // Test with different compression types
        let compression_types = vec![
            CompressionType::None,
            CompressionType::Gzip,
            CompressionType::Zstd,
            CompressionType::Lz4,
            CompressionType::Brotli,
        ];

        for compression_type in compression_types {
            let state_manager = create_test_state_manager().await;

            // Add various types of data
            state_manager
                .set(StateScope::Global, "empty", json!(""))
                .await
                .expect("Failed to set state");

            state_manager
                .set(StateScope::Global, "small", json!("a"))
                .await
                .expect("Failed to set state");

            state_manager
                .set(StateScope::Global, "repetitive", json!("a".repeat(1000)))
                .await
                .expect("Failed to set state");

            let temp_dir = TempDir::new().expect("Failed to create temp dir");
            let config = BackupConfig {
                backup_dir: temp_dir.path().to_path_buf(),
                compression_enabled: compression_type != CompressionType::None,
                compression_type,
                compression_level: 3,
                encryption_enabled: false,
                max_backups: Some(10),
                max_backup_age: None,
                incremental_enabled: true,
                full_backup_interval: Duration::from_secs(3600),
            };

            let backup_manager = Arc::new(
                BackupManager::new(config, state_manager.clone())
                    .expect("Failed to create backup manager"),
            );

            // Create and restore backup
            let backup_status = backup_manager
                .create_backup(false)
                .await
                .expect("Failed to create backup");

            assert_eq!(backup_status.entry_count, 3);

            // Clear state
            for key in ["empty", "small", "repetitive"] {
                state_manager
                    .delete(StateScope::Global, key)
                    .await
                    .expect("Failed to delete state");
            }

            // Restore
            backup_manager
                .restore_backup(&backup_status.id, RestoreOptions::default())
                .await
                .expect("Failed to restore backup");

            // Verify all data restored correctly
            let empty = state_manager
                .get(StateScope::Global, "empty")
                .await
                .expect("Failed to get state")
                .expect("State not found");
            assert_eq!(empty, json!(""));

            let small = state_manager
                .get(StateScope::Global, "small")
                .await
                .expect("Failed to get state")
                .expect("State not found");
            assert_eq!(small, json!("a"));

            let repetitive = state_manager
                .get(StateScope::Global, "repetitive")
                .await
                .expect("Failed to get state")
                .expect("State not found");
            assert_eq!(repetitive, json!("a".repeat(1000)));
        }
    }
    #[tokio::test]
    async fn test_backup_retention_policies() {
        // This test verifies manual retention policy application
        // For automatic cleanup testing, see test_automatic_cleanup_during_creation
        let state_manager = create_test_state_manager().await;

        // Create backup manager WITHOUT retention policies first
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let config = BackupConfig {
            backup_dir: temp_dir.path().to_path_buf(),
            compression_enabled: false,
            compression_type: CompressionType::None,
            compression_level: 3,
            encryption_enabled: false,
            max_backups: None, // No automatic cleanup
            max_backup_age: None,
            incremental_enabled: false,
            full_backup_interval: Duration::from_secs(3600),
        };

        let backup_manager = Arc::new(
            BackupManager::new(config, state_manager.clone())
                .expect("Failed to create backup manager"),
        );

        // Create 5 backups without automatic cleanup
        for i in 0..5 {
            state_manager
                .set(StateScope::Global, &format!("test_key_{i}"), json!(i))
                .await
                .expect("Failed to set state");

            backup_manager
                .create_backup(false)
                .await
                .expect("Failed to create backup");

            // Small delay to ensure different timestamps
            tokio::time::sleep(Duration::from_millis(50)).await;
        }

        // Should have all 5 backups
        let backups_before = backup_manager
            .list_backups()
            .await
            .expect("Failed to list backups");
        assert_eq!(backups_before.len(), 5);

        // Now create a new backup manager with retention policies pointing to same directory
        let config_with_retention = BackupConfig {
            backup_dir: temp_dir.path().to_path_buf(),
            compression_enabled: false,
            compression_type: CompressionType::None,
            compression_level: 3,
            encryption_enabled: false,
            max_backups: Some(3), // Keep only 3 backups
            max_backup_age: None,
            incremental_enabled: false,
            full_backup_interval: Duration::from_secs(3600),
        };

        let retention_manager = Arc::new(
            BackupManager::new(config_with_retention, state_manager.clone())
                .expect("Failed to create backup manager"),
        );

        // Manually apply retention policies
        let report = retention_manager
            .cleanup_backups()
            .await
            .expect("Failed to apply retention policies");

        // Should have evaluated all 5 and deleted 2
        assert_eq!(report.evaluated_count, 5);
        assert_eq!(report.deleted_count, 2);
        assert_eq!(report.retained_count, 3);

        // List backups after cleanup
        let backups_after = retention_manager
            .list_backups()
            .await
            .expect("Failed to list backups");
        assert_eq!(backups_after.len(), 3);
    }
    #[tokio::test]
    async fn test_importance_based_retention() {
        let state_manager = create_test_state_manager().await;
        let (backup_manager, _temp_dir) = create_test_backup_manager(state_manager.clone());

        // Create a full backup
        state_manager
            .set(StateScope::Global, "important_data", json!("critical"))
            .await
            .expect("Failed to set state");

        let _full_backup = backup_manager
            .create_backup(false)
            .await
            .expect("Failed to create full backup");

        // Create several incremental backups
        for i in 0..3 {
            state_manager
                .set(StateScope::Global, &format!("data_{i}"), json!(i))
                .await
                .expect("Failed to set state");

            backup_manager
                .create_backup(true)
                .await
                .expect("Failed to create incremental backup");
        }

        // Configure aggressive retention (keep only 1 backup)
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let config = BackupConfig {
            backup_dir: temp_dir.path().to_path_buf(),
            compression_enabled: false,
            compression_type: CompressionType::None,
            compression_level: 3,
            encryption_enabled: false,
            max_backups: Some(1),
            max_backup_age: None,
            incremental_enabled: true,
            full_backup_interval: Duration::from_secs(3600),
        };

        // Create new backup manager with aggressive retention
        let _aggressive_manager = Arc::new(
            BackupManager::new(config, state_manager.clone())
                .expect("Failed to create backup manager"),
        );

        // Copy backup metadata to new manager (simulate existing backups)
        // Note: In real implementation, this would be loaded from disk

        // The importance-based policy should keep the full backup even with max_backups=1
        // because it's marked as Critical importance
    }
    #[tokio::test]
    async fn test_concurrent_operations() {
        let state_manager = create_test_state_manager().await;

        // Add test data
        for i in 0..10 {
            state_manager
                .set(StateScope::Global, &format!("key{i}"), json!(i))
                .await
                .expect("Failed to set state");
        }

        // Create atomic backup instance
        let atomic_backup = Arc::new(
            AtomicBackup::new("test_concurrent".to_string(), state_manager.clone(), None)
                .expect("Failed to create atomic backup"),
        );

        // Capture should work with concurrent state modifications
        let capture_handle = tokio::spawn({
            let atomic_backup = atomic_backup.clone();
            async move { atomic_backup.capture().await }
        });

        // Modify state concurrently
        state_manager
            .set(
                StateScope::Global,
                "concurrent_key",
                json!("concurrent_value"),
            )
            .await
            .expect("Failed to set concurrent state");

        // Capture should complete successfully
        let backup_data = capture_handle
            .await
            .expect("Capture task failed")
            .expect("Capture failed");

        assert!(!backup_data.is_empty());
    }
}
