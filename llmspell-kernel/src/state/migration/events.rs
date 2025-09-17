// ABOUTME: Migration-specific events that integrate with existing event correlation system
// ABOUTME: Defines migration events for timeline reconstruction and correlation tracking

use crate::state::schema::SemanticVersion;
use chrono::Utc;
use llmspell_events::{EventMetadata, Language, UniversalEvent};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use uuid::Uuid;

/// Migration-specific events that integrate with existing event correlation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum MigrationEvent {
    /// Migration started with version information
    MigrationStarted {
        migration_id: Uuid,
        from_version: SemanticVersion,
        to_version: SemanticVersion,
        total_steps: usize,
        dry_run: bool,
    },

    /// Migration step started
    StepStarted {
        migration_id: Uuid,
        step_index: usize,
        step_description: String,
    },

    /// Migration step completed
    StepCompleted {
        migration_id: Uuid,
        step_index: usize,
        items_processed: usize,
        duration: Duration,
    },

    /// Migration step failed
    StepFailed {
        migration_id: Uuid,
        step_index: usize,
        error: String,
        items_processed: usize,
    },

    /// Migration completed successfully
    MigrationCompleted {
        migration_id: Uuid,
        from_version: SemanticVersion,
        to_version: SemanticVersion,
        total_duration: Duration,
        items_migrated: usize,
        steps_completed: usize,
    },

    /// Migration failed with details
    MigrationFailed {
        migration_id: Uuid,
        from_version: SemanticVersion,
        to_version: SemanticVersion,
        error: String,
        items_processed: usize,
        rollback_initiated: bool,
    },

    /// Migration rollback started
    RollbackStarted {
        migration_id: Uuid,
        original_version: SemanticVersion,
        failed_version: SemanticVersion,
    },

    /// Migration rollback completed
    RollbackCompleted {
        migration_id: Uuid,
        restored_version: SemanticVersion,
        duration: Duration,
        items_restored: usize,
    },

    /// Migration rollback failed
    RollbackFailed {
        migration_id: Uuid,
        error: String,
        items_restored: usize,
    },

    /// Migration validation event
    ValidationEvent {
        migration_id: Uuid,
        phase: ValidationPhase,
        result: ValidationResult,
        items_validated: usize,
        duration: Duration,
    },

    /// Migration backup event
    BackupEvent {
        migration_id: Uuid,
        backup_type: BackupType,
        backup_size_bytes: usize,
        duration: Duration,
    },
}

/// Validation phases during migration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ValidationPhase {
    PreMigration,
    PostMigration,
    PostRollback,
}

/// Validation results
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ValidationResult {
    Passed,
    Warning { message: String },
    Failed { message: String },
}

/// Backup types for migration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum BackupType {
    Full,
    Incremental,
    Schema,
}

impl MigrationEvent {
    /// Get the migration ID from any migration event
    pub fn migration_id(&self) -> Uuid {
        match self {
            MigrationEvent::MigrationStarted { migration_id, .. }
            | MigrationEvent::StepStarted { migration_id, .. }
            | MigrationEvent::StepCompleted { migration_id, .. }
            | MigrationEvent::StepFailed { migration_id, .. }
            | MigrationEvent::MigrationCompleted { migration_id, .. }
            | MigrationEvent::MigrationFailed { migration_id, .. }
            | MigrationEvent::RollbackStarted { migration_id, .. }
            | MigrationEvent::RollbackCompleted { migration_id, .. }
            | MigrationEvent::RollbackFailed { migration_id, .. }
            | MigrationEvent::ValidationEvent { migration_id, .. }
            | MigrationEvent::BackupEvent { migration_id, .. } => *migration_id,
        }
    }

    /// Get event type as string for correlation
    pub fn event_type(&self) -> &'static str {
        match self {
            MigrationEvent::MigrationStarted { .. } => "migration_started",
            MigrationEvent::StepStarted { .. } => "migration_step_started",
            MigrationEvent::StepCompleted { .. } => "migration_step_completed",
            MigrationEvent::StepFailed { .. } => "migration_step_failed",
            MigrationEvent::MigrationCompleted { .. } => "migration_completed",
            MigrationEvent::MigrationFailed { .. } => "migration_failed",
            MigrationEvent::RollbackStarted { .. } => "migration_rollback_started",
            MigrationEvent::RollbackCompleted { .. } => "migration_rollback_completed",
            MigrationEvent::RollbackFailed { .. } => "migration_rollback_failed",
            MigrationEvent::ValidationEvent { .. } => "migration_validation",
            MigrationEvent::BackupEvent { .. } => "migration_backup",
        }
    }

    /// Check if this is an error event
    pub fn is_error(&self) -> bool {
        matches!(
            self,
            MigrationEvent::StepFailed { .. }
                | MigrationEvent::MigrationFailed { .. }
                | MigrationEvent::RollbackFailed { .. }
                | MigrationEvent::ValidationEvent {
                    result: ValidationResult::Failed { .. },
                    ..
                }
        )
    }

    /// Check if this is a completion event (success or failure)
    pub fn is_completion(&self) -> bool {
        matches!(
            self,
            MigrationEvent::MigrationCompleted { .. }
                | MigrationEvent::MigrationFailed { .. }
                | MigrationEvent::RollbackCompleted { .. }
                | MigrationEvent::RollbackFailed { .. }
        )
    }

    /// Create event metadata for correlation tracking
    pub fn create_metadata(&self, correlation_id: Uuid) -> EventMetadata {
        let mut tags = vec!["migration".to_string(), self.event_type().to_string()];

        // Add event-specific tags
        match self {
            MigrationEvent::MigrationStarted { dry_run, .. } => {
                if *dry_run {
                    tags.push("dry_run".to_string());
                }
            }
            MigrationEvent::MigrationFailed {
                rollback_initiated, ..
            } => {
                tags.push("failed".to_string());
                if *rollback_initiated {
                    tags.push("rollback".to_string());
                }
            }
            MigrationEvent::StepFailed { .. } => {
                tags.push("step_failed".to_string());
            }
            _ => {}
        }

        EventMetadata {
            correlation_id,
            source: Some("migration_engine".to_string()),
            target: None,
            tags,
            priority: 1, // High priority for migration events
            ttl: None,   // No expiration for migration events
        }
    }
}

/// Convert `MigrationEvent` to `UniversalEvent` for existing event system integration
impl From<MigrationEvent> for UniversalEvent {
    fn from(migration_event: MigrationEvent) -> Self {
        let event_type = migration_event.event_type().to_string();

        UniversalEvent {
            id: Uuid::new_v4(),
            event_type,
            data: serde_json::to_value(&migration_event).unwrap_or_default(),
            language: Language::Rust,
            timestamp: Utc::now(),
            sequence: 0,                     // Will be set by event bus
            schema_version: "1".to_string(), // Migration event schema version
            metadata: llmspell_events::EventMetadata {
                correlation_id: migration_event.migration_id(),
                source: Some("migration_engine".to_string()),
                target: None,
                tags: vec!["migration".to_string()],
                priority: 1,
                ttl: None,
            },
        }
    }
}

/// Migration event builder for convenient event creation
pub struct MigrationEventBuilder {
    migration_id: Uuid,
}

impl MigrationEventBuilder {
    pub fn new(migration_id: Uuid) -> Self {
        Self { migration_id }
    }

    pub fn migration_started(
        &self,
        from_version: SemanticVersion,
        to_version: SemanticVersion,
        total_steps: usize,
        dry_run: bool,
    ) -> MigrationEvent {
        MigrationEvent::MigrationStarted {
            migration_id: self.migration_id,
            from_version,
            to_version,
            total_steps,
            dry_run,
        }
    }

    pub fn step_started(&self, step_index: usize, step_description: String) -> MigrationEvent {
        MigrationEvent::StepStarted {
            migration_id: self.migration_id,
            step_index,
            step_description,
        }
    }

    pub fn step_completed(
        &self,
        step_index: usize,
        items_processed: usize,
        duration: Duration,
    ) -> MigrationEvent {
        MigrationEvent::StepCompleted {
            migration_id: self.migration_id,
            step_index,
            items_processed,
            duration,
        }
    }

    pub fn step_failed(
        &self,
        step_index: usize,
        error: String,
        items_processed: usize,
    ) -> MigrationEvent {
        MigrationEvent::StepFailed {
            migration_id: self.migration_id,
            step_index,
            error,
            items_processed,
        }
    }

    pub fn migration_completed(
        &self,
        from_version: SemanticVersion,
        to_version: SemanticVersion,
        total_duration: Duration,
        items_migrated: usize,
        steps_completed: usize,
    ) -> MigrationEvent {
        MigrationEvent::MigrationCompleted {
            migration_id: self.migration_id,
            from_version,
            to_version,
            total_duration,
            items_migrated,
            steps_completed,
        }
    }

    pub fn migration_failed(
        &self,
        from_version: SemanticVersion,
        to_version: SemanticVersion,
        error: String,
        items_processed: usize,
        rollback_initiated: bool,
    ) -> MigrationEvent {
        MigrationEvent::MigrationFailed {
            migration_id: self.migration_id,
            from_version,
            to_version,
            error,
            items_processed,
            rollback_initiated,
        }
    }

    pub fn validation_event(
        &self,
        phase: ValidationPhase,
        result: ValidationResult,
        items_validated: usize,
        duration: Duration,
    ) -> MigrationEvent {
        MigrationEvent::ValidationEvent {
            migration_id: self.migration_id,
            phase,
            result,
            items_validated,
            duration,
        }
    }

    pub fn backup_event(
        &self,
        backup_type: BackupType,
        backup_size_bytes: usize,
        duration: Duration,
    ) -> MigrationEvent {
        MigrationEvent::BackupEvent {
            migration_id: self.migration_id,
            backup_type,
            backup_size_bytes,
            duration,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_migration_event_creation() {
        let migration_id = Uuid::new_v4();
        let builder = MigrationEventBuilder::new(migration_id);

        let event = builder.migration_started(
            SemanticVersion::new(1, 0, 0),
            SemanticVersion::new(2, 0, 0),
            5,
            false,
        );

        assert_eq!(event.migration_id(), migration_id);
        assert_eq!(event.event_type(), "migration_started");
        assert!(!event.is_error());
        assert!(!event.is_completion());
    }
    #[test]
    fn test_migration_event_error_classification() {
        let migration_id = Uuid::new_v4();
        let builder = MigrationEventBuilder::new(migration_id);

        let error_event = builder.step_failed(1, "Test error".to_string(), 10);
        assert!(error_event.is_error());
        assert!(!error_event.is_completion());

        let completion_event = builder.migration_completed(
            SemanticVersion::new(1, 0, 0),
            SemanticVersion::new(2, 0, 0),
            Duration::from_secs(60),
            100,
            5,
        );
        assert!(!completion_event.is_error());
        assert!(completion_event.is_completion());
    }
    #[test]
    fn test_universal_event_conversion() {
        let migration_id = Uuid::new_v4();
        let builder = MigrationEventBuilder::new(migration_id);

        let migration_event = builder.migration_started(
            SemanticVersion::new(1, 0, 0),
            SemanticVersion::new(2, 0, 0),
            3,
            true,
        );

        let universal_event: UniversalEvent = migration_event.into();
        assert_eq!(universal_event.event_type, "migration_started");
        assert_eq!(
            universal_event.metadata.source,
            Some("migration_engine".to_string())
        );
        assert_eq!(universal_event.metadata.correlation_id, migration_id);
    }
    #[test]
    fn test_event_metadata_creation() {
        let migration_id = Uuid::new_v4();
        let correlation_id = Uuid::new_v4();
        let builder = MigrationEventBuilder::new(migration_id);

        let event = builder.migration_started(
            SemanticVersion::new(1, 0, 0),
            SemanticVersion::new(2, 0, 0),
            5,
            false,
        );

        let metadata = event.create_metadata(correlation_id);
        assert_eq!(metadata.correlation_id, correlation_id);
        assert_eq!(metadata.source, Some("migration_engine".to_string()));
        assert!(metadata.tags.contains(&"migration".to_string()));
        assert!(metadata.tags.contains(&"migration_started".to_string()));
    }
}
