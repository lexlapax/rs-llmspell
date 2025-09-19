// ABOUTME: State migration framework for schema version transitions
// ABOUTME: Integrates with existing StateManager, storage adapters, and hook system

pub mod engine;
pub mod events;
pub mod planner;
pub mod transforms;
pub mod validator;

pub use engine::{MigrationEngine, MigrationEngineError, MigrationExecutor};
pub use events::{
    BackupType, MigrationEvent, MigrationEventBuilder, ValidationPhase,
    ValidationResult as EventValidationResult,
};
pub use planner::{MigrationPlan, MigrationPlanner, MigrationStep as NewMigrationStep};
pub use transforms::{DataTransformer, FieldTransform, StateTransformation, TransformationError};
pub use validator::{MigrationValidator, ValidationResult, ValidationRules};

use crate::state::schema::{SchemaRegistry, SemanticVersion};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Migration execution status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum MigrationStatus {
    NotStarted,
    InProgress,
    Completed,
    Failed(String),
    RolledBack,
}

/// Migration execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationResult {
    pub status: MigrationStatus,
    pub from_version: SemanticVersion,
    pub to_version: SemanticVersion,
    pub steps_completed: usize,
    pub total_steps: usize,
    pub items_migrated: usize,
    pub duration: std::time::Duration,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
}

impl MigrationResult {
    pub fn new(
        from_version: SemanticVersion,
        to_version: SemanticVersion,
        total_steps: usize,
    ) -> Self {
        Self {
            status: MigrationStatus::NotStarted,
            from_version,
            to_version,
            steps_completed: 0,
            total_steps,
            items_migrated: 0,
            duration: std::time::Duration::from_secs(0),
            warnings: Vec::new(),
            errors: Vec::new(),
        }
    }

    pub fn mark_in_progress(&mut self) {
        self.status = MigrationStatus::InProgress;
    }

    pub fn mark_completed(&mut self, items_migrated: usize, duration: std::time::Duration) {
        self.status = MigrationStatus::Completed;
        self.items_migrated = items_migrated;
        self.duration = duration;
        self.steps_completed = self.total_steps;
    }

    pub fn mark_failed(&mut self, error: String) {
        self.status = MigrationStatus::Failed(error);
    }

    pub fn mark_rolled_back(&mut self) {
        self.status = MigrationStatus::RolledBack;
    }

    pub fn add_warning(&mut self, warning: String) {
        self.warnings.push(warning);
    }

    pub fn add_error(&mut self, error: String) {
        self.errors.push(error);
    }

    pub fn is_successful(&self) -> bool {
        matches!(self.status, MigrationStatus::Completed)
    }

    pub fn is_failed(&self) -> bool {
        matches!(self.status, MigrationStatus::Failed(_))
    }
}

/// Migration configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationConfig {
    pub dry_run: bool,
    pub create_backup: bool,
    pub batch_size: usize,
    pub max_concurrent_migrations: usize,
    pub validation_level: ValidationLevel,
    pub rollback_on_error: bool,
    pub timeout: std::time::Duration,
}

impl Default for MigrationConfig {
    fn default() -> Self {
        Self {
            dry_run: false,
            create_backup: true,
            batch_size: 100,
            max_concurrent_migrations: 1,
            validation_level: ValidationLevel::Strict,
            rollback_on_error: true,
            timeout: std::time::Duration::from_secs(300), // 5 minutes
        }
    }
}

/// Validation level for migrations
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ValidationLevel {
    None,     // No validation
    Basic,    // Basic schema validation
    Strict,   // Full validation including data integrity
    Paranoid, // Maximum validation with backups
}

/// Migration context for tracking progress
#[derive(Debug, Clone)]
pub struct MigrationContext {
    pub config: MigrationConfig,
    pub schema_registry: SchemaRegistry,
    pub current_step: usize,
    pub total_steps: usize,
    pub start_time: std::time::Instant,
    pub metadata: HashMap<String, serde_json::Value>,
}

impl MigrationContext {
    pub fn new(
        config: MigrationConfig,
        schema_registry: SchemaRegistry,
        total_steps: usize,
    ) -> Self {
        Self {
            config,
            schema_registry,
            current_step: 0,
            total_steps,
            start_time: std::time::Instant::now(),
            metadata: HashMap::new(),
        }
    }

    pub fn progress(&self) -> f64 {
        if self.total_steps == 0 {
            0.0
        } else {
            #[allow(clippy::cast_precision_loss)]
            let current_step_f64 = self.current_step as f64;
            #[allow(clippy::cast_precision_loss)]
            let total_steps_f64 = self.total_steps as f64;
            current_step_f64 / total_steps_f64
        }
    }

    pub fn elapsed_time(&self) -> std::time::Duration {
        self.start_time.elapsed()
    }

    pub fn set_metadata(&mut self, key: String, value: serde_json::Value) {
        self.metadata.insert(key, value);
    }

    pub fn get_metadata(&self, key: &str) -> Option<&serde_json::Value> {
        self.metadata.get(key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_migration_result_lifecycle() {
        let from_version = SemanticVersion::new(1, 0, 0);
        let to_version = SemanticVersion::new(2, 0, 0);
        let mut result = MigrationResult::new(from_version.clone(), to_version.clone(), 3);

        assert_eq!(result.status, MigrationStatus::NotStarted);
        assert_eq!(result.from_version, from_version);
        assert_eq!(result.to_version, to_version);
        assert_eq!(result.total_steps, 3);

        result.mark_in_progress();
        assert_eq!(result.status, MigrationStatus::InProgress);

        result.add_warning("Test warning".to_string());
        assert_eq!(result.warnings.len(), 1);

        result.mark_completed(50, std::time::Duration::from_secs(10));
        assert!(result.is_successful());
        assert_eq!(result.items_migrated, 50);
        assert_eq!(result.steps_completed, 3);
    }
    #[test]
    fn test_migration_context() {
        let config = MigrationConfig::default();
        let schema_registry = SchemaRegistry::new();
        let context = MigrationContext::new(config, schema_registry, 5);

        assert!((context.progress() - 0.0).abs() < f64::EPSILON);
        assert_eq!(context.current_step, 0);
        assert_eq!(context.total_steps, 5);
    }
    #[test]
    fn test_migration_config_defaults() {
        let config = MigrationConfig::default();

        assert!(!config.dry_run);
        assert!(config.create_backup);
        assert_eq!(config.batch_size, 100);
        assert_eq!(config.max_concurrent_migrations, 1);
        assert_eq!(config.validation_level, ValidationLevel::Strict);
        assert!(config.rollback_on_error);
    }
}
