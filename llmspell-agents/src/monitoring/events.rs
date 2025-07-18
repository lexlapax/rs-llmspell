//! ABOUTME: Event logging system for agent activities
//! ABOUTME: Provides structured logging, event correlation, and log aggregation

use chrono::{DateTime, Utc};
use llmspell_core::Result;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, RwLock};

/// Log levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum LogLevel {
    /// Trace level - most verbose
    Trace,
    /// Debug level
    Debug,
    /// Informational
    Info,
    /// Warning
    Warn,
    /// Error
    Error,
    /// Fatal/Critical
    Fatal,
}

impl LogLevel {
    /// Check if this level should be logged given a minimum level
    pub fn should_log(&self, min_level: LogLevel) -> bool {
        *self >= min_level
    }

    /// Convert to string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            LogLevel::Trace => "TRACE",
            LogLevel::Debug => "DEBUG",
            LogLevel::Info => "INFO",
            LogLevel::Warn => "WARN",
            LogLevel::Error => "ERROR",
            LogLevel::Fatal => "FATAL",
        }
    }
}

/// Log event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEvent {
    /// Event ID
    pub id: String,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Log level
    pub level: LogLevel,
    /// Agent ID
    pub agent_id: String,
    /// Component/module name
    pub component: String,
    /// Log message
    pub message: String,
    /// Structured fields
    pub fields: HashMap<String, serde_json::Value>,
    /// Trace ID (for correlation)
    pub trace_id: Option<String>,
    /// Span ID (for correlation)
    pub span_id: Option<String>,
    /// Error details
    pub error: Option<ErrorDetails>,
}

impl LogEvent {
    /// Create a new log event
    pub fn new(level: LogLevel, agent_id: String, component: String, message: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            level,
            agent_id,
            component,
            message,
            fields: HashMap::new(),
            trace_id: None,
            span_id: None,
            error: None,
        }
    }

    /// Add a field
    pub fn with_field(mut self, key: String, value: serde_json::Value) -> Self {
        self.fields.insert(key, value);
        self
    }

    /// Add trace context
    pub fn with_trace(mut self, trace_id: String, span_id: String) -> Self {
        self.trace_id = Some(trace_id);
        self.span_id = Some(span_id);
        self
    }

    /// Add error details
    pub fn with_error(mut self, error: ErrorDetails) -> Self {
        self.error = Some(error);
        self
    }

    /// Format as a log line
    pub fn format(&self) -> String {
        let mut parts = vec![
            format!("[{}]", self.level.as_str()),
            format!("[{}]", self.agent_id),
            format!("[{}]", self.component),
            self.message.clone(),
        ];

        if !self.fields.is_empty() {
            let fields: Vec<String> = self
                .fields
                .iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect();
            parts.push(format!("fields={{{}}}", fields.join(", ")));
        }

        if let Some(trace_id) = &self.trace_id {
            parts.push(format!("trace_id={}", trace_id));
        }

        if let Some(error) = &self.error {
            parts.push(format!("error={}", error.message));
        }

        parts.join(" ")
    }
}

/// Error details for log events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorDetails {
    /// Error type/category
    pub error_type: String,
    /// Error message
    pub message: String,
    /// Stack trace (if available)
    pub stack_trace: Option<String>,
    /// Additional context
    pub context: HashMap<String, String>,
}

/// Event logger for managing log events
pub struct EventLogger {
    /// Agent ID
    agent_id: String,
    /// Minimum log level
    min_level: RwLock<LogLevel>,
    /// Log buffer (ring buffer)
    buffer: Arc<RwLock<VecDeque<LogEvent>>>,
    /// Maximum buffer size
    max_buffer_size: usize,
    /// Log exporters
    exporters: Vec<Box<dyn LogExporter>>,
    /// Event filters
    filters: Vec<Box<dyn EventFilter>>,
}

impl std::fmt::Debug for EventLogger {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EventLogger")
            .field("agent_id", &self.agent_id)
            .field("min_level", &*self.min_level.read().unwrap())
            .field("max_buffer_size", &self.max_buffer_size)
            .field("exporters_count", &self.exporters.len())
            .field("filters_count", &self.filters.len())
            .finish()
    }
}

impl EventLogger {
    /// Create a new event logger
    pub fn new(agent_id: String, max_buffer_size: usize) -> Self {
        Self {
            agent_id,
            min_level: RwLock::new(LogLevel::Info),
            buffer: Arc::new(RwLock::new(VecDeque::with_capacity(max_buffer_size))),
            max_buffer_size,
            exporters: Vec::new(),
            filters: Vec::new(),
        }
    }

    /// Set minimum log level
    pub fn set_level(&self, level: LogLevel) {
        *self.min_level.write().unwrap() = level;
    }

    /// Get current log level
    pub fn get_level(&self) -> LogLevel {
        *self.min_level.read().unwrap()
    }

    /// Add a log exporter
    pub fn add_exporter(&mut self, exporter: Box<dyn LogExporter>) {
        self.exporters.push(exporter);
    }

    /// Add an event filter
    pub fn add_filter(&mut self, filter: Box<dyn EventFilter>) {
        self.filters.push(filter);
    }

    /// Log an event
    pub fn log(&self, event: LogEvent) -> Result<()> {
        // Check minimum level
        if !event.level.should_log(self.get_level()) {
            return Ok(());
        }

        // Apply filters
        for filter in &self.filters {
            if !filter.should_log(&event) {
                return Ok(());
            }
        }

        // Export to all exporters
        for exporter in &self.exporters {
            exporter.export(&event)?;
        }

        // Store in buffer
        let mut buffer = self.buffer.write().unwrap();
        if buffer.len() >= self.max_buffer_size {
            buffer.pop_front();
        }
        buffer.push_back(event);

        Ok(())
    }

    /// Log a trace event
    pub fn trace(&self, component: &str, message: &str) -> Result<()> {
        let event = LogEvent::new(
            LogLevel::Trace,
            self.agent_id.clone(),
            component.to_string(),
            message.to_string(),
        );
        self.log(event)
    }

    /// Log a debug event
    pub fn debug(&self, component: &str, message: &str) -> Result<()> {
        let event = LogEvent::new(
            LogLevel::Debug,
            self.agent_id.clone(),
            component.to_string(),
            message.to_string(),
        );
        self.log(event)
    }

    /// Log an info event
    pub fn info(&self, component: &str, message: &str) -> Result<()> {
        let event = LogEvent::new(
            LogLevel::Info,
            self.agent_id.clone(),
            component.to_string(),
            message.to_string(),
        );
        self.log(event)
    }

    /// Log a warning event
    pub fn warn(&self, component: &str, message: &str) -> Result<()> {
        let event = LogEvent::new(
            LogLevel::Warn,
            self.agent_id.clone(),
            component.to_string(),
            message.to_string(),
        );
        self.log(event)
    }

    /// Log an error event
    pub fn error(&self, component: &str, message: &str, error: Option<ErrorDetails>) -> Result<()> {
        let mut event = LogEvent::new(
            LogLevel::Error,
            self.agent_id.clone(),
            component.to_string(),
            message.to_string(),
        );

        if let Some(error) = error {
            event = event.with_error(error);
        }

        self.log(event)
    }

    /// Log a fatal event
    pub fn fatal(&self, component: &str, message: &str, error: Option<ErrorDetails>) -> Result<()> {
        let mut event = LogEvent::new(
            LogLevel::Fatal,
            self.agent_id.clone(),
            component.to_string(),
            message.to_string(),
        );

        if let Some(error) = error {
            event = event.with_error(error);
        }

        self.log(event)
    }

    /// Get recent events
    pub fn get_recent_events(&self, count: usize) -> Vec<LogEvent> {
        let buffer = self.buffer.read().unwrap();
        buffer.iter().rev().take(count).cloned().collect()
    }

    /// Get events by level
    pub fn get_events_by_level(&self, level: LogLevel) -> Vec<LogEvent> {
        let buffer = self.buffer.read().unwrap();
        buffer
            .iter()
            .filter(|e| e.level == level)
            .cloned()
            .collect()
    }

    /// Get events by trace ID
    pub fn get_events_by_trace(&self, trace_id: &str) -> Vec<LogEvent> {
        let buffer = self.buffer.read().unwrap();
        buffer
            .iter()
            .filter(|e| e.trace_id.as_ref() == Some(&trace_id.to_string()))
            .cloned()
            .collect()
    }

    /// Clear the event buffer
    pub fn clear_buffer(&self) {
        self.buffer.write().unwrap().clear();
    }

    /// Get event statistics
    pub fn get_statistics(&self) -> EventStatistics {
        let buffer = self.buffer.read().unwrap();

        let mut level_counts = HashMap::new();
        let mut component_counts = HashMap::new();
        let mut error_count = 0;

        for event in buffer.iter() {
            *level_counts.entry(event.level).or_insert(0) += 1;
            *component_counts.entry(event.component.clone()).or_insert(0) += 1;

            if event.error.is_some() {
                error_count += 1;
            }
        }

        EventStatistics {
            total_events: buffer.len(),
            level_counts,
            component_counts,
            error_count,
            buffer_utilization: (buffer.len() as f64 / self.max_buffer_size as f64) * 100.0,
        }
    }
}

/// Event statistics
#[derive(Debug)]
pub struct EventStatistics {
    /// Total number of events
    pub total_events: usize,
    /// Count by log level
    pub level_counts: HashMap<LogLevel, usize>,
    /// Count by component
    pub component_counts: HashMap<String, usize>,
    /// Number of error events
    pub error_count: usize,
    /// Buffer utilization percentage
    pub buffer_utilization: f64,
}

/// Trait for exporting log events
pub trait LogExporter: Send + Sync {
    /// Export a log event
    fn export(&self, event: &LogEvent) -> Result<()>;
}

/// Console log exporter
#[derive(Debug)]
pub struct ConsoleLogExporter;

impl LogExporter for ConsoleLogExporter {
    fn export(&self, event: &LogEvent) -> Result<()> {
        println!("{}", event.format());
        Ok(())
    }
}

/// Trait for filtering events
pub trait EventFilter: Send + Sync {
    /// Check if an event should be logged
    fn should_log(&self, event: &LogEvent) -> bool;
}

/// Component filter
#[derive(Debug)]
pub struct ComponentFilter {
    /// Allowed components
    allowed_components: Vec<String>,
}

impl ComponentFilter {
    /// Create a new component filter
    pub fn new(allowed_components: Vec<String>) -> Self {
        Self { allowed_components }
    }
}

impl EventFilter for ComponentFilter {
    fn should_log(&self, event: &LogEvent) -> bool {
        self.allowed_components.contains(&event.component)
    }
}

/// Level filter
#[derive(Debug)]
pub struct LevelFilter {
    /// Minimum level
    min_level: LogLevel,
}

impl LevelFilter {
    /// Create a new level filter
    pub fn new(min_level: LogLevel) -> Self {
        Self { min_level }
    }
}

impl EventFilter for LevelFilter {
    fn should_log(&self, event: &LogEvent) -> bool {
        event.level >= self.min_level
    }
}

/// Rate limiting filter
#[derive(Debug)]
pub struct RateLimitFilter {
    /// Maximum events per second
    max_per_second: usize,
    /// Event counts
    counts: RwLock<VecDeque<DateTime<Utc>>>,
}

impl RateLimitFilter {
    /// Create a new rate limit filter
    pub fn new(max_per_second: usize) -> Self {
        Self {
            max_per_second,
            counts: RwLock::new(VecDeque::new()),
        }
    }
}

impl EventFilter for RateLimitFilter {
    fn should_log(&self, _event: &LogEvent) -> bool {
        let now = Utc::now();
        let mut counts = self.counts.write().unwrap();

        // Remove old entries
        while let Some(front) = counts.front() {
            if (now - *front).num_seconds() >= 1 {
                counts.pop_front();
            } else {
                break;
            }
        }

        // Check rate limit
        if counts.len() < self.max_per_second {
            counts.push_back(now);
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_levels() {
        assert!(LogLevel::Error > LogLevel::Warn);
        assert!(LogLevel::Warn > LogLevel::Info);
        assert!(LogLevel::Info.should_log(LogLevel::Debug));
        assert!(!LogLevel::Debug.should_log(LogLevel::Info));
    }

    #[test]
    fn test_log_event_creation() {
        let mut event = LogEvent::new(
            LogLevel::Info,
            "agent-1".to_string(),
            "test-component".to_string(),
            "Test message".to_string(),
        );

        event = event
            .with_field("user_id".to_string(), serde_json::json!("123"))
            .with_trace("trace-123".to_string(), "span-456".to_string());

        assert_eq!(event.level, LogLevel::Info);
        assert_eq!(event.agent_id, "agent-1");
        assert_eq!(event.fields.get("user_id"), Some(&serde_json::json!("123")));
        assert_eq!(event.trace_id, Some("trace-123".to_string()));
    }

    #[test]
    fn test_event_logger() {
        let logger = EventLogger::new("test-agent".to_string(), 100);

        // Log some events
        logger.info("component-1", "Info message").unwrap();
        logger.warn("component-2", "Warning message").unwrap();
        logger.error("component-1", "Error message", None).unwrap();

        // Check recent events
        let recent = logger.get_recent_events(10);
        assert_eq!(recent.len(), 3);

        // Check events by level
        let errors = logger.get_events_by_level(LogLevel::Error);
        assert_eq!(errors.len(), 1);

        // Check statistics
        let stats = logger.get_statistics();
        assert_eq!(stats.total_events, 3);
        assert_eq!(stats.level_counts.get(&LogLevel::Info), Some(&1));
        assert_eq!(stats.level_counts.get(&LogLevel::Warn), Some(&1));
        assert_eq!(stats.level_counts.get(&LogLevel::Error), Some(&1));
    }

    #[test]
    fn test_event_filters() {
        let mut logger = EventLogger::new("test-agent".to_string(), 100);

        // Add component filter
        let filter = ComponentFilter::new(vec!["allowed-component".to_string()]);
        logger.add_filter(Box::new(filter));

        // Log events
        logger
            .info("allowed-component", "This should be logged")
            .unwrap();
        logger
            .info("blocked-component", "This should be filtered")
            .unwrap();

        let events = logger.get_recent_events(10);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].component, "allowed-component");
    }

    #[test]
    fn test_rate_limit_filter() {
        let filter = RateLimitFilter::new(2);
        let event = LogEvent::new(
            LogLevel::Info,
            "agent".to_string(),
            "component".to_string(),
            "message".to_string(),
        );

        // First two should pass
        assert!(filter.should_log(&event));
        assert!(filter.should_log(&event));

        // Third should be rate limited
        assert!(!filter.should_log(&event));

        // Wait and try again
        std::thread::sleep(std::time::Duration::from_secs(1));
        assert!(filter.should_log(&event));
    }

    #[test]
    fn test_log_formatting() {
        let event = LogEvent::new(
            LogLevel::Error,
            "agent-1".to_string(),
            "auth".to_string(),
            "Authentication failed".to_string(),
        )
        .with_field("user_id".to_string(), serde_json::json!("user123"))
        .with_trace("trace-456".to_string(), "span-789".to_string());

        let formatted = event.format();
        assert!(formatted.contains("[ERROR]"));
        assert!(formatted.contains("[agent-1]"));
        assert!(formatted.contains("[auth]"));
        assert!(formatted.contains("Authentication failed"));
        assert!(formatted.contains("user_id=\"user123\""));
        assert!(formatted.contains("trace_id=trace-456"));
    }
}
