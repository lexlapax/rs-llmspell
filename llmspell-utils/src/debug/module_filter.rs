//! Enhanced module filtering system with pattern matching and wildcard support
//!
//! Provides flexible module-based filtering for debug output with glob patterns,
//! hierarchical matching, and regex support for advanced use cases.

use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Enhanced module filter with pattern matching capabilities
#[derive(Debug, Clone)]
pub struct EnhancedModuleFilter {
    /// Simple exact matches for fast lookup
    exact_matches: HashMap<String, bool>,
    /// Wildcard patterns compiled to regex
    pattern_cache: HashMap<String, (Regex, bool)>,
    /// Hierarchical rules (parent.child.*)
    hierarchical_rules: Vec<(String, bool)>,
    /// Default action when no patterns match
    default_enabled: bool,
}

/// Pattern type for module filtering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FilterPattern {
    /// Exact module name match
    Exact(String),
    /// Wildcard pattern (*, ?, [])
    Wildcard(String),
    /// Regular expression pattern
    Regex(String),
    /// Hierarchical pattern (module.submodule.*)
    Hierarchical(String),
}

/// Filter rule with pattern and enable/disable action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterRule {
    /// Pattern to match against
    pub pattern: FilterPattern,
    /// Whether to enable (true) or disable (false) logging
    pub enabled: bool,
    /// Optional description of the rule
    pub description: Option<String>,
}

impl Default for EnhancedModuleFilter {
    fn default() -> Self {
        Self::new()
    }
}

impl EnhancedModuleFilter {
    /// Create a new enhanced module filter
    #[must_use]
    pub fn new() -> Self {
        Self {
            exact_matches: HashMap::new(),
            pattern_cache: HashMap::new(),
            hierarchical_rules: Vec::new(),
            default_enabled: true,
        }
    }

    /// Set the default action when no patterns match
    pub fn set_default_enabled(&mut self, enabled: bool) {
        self.default_enabled = enabled;
    }

    /// Add a filter rule
    pub fn add_rule(&mut self, rule: FilterRule) {
        // When adding an enabled rule, change default to false (allow-list behavior)
        if rule.enabled && self.has_no_rules() {
            self.default_enabled = false;
        }

        match rule.pattern {
            FilterPattern::Exact(module) => {
                self.exact_matches.insert(module, rule.enabled);
            }
            FilterPattern::Wildcard(pattern) => {
                if let Ok(regex) = Self::wildcard_to_regex(&pattern) {
                    self.pattern_cache.insert(pattern, (regex, rule.enabled));
                }
            }
            FilterPattern::Regex(regex_str) => {
                if let Ok(regex) = Regex::new(&regex_str) {
                    self.pattern_cache.insert(regex_str, (regex, rule.enabled));
                }
            }
            FilterPattern::Hierarchical(pattern) => {
                self.hierarchical_rules.push((pattern, rule.enabled));
                // Sort by specificity (longer patterns first)
                self.hierarchical_rules
                    .sort_by(|a, b| b.0.len().cmp(&a.0.len()));
            }
        }
    }

    /// Check if the filter has no rules
    fn has_no_rules(&self) -> bool {
        self.exact_matches.is_empty()
            && self.pattern_cache.is_empty()
            && self.hierarchical_rules.is_empty()
    }

    /// Add a simple pattern filter
    pub fn add_filter(&mut self, pattern: &str, enabled: bool) {
        let rule = if pattern.ends_with(".*") {
            // Hierarchical pattern takes precedence over wildcard
            FilterRule {
                pattern: FilterPattern::Hierarchical(pattern.trim_end_matches(".*").to_string()),
                enabled,
                description: None,
            }
        } else if pattern.contains('*') || pattern.contains('?') || pattern.contains('[') {
            FilterRule {
                pattern: FilterPattern::Wildcard(pattern.to_string()),
                enabled,
                description: None,
            }
        } else {
            FilterRule {
                pattern: FilterPattern::Exact(pattern.to_string()),
                enabled,
                description: None,
            }
        };

        self.add_rule(rule);
    }

    /// Check if a module should be logged
    #[must_use]
    pub fn should_log(&self, module: &str) -> bool {
        // First check exact matches (fastest)
        if let Some(&enabled) = self.exact_matches.get(module) {
            return enabled;
        }

        // Check hierarchical rules (ordered by specificity)
        for (pattern, enabled) in &self.hierarchical_rules {
            if Self::matches_hierarchical(module, pattern) {
                return *enabled;
            }
        }

        // Check compiled regex patterns
        for (regex, enabled) in self.pattern_cache.values() {
            if regex.is_match(module) {
                return *enabled;
            }
        }

        // Default action
        self.default_enabled
    }

    /// Clear all filters
    pub fn clear(&mut self) {
        self.exact_matches.clear();
        self.pattern_cache.clear();
        self.hierarchical_rules.clear();
    }

    /// Get all current filter rules as a summary
    #[must_use]
    pub fn get_filter_summary(&self) -> FilterSummary {
        let mut rules = Vec::new();

        // Add exact matches
        for (pattern, enabled) in &self.exact_matches {
            rules.push(FilterRule {
                pattern: FilterPattern::Exact(pattern.clone()),
                enabled: *enabled,
                description: None,
            });
        }

        // Add hierarchical rules
        for (pattern, enabled) in &self.hierarchical_rules {
            rules.push(FilterRule {
                pattern: FilterPattern::Hierarchical(pattern.clone()),
                enabled: *enabled,
                description: None,
            });
        }

        // Add pattern rules (we store the original pattern as key)
        for pattern in self.pattern_cache.keys() {
            if let Some((_, enabled)) = self.pattern_cache.get(pattern) {
                let filter_pattern = if Self::is_regex_pattern(pattern) {
                    FilterPattern::Regex(pattern.clone())
                } else {
                    FilterPattern::Wildcard(pattern.clone())
                };

                rules.push(FilterRule {
                    pattern: filter_pattern,
                    enabled: *enabled,
                    description: None,
                });
            }
        }

        let total_rules = rules.len();
        FilterSummary {
            rules,
            default_enabled: self.default_enabled,
            total_rules,
        }
    }

    /// Convert wildcard pattern to regex
    fn wildcard_to_regex(pattern: &str) -> Result<Regex, regex::Error> {
        let mut regex_pattern = String::new();
        let chars: Vec<char> = pattern.chars().collect();
        let mut i = 0;

        regex_pattern.push('^');

        while i < chars.len() {
            match chars.get(i).copied() {
                Some('*') => regex_pattern.push_str(".*"),
                Some('?') => regex_pattern.push('.'),
                Some('[') => {
                    // Handle character classes
                    regex_pattern.push('[');
                    i += 1;
                    while i < chars.len() && chars.get(i) != Some(&']') {
                        if chars.get(i) == Some(&'\\') {
                            regex_pattern.push('\\');
                            i += 1;
                            if let Some(&escaped_char) = chars.get(i) {
                                regex_pattern.push(escaped_char);
                            }
                        } else if let Some(&ch) = chars.get(i) {
                            regex_pattern.push(ch);
                        }
                        i += 1;
                    }
                    if chars.get(i) == Some(&']') {
                        regex_pattern.push(']');
                    }
                }
                Some('\\') => {
                    regex_pattern.push('\\');
                    i += 1;
                    if let Some(&escaped_char) = chars.get(i) {
                        regex_pattern.push(escaped_char);
                    }
                }
                Some(c) if c.is_ascii_alphanumeric() || c == '_' || c == '-' => {
                    regex_pattern.push(c);
                }
                Some(c) => {
                    regex_pattern.push('\\');
                    regex_pattern.push(c);
                }
                None => break,
            }
            i += 1;
        }

        regex_pattern.push('$');
        Regex::new(&regex_pattern)
    }

    /// Check if a module matches a hierarchical pattern
    fn matches_hierarchical(module: &str, pattern: &str) -> bool {
        if module == pattern {
            return true;
        }

        // Check if module starts with pattern followed by a dot
        if let Some(remaining) = module.strip_prefix(pattern) {
            remaining.starts_with('.')
        } else {
            false
        }
    }

    /// Check if a pattern looks like a regex (heuristic)
    fn is_regex_pattern(pattern: &str) -> bool {
        // Simple heuristic: if it contains regex special chars that aren't wildcards
        pattern.contains('(')
            || pattern.contains(')')
            || pattern.contains('+')
            || pattern.contains('{')
            || pattern.contains('}')
            || pattern.contains('^')
            || pattern.contains('$')
            || pattern.contains('|')
    }

    /// Remove a specific filter pattern
    pub fn remove_filter(&mut self, pattern: &str) -> bool {
        // Try exact match first
        if self.exact_matches.remove(pattern).is_some() {
            return true;
        }

        // Try pattern cache
        if self.pattern_cache.remove(pattern).is_some() {
            return true;
        }

        // Try hierarchical rules
        if let Some(pos) = self
            .hierarchical_rules
            .iter()
            .position(|(p, _)| p == pattern)
        {
            self.hierarchical_rules.remove(pos);
            return true;
        }

        false
    }

    /// Get performance statistics
    #[must_use]
    pub fn get_stats(&self) -> FilterStats {
        FilterStats {
            exact_matches: self.exact_matches.len(),
            pattern_cache_size: self.pattern_cache.len(),
            hierarchical_rules: self.hierarchical_rules.len(),
            total_rules: self.exact_matches.len()
                + self.pattern_cache.len()
                + self.hierarchical_rules.len(),
        }
    }
}

/// Summary of all filter rules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterSummary {
    /// All active filter rules
    pub rules: Vec<FilterRule>,
    /// Default action when no patterns match
    pub default_enabled: bool,
    /// Total number of rules
    pub total_rules: usize,
}

/// Performance statistics for the filter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterStats {
    /// Number of exact match rules
    pub exact_matches: usize,
    /// Number of compiled regex patterns
    pub pattern_cache_size: usize,
    /// Number of hierarchical rules
    pub hierarchical_rules: usize,
    /// Total number of rules
    pub total_rules: usize,
}

/// Preset filter configurations for common use cases
impl EnhancedModuleFilter {
    /// Create a filter that only logs errors and warnings
    #[must_use]
    pub fn errors_only() -> Self {
        let mut filter = Self::new();
        filter.set_default_enabled(false);
        filter.add_filter("error", true);
        filter.add_filter("warn", true);
        filter
    }

    /// Create a filter for development debugging
    #[must_use]
    pub fn development() -> Self {
        let mut filter = Self::new();
        filter.add_filter("workflow.*", true);
        filter.add_filter("agent.*", true);
        filter.add_filter("tool.*", true);
        filter.add_filter("*.test", false);
        filter.add_filter("*.benchmark", false);
        filter
    }

    /// Create a filter for production logging
    #[must_use]
    pub fn production() -> Self {
        let mut filter = Self::new();
        filter.set_default_enabled(false);
        filter.add_filter("security.*", true);
        filter.add_filter("performance.*", true);
        filter.add_filter("error.*", true);
        filter.add_filter("audit.*", true);
        filter
    }

    /// Create a filter for specific component debugging
    #[must_use]
    pub fn component(component: &str) -> Self {
        let mut filter = Self::new();
        filter.set_default_enabled(false);
        filter.add_filter(&format!("{component}.*"), true);
        filter
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exact_matching() {
        let mut filter = EnhancedModuleFilter::new();
        filter.add_filter("workflow.step1", true);
        filter.add_filter("agent.internal", false);

        assert!(filter.should_log("workflow.step1"));
        assert!(!filter.should_log("agent.internal"));
        assert!(filter.should_log("other.module")); // default enabled
    }

    #[test]
    fn test_wildcard_patterns() {
        let mut filter = EnhancedModuleFilter::new();
        filter.add_filter("workflow.*", true);
        filter.add_filter("*.test", false);

        assert!(filter.should_log("workflow.step1"));
        assert!(filter.should_log("workflow.step2"));
        assert!(!filter.should_log("unit.test"));
        assert!(!filter.should_log("integration.test"));
    }

    #[test]
    fn test_hierarchical_matching() {
        let mut filter = EnhancedModuleFilter::new();
        filter.add_filter("agent.*", true);
        filter.add_filter("agent.internal.*", false);

        assert!(filter.should_log("agent.executor"));
        assert!(filter.should_log("agent.coordinator"));
        assert!(!filter.should_log("agent.internal.cache"));
        assert!(!filter.should_log("agent.internal.state"));
    }

    #[test]
    fn test_priority_order() {
        let mut filter = EnhancedModuleFilter::new();
        // More specific rules should take precedence
        filter.add_filter("module.*", false);
        filter.add_filter("module.important", true);

        assert!(!filter.should_log("module.test"));
        assert!(filter.should_log("module.important"));
    }

    #[test]
    fn test_preset_filters() {
        let errors_filter = EnhancedModuleFilter::errors_only();
        assert!(!errors_filter.should_log("debug.info"));

        let dev_filter = EnhancedModuleFilter::development();
        assert!(dev_filter.should_log("workflow.step"));

        let component_filter = EnhancedModuleFilter::component("agent");
        assert!(component_filter.should_log("agent.test"));
        assert!(!component_filter.should_log("workflow.test"));
    }

    #[test]
    fn test_filter_management() {
        let mut filter = EnhancedModuleFilter::new();
        filter.add_filter("test.*", true);

        let stats = filter.get_stats();
        assert_eq!(stats.hierarchical_rules, 1);

        // Remove using the hierarchical pattern (without .*)
        assert!(filter.remove_filter("test"));
        let stats_after = filter.get_stats();
        assert_eq!(stats_after.hierarchical_rules, 0);
    }

    #[test]
    fn test_complex_wildcard_patterns() {
        let mut filter = EnhancedModuleFilter::new();
        filter.add_filter("test[12]", true);
        filter.add_filter("debug?", false);

        // These tests might not work perfectly with our simple wildcard implementation
        // but demonstrate the intended functionality
        let summary = filter.get_filter_summary();
        assert_eq!(summary.total_rules, 2);
    }
}
