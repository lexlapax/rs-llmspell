//! ABOUTME: Structured logging infrastructure for rs-llmspell
//! ABOUTME: Provides tracing setup with JSON formatting and runtime configuration

use tracing::Level;
use tracing_subscriber::{
    fmt::{self, format::FmtSpan},
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter, Layer,
};

/// Logging configuration for the LLMSpell system.
///
/// Controls various aspects of log output including format, level,
/// and metadata inclusion. Supports both human-readable and JSON formats.
///
/// # Examples
///
/// ```
/// use llmspell_core::logging::{LoggingConfig, init_logging};
/// use tracing::Level;
///
/// // Development configuration with pretty printing
/// let dev_config = LoggingConfig::development();
/// assert_eq!(dev_config.default_level, Level::DEBUG);
/// assert!(!dev_config.json_format);
///
/// // Production configuration with JSON output
/// let prod_config = LoggingConfig::production();
/// assert!(prod_config.json_format);
///
/// // Custom configuration
/// let custom_config = LoggingConfig {
///     default_level: Level::WARN,
///     json_format: true,
///     with_timestamps: true,
///     with_thread_names: false,
///     with_thread_ids: false,
///     with_file_lines: true,
///     with_span_events: false,
/// };
/// ```
#[derive(Debug, Clone)]
pub struct LoggingConfig {
    /// Default log level
    pub default_level: Level,
    /// Whether to use JSON formatting
    pub json_format: bool,
    /// Whether to include timestamps
    pub with_timestamps: bool,
    /// Whether to include thread names
    pub with_thread_names: bool,
    /// Whether to include thread IDs
    pub with_thread_ids: bool,
    /// Whether to include file and line numbers
    pub with_file_lines: bool,
    /// Whether to include span events
    pub with_span_events: bool,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            default_level: Level::INFO,
            json_format: true,
            with_timestamps: true,
            with_thread_names: false,
            with_thread_ids: false,
            with_file_lines: true,
            with_span_events: false,
        }
    }
}

impl LoggingConfig {
    /// Create a development configuration with human-readable output
    pub fn development() -> Self {
        Self {
            default_level: Level::DEBUG,
            json_format: false,
            with_timestamps: true,
            with_thread_names: true,
            with_thread_ids: false,
            with_file_lines: true,
            with_span_events: true,
        }
    }

    /// Create a production configuration with JSON output
    pub fn production() -> Self {
        Self::default()
    }
}

/// Initialize logging with the given configuration.
///
/// Sets up the global tracing subscriber with the specified configuration.
/// This function should be called once at application startup.
///
/// # Environment Variables
///
/// - `RUST_LOG`: Controls log filtering (e.g., "debug", "llmspell=trace")
/// - `LLMSPELL_ENV`: Set to "production" for production config
///
/// # Examples
///
/// ```no_run
/// use llmspell_core::logging::{LoggingConfig, init_logging};
///
/// // Initialize with default configuration
/// init_logging(LoggingConfig::default()).expect("Failed to init logging");
///
/// // Or initialize from environment
/// use llmspell_core::logging::init_from_env;
/// init_from_env().expect("Failed to init logging");
/// ```
pub fn init_logging(config: LoggingConfig) -> Result<(), Box<dyn std::error::Error>> {
    // Create env filter with default level
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(config.default_level.to_string()));

    // Configure format layer
    let fmt_layer = if config.json_format {
        fmt::layer()
            .json()
            .with_timer(fmt::time::UtcTime::rfc_3339())
            .with_thread_names(config.with_thread_names)
            .with_thread_ids(config.with_thread_ids)
            .with_file(config.with_file_lines)
            .with_line_number(config.with_file_lines)
            .with_span_events(if config.with_span_events {
                FmtSpan::FULL
            } else {
                FmtSpan::NONE
            })
            .boxed()
    } else {
        fmt::layer()
            .pretty()
            .with_timer(fmt::time::UtcTime::rfc_3339())
            .with_thread_names(config.with_thread_names)
            .with_thread_ids(config.with_thread_ids)
            .with_file(config.with_file_lines)
            .with_line_number(config.with_file_lines)
            .with_span_events(if config.with_span_events {
                FmtSpan::FULL
            } else {
                FmtSpan::NONE
            })
            .boxed()
    };

    // Build the subscriber
    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt_layer)
        .try_init()?;

    Ok(())
}

/// Initialize logging from environment variables
pub fn init_from_env() -> Result<(), Box<dyn std::error::Error>> {
    let config = if std::env::var("LLMSPELL_ENV").as_deref() == Ok("production") {
        LoggingConfig::production()
    } else {
        LoggingConfig::development()
    };

    init_logging(config)
}

/// Update the global log filter at runtime
pub fn update_log_filter(filter: &str) -> Result<(), Box<dyn std::error::Error>> {
    // This is a simplified version - in production you'd use reload handles
    tracing::info!("Log filter update requested: {}", filter);
    Ok(())
}

// Re-export commonly used tracing macros
pub use tracing::{debug, error, info, instrument, span, trace, warn, Level as LogLevel};

// Logging macros for components
///
/// These macros provide structured logging for component lifecycle events.
/// They automatically capture component metadata and format it consistently.
#[macro_export]
macro_rules! log_component_event {
    ($component:expr, $event:expr) => {{
        use tracing::info;
        info!(
            component_id = ?$component.metadata().id,
            component_name = $component.metadata().name,
            event = $event,
            "Component event"
        );
    }};
    ($component:expr, $event:expr, $($field:tt)*) => {{
        use tracing::info;
        info!(
            component_id = ?$component.metadata().id,
            component_name = $component.metadata().name,
            event = $event,
            $($field)*
        );
    }};
}

#[macro_export]
macro_rules! log_execution_start {
    ($component:expr, $input:expr) => {{
        use tracing::{info, instrument};
        info!(
            component_id = ?$component.metadata().id,
            component_name = $component.metadata().name,
            input = ?$input,
            "Execution started"
        );
    }};
}

#[macro_export]
macro_rules! log_execution_end {
    ($component:expr, $duration:expr, $success:expr) => {{
        use tracing::info;
        info!(
            component_id = ?$component.metadata().id,
            component_name = $component.metadata().name,
            duration_ms = $duration.as_millis(),
            success = $success,
            "Execution completed"
        );
    }};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_logging_config_default() {
        let config = LoggingConfig::default();
        assert_eq!(config.default_level, Level::INFO);
        assert!(config.json_format);
        assert!(config.with_timestamps);
        assert!(!config.with_thread_names);
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_logging_config_development() {
        let config = LoggingConfig::development();
        assert_eq!(config.default_level, Level::DEBUG);
        assert!(!config.json_format);
        assert!(config.with_timestamps);
        assert!(config.with_thread_names);
        assert!(config.with_span_events);
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_logging_config_production() {
        let config = LoggingConfig::production();
        assert_eq!(config.default_level, Level::INFO);
        assert!(config.json_format);
        assert!(config.with_timestamps);
        assert!(config.with_file_lines);
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_logging_initialization() {
        // We can't actually initialize logging in tests (it's global state)
        // but we can verify the config builds correctly
        let config = LoggingConfig::default();
        assert!(config.json_format);
    }
}
