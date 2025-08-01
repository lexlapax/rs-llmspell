//! ABOUTME: Information disclosure prevention framework for protecting sensitive data
//! ABOUTME: Provides error sanitization, stack trace removal, and debug info filtering

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::Arc;

/// Information disclosure prevention configuration
#[derive(Debug, Clone)]
#[allow(clippy::struct_excessive_bools)]
pub struct InfoDisclosureConfig {
    /// Whether to include stack traces in errors
    pub include_stack_traces: bool,
    /// Whether to sanitize file paths
    pub sanitize_paths: bool,
    /// Whether to mask sensitive data
    pub mask_sensitive_data: bool,
    /// Whether to filter debug information
    pub filter_debug_info: bool,
    /// Maximum error message length
    pub max_error_length: usize,
    /// Sensitive patterns to mask
    pub sensitive_patterns: Vec<regex::Regex>,
    /// Allowed error details in production
    pub allowed_error_details: HashSet<String>,
}

impl Default for InfoDisclosureConfig {
    fn default() -> Self {
        Self {
            include_stack_traces: false,
            sanitize_paths: true,
            mask_sensitive_data: true,
            filter_debug_info: true,
            max_error_length: 500,
            sensitive_patterns: Self::default_sensitive_patterns(),
            allowed_error_details: Self::default_allowed_details(),
        }
    }
}

impl InfoDisclosureConfig {
    /// Create development configuration (more verbose)
    #[must_use]
    pub fn development() -> Self {
        Self {
            include_stack_traces: true,
            sanitize_paths: false,
            mask_sensitive_data: true,
            filter_debug_info: false,
            max_error_length: 2000,
            sensitive_patterns: Self::default_sensitive_patterns(),
            allowed_error_details: HashSet::new(), // Allow all in dev
        }
    }

    /// Create production configuration (most secure)
    #[must_use]
    pub fn production() -> Self {
        Self::default()
    }

    /// Default sensitive patterns to mask
    fn default_sensitive_patterns() -> Vec<regex::Regex> {
        vec![
            // API keys and tokens - capture the whole pattern including key=value
            regex::Regex::new(r"(?i)(api[_-]?key|token|secret|password)[\s:=]+[\w\-\.]+").unwrap(),
            // Email addresses
            regex::Regex::new(r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b").unwrap(),
            // IP addresses
            regex::Regex::new(r"\b(?:[0-9]{1,3}\.){3}[0-9]{1,3}\b").unwrap(),
            // File paths with username
            regex::Regex::new(r"/(?:home|Users)/[^/\s]+").unwrap(),
            // URLs with credentials
            regex::Regex::new(r"(?i)https?://[^:]+:[^@]+@").unwrap(),
            // Credit card numbers
            regex::Regex::new(r"\b\d{4}[\s\-]?\d{4}[\s\-]?\d{4}[\s\-]?\d{4}\b").unwrap(),
            // Social Security Numbers
            regex::Regex::new(r"\b\d{3}-\d{2}-\d{4}\b").unwrap(),
            // Database connection strings
            regex::Regex::new(r"(?i)(postgres|mysql|mongodb)://[^\s]+").unwrap(),
        ]
    }

    /// Default allowed error details
    fn default_allowed_details() -> HashSet<String> {
        let mut allowed = HashSet::new();
        allowed.insert("validation".to_string());
        allowed.insert("permission".to_string());
        allowed.insert("not_found".to_string());
        allowed.insert("timeout".to_string());
        allowed.insert("rate_limit".to_string());
        allowed
    }
}

/// Error information that might be disclosed
#[derive(Debug, Clone)]
pub struct ErrorInfo {
    /// Error message
    pub message: String,
    /// Error kind/category
    pub kind: Option<String>,
    /// Stack trace if available
    pub stack_trace: Option<String>,
    /// Additional context
    pub context: HashMap<String, String>,
    /// Source file and line
    pub source_location: Option<(String, u32)>,
}

/// Sanitized error information safe for disclosure
#[derive(Debug, Clone, serde::Serialize)]
pub struct SanitizedError {
    /// Sanitized error message
    pub message: String,
    /// Error category (if allowed)
    pub category: Option<String>,
    /// Error code for client reference
    pub error_code: String,
    /// Whether the error is retriable
    pub retriable: bool,
}

/// Information disclosure preventer
pub struct InfoDisclosurePreventer {
    config: InfoDisclosureConfig,
    /// Cache of sanitized paths
    path_cache: parking_lot::Mutex<HashMap<PathBuf, String>>,
}

impl InfoDisclosurePreventer {
    /// Create a new information disclosure preventer
    #[must_use]
    pub fn new(config: InfoDisclosureConfig) -> Self {
        Self {
            config,
            path_cache: parking_lot::Mutex::new(HashMap::new()),
        }
    }

    /// Create with default production configuration
    #[must_use]
    pub fn production() -> Self {
        Self::new(InfoDisclosureConfig::production())
    }

    /// Create with development configuration
    #[must_use]
    pub fn development() -> Self {
        Self::new(InfoDisclosureConfig::development())
    }

    /// Sanitize an error for safe disclosure
    pub fn sanitize_error(&self, error_info: &ErrorInfo) -> SanitizedError {
        let mut message = error_info.message.clone();

        // Apply sanitization steps
        if self.config.mask_sensitive_data {
            message = self.mask_sensitive_data(&message);
        }

        if self.config.sanitize_paths {
            message = self.sanitize_paths(&message);
        }

        if self.config.filter_debug_info {
            message = Self::filter_debug_info(&message);
        }

        // Truncate if too long
        if message.len() > self.config.max_error_length {
            message.truncate(self.config.max_error_length);
            message.push_str("...");
        }

        // Determine if error details should be included
        let category = if let Some(ref kind) = error_info.kind {
            if self.config.allowed_error_details.is_empty()
                || self.config.allowed_error_details.contains(kind)
            {
                Some(kind.clone())
            } else {
                None
            }
        } else {
            None
        };

        // Generate error code from hash of original message
        let error_code = self.generate_error_code(&error_info.message);

        // Determine if retriable based on error kind
        let retriable = matches!(
            category.as_deref(),
            Some("timeout" | "rate_limit" | "temporary")
        );

        SanitizedError {
            message,
            category,
            error_code,
            retriable,
        }
    }

    /// Mask sensitive data in text
    fn mask_sensitive_data(&self, text: &str) -> String {
        let mut result = text.to_string();

        for pattern in &self.config.sensitive_patterns {
            result = pattern.replace_all(&result, "[REDACTED]").to_string();
        }

        result
    }

    /// Sanitize file paths
    fn sanitize_paths(&self, text: &str) -> String {
        let mut cache = self.path_cache.lock();
        let mut result = text.to_string();

        // Common path patterns to sanitize
        let path_patterns = [
            regex::Regex::new(r"(/[^/\s]+)+/([^/\s]+)").unwrap(),
            regex::Regex::new(r"([A-Za-z]:\\(?:[^\\]+\\)*[^\\]+)").unwrap(),
        ];

        for pattern in &path_patterns {
            for captures in pattern.captures_iter(text) {
                if let Some(full_match) = captures.get(0) {
                    let path = PathBuf::from(full_match.as_str());

                    // Check cache first
                    if let Some(sanitized) = cache.get(&path) {
                        result = result.replace(full_match.as_str(), sanitized);
                        continue;
                    }

                    // Sanitize the path
                    let sanitized = if let Some(file_name) = path.file_name() {
                        format!("[path]/.../{}", file_name.to_string_lossy())
                    } else {
                        "[path]".to_string()
                    };

                    cache.insert(path, sanitized.clone());
                    result = result.replace(full_match.as_str(), &sanitized);
                }
            }
        }

        result
    }

    /// Filter debug information
    fn filter_debug_info(text: &str) -> String {
        let mut result = text.to_string();

        // Remove memory addresses
        let addr_pattern = regex::Regex::new(r"0x[0-9a-fA-F]+").unwrap();
        result = addr_pattern.replace_all(&result, "[addr]").to_string();

        // Remove thread IDs
        let thread_pattern = regex::Regex::new(r"thread\s+'[^']+'\s+panicked").unwrap();
        result = thread_pattern
            .replace_all(&result, "thread panicked")
            .to_string();

        // Remove specific version numbers
        let version_pattern = regex::Regex::new(r"v?\d+\.\d+\.\d+(-[a-zA-Z0-9]+)?").unwrap();
        result = version_pattern
            .replace_all(&result, "[version]")
            .to_string();

        // Remove line numbers from stack traces
        let line_pattern = regex::Regex::new(r":\d+:\d+").unwrap();
        result = line_pattern.replace_all(&result, ":[line]").to_string();

        result
    }

    /// Generate a stable error code from message
    pub fn generate_error_code(&self, message: &str) -> String {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        message.hash(&mut hasher);
        let hash = hasher.finish();
        format!("ERR_{:08X}", (hash & 0xFFFF_FFFF) as u32)
    }

    /// Sanitize a log message
    pub fn sanitize_log(&self, message: &str) -> String {
        let mut result = message.to_string();

        if self.config.mask_sensitive_data {
            result = self.mask_sensitive_data(&result);
        }

        if self.config.sanitize_paths {
            result = self.sanitize_paths(&result);
        }

        result
    }

    /// Check if a string contains sensitive information
    pub fn contains_sensitive_info(&self, text: &str) -> bool {
        self.config
            .sensitive_patterns
            .iter()
            .any(|pattern| pattern.is_match(text))
    }
}

/// Production error handler that prevents information disclosure
pub struct ProductionErrorHandler {
    preventer: Arc<InfoDisclosurePreventer>,
    /// Whether to log full errors internally
    log_full_errors: bool,
}

impl ProductionErrorHandler {
    /// Create a new production error handler
    #[must_use]
    pub fn new(preventer: Arc<InfoDisclosurePreventer>) -> Self {
        Self {
            preventer,
            log_full_errors: true,
        }
    }

    /// Handle an error safely
    pub fn handle_error(&self, error: &dyn std::error::Error) -> SanitizedError {
        let error_info = ErrorInfo {
            message: error.to_string(),
            kind: Some(Self::categorize_error(error)),
            stack_trace: None, // Would be captured in production
            context: HashMap::new(),
            source_location: None,
        };

        // Log full error internally if enabled
        if self.log_full_errors {
            tracing::error!(
                error_code = %self.preventer.generate_error_code(&error_info.message),
                "Internal error: {}",
                error
            );
        }

        self.preventer.sanitize_error(&error_info)
    }

    /// Categorize error for allowed disclosure
    fn categorize_error(error: &dyn std::error::Error) -> String {
        let error_str = error.to_string().to_lowercase();

        if error_str.contains("validation") || error_str.contains("invalid") {
            "validation".to_string()
        } else if error_str.contains("permission") || error_str.contains("denied") {
            "permission".to_string()
        } else if error_str.contains("not found") || error_str.contains("404") {
            "not_found".to_string()
        } else if error_str.contains("timeout") {
            "timeout".to_string()
        } else if error_str.contains("rate limit") || error_str.contains("429") {
            "rate_limit".to_string()
        } else {
            "internal".to_string()
        }
    }
}

/// Logging filter to prevent information disclosure
pub struct LoggingFilter {
    preventer: Arc<InfoDisclosurePreventer>,
}

impl LoggingFilter {
    /// Create a new logging filter
    #[must_use]
    pub fn new(preventer: Arc<InfoDisclosurePreventer>) -> Self {
        Self { preventer }
    }

    /// Filter a log message
    #[must_use]
    pub fn filter(&self, message: &str) -> String {
        self.preventer.sanitize_log(message)
    }

    /// Check if a log message should be filtered out entirely
    #[must_use]
    pub fn should_filter(&self, message: &str) -> bool {
        // Filter out messages that are mostly sensitive data
        let sanitized = self.preventer.sanitize_log(message);
        let redacted_count = sanitized.matches("[REDACTED]").count();
        let word_count = sanitized.split_whitespace().count();

        // If more than 50% of words are redacted, filter the message
        redacted_count > 0 && redacted_count * 2 > word_count
    }
}

#[cfg(test)]
#[cfg_attr(test_category = "util")]
mod tests {
    use super::*;

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_sensitive_data_masking() {
        let config = InfoDisclosureConfig::default();
        let preventer = InfoDisclosurePreventer::new(config);

        let test_cases = vec![
            ("API_KEY=abc123def456", "[REDACTED]"),
            ("token: xyz789ghi012", "[REDACTED]"),
            ("email: user@example.com", "[REDACTED]"),
            ("IP: 192.168.1.1", "[REDACTED]"),
            ("password=secret123", "[REDACTED]"),
        ];

        for (input, expected) in test_cases {
            let result = preventer.mask_sensitive_data(input);
            assert!(result.contains(expected), "Input: {input}, Got: {result}");
        }
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_path_sanitization() {
        let config = InfoDisclosureConfig::default();
        let preventer = InfoDisclosurePreventer::new(config);

        let test_cases = vec![
            ("/home/user/project/file.rs", "[path]/.../file.rs"),
            ("/Users/john/Documents/secret.txt", "[path]/.../secret.txt"),
            // Windows paths need the full path to be sanitized
            ("C:\\Users\\Admin\\Desktop\\data.csv", "[path]"),
        ];

        for (input, expected) in test_cases {
            let result = preventer.sanitize_paths(input);
            assert!(result.contains(expected), "Input: {input}, Got: {result}");
        }
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_debug_info_filtering() {
        let config = InfoDisclosureConfig::default();
        let _preventer = InfoDisclosurePreventer::new(config);

        let test_cases = vec![
            ("at address 0x7fff5fbff8c0", "at address [addr]"),
            ("thread 'main' panicked", "thread panicked"),
            ("version 1.2.3-beta", "version [version]"),
            ("at src/main.rs:42:15", "at src/main.rs:[line]"),
        ];

        for (input, expected) in test_cases {
            let result = InfoDisclosurePreventer::filter_debug_info(input);
            assert_eq!(result, expected);
        }
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_error_sanitization() {
        let config = InfoDisclosureConfig::production();
        let preventer = InfoDisclosurePreventer::new(config);

        let error_info = ErrorInfo {
            message: "Failed to connect to database at postgres://user:pass@localhost/db"
                .to_string(),
            kind: Some("database".to_string()),
            stack_trace: Some("at main.rs:42:15".to_string()),
            context: HashMap::new(),
            source_location: Some(("main.rs".to_string(), 42)),
        };

        let sanitized = preventer.sanitize_error(&error_info);
        assert!(sanitized.message.contains("[REDACTED]"));
        assert!(sanitized.error_code.starts_with("ERR_"));
        assert_eq!(sanitized.category, None); // database not in allowed list
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_development_vs_production() {
        let dev_config = InfoDisclosureConfig::development();
        let prod_config = InfoDisclosureConfig::production();

        assert!(dev_config.include_stack_traces);
        assert!(!prod_config.include_stack_traces);

        assert!(!dev_config.sanitize_paths);
        assert!(prod_config.sanitize_paths);
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_logging_filter() {
        let config = InfoDisclosureConfig::default();
        let preventer = Arc::new(InfoDisclosurePreventer::new(config));
        let filter = LoggingFilter::new(preventer);

        let message = "Processing user@example.com with token abc123";
        let filtered = filter.filter(message);
        assert!(filtered.contains("[REDACTED]"));

        // Test should_filter
        let mostly_sensitive = "API_KEY=abc123 TOKEN=xyz789 SECRET=qwe456";
        assert!(filter.should_filter(mostly_sensitive));

        let mostly_safe = "Processing request for user ID 123";
        assert!(!filter.should_filter(mostly_safe));
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_error_categorization() {
        let preventer = Arc::new(InfoDisclosurePreventer::production());
        let handler = ProductionErrorHandler::new(preventer);

        // Create a simple error for testing
        let validation_error =
            std::io::Error::new(std::io::ErrorKind::InvalidInput, "Invalid input provided");

        let sanitized = handler.handle_error(&validation_error);
        assert_eq!(sanitized.category, Some("validation".to_string()));
        assert!(!sanitized.retriable);
    }
}
