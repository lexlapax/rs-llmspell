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
}
