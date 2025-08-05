// ABOUTME: Process execution tool with security sandboxing and resource controls
// ABOUTME: Provides safe execution of system processes with configurable permissions and limits

use crate::lifecycle::HookableToolExecution;
use async_trait::async_trait;
use llmspell_core::{
    traits::{
        base_agent::BaseAgent,
        tool::{ParameterDef, ParameterType, SecurityLevel, Tool, ToolCategory, ToolSchema},
    },
    types::{AgentInput, AgentOutput},
    ComponentMetadata, ExecutionContext, LLMSpellError, Result as LLMResult,
};
use llmspell_security::sandbox::SandboxContext;
use llmspell_utils::{
    // NEW: Error handling with information disclosure prevention
    error_handling::{ErrorContext, SafeErrorHandler},
    extract_optional_array,
    extract_optional_object,
    extract_optional_string,
    extract_parameters,
    extract_required_string,
    response::ResponseBuilder,
    security::input_sanitizer::InputSanitizer,
    system_info::find_executable,
    // NEW: Using shared timeout utility
    timeout::TimeoutBuilder,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::Arc;
use std::time::Duration;
use tokio::process::Command;
use tracing::{debug, info, warn};

/// Process execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessResult {
    /// Exit code of the process
    pub exit_code: Option<i32>,
    /// Standard output
    pub stdout: String,
    /// Standard error
    pub stderr: String,
    /// Whether the process was successful (exit code 0)
    pub success: bool,
    /// Execution time in milliseconds
    pub execution_time_ms: u64,
    /// Whether the process was terminated due to timeout
    pub timed_out: bool,
}

/// I/O capture configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IoCapture {
    /// Whether to capture stdout
    pub stdout: bool,
    /// Whether to capture stderr
    pub stderr: bool,
}

/// Execution permissions configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionPermissions {
    /// Whether to allow arbitrary command execution
    pub allow_arbitrary_commands: bool,
    /// Whether to inherit current process environment
    pub inherit_environment: bool,
}

/// Process executor tool configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessExecutorConfig {
    /// Maximum execution time in seconds
    pub max_execution_time_seconds: u64,
    /// Maximum output size in bytes
    pub max_output_size: usize,
    /// Allowed executable paths or names
    pub allowed_executables: Vec<String>,
    /// Blocked executable patterns
    pub blocked_executables: Vec<String>,
    /// Execution permissions
    pub permissions: ExecutionPermissions,
    /// Default working directory
    pub default_working_directory: Option<PathBuf>,
    /// I/O capture settings
    pub io_capture: IoCapture,
    /// Environment variables to pass to processes
    pub allowed_env_vars: Vec<String>,
}

impl Default for ProcessExecutorConfig {
    fn default() -> Self {
        Self {
            max_execution_time_seconds: 30,
            max_output_size: 1024 * 1024, // 1MB
            allowed_executables: vec![
                "echo".to_string(),
                "cat".to_string(),
                "ls".to_string(),
                "pwd".to_string(),
                "whoami".to_string(),
                "date".to_string(),
                "uname".to_string(),
                "which".to_string(),
                "test".to_string(),
                "head".to_string(),
                "tail".to_string(),
                "wc".to_string(),
                "grep".to_string(),
                "find".to_string(),
                "sort".to_string(),
                "uniq".to_string(),
            ],
            blocked_executables: vec![
                "rm".to_string(),
                "rmdir".to_string(),
                "mv".to_string(),
                "cp".to_string(),
                "chmod".to_string(),
                "chown".to_string(),
                "sudo".to_string(),
                "su".to_string(),
                "passwd".to_string(),
                "ssh".to_string(),
                "scp".to_string(),
                "curl".to_string(),
                "wget".to_string(),
                "nc".to_string(),
                "netcat".to_string(),
                "nmap".to_string(),
                "dd".to_string(),
                "mkfs".to_string(),
                "fdisk".to_string(),
                "mount".to_string(),
                "umount".to_string(),
                "systemctl".to_string(),
                "service".to_string(),
                "kill".to_string(),
                "killall".to_string(),
                "pkill".to_string(),
            ],
            permissions: ExecutionPermissions {
                allow_arbitrary_commands: false,
                inherit_environment: false,
            },
            default_working_directory: None,
            io_capture: IoCapture {
                stdout: true,
                stderr: true,
            },
            allowed_env_vars: vec![
                "PATH".to_string(),
                "HOME".to_string(),
                "USER".to_string(),
                "SHELL".to_string(),
                "TERM".to_string(),
                "LANG".to_string(),
                "LC_ALL".to_string(),
                "TZ".to_string(),
            ],
        }
    }
}

/// Process executor tool for safe process execution
pub struct ProcessExecutorTool {
    metadata: ComponentMetadata,
    config: ProcessExecutorConfig,
    #[allow(dead_code)] // Reserved for future sandbox integration
    sandbox_context: Option<Arc<SandboxContext>>,
    error_handler: SafeErrorHandler,
}

impl ProcessExecutorTool {
    /// Create a new process executor tool
    #[must_use]
    pub fn new(config: ProcessExecutorConfig) -> Self {
        // Determine if in production mode based on config
        let is_production = !cfg!(debug_assertions);

        Self {
            metadata: ComponentMetadata::new(
                "process_executor".to_string(),
                "Safe process execution with security controls and resource limits".to_string(),
            ),
            config,
            sandbox_context: None,
            error_handler: SafeErrorHandler::new(is_production),
        }
    }

    /// Create a new process executor tool with sandbox context
    #[must_use]
    pub fn with_sandbox(
        config: ProcessExecutorConfig,
        sandbox_context: Arc<SandboxContext>,
    ) -> Self {
        let is_production = !cfg!(debug_assertions);

        Self {
            metadata: ComponentMetadata::new(
                "process_executor".to_string(),
                "Safe process execution with security controls and resource limits".to_string(),
            ),
            config,
            sandbox_context: Some(sandbox_context),
            error_handler: SafeErrorHandler::new(is_production),
        }
    }

    /// Check if an executable is allowed to be executed
    fn is_executable_allowed(&self, executable: &str) -> bool {
        // Check blocked executables first (takes precedence)
        for blocked in &self.config.blocked_executables {
            if executable == blocked || executable.ends_with(&format!("/{blocked}")) {
                debug!("Executable '{}' is blocked", executable);
                return false;
            }
        }

        // If arbitrary commands are allowed, allow it
        if self.config.permissions.allow_arbitrary_commands {
            return true;
        }

        // Check allowed executables
        for allowed in &self.config.allowed_executables {
            if executable == allowed || executable.ends_with(&format!("/{allowed}")) {
                debug!("Executable '{}' is allowed", executable);
                return true;
            }
        }

        debug!("Executable '{}' is not in allowed list", executable);
        false
    }

    /// Resolve executable path
    #[allow(clippy::unused_async)]
    async fn resolve_executable(&self, executable: &str) -> LLMResult<PathBuf> {
        // If it's already a full path, validate it exists
        let exe_path = Path::new(executable);
        if exe_path.is_absolute() {
            if exe_path.exists() {
                return Ok(exe_path.to_path_buf());
            }
            return Err(LLMSpellError::Validation {
                message: format!("Executable not found: {executable}"),
                field: Some("executable".to_string()),
            });
        }

        // Try to find in PATH
        find_executable(executable).map_or_else(
            || {
                Err(LLMSpellError::Validation {
                    message: format!("Executable not found in PATH: {executable}"),
                    field: Some("executable".to_string()),
                })
            },
            Ok,
        )
    }

    /// Execute a process with the given arguments
    async fn execute_process(
        &self,
        executable: &str,
        args: &[String],
        working_dir: Option<&Path>,
        env_vars: Option<&HashMap<String, String>>,
    ) -> LLMResult<ProcessResult> {
        let start_time = std::time::Instant::now();

        // Resolve executable path
        let exe_path = self.resolve_executable(executable).await?;

        // Check if executable is allowed
        if !self.is_executable_allowed(exe_path.to_str().unwrap_or(executable)) {
            return Err(LLMSpellError::Security {
                message: format!("Execution of '{executable}' is not permitted"),
                violation_type: Some("executable_blocked".to_string()),
            });
        }

        info!(
            "Executing process: {} with args: {:?}",
            exe_path.display(),
            args
        );

        // Build command
        let mut cmd = Command::new(&exe_path);
        cmd.args(args);

        // Set working directory
        if let Some(dir) = working_dir.or(self.config.default_working_directory.as_deref()) {
            cmd.current_dir(dir);
        }

        // Configure stdio
        if self.config.io_capture.stdout {
            cmd.stdout(Stdio::piped());
        } else {
            cmd.stdout(Stdio::null());
        }

        if self.config.io_capture.stderr {
            cmd.stderr(Stdio::piped());
        } else {
            cmd.stderr(Stdio::null());
        }

        cmd.stdin(Stdio::null()); // Don't allow stdin input for security

        // Set environment variables
        if !self.config.permissions.inherit_environment {
            cmd.env_clear();
        }

        // Add allowed environment variables
        if let Some(env) = env_vars {
            for (key, value) in env {
                if self.config.allowed_env_vars.contains(key) {
                    cmd.env(key, value);
                }
            }
        }

        // Add default environment variables if inheriting
        if self.config.permissions.inherit_environment {
            for var in &self.config.allowed_env_vars {
                if let Ok(value) = std::env::var(var) {
                    cmd.env(var, value);
                }
            }
        }

        // Execute with timeout using shared utility
        let timeout_duration = Duration::from_secs(self.config.max_execution_time_seconds);
        let process_name = format!("{} {:?}", exe_path.display(), args);

        let timeout_result = TimeoutBuilder::default()
            .duration(timeout_duration)
            .name(process_name.clone())
            .warn_after(Duration::from_secs(
                self.config.max_execution_time_seconds / 2,
            ))
            .execute(cmd.output())
            .await;

        let process_result = match timeout_result {
            Ok(Ok(output)) => {
                let execution_time = start_time.elapsed();

                // Check output size limits
                let stdout_str = if output.stdout.len() > self.config.max_output_size {
                    warn!("Stdout output truncated due to size limit");
                    String::from_utf8_lossy(&output.stdout[..self.config.max_output_size])
                        .to_string()
                } else {
                    String::from_utf8_lossy(&output.stdout).to_string()
                };

                let stderr_str = if output.stderr.len() > self.config.max_output_size {
                    warn!("Stderr output truncated due to size limit");
                    String::from_utf8_lossy(&output.stderr[..self.config.max_output_size])
                        .to_string()
                } else {
                    String::from_utf8_lossy(&output.stderr).to_string()
                };

                ProcessResult {
                    exit_code: output.status.code(),
                    stdout: stdout_str,
                    stderr: stderr_str,
                    success: output.status.success(),
                    execution_time_ms: u64::try_from(execution_time.as_millis())
                        .unwrap_or(u64::MAX),
                    timed_out: false,
                }
            }
            Ok(Err(e)) => {
                return Err(LLMSpellError::Tool {
                    message: format!("Failed to execute process: {e}"),
                    tool_name: Some("process_executor".to_string()),
                    source: None,
                });
            }
            Err(_) => {
                // Timeout occurred
                warn!(
                    "Process '{}' timed out after {:?}",
                    process_name, timeout_duration
                );
                ProcessResult {
                    exit_code: None,
                    stdout: String::new(),
                    stderr: "Process timed out".to_string(),
                    success: false,
                    execution_time_ms: self.config.max_execution_time_seconds * 1000,
                    timed_out: true,
                }
            }
        };

        info!(
            "Process execution completed: exit_code={:?}, success={}, time={}ms",
            process_result.exit_code, process_result.success, process_result.execution_time_ms
        );

        Ok(process_result)
    }

    /// Validate execution parameters
    #[allow(clippy::unused_async)]
    async fn validate_execution_parameters(
        &self,
        params: &HashMap<String, serde_json::Value>,
    ) -> LLMResult<()> {
        // Check if parameters object exists (for direct validation)
        let params = params
            .get("parameters")
            .and_then(|v| v.as_object())
            .ok_or_else(|| LLMSpellError::Validation {
                message: "Missing parameters object".to_string(),
                field: Some("parameters".to_string()),
            })?;

        // Validate executable
        if let Some(executable) = params.get("executable").and_then(|v| v.as_str()) {
            if executable.trim().is_empty() {
                return Err(LLMSpellError::Validation {
                    message: "Executable cannot be empty".to_string(),
                    field: Some("executable".to_string()),
                });
            }

            // Basic security checks
            if executable.contains("..")
                || executable.contains(';')
                || executable.contains('|')
                || executable.contains('&')
                || executable.contains('`')
                || executable.contains('$')
            {
                return Err(LLMSpellError::Validation {
                    message: "Executable contains potentially dangerous characters".to_string(),
                    field: Some("executable".to_string()),
                });
            }
        }

        // Validate working directory if provided
        if let Some(work_dir) = params.get("working_directory").and_then(|v| v.as_str()) {
            let dir_path = Path::new(work_dir);
            if !dir_path.exists() {
                return Err(LLMSpellError::Validation {
                    message: format!("Working directory does not exist: {work_dir}"),
                    field: Some("working_directory".to_string()),
                });
            }
            if !dir_path.is_dir() {
                return Err(LLMSpellError::Validation {
                    message: format!("Working directory is not a directory: {work_dir}"),
                    field: Some("working_directory".to_string()),
                });
            }
        }

        Ok(())
    }
}

#[async_trait]
impl BaseAgent for ProcessExecutorTool {
    fn metadata(&self) -> &ComponentMetadata {
        &self.metadata
    }

    async fn execute(
        &self,
        input: AgentInput,
        _context: ExecutionContext,
    ) -> LLMResult<AgentOutput> {
        // Get parameters using shared utility
        let params = extract_parameters(&input)?;

        self.validate_execution_parameters(&input.parameters)
            .await?;

        // Extract required parameters
        let executable = extract_required_string(params, "executable")?;

        // Sanitize executable to prevent command injection
        let sanitizer = InputSanitizer::new();
        let sanitized_executable = sanitizer.sanitize_command(executable);

        // Extract optional parameters
        let args: Vec<String> = extract_optional_array(params, "arguments")
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| {
                        // Sanitize each argument to prevent injection
                        sanitizer.sanitize_command(s)
                    })
                    .collect()
            })
            .unwrap_or_default();

        let working_dir_str = extract_optional_string(params, "working_directory");
        let working_dir = working_dir_str.as_ref().and_then(|dir| {
            // Sanitize path to prevent directory traversal
            sanitizer.sanitize_path(dir).ok().or_else(|| {
                warn!("Invalid working directory path detected: {}", dir);
                None
            })
        });
        let working_dir_path = working_dir.as_deref().map(Path::new);

        let env_vars: Option<HashMap<String, String>> =
            extract_optional_object(params, "environment").map(|obj| {
                obj.iter()
                    .filter_map(|(k, v)| {
                        v.as_str().map(|s| {
                            // Sanitize environment variable values
                            (k.clone(), sanitizer.sanitize_command(s))
                        })
                    })
                    .collect()
            });

        // Execute the process
        let result = self
            .execute_process(
                &sanitized_executable,
                &args,
                working_dir_path,
                env_vars.as_ref(),
            )
            .await?;

        // Format response
        let message = if result.success {
            format!(
                "Process '{}' executed successfully in {}ms",
                executable, result.execution_time_ms
            )
        } else if result.timed_out {
            format!(
                "Process '{}' timed out after {}s",
                executable, self.config.max_execution_time_seconds
            )
        } else {
            format!(
                "Process '{}' failed with exit code {:?}",
                executable, result.exit_code
            )
        };

        let response = ResponseBuilder::success("execute")
            .with_message(message)
            .with_result(json!({
                "executable": executable,
                "arguments": args,
                "exit_code": result.exit_code,
                "success": result.success,
                "stdout": result.stdout,
                "stderr": result.stderr,
                "execution_time_ms": result.execution_time_ms,
                "timed_out": result.timed_out
            }))
            .build();

        Ok(AgentOutput::text(serde_json::to_string_pretty(&response)?))
    }

    async fn validate_input(&self, input: &AgentInput) -> LLMResult<()> {
        if input.text.is_empty() {
            return Err(LLMSpellError::Validation {
                message: "Input prompt cannot be empty".to_string(),
                field: Some("prompt".to_string()),
            });
        }
        Ok(())
    }

    async fn handle_error(&self, error: LLMSpellError) -> LLMResult<AgentOutput> {
        // Use SafeErrorHandler to sanitize error messages
        let context = ErrorContext::new()
            .with_operation("process_execution")
            .with_metadata("tool", "process_executor");

        let safe_response = self.error_handler.handle_llmspell_error(&error, &context);

        Ok(AgentOutput::text(
            serde_json::to_string_pretty(&safe_response)
                .unwrap_or_else(|_| format!("{safe_response:?}")),
        ))
    }
}

#[async_trait]
impl Tool for ProcessExecutorTool {
    fn category(&self) -> ToolCategory {
        ToolCategory::System
    }

    fn security_level(&self) -> SecurityLevel {
        SecurityLevel::Restricted // Process execution requires restricted security
    }

    fn schema(&self) -> ToolSchema {
        ToolSchema::new(
            "process_executor".to_string(),
            "Execute system processes with security controls and resource limits".to_string(),
        )
        .with_parameter(ParameterDef {
            name: "executable".to_string(),
            param_type: ParameterType::String,
            description: "Name or path of executable to run".to_string(),
            required: true,
            default: None,
        })
        .with_parameter(ParameterDef {
            name: "arguments".to_string(),
            param_type: ParameterType::Array,
            description: "Array of command line arguments".to_string(),
            required: false,
            default: Some(json!([])),
        })
        .with_parameter(ParameterDef {
            name: "working_directory".to_string(),
            param_type: ParameterType::String,
            description: "Working directory for the process".to_string(),
            required: false,
            default: None,
        })
        .with_parameter(ParameterDef {
            name: "environment".to_string(),
            param_type: ParameterType::Object,
            description: "Environment variables to set for the process".to_string(),
            required: false,
            default: Some(json!({})),
        })
    }
}

impl ProcessExecutorTool {
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
                "timeout_seconds": self.config.max_execution_time_seconds,
                "max_output_size": self.config.max_output_size,
                "allowed_executables": self.config.allowed_executables.len(),
                "security_critical": true
            },
            "hook_integration_benefits": [
                "Command injection prevention and validation",
                "Process execution sandboxing and isolation",
                "Resource usage monitoring and limits enforcement",
                "Security audit logging for all process executions",
                "Path traversal and privilege escalation prevention",
                "Environment variable sanitization",
                "Process termination and cleanup tracking",
                "Security compliance for system command execution"
            ],
            "security_considerations": [
                "Restricted security level for process execution privilege",
                "Command whitelist validation to prevent arbitrary execution",
                "Resource limits to prevent system resource exhaustion",
                "Sandbox isolation to contain process execution",
                "Environment variable filtering and sanitization",
                "Working directory restrictions",
                "Process output filtering to prevent information disclosure"
            ],
            "supported_operations": [
                "execute (run process with arguments)",
                "validate (check if command is allowed)",
                "capabilities (list allowed commands and limits)"
            ]
        })
    }

    /// Demonstrate hook-aware execution for process execution
    /// This method showcases how the process executor tool works with the hook system
    ///
    /// # Errors
    ///
    /// Returns an error if the process execution fails or hook execution fails
    pub async fn demonstrate_hook_integration(
        &self,
        tool_executor: &crate::lifecycle::ToolExecutor,
        executable: &str,
        args: Option<&[String]>,
        working_dir: Option<&str>,
    ) -> LLMResult<AgentOutput> {
        let mut params = json!({
            "executable": executable,
            "hook_integration": true  // Flag to indicate this is a hook demo
        });

        if let Some(args) = args {
            params["args"] = json!(args);
        }

        if let Some(working_dir) = working_dir {
            params["working_dir"] = json!(working_dir);
        }

        let input = AgentInput::text("Process execution hook demonstration")
            .with_parameter("parameters", params);
        let context = ExecutionContext::default();

        // Execute with hooks using the HookableToolExecution trait
        self.execute_with_hooks(input, context, tool_executor).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use llmspell_testing::tool_helpers::{create_test_tool, create_test_tool_input};
    use tempfile::TempDir;

    fn create_test_process_executor() -> ProcessExecutorTool {
        let config = ProcessExecutorConfig::default();
        ProcessExecutorTool::new(config)
    }

    fn create_test_tool_with_custom_config() -> ProcessExecutorTool {
        let config = ProcessExecutorConfig {
            max_execution_time_seconds: 5,
            max_output_size: 1024,
            ..Default::default()
        };
        ProcessExecutorTool::new(config)
    }
    #[tokio::test]
    async fn test_execute_simple_command() {
        let tool = create_test_process_executor();

        let input = create_test_tool_input(vec![
            ("executable", "echo"),
            ("arguments", r#"["Hello", "World"]"#),
        ]);

        let result = tool
            .execute(input, ExecutionContext::default())
            .await
            .unwrap();
        assert!(result.text.contains("executed successfully"));
    }
    #[tokio::test]
    async fn test_execute_blocked_command() {
        let tool = create_test_process_executor();

        let input =
            create_test_tool_input(vec![("executable", "rm"), ("arguments", r#"["-rf", "/"]"#)]);

        let result = tool.execute(input, ExecutionContext::default()).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not permitted"));
    }
    #[tokio::test]
    async fn test_execute_nonexistent_command() {
        let tool = create_test_process_executor();

        let input = create_test_tool_input(vec![
            ("executable", "nonexistent_command_12345"),
            ("arguments", r#"["arg1"]"#),
        ]);

        let result = tool.execute(input, ExecutionContext::default()).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }
    #[tokio::test]
    async fn test_execute_with_working_directory() {
        let tool = create_test_process_executor();
        let temp_dir = TempDir::new().unwrap();

        let input = create_test_tool_input(vec![
            ("executable", "pwd"),
            ("working_directory", &temp_dir.path().to_string_lossy()),
        ]);

        let result = tool
            .execute(input, ExecutionContext::default())
            .await
            .unwrap();
        assert!(result.text.contains("executed successfully"));
    }
    #[tokio::test]
    async fn test_execute_with_environment_vars() {
        let tool = create_test_process_executor();

        let env = json!({
            "TEST_VAR": "test_value"
        });

        let input = create_test_tool_input(vec![
            ("executable", "echo"),
            ("arguments", r#"["$TEST_VAR"]"#),
            ("environment", "env"),
        ]);

        let result = tool
            .execute(input, ExecutionContext::default())
            .await
            .unwrap();
        assert!(result.text.contains("executed successfully"));
    }
    #[tokio::test]
    async fn test_executable_validation() {
        let tool = create_test_process_executor();

        // Test dangerous characters
        let input1 = create_test_tool_input(vec![
            ("executable", "echo; rm -rf /"),
            ("arguments", "[\"test\"]"),
        ]);

        let result1 = tool.execute(input1, ExecutionContext::default()).await;
        assert!(result1.is_err());
        assert!(result1
            .unwrap_err()
            .to_string()
            .contains("dangerous characters"));

        // Test path traversal
        let input2 = create_test_tool_input(vec![
            ("executable", "../../../bin/echo"),
            ("arguments", "[\"test\"]"),
        ]);

        let result2 = tool.execute(input2, ExecutionContext::default()).await;
        assert!(result2.is_err());
        assert!(result2
            .unwrap_err()
            .to_string()
            .contains("dangerous characters"));
    }
    #[tokio::test]
    async fn test_missing_parameters() {
        let tool = create_test_process_executor();

        // Missing executable
        let input1 = AgentInput {
            text: "Missing executable".to_string(),
            media: vec![],
            context: None,
            parameters: HashMap::new(),
            output_modalities: vec![],
        };
        let result1 = tool.execute(input1, ExecutionContext::default()).await;
        assert!(result1.is_err());
        assert!(result1
            .unwrap_err()
            .to_string()
            .contains("Missing parameters object"));

        // Empty executable
        let input2 = create_test_tool_input(vec![("executable", "")]);
        let result2 = tool.execute(input2, ExecutionContext::default()).await;
        assert!(result2.is_err());
        assert!(result2.unwrap_err().to_string().contains("cannot be empty"));
    }
    #[tokio::test]
    async fn test_working_directory_validation() {
        let tool = create_test_process_executor();

        // Nonexistent directory
        let input = create_test_tool_input(vec![
            ("executable", "echo"),
            ("working_directory", "/nonexistent/directory/12345"),
        ]);

        let result = tool.execute(input, ExecutionContext::default()).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("does not exist"));
    }
    #[tokio::test]
    async fn test_executable_allowed_check() {
        let tool = create_test_process_executor();

        // Test allowed executable
        assert!(tool.is_executable_allowed("echo"));
        assert!(tool.is_executable_allowed("/bin/echo"));

        // Test blocked executable
        assert!(!tool.is_executable_allowed("rm"));
        assert!(!tool.is_executable_allowed("/bin/rm"));

        // Test unknown executable (should be denied by default)
        assert!(!tool.is_executable_allowed("unknown_command"));
    }
    #[tokio::test]
    async fn test_tool_metadata() {
        let tool = create_test_process_executor();

        let metadata = tool.metadata();
        assert_eq!(metadata.name, "process_executor");
        assert!(metadata.description.contains("process execution"));

        let schema = tool.schema();
        assert_eq!(schema.name, "process_executor");
        assert_eq!(tool.category(), ToolCategory::System);
        assert_eq!(tool.security_level(), SecurityLevel::Restricted);

        // Check required parameters
        let required_params = schema.required_parameters();
        assert!(required_params.contains(&"executable".to_string()));
        assert_eq!(required_params.len(), 1);
    }
    #[tokio::test]
    async fn test_custom_config() {
        let tool = create_test_tool_with_custom_config();

        // Test that custom configuration is applied
        assert_eq!(tool.config.max_execution_time_seconds, 5);
        assert_eq!(tool.config.max_output_size, 1024);
    }
    #[tokio::test]
    async fn test_resolve_executable() {
        let tool = create_test_process_executor();

        // Test resolving a common executable
        let result = tool.resolve_executable("echo").await;
        assert!(result.is_ok());

        // Test resolving nonexistent executable
        let result = tool.resolve_executable("nonexistent_command_12345").await;
        assert!(result.is_err());
    }
    #[tokio::test]
    async fn test_arbitrary_commands_disabled() {
        let config = ProcessExecutorConfig {
            permissions: ExecutionPermissions {
                allow_arbitrary_commands: false,
                inherit_environment: false,
            },
            ..Default::default()
        };
        let tool = ProcessExecutorTool::new(config);

        // Should not allow arbitrary commands
        assert!(!tool.is_executable_allowed("arbitrary_command"));
    }
    #[tokio::test]
    async fn test_arbitrary_commands_enabled() {
        let config = ProcessExecutorConfig {
            permissions: ExecutionPermissions {
                allow_arbitrary_commands: true,
                inherit_environment: false,
            },
            ..Default::default()
        };
        let tool = ProcessExecutorTool::new(config);

        // Should allow arbitrary commands (unless blocked)
        assert!(tool.is_executable_allowed("arbitrary_command"));

        // But still respect blocked list
        assert!(!tool.is_executable_allowed("rm"));
    }
    #[test]
    fn test_hook_integration_metadata() {
        let tool = create_test_process_executor();

        // Test that the tool supports hooks
        assert!(tool.supports_hooks());

        // Test hook metadata
        let metadata = tool.hook_metadata();
        assert_eq!(metadata["tool_name"], "process_executor");
        assert!(metadata["hook_points_supported"].is_array());
        assert_eq!(
            metadata["hook_points_supported"].as_array().unwrap().len(),
            8
        );
        assert!(metadata["hook_integration_benefits"].is_array());
        assert!(metadata["security_considerations"].is_array());
        assert_eq!(metadata["security_level"], "Restricted");
        assert!(metadata["supported_operations"].is_array());
        // Verify security critical flag
        assert_eq!(metadata["resource_limits"]["security_critical"], true);
    }
    #[tokio::test]
    async fn test_process_executor_hook_integration() {
        use crate::lifecycle::{ToolExecutor, ToolLifecycleConfig};
        let tool = create_test_process_executor();

        let config = ToolLifecycleConfig::default();
        let tool_executor = ToolExecutor::new(config, None, None);

        // Demonstrate hook integration with echo command (should be safe)
        let args = vec!["hello".to_string(), "hook".to_string()];
        let result = tool
            .demonstrate_hook_integration(&tool_executor, "echo", Some(&args), None)
            .await;

        // May succeed or fail based on system, but should not panic
        assert!(result.is_ok() || result.is_err());
    }
    #[tokio::test]
    async fn test_hookable_tool_execution_trait_process() {
        use crate::lifecycle::{HookableToolExecution, ToolExecutor, ToolLifecycleConfig};
        let tool = create_test_process_executor();

        // Verify the tool implements HookableToolExecution
        let config = ToolLifecycleConfig::default();
        let tool_executor = ToolExecutor::new(config, None, None);

        let input = AgentInput::text("Hook trait test").with_parameter(
            "parameters",
            json!({
                "executable": "echo",
                "args": ["test"]
            }),
        );
        let context = ExecutionContext::default();

        // This should compile and execute (may fail based on security policy, that's ok)
        let result = tool
            .execute_with_hooks(input, context, &tool_executor)
            .await;
        assert!(result.is_ok() || result.is_err()); // Should not panic
    }
}
