//! ABOUTME: Data validation tool with built-in and custom validation rules
//! ABOUTME: Provides comprehensive data validation with detailed error reporting

use async_trait::async_trait;
use llmspell_core::{
    traits::{
        base_agent::BaseAgent,
        tool::{ParameterDef, ParameterType, SecurityLevel, Tool, ToolCategory, ToolSchema},
    },
    types::{AgentInput, AgentOutput, ExecutionContext},
    ComponentMetadata, LLMSpellError, Result,
};
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use tracing::info;

/// Validation rule types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ValidationRule {
    /// Required field validation
    Required,

    /// Type validation
    Type { expected: String },

    /// String length validation
    Length {
        min: Option<usize>,
        max: Option<usize>,
    },

    /// Numeric range validation
    Range { min: Option<f64>, max: Option<f64> },

    /// Regular expression pattern matching
    Pattern { regex: String },

    /// Enumeration validation
    Enum { values: Vec<Value> },

    /// Email validation
    Email,

    /// URL validation
    Url,

    /// Date format validation
    Date { format: String },

    /// Custom validation function (name references predefined validators)
    Custom { name: String },

    /// Array validation
    Array {
        min_items: Option<usize>,
        max_items: Option<usize>,
        unique: bool,
        item_rules: Option<Box<ValidationRules>>,
    },

    /// Object validation
    Object {
        properties: HashMap<String, ValidationRules>,
        required: Vec<String>,
        additional_properties: bool,
    },
}

/// Collection of validation rules for a field
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ValidationRules {
    pub rules: Vec<ValidationRule>,
}

impl ValidationRules {
    /// Create a new set of validation rules
    pub fn new() -> Self {
        Self { rules: Vec::new() }
    }

    /// Add a rule to the collection
    pub fn add_rule(mut self, rule: ValidationRule) -> Self {
        self.rules.push(rule);
        self
    }
}

/// Validation error details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    pub field: String,
    pub value: Value,
    pub rule: String,
    pub message: String,
}

/// Validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub valid: bool,
    pub errors: Vec<ValidationError>,
}

/// Data validation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataValidationConfig {
    /// Whether to stop on first error
    #[serde(default = "default_fail_fast")]
    pub fail_fast: bool,

    /// Maximum validation errors to collect
    #[serde(default = "default_max_errors")]
    pub max_errors: usize,

    /// Whether to validate nested structures
    #[serde(default = "default_validate_nested")]
    pub validate_nested: bool,

    /// Custom error messages
    #[serde(default)]
    pub custom_messages: HashMap<String, String>,
}

fn default_fail_fast() -> bool {
    false
}
fn default_max_errors() -> usize {
    100
}
fn default_validate_nested() -> bool {
    true
}

impl Default for DataValidationConfig {
    fn default() -> Self {
        Self {
            fail_fast: default_fail_fast(),
            max_errors: default_max_errors(),
            validate_nested: default_validate_nested(),
            custom_messages: HashMap::new(),
        }
    }
}

/// Type alias for custom validator functions
type ValidatorFn = Box<dyn Fn(&Value) -> Result<()> + Send + Sync>;

/// Data validation tool
pub struct DataValidationTool {
    metadata: ComponentMetadata,
    config: DataValidationConfig,
    custom_validators: HashMap<String, ValidatorFn>,
}

impl DataValidationTool {
    /// Create a new data validation tool
    pub fn new() -> Self {
        Self::with_config(DataValidationConfig::default())
    }

    /// Create with custom configuration
    pub fn with_config(config: DataValidationConfig) -> Self {
        let mut tool = Self {
            metadata: ComponentMetadata::new(
                "data-validation-tool".to_string(),
                "Validate data against custom rules with detailed error reporting".to_string(),
            ),
            config,
            custom_validators: HashMap::new(),
        };

        // Register built-in custom validators
        tool.register_builtin_validators();
        tool
    }

    /// Register built-in custom validators
    fn register_builtin_validators(&mut self) {
        // Phone number validator
        self.custom_validators.insert(
            "phone".to_string(),
            Box::new(|value| {
                if let Some(s) = value.as_str() {
                    let phone_regex = Regex::new(r"^\+?[1-9]\d{1,14}$").unwrap();
                    if phone_regex.is_match(s) {
                        Ok(())
                    } else {
                        Err(LLMSpellError::Validation {
                            message: "Invalid phone number format".to_string(),
                            field: None,
                        })
                    }
                } else {
                    Err(LLMSpellError::Validation {
                        message: "Value must be a string".to_string(),
                        field: None,
                    })
                }
            }),
        );

        // UUID validator
        self.custom_validators.insert(
            "uuid".to_string(),
            Box::new(|value| {
                if let Some(s) = value.as_str() {
                    let uuid_regex = Regex::new(
                        r"^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}$"
                    ).unwrap();
                    if uuid_regex.is_match(s) {
                        Ok(())
                    } else {
                        Err(LLMSpellError::Validation {
                            message: "Invalid UUID format".to_string(),
                            field: None,
                        })
                    }
                } else {
                    Err(LLMSpellError::Validation {
                        message: "Value must be a string".to_string(),
                        field: None,
                    })
                }
            }),
        );

        // Credit card validator (basic Luhn check)
        self.custom_validators.insert(
            "credit_card".to_string(),
            Box::new(|value| {
                if let Some(s) = value.as_str() {
                    let cleaned: String = s.chars().filter(|c| c.is_numeric()).collect();
                    if cleaned.len() < 13 || cleaned.len() > 19 {
                        return Err(LLMSpellError::Validation {
                            message: "Invalid credit card length".to_string(),
                            field: None,
                        });
                    }

                    // Basic Luhn algorithm
                    let mut sum = 0;
                    let mut alternate = false;
                    for c in cleaned.chars().rev() {
                        let n = c.to_digit(10).unwrap();
                        if alternate {
                            let doubled = n * 2;
                            sum += if doubled > 9 { doubled - 9 } else { doubled };
                        } else {
                            sum += n;
                        }
                        alternate = !alternate;
                    }

                    if sum % 10 == 0 {
                        Ok(())
                    } else {
                        Err(LLMSpellError::Validation {
                            message: "Invalid credit card number".to_string(),
                            field: None,
                        })
                    }
                } else {
                    Err(LLMSpellError::Validation {
                        message: "Value must be a string".to_string(),
                        field: None,
                    })
                }
            }),
        );
    }

    /// Validate a value against a set of rules
    fn validate_value(
        &self,
        field: &str,
        value: &Value,
        rules: &ValidationRules,
        errors: &mut Vec<ValidationError>,
    ) -> Result<()> {
        for rule in &rules.rules {
            if errors.len() >= self.config.max_errors {
                break;
            }

            if let Err(e) = self.validate_rule(field, value, rule, errors) {
                errors.push(ValidationError {
                    field: field.to_string(),
                    value: value.clone(),
                    rule: format!("{:?}", rule),
                    message: e.to_string(),
                });

                if self.config.fail_fast {
                    break;
                }
            }
        }
        Ok(())
    }

    /// Validate a single rule
    fn validate_rule(
        &self,
        field: &str,
        value: &Value,
        rule: &ValidationRule,
        errors: &mut Vec<ValidationError>,
    ) -> Result<()> {
        match rule {
            ValidationRule::Required => {
                if value.is_null() || (value.is_string() && value.as_str().unwrap().is_empty()) {
                    return Err(LLMSpellError::Validation {
                        message: self.get_error_message("required", field),
                        field: Some(field.to_string()),
                    });
                }
            }

            ValidationRule::Type { expected } => {
                let actual_type = match value {
                    Value::Null => "null",
                    Value::Bool(_) => "boolean",
                    Value::Number(_) => "number",
                    Value::String(_) => "string",
                    Value::Array(_) => "array",
                    Value::Object(_) => "object",
                };

                if actual_type != expected {
                    return Err(LLMSpellError::Validation {
                        message: format!("Expected type '{}', got '{}'", expected, actual_type),
                        field: Some(field.to_string()),
                    });
                }
            }

            ValidationRule::Length { min, max } => {
                if let Some(s) = value.as_str() {
                    let len = s.len();
                    if let Some(min_len) = min {
                        if len < *min_len {
                            return Err(LLMSpellError::Validation {
                                message: format!("Length must be at least {}", min_len),
                                field: Some(field.to_string()),
                            });
                        }
                    }
                    if let Some(max_len) = max {
                        if len > *max_len {
                            return Err(LLMSpellError::Validation {
                                message: format!("Length must be at most {}", max_len),
                                field: Some(field.to_string()),
                            });
                        }
                    }
                } else if let Some(arr) = value.as_array() {
                    let len = arr.len();
                    if let Some(min_len) = min {
                        if len < *min_len {
                            return Err(LLMSpellError::Validation {
                                message: format!("Array must have at least {} items", min_len),
                                field: Some(field.to_string()),
                            });
                        }
                    }
                    if let Some(max_len) = max {
                        if len > *max_len {
                            return Err(LLMSpellError::Validation {
                                message: format!("Array must have at most {} items", max_len),
                                field: Some(field.to_string()),
                            });
                        }
                    }
                }
            }

            ValidationRule::Range { min, max } => {
                if let Some(n) = value.as_f64() {
                    if let Some(min_val) = min {
                        if n < *min_val {
                            return Err(LLMSpellError::Validation {
                                message: format!("Value must be at least {}", min_val),
                                field: Some(field.to_string()),
                            });
                        }
                    }
                    if let Some(max_val) = max {
                        if n > *max_val {
                            return Err(LLMSpellError::Validation {
                                message: format!("Value must be at most {}", max_val),
                                field: Some(field.to_string()),
                            });
                        }
                    }
                }
            }

            ValidationRule::Pattern { regex } => {
                if let Some(s) = value.as_str() {
                    let re = Regex::new(regex).map_err(|e| LLMSpellError::Validation {
                        message: format!("Invalid regex pattern: {}", e),
                        field: Some(field.to_string()),
                    })?;

                    if !re.is_match(s) {
                        return Err(LLMSpellError::Validation {
                            message: format!("Value does not match pattern: {}", regex),
                            field: Some(field.to_string()),
                        });
                    }
                }
            }

            ValidationRule::Enum { values } => {
                if !values.contains(value) {
                    return Err(LLMSpellError::Validation {
                        message: format!("Value must be one of: {:?}", values),
                        field: Some(field.to_string()),
                    });
                }
            }

            ValidationRule::Email => {
                if let Some(s) = value.as_str() {
                    let email_regex =
                        Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();
                    if !email_regex.is_match(s) {
                        return Err(LLMSpellError::Validation {
                            message: "Invalid email format".to_string(),
                            field: Some(field.to_string()),
                        });
                    }
                }
            }

            ValidationRule::Url => {
                if let Some(s) = value.as_str() {
                    if url::Url::parse(s).is_err() {
                        return Err(LLMSpellError::Validation {
                            message: "Invalid URL format".to_string(),
                            field: Some(field.to_string()),
                        });
                    }
                }
            }

            ValidationRule::Date { format } => {
                if let Some(s) = value.as_str() {
                    if chrono::NaiveDateTime::parse_from_str(s, format).is_err() {
                        return Err(LLMSpellError::Validation {
                            message: format!("Invalid date format. Expected: {}", format),
                            field: Some(field.to_string()),
                        });
                    }
                }
            }

            ValidationRule::Custom { name } => {
                if let Some(validator) = self.custom_validators.get(name) {
                    validator(value)?;
                } else {
                    return Err(LLMSpellError::Validation {
                        message: format!("Unknown custom validator: {}", name),
                        field: Some(field.to_string()),
                    });
                }
            }

            ValidationRule::Array {
                min_items,
                max_items,
                unique,
                item_rules,
            } => {
                if let Some(arr) = value.as_array() {
                    // Check array length
                    if let Some(min) = min_items {
                        if arr.len() < *min {
                            return Err(LLMSpellError::Validation {
                                message: format!("Array must have at least {} items", min),
                                field: Some(field.to_string()),
                            });
                        }
                    }
                    if let Some(max) = max_items {
                        if arr.len() > *max {
                            return Err(LLMSpellError::Validation {
                                message: format!("Array must have at most {} items", max),
                                field: Some(field.to_string()),
                            });
                        }
                    }

                    // Check uniqueness
                    if *unique {
                        let mut seen = std::collections::HashSet::new();
                        for item in arr {
                            let serialized = serde_json::to_string(item).unwrap();
                            if !seen.insert(serialized) {
                                return Err(LLMSpellError::Validation {
                                    message: "Array items must be unique".to_string(),
                                    field: Some(field.to_string()),
                                });
                            }
                        }
                    }

                    // Validate items if rules provided
                    if self.config.validate_nested {
                        if let Some(rules) = item_rules {
                            for (i, item) in arr.iter().enumerate() {
                                let item_field = format!("{}[{}]", field, i);
                                self.validate_value(&item_field, item, rules, errors)?;
                            }
                        }
                    }
                }
            }

            ValidationRule::Object {
                properties,
                required,
                additional_properties,
            } => {
                if let Some(obj) = value.as_object() {
                    // Check required fields
                    for req_field in required {
                        if !obj.contains_key(req_field) {
                            return Err(LLMSpellError::Validation {
                                message: format!("Missing required field: {}", req_field),
                                field: Some(format!("{}.{}", field, req_field)),
                            });
                        }
                    }

                    // Check for additional properties
                    if !additional_properties {
                        for key in obj.keys() {
                            if !properties.contains_key(key) {
                                return Err(LLMSpellError::Validation {
                                    message: format!("Additional property not allowed: {}", key),
                                    field: Some(format!("{}.{}", field, key)),
                                });
                            }
                        }
                    }

                    // Validate properties
                    if self.config.validate_nested {
                        for (prop, rules) in properties {
                            if let Some(prop_value) = obj.get(prop) {
                                let prop_field = format!("{}.{}", field, prop);
                                self.validate_value(&prop_field, prop_value, rules, errors)?;
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Get custom error message or default
    fn get_error_message(&self, key: &str, field: &str) -> String {
        if let Some(msg) = self.config.custom_messages.get(key) {
            msg.replace("{field}", field)
        } else {
            match key {
                "required" => format!("{} is required", field),
                _ => format!("Validation failed for {}", field),
            }
        }
    }
}

impl Default for DataValidationTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseAgent for DataValidationTool {
    fn metadata(&self) -> &ComponentMetadata {
        &self.metadata
    }

    async fn execute(&self, input: AgentInput, _context: ExecutionContext) -> Result<AgentOutput> {
        let params =
            input
                .parameters
                .get("parameters")
                .ok_or_else(|| LLMSpellError::Validation {
                    message: "Missing parameters".to_string(),
                    field: Some("parameters".to_string()),
                })?;

        // Extract parameters
        let data = params
            .get("data")
            .ok_or_else(|| LLMSpellError::Validation {
                message: "Missing 'data' parameter".to_string(),
                field: Some("data".to_string()),
            })?;

        let rules = params
            .get("rules")
            .ok_or_else(|| LLMSpellError::Validation {
                message: "Missing 'rules' parameter".to_string(),
                field: Some("rules".to_string()),
            })?;

        // Parse validation rules
        let validation_rules: ValidationRules =
            serde_json::from_value(rules.clone()).map_err(|e| LLMSpellError::Validation {
                message: format!("Invalid rules format: {}", e),
                field: Some("rules".to_string()),
            })?;

        info!(
            "Validating data against {} rules",
            validation_rules.rules.len()
        );

        // Perform validation
        let mut errors = Vec::new();
        self.validate_value("data", data, &validation_rules, &mut errors)?;

        let result = ValidationResult {
            valid: errors.is_empty(),
            errors,
        };

        // Create output

        // Add metadata
        let mut metadata = llmspell_core::types::OutputMetadata::default();
        metadata.extra.insert(
            "rule_count".to_string(),
            Value::Number(serde_json::Number::from(validation_rules.rules.len())),
        );
        metadata.extra.insert(
            "error_count".to_string(),
            Value::Number(serde_json::Number::from(result.errors.len())),
        );

        Ok(AgentOutput::text(serde_json::to_string_pretty(&result)?).with_metadata(metadata))
    }

    async fn validate_input(&self, input: &AgentInput) -> Result<()> {
        if input.parameters.is_empty() {
            return Err(LLMSpellError::Validation {
                message: "No parameters provided".to_string(),
                field: Some("parameters".to_string()),
            });
        }
        Ok(())
    }

    async fn handle_error(&self, error: LLMSpellError) -> Result<AgentOutput> {
        Ok(AgentOutput::text(format!("Validation error: {}", error)))
    }
}

#[async_trait]
impl Tool for DataValidationTool {
    fn category(&self) -> ToolCategory {
        ToolCategory::Utility
    }

    fn security_level(&self) -> SecurityLevel {
        SecurityLevel::Safe
    }

    fn schema(&self) -> ToolSchema {
        ToolSchema {
            name: "data_validation".to_string(),
            description: "Validate data against custom rules with detailed error reporting"
                .to_string(),
            parameters: vec![
                ParameterDef {
                    name: "data".to_string(),
                    description: "Data to validate".to_string(),
                    param_type: ParameterType::Object,
                    required: true,
                    default: None,
                },
                ParameterDef {
                    name: "rules".to_string(),
                    description: "Validation rules to apply".to_string(),
                    param_type: ParameterType::Object,
                    required: true,
                    default: None,
                },
            ],
            returns: Some(ParameterType::Object),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_required_validation() {
        let tool = DataValidationTool::new();

        let params = serde_json::json!({
            "data": null,
            "rules": {
                "rules": [
                    {"type": "required"}
                ]
            }
        });

        let input = AgentInput::text("").with_parameter("parameters", params);
        let result = tool
            .execute(input, ExecutionContext::default())
            .await
            .unwrap();

        let validation_result: ValidationResult = serde_json::from_str(&result.text).unwrap();
        assert!(!validation_result.valid);
        assert_eq!(validation_result.errors.len(), 1);
    }

    #[tokio::test]
    async fn test_type_validation() {
        let tool = DataValidationTool::new();

        let params = serde_json::json!({
            "data": "hello",
            "rules": {
                "rules": [
                    {"type": "type", "expected": "number"}
                ]
            }
        });

        let input = AgentInput::text("").with_parameter("parameters", params);
        let result = tool
            .execute(input, ExecutionContext::default())
            .await
            .unwrap();

        let validation_result: ValidationResult = serde_json::from_str(&result.text).unwrap();
        assert!(!validation_result.valid);
        assert!(validation_result.errors[0]
            .message
            .contains("Expected type 'number'"));
    }

    #[tokio::test]
    async fn test_length_validation() {
        let tool = DataValidationTool::new();

        let params = serde_json::json!({
            "data": "hi",
            "rules": {
                "rules": [
                    {"type": "length", "min": 3, "max": 10}
                ]
            }
        });

        let input = AgentInput::text("").with_parameter("parameters", params);
        let result = tool
            .execute(input, ExecutionContext::default())
            .await
            .unwrap();

        let validation_result: ValidationResult = serde_json::from_str(&result.text).unwrap();
        assert!(!validation_result.valid);
        assert!(validation_result.errors[0].message.contains("at least 3"));
    }

    #[tokio::test]
    async fn test_email_validation() {
        let tool = DataValidationTool::new();

        // Valid email
        let params = serde_json::json!({
            "data": "test@example.com",
            "rules": {
                "rules": [
                    {"type": "email"}
                ]
            }
        });

        let input = AgentInput::text("").with_parameter("parameters", params);
        let result = tool
            .execute(input, ExecutionContext::default())
            .await
            .unwrap();

        let validation_result: ValidationResult = serde_json::from_str(&result.text).unwrap();
        assert!(validation_result.valid);

        // Invalid email
        let params = serde_json::json!({
            "data": "not-an-email",
            "rules": {
                "rules": [
                    {"type": "email"}
                ]
            }
        });

        let input = AgentInput::text("").with_parameter("parameters", params);
        let result = tool
            .execute(input, ExecutionContext::default())
            .await
            .unwrap();

        let validation_result: ValidationResult = serde_json::from_str(&result.text).unwrap();
        assert!(!validation_result.valid);
    }

    #[tokio::test]
    async fn test_custom_validator() {
        let tool = DataValidationTool::new();

        let params = serde_json::json!({
            "data": "+1234567890",
            "rules": {
                "rules": [
                    {"type": "custom", "name": "phone"}
                ]
            }
        });

        let input = AgentInput::text("").with_parameter("parameters", params);
        let result = tool
            .execute(input, ExecutionContext::default())
            .await
            .unwrap();

        let validation_result: ValidationResult = serde_json::from_str(&result.text).unwrap();
        assert!(validation_result.valid);
    }

    #[tokio::test]
    async fn test_complex_object_validation() {
        let tool = DataValidationTool::new();

        let params = serde_json::json!({
            "data": {
                "name": "John Doe",
                "email": "john@example.com",
                "age": 25
            },
            "rules": {
                "rules": [
                    {
                        "type": "object",
                        "properties": {
                            "name": {
                                "rules": [
                                    {"type": "required"},
                                    {"type": "type", "expected": "string"},
                                    {"type": "length", "min": 1, "max": 50}
                                ]
                            },
                            "email": {
                                "rules": [
                                    {"type": "required"},
                                    {"type": "email"}
                                ]
                            },
                            "age": {
                                "rules": [
                                    {"type": "type", "expected": "number"},
                                    {"type": "range", "min": 18, "max": 120}
                                ]
                            }
                        },
                        "required": ["name", "email"],
                        "additional_properties": false
                    }
                ]
            }
        });

        let input = AgentInput::text("").with_parameter("parameters", params);
        let result = tool
            .execute(input, ExecutionContext::default())
            .await
            .unwrap();

        let validation_result: ValidationResult = serde_json::from_str(&result.text).unwrap();
        assert!(validation_result.valid);
    }
}
