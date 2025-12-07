//! Debug configuration for llmspell
//!
//! Provides configuration structures and parsing for the debug system.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Debug configuration
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct DebugConfig {
    /// Whether debugging is enabled
    pub enabled: bool,

    /// Default debug level (Off, Error, Warn, Info, Debug, Trace)
    pub level: String,

    /// Output configuration
    pub output: DebugOutputConfig,

    /// Module filters for targeted debugging
    pub module_filters: ModuleFilterConfig,

    /// Performance profiling settings
    pub performance: PerformanceConfig,

    /// Stack trace settings
    pub stack_trace: StackTraceConfig,
}

impl Default for DebugConfig {
    fn default() -> Self {
        Self {
            enabled: false,            // Default: disabled in production
            level: "info".to_string(), // Default: info level
            output: DebugOutputConfig::default(),
            module_filters: ModuleFilterConfig::default(),
            performance: PerformanceConfig::default(),
            stack_trace: StackTraceConfig::default(),
        }
    }
}

/// Debug output configuration
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct DebugOutputConfig {
    /// Enable stdout output
    pub stdout: bool,

    /// Enable colored output for stdout
    pub colored: bool,

    /// File output configuration
    pub file: Option<FileOutputConfig>,

    /// Buffer configuration for capturing debug output
    pub buffer: BufferConfig,

    /// Format for output (text, json, json_pretty)
    pub format: String,
}

impl Default for DebugOutputConfig {
    fn default() -> Self {
        Self {
            stdout: true,  // Default: stdout enabled
            colored: true, // Default: colored output for better UX
            file: None,
            buffer: BufferConfig::default(),
            format: "text".to_string(), // Default: text format
        }
    }
}

/// File output configuration
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FileOutputConfig {
    /// Path to debug log file
    pub path: PathBuf,

    /// Whether to append to existing file
    pub append: bool,

    /// Maximum file size before rotation (e.g., "10MB")
    pub max_size: Option<String>,

    /// Number of rotated files to keep
    pub max_files: Option<usize>,
}

/// Buffer configuration for debug capture
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct BufferConfig {
    /// Whether to enable buffer capture
    pub enabled: bool,

    /// Maximum number of entries to buffer
    pub max_entries: usize,
}

impl Default for BufferConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_entries: 10000,
        }
    }
}

/// Module filter configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct ModuleFilterConfig {
    /// Modules to enable (wildcards supported)
    pub enabled: Vec<String>,

    /// Modules to disable (wildcards supported)
    pub disabled: Vec<String>,

    /// Per-module level overrides
    pub levels: HashMap<String, String>,
}

/// Performance profiling configuration
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PerformanceConfig {
    /// Enable performance tracking
    pub enabled: bool,

    /// Auto-report interval in seconds (0 = disabled)
    pub auto_report_interval: u64,

    /// Include child timers in reports
    pub include_children: bool,

    /// Minimum duration to include in reports (milliseconds)
    pub min_duration_ms: u64,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            auto_report_interval: 0,
            include_children: true,
            min_duration_ms: 1,
        }
    }
}

/// Stack trace configuration
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct StackTraceConfig {
    /// Enable stack trace capture
    pub enabled: bool,

    /// Capture on error level messages
    pub on_error: bool,

    /// Maximum stack depth to capture
    pub max_depth: usize,

    /// Include source file locations
    pub include_source: bool,
}

impl Default for StackTraceConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            on_error: true,
            max_depth: 50,
            include_source: true,
        }
    }
}

impl DebugConfig {
    /// Create debug config from environment variables
    pub fn from_env() -> Self {
        let mut config = Self::default();

        // LLMSPELL_DEBUG=true/false or 1/0
        if let Ok(enabled) = std::env::var("LLMSPELL_DEBUG") {
            config.enabled = enabled == "true" || enabled == "1";
        }

        // LLMSPELL_DEBUG_LEVEL=trace/debug/info/warn/error/off
        if let Ok(level) = std::env::var("LLMSPELL_DEBUG_LEVEL") {
            config.level = level.to_lowercase();
        }

        // LLMSPELL_DEBUG_OUTPUT=stdout,file:/path/to/debug.log
        if let Ok(output) = std::env::var("LLMSPELL_DEBUG_OUTPUT") {
            config.parse_output_env(&output);
        }

        // LLMSPELL_DEBUG_MODULES=+workflow.*,-agent.executor
        if let Ok(modules) = std::env::var("LLMSPELL_DEBUG_MODULES") {
            config.parse_module_filters_env(&modules);
        }

        // LLMSPELL_DEBUG_PERFORMANCE=true/false
        if let Ok(perf) = std::env::var("LLMSPELL_DEBUG_PERFORMANCE") {
            config.performance.enabled = perf == "true" || perf == "1";
        }

        config
    }

    /// Parse output configuration from environment string
    fn parse_output_env(&mut self, output: &str) {
        for part in output.split(',') {
            let part = part.trim();
            if part == "stdout" {
                self.output.stdout = true;
            } else if part == "colored" {
                self.output.colored = true;
            } else if let Some(path) = part.strip_prefix("file:") {
                self.output.file = Some(FileOutputConfig {
                    path: PathBuf::from(path),
                    append: true,
                    max_size: None,
                    max_files: None,
                });
            }
        }
    }

    /// Parse module filters from environment string
    fn parse_module_filters_env(&mut self, filters: &str) {
        for filter in filters.split(',') {
            let filter = filter.trim();
            if let Some(module) = filter.strip_prefix('+') {
                self.module_filters.enabled.push(module.to_string());
            } else if let Some(module) = filter.strip_prefix('-') {
                self.module_filters.disabled.push(module.to_string());
            }
        }
    }

    /// Merge with another config (other takes precedence)
    pub fn merge(mut self, other: Self) -> Self {
        if other.enabled {
            self.enabled = other.enabled;
        }
        if other.level != "info" {
            // Assuming "info" is default
            self.level = other.level;
        }

        // Merge output config
        if other.output.stdout {
            self.output.stdout = other.output.stdout;
        }
        if other.output.colored {
            self.output.colored = other.output.colored;
        }
        if other.output.file.is_some() {
            self.output.file = other.output.file;
        }

        // Merge module filters
        self.module_filters
            .enabled
            .extend(other.module_filters.enabled);
        self.module_filters
            .disabled
            .extend(other.module_filters.disabled);
        self.module_filters
            .levels
            .extend(other.module_filters.levels);

        // Merge performance config
        if other.performance.enabled {
            self.performance = other.performance;
        }

        // Merge stack trace config
        if other.stack_trace.enabled {
            self.stack_trace = other.stack_trace;
        }

        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_debug_config_default() {
        let config = DebugConfig::default();
        assert!(!config.enabled);
        assert_eq!(config.level, "info");
        assert!(config.output.stdout); // Changed default to true
    }

    #[test]
    fn test_debug_config_from_env() {
        std::env::set_var("LLMSPELL_DEBUG", "true");
        std::env::set_var("LLMSPELL_DEBUG_LEVEL", "debug");
        std::env::set_var("LLMSPELL_DEBUG_OUTPUT", "stdout,colored");

        let config = DebugConfig::from_env();
        assert!(config.enabled);
        assert_eq!(config.level, "debug");
        assert!(config.output.stdout);
        assert!(config.output.colored);

        // Clean up
        std::env::remove_var("LLMSPELL_DEBUG");
        std::env::remove_var("LLMSPELL_DEBUG_LEVEL");
        std::env::remove_var("LLMSPELL_DEBUG_OUTPUT");
    }

    #[test]
    fn test_module_filter_parsing() {
        std::env::set_var(
            "LLMSPELL_DEBUG_MODULES",
            "+workflow.*,-agent.executor,+tool",
        );

        let config = DebugConfig::from_env();
        assert_eq!(config.module_filters.enabled.len(), 2);
        assert_eq!(config.module_filters.disabled.len(), 1);
        assert!(config
            .module_filters
            .enabled
            .contains(&"workflow.*".to_string()));
        assert!(config.module_filters.enabled.contains(&"tool".to_string()));
        assert!(config
            .module_filters
            .disabled
            .contains(&"agent.executor".to_string()));

        std::env::remove_var("LLMSPELL_DEBUG_MODULES");
    }
}
