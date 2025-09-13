//! REPL session management with kernel connection
//!
//! Provides the core business logic for REPL operations, command handling,
//! and kernel communication. This is the main logic layer - CLI provides only terminal I/O.

use anyhow::Result;
use serde_json::Value;
use std::collections::HashMap;
use std::fmt::Write as _;
use std::time::Instant;

/// Trait for kernel connections (will be provided by llmspell-cli's `kernel_client`)
#[async_trait::async_trait]
pub trait KernelConnection: Send + Sync {
    async fn connect_or_start(&mut self) -> Result<()>;
    async fn execute(&mut self, code: &str) -> Result<String>;
    async fn send_debug_command(&mut self, command: Value) -> Result<Value>;
    async fn disconnect(&mut self) -> Result<()>;
    fn is_connected(&self) -> bool;
    fn classify_workload(&self, operation: &str) -> WorkloadType;
    fn execution_manager(&self) -> Option<&dyn std::any::Any>;
}

/// Workload classification for performance monitoring
#[derive(Debug, Clone, Copy)]
pub enum WorkloadType {
    Micro,  // <10ms expected
    Light,  // <100ms expected
    Medium, // <1s expected
    Heavy,  // >1s expected
}

/// Configuration for REPL session
pub struct ReplConfig {
    pub enable_performance_monitoring: bool,
    pub enable_debug_commands: bool,
}

impl Default for ReplConfig {
    fn default() -> Self {
        Self {
            enable_performance_monitoring: true,
            enable_debug_commands: true,
        }
    }
}

/// REPL session manager - core business logic
pub struct ReplSession {
    kernel: Box<dyn KernelConnection>,
    config: ReplConfig,
    execution_count: u32,
    command_history: Vec<String>,
    variables: HashMap<String, Value>,
}

impl ReplSession {
    /// Create a new REPL session
    ///
    /// # Errors
    ///
    /// Returns an error if the kernel is not connected.
    pub async fn new(kernel: Box<dyn KernelConnection>, config: ReplConfig) -> Result<Self> {
        let mut session = Self {
            kernel,
            config,
            execution_count: 0,
            command_history: Vec::new(),
            variables: HashMap::new(),
        };

        // Connect to kernel
        session.kernel.connect_or_start().await?;

        Ok(session)
    }

    /// Handle user input - main entry point
    ///
    /// # Errors
    ///
    /// Returns an error if command execution fails.
    pub async fn handle_input(&mut self, input: &str) -> Result<ReplResponse> {
        let input = input.trim();

        if input.is_empty() {
            return Ok(ReplResponse::Empty);
        }

        // Record in history
        self.command_history.push(input.to_string());

        // Check if it's a command or code
        if input.starts_with('.') {
            self.handle_command(input).await
        } else {
            self.execute_code(input).await
        }
    }

    /// Execute code on the kernel
    async fn execute_code(&mut self, code: &str) -> Result<ReplResponse> {
        self.execution_count += 1;

        // Performance monitoring
        let start = if self.config.enable_performance_monitoring {
            Some(Instant::now())
        } else {
            None
        };

        // Classify workload
        let workload = self.kernel.classify_workload(if code.lines().count() > 10 {
            "execute_block"
        } else {
            "execute_line"
        });

        // Execute on kernel
        let result = self.kernel.execute(code).await?;

        // Check performance
        if let Some(start_time) = start {
            let duration = start_time.elapsed();
            Self::check_performance(workload, duration);
        }

        Ok(ReplResponse::ExecutionResult {
            output: result,
            execution_count: self.execution_count,
        })
    }

    /// Handle REPL commands
    async fn handle_command(&mut self, command: &str) -> Result<ReplResponse> {
        let parts: Vec<&str> = command.split_whitespace().collect();
        let cmd = parts.first().copied().unwrap_or("");

        match cmd {
            ".help" => Ok(ReplResponse::Help(self.get_help_text())),
            ".exit" | ".quit" => Ok(ReplResponse::Exit),
            ".vars" => Ok(self.handle_vars_command()),
            ".clear" => Ok(self.handle_clear_command()),
            ".history" => Ok(self.handle_history_command()),
            ".info" => Ok(self.handle_info_command()),
            ".state" => self.handle_state_command(&parts).await,
            ".session" => self.handle_session_command(&parts).await,

            // Debug commands (if enabled)
            ".break" if self.config.enable_debug_commands => {
                self.handle_breakpoint_command(&parts).await
            }
            ".step" if self.config.enable_debug_commands => self.handle_step_command().await,
            ".continue" if self.config.enable_debug_commands => {
                self.handle_continue_command().await
            }
            ".locals" if self.config.enable_debug_commands => self.handle_locals_command().await,
            ".globals" if self.config.enable_debug_commands => self.handle_globals_command().await,
            ".upvalues" if self.config.enable_debug_commands => {
                self.handle_upvalues_command().await
            }
            ".stack" if self.config.enable_debug_commands => self.handle_stack_command().await,
            ".watch" if self.config.enable_debug_commands => Ok(Self::handle_watch_command(&parts)),

            _ => Ok(ReplResponse::Error(format!("Unknown command: {cmd}"))),
        }
    }

    /// Handle breakpoint command
    async fn handle_breakpoint_command(&mut self, parts: &[&str]) -> Result<ReplResponse> {
        if parts.len() < 3 {
            return Ok(ReplResponse::Error(
                "Usage: .break <file> <line>".to_string(),
            ));
        }

        let file = parts[1];
        let line: u32 = parts[2]
            .parse()
            .map_err(|_| anyhow::anyhow!("Invalid line number"))?;

        let request = serde_json::json!({
            "command": "setBreakpoints",
            "arguments": {
                "source": {
                    "name": file,
                    "path": file
                },
                "lines": [line]
            }
        });

        let response = self.kernel.send_debug_command(request).await?;
        Ok(ReplResponse::DebugResponse(response))
    }

    /// Handle step command
    async fn handle_step_command(&mut self) -> Result<ReplResponse> {
        let request = serde_json::json!({
            "command": "stepIn",
            "arguments": {
                "threadId": 1
            }
        });

        let response = self.kernel.send_debug_command(request).await?;
        Ok(ReplResponse::DebugResponse(response))
    }

    /// Handle continue command
    async fn handle_continue_command(&mut self) -> Result<ReplResponse> {
        let request = serde_json::json!({
            "command": "continue",
            "arguments": {
                "threadId": 1
            }
        });

        let response = self.kernel.send_debug_command(request).await?;
        Ok(ReplResponse::DebugResponse(response))
    }

    /// Handle locals command
    async fn handle_locals_command(&mut self) -> Result<ReplResponse> {
        // Create DAP request for variables
        let dap_request = serde_json::json!({
            "seq": 1,
            "type": "request",
            "command": "variables",
            "arguments": {
                "variablesReference": 1000,  // 1000 = locals for current frame
            }
        });

        let response = self.kernel.send_debug_command(dap_request).await?;

        // Check if request was successful
        if response.get("success").and_then(serde_json::Value::as_bool) != Some(true) {
            let error_msg = response
                .get("message")
                .and_then(|m| m.as_str())
                .unwrap_or("Failed to get variables");
            return Ok(ReplResponse::Error(error_msg.to_string()));
        }

        // Extract variables from response
        let variables = response["body"]["variables"]
            .as_array()
            .ok_or_else(|| anyhow::anyhow!("Invalid response format"))?;

        if variables.is_empty() {
            return Ok(ReplResponse::Info(
                "No local variables in current scope".to_string(),
            ));
        }

        let mut output = String::from("Local variables:\n");
        for var in variables {
            let name = var["name"].as_str().unwrap_or("?");
            let value = var["value"].as_str().unwrap_or("?");
            let var_type = var["type"].as_str().unwrap_or("unknown");

            // Handle special characters in variable names
            let formatted_name = if name.contains(|c: char| !c.is_alphanumeric() && c != '_')
                || name.chars().next().is_some_and(char::is_numeric)
            {
                // Quote names with special characters or starting with numbers
                format!("[\"{}\"]", name.replace('\\', "\\\\").replace('"', "\\\""))
            } else {
                name.to_string()
            };

            writeln!(output, "  {formatted_name} = {value} ({var_type})").unwrap();
        }

        Ok(ReplResponse::Info(output))
    }

    /// Handle globals command
    async fn handle_globals_command(&mut self) -> Result<ReplResponse> {
        // Create DAP request for global variables
        let dap_request = serde_json::json!({
            "seq": 1,
            "type": "request",
            "command": "variables",
            "arguments": {
                "variablesReference": 2000,  // 2000 = globals for current frame
            }
        });

        let response = self.kernel.send_debug_command(dap_request).await?;

        // Check if request was successful
        if response.get("success").and_then(serde_json::Value::as_bool) != Some(true) {
            let error_msg = response
                .get("message")
                .and_then(|m| m.as_str())
                .unwrap_or("Failed to get global variables");
            return Ok(ReplResponse::Error(error_msg.to_string()));
        }

        // Extract variables from response
        let variables = response["body"]["variables"]
            .as_array()
            .ok_or_else(|| anyhow::anyhow!("Invalid response format"))?;

        if variables.is_empty() {
            return Ok(ReplResponse::Info("No global variables".to_string()));
        }

        let mut output = String::from("Global variables:\n");
        for var in variables {
            let name = var["name"].as_str().unwrap_or("?");
            let value = var["value"].as_str().unwrap_or("?");
            let var_type = var["type"].as_str().unwrap_or("unknown");

            // Handle special characters in variable names
            let formatted_name = if name.contains(|c: char| !c.is_alphanumeric() && c != '_')
                || name.chars().next().is_some_and(char::is_numeric)
            {
                // Quote names with special characters or starting with numbers
                format!("[\"{}\"]", name.replace('\\', "\\\\").replace('"', "\\\""))
            } else {
                name.to_string()
            };

            writeln!(output, "  {formatted_name} = {value} ({var_type})").unwrap();
        }

        Ok(ReplResponse::Info(output))
    }

    /// Handle upvalues command
    async fn handle_upvalues_command(&mut self) -> Result<ReplResponse> {
        // Create DAP request for upvalues/closure variables
        let dap_request = serde_json::json!({
            "seq": 1,
            "type": "request",
            "command": "variables",
            "arguments": {
                "variablesReference": 3000,  // 3000 = upvalues for current frame
            }
        });

        let response = self.kernel.send_debug_command(dap_request).await?;

        // Check if request was successful
        if response.get("success").and_then(serde_json::Value::as_bool) != Some(true) {
            let error_msg = response
                .get("message")
                .and_then(|m| m.as_str())
                .unwrap_or("Failed to get upvalues");
            return Ok(ReplResponse::Error(error_msg.to_string()));
        }

        // Extract variables from response
        let variables = response["body"]["variables"]
            .as_array()
            .ok_or_else(|| anyhow::anyhow!("Invalid response format"))?;

        if variables.is_empty() {
            return Ok(ReplResponse::Info(
                "No upvalues in current scope".to_string(),
            ));
        }

        let mut output = String::from("Upvalues (closure variables):\n");
        for var in variables {
            let name = var["name"].as_str().unwrap_or("?");
            let value = var["value"].as_str().unwrap_or("?");
            let var_type = var["type"].as_str().unwrap_or("unknown");

            // Handle special characters in variable names
            let formatted_name = if name.contains(|c: char| !c.is_alphanumeric() && c != '_')
                || name.chars().next().is_some_and(char::is_numeric)
            {
                // Quote names with special characters or starting with numbers
                format!("[\"{}\"]", name.replace('\\', "\\\\").replace('"', "\\\""))
            } else {
                name.to_string()
            };

            writeln!(output, "  {formatted_name} = {value} ({var_type})").unwrap();
        }

        Ok(ReplResponse::Info(output))
    }

    /// Handle stack command
    async fn handle_stack_command(&mut self) -> Result<ReplResponse> {
        let request = serde_json::json!({
            "command": "stackTrace",
            "arguments": {
                "threadId": 1,
                "startFrame": 0,
                "levels": 20
            }
        });

        let response = self.kernel.send_debug_command(request).await?;
        Ok(ReplResponse::DebugResponse(response))
    }

    /// Handle watch command
    fn handle_watch_command(parts: &[&str]) -> ReplResponse {
        if parts.len() < 2 {
            return ReplResponse::Error("Usage: .watch <expression>".to_string());
        }

        let expression = parts[1..].join(" ");
        ReplResponse::Info(format!("Watching expression: {expression}"))
    }

    /// Handle vars command - show variables
    fn handle_vars_command(&self) -> ReplResponse {
        if self.variables.is_empty() {
            ReplResponse::Info("No variables defined".to_string())
        } else {
            let vars: Vec<String> = self
                .variables
                .iter()
                .map(|(k, v)| format!("{k}: {v}"))
                .collect();
            ReplResponse::Info(vars.join("\n"))
        }
    }

    /// Handle clear command
    fn handle_clear_command(&mut self) -> ReplResponse {
        self.variables.clear();
        ReplResponse::Info("Variables cleared".to_string())
    }

    /// Handle history command
    fn handle_history_command(&self) -> ReplResponse {
        if self.command_history.is_empty() {
            ReplResponse::Info("No command history".to_string())
        } else {
            let history: Vec<String> = self
                .command_history
                .iter()
                .enumerate()
                .map(|(i, cmd)| format!("{:4}: {}", i + 1, cmd))
                .collect();
            ReplResponse::Info(history.join("\n"))
        }
    }

    /// Handle info command
    fn handle_info_command(&self) -> ReplResponse {
        let connected = if self.kernel.is_connected() {
            "Connected"
        } else {
            "Disconnected"
        };

        let info = format!(
            "Kernel status: {}\nExecution count: {}\nHistory entries: {}",
            connected,
            self.execution_count,
            self.command_history.len()
        );

        ReplResponse::Info(info)
    }

    /// Get help text
    fn get_help_text(&self) -> String {
        let mut help = vec![
            "Available commands:",
            "  .help              - Show this help message",
            "  .exit/.quit        - Exit REPL",
            "  .vars              - Show defined variables",
            "  .clear             - Clear variables",
            "  .history           - Show command history",
            "  .info              - Show session info",
            "  .state [key]       - Show persistent state",
            "  .session           - Show current session info",
        ];

        if self.config.enable_debug_commands {
            help.extend_from_slice(&[
                "  .break <file> <ln> - Set breakpoint",
                "  .step              - Step to next line",
                "  .continue          - Continue execution",
                "  .locals            - Show local variables",
                "  .globals           - Show global variables",
                "  .upvalues          - Show closure variables",
                "  .stack             - Show call stack",
                "  .watch <expr>      - Watch expression",
            ]);
        }

        help.push("");
        help.push("Enter any other text to execute as code");

        help.join("\n")
    }

    /// Handle state command
    async fn handle_state_command(&mut self, parts: &[&str]) -> Result<ReplResponse> {
        // Get optional key from command
        let key = parts.get(1).map(|s| (*s).to_string());

        // Execute state retrieval code
        let code = key.as_ref().map_or_else(
            || {
                "
                local keys = State.keys()
                if keys and #keys > 0 then
                    print(\"State keys (\" .. #keys .. \" total):\")
                    for i, k in ipairs(keys) do
                        local v = State.get(k)
                        print(\"  \" .. k .. \" = \" .. tostring(v))
                    end
                else
                    print(\"No state keys found\")
                end
            "
                .to_string()
            },
            |k| {
                format!(
                    "
                local value = State.get(\"{k}\")
                if value ~= nil then
                    print(\"State['{k}'] = \" .. tostring(value))
                else
                    print(\"State key '{k}' not found\")
                end
            "
                )
            },
        );

        // Execute the code
        let result = self.kernel.execute(&code).await?;
        Ok(ReplResponse::Info(result))
    }

    /// Handle session command
    async fn handle_session_command(&mut self, _parts: &[&str]) -> Result<ReplResponse> {
        // Get session info from kernel
        let code = "
            if Session and Session.info then
                local info = Session.info()
                print(\"Session Info:\")
                print(\"  ID: \" .. tostring(info.id or 'unknown'))
                print(\"  Created: \" .. tostring(info.created or 'unknown'))
                print(\"  Execution Count: \" .. tostring(info.execution_count or 0))
            else
                print(\"Session information not available\")
            end
        ";

        let result = self.kernel.execute(code).await?;
        Ok(ReplResponse::Info(result))
    }

    /// Check performance against workload expectations
    fn check_performance(workload: WorkloadType, duration: std::time::Duration) {
        let threshold = match workload {
            WorkloadType::Micro => std::time::Duration::from_millis(10),
            WorkloadType::Light => std::time::Duration::from_millis(100),
            WorkloadType::Medium => std::time::Duration::from_secs(1),
            WorkloadType::Heavy => std::time::Duration::from_secs(10),
        };

        if duration > threshold {
            tracing::warn!(
                "{:?} operation took {:?} (expected < {:?})",
                workload,
                duration,
                threshold
            );
        }
    }

    /// Disconnect from kernel on drop
    ///
    /// # Errors
    ///
    /// Returns an error if kernel disconnection fails.
    pub async fn disconnect(&mut self) -> Result<()> {
        self.kernel.disconnect().await
    }
}

/// Response types from REPL operations
#[derive(Debug)]
pub enum ReplResponse {
    Empty,
    Exit,
    Help(String),
    Info(String),
    Error(String),
    ExecutionResult {
        output: String,
        execution_count: u32,
    },
    DebugResponse(Value),
}

impl ReplResponse {
    /// Format response for display
    #[must_use]
    pub fn format(&self) -> String {
        match self {
            Self::Empty | Self::Exit => String::new(),
            Self::Help(text) | Self::Info(text) => text.clone(),
            Self::Error(msg) => format!("Error: {msg}"),
            Self::ExecutionResult { output, .. } => output.clone(),
            Self::DebugResponse(value) => {
                serde_json::to_string_pretty(value).unwrap_or_else(|_| format!("{value:?}"))
            }
        }
    }

    /// Check if this response should exit the REPL
    #[must_use]
    pub const fn should_exit(&self) -> bool {
        matches!(self, Self::Exit)
    }
}
