// ABOUTME: Event pattern matching for subscription routing
// ABOUTME: Supports glob-style patterns for flexible event filtering

use serde::{Deserialize, Serialize};

/// Event pattern for matching event types
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EventPattern {
    pattern: String,
    is_wildcard: bool,
    prefix: Option<String>,
}

impl EventPattern {
    /// Create a new event pattern
    pub fn new(pattern: &str) -> Result<Self, String> {
        if pattern.is_empty() {
            return Err("Pattern cannot be empty".to_string());
        }

        let is_wildcard = pattern.contains('*');
        let prefix = pattern.strip_suffix('*').map(str::to_string);

        Ok(Self {
            pattern: pattern.to_string(),
            is_wildcard,
            prefix,
        })
    }

    /// Check if this pattern matches an event type
    pub fn matches(&self, event_type: &str) -> bool {
        if self.pattern == "*" {
            return true;
        }

        if let Some(prefix) = &self.prefix {
            event_type.starts_with(prefix)
        } else {
            event_type == self.pattern
        }
    }

    /// Get the pattern string
    pub fn as_str(&self) -> &str {
        &self.pattern
    }
}

/// Pattern matcher for efficient event routing
#[derive(Debug, Clone)]
pub struct PatternMatcher {
    // For future optimization - could add compiled patterns, etc.
}

impl PatternMatcher {
    pub fn new() -> Self {
        Self {}
    }

    /// Check if an event type matches a pattern
    pub fn matches(&self, event_type: &str, pattern: &str) -> bool {
        if pattern == "*" {
            return true;
        }

        if let Some(prefix) = pattern.strip_suffix('*') {
            event_type.starts_with(prefix)
        } else {
            event_type == pattern
        }
    }
}

impl Default for PatternMatcher {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_pattern_creation() {
        let pattern = EventPattern::new("test.*").unwrap();
        assert!(pattern.is_wildcard);
        assert_eq!(pattern.prefix, Some("test.".to_string()));

        let pattern = EventPattern::new("exact.match").unwrap();
        assert!(!pattern.is_wildcard);
        assert_eq!(pattern.prefix, None);
    }
    #[test]
    fn test_pattern_matching() {
        let pattern = EventPattern::new("system.*").unwrap();
        assert!(pattern.matches("system.startup"));
        assert!(pattern.matches("system.shutdown"));
        assert!(!pattern.matches("agent.created"));

        let pattern = EventPattern::new("exact").unwrap();
        assert!(pattern.matches("exact"));
        assert!(!pattern.matches("exact.match"));
    }
    #[test]
    fn test_pattern_matcher() {
        let matcher = PatternMatcher::new();

        assert!(matcher.matches("system.startup", "system.*"));
        assert!(matcher.matches("anything", "*"));
        assert!(!matcher.matches("system.startup", "agent.*"));
    }
}
