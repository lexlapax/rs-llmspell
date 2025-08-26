//! ABOUTME: File system access control sandbox
//! ABOUTME: Restricts file operations to allowed paths with path traversal protection

use super::{SandboxContext, SandboxViolation};
use llmspell_core::{error::LLMSpellError, Result};
use std::fs;
use std::path::{Path, PathBuf};
use tokio::fs as async_fs;
use tracing::{debug, warn};

/// File sandbox for controlling file system access
pub struct FileSandbox {
    context: SandboxContext,
    violations: Vec<SandboxViolation>,
}

impl FileSandbox {
    /// Create a new file sandbox
    pub fn new(context: SandboxContext) -> Result<Self> {
        Ok(Self {
            context,
            violations: Vec::new(),
        })
    }

    /// Check if a path is safe (no path traversal, within allowed paths)
    pub fn validate_path(&self, path: &Path) -> Result<PathBuf> {
        // Normalize the path to prevent path traversal
        let normalized = self.normalize_path(path)?;

        // Check if the path is within allowed directories
        if !self.context.is_path_allowed(&normalized) {
            let violation = SandboxViolation::FileAccess {
                path: normalized.to_string_lossy().to_string(),
                operation: "access".to_string(),
                reason: "Path not in allowed list".to_string(),
            };
            warn!("File access violation: {}", violation);
            return Err(LLMSpellError::Security {
                message: violation.to_string(),
                violation_type: Some("file_access".to_string()),
            });
        }

        debug!("Path validation successful: {:?}", normalized);
        Ok(normalized)
    }

    /// Normalize path to prevent traversal attacks
    fn normalize_path(&self, path: &Path) -> Result<PathBuf> {
        // Convert to absolute path
        let absolute = if path.is_absolute() {
            path.to_path_buf()
        } else {
            PathBuf::from(&self.context.working_directory).join(path)
        };

        // Manually resolve .. and . components for security validation
        let mut components = Vec::new();
        for component in absolute.components() {
            match component {
                std::path::Component::Normal(name) => {
                    components.push(name);
                }
                std::path::Component::ParentDir => {
                    if components.is_empty() {
                        return Err(LLMSpellError::Security {
                            message: "Path traversal detected: cannot go above root".to_string(),
                            violation_type: Some("path_traversal".to_string()),
                        });
                    }
                    components.pop();
                }
                std::path::Component::CurDir => {
                    // Ignore current directory references
                }
                std::path::Component::RootDir | std::path::Component::Prefix(_) => {
                    // Keep root and prefix components
                    components.clear();
                    if let std::path::Component::Prefix(_) = component {
                        components.push(component.as_os_str());
                    }
                }
            }
        }

        // Reconstruct the normalized path
        let mut normalized = PathBuf::new();
        normalized.push("/"); // Start with root
        for component in components {
            normalized.push(component);
        }

        // Final check for any remaining traversal patterns
        let path_str = normalized.to_string_lossy();
        if path_str.contains("..") || path_str.contains("./") {
            return Err(LLMSpellError::Security {
                message: "Path traversal pattern detected".to_string(),
                violation_type: Some("path_traversal".to_string()),
            });
        }

        Ok(normalized)
    }

    /// Safe file read operation
    pub async fn read_file(&mut self, path: &Path) -> Result<Vec<u8>> {
        let safe_path = self.validate_path(path)?;

        debug!("Reading file: {:?}", safe_path);

        async_fs::read(&safe_path).await.map_err(|e| {
            let violation = SandboxViolation::FileAccess {
                path: safe_path.to_string_lossy().to_string(),
                operation: "read".to_string(),
                reason: format!("IO error: {}", e),
            };
            self.violations.push(violation.clone());
            LLMSpellError::Security {
                message: violation.to_string(),
                violation_type: Some("file_read".to_string()),
            }
        })
    }

    /// Safe file write operation
    pub async fn write_file(&mut self, path: &Path, contents: &[u8]) -> Result<()> {
        let safe_path = self.validate_path(path)?;

        debug!("Writing file: {:?}", safe_path);

        // Ensure parent directory exists
        if let Some(parent) = safe_path.parent() {
            if !parent.exists() {
                async_fs::create_dir_all(parent)
                    .await
                    .map_err(|e| LLMSpellError::Security {
                        message: format!("Cannot create parent directory: {}", e),
                        violation_type: Some("directory_creation".to_string()),
                    })?;
            }
        }

        async_fs::write(&safe_path, contents).await.map_err(|e| {
            let violation = SandboxViolation::FileAccess {
                path: safe_path.to_string_lossy().to_string(),
                operation: "write".to_string(),
                reason: format!("IO error: {}", e),
            };
            self.violations.push(violation.clone());
            LLMSpellError::Security {
                message: violation.to_string(),
                violation_type: Some("file_write".to_string()),
            }
        })
    }

    /// Safe file append operation
    pub async fn append_file(&mut self, path: &Path, contents: &[u8]) -> Result<()> {
        let safe_path = self.validate_path(path)?;

        debug!("Appending to file: {:?}", safe_path);

        // Read existing content
        let mut existing = if safe_path.exists() {
            self.read_file(&safe_path).await?
        } else {
            Vec::new()
        };

        // Append new content
        existing.extend_from_slice(contents);

        // Write back
        self.write_file(&safe_path, &existing).await
    }

    /// Safe directory creation
    pub async fn create_dir(&mut self, path: &Path) -> Result<()> {
        let safe_path = self.validate_path(path)?;

        debug!("Creating directory: {:?}", safe_path);

        async_fs::create_dir_all(&safe_path).await.map_err(|e| {
            let violation = SandboxViolation::FileAccess {
                path: safe_path.to_string_lossy().to_string(),
                operation: "create_dir".to_string(),
                reason: format!("IO error: {}", e),
            };
            self.violations.push(violation.clone());
            LLMSpellError::Security {
                message: violation.to_string(),
                violation_type: Some("directory_creation".to_string()),
            }
        })
    }

    /// Safe file deletion
    pub async fn delete_file(&mut self, path: &Path) -> Result<()> {
        let safe_path = self.validate_path(path)?;

        debug!("Deleting file: {:?}", safe_path);

        async_fs::remove_file(&safe_path).await.map_err(|e| {
            let violation = SandboxViolation::FileAccess {
                path: safe_path.to_string_lossy().to_string(),
                operation: "delete".to_string(),
                reason: format!("IO error: {}", e),
            };
            self.violations.push(violation.clone());
            LLMSpellError::Security {
                message: violation.to_string(),
                violation_type: Some("file_delete".to_string()),
            }
        })
    }

    /// Safe directory listing
    pub async fn list_dir(&mut self, path: &Path) -> Result<Vec<PathBuf>> {
        let safe_path = self.validate_path(path)?;

        debug!("Listing directory: {:?}", safe_path);

        let mut entries = Vec::new();
        let mut read_dir = async_fs::read_dir(&safe_path).await.map_err(|e| {
            let violation = SandboxViolation::FileAccess {
                path: safe_path.to_string_lossy().to_string(),
                operation: "list".to_string(),
                reason: format!("IO error: {}", e),
            };
            self.violations.push(violation.clone());
            LLMSpellError::Security {
                message: violation.to_string(),
                violation_type: Some("directory_list".to_string()),
            }
        })?;

        while let Some(entry) =
            read_dir
                .next_entry()
                .await
                .map_err(|e| LLMSpellError::Security {
                    message: format!("Error reading directory entry: {}", e),
                    violation_type: Some("directory_list".to_string()),
                })?
        {
            entries.push(entry.path());
        }

        Ok(entries)
    }

    /// Check if file exists
    pub async fn file_exists(&mut self, path: &Path) -> Result<bool> {
        let safe_path = self.validate_path(path)?;
        Ok(safe_path.exists())
    }

    /// Get file metadata
    pub async fn file_metadata(&mut self, path: &Path) -> Result<fs::Metadata> {
        let safe_path = self.validate_path(path)?;

        debug!("Getting metadata for: {:?}", safe_path);

        async_fs::metadata(&safe_path).await.map_err(|e| {
            let violation = SandboxViolation::FileAccess {
                path: safe_path.to_string_lossy().to_string(),
                operation: "metadata".to_string(),
                reason: format!("IO error: {}", e),
            };
            self.violations.push(violation.clone());
            LLMSpellError::Security {
                message: violation.to_string(),
                violation_type: Some("file_metadata".to_string()),
            }
        })
    }

    /// Get all violations that occurred
    pub fn get_violations(&self) -> &[SandboxViolation] {
        &self.violations
    }

    /// Clear violations history
    pub fn clear_violations(&mut self) {
        self.violations.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use llmspell_core::traits::tool::{ResourceLimits, SecurityRequirements};
    use std::path::PathBuf;
    use tempfile::TempDir;

    async fn create_test_sandbox() -> (FileSandbox, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path().to_string_lossy().to_string();

        let security_reqs = SecurityRequirements::safe().with_file_access(&temp_path);

        let context = SandboxContext::new(
            "test-sandbox".to_string(),
            security_reqs,
            ResourceLimits::strict(),
        );

        let sandbox = FileSandbox::new(context).unwrap();
        (sandbox, temp_dir)
    }
    #[tokio::test]
    async fn test_file_operations() {
        let (mut sandbox, temp_dir) = create_test_sandbox().await;
        let test_file = temp_dir.path().join("test.txt");

        // Test write
        let content = b"Hello, sandbox!";
        sandbox.write_file(&test_file, content).await.unwrap();

        // Test read
        let read_content = sandbox.read_file(&test_file).await.unwrap();
        assert_eq!(read_content, content);

        // Test append
        let append_content = b" More content.";
        sandbox
            .append_file(&test_file, append_content)
            .await
            .unwrap();

        let full_content = sandbox.read_file(&test_file).await.unwrap();
        assert_eq!(full_content, b"Hello, sandbox! More content.");

        // Test file exists
        assert!(sandbox.file_exists(&test_file).await.unwrap());

        // Test metadata
        let metadata = sandbox.file_metadata(&test_file).await.unwrap();
        assert!(metadata.is_file());
    }
    #[tokio::test]
    async fn test_directory_operations() {
        let (mut sandbox, temp_dir) = create_test_sandbox().await;
        let test_dir = temp_dir.path().join("subdir");

        // Test directory creation
        sandbox.create_dir(&test_dir).await.unwrap();
        assert!(test_dir.exists());

        // Test directory listing
        let entries = sandbox.list_dir(temp_dir.path()).await.unwrap();
        assert!(entries.contains(&test_dir));
    }
    #[tokio::test]
    async fn test_path_traversal_protection() {
        let (mut sandbox, _temp_dir) = create_test_sandbox().await;

        // Test path traversal attempt - create a path that goes above the allowed directory
        // This should be caught as path traversal since it tries to go above the allowed root
        let traversal_path = Path::new("../../../../etc/passwd");
        let result = sandbox.read_file(traversal_path).await;
        assert!(result.is_err());

        // The actual behavior: our normalization resolves to /etc/passwd which fails as unauthorized access
        // This is actually correct security behavior - the path should not be allowed
        match result.unwrap_err() {
            LLMSpellError::Security { violation_type, .. } => {
                // Path normalization correctly prevents traversal and results in unauthorized access
                assert!(
                    violation_type == Some("file_access".to_string())
                        || violation_type == Some("path_traversal".to_string())
                );
            }
            _ => panic!("Expected Security error"),
        }
    }
    #[tokio::test]
    async fn test_unauthorized_path_access() {
        let (mut sandbox, _temp_dir) = create_test_sandbox().await;

        // Try to access a path outside the allowed directory
        let unauthorized_path = PathBuf::from("/etc/passwd");
        let result = sandbox.read_file(&unauthorized_path).await;
        assert!(result.is_err());

        // Should be a security violation
        match result.unwrap_err() {
            LLMSpellError::Security { violation_type, .. } => {
                assert_eq!(violation_type, Some("file_access".to_string()));
            }
            _ => panic!("Expected SecurityViolation"),
        }
    }
    #[tokio::test]
    async fn test_file_deletion() {
        let (mut sandbox, temp_dir) = create_test_sandbox().await;
        let test_file = temp_dir.path().join("delete_me.txt");

        // Create file
        sandbox.write_file(&test_file, b"Delete me").await.unwrap();
        assert!(sandbox.file_exists(&test_file).await.unwrap());

        // Delete file
        sandbox.delete_file(&test_file).await.unwrap();
        assert!(!sandbox.file_exists(&test_file).await.unwrap());
    }
}
