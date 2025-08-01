// ABOUTME: Backup and recovery event types for progress tracking and monitoring
// ABOUTME: Integrates with EventBus for real-time backup/restore notifications

use super::{BackupId, BackupType};
use llmspell_events::{Language, UniversalEvent};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::time::Duration;
use uuid::Uuid;

/// Backup-related events
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum BackupEvent {
    /// Backup operation started
    BackupStarted {
        backup_id: BackupId,
        backup_type: BackupType,
        parent_id: Option<BackupId>,
        correlation_id: Uuid,
    },

    /// Backup progress update
    BackupProgress {
        backup_id: BackupId,
        entries_processed: usize,
        total_entries: usize,
        bytes_processed: u64,
        compression_ratio: Option<f64>,
        correlation_id: Uuid,
    },

    /// Backup completed successfully
    BackupCompleted {
        backup_id: BackupId,
        duration: Duration,
        total_entries: usize,
        compressed_size: u64,
        original_size: u64,
        correlation_id: Uuid,
    },

    /// Backup failed
    BackupFailed {
        backup_id: BackupId,
        error: String,
        entries_processed: usize,
        correlation_id: Uuid,
    },

    /// Restore operation started
    RestoreStarted {
        backup_id: BackupId,
        restore_chain: Vec<BackupId>,
        dry_run: bool,
        correlation_id: Uuid,
    },

    /// Restore progress update
    RestoreProgress {
        backup_id: BackupId,
        current_backup_index: usize,
        total_backups: usize,
        entries_restored: usize,
        total_entries: usize,
        correlation_id: Uuid,
    },

    /// Restore completed successfully
    RestoreCompleted {
        backup_id: BackupId,
        duration: Duration,
        entries_restored: usize,
        backups_processed: usize,
        correlation_id: Uuid,
    },

    /// Restore failed
    RestoreFailed {
        backup_id: BackupId,
        error: String,
        entries_restored: usize,
        rollback_available: bool,
        correlation_id: Uuid,
    },

    /// Rollback started
    RollbackStarted {
        original_backup: BackupId,
        rollback_to: BackupId,
        correlation_id: Uuid,
    },

    /// Rollback completed
    RollbackCompleted {
        rollback_to: BackupId,
        duration: Duration,
        correlation_id: Uuid,
    },

    /// Backup validation started
    ValidationStarted {
        backup_id: BackupId,
        correlation_id: Uuid,
    },

    /// Backup validation completed
    ValidationCompleted {
        backup_id: BackupId,
        is_valid: bool,
        errors: Vec<String>,
        warnings: Vec<String>,
        correlation_id: Uuid,
    },

    /// Cleanup operation started
    CleanupStarted {
        total_backups: usize,
        retention_policy: String,
        dry_run: bool,
        correlation_id: Uuid,
    },

    /// Individual backup deleted during cleanup
    BackupDeleted {
        backup_id: BackupId,
        size_freed: u64,
        reason: String,
        correlation_id: Uuid,
    },

    /// Cleanup operation completed
    CleanupCompleted {
        evaluated_count: usize,
        deleted_count: usize,
        retained_count: usize,
        space_freed: u64,
        duration: Duration,
        correlation_id: Uuid,
    },
}

impl BackupEvent {
    /// Convert to UniversalEvent for EventBus
    pub fn to_universal_event(&self) -> UniversalEvent {
        let event_type = match self {
            Self::BackupStarted { .. } => "backup.started",
            Self::BackupProgress { .. } => "backup.progress",
            Self::BackupCompleted { .. } => "backup.completed",
            Self::BackupFailed { .. } => "backup.failed",
            Self::RestoreStarted { .. } => "restore.started",
            Self::RestoreProgress { .. } => "restore.progress",
            Self::RestoreCompleted { .. } => "restore.completed",
            Self::RestoreFailed { .. } => "restore.failed",
            Self::RollbackStarted { .. } => "rollback.started",
            Self::RollbackCompleted { .. } => "rollback.completed",
            Self::ValidationStarted { .. } => "validation.started",
            Self::ValidationCompleted { .. } => "validation.completed",
            Self::CleanupStarted { .. } => "cleanup.started",
            Self::BackupDeleted { .. } => "backup.deleted",
            Self::CleanupCompleted { .. } => "cleanup.completed",
        };

        let correlation_id = match self {
            Self::BackupStarted { correlation_id, .. }
            | Self::BackupProgress { correlation_id, .. }
            | Self::BackupCompleted { correlation_id, .. }
            | Self::BackupFailed { correlation_id, .. }
            | Self::RestoreStarted { correlation_id, .. }
            | Self::RestoreProgress { correlation_id, .. }
            | Self::RestoreCompleted { correlation_id, .. }
            | Self::RestoreFailed { correlation_id, .. }
            | Self::RollbackStarted { correlation_id, .. }
            | Self::RollbackCompleted { correlation_id, .. }
            | Self::ValidationStarted { correlation_id, .. }
            | Self::ValidationCompleted { correlation_id, .. }
            | Self::CleanupStarted { correlation_id, .. }
            | Self::BackupDeleted { correlation_id, .. }
            | Self::CleanupCompleted { correlation_id, .. } => *correlation_id,
        };

        UniversalEvent::new(event_type, json!(self), Language::Rust)
            .with_correlation_id(correlation_id)
            .with_tag("backup")
    }

    /// Get correlation ID for the event
    pub fn correlation_id(&self) -> Uuid {
        match self {
            Self::BackupStarted { correlation_id, .. }
            | Self::BackupProgress { correlation_id, .. }
            | Self::BackupCompleted { correlation_id, .. }
            | Self::BackupFailed { correlation_id, .. }
            | Self::RestoreStarted { correlation_id, .. }
            | Self::RestoreProgress { correlation_id, .. }
            | Self::RestoreCompleted { correlation_id, .. }
            | Self::RestoreFailed { correlation_id, .. }
            | Self::RollbackStarted { correlation_id, .. }
            | Self::RollbackCompleted { correlation_id, .. }
            | Self::ValidationStarted { correlation_id, .. }
            | Self::ValidationCompleted { correlation_id, .. }
            | Self::CleanupStarted { correlation_id, .. }
            | Self::BackupDeleted { correlation_id, .. }
            | Self::CleanupCompleted { correlation_id, .. } => *correlation_id,
        }
    }
}

/// Builder for creating backup events
pub struct BackupEventBuilder {
    correlation_id: Uuid,
}

impl BackupEventBuilder {
    /// Create a new event builder
    pub fn new() -> Self {
        Self {
            correlation_id: Uuid::new_v4(),
        }
    }

    /// Create with specific correlation ID
    pub fn with_correlation_id(correlation_id: Uuid) -> Self {
        Self { correlation_id }
    }

    /// Build backup started event
    pub fn backup_started(
        &self,
        backup_id: BackupId,
        backup_type: BackupType,
        parent_id: Option<BackupId>,
    ) -> BackupEvent {
        BackupEvent::BackupStarted {
            backup_id,
            backup_type,
            parent_id,
            correlation_id: self.correlation_id,
        }
    }

    /// Build backup progress event
    pub fn backup_progress(
        &self,
        backup_id: BackupId,
        entries_processed: usize,
        total_entries: usize,
        bytes_processed: u64,
        compression_ratio: Option<f64>,
    ) -> BackupEvent {
        BackupEvent::BackupProgress {
            backup_id,
            entries_processed,
            total_entries,
            bytes_processed,
            compression_ratio,
            correlation_id: self.correlation_id,
        }
    }

    /// Build restore started event
    pub fn restore_started(
        &self,
        backup_id: BackupId,
        restore_chain: Vec<BackupId>,
        dry_run: bool,
    ) -> BackupEvent {
        BackupEvent::RestoreStarted {
            backup_id,
            restore_chain,
            dry_run,
            correlation_id: self.correlation_id,
        }
    }

    /// Build restore progress event
    pub fn restore_progress(
        &self,
        backup_id: BackupId,
        current_backup_index: usize,
        total_backups: usize,
        entries_restored: usize,
        total_entries: usize,
    ) -> BackupEvent {
        BackupEvent::RestoreProgress {
            backup_id,
            current_backup_index,
            total_backups,
            entries_restored,
            total_entries,
            correlation_id: self.correlation_id,
        }
    }

    /// Build cleanup started event
    pub fn cleanup_started(
        &self,
        total_backups: usize,
        retention_policy: String,
        dry_run: bool,
    ) -> BackupEvent {
        BackupEvent::CleanupStarted {
            total_backups,
            retention_policy,
            dry_run,
            correlation_id: self.correlation_id,
        }
    }

    /// Build backup deleted event
    pub fn backup_deleted(
        &self,
        backup_id: BackupId,
        size_freed: u64,
        reason: String,
    ) -> BackupEvent {
        BackupEvent::BackupDeleted {
            backup_id,
            size_freed,
            reason,
            correlation_id: self.correlation_id,
        }
    }

    /// Build cleanup completed event
    pub fn cleanup_completed(
        &self,
        evaluated_count: usize,
        deleted_count: usize,
        retained_count: usize,
        space_freed: u64,
        duration: Duration,
    ) -> BackupEvent {
        BackupEvent::CleanupCompleted {
            evaluated_count,
            deleted_count,
            retained_count,
            space_freed,
            duration,
            correlation_id: self.correlation_id,
        }
    }
}

impl Default for BackupEventBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[cfg_attr(test_category = "state")]
mod tests {
    use super::*;

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_backup_event_to_universal() {
        let event = BackupEvent::BackupStarted {
            backup_id: "test_backup".to_string(),
            backup_type: BackupType::Full,
            parent_id: None,
            correlation_id: Uuid::new_v4(),
        };

        let universal = event.to_universal_event();
        assert_eq!(universal.event_type, "backup.started");
        assert_eq!(universal.language, Language::Rust);
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_event_builder() {
        let builder = BackupEventBuilder::new();
        let event = builder.backup_started(
            "backup_123".to_string(),
            BackupType::Incremental,
            Some("parent_123".to_string()),
        );

        match event {
            BackupEvent::BackupStarted {
                backup_id,
                backup_type,
                parent_id,
                ..
            } => {
                assert_eq!(backup_id, "backup_123");
                assert_eq!(backup_type, BackupType::Incremental);
                assert_eq!(parent_id, Some("parent_123".to_string()));
            }
            _ => panic!("Wrong event type"),
        }
    }
}
