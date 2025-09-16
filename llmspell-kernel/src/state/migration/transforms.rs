// ABOUTME: Data transformation engine for state migrations
// ABOUTME: Handles field mappings, type conversions, and data validation during migrations

use crate::state::manager::SerializableState;
use crate::state::sensitive_data::{SensitiveDataConfig, SensitiveDataProtector};
use crate::state::{StateError, StateResult};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use thiserror::Error;
use tracing::{debug, warn};

#[derive(Debug, Error)]
pub enum TransformationError {
    #[error("Field transformation failed: {field} - {reason}")]
    FieldTransformFailed { field: String, reason: String },

    #[error("Type conversion failed: {from_type} -> {to_type} for field {field}")]
    TypeConversionFailed {
        field: String,
        from_type: String,
        to_type: String,
    },

    #[error("Required field missing: {field}")]
    RequiredFieldMissing { field: String },

    #[error("Validation failed: {details}")]
    ValidationFailed { details: String },

    #[error("Sensitive data protection failed: {reason}")]
    SensitiveDataError { reason: String },
}

impl From<TransformationError> for StateError {
    fn from(err: TransformationError) -> Self {
        StateError::MigrationError(err.to_string())
    }
}

/// Field transformation specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FieldTransform {
    /// Direct field copy
    Copy {
        from_field: String,
        to_field: String,
    },
    /// Type conversion
    Convert {
        from_field: String,
        to_field: String,
        from_type: String,
        to_type: String,
        converter: String,
    },
    /// Default value assignment
    Default { field: String, value: Value },
    /// Field removal
    Remove { field: String },
    /// Field splitting (one to many)
    Split {
        from_field: String,
        to_fields: Vec<String>,
        splitter: String,
    },
    /// Field merging (many to one)
    Merge {
        from_fields: Vec<String>,
        to_field: String,
        merger: String,
    },
    /// Custom transformation
    Custom {
        from_fields: Vec<String>,
        to_fields: Vec<String>,
        transformer: String,
        config: HashMap<String, Value>,
    },
}

impl FieldTransform {
    /// Get the source fields for this transformation
    pub fn source_fields(&self) -> Vec<&str> {
        match self {
            FieldTransform::Copy { from_field, .. } => vec![from_field],
            FieldTransform::Convert { from_field, .. } => vec![from_field],
            FieldTransform::Default { .. } => vec![],
            FieldTransform::Remove { field } => vec![field],
            FieldTransform::Split { from_field, .. } => vec![from_field],
            FieldTransform::Merge { from_fields, .. } => {
                from_fields.iter().map(String::as_str).collect()
            }
            FieldTransform::Custom { from_fields, .. } => {
                from_fields.iter().map(String::as_str).collect()
            }
        }
    }

    /// Get the target fields for this transformation
    pub fn target_fields(&self) -> Vec<&str> {
        match self {
            FieldTransform::Copy { to_field, .. } => vec![to_field],
            FieldTransform::Convert { to_field, .. } => vec![to_field],
            FieldTransform::Default { field, .. } => vec![field],
            FieldTransform::Remove { .. } => vec![],
            FieldTransform::Split { to_fields, .. } => {
                to_fields.iter().map(String::as_str).collect()
            }
            FieldTransform::Merge { to_field, .. } => vec![to_field],
            FieldTransform::Custom { to_fields, .. } => {
                to_fields.iter().map(String::as_str).collect()
            }
        }
    }
}

/// State transformation specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateTransformation {
    pub id: String,
    pub description: String,
    pub from_schema_version: u32,
    pub to_schema_version: u32,
    pub field_transforms: Vec<FieldTransform>,
    pub validation_rules: Vec<ValidationRule>,
    pub preserve_unknown_fields: bool,
    pub protect_sensitive_data: bool,
}

impl StateTransformation {
    pub fn new(id: String, description: String, from_version: u32, to_version: u32) -> Self {
        Self {
            id,
            description,
            from_schema_version: from_version,
            to_schema_version: to_version,
            field_transforms: Vec::new(),
            validation_rules: Vec::new(),
            preserve_unknown_fields: true,
            protect_sensitive_data: true,
        }
    }

    pub fn add_transform(&mut self, transform: FieldTransform) {
        self.field_transforms.push(transform);
    }

    pub fn add_validation_rule(&mut self, rule: ValidationRule) {
        self.validation_rules.push(rule);
    }
}

/// Validation rule for transformed data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationRule {
    pub field: String,
    pub rule_type: ValidationType,
    pub config: HashMap<String, Value>,
    pub required: bool,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationType {
    NotNull,
    Type(String),
    Range {
        min: Option<f64>,
        max: Option<f64>,
    },
    Length {
        min: Option<usize>,
        max: Option<usize>,
    },
    Pattern(String),
    Custom(String),
}

/// Data transformer that applies transformations to state data
pub struct DataTransformer {
    #[allow(dead_code)]
    sensitive_data_protector: SensitiveDataProtector,
}

impl DataTransformer {
    pub fn new() -> Self {
        Self {
            sensitive_data_protector: SensitiveDataProtector::new(SensitiveDataConfig::default()),
        }
    }

    /// Helper function to set a nested field using dot notation (e.g., "profile.age")
    fn set_nested_field(target: &mut Value, field_path: &str, value: Value) -> bool {
        let parts: Vec<&str> = field_path.split('.').collect();
        if parts.len() == 1 {
            // Simple field, no nesting
            if let Some(obj) = target.as_object_mut() {
                obj.insert(field_path.to_string(), value);
                return true;
            }
            return false;
        }

        // Nested field path
        let mut current = target;

        // Navigate to the parent of the final field
        for part in &parts[..parts.len() - 1] {
            if let Some(obj) = current.as_object_mut() {
                // Create nested object if it doesn't exist
                if !obj.contains_key(*part) {
                    obj.insert(part.to_string(), Value::Object(serde_json::Map::new()));
                }
                // Move to the nested object
                current = obj
                    .get_mut(*part)
                    .expect("object key should exist after inserting");
            } else {
                return false;
            }
        }

        // Set the final field
        if let Some(obj) = current.as_object_mut() {
            obj.insert(parts[parts.len() - 1].to_string(), value);
            true
        } else {
            false
        }
    }

    /// Helper function to get a nested field using dot notation (e.g., "profile.age")  
    fn get_nested_field<'a>(source: &'a Value, field_path: &str) -> Option<&'a Value> {
        let parts: Vec<&str> = field_path.split('.').collect();
        if parts.len() == 1 {
            return source.get(field_path);
        }

        let mut current = source;
        for part in parts {
            current = current.get(part)?;
        }
        Some(current)
    }

    /// Helper function to remove a nested field using dot notation
    fn remove_nested_field(target: &mut Value, field_path: &str) -> bool {
        let parts: Vec<&str> = field_path.split('.').collect();
        if parts.len() == 1 {
            if let Some(obj) = target.as_object_mut() {
                return obj.remove(field_path).is_some();
            }
            return false;
        }

        // Navigate to the parent of the final field
        let mut current = target;
        for part in &parts[..parts.len() - 1] {
            current = match current.get_mut(*part) {
                Some(val) => val,
                None => return false,
            };
        }

        // Remove the final field
        if let Some(obj) = current.as_object_mut() {
            obj.remove(parts[parts.len() - 1]).is_some()
        } else {
            false
        }
    }

    pub fn with_sensitive_data_config(config: crate::state::sensitive_data::SensitiveDataConfig) -> Self {
        Self {
            sensitive_data_protector: SensitiveDataProtector::new(config),
        }
    }

    /// Transform a single state item
    pub fn transform_state(
        &self,
        state: &mut SerializableState,
        transformation: &StateTransformation,
    ) -> StateResult<TransformationResult> {
        let mut result = TransformationResult::new(transformation.id.clone());

        debug!(
            "Applying transformation '{}' to state key '{}'",
            transformation.id, state.key
        );

        // Create a new value object for the transformed data
        let mut new_value = if transformation.preserve_unknown_fields {
            state.value.clone()
        } else {
            Value::Object(serde_json::Map::new())
        };

        // Apply field transformations
        for transform in &transformation.field_transforms {
            match self.apply_field_transform(&state.value, &mut new_value, transform) {
                Ok(applied) => {
                    if applied {
                        result.fields_transformed += 1;
                        debug!("Applied transform: {:?}", transform);
                    }
                }
                Err(e) => {
                    result.add_error(format!("Transform failed: {}", e));
                    warn!("Field transformation failed: {}", e);

                    // Continue with other transforms unless it's a critical error
                    if matches!(e, TransformationError::RequiredFieldMissing { .. }) {
                        return Err(e.into());
                    }
                }
            }
        }

        // Update the state value
        state.value = new_value;
        state.schema_version = transformation.to_schema_version;
        state.timestamp = std::time::SystemTime::now();

        // Protect sensitive data if enabled
        if transformation.protect_sensitive_data {
            // Note: We would need a mutable reference to the protector for this to work
            // For now, skip sensitive data protection during transformation
            debug!("Sensitive data protection during transformation not yet implemented");
        }

        // Apply validation rules
        for rule in &transformation.validation_rules {
            if let Err(e) = self.apply_validation_rule(&state.value, rule) {
                result.add_error(format!("Validation failed: {}", e));
                if rule.required {
                    return Err(e.into());
                }
            }
        }

        result.mark_success();
        Ok(result)
    }

    /// Apply a single field transformation
    fn apply_field_transform(
        &self,
        source: &Value,
        target: &mut Value,
        transform: &FieldTransform,
    ) -> Result<bool, TransformationError> {
        match transform {
            FieldTransform::Copy {
                from_field,
                to_field,
            } => {
                if let Some(value) = Self::get_nested_field(source, from_field) {
                    if Self::set_nested_field(target, to_field, value.clone()) {
                        // Remove the source field if it's different from target
                        if from_field != to_field {
                            Self::remove_nested_field(target, from_field);
                        }
                        return Ok(true);
                    }
                }
                Ok(false)
            }

            FieldTransform::Convert {
                from_field,
                to_field,
                from_type,
                to_type,
                converter,
            } => {
                if let Some(value) = Self::get_nested_field(source, from_field) {
                    let converted = self.convert_value(value, from_type, to_type, converter)?;
                    if Self::set_nested_field(target, to_field, converted) {
                        if from_field != to_field {
                            Self::remove_nested_field(target, from_field);
                        }
                        return Ok(true);
                    }
                }
                Ok(false)
            }

            FieldTransform::Default { field, value } => {
                // Check if the field already exists (handling nested paths)
                if Self::get_nested_field(target, field).is_none()
                    && Self::set_nested_field(target, field, value.clone())
                {
                    return Ok(true);
                }
                Ok(false)
            }

            FieldTransform::Remove { field } => Ok(Self::remove_nested_field(target, field)),

            FieldTransform::Split {
                from_field,
                to_fields,
                splitter,
            } => {
                if let Some(value) = source.get(from_field) {
                    let split_values = self.split_value(value, to_fields, splitter)?;
                    if let Some(target_obj) = target.as_object_mut() {
                        for (field, split_value) in split_values {
                            target_obj.insert(field, split_value);
                        }
                        target_obj.remove(from_field);
                        return Ok(true);
                    }
                }
                Ok(false)
            }

            FieldTransform::Merge {
                from_fields,
                to_field,
                merger,
            } => {
                let source_values: Vec<&Value> = from_fields
                    .iter()
                    .filter_map(|field| source.get(field))
                    .collect();

                if !source_values.is_empty() {
                    let merged = self.merge_values(&source_values, merger)?;
                    if let Some(target_obj) = target.as_object_mut() {
                        target_obj.insert(to_field.clone(), merged);
                        // Remove source fields
                        for field in from_fields {
                            target_obj.remove(field);
                        }
                        return Ok(true);
                    }
                }
                Ok(false)
            }

            FieldTransform::Custom {
                from_fields,
                to_fields,
                transformer,
                config,
            } => {
                let source_values: HashMap<String, &Value> = from_fields
                    .iter()
                    .filter_map(|field| source.get(field).map(|v| (field.clone(), v)))
                    .collect();

                if !source_values.is_empty() {
                    let transformed =
                        self.apply_custom_transform(&source_values, transformer, config)?;
                    if let Some(target_obj) = target.as_object_mut() {
                        for (i, to_field) in to_fields.iter().enumerate() {
                            if let Some(value) = transformed.get(i) {
                                target_obj.insert(to_field.clone(), value.clone());
                            }
                        }
                        // Remove source fields
                        for field in from_fields {
                            target_obj.remove(field);
                        }
                        return Ok(true);
                    }
                }
                Ok(false)
            }
        }
    }

    /// Convert a value from one type to another
    fn convert_value(
        &self,
        value: &Value,
        from_type: &str,
        to_type: &str,
        converter: &str,
    ) -> Result<Value, TransformationError> {
        match (from_type, to_type, converter) {
            ("string", "number", "parse_int") => {
                if let Some(s) = value.as_str() {
                    s.parse::<i64>()
                        .map(|n| Value::Number(n.into()))
                        .map_err(|_| TransformationError::TypeConversionFailed {
                            field: "unknown".to_string(),
                            from_type: from_type.to_string(),
                            to_type: to_type.to_string(),
                        })
                } else {
                    Err(TransformationError::TypeConversionFailed {
                        field: "unknown".to_string(),
                        from_type: from_type.to_string(),
                        to_type: to_type.to_string(),
                    })
                }
            }
            ("string", "number", "parse_float") => {
                if let Some(s) = value.as_str() {
                    s.parse::<f64>()
                        .map_err(|_| TransformationError::TypeConversionFailed {
                            field: "unknown".to_string(),
                            from_type: from_type.to_string(),
                            to_type: to_type.to_string(),
                        })
                        .and_then(|n| {
                            serde_json::Number::from_f64(n)
                                .map(Value::Number)
                                .ok_or_else(|| TransformationError::TypeConversionFailed {
                                    field: "unknown".to_string(),
                                    from_type: from_type.to_string(),
                                    to_type: to_type.to_string(),
                                })
                        })
                } else {
                    Err(TransformationError::TypeConversionFailed {
                        field: "unknown".to_string(),
                        from_type: from_type.to_string(),
                        to_type: to_type.to_string(),
                    })
                }
            }
            ("number", "string", "to_string") => Ok(Value::String(value.to_string())),
            ("boolean", "string", "to_string") => {
                Ok(Value::String(value.as_bool().unwrap_or(false).to_string()))
            }
            _ => {
                // Default: try to convert directly
                debug!(
                    "Using default conversion for {} -> {} with {}",
                    from_type, to_type, converter
                );
                Ok(value.clone())
            }
        }
    }

    /// Split a value into multiple values
    fn split_value(
        &self,
        value: &Value,
        to_fields: &[String],
        splitter: &str,
    ) -> Result<Vec<(String, Value)>, TransformationError> {
        match splitter {
            "comma_split" => {
                if let Some(s) = value.as_str() {
                    let parts: Vec<&str> = s.split(',').map(str::trim).collect();
                    let mut result = Vec::new();
                    for (i, field) in to_fields.iter().enumerate() {
                        let value = if i < parts.len() {
                            Value::String(parts[i].to_string())
                        } else {
                            Value::Null
                        };
                        result.push((field.clone(), value));
                    }
                    Ok(result)
                } else {
                    Err(TransformationError::FieldTransformFailed {
                        field: "unknown".to_string(),
                        reason: "Value is not a string for comma_split".to_string(),
                    })
                }
            }
            _ => {
                debug!("Unknown splitter '{}', using identity split", splitter);
                Ok(to_fields
                    .iter()
                    .map(|f| (f.clone(), value.clone()))
                    .collect())
            }
        }
    }

    /// Merge multiple values into one
    fn merge_values(&self, values: &[&Value], merger: &str) -> Result<Value, TransformationError> {
        match merger {
            "concat_strings" => {
                let strings: Vec<String> = values
                    .iter()
                    .filter_map(|v| v.as_str())
                    .map(str::to_string)
                    .collect();
                Ok(Value::String(strings.join(" ")))
            }
            "sum_numbers" => {
                let sum: f64 = values.iter().filter_map(|v| v.as_f64()).sum();
                serde_json::Number::from_f64(sum)
                    .map(Value::Number)
                    .ok_or_else(|| TransformationError::TypeConversionFailed {
                        field: "sum_numbers".to_string(),
                        from_type: "array".to_string(),
                        to_type: "number".to_string(),
                    })
            }
            "first_non_null" => {
                for value in values {
                    if !value.is_null() {
                        return Ok((*value).clone());
                    }
                }
                Ok(Value::Null)
            }
            _ => {
                debug!("Unknown merger '{}', using first value", merger);
                Ok(values.first().copied().unwrap_or(&Value::Null).clone())
            }
        }
    }

    /// Apply custom transformation logic
    fn apply_custom_transform(
        &self,
        _source_values: &HashMap<String, &Value>,
        transformer: &str,
        _config: &HashMap<String, Value>,
    ) -> Result<Vec<Value>, TransformationError> {
        // Placeholder for custom transformation logic
        debug!(
            "Custom transformer '{}' not implemented, returning empty",
            transformer
        );
        Ok(vec![])
    }

    /// Apply validation rule to a value
    fn apply_validation_rule(
        &self,
        data: &Value,
        rule: &ValidationRule,
    ) -> Result<(), TransformationError> {
        let field_value = data.get(&rule.field);

        match &rule.rule_type {
            ValidationType::NotNull => {
                if field_value.is_none_or(|v| v.is_null()) {
                    return Err(TransformationError::ValidationFailed {
                        details: rule
                            .error_message
                            .clone()
                            .unwrap_or_else(|| format!("Field '{}' cannot be null", rule.field)),
                    });
                }
            }
            ValidationType::Type(expected_type) => {
                if let Some(value) = field_value {
                    let actual_type = match value {
                        Value::String(_) => "string",
                        Value::Number(_) => "number",
                        Value::Bool(_) => "boolean",
                        Value::Array(_) => "array",
                        Value::Object(_) => "object",
                        Value::Null => "null",
                    };
                    if actual_type != expected_type {
                        return Err(TransformationError::ValidationFailed {
                            details: format!(
                                "Field '{}' expected type '{}', got '{}'",
                                rule.field, expected_type, actual_type
                            ),
                        });
                    }
                }
            }
            ValidationType::Range { min, max } => {
                if let Some(value) = field_value {
                    if let Some(num) = value.as_f64() {
                        if let Some(min_val) = min {
                            if num < *min_val {
                                return Err(TransformationError::ValidationFailed {
                                    details: format!(
                                        "Field '{}' value {} below minimum {}",
                                        rule.field, num, min_val
                                    ),
                                });
                            }
                        }
                        if let Some(max_val) = max {
                            if num > *max_val {
                                return Err(TransformationError::ValidationFailed {
                                    details: format!(
                                        "Field '{}' value {} above maximum {}",
                                        rule.field, num, max_val
                                    ),
                                });
                            }
                        }
                    }
                }
            }
            ValidationType::Length { min, max } => {
                if let Some(value) = field_value {
                    let length = match value {
                        Value::String(s) => s.len(),
                        Value::Array(a) => a.len(),
                        _ => return Ok(()), // Skip validation for non-string/array types
                    };

                    if let Some(min_len) = min {
                        if length < *min_len {
                            return Err(TransformationError::ValidationFailed {
                                details: format!(
                                    "Field '{}' length {} below minimum {}",
                                    rule.field, length, min_len
                                ),
                            });
                        }
                    }
                    if let Some(max_len) = max {
                        if length > *max_len {
                            return Err(TransformationError::ValidationFailed {
                                details: format!(
                                    "Field '{}' length {} above maximum {}",
                                    rule.field, length, max_len
                                ),
                            });
                        }
                    }
                }
            }
            ValidationType::Pattern(pattern) => {
                if let Some(value) = field_value {
                    if let Some(s) = value.as_str() {
                        // Simple pattern matching - in real implementation would use regex
                        if !s.contains(pattern) {
                            return Err(TransformationError::ValidationFailed {
                                details: format!(
                                    "Field '{}' does not match pattern '{}'",
                                    rule.field, pattern
                                ),
                            });
                        }
                    }
                }
            }
            ValidationType::Custom(validator) => {
                debug!(
                    "Custom validator '{}' not implemented for field '{}'",
                    validator, rule.field
                );
            }
        }

        Ok(())
    }
}

impl Default for DataTransformer {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of a transformation operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformationResult {
    pub transformation_id: String,
    pub success: bool,
    pub fields_transformed: usize,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
    pub duration: std::time::Duration,
}

impl TransformationResult {
    pub fn new(transformation_id: String) -> Self {
        Self {
            transformation_id,
            success: false,
            fields_transformed: 0,
            warnings: Vec::new(),
            errors: Vec::new(),
            duration: std::time::Duration::from_secs(0),
        }
    }

    pub fn mark_success(&mut self) {
        self.success = true;
    }

    pub fn add_warning(&mut self, warning: String) {
        self.warnings.push(warning);
    }

    pub fn add_error(&mut self, error: String) {
        self.errors.push(error);
        self.success = false;
    }

    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    pub fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_field_transform_source_fields() {
        let transform = FieldTransform::Copy {
            from_field: "old_field".to_string(),
            to_field: "new_field".to_string(),
        };

        assert_eq!(transform.source_fields(), vec!["old_field"]);
        assert_eq!(transform.target_fields(), vec!["new_field"]);
    }
    #[test]
    fn test_state_transformation_creation() {
        let mut transformation = StateTransformation::new(
            "test_transform".to_string(),
            "Test transformation".to_string(),
            1,
            2,
        );

        assert_eq!(transformation.from_schema_version, 1);
        assert_eq!(transformation.to_schema_version, 2);
        assert!(transformation.field_transforms.is_empty());

        transformation.add_transform(FieldTransform::Copy {
            from_field: "old".to_string(),
            to_field: "new".to_string(),
        });

        assert_eq!(transformation.field_transforms.len(), 1);
    }
    #[tokio::test]
    async fn test_data_transformer() {
        let transformer = DataTransformer::new();
        let mut state = SerializableState {
            key: "test_key".to_string(),
            value: serde_json::json!({
                "old_field": "test_value",
                "number_field": "42"
            }),
            timestamp: std::time::SystemTime::now(),
            schema_version: 1,
        };

        let mut transformation =
            StateTransformation::new("test".to_string(), "Test".to_string(), 1, 2);

        transformation.add_transform(FieldTransform::Copy {
            from_field: "old_field".to_string(),
            to_field: "new_field".to_string(),
        });

        transformation.add_transform(FieldTransform::Convert {
            from_field: "number_field".to_string(),
            to_field: "parsed_number".to_string(),
            from_type: "string".to_string(),
            to_type: "number".to_string(),
            converter: "parse_int".to_string(),
        });

        let result = transformer
            .transform_state(&mut state, &transformation)
            .unwrap();

        assert!(result.success);
        assert_eq!(result.fields_transformed, 2);
        assert_eq!(state.schema_version, 2);
        assert!(state.value.get("new_field").is_some());
        assert!(state.value.get("parsed_number").is_some());
    }
    #[test]
    fn test_transformation_result() {
        let mut result = TransformationResult::new("test".to_string());

        assert!(!result.success);
        assert_eq!(result.fields_transformed, 0);

        result.add_warning("Test warning".to_string());
        result.mark_success();

        assert!(result.success);
        assert!(result.has_warnings());
        assert!(!result.has_errors());
    }
}
