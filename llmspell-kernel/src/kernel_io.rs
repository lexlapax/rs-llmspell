//! Kernel-specific IO implementations that route through the messaging protocol
//!
//! This module provides IOContext implementations that route all IO operations
//! through the kernel's messaging channels (IOPub for output, stdin for input).

use async_trait::async_trait;
use llmspell_core::error::LLMSpellError;
use llmspell_core::io::{IOContext, IOInput, IOPerformanceHints, IOStream, SignalHandler};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Weak};
use tokio::sync::{mpsc, Mutex};

use crate::kernel::GenericKernel;
use crate::traits::{Protocol, Transport};

/// Stream type for routing output
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StreamType {
    Stdout,
    Stderr,
}

impl StreamType {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Stdout => "stdout",
            Self::Stderr => "stderr",
        }
    }
}

/// Kernel IO stream that routes output through IOPub channel
pub struct KernelIOStream<T: Transport, P: Protocol> {
    /// Weak reference to the kernel to avoid circular dependencies
    kernel: Weak<Mutex<GenericKernel<T, P>>>,
    /// Stream type (stdout or stderr)
    stream_type: StreamType,
}

impl<T: Transport, P: Protocol> KernelIOStream<T, P> {
    pub fn new(kernel: Weak<Mutex<GenericKernel<T, P>>>, stream_type: StreamType) -> Self {
        Self {
            kernel,
            stream_type,
        }
    }
}

impl<T: Transport + 'static, P: Protocol + 'static> IOStream for KernelIOStream<T, P> {
    fn write(&self, data: &str) -> Result<(), LLMSpellError> {
        // For write without newline, we need to handle buffering
        // For now, just write directly
        self.write_line(data)
    }

    fn write_line(&self, line: &str) -> Result<(), LLMSpellError> {
        if let Some(kernel) = self.kernel.upgrade() {
            // Use tokio's block_in_place to safely run async code
            tokio::task::block_in_place(|| {
                let rt = tokio::runtime::Handle::current();
                rt.block_on(async {
                    let kernel = kernel.lock().await;
                    kernel
                        .publish_stream(self.stream_type.as_str(), line)
                        .await
                        .map_err(|e| LLMSpellError::Io {
                            operation: format!("publish to {}", self.stream_type.as_str()),
                            source: std::io::Error::new(std::io::ErrorKind::Other, e.to_string()),
                        })
                })
            })
        } else {
            // Kernel has been dropped, silently ignore
            Ok(())
        }
    }

    fn flush(&self) -> Result<(), LLMSpellError> {
        // IOPub is always flushed immediately
        Ok(())
    }
}

/// Kernel IO input that handles stdin through input_request/input_reply
pub struct KernelIOInput<T: Transport, P: Protocol> {
    /// Weak reference to the kernel
    kernel: Weak<Mutex<GenericKernel<T, P>>>,
    /// Channel for receiving input replies
    input_receiver: Arc<Mutex<Option<mpsc::Receiver<String>>>>,
}

impl<T: Transport, P: Protocol> KernelIOInput<T, P> {
    pub fn new(kernel: Weak<Mutex<GenericKernel<T, P>>>) -> Self {
        Self {
            kernel,
            input_receiver: Arc::new(Mutex::new(None)),
        }
    }

    /// Set up the input receiver channel
    pub async fn setup_input_channel(&self) -> mpsc::Sender<String> {
        let (tx, rx) = mpsc::channel(1);
        *self.input_receiver.lock().await = Some(rx);
        tx
    }
}

#[async_trait]
impl<T: Transport + 'static, P: Protocol + 'static> IOInput for KernelIOInput<T, P> {
    async fn read_line(&self, prompt: &str) -> Result<String, LLMSpellError> {
        if let Some(kernel) = self.kernel.upgrade() {
            let kernel = kernel.lock().await;

            // Send input_request message
            kernel
                .publish_iopub(
                    "input_request",
                    serde_json::json!({
                        "prompt": prompt,
                        "password": false
                    }),
                )
                .await
                .map_err(|e| LLMSpellError::Io {
                    operation: "send input_request".to_string(),
                    source: std::io::Error::new(std::io::ErrorKind::Other, e.to_string()),
                })?;

            // Drop kernel lock before waiting for input
            drop(kernel);

            // Wait for input_reply
            let mut receiver_guard = self.input_receiver.lock().await;
            if let Some(ref mut receiver) = *receiver_guard {
                receiver.recv().await.ok_or_else(|| LLMSpellError::Io {
                    operation: "receive input_reply".to_string(),
                    source: std::io::Error::new(
                        std::io::ErrorKind::UnexpectedEof,
                        "Input channel closed",
                    ),
                })
            } else {
                Err(LLMSpellError::Io {
                    operation: "read_line".to_string(),
                    source: std::io::Error::new(
                        std::io::ErrorKind::NotConnected,
                        "Input channel not set up",
                    ),
                })
            }
        } else {
            Err(LLMSpellError::Io {
                operation: "read_line".to_string(),
                source: std::io::Error::new(std::io::ErrorKind::NotConnected, "Kernel dropped"),
            })
        }
    }

    async fn read_password(&self, prompt: &str) -> Result<String, LLMSpellError> {
        if let Some(kernel) = self.kernel.upgrade() {
            let kernel = kernel.lock().await;

            // Send input_request message with password flag
            kernel
                .publish_iopub(
                    "input_request",
                    serde_json::json!({
                        "prompt": prompt,
                        "password": true
                    }),
                )
                .await
                .map_err(|e| LLMSpellError::Io {
                    operation: "send input_request".to_string(),
                    source: std::io::Error::new(std::io::ErrorKind::Other, e.to_string()),
                })?;

            // Drop kernel lock before waiting for input
            drop(kernel);

            // Wait for input_reply (same as read_line but client should hide input)
            let mut receiver_guard = self.input_receiver.lock().await;
            if let Some(ref mut receiver) = *receiver_guard {
                receiver.recv().await.ok_or_else(|| LLMSpellError::Io {
                    operation: "receive input_reply".to_string(),
                    source: std::io::Error::new(
                        std::io::ErrorKind::UnexpectedEof,
                        "Input channel closed",
                    ),
                })
            } else {
                Err(LLMSpellError::Io {
                    operation: "read_password".to_string(),
                    source: std::io::Error::new(
                        std::io::ErrorKind::NotConnected,
                        "Input channel not set up",
                    ),
                })
            }
        } else {
            Err(LLMSpellError::Io {
                operation: "read_password".to_string(),
                source: std::io::Error::new(std::io::ErrorKind::NotConnected, "Kernel dropped"),
            })
        }
    }
}

/// Kernel signal handler for interrupt propagation
pub struct KernelSignalHandler {
    /// Shared interrupt flag
    interrupt_flag: Arc<AtomicBool>,
}

impl KernelSignalHandler {
    pub fn new() -> Self {
        Self {
            interrupt_flag: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn interrupt(&self) {
        self.interrupt_flag.store(true, Ordering::Relaxed);
    }

    pub fn reset(&self) {
        self.interrupt_flag.store(false, Ordering::Relaxed);
    }

    pub fn clone_flag(&self) -> Arc<AtomicBool> {
        self.interrupt_flag.clone()
    }
}

impl Default for KernelSignalHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl SignalHandler for KernelSignalHandler {
    fn handle_interrupt(&self) -> bool {
        self.interrupt_flag.store(true, Ordering::Relaxed);
        true
    }

    fn is_interrupted(&self) -> bool {
        self.interrupt_flag.load(Ordering::Relaxed)
    }
}

/// Create a kernel IO context for script execution
pub fn create_kernel_io_context<T: Transport + 'static, P: Protocol + 'static>(
    kernel: Weak<Mutex<GenericKernel<T, P>>>,
    signal_handler: Arc<KernelSignalHandler>,
) -> Arc<IOContext> {
    Arc::new(IOContext::new(
        Arc::new(KernelIOStream::new(kernel.clone(), StreamType::Stdout)),
        Arc::new(KernelIOStream::new(kernel.clone(), StreamType::Stderr)),
        Arc::new(KernelIOInput::new(kernel)),
        signal_handler,
        IOPerformanceHints {
            batch_size: 10,         // Batch up to 10 lines
            flush_interval_ms: 100, // Flush every 100ms
            async_capable: true,    // Kernel is async-capable
        },
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stream_type() {
        assert_eq!(StreamType::Stdout.as_str(), "stdout");
        assert_eq!(StreamType::Stderr.as_str(), "stderr");
    }

    #[test]
    fn test_signal_handler() {
        let handler = KernelSignalHandler::new();
        assert!(!handler.is_interrupted());

        handler.handle_interrupt();
        assert!(handler.is_interrupted());

        handler.reset();
        assert!(!handler.is_interrupted());
    }
}
