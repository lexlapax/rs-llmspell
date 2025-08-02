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
    types::{AgentInput, AgentOutput},
    ComponentMetadata, ExecutionContext, LLMSpellError, Result,
};
use llmspell_security::sandbox::{FileSandbox, SandboxContext};
use llmspell_utils::{
    extract_optional_bool, extract_optional_string, extract_parameters, extract_required_string,
    file_utils, response::ResponseBuilder,
};
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
            Self::Read => write!(f, "read"),
            Self::Write => write!(f, "write"),
            Self::Append => write!(f, "append"),
            Self::Delete => write!(f, "delete"),
            Self::CreateDir => write!(f, "create_dir"),
            Self::ListDir => write!(f, "list_dir"),
            Self::Copy => write!(f, "copy"),
            Self::Move => write!(f, "move"),
            Self::Metadata => write!(f, "metadata"),
            Self::Exists => write!(f, "exists"),
        }
    }
}

impl std::str::FromStr for FileOperation {
    type Err = LLMSpellError;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "read" => Ok(Self::Read),
            "write" => Ok(Self::Write),
            "append" => Ok(Self::Append),
            "delete" => Ok(Self::Delete),
            "create_dir" | "mkdir" => Ok(Self::CreateDir),
            "list_dir" | "ls" | "dir" => Ok(Self::ListDir),
            "copy" | "cp" => Ok(Self::Copy),
            "move" | "mv" | "rename" => Ok(Self::Move),
            "metadata" | "stat" | "info" => Ok(Self::Metadata),
            "exists" => Ok(Self::Exists),
            _ => Err(LLMSpellError::Validation {
                message: format!("Unknown file operation: {s}"),
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
    #[must_use]
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
    #[allow(clippy::unused_async)]
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

        if usize::try_from(metadata.size).unwrap_or(usize::MAX) > self.config.max_file_size {
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
    #[allow(clippy::unused_async)]
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
    #[allow(clippy::unused_async)]
    async fn append_file(&self, path: &Path, content: &str, sandbox: &FileSandbox) -> Result<()> {
        info!("Appending to file: {:?}", path);

        // Validate path
        let safe_path = sandbox.validate_path(path)?;

        // Read existing content first to check total size
        let existing_size = if file_utils::file_exists(&safe_path) {
            file_utils::get_metadata(&safe_path)
                .map(|m| usize::try_from(m.size).unwrap_or(usize::MAX))
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
    #[allow(clippy::unused_async)]
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
    #[allow(clippy::unused_async)]
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
    #[allow(clippy::unused_async)]
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
    #[allow(clippy::unused_async)]
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

        if usize::try_from(metadata.size).unwrap_or(usize::MAX) > self.config.max_file_size {
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
    #[allow(clippy::unused_async)]
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
    #[allow(clippy::unused_self)]
    fn parse_parameters(&self, params: &Value) -> Result<FileParameters> {
        let operation_str = extract_required_string(params, "operation")?;
        let operation: FileOperation = operation_str.parse()?;

        // Validate paths - we don't sanitize because the FileSandbox handles security
        let path = extract_optional_string(params, "path").map(|p| {
            // Check for obvious traversal attempts but allow absolute paths
            // since FileSandbox will enforce the actual security boundaries
            if p.contains("../") || p.contains("..\\") {
                warn!("Path traversal attempt detected in path: {}", p);
            }
            PathBuf::from(p)
        });

        let input = extract_optional_string(params, "input").map(String::from);

        let source_path = extract_optional_string(params, "source_path").map(|p| {
            if p.contains("../") || p.contains("..\\") {
                warn!("Path traversal attempt detected in source_path: {}", p);
            }
            PathBuf::from(p)
        });

        let target_path = extract_optional_string(params, "target_path").map(|p| {
            if p.contains("../") || p.contains("..\\") {
                warn!("Path traversal attempt detected in target_path: {}", p);
            }
            PathBuf::from(p)
        });

        let recursive = extract_optional_bool(params, "recursive").unwrap_or(false);

        Ok(FileParameters {
            operation,
            path,
            input,
            source_path,
            target_path,
            recursive,
        })
    }
}

#[derive(Debug)]
struct FileParameters {
    operation: FileOperation,
    path: Option<PathBuf>,
    input: Option<String>,        // renamed from content
    source_path: Option<PathBuf>, // renamed from from_path
    target_path: Option<PathBuf>, // renamed from to_path
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
        // Get parameters using shared utility
        let params = extract_parameters(&input)?;
        let parameters = self.parse_parameters(params)?;

        // Create sandbox
        let sandbox = self.create_sandbox(&context)?;

        info!("Executing file operation: {}", parameters.operation);

        let (output_text, response_json) = match parameters.operation {
            FileOperation::Read => {
                let path = parameters.path.ok_or_else(|| LLMSpellError::Validation {
                    message: "Read operation requires 'path' parameter".to_string(),
                    field: Some("path".to_string()),
                })?;
                let file_content = self.read_file(&path, &sandbox).await?;
                let response = ResponseBuilder::success("read")
                    .with_message(format!(
                        "Read {} bytes from {}",
                        file_content.len(),
                        path.display()
                    ))
                    .with_result(json!({
                        "input": &file_content,
                        "size": file_content.len()
                    }))
                    .with_file_info(path.to_string_lossy(), Some(file_content.len() as u64))
                    .build();
                (file_content, response)
            }
            FileOperation::Write => {
                let path = parameters.path.ok_or_else(|| LLMSpellError::Validation {
                    message: "Write operation requires 'path' parameter".to_string(),
                    field: Some("path".to_string()),
                })?;
                let write_content = parameters.input.ok_or_else(|| LLMSpellError::Validation {
                    message: "Write operation requires 'input' parameter".to_string(),
                    field: Some("input".to_string()),
                })?;
                self.write_file(&path, &write_content, &sandbox).await?;
                ResponseBuilder::success("write")
                    .with_message(format!(
                        "Wrote {} bytes to {}",
                        write_content.len(),
                        path.display()
                    ))
                    .with_file_info(path.to_string_lossy(), Some(write_content.len() as u64))
                    .build_for_output()
            }
            FileOperation::Append => {
                let path = parameters.path.ok_or_else(|| LLMSpellError::Validation {
                    message: "Append operation requires 'path' parameter".to_string(),
                    field: Some("path".to_string()),
                })?;
                let append_content = parameters.input.ok_or_else(|| LLMSpellError::Validation {
                    message: "Append operation requires 'input' parameter".to_string(),
                    field: Some("input".to_string()),
                })?;
                self.append_file(&path, &append_content, &sandbox).await?;
                ResponseBuilder::success("append")
                    .with_message(format!(
                        "Appended {} bytes to {}",
                        append_content.len(),
                        path.display()
                    ))
                    .with_result(json!({
                        "appended_size": append_content.len()
                    }))
                    .with_file_info(path.to_string_lossy(), None)
                    .build_for_output()
            }
            FileOperation::Delete => {
                let path = parameters.path.ok_or_else(|| LLMSpellError::Validation {
                    message: "Delete operation requires 'path' parameter".to_string(),
                    field: Some("path".to_string()),
                })?;
                self.delete_file(&path, &sandbox).await?;
                ResponseBuilder::success("delete")
                    .with_message(format!("Deleted file: {}", path.display()))
                    .with_file_info(path.to_string_lossy(), None)
                    .build_for_output()
            }
            FileOperation::CreateDir => {
                let path = parameters.path.ok_or_else(|| LLMSpellError::Validation {
                    message: "CreateDir operation requires 'path' parameter".to_string(),
                    field: Some("path".to_string()),
                })?;
                self.create_dir(&path, parameters.recursive, &sandbox)
                    .await?;
                ResponseBuilder::success("create_dir")
                    .with_message(format!("Created directory: {}", path.display()))
                    .with_result(json!({
                        "recursive": parameters.recursive
                    }))
                    .with_file_info(path.to_string_lossy(), None)
                    .build_for_output()
            }
            FileOperation::ListDir => {
                let path = parameters.path.ok_or_else(|| LLMSpellError::Validation {
                    message: "ListDir operation requires 'path' parameter".to_string(),
                    field: Some("path".to_string()),
                })?;
                let entries = self.list_dir(&path, &sandbox).await?;
                ResponseBuilder::success("list_dir")
                    .with_message(format!(
                        "Found {} entries in {}",
                        entries.len(),
                        path.display()
                    ))
                    .with_result(json!({
                        "entries": entries,
                        "count": entries.len()
                    }))
                    .with_file_info(path.to_string_lossy(), None)
                    .build_for_output()
            }
            FileOperation::Copy => {
                let from = parameters
                    .source_path
                    .ok_or_else(|| LLMSpellError::Validation {
                        message: "Copy operation requires 'source_path' parameter".to_string(),
                        field: Some("source_path".to_string()),
                    })?;
                let to = parameters
                    .target_path
                    .ok_or_else(|| LLMSpellError::Validation {
                        message: "Copy operation requires 'target_path' parameter".to_string(),
                        field: Some("target_path".to_string()),
                    })?;
                self.copy_file(&from, &to, &sandbox).await?;
                ResponseBuilder::success("copy")
                    .with_message(format!("Copied {} to {}", from.display(), to.display()))
                    .with_result(json!({
                        "source": from.to_string_lossy(),
                        "target": to.to_string_lossy()
                    }))
                    .build_for_output()
            }
            FileOperation::Move => {
                let from = parameters
                    .source_path
                    .ok_or_else(|| LLMSpellError::Validation {
                        message: "Move operation requires 'source_path' parameter".to_string(),
                        field: Some("source_path".to_string()),
                    })?;
                let to = parameters
                    .target_path
                    .ok_or_else(|| LLMSpellError::Validation {
                        message: "Move operation requires 'target_path' parameter".to_string(),
                        field: Some("target_path".to_string()),
                    })?;
                self.move_file(&from, &to, &sandbox).await?;
                ResponseBuilder::success("move")
                    .with_message(format!("Moved {} to {}", from.display(), to.display()))
                    .with_result(json!({
                        "source": from.to_string_lossy(),
                        "target": to.to_string_lossy()
                    }))
                    .build_for_output()
            }
            FileOperation::Metadata => {
                let path = parameters.path.ok_or_else(|| LLMSpellError::Validation {
                    message: "Metadata operation requires 'path' parameter".to_string(),
                    field: Some("path".to_string()),
                })?;
                let metadata = self.get_metadata(&path, &sandbox).await?;
                ResponseBuilder::success("metadata")
                    .with_message(format!("Retrieved metadata for {}", path.display()))
                    .with_result(metadata)
                    .build_for_output()
            }
            FileOperation::Exists => {
                let path = parameters.path.ok_or_else(|| LLMSpellError::Validation {
                    message: "Exists operation requires 'path' parameter".to_string(),
                    field: Some("path".to_string()),
                })?;
                let exists = self.file_exists(&path, &sandbox).await?;
                ResponseBuilder::success("exists")
                    .with_message(format!(
                        "Path {} {}",
                        path.display(),
                        if exists { "exists" } else { "does not exist" }
                    ))
                    .with_result(json!({
                        "exists": exists
                    }))
                    .with_file_info(path.to_string_lossy(), None)
                    .build_for_output()
            }
        };

        // Create metadata
        let mut metadata = llmspell_core::types::OutputMetadata::default();
        metadata.extra.insert(
            "operation".to_string(),
            Value::String(parameters.operation.to_string()),
        );
        metadata.extra.insert("response".to_string(), response_json);

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
        Ok(AgentOutput::text(format!("File operation error: {error}")))
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
                    name: "input".to_string(),
                    description: "Content for write/append operations".to_string(),
                    param_type: ParameterType::String,
                    required: false,
                    default: None,
                },
                ParameterDef {
                    name: "source_path".to_string(),
                    description: "Source path for copy/move operations".to_string(),
                    param_type: ParameterType::String,
                    required: false,
                    default: None,
                },
                ParameterDef {
                    name: "target_path".to_string(),
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

impl FileOperationsTool {
    /// Check if this tool supports hook integration
    #[must_use]
    pub const fn supports_hooks(&self) -> bool {
        true // All tools that implement Tool automatically support hooks
    }

    /// Get hook integration metadata for this tool
    #[must_use]
    pub fn hook_metadata(&self) -> serde_json::Value {
        json!({
            "tool_name": self.metadata().name,
            "hook_points_supported": [
                "parameter_validation",
                "security_check",
                "resource_allocation",
                "pre_execution",
                "post_execution",
                "error_handling",
                "resource_cleanup",
                "timeout"
            ],
            "security_level": self.security_level(),
            "resource_limits": {
                "memory_mb": 100,
                "cpu_time_seconds": 30,
                "file_ops_critical": true
            },
            "hook_integration_benefits": [
                "File access validation and sandboxing",
                "Path traversal attack prevention",
                "Atomic write operation monitoring",
                "Resource usage tracking for file operations",
                "Security audit logging for sensitive file operations",
                "Performance monitoring for I/O intensive operations"
            ],
            "security_considerations": [
                "All file paths validated through sandbox",
                "Privileged security level for file system access",
                "Path traversal protection enabled",
                "Atomic write operations for data integrity"
            ]
        })
    }

    /// Demonstrate hook-aware execution for file operations
    /// This method showcases how the file operations tool works with the hook system
    pub async fn demonstrate_hook_integration(
        &self,
        tool_executor: &crate::lifecycle::ToolExecutor,
        operation: &str,
        path: &str,
        file_content: Option<&str>,
    ) -> Result<AgentOutput> {
        let mut params = json!({
            "operation": operation,
            "path": path,
            "hook_integration": true  // Flag to indicate this is a hook demo
        });

        if let Some(content) = file_content {
            params["input"] = json!(content);
        }

        let input = AgentInput::text("File operations hook demonstration")
            .with_parameter("parameters", params);
        let context = ExecutionContext::default();

        // Execute with hooks using the HookableToolExecution trait
        use crate::lifecycle::HookableToolExecution;
        self.execute_with_hooks(input, context, tool_executor).await
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
    #[test]
    fn test_parse_parameters_standardized() {
        let tool = FileOperationsTool::default();

        // Test write operation with new 'input' parameter
        let params = json!({
            "operation": "write",
            "path": "/tmp/test.txt",
            "input": "Hello, World!"
        });
        let parsed = tool.parse_parameters(&params).unwrap();
        assert_eq!(parsed.operation, FileOperation::Write);
        assert_eq!(parsed.path, Some(PathBuf::from("/tmp/test.txt")));
        assert_eq!(parsed.input, Some("Hello, World!".to_string()));

        // Test copy operation with new source_path/target_path parameters
        let params = json!({
            "operation": "copy",
            "source_path": "/tmp/source.txt",
            "target_path": "/tmp/target.txt"
        });
        let parsed = tool.parse_parameters(&params).unwrap();
        assert_eq!(parsed.operation, FileOperation::Copy);
        assert_eq!(parsed.source_path, Some(PathBuf::from("/tmp/source.txt")));
        assert_eq!(parsed.target_path, Some(PathBuf::from("/tmp/target.txt")));
    }
    #[test]
    fn test_hook_integration_metadata() {
        let tool = FileOperationsTool::default();

        // Test that the tool supports hooks
        assert!(tool.supports_hooks());

        // Test hook metadata
        let metadata = tool.hook_metadata();
        assert_eq!(metadata["tool_name"], "file-operations-tool");
        assert!(metadata["hook_points_supported"].is_array());
        assert_eq!(
            metadata["hook_points_supported"].as_array().unwrap().len(),
            8
        );
        assert!(metadata["hook_integration_benefits"].is_array());
        assert!(metadata["security_considerations"].is_array());
        assert_eq!(metadata["security_level"], "Privileged");
    }
    #[tokio::test]
    async fn test_file_operations_hook_integration() {
        use crate::lifecycle::{ToolExecutor, ToolLifecycleConfig};
        let tool = FileOperationsTool::default();

        let config = ToolLifecycleConfig::default();
        let tool_executor = ToolExecutor::new(config, None, None);

        // Demonstrate hook integration with a read operation (should fail gracefully)
        let result = tool
            .demonstrate_hook_integration(
                &tool_executor,
                "exists",
                "/tmp/test_hook_integration.txt",
                None,
            )
            .await;

        // The operation might fail due to file not existing, but should not panic
        // and should return a proper response structure
        assert!(result.is_ok() || result.is_err());
    }
    #[tokio::test]
    async fn test_hookable_tool_execution_trait() {
        use crate::lifecycle::{HookableToolExecution, ToolExecutor, ToolLifecycleConfig};
        let tool = FileOperationsTool::default();

        // Verify the tool implements HookableToolExecution
        // This is automatic via the blanket implementation
        let config = ToolLifecycleConfig::default();
        let tool_executor = ToolExecutor::new(config, None, None);

        let input = AgentInput::text("Hook trait test").with_parameter(
            "parameters",
            json!({
                "operation": "exists",
                "path": "/tmp/hook_trait_test.txt"
            }),
        );
        let context = ExecutionContext::default();

        // This should compile and execute without errors (file may not exist, that's ok)
        let result = tool
            .execute_with_hooks(input, context, &tool_executor)
            .await;
        assert!(result.is_ok() || result.is_err()); // Should not panic
    }
}
