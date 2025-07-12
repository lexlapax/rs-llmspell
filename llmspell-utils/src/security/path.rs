// ABOUTME: Path security utilities for preventing path traversal and symlink attacks
// ABOUTME: Provides comprehensive path validation with jail enforcement and attack detection

//! Path security utilities
//!
//! This module provides enhanced path security validation to prevent:
//! - Path traversal attacks
//! - Symlink attacks
//! - Directory escape attempts
//! - Hidden file access (optional)
//! - System directory access

use llmspell_core::{LLMSpellError, Result as LLMResult};
use std::path::{Path, PathBuf};

/// Configuration for path security validation
#[derive(Debug, Clone)]
pub struct PathSecurityConfig {
    /// Allow access to hidden files (starting with .)
    pub allow_hidden: bool,
    /// Allow following symlinks
    pub allow_symlinks: bool,
    /// Jail directory to restrict access
    pub jail_directory: Option<PathBuf>,
    /// Disallowed path prefixes (e.g., /etc, /sys)
    pub disallowed_prefixes: Vec<PathBuf>,
    /// Maximum path depth allowed
    pub max_depth: Option<usize>,
}

impl Default for PathSecurityConfig {
    fn default() -> Self {
        Self {
            allow_hidden: false,
            allow_symlinks: false,
            jail_directory: None,
            disallowed_prefixes: vec![
                PathBuf::from("/etc"),
                PathBuf::from("/sys"),
                PathBuf::from("/proc"),
                PathBuf::from("/dev"),
                #[cfg(target_os = "windows")]
                PathBuf::from("C:\\Windows\\System32"),
                #[cfg(target_os = "windows")]
                PathBuf::from("C:\\Windows\\SysWOW64"),
            ],
            max_depth: Some(20),
        }
    }
}

impl PathSecurityConfig {
    /// Create a new configuration with default values
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a strict configuration for untrusted input
    #[must_use]
    pub fn strict() -> Self {
        Self {
            allow_hidden: false,
            allow_symlinks: false,
            jail_directory: None,
            disallowed_prefixes: Self::default().disallowed_prefixes,
            max_depth: Some(10),
        }
    }

    /// Create a relaxed configuration for trusted input
    #[must_use]
    pub fn relaxed() -> Self {
        Self {
            allow_hidden: true,
            allow_symlinks: true,
            jail_directory: None,
            disallowed_prefixes: vec![],
            max_depth: Some(50),
        }
    }

    /// Set the jail directory
    #[must_use]
    pub fn with_jail(mut self, jail: PathBuf) -> Self {
        self.jail_directory = Some(jail);
        self
    }
}

/// Enhanced path security validator
#[derive(Debug, Clone)]
pub struct PathSecurityValidator {
    config: PathSecurityConfig,
}

impl PathSecurityValidator {
    /// Create a new validator with default configuration
    #[must_use]
    pub fn new() -> Self {
        Self {
            config: PathSecurityConfig::default(),
        }
    }

    /// Create validator with custom configuration
    #[must_use]
    pub fn with_config(config: PathSecurityConfig) -> Self {
        Self { config }
    }

    /// Validate a path for security concerns
    ///
    /// # Errors
    ///
    /// Returns `LLMSpellError::Validation` if the path fails security checks
    pub fn validate(&self, path: &Path) -> LLMResult<PathBuf> {
        // First, convert to absolute path if relative
        let abs_path = if path.is_absolute() {
            path.to_path_buf()
        } else {
            std::env::current_dir()
                .map_err(|e| LLMSpellError::Validation {
                    message: format!("Failed to get current directory: {e}"),
                    field: Some("path".to_string()),
                })?
                .join(path)
        };

        // Check for path traversal patterns BEFORE normalization
        Self::check_path_traversal(&abs_path)?;

        // Normalize the path to resolve . and .. components
        let normalized = Self::normalize_path_internal(&abs_path)?;

        // Check depth
        self.check_path_depth(&normalized)?;

        // Check for hidden files
        self.check_hidden_files(&normalized)?;

        // Check against disallowed prefixes
        self.check_disallowed_prefixes(&normalized)?;

        // Check jail directory
        self.check_jail_directory(&normalized)?;

        // Check symlinks
        self.check_symlinks(&normalized)?;

        Ok(normalized)
    }

    /// Normalize a path, resolving . and .. components
    fn normalize_path_internal(path: &Path) -> LLMResult<PathBuf> {
        use crate::file_utils::normalize_path;

        let normalized = normalize_path(path);

        // Additional check: ensure the normalized path doesn't contain any .. components
        if normalized
            .components()
            .any(|c| matches!(c, std::path::Component::ParentDir))
        {
            return Err(LLMSpellError::Validation {
                message: "Path normalization failed to resolve all .. components".to_string(),
                field: Some("path".to_string()),
            });
        }

        Ok(normalized)
    }

    /// Check for path traversal attempts
    fn check_path_traversal(path: &Path) -> LLMResult<()> {
        let path_str = path.to_string_lossy();

        // Check for various path traversal patterns
        let dangerous_patterns = [
            "..",
            "..\\",
            "../",
            "%2e%2e",
            "%252e%252e",
            "..%2f",
            "..%5c",
            "..%252f",
            "..%255c",
        ];

        for pattern in &dangerous_patterns {
            if path_str.to_lowercase().contains(pattern) {
                return Err(LLMSpellError::Validation {
                    message: format!("Path contains dangerous pattern: {pattern}"),
                    field: Some("path".to_string()),
                });
            }
        }

        Ok(())
    }

    /// Check path depth
    fn check_path_depth(&self, path: &Path) -> LLMResult<()> {
        if let Some(max_depth) = self.config.max_depth {
            let depth = path.components().count();
            if depth > max_depth {
                return Err(LLMSpellError::Validation {
                    message: format!("Path depth {depth} exceeds maximum allowed {max_depth}"),
                    field: Some("path".to_string()),
                });
            }
        }
        Ok(())
    }

    /// Check for hidden files
    fn check_hidden_files(&self, path: &Path) -> LLMResult<()> {
        if !self.config.allow_hidden {
            for component in path.components() {
                if let std::path::Component::Normal(name) = component {
                    if let Some(name_str) = name.to_str() {
                        if name_str.starts_with('.') && name_str != "." && name_str != ".." {
                            return Err(LLMSpellError::Validation {
                                message: "Access to hidden files is not allowed".to_string(),
                                field: Some("path".to_string()),
                            });
                        }
                    }
                }
            }
        }
        Ok(())
    }

    /// Check against disallowed prefixes
    fn check_disallowed_prefixes(&self, path: &Path) -> LLMResult<()> {
        for prefix in &self.config.disallowed_prefixes {
            if path.starts_with(prefix) {
                return Err(LLMSpellError::Validation {
                    message: format!("Access to {} is not allowed", prefix.display()),
                    field: Some("path".to_string()),
                });
            }
        }
        Ok(())
    }

    /// Check jail directory constraint
    fn check_jail_directory(&self, path: &Path) -> LLMResult<()> {
        if let Some(jail) = &self.config.jail_directory {
            let jail_normalized = Self::normalize_path_internal(jail)?;
            if !path.starts_with(&jail_normalized) {
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
        Ok(())
    }

    /// Check for symlinks
    fn check_symlinks(&self, path: &Path) -> LLMResult<()> {
        if !self.config.allow_symlinks {
            // Check each component of the path for symlinks
            let mut current = PathBuf::new();
            for component in path.components() {
                current.push(component);
                if current.exists() && current.read_link().is_ok() {
                    return Err(LLMSpellError::Validation {
                        message: format!(
                            "Symlink detected at '{}', symlinks are not allowed",
                            current.display()
                        ),
                        field: Some("path".to_string()),
                    });
                }
            }
        }
        Ok(())
    }
}

impl Default for PathSecurityValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_path_traversal_detection() {
        let validator = PathSecurityValidator::new();

        // Various path traversal attempts
        let dangerous_paths = vec![
            "../etc/passwd",
            "../../secret.txt",
            "foo/../../../etc/passwd",
            "foo/..\\..\\windows",
            "%2e%2e/etc/passwd",
        ];

        for path in dangerous_paths {
            let result = validator.validate(Path::new(path));
            assert!(result.is_err(), "Path '{}' should be rejected", path);
        }
    }

    #[test]
    fn test_jail_directory_enforcement() {
        let temp_dir = TempDir::new().unwrap();
        let jail_path = temp_dir.path().to_path_buf();

        let config = PathSecurityConfig::relaxed().with_jail(jail_path.clone());
        let validator = PathSecurityValidator::with_config(config);

        // Safe path within jail
        let safe_path = jail_path.join("subdir/file.txt");
        assert!(validator.validate(&safe_path).is_ok());

        // Path outside jail
        let outside_path = temp_dir.path().parent().unwrap().join("outside.txt");
        assert!(validator.validate(&outside_path).is_err());
    }

    #[test]
    fn test_hidden_file_detection() {
        let strict_validator = PathSecurityValidator::with_config(PathSecurityConfig::strict());
        let relaxed_validator = PathSecurityValidator::with_config(PathSecurityConfig::relaxed());

        let hidden_path = Path::new("/tmp/.hidden_file");

        // Strict should reject
        assert!(strict_validator.validate(hidden_path).is_err());

        // Relaxed should allow
        assert!(relaxed_validator.validate(hidden_path).is_ok());
    }

    #[test]
    fn test_system_directory_protection() {
        let validator = PathSecurityValidator::new();

        let system_paths = vec![
            "/etc/passwd",
            "/sys/kernel/debug",
            "/proc/self/environ",
            "/dev/null",
        ];

        for path in system_paths {
            let result = validator.validate(Path::new(path));
            assert!(result.is_err(), "System path '{}' should be rejected", path);
        }
    }

    #[test]
    fn test_path_depth_limit() {
        let config = PathSecurityConfig {
            max_depth: Some(5),
            ..Default::default()
        };
        let validator = PathSecurityValidator::with_config(config);

        // Path within depth limit (5 components: /, a, b, c, file.txt)
        let shallow_path = Path::new("/a/b/c/file.txt");
        assert!(validator.validate(shallow_path).is_ok());

        // Path exceeding depth limit
        let deep_path = Path::new("/a/b/c/d/e/f/g/h/file.txt");
        assert!(validator.validate(deep_path).is_err());
    }
}
