//! Comprehensive Tracing Infrastructure
//!
//! This module provides structured tracing for the kernel, enabling detailed
//! visibility into execution flows, debugging operations, and performance monitoring.
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

/// Tracing instrumentation for kernel operations
#[derive(Clone)]
pub struct TracingInstrumentation {
    session_id: String,
    kernel_span: Span,
    execution_span: Arc<RwLock<Option<Span>>>,
    debug_span: Arc<RwLock<Option<Span>>>,
    application_span: Arc<RwLock<Option<Span>>>,
    metadata: Arc<RwLock<TracingMetadata>>,
}

/// Metadata for tracing context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TracingMetadata {
    /// Type of kernel (e.g., "integrated")
    pub kernel_type: String,
    /// When the session started
    pub start_time: DateTime<Utc>,
    /// Path to the script being executed
    pub script_path: Option<String>,
    /// Number of agents in the script
    pub agent_count: Option<usize>,
    /// Type of application being executed
    pub application_type: Option<String>,
    /// Complexity layer (1-6) of the application
    pub complexity_layer: Option<u8>,
    /// Expected runtime in seconds
    pub expected_runtime_seconds: Option<u64>,
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
    pub fn new_kernel_session(session_id: Option<String>) -> Self {
        let session_id = session_id.unwrap_or_else(|| Uuid::new_v4().to_string());

        let kernel_span = tracing::info_span!(
            "kernel_session",
            session_id = %session_id,
            kernel_type = "integrated",
            version = %crate::KERNEL_VERSION,
        );

        let metadata = TracingMetadata {
            kernel_type: "integrated".to_string(),
            start_time: Utc::now(),
            script_path: None,
            agent_count: None,
            application_type: None,
            complexity_layer: None,
            expected_runtime_seconds: None,
        };

        info!("Starting kernel session: {}", session_id);

        Self {
            session_id,
            kernel_span,
            execution_span: Arc::new(RwLock::new(None)),
            debug_span: Arc::new(RwLock::new(None)),
            application_span: Arc::new(RwLock::new(None)),
            metadata: Arc::new(RwLock::new(metadata)),
        }
    }

    /// Start execution tracing
    #[instrument(level = "debug", skip(self))]
    pub fn start_execution(&self, script_path: &str, agent_count: usize) {
        let execution_span = tracing::debug_span!(
            parent: &self.kernel_span,
            "script_execution",
            script = %script_path,
            agents = agent_count,
        );

        // Update metadata
        {
            let mut metadata = self.metadata.write();
            metadata.script_path = Some(script_path.to_string());
            metadata.agent_count = Some(agent_count);
        }

        info!(
            session_id = %self.session_id,
            "Starting execution: {} agents in {}",
            agent_count,
            script_path
        );

        *self.execution_span.write() = Some(execution_span);
    }

    /// Start debug operation tracing
    #[instrument(level = "trace", skip(self))]
    pub fn debug_operation(&self, operation: &str, line: u32) {
        let execution_span = self.execution_span.read();
        let parent_span = execution_span.as_ref().unwrap_or(&self.kernel_span);

        if self.debug_span.read().is_none() {
            let debug_span = tracing::trace_span!(
                parent: parent_span,
                "debug_session",
            );
            *self.debug_span.write() = Some(debug_span);
        }

        trace!(
            session_id = %self.session_id,
            "Debug operation: {} at line {}",
            operation,
            line
        );
    }

    /// Start application execution tracing
    #[instrument(level = "info", skip(self))]
    pub fn start_application(
        &self,
        app_type: &str,
        complexity_layer: u8,
        expected_agents: usize,
        expected_runtime: u64,
    ) {
        let app_span = tracing::info_span!(
            parent: &self.kernel_span,
            "application_execution",
            app_type = %app_type,
            expected_agents = expected_agents,
            expected_runtime = expected_runtime,
            complexity_layer = complexity_layer,
        );

        // Update metadata
        {
            let mut metadata = self.metadata.write();
            metadata.application_type = Some(app_type.to_string());
            metadata.complexity_layer = Some(complexity_layer);
            metadata.expected_runtime_seconds = Some(expected_runtime);
        }

        info!(
            session_id = %self.session_id,
            "Executing {} application (Layer {}) with {} agents, expected {}s",
            app_type,
            complexity_layer,
            expected_agents,
            expected_runtime
        );

        *self.application_span.write() = Some(app_span);
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

    /// Complete execution with results
    #[instrument(level = "info", skip(self))]
    pub fn complete_execution(&self, success: bool, runtime_ms: u64) {
        let status = if success { "success" } else { "failure" };

        info!(
            session_id = %self.session_id,
            status = status,
            runtime_ms = runtime_ms,
            "Execution completed"
        );

        // Check against expected runtime
        if let Some(expected) = self.metadata.read().expected_runtime_seconds {
            let actual_seconds = runtime_ms / 1000;
            if actual_seconds > expected * 2 {
                warn!(
                    session_id = %self.session_id,
                    "Application took {}% longer than expected",
                    (actual_seconds * 100) / expected
                );
            }
        }

        // Clear execution span
        *self.execution_span.write() = None;
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

/// Application detection for complexity-aware tracing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplicationDetection {
    /// Type of application detected
    pub app_type: String,
    /// Expected number of agents
    pub agent_count: usize,
    /// Estimated runtime in seconds
    pub estimated_seconds: u64,
    /// Complexity layer (1-6)
    pub complexity_layer: u8,
}

/// Detect application type from script path or content
pub fn detect_application_type(script_path: &str) -> ApplicationDetection {
    // Simple detection based on path patterns
    let app_type = if script_path.contains("file-organizer") {
        ("file-organizer", 3, 10, 1)
    } else if script_path.contains("research-collector") {
        ("research-collector", 2, 60, 1)
    } else if script_path.contains("content-creator") {
        ("content-creator", 4, 30, 2)
    } else if script_path.contains("personal-assistant") {
        ("personal-assistant", 5, 60, 3)
    } else if script_path.contains("communication-manager") {
        ("communication-manager", 5, 60, 3)
    } else if script_path.contains("code-review-assistant") {
        ("code-review-assistant", 7, 60, 3)
    } else if script_path.contains("process-orchestrator") {
        ("process-orchestrator", 8, 120, 4)
    } else if script_path.contains("knowledge-base") {
        ("knowledge-base", 6, 90, 4)
    } else if script_path.contains("webapp-creator") {
        ("webapp-creator", 21, 180, 5)
    } else {
        ("unknown", 1, 30, 1)
    };

    ApplicationDetection {
        app_type: app_type.0.to_string(),
        agent_count: app_type.1,
        estimated_seconds: app_type.2,
        complexity_layer: app_type.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_log::test;

    #[test]
    fn test_tracing_instrumentation_creation() {
        let tracing = TracingInstrumentation::new_kernel_session(None);
        assert!(!tracing.session_id().is_empty());

        let metadata = tracing.metadata();
        assert_eq!(metadata.kernel_type, "integrated");
        assert!(metadata.script_path.is_none());
    }

    #[test]
    fn test_tracing_execution() {
        let tracing = TracingInstrumentation::new_kernel_session(Some("test-session".to_string()));
        assert_eq!(tracing.session_id(), "test-session");

        tracing.start_execution("test_script.lua", 5);

        let metadata = tracing.metadata();
        assert_eq!(metadata.script_path, Some("test_script.lua".to_string()));
        assert_eq!(metadata.agent_count, Some(5));
    }

    #[test]
    fn test_tracing_application() {
        let tracing = TracingInstrumentation::new_kernel_session(None);

        tracing.start_application("content-creator", 2, 4, 30);

        let metadata = tracing.metadata();
        assert_eq!(metadata.application_type, Some("content-creator".to_string()));
        assert_eq!(metadata.complexity_layer, Some(2));
        assert_eq!(metadata.expected_runtime_seconds, Some(30));
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
    fn test_application_detection() {
        let detection = detect_application_type("examples/content-creator/main.lua");
        assert_eq!(detection.app_type, "content-creator");
        assert_eq!(detection.agent_count, 4);
        assert_eq!(detection.complexity_layer, 2);

        let detection = detect_application_type("examples/webapp-creator/main.lua");
        assert_eq!(detection.app_type, "webapp-creator");
        assert_eq!(detection.agent_count, 21);
        assert_eq!(detection.complexity_layer, 5);

        let detection = detect_application_type("random/script.lua");
        assert_eq!(detection.app_type, "unknown");
        assert_eq!(detection.complexity_layer, 1);
    }

    #[test]
    fn test_complete_execution() {
        let tracing = TracingInstrumentation::new_kernel_session(None);
        tracing.start_application("test-app", 1, 2, 10);

        // Complete successfully within expected time
        tracing.complete_execution(true, 8000);

        // Complete with longer runtime (should trigger warning in logs)
        tracing.complete_execution(true, 25000);
    }

    #[test]
    fn test_debug_operation() {
        let tracing = TracingInstrumentation::new_kernel_session(None);
        tracing.start_execution("debug_test.lua", 1);

        // Should not panic even without debug span
        tracing.debug_operation("breakpoint", 42);
        tracing.debug_operation("step", 43);
    }

    #[test]
    fn test_record_metric() {
        let tracing = TracingInstrumentation::new_kernel_session(None);

        tracing.record_metric("agent_creation_time", 45.5, "ms");
        tracing.record_metric("memory_usage", 1024.0, "MB");
    }
}