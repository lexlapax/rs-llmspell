//! IO abstraction layer for protocol-agnostic input/output handling
//!
//! This module provides a unified IO interface that allows all components
//! to perform IO operations without knowing the underlying transport mechanism.
//! This enables the same code to work with:
//! - Direct stdout/stderr/stdin (CLI mode)
//! - Jupyter protocol IOPub/stdin channels (kernel mode)
//! - LSP/DAP protocol messages (future)
//! - Mock IO for testing

use crate::error::LLMSpellError;
use async_trait::async_trait;
use std::io::{self, Write as StdWrite};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

/// Performance hints for IO operations
#[derive(Debug, Clone)]
pub struct IOPerformanceHints {
    /// Buffer this many lines before flushing
    pub batch_size: usize,
    /// Force flush after this many milliseconds
    pub flush_interval_ms: u64,
    /// Whether this IO context can handle async operations efficiently
    pub async_capable: bool,
}

impl Default for IOPerformanceHints {
    fn default() -> Self {
        Self {
            batch_size: 1, // Unbuffered by default for interactive use
            flush_interval_ms: 100,
            async_capable: false,
        }
    }
}

/// Trait for output streams (stdout/stderr)
pub trait IOStream: Send + Sync {
    /// Write raw data to the stream
    fn write(&self, data: &str) -> Result<(), LLMSpellError>;

    /// Write a line to the stream (adds newline if not present)
    fn write_line(&self, line: &str) -> Result<(), LLMSpellError>;

    /// Flush any buffered data
    fn flush(&self) -> Result<(), LLMSpellError>;
}

/// Trait for input operations (stdin)
#[async_trait]
pub trait IOInput: Send + Sync {
    /// Read a line from input with optional prompt
    async fn read_line(&self, prompt: &str) -> Result<String, LLMSpellError>;

    /// Read a password from input (hidden characters)
    async fn read_password(&self, prompt: &str) -> Result<String, LLMSpellError>;
}

/// Trait for signal handling
pub trait SignalHandler: Send + Sync {
    /// Handle an interrupt signal (returns true if handled)
    fn handle_interrupt(&self) -> bool;

    /// Check if an interrupt has been requested
    fn is_interrupted(&self) -> bool;
}

/// Main IO context that bundles all IO operations
pub struct IOContext {
    pub stdout: Arc<dyn IOStream>,
    pub stderr: Arc<dyn IOStream>,
    pub stdin: Arc<dyn IOInput>,
    pub signal_handler: Arc<dyn SignalHandler>,
    pub performance_hints: IOPerformanceHints,
}

impl IOContext {
    /// Create a new IOContext with the given components
    pub fn new(
        stdout: Arc<dyn IOStream>,
        stderr: Arc<dyn IOStream>,
        stdin: Arc<dyn IOInput>,
        signal_handler: Arc<dyn SignalHandler>,
        performance_hints: IOPerformanceHints,
    ) -> Self {
        Self {
            stdout,
            stderr,
            stdin,
            signal_handler,
            performance_hints,
        }
    }

    /// Create a standard IOContext for CLI usage
    pub fn stdio() -> Self {
        Self::new(
            Arc::new(StdoutStream::new()),
            Arc::new(StderrStream::new()),
            Arc::new(StdinInput::new()),
            Arc::new(NoOpSignalHandler),
            IOPerformanceHints::default(),
        )
    }

    /// Create a null IOContext for testing (discards all output)
    pub fn null() -> Self {
        Self::new(
            Arc::new(NullStream),
            Arc::new(NullStream),
            Arc::new(NullInput),
            Arc::new(NoOpSignalHandler),
            IOPerformanceHints::default(),
        )
    }
}

// === Default Implementations ===

/// Standard output stream implementation
pub struct StdoutStream {
    handle: Mutex<io::Stdout>,
}

impl StdoutStream {
    pub fn new() -> Self {
        Self {
            handle: Mutex::new(io::stdout()),
        }
    }
}

impl Default for StdoutStream {
    fn default() -> Self {
        Self::new()
    }
}

impl IOStream for StdoutStream {
    fn write(&self, data: &str) -> Result<(), LLMSpellError> {
        let mut handle = self.handle.lock().unwrap();
        handle
            .write_all(data.as_bytes())
            .map_err(|e| LLMSpellError::Io {
                operation: "write".to_string(),
                source: e,
            })
    }

    fn write_line(&self, line: &str) -> Result<(), LLMSpellError> {
        let mut handle = self.handle.lock().unwrap();
        writeln!(handle, "{}", line).map_err(|e| LLMSpellError::Io {
            operation: "write_line".to_string(),
            source: e,
        })
    }

    fn flush(&self) -> Result<(), LLMSpellError> {
        let mut handle = self.handle.lock().unwrap();
        handle.flush().map_err(|e| LLMSpellError::Io {
            operation: "flush".to_string(),
            source: e,
        })
    }
}

/// Standard error stream implementation
pub struct StderrStream {
    handle: Mutex<io::Stderr>,
}

impl StderrStream {
    pub fn new() -> Self {
        Self {
            handle: Mutex::new(io::stderr()),
        }
    }
}

impl Default for StderrStream {
    fn default() -> Self {
        Self::new()
    }
}

impl IOStream for StderrStream {
    fn write(&self, data: &str) -> Result<(), LLMSpellError> {
        let mut handle = self.handle.lock().unwrap();
        handle
            .write_all(data.as_bytes())
            .map_err(|e| LLMSpellError::Io {
                operation: "write".to_string(),
                source: e,
            })
    }

    fn write_line(&self, line: &str) -> Result<(), LLMSpellError> {
        let mut handle = self.handle.lock().unwrap();
        writeln!(handle, "{}", line).map_err(|e| LLMSpellError::Io {
            operation: "write_line".to_string(),
            source: e,
        })
    }

    fn flush(&self) -> Result<(), LLMSpellError> {
        let mut handle = self.handle.lock().unwrap();
        handle.flush().map_err(|e| LLMSpellError::Io {
            operation: "flush".to_string(),
            source: e,
        })
    }
}

/// Standard input implementation
pub struct StdinInput;

impl StdinInput {
    pub fn new() -> Self {
        Self
    }
}

impl Default for StdinInput {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl IOInput for StdinInput {
    async fn read_line(&self, prompt: &str) -> Result<String, LLMSpellError> {
        // Print prompt if provided
        if !prompt.is_empty() {
            print!("{}", prompt);
            io::stdout().flush().map_err(|e| LLMSpellError::Io {
                operation: "flush prompt".to_string(),
                source: e,
            })?;
        }

        // Read line from stdin
        let mut buffer = String::new();
        io::stdin()
            .read_line(&mut buffer)
            .map_err(|e| LLMSpellError::Io {
                operation: "read_line".to_string(),
                source: e,
            })?;

        // Remove trailing newline
        if buffer.ends_with('\n') {
            buffer.pop();
            if buffer.ends_with('\r') {
                buffer.pop();
            }
        }

        Ok(buffer)
    }

    async fn read_password(&self, prompt: &str) -> Result<String, LLMSpellError> {
        // For now, just read normally (TODO: use rpassword crate)
        self.read_line(prompt).await
    }
}

/// Null stream that discards all output
pub struct NullStream;

impl IOStream for NullStream {
    fn write(&self, _data: &str) -> Result<(), LLMSpellError> {
        Ok(())
    }

    fn write_line(&self, _line: &str) -> Result<(), LLMSpellError> {
        Ok(())
    }

    fn flush(&self) -> Result<(), LLMSpellError> {
        Ok(())
    }
}

/// Null input that returns empty strings
pub struct NullInput;

#[async_trait]
impl IOInput for NullInput {
    async fn read_line(&self, _prompt: &str) -> Result<String, LLMSpellError> {
        Ok(String::new())
    }

    async fn read_password(&self, _prompt: &str) -> Result<String, LLMSpellError> {
        Ok(String::new())
    }
}

/// No-op signal handler
pub struct NoOpSignalHandler;

impl SignalHandler for NoOpSignalHandler {
    fn handle_interrupt(&self) -> bool {
        false // Not handled
    }

    fn is_interrupted(&self) -> bool {
        false
    }
}

/// Buffered output stream for performance
pub struct BufferedStream {
    inner: Arc<dyn IOStream>,
    buffer: Mutex<Vec<String>>,
    batch_size: usize,
}

impl BufferedStream {
    pub fn new(inner: Arc<dyn IOStream>, batch_size: usize) -> Self {
        Self {
            inner,
            buffer: Mutex::new(Vec::with_capacity(batch_size)),
            batch_size,
        }
    }

    fn flush_if_needed(&self) -> Result<(), LLMSpellError> {
        let mut buffer = self.buffer.lock().unwrap();
        if buffer.len() >= self.batch_size {
            for line in buffer.drain(..) {
                self.inner.write_line(&line)?;
            }
            self.inner.flush()?;
        }
        Ok(())
    }
}

impl IOStream for BufferedStream {
    fn write(&self, data: &str) -> Result<(), LLMSpellError> {
        self.inner.write(data)?;
        Ok(())
    }

    fn write_line(&self, line: &str) -> Result<(), LLMSpellError> {
        {
            let mut buffer = self.buffer.lock().unwrap();
            buffer.push(line.to_string());
        }
        self.flush_if_needed()
    }

    fn flush(&self) -> Result<(), LLMSpellError> {
        let mut buffer = self.buffer.lock().unwrap();
        for line in buffer.drain(..) {
            self.inner.write_line(&line)?;
        }
        self.inner.flush()
    }
}

/// Mock IO stream for testing
pub struct MockStream {
    pub lines: Arc<Mutex<Vec<String>>>,
}

impl MockStream {
    pub fn new() -> Self {
        Self {
            lines: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn get_lines(&self) -> Vec<String> {
        self.lines.lock().unwrap().clone()
    }
}

impl Default for MockStream {
    fn default() -> Self {
        Self::new()
    }
}

impl IOStream for MockStream {
    fn write(&self, data: &str) -> Result<(), LLMSpellError> {
        let mut lines = self.lines.lock().unwrap();
        if let Some(last) = lines.last_mut() {
            last.push_str(data);
        } else {
            lines.push(data.to_string());
        }
        Ok(())
    }

    fn write_line(&self, line: &str) -> Result<(), LLMSpellError> {
        let mut lines = self.lines.lock().unwrap();
        lines.push(line.to_string());
        Ok(())
    }

    fn flush(&self) -> Result<(), LLMSpellError> {
        Ok(())
    }
}

/// Interruptible signal handler using atomic flag
pub struct InterruptibleSignalHandler {
    interrupted: Arc<AtomicBool>,
}

impl InterruptibleSignalHandler {
    pub fn new() -> Self {
        Self {
            interrupted: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn interrupt(&self) {
        self.interrupted.store(true, Ordering::Relaxed);
    }

    pub fn reset(&self) {
        self.interrupted.store(false, Ordering::Relaxed);
    }
}

impl Default for InterruptibleSignalHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl SignalHandler for InterruptibleSignalHandler {
    fn handle_interrupt(&self) -> bool {
        self.interrupted.store(true, Ordering::Relaxed);
        true
    }

    fn is_interrupted(&self) -> bool {
        self.interrupted.load(Ordering::Relaxed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_stream() {
        let stream = MockStream::new();
        stream.write_line("line 1").unwrap();
        stream.write_line("line 2").unwrap();
        stream.write("partial").unwrap();
        stream.write(" data").unwrap();

        let lines = stream.get_lines();
        // Note: partial writes are appended to the last line if it exists
        assert_eq!(lines, vec!["line 1", "line 2partial data"]);
    }

    #[test]
    fn test_null_stream() {
        let stream = NullStream;
        // Should not error
        stream.write_line("test").unwrap();
        stream.flush().unwrap();
    }

    #[test]
    fn test_signal_handler() {
        let handler = InterruptibleSignalHandler::new();
        assert!(!handler.is_interrupted());

        handler.handle_interrupt();
        assert!(handler.is_interrupted());

        handler.reset();
        assert!(!handler.is_interrupted());
    }

    #[tokio::test]
    async fn test_null_input() {
        let input = NullInput;
        let result = input.read_line("prompt").await.unwrap();
        assert_eq!(result, "");
    }
}
