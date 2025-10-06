//! Debug bridge for script engines
//!
//! Provides a unified interface for all script engines to access
//! the centralized debug infrastructure.

use llmspell_utils::debug::{global_debug_manager, DebugEntry, DebugLevel, PerformanceTracker};
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;

/// Language-neutral stack trace verbosity level
///
/// Abstracts stack trace detail configuration from language-specific types.
/// Each script engine implements `From<StackTraceLevel>` for its options type,
/// enabling language-neutral debug configuration across Lua, JavaScript, Python, etc.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StackTraceLevel {
    /// Full stack trace with locals and upvalues (verbose, for trace-level debugging)
    Trace,
    /// Error-focused stack trace (minimal overhead, for error reporting)
    Error,
    /// Standard stack trace (default detail level)
    Default,
}

/// Debug bridge that script engines interact with
#[derive(Clone)]
pub struct DebugBridge {
    /// Reference to the global debug manager
    manager: Arc<llmspell_utils::debug::DebugManager>,
    /// Active performance trackers by ID (using interior mutability)
    trackers: Arc<Mutex<HashMap<String, Arc<PerformanceTracker>>>>,
}

impl DebugBridge {
    /// Create a new debug bridge
    #[must_use]
    pub fn new() -> Self {
        Self {
            manager: global_debug_manager(),
            trackers: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Log a message at the specified level
    pub fn log(&self, level: &str, message: &str, module: Option<&str>) {
        if let Ok(debug_level) = level.parse::<DebugLevel>() {
            self.manager
                .log(debug_level, message, module.map(String::from));
        }
    }

    /// Log with metadata
    pub fn log_with_metadata(
        &self,
        level: &str,
        message: &str,
        module: Option<&str>,
        metadata: Value,
    ) {
        if let Ok(debug_level) = level.parse::<DebugLevel>() {
            self.manager.log_with_metadata(
                debug_level,
                message,
                module.map(String::from),
                metadata,
            );
        }
    }

    /// Start a performance timer
    #[must_use]
    pub fn start_timer(&self, name: &str) -> String {
        let tracker = self.manager.start_timer(name);
        let id = format!("timer_{}", uuid::Uuid::new_v4());
        self.trackers.lock().insert(id.clone(), tracker);
        id
    }

    /// Stop a timer and get the duration in milliseconds
    #[must_use]
    pub fn stop_timer(&self, id: &str) -> Option<f64> {
        self.trackers
            .lock()
            .remove(id)
            .map(|tracker| tracker.stop().as_secs_f64() * 1000.0)
    }

    /// Record a lap for a timer
    #[must_use]
    pub fn lap_timer(&self, id: &str, lap_name: &str) -> bool {
        self.trackers.lock().get(id).is_some_and(|tracker| {
            tracker.lap(lap_name);
            true
        })
    }

    /// Get the elapsed time for a timer without stopping it
    #[must_use]
    pub fn elapsed_timer(&self, id: &str) -> Option<f64> {
        self.trackers
            .lock()
            .get(id)
            .map(|tracker| tracker.elapsed().as_secs_f64() * 1000.0)
    }

    /// Set the debug level
    #[must_use]
    pub fn set_level(&self, level: &str) -> bool {
        level.parse::<DebugLevel>().is_ok_and(|debug_level| {
            self.manager.set_level(debug_level);
            true
        })
    }

    /// Get the current debug level
    #[must_use]
    pub fn get_level(&self) -> String {
        self.manager.get_level().to_string()
    }

    /// Enable or disable debugging
    pub fn set_enabled(&self, enabled: bool) {
        self.manager.set_enabled(enabled);
    }

    /// Check if debugging is enabled
    #[must_use]
    pub fn is_enabled(&self) -> bool {
        self.manager.is_enabled()
    }

    /// Add a module filter
    pub fn add_module_filter(&self, pattern: &str, enabled: bool) {
        self.manager.add_module_filter(pattern, enabled);
    }

    /// Clear all module filters
    pub fn clear_module_filters(&self) {
        self.manager.clear_module_filters();
    }

    /// Get module filter summary
    #[must_use]
    pub fn get_filter_summary(&self) -> llmspell_utils::debug::FilterSummary {
        self.manager.get_filter_summary()
    }

    /// Remove a specific filter pattern
    #[must_use]
    pub fn remove_module_filter(&self, pattern: &str) -> bool {
        self.manager.remove_module_filter(pattern)
    }

    /// Set default filter behavior  
    pub fn set_default_filter_enabled(&self, enabled: bool) {
        self.manager.set_default_filter_enabled(enabled);
    }

    /// Add advanced filter rule
    #[must_use]
    pub fn add_filter_rule(&self, pattern: &str, pattern_type: &str, enabled: bool) -> bool {
        use llmspell_utils::debug::{FilterPattern, FilterRule};

        let filter_pattern = match pattern_type {
            "exact" => FilterPattern::Exact(pattern.to_string()),
            "wildcard" => FilterPattern::Wildcard(pattern.to_string()),
            "regex" => FilterPattern::Regex(pattern.to_string()),
            "hierarchical" => FilterPattern::Hierarchical(pattern.to_string()),
            _ => return false,
        };

        let rule = FilterRule {
            pattern: filter_pattern,
            enabled,
            description: None,
        };

        self.manager.add_filter_rule(rule);
        true
    }

    /// Get captured debug entries
    pub fn get_captured_entries(&self, limit: Option<usize>) -> Vec<DebugEntryInfo> {
        let entries = limit.map_or_else(
            || self.manager.get_captured_entries(),
            |n| self.manager.get_last_entries(n),
        );

        entries.into_iter().map(Into::into).collect()
    }

    /// Clear captured entries
    pub fn clear_captured(&self) {
        self.manager.clear_captured();
    }

    /// Generate a performance report
    #[must_use]
    pub fn generate_performance_report(&self) -> String {
        self.manager.generate_performance_report()
    }

    /// Dump a value for debugging (pretty-print) - JSON fallback
    #[must_use]
    pub fn dump_value(&self, value: &Value, label: Option<&str>) -> String {
        let pretty = serde_json::to_string_pretty(value)
            .unwrap_or_else(|_| "Failed to serialize".to_string());

        if let Some(label) = label {
            format!("{label}: {pretty}")
        } else {
            pretty
        }
    }

    /// Dump a value with enhanced formatting options (for script engines with advanced dumping)
    #[must_use]
    pub fn dump_value_enhanced(
        &self,
        value: &Value,
        label: Option<&str>,
        _compact: bool,
    ) -> String {
        // This will be used by script-specific implementations
        self.dump_value(value, label)
    }

    /// Get memory statistics (placeholder for future implementation)
    #[must_use]
    pub const fn get_memory_stats(&self) -> MemoryStats {
        MemoryStats {
            used_bytes: 0,
            allocated_bytes: 0,
            resident_bytes: 0,
            collections: 0,
        }
    }

    /// Generate JSON performance report
    ///
    /// # Errors
    ///
    /// Returns an error if JSON serialization fails.
    pub fn generate_json_report(&self) -> Result<String, String> {
        // Use the global debug manager's profiler
        let profiler = llmspell_utils::debug::performance::Profiler::new();
        profiler
            .generate_json_report()
            .map_err(|e| format!("JSON serialization failed: {e}"))
    }

    /// Generate flame graph compatible output
    #[must_use]
    pub fn generate_flame_graph(&self) -> String {
        let profiler = llmspell_utils::debug::performance::Profiler::new();
        profiler.generate_flame_graph()
    }

    /// Get memory usage snapshot
    #[must_use]
    pub fn get_memory_snapshot(&self) -> llmspell_utils::debug::performance::MemorySnapshot {
        let profiler = llmspell_utils::debug::performance::Profiler::new();
        profiler.generate_memory_snapshot()
    }

    /// Record a custom event on a timer
    #[must_use]
    pub fn record_event(
        &self,
        timer_id: &str,
        event_name: &str,
        metadata: Option<serde_json::Value>,
    ) -> bool {
        self.trackers.lock().get(timer_id).is_some_and(|tracker| {
            tracker.event(event_name, metadata);
            true
        })
    }

    /// Get stack trace level for different debug levels
    ///
    /// Returns a language-neutral stack trace level that can be converted
    /// to language-specific options via `From<StackTraceLevel>` trait implementations.
    ///
    /// # Examples
    /// ```ignore
    /// let level = bridge.stack_trace_options_for_level("trace");
    /// // Lua: let opts: StackTraceOptions = level.into();
    /// // JS: let opts: JsStackTraceOptions = level.into();
    /// ```
    #[must_use]
    pub fn stack_trace_options_for_level(&self, level: &str) -> StackTraceLevel {
        match level {
            "trace" | "TRACE" => StackTraceLevel::Trace,
            "error" | "ERROR" => StackTraceLevel::Error,
            _ => StackTraceLevel::Default,
        }
    }
}

impl Default for DebugBridge {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for DebugBridge {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DebugBridge")
            .field("enabled", &self.is_enabled())
            .field("level", &self.get_level())
            .field("tracker_count", &self.trackers.lock().len())
            .finish_non_exhaustive()
    }
}

/// Simplified debug entry for script consumption
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugEntryInfo {
    pub timestamp: String,
    pub level: String,
    pub module: Option<String>,
    pub message: String,
    pub metadata: Option<Value>,
}

impl From<DebugEntry> for DebugEntryInfo {
    fn from(entry: DebugEntry) -> Self {
        Self {
            timestamp: entry.timestamp.to_rfc3339(),
            level: entry.level.to_string(),
            module: entry.module,
            message: entry.message,
            metadata: entry.metadata,
        }
    }
}

/// Memory statistics for script debugging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStats {
    pub used_bytes: u64,
    pub allocated_bytes: u64,
    pub resident_bytes: u64,
    pub collections: u32,
}

/// Timer handle for script usage
#[derive(Debug, Clone)]
pub struct TimerHandle {
    pub id: String,
    pub name: String,
}

impl TimerHandle {
    /// Create a new timer handle
    #[must_use]
    pub const fn new(id: String, name: String) -> Self {
        Self { id, name }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_debug_bridge_logging() {
        let bridge = DebugBridge::new();

        // Test basic logging
        bridge.log("info", "Test message", Some("test_module"));
        bridge.log("debug", "Debug message", None);

        // Test invalid level
        bridge.log("invalid", "Should not log", None);
    }

    #[test]
    fn test_debug_bridge_timer() {
        let bridge = DebugBridge::new();

        // Start a timer
        let timer_id = bridge.start_timer("test_timer");
        assert!(!timer_id.is_empty());

        // Check elapsed time
        let elapsed = bridge.elapsed_timer(&timer_id);
        assert!(elapsed.is_some());

        // Record a lap
        assert!(bridge.lap_timer(&timer_id, "checkpoint"));

        // Stop the timer
        let duration = bridge.stop_timer(&timer_id);
        assert!(duration.is_some());

        // Timer should no longer exist
        assert!(bridge.elapsed_timer(&timer_id).is_none());
    }

    #[test]
    fn test_debug_bridge_configuration() {
        let bridge = DebugBridge::new();

        // Test level setting
        assert!(bridge.set_level("debug"));
        assert_eq!(bridge.get_level(), "DEBUG");

        // Test invalid level
        assert!(!bridge.set_level("invalid"));

        // Test enable/disable
        bridge.set_enabled(false);
        assert!(!bridge.is_enabled());
        bridge.set_enabled(true);
        assert!(bridge.is_enabled());
    }

    #[test]
    fn test_module_filters() {
        let bridge = DebugBridge::new();

        // Add filters
        bridge.add_module_filter("workflow", true);
        bridge.add_module_filter("agent.internal", false);

        // Clear filters
        bridge.clear_module_filters();
    }

    #[test]
    fn test_value_dumping() {
        let bridge = DebugBridge::new();

        let value = serde_json::json!({
            "key": "value",
            "nested": {
                "array": [1, 2, 3]
            }
        });

        let dump = bridge.dump_value(&value, Some("test_object"));
        assert!(dump.contains("test_object"));
        assert!(dump.contains("\"key\": \"value\""));
    }
}
