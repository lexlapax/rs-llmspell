// ABOUTME: Schema compatibility checking and validation system
// ABOUTME: Provides compatibility analysis, breaking change detection, and upgrade path validation

use super::{EnhancedStateSchema, SemanticVersion};
use super::super::config::{CompatibilityLevel, FieldSchema};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CompatibilityError {
    #[error("Incompatible schema versions: {from} -> {to}")]
    IncompatibleVersions {
        from: SemanticVersion,
        to: SemanticVersion,
    },

    #[error("Breaking change detected: {change}")]
    BreakingChange { change: String },

    #[error("Missing required field: {field}")]
    MissingRequiredField { field: String },

    #[error("Field type mismatch: {field} ({expected} -> {actual})")]
    FieldTypeMismatch {
        field: String,
        expected: String,
        actual: String,
    },
}

/// Result of compatibility analysis between two schemas
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompatibilityResult {
    pub compatible: bool,
    pub compatibility_level: CompatibilityLevel,
    pub breaking_changes: Vec<String>,
    pub warnings: Vec<String>,
    pub field_changes: HashMap<String, FieldChange>,
    pub migration_required: bool,
    pub risk_level: RiskLevel,
}

/// Type of change made to a field
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FieldChange {
    Added {
        new_field: FieldSchema,
    },
    Removed {
        old_field: FieldSchema,
    },
    Modified {
        old_field: FieldSchema,
        new_field: FieldSchema,
        changes: Vec<String>,
    },
    TypeChanged {
        old_type: String,
        new_type: String,
    },
    RequiredChanged {
        was_required: bool,
        now_required: bool,
    },
}

/// Risk level of schema migration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RiskLevel {
    Low,      // Only additions, backward compatible
    Medium,   // Some modifications, migration recommended
    High,     // Breaking changes, migration required
    Critical, // Major structural changes, data loss possible
}

/// Schema compatibility checker
pub struct CompatibilityChecker;

impl CompatibilityChecker {
    /// Check compatibility between two schemas
    pub fn check_compatibility(
        from_schema: &EnhancedStateSchema,
        to_schema: &EnhancedStateSchema,
    ) -> CompatibilityResult {
        let mut result = CompatibilityResult {
            compatible: true,
            compatibility_level: CompatibilityLevel::BackwardCompatible,
            breaking_changes: Vec::new(),
            warnings: Vec::new(),
            field_changes: HashMap::new(),
            migration_required: false,
            risk_level: RiskLevel::Low,
        };

        // Check version compatibility
        Self::check_version_compatibility(&from_schema.version, &to_schema.version, &mut result);

        // Check field compatibility
        Self::check_field_compatibility(&from_schema.fields, &to_schema.fields, &mut result);

        // Check dependency compatibility
        Self::check_dependency_compatibility(
            &from_schema.dependencies,
            &to_schema.dependencies,
            &mut result,
        );

        // Determine overall compatibility and risk level
        Self::determine_overall_compatibility(&mut result);

        result
    }

    /// Quick compatibility check (boolean result only)
    pub fn is_compatible(
        from_schema: &EnhancedStateSchema,
        to_schema: &EnhancedStateSchema,
    ) -> bool {
        let result = Self::check_compatibility(from_schema, to_schema);
        result.compatible
    }

    /// Check if upgrade is safe (no data loss)
    pub fn is_safe_upgrade(
        from_schema: &EnhancedStateSchema,
        to_schema: &EnhancedStateSchema,
    ) -> bool {
        let result = Self::check_compatibility(from_schema, to_schema);
        result.risk_level == RiskLevel::Low || result.risk_level == RiskLevel::Medium
    }

    fn check_version_compatibility(
        from_version: &SemanticVersion,
        to_version: &SemanticVersion,
        result: &mut CompatibilityResult,
    ) {
        if to_version.is_breaking_change_from(from_version) {
            result.breaking_changes.push(format!(
                "Major version change: {} -> {}",
                from_version, to_version
            ));
            result.compatibility_level = CompatibilityLevel::BreakingChange;
            result.migration_required = true;
        } else if to_version < from_version {
            result.breaking_changes.push(format!(
                "Downgrade detected: {} -> {}",
                from_version, to_version
            ));
            result.compatibility_level = CompatibilityLevel::BreakingChange;
        } else if to_version.minor > from_version.minor {
            result.warnings.push(format!(
                "Minor version upgrade: {} -> {}",
                from_version, to_version
            ));
        }
    }

    fn check_field_compatibility(
        from_fields: &HashMap<String, FieldSchema>,
        to_fields: &HashMap<String, FieldSchema>,
        result: &mut CompatibilityResult,
    ) {
        let from_field_names: HashSet<String> = from_fields.keys().cloned().collect();
        let to_field_names: HashSet<String> = to_fields.keys().cloned().collect();

        // Check for removed fields
        for removed_field in from_field_names.difference(&to_field_names) {
            let old_field = from_fields.get(removed_field).unwrap();

            if old_field.required {
                result
                    .breaking_changes
                    .push(format!("Required field '{}' was removed", removed_field));
            } else {
                result
                    .warnings
                    .push(format!("Optional field '{}' was removed", removed_field));
            }

            result.field_changes.insert(
                removed_field.clone(),
                FieldChange::Removed {
                    old_field: old_field.clone(),
                },
            );
            result.migration_required = true;
        }

        // Check for added fields
        for added_field in to_field_names.difference(&from_field_names) {
            let new_field = to_fields.get(added_field).unwrap();

            if new_field.required && new_field.default_value.is_none() {
                result.breaking_changes.push(format!(
                    "Required field '{}' was added without default value",
                    added_field
                ));
            } else {
                result
                    .warnings
                    .push(format!("Field '{}' was added", added_field));
            }

            result.field_changes.insert(
                added_field.clone(),
                FieldChange::Added {
                    new_field: new_field.clone(),
                },
            );
            result.migration_required = true; // Adding fields requires migration
        }

        // Check for modified fields
        for common_field in from_field_names.intersection(&to_field_names) {
            let old_field = from_fields.get(common_field).unwrap();
            let new_field = to_fields.get(common_field).unwrap();

            Self::check_field_modification(common_field, old_field, new_field, result);
        }
    }

    fn check_field_modification(
        field_name: &str,
        old_field: &FieldSchema,
        new_field: &FieldSchema,
        result: &mut CompatibilityResult,
    ) {
        let mut changes = Vec::new();

        // Check type changes
        if old_field.field_type != new_field.field_type {
            result.breaking_changes.push(format!(
                "Field '{}' type changed: {} -> {}",
                field_name, old_field.field_type, new_field.field_type
            ));
            changes.push(format!(
                "Type: {} -> {}",
                old_field.field_type, new_field.field_type
            ));

            result.field_changes.insert(
                field_name.to_string(),
                FieldChange::TypeChanged {
                    old_type: old_field.field_type.clone(),
                    new_type: new_field.field_type.clone(),
                },
            );
        }

        // Check required changes
        if old_field.required != new_field.required {
            if new_field.required && !old_field.required {
                // Optional -> Required is breaking
                result.breaking_changes.push(format!(
                    "Field '{}' changed from optional to required",
                    field_name
                ));
            } else {
                // Required -> Optional is safe
                result.warnings.push(format!(
                    "Field '{}' changed from required to optional",
                    field_name
                ));
            }
            changes.push(format!(
                "Required: {} -> {}",
                old_field.required, new_field.required
            ));

            result.field_changes.insert(
                field_name.to_string(),
                FieldChange::RequiredChanged {
                    was_required: old_field.required,
                    now_required: new_field.required,
                },
            );
        }

        // Check default value changes
        if old_field.default_value != new_field.default_value {
            result
                .warnings
                .push(format!("Field '{}' default value changed", field_name));
            changes.push("Default value changed".to_string());
        }

        // Check validator changes
        if old_field.validators != new_field.validators {
            result
                .warnings
                .push(format!("Field '{}' validators changed", field_name));
            changes.push("Validators changed".to_string());
        }

        // If any changes were detected, record them
        if !changes.is_empty() {
            result.field_changes.insert(
                field_name.to_string(),
                FieldChange::Modified {
                    old_field: old_field.clone(),
                    new_field: new_field.clone(),
                    changes,
                },
            );
        }
    }

    fn check_dependency_compatibility(
        from_deps: &[SemanticVersion],
        to_deps: &[SemanticVersion],
        result: &mut CompatibilityResult,
    ) {
        let from_deps_set: HashSet<_> = from_deps.iter().collect();
        let to_deps_set: HashSet<_> = to_deps.iter().collect();

        // Check for removed dependencies
        for removed_dep in from_deps_set.difference(&to_deps_set) {
            result
                .warnings
                .push(format!("Dependency on schema {} was removed", removed_dep));
        }

        // Check for added dependencies
        for added_dep in to_deps_set.difference(&from_deps_set) {
            result
                .warnings
                .push(format!("Dependency on schema {} was added", added_dep));
        }
    }

    fn determine_overall_compatibility(result: &mut CompatibilityResult) {
        // Determine compatibility based on breaking changes
        if !result.breaking_changes.is_empty() {
            result.compatible = false;
            result.compatibility_level = CompatibilityLevel::BreakingChange;
        }

        // Determine risk level
        result.risk_level = if !result.breaking_changes.is_empty() {
            let critical_changes = result
                .breaking_changes
                .iter()
                .any(|change| change.contains("removed") || change.contains("type changed"));

            if critical_changes {
                RiskLevel::Critical
            } else {
                RiskLevel::High
            }
        } else if result.migration_required || !result.warnings.is_empty() {
            RiskLevel::Medium
        } else {
            RiskLevel::Low
        };
    }
}

/// Compatibility matrix for managing multiple schema versions
#[derive(Debug, Clone)]
pub struct CompatibilityMatrix {
    results: HashMap<(SemanticVersion, SemanticVersion), CompatibilityResult>,
}

impl CompatibilityMatrix {
    pub fn new() -> Self {
        Self {
            results: HashMap::new(),
        }
    }

    /// Add a compatibility result to the matrix
    pub fn add_result(
        &mut self,
        from: SemanticVersion,
        to: SemanticVersion,
        result: CompatibilityResult,
    ) {
        self.results.insert((from, to), result);
    }

    /// Get compatibility result between two versions
    pub fn get_result(
        &self,
        from: &SemanticVersion,
        to: &SemanticVersion,
    ) -> Option<&CompatibilityResult> {
        self.results.get(&(from.clone(), to.clone()))
    }

    /// Check if upgrade path exists between versions
    pub fn has_upgrade_path(&self, from: &SemanticVersion, to: &SemanticVersion) -> bool {
        if let Some(result) = self.get_result(from, to) {
            return result.compatible || result.migration_required;
        }

        // Try to find indirect path through intermediate versions
        self.find_upgrade_path(from, to).is_some()
    }

    /// Find upgrade path between versions
    pub fn find_upgrade_path(
        &self,
        from: &SemanticVersion,
        to: &SemanticVersion,
    ) -> Option<Vec<SemanticVersion>> {
        // Simple implementation - could be enhanced with graph algorithms
        let mut path = vec![from.clone()];
        let mut current = from.clone();

        while current < *to {
            let mut next_version = None;
            let mut min_risk = RiskLevel::Critical;

            // Find the best next step
            for ((from_ver, to_ver), result) in &self.results {
                if from_ver == &current
                    && to_ver <= to
                    && result.compatible
                    && result.risk_level <= min_risk
                {
                    min_risk = result.risk_level.clone();
                    next_version = Some(to_ver.clone());
                }
            }

            if let Some(next) = next_version {
                path.push(next.clone());
                current = next;

                if current == *to {
                    return Some(path);
                }
            } else {
                break;
            }
        }

        None
    }

    /// Get all compatible target versions for a source version
    pub fn get_compatible_targets(&self, from: &SemanticVersion) -> Vec<SemanticVersion> {
        self.results
            .iter()
            .filter_map(|((from_ver, to_ver), result)| {
                if from_ver == from && result.compatible {
                    Some(to_ver.clone())
                } else {
                    None
                }
            })
            .collect()
    }

    /// Get statistics about the compatibility matrix
    pub fn get_stats(&self) -> CompatibilityStats {
        let total_pairs = self.results.len();
        let compatible_pairs = self.results.values().filter(|r| r.compatible).count();
        let migration_required = self
            .results
            .values()
            .filter(|r| r.migration_required)
            .count();

        let risk_distribution = {
            let mut distribution = HashMap::new();
            for result in self.results.values() {
                *distribution.entry(result.risk_level.clone()).or_insert(0) += 1;
            }
            distribution
        };

        CompatibilityStats {
            total_pairs,
            compatible_pairs,
            migration_required,
            risk_distribution,
        }
    }
}

impl Default for CompatibilityMatrix {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics about compatibility matrix
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompatibilityStats {
    pub total_pairs: usize,
    pub compatible_pairs: usize,
    pub migration_required: usize,
    pub risk_distribution: HashMap<RiskLevel, usize>,
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_compatible_schemas() {
        let schema_v1 = create_test_schema(SemanticVersion::new(1, 0, 0));
        let mut schema_v1_1 = create_test_schema(SemanticVersion::new(1, 1, 0));

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

        let result = CompatibilityChecker::check_compatibility(&schema_v1, &schema_v1_1);

        assert!(result.compatible);
        assert_eq!(
            result.compatibility_level,
            CompatibilityLevel::BackwardCompatible
        );
        assert_eq!(result.risk_level, RiskLevel::Medium); // Adding fields requires migration
        assert!(result.breaking_changes.is_empty());
        assert_eq!(result.warnings.len(), 2); // Version upgrade + field added
    }
    #[test]
    fn test_breaking_change_detection() {
        let mut schema_v1 = create_test_schema(SemanticVersion::new(1, 0, 0));
        let schema_v2 = create_test_schema(SemanticVersion::new(2, 0, 0));

        let result = CompatibilityChecker::check_compatibility(&schema_v1, &schema_v2);

        assert!(!result.compatible);
        assert_eq!(
            result.compatibility_level,
            CompatibilityLevel::BreakingChange
        );
        assert!(!result.breaking_changes.is_empty());
        assert!(result.migration_required);

        // Test field removal breaking change
        let original_schema = create_test_schema(SemanticVersion::new(1, 0, 0));
        schema_v1.remove_field("test_field");
        let result2 = CompatibilityChecker::check_compatibility(&original_schema, &schema_v1);

        assert!(!result2.compatible);
        assert!(result2.migration_required);
    }
    #[test]
    fn test_field_type_change() {
        let schema_v1 = create_test_schema(SemanticVersion::new(1, 0, 0));
        let mut schema_v1_1 = create_test_schema(SemanticVersion::new(1, 1, 0));

        // Change field type
        schema_v1_1.fields.get_mut("test_field").unwrap().field_type = "number".to_string();

        let result = CompatibilityChecker::check_compatibility(&schema_v1, &schema_v1_1);

        assert!(!result.compatible);
        assert!(!result.breaking_changes.is_empty());
        assert!(result
            .breaking_changes
            .iter()
            .any(|c| c.contains("type changed")));
        assert_eq!(result.risk_level, RiskLevel::Critical);
    }
    #[test]
    fn test_required_field_changes() {
        let schema_v1 = create_test_schema(SemanticVersion::new(1, 0, 0));
        let mut schema_v1_1 = create_test_schema(SemanticVersion::new(1, 1, 0));

        // Change from required to optional (safe)
        schema_v1_1.fields.get_mut("test_field").unwrap().required = false;
        let result1 = CompatibilityChecker::check_compatibility(&schema_v1, &schema_v1_1);
        assert!(result1.compatible);
        assert!(result1.breaking_changes.is_empty());

        // Change from optional to required (breaking)
        let result2 = CompatibilityChecker::check_compatibility(&schema_v1_1, &schema_v1);
        assert!(!result2.compatible);
        assert!(!result2.breaking_changes.is_empty());
    }
    #[test]
    fn test_compatibility_matrix() {
        let mut matrix = CompatibilityMatrix::new();

        let v1_0_0 = SemanticVersion::new(1, 0, 0);
        let v1_1_0 = SemanticVersion::new(1, 1, 0);
        let v2_0_0 = SemanticVersion::new(2, 0, 0);

        // Add compatibility results
        let compatible_result = CompatibilityResult {
            compatible: true,
            compatibility_level: CompatibilityLevel::BackwardCompatible,
            breaking_changes: vec![],
            warnings: vec!["Minor version upgrade".to_string()],
            field_changes: HashMap::new(),
            migration_required: false,
            risk_level: RiskLevel::Low,
        };

        let breaking_result = CompatibilityResult {
            compatible: false,
            compatibility_level: CompatibilityLevel::BreakingChange,
            breaking_changes: vec!["Major version change".to_string()],
            warnings: vec![],
            field_changes: HashMap::new(),
            migration_required: true,
            risk_level: RiskLevel::High,
        };

        matrix.add_result(v1_0_0.clone(), v1_1_0.clone(), compatible_result);
        matrix.add_result(v1_0_0.clone(), v2_0_0.clone(), breaking_result);

        // Test retrieval
        let result = matrix.get_result(&v1_0_0, &v1_1_0).unwrap();
        assert!(result.compatible);

        let result2 = matrix.get_result(&v1_0_0, &v2_0_0).unwrap();
        assert!(!result2.compatible);

        // Test compatible targets
        let targets = matrix.get_compatible_targets(&v1_0_0);
        assert_eq!(targets.len(), 1);
        assert!(targets.contains(&v1_1_0));

        // Test statistics
        let stats = matrix.get_stats();
        assert_eq!(stats.total_pairs, 2);
        assert_eq!(stats.compatible_pairs, 1);
        assert_eq!(stats.migration_required, 1);
    }
    #[test]
    fn test_safe_upgrade_check() {
        let schema_v1 = create_test_schema(SemanticVersion::new(1, 0, 0));
        let mut schema_safe = create_test_schema(SemanticVersion::new(1, 1, 0));
        let schema_unsafe = create_test_schema(SemanticVersion::new(2, 0, 0));

        // Add optional field to safe upgrade
        schema_safe.add_field(
            "optional_field".to_string(),
            FieldSchema {
                field_type: "string".to_string(),
                required: false,
                default_value: Some(serde_json::json!("")),
                validators: vec![],
            },
        );

        assert!(CompatibilityChecker::is_safe_upgrade(
            &schema_v1,
            &schema_safe
        ));
        assert!(!CompatibilityChecker::is_safe_upgrade(
            &schema_v1,
            &schema_unsafe
        ));
    }
}
