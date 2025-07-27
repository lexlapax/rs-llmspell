// ABOUTME: Backup system module providing atomic point-in-time state snapshots
// ABOUTME: Implements non-blocking backup operations with compression and encryption

pub mod atomic;
pub mod cleanup;
pub mod compression;
pub mod events;
pub mod manager;
pub mod recovery;
pub mod retention;

pub use atomic::{AtomicBackup, AtomicBackupBuilder, BackupOperation};
pub use cleanup::{BackupCleanup, CleanupResult, CleanupScheduler};
pub use compression::{BackupCompression, CompressionAnalysis, CompressionLevel};
pub use events::{BackupEvent, BackupEventBuilder};
pub use manager::{
    BackupManager, BackupMetadata, BackupSchedule, BackupStats, BackupStatus, BackupType,
    IncrementalBackup,
};
pub use recovery::{
    AdvancedRecoveryOptions, RecoveryOrchestrator, RecoveryProgress, RecoveryState, RecoveryStatus,
};
pub use retention::{
    CompositePolicy, CountBasedPolicy, ImportanceBasedPolicy, RetentionContext, RetentionDecision,
    RetentionPolicy, RetentionPriority, RetentionReport, TimeBasedPolicy,
};

// Re-export backup config from config module
pub use crate::config::{BackupConfig, CompressionType};

use crate::error::StateError;
use serde::{Deserialize, Serialize};
use std::time::SystemTime;

/// Result type for backup operations
pub type BackupResult<T> = Result<T, StateError>;

/// Backup identifier type
pub type BackupId = String;

/// Backup validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupValidation {
    /// Whether the backup is valid
    pub is_valid: bool,

    /// Validation timestamp
    pub validated_at: SystemTime,

    /// Checksum verification result
    pub checksum_valid: bool,

    /// Data integrity verification result
    pub integrity_valid: bool,

    /// Any validation errors
    pub errors: Vec<String>,

    /// Any validation warnings
    pub warnings: Vec<String>,
}

/// Backup restore options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestoreOptions {
    /// Whether to verify checksums before restore
    pub verify_checksums: bool,

    /// Whether to create a backup of current state before restore
    pub backup_current: bool,

    /// Target state version for restore
    pub target_version: Option<String>,

    /// Whether to perform a dry run without actual restore
    pub dry_run: bool,
}

impl Default for RestoreOptions {
    fn default() -> Self {
        Self {
            verify_checksums: true,
            backup_current: true,
            target_version: None,
            dry_run: false,
        }
    }
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod retention_test;

#[cfg(test)]
mod retention_automatic_test;

#[cfg(test)]
mod simple_tests {
    use super::*;

    #[test]
    fn test_restore_options_defaults() {
        let options = RestoreOptions::default();
        assert!(options.verify_checksums);
        assert!(options.backup_current);
        assert!(!options.dry_run);
    }
}
