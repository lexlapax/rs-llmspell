// ABOUTME: Migration validation system for ensuring data integrity
// ABOUTME: Provides pre and post-migration validation with existing StateManager integration

use crate::manager::SerializableState;
use crate::schema::{EnhancedStateSchema, SchemaRegistry};
use llmspell_state_traits::{StateError, StateResult};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use thiserror::Error;
use tracing::debug;

#[derive(Debug, Error)]
pub enum ValidationError {
    #[error("Schema validation failed: {details}")]
    SchemaValidationFailed { details: String },

    #[error("Data integrity check failed: {field} - {reason}")]
    DataIntegrityFailed { field: String, reason: String },

    #[error("Required field missing: {field}")]
    RequiredFieldMissing { field: String },

    #[error("Field type mismatch: {field} expected {expected}, got {actual}")]
    TypeMismatch {
        field: String,
        expected: String,
        actual: String,
    },

    #[error("Constraint violation: {constraint} - {details}")]
    ConstraintViolation { constraint: String, details: String },

    #[error("Custom validation failed: {validator} - {reason}")]
    CustomValidationFailed { validator: String, reason: String },
}

impl From<ValidationError> for StateError {
    fn from(err: ValidationError) -> Self {
        StateError::validation_error(err.to_string())
    }
}

/// Validation severity level
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum ValidationSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

/// Validation rule definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationRule {
    pub id: String,
    pub name: String,
    pub description: String,
    pub severity: ValidationSeverity,
    pub rule_type: ValidationRuleType,
    pub enabled: bool,
    pub applies_to_fields: Vec<String>,
    pub config: HashMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationRuleType {
    /// Schema conformance validation
    SchemaConformance,
    /// Required field presence
    RequiredFields,
    /// Data type validation
    TypeValidation,
    /// Value range validation
    RangeValidation,
    /// String length validation
    LengthValidation,
    /// Pattern matching validation
    PatternValidation,
    /// Custom validation logic
    CustomValidation(String),
    /// Foreign key integrity
    ReferentialIntegrity,
    /// Data uniqueness constraints
    UniquenessConstraint,
    /// Cross-field validation
    CrossFieldValidation,
}

/// Validation result for a single rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationIssue {
    pub rule_id: String,
    pub severity: ValidationSeverity,
    pub field: Option<String>,
    pub message: String,
    pub details: Option<String>,
    pub suggestion: Option<String>,
}

/// Complete validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub passed: bool,
    pub issues: Vec<ValidationIssue>,
    pub warnings_count: usize,
    pub errors_count: usize,
    pub critical_count: usize,
    pub validated_items: usize,
    pub duration: std::time::Duration,
}

impl ValidationResult {
    pub fn new() -> Self {
        Self {
            passed: true,
            issues: Vec::new(),
            warnings_count: 0,
            errors_count: 0,
            critical_count: 0,
            validated_items: 0,
            duration: std::time::Duration::from_secs(0),
        }
    }

    pub fn add_issue(&mut self, issue: ValidationIssue) {
        match issue.severity {
            ValidationSeverity::Info => {}
            ValidationSeverity::Warning => self.warnings_count += 1,
            ValidationSeverity::Error => {
                self.errors_count += 1;
                self.passed = false;
            }
            ValidationSeverity::Critical => {
                self.critical_count += 1;
                self.passed = false;
            }
        }
        self.issues.push(issue);
    }

    pub fn has_errors(&self) -> bool {
        self.errors_count > 0 || self.critical_count > 0
    }

    pub fn has_warnings(&self) -> bool {
        self.warnings_count > 0
    }

    pub fn is_critical(&self) -> bool {
        self.critical_count > 0
    }

    pub fn merge(&mut self, other: ValidationResult) {
        self.passed = self.passed && other.passed;
        self.warnings_count += other.warnings_count;
        self.errors_count += other.errors_count;
        self.critical_count += other.critical_count;
        self.validated_items += other.validated_items;
        self.issues.extend(other.issues);
    }
}

impl Default for ValidationResult {
    fn default() -> Self {
        Self::new()
    }
}

/// Validation rules configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationRules {
    pub rules: Vec<ValidationRule>,
    pub strict_mode: bool,
    pub fail_fast: bool,
    pub max_issues: Option<usize>,
}

impl ValidationRules {
    pub fn new() -> Self {
        Self {
            rules: Vec::new(),
            strict_mode: false,
            fail_fast: false,
            max_issues: None,
        }
    }

    pub fn strict() -> Self {
        Self {
            rules: Self::default_strict_rules(),
            strict_mode: true,
            fail_fast: true,
            max_issues: Some(100),
        }
    }

    pub fn permissive() -> Self {
        Self {
            rules: Self::default_permissive_rules(),
            strict_mode: false,
            fail_fast: false,
            max_issues: None,
        }
    }

    fn default_strict_rules() -> Vec<ValidationRule> {
        vec![
            ValidationRule {
                id: "schema_conformance".to_string(),
                name: "Schema Conformance".to_string(),
                description: "Validate data conforms to target schema".to_string(),
                severity: ValidationSeverity::Error,
                rule_type: ValidationRuleType::SchemaConformance,
                enabled: true,
                applies_to_fields: vec!["*".to_string()],
                config: HashMap::new(),
            },
            ValidationRule {
                id: "required_fields".to_string(),
                name: "Required Fields".to_string(),
                description: "Ensure all required fields are present".to_string(),
                severity: ValidationSeverity::Critical,
                rule_type: ValidationRuleType::RequiredFields,
                enabled: true,
                applies_to_fields: vec!["*".to_string()],
                config: HashMap::new(),
            },
            ValidationRule {
                id: "type_validation".to_string(),
                name: "Type Validation".to_string(),
                description: "Validate field types match schema".to_string(),
                severity: ValidationSeverity::Error,
                rule_type: ValidationRuleType::TypeValidation,
                enabled: true,
                applies_to_fields: vec!["*".to_string()],
                config: HashMap::new(),
            },
        ]
    }

    fn default_permissive_rules() -> Vec<ValidationRule> {
        vec![ValidationRule {
            id: "required_fields".to_string(),
            name: "Required Fields".to_string(),
            description: "Check for required fields".to_string(),
            severity: ValidationSeverity::Warning,
            rule_type: ValidationRuleType::RequiredFields,
            enabled: true,
            applies_to_fields: vec!["*".to_string()],
            config: HashMap::new(),
        }]
    }

    pub fn add_rule(&mut self, rule: ValidationRule) {
        self.rules.push(rule);
    }

    pub fn get_enabled_rules(&self) -> Vec<&ValidationRule> {
        self.rules.iter().filter(|r| r.enabled).collect()
    }
}

impl Default for ValidationRules {
    fn default() -> Self {
        Self::new()
    }
}

/// Migration validator that integrates with existing StateManager
pub struct MigrationValidator {
    rules: ValidationRules,
    #[allow(dead_code)]
    schema_registry: SchemaRegistry,
}

impl MigrationValidator {
    pub fn new(rules: ValidationRules) -> Self {
        Self {
            rules,
            schema_registry: SchemaRegistry::new(),
        }
    }

    pub fn with_schema_registry(rules: ValidationRules, schema_registry: SchemaRegistry) -> Self {
        Self {
            rules,
            schema_registry,
        }
    }

    /// Validate data before migration
    pub fn validate_pre_migration(
        &self,
        data: &[SerializableState],
        source_schema: &EnhancedStateSchema,
    ) -> StateResult<ValidationResult> {
        let start_time = std::time::Instant::now();
        let mut result = ValidationResult::new();

        debug!("Starting pre-migration validation for {} items", data.len());

        for state in data {
            let item_result = self.validate_state_item(state, source_schema, "pre_migration")?;
            result.merge(item_result);
            result.validated_items += 1;

            // Check if we should fail fast
            if self.rules.fail_fast && result.has_errors() {
                break;
            }

            // Check max issues limit
            if let Some(max_issues) = self.rules.max_issues {
                if result.issues.len() >= max_issues {
                    result.add_issue(ValidationIssue {
                        rule_id: "max_issues_reached".to_string(),
                        severity: ValidationSeverity::Warning,
                        field: None,
                        message: format!("Maximum issue limit reached ({})", max_issues),
                        details: Some("Validation stopped early".to_string()),
                        suggestion: Some("Review and fix existing issues".to_string()),
                    });
                    break;
                }
            }
        }

        result.duration = start_time.elapsed();
        debug!(
            "Pre-migration validation completed in {:?}",
            result.duration
        );

        Ok(result)
    }

    /// Validate data after migration
    pub fn validate_post_migration(
        &self,
        data: &[SerializableState],
        target_schema: &EnhancedStateSchema,
    ) -> StateResult<ValidationResult> {
        let start_time = std::time::Instant::now();
        let mut result = ValidationResult::new();

        debug!(
            "Starting post-migration validation for {} items",
            data.len()
        );

        for state in data {
            let item_result = self.validate_state_item(state, target_schema, "post_migration")?;
            result.merge(item_result);
            result.validated_items += 1;

            if self.rules.fail_fast && result.has_errors() {
                break;
            }

            if let Some(max_issues) = self.rules.max_issues {
                if result.issues.len() >= max_issues {
                    result.add_issue(ValidationIssue {
                        rule_id: "max_issues_reached".to_string(),
                        severity: ValidationSeverity::Warning,
                        field: None,
                        message: format!("Maximum issue limit reached ({})", max_issues),
                        details: Some("Validation stopped early".to_string()),
                        suggestion: Some("Review and fix existing issues".to_string()),
                    });
                    break;
                }
            }
        }

        result.duration = start_time.elapsed();
        debug!(
            "Post-migration validation completed in {:?}",
            result.duration
        );

        Ok(result)
    }

    /// Validate a single state item against a schema
    fn validate_state_item(
        &self,
        state: &SerializableState,
        schema: &EnhancedStateSchema,
        phase: &str,
    ) -> StateResult<ValidationResult> {
        let mut result = ValidationResult::new();

        for rule in self.rules.get_enabled_rules() {
            if self.rule_applies_to_phase(rule, phase) {
                match self.apply_validation_rule(state, schema, rule) {
                    Ok(issues) => {
                        for issue in issues {
                            result.add_issue(issue);
                        }
                    }
                    Err(e) => {
                        result.add_issue(ValidationIssue {
                            rule_id: rule.id.clone(),
                            severity: ValidationSeverity::Error,
                            field: None,
                            message: format!("Validation rule execution failed: {}", e),
                            details: Some(e.to_string()),
                            suggestion: Some("Check validation rule configuration".to_string()),
                        });
                    }
                }
            }
        }

        Ok(result)
    }

    /// Apply a single validation rule
    fn apply_validation_rule(
        &self,
        state: &SerializableState,
        schema: &EnhancedStateSchema,
        rule: &ValidationRule,
    ) -> StateResult<Vec<ValidationIssue>> {
        let mut issues = Vec::new();

        match &rule.rule_type {
            ValidationRuleType::SchemaConformance => {
                issues.extend(self.validate_schema_conformance(state, schema, rule)?);
            }
            ValidationRuleType::RequiredFields => {
                issues.extend(self.validate_required_fields(state, schema, rule)?);
            }
            ValidationRuleType::TypeValidation => {
                issues.extend(self.validate_field_types(state, schema, rule)?);
            }
            ValidationRuleType::RangeValidation => {
                issues.extend(self.validate_value_ranges(state, schema, rule)?);
            }
            ValidationRuleType::LengthValidation => {
                issues.extend(self.validate_field_lengths(state, schema, rule)?);
            }
            ValidationRuleType::PatternValidation => {
                issues.extend(self.validate_field_patterns(state, schema, rule)?);
            }
            ValidationRuleType::CustomValidation(validator_name) => {
                issues.extend(self.validate_custom(state, schema, rule, validator_name)?);
            }
            ValidationRuleType::ReferentialIntegrity => {
                issues.extend(self.validate_referential_integrity(state, schema, rule)?);
            }
            ValidationRuleType::UniquenessConstraint => {
                issues.extend(self.validate_uniqueness(state, schema, rule)?);
            }
            ValidationRuleType::CrossFieldValidation => {
                issues.extend(self.validate_cross_field(state, schema, rule)?);
            }
        }

        Ok(issues)
    }

    /// Validate schema conformance
    fn validate_schema_conformance(
        &self,
        state: &SerializableState,
        schema: &EnhancedStateSchema,
        rule: &ValidationRule,
    ) -> StateResult<Vec<ValidationIssue>> {
        let mut issues = Vec::new();

        // Check if state schema version matches
        if state.schema_version != schema.version.major {
            issues.push(ValidationIssue {
                rule_id: rule.id.clone(),
                severity: rule.severity.clone(),
                field: None,
                message: format!(
                    "Schema version mismatch: state has {}, schema expects {}",
                    state.schema_version, schema.version.major
                ),
                details: Some("State may not have been properly migrated".to_string()),
                suggestion: Some("Run migration to update schema version".to_string()),
            });
        }

        Ok(issues)
    }

    /// Validate required fields
    fn validate_required_fields(
        &self,
        state: &SerializableState,
        schema: &EnhancedStateSchema,
        rule: &ValidationRule,
    ) -> StateResult<Vec<ValidationIssue>> {
        let mut issues = Vec::new();

        if let Some(obj) = state.value.as_object() {
            for (field_name, field_schema) in &schema.fields {
                if field_schema.required && !obj.contains_key(field_name) {
                    issues.push(ValidationIssue {
                        rule_id: rule.id.clone(),
                        severity: rule.severity.clone(),
                        field: Some(field_name.clone()),
                        message: format!("Required field '{}' is missing", field_name),
                        details: Some(format!(
                            "Field is required by schema version {}",
                            schema.version
                        )),
                        suggestion: Some(
                            "Add the missing field or provide a default value".to_string(),
                        ),
                    });
                }
            }
        }

        Ok(issues)
    }

    /// Validate field types
    fn validate_field_types(
        &self,
        state: &SerializableState,
        schema: &EnhancedStateSchema,
        rule: &ValidationRule,
    ) -> StateResult<Vec<ValidationIssue>> {
        let mut issues = Vec::new();

        if let Some(obj) = state.value.as_object() {
            for (field_name, field_value) in obj {
                if let Some(field_schema) = schema.fields.get(field_name) {
                    let expected_type = &field_schema.field_type;
                    let actual_type = self.get_value_type(field_value);

                    if !self.types_compatible(expected_type, &actual_type) {
                        issues.push(ValidationIssue {
                            rule_id: rule.id.clone(),
                            severity: rule.severity.clone(),
                            field: Some(field_name.clone()),
                            message: format!(
                                "Type mismatch: expected '{}', got '{}'",
                                expected_type, actual_type
                            ),
                            details: Some(format!(
                                "Field '{}' value: {:?}",
                                field_name, field_value
                            )),
                            suggestion: Some("Convert field to the expected type".to_string()),
                        });
                    }
                }
            }
        }

        Ok(issues)
    }

    /// Validate value ranges (placeholder implementations for other validation types)
    fn validate_value_ranges(
        &self,
        _state: &SerializableState,
        _schema: &EnhancedStateSchema,
        _rule: &ValidationRule,
    ) -> StateResult<Vec<ValidationIssue>> {
        Ok(vec![])
    }

    fn validate_field_lengths(
        &self,
        _state: &SerializableState,
        _schema: &EnhancedStateSchema,
        _rule: &ValidationRule,
    ) -> StateResult<Vec<ValidationIssue>> {
        Ok(vec![])
    }

    fn validate_field_patterns(
        &self,
        _state: &SerializableState,
        _schema: &EnhancedStateSchema,
        _rule: &ValidationRule,
    ) -> StateResult<Vec<ValidationIssue>> {
        Ok(vec![])
    }

    fn validate_custom(
        &self,
        _state: &SerializableState,
        _schema: &EnhancedStateSchema,
        rule: &ValidationRule,
        validator_name: &str,
    ) -> StateResult<Vec<ValidationIssue>> {
        debug!(
            "Custom validator '{}' not implemented for rule '{}'",
            validator_name, rule.id
        );
        Ok(vec![])
    }

    fn validate_referential_integrity(
        &self,
        _state: &SerializableState,
        _schema: &EnhancedStateSchema,
        _rule: &ValidationRule,
    ) -> StateResult<Vec<ValidationIssue>> {
        Ok(vec![])
    }

    fn validate_uniqueness(
        &self,
        _state: &SerializableState,
        _schema: &EnhancedStateSchema,
        _rule: &ValidationRule,
    ) -> StateResult<Vec<ValidationIssue>> {
        Ok(vec![])
    }

    fn validate_cross_field(
        &self,
        _state: &SerializableState,
        _schema: &EnhancedStateSchema,
        _rule: &ValidationRule,
    ) -> StateResult<Vec<ValidationIssue>> {
        Ok(vec![])
    }

    /// Helper methods
    fn rule_applies_to_phase(&self, _rule: &ValidationRule, _phase: &str) -> bool {
        true // For now, all rules apply to all phases
    }

    fn get_value_type(&self, value: &Value) -> String {
        match value {
            Value::String(_) => "string".to_string(),
            Value::Number(_) => "number".to_string(),
            Value::Bool(_) => "boolean".to_string(),
            Value::Array(_) => "array".to_string(),
            Value::Object(_) => "object".to_string(),
            Value::Null => "null".to_string(),
        }
    }

    fn types_compatible(&self, expected: &str, actual: &str) -> bool {
        expected == actual || expected == "any" || actual == "null"
    }
}

#[cfg(test)]
#[cfg_attr(test_category = "state")]
mod tests {
    use super::*;
    use crate::schema::EnhancedStateSchema;

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_validation_result() {
        let mut result = ValidationResult::new();
        assert!(result.passed);
        assert_eq!(result.errors_count, 0);

        result.add_issue(ValidationIssue {
            rule_id: "test".to_string(),
            severity: ValidationSeverity::Warning,
            field: None,
            message: "Test warning".to_string(),
            details: None,
            suggestion: None,
        });

        assert!(result.passed); // Still passes with warnings
        assert_eq!(result.warnings_count, 1);

        result.add_issue(ValidationIssue {
            rule_id: "test2".to_string(),
            severity: ValidationSeverity::Error,
            field: None,
            message: "Test error".to_string(),
            details: None,
            suggestion: None,
        });

        assert!(!result.passed); // Now fails due to error
        assert_eq!(result.errors_count, 1);
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_validation_rules() {
        let rules = ValidationRules::strict();
        assert!(rules.strict_mode);
        assert!(rules.fail_fast);

        let enabled_rules = rules.get_enabled_rules();
        assert!(!enabled_rules.is_empty());
    }

    #[cfg_attr(test_category = "unit")]
    #[tokio::test]
    async fn test_migration_validator() {
        use crate::schema::SemanticVersion;

        let rules = ValidationRules::permissive();
        let validator = MigrationValidator::new(rules);

        let schema = EnhancedStateSchema::new(SemanticVersion::new(1, 0, 0));
        let state = SerializableState {
            key: "test".to_string(),
            value: serde_json::json!({"test_field": "test_value"}),
            timestamp: std::time::SystemTime::now(),
            schema_version: 1,
        };

        let result = validator.validate_pre_migration(&[state], &schema).unwrap();
        assert!(!result.has_errors());
    }
}
