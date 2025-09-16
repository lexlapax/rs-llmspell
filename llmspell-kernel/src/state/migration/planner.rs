// ABOUTME: Migration planner integration with existing schema system
// ABOUTME: Provides high-level migration planning interface for StateManager integration

use super::super::config::MigrationStep as LegacyMigrationStep;
use crate::state::schema::{
    CompatibilityChecker, CompatibilityResult, EnhancedStateSchema,
    MigrationPlan as SchemaMigrationPlan, MigrationPlanner as SchemaMigrationPlanner,
    SchemaRegistry, SemanticVersion,
};
use crate::state::{StateError, StateResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum MigrationPlannerError {
    #[error("Planning failed: {reason}")]
    PlanningFailed { reason: String },

    #[error("Schema incompatible: {details}")]
    IncompatibleSchema { details: String },

    #[error("Migration path not found: {from} -> {to}")]
    NoMigrationPath {
        from: SemanticVersion,
        to: SemanticVersion,
    },
}

impl From<MigrationPlannerError> for StateError {
    fn from(err: MigrationPlannerError) -> Self {
        StateError::MigrationError(err.to_string())
    }
}

/// Enhanced migration step with more detail than legacy MigrationStep
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationStep {
    pub id: String,
    pub from_version: SemanticVersion,
    pub to_version: SemanticVersion,
    pub migration_type: String,
    pub description: String,
    pub estimated_duration: Duration,
    pub risk_level: crate::state::schema::compatibility::RiskLevel,
    pub requires_backup: bool,
    pub validation_rules: Vec<String>,
}

impl MigrationStep {
    pub fn from_legacy(legacy: &LegacyMigrationStep) -> Self {
        Self {
            id: format!("{}_{}", legacy.from_version, legacy.to_version),
            from_version: SemanticVersion::new(legacy.from_version, 0, 0),
            to_version: SemanticVersion::new(legacy.to_version, 0, 0),
            migration_type: legacy.migration_type.clone(),
            description: legacy.description.clone(),
            estimated_duration: Duration::from_secs(60), // Default estimate
            risk_level: crate::state::schema::compatibility::RiskLevel::Medium,
            requires_backup: legacy.migration_type.contains("breaking"),
            validation_rules: vec!["basic_validation".to_string()],
        }
    }

    pub fn to_legacy(&self) -> LegacyMigrationStep {
        LegacyMigrationStep {
            from_version: self.from_version.major,
            to_version: self.to_version.major,
            migration_type: self.migration_type.clone(),
            description: self.description.clone(),
        }
    }
}

/// Enhanced migration plan that integrates with StateManager
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationPlan {
    pub id: String,
    pub from_version: SemanticVersion,
    pub to_version: SemanticVersion,
    pub steps: Vec<MigrationStep>,
    pub estimated_duration: Duration,
    pub total_risk_level: crate::state::schema::compatibility::RiskLevel,
    pub requires_backup: bool,
    pub compatibility_analysis: CompatibilityResult,
    pub warnings: Vec<String>,
    pub metadata: HashMap<String, serde_json::Value>,
    pub created_at: std::time::SystemTime,
}

impl MigrationPlan {
    pub fn from_schema_plan(
        schema_plan: SchemaMigrationPlan,
        compatibility: CompatibilityResult,
    ) -> Self {
        let steps: Vec<MigrationStep> = schema_plan
            .steps
            .iter()
            .map(|step| MigrationStep {
                id: format!("{}_{}", step.from_version, step.to_version),
                from_version: SemanticVersion::new(step.from_version, 0, 0),
                to_version: SemanticVersion::new(step.to_version, 0, 0),
                migration_type: step.migration_type.clone(),
                description: step.description.clone(),
                estimated_duration: Duration::from_secs(60),
                risk_level: schema_plan.risk_level.clone(),
                requires_backup: schema_plan.requires_backup,
                validation_rules: vec!["schema_validation".to_string()],
            })
            .collect();

        Self {
            id: uuid::Uuid::new_v4().to_string(),
            from_version: schema_plan.from_version,
            to_version: schema_plan.to_version,
            steps,
            estimated_duration: schema_plan.estimated_duration,
            total_risk_level: schema_plan.risk_level,
            requires_backup: schema_plan.requires_backup,
            compatibility_analysis: compatibility,
            warnings: schema_plan.warnings,
            metadata: HashMap::new(),
            created_at: std::time::SystemTime::now(),
        }
    }

    pub fn is_safe(&self) -> bool {
        self.total_risk_level <= crate::state::schema::compatibility::RiskLevel::Medium
            && self.compatibility_analysis.compatible
    }

    pub fn has_breaking_changes(&self) -> bool {
        !self.compatibility_analysis.breaking_changes.is_empty()
    }

    pub fn get_affected_fields(&self) -> Vec<String> {
        self.compatibility_analysis
            .field_changes
            .keys()
            .cloned()
            .collect()
    }
}

/// Migration planner that integrates with StateManager
pub struct MigrationPlanner {
    schema_planner: SchemaMigrationPlanner,
    schema_registry: SchemaRegistry,
}

impl MigrationPlanner {
    pub fn new() -> Self {
        Self {
            schema_planner: SchemaMigrationPlanner::new(),
            schema_registry: SchemaRegistry::new(),
        }
    }

    pub fn with_registry(schema_registry: SchemaRegistry) -> Self {
        Self {
            schema_planner: SchemaMigrationPlanner::new(),
            schema_registry,
        }
    }

    /// Register a schema for migration planning
    pub fn register_schema(&mut self, schema: EnhancedStateSchema) -> StateResult<()> {
        // Register with both the schema registry and planner
        self.schema_registry
            .register_schema(schema.clone(), None)
            .map_err(|e| StateError::MigrationError(e.to_string()))?;

        self.schema_planner.register_schema(schema);
        Ok(())
    }

    /// Create a migration plan between versions
    pub fn create_migration_plan(
        &mut self,
        from_version: &SemanticVersion,
        to_version: &SemanticVersion,
    ) -> StateResult<MigrationPlan> {
        // Get schemas for compatibility analysis
        let from_schema = self
            .schema_registry
            .get_schema(from_version)
            .ok_or_else(|| MigrationPlannerError::PlanningFailed {
                reason: format!("Source schema {} not found", from_version),
            })?;

        let to_schema = self.schema_registry.get_schema(to_version).ok_or_else(|| {
            MigrationPlannerError::PlanningFailed {
                reason: format!("Target schema {} not found", to_version),
            }
        })?;

        // Analyze compatibility
        let compatibility = CompatibilityChecker::check_compatibility(&from_schema, &to_schema);

        // Create schema migration plan
        let schema_plan = self
            .schema_planner
            .create_migration_plan(from_version, to_version)
            .map_err(|e| MigrationPlannerError::PlanningFailed {
                reason: e.to_string(),
            })?;

        // Convert to high-level migration plan
        let migration_plan = MigrationPlan::from_schema_plan(schema_plan, compatibility);

        // Validate the plan
        self.validate_plan(&migration_plan)?;

        Ok(migration_plan)
    }

    /// Validate a migration plan
    pub fn validate_plan(&self, plan: &MigrationPlan) -> StateResult<()> {
        // Check for empty plan
        if plan.steps.is_empty() && plan.from_version != plan.to_version {
            return Err(MigrationPlannerError::PlanningFailed {
                reason: "Migration plan has no steps but versions differ".to_string(),
            }
            .into());
        }

        // Check version sequence
        if !plan.steps.is_empty() {
            let first_step = &plan.steps[0];
            if first_step.from_version != plan.from_version {
                return Err(MigrationPlannerError::PlanningFailed {
                    reason: "First step doesn't match plan source version".to_string(),
                }
                .into());
            }

            let last_step = &plan.steps[plan.steps.len() - 1];
            if last_step.to_version != plan.to_version {
                return Err(MigrationPlannerError::PlanningFailed {
                    reason: "Last step doesn't match plan target version".to_string(),
                }
                .into());
            }

            // Check step sequence
            for i in 1..plan.steps.len() {
                if plan.steps[i - 1].to_version != plan.steps[i].from_version {
                    return Err(MigrationPlannerError::PlanningFailed {
                        reason: format!("Step sequence broken between steps {} and {}", i - 1, i),
                    }
                    .into());
                }
            }
        }

        // Check for breaking changes without backup
        if plan.has_breaking_changes() && !plan.requires_backup {
            return Err(MigrationPlannerError::IncompatibleSchema {
                details: "Breaking changes detected but backup not configured".to_string(),
            }
            .into());
        }

        Ok(())
    }

    /// Find all possible migration paths from a version
    pub fn find_migration_paths(
        &self,
        from_version: &SemanticVersion,
    ) -> StateResult<Vec<SemanticVersion>> {
        Ok(self.schema_registry.find_migration_candidates(from_version))
    }

    /// Check if migration is possible between versions
    pub fn is_migration_possible(
        &self,
        from_version: &SemanticVersion,
        to_version: &SemanticVersion,
    ) -> bool {
        if let (Some(from_schema), Some(to_schema)) = (
            self.schema_registry.get_schema(from_version),
            self.schema_registry.get_schema(to_version),
        ) {
            CompatibilityChecker::is_compatible(&from_schema, &to_schema)
                || from_version < to_version // Allow forward migration with plan
        } else {
            false
        }
    }

    /// Get migration complexity estimate
    pub fn estimate_complexity(
        &self,
        from_version: &SemanticVersion,
        to_version: &SemanticVersion,
    ) -> StateResult<MigrationComplexity> {
        if let (Some(from_schema), Some(to_schema)) = (
            self.schema_registry.get_schema(from_version),
            self.schema_registry.get_schema(to_version),
        ) {
            let compatibility = CompatibilityChecker::check_compatibility(&from_schema, &to_schema);

            #[allow(clippy::cast_possible_truncation)]
            let field_changes_count = compatibility.field_changes.len() as u64;
            let complexity = MigrationComplexity {
                risk_level: compatibility.risk_level.clone(),
                field_changes: compatibility.field_changes.len(),
                breaking_changes: compatibility.breaking_changes.len(),
                estimated_duration: Duration::from_secs(field_changes_count * 10 + 60),
                requires_backup: compatibility.risk_level
                    >= crate::state::schema::compatibility::RiskLevel::High,
                complexity_score: self.calculate_complexity_score(&compatibility),
            };

            Ok(complexity)
        } else {
            Err(MigrationPlannerError::PlanningFailed {
                reason: "One or both schemas not found".to_string(),
            }
            .into())
        }
    }

    /// Calculate a numeric complexity score
    fn calculate_complexity_score(&self, compatibility: &CompatibilityResult) -> u32 {
        let mut score = 0u32;

        // Base score for field changes
        #[allow(clippy::cast_possible_truncation)]
        let field_changes_u32 = compatibility.field_changes.len() as u32;
        score += field_changes_u32 * 10;

        // Penalty for breaking changes
        score += compatibility.breaking_changes.len() as u32 * 50;

        // Risk level multiplier
        let risk_multiplier = match compatibility.risk_level {
            crate::state::schema::compatibility::RiskLevel::Low => 1,
            crate::state::schema::compatibility::RiskLevel::Medium => 2,
            crate::state::schema::compatibility::RiskLevel::High => 4,
            crate::state::schema::compatibility::RiskLevel::Critical => 8,
        };

        score * risk_multiplier
    }

    /// Get schema registry
    pub fn schema_registry(&self) -> &SchemaRegistry {
        &self.schema_registry
    }
}

impl Default for MigrationPlanner {
    fn default() -> Self {
        Self::new()
    }
}

/// Migration complexity assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationComplexity {
    pub risk_level: crate::state::schema::compatibility::RiskLevel,
    pub field_changes: usize,
    pub breaking_changes: usize,
    pub estimated_duration: Duration,
    pub requires_backup: bool,
    pub complexity_score: u32,
}

impl MigrationComplexity {
    pub fn is_simple(&self) -> bool {
        self.complexity_score < 100
            && self.risk_level <= crate::state::schema::compatibility::RiskLevel::Low
    }

    pub fn is_complex(&self) -> bool {
        self.complexity_score > 500
            || self.risk_level >= crate::state::schema::compatibility::RiskLevel::High
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::super::config::FieldSchema;

    fn create_test_schema(version: SemanticVersion) -> EnhancedStateSchema {
        let mut schema = EnhancedStateSchema::new(version);
        schema.add_field(
            "test_field".to_string(),
            FieldSchema {
                field_type: "string".to_string(),
                required: true,
                default_value: None,
                validators: vec![],
            },
        );
        schema
    }
    #[test]
    fn test_migration_step_conversion() {
        let legacy = LegacyMigrationStep {
            from_version: 1,
            to_version: 2,
            migration_type: "breaking_migration".to_string(),
            description: "Test migration".to_string(),
        };

        let enhanced = MigrationStep::from_legacy(&legacy);
        assert_eq!(enhanced.from_version.major, 1);
        assert_eq!(enhanced.to_version.major, 2);
        assert!(enhanced.requires_backup); // Because migration_type contains "breaking"

        let converted_back = enhanced.to_legacy();
        assert_eq!(converted_back.from_version, legacy.from_version);
        assert_eq!(converted_back.to_version, legacy.to_version);
        assert_eq!(converted_back.migration_type, legacy.migration_type);
    }
    #[tokio::test]
    async fn test_migration_planner() {
        let mut planner = MigrationPlanner::new();

        let v1_0_0 = SemanticVersion::new(1, 0, 0);
        let v1_1_0 = SemanticVersion::new(1, 1, 0);

        let schema_v1 = create_test_schema(v1_0_0.clone());
        let mut schema_v1_1 = create_test_schema(v1_1_0.clone());

        // Add a field to make it a non-trivial migration
        schema_v1_1.add_field(
            "new_field".to_string(),
            FieldSchema {
                field_type: "number".to_string(),
                required: false,
                default_value: Some(serde_json::json!(0)),
                validators: vec![],
            },
        );

        planner.register_schema(schema_v1).unwrap();
        planner.register_schema(schema_v1_1).unwrap();

        // Test migration possibility
        assert!(planner.is_migration_possible(&v1_0_0, &v1_1_0));

        // Test complexity estimation
        let complexity = planner.estimate_complexity(&v1_0_0, &v1_1_0).unwrap();
        assert!(complexity.field_changes > 0); // Should detect the new field

        // Test migration paths
        let paths = planner.find_migration_paths(&v1_0_0).unwrap();
        assert!(!paths.is_empty());
    }
    #[test]
    fn test_migration_complexity() {
        let complexity = MigrationComplexity {
            risk_level: crate::state::schema::compatibility::RiskLevel::Low,
            field_changes: 2,
            breaking_changes: 0,
            estimated_duration: Duration::from_secs(80),
            requires_backup: false,
            complexity_score: 20,
        };

        assert!(complexity.is_simple());
        assert!(!complexity.is_complex());
    }
    #[test]
    fn test_plan_validation() {
        let planner = MigrationPlanner::new();

        // Test invalid plan with empty steps but different versions
        let invalid_plan = MigrationPlan {
            id: "test".to_string(),
            from_version: SemanticVersion::new(1, 0, 0),
            to_version: SemanticVersion::new(2, 0, 0),
            steps: vec![],
            estimated_duration: Duration::from_secs(60),
            total_risk_level: crate::state::schema::compatibility::RiskLevel::Low,
            requires_backup: false,
            compatibility_analysis: CompatibilityResult {
                compatible: true,
                compatibility_level: crate::state::config::CompatibilityLevel::BackwardCompatible,
                breaking_changes: vec![],
                warnings: vec![],
                field_changes: HashMap::new(),
                migration_required: false,
                risk_level: crate::state::schema::compatibility::RiskLevel::Low,
            },
            warnings: vec![],
            metadata: HashMap::new(),
            created_at: std::time::SystemTime::now(),
        };

        let result = planner.validate_plan(&invalid_plan);
        assert!(result.is_err());
    }
}
