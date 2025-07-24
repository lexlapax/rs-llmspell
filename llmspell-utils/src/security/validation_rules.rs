//! ABOUTME: Validation rules framework for input sanitization
//! ABOUTME: Provides reusable validation rules and sanitization pipelines for tools

use super::input_sanitizer::{InputSanitizer, SanitizationConfig, ValidationReport};
use crate::params::ParamValue;
use lazy_static::lazy_static;
use llmspell_core::{LLMSpellError, Result as LLMResult};
use regex::Regex;
use serde_json::Value;
use std::collections::HashMap;

/// Validation rule that can be applied to input
pub trait ValidationRule: Send + Sync {
    /// Name of the validation rule
    fn name(&self) -> &str;

    /// Validate input and return sanitized version or error
    fn validate(&self, input: &str) -> Result<String, ValidationError>;

    /// Check if input is valid without modifying it
    fn is_valid(&self, input: &str) -> bool {
        self.validate(input).is_ok()
    }
}

/// Collection of validation rules
pub struct ValidationRuleSet {
    rules: Vec<Box<dyn ValidationRule>>,
    config: ValidationConfig,
}

/// Configuration for validation
#[derive(Debug, Clone)]
pub struct ValidationConfig {
    /// Stop on first validation error
    pub fail_fast: bool,
    /// Apply sanitization automatically
    pub auto_sanitize: bool,
    /// Maximum validation time per rule
    pub max_validation_time_ms: u64,
}

impl Default for ValidationConfig {
    fn default() -> Self {
        Self {
            fail_fast: true,
            auto_sanitize: true,
            max_validation_time_ms: 100,
        }
    }
}

impl ValidationRuleSet {
    /// Create a new rule set
    #[must_use]
    pub fn new() -> Self {
        Self {
            rules: Vec::new(),
            config: ValidationConfig::default(),
        }
    }

    /// Create with configuration
    #[must_use]
    pub fn with_config(config: ValidationConfig) -> Self {
        Self {
            rules: Vec::new(),
            config,
        }
    }

    /// Add a validation rule
    pub fn add_rule(mut self, rule: Box<dyn ValidationRule>) -> Self {
        self.rules.push(rule);
        self
    }

    /// Validate input against all rules
    pub fn validate(&self, input: &str) -> Result<String, ValidationError> {
        let mut current = input.to_string();

        for rule in &self.rules {
            match rule.validate(&current) {
                Ok(sanitized) => {
                    if self.config.auto_sanitize {
                        current = sanitized;
                    }
                }
                Err(e) => {
                    if self.config.fail_fast {
                        return Err(e);
                    }
                }
            }
        }

        Ok(current)
    }

    /// Create a standard rule set for web inputs
    pub fn web_standard() -> Self {
        Self::new()
            .add_rule(Box::new(LengthRule::new(1_000_000)))
            .add_rule(Box::new(HtmlSanitizationRule::new()))
            .add_rule(Box::new(JavaScriptSanitizationRule::new()))
            .add_rule(Box::new(UrlValidationRule::new()))
    }

    /// Create a standard rule set for SQL inputs
    pub fn sql_standard() -> Self {
        Self::new()
            .add_rule(Box::new(LengthRule::new(10_000)))
            .add_rule(Box::new(SqlSanitizationRule::new()))
            .add_rule(Box::new(NoNullBytesRule::new()))
    }

    /// Create a standard rule set for command inputs
    pub fn command_standard() -> Self {
        Self::new()
            .add_rule(Box::new(LengthRule::new(1_000)))
            .add_rule(Box::new(CommandSanitizationRule::new()))
            .add_rule(Box::new(NoNullBytesRule::new()))
            .add_rule(Box::new(AlphanumericRule::with_allowed(".-_/")))
    }

    /// Create a standard rule set for file paths
    pub fn path_standard() -> Self {
        Self::new()
            .add_rule(Box::new(LengthRule::new(4096)))
            .add_rule(Box::new(PathSanitizationRule::new()))
            .add_rule(Box::new(NoNullBytesRule::new()))
    }
}

/// Validation error
#[derive(Debug, Clone)]
pub struct ValidationError {
    /// Rule that failed
    pub rule: String,
    /// Error message
    pub message: String,
    /// Original input that failed
    pub input: String,
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Validation failed ({}): {}", self.rule, self.message)
    }
}

impl std::error::Error for ValidationError {}

// Built-in validation rules

/// Length validation rule
pub struct LengthRule {
    max_length: usize,
}

impl LengthRule {
    #[must_use]
    pub fn new(max_length: usize) -> Self {
        Self { max_length }
    }
}

impl ValidationRule for LengthRule {
    fn name(&self) -> &str {
        "length"
    }

    fn validate(&self, input: &str) -> Result<String, ValidationError> {
        if input.len() > self.max_length {
            Err(ValidationError {
                rule: self.name().to_string(),
                message: format!("Input too long: {} > {}", input.len(), self.max_length),
                input: input.to_string(),
            })
        } else {
            Ok(input.to_string())
        }
    }
}

/// HTML sanitization rule
pub struct HtmlSanitizationRule {
    sanitizer: InputSanitizer,
}

impl HtmlSanitizationRule {
    #[must_use]
    pub fn new() -> Self {
        let config = SanitizationConfig {
            sanitize_html: true,
            sanitize_sql: false,
            sanitize_command: false,
            sanitize_format: false,
            prevent_xxe: false,
            ..Default::default()
        };
        Self {
            sanitizer: InputSanitizer::with_config(config),
        }
    }
}

impl ValidationRule for HtmlSanitizationRule {
    fn name(&self) -> &str {
        "html_sanitization"
    }

    fn validate(&self, input: &str) -> Result<String, ValidationError> {
        Ok(self.sanitizer.sanitize_html(input))
    }
}

/// JavaScript sanitization rule
pub struct JavaScriptSanitizationRule {
    patterns: Vec<Regex>,
}

impl JavaScriptSanitizationRule {
    #[must_use]
    pub fn new() -> Self {
        lazy_static! {
            static ref PATTERNS: Vec<Regex> = vec![
                Regex::new(r"(?i)<script[^>]*>.*?</script>").unwrap(),
                Regex::new(r"(?i)javascript:").unwrap(),
                Regex::new(r"(?i)on\w+\s*=").unwrap(),
                Regex::new(r"(?i)eval\s*\(").unwrap(),
                Regex::new(r"(?i)expression\s*\(").unwrap(),
            ];
        }
        Self {
            patterns: PATTERNS.clone(),
        }
    }
}

impl ValidationRule for JavaScriptSanitizationRule {
    fn name(&self) -> &str {
        "javascript_sanitization"
    }

    fn validate(&self, input: &str) -> Result<String, ValidationError> {
        let mut result = input.to_string();
        for pattern in &self.patterns {
            result = pattern.replace_all(&result, "").to_string();
        }
        Ok(result)
    }
}

/// SQL sanitization rule
pub struct SqlSanitizationRule {
    sanitizer: InputSanitizer,
}

impl SqlSanitizationRule {
    #[must_use]
    pub fn new() -> Self {
        let config = SanitizationConfig {
            sanitize_html: false,
            sanitize_sql: true,
            sanitize_command: false,
            sanitize_format: false,
            prevent_xxe: false,
            ..Default::default()
        };
        Self {
            sanitizer: InputSanitizer::with_config(config),
        }
    }
}

impl ValidationRule for SqlSanitizationRule {
    fn name(&self) -> &str {
        "sql_sanitization"
    }

    fn validate(&self, input: &str) -> Result<String, ValidationError> {
        Ok(self.sanitizer.sanitize_sql(input))
    }
}

/// Command sanitization rule
pub struct CommandSanitizationRule {
    sanitizer: InputSanitizer,
}

impl CommandSanitizationRule {
    #[must_use]
    pub fn new() -> Self {
        let config = SanitizationConfig {
            sanitize_html: false,
            sanitize_sql: false,
            sanitize_command: true,
            sanitize_format: false,
            prevent_xxe: false,
            ..Default::default()
        };
        Self {
            sanitizer: InputSanitizer::with_config(config),
        }
    }
}

impl ValidationRule for CommandSanitizationRule {
    fn name(&self) -> &str {
        "command_sanitization"
    }

    fn validate(&self, input: &str) -> Result<String, ValidationError> {
        Ok(self.sanitizer.sanitize_command(input))
    }
}

/// Path sanitization rule
pub struct PathSanitizationRule {
    sanitizer: InputSanitizer,
}

impl PathSanitizationRule {
    #[must_use]
    pub fn new() -> Self {
        Self {
            sanitizer: InputSanitizer::new(),
        }
    }
}

impl ValidationRule for PathSanitizationRule {
    fn name(&self) -> &str {
        "path_sanitization"
    }

    fn validate(&self, input: &str) -> Result<String, ValidationError> {
        self.sanitizer.sanitize_path(input).map_err(|e| ValidationError {
            rule: self.name().to_string(),
            message: e.to_string(),
            input: input.to_string(),
        })
    }
}

/// URL validation rule
pub struct UrlValidationRule {
    allowed_schemes: Vec<String>,
}

impl UrlValidationRule {
    #[must_use]
    pub fn new() -> Self {
        Self {
            allowed_schemes: vec!["http".to_string(), "https".to_string()],
        }
    }

    #[must_use]
    pub fn with_schemes(schemes: Vec<String>) -> Self {
        Self {
            allowed_schemes: schemes,
        }
    }
}

impl ValidationRule for UrlValidationRule {
    fn name(&self) -> &str {
        "url_validation"
    }

    fn validate(&self, input: &str) -> Result<String, ValidationError> {
        match url::Url::parse(input) {
            Ok(url) => {
                let scheme = url.scheme();
                if self.allowed_schemes.contains(&scheme.to_string()) {
                    Ok(input.to_string())
                } else {
                    Err(ValidationError {
                        rule: self.name().to_string(),
                        message: format!("Scheme '{}' not allowed", scheme),
                        input: input.to_string(),
                    })
                }
            }
            Err(e) => Err(ValidationError {
                rule: self.name().to_string(),
                message: format!("Invalid URL: {}", e),
                input: input.to_string(),
            }),
        }
    }
}

/// No null bytes rule
pub struct NoNullBytesRule;

impl NoNullBytesRule {
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl ValidationRule for NoNullBytesRule {
    fn name(&self) -> &str {
        "no_null_bytes"
    }

    fn validate(&self, input: &str) -> Result<String, ValidationError> {
        if input.contains('\0') {
            Ok(input.replace('\0', ""))
        } else {
            Ok(input.to_string())
        }
    }
}

/// Alphanumeric validation rule
pub struct AlphanumericRule {
    allowed_chars: String,
}

impl AlphanumericRule {
    #[must_use]
    pub fn new() -> Self {
        Self {
            allowed_chars: String::new(),
        }
    }

    #[must_use]
    pub fn with_allowed(chars: &str) -> Self {
        Self {
            allowed_chars: chars.to_string(),
        }
    }
}

impl ValidationRule for AlphanumericRule {
    fn name(&self) -> &str {
        "alphanumeric"
    }

    fn validate(&self, input: &str) -> Result<String, ValidationError> {
        let is_valid = input.chars().all(|c| {
            c.is_alphanumeric() || self.allowed_chars.contains(c)
        });

        if is_valid {
            Ok(input.to_string())
        } else {
            Err(ValidationError {
                rule: self.name().to_string(),
                message: "Input contains non-alphanumeric characters".to_string(),
                input: input.to_string(),
            })
        }
    }
}

/// Helper functions for parameter validation
pub mod param_validators {
    use super::*;

    /// Validate and sanitize a string parameter
    pub fn validate_string_param(
        value: &Value,
        param_name: &str,
        rules: &ValidationRuleSet,
    ) -> LLMResult<String> {
        let str_value = value
            .as_str()
            .ok_or_else(|| LLMSpellError::Validation {
                message: format!("Parameter '{}' must be a string", param_name),
                field: Some(param_name.to_string()),
            })?;

        rules.validate(str_value).map_err(|e| LLMSpellError::Validation {
            message: format!("Parameter '{}' validation failed: {}", param_name, e),
            field: Some(param_name.to_string()),
        })
    }

    /// Validate and sanitize parameters map
    pub fn validate_params(
        params: &serde_json::Map<String, Value>,
        param_rules: HashMap<String, ValidationRuleSet>,
    ) -> LLMResult<serde_json::Map<String, Value>> {
        let mut validated = serde_json::Map::new();

        for (key, value) in params {
            if let Some(rules) = param_rules.get(key) {
                if let Some(str_val) = value.as_str() {
                    let sanitized = rules.validate(str_val).map_err(|e| {
                        LLMSpellError::Validation {
                            message: format!("Parameter '{}': {}", key, e),
                            field: Some(key.clone()),
                        }
                    })?;
                    validated.insert(key.clone(), Value::String(sanitized));
                } else {
                    validated.insert(key.clone(), value.clone());
                }
            } else {
                validated.insert(key.clone(), value.clone());
            }
        }

        Ok(validated)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_length_rule() {
        let rule = LengthRule::new(10);
        assert!(rule.validate("short").is_ok());
        assert!(rule.validate("this is too long").is_err());
    }

    #[test]
    fn test_html_sanitization_rule() {
        let rule = HtmlSanitizationRule::new();
        let result = rule.validate("<script>alert(1)</script>").unwrap();
        assert!(!result.contains("<script"));
    }

    #[test]
    fn test_sql_sanitization_rule() {
        let rule = SqlSanitizationRule::new();
        let result = rule.validate("'; DROP TABLE users; --").unwrap();
        assert!(!result.contains("DROP"));
    }

    #[test]
    fn test_url_validation_rule() {
        let rule = UrlValidationRule::new();
        assert!(rule.validate("https://example.com").is_ok());
        assert!(rule.validate("javascript:alert(1)").is_err());
        assert!(rule.validate("not a url").is_err());
    }

    #[test]
    fn test_alphanumeric_rule() {
        let rule = AlphanumericRule::with_allowed(".-_");
        assert!(rule.validate("test_123").is_ok());
        assert!(rule.validate("test-file.txt").is_ok());
        assert!(rule.validate("test@file").is_err());
    }

    #[test]
    fn test_validation_rule_set() {
        let rules = ValidationRuleSet::new()
            .add_rule(Box::new(LengthRule::new(100)))
            .add_rule(Box::new(NoNullBytesRule::new()))
            .add_rule(Box::new(HtmlSanitizationRule::new()));

        let input = "Hello <script>alert(1)</script>\0World";
        let result = rules.validate(input).unwrap();
        assert!(!result.contains("<script"));
        assert!(!result.contains('\0'));
    }

    #[test]
    fn test_web_standard_rules() {
        let rules = ValidationRuleSet::web_standard();
        let input = "<img src=x onerror='alert(1)'>";
        let result = rules.validate(input).unwrap();
        assert!(!result.contains("onerror"));
    }

    #[test]
    fn test_command_standard_rules() {
        let rules = ValidationRuleSet::command_standard();
        assert!(rules.validate("ls -la").is_ok());
        assert!(rules.validate("rm -rf / && echo pwned").is_err());
    }

    #[test]
    fn test_path_standard_rules() {
        let rules = ValidationRuleSet::path_standard();
        assert!(rules.validate("data/file.txt").is_ok());
        assert!(rules.validate("../../etc/passwd").is_err());
        assert!(rules.validate("/etc/passwd").is_err());
    }
}