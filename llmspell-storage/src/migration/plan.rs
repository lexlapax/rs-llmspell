//! ABOUTME: Migration plan YAML format
//! ABOUTME: Declarative migration configuration with validation rules and rollback metadata

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Migration plan - declarative configuration for storage migration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationPlan {
    /// Plan format version
    pub version: String,

    /// Plan creation timestamp
    pub created_at: DateTime<Utc>,

    /// Source backend configuration
    pub source: BackendConfig,

    /// Target backend configuration
    pub target: BackendConfig,

    /// Components to migrate
    pub components: Vec<ComponentMigration>,

    /// Validation rules
    pub validation: ValidationRules,

    /// Rollback metadata
    pub rollback: RollbackMetadata,
}

impl MigrationPlan {
    /// Create new migration plan
    pub fn new(source_backend: &str, target_backend: &str, component_names: Vec<String>) -> Self {
        let components = component_names
            .into_iter()
            .map(|name| ComponentMigration {
                name,
                estimated_count: 0, // Will be populated during plan generation
                batch_size: 1000,   // Default batch size for Phase 1
            })
            .collect();

        Self {
            version: "1.0".to_string(),
            created_at: Utc::now(),
            source: BackendConfig {
                backend: source_backend.to_string(),
                path: None,
                connection: None,
            },
            target: BackendConfig {
                backend: target_backend.to_string(),
                path: None,
                connection: None,
            },
            components,
            validation: ValidationRules {
                checksum_sample_percent: 10,
                full_comparison_threshold: 100,
            },
            rollback: RollbackMetadata {
                backup_enabled: true,
                backup_path: None, // Will be set by BackupManager
            },
        }
    }

    /// Load plan from YAML file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let plan: MigrationPlan = serde_yaml::from_str(&content)?;
        Ok(plan)
    }

    /// Save plan to YAML file
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let yaml = serde_yaml::to_string(self)?;
        std::fs::write(path, yaml)?;
        Ok(())
    }
}

/// Backend configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackendConfig {
    /// Backend type ("sled", "postgres", "memory")
    pub backend: String,

    /// Path for file-based backends (e.g., Sled database path)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,

    /// Connection string for network backends (e.g., PostgreSQL connection URL)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub connection: Option<String>,
}

/// Component migration configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentMigration {
    /// Component name ("agent_state", "workflow_state", "sessions")
    pub name: String,

    /// Estimated record count (populated during plan generation)
    pub estimated_count: usize,

    /// Batch size for migration (number of records per batch)
    pub batch_size: usize,
}

/// Validation rules for migration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationRules {
    /// Percentage of records to sample for checksum validation (0-100)
    pub checksum_sample_percent: u8,

    /// Threshold for full data comparison (if count < threshold, do full comparison)
    pub full_comparison_threshold: usize,
}

/// Rollback metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackMetadata {
    /// Whether backup is enabled before migration
    pub backup_enabled: bool,

    /// Backup path (set by BackupManager during execution)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub backup_path: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plan_creation() {
        let plan = MigrationPlan::new(
            "sled",
            "postgres",
            vec![
                "agent_state".to_string(),
                "workflow_state".to_string(),
                "sessions".to_string(),
            ],
        );

        assert_eq!(plan.version, "1.0");
        assert_eq!(plan.source.backend, "sled");
        assert_eq!(plan.target.backend, "postgres");
        assert_eq!(plan.components.len(), 3);
        assert_eq!(plan.components[0].name, "agent_state");
        assert_eq!(plan.validation.checksum_sample_percent, 10);
        assert!(plan.rollback.backup_enabled);
    }

    #[test]
    fn test_plan_serialization() {
        let plan = MigrationPlan::new("sled", "postgres", vec!["agent_state".to_string()]);

        let yaml = serde_yaml::to_string(&plan).unwrap();
        assert!(yaml.contains("version: '1.0'"));
        assert!(yaml.contains("backend: sled"));
        assert!(yaml.contains("backend: postgres"));
        assert!(yaml.contains("agent_state"));

        let deserialized: MigrationPlan = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(deserialized.version, plan.version);
        assert_eq!(deserialized.components.len(), plan.components.len());
    }
}
