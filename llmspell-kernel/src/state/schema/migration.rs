// ABOUTME: Migration planning and execution system for schema version transitions
// ABOUTME: Provides automated migration path discovery and execution planning

use super::compatibility::RiskLevel;
use super::{CompatibilityChecker, CompatibilityResult, EnhancedStateSchema, SemanticVersion};
use crate::config::MigrationStep;
use llmspell_state_traits::StateError;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum MigrationPlannerError {
    #[error("No migration path found from {from} to {to}")]
    NoMigrationPath {
        from: Box<SemanticVersion>,
        to: Box<SemanticVersion>,
    },

    #[error("Migration step validation failed: {reason}")]
    ValidationFailed { reason: String },

    #[error("Circular dependency detected in migration path")]
    CircularDependency,

    #[error("Invalid migration step: {step_id}")]
    InvalidMigrationStep { step_id: String },

    #[error("Migration planning failed: {details}")]
    PlanningFailed { details: String },
}

impl From<MigrationPlannerError> for StateError {
    fn from(err: MigrationPlannerError) -> Self {
        StateError::MigrationError(err.to_string())
    }
}

/// A complete migration plan from one schema version to another
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationPlan {
    pub from_version: SemanticVersion,
    pub to_version: SemanticVersion,
    pub steps: Vec<MigrationStep>,
    pub estimated_duration: std::time::Duration,
    pub risk_level: RiskLevel,
    pub requires_backup: bool,
    pub data_transformations: Vec<DataTransformation>,
    pub rollback_plan: Option<Box<MigrationPlan>>,
    pub warnings: Vec<String>,
}

/// Data transformation step within a migration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataTransformation {
    pub transformation_id: String,
    pub description: String,
    pub field_mappings: HashMap<String, FieldMapping>,
    pub custom_logic: Option<String>,
    pub validation_rules: Vec<String>,
}

/// Field mapping for data transformation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FieldMapping {
    Direct {
        from_field: String,
        to_field: String,
    },
    Transform {
        from_field: String,
        to_field: String,
        transformation: String,
    },
    Split {
        from_field: String,
        to_fields: Vec<String>,
        logic: String,
    },
    Merge {
        from_fields: Vec<String>,
        to_field: String,
        logic: String,
    },
    Default {
        to_field: String,
        default_value: serde_json::Value,
    },
    Remove {
        field: String,
    },
}

/// Migration planner for creating migration plans between schema versions
pub struct MigrationPlanner {
    schema_registry: HashMap<SemanticVersion, EnhancedStateSchema>,
    compatibility_cache: HashMap<(SemanticVersion, SemanticVersion), CompatibilityResult>,
}

impl MigrationPlanner {
    pub fn new() -> Self {
        Self {
            schema_registry: HashMap::new(),
            compatibility_cache: HashMap::new(),
        }
    }

    /// Register a schema for migration planning
    pub fn register_schema(&mut self, schema: EnhancedStateSchema) {
        self.schema_registry.insert(schema.version.clone(), schema);
    }

    /// Create a migration plan between two schema versions
    pub fn create_migration_plan(
        &mut self,
        from_version: &SemanticVersion,
        to_version: &SemanticVersion,
    ) -> Result<MigrationPlan, MigrationPlannerError> {
        if from_version == to_version {
            return Ok(Self::create_no_op_plan(from_version.clone()));
        }

        let _from_schema = self.schema_registry.get(from_version).ok_or_else(|| {
            MigrationPlannerError::PlanningFailed {
                details: format!("Source schema {} not found", from_version),
            }
        })?;

        let _to_schema = self.schema_registry.get(to_version).ok_or_else(|| {
            MigrationPlannerError::PlanningFailed {
                details: format!("Target schema {} not found", to_version),
            }
        })?;

        // Find the optimal migration path
        let path = self.find_migration_path(from_version, to_version)?;

        // Create migration steps for the path
        let mut steps = Vec::new();
        let mut data_transformations = Vec::new();
        let mut warnings = Vec::new();
        let mut overall_risk = RiskLevel::Low;
        let mut requires_backup = false;

        for i in 0..path.len() - 1 {
            let step_from = &path[i];
            let step_to = &path[i + 1];

            let compatibility = self.get_or_compute_compatibility(step_from, step_to)?;

            // Update overall risk level
            if compatibility.risk_level > overall_risk {
                overall_risk = compatibility.risk_level.clone();
            }

            if compatibility.risk_level >= RiskLevel::High {
                requires_backup = true;
            }

            // Create migration step
            let migration_step = MigrationStep {
                from_version: step_from.major, // Convert to legacy format for compatibility
                to_version: step_to.major,
                migration_type: Self::determine_migration_type(&compatibility),
                description: format!("Migrate from {} to {}", step_from, step_to),
            };
            steps.push(migration_step);

            // Create data transformations
            let transformation =
                self.create_data_transformation(step_from, step_to, &compatibility)?;
            data_transformations.push(transformation);

            // Collect warnings
            warnings.extend(compatibility.warnings.clone());
        }

        // Estimate duration based on complexity
        let estimated_duration = Self::estimate_migration_duration(&steps, &data_transformations);

        // Create rollback plan if needed
        let rollback_plan = if requires_backup {
            Some(Box::new(
                self.create_rollback_plan(to_version, from_version)?,
            ))
        } else {
            None
        };

        Ok(MigrationPlan {
            from_version: from_version.clone(),
            to_version: to_version.clone(),
            steps,
            estimated_duration,
            risk_level: overall_risk,
            requires_backup,
            data_transformations,
            rollback_plan,
            warnings,
        })
    }

    /// Find the optimal migration path between two versions
    fn find_migration_path(
        &self,
        from_version: &SemanticVersion,
        to_version: &SemanticVersion,
    ) -> Result<Vec<SemanticVersion>, MigrationPlannerError> {
        // Use breadth-first search to find the shortest path
        let mut queue = VecDeque::new();
        let mut visited = std::collections::HashSet::new();
        let mut parent: HashMap<SemanticVersion, SemanticVersion> = HashMap::new();

        queue.push_back(from_version.clone());
        visited.insert(from_version.clone());

        while let Some(current) = queue.pop_front() {
            if current == *to_version {
                // Reconstruct path
                let mut path = Vec::new();
                let mut node = to_version.clone();

                while let Some(prev) = parent.get(&node) {
                    path.push(node.clone());
                    node = prev.clone();
                }
                path.push(from_version.clone());
                path.reverse();

                return Ok(path);
            }

            // Find all reachable versions from current
            for next_version in self.get_reachable_versions(&current) {
                if !visited.contains(&next_version) {
                    visited.insert(next_version.clone());
                    parent.insert(next_version.clone(), current.clone());
                    queue.push_back(next_version);
                }
            }
        }

        Err(MigrationPlannerError::NoMigrationPath {
            from: Box::new(from_version.clone()),
            to: Box::new(to_version.clone()),
        })
    }

    /// Get all versions reachable from a given version
    fn get_reachable_versions(&self, from_version: &SemanticVersion) -> Vec<SemanticVersion> {
        self.schema_registry
            .keys()
            .filter(|version| {
                *version > from_version && self.is_migration_possible(from_version, version)
            })
            .cloned()
            .collect()
    }

    /// Check if migration is possible between two versions
    fn is_migration_possible(&self, from: &SemanticVersion, to: &SemanticVersion) -> bool {
        // Allow migration within same major version or to next major version
        to.major <= from.major + 1
    }

    /// Get or compute compatibility between two versions
    fn get_or_compute_compatibility(
        &mut self,
        from: &SemanticVersion,
        to: &SemanticVersion,
    ) -> Result<CompatibilityResult, MigrationPlannerError> {
        let key = (from.clone(), to.clone());

        if let Some(cached) = self.compatibility_cache.get(&key) {
            return Ok(cached.clone());
        }

        let from_schema = self.schema_registry.get(from).ok_or_else(|| {
            MigrationPlannerError::PlanningFailed {
                details: format!("Schema {} not found", from),
            }
        })?;

        let to_schema =
            self.schema_registry
                .get(to)
                .ok_or_else(|| MigrationPlannerError::PlanningFailed {
                    details: format!("Schema {} not found", to),
                })?;

        let compatibility = CompatibilityChecker::check_compatibility(from_schema, to_schema);
        self.compatibility_cache.insert(key, compatibility.clone());

        Ok(compatibility)
    }

    /// Create data transformation for a migration step
    fn create_data_transformation(
        &self,
        from_version: &SemanticVersion,
        to_version: &SemanticVersion,
        compatibility: &CompatibilityResult,
    ) -> Result<DataTransformation, MigrationPlannerError> {
        let transformation_id = format!("transform_{}_{}", from_version, to_version);
        let description = format!("Transform data from {} to {}", from_version, to_version);

        let mut field_mappings = HashMap::new();
        let mut validation_rules = Vec::new();

        // Create field mappings based on compatibility analysis
        for (field_name, field_change) in &compatibility.field_changes {
            match field_change {
                super::compatibility::FieldChange::Added { new_field } => {
                    if let Some(default_value) = &new_field.default_value {
                        field_mappings.insert(
                            field_name.clone(),
                            FieldMapping::Default {
                                to_field: field_name.clone(),
                                default_value: default_value.clone(),
                            },
                        );
                    }
                }
                super::compatibility::FieldChange::Removed { .. } => {
                    field_mappings.insert(
                        field_name.clone(),
                        FieldMapping::Remove {
                            field: field_name.clone(),
                        },
                    );
                }
                super::compatibility::FieldChange::Modified {
                    old_field,
                    new_field,
                    ..
                } => {
                    if old_field.field_type != new_field.field_type {
                        field_mappings.insert(
                            field_name.clone(),
                            FieldMapping::Transform {
                                from_field: field_name.clone(),
                                to_field: field_name.clone(),
                                transformation: format!(
                                    "convert_{}_to_{}",
                                    old_field.field_type, new_field.field_type
                                ),
                            },
                        );
                    } else {
                        field_mappings.insert(
                            field_name.clone(),
                            FieldMapping::Direct {
                                from_field: field_name.clone(),
                                to_field: field_name.clone(),
                            },
                        );
                    }
                }
                super::compatibility::FieldChange::TypeChanged { old_type, new_type } => {
                    field_mappings.insert(
                        field_name.clone(),
                        FieldMapping::Transform {
                            from_field: field_name.clone(),
                            to_field: field_name.clone(),
                            transformation: format!("convert_{}_to_{}", old_type, new_type),
                        },
                    );
                }
                super::compatibility::FieldChange::RequiredChanged { .. } => {
                    field_mappings.insert(
                        field_name.clone(),
                        FieldMapping::Direct {
                            from_field: field_name.clone(),
                            to_field: field_name.clone(),
                        },
                    );
                }
            }
        }

        // Add validation rules based on risk level
        match compatibility.risk_level {
            RiskLevel::High | RiskLevel::Critical => {
                validation_rules.push("validate_all_required_fields".to_string());
                validation_rules.push("validate_data_integrity".to_string());
            }
            RiskLevel::Medium => {
                validation_rules.push("validate_required_fields".to_string());
            }
            _ => {}
        }

        Ok(DataTransformation {
            transformation_id,
            description,
            field_mappings,
            custom_logic: None,
            validation_rules,
        })
    }

    /// Create a rollback plan
    fn create_rollback_plan(
        &mut self,
        from_version: &SemanticVersion,
        to_version: &SemanticVersion,
    ) -> Result<MigrationPlan, MigrationPlannerError> {
        // Create a simple rollback plan (reverse migration)
        Ok(MigrationPlan {
            from_version: from_version.clone(),
            to_version: to_version.clone(),
            steps: vec![MigrationStep {
                from_version: from_version.major,
                to_version: to_version.major,
                migration_type: "rollback".to_string(),
                description: format!("Rollback from {} to {}", from_version, to_version),
            }],
            estimated_duration: std::time::Duration::from_secs(300), // 5 minutes default
            risk_level: RiskLevel::High,                             // Rollbacks are always risky
            requires_backup: true,
            data_transformations: vec![],
            rollback_plan: None, // No nested rollback plans
            warnings: vec!["Rollback may cause data loss".to_string()],
        })
    }

    /// Create a no-op migration plan for same version
    fn create_no_op_plan(version: SemanticVersion) -> MigrationPlan {
        MigrationPlan {
            from_version: version.clone(),
            to_version: version,
            steps: vec![],
            estimated_duration: std::time::Duration::from_secs(0),
            risk_level: RiskLevel::Low,
            requires_backup: false,
            data_transformations: vec![],
            rollback_plan: None,
            warnings: vec![],
        }
    }

    /// Determine migration type based on compatibility
    fn determine_migration_type(compatibility: &CompatibilityResult) -> String {
        match compatibility.risk_level {
            RiskLevel::Low => "minor_upgrade".to_string(),
            RiskLevel::Medium => "standard_migration".to_string(),
            RiskLevel::High => "breaking_migration".to_string(),
            RiskLevel::Critical => "major_migration".to_string(),
        }
    }

    /// Estimate migration duration
    fn estimate_migration_duration(
        steps: &[MigrationStep],
        transformations: &[DataTransformation],
    ) -> std::time::Duration {
        let base_duration = std::time::Duration::from_secs(60); // 1 minute base
        #[allow(clippy::cast_possible_truncation)]
        let steps_len_u64 = steps.len() as u64;
        let step_duration = std::time::Duration::from_secs(30 * steps_len_u64);
        let transform_duration = std::time::Duration::from_secs(
            transformations
                .iter()
                .map(|t| {
                    #[allow(clippy::cast_possible_truncation)]
                    let field_mappings_len_u64 = t.field_mappings.len() as u64;
                    field_mappings_len_u64 * 10
                })
                .sum(),
        );

        base_duration + step_duration + transform_duration
    }

    /// Validate a migration plan
    pub fn validate_plan(&self, plan: &MigrationPlan) -> Result<(), MigrationPlannerError> {
        // Check that all schemas in the plan exist
        if !self.schema_registry.contains_key(&plan.from_version) {
            return Err(MigrationPlannerError::ValidationFailed {
                reason: format!("Source schema {} not found", plan.from_version),
            });
        }

        if !self.schema_registry.contains_key(&plan.to_version) {
            return Err(MigrationPlannerError::ValidationFailed {
                reason: format!("Target schema {} not found", plan.to_version),
            });
        }

        // Validate step sequence
        if !plan.steps.is_empty() {
            let first_step = &plan.steps[0];
            if first_step.from_version != plan.from_version.major {
                return Err(MigrationPlannerError::ValidationFailed {
                    reason: "First step doesn't match plan source version".to_string(),
                });
            }

            let last_step = &plan.steps[plan.steps.len() - 1];
            if last_step.to_version != plan.to_version.major {
                return Err(MigrationPlannerError::ValidationFailed {
                    reason: "Last step doesn't match plan target version".to_string(),
                });
            }
        }

        // Validate data transformations
        for (i, transformation) in plan.data_transformations.iter().enumerate() {
            if transformation.field_mappings.is_empty() && plan.steps.len() > i {
                return Err(MigrationPlannerError::ValidationFailed {
                    reason: format!("Transformation {} has no field mappings", i),
                });
            }
        }

        Ok(())
    }

    /// Get migration statistics
    pub fn get_migration_stats(&self) -> MigrationStats {
        let total_schemas = self.schema_registry.len();
        let total_compatibility_checks = self.compatibility_cache.len();

        let risk_distribution = {
            let mut distribution = HashMap::new();
            for result in self.compatibility_cache.values() {
                *distribution.entry(result.risk_level.clone()).or_insert(0) += 1;
            }
            distribution
        };

        let average_migration_complexity = if !self.compatibility_cache.is_empty() {
            #[allow(clippy::cast_precision_loss)]
            let sum_f64 = self
                .compatibility_cache
                .values()
                .map(|r| r.field_changes.len())
                .sum::<usize>() as f64;
            #[allow(clippy::cast_precision_loss)]
            let len_f64 = self.compatibility_cache.len() as f64;
            sum_f64 / len_f64
        } else {
            0.0
        };

        MigrationStats {
            total_schemas,
            total_compatibility_checks,
            risk_distribution,
            average_migration_complexity,
        }
    }
}

impl Default for MigrationPlanner {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics about migration planning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationStats {
    pub total_schemas: usize,
    pub total_compatibility_checks: usize,
    pub risk_distribution: HashMap<RiskLevel, usize>,
    pub average_migration_complexity: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::FieldSchema;

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
    fn test_migration_planner_creation() {
        let planner = MigrationPlanner::new();
        let stats = planner.get_migration_stats();

        assert_eq!(stats.total_schemas, 0);
        assert_eq!(stats.total_compatibility_checks, 0);
    }
    #[test]
    fn test_schema_registration() {
        let mut planner = MigrationPlanner::new();
        let schema = create_test_schema(SemanticVersion::new(1, 0, 0));
        let version = schema.version.clone();

        planner.register_schema(schema);

        assert!(planner.schema_registry.contains_key(&version));
        assert_eq!(planner.get_migration_stats().total_schemas, 1);
    }
    #[test]
    fn test_no_op_migration_plan() {
        let mut planner = MigrationPlanner::new();
        let version = SemanticVersion::new(1, 0, 0);
        let schema = create_test_schema(version.clone());

        planner.register_schema(schema);

        let plan = planner.create_migration_plan(&version, &version).unwrap();

        assert_eq!(plan.from_version, version);
        assert_eq!(plan.to_version, version);
        assert!(plan.steps.is_empty());
        assert_eq!(plan.risk_level, RiskLevel::Low);
        assert!(!plan.requires_backup);
    }
    #[test]
    fn test_simple_migration_plan() {
        let mut planner = MigrationPlanner::new();

        let v1_0_0 = SemanticVersion::new(1, 0, 0);
        let v1_1_0 = SemanticVersion::new(1, 1, 0);

        let schema_v1 = create_test_schema(v1_0_0.clone());
        let mut schema_v1_1 = create_test_schema(v1_1_0.clone());

        // Add a new optional field
        schema_v1_1.add_field(
            "new_field".to_string(),
            FieldSchema {
                field_type: "number".to_string(),
                required: false,
                default_value: Some(serde_json::json!(0)),
                validators: vec![],
            },
        );

        planner.register_schema(schema_v1);
        planner.register_schema(schema_v1_1);

        let plan = planner.create_migration_plan(&v1_0_0, &v1_1_0).unwrap();

        assert_eq!(plan.from_version, v1_0_0);
        assert_eq!(plan.to_version, v1_1_0);
        assert_eq!(plan.steps.len(), 1);
        assert_eq!(plan.data_transformations.len(), 1);
        assert!(!plan.requires_backup);

        // Validate the plan
        planner.validate_plan(&plan).unwrap();
    }
    #[test]
    fn test_breaking_migration_plan() {
        let mut planner = MigrationPlanner::new();

        let v1_0_0 = SemanticVersion::new(1, 0, 0);
        let v2_0_0 = SemanticVersion::new(2, 0, 0);

        // Create schemas with breaking changes between versions
        let schema_v1 = create_test_schema(v1_0_0.clone());
        let mut schema_v2 = create_test_schema(v2_0_0.clone());

        // Make a breaking change - change field requirement
        schema_v2.add_field(
            "breaking_field".to_string(),
            FieldSchema {
                field_type: "string".to_string(),
                required: true, // This creates a breaking change
                default_value: None,
                validators: vec![],
            },
        );

        planner.register_schema(schema_v1);
        planner.register_schema(schema_v2);

        let plan = planner.create_migration_plan(&v1_0_0, &v2_0_0).unwrap();

        assert_eq!(plan.risk_level, RiskLevel::High);
        assert!(plan.requires_backup);
        assert!(plan.rollback_plan.is_some());
        // Note: warnings may be empty if compatibility checker doesn't generate them
        // for major version changes, which is expected behavior
    }
    #[test]
    fn test_field_mapping_creation() {
        let mut planner = MigrationPlanner::new();

        let v1_0_0 = SemanticVersion::new(1, 0, 0);
        let v1_1_0 = SemanticVersion::new(1, 1, 0);

        let schema_v1 = create_test_schema(v1_0_0.clone());
        let mut schema_v1_1 = create_test_schema(v1_1_0.clone());

        // Modify field type
        schema_v1_1.fields.get_mut("test_field").unwrap().field_type = "number".to_string();

        planner.register_schema(schema_v1);
        planner.register_schema(schema_v1_1);

        let plan = planner.create_migration_plan(&v1_0_0, &v1_1_0).unwrap();

        assert_eq!(plan.data_transformations.len(), 1);
        let transformation = &plan.data_transformations[0];

        assert!(transformation.field_mappings.contains_key("test_field"));
        match transformation.field_mappings.get("test_field").unwrap() {
            FieldMapping::Transform { transformation, .. } => {
                assert!(transformation.contains("string_to_number"));
            }
            _ => panic!("Expected Transform mapping"),
        }
    }
    #[test]
    fn test_migration_path_finding() {
        let mut planner = MigrationPlanner::new();

        let v1_0_0 = SemanticVersion::new(1, 0, 0);
        let v1_1_0 = SemanticVersion::new(1, 1, 0);
        let v1_2_0 = SemanticVersion::new(1, 2, 0);

        planner.register_schema(create_test_schema(v1_0_0.clone()));
        planner.register_schema(create_test_schema(v1_1_0.clone()));
        planner.register_schema(create_test_schema(v1_2_0.clone()));

        let path = planner.find_migration_path(&v1_0_0, &v1_2_0).unwrap();

        // BFS finds shortest path: v1.0.0 -> v1.2.0 (direct jump is allowed within major version)
        assert_eq!(path.len(), 2);
        assert_eq!(path[0], v1_0_0);
        assert_eq!(path[1], v1_2_0);
    }
    #[test]
    fn test_plan_validation() {
        let planner = MigrationPlanner::new();

        let v1_0_0 = SemanticVersion::new(1, 0, 0);
        let v1_1_0 = SemanticVersion::new(1, 1, 0);

        // Test invalid plan (missing schemas)
        let invalid_plan = MigrationPlan {
            from_version: v1_0_0.clone(),
            to_version: v1_1_0.clone(),
            steps: vec![],
            estimated_duration: std::time::Duration::from_secs(60),
            risk_level: RiskLevel::Low,
            requires_backup: false,
            data_transformations: vec![],
            rollback_plan: None,
            warnings: vec![],
        };

        let result = planner.validate_plan(&invalid_plan);
        assert!(result.is_err());
        assert!(matches!(
            result,
            Err(MigrationPlannerError::ValidationFailed { .. })
        ));
    }
    #[test]
    fn test_migration_stats() {
        let mut planner = MigrationPlanner::new();

        planner.register_schema(create_test_schema(SemanticVersion::new(1, 0, 0)));
        planner.register_schema(create_test_schema(SemanticVersion::new(1, 1, 0)));

        // Create a migration plan to populate compatibility cache
        let _ = planner.create_migration_plan(
            &SemanticVersion::new(1, 0, 0),
            &SemanticVersion::new(1, 1, 0),
        );

        let stats = planner.get_migration_stats();

        assert_eq!(stats.total_schemas, 2);
        assert!(stats.total_compatibility_checks > 0);
        assert!(!stats.risk_distribution.is_empty());
    }
}
