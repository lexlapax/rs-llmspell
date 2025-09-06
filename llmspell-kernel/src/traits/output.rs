//! Output capture trait for script runtime integration
//!
//! This trait allows script runtimes to capture output (stdout, stderr, results, errors)
//! and route them through the protocol layer for proper message handling.

use super::protocol::ExecutionError;
use serde_json::Value;

/// Trait for capturing output during script execution
///
/// Implementations of this trait can buffer, transform, or immediately
/// forward output to the appropriate protocol handlers.
pub trait OutputCapture: Send {
    /// Capture stdout output
    fn capture_stdout(&mut self, text: &str);

    /// Capture stderr output
    fn capture_stderr(&mut self, text: &str);

    /// Capture an execution result value
    fn capture_result(&mut self, value: Value);

    /// Capture an execution error
    fn capture_error(&mut self, error: ExecutionError);

    /// Flush any buffered output
    fn flush(&mut self) {
        // Default: no-op, implementations can override if they buffer
    }
}

/// Simple output capture that collects output in memory
#[derive(Debug, Default)]
pub struct MemoryOutputCapture {
    pub stdout: Vec<String>,
    pub stderr: Vec<String>,
    pub results: Vec<Value>,
    pub errors: Vec<ExecutionError>,
}

impl OutputCapture for MemoryOutputCapture {
    fn capture_stdout(&mut self, text: &str) {
        self.stdout.push(text.to_string());
    }

    fn capture_stderr(&mut self, text: &str) {
        self.stderr.push(text.to_string());
    }

    fn capture_result(&mut self, value: Value) {
        self.results.push(value);
    }

    fn capture_error(&mut self, error: ExecutionError) {
        self.errors.push(error);
    }
}

/// Protocol-aware output capture that creates protocol messages
pub struct ProtocolOutputCapture<P: super::Protocol> {
    protocol: P,
    context: P::OutputContext,
    messages: Vec<(String, P::Message)>,
}

impl<P: super::Protocol> ProtocolOutputCapture<P> {
    /// Create a new protocol output capture
    pub fn new(protocol: P) -> Self {
        let context = protocol.create_output_context();
        Self {
            protocol,
            context,
            messages: Vec::new(),
        }
    }

    /// Get the accumulated messages
    pub fn into_messages(mut self) -> Vec<(String, P::Message)> {
        // Flush any remaining buffered output
        let flushed = self.protocol.flush_output(self.context);
        self.messages.extend(flushed);
        self.messages
    }
}

impl<P: super::Protocol> OutputCapture for ProtocolOutputCapture<P> {
    fn capture_stdout(&mut self, text: &str) {
        self.protocol.handle_output(
            &mut self.context,
            super::protocol::OutputChunk::Stdout(text.to_string()),
        );
    }

    fn capture_stderr(&mut self, text: &str) {
        self.protocol.handle_output(
            &mut self.context,
            super::protocol::OutputChunk::Stderr(text.to_string()),
        );
    }

    fn capture_result(&mut self, value: Value) {
        self.protocol.handle_output(
            &mut self.context,
            super::protocol::OutputChunk::Result(value),
        );
    }

    fn capture_error(&mut self, error: ExecutionError) {
        self.protocol.handle_output(
            &mut self.context,
            super::protocol::OutputChunk::Error(error),
        );
    }

    fn flush(&mut self) {
        // Flush buffered output to messages
        let messages = self.protocol.flush_output(std::mem::replace(
            &mut self.context,
            self.protocol.create_output_context(),
        ));
        self.messages.extend(messages);
    }
}
