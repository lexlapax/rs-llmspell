//! ABOUTME: Safe file system operations tool with sandboxing and atomic writes
//! ABOUTME: Provides read, write, and directory operations with path traversal protection

use async_trait::async_trait;
use llmspell_core::{
    traits::{
        base_agent::BaseAgent,
        tool::{
            ParameterDef, ParameterType, ResourceLimits, SecurityLevel, SecurityRequirements, Tool,
            ToolCategory, ToolSchema,
        },
    },
    types::{AgentInput, AgentOutput, ExecutionContext},
    ComponentMetadata, LLMSpellError, Result,
};
use llmspell_security::sandbox::{FileSandbox, SandboxContext};
use llmspell_utils::file_utils;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tracing::{debug, info, warn};

/// File operation types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FileOperation {
    /// Read file contents
    Read,
    /// Write file contents
    Write,
    /// Append to file
    Append,
    /// Delete file
    Delete,
    /// Create directory
    CreateDir,
    /// List directory contents
    ListDir,
    /// Copy file
    Copy,
    /// Move/rename file
    Move,
    /// Get file metadata
    Metadata,
    /// Check if file exists
    Exists,
}

impl std::fmt::Display for FileOperation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FileOperation::Read => write!(f, "read"),
            FileOperation::Write => write!(f, "write"),
            FileOperation::Append => write!(f, "append"),
            FileOperation::Delete => write!(f, "delete"),
            FileOperation::CreateDir => write!(f, "create_dir"),
            FileOperation::ListDir => write!(f, "list_dir"),
            FileOperation::Copy => write!(f, "copy"),
            FileOperation::Move => write!(f, "move"),
            FileOperation::Metadata => write!(f, "metadata"),
            FileOperation::Exists => write!(f, "exists"),
        }
    }
}

impl std::str::FromStr for FileOperation {
    type Err = LLMSpellError;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "read" => Ok(FileOperation::Read),
            "write" => Ok(FileOperation::Write),
            "append" => Ok(FileOperation::Append),
            "delete" => Ok(FileOperation::Delete),
            "create_dir" | "mkdir" => Ok(FileOperation::CreateDir),
            "list_dir" | "ls" | "dir" => Ok(FileOperation::ListDir),
            "copy" | "cp" => Ok(FileOperation::Copy),
            "move" | "mv" | "rename" => Ok(FileOperation::Move),
            "metadata" | "stat" | "info" => Ok(FileOperation::Metadata),
            "exists" => Ok(FileOperation::Exists),
            _ => Err(LLMSpellError::Validation {
                message: format!("Unknown file operation: {}", s),
                field: Some("operation".to_string()),
            }),
        }
    }
}

/// File operations configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileOperationsConfig {
    /// Enable atomic writes
    pub atomic_writes: bool,
    /// Maximum file size for operations (in bytes)
    pub max_file_size: usize,
    /// Maximum directory listing size
    pub max_dir_entries: usize,
    /// Enable recursive directory operations
    pub allow_recursive: bool,
    /// Default file permissions for new files
    pub default_permissions: u32,
}

impl Default for FileOperationsConfig {
    fn default() -> Self {
        Self {
            atomic_writes: true,
            max_file_size: 100 * 1024 * 1024, // 100MB
            max_dir_entries: 10000,
            allow_recursive: false,
            default_permissions: 0o644,
        }
    }
}

/// File operations tool
pub struct FileOperationsTool {
    metadata: ComponentMetadata,
    config: FileOperationsConfig,
}

impl FileOperationsTool {
    pub fn new(config: FileOperationsConfig) -> Self {
        Self {
            metadata: ComponentMetadata::new(
                "file-operations-tool".to_string(),
                "Safe file system operations with sandboxing".to_string(),
            ),
            config,
        }
    }

    /// Create a file sandbox from context
    fn create_sandbox(&self, context: &ExecutionContext) -> Result<FileSandbox> {
        let sandbox_context = SandboxContext::new(
            format!(
                "file_ops_{}",
                context.session_id.as_deref().unwrap_or("default")
            ),
            self.security_requirements(),
            self.resource_limits(),
        );

        FileSandbox::new(sandbox_context)
    }

    /// Perform read operation
    async fn read_file(&self, path: &Path, sandbox: &FileSandbox) -> Result<String> {
        info!("Reading file: {:?}", path);

        // Validate path
        let safe_path = sandbox.validate_path(path)?;

        // Check file size before reading
        let metadata =
            file_utils::get_metadata(&safe_path).map_err(|e| LLMSpellError::Storage {
                message: format!(
                    "Failed to access path: {} - {}",
                    safe_path.to_string_lossy(),
                    e
                ),
                operation: Some("metadata".to_string()),
                source: None,
            })?;

        if metadata.size as usize > self.config.max_file_size {
            return Err(LLMSpellError::Validation {
                message: format!(
                    "File size {} exceeds maximum allowed size {}",
                    metadata.size, self.config.max_file_size
                ),
                field: Some("file_size".to_string()),
            });
        }

        // Read file using file_utils
        let content = file_utils::read_file(&safe_path).map_err(|e| LLMSpellError::Storage {
            message: format!(
                "Failed to access path: {} - {}",
                safe_path.to_string_lossy(),
                e
            ),
            operation: Some("read".to_string()),
            source: None,
        })?;

        // Convert bytes to string
        String::from_utf8(content).map_err(|e| LLMSpellError::Storage {
            message: format!(
                "File contains invalid UTF-8: {} - {}",
                safe_path.to_string_lossy(),
                e
            ),
            operation: Some("read".to_string()),
            source: None,
        })
    }

    /// Perform write operation with optional atomic write
    async fn write_file(&self, path: &Path, content: &str, sandbox: &FileSandbox) -> Result<()> {
        info!("Writing file: {:?}", path);

        // Validate path
        let safe_path = sandbox.validate_path(path)?;

        // Check content size
        if content.len() > self.config.max_file_size {
            return Err(LLMSpellError::Validation {
                message: format!(
                    "Content size {} exceeds maximum allowed size {}",
                    content.len(),
                    self.config.max_file_size
                ),
                field: Some("content_size".to_string()),
            });
        }

        // Use file_utils for atomic or regular write
        let write_fn = if self.config.atomic_writes {
            file_utils::write_file_atomic
        } else {
            file_utils::write_file
        };

        write_fn(&safe_path, content.as_bytes()).map_err(|e| LLMSpellError::Storage {
            message: format!(
                "Failed to access path: {} - {}",
                safe_path.to_string_lossy(),
                e
            ),
            operation: Some("write".to_string()),
            source: None,
        })
    }

    /// Perform append operation
    async fn append_file(&self, path: &Path, content: &str, sandbox: &FileSandbox) -> Result<()> {
        info!("Appending to file: {:?}", path);

        // Validate path
        let safe_path = sandbox.validate_path(path)?;

        // Read existing content first to check total size
        let existing_size = if file_utils::file_exists(&safe_path) {
            file_utils::get_metadata(&safe_path)
                .map(|m| m.size as usize)
                .unwrap_or(0)
        } else {
            0
        };

        let total_size = existing_size + content.len();
        if total_size > self.config.max_file_size {
            return Err(LLMSpellError::Validation {
                message: format!(
                    "Total size {} would exceed maximum allowed size {}",
                    total_size, self.config.max_file_size
                ),
                field: Some("total_size".to_string()),
            });
        }

        // Append content using file_utils
        file_utils::append_file(&safe_path, content.as_bytes()).map_err(|e| {
            LLMSpellError::Storage {
                message: format!(
                    "Failed to access path: {} - {}",
                    safe_path.to_string_lossy(),
                    e
                ),
                operation: Some("append".to_string()),
                source: None,
            }
        })
    }

    /// Delete file
    async fn delete_file(&self, path: &Path, sandbox: &FileSandbox) -> Result<()> {
        info!("Deleting file: {:?}", path);

        // Validate path
        let safe_path = sandbox.validate_path(path)?;

        // Check if it's a file
        let metadata =
            file_utils::get_metadata(&safe_path).map_err(|e| LLMSpellError::Storage {
                message: format!(
                    "Failed to access path: {} - {}",
                    safe_path.to_string_lossy(),
                    e
                ),
                operation: Some("metadata".to_string()),
                source: None,
            })?;

        if metadata.is_dir {
            return Err(LLMSpellError::Validation {
                message: "Cannot delete directory with delete operation, use remove_dir"
                    .to_string(),
                field: Some("path".to_string()),
            });
        }

        file_utils::remove_file_if_exists(&safe_path).map_err(|e| LLMSpellError::Storage {
            message: format!(
                "Failed to access path: {} - {}",
                safe_path.to_string_lossy(),
                e
            ),
            operation: Some("delete".to_string()),
            source: None,
        })
    }

    /// Create directory
    async fn create_dir(&self, path: &Path, recursive: bool, sandbox: &FileSandbox) -> Result<()> {
        info!("Creating directory: {:?} (recursive: {})", path, recursive);

        // Validate path
        let safe_path = sandbox.validate_path(path)?;

        if recursive && self.config.allow_recursive {
            // ensure_dir creates parent directories as needed
            file_utils::ensure_dir(&safe_path).map_err(|e| LLMSpellError::Storage {
                message: format!(
                    "Failed to access path: {} - {}",
                    safe_path.to_string_lossy(),
                    e
                ),
                operation: Some("create_dir_all".to_string()),
                source: None,
            })
        } else {
            // For non-recursive, check parent exists first
            if let Some(parent) = safe_path.parent() {
                if !file_utils::file_exists(parent) {
                    return Err(LLMSpellError::Storage {
                        message: format!(
                            "Parent directory does not exist: {}",
                            parent.to_string_lossy()
                        ),
                        operation: Some("create_dir".to_string()),
                        source: None,
                    });
                }
            }

            file_utils::ensure_dir(&safe_path).map_err(|e| LLMSpellError::Storage {
                message: format!(
                    "Failed to access path: {} - {}",
                    safe_path.to_string_lossy(),
                    e
                ),
                operation: Some("create_dir".to_string()),
                source: None,
            })
        }
    }

    /// List directory contents
    async fn list_dir(&self, path: &Path, sandbox: &FileSandbox) -> Result<Vec<Value>> {
        info!("Listing directory: {:?}", path);

        // Validate path
        let safe_path = sandbox.validate_path(path)?;

        // Use file_utils to list directory
        let dir_entries = file_utils::list_dir(&safe_path).map_err(|e| LLMSpellError::Storage {
            message: format!(
                "Failed to access path: {} - {}",
                safe_path.to_string_lossy(),
                e
            ),
            operation: Some("list_dir".to_string()),
            source: None,
        })?;

        // Convert to JSON format, respecting max entries
        let mut entries = Vec::new();
        for (i, entry) in dir_entries.iter().enumerate() {
            if i >= self.config.max_dir_entries {
                warn!("Directory listing truncated at {} entries", i);
                break;
            }

            let file_type = if entry.is_dir {
                "directory"
            } else if entry.is_symlink {
                "symlink"
            } else if entry.is_file {
                "file"
            } else {
                "unknown"
            };

            entries.push(json!({
                "name": entry.name,
                "path": entry.path.to_string_lossy(),
                "type": file_type,
                "size": entry.size,
                "modified": entry.modified.map(|t| {
                    chrono::DateTime::<chrono::Utc>::from(t).to_rfc3339()
                }),
            }));
        }

        Ok(entries)
    }

    /// Copy file
    async fn copy_file(&self, from: &Path, to: &Path, sandbox: &FileSandbox) -> Result<()> {
        info!("Copying file from {:?} to {:?}", from, to);

        // Validate both paths
        let safe_from = sandbox.validate_path(from)?;
        let safe_to = sandbox.validate_path(to)?;

        // Check source file size
        let metadata =
            file_utils::get_metadata(&safe_from).map_err(|e| LLMSpellError::Storage {
                message: format!(
                    "Failed to access source path: {} - {}",
                    safe_from.to_string_lossy(),
                    e
                ),
                operation: Some("metadata".to_string()),
                source: None,
            })?;

        if metadata.size as usize > self.config.max_file_size {
            return Err(LLMSpellError::Validation {
                message: format!(
                    "File size {} exceeds maximum allowed size {}",
                    metadata.size, self.config.max_file_size
                ),
                field: Some("file_size".to_string()),
            });
        }

        file_utils::copy_file(&safe_from, &safe_to).map_err(|e| LLMSpellError::Storage {
            message: format!(
                "Failed to access source path: {} - {}",
                safe_from.to_string_lossy(),
                e
            ),
            operation: Some("copy".to_string()),
            source: None,
        })?;

        Ok(())
    }

    /// Move/rename file
    async fn move_file(&self, from: &Path, to: &Path, sandbox: &FileSandbox) -> Result<()> {
        info!("Moving file from {:?} to {:?}", from, to);

        // Validate both paths
        let safe_from = sandbox.validate_path(from)?;
        let safe_to = sandbox.validate_path(to)?;

        file_utils::move_file(&safe_from, &safe_to).map_err(|e| LLMSpellError::Storage {
            message: format!(
                "Failed to access source path: {} - {}",
                safe_from.to_string_lossy(),
                e
            ),
            operation: Some("move".to_string()),
            source: None,
        })
    }

    /// Get file metadata
    async fn get_metadata(&self, path: &Path, sandbox: &FileSandbox) -> Result<Value> {
        info!("Getting metadata for: {:?}", path);

        // Validate path
        let safe_path = sandbox.validate_path(path)?;

        let metadata =
            file_utils::get_metadata(&safe_path).map_err(|e| LLMSpellError::Storage {
                message: format!(
                    "Failed to access path: {} - {}",
                    safe_path.to_string_lossy(),
                    e
                ),
                operation: Some("metadata".to_string()),
                source: None,
            })?;

        let file_type = if metadata.is_dir {
            "directory"
        } else if metadata.is_symlink {
            "symlink"
        } else if metadata.is_file {
            "file"
        } else {
            "unknown"
        };

        Ok(json!({
            "path": safe_path.to_string_lossy(),
            "type": file_type,
            "size": metadata.size,
            "readonly": metadata.readonly,
            "created": metadata.created.map(|t| {
                chrono::DateTime::<chrono::Utc>::from(t).to_rfc3339()
            }),
            "modified": metadata.modified.map(|t| {
                chrono::DateTime::<chrono::Utc>::from(t).to_rfc3339()
            }),
            "accessed": metadata.accessed.map(|t| {
                chrono::DateTime::<chrono::Utc>::from(t).to_rfc3339()
            }),
        }))
    }

    /// Check if file exists
    async fn file_exists(&self, path: &Path, sandbox: &FileSandbox) -> Result<bool> {
        debug!("Checking if file exists: {:?}", path);

        // Validate path
        let safe_path = sandbox.validate_path(path)?;

        Ok(file_utils::file_exists(&safe_path))
    }

    /// Parse parameters from input
    fn parse_parameters(&self, params: &Value) -> Result<FileParameters> {
        let operation_str = params
            .get("operation")
            .and_then(|v| v.as_str())
            .ok_or_else(|| LLMSpellError::Validation {
                message: "Missing required parameter 'operation'".to_string(),
                field: Some("operation".to_string()),
            })?;
        let operation: FileOperation = operation_str.parse()?;

        let path = params
            .get("path")
            .and_then(|v| v.as_str())
            .map(PathBuf::from);

        let content = params
            .get("content")
            .and_then(|v| v.as_str())
            .map(String::from);

        let from_path = params
            .get("from")
            .and_then(|v| v.as_str())
            .map(PathBuf::from);

        let to_path = params.get("to").and_then(|v| v.as_str()).map(PathBuf::from);

        let recursive = params
            .get("recursive")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        Ok(FileParameters {
            operation,
            path,
            content,
            from_path,
            to_path,
            recursive,
        })
    }
}

#[derive(Debug)]
struct FileParameters {
    operation: FileOperation,
    path: Option<PathBuf>,
    content: Option<String>,
    from_path: Option<PathBuf>,
    to_path: Option<PathBuf>,
    recursive: bool,
}

impl Default for FileOperationsTool {
    fn default() -> Self {
        Self::new(FileOperationsConfig::default())
    }
}

#[async_trait]
impl BaseAgent for FileOperationsTool {
    fn metadata(&self) -> &ComponentMetadata {
        &self.metadata
    }

    async fn execute(&self, input: AgentInput, context: ExecutionContext) -> Result<AgentOutput> {
        let params =
            input
                .parameters
                .get("parameters")
                .ok_or_else(|| LLMSpellError::Validation {
                    message: "Missing parameters".to_string(),
                    field: Some("parameters".to_string()),
                })?;

        let parameters = self.parse_parameters(params)?;

        // Create sandbox
        let sandbox = self.create_sandbox(&context)?;

        info!("Executing file operation: {}", parameters.operation);

        let result = match parameters.operation {
            FileOperation::Read => {
                let path = parameters.path.ok_or_else(|| LLMSpellError::Validation {
                    message: "Read operation requires 'path' parameter".to_string(),
                    field: Some("path".to_string()),
                })?;
                let content = self.read_file(&path, &sandbox).await?;
                json!({
                    "operation": "read",
                    "path": path.to_string_lossy(),
                    "content": content,
                    "size": content.len()
                })
            }
            FileOperation::Write => {
                let path = parameters.path.ok_or_else(|| LLMSpellError::Validation {
                    message: "Write operation requires 'path' parameter".to_string(),
                    field: Some("path".to_string()),
                })?;
                let content = parameters
                    .content
                    .ok_or_else(|| LLMSpellError::Validation {
                        message: "Write operation requires 'content' parameter".to_string(),
                        field: Some("content".to_string()),
                    })?;
                self.write_file(&path, &content, &sandbox).await?;
                json!({
                    "operation": "write",
                    "path": path.to_string_lossy(),
                    "size": content.len(),
                    "success": true
                })
            }
            FileOperation::Append => {
                let path = parameters.path.ok_or_else(|| LLMSpellError::Validation {
                    message: "Append operation requires 'path' parameter".to_string(),
                    field: Some("path".to_string()),
                })?;
                let content = parameters
                    .content
                    .ok_or_else(|| LLMSpellError::Validation {
                        message: "Append operation requires 'content' parameter".to_string(),
                        field: Some("content".to_string()),
                    })?;
                self.append_file(&path, &content, &sandbox).await?;
                json!({
                    "operation": "append",
                    "path": path.to_string_lossy(),
                    "appended_size": content.len(),
                    "success": true
                })
            }
            FileOperation::Delete => {
                let path = parameters.path.ok_or_else(|| LLMSpellError::Validation {
                    message: "Delete operation requires 'path' parameter".to_string(),
                    field: Some("path".to_string()),
                })?;
                self.delete_file(&path, &sandbox).await?;
                json!({
                    "operation": "delete",
                    "path": path.to_string_lossy(),
                    "success": true
                })
            }
            FileOperation::CreateDir => {
                let path = parameters.path.ok_or_else(|| LLMSpellError::Validation {
                    message: "CreateDir operation requires 'path' parameter".to_string(),
                    field: Some("path".to_string()),
                })?;
                self.create_dir(&path, parameters.recursive, &sandbox)
                    .await?;
                json!({
                    "operation": "create_dir",
                    "path": path.to_string_lossy(),
                    "recursive": parameters.recursive,
                    "success": true
                })
            }
            FileOperation::ListDir => {
                let path = parameters.path.ok_or_else(|| LLMSpellError::Validation {
                    message: "ListDir operation requires 'path' parameter".to_string(),
                    field: Some("path".to_string()),
                })?;
                let entries = self.list_dir(&path, &sandbox).await?;
                json!({
                    "operation": "list_dir",
                    "path": path.to_string_lossy(),
                    "entries": entries,
                    "count": entries.len()
                })
            }
            FileOperation::Copy => {
                let from = parameters
                    .from_path
                    .ok_or_else(|| LLMSpellError::Validation {
                        message: "Copy operation requires 'from' parameter".to_string(),
                        field: Some("from".to_string()),
                    })?;
                let to = parameters
                    .to_path
                    .ok_or_else(|| LLMSpellError::Validation {
                        message: "Copy operation requires 'to' parameter".to_string(),
                        field: Some("to".to_string()),
                    })?;
                self.copy_file(&from, &to, &sandbox).await?;
                json!({
                    "operation": "copy",
                    "from": from.to_string_lossy(),
                    "to": to.to_string_lossy(),
                    "success": true
                })
            }
            FileOperation::Move => {
                let from = parameters
                    .from_path
                    .ok_or_else(|| LLMSpellError::Validation {
                        message: "Move operation requires 'from' parameter".to_string(),
                        field: Some("from".to_string()),
                    })?;
                let to = parameters
                    .to_path
                    .ok_or_else(|| LLMSpellError::Validation {
                        message: "Move operation requires 'to' parameter".to_string(),
                        field: Some("to".to_string()),
                    })?;
                self.move_file(&from, &to, &sandbox).await?;
                json!({
                    "operation": "move",
                    "from": from.to_string_lossy(),
                    "to": to.to_string_lossy(),
                    "success": true
                })
            }
            FileOperation::Metadata => {
                let path = parameters.path.ok_or_else(|| LLMSpellError::Validation {
                    message: "Metadata operation requires 'path' parameter".to_string(),
                    field: Some("path".to_string()),
                })?;
                let metadata = self.get_metadata(&path, &sandbox).await?;
                json!({
                    "operation": "metadata",
                    "metadata": metadata
                })
            }
            FileOperation::Exists => {
                let path = parameters.path.ok_or_else(|| LLMSpellError::Validation {
                    message: "Exists operation requires 'path' parameter".to_string(),
                    field: Some("path".to_string()),
                })?;
                let exists = self.file_exists(&path, &sandbox).await?;
                json!({
                    "operation": "exists",
                    "path": path.to_string_lossy(),
                    "exists": exists
                })
            }
        };

        let output_text = serde_json::to_string_pretty(&result)?;

        // Create metadata
        let mut metadata = llmspell_core::types::OutputMetadata::default();
        metadata.extra.insert(
            "operation".to_string(),
            Value::String(parameters.operation.to_string()),
        );

        Ok(AgentOutput::text(output_text).with_metadata(metadata))
    }

    async fn validate_input(&self, input: &AgentInput) -> Result<()> {
        if input.parameters.is_empty() {
            return Err(LLMSpellError::Validation {
                message: "No parameters provided".to_string(),
                field: Some("parameters".to_string()),
            });
        }

        // Check required parameters based on operation
        if let Some(params) = input.parameters.get("parameters") {
            if params.get("operation").is_none() {
                return Err(LLMSpellError::Validation {
                    message: "Missing required parameter 'operation'".to_string(),
                    field: Some("operation".to_string()),
                });
            }
        }

        Ok(())
    }

    async fn handle_error(&self, error: LLMSpellError) -> Result<AgentOutput> {
        Ok(AgentOutput::text(format!(
            "File operation error: {}",
            error
        )))
    }
}

#[async_trait]
impl Tool for FileOperationsTool {
    fn category(&self) -> ToolCategory {
        ToolCategory::Filesystem
    }

    fn security_level(&self) -> SecurityLevel {
        SecurityLevel::Privileged
    }

    fn schema(&self) -> ToolSchema {
        ToolSchema {
            name: "file_operations".to_string(),
            description: "Perform safe file system operations with sandboxing".to_string(),
            parameters: vec![
                ParameterDef {
                    name: "operation".to_string(),
                    description: "Operation type: read, write, append, delete, create_dir, list_dir, copy, move, metadata, exists"
                        .to_string(),
                    param_type: ParameterType::String,
                    required: true,
                    default: None,
                },
                ParameterDef {
                    name: "path".to_string(),
                    description: "File or directory path".to_string(),
                    param_type: ParameterType::String,
                    required: false,
                    default: None,
                },
                ParameterDef {
                    name: "content".to_string(),
                    description: "Content for write/append operations".to_string(),
                    param_type: ParameterType::String,
                    required: false,
                    default: None,
                },
                ParameterDef {
                    name: "from".to_string(),
                    description: "Source path for copy/move operations".to_string(),
                    param_type: ParameterType::String,
                    required: false,
                    default: None,
                },
                ParameterDef {
                    name: "to".to_string(),
                    description: "Destination path for copy/move operations".to_string(),
                    param_type: ParameterType::String,
                    required: false,
                    default: None,
                },
                ParameterDef {
                    name: "recursive".to_string(),
                    description: "Enable recursive directory operations".to_string(),
                    param_type: ParameterType::Boolean,
                    required: false,
                    default: Some(json!(false)),
                },
            ],
            returns: Some(ParameterType::Object),
        }
    }

    fn security_requirements(&self) -> SecurityRequirements {
        SecurityRequirements {
            level: SecurityLevel::Privileged,
            file_permissions: vec!["/tmp".to_string()], // Default to /tmp only
            network_permissions: vec![],
            env_permissions: vec![],
            custom_requirements: HashMap::new(),
        }
    }

    fn resource_limits(&self) -> ResourceLimits {
        ResourceLimits::default()
            .with_memory_limit(100 * 1024 * 1024) // 100MB
            .with_cpu_limit(30000) // 30s
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_operation_parsing() {
        assert_eq!(
            "read".parse::<FileOperation>().unwrap(),
            FileOperation::Read
        );
        assert_eq!(
            "write".parse::<FileOperation>().unwrap(),
            FileOperation::Write
        );
        assert_eq!(
            "mkdir".parse::<FileOperation>().unwrap(),
            FileOperation::CreateDir
        );
        assert_eq!(
            "ls".parse::<FileOperation>().unwrap(),
            FileOperation::ListDir
        );
        assert!("invalid".parse::<FileOperation>().is_err());
    }

    #[tokio::test]
    async fn test_file_operations_tool_creation() {
        let config = FileOperationsConfig::default();
        let tool = FileOperationsTool::new(config);

        assert_eq!(tool.metadata().name, "file-operations-tool");
    }
}
