//! Debug entry structure for capturing debug messages
//!
//! Provides a structured format for debug messages with metadata
//! that can be serialized to various output formats.

use crate::debug::levels::DebugLevel;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt;

/// A single debug entry with all associated metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugEntry {
    /// Timestamp when the entry was created
    pub timestamp: DateTime<Utc>,
    /// Debug level of this entry
    pub level: DebugLevel,
    /// Module or component that generated this entry
    pub module: Option<String>,
    /// Main message content
    pub message: String,
    /// Optional structured metadata
    pub metadata: Option<Value>,
    /// Source file location (if available)
    pub source_location: Option<SourceLocation>,
    /// Thread ID that generated this entry
    pub thread_id: Option<String>,
    /// Correlation ID for tracing related entries
    pub correlation_id: Option<String>,
}

/// Source location information for debug entries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceLocation {
    /// Source file path
    pub file: String,
    /// Line number
    pub line: u32,
    /// Column number (if available)
    pub column: Option<u32>,
    /// Function name (if available)
    pub function: Option<String>,
}

impl DebugEntry {
    /// Create a new debug entry
    #[must_use]
    pub fn new(level: DebugLevel, message: impl Into<String>) -> Self {
        Self {
            timestamp: Utc::now(),
            level,
            module: None,
            message: message.into(),
            metadata: None,
            source_location: None,
            thread_id: std::thread::current()
                .name()
                .map(String::from)
                .or_else(|| Some(format!("{:?}", std::thread::current().id()))),
            correlation_id: None,
        }
    }

    /// Set the module for this entry
    #[must_use]
    pub fn with_module(mut self, module: impl Into<String>) -> Self {
        self.module = Some(module.into());
        self
    }

    /// Add metadata to this entry
    #[must_use]
    pub fn with_metadata(mut self, metadata: Value) -> Self {
        self.metadata = Some(metadata);
        self
    }

    /// Add source location to this entry
    #[must_use]
    pub fn with_source_location(mut self, location: SourceLocation) -> Self {
        self.source_location = Some(location);
        self
    }

    /// Add correlation ID for tracing
    #[must_use]
    pub fn with_correlation_id(mut self, id: impl Into<String>) -> Self {
        self.correlation_id = Some(id.into());
        self
    }

    /// Format as plain text
    #[must_use]
    pub fn format_text(&self, colored: bool) -> String {
        let level_str = if colored {
            self.level.colored()
        } else {
            &format!("{}", self.level)
        };

        let module_str = self
            .module
            .as_ref()
            .map(|m| format!("[{m}]"))
            .unwrap_or_default();

        let location_str = self
            .source_location
            .as_ref()
            .map(|loc| format!(" at {}:{}", loc.file, loc.line))
            .unwrap_or_default();

        format!(
            "{} {} {} {}{}",
            self.timestamp.format("%Y-%m-%d %H:%M:%S%.3f"),
            level_str,
            module_str,
            self.message,
            location_str
        )
    }

    /// Format as JSON
    #[must_use]
    pub fn format_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|_| self.format_text(false))
    }

    /// Format as pretty JSON
    #[must_use]
    pub fn format_json_pretty(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_else(|_| self.format_text(false))
    }
}

impl fmt::Display for DebugEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.format_text(false))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_debug_entry_creation() {
        let entry = DebugEntry::new(DebugLevel::Info, "Test message")
            .with_module("test_module")
            .with_metadata(json!({"key": "value"}));

        assert_eq!(entry.level, DebugLevel::Info);
        assert_eq!(entry.message, "Test message");
        assert_eq!(entry.module.as_deref(), Some("test_module"));
        assert!(entry.metadata.is_some());
    }

    #[test]
    fn test_format_text() {
        let entry = DebugEntry::new(DebugLevel::Debug, "Debug message").with_module("app");

        let text = entry.format_text(false);
        assert!(text.contains("DEBUG"));
        assert!(text.contains("[app]"));
        assert!(text.contains("Debug message"));
    }

    #[test]
    fn test_json_serialization() {
        let entry = DebugEntry::new(DebugLevel::Error, "Error occurred");
        let json = entry.format_json();
        assert!(json.contains("\"level\":\"Error\""));
        assert!(json.contains("\"message\":\"Error occurred\""));
    }
}
