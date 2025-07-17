// ABOUTME: Security utilities for preventing DoS attacks and enforcing resource limits
// ABOUTME: Provides expression complexity analysis, timeout enforcement, and resource protection

//! Security utilities for tool protection
//!
//! This module provides security utilities to protect tools from various attacks:
//! - Expression complexity analysis for calculator DoS prevention
//! - Timeout enforcement for long-running operations
//! - Resource limit enforcement
//! - Input size validation
//! - Path security validation

pub mod credential_protection;
pub mod expression_analyzer_enhanced;
pub mod file_upload_security;
pub mod input_sanitizer;
pub mod memory_tracker;
pub mod path;
pub mod ssrf_protection;

pub use credential_protection::{
    CredentialAuditEntry, CredentialAuditor, CredentialFilter, CredentialType, ErrorSanitizer,
    SecureCredential, SecureString,
};
pub use expression_analyzer_enhanced::{EnhancedExpressionAnalyzer, EnhancedExpressionConfig};
pub use file_upload_security::{
    FileProcessingSandbox, FileUploadConfig, FileUploadValidator, FileValidationError,
    FileValidationResult,
};
pub use input_sanitizer::{
    InputSanitizer, SanitizationConfig, SanitizationError, ValidationReport,
};
pub use memory_tracker::{MemoryGuard, MemoryTracker, ScopedMemoryTracker};
pub use ssrf_protection::{SsrfError, SsrfProtectionConfig, SsrfProtector, ValidatedUrl};

use std::time::Duration;

/// Configuration for expression complexity analysis
#[derive(Debug, Clone)]
pub struct ExpressionComplexityConfig {
    /// Maximum expression length in characters
    pub max_length: usize,
    /// Maximum nesting depth for parentheses and functions
    pub max_depth: usize,
    /// Maximum number of operations allowed
    pub max_operations: usize,
    /// Maximum number of function calls allowed
    pub max_functions: usize,
    /// Maximum evaluation time allowed
    pub max_evaluation_time: Duration,
}

impl Default for ExpressionComplexityConfig {
    fn default() -> Self {
        Self {
            max_length: 1000,
            max_depth: 20,
            max_operations: 100,
            max_functions: 50,
            max_evaluation_time: Duration::from_millis(100),
        }
    }
}

impl ExpressionComplexityConfig {
    /// Create a new configuration with default values
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a strict configuration for untrusted input
    #[must_use]
    pub fn strict() -> Self {
        Self {
            max_length: 500,
            max_depth: 10,
            max_operations: 50,
            max_functions: 20,
            max_evaluation_time: Duration::from_millis(50),
        }
    }

    /// Create a relaxed configuration for trusted input
    #[must_use]
    pub fn relaxed() -> Self {
        Self {
            max_length: 5000,
            max_depth: 50,
            max_operations: 500,
            max_functions: 200,
            max_evaluation_time: Duration::from_millis(500),
        }
    }
}

/// Result of expression complexity analysis
#[derive(Debug, Clone)]
pub struct ExpressionComplexity {
    /// Length of the expression
    pub length: usize,
    /// Maximum nesting depth
    pub max_depth: usize,
    /// Number of operations
    pub operation_count: usize,
    /// Number of function calls
    pub function_count: usize,
    /// Whether the expression is safe to evaluate
    pub is_safe: bool,
    /// Reason if expression is unsafe
    pub unsafe_reason: Option<String>,
}

/// Analyzes expression complexity to prevent `DoS` attacks
#[derive(Debug, Clone)]
pub struct ExpressionAnalyzer {
    config: ExpressionComplexityConfig,
}

impl ExpressionAnalyzer {
    /// Create a new analyzer with default configuration
    #[must_use]
    pub fn new() -> Self {
        Self {
            config: ExpressionComplexityConfig::default(),
        }
    }

    /// Create analyzer with custom configuration
    #[must_use]
    pub fn with_config(config: ExpressionComplexityConfig) -> Self {
        Self { config }
    }

    /// Analyze expression complexity
    #[must_use]
    pub fn analyze(&self, expression: &str) -> ExpressionComplexity {
        let length = expression.len();

        // Check length first
        if length > self.config.max_length {
            return ExpressionComplexity {
                length,
                max_depth: 0,
                operation_count: 0,
                function_count: 0,
                is_safe: false,
                unsafe_reason: Some(format!(
                    "Expression too long: {} > {} characters",
                    length, self.config.max_length
                )),
            };
        }

        // Calculate nesting depth
        let max_depth = Self::calculate_max_depth(expression);
        if max_depth > self.config.max_depth {
            return ExpressionComplexity {
                length,
                max_depth,
                operation_count: 0,
                function_count: 0,
                is_safe: false,
                unsafe_reason: Some(format!(
                    "Nesting too deep: {} > {} levels",
                    max_depth, self.config.max_depth
                )),
            };
        }

        // Count operations
        let operation_count = Self::count_operations(expression);
        if operation_count > self.config.max_operations {
            return ExpressionComplexity {
                length,
                max_depth,
                operation_count,
                function_count: 0,
                is_safe: false,
                unsafe_reason: Some(format!(
                    "Too many operations: {} > {}",
                    operation_count, self.config.max_operations
                )),
            };
        }

        // Count function calls
        let function_count = Self::count_functions(expression);
        if function_count > self.config.max_functions {
            return ExpressionComplexity {
                length,
                max_depth,
                operation_count,
                function_count,
                is_safe: false,
                unsafe_reason: Some(format!(
                    "Too many function calls: {} > {}",
                    function_count, self.config.max_functions
                )),
            };
        }

        // Check for potentially dangerous patterns
        if let Some(reason) = Self::check_dangerous_patterns(expression) {
            return ExpressionComplexity {
                length,
                max_depth,
                operation_count,
                function_count,
                is_safe: false,
                unsafe_reason: Some(reason),
            };
        }

        ExpressionComplexity {
            length,
            max_depth,
            operation_count,
            function_count,
            is_safe: true,
            unsafe_reason: None,
        }
    }

    /// Calculate maximum nesting depth
    fn calculate_max_depth(expression: &str) -> usize {
        let mut current_depth: usize = 0;
        let mut max_depth: usize = 0;

        for ch in expression.chars() {
            match ch {
                '(' | '[' | '{' => {
                    current_depth += 1;
                    max_depth = max_depth.max(current_depth);
                }
                ')' | ']' | '}' => {
                    current_depth = current_depth.saturating_sub(1);
                }
                _ => {}
            }
        }

        max_depth
    }

    /// Count operations in expression
    fn count_operations(expression: &str) -> usize {
        let operators = ['+', '-', '*', '/', '%', '^', '&', '|', '!', '<', '>', '='];
        let mut count = 0;
        let mut prev_was_operator = false;

        for ch in expression.chars() {
            if operators.contains(&ch) {
                if !prev_was_operator {
                    count += 1;
                }
                prev_was_operator = true;
            } else if !ch.is_whitespace() {
                prev_was_operator = false;
            }
        }

        count
    }

    /// Count function calls in expression
    fn count_functions(expression: &str) -> usize {
        // Common mathematical functions
        let functions = [
            "sin", "cos", "tan", "asin", "acos", "atan", "sinh", "cosh", "tanh", "asinh", "acosh",
            "atanh", "sqrt", "exp", "ln", "log", "abs", "sign", "int", "ceil", "floor", "round",
            "min", "max", "pi", "e",
        ];

        let mut count = 0;
        for func in &functions {
            count += expression.matches(func).count();
        }

        count
    }

    /// Check for potentially dangerous patterns
    fn check_dangerous_patterns(expression: &str) -> Option<String> {
        // Check for excessive consecutive operations
        if expression.contains("+++")
            || expression.contains("---")
            || expression.contains("***")
            || expression.contains("///")
        {
            return Some("Excessive consecutive operations detected".to_string());
        }

        // Check for potential stack overflow patterns
        let repeated_opens = expression.matches("((((").count();
        if repeated_opens > 0 {
            return Some("Potential stack overflow pattern detected".to_string());
        }

        // Check for extremely large numbers that could cause issues
        if let Some(captures) = regex::Regex::new(r"\d{10,}").unwrap().captures(expression) {
            if let Some(num_str) = captures.get(0) {
                return Some(format!(
                    "Extremely large number detected: {}",
                    num_str.as_str()
                ));
            }
        }

        None
    }

    /// Get the configured maximum evaluation time
    #[must_use]
    pub fn max_evaluation_time(&self) -> Duration {
        self.config.max_evaluation_time
    }
}

impl Default for ExpressionAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expression_length_check() {
        let analyzer = ExpressionAnalyzer::with_config(ExpressionComplexityConfig {
            max_length: 10,
            ..Default::default()
        });

        let result = analyzer.analyze("1 + 2");
        assert!(result.is_safe);

        let result = analyzer.analyze("1 + 2 + 3 + 4 + 5");
        assert!(!result.is_safe);
        assert!(result.unsafe_reason.unwrap().contains("too long"));
    }

    #[test]
    fn test_nesting_depth_check() {
        let analyzer = ExpressionAnalyzer::with_config(ExpressionComplexityConfig {
            max_depth: 3,
            ..Default::default()
        });

        let result = analyzer.analyze("((1 + 2) * 3)");
        assert!(result.is_safe);
        assert_eq!(result.max_depth, 2);

        let result = analyzer.analyze("((((1 + 2))))");
        assert!(!result.is_safe);
        assert!(result.unsafe_reason.unwrap().contains("too deep"));
    }

    #[test]
    fn test_operation_count() {
        let analyzer = ExpressionAnalyzer::with_config(ExpressionComplexityConfig {
            max_operations: 3,
            ..Default::default()
        });

        let result = analyzer.analyze("1 + 2 * 3");
        assert!(result.is_safe);
        assert_eq!(result.operation_count, 2);

        let result = analyzer.analyze("1 + 2 * 3 / 4 - 5");
        assert!(!result.is_safe);
        assert!(result.unsafe_reason.unwrap().contains("operations"));
    }

    #[test]
    fn test_function_count() {
        let analyzer = ExpressionAnalyzer::with_config(ExpressionComplexityConfig {
            max_functions: 2,
            ..Default::default()
        });

        let result = analyzer.analyze("sin(x) + cos(y)");
        assert!(result.is_safe);
        assert_eq!(result.function_count, 2);

        let result = analyzer.analyze("sin(cos(tan(x)))");
        assert!(!result.is_safe);
        assert!(result.unsafe_reason.unwrap().contains("function"));
    }

    #[test]
    fn test_dangerous_patterns() {
        let analyzer = ExpressionAnalyzer::new();

        let result = analyzer.analyze("1 +++ 2");
        assert!(!result.is_safe);
        assert!(result.unsafe_reason.unwrap().contains("consecutive"));

        let result = analyzer.analyze("((((((x");
        assert!(!result.is_safe);
        assert!(result.unsafe_reason.unwrap().contains("stack overflow"));

        let result = analyzer.analyze("12345678901234567890");
        assert!(!result.is_safe);
        assert!(result.unsafe_reason.unwrap().contains("large number"));
    }

    #[test]
    fn test_safe_expressions() {
        let analyzer = ExpressionAnalyzer::new();

        let safe_expressions = [
            "2 + 3 * 4",
            "sin(pi()/2)",
            "sqrt(x^2 + y^2)",
            "(a + b) * (c - d)",
            "log(10, 100)",
        ];

        for expr in &safe_expressions {
            let result = analyzer.analyze(expr);
            assert!(result.is_safe, "Expression '{}' should be safe", expr);
        }
    }

    #[test]
    fn test_config_presets() {
        let strict = ExpressionComplexityConfig::strict();
        assert_eq!(strict.max_length, 500);
        assert_eq!(strict.max_depth, 10);

        let relaxed = ExpressionComplexityConfig::relaxed();
        assert_eq!(relaxed.max_length, 5000);
        assert_eq!(relaxed.max_depth, 50);
    }
}
