// ABOUTME: Enhanced expression analyzer with advanced DoS protection capabilities
// ABOUTME: Provides sophisticated pattern detection, memory tracking, and recursive depth analysis

//! Enhanced expression analyzer for comprehensive `DoS` protection
//!
//! This module provides advanced analysis capabilities beyond the basic
//! expression analyzer, including:
//! - Sophisticated pattern detection
//! - Memory usage estimation
//! - Recursive depth tracking
//! - Advanced attack vector detection

use super::{ExpressionComplexity, ExpressionComplexityConfig};
use std::collections::HashSet;

/// Enhanced configuration for expression analysis
#[derive(Debug, Clone)]
pub struct EnhancedExpressionConfig {
    /// Base configuration
    pub base: ExpressionComplexityConfig,
    /// Maximum memory allocation estimate (in bytes)
    pub max_memory_bytes: usize,
    /// Maximum recursive function depth
    pub max_recursive_depth: usize,
    /// Banned function patterns
    pub banned_patterns: HashSet<String>,
    /// Maximum number of unique variables
    pub max_variables: usize,
    /// Maximum exponent value allowed
    pub max_exponent: f64,
}

impl Default for EnhancedExpressionConfig {
    fn default() -> Self {
        Self {
            base: ExpressionComplexityConfig::default(),
            max_memory_bytes: 1_000_000, // 1MB
            max_recursive_depth: 3,      // Reduced from 5
            banned_patterns: Self::default_banned_patterns(),
            max_variables: 50,
            max_exponent: 100.0,
        }
    }
}

impl EnhancedExpressionConfig {
    /// Get default banned patterns
    fn default_banned_patterns() -> HashSet<String> {
        let mut patterns = HashSet::new();
        // Patterns that could cause exponential growth
        patterns.insert("exp(exp".to_string());
        patterns.insert("^(^".to_string()); // Nested power operations
        patterns.insert("pow(pow".to_string());
        patterns.insert("factorial".to_string()); // Not supported but dangerous if it were
                                                  // Patterns that could cause infinite loops
        patterns.insert("while".to_string());
        patterns.insert("for".to_string());
        patterns.insert("loop".to_string());
        // Code execution attempts
        patterns.insert("eval".to_string());
        patterns.insert("exec".to_string());
        patterns.insert("system".to_string());
        patterns
    }

    /// Create strict configuration
    #[must_use]
    pub fn strict() -> Self {
        Self {
            base: ExpressionComplexityConfig::strict(),
            max_memory_bytes: 100_000, // 100KB
            max_recursive_depth: 3,
            banned_patterns: Self::default_banned_patterns(),
            max_variables: 20,
            max_exponent: 50.0,
        }
    }
}

/// Enhanced expression analyzer
#[derive(Debug, Clone)]
pub struct EnhancedExpressionAnalyzer {
    config: EnhancedExpressionConfig,
}

impl EnhancedExpressionAnalyzer {
    /// Create new enhanced analyzer
    #[must_use]
    pub fn new() -> Self {
        Self {
            config: EnhancedExpressionConfig::default(),
        }
    }

    /// Create with custom configuration
    #[must_use]
    pub fn with_config(config: EnhancedExpressionConfig) -> Self {
        Self { config }
    }

    /// Analyze expression with enhanced checks
    #[must_use]
    pub fn analyze(&self, expression: &str) -> ExpressionComplexity {
        // First run basic analysis
        let basic_analyzer = super::ExpressionAnalyzer::with_config(self.config.base.clone());
        let mut result = basic_analyzer.analyze(expression);

        // If basic analysis failed, return immediately
        if !result.is_safe {
            return result;
        }

        // Check for banned patterns
        if let Some(pattern) = self.check_banned_patterns(expression) {
            result.is_safe = false;
            result.unsafe_reason = Some(format!("Banned pattern detected: {pattern}"));
            return result;
        }

        // Estimate memory usage
        let memory_estimate = Self::estimate_memory_usage(expression);
        if memory_estimate > self.config.max_memory_bytes {
            result.is_safe = false;
            result.unsafe_reason = Some(format!(
                "Estimated memory usage too high: {} > {} bytes",
                memory_estimate, self.config.max_memory_bytes
            ));
            return result;
        }

        // Check recursive depth
        let recursive_depth = Self::estimate_recursive_depth(expression);
        if recursive_depth > self.config.max_recursive_depth {
            result.is_safe = false;
            result.unsafe_reason = Some(format!(
                "Recursive depth too high: {} > {}",
                recursive_depth, self.config.max_recursive_depth
            ));
            return result;
        }

        // Check for exponential patterns
        if let Some(reason) = Self::check_exponential_patterns(expression, &self.config) {
            result.is_safe = false;
            result.unsafe_reason = Some(reason);
            return result;
        }

        // Check variable count
        let var_count = Self::count_unique_variables(expression);
        if var_count > self.config.max_variables {
            result.is_safe = false;
            result.unsafe_reason = Some(format!(
                "Too many unique variables: {} > {}",
                var_count, self.config.max_variables
            ));
            return result;
        }

        result
    }

    /// Check for banned patterns
    fn check_banned_patterns(&self, expression: &str) -> Option<String> {
        let expr_lower = expression.to_lowercase();
        for pattern in &self.config.banned_patterns {
            if expr_lower.contains(pattern) {
                return Some(pattern.clone());
            }
        }
        None
    }

    /// Estimate memory usage based on expression complexity
    fn estimate_memory_usage(expression: &str) -> usize {
        let base_size = expression.len() * 8; // Assume 8 bytes per character
        let function_count = Self::count_all_functions(expression);
        let operation_count = super::ExpressionAnalyzer::count_operations(expression);

        // Estimate: base + 1KB per function + 100 bytes per operation
        base_size + (function_count * 1024) + (operation_count * 100)
    }

    /// Estimate maximum recursive depth
    fn estimate_recursive_depth(expression: &str) -> usize {
        // Look for nested function calls of the same type
        let functions = ["pow", "exp", "log", "sqrt", "sin", "cos", "tan", "ln"];
        let mut max_depth = 0;

        for func in &functions {
            let pattern = format!("{func}(");
            let depth = Self::count_nested_occurrences(expression, &pattern);
            max_depth = max_depth.max(depth);
        }

        max_depth
    }

    /// Count nested occurrences of a pattern
    fn count_nested_occurrences(expression: &str, pattern: &str) -> usize {
        let mut depth: usize = 0;
        let mut max_depth: usize = 0;
        let mut chars = expression.chars().peekable();
        let pattern_chars: Vec<char> = pattern.chars().collect();

        while let Some(ch) = chars.next() {
            // Check if we're starting the pattern
            if ch == pattern_chars[0] {
                let mut matches = true;
                for (_i, &pch) in pattern_chars.iter().enumerate().skip(1) {
                    if chars.peek() != Some(&pch) {
                        matches = false;
                        break;
                    }
                    chars.next();
                }
                if matches {
                    depth += 1;
                    max_depth = max_depth.max(depth);
                }
            } else if ch == ')' {
                depth = depth.saturating_sub(1);
            }
        }

        max_depth
    }

    /// Check for exponential growth patterns
    fn check_exponential_patterns(
        expression: &str,
        config: &EnhancedExpressionConfig,
    ) -> Option<String> {
        // Check for large exponents
        if let Some(captures) = regex::Regex::new(r"\^[\s]*(\d+(?:\.\d+)?)")
            .unwrap()
            .captures(expression)
        {
            if let Some(exp_str) = captures.get(1) {
                if let Ok(exp_val) = exp_str.as_str().parse::<f64>() {
                    if exp_val > config.max_exponent {
                        return Some(format!(
                            "Exponent too large: {} > {}",
                            exp_val, config.max_exponent
                        ));
                    }
                }
            }
        }

        // Check for factorial of large numbers
        if let Some(captures) = regex::Regex::new(r"factorial\s*\(\s*(\d+)")
            .unwrap()
            .captures(expression)
        {
            if let Some(num_str) = captures.get(1) {
                if let Ok(num) = num_str.as_str().parse::<i32>() {
                    if num > 20 {
                        return Some(format!("Factorial of large number: {num}!"));
                    }
                }
            }
        }

        // Check for nested exponentials
        if expression.contains("exp(") && expression.contains("pow(") {
            let exp_count = expression.matches("exp(").count();
            let pow_count = expression.matches("pow(").count();
            if exp_count * pow_count > 4 {
                return Some("Too many nested exponential operations".to_string());
            }
        }

        None
    }

    /// Count unique variables
    fn count_unique_variables(expression: &str) -> usize {
        let var_regex = regex::Regex::new(r"\b[a-zA-Z_][a-zA-Z0-9_]*\b").unwrap();
        let mut variables = HashSet::new();

        for mat in var_regex.find_iter(expression) {
            let var = mat.as_str();
            // Skip function names
            if !Self::is_function_name(var) {
                variables.insert(var.to_string());
            }
        }

        variables.len()
    }

    /// Check if a word is a function name
    fn is_function_name(word: &str) -> bool {
        const FUNCTIONS: &[&str] = &[
            "sin", "cos", "tan", "asin", "acos", "atan", "sinh", "cosh", "tanh", "sqrt", "exp",
            "ln", "log", "abs", "sign", "int", "ceil", "floor", "round", "min", "max", "pi", "e",
            "pow",
        ];
        FUNCTIONS.contains(&word)
    }

    /// Count all functions (including custom ones)
    fn count_all_functions(expression: &str) -> usize {
        let func_regex = regex::Regex::new(r"\b[a-zA-Z_][a-zA-Z0-9_]*\s*\(").unwrap();
        func_regex.find_iter(expression).count()
    }
}

impl Default for EnhancedExpressionAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_banned_patterns() {
        let analyzer = EnhancedExpressionAnalyzer::new();

        let result = analyzer.analyze("pow(pow(2, 3), 4)");
        assert!(!result.is_safe);
        assert!(result.unsafe_reason.unwrap().contains("Banned pattern"));

        let result = analyzer.analyze("while(true) { x++ }");
        assert!(!result.is_safe);
    }
    #[test]
    fn test_memory_estimation() {
        let config = EnhancedExpressionConfig {
            max_memory_bytes: 1000, // Very low for testing
            ..Default::default()
        };
        let analyzer = EnhancedExpressionAnalyzer::with_config(config);

        let result = analyzer.analyze("sin(x) + cos(y) + tan(z) + exp(a) + log(b)");
        assert!(!result.is_safe);
        assert!(result.unsafe_reason.unwrap().contains("memory usage"));
    }
    #[test]
    fn test_recursive_depth() {
        let config = EnhancedExpressionConfig {
            max_recursive_depth: 2,
            ..Default::default()
        };
        let analyzer = EnhancedExpressionAnalyzer::with_config(config);

        // Test with a function that we actually check for
        let result = analyzer.analyze("sqrt(sqrt(sqrt(16)))");
        assert!(!result.is_safe);
        let reason = result.unsafe_reason.unwrap();
        assert!(reason.contains("Recursive depth") || reason.contains("recursive"));
    }
    #[test]
    fn test_exponential_patterns() {
        let analyzer = EnhancedExpressionAnalyzer::new();

        let result = analyzer.analyze("2^1000");
        assert!(!result.is_safe);
        assert!(result.unsafe_reason.unwrap().contains("Exponent too large"));

        // Test factorial pattern detection even though it's not a valid function
        let result = analyzer.analyze("factorial(50)");
        assert!(!result.is_safe);
        let reason = result.unsafe_reason.unwrap();
        // Check for factorial in the error message - case insensitive
        assert!(
            reason.to_lowercase().contains("factorial")
                || reason.contains("banned")
                || reason.contains("large number")
        );
    }
    #[test]
    fn test_variable_counting() {
        let config = EnhancedExpressionConfig {
            max_variables: 3,
            ..Default::default()
        };
        let analyzer = EnhancedExpressionAnalyzer::with_config(config);

        let result = analyzer.analyze("a + b + c");
        assert!(result.is_safe);

        let result = analyzer.analyze("a + b + c + d + e");
        assert!(!result.is_safe);
        assert!(result
            .unsafe_reason
            .unwrap()
            .contains("Too many unique variables"));
    }
}
