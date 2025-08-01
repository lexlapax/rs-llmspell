//! ABOUTME: Credential protection framework for secure handling of sensitive data
//! ABOUTME: Provides secure string types, memory scrubbing, and credential filtering

#![allow(clippy::must_use_candidate)]

use serde::{Deserialize, Serialize};
use std::fmt;
use zeroize::{Zeroize, ZeroizeOnDrop};

/// Secure string that zeros memory on drop
#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct SecureString {
    inner: String,
}

impl SecureString {
    /// Create a new `SecureString`
    #[must_use]
    pub fn new(value: String) -> Self {
        Self { inner: value }
    }

    /// Create from a str
    #[must_use]
    pub fn from(value: &str) -> Self {
        Self {
            inner: value.to_string(),
        }
    }

    /// Get the inner value (use with caution)
    pub fn expose_secret(&self) -> &str {
        &self.inner
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Get length
    pub fn len(&self) -> usize {
        self.inner.len()
    }
}

impl fmt::Debug for SecureString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SecureString[REDACTED]")
    }
}

impl fmt::Display for SecureString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[REDACTED]")
    }
}

/// Credential type enumeration
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CredentialType {
    /// API key credential
    ApiKey,
    /// Password credential
    Password,
    /// Authentication token
    Token,
    /// Generic secret
    Secret,
    /// Private key
    PrivateKey,
    /// Certificate
    Certificate,
    /// Other credential type
    Other(String),
}

/// Secure credential container
#[derive(Clone)]
pub struct SecureCredential {
    credential_type: CredentialType,
    value: SecureString,
    metadata: Option<String>,
}

impl SecureCredential {
    /// Create a new secure credential
    pub fn new(credential_type: CredentialType, value: String) -> Self {
        Self {
            credential_type,
            value: SecureString::new(value),
            metadata: None,
        }
    }

    /// Create with metadata
    pub fn with_metadata(credential_type: CredentialType, value: String, metadata: String) -> Self {
        Self {
            credential_type,
            value: SecureString::new(value),
            metadata: Some(metadata),
        }
    }

    /// Get credential type
    pub fn credential_type(&self) -> &CredentialType {
        &self.credential_type
    }

    /// Get the value (use with caution)
    pub fn expose_secret(&self) -> &str {
        self.value.expose_secret()
    }

    /// Get metadata
    pub fn metadata(&self) -> Option<&str> {
        self.metadata.as_deref()
    }
}

impl fmt::Debug for SecureCredential {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "SecureCredential {{ type: {:?}, value: [REDACTED], metadata: {:?} }}",
            self.credential_type, self.metadata
        )
    }
}

impl Drop for SecureCredential {
    fn drop(&mut self) {
        // Additional cleanup if needed
        if let Some(ref mut metadata) = self.metadata {
            metadata.zeroize();
        }
    }
}

/// Log filter for removing credentials from logs
pub struct CredentialFilter {
    patterns: Vec<regex::Regex>,
}

impl CredentialFilter {
    /// Create a new credential filter with default patterns
    ///
    /// # Panics
    ///
    /// Panics if any of the default regex patterns fail to compile
    pub fn new() -> Self {
        let patterns = vec![
            // API keys
            regex::Regex::new(r#"(?i)(api[_-]?key|apikey)\s*[:=]?\s*['"]*([A-Za-z0-9\-_]{5,})['"]*"#).unwrap(),
            // Passwords
            regex::Regex::new(r#"(?i)(password|passwd|pwd)\s*[:=]?\s*['"]*([^\s'"]+)['"]*"#).unwrap(),
            // Tokens
            regex::Regex::new(r#"(?i)(token|auth|authorization)\s*[:=]?\s*['"]*([A-Za-z0-9\-_\.]{10,})['"]*"#).unwrap(),
            // Bearer tokens
            regex::Regex::new(r"(?i)bearer\s+([A-Za-z0-9\-_\.]+)").unwrap(),
            // Basic auth
            regex::Regex::new(r"(?i)basic\s+([A-Za-z0-9+/=]+)").unwrap(),
            // AWS credentials
            regex::Regex::new(r#"(?i)(aws_access_key_id|aws_secret_access_key)\s*[:=]\s*['"]*([A-Za-z0-9+/=]+)['"]*"#).unwrap(),
            // Private keys
            regex::Regex::new(r"-----BEGIN\s+(?:RSA\s+)?PRIVATE\s+KEY-----[\s\S]+?-----END\s+(?:RSA\s+)?PRIVATE\s+KEY-----").unwrap(),
            // Connection strings with passwords
            regex::Regex::new(r"(?i)(mongodb|mysql|postgresql|redis|amqp)://[^:]+:([^@]+)@").unwrap(),
            // Generic secrets
            regex::Regex::new(r#"(?i)(secret|private[_-]?key|encryption[_-]?key)\s*[:=]\s*['"]*([A-Za-z0-9\-_]{16,})['"]*"#).unwrap(),
        ];

        Self { patterns }
    }

    /// Add a custom pattern
    ///
    /// # Errors
    ///
    /// Returns a regex error if the pattern is invalid
    pub fn add_pattern(&mut self, pattern: &str) -> Result<(), regex::Error> {
        let regex = regex::Regex::new(pattern)?;
        self.patterns.push(regex);
        Ok(())
    }

    /// Filter credentials from text
    pub fn filter(&self, text: &str) -> String {
        let mut filtered = text.to_string();

        for pattern in &self.patterns {
            filtered = pattern
                .replace_all(&filtered, |caps: &regex::Captures| {
                    // Keep the key part but redact the value
                    if caps.len() > 2 {
                        format!("{}[REDACTED]", &caps[1])
                    } else {
                        "[REDACTED]".to_string()
                    }
                })
                .to_string();
        }

        filtered
    }

    /// Check if text contains credentials
    pub fn contains_credentials(&self, text: &str) -> bool {
        self.patterns.iter().any(|pattern| pattern.is_match(text))
    }

    /// Get detected credential types
    ///
    /// # Panics
    ///
    /// Panics if any of the detection regex patterns fail to compile
    pub fn detect_credential_types(&self, text: &str) -> Vec<String> {
        let mut types = Vec::new();

        if regex::Regex::new(r"(?i)api[_-]?key")
            .unwrap()
            .is_match(text)
        {
            types.push("API Key".to_string());
        }
        if regex::Regex::new(r"(?i)password|passwd|pwd")
            .unwrap()
            .is_match(text)
        {
            types.push("Password".to_string());
        }
        if regex::Regex::new(r"(?i)token|auth|authorization")
            .unwrap()
            .is_match(text)
        {
            types.push("Token".to_string());
        }
        if regex::Regex::new(r"-----BEGIN\s+(?:RSA\s+)?PRIVATE\s+KEY-----")
            .unwrap()
            .is_match(text)
        {
            types.push("Private Key".to_string());
        }
        if regex::Regex::new(r"(?i)secret").unwrap().is_match(text) {
            types.push("Secret".to_string());
        }

        types
    }
}

impl Default for CredentialFilter {
    fn default() -> Self {
        Self::new()
    }
}

/// Error message sanitizer
pub struct ErrorSanitizer {
    filter: CredentialFilter,
}

impl ErrorSanitizer {
    /// Create a new error sanitizer
    pub fn new() -> Self {
        Self {
            filter: CredentialFilter::new(),
        }
    }

    /// Sanitize an error message
    ///
    /// # Panics
    ///
    /// Panics if any of the sanitization regex patterns fail to compile
    pub fn sanitize(&self, error: &str) -> String {
        // First filter credentials
        let mut sanitized = self.filter.filter(error);

        // Remove file paths that might contain usernames
        let path_regex =
            regex::Regex::new(r"(/home/[^/\s]+|/Users/[^/\s]+|C:\\Users\\[^\\s]+)").unwrap();
        sanitized = path_regex.replace_all(&sanitized, "[PATH]").to_string();

        // Remove email addresses
        let email_regex =
            regex::Regex::new(r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b").unwrap();
        sanitized = email_regex.replace_all(&sanitized, "[EMAIL]").to_string();

        // Remove IP addresses
        let ip_regex = regex::Regex::new(r"\b(?:\d{1,3}\.){3}\d{1,3}\b").unwrap();
        sanitized = ip_regex.replace_all(&sanitized, "[IP]").to_string();

        // Remove URLs that might contain sensitive info
        let url_regex = regex::Regex::new(r"https?://[^\s]+").unwrap();
        sanitized = url_regex
            .replace_all(&sanitized, |caps: &regex::Captures| {
                let url = &caps[0];
                if self.filter.contains_credentials(url) {
                    "[URL-REDACTED]".to_string()
                } else {
                    // Keep domain but remove path/query
                    if let Some(idx) = url[8..].find('/').map(|i| i + 8) {
                        format!("{}[PATH]", &url[..idx])
                    } else {
                        url.to_string()
                    }
                }
            })
            .to_string();

        sanitized
    }
}

impl Default for ErrorSanitizer {
    fn default() -> Self {
        Self::new()
    }
}

/// Audit logger for credential access
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CredentialAuditEntry {
    /// Timestamp of the credential access
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Action performed (e.g., "access", "use")
    pub action: String,
    /// Type of credential accessed
    pub credential_type: String,
    /// Optional metadata about the access
    pub metadata: Option<String>,
    /// Tool that accessed the credential
    pub tool: String,
    /// Whether the access was successful
    pub success: bool,
    /// Error message if access failed
    pub error: Option<String>,
}

impl CredentialAuditEntry {
    /// Create a new audit entry
    pub fn new(action: String, credential_type: String, tool: String, success: bool) -> Self {
        Self {
            timestamp: chrono::Utc::now(),
            action,
            credential_type,
            metadata: None,
            tool,
            success,
            error: None,
        }
    }

    /// Add metadata
    #[must_use]
    pub fn with_metadata(mut self, metadata: String) -> Self {
        self.metadata = Some(metadata);
        self
    }

    /// Add error
    #[must_use]
    pub fn with_error(mut self, error: &str) -> Self {
        let sanitizer = ErrorSanitizer::new();
        self.error = Some(sanitizer.sanitize(error));
        self
    }
}

/// Credential audit logger
pub struct CredentialAuditor {
    entries: Vec<CredentialAuditEntry>,
}

impl CredentialAuditor {
    /// Create a new auditor
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// Log a credential access
    pub fn log_access(&mut self, entry: CredentialAuditEntry) {
        self.entries.push(entry);
    }

    /// Get audit entries
    pub fn entries(&self) -> &[CredentialAuditEntry] {
        &self.entries
    }

    /// Get entries for a specific tool
    pub fn entries_for_tool(&self, tool: &str) -> Vec<&CredentialAuditEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.tool == tool)
            .collect()
    }

    /// Clear audit log
    pub fn clear(&mut self) {
        self.entries.clear();
    }
}

impl Default for CredentialAuditor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_secure_string() {
        let secret = SecureString::from("my-secret-value");
        assert_eq!(secret.expose_secret(), "my-secret-value");
        assert_eq!(format!("{secret:?}"), "SecureString[REDACTED]");
        assert_eq!(format!("{secret}"), "[REDACTED]");
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_secure_credential() {
        let cred = SecureCredential::new(CredentialType::ApiKey, "sk-1234567890abcdef".to_string());
        assert_eq!(cred.expose_secret(), "sk-1234567890abcdef");
        assert!(format!("{cred:?}").contains("[REDACTED]"));
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_credential_filter() {
        let filter = CredentialFilter::new();

        // Test API key filtering
        let text = "api_key: sk-1234567890abcdef";
        let filtered = filter.filter(text);
        assert_eq!(filtered, "api_key[REDACTED]");

        // Test password filtering
        let text = "password=mysecretpass123";
        let filtered = filter.filter(text);
        assert_eq!(filtered, "password[REDACTED]");

        // Test token filtering
        let text = "Authorization: Bearer eyJ0eXAiOiJKV1QiLCJhbGc";
        let filtered = filter.filter(text);
        assert!(filtered.contains("[REDACTED]"));

        // Test connection string
        let text = "mongodb://user:password123@localhost:27017";
        let filtered = filter.filter(text);
        assert!(filtered.contains("[REDACTED]"));
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_error_sanitizer() {
        let sanitizer = ErrorSanitizer::new();

        // Test credential removal
        let cred_error = "Failed to connect with api_key=sk-12345";
        let cred_sanitized = sanitizer.sanitize(cred_error);
        assert!(cred_sanitized.contains("[REDACTED]"));

        // Test path removal
        let path_error = "File not found: /home/username/secret.txt";
        let path_sanitized = sanitizer.sanitize(path_error);
        assert!(path_sanitized.contains("[PATH]"));

        // Test email removal
        let email_error = "Invalid email: user@example.com";
        let email_sanitized = sanitizer.sanitize(email_error);
        assert!(email_sanitized.contains("[EMAIL]"));

        // Test IP removal
        let ip_error = "Connection failed to 192.168.1.1";
        let ip_sanitized = sanitizer.sanitize(ip_error);
        assert!(ip_sanitized.contains("[IP]"));
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_credential_detection() {
        let filter = CredentialFilter::new();

        assert!(filter.contains_credentials("api_key=12345"));
        assert!(filter.contains_credentials("password: secret"));
        assert!(filter.contains_credentials("Bearer token123"));
        assert!(!filter.contains_credentials("This is just plain text"));

        let types = filter.detect_credential_types("api_key=123 password=456");
        assert!(types.contains(&"API Key".to_string()));
        assert!(types.contains(&"Password".to_string()));
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_audit_logger() {
        let mut auditor = CredentialAuditor::new();

        let entry = CredentialAuditEntry::new(
            "access".to_string(),
            "api_key".to_string(),
            "web_search".to_string(),
            true,
        );

        auditor.log_access(entry);
        assert_eq!(auditor.entries().len(), 1);

        let tool_entries = auditor.entries_for_tool("web_search");
        assert_eq!(tool_entries.len(), 1);
    }
}
