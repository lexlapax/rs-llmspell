//! Callback-based IO implementation for the kernel
//!
//! This module provides a simpler IO implementation that uses callbacks
//! instead of references to the kernel, avoiding circular dependencies.

use async_trait::async_trait;
use llmspell_core::error::LLMSpellError;
use llmspell_core::io::{IOContext, IOInput, IOPerformanceHints, IOStream, SignalHandler};
use std::sync::Arc;

/// Callback-based IO stream that uses a closure to handle output
pub struct CallbackIOStream<F>
where
    F: Fn(&str) -> Result<(), LLMSpellError> + Send + Sync,
{
    callback: Arc<F>,
}

impl<F> CallbackIOStream<F>
where
    F: Fn(&str) -> Result<(), LLMSpellError> + Send + Sync,
{
    pub fn new(callback: F) -> Self {
        Self {
            callback: Arc::new(callback),
        }
    }
}

impl<F> IOStream for CallbackIOStream<F>
where
    F: Fn(&str) -> Result<(), LLMSpellError> + Send + Sync + 'static,
{
    fn write(&self, data: &str) -> Result<(), LLMSpellError> {
        (self.callback)(data)
    }

    fn write_line(&self, line: &str) -> Result<(), LLMSpellError> {
        let mut output = line.to_string();
        if !output.ends_with('\n') {
            output.push('\n');
        }
        (self.callback)(&output)
    }

    fn flush(&self) -> Result<(), LLMSpellError> {
        // Callback-based stream is always flushed
        Ok(())
    }
}

/// Simple stdin implementation that returns empty for now
pub struct SimpleIOInput;

#[async_trait]
impl IOInput for SimpleIOInput {
    async fn read_line(&self, _prompt: &str) -> Result<String, LLMSpellError> {
        // TODO: Implement proper stdin handling via input_request/reply
        Err(LLMSpellError::Io {
            operation: "read_line".to_string(),
            source: std::io::Error::new(
                std::io::ErrorKind::Unsupported,
                "stdin not yet implemented in kernel",
            ),
        })
    }

    async fn read_password(&self, _prompt: &str) -> Result<String, LLMSpellError> {
        Err(LLMSpellError::Io {
            operation: "read_password".to_string(),
            source: std::io::Error::new(
                std::io::ErrorKind::Unsupported,
                "stdin not yet implemented in kernel",
            ),
        })
    }
}

/// Create a callback-based IO context for kernel execution
pub fn create_callback_io_context<F1, F2>(
    stdout_callback: F1,
    stderr_callback: F2,
    signal_handler: Arc<dyn SignalHandler>,
) -> Arc<IOContext>
where
    F1: Fn(&str) -> Result<(), LLMSpellError> + Send + Sync + 'static,
    F2: Fn(&str) -> Result<(), LLMSpellError> + Send + Sync + 'static,
{
    Arc::new(IOContext::new(
        Arc::new(CallbackIOStream::new(stdout_callback)),
        Arc::new(CallbackIOStream::new(stderr_callback)),
        Arc::new(SimpleIOInput),
        signal_handler,
        IOPerformanceHints {
            batch_size: 1,        // Send immediately for kernel
            flush_interval_ms: 0, // No delay
            async_capable: true,
        },
    ))
}
