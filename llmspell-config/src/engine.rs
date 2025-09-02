//! ABOUTME: UnifiedProtocolEngine configuration for llmspell
//! ABOUTME: Handles engine binding, routing, debug, REPL, and performance settings

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Import RoutingStrategy from the engine
pub use llmspell_engine::engine::RoutingStrategy;

/// Central UnifiedProtocolEngine configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct EngineConfig {
    /// Network binding configuration
    pub binding: BindingConfig,
    /// Message routing configuration
    pub routing: RoutingConfig,
    /// Debug system configuration
    pub debug: DebugConfig,
    /// REPL behavior configuration
    pub repl: ReplConfig,
    /// Performance configuration
    pub performance: PerformanceConfig,
}

/// Network binding configuration for UnifiedProtocolEngine
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct BindingConfig {
    /// IP address to bind to
    pub ip: String,
    /// Starting port for port range
    pub port_range_start: u16,
    /// Maximum number of concurrent clients
    pub max_clients: usize,
    /// Connection timeout in seconds
    pub connection_timeout_seconds: u64,
    /// Keep-alive timeout in seconds
    pub keep_alive_timeout_seconds: u64,
}

impl Default for BindingConfig {
    fn default() -> Self {
        Self {
            ip: "127.0.0.1".to_string(),
            port_range_start: 9555,
            max_clients: 10,
            connection_timeout_seconds: 30,
            keep_alive_timeout_seconds: 60,
        }
    }
}

/// Message routing configuration for different channel types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct RoutingConfig {
    /// Routing strategy for Shell channel
    pub shell_strategy: RoutingStrategy,
    /// Routing strategy for IOPub channel
    pub iopub_strategy: RoutingStrategy,
    /// Routing strategy for Control channel
    pub control_strategy: RoutingStrategy,
    /// Default routing strategy for custom channels
    pub default_strategy: RoutingStrategy,
    /// Enable routing metrics collection
    pub enable_metrics: bool,
    /// Handler registration timeout in milliseconds
    pub handler_registration_timeout_ms: u64,
}

impl Default for RoutingConfig {
    fn default() -> Self {
        Self {
            shell_strategy: RoutingStrategy::Direct,
            iopub_strategy: RoutingStrategy::Broadcast,
            control_strategy: RoutingStrategy::RoundRobin,
            default_strategy: RoutingStrategy::Direct,
            enable_metrics: false,
            handler_registration_timeout_ms: 5000,
        }
    }
}

/// Debug configuration for UnifiedProtocolEngine integration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct DebugConfig {
    /// Enable debug mode globally
    pub enabled: bool,
    /// Enable breakpoint functionality
    pub breakpoints_enabled: bool,
    /// Enable step debugging
    pub step_debugging_enabled: bool,
    /// Enable variable inspection
    pub variable_inspection_enabled: bool,
    /// Enable hook profiling for debug overhead measurement
    pub hook_profiling_enabled: bool,
    /// Debug session timeout in seconds
    pub session_timeout_seconds: u64,
    /// Maximum debug messages to buffer
    pub max_debug_buffer_size: usize,
    /// Enable debug protocol tracing
    pub protocol_tracing_enabled: bool,
}

impl Default for DebugConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            breakpoints_enabled: true,
            step_debugging_enabled: true,
            variable_inspection_enabled: true,
            hook_profiling_enabled: false,
            session_timeout_seconds: 1800, // 30 minutes
            max_debug_buffer_size: 10000,
            protocol_tracing_enabled: false,
        }
    }
}

/// REPL behavior configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ReplConfig {
    /// Command history size
    pub history_size: usize,
    /// Path to history file
    pub history_file: Option<PathBuf>,
    /// Enable tab completion
    pub tab_completion: bool,
    /// Enable Ctrl+R reverse search
    pub ctrl_r_search: bool,
    /// Output formatting style
    pub output_formatting: OutputFormat,
    /// Enable multiline input support
    pub multiline_support: bool,
    /// Prompt string for interactive mode
    pub prompt: String,
    /// Continuation prompt for multiline input
    pub continuation_prompt: String,
    /// Maximum line length before wrapping
    pub max_line_length: usize,
    /// Enable syntax highlighting
    pub syntax_highlighting: bool,
}

impl Default for ReplConfig {
    fn default() -> Self {
        Self {
            history_size: 1000,
            history_file: None, // Will default to ~/.llmspell/history
            tab_completion: true,
            ctrl_r_search: true,
            output_formatting: OutputFormat::Enhanced,
            multiline_support: true,
            prompt: "llmspell> ".to_string(),
            continuation_prompt: "... ".to_string(),
            max_line_length: 120,
            syntax_highlighting: true,
        }
    }
}

/// Output formatting options for REPL
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OutputFormat {
    /// Plain text output
    Plain,
    /// Enhanced output with colors and formatting
    Enhanced,
    /// JSON formatted output
    Json,
    /// Compact single-line output
    Compact,
}

impl Default for OutputFormat {
    fn default() -> Self {
        Self::Enhanced
    }
}

/// Performance configuration for UnifiedProtocolEngine
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct PerformanceConfig {
    /// Maximum message processing concurrency
    pub max_concurrent_messages: usize,
    /// Message processing timeout in milliseconds
    pub message_timeout_ms: u64,
    /// Enable message batching for performance
    pub enable_batching: bool,
    /// Batch size for message processing
    pub batch_size: usize,
    /// Batch timeout in milliseconds
    pub batch_timeout_ms: u64,
    /// Connection pool size
    pub connection_pool_size: usize,
    /// Enable connection pooling
    pub enable_connection_pooling: bool,
    /// Memory limit per connection in bytes
    pub memory_limit_per_connection_bytes: usize,
    /// Enable performance metrics collection
    pub enable_performance_metrics: bool,
    /// Metrics collection interval in seconds
    pub metrics_collection_interval_seconds: u64,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            max_concurrent_messages: 100,
            message_timeout_ms: 30000, // 30 seconds
            enable_batching: false,
            batch_size: 10,
            batch_timeout_ms: 100,
            connection_pool_size: 10,
            enable_connection_pooling: true,
            memory_limit_per_connection_bytes: 10_000_000, // 10MB
            enable_performance_metrics: false,
            metrics_collection_interval_seconds: 60,
        }
    }
}

impl EngineConfig {
    /// Create a new builder for EngineConfig
    #[must_use]
    pub fn builder() -> EngineConfigBuilder {
        EngineConfigBuilder::new()
    }

    /// Validate the configuration for consistency
    pub fn validate(&self) -> Result<(), EngineConfigError> {
        // Validate binding configuration
        if self.binding.port_range_start == 0 {
            return Err(EngineConfigError::Validation {
                field: "binding.port_range_start".to_string(),
                message: "Port range start must be greater than 0".to_string(),
            });
        }

        if self.binding.max_clients == 0 {
            return Err(EngineConfigError::Validation {
                field: "binding.max_clients".to_string(),
                message: "Maximum clients must be greater than 0".to_string(),
            });
        }

        // Validate performance configuration
        if self.performance.max_concurrent_messages == 0 {
            return Err(EngineConfigError::Validation {
                field: "performance.max_concurrent_messages".to_string(),
                message: "Maximum concurrent messages must be greater than 0".to_string(),
            });
        }

        if self.performance.enable_batching && self.performance.batch_size == 0 {
            return Err(EngineConfigError::Validation {
                field: "performance.batch_size".to_string(),
                message: "Batch size must be greater than 0 when batching is enabled".to_string(),
            });
        }

        // Validate REPL configuration
        if self.repl.history_size == 0 {
            return Err(EngineConfigError::Validation {
                field: "repl.history_size".to_string(),
                message: "History size must be greater than 0".to_string(),
            });
        }

        // Validate debug configuration
        if self.debug.session_timeout_seconds == 0 {
            return Err(EngineConfigError::Validation {
                field: "debug.session_timeout_seconds".to_string(),
                message: "Debug session timeout must be greater than 0".to_string(),
            });
        }

        Ok(())
    }
}

/// Builder for EngineConfig
#[derive(Debug, Clone)]
pub struct EngineConfigBuilder {
    config: EngineConfig,
}

impl EngineConfigBuilder {
    /// Create a new builder with default configuration
    #[must_use]
    pub fn new() -> Self {
        Self {
            config: EngineConfig::default(),
        }
    }

    /// Set the binding configuration
    #[must_use]
    pub fn binding(mut self, binding: BindingConfig) -> Self {
        self.config.binding = binding;
        self
    }

    /// Set the routing configuration
    #[must_use]
    pub fn routing(mut self, routing: RoutingConfig) -> Self {
        self.config.routing = routing;
        self
    }

    /// Set the debug configuration
    #[must_use]
    pub fn debug(mut self, debug: DebugConfig) -> Self {
        self.config.debug = debug;
        self
    }

    /// Set the REPL configuration
    #[must_use]
    pub fn repl(mut self, repl: ReplConfig) -> Self {
        self.config.repl = repl;
        self
    }

    /// Set the performance configuration
    #[must_use]
    pub fn performance(mut self, performance: PerformanceConfig) -> Self {
        self.config.performance = performance;
        self
    }

    /// Build the configuration
    #[must_use]
    pub fn build(self) -> EngineConfig {
        self.config
    }
}

impl Default for EngineConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Engine configuration errors
#[derive(Debug, thiserror::Error)]
pub enum EngineConfigError {
    #[error("Configuration validation failed in field '{field}': {message}")]
    Validation { field: String, message: String },

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_config_default() {
        let config = EngineConfig::default();
        assert_eq!(config.binding.ip, "127.0.0.1");
        assert_eq!(config.binding.port_range_start, 9555);
        assert_eq!(config.binding.max_clients, 10);

        assert!(matches!(
            config.routing.shell_strategy,
            RoutingStrategy::Direct
        ));
        assert!(matches!(
            config.routing.iopub_strategy,
            RoutingStrategy::Broadcast
        ));
        assert!(matches!(
            config.routing.control_strategy,
            RoutingStrategy::RoundRobin
        ));

        assert!(config.debug.enabled);
        assert!(config.debug.breakpoints_enabled);
        assert!(config.debug.step_debugging_enabled);

        assert_eq!(config.repl.history_size, 1000);
        assert!(config.repl.tab_completion);
        assert!(config.repl.ctrl_r_search);

        assert_eq!(config.performance.max_concurrent_messages, 100);
        assert!(!config.performance.enable_batching);
    }

    #[test]
    fn test_engine_config_builder() {
        let config = EngineConfig::builder()
            .binding(BindingConfig {
                ip: "0.0.0.0".to_string(),
                port_range_start: 8080,
                max_clients: 20,
                ..Default::default()
            })
            .debug(DebugConfig {
                enabled: false,
                ..Default::default()
            })
            .build();

        assert_eq!(config.binding.ip, "0.0.0.0");
        assert_eq!(config.binding.port_range_start, 8080);
        assert_eq!(config.binding.max_clients, 20);
        assert!(!config.debug.enabled);
    }

    #[test]
    fn test_engine_config_validation_success() {
        let config = EngineConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_engine_config_validation_failure() {
        let mut config = EngineConfig::default();
        config.binding.port_range_start = 0;

        let result = config.validate();
        assert!(result.is_err());

        match result.unwrap_err() {
            EngineConfigError::Validation { field, message } => {
                assert_eq!(field, "binding.port_range_start");
                assert!(message.contains("must be greater than 0"));
            }
            _ => panic!("Expected validation error"),
        }
    }

    #[test]
    fn test_batching_validation() {
        let mut config = EngineConfig::default();
        config.performance.enable_batching = true;
        config.performance.batch_size = 0;

        let result = config.validate();
        assert!(result.is_err());

        match result.unwrap_err() {
            EngineConfigError::Validation { field, .. } => {
                assert_eq!(field, "performance.batch_size");
            }
            _ => panic!("Expected validation error"),
        }
    }

    #[test]
    fn test_output_format_serialization() {
        let formats = vec![
            OutputFormat::Plain,
            OutputFormat::Enhanced,
            OutputFormat::Json,
            OutputFormat::Compact,
        ];

        for format in formats {
            let serialized = serde_json::to_string(&format).unwrap();
            let deserialized: OutputFormat = serde_json::from_str(&serialized).unwrap();

            match (format, deserialized) {
                (OutputFormat::Plain, OutputFormat::Plain) => (),
                (OutputFormat::Enhanced, OutputFormat::Enhanced) => (),
                (OutputFormat::Json, OutputFormat::Json) => (),
                (OutputFormat::Compact, OutputFormat::Compact) => (),
                _ => panic!("Serialization/deserialization mismatch"),
            }
        }
    }

    #[test]
    fn test_repl_config_default_history_file() {
        let config = ReplConfig::default();
        assert!(config.history_file.is_none());
        assert_eq!(config.history_size, 1000);
        assert!(config.tab_completion);
        assert!(config.ctrl_r_search);
        assert_eq!(config.prompt, "llmspell> ");
        assert_eq!(config.continuation_prompt, "... ");
    }

    #[test]
    fn test_performance_config_defaults() {
        let config = PerformanceConfig::default();
        assert_eq!(config.max_concurrent_messages, 100);
        assert_eq!(config.message_timeout_ms, 30000);
        assert!(!config.enable_batching);
        assert_eq!(config.batch_size, 10);
        assert!(config.enable_connection_pooling);
        assert_eq!(config.connection_pool_size, 10);
    }

    #[test]
    fn test_debug_config_defaults() {
        let config = DebugConfig::default();
        assert!(config.enabled);
        assert!(config.breakpoints_enabled);
        assert!(config.step_debugging_enabled);
        assert!(config.variable_inspection_enabled);
        assert!(!config.hook_profiling_enabled);
        assert_eq!(config.session_timeout_seconds, 1800);
        assert_eq!(config.max_debug_buffer_size, 10000);
        assert!(!config.protocol_tracing_enabled);
    }
}
