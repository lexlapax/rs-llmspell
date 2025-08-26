// ABOUTME: Common validation functions for consistent error handling across tools
// ABOUTME: Provides file size, string length, path, and other validation utilities

//! Common validation utilities
//!
//! This module provides reusable validation functions to ensure consistent
//! validation logic and error messages across all LLMSpell tools.

use llmspell_core::{LLMSpellError, Result as LLMResult};
use std::path::Path;

/// Validate that a file size is within allowed limits
///
/// # Examples
/// ```rust,ignore
/// validate_file_size(file_size, 100 * 1024 * 1024)?; // 100MB limit
/// ```
///
/// # Errors
///
/// Returns `LLMSpellError::Validation` if the file size exceeds the maximum allowed size
pub fn validate_file_size(size: u64, max_size: u64) -> LLMResult<()> {
    if size > max_size {
        return Err(LLMSpellError::Validation {
            message: format!(
                "File size ({size} bytes) exceeds maximum allowed size ({max_size} bytes)"
            ),
            field: Some("file_size".to_string()),
        });
    }
    Ok(())
}

/// Validate that a string length is within allowed limits
///
/// # Errors
///
/// Returns `LLMSpellError::Validation` if the string length exceeds the maximum allowed length
pub fn validate_string_length(s: &str, max_length: usize) -> LLMResult<()> {
    if s.len() > max_length {
        return Err(LLMSpellError::Validation {
            message: format!(
                "String length ({} characters) exceeds maximum allowed length ({max_length} characters)",
                s.len()
            ),
            field: Some("string_length".to_string()),
        });
    }
    Ok(())
}

/// Validate that a string is not empty
///
/// # Errors
///
/// Returns `LLMSpellError::Validation` if the string is empty
pub fn validate_not_empty(s: &str, field_name: &str) -> LLMResult<()> {
    if s.is_empty() {
        return Err(LLMSpellError::Validation {
            message: format!("{field_name} cannot be empty"),
            field: Some(field_name.to_string()),
        });
    }
    Ok(())
}

/// Validate that a required field is present
///
/// # Errors
///
/// Returns `LLMSpellError::Validation` if the value is `None`
pub fn validate_required_field<T>(value: Option<T>, field_name: &str) -> LLMResult<T> {
    value.ok_or_else(|| LLMSpellError::Validation {
        message: format!("{field_name} is required"),
        field: Some(field_name.to_string()),
    })
}

/// Validate that a number is within a range
///
/// # Errors
///
/// Returns `LLMSpellError::Validation` if the value is outside the specified range
pub fn validate_range<T>(value: &T, min: &T, max: &T, field_name: &str) -> LLMResult<()>
where
    T: PartialOrd + std::fmt::Display,
{
    if value < min || value > max {
        return Err(LLMSpellError::Validation {
            message: format!("{field_name} must be between {min} and {max} (got {value})"),
            field: Some(field_name.to_string()),
        });
    }
    Ok(())
}

/// Validate that a path exists
///
/// # Errors
///
/// Returns `LLMSpellError::Storage` if the path does not exist
pub fn validate_path_exists(path: &Path, path_type: &str) -> LLMResult<()> {
    if !path.exists() {
        return Err(LLMSpellError::Storage {
            message: format!("{path_type} does not exist: {}", path.display()),
            operation: Some("validate".to_string()),
            source: None,
        });
    }
    Ok(())
}

/// Validate that a path is a file
///
/// # Errors
///
/// Returns `LLMSpellError::Validation` if the path is not a file
pub fn validate_is_file(path: &Path) -> LLMResult<()> {
    if !path.is_file() {
        return Err(LLMSpellError::Validation {
            message: format!("Path is not a file: {}", path.display()),
            field: Some("path".to_string()),
        });
    }
    Ok(())
}

/// Validate that a path is a directory
///
/// # Errors
///
/// Returns `LLMSpellError::Validation` if the path is not a directory
pub fn validate_is_directory(path: &Path) -> LLMResult<()> {
    if !path.is_dir() {
        return Err(LLMSpellError::Validation {
            message: format!("Path is not a directory: {}", path.display()),
            field: Some("path".to_string()),
        });
    }
    Ok(())
}

/// Validate that a string matches a pattern
///
/// # Errors
///
/// Returns `LLMSpellError::Validation` if the regex pattern is invalid or the string doesn't match
pub fn validate_pattern(s: &str, pattern: &str, field_name: &str) -> LLMResult<()> {
    let re = regex::Regex::new(pattern).map_err(|e| LLMSpellError::Validation {
        message: format!("Invalid regex pattern: {e}"),
        field: Some("pattern".to_string()),
    })?;

    if !re.is_match(s) {
        return Err(LLMSpellError::Validation {
            message: format!("{field_name} does not match required pattern: {pattern}"),
            field: Some(field_name.to_string()),
        });
    }
    Ok(())
}

/// Validate that a string is a valid identifier (alphanumeric + underscore)
///
/// # Errors
///
/// Returns `LLMSpellError::Validation` if the string is not a valid identifier
///
/// # Panics
///
/// This function should not panic as we check for empty string before calling unwrap
pub fn validate_identifier(s: &str, field_name: &str) -> LLMResult<()> {
    if s.is_empty() {
        return Err(LLMSpellError::Validation {
            message: format!("{field_name} cannot be empty"),
            field: Some(field_name.to_string()),
        });
    }

    if !s.chars().all(|c| c.is_alphanumeric() || c == '_') {
        return Err(LLMSpellError::Validation {
            message: format!(
                "{field_name} must contain only alphanumeric characters and underscores"
            ),
            field: Some(field_name.to_string()),
        });
    }

    if s.chars().next().unwrap().is_numeric() {
        return Err(LLMSpellError::Validation {
            message: format!("{field_name} cannot start with a number"),
            field: Some(field_name.to_string()),
        });
    }

    Ok(())
}

/// Validate that a collection is not empty
///
/// # Errors
///
/// Returns `LLMSpellError::Validation` if the collection is empty
pub fn validate_not_empty_collection<T>(collection: &[T], field_name: &str) -> LLMResult<()> {
    if collection.is_empty() {
        return Err(LLMSpellError::Validation {
            message: format!("{field_name} cannot be empty"),
            field: Some(field_name.to_string()),
        });
    }
    Ok(())
}

/// Validate that a value is one of allowed values
///
/// # Errors
///
/// Returns `LLMSpellError::Validation` if the value is not in the allowed list
pub fn validate_enum<T>(value: &T, allowed: &[T], field_name: &str) -> LLMResult<()>
where
    T: PartialEq + std::fmt::Display,
{
    if !allowed.contains(value) {
        let allowed_str: Vec<String> = allowed
            .iter()
            .map(std::string::ToString::to_string)
            .collect();
        return Err(LLMSpellError::Validation {
            message: format!(
                "Invalid {field_name}: '{value}'. Allowed values: {}",
                allowed_str.join(", ")
            ),
            field: Some(field_name.to_string()),
        });
    }
    Ok(())
}

/// Validate URL format
///
/// # Errors
///
/// Returns `LLMSpellError::Validation` if the URL is invalid
pub fn validate_url(url: &str, field_name: &str) -> LLMResult<()> {
    url::Url::parse(url).map_err(|e| LLMSpellError::Validation {
        message: format!("Invalid URL in {field_name}: {e}"),
        field: Some(field_name.to_string()),
    })?;
    Ok(())
}

/// Validate email format
///
/// # Errors
///
/// Returns `LLMSpellError::Validation` if the email format is invalid
pub fn validate_email(email: &str, field_name: &str) -> LLMResult<()> {
    // Simple email validation - not RFC compliant but good enough for most cases
    let email_regex = r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$";
    validate_pattern(email, email_regex, field_name)
}

/// Validate JSON data against a JSON schema
///
/// # Errors
///
/// Returns `LLMSpellError::Validation` if the schema is invalid or the data doesn't match
pub fn validate_json_schema(data: &serde_json::Value, schema: &serde_json::Value) -> LLMResult<()> {
    use jsonschema::{Draft, JSONSchema};

    let compiled = JSONSchema::options()
        .with_draft(Draft::Draft7)
        .compile(schema)
        .map_err(|e| LLMSpellError::Validation {
            message: format!("Invalid JSON schema: {e}"),
            field: Some("schema".to_string()),
        })?;

    let result = compiled.validate(data);
    if let Err(errors) = result {
        let error_messages: Vec<String> = errors
            .into_iter()
            .map(|e| format!("- {}: {e}", e.instance_path))
            .collect();
        return Err(LLMSpellError::Validation {
            message: format!("Schema validation failed:\n{}", error_messages.join("\n")),
            field: Some("data".to_string()),
        });
    }

    Ok(())
}

/// Validate that a regex pattern is valid
///
/// # Errors
///
/// Returns `LLMSpellError::Validation` if the regex pattern is invalid
pub fn validate_regex_pattern(pattern: &str) -> LLMResult<()> {
    regex::Regex::new(pattern).map_err(|e| LLMSpellError::Validation {
        message: format!("Invalid regex pattern: {e}"),
        field: Some("pattern".to_string()),
    })?;
    Ok(())
}

/// Validate that a date string matches a specific format
///
/// # Errors
///
/// Returns `LLMSpellError::Validation` if the date doesn't match the format
pub fn validate_date_format(date: &str, format: &str) -> LLMResult<()> {
    use chrono::{NaiveDate, NaiveDateTime};

    // Try parsing as datetime first
    if NaiveDateTime::parse_from_str(date, format).is_ok() {
        return Ok(());
    }

    // Try parsing as date only
    if NaiveDate::parse_from_str(date, format).is_ok() {
        return Ok(());
    }

    Err(LLMSpellError::Validation {
        message: format!("Invalid date format. Expected: {format}"),
        field: Some("date".to_string()),
    })
}

/// Validate that a path is safe (no path traversal, symlink attacks, etc.)
///
/// # Errors
///
/// Returns `LLMSpellError::Validation` if the path contains dangerous patterns
pub fn validate_safe_path(path: &Path, jail_dir: Option<&Path>) -> LLMResult<()> {
    use crate::file_utils::normalize_path;

    // Check for path traversal attempts
    let path_str = path.to_string_lossy();
    if path_str.contains("..") || path_str.contains('~') {
        return Err(LLMSpellError::Validation {
            message: "Path contains potentially dangerous patterns (.. or ~)".to_string(),
            field: Some("path".to_string()),
        });
    }

    // Normalize the path to resolve any relative components
    let normalized = normalize_path(path);

    // If a jail directory is specified, ensure the path is within it
    if let Some(jail) = jail_dir {
        let jail_normalized = normalize_path(jail);
        if !normalized.starts_with(&jail_normalized) {
            return Err(LLMSpellError::Validation {
                message: format!(
                    "Path '{}' is outside the allowed directory '{}'",
                    path.display(),
                    jail.display()
                ),
                field: Some("path".to_string()),
            });
        }
    }

    // Check if path is a symlink (potential security risk)
    if path.exists() && path.read_link().is_ok() {
        return Err(LLMSpellError::Validation {
            message: "Symlinks are not allowed for security reasons".to_string(),
            field: Some("path".to_string()),
        });
    }

    Ok(())
}

/// Sanitize a string for safe usage (remove control characters, etc.)
///
/// This function removes or replaces potentially dangerous characters
#[must_use]
pub fn sanitize_string(input: &str, allow_newlines: bool) -> String {
    input
        .chars()
        .filter(|&c| {
            if c.is_control() {
                allow_newlines && (c == '\n' || c == '\r' || c == '\t')
            } else {
                true
            }
        })
        .collect()
}

/// Validate resource limits (memory, file size, etc.)
///
/// # Errors
///
/// Returns `LLMSpellError::Validation` if the value exceeds the limit
pub fn validate_resource_limit(
    value: u64,
    limit: u64,
    resource_name: &str,
    unit: &str,
) -> LLMResult<()> {
    if value > limit {
        return Err(LLMSpellError::Validation {
            message: format!(
                "{resource_name} ({value} {unit}) exceeds maximum allowed limit ({limit} {unit})"
            ),
            field: Some(resource_name.to_string()),
        });
    }
    Ok(())
}

/// Validate that a string doesn't contain dangerous shell characters
///
/// # Errors
///
/// Returns `LLMSpellError::Validation` if dangerous characters are found
pub fn validate_no_shell_injection(input: &str, field_name: &str) -> LLMResult<()> {
    const DANGEROUS_CHARS: &[char] = &['$', '`', '\\', ';', '|', '&', '>', '<', '(', ')', '{', '}'];

    for c in DANGEROUS_CHARS {
        if input.contains(*c) {
            return Err(LLMSpellError::Validation {
                message: format!(
                    "{field_name} contains potentially dangerous character '{c}' that could be used for shell injection"
                ),
                field: Some(field_name.to_string()),
            });
        }
    }
    Ok(())
}

/// Validate file permissions (Unix-specific)
///
/// # Errors
///
/// Returns `LLMSpellError::Storage` if unable to get file metadata
/// Returns `LLMSpellError::Validation` if permissions don't match required mode
#[cfg(unix)]
pub fn validate_file_permissions(path: &Path, required_mode: u32) -> LLMResult<()> {
    use std::os::unix::fs::PermissionsExt;

    let metadata = path.metadata().map_err(|e| LLMSpellError::Storage {
        message: format!("Failed to get file metadata: {e}"),
        operation: Some("validate_permissions".to_string()),
        source: Some(Box::new(e)),
    })?;

    let permissions = metadata.permissions();
    let mode = permissions.mode();

    if (mode & 0o777) != required_mode {
        return Err(LLMSpellError::Validation {
            message: format!(
                "File permissions ({:o}) do not match required permissions ({:o})",
                mode & 0o777,
                required_mode
            ),
            field: Some("permissions".to_string()),
        });
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;
    #[test]
    fn test_validate_file_size() {
        assert!(validate_file_size(100, 1000).is_ok());
        assert!(validate_file_size(2000, 1000).is_err());
    }
    #[test]
    fn test_validate_string_length() {
        assert!(validate_string_length("hello", 10).is_ok());
        assert!(validate_string_length("hello world", 5).is_err());
    }
    #[test]
    fn test_validate_not_empty() {
        assert!(validate_not_empty("hello", "field").is_ok());
        assert!(validate_not_empty("", "field").is_err());
    }
    #[test]
    fn test_validate_required_field() {
        assert_eq!(validate_required_field(Some(42), "field").unwrap(), 42);
        assert!(validate_required_field::<i32>(None, "field").is_err());
    }
    #[test]
    fn test_validate_range() {
        assert!(validate_range(&5, &1, &10, "value").is_ok());
        assert!(validate_range(&0, &1, &10, "value").is_err());
        assert!(validate_range(&11, &1, &10, "value").is_err());
    }
    #[test]
    fn test_validate_path_exists() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        fs::write(&file_path, "test").unwrap();

        assert!(validate_path_exists(&file_path, "file").is_ok());
        assert!(validate_path_exists(&temp_dir.path().join("missing.txt"), "file").is_err());
    }
    #[test]
    fn test_validate_is_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        fs::write(&file_path, "test").unwrap();

        assert!(validate_is_file(&file_path).is_ok());
        assert!(validate_is_file(temp_dir.path()).is_err());
    }
    #[test]
    fn test_validate_is_directory() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        fs::write(&file_path, "test").unwrap();

        assert!(validate_is_directory(temp_dir.path()).is_ok());
        assert!(validate_is_directory(&file_path).is_err());
    }
    #[test]
    fn test_validate_identifier() {
        assert!(validate_identifier("valid_name", "field").is_ok());
        assert!(validate_identifier("_valid", "field").is_ok());
        assert!(validate_identifier("", "field").is_err());
        assert!(validate_identifier("123invalid", "field").is_err());
        assert!(validate_identifier("invalid-name", "field").is_err());
    }
    #[test]
    fn test_validate_not_empty_collection() {
        assert!(validate_not_empty_collection(&[1, 2, 3], "list").is_ok());
        assert!(validate_not_empty_collection::<i32>(&[], "list").is_err());
    }
    #[test]
    fn test_validate_enum() {
        let allowed = ["a", "b", "c"];
        assert!(validate_enum(&"b", &allowed, "choice").is_ok());
        assert!(validate_enum(&"d", &allowed, "choice").is_err());
    }
    #[test]
    fn test_validate_url() {
        assert!(validate_url("https://example.com", "url").is_ok());
        assert!(validate_url("http://localhost:8080/path", "url").is_ok());
        assert!(validate_url("not a url", "url").is_err());
    }
    #[test]
    fn test_validate_email() {
        assert!(validate_email("user@example.com", "email").is_ok());
        assert!(validate_email("user.name+tag@example.co.uk", "email").is_ok());
        assert!(validate_email("invalid", "email").is_err());
        assert!(validate_email("@example.com", "email").is_err());
    }
    #[test]
    fn test_validate_json_schema() {
        use serde_json::json;

        let schema = json!({
            "type": "object",
            "properties": {
                "name": {"type": "string"},
                "age": {"type": "number", "minimum": 0}
            },
            "required": ["name"]
        });

        let valid_data = json!({"name": "Alice", "age": 30});
        let invalid_data = json!({"age": -5});

        assert!(validate_json_schema(&valid_data, &schema).is_ok());
        assert!(validate_json_schema(&invalid_data, &schema).is_err());
    }
    #[test]
    fn test_validate_regex_pattern() {
        assert!(validate_regex_pattern(r"\d+").is_ok());
        assert!(validate_regex_pattern(r"[a-z]+").is_ok());
        assert!(validate_regex_pattern(r"(unclosed").is_err());
        assert!(validate_regex_pattern(r"(?P<invalid)").is_err());
    }
    #[test]
    fn test_validate_date_format() {
        assert!(validate_date_format("2024-01-15 14:30:00", "%Y-%m-%d %H:%M:%S").is_ok());
        assert!(validate_date_format("15/01/2024", "%d/%m/%Y").is_ok());
        assert!(validate_date_format("invalid date", "%Y-%m-%d").is_err());
        assert!(validate_date_format("2024-01-15", "%d/%m/%Y").is_err());
    }
    #[test]
    fn test_validate_safe_path() {
        let temp_dir = TempDir::new().unwrap();
        let safe_path = temp_dir.path().join("safe_file.txt");
        let jail_dir = temp_dir.path();

        // Safe paths should pass
        assert!(validate_safe_path(&safe_path, Some(jail_dir)).is_ok());

        // Paths with .. should fail
        let dangerous_path = temp_dir.path().join("../escape.txt");
        assert!(validate_safe_path(&dangerous_path, None).is_err());

        // Paths with ~ should fail
        let home_path = Path::new("~/file.txt");
        assert!(validate_safe_path(home_path, None).is_err());
    }
    #[test]
    fn test_sanitize_string() {
        // Test removing control characters
        assert_eq!(sanitize_string("Hello\x00World", false), "HelloWorld");
        assert_eq!(sanitize_string("Hello\nWorld", false), "HelloWorld");
        assert_eq!(sanitize_string("Hello\nWorld", true), "Hello\nWorld");
        assert_eq!(sanitize_string("Tab\there", true), "Tab\there");
        assert_eq!(sanitize_string("Normal text!", true), "Normal text!");
    }
    #[test]
    fn test_validate_resource_limit() {
        assert!(validate_resource_limit(100, 1000, "memory", "MB").is_ok());
        assert!(validate_resource_limit(2000, 1000, "memory", "MB").is_err());
        assert!(validate_resource_limit(1000, 1000, "file_size", "bytes").is_ok());
    }
    #[test]
    fn test_validate_no_shell_injection() {
        assert!(validate_no_shell_injection("safe_command", "command").is_ok());
        assert!(validate_no_shell_injection("hello world", "input").is_ok());
        assert!(validate_no_shell_injection("rm -rf $HOME", "command").is_err());
        assert!(validate_no_shell_injection("echo `date`", "command").is_err());
        assert!(validate_no_shell_injection("cmd1; cmd2", "command").is_err());
        assert!(validate_no_shell_injection("cmd1 | cmd2", "command").is_err());
    }
    #[test]
    #[cfg(unix)]
    fn test_validate_file_permissions() {
        use std::os::unix::fs::PermissionsExt;

        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        fs::write(&file_path, "test").unwrap();

        // Set specific permissions
        let mut perms = fs::metadata(&file_path).unwrap().permissions();
        perms.set_mode(0o644);
        fs::set_permissions(&file_path, perms).unwrap();

        // Validate correct permissions
        assert!(validate_file_permissions(&file_path, 0o644).is_ok());
        // Validate incorrect permissions
        assert!(validate_file_permissions(&file_path, 0o755).is_err());
    }
}
