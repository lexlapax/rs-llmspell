// ABOUTME: Unit tests for backup functionality
// ABOUTME: Tests atomic backup operations, compression, and backup management

#[cfg(test)]
mod backup_tests {
    use crate::backup::atomic::{AtomicBackup, BackupOperation, OperationStatus};
    use crate::backup::compression::{BackupCompression, CompressionLevel};
    use crate::backup::manager::{BackupMetadata, BackupStats, BackupType};
    use crate::backup::*;
    use crate::StateScope;

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
