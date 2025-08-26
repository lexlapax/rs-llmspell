//! Debug infrastructure for script engines
//!
//! Provides centralized debug management that all script engines call into,
//! with hierarchical logging levels, performance tracking, and flexible output.

pub mod entry;
pub mod levels;
pub mod module_filter;
pub mod output;
pub mod performance;

// Re-export commonly used types
pub use self::entry::DebugEntry;
pub use self::levels::DebugLevel;
pub use self::module_filter::{EnhancedModuleFilter, FilterPattern, FilterRule, FilterSummary};
pub use self::output::{BufferOutput, DebugOutput, FileOutput, MultiOutput, StdoutOutput};
pub use self::performance::{PerformanceTracker, Profiler};

// Internal imports (no duplicates needed since we're re-exporting)
use dashmap::DashMap;
use parking_lot::RwLock;
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::sync::Arc;
use std::sync::LazyLock;

/// Global debug manager instance
static GLOBAL_DEBUG_MANAGER: LazyLock<Arc<DebugManager>> =
    LazyLock::new(|| Arc::new(DebugManager::new()));

/// Get the global debug manager instance
#[must_use]
pub fn global_debug_manager() -> Arc<DebugManager> {
    GLOBAL_DEBUG_MANAGER.clone()
}

/// Central debug manager that coordinates all debug operations
pub struct DebugManager {
    /// Current debug level
    level: AtomicU8,
    /// Whether debugging is enabled at all
    enabled: AtomicBool,
    /// Output handler for debug messages
    output_handler: Arc<RwLock<Box<dyn DebugOutput>>>,
    /// Performance profiler
    profiler: Arc<Profiler>,
    /// Active performance trackers by ID
    performance_trackers: DashMap<String, Arc<PerformanceTracker>>,
    /// Module filters for targeted debugging
    module_filters: Arc<RwLock<EnhancedModuleFilter>>,
    /// Capture buffer for later analysis
    capture_buffer: Arc<BufferOutput>,
}

impl DebugManager {
    /// Create a new debug manager with default settings
    #[must_use]
    pub fn new() -> Self {
        Self::with_output(Box::new(StdoutOutput::new(true)))
    }

    /// Create a debug manager with a specific output handler
    #[must_use]
    pub fn with_output(output: Box<dyn DebugOutput>) -> Self {
        let capture_buffer = Arc::new(BufferOutput::new(10000));

        // Combine the provided output with the capture buffer
        let multi_output = MultiOutput::new(vec![
            output,
            Box::new(capture_buffer.clone()) as Box<dyn DebugOutput>,
        ]);

        Self {
            level: AtomicU8::new(DebugLevel::Info as u8),
            enabled: AtomicBool::new(true),
            output_handler: Arc::new(RwLock::new(Box::new(multi_output))),
            profiler: Arc::new(Profiler::new()),
            performance_trackers: DashMap::new(),
            module_filters: Arc::new(RwLock::new(EnhancedModuleFilter::new())),
            capture_buffer,
        }
    }

    /// Set the debug level
    pub fn set_level(&self, level: DebugLevel) {
        self.level.store(level as u8, Ordering::Relaxed);
    }

    /// Get the current debug level
    #[must_use]
    pub fn get_level(&self) -> DebugLevel {
        DebugLevel::from_u8(self.level.load(Ordering::Relaxed)).unwrap_or(DebugLevel::Info)
    }

    /// Enable or disable debugging entirely
    pub fn set_enabled(&self, enabled: bool) {
        self.enabled.store(enabled, Ordering::Relaxed);
    }

    /// Check if debugging is enabled
    #[must_use]
    pub fn is_enabled(&self) -> bool {
        self.enabled.load(Ordering::Relaxed)
    }

    /// Log a debug message
    pub fn log(&self, level: DebugLevel, message: impl Into<String>, module: Option<String>) {
        // Quick exit if disabled or level filtered
        if !self.is_enabled() || !self.should_log(level, module.as_deref()) {
            return;
        }

        let mut entry = DebugEntry::new(level, message);
        if let Some(m) = module {
            entry = entry.with_module(m);
        }

        self.output_handler.read().write(&entry);
    }

    /// Log with metadata
    pub fn log_with_metadata(
        &self,
        level: DebugLevel,
        message: impl Into<String>,
        module: Option<String>,
        metadata: serde_json::Value,
    ) {
        if !self.is_enabled() || !self.should_log(level, module.as_deref()) {
            return;
        }

        let mut entry = DebugEntry::new(level, message).with_metadata(metadata);

        if let Some(m) = module {
            entry = entry.with_module(m);
        }

        self.output_handler.read().write(&entry);
    }

    /// Start a performance timer
    pub fn start_timer(&self, name: impl Into<String>) -> Arc<PerformanceTracker> {
        let name = name.into();
        let tracker = Arc::new(PerformanceTracker::new(name.clone()));
        self.performance_trackers
            .insert(name.clone(), tracker.clone());
        tracker
    }

    /// Get a performance tracker by name
    #[must_use]
    pub fn get_tracker(&self, name: &str) -> Option<Arc<PerformanceTracker>> {
        self.performance_trackers.get(name).map(|t| t.clone())
    }

    /// Generate a performance report
    #[must_use]
    pub fn generate_performance_report(&self) -> String {
        self.profiler.generate_report().format()
    }

    /// Get captured debug entries
    #[must_use]
    pub fn get_captured_entries(&self) -> Vec<DebugEntry> {
        self.capture_buffer.get_entries()
    }

    /// Get the last N captured entries
    #[must_use]
    pub fn get_last_entries(&self, n: usize) -> Vec<DebugEntry> {
        self.capture_buffer.get_last_entries(n)
    }

    /// Clear captured entries
    pub fn clear_captured(&self) {
        self.capture_buffer.clear();
    }

    /// Set the output handler
    pub fn set_output(&self, output: Box<dyn DebugOutput>) {
        // Keep the capture buffer in the multi-output
        let multi = MultiOutput::new(vec![
            output,
            Box::new(self.capture_buffer.clone()) as Box<dyn DebugOutput>,
        ]);
        *self.output_handler.write() = Box::new(multi);
    }

    /// Add a module filter
    pub fn add_module_filter(&self, pattern: &str, enabled: bool) {
        self.module_filters.write().add_filter(pattern, enabled);
    }

    /// Clear all module filters
    pub fn clear_module_filters(&self) {
        self.module_filters.write().clear();
    }

    /// Get module filter summary
    pub fn get_filter_summary(&self) -> FilterSummary {
        self.module_filters.read().get_filter_summary()
    }

    /// Remove a specific filter pattern
    pub fn remove_module_filter(&self, pattern: &str) -> bool {
        self.module_filters.write().remove_filter(pattern)
    }

    /// Set default filter behavior
    pub fn set_default_filter_enabled(&self, enabled: bool) {
        self.module_filters.write().set_default_enabled(enabled);
    }

    /// Add a filter rule with full configuration
    pub fn add_filter_rule(&self, rule: FilterRule) {
        self.module_filters.write().add_rule(rule);
    }

    /// Check if a message should be logged based on level and module
    fn should_log(&self, level: DebugLevel, module: Option<&str>) -> bool {
        // Check level first
        if !level.should_show(self.get_level()) {
            return false;
        }

        // Then check module filter if module is specified
        if let Some(m) = module {
            self.module_filters.read().should_log(m)
        } else {
            true // No module means always log if level passes
        }
    }

    /// Flush all outputs
    pub fn flush(&self) {
        self.output_handler.read().flush();
    }
}

impl Default for DebugManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Convenience functions for the global debug manager
pub mod global {
    use super::{global_debug_manager, Arc, DebugLevel, PerformanceTracker};

    /// Log a trace message
    pub fn trace(message: impl Into<String>) {
        global_debug_manager().log(DebugLevel::Trace, message, None);
    }

    /// Log a debug message
    pub fn debug(message: impl Into<String>) {
        global_debug_manager().log(DebugLevel::Debug, message, None);
    }

    /// Log an info message
    pub fn info(message: impl Into<String>) {
        global_debug_manager().log(DebugLevel::Info, message, None);
    }

    /// Log a warning message
    pub fn warn(message: impl Into<String>) {
        global_debug_manager().log(DebugLevel::Warn, message, None);
    }

    /// Log an error message
    pub fn error(message: impl Into<String>) {
        global_debug_manager().log(DebugLevel::Error, message, None);
    }

    /// Set the global debug level
    pub fn set_level(level: DebugLevel) {
        global_debug_manager().set_level(level);
    }

    /// Start a timer
    pub fn timer(name: impl Into<String>) -> Arc<PerformanceTracker> {
        global_debug_manager().start_timer(name)
    }
}

#[cfg(test)]
mod tests {
    use super::{DebugLevel, DebugManager};
    use serde_json::json;

    #[test]
    fn test_debug_manager_levels() {
        let manager = DebugManager::new();
        manager.set_level(DebugLevel::Warn);

        // These should not be logged
        manager.log(DebugLevel::Debug, "Debug message", None);
        manager.log(DebugLevel::Info, "Info message", None);

        // These should be logged
        manager.log(DebugLevel::Warn, "Warning message", None);
        manager.log(DebugLevel::Error, "Error message", None);

        let entries = manager.get_captured_entries();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].level, DebugLevel::Warn);
        assert_eq!(entries[1].level, DebugLevel::Error);
    }

    #[test]
    fn test_module_filtering() {
        let manager = DebugManager::new();
        manager.set_level(DebugLevel::Debug);

        // Enable workflow modules using hierarchical pattern
        manager.add_module_filter("workflow.*", true);

        manager.log(
            DebugLevel::Info,
            "Test 1",
            Some("workflow.step".to_string()),
        );
        manager.log(
            DebugLevel::Info,
            "Test 2",
            Some("agent.executor".to_string()),
        );
        manager.log(
            DebugLevel::Info,
            "Test 3",
            Some("workflow.parallel".to_string()),
        );

        let entries = manager.get_captured_entries();
        assert_eq!(entries.len(), 2); // Only workflow messages
        assert!(entries[0].module.as_ref().unwrap().contains("workflow"));
        assert!(entries[1].module.as_ref().unwrap().contains("workflow"));
    }

    #[test]
    fn test_metadata_logging() {
        let manager = DebugManager::new();

        manager.log_with_metadata(
            DebugLevel::Info,
            "Operation completed",
            Some("test".to_string()),
            json!({
                "duration_ms": 150,
                "items_processed": 42
            }),
        );

        let entries = manager.get_captured_entries();
        assert_eq!(entries.len(), 1);
        assert!(entries[0].metadata.is_some());
    }
}
