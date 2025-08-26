//! ABOUTME: Data validation tool with built-in and custom validation rules
//! ABOUTME: Provides comprehensive data validation with detailed error reporting

use async_trait::async_trait;
use llmspell_core::{
    traits::{
        base_agent::BaseAgent,
        tool::{ParameterDef, ParameterType, SecurityLevel, Tool, ToolCategory, ToolSchema},
    },
    types::{AgentInput, AgentOutput},
    ComponentMetadata, ExecutionContext, LLMSpellError, Result,
};
use llmspell_utils::{
    error_builders::llmspell::validation_error, params::extract_parameters,
    response::ResponseBuilder,
};
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
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
    #[must_use]
    pub const fn new() -> Self {
        Self { rules: Vec::new() }
    }

    /// Add a rule to the collection
    #[must_use]
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

const fn default_fail_fast() -> bool {
    false
}
const fn default_max_errors() -> usize {
    100
}
const fn default_validate_nested() -> bool {
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
    #[must_use]
    pub fn new() -> Self {
        Self::with_config(DataValidationConfig::default())
    }

    /// Create with custom configuration
    #[must_use]
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
                value.as_str().map_or_else(
                    || Err(validation_error("Value must be a string", None)),
                    |s| {
                        let phone_regex = Regex::new(r"^\+?[1-9]\d{1,14}$").unwrap();
                        if phone_regex.is_match(s) {
                            Ok(())
                        } else {
                            Err(validation_error("Invalid phone number format", None))
                        }
                    },
                )
            }),
        );

        // UUID validator
        self.custom_validators.insert(
            "uuid".to_string(),
            Box::new(|value| {
                value.as_str().map_or_else(|| Err(validation_error(
                    "Value must be a string",
                    None,
                )), |s| {
                    let uuid_regex = Regex::new(
                        r"^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}$"
                    ).unwrap();
                    if uuid_regex.is_match(s) {
                        Ok(())
                    } else {
                        Err(validation_error(
                            "Invalid UUID format",
                            None,
                        ))
                    }
                })
            }),
        );

        // Credit card validator (basic Luhn check)
        self.custom_validators.insert(
            "credit_card".to_string(),
            Box::new(|value| {
                if let Some(s) = value.as_str() {
                    let cleaned: String = s.chars().filter(|c| c.is_numeric()).collect();
                    if cleaned.len() < 13 || cleaned.len() > 19 {
                        return Err(validation_error("Invalid credit card length", None));
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
                        Err(validation_error("Invalid credit card number", None))
                    }
                } else {
                    Err(validation_error("Value must be a string", None))
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
    ) {
        for rule in &rules.rules {
            if errors.len() >= self.config.max_errors {
                break;
            }

            if let Err(e) = self.validate_rule(field, value, rule, errors) {
                errors.push(ValidationError {
                    field: field.to_string(),
                    value: value.clone(),
                    rule: format!("{rule:?}"),
                    message: e.to_string(),
                });

                if self.config.fail_fast {
                    break;
                }
            }
        }
    }

    /// Validate a single rule
    #[allow(clippy::too_many_lines, clippy::cognitive_complexity)]
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
                    return Err(validation_error(
                        self.get_error_message("required", field),
                        Some(field.to_string()),
                    ));
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
                    return Err(validation_error(
                        format!("Expected type '{expected}', got '{actual_type}'"),
                        Some(field.to_string()),
                    ));
                }
            }

            ValidationRule::Length { min, max } => {
                if let Some(s) = value.as_str() {
                    let len = s.len();
                    if let Some(min_len) = min {
                        if len < *min_len {
                            return Err(validation_error(
                                format!("Length must be at least {min_len}"),
                                Some(field.to_string()),
                            ));
                        }
                    }
                    if let Some(max_len) = max {
                        if len > *max_len {
                            return Err(validation_error(
                                format!("Length must be at most {max_len}"),
                                Some(field.to_string()),
                            ));
                        }
                    }
                } else if let Some(arr) = value.as_array() {
                    let len = arr.len();
                    if let Some(min_len) = min {
                        if len < *min_len {
                            return Err(validation_error(
                                format!("Array must have at least {min_len} items"),
                                Some(field.to_string()),
                            ));
                        }
                    }
                    if let Some(max_len) = max {
                        if len > *max_len {
                            return Err(validation_error(
                                format!("Array must have at most {max_len} items"),
                                Some(field.to_string()),
                            ));
                        }
                    }
                }
            }

            ValidationRule::Range { min, max } => {
                if let Some(n) = value.as_f64() {
                    if let Some(min_val) = min {
                        if n < *min_val {
                            return Err(validation_error(
                                format!("Value must be at least {min_val}"),
                                Some(field.to_string()),
                            ));
                        }
                    }
                    if let Some(max_val) = max {
                        if n > *max_val {
                            return Err(validation_error(
                                format!("Value must be at most {max_val}"),
                                Some(field.to_string()),
                            ));
                        }
                    }
                }
            }

            ValidationRule::Pattern { regex } => {
                if let Some(s) = value.as_str() {
                    let re = Regex::new(regex).map_err(|e| {
                        validation_error(
                            format!("Invalid regex pattern: {e}"),
                            Some(field.to_string()),
                        )
                    })?;

                    if !re.is_match(s) {
                        return Err(validation_error(
                            format!("Value does not match pattern: {regex}"),
                            Some(field.to_string()),
                        ));
                    }
                }
            }

            ValidationRule::Enum { values } => {
                if !values.contains(value) {
                    return Err(validation_error(
                        format!("Value must be one of: {values:?}"),
                        Some(field.to_string()),
                    ));
                }
            }

            ValidationRule::Email => {
                if let Some(s) = value.as_str() {
                    let email_regex =
                        Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();
                    if !email_regex.is_match(s) {
                        return Err(validation_error(
                            "Invalid email format",
                            Some(field.to_string()),
                        ));
                    }
                }
            }

            ValidationRule::Url => {
                if let Some(s) = value.as_str() {
                    if url::Url::parse(s).is_err() {
                        return Err(validation_error(
                            "Invalid URL format",
                            Some(field.to_string()),
                        ));
                    }
                }
            }

            ValidationRule::Date { format } => {
                if let Some(s) = value.as_str() {
                    if chrono::NaiveDateTime::parse_from_str(s, format).is_err() {
                        return Err(validation_error(
                            format!("Invalid date format. Expected: {format}"),
                            Some(field.to_string()),
                        ));
                    }
                }
            }

            ValidationRule::Custom { name } => {
                if let Some(validator) = self.custom_validators.get(name) {
                    validator(value)?;
                } else {
                    return Err(validation_error(
                        format!("Unknown custom validator: {name}"),
                        Some(field.to_string()),
                    ));
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
                            return Err(validation_error(
                                format!("Array must have at least {min} items"),
                                Some(field.to_string()),
                            ));
                        }
                    }
                    if let Some(max) = max_items {
                        if arr.len() > *max {
                            return Err(validation_error(
                                format!("Array must have at most {max} items"),
                                Some(field.to_string()),
                            ));
                        }
                    }

                    // Check uniqueness
                    if *unique {
                        let mut seen = std::collections::HashSet::new();
                        for item in arr {
                            let serialized = serde_json::to_string(item).unwrap();
                            if !seen.insert(serialized) {
                                return Err(validation_error(
                                    "Array items must be unique",
                                    Some(field.to_string()),
                                ));
                            }
                        }
                    }

                    // Validate items if rules provided
                    if self.config.validate_nested {
                        if let Some(rules) = item_rules {
                            for (i, item) in arr.iter().enumerate() {
                                let item_field = format!("{field}[{i}]");
                                self.validate_value(&item_field, item, rules, errors);
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
                            return Err(validation_error(
                                format!("Missing required field: {req_field}"),
                                Some(format!("{field}.{req_field}")),
                            ));
                        }
                    }

                    // Check for additional properties
                    if !additional_properties {
                        for key in obj.keys() {
                            if !properties.contains_key(key) {
                                return Err(validation_error(
                                    format!("Additional property not allowed: {key}"),
                                    Some(format!("{field}.{key}")),
                                ));
                            }
                        }
                    }

                    // Validate properties
                    if self.config.validate_nested {
                        for (prop, rules) in properties {
                            if let Some(prop_value) = obj.get(prop) {
                                let prop_field = format!("{field}.{prop}");
                                self.validate_value(&prop_field, prop_value, rules, errors);
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
        self.config.custom_messages.get(key).map_or_else(
            || match key {
                "required" => format!("{field} is required"),
                _ => format!("Validation failed for {field}"),
            },
            |msg| msg.replace(&format!("{}{}{}", "{", "field", "}"), field),
        )
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

    async fn execute_impl(
        &self,
        input: AgentInput,
        _context: ExecutionContext,
    ) -> Result<AgentOutput> {
        // Get parameters using shared utility
        let params = extract_parameters(&input)?;

        // Extract parameters
        let data = params.get("input").ok_or_else(|| {
            validation_error("Missing 'input' parameter", Some("input".to_string()))
        })?;

        let rules = params.get("rules").ok_or_else(|| {
            validation_error("Missing 'rules' parameter", Some("rules".to_string()))
        })?;

        // Parse validation rules
        let validation_rules: ValidationRules =
            serde_json::from_value(rules.clone()).map_err(|e| {
                validation_error(
                    format!("Invalid rules format: {e}"),
                    Some("rules".to_string()),
                )
            })?;

        info!(
            "Validating data against {} rules",
            validation_rules.rules.len()
        );

        // Perform validation
        let mut errors = Vec::new();
        self.validate_value("input", data, &validation_rules, &mut errors);

        let result = ValidationResult {
            valid: errors.is_empty(),
            errors,
        };

        // Create response using ResponseBuilder
        let response = ResponseBuilder::success("validate_data")
            .with_message(if result.valid {
                "Data validation passed"
            } else {
                "Data validation failed"
            })
            .with_result(serde_json::to_value(&result)?)
            .with_metadata("rule_count", json!(validation_rules.rules.len()))
            .with_metadata("error_count", json!(result.errors.len()))
            .build();

        Ok(AgentOutput::text(serde_json::to_string_pretty(&response)?))
    }

    async fn validate_input(&self, input: &AgentInput) -> Result<()> {
        if input.parameters.is_empty() {
            return Err(validation_error(
                "No parameters provided",
                Some("parameters".to_string()),
            ));
        }
        Ok(())
    }

    async fn handle_error(&self, error: LLMSpellError) -> Result<AgentOutput> {
        Ok(AgentOutput::text(format!("Validation error: {error}")))
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
                    name: "input".to_string(),
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
            "input": null,
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

        let output: Value = serde_json::from_str(&result.text).unwrap();
        assert!(output["success"].as_bool().unwrap_or(false));
        let validation_result: ValidationResult =
            serde_json::from_value(output["result"].clone()).unwrap();
        assert!(!validation_result.valid);
        assert_eq!(validation_result.errors.len(), 1);
    }
    #[tokio::test]
    async fn test_type_validation() {
        let tool = DataValidationTool::new();

        let params = serde_json::json!({
            "input": "hello",
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

        let output: Value = serde_json::from_str(&result.text).unwrap();
        assert!(output["success"].as_bool().unwrap_or(false));
        let validation_result: ValidationResult =
            serde_json::from_value(output["result"].clone()).unwrap();
        assert!(!validation_result.valid);
        assert!(validation_result.errors[0]
            .message
            .contains("Expected type 'number'"));
    }
    #[tokio::test]
    async fn test_length_validation() {
        let tool = DataValidationTool::new();

        let params = serde_json::json!({
            "input": "hi",
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

        let output: Value = serde_json::from_str(&result.text).unwrap();
        assert!(output["success"].as_bool().unwrap_or(false));
        let validation_result: ValidationResult =
            serde_json::from_value(output["result"].clone()).unwrap();
        assert!(!validation_result.valid);
        assert!(validation_result.errors[0].message.contains("at least 3"));
    }
    #[tokio::test]
    async fn test_email_validation() {
        let tool = DataValidationTool::new();

        // Valid email
        let params = serde_json::json!({
            "input": "test@example.com",
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

        let output: Value = serde_json::from_str(&result.text).unwrap();
        assert!(output["success"].as_bool().unwrap_or(false));
        let validation_result: ValidationResult =
            serde_json::from_value(output["result"].clone()).unwrap();
        assert!(validation_result.valid);

        // Invalid email
        let params = serde_json::json!({
            "input": "not-an-email",
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

        let output: Value = serde_json::from_str(&result.text).unwrap();
        assert!(output["success"].as_bool().unwrap_or(false));
        let validation_result: ValidationResult =
            serde_json::from_value(output["result"].clone()).unwrap();
        assert!(!validation_result.valid);
    }
    #[tokio::test]
    async fn test_custom_validator() {
        let tool = DataValidationTool::new();

        let params = serde_json::json!({
            "input": "+1234567890",
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

        let output: Value = serde_json::from_str(&result.text).unwrap();
        assert!(output["success"].as_bool().unwrap_or(false));
        let validation_result: ValidationResult =
            serde_json::from_value(output["result"].clone()).unwrap();
        assert!(validation_result.valid);
    }
    #[tokio::test]
    async fn test_complex_object_validation() {
        let tool = DataValidationTool::new();

        let params = serde_json::json!({
            "input": {
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

        let output: Value = serde_json::from_str(&result.text).unwrap();
        assert!(output["success"].as_bool().unwrap_or(false));
        let validation_result: ValidationResult =
            serde_json::from_value(output["result"].clone()).unwrap();
        assert!(validation_result.valid);
    }
}
