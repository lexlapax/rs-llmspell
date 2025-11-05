//! ABOUTME: Storage migration framework (Phase 13b.14)
//! ABOUTME: Provides plan-based migration with validation and rollback

mod adapters;
mod engine;
mod plan;
mod progress;
mod traits;
mod validator;

pub use engine::MigrationEngine;
pub use plan::{BackendConfig, ComponentMigration, MigrationPlan, RollbackMetadata, ValidationRules};
pub use progress::{MigrationProgress, MigrationReport};
pub use traits::{MigrationSource, MigrationTarget};
pub use validator::{ChecksumReport, MigrationValidator, PreFlightReport, ValidationReport};

// Re-export adapters are automatically available via trait implementations
