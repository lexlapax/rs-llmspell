//! ABOUTME: Comprehensive input sanitization framework for preventing injection attacks
//! ABOUTME: Provides HTML, SQL, command, format string, and XXE protection

#![allow(clippy::non_std_lazy_statics)]

use html_escape::encode_safe;
use lazy_static::lazy_static;
use regex::Regex;

/// Input sanitization configuration
#[derive(Debug, Clone)]
#[allow(clippy::struct_excessive_bools)]
pub struct SanitizationConfig {
    /// Enable HTML/script sanitization
    pub sanitize_html: bool,
    /// Enable SQL injection protection
    pub sanitize_sql: bool,
    /// Enable command injection protection
    pub sanitize_command: bool,
    /// Enable format string protection
    pub sanitize_format: bool,
    /// Enable XXE prevention
    pub prevent_xxe: bool,
    /// Maximum input length
    pub max_input_length: Option<usize>,
    /// Custom blocked patterns
    pub blocked_patterns: Vec<String>,
}

impl Default for SanitizationConfig {
    fn default() -> Self {
        Self {
            sanitize_html: true,
            sanitize_sql: true,
            sanitize_command: true,
            sanitize_format: true,
            prevent_xxe: true,
            max_input_length: Some(1_000_000), // 1MB default
            blocked_patterns: Vec::new(),
        }
    }
}

impl SanitizationConfig {
    /// Create strict configuration for untrusted input
    #[must_use]
    pub fn strict() -> Self {
        Self {
            sanitize_html: true,
            sanitize_sql: true,
            sanitize_command: true,
            sanitize_format: true,
            prevent_xxe: true,
            max_input_length: Some(10_000), // 10KB for strict mode
            blocked_patterns: vec![
                r"<script".to_string(),
                r"javascript:".to_string(),
                r"on\w+\s*=".to_string(),
                r"eval\s*\(".to_string(),
            ],
        }
    }

    /// Create relaxed configuration for semi-trusted input
    #[must_use]
    pub fn relaxed() -> Self {
        Self {
            sanitize_html: true,
            sanitize_sql: true,
            sanitize_command: true,
            sanitize_format: false,
            prevent_xxe: true,
            max_input_length: Some(10_000_000), // 10MB
            blocked_patterns: Vec::new(),
        }
    }
}

/// Input sanitizer for preventing injection attacks
pub struct InputSanitizer {
    config: SanitizationConfig,
}

impl InputSanitizer {
    /// Create a new sanitizer with default configuration
    #[must_use]
    pub fn new() -> Self {
        Self {
            config: SanitizationConfig::default(),
        }
    }

    /// Create sanitizer with custom configuration
    #[must_use]
    pub fn with_config(config: SanitizationConfig) -> Self {
        Self { config }
    }

    /// Sanitize input based on configuration
    ///
    /// # Errors
    ///
    /// Returns `SanitizationError` if:
    /// - Input exceeds maximum length
    /// - Input contains blocked patterns
    pub fn sanitize(&self, input: &str) -> Result<String, SanitizationError> {
        // Check input length first
        if let Some(max_len) = self.config.max_input_length {
            if input.len() > max_len {
                return Err(SanitizationError::InputTooLong {
                    actual: input.len(),
                    max: max_len,
                });
            }
        }

        // Check blocked patterns
        for pattern in &self.config.blocked_patterns {
            if let Ok(re) = Regex::new(pattern) {
                if re.is_match(input) {
                    return Err(SanitizationError::BlockedPattern {
                        pattern: pattern.clone(),
                    });
                }
            }
        }

        let mut sanitized = input.to_string();

        // Apply sanitizations based on config
        if self.config.sanitize_html {
            sanitized = self.sanitize_html(&sanitized);
        }

        if self.config.sanitize_sql {
            sanitized = self.sanitize_sql(&sanitized);
        }

        if self.config.sanitize_command {
            sanitized = self.sanitize_command(&sanitized);
        }

        if self.config.sanitize_format {
            sanitized = self.sanitize_format_string(&sanitized);
        }

        Ok(sanitized)
    }

    /// Sanitize HTML/JavaScript to prevent XSS
    #[must_use]
    pub fn sanitize_html(&self, input: &str) -> String {
        lazy_static! {
            static ref SCRIPT_TAG_RE: Regex = Regex::new(r"(?i)<script[^>]*>.*?</script>").unwrap();
            static ref EVENT_HANDLER_RE: Regex =
                Regex::new(r#"(?i)\bon\w+\s*=\s*['\"].*?['\"]"#).unwrap();
            static ref JAVASCRIPT_PROTOCOL_RE: Regex = Regex::new(r"(?i)javascript:").unwrap();
            static ref DATA_PROTOCOL_RE: Regex = Regex::new(r"(?i)data:.*?base64").unwrap();
        }

        let mut result = input.to_string();

        // Remove script tags
        result = SCRIPT_TAG_RE.replace_all(&result, "").to_string();

        // Remove event handlers
        result = EVENT_HANDLER_RE.replace_all(&result, "").to_string();

        // Remove javascript: protocol
        result = JAVASCRIPT_PROTOCOL_RE.replace_all(&result, "").to_string();

        // Remove data: protocol with base64 (potential XSS vector)
        result = DATA_PROTOCOL_RE.replace_all(&result, "").to_string();

        // HTML encode remaining content
        encode_safe(&result).to_string()
    }

    /// Sanitize SQL input to prevent SQL injection
    #[must_use]
    pub fn sanitize_sql(&self, input: &str) -> String {
        lazy_static! {
            static ref SQL_COMMENT_RE: Regex = Regex::new(r"(-{2,}|/\*|\*/)").unwrap();
            static ref SQL_KEYWORDS_RE: Regex = Regex::new(
                r"(?i)\b(union|select|insert|update|delete|drop|create|alter|exec|execute|script|javascript)\b"
            ).unwrap();
        }

        let mut result = input.to_string();

        // Remove SQL comments
        result = SQL_COMMENT_RE.replace_all(&result, "").to_string();

        // Escape single quotes
        result = result.replace('\'', "''");

        // Remove dangerous SQL keywords in suspicious contexts
        if result.contains(';') || result.contains("--") {
            result = SQL_KEYWORDS_RE.replace_all(&result, "").to_string();
        }

        result
    }

    /// Sanitize command input to prevent command injection
    #[must_use]
    pub fn sanitize_command(&self, input: &str) -> String {
        lazy_static! {
            static ref SHELL_METACHARACTERS: Regex = Regex::new(r"[;&|`$<>\\]").unwrap();
            static ref COMMAND_SUBSTITUTION_RE: Regex = Regex::new(r"\$\(.*?\)").unwrap();
            static ref BACKTICK_RE: Regex = Regex::new(r"`.*?`").unwrap();
        }

        let mut result = input.to_string();

        // Remove command substitution
        result = COMMAND_SUBSTITUTION_RE.replace_all(&result, "").to_string();
        result = BACKTICK_RE.replace_all(&result, "").to_string();

        // Escape shell metacharacters
        result = SHELL_METACHARACTERS
            .replace_all(&result, "\\$0")
            .to_string();

        // Remove null bytes
        result = result.replace('\0', "");

        result
    }

    /// Sanitize format strings to prevent format string attacks
    #[must_use]
    pub fn sanitize_format_string(&self, input: &str) -> String {
        lazy_static! {
            static ref FORMAT_SPECIFIER_RE: Regex =
                Regex::new(r"%[0-9]*[diouxXeEfFgGaAcspn%]").unwrap();
            static ref DANGEROUS_FORMAT_RE: Regex = Regex::new(r"%[ns]").unwrap();
        }

        let mut result = input.to_string();

        // Remove dangerous format specifiers (%n writes to memory, %s can read memory)
        result = DANGEROUS_FORMAT_RE.replace_all(&result, "").to_string();

        // Escape remaining % characters that aren't part of valid format specifiers
        let mut final_result = String::new();
        let mut chars = result.chars().peekable();

        while let Some(ch) = chars.next() {
            if ch == '%' {
                // Check if this is a valid format specifier
                let remaining: String = chars.clone().collect();
                if FORMAT_SPECIFIER_RE.is_match(&format!("%{}", &remaining)) {
                    final_result.push(ch);
                } else {
                    final_result.push_str("%%");
                }
            } else {
                final_result.push(ch);
            }
        }

        final_result
    }

    /// Sanitize XML to prevent XXE attacks
    #[must_use]
    pub fn sanitize_xml(&self, input: &str) -> String {
        lazy_static! {
            static ref DOCTYPE_RE: Regex = Regex::new(r"(?i)<!DOCTYPE[^>]*>").unwrap();
            static ref ENTITY_RE: Regex = Regex::new(r"(?i)<!ENTITY[^>]*>").unwrap();
            static ref SYSTEM_RE: Regex = Regex::new(r"(?i)SYSTEM").unwrap();
            static ref PUBLIC_RE: Regex = Regex::new(r"(?i)PUBLIC").unwrap();
        }

        let mut result = input.to_string();

        // Remove DOCTYPE declarations
        result = DOCTYPE_RE.replace_all(&result, "").to_string();

        // Remove ENTITY declarations
        result = ENTITY_RE.replace_all(&result, "").to_string();

        // Remove SYSTEM keyword
        result = SYSTEM_RE.replace_all(&result, "").to_string();

        // Remove PUBLIC keyword
        result = PUBLIC_RE.replace_all(&result, "").to_string();

        result
    }

    /// Validate and sanitize file paths
    ///
    /// # Errors
    ///
    /// Returns `SanitizationError` if:
    /// - Path contains traversal attempts (.. or ~)
    /// - Path is absolute (starts with / or \ or contains drive letter)
    pub fn sanitize_path(&self, input: &str) -> Result<String, SanitizationError> {
        // Check for path traversal attempts
        if input.contains("..") || input.contains('~') {
            return Err(SanitizationError::PathTraversal);
        }

        // Remove null bytes
        let sanitized = input.replace('\0', "");

        // Check for absolute paths (security risk in many contexts)
        if sanitized.starts_with('/') || sanitized.starts_with('\\') {
            return Err(SanitizationError::AbsolutePath);
        }

        // Windows drive letters
        if sanitized.len() >= 2 && sanitized.chars().nth(1) == Some(':') {
            return Err(SanitizationError::AbsolutePath);
        }

        Ok(sanitized)
    }

    /// Create a validation report for input
    #[must_use]
    pub fn validate(&self, input: &str) -> ValidationReport {
        let mut issues = Vec::new();

        // Check length
        if let Some(max_len) = self.config.max_input_length {
            if input.len() > max_len {
                issues.push(ValidationIssue {
                    issue_type: IssueType::Length,
                    severity: Severity::Error,
                    message: format!("Input too long: {} > {}", input.len(), max_len),
                });
            }
        }

        // Check for various injection patterns
        if Self::contains_html_injection(input) {
            issues.push(ValidationIssue {
                issue_type: IssueType::HtmlInjection,
                severity: Severity::High,
                message: "Potential HTML/JavaScript injection detected".to_string(),
            });
        }

        if Self::contains_sql_injection(input) {
            issues.push(ValidationIssue {
                issue_type: IssueType::SqlInjection,
                severity: Severity::High,
                message: "Potential SQL injection detected".to_string(),
            });
        }

        if Self::contains_command_injection(input) {
            issues.push(ValidationIssue {
                issue_type: IssueType::CommandInjection,
                severity: Severity::Critical,
                message: "Potential command injection detected".to_string(),
            });
        }

        ValidationReport {
            is_safe: issues.is_empty(),
            issues,
            sanitized: self.sanitize(input).ok(),
        }
    }

    fn contains_html_injection(input: &str) -> bool {
        input.contains('<') && input.contains('>')
            || input.contains("javascript:")
            || input.contains("onerror")
            || input.contains("onload")
    }

    fn contains_sql_injection(input: &str) -> bool {
        let lower = input.to_lowercase();
        (lower.contains("union") && lower.contains("select"))
            || lower.contains("'; drop")
            || lower.contains("' or '")
            || lower.contains("' or 1=1")
    }

    fn contains_command_injection(input: &str) -> bool {
        input.contains(';')
            && (input.contains("rm") || input.contains("cat") || input.contains("ls"))
            || input.contains("$(")
            || input.contains('`')
            || input.contains("&&")
            || input.contains("||")
    }
}

impl Default for InputSanitizer {
    fn default() -> Self {
        Self::new()
    }
}

/// Sanitization error types
#[derive(Debug, Clone, PartialEq)]
pub enum SanitizationError {
    /// Input exceeds maximum length
    InputTooLong {
        /// Actual length of the input
        actual: usize,
        /// Maximum allowed length
        max: usize,
    },
    /// Input contains blocked pattern
    BlockedPattern {
        /// The pattern that was matched
        pattern: String,
    },
    /// Path traversal attempt detected
    PathTraversal,
    /// Absolute path not allowed
    AbsolutePath,
}

impl std::fmt::Display for SanitizationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InputTooLong { actual, max } => {
                write!(f, "Input too long: {actual} bytes (max: {max})")
            }
            Self::BlockedPattern { pattern } => write!(f, "Blocked pattern detected: {pattern}"),
            Self::PathTraversal => write!(f, "Path traversal attempt detected"),
            Self::AbsolutePath => write!(f, "Absolute path not allowed"),
        }
    }
}

impl std::error::Error for SanitizationError {}

/// Validation report for input analysis
#[derive(Debug, Clone)]
pub struct ValidationReport {
    /// Whether the input is considered safe
    pub is_safe: bool,
    /// List of validation issues found
    pub issues: Vec<ValidationIssue>,
    /// Sanitized version of input (if sanitization succeeded)
    pub sanitized: Option<String>,
}

/// Individual validation issue
#[derive(Debug, Clone)]
pub struct ValidationIssue {
    /// Type of issue
    pub issue_type: IssueType,
    /// Severity level
    pub severity: Severity,
    /// Human-readable message
    pub message: String,
}

/// Types of validation issues
#[derive(Debug, Clone, PartialEq)]
pub enum IssueType {
    /// Input length violation
    Length,
    /// HTML/JavaScript injection
    HtmlInjection,
    /// SQL injection
    SqlInjection,
    /// Command injection
    CommandInjection,
    /// Format string attack
    FormatString,
    /// XXE attack
    XxeAttack,
    /// Path traversal
    PathTraversal,
}

/// Severity levels for validation issues
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum Severity {
    /// Informational
    Info,
    /// Warning
    Warning,
    /// Error
    Error,
    /// High severity
    High,
    /// Critical severity
    Critical,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_html_sanitization() {
        let sanitizer = InputSanitizer::new();

        // Test script tag removal
        let script_input = "Hello <script>alert('xss')</script> World";
        let script_sanitized = sanitizer.sanitize_html(script_input);
        assert!(!script_sanitized.contains("<script"));
        assert!(!script_sanitized.contains("alert"));

        // Test event handler removal
        let event_input = "<img src=x onerror='alert(1)'>";
        let event_sanitized = sanitizer.sanitize_html(event_input);
        assert!(!event_sanitized.contains("onerror"));

        // Test javascript: protocol removal
        let js_input = "<a href='javascript:alert(1)'>Click</a>";
        let js_sanitized = sanitizer.sanitize_html(js_input);
        assert!(!js_sanitized.contains("javascript:"));
    }
    #[test]
    fn test_sql_sanitization() {
        let sanitizer = InputSanitizer::new();

        // Test SQL comment removal
        let sql_drop_input = "'; DROP TABLE users; --";
        let sql_drop_sanitized = sanitizer.sanitize_sql(sql_drop_input);
        assert!(!sql_drop_sanitized.contains("--"));

        // Test quote escaping
        let quote_input = "O'Brien";
        let quote_sanitized = sanitizer.sanitize_sql(quote_input);
        assert_eq!(quote_sanitized, "O''Brien");

        // Test union select removal
        let union_input = "' UNION SELECT * FROM passwords; --";
        let union_sanitized = sanitizer.sanitize_sql(union_input);
        assert!(!union_sanitized.contains("UNION"));
        assert!(!union_sanitized.contains("SELECT"));
    }
    #[test]
    fn test_command_sanitization() {
        let sanitizer = InputSanitizer::new();

        // Test command substitution removal
        let cmd_sub_input = "echo $(cat /etc/passwd)";
        let cmd_sub_sanitized = sanitizer.sanitize_command(cmd_sub_input);
        assert!(!cmd_sub_sanitized.contains("$("));

        // Test backtick removal
        let backtick_input = "echo `whoami`";
        let backtick_sanitized = sanitizer.sanitize_command(backtick_input);
        assert!(!backtick_sanitized.contains('`'));

        // Test metacharacter escaping
        let meta_input = "file.txt; rm -rf /";
        let meta_sanitized = sanitizer.sanitize_command(meta_input);
        assert!(meta_sanitized.contains("\\;"));
    }
    #[test]
    fn test_format_string_sanitization() {
        let sanitizer = InputSanitizer::new();

        // Test dangerous format specifier removal
        let danger_fmt_input = "Hello %n%s World";
        let danger_fmt_sanitized = sanitizer.sanitize_format_string(danger_fmt_input);
        assert!(!danger_fmt_sanitized.contains("%n"));
        assert!(!danger_fmt_sanitized.contains("%s"));

        // Test safe format specifiers preserved
        let safe_fmt_input = "Value: %d, Float: %.2f";
        let safe_fmt_sanitized = sanitizer.sanitize_format_string(safe_fmt_input);
        assert!(safe_fmt_sanitized.contains("%d"));
        assert!(safe_fmt_sanitized.contains("%.2f"));
    }
    #[test]
    fn test_xml_sanitization() {
        let sanitizer = InputSanitizer::new();

        // Test DOCTYPE removal
        let doctype_input = "<!DOCTYPE foo SYSTEM 'file:///etc/passwd'><root/>";
        let doctype_sanitized = sanitizer.sanitize_xml(doctype_input);
        assert!(!doctype_sanitized.contains("DOCTYPE"));
        assert!(!doctype_sanitized.contains("SYSTEM"));

        // Test ENTITY removal
        let entity_input = "<!ENTITY xxe SYSTEM 'http://evil.com'><root>&xxe;</root>";
        let entity_sanitized = sanitizer.sanitize_xml(entity_input);
        assert!(!entity_sanitized.contains("ENTITY"));
    }
    #[test]
    fn test_path_sanitization() {
        let sanitizer = InputSanitizer::new();

        // Test path traversal detection
        assert!(sanitizer.sanitize_path("../../etc/passwd").is_err());
        assert!(sanitizer.sanitize_path("~/secret").is_err());

        // Test absolute path detection
        assert!(sanitizer.sanitize_path("/etc/passwd").is_err());
        assert!(sanitizer.sanitize_path("C:\\Windows\\System32").is_err());

        // Test valid paths
        assert!(sanitizer.sanitize_path("data/file.txt").is_ok());
        assert!(sanitizer.sanitize_path("subdir/file.txt").is_ok());
    }
    #[test]
    fn test_validation_report() {
        let sanitizer = InputSanitizer::new();

        // Test multiple injection detection
        let input = "<script>alert(1)</script>'; DROP TABLE users; --";
        let report = sanitizer.validate(input);
        assert!(!report.is_safe);
        assert!(report.issues.len() >= 2);
        assert!(report
            .issues
            .iter()
            .any(|i| i.issue_type == IssueType::HtmlInjection));
        assert!(report
            .issues
            .iter()
            .any(|i| i.issue_type == IssueType::SqlInjection));
    }
    #[test]
    fn test_config_modes() {
        // Test strict mode
        let strict = InputSanitizer::with_config(SanitizationConfig::strict());
        let long_input = "a".repeat(20_000);
        assert!(strict.sanitize(&long_input).is_err());

        // Test relaxed mode
        let relaxed = InputSanitizer::with_config(SanitizationConfig::relaxed());
        assert!(relaxed.sanitize(&long_input).is_ok());
    }
    #[test]
    fn test_blocked_patterns() {
        let config = SanitizationConfig {
            blocked_patterns: vec![r"eval\s*\(".to_string(), r"Function\s*\(".to_string()],
            ..Default::default()
        };
        let sanitizer = InputSanitizer::with_config(config);

        assert!(sanitizer.sanitize("eval(code)").is_err());
        assert!(sanitizer.sanitize("new Function('alert(1)')").is_err());
        assert!(sanitizer.sanitize("normal text").is_ok());
    }
}
