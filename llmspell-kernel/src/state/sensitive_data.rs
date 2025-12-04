// ABOUTME: Sensitive data protection for agent state serialization
// ABOUTME: Redacts API keys and credentials during state persistence

use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::LazyLock;

/// Patterns for detecting sensitive data
static API_KEY_PATTERNS: LazyLock<Vec<Regex>> = LazyLock::new(|| {
    vec![
        // OpenAI API keys
        Regex::new(r"sk-[a-zA-Z0-9]{48}").unwrap(),
        // Anthropic API keys
        Regex::new(r"sk-ant-[a-zA-Z0-9-]{95}").unwrap(),
        // Generic API key patterns
        Regex::new(r#"(?i)(api[_-]?key|apikey|api[_-]?secret|access[_-]?token|auth[_-]?token|bearer)\s*[:=]\s*['\"]?([a-zA-Z0-9_\-\.]{20,})['\"]?"#).unwrap(),
        // JWT tokens
        Regex::new(r"eyJ[a-zA-Z0-9_-]+\.eyJ[a-zA-Z0-9_-]+\.[a-zA-Z0-9_-]+").unwrap(),
        // AWS credentials
        Regex::new(r"AKIA[0-9A-Z]{16}").unwrap(),
        // GitHub tokens
        Regex::new(r"ghp_[a-zA-Z0-9]{36}").unwrap(),
        Regex::new(r"gho_[a-zA-Z0-9]{36}").unwrap(),
        // Generic secrets
        Regex::new(r#"(?i)(password|passwd|pwd|secret|private[_-]?key)\s*[:=]\s*['\"]?([^\s'\"]{8,})['\"]?"#).unwrap(),
    ]
});

/// Sensitive field names to redact
static SENSITIVE_FIELD_NAMES: LazyLock<Vec<&'static str>> = LazyLock::new(|| {
    vec![
        "api_key",
        "apikey",
        "api-key",
        "secret",
        "secret_key",
        "secret-key",
        "password",
        "passwd",
        "pwd",
        "token",
        "access_token",
        "auth_token",
        "bearer_token",
        "private_key",
        "private-key",
        "privatekey",
        "credential",
        "credentials",
        "authorization",
        "auth",
    ]
});

/// Configuration for sensitive data protection
#[derive(Debug, Clone)]
pub struct SensitiveDataConfig {
    /// Whether to redact sensitive data
    pub redact_enabled: bool,
    /// Redaction placeholder
    pub redaction_text: String,
    /// Additional patterns to detect
    pub custom_patterns: Vec<Regex>,
    /// Additional field names to redact
    pub custom_field_names: Vec<String>,
    /// Whether to hash redacted values for tracking
    pub hash_redacted: bool,
}

impl Default for SensitiveDataConfig {
    fn default() -> Self {
        Self {
            redact_enabled: true,
            redaction_text: "[REDACTED]".to_string(),
            custom_patterns: Vec::new(),
            custom_field_names: Vec::new(),
            hash_redacted: true,
        }
    }
}

impl SensitiveDataConfig {
    /// Create a disabled configuration (no redaction)
    pub fn disabled() -> Self {
        Self {
            redact_enabled: false,
            redaction_text: "[REDACTED]".to_string(),
            custom_patterns: Vec::new(),
            custom_field_names: Vec::new(),
            hash_redacted: false,
        }
    }

    /// Check if this field name should be redacted
    pub fn is_sensitive_field(&self, field_name: &str) -> bool {
        if !self.redact_enabled {
            return false;
        }

        let lower = field_name.to_lowercase();
        SENSITIVE_FIELD_NAMES
            .iter()
            .any(|&name| lower.contains(name))
            || self
                .custom_field_names
                .iter()
                .any(|name| lower.contains(&name.to_lowercase()))
    }

    /// Check if this value contains sensitive patterns
    pub fn contains_sensitive_pattern(&self, value: &str) -> bool {
        if !self.redact_enabled {
            return false;
        }

        API_KEY_PATTERNS
            .iter()
            .any(|pattern| pattern.is_match(value))
            || self
                .custom_patterns
                .iter()
                .any(|pattern| pattern.is_match(value))
    }
}

/// Sensitive data protector
pub struct SensitiveDataProtector {
    config: SensitiveDataConfig,
    redaction_map: HashMap<String, String>,
}

impl SensitiveDataProtector {
    pub fn new(config: SensitiveDataConfig) -> Self {
        Self {
            config,
            redaction_map: HashMap::new(),
        }
    }

    pub fn with_default() -> Self {
        Self::new(SensitiveDataConfig::default())
    }

    /// Redact sensitive data from a JSON value
    pub fn redact_value(&mut self, value: &mut Value) {
        if !self.config.redact_enabled {
            return;
        }

        self.redact_value_recursive(value);
    }

    fn redact_value_recursive(&mut self, value: &mut Value) {
        match value {
            Value::Object(map) => {
                let keys: Vec<String> = map.keys().cloned().collect();

                for key in keys {
                    // Check if field name is sensitive
                    if self.is_sensitive_field(&key) {
                        if let Some(val) = map.get_mut(&key) {
                            self.redact_field_value(val, &key);
                        }
                    } else {
                        // Recursively check nested values
                        if let Some(val) = map.get_mut(&key) {
                            self.redact_value_recursive(val);
                        }
                    }
                }
            }
            Value::Array(arr) => {
                for val in arr.iter_mut() {
                    self.redact_value_recursive(val);
                }
            }
            Value::String(s) => {
                // Check string content for sensitive patterns
                if self.contains_sensitive_data(s) {
                    self.redact_string(s);
                }
            }
            _ => {
                // Other types don't need redaction
            }
        }
    }

    fn is_sensitive_field(&self, field_name: &str) -> bool {
        let lower = field_name.to_lowercase();

        // Check built-in sensitive field names
        for &sensitive in SENSITIVE_FIELD_NAMES.iter() {
            if lower.contains(sensitive) {
                return true;
            }
        }

        // Check custom field names
        for custom in &self.config.custom_field_names {
            if lower.contains(&custom.to_lowercase()) {
                return true;
            }
        }

        false
    }

    fn contains_sensitive_data(&self, text: &str) -> bool {
        // Check built-in patterns
        for pattern in API_KEY_PATTERNS.iter() {
            if pattern.is_match(text) {
                return true;
            }
        }

        // Check custom patterns
        for pattern in &self.config.custom_patterns {
            if pattern.is_match(text) {
                return true;
            }
        }

        false
    }

    fn redact_field_value(&mut self, value: &mut Value, _field_name: &str) {
        match value {
            Value::String(s) => {
                self.redact_string(s);
            }
            Value::Object(_) | Value::Array(_) => {
                // For complex types, replace entire value
                let hash = if self.config.hash_redacted {
                    format!("{}_{}", self.config.redaction_text, Self::hash_value(value))
                } else {
                    self.config.redaction_text.clone()
                };

                *value = Value::String(hash);
            }
            _ => {
                // For other types, convert to string and redact
                let original = value.to_string();
                if self.config.hash_redacted {
                    let hash = Self::hash_string(&original);
                    self.redaction_map.insert(hash.clone(), original);
                    *value = Value::String(format!("{}_{}", self.config.redaction_text, hash));
                } else {
                    *value = Value::String(self.config.redaction_text.clone());
                }
            }
        }
    }

    fn redact_string(&mut self, s: &mut String) {
        if self.config.hash_redacted {
            let hash = Self::hash_string(s);
            self.redaction_map.insert(hash.clone(), s.clone());
            *s = format!("{}_{}", self.config.redaction_text, hash);
        } else {
            s.clone_from(&self.config.redaction_text);
        }
    }

    fn hash_string(s: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        s.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }

    fn hash_value(value: &Value) -> String {
        Self::hash_string(&value.to_string())
    }

    /// Get the redaction map for recovery purposes
    pub fn redaction_map(&self) -> &HashMap<String, String> {
        &self.redaction_map
    }

    /// Restore redacted values (for internal use only)
    pub fn restore_value(&self, value: &mut Value) {
        if !self.config.hash_redacted || self.redaction_map.is_empty() {
            return;
        }

        self.restore_value_recursive(value);
    }

    fn restore_value_recursive(&self, value: &mut Value) {
        match value {
            Value::String(s) => {
                if s.starts_with(&self.config.redaction_text) {
                    // Extract hash
                    if let Some(hash) = s.strip_prefix(&format!("{}_", self.config.redaction_text))
                    {
                        if let Some(original) = self.redaction_map.get(hash) {
                            *s = original.clone();
                        }
                    }
                }
            }
            Value::Object(map) => {
                for val in map.values_mut() {
                    self.restore_value_recursive(val);
                }
            }
            Value::Array(arr) => {
                for val in arr.iter_mut() {
                    self.restore_value_recursive(val);
                }
            }
            _ => {}
        }
    }
}

/// Trait for types that can have sensitive data redacted
pub trait RedactSensitiveData {
    /// Redact sensitive data from this value
    ///
    /// # Errors
    ///
    /// Returns an error string if:
    /// - Serialization to JSON fails
    /// - Deserialization after redaction fails
    fn redact_sensitive_data(&mut self, config: &SensitiveDataConfig) -> Result<(), String>;
}

impl<T: Serialize + for<'de> Deserialize<'de>> RedactSensitiveData for T {
    fn redact_sensitive_data(&mut self, config: &SensitiveDataConfig) -> Result<(), String> {
        // Serialize to JSON
        let mut value = serde_json::to_value(&*self)
            .map_err(|e| format!("Failed to serialize for redaction: {e}"))?;

        // Redact sensitive data
        let mut protector = SensitiveDataProtector::new(config.clone());
        protector.redact_value(&mut value);

        // Deserialize back
        *self = serde_json::from_value(value)
            .map_err(|e| format!("Failed to deserialize after redaction: {e}"))?;

        Ok(())
    }
}

/// Safe serialization with sensitive data redaction
///
/// # Errors
///
/// Returns an error string if:
/// - Serialization to JSON fails
/// - `MessagePack` serialization fails
pub fn safe_serialize_with_redaction<T: Serialize + Clone>(
    value: &T,
    config: &SensitiveDataConfig,
) -> Result<Vec<u8>, String> {
    // Serialize to JSON value first
    let mut json_value =
        serde_json::to_value(value).map_err(|e| format!("Serialization failed: {e}"))?;

    // Redact sensitive data
    let mut protector = SensitiveDataProtector::new(config.clone());
    protector.redact_value(&mut json_value);

    // Serialize to bytes
    serde_json::to_vec(&json_value).map_err(|e| format!("Final serialization failed: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn test_sensitive_field_detection() {
        let protector = SensitiveDataProtector::with_default();

        assert!(protector.is_sensitive_field("api_key"));
        assert!(protector.is_sensitive_field("API_KEY"));
        assert!(protector.is_sensitive_field("apiKey"));
        assert!(protector.is_sensitive_field("password"));
        assert!(protector.is_sensitive_field("user_password"));
        assert!(!protector.is_sensitive_field("username"));
    }
    #[test]
    fn test_api_key_pattern_detection() {
        let protector = SensitiveDataProtector::with_default();

        // OpenAI key pattern
        assert!(protector
            .contains_sensitive_data("sk-abc123def456ghi789jkl012mno345pqr678stu901vwx234"));

        // Generic API key
        assert!(protector.contains_sensitive_data("api_key: 'my-super-secret-api-key-12345'"));

        // JWT token
        assert!(protector.contains_sensitive_data("eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0.dozjgNryP4J3jVmNHl0w5N_XgL0n3I9PlFUP0THsR8U"));

        // Not sensitive
        assert!(!protector.contains_sensitive_data("just a normal string"));
    }
    #[test]
    fn test_value_redaction() {
        let mut value = json!({
            "name": "test-agent",
            "api_key": "sk-abc123def456ghi789jkl012mno345pqr678stu901vwx234",
            "config": {
                "password": "super-secret-password",
                "url": "https://api.example.com"
            },
            "tokens": ["normal-token", "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0.dozjgNryP4J3jVmNHl0w5N_XgL0n3I9PlFUP0THsR8U"]
        });

        let mut protector = SensitiveDataProtector::with_default();
        protector.redact_value(&mut value);

        // Check that sensitive fields were redacted
        assert!(value["api_key"].as_str().unwrap().starts_with("[REDACTED]"));
        assert!(value["config"]["password"]
            .as_str()
            .unwrap()
            .starts_with("[REDACTED]"));

        // Check that non-sensitive fields were not changed
        assert_eq!(value["name"], "test-agent");
        assert_eq!(value["config"]["url"], "https://api.example.com");

        // Check array redaction - entire "tokens" field is redacted because field name contains "token"
        assert!(value["tokens"].as_str().unwrap().starts_with("[REDACTED]"));
    }
    #[test]
    fn test_restore_redacted_values() {
        let original = json!({
            "api_key": "sk-test-key-12345",
            "name": "agent"
        });

        let mut value = original.clone();
        let mut protector = SensitiveDataProtector::with_default();

        // Redact
        protector.redact_value(&mut value);
        assert!(value["api_key"].as_str().unwrap().starts_with("[REDACTED]"));

        // Restore
        protector.restore_value(&mut value);
        assert_eq!(value["api_key"], "sk-test-key-12345");
    }
}
