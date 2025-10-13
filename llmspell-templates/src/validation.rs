//! Parameter validation with JSON Schema-inspired configuration

use crate::error::{Result, ValidationError};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// Configuration schema for template parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigSchema {
    /// Schema parameters
    pub parameters: Vec<ParameterSchema>,

    /// Schema version
    #[serde(default = "default_version")]
    pub version: String,
}

fn default_version() -> String {
    "1.0".to_string()
}

impl ConfigSchema {
    /// Create a new config schema
    pub fn new(parameters: Vec<ParameterSchema>) -> Self {
        Self {
            parameters,
            version: default_version(),
        }
    }

    /// Validate parameters against schema
    pub fn validate(&self, params: &HashMap<String, Value>) -> Result<()> {
        let mut errors = Vec::new();

        // Check required parameters
        for param in &self.parameters {
            if param.required && !params.contains_key(&param.name) {
                errors.push(format!("Required parameter missing: {}", param.name));
                continue;
            }

            if let Some(value) = params.get(&param.name) {
                // Validate type
                if let Err(e) = param.validate_type(value) {
                    errors.push(e.to_string());
                }

                // Validate value constraints
                if let Err(e) = param.validate_value(value) {
                    errors.push(e.to_string());
                }
            }
        }

        // Check for unsupported parameters (optional warning, not error)
        let schema_param_names: std::collections::HashSet<_> =
            self.parameters.iter().map(|p| p.name.as_str()).collect();
        for param_name in params.keys() {
            if !schema_param_names.contains(param_name.as_str()) {
                tracing::warn!("Unknown parameter provided: {}", param_name);
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(ValidationError::multiple(errors).into())
        }
    }

    /// Get parameter by name
    pub fn get_parameter(&self, name: &str) -> Option<&ParameterSchema> {
        self.parameters.iter().find(|p| p.name == name)
    }
}

/// Parameter schema definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterSchema {
    /// Parameter name
    pub name: String,

    /// Parameter description
    pub description: String,

    /// Parameter type
    #[serde(rename = "type")]
    pub param_type: ParameterType,

    /// Whether parameter is required
    #[serde(default)]
    pub required: bool,

    /// Default value (if not required)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<Value>,

    /// Validation constraints
    #[serde(skip_serializing_if = "Option::is_none")]
    pub constraints: Option<ParameterConstraints>,
}

impl ParameterSchema {
    /// Create a required parameter
    pub fn required(
        name: impl Into<String>,
        description: impl Into<String>,
        param_type: ParameterType,
    ) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            param_type,
            required: true,
            default: None,
            constraints: None,
        }
    }

    /// Create an optional parameter with default
    pub fn optional(
        name: impl Into<String>,
        description: impl Into<String>,
        param_type: ParameterType,
        default: Value,
    ) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            param_type,
            required: false,
            default: Some(default),
            constraints: None,
        }
    }

    /// Add constraints to parameter
    pub fn with_constraints(mut self, constraints: ParameterConstraints) -> Self {
        self.constraints = Some(constraints);
        self
    }

    /// Validate parameter type
    fn validate_type(&self, value: &Value) -> Result<()> {
        let matches = match (&self.param_type, value) {
            (ParameterType::String, Value::String(_)) => true,
            (ParameterType::Number, Value::Number(_)) => true,
            (ParameterType::Integer, Value::Number(n)) => n.is_i64(),
            (ParameterType::Boolean, Value::Bool(_)) => true,
            (ParameterType::Array, Value::Array(_)) => true,
            (ParameterType::Object, Value::Object(_)) => true,
            _ => false,
        };

        if matches {
            Ok(())
        } else {
            Err(ValidationError::type_mismatch(
                &self.name,
                format!("{:?}", self.param_type),
                format!("{:?}", value),
            )
            .into())
        }
    }

    /// Validate parameter value against constraints
    fn validate_value(&self, value: &Value) -> Result<()> {
        if let Some(constraints) = &self.constraints {
            constraints.validate(&self.name, value)?;
        }
        Ok(())
    }
}

/// Parameter type
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ParameterType {
    String,
    Number,
    Integer,
    Boolean,
    Array,
    Object,
}

/// Parameter validation constraints
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ParameterConstraints {
    /// Minimum value (for numbers)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min: Option<f64>,

    /// Maximum value (for numbers)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max: Option<f64>,

    /// Minimum length (for strings/arrays)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_length: Option<usize>,

    /// Maximum length (for strings/arrays)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_length: Option<usize>,

    /// Pattern (regex for strings)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pattern: Option<String>,

    /// Allowed values (enum)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_values: Option<Vec<Value>>,
}

impl ParameterConstraints {
    /// Validate value against constraints
    fn validate(&self, param_name: &str, value: &Value) -> Result<()> {
        // Validate numeric constraints
        if let Some(num) = value.as_f64() {
            if let Some(min) = self.min {
                if num < min {
                    return Err(ValidationError::out_of_range(
                        param_name,
                        format!("must be >= {}", min),
                    )
                    .into());
                }
            }
            if let Some(max) = self.max {
                if num > max {
                    return Err(ValidationError::out_of_range(
                        param_name,
                        format!("must be <= {}", max),
                    )
                    .into());
                }
            }
        }

        // Validate string constraints
        if let Some(s) = value.as_str() {
            if let Some(min_length) = self.min_length {
                if s.len() < min_length {
                    return Err(ValidationError::out_of_range(
                        param_name,
                        format!("length must be >= {}", min_length),
                    )
                    .into());
                }
            }
            if let Some(max_length) = self.max_length {
                if s.len() > max_length {
                    return Err(ValidationError::out_of_range(
                        param_name,
                        format!("length must be <= {}", max_length),
                    )
                    .into());
                }
            }
            if let Some(pattern) = &self.pattern {
                // Simple pattern matching (not full regex for now)
                if !s.contains(pattern) {
                    return Err(ValidationError::invalid_value(
                        param_name,
                        format!("must match pattern: {}", pattern),
                    )
                    .into());
                }
            }
        }

        // Validate array constraints
        if let Some(arr) = value.as_array() {
            if let Some(min_length) = self.min_length {
                if arr.len() < min_length {
                    return Err(ValidationError::out_of_range(
                        param_name,
                        format!("array length must be >= {}", min_length),
                    )
                    .into());
                }
            }
            if let Some(max_length) = self.max_length {
                if arr.len() > max_length {
                    return Err(ValidationError::out_of_range(
                        param_name,
                        format!("array length must be <= {}", max_length),
                    )
                    .into());
                }
            }
        }

        // Validate allowed values (enum)
        if let Some(allowed) = &self.allowed_values {
            if !allowed.contains(value) {
                return Err(ValidationError::invalid_value(
                    param_name,
                    format!("must be one of: {:?}", allowed),
                )
                .into());
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_required_parameter_validation() {
        let schema = ConfigSchema::new(vec![ParameterSchema::required(
            "topic",
            "Research topic",
            ParameterType::String,
        )]);

        // Missing required parameter
        let params = HashMap::new();
        assert!(schema.validate(&params).is_err());

        // Valid required parameter
        let mut params = HashMap::new();
        params.insert("topic".to_string(), json!("Rust"));
        assert!(schema.validate(&params).is_ok());
    }

    #[test]
    fn test_type_validation() {
        let schema = ConfigSchema::new(vec![ParameterSchema::required(
            "max_sources",
            "Maximum sources",
            ParameterType::Integer,
        )]);

        // Wrong type
        let mut params = HashMap::new();
        params.insert("max_sources".to_string(), json!("not a number"));
        assert!(schema.validate(&params).is_err());

        // Correct type
        let mut params = HashMap::new();
        params.insert("max_sources".to_string(), json!(10));
        assert!(schema.validate(&params).is_ok());
    }

    #[test]
    fn test_numeric_constraints() {
        let schema = ConfigSchema::new(vec![ParameterSchema::required(
            "max_sources",
            "Maximum sources",
            ParameterType::Integer,
        )
        .with_constraints(ParameterConstraints {
            min: Some(1.0),
            max: Some(100.0),
            ..Default::default()
        })]);

        // Below minimum
        let mut params = HashMap::new();
        params.insert("max_sources".to_string(), json!(0));
        assert!(schema.validate(&params).is_err());

        // Above maximum
        let mut params = HashMap::new();
        params.insert("max_sources".to_string(), json!(101));
        assert!(schema.validate(&params).is_err());

        // Within range
        let mut params = HashMap::new();
        params.insert("max_sources".to_string(), json!(50));
        assert!(schema.validate(&params).is_ok());
    }

    #[test]
    fn test_string_length_constraints() {
        let schema = ConfigSchema::new(vec![ParameterSchema::required(
            "topic",
            "Research topic",
            ParameterType::String,
        )
        .with_constraints(ParameterConstraints {
            min_length: Some(3),
            max_length: Some(100),
            ..Default::default()
        })]);

        // Too short
        let mut params = HashMap::new();
        params.insert("topic".to_string(), json!("ab"));
        assert!(schema.validate(&params).is_err());

        // Valid length
        let mut params = HashMap::new();
        params.insert("topic".to_string(), json!("Rust"));
        assert!(schema.validate(&params).is_ok());
    }

    #[test]
    fn test_optional_parameter_with_default() {
        let schema = ConfigSchema::new(vec![ParameterSchema::optional(
            "model",
            "LLM model",
            ParameterType::String,
            json!("ollama/llama3.2:3b"),
        )]);

        // Missing optional parameter is OK
        let params = HashMap::new();
        assert!(schema.validate(&params).is_ok());

        // Provided optional parameter is validated
        let mut params = HashMap::new();
        params.insert("model".to_string(), json!("ollama/llama3.2:3b"));
        assert!(schema.validate(&params).is_ok());
    }
}
