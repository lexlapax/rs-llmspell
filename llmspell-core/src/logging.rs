//! ABOUTME: Logging infrastructure and structured logging utilities
//! ABOUTME: Provides tracing initialization and logging macros

/// Initialize structured logging
pub fn init_logging() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    Ok(())
}