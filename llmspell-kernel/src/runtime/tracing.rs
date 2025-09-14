//! Comprehensive Tracing Infrastructure
//!
//! This module provides structured tracing for the entire LLMSpell system,
//! covering all phases of operation from script execution to infrastructure.
//!
//! ## Tracing Hierarchy
//!
//! 1. **Kernel Level** - Top-level kernel operations
//! 2. **Session Level** - Script, REPL, exec, debug sessions
//! 3. **Runtime Level** - Script engine operations (Lua, JS, Python)
//! 4. **Infrastructure Level** - Tools, agents, workflows, hooks, events, state
//! 5. **Security Level** - Sandboxing, permissions, authentication
//!
//! Tracing is always present in the code but conditionally enabled via the
//! `RUST_LOG` environment variable.

use anyhow::Result;
use chrono::{DateTime, Utc};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, info, instrument, trace, warn, Level, Span};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};
use uuid::Uuid;

/// Session type for tracing context
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SessionType {
    /// Script execution session (run command)
    Script,
    /// Interactive REPL session
    Repl,
    /// Single execution session (exec command)
    Exec,
    /// Debug session with breakpoints and stepping
    Debug,
    /// State management session
    State,
    /// Session management operations
    Session,
    /// Daemon/service mode session
    Daemon,
}

/// Operation category for structured tracing
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OperationCategory {
    /// Script runtime operations (execution, evaluation)
    ScriptRuntime,
    /// Tool invocations and results
    Tool,
    /// Agent operations (LLM calls, responses)
    Agent,
    /// Workflow orchestration
    Workflow,
    /// Hook execution (pre/post hooks, replay)
    Hook,
    /// Event emission and correlation
    Event,
    /// State operations (get/set/delete, persistence)
    State,
    /// Session management (creation, artifacts)
    Session,
    /// Security operations (sandboxing, permissions)
    Security,
    /// Vector operations (embeddings, RAG)
    Vector,
    /// Transport operations (Jupyter, LSP, DAP)
    Transport,
    /// Debug operations (breakpoints, stepping)
    Debug,
}

/// Tracing instrumentation for kernel operations
#[derive(Clone)]
pub struct TracingInstrumentation {
    session_id: String,
    kernel_span: Span,

    // Session-level spans
    session_span: Arc<RwLock<Option<Span>>>,

    // Runtime spans (script execution)
    runtime_span: Arc<RwLock<Option<Span>>>,

    // Infrastructure spans
    tool_span: Arc<RwLock<Option<Span>>>,
    agent_span: Arc<RwLock<Option<Span>>>,
    workflow_span: Arc<RwLock<Option<Span>>>,
    hook_span: Arc<RwLock<Option<Span>>>,
    event_span: Arc<RwLock<Option<Span>>>,
    state_span: Arc<RwLock<Option<Span>>>,

    // Specialized spans
    debug_span: Arc<RwLock<Option<Span>>>,
    security_span: Arc<RwLock<Option<Span>>>,
    vector_span: Arc<RwLock<Option<Span>>>,
    transport_span: Arc<RwLock<Option<Span>>>,

    metadata: Arc<RwLock<TracingMetadata>>,
    operation_counts: Arc<RwLock<HashMap<OperationCategory, u64>>>,
}

/// Metadata for tracing context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TracingMetadata {
    /// Type of kernel (e.g., "integrated", "subprocess", "daemon")
    pub kernel_type: String,
    /// When the session started
    pub start_time: DateTime<Utc>,
    /// Type of session being executed
    pub session_type: Option<SessionType>,
    /// Path to the script being executed (if applicable)
    pub script_path: Option<String>,

    // Feature flags
    /// Debug mode enabled
    pub debug_enabled: bool,
    /// State persistence enabled
    pub state_persistent: bool,
    /// Session persistence enabled
    pub session_persistent: bool,
    /// Hooks enabled
    pub hooks_enabled: bool,
    /// Events enabled
    pub events_enabled: bool,
    /// Security sandboxing enabled
    pub security_enabled: bool,
    /// Vector storage enabled
    pub vector_enabled: bool,

    // Runtime information
    /// Script engine type (lua, javascript, python)
    pub script_engine: Option<String>,
    /// Active providers
    pub active_providers: Vec<String>,
    /// Connected transports
    pub connected_transports: Vec<String>,
}

/// Tracing level configuration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TracingLevel {
    /// Only errors
    Error,
    /// Warnings and errors
    Warn,
    /// Info, warnings, and errors
    Info,
    /// Debug information and above
    Debug,
    /// Detailed trace information
    Trace,
}

impl TracingInstrumentation {
    /// Create a new kernel session with tracing
    #[instrument(level = "info", skip_all)]
    pub fn new_kernel_session(session_id: Option<String>, kernel_type: &str) -> Self {
        let session_id = session_id.unwrap_or_else(|| Uuid::new_v4().to_string());

        let kernel_span = tracing::info_span!(
            "kernel",
            session_id = %session_id,
            kernel_type = %kernel_type,
            version = %crate::KERNEL_VERSION,
        );

        let metadata = TracingMetadata {
            kernel_type: kernel_type.to_string(),
            start_time: Utc::now(),
            session_type: None,
            script_path: None,
            debug_enabled: false,
            state_persistent: false,
            session_persistent: false,
            hooks_enabled: false,
            events_enabled: false,
            security_enabled: false,
            vector_enabled: false,
            script_engine: None,
            active_providers: Vec::new(),
            connected_transports: Vec::new(),
        };

        info!("Starting kernel session: {} (type: {})", session_id, kernel_type);

        Self {
            session_id,
            kernel_span,
            session_span: Arc::new(RwLock::new(None)),
            runtime_span: Arc::new(RwLock::new(None)),
            tool_span: Arc::new(RwLock::new(None)),
            agent_span: Arc::new(RwLock::new(None)),
            workflow_span: Arc::new(RwLock::new(None)),
            hook_span: Arc::new(RwLock::new(None)),
            event_span: Arc::new(RwLock::new(None)),
            state_span: Arc::new(RwLock::new(None)),
            debug_span: Arc::new(RwLock::new(None)),
            security_span: Arc::new(RwLock::new(None)),
            vector_span: Arc::new(RwLock::new(None)),
            transport_span: Arc::new(RwLock::new(None)),
            metadata: Arc::new(RwLock::new(metadata)),
            operation_counts: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Start a session of specific type
    #[instrument(level = "debug", skip(self))]
    pub fn start_session(&self, session_type: SessionType, script_path: Option<&str>) {
        let session_span = match session_type {
            SessionType::Script => tracing::debug_span!(
                parent: &self.kernel_span,
                "script_session",
                script = script_path.unwrap_or("<none>"),
            ),
            SessionType::Repl => tracing::debug_span!(
                parent: &self.kernel_span,
                "repl_session",
                interactive = true,
            ),
            SessionType::Exec => tracing::debug_span!(
                parent: &self.kernel_span,
                "exec_session",
                single_shot = true,
            ),
            SessionType::Debug => tracing::debug_span!(
                parent: &self.kernel_span,
                "debug_session",
                breakpoints_enabled = true,
            ),
            SessionType::State => tracing::debug_span!(
                parent: &self.kernel_span,
                "state_session",
                persistent = true,
            ),
            SessionType::Session => tracing::debug_span!(
                parent: &self.kernel_span,
                "session_mgmt",
            ),
            SessionType::Daemon => tracing::debug_span!(
                parent: &self.kernel_span,
                "daemon_session",
                service_mode = true,
            ),
        };

        // Update metadata
        {
            let mut metadata = self.metadata.write();
            metadata.session_type = Some(session_type);
            if let Some(path) = script_path {
                metadata.script_path = Some(path.to_string());
            }
        }

        info!(
            session_id = %self.session_id,
            "Starting {:?} session{}",
            session_type,
            script_path.map(|p| format!(" for {p}")).unwrap_or_default()
        );

        *self.session_span.write() = Some(session_span);
    }

    /// Trace a script runtime operation
    #[instrument(level = "trace", skip(self))]
    pub fn trace_runtime_operation(&self, engine: &str, operation: &str, details: Option<&str>) {
        self.trace_operation(OperationCategory::ScriptRuntime, operation, details);

        // Update engine type if not set
        let mut metadata = self.metadata.write();
        if metadata.script_engine.is_none() {
            metadata.script_engine = Some(engine.to_string());
        }
    }

    /// Trace a tool invocation
    #[instrument(level = "debug", skip(self))]
    pub fn trace_tool_invocation(&self, tool_name: &str, params: Option<&str>) {
        self.trace_operation(OperationCategory::Tool, tool_name, params);
    }

    /// Trace an agent operation
    #[instrument(level = "debug", skip(self))]
    pub fn trace_agent_operation(&self, agent_name: &str, provider: &str, operation: &str) {
        self.trace_operation(OperationCategory::Agent, &format!("{agent_name}.{operation}"), Some(provider));

        // Track active providers
        let mut metadata = self.metadata.write();
        if !metadata.active_providers.contains(&provider.to_string()) {
            metadata.active_providers.push(provider.to_string());
        }
    }

    /// Trace a workflow step
    #[instrument(level = "debug", skip(self))]
    pub fn trace_workflow_step(&self, workflow_name: &str, step: &str) {
        self.trace_operation(OperationCategory::Workflow, &format!("{workflow_name}.{step}"), None);
    }

    /// Trace a hook execution
    #[instrument(level = "trace", skip(self))]
    pub fn trace_hook_execution(&self, hook_type: &str, hook_name: &str, phase: &str) {
        self.trace_operation(OperationCategory::Hook, &format!("{hook_type}.{hook_name}"), Some(phase));

        // Enable hooks flag
        self.metadata.write().hooks_enabled = true;
    }

    /// Trace an event emission
    #[instrument(level = "trace", skip(self))]
    pub fn trace_event_emission(&self, event_type: &str, correlation_id: Option<&str>) {
        self.trace_operation(OperationCategory::Event, event_type, correlation_id);

        // Enable events flag
        self.metadata.write().events_enabled = true;
    }

    /// Trace state operations
    #[instrument(level = "debug", skip(self, value))]
    pub fn trace_state_operation(&self, operation: &str, key: &str, value: Option<&str>) {
        let details = format!("key={key}, has_value={}", value.is_some());
        self.trace_operation(OperationCategory::State, operation, Some(&details));

        // Enable state flag if it's a persistence operation
        if operation == "persist" || operation == "restore" {
            self.metadata.write().state_persistent = true;
        }
    }

    /// Trace session management operations
    #[instrument(level = "debug", skip(self))]
    pub fn trace_session_operation(&self, operation: &str, session_id: &str) {
        self.trace_operation(OperationCategory::Session, operation, Some(session_id));

        // Enable session persistence if relevant
        if operation == "persist" || operation == "restore" {
            self.metadata.write().session_persistent = true;
        }
    }

    /// Trace security operations
    #[instrument(level = "info", skip(self))]
    pub fn trace_security_operation(&self, operation: &str, details: &str) {
        self.trace_operation(OperationCategory::Security, operation, Some(details));

        // Enable security flag
        self.metadata.write().security_enabled = true;
    }

    /// Trace vector/RAG operations
    #[instrument(level = "debug", skip(self))]
    pub fn trace_vector_operation(&self, operation: &str, collection: &str, count: usize) {
        let details = format!("collection={collection}, count={count}");
        self.trace_operation(OperationCategory::Vector, operation, Some(&details));

        // Enable vector flag
        self.metadata.write().vector_enabled = true;
    }

    /// Trace transport operations
    #[instrument(level = "trace", skip(self))]
    pub fn trace_transport_operation(&self, transport_type: &str, channel: &str, operation: &str) {
        let details = format!("{transport_type}.{channel}");
        self.trace_operation(OperationCategory::Transport, operation, Some(&details));

        // Track connected transports
        let mut metadata = self.metadata.write();
        if !metadata.connected_transports.contains(&transport_type.to_string()) {
            metadata.connected_transports.push(transport_type.to_string());
        }
    }

    /// Trace debug operations (breakpoints, stepping, etc.)
    #[instrument(level = "trace", skip(self))]
    pub fn trace_debug_operation(&self, operation: &str, details: Option<&str>) {
        self.trace_operation(OperationCategory::Debug, operation, details);

        // Enable debug flag
        self.metadata.write().debug_enabled = true;
    }

    /// Trace REPL command execution
    #[instrument(level = "debug", skip(self))]
    pub fn trace_repl_command(&self, command: &str, line_number: usize) {
        debug!(
            session_id = %self.session_id,
            line = line_number,
            "REPL[{}]: {}",
            line_number,
            command
        );
    }

    /// Core operation tracing
    pub fn trace_operation(&self, category: OperationCategory, operation: &str, details: Option<&str>) {
        // Get or create the appropriate span
        let span = match category {
            OperationCategory::ScriptRuntime => &self.runtime_span,
            OperationCategory::Tool => &self.tool_span,
            OperationCategory::Agent => &self.agent_span,
            OperationCategory::Workflow => &self.workflow_span,
            OperationCategory::Hook => &self.hook_span,
            OperationCategory::Event => &self.event_span,
            OperationCategory::State => &self.state_span,
            OperationCategory::Session => &self.session_span,
            OperationCategory::Security => &self.security_span,
            OperationCategory::Vector => &self.vector_span,
            OperationCategory::Transport => &self.transport_span,
            OperationCategory::Debug => &self.debug_span,
        };

        // Create span if it doesn't exist
        if span.read().is_none() {
            let parent_span = self.session_span.read();
            let parent = parent_span.as_ref().unwrap_or(&self.kernel_span);

            let new_span = tracing::trace_span!(
                parent: parent,
                "operation",
                category = ?category,
            );
            *span.write() = Some(new_span);
        }

        // Increment operation count
        {
            let mut counts = self.operation_counts.write();
            *counts.entry(category).or_insert(0) += 1;
        }

        // Log the operation
        trace!(
            session_id = %self.session_id,
            category = ?category,
            operation = %operation,
            details = details,
            "{:?}: {}{}",
            category,
            operation,
            details.map(|d| format!(": {d}")).unwrap_or_default()
        );
    }

    /// Record a performance metric
    #[instrument(level = "debug", skip(self, value))]
    pub fn record_metric(&self, metric_name: &str, value: f64, unit: &str) {
        debug!(
            session_id = %self.session_id,
            metric = metric_name,
            value = value,
            unit = unit,
            "Performance metric recorded"
        );
    }

    /// Log a warning with session context
    pub fn warn_with_context(&self, message: &str) {
        warn!(
            session_id = %self.session_id,
            "{}",
            message
        );
    }

    /// Complete session with results
    #[instrument(level = "info", skip(self))]
    pub fn complete_session(&self, success: bool, runtime_ms: u64) {
        let status = if success { "success" } else { "failure" };
        let session_type = self.metadata.read().session_type;

        // Get operation counts
        let counts = self.operation_counts.read();

        info!(
            session_id = %self.session_id,
            status = status,
            runtime_ms = runtime_ms,
            session_type = ?session_type,
            tool_ops = counts.get(&OperationCategory::Tool).unwrap_or(&0),
            agent_ops = counts.get(&OperationCategory::Agent).unwrap_or(&0),
            state_ops = counts.get(&OperationCategory::State).unwrap_or(&0),
            "Session completed"
        );

        // Clear all spans
        *self.session_span.write() = None;
        *self.runtime_span.write() = None;
        *self.tool_span.write() = None;
        *self.agent_span.write() = None;
        *self.workflow_span.write() = None;
        *self.hook_span.write() = None;
        *self.event_span.write() = None;
        *self.state_span.write() = None;
        *self.debug_span.write() = None;
        *self.security_span.write() = None;
        *self.vector_span.write() = None;
        *self.transport_span.write() = None;
    }

    /// Get session ID
    pub fn session_id(&self) -> &str {
        &self.session_id
    }

    /// Get current metadata
    pub fn metadata(&self) -> TracingMetadata {
        self.metadata.read().clone()
    }

    /// Get operation statistics
    pub fn operation_stats(&self) -> HashMap<OperationCategory, u64> {
        self.operation_counts.read().clone()
    }

    /// Enter kernel span context
    pub fn enter(&self) -> tracing::span::Entered<'_> {
        self.kernel_span.enter()
    }
}

impl TracingLevel {
    /// Convert to tracing Level
    pub fn to_level(self) -> Level {
        match self {
            Self::Error => Level::ERROR,
            Self::Warn => Level::WARN,
            Self::Info => Level::INFO,
            Self::Debug => Level::DEBUG,
            Self::Trace => Level::TRACE,
        }
    }

    /// Parse from string
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "error" => Some(Self::Error),
            "warn" => Some(Self::Warn),
            "info" => Some(Self::Info),
            "debug" => Some(Self::Debug),
            "trace" => Some(Self::Trace),
            _ => None,
        }
    }
}

/// Initialize tracing subscriber
///
/// This should be called once at application startup to configure tracing.
/// It respects the `RUST_LOG` environment variable for filtering.
///
/// # Errors
///
/// Returns an error if the subscriber cannot be initialized.
pub fn init_tracing() -> Result<()> {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_target(true)
        .with_thread_ids(true)
        .with_thread_names(true)
        .with_file(true)
        .with_line_number(true);

    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt_layer)
        .try_init()
        .map_err(|e| anyhow::anyhow!("Failed to initialize tracing: {}", e))
}

/// Initialize tracing with custom filter
///
/// # Errors
///
/// Returns an error if the filter is invalid or subscriber cannot be initialized.
pub fn init_tracing_with_filter(filter: &str) -> Result<()> {
    let env_filter = EnvFilter::try_new(filter)
        .map_err(|e| anyhow::anyhow!("Invalid filter: {}", e))?;

    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_target(true)
        .with_thread_ids(true)
        .with_thread_names(true)
        .with_file(true)
        .with_line_number(true);

    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt_layer)
        .try_init()
        .map_err(|e| anyhow::anyhow!("Failed to initialize tracing: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_log::test;

    #[test]
    fn test_tracing_instrumentation_creation() {
        let tracing = TracingInstrumentation::new_kernel_session(None, "integrated");
        assert!(!tracing.session_id().is_empty());

        let metadata = tracing.metadata();
        assert_eq!(metadata.kernel_type, "integrated");
        assert!(metadata.script_path.is_none());
    }

    #[test]
    fn test_tracing_sessions() {
        let tracing = TracingInstrumentation::new_kernel_session(Some("test-session".to_string()), "integrated");
        assert_eq!(tracing.session_id(), "test-session");

        tracing.start_session(SessionType::Script, Some("test_script.lua"));

        let metadata = tracing.metadata();
        assert_eq!(metadata.script_path, Some("test_script.lua".to_string()));
        assert_eq!(metadata.session_type, Some(SessionType::Script));
    }

    #[test]
    fn test_session_types() {
        let tracing = TracingInstrumentation::new_kernel_session(None, "integrated");

        // Test different session types
        tracing.start_session(SessionType::Repl, None);
        assert_eq!(tracing.metadata().session_type, Some(SessionType::Repl));

        tracing.start_session(SessionType::Debug, Some("debug.lua"));
        assert_eq!(tracing.metadata().session_type, Some(SessionType::Debug));

        tracing.start_session(SessionType::Daemon, None);
        assert_eq!(tracing.metadata().session_type, Some(SessionType::Daemon));
    }

    #[test]
    fn test_runtime_tracing() {
        let tracing = TracingInstrumentation::new_kernel_session(None, "integrated");

        tracing.trace_runtime_operation("lua", "eval", Some("print('hello')"));
        assert_eq!(tracing.metadata().script_engine, Some("lua".to_string()));

        tracing.trace_runtime_operation("javascript", "compile", None);
        // Should not override
        assert_eq!(tracing.metadata().script_engine, Some("lua".to_string()));
    }

    #[test]
    fn test_tool_tracing() {
        let tracing = TracingInstrumentation::new_kernel_session(None, "integrated");

        tracing.trace_tool_invocation("file_system", Some("read /tmp/test.txt"));
        tracing.trace_tool_invocation("http_request", Some("GET https://api.example.com"));

        let stats = tracing.operation_stats();
        assert_eq!(stats.get(&OperationCategory::Tool), Some(&2));
    }

    #[test]
    fn test_agent_tracing() {
        let tracing = TracingInstrumentation::new_kernel_session(None, "integrated");

        tracing.trace_agent_operation("research_agent", "openai", "chat_completion");
        tracing.trace_agent_operation("code_agent", "anthropic", "generate");

        let metadata = tracing.metadata();
        assert!(metadata.active_providers.contains(&"openai".to_string()));
        assert!(metadata.active_providers.contains(&"anthropic".to_string()));
    }

    #[test]
    fn test_hook_event_tracing() {
        let tracing = TracingInstrumentation::new_kernel_session(None, "integrated");

        tracing.trace_hook_execution("pre", "validation_hook", "before");
        tracing.trace_event_emission("task_completed", Some("corr-123"));

        let metadata = tracing.metadata();
        assert!(metadata.hooks_enabled);
        assert!(metadata.events_enabled);
    }

    #[test]
    fn test_state_tracing() {
        let tracing = TracingInstrumentation::new_kernel_session(None, "integrated");
        tracing.start_session(SessionType::State, None);

        tracing.trace_state_operation("set", "user.name", Some("test"));
        tracing.trace_state_operation("get", "user.name", None);
        tracing.trace_state_operation("persist", "state", None);

        let metadata = tracing.metadata();
        assert!(metadata.state_persistent);
    }

    #[test]
    fn test_security_tracing() {
        let tracing = TracingInstrumentation::new_kernel_session(None, "integrated");

        tracing.trace_security_operation("sandbox_init", "lua sandbox created");
        tracing.trace_security_operation("permission_check", "file_system.write denied");

        let metadata = tracing.metadata();
        assert!(metadata.security_enabled);
    }

    #[test]
    fn test_vector_tracing() {
        let tracing = TracingInstrumentation::new_kernel_session(None, "integrated");

        tracing.trace_vector_operation("embed", "documents", 100);
        tracing.trace_vector_operation("search", "documents", 10);

        let metadata = tracing.metadata();
        assert!(metadata.vector_enabled);
    }

    #[test]
    fn test_transport_tracing() {
        let tracing = TracingInstrumentation::new_kernel_session(None, "integrated");

        tracing.trace_transport_operation("jupyter", "shell", "execute_request");
        tracing.trace_transport_operation("lsp", "stdio", "initialize");

        let metadata = tracing.metadata();
        assert!(metadata.connected_transports.contains(&"jupyter".to_string()));
        assert!(metadata.connected_transports.contains(&"lsp".to_string()));
    }

    #[test]
    fn test_repl_tracing() {
        let tracing = TracingInstrumentation::new_kernel_session(None, "integrated");
        tracing.start_session(SessionType::Repl, None);

        tracing.trace_repl_command("print('hello')", 1);
        tracing.trace_repl_command("local x = 42", 2);

        // Should not panic
    }

    #[test]
    fn test_debug_operations() {
        let tracing = TracingInstrumentation::new_kernel_session(None, "integrated");
        tracing.start_session(SessionType::Debug, Some("debug_test.lua"));

        tracing.trace_debug_operation("breakpoint", Some("line 42"));
        tracing.trace_debug_operation("step_over", None);
        tracing.trace_debug_operation("continue", None);

        let metadata = tracing.metadata();
        assert!(metadata.debug_enabled);
    }

    #[test]
    fn test_complete_session() {
        let tracing = TracingInstrumentation::new_kernel_session(None, "integrated");
        tracing.start_session(SessionType::Exec, None);

        // Perform various operations
        tracing.trace_tool_invocation("test_tool", None);
        tracing.trace_agent_operation("test_agent", "openai", "chat");
        tracing.trace_state_operation("set", "key", Some("value"));

        // Complete successfully
        tracing.complete_session(true, 1500);

        // Start another session and fail it
        tracing.start_session(SessionType::Script, Some("test.lua"));
        tracing.complete_session(false, 500);
    }

    #[test]
    fn test_record_metric() {
        let tracing = TracingInstrumentation::new_kernel_session(None, "integrated");

        tracing.record_metric("agent_creation_time", 45.5, "ms");
        tracing.record_metric("memory_usage", 1024.0, "MB");
    }

    #[test]
    fn test_operation_statistics() {
        let tracing = TracingInstrumentation::new_kernel_session(None, "integrated");

        // Perform various operations
        tracing.trace_tool_invocation("tool1", None);
        tracing.trace_tool_invocation("tool2", None);
        tracing.trace_agent_operation("agent1", "openai", "chat");
        tracing.trace_state_operation("set", "key", None);
        tracing.trace_state_operation("get", "key", None);
        tracing.trace_state_operation("delete", "key", None);

        let stats = tracing.operation_stats();
        assert_eq!(stats.get(&OperationCategory::Tool), Some(&2));
        assert_eq!(stats.get(&OperationCategory::Agent), Some(&1));
        assert_eq!(stats.get(&OperationCategory::State), Some(&3));
    }

    #[test]
    fn test_tracing_level_conversion() {
        assert_eq!(TracingLevel::Error.to_level(), Level::ERROR);
        assert_eq!(TracingLevel::Debug.to_level(), Level::DEBUG);
        assert_eq!(TracingLevel::Trace.to_level(), Level::TRACE);
    }

    #[test]
    fn test_tracing_level_parsing() {
        assert_eq!(TracingLevel::from_str("error"), Some(TracingLevel::Error));
        assert_eq!(TracingLevel::from_str("DEBUG"), Some(TracingLevel::Debug));
        assert_eq!(TracingLevel::from_str("invalid"), None);
    }
}