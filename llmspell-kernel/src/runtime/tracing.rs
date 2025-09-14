//! Comprehensive Tracing Infrastructure
//!
//! This module provides structured tracing for the kernel, enabling detailed
//! visibility into kernel operations, sessions, and debugging.
//! Tracing is always present in the code but conditionally enabled via the
//! RUST_LOG environment variable.

use anyhow::Result;
use chrono::{DateTime, Utc};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
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
}

/// Tracing instrumentation for kernel operations
#[derive(Clone)]
pub struct TracingInstrumentation {
    session_id: String,
    kernel_span: Span,
    session_span: Arc<RwLock<Option<Span>>>,
    execution_span: Arc<RwLock<Option<Span>>>,
    debug_span: Arc<RwLock<Option<Span>>>,
    repl_span: Arc<RwLock<Option<Span>>>,
    metadata: Arc<RwLock<TracingMetadata>>,
}

/// Metadata for tracing context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TracingMetadata {
    /// Type of kernel (e.g., "integrated", "subprocess")
    pub kernel_type: String,
    /// When the session started
    pub start_time: DateTime<Utc>,
    /// Type of session being executed
    pub session_type: Option<SessionType>,
    /// Path to the script being executed (if applicable)
    pub script_path: Option<String>,
    /// Debug mode enabled
    pub debug_enabled: bool,
    /// State persistence enabled
    pub state_persistent: bool,
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
        };

        info!("Starting kernel session: {} (type: {})", session_id, kernel_type);

        Self {
            session_id,
            kernel_span,
            session_span: Arc::new(RwLock::new(None)),
            execution_span: Arc::new(RwLock::new(None)),
            debug_span: Arc::new(RwLock::new(None)),
            repl_span: Arc::new(RwLock::new(None)),
            metadata: Arc::new(RwLock::new(metadata)),
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
            script_path.map(|p| format!(" for {}", p)).unwrap_or_default()
        );

        *self.session_span.write() = Some(session_span);
    }

    /// Trace debug operations (breakpoints, stepping, etc.)
    #[instrument(level = "trace", skip(self))]
    pub fn trace_debug_operation(&self, operation: &str, details: Option<&str>) {
        let session_span = self.session_span.read();
        let parent_span = session_span.as_ref().unwrap_or(&self.kernel_span);

        if self.debug_span.read().is_none() {
            let debug_span = tracing::trace_span!(
                parent: parent_span,
                "debug_ops",
            );
            *self.debug_span.write() = Some(debug_span);
        }

        trace!(
            session_id = %self.session_id,
            operation = %operation,
            "Debug: {}{}",
            operation,
            details.map(|d| format!(": {}", d)).unwrap_or_default()
        );
    }

    /// Start REPL command execution
    #[instrument(level = "debug", skip(self))]
    pub fn trace_repl_command(&self, command: &str, line_number: usize) {
        if self.repl_span.read().is_none() {
            let session_span = self.session_span.read();
            let parent_span = session_span.as_ref().unwrap_or(&self.kernel_span);

            let repl_span = tracing::debug_span!(
                parent: parent_span,
                "repl_commands",
            );
            *self.repl_span.write() = Some(repl_span);
        }

        debug!(
            session_id = %self.session_id,
            line = line_number,
            "REPL[{}]: {}",
            line_number,
            command
        );
    }

    /// Trace state operations
    #[instrument(level = "debug", skip(self, value))]
    pub fn trace_state_operation(&self, operation: &str, key: &str, value: Option<&str>) {
        debug!(
            session_id = %self.session_id,
            operation = %operation,
            key = %key,
            has_value = value.is_some(),
            "State operation: {} on key '{}'",
            operation,
            key
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

        info!(
            session_id = %self.session_id,
            status = status,
            runtime_ms = runtime_ms,
            session_type = ?session_type,
            "Session completed"
        );

        // Clear session spans
        *self.session_span.write() = None;
        *self.execution_span.write() = None;
        *self.debug_span.write() = None;
        *self.repl_span.write() = None;
    }

    /// Get session ID
    pub fn session_id(&self) -> &str {
        &self.session_id
    }

    /// Get current metadata
    pub fn metadata(&self) -> TracingMetadata {
        self.metadata.read().clone()
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
/// It respects the RUST_LOG environment variable for filtering.
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

    #[test]
    fn test_repl_tracing() {
        let tracing = TracingInstrumentation::new_kernel_session(None, "integrated");
        tracing.start_session(SessionType::Repl, None);

        tracing.trace_repl_command("print('hello')", 1);
        tracing.trace_repl_command("local x = 42", 2);

        // Should not panic
    }

    #[test]
    fn test_state_tracing() {
        let tracing = TracingInstrumentation::new_kernel_session(None, "integrated");
        tracing.start_session(SessionType::State, None);

        tracing.trace_state_operation("set", "user.name", Some("test"));
        tracing.trace_state_operation("get", "user.name", None);
        tracing.trace_state_operation("delete", "user.name", None);

        // Should not panic
    }

    #[test]
    fn test_complete_session() {
        let tracing = TracingInstrumentation::new_kernel_session(None, "integrated");
        tracing.start_session(SessionType::Exec, None);

        // Complete successfully
        tracing.complete_session(true, 1500);

        // Start another session and fail it
        tracing.start_session(SessionType::Script, Some("test.lua"));
        tracing.complete_session(false, 500);
    }

    #[test]
    fn test_debug_operations() {
        let tracing = TracingInstrumentation::new_kernel_session(None, "integrated");
        tracing.start_session(SessionType::Debug, Some("debug_test.lua"));

        // Should not panic
        tracing.trace_debug_operation("breakpoint", Some("line 42"));
        tracing.trace_debug_operation("step_over", None);
        tracing.trace_debug_operation("continue", None);
    }

    #[test]
    fn test_record_metric() {
        let tracing = TracingInstrumentation::new_kernel_session(None, "integrated");

        tracing.record_metric("agent_creation_time", 45.5, "ms");
        tracing.record_metric("memory_usage", 1024.0, "MB");
    }
}