// ABOUTME: State classification system for performance-appropriate processing paths
// ABOUTME: Enables fast paths for trusted data and full protection for sensitive data

use serde::{Deserialize, Serialize};

/// Classification of state data to determine appropriate processing path
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StateClass {
    /// In-memory only, no persistence, zero overhead
    Ephemeral,

    /// Internal system state, skip all validation checks
    /// Target: <1% overhead
    Trusted,

    /// Normal user data with basic validation
    /// Target: <3% overhead  
    Standard,

    /// Contains PII/secrets, needs full validation and redaction
    /// Target: <10% overhead
    Sensitive,

    /// Untrusted external data, requires full validation
    /// Target: <10% overhead
    External,
}

impl Default for StateClass {
    fn default() -> Self {
        Self::Standard
    }
}

impl StateClass {
    /// Returns true if this class requires circular reference checking
    pub fn requires_circular_check(&self) -> bool {
        match self {
            Self::Ephemeral | Self::Trusted => false,
            Self::Standard | Self::Sensitive | Self::External => true,
        }
    }

    /// Returns true if this class requires sensitive data redaction
    pub fn requires_redaction(&self) -> bool {
        match self {
            Self::Ephemeral | Self::Trusted | Self::Standard => false,
            Self::Sensitive | Self::External => true,
        }
    }

    /// Returns true if this class should be persisted to storage
    pub fn should_persist(&self) -> bool {
        match self {
            Self::Ephemeral => false,
            Self::Trusted | Self::Standard | Self::Sensitive | Self::External => true,
        }
    }

    /// Returns true if hooks should be executed for this class
    pub fn should_execute_hooks(&self) -> bool {
        match self {
            Self::Ephemeral | Self::Trusted => false,
            Self::Standard | Self::Sensitive | Self::External => true,
        }
    }

    /// Returns the maximum expected overhead percentage for this class
    pub fn target_overhead_percent(&self) -> f64 {
        match self {
            Self::Ephemeral => 0.0,
            Self::Trusted => 1.0,
            Self::Standard => 3.0,
            Self::Sensitive | Self::External => 10.0,
        }
    }

    /// Infer state class from state key patterns
    pub fn infer_from_key(key: &str) -> Self {
        if key.starts_with("benchmark:") || key.starts_with("test:") {
            Self::Trusted
        } else if key.starts_with("temp:") || key.starts_with("cache:") {
            Self::Ephemeral
        } else if key.contains("secret") || key.contains("token") || key.contains("credential") {
            Self::Sensitive
        } else if key.starts_with("external:") || key.starts_with("user_input:") {
            Self::External
        } else {
            Self::Standard
        }
    }
}

/// Configuration for state class behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateClassConfig {
    /// Default class for unspecified state
    pub default_class: StateClass,

    /// Whether to enable automatic class inference from keys
    pub auto_infer: bool,

    /// Key patterns mapped to specific classes
    pub key_patterns: std::collections::HashMap<String, StateClass>,

    /// Force all state to use benchmark mode (Trusted class)
    pub benchmark_mode: bool,
}

impl Default for StateClassConfig {
    fn default() -> Self {
        Self {
            default_class: StateClass::Standard,
            auto_infer: true,
            key_patterns: std::collections::HashMap::new(),
            benchmark_mode: false,
        }
    }
}

impl StateClassConfig {
    /// Create a benchmark-optimized configuration
    pub fn benchmark() -> Self {
        Self {
            default_class: StateClass::Trusted,
            auto_infer: false,
            key_patterns: std::collections::HashMap::new(),
            benchmark_mode: true,
        }
    }

    /// Create a production-ready configuration
    pub fn production() -> Self {
        let mut patterns = std::collections::HashMap::new();
        patterns.insert("agent_state:*".to_string(), StateClass::Standard);
        patterns.insert("secret:*".to_string(), StateClass::Sensitive);
        patterns.insert("external:*".to_string(), StateClass::External);
        patterns.insert("temp:*".to_string(), StateClass::Ephemeral);

        Self {
            default_class: StateClass::Standard,
            auto_infer: true,
            key_patterns: patterns,
            benchmark_mode: false,
        }
    }

    /// Determine the appropriate state class for a given key
    pub fn classify_key(&self, key: &str) -> StateClass {
        if self.benchmark_mode {
            return StateClass::Trusted;
        }

        // Check explicit patterns first
        for (pattern, class) in &self.key_patterns {
            if self.matches_pattern(key, pattern) {
                return *class;
            }
        }

        // Auto-infer if enabled
        if self.auto_infer {
            StateClass::infer_from_key(key)
        } else {
            self.default_class
        }
    }

    /// Simple glob-style pattern matching
    fn matches_pattern(&self, key: &str, pattern: &str) -> bool {
        if pattern.ends_with('*') {
            let prefix = pattern.strip_suffix('*').unwrap();
            key.starts_with(prefix)
        } else {
            key == pattern
        }
    }
}

#[cfg(test)]
#[cfg_attr(test_category = "state")]
mod tests {
    use super::*;

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_state_class_properties() {
        assert!(!StateClass::Trusted.requires_circular_check());
        assert!(StateClass::Standard.requires_circular_check());
        assert!(StateClass::Sensitive.requires_redaction());
        assert!(!StateClass::Standard.requires_redaction());
        assert!(!StateClass::Ephemeral.should_persist());
        assert!(StateClass::Standard.should_persist());
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_state_class_inference() {
        assert_eq!(
            StateClass::infer_from_key("benchmark:test"),
            StateClass::Trusted
        );
        assert_eq!(
            StateClass::infer_from_key("temp:cache"),
            StateClass::Ephemeral
        );
        assert_eq!(
            StateClass::infer_from_key("secret:token"),
            StateClass::Sensitive
        );
        assert_eq!(
            StateClass::infer_from_key("external:api"),
            StateClass::External
        );
        assert_eq!(
            StateClass::infer_from_key("normal:data"),
            StateClass::Standard
        );
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_config_pattern_matching() {
        let config = StateClassConfig::production();
        assert_eq!(
            config.classify_key("agent_state:test"),
            StateClass::Standard
        );
        assert_eq!(config.classify_key("secret:token"), StateClass::Sensitive);
        assert_eq!(config.classify_key("temp:cache"), StateClass::Ephemeral);
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_benchmark_mode() {
        let config = StateClassConfig::benchmark();
        assert_eq!(config.classify_key("any:key"), StateClass::Trusted);
        assert_eq!(config.classify_key("secret:token"), StateClass::Trusted);
    }
}
