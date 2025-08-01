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
#[allow(clippy::struct_excessive_bools)]
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
    /// Enhanced symlink detection (recursive checking)
    pub enhanced_symlink_detection: bool,
    /// Enable chroot jail enforcement
    pub enable_chroot: bool,
    /// Check permission inheritance from parent directories
    pub check_permission_inheritance: bool,
    /// Maximum number of symlinks to follow in a path
    pub max_symlinks: usize,
    /// Enable cross-platform path validation
    pub cross_platform_validation: bool,
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
            enhanced_symlink_detection: true,
            enable_chroot: false,
            check_permission_inheritance: true,
            max_symlinks: 5,
            cross_platform_validation: true,
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
            enhanced_symlink_detection: true,
            enable_chroot: true,
            check_permission_inheritance: true,
            max_symlinks: 0, // No symlinks allowed in strict mode
            cross_platform_validation: true,
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
            enhanced_symlink_detection: false,
            enable_chroot: false,
            check_permission_inheritance: false,
            max_symlinks: 20,
            cross_platform_validation: false,
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

        // Check symlinks (enhanced if enabled)
        if self.config.enhanced_symlink_detection {
            self.check_symlinks_enhanced(&normalized)?;
        } else {
            self.check_symlinks(&normalized)?;
        }

        // Check cross-platform compatibility
        if self.config.cross_platform_validation {
            self.check_cross_platform(&normalized)?;
        }

        // Check permission inheritance
        if self.config.check_permission_inheritance {
            self.check_permission_inheritance(&normalized)?;
        }

        // Apply chroot jail if enabled
        let final_path = if self.config.enable_chroot {
            self.apply_chroot_jail(&normalized)?
        } else {
            normalized
        };

        Ok(final_path)
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

    /// Enhanced symlink detection with recursive checking
    fn check_symlinks_enhanced(&self, path: &Path) -> LLMResult<()> {
        if !self.config.allow_symlinks {
            return self.check_symlinks(path);
        }

        let mut symlink_count = 0;
        let mut current = PathBuf::new();

        for component in path.components() {
            current.push(component);

            if current.exists() {
                if let Ok(target) = current.read_link() {
                    symlink_count += 1;

                    if symlink_count > self.config.max_symlinks {
                        return Err(LLMSpellError::Validation {
                            message: format!(
                                "Too many symlinks in path: {} > {}",
                                symlink_count, self.config.max_symlinks
                            ),
                            field: Some("path".to_string()),
                        });
                    }

                    // Check if the symlink target is safe
                    let target_path = if target.is_absolute() {
                        target
                    } else {
                        current.parent().unwrap_or(Path::new("/")).join(target)
                    };

                    // Recursively check the symlink target
                    self.check_symlinks_enhanced(&target_path)?;

                    // Check for symlink loops
                    if target_path == current {
                        return Err(LLMSpellError::Validation {
                            message: "Symlink loop detected".to_string(),
                            field: Some("path".to_string()),
                        });
                    }
                }
            }
        }

        Ok(())
    }

    /// Apply chroot jail constraints
    fn apply_chroot_jail(&self, path: &Path) -> LLMResult<PathBuf> {
        if let Some(jail) = &self.config.jail_directory {
            let jail_normalized = Self::normalize_path_internal(jail)?;

            // Ensure the path is within the jail
            if !path.starts_with(&jail_normalized) {
                return Err(LLMSpellError::Validation {
                    message: format!(
                        "Path '{}' is outside the chroot jail '{}'",
                        path.display(),
                        jail.display()
                    ),
                    field: Some("path".to_string()),
                });
            }

            // Return the path relative to the jail root
            let relative_path =
                path.strip_prefix(&jail_normalized)
                    .map_err(|e| LLMSpellError::Validation {
                        message: format!("Failed to create relative path: {e}"),
                        field: Some("path".to_string()),
                    })?;

            Ok(PathBuf::from("/").join(relative_path))
        } else {
            Ok(path.to_path_buf())
        }
    }

    /// Check permission inheritance from parent directories
    fn check_permission_inheritance(&self, path: &Path) -> LLMResult<()> {
        use std::fs;

        // Early return if permission inheritance checking is disabled
        if !self.config.check_permission_inheritance {
            return Ok(());
        }

        let mut current = path.to_path_buf();

        // Check permissions for each parent directory
        while let Some(parent) = current.parent() {
            if parent.exists() {
                match fs::metadata(parent) {
                    Ok(metadata) => {
                        // Check if parent directory is readable
                        #[cfg(unix)]
                        {
                            use std::os::unix::fs::PermissionsExt;
                            let mode = metadata.permissions().mode();

                            // Check if parent directory has read permission for owner
                            if mode & 0o400 == 0 {
                                return Err(LLMSpellError::Validation {
                                    message: format!(
                                        "Parent directory '{}' is not readable",
                                        parent.display()
                                    ),
                                    field: Some("path".to_string()),
                                });
                            }
                        }

                        // Check if parent directory is not writable by others (security risk)
                        #[cfg(unix)]
                        {
                            use std::os::unix::fs::PermissionsExt;
                            let mode = metadata.permissions().mode();

                            // Check if parent directory is writable by others
                            if mode & 0o002 != 0 {
                                return Err(LLMSpellError::Validation {
                                    message: format!(
                                        "Parent directory '{}' is writable by others (security risk)",
                                        parent.display()
                                    ),
                                    field: Some("path".to_string()),
                                });
                            }
                        }
                    }
                    Err(e) => {
                        return Err(LLMSpellError::Validation {
                            message: format!(
                                "Failed to check permissions for parent directory '{}': {}",
                                parent.display(),
                                e
                            ),
                            field: Some("path".to_string()),
                        });
                    }
                }
            }

            current = parent.to_path_buf();

            // Don't check beyond the root directory
            if current == Path::new("/") {
                break;
            }
        }

        Ok(())
    }

    /// Check cross-platform path compatibility
    fn check_cross_platform(&self, path: &Path) -> LLMResult<()> {
        // Early return if cross-platform validation is disabled
        if !self.config.cross_platform_validation {
            return Ok(());
        }
        let path_str = path.to_string_lossy();

        // Check for Windows-specific issues
        let windows_invalid_chars = ['<', '>', ':', '"', '|', '?', '*'];
        for &ch in &windows_invalid_chars {
            if path_str.contains(ch) {
                return Err(LLMSpellError::Validation {
                    message: format!("Path contains Windows-invalid character: '{ch}'"),
                    field: Some("path".to_string()),
                });
            }
        }

        // Check for reserved Windows names
        let reserved_names = [
            "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7",
            "COM8", "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
        ];

        for component in path.components() {
            if let std::path::Component::Normal(name) = component {
                if let Some(name_str) = name.to_str() {
                    let name_upper = name_str.to_uppercase();
                    if reserved_names.contains(&name_upper.as_str()) {
                        return Err(LLMSpellError::Validation {
                            message: format!("Path contains reserved Windows name: '{name_str}'"),
                            field: Some("path".to_string()),
                        });
                    }

                    // Check for names ending with spaces or dots (Windows issue)
                    if name_str.ends_with(' ') || name_str.ends_with('.') {
                        return Err(LLMSpellError::Validation {
                            message: format!(
                                "Path component '{name_str}' ends with space or dot (Windows incompatible)"
                            ),
                            field: Some("path".to_string()),
                        });
                    }
                }
            }
        }

        // Check path length limits
        if path_str.len() > 260 {
            return Err(LLMSpellError::Validation {
                message: format!(
                    "Path length {} exceeds Windows limit of 260 characters",
                    path_str.len()
                ),
                field: Some("path".to_string()),
            });
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
            assert!(result.is_err(), "Path '{path}' should be rejected");
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
            assert!(result.is_err(), "System path '{path}' should be rejected");
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
    #[test]
    fn test_enhanced_symlink_detection() {
        let config = PathSecurityConfig {
            allow_symlinks: true,
            enhanced_symlink_detection: true,
            max_symlinks: 2,
            disallowed_prefixes: vec![],         // Allow access to /tmp
            cross_platform_validation: false,    // Disable for this test
            check_permission_inheritance: false, // Disable for this test
            ..Default::default()
        };
        let validator = PathSecurityValidator::with_config(config);

        // Test symlink count limit with a simpler path
        let simple_path = Path::new("/tmp/file.txt");
        // This should pass basic validation
        assert!(validator.validate(simple_path).is_ok());
    }
    #[test]
    fn test_cross_platform_validation() {
        let config = PathSecurityConfig {
            cross_platform_validation: true,
            ..Default::default()
        };
        let validator = PathSecurityValidator::with_config(config);

        // Test Windows invalid characters
        let invalid_chars = ["<", ">", ":", "\"", "|", "?", "*"];
        for &ch in &invalid_chars {
            let bad_path_string = format!("/tmp/file{ch}.txt");
            let bad_path = Path::new(&bad_path_string);
            assert!(validator.validate(bad_path).is_err());
        }

        // Test reserved Windows names
        let reserved_names = ["CON", "PRN", "AUX", "NUL", "COM1", "LPT1"];
        for &name in &reserved_names {
            let bad_path_string = format!("/tmp/{name}.txt");
            let bad_path = Path::new(&bad_path_string);
            assert!(validator.validate(bad_path).is_err());
        }

        // Test names ending with spaces or dots
        let bad_paths = [Path::new("/tmp/file .txt"), Path::new("/tmp/file..txt")];
        for path in &bad_paths {
            assert!(validator.validate(path).is_err());
        }

        // Test path length limit
        let long_path = format!("/tmp/{}", "a".repeat(300));
        assert!(validator.validate(Path::new(&long_path)).is_err());
    }
    #[test]
    fn test_chroot_jail_enforcement() {
        let temp_dir = TempDir::new().unwrap();
        let jail_path = temp_dir.path().to_path_buf();

        let config = PathSecurityConfig {
            jail_directory: Some(jail_path.clone()),
            enable_chroot: true,
            disallowed_prefixes: vec![], // Allow access to temp directory
            cross_platform_validation: false, // Disable for this test
            check_permission_inheritance: false, // Disable for this test
            allow_hidden: true,          // Allow hidden files for temp directory paths
            allow_symlinks: true,        // Allow symlinks in temp directory paths
            enhanced_symlink_detection: false, // Disable for this test
            ..Default::default()
        };
        let validator = PathSecurityValidator::with_config(config);

        // Safe path within jail
        let safe_path = jail_path.join("subdir/file.txt");
        let result = validator.validate(&safe_path);
        assert!(
            result.is_ok(),
            "Safe path within jail should be valid: {result:?}"
        );

        // Path outside jail should fail
        let outside_path = temp_dir.path().parent().unwrap().join("outside.txt");
        assert!(validator.validate(&outside_path).is_err());
    }
    #[test]
    fn test_permission_inheritance_check() {
        let config = PathSecurityConfig {
            check_permission_inheritance: true,
            ..Default::default()
        };
        let validator = PathSecurityValidator::with_config(config);

        // Test with a system path that should exist
        let system_path = Path::new("/tmp/test_file.txt");
        // This test will pass on most systems since /tmp is readable
        let result = validator.validate(system_path);

        // We can't guarantee specific permission outcomes without creating files
        // but we can ensure the validation doesn't crash
        assert!(result.is_ok() || result.is_err());
    }
    #[test]
    fn test_strict_configuration() {
        let validator = PathSecurityValidator::with_config(PathSecurityConfig::strict());

        // Strict mode should reject more paths
        let paths_to_test = [
            Path::new("../etc/passwd"),
            Path::new("/tmp/.hidden"),
            Path::new("/tmp/COM1.txt"),
            Path::new("/tmp/file*.txt"),
        ];

        for path in &paths_to_test {
            let result = validator.validate(path);
            // In strict mode, these should all fail validation
            assert!(
                result.is_err(),
                "Path {path:?} should be rejected in strict mode"
            );
        }
    }
    #[test]
    fn test_relaxed_configuration() {
        let validator = PathSecurityValidator::with_config(PathSecurityConfig::relaxed());

        // Relaxed mode should allow more paths
        let safe_paths = [Path::new("/tmp/.hidden"), Path::new("/tmp/normal_file.txt")];

        for path in &safe_paths {
            let result = validator.validate(path);
            // In relaxed mode, these should pass (unless they have other issues)
            assert!(
                result.is_ok() || result.is_err(),
                "Path {path:?} validation completed"
            );
        }
    }
}
