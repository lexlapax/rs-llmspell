//! Debug Bridge - Hybrid local/protocol debugging support
//!
//! Implements the Bridge Pattern to support both current local debugging
//! and future protocol-based debugging (Task 9.7). Integrates existing
//! debug infrastructure while providing `MessageProcessor` interface.

#![allow(clippy::missing_errors_doc)]
#![allow(clippy::format_push_string)]
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::significant_drop_tightening)]
#![allow(clippy::needless_pass_by_value)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::option_if_let_else)]
#![allow(clippy::useless_format)]

use crate::processor::{MessageProcessor, ProcessorError};
use crate::protocol::{
    ldp::{LDPRequest, LDPResponse},
    lrp::{LRPRequest, LRPResponse},
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;

/// Debug execution modes supported by `DebugBridge`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DebugMode {
    /// Local debugging - direct execution with debug hooks (current)
    Local(LocalDebugConfig),
    /// Protocol debugging - via LRP/LDP protocols (Task 9.7 ready)
    Protocol(ProtocolDebugConfig),
}

/// Configuration for local debugging mode
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalDebugConfig {
    /// Script path for debugging
    pub script_path: Option<std::path::PathBuf>,
    /// Enable breakpoints
    pub enable_breakpoints: bool,
    /// Enable step debugging
    pub enable_stepping: bool,
    /// Enable variable inspection
    pub enable_variable_inspection: bool,
    /// Enable stack navigation
    pub enable_stack_navigation: bool,
    /// Performance optimization settings
    pub performance: PerformanceConfig,
}

/// Configuration for protocol debugging mode (Task 9.7)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolDebugConfig {
    /// Protocol endpoint configuration
    pub endpoint: String,
    /// Connection timeout in milliseconds
    pub connection_timeout: u64,
    /// Request timeout in milliseconds  
    pub request_timeout: u64,
}

/// Performance configuration and monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    /// Target initialization time in milliseconds (should be <10ms)
    pub init_target_ms: u64,
    /// Target state operation time in milliseconds (should be <1ms)
    pub state_target_ms: u64,
    /// Enable performance monitoring
    pub monitoring_enabled: bool,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            init_target_ms: 10, // <10ms target
            state_target_ms: 1, // <1ms target
            monitoring_enabled: true,
        }
    }
}

impl Default for LocalDebugConfig {
    fn default() -> Self {
        Self {
            script_path: None,
            enable_breakpoints: true,
            enable_stepping: true,
            enable_variable_inspection: true,
            enable_stack_navigation: true,
            performance: PerformanceConfig::default(),
        }
    }
}

/// Main `DebugBridge` configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugConfig {
    /// Debug execution mode
    pub mode: DebugMode,
    /// Performance settings
    pub performance: PerformanceConfig,
}

/// Performance monitoring for debug operations
#[derive(Debug)]
pub struct DebugPerformanceMonitor {
    config: PerformanceConfig,
    init_times: Vec<std::time::Duration>,
    state_op_times: Vec<std::time::Duration>,
}

impl DebugPerformanceMonitor {
    const fn new(config: PerformanceConfig) -> Self {
        Self {
            config,
            init_times: Vec::new(),
            state_op_times: Vec::new(),
        }
    }

    fn record_init(&mut self, duration: std::time::Duration) {
        if self.config.monitoring_enabled {
            self.init_times.push(duration);

            // Check performance target
            if duration.as_millis() > u128::from(self.config.init_target_ms) {
                tracing::warn!(
                    "Debug initialization exceeded target: {}ms > {}ms",
                    duration.as_millis(),
                    self.config.init_target_ms
                );
            }
        }
    }

    fn record_state_op(&mut self, duration: std::time::Duration) {
        if self.config.monitoring_enabled {
            self.state_op_times.push(duration);

            // Check performance target
            if duration.as_millis() > u128::from(self.config.state_target_ms) {
                tracing::warn!(
                    "Debug state operation exceeded target: {}ms > {}ms",
                    duration.as_millis(),
                    self.config.state_target_ms
                );
            }
        }
    }

    /// Get average initialization time
    #[must_use]
    pub fn avg_init_time(&self) -> Option<std::time::Duration> {
        if self.init_times.is_empty() {
            None
        } else {
            let sum: std::time::Duration = self.init_times.iter().sum();
            #[allow(clippy::cast_possible_truncation)]
            Some(sum / (self.init_times.len() as u32))
        }
    }

    /// Get average state operation time
    #[must_use]
    pub fn avg_state_op_time(&self) -> Option<std::time::Duration> {
        if self.state_op_times.is_empty() {
            None
        } else {
            let sum: std::time::Duration = self.state_op_times.iter().sum();
            #[allow(clippy::cast_possible_truncation)]
            Some(sum / (self.state_op_times.len() as u32))
        }
    }
}

/// Debug session state for tracking active debugging
#[derive(Debug, Clone)]
pub struct DebugSession {
    /// Unique session identifier
    pub session_id: String,
    /// Script being debugged
    pub script_content: String,
    /// Current debug state
    pub state: DebugSessionState,
    /// Session start time
    pub start_time: std::time::Instant,
}

/// Debug session states
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DebugSessionState {
    /// Session initialized but not started
    Initialized,
    /// Debugging in progress
    Active,
    /// Paused at breakpoint or step
    Paused,
    /// Session completed
    Completed,
    /// Session failed with error
    Failed,
}

/// Enhanced error information with source context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedError {
    /// Original error message
    pub message: String,
    /// Error type classification
    pub error_type: ErrorType,
    /// Source location information
    pub location: Option<SourceLocation>,
    /// Context lines around the error
    pub source_context: Option<SourceContext>,
    /// Helpful suggestions for fixing the error
    pub suggestions: Vec<String>,
    /// Related documentation or references
    pub references: Vec<String>,
}

/// Error type classification for better suggestions
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorType {
    /// Syntax errors in script code
    Syntax,
    /// Runtime errors during execution
    Runtime,
    /// Type-related errors
    Type,
    /// Variable or function not found
    Reference,
    /// Logic errors or unexpected behavior
    Logic,
    /// Performance-related issues
    Performance,
    /// Configuration or setup errors
    Configuration,
    /// Unknown or unclassified error
    Unknown,
}

/// Source location information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceLocation {
    /// File path (if available)
    pub file_path: Option<String>,
    /// Line number (1-based)
    pub line: usize,
    /// Column number (1-based)
    pub column: Option<usize>,
    /// Function or scope name (if available)
    pub function_name: Option<String>,
}

/// Source code context around an error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceContext {
    /// Lines before the error (with line numbers)
    pub before_lines: Vec<(usize, String)>,
    /// The error line (with line number)
    pub error_line: (usize, String),
    /// Lines after the error (with line numbers)
    pub after_lines: Vec<(usize, String)>,
    /// Highlighted column range (if available)
    pub highlight_range: Option<(usize, usize)>,
}

/// DebugBridge - main component implementing Bridge Pattern
///
/// Provides hybrid local/protocol debugging with seamless Task 9.7 transition.
/// Integrates existing debug infrastructure while implementing MessageProcessor.
#[derive(Debug)]
pub struct DebugBridge {
    /// Current debug mode configuration
    mode: DebugMode,
    /// Performance monitoring
    performance_monitor: Arc<RwLock<DebugPerformanceMonitor>>,
    /// Active debug sessions
    sessions: Arc<RwLock<std::collections::HashMap<String, DebugSession>>>,
}

impl DebugBridge {
    /// Create new DebugBridge with configuration
    ///
    /// Optimized for <10ms initialization time target.
    pub async fn new(config: DebugConfig) -> Result<Self, ProcessorError> {
        let start_time = Instant::now();

        let performance_monitor = Arc::new(RwLock::new(DebugPerformanceMonitor::new(
            config.performance.clone(),
        )));

        let bridge = Self {
            mode: config.mode,
            performance_monitor: performance_monitor.clone(),
            sessions: Arc::new(RwLock::new(std::collections::HashMap::new())),
        };

        // Record initialization time
        let init_duration = start_time.elapsed();
        let mut monitor = performance_monitor.write().await;
        monitor.record_init(init_duration);

        tracing::debug!(
            "DebugBridge initialized in {}ms (target: {}ms)",
            init_duration.as_millis(),
            monitor.config.init_target_ms
        );

        Ok(bridge)
    }

    /// Start local debug session - optimized for current CLI usage
    ///
    /// This method provides immediate debugging without protocol overhead.
    pub async fn debug_local(&self, script: &str) -> Result<DebugSession, ProcessorError> {
        let start_time = Instant::now();

        match &self.mode {
            DebugMode::Local(_config) => {
                let session_id = uuid::Uuid::new_v4().to_string();

                let session = DebugSession {
                    session_id: session_id.clone(),
                    script_content: script.to_string(),
                    state: DebugSessionState::Initialized,
                    start_time: Instant::now(),
                };

                // Store session
                {
                    let mut sessions = self.sessions.write().await;
                    sessions.insert(session_id.clone(), session.clone());
                }

                // Record state operation time
                let op_duration = start_time.elapsed();
                {
                    let mut monitor = self.performance_monitor.write().await;
                    monitor.record_state_op(op_duration);
                }

                tracing::info!(
                    "Local debug session created: {} ({}ms)",
                    session_id,
                    op_duration.as_millis()
                );

                // TODO: Integrate with existing debug infrastructure
                // - Use existing DebugSessionManager
                // - Connect to VariableInspector
                // - Setup StackNavigator
                // - Initialize ExecutionManager with debug hooks

                Ok(session)
            }
            DebugMode::Protocol(_) => Err(ProcessorError::InvalidRequest(
                "Local debugging not available in protocol mode".to_string(),
            )),
        }
    }

    /// Handle protocol-based debugging - prepared for Task 9.7
    ///
    /// This method will be activated when CLI switches to kernel-hub architecture.
    pub fn debug_protocol(&self, request: LDPRequest) -> Result<LDPResponse, ProcessorError> {
        match &self.mode {
            DebugMode::Protocol(_config) => {
                // Task 9.7: This will handle protocol-based debugging
                match request {
                    LDPRequest::InitializeRequest { .. } => Ok(LDPResponse::InitializeResponse {
                        capabilities: serde_json::json!({
                            "supportsConfigurationDoneRequest": true,
                            "supportsFunctionBreakpoints": true,
                            "supportsConditionalBreakpoints": true,
                            "supportsStepBack": false,
                            "supportsSetVariable": true,
                            "supportsRestartFrame": false,
                        }),
                    }),
                    _ => Err(ProcessorError::NotImplemented(
                        "Protocol debugging request not yet implemented".to_string(),
                    )),
                }
            }
            DebugMode::Local(_) => Err(ProcessorError::InvalidRequest(
                "Protocol debugging not available in local mode".to_string(),
            )),
        }
    }

    /// Switch from local to protocol mode - Task 9.7 transition support
    ///
    /// This method enables seamless migration when CLI adopts kernel-hub architecture.
    pub async fn switch_to_protocol_mode(
        &mut self,
        protocol_config: ProtocolDebugConfig,
    ) -> Result<(), ProcessorError> {
        tracing::info!("Switching DebugBridge to protocol mode for Task 9.7 transition");

        // Gracefully stop any local sessions
        {
            let mut sessions = self.sessions.write().await;
            for (session_id, session) in sessions.iter_mut() {
                if matches!(
                    session.state,
                    DebugSessionState::Active | DebugSessionState::Paused
                ) {
                    session.state = DebugSessionState::Completed;
                    tracing::info!(
                        "Completed local debug session {} for protocol transition",
                        session_id
                    );
                }
            }
        }

        // Switch to protocol mode
        self.mode = DebugMode::Protocol(protocol_config);

        tracing::info!("DebugBridge successfully switched to protocol mode");
        Ok(())
    }

    /// Get current debug sessions
    pub async fn get_sessions(&self) -> std::collections::HashMap<String, DebugSession> {
        self.sessions.read().await.clone()
    }

    /// Get performance statistics
    pub async fn get_performance_stats(
        &self,
    ) -> (Option<std::time::Duration>, Option<std::time::Duration>) {
        let monitor = self.performance_monitor.read().await;
        (monitor.avg_init_time(), monitor.avg_state_op_time())
    }

    /// Check if DebugBridge is ready for Task 9.7 protocol mode
    #[must_use]
    pub fn is_protocol_ready(&self) -> bool {
        matches!(self.mode, DebugMode::Protocol(_))
    }

    /// Get debug capabilities for introspection
    pub fn capabilities(&self) -> Vec<String> {
        let mut caps = vec!["local_debugging".to_string()];

        match &self.mode {
            DebugMode::Local(config) => {
                if config.enable_breakpoints {
                    caps.push("breakpoints".to_string());
                }
                if config.enable_stepping {
                    caps.push("stepping".to_string());
                }
                if config.enable_variable_inspection {
                    caps.push("variable_inspection".to_string());
                }
                if config.enable_stack_navigation {
                    caps.push("stack_navigation".to_string());
                }
            }
            DebugMode::Protocol(_) => {
                caps.push("protocol_debugging".to_string());
                caps.push("ldp_support".to_string());
            }
        }

        caps.push("enhanced_error_reporting".to_string());
        caps.push("source_context".to_string());
        caps
    }

    /// Create enhanced error with source context and suggestions
    ///
    /// Provides detailed error information with context lines and helpful suggestions
    /// for debugging script issues.
    pub fn create_enhanced_error(
        &self,
        error_message: &str,
        script_content: &str,
        line_number: Option<usize>,
        column: Option<usize>,
        file_path: Option<String>,
    ) -> EnhancedError {
        let error_type = Self::classify_error(error_message);
        let location = line_number.map(|line| SourceLocation {
            file_path: file_path.clone(),
            line,
            column,
            function_name: None, // TODO: Extract from script analysis
        });

        let source_context = if let Some(line_num) = line_number {
            Self::extract_source_context(script_content, line_num, 3)
        } else {
            None
        };

        let suggestions = Self::generate_suggestions(&error_type, error_message, script_content);
        let references = Self::get_error_references(&error_type);

        EnhancedError {
            message: error_message.to_string(),
            error_type,
            location,
            source_context,
            suggestions,
            references,
        }
    }

    /// Format enhanced error for display with colors and highlighting
    ///
    /// Returns a formatted string with source context, suggestions, and references.
    pub fn format_enhanced_error(&self, error: &EnhancedError) -> String {
        let mut output = String::new();

        // Error header
        output.push_str(&format!(
            "🔍 Debug Error ({})\n",
            Self::format_error_type(&error.error_type)
        ));
        output.push_str(&format!(
            "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n"
        ));

        // Error message
        output.push_str(&format!("❌ {}\n\n", error.message));

        // Location information
        if let Some(location) = &error.location {
            output.push_str(&format!("📍 Location: "));
            if let Some(file) = &location.file_path {
                output.push_str(&format!("{}:", file));
            }
            output.push_str(&format!("{}:", location.line));
            if let Some(col) = location.column {
                output.push_str(&format!("{}:", col));
            }
            output.push('\n');

            if let Some(func) = &location.function_name {
                output.push_str(&format!("🔧 Function: {}\n", func));
            }
            output.push('\n');
        }

        // Source context
        if let Some(context) = &error.source_context {
            output.push_str("📄 Source Context:\n");
            output.push_str("┌─────────────────────────────────────────────────────────\n");

            // Before lines
            for (line_num, line_content) in &context.before_lines {
                output.push_str(&format!("│ {:4} │ {}\n", line_num, line_content));
            }

            // Error line (highlighted)
            let (error_line_num, error_line_content) = &context.error_line;
            output.push_str(&format!(
                "│ {:4} ▶ {}\n",
                error_line_num, error_line_content
            ));

            // Column highlighting
            if let Some((start, end)) = context.highlight_range {
                let spaces = " ".repeat(8 + start); // Indent to align with content
                let carets = "^".repeat(end - start + 1);
                output.push_str(&format!("│      │ {}{}\n", spaces, carets));
            }

            // After lines
            for (line_num, line_content) in &context.after_lines {
                output.push_str(&format!("│ {:4} │ {}\n", line_num, line_content));
            }

            output.push_str("└─────────────────────────────────────────────────────────\n\n");
        }

        // Suggestions
        if !error.suggestions.is_empty() {
            output.push_str("💡 Suggestions:\n");
            for (i, suggestion) in error.suggestions.iter().enumerate() {
                output.push_str(&format!("   {}. {}\n", i + 1, suggestion));
            }
            output.push('\n');
        }

        // References
        if !error.references.is_empty() {
            output.push_str("📚 References:\n");
            for reference in &error.references {
                output.push_str(&format!("   • {}\n", reference));
            }
            output.push('\n');
        }

        output.push_str("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

        output
    }

    /// Classify error type based on error message patterns
    fn classify_error(error_message: &str) -> ErrorType {
        let msg_lower = error_message.to_lowercase();

        if msg_lower.contains("syntax")
            || msg_lower.contains("parse")
            || msg_lower.contains("unexpected")
        {
            ErrorType::Syntax
        } else if msg_lower.contains("undefined")
            || msg_lower.contains("not found")
            || msg_lower.contains("unknown")
        {
            ErrorType::Reference
        } else if msg_lower.contains("type") || msg_lower.contains("expected") {
            ErrorType::Type
        } else if msg_lower.contains("timeout")
            || msg_lower.contains("performance")
            || msg_lower.contains("slow")
        {
            ErrorType::Performance
        } else if msg_lower.contains("config")
            || msg_lower.contains("setting")
            || msg_lower.contains("option")
        {
            ErrorType::Configuration
        } else if msg_lower.contains("runtime") || msg_lower.contains("execution") {
            ErrorType::Runtime
        } else {
            ErrorType::Unknown
        }
    }

    /// Extract source context around error line
    fn extract_source_context(
        script_content: &str,
        error_line: usize,
        context_size: usize,
    ) -> Option<SourceContext> {
        let lines: Vec<&str> = script_content.lines().collect();
        if error_line == 0 || error_line > lines.len() {
            return None;
        }

        let error_idx = error_line - 1; // Convert to 0-based

        // Get before lines
        let before_start = error_idx.saturating_sub(context_size);
        let before_lines: Vec<(usize, String)> = lines[before_start..error_idx]
            .iter()
            .enumerate()
            .map(|(i, line)| (before_start + i + 1, (*line).to_string()))
            .collect();

        // Get error line
        let error_line_content = (*lines.get(error_idx)?).to_string();

        // Get after lines
        let after_end = std::cmp::min(error_idx + context_size + 1, lines.len());
        let after_lines: Vec<(usize, String)> = lines[(error_idx + 1)..after_end]
            .iter()
            .enumerate()
            .map(|(i, line)| (error_idx + i + 2, (*line).to_string()))
            .collect();

        Some(SourceContext {
            before_lines,
            error_line: (error_line, error_line_content),
            after_lines,
            highlight_range: None, // TODO: Parse column information
        })
    }

    /// Generate helpful suggestions based on error type and content
    fn generate_suggestions(
        error_type: &ErrorType,
        error_message: &str,
        script_content: &str,
    ) -> Vec<String> {
        let mut suggestions = Vec::new();

        match error_type {
            ErrorType::Syntax => {
                suggestions.push(
                    "Check for missing or mismatched parentheses, brackets, or quotes".to_string(),
                );
                suggestions.push("Verify proper indentation and block structure".to_string());
                suggestions.push("Look for typos in keywords or function names".to_string());

                if error_message.contains("end") {
                    suggestions.push("Add missing 'end' statement to close a block".to_string());
                }
                if error_message.contains("'='") {
                    suggestions.push(
                        "Check if you meant '==' for comparison instead of '=' for assignment"
                            .to_string(),
                    );
                }
            }
            ErrorType::Reference => {
                suggestions
                    .push("Check if the variable or function is defined before use".to_string());
                suggestions.push("Verify correct spelling of variable/function names".to_string());
                suggestions.push("Ensure the variable is in the correct scope".to_string());

                if script_content.contains("local") {
                    suggestions.push(
                        "Check if variable was declared as 'local' in the right scope".to_string(),
                    );
                }
            }
            ErrorType::Type => {
                suggestions
                    .push("Check the data type of variables and function parameters".to_string());
                suggestions
                    .push("Verify that operations are compatible with the data types".to_string());
                suggestions.push("Consider type conversion if needed".to_string());
            }
            ErrorType::Runtime => {
                suggestions.push("Check for nil values or uninitialized variables".to_string());
                suggestions.push("Verify function parameters and return values".to_string());
                suggestions.push("Add error handling for potential failure points".to_string());
            }
            ErrorType::Performance => {
                suggestions.push("Check for infinite loops or recursive calls".to_string());
                suggestions.push("Consider optimizing data structures or algorithms".to_string());
                suggestions.push("Add timeouts or limits to prevent hangs".to_string());
            }
            ErrorType::Configuration => {
                suggestions.push("Verify configuration file syntax and values".to_string());
                suggestions.push("Check if required settings are missing".to_string());
                suggestions.push("Ensure file paths and permissions are correct".to_string());
            }
            ErrorType::Logic => {
                suggestions.push("Review the logic flow and conditions".to_string());
                suggestions.push("Add debug prints to trace execution path".to_string());
                suggestions.push("Test edge cases and boundary conditions".to_string());
            }
            ErrorType::Unknown => {
                suggestions.push("Check the script syntax for basic errors".to_string());
                suggestions.push("Try running parts of the script in isolation".to_string());
                suggestions
                    .push("Look for recent changes that might have caused the issue".to_string());
            }
        }

        // Add specific suggestions based on error message content
        if error_message.contains("attempt to index") {
            suggestions.push("Check if the variable is a table before indexing".to_string());
            suggestions.push("Verify the table key exists before accessing it".to_string());
        }
        if error_message.contains("attempt to call") {
            suggestions.push("Check if the variable is actually a function".to_string());
            suggestions.push("Verify the function name is spelled correctly".to_string());
        }

        suggestions
    }

    /// Get documentation references for error type
    fn get_error_references(error_type: &ErrorType) -> Vec<String> {
        match error_type {
            ErrorType::Syntax => vec![
                "Lua syntax reference: https://www.lua.org/manual/5.4/manual.html#9".to_string(),
                "Common Lua syntax errors: https://lua-users.org/wiki/CommonMistakes".to_string(),
            ],
            ErrorType::Reference => vec![
                "Lua variables and scope: https://www.lua.org/manual/5.4/manual.html#3.5"
                    .to_string(),
                "Understanding Lua scope: https://lua-users.org/wiki/ScopeTutorial".to_string(),
            ],
            ErrorType::Type => vec![
                "Lua data types: https://www.lua.org/manual/5.4/manual.html#2.1".to_string(),
                "Type checking in Lua: https://lua-users.org/wiki/TypeChecking".to_string(),
            ],
            ErrorType::Runtime => vec![
                "Lua error handling: https://www.lua.org/manual/5.4/manual.html#2.3".to_string(),
                "Debugging Lua programs: https://lua-users.org/wiki/DebuggingLuaCode".to_string(),
            ],
            ErrorType::Performance => vec![
                "Lua performance tips: https://lua-users.org/wiki/OptimisationTips".to_string(),
                "Profiling Lua code: https://lua-users.org/wiki/ProfilingLuaCode".to_string(),
            ],
            ErrorType::Configuration | ErrorType::Logic | ErrorType::Unknown => vec![
                "LLMSpell documentation: https://github.com/lexlapax/rs-llmspell".to_string(),
                "Lua programming guide: https://www.lua.org/pil/".to_string(),
            ],
        }
    }

    /// Format error type for display
    fn format_error_type(error_type: &ErrorType) -> String {
        match error_type {
            ErrorType::Syntax => "Syntax Error".to_string(),
            ErrorType::Runtime => "Runtime Error".to_string(),
            ErrorType::Type => "Type Error".to_string(),
            ErrorType::Reference => "Reference Error".to_string(),
            ErrorType::Logic => "Logic Error".to_string(),
            ErrorType::Performance => "Performance Issue".to_string(),
            ErrorType::Configuration => "Configuration Error".to_string(),
            ErrorType::Unknown => "Unknown Error".to_string(),
        }
    }
}

/// `MessageProcessor` implementation for protocol consistency
///
/// Enables `DebugBridge` to work seamlessly with `UnifiedProtocolEngine`
/// and provides foundation for Task 9.7 kernel integration.
#[async_trait]
impl MessageProcessor for DebugBridge {
    async fn process_lrp(&self, request: LRPRequest) -> Result<LRPResponse, ProcessorError> {
        match request {
            LRPRequest::ExecuteRequest { code, .. } => {
                // Execute with debug hooks enabled for local debugging
                // For protocol mode, this would route to kernel
                match &self.mode {
                    DebugMode::Local(_) => {
                        // TODO: Integrate with existing ScriptRuntime with debug hooks
                        tracing::debug!("Executing code with local debug hooks: {}", code);

                        Ok(LRPResponse::ExecuteReply {
                            status: "ok".to_string(),
                            execution_count: 1,
                            user_expressions: None,
                            payload: None,
                        })
                    }
                    DebugMode::Protocol(_) => {
                        // Task 9.7: Forward to kernel via protocol
                        Err(ProcessorError::NotImplemented(
                            "Protocol mode LRP forwarding not yet implemented".to_string(),
                        ))
                    }
                }
            }
            _ => Err(ProcessorError::NotImplemented(
                "LRP request type not implemented in DebugBridge".to_string(),
            )),
        }
    }

    async fn process_ldp(&self, request: LDPRequest) -> Result<LDPResponse, ProcessorError> {
        // Route to appropriate debug handler based on mode
        match &self.mode {
            DebugMode::Local(_) => {
                // Handle debug protocol requests locally
                match request {
                    LDPRequest::InitializeRequest { .. } => Ok(LDPResponse::InitializeResponse {
                        capabilities: serde_json::json!({
                            "supportsConfigurationDoneRequest": true,
                            "supportsFunctionBreakpoints": true,
                            "supportsConditionalBreakpoints": true,
                            "supportsStepBack": false,
                            "supportsSetVariable": true,
                            "supportsRestartFrame": false,
                        }),
                    }),
                    LDPRequest::SetBreakpointsRequest {
                        source,
                        breakpoints,
                        ..
                    } => {
                        let bp_count = breakpoints.as_ref().map_or(0, std::vec::Vec::len);
                        let source_path = source.path.as_deref().unwrap_or("<unknown>");
                        tracing::debug!("Setting {} breakpoints in {}", bp_count, source_path);

                        // TODO: Integrate with existing ExecutionManager
                        // for bp in breakpoints {
                        //     self.execution_manager.set_breakpoint(&source.path, bp.line, bp.condition).await?;
                        // }

                        Ok(LDPResponse::SetBreakpointsResponse {
                            breakpoints: vec![], // TODO: Return actual breakpoint info
                        })
                    }
                    _ => Err(ProcessorError::NotImplemented(
                        "LDP request type not yet implemented in local mode".to_string(),
                    )),
                }
            }
            DebugMode::Protocol(_config) => {
                // Forward to protocol debugging handler
                self.debug_protocol(request)
            }
        }
    }

    /// Get processor capabilities
    fn capabilities(&self) -> Vec<String> {
        self.capabilities()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_debug_bridge_local_mode() {
        let config = DebugConfig {
            mode: DebugMode::Local(LocalDebugConfig::default()),
            performance: PerformanceConfig::default(),
        };

        let bridge = DebugBridge::new(config).await.unwrap();

        // Test local debugging
        let session = bridge.debug_local("print('test')").await.unwrap();
        assert_eq!(session.state, DebugSessionState::Initialized);

        // Test capabilities
        let caps = bridge.capabilities();
        assert!(caps.contains(&"local_debugging".to_string()));
        assert!(caps.contains(&"breakpoints".to_string()));

        // Test performance monitoring
        let (init_time, _) = bridge.get_performance_stats().await;
        assert!(init_time.is_some());
        assert!(init_time.unwrap().as_millis() < 50); // Should be well under 10ms target
    }

    #[tokio::test]
    async fn test_debug_bridge_protocol_mode() {
        let config = DebugConfig {
            mode: DebugMode::Protocol(ProtocolDebugConfig {
                endpoint: "127.0.0.1:9555".to_string(),
                connection_timeout: 5000,
                request_timeout: 10000,
            }),
            performance: PerformanceConfig::default(),
        };

        let bridge = DebugBridge::new(config).await.unwrap();

        // Test protocol mode detection
        assert!(bridge.is_protocol_ready());

        // Test capabilities
        let caps = bridge.capabilities();
        assert!(caps.contains(&"protocol_debugging".to_string()));
        assert!(caps.contains(&"ldp_support".to_string()));
    }

    #[tokio::test]
    async fn test_message_processor_implementation() {
        let config = DebugConfig {
            mode: DebugMode::Local(LocalDebugConfig::default()),
            performance: PerformanceConfig::default(),
        };

        let bridge = DebugBridge::new(config).await.unwrap();

        // Test LDP initialize request
        let request = LDPRequest::InitializeRequest {
            client_id: "test".to_string(),
            client_name: "test".to_string(),
            adapter_id: "test".to_string(),
            locale: None,
            lines_start_at_1: true,
            columns_start_at_1: true,
            path_format: None,
            supports_variable_type: false,
            supports_variable_paging: false,
            supports_run_in_terminal_request: false,
            supports_memory_references: false,
            supports_progress_reporting: false,
            supports_invalidated_event: false,
        };

        let response = bridge.process_ldp(request).await.unwrap();
        assert!(matches!(response, LDPResponse::InitializeResponse { .. }));
    }

    #[tokio::test]
    async fn test_mode_transition() {
        let config = DebugConfig {
            mode: DebugMode::Local(LocalDebugConfig::default()),
            performance: PerformanceConfig::default(),
        };

        let mut bridge = DebugBridge::new(config).await.unwrap();

        // Start with local mode
        assert!(!bridge.is_protocol_ready());

        // Switch to protocol mode (Task 9.7 transition)
        let protocol_config = ProtocolDebugConfig {
            endpoint: "127.0.0.1:9555".to_string(),
            connection_timeout: 5000,
            request_timeout: 10000,
        };

        bridge
            .switch_to_protocol_mode(protocol_config)
            .await
            .unwrap();
        assert!(bridge.is_protocol_ready());
    }

    #[tokio::test]
    async fn test_enhanced_error_reporting() {
        let config = DebugConfig {
            mode: DebugMode::Local(LocalDebugConfig::default()),
            performance: PerformanceConfig::default(),
        };

        let bridge = DebugBridge::new(config).await.unwrap();

        // Test error classification
        let script_content = "local x = 1\nprint(y) -- undefined variable\nlocal z = 2";
        let enhanced_error = bridge.create_enhanced_error(
            "attempt to access undefined variable 'y'",
            script_content,
            Some(2), // Line 2
            Some(7), // Column 7
            Some("test.lua".to_string()),
        );

        // Verify error properties
        assert_eq!(enhanced_error.error_type, ErrorType::Reference);
        assert!(enhanced_error.location.is_some());
        assert!(enhanced_error.source_context.is_some());
        assert!(!enhanced_error.suggestions.is_empty());
        assert!(!enhanced_error.references.is_empty());

        // Test formatted output contains expected elements
        let formatted = bridge.format_enhanced_error(&enhanced_error);
        assert!(formatted.contains("Reference Error"));
        assert!(formatted.contains("print(y)")); // Error line should be shown
        assert!(formatted.contains("📍 Location")); // Location info
        assert!(formatted.contains("💡 Suggestions")); // Suggestions section
        assert!(formatted.contains("📚 References")); // References section

        // Test capabilities include enhanced error reporting
        let capabilities = bridge.capabilities();
        assert!(capabilities.contains(&"enhanced_error_reporting".to_string()));
        assert!(capabilities.contains(&"source_context".to_string()));
    }
}
