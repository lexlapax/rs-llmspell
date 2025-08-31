//! Shared execution context for both diagnostics and debugging
//!
//! This module provides a shared context that bridges diagnostics (logging/profiling)
//! and execution debugging (breakpoints/stepping), allowing them to enrich each other
//! with contextual information.

use crate::execution_bridge::StackFrame;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Source location in a script
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceLocation {
    /// Source file or script identifier
    pub source: String,
    /// Line number (1-based)
    pub line: u32,
    /// Column number (optional, 1-based)
    pub column: Option<u32>,
}

/// Shared execution context accessible by both systems
#[derive(Debug, Clone)]
pub struct SharedExecutionContext {
    /// Current execution stack
    pub stack: Vec<StackFrame>,
    /// Current execution location
    pub location: Option<SourceLocation>,
    /// Currently accessible variables
    pub variables: HashMap<String, serde_json::Value>,
    /// Recent diagnostic entries (for debugging context)
    pub recent_logs: Vec<DiagnosticEntry>,
    /// Performance metrics at current location
    pub performance_metrics: PerformanceMetrics,
    /// Correlation ID for async operation tracking
    pub correlation_id: Option<Uuid>,
}

/// Simplified diagnostic entry for execution context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticEntry {
    /// Log level
    pub level: String,
    /// Message
    pub message: String,
    /// Source location when logged
    pub location: Option<SourceLocation>,
    /// Timestamp
    pub timestamp: u64,
}

/// Performance metrics at current execution point
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    /// Time spent in current function (microseconds)
    pub function_time_us: u64,
    /// Memory allocated in current function (bytes)
    pub memory_allocated: usize,
    /// Number of times this line has been executed
    pub execution_count: u32,
}

/// Snapshot of context state for async boundary preservation
#[derive(Debug, Clone)]
pub struct ContextSnapshot {
    /// Stack snapshot
    pub stack: Vec<StackFrame>,
    /// Location snapshot
    pub location: Option<SourceLocation>,
    /// Variables snapshot
    pub variables: HashMap<String, serde_json::Value>,
    /// Correlation ID
    pub correlation_id: Option<Uuid>,
    /// Performance metrics snapshot
    pub performance_metrics: PerformanceMetrics,
}

impl SharedExecutionContext {
    /// Create a new shared execution context
    #[must_use]
    pub fn new() -> Self {
        Self {
            stack: Vec::new(),
            location: None,
            variables: HashMap::new(),
            recent_logs: Vec::new(),
            performance_metrics: PerformanceMetrics::default(),
            correlation_id: None,
        }
    }

    /// Create a context with async support
    #[must_use]
    pub fn with_async_support(mut self) -> Self {
        if self.correlation_id.is_none() {
            self.correlation_id = Some(Uuid::new_v4());
        }
        self
    }

    /// Preserve context state across async boundary
    #[must_use]
    pub fn preserve_across_async_boundary(&self) -> ContextSnapshot {
        ContextSnapshot {
            stack: self.stack.clone(),
            location: self.location.clone(),
            variables: self.variables.clone(),
            correlation_id: self.correlation_id,
            performance_metrics: self.performance_metrics.clone(),
        }
    }

    /// Restore context state from async boundary
    pub fn restore_from_async_boundary(&mut self, snapshot: ContextSnapshot) {
        self.stack = snapshot.stack;
        self.location = snapshot.location;
        self.variables = snapshot.variables;
        self.correlation_id = snapshot.correlation_id;
        self.performance_metrics = snapshot.performance_metrics;
    }

    /// Update the current location
    pub fn set_location(&mut self, location: SourceLocation) {
        self.location = Some(location);
    }

    /// Push a new stack frame
    pub fn push_frame(&mut self, frame: StackFrame) {
        self.stack.push(frame);
    }

    /// Pop the top stack frame
    pub fn pop_frame(&mut self) -> Option<StackFrame> {
        self.stack.pop()
    }

    /// Add a diagnostic entry
    pub fn add_diagnostic(&mut self, entry: DiagnosticEntry) {
        // Keep only the last 100 entries
        if self.recent_logs.len() >= 100 {
            self.recent_logs.remove(0);
        }
        self.recent_logs.push(entry);
    }

    /// Get diagnostics at current location
    #[must_use]
    pub fn get_diagnostics_at_location(&self) -> Vec<&DiagnosticEntry> {
        self.location.as_ref().map_or_else(Vec::new, |location| {
            self.recent_logs
                .iter()
                .filter(|entry| {
                    entry.location.as_ref().is_some_and(|loc| {
                        loc.source == location.source && loc.line == location.line
                    })
                })
                .collect()
        })
    }

    /// Update performance metrics
    pub const fn update_metrics(&mut self, time_us: u64, memory: usize) {
        self.performance_metrics.function_time_us += time_us;
        self.performance_metrics.memory_allocated += memory;
        self.performance_metrics.execution_count += 1;
    }

    /// Clear the context
    pub fn clear(&mut self) {
        self.stack.clear();
        self.location = None;
        self.variables.clear();
        self.recent_logs.clear();
        self.performance_metrics = PerformanceMetrics::default();
    }
}

impl Default for SharedExecutionContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Bridge trait for connecting diagnostics and execution debugging
pub trait ExecutionContextBridge: Send + Sync {
    /// Get the current shared context
    fn get_context(&self) -> SharedExecutionContext;

    /// Update the shared context
    fn update_context(&self, context: SharedExecutionContext);

    /// Enrich a diagnostic message with execution context
    fn enrich_diagnostic(&self, message: &str) -> String {
        let context = self.get_context();
        if let Some(location) = context.location {
            format!("{} [{}:{}]", message, location.source, location.line)
        } else {
            message.to_string()
        }
    }

    /// Get performance summary at current location
    fn get_performance_summary(&self) -> String {
        let context = self.get_context();
        let metrics = &context.performance_metrics;
        format!(
            "Function time: {}μs, Memory: {} bytes, Executions: {}",
            metrics.function_time_us, metrics.memory_allocated, metrics.execution_count
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execution_context() {
        let mut context = SharedExecutionContext::new();

        // Test location setting
        context.set_location(SourceLocation {
            source: "test.lua".to_string(),
            line: 10,
            column: Some(5),
        });
        assert!(context.location.is_some());

        // Test stack operations
        let frame = StackFrame {
            id: "frame_1".to_string(),
            name: "test_function".to_string(),
            source: "test.lua".to_string(),
            line: 10,
            column: None,
            locals: Vec::new(),
            is_user_code: true,
        };
        #[allow(clippy::redundant_clone)]
        context.push_frame(frame.clone());
        assert_eq!(context.stack.len(), 1);

        let popped = context.pop_frame();
        assert!(popped.is_some());
        assert_eq!(popped.unwrap().id, "frame_1");
    }

    #[test]
    fn test_diagnostic_filtering() {
        let mut context = SharedExecutionContext::new();

        context.set_location(SourceLocation {
            source: "test.lua".to_string(),
            line: 10,
            column: None,
        });

        // Add diagnostics at different locations
        context.add_diagnostic(DiagnosticEntry {
            level: "info".to_string(),
            message: "At line 10".to_string(),
            location: Some(SourceLocation {
                source: "test.lua".to_string(),
                line: 10,
                column: None,
            }),
            timestamp: 1000,
        });

        context.add_diagnostic(DiagnosticEntry {
            level: "warn".to_string(),
            message: "At line 20".to_string(),
            location: Some(SourceLocation {
                source: "test.lua".to_string(),
                line: 20,
                column: None,
            }),
            timestamp: 2000,
        });

        let at_location = context.get_diagnostics_at_location();
        assert_eq!(at_location.len(), 1);
        assert_eq!(at_location[0].message, "At line 10");
    }

    #[test]
    fn test_performance_metrics() {
        let mut context = SharedExecutionContext::new();

        context.update_metrics(100, 1024);
        context.update_metrics(50, 512);

        assert_eq!(context.performance_metrics.function_time_us, 150);
        assert_eq!(context.performance_metrics.memory_allocated, 1536);
        assert_eq!(context.performance_metrics.execution_count, 2);
    }
}
