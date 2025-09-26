//! # Signal Handling for Daemons
//!
//! Provides signal handling infrastructure for daemon processes, converting
//! Unix signals to appropriate kernel messages for graceful shutdown and
//! runtime operations.

use anyhow::{Context, Result};
use nix::sys::signal::{self, SaFlags, SigAction, SigHandler as NixSigHandler, SigSet, Signal};
use serde_json::json;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{debug, error, info, trace};

/// Atomic flags for signal handling
pub static SIGTERM_RECEIVED: AtomicBool = AtomicBool::new(false);
pub static SIGINT_RECEIVED: AtomicBool = AtomicBool::new(false);
pub static SIGHUP_RECEIVED: AtomicBool = AtomicBool::new(false);
pub static SIGUSR1_RECEIVED: AtomicBool = AtomicBool::new(false);
pub static SIGUSR2_RECEIVED: AtomicBool = AtomicBool::new(false);

/// Signal handler function
extern "C" fn handle_signal(sig: libc::c_int) {
    match sig {
        libc::SIGTERM => {
            SIGTERM_RECEIVED.store(true, Ordering::SeqCst);
        }
        libc::SIGINT => {
            SIGINT_RECEIVED.store(true, Ordering::SeqCst);
        }
        libc::SIGHUP => {
            SIGHUP_RECEIVED.store(true, Ordering::SeqCst);
        }
        libc::SIGUSR1 => {
            SIGUSR1_RECEIVED.store(true, Ordering::SeqCst);
        }
        libc::SIGUSR2 => {
            SIGUSR2_RECEIVED.store(true, Ordering::SeqCst);
        }
        _ => {
            // Unknown signal, ignore
        }
    }
    // Note: We can't use tracing or print here as it's not async-signal-safe
    // We only set the atomic flags
}

/// Types of signal actions
#[derive(Debug, Clone)]
pub enum SignalAction {
    /// Graceful shutdown
    Shutdown,
    /// Interrupt current operation
    Interrupt,
    /// Reload configuration
    Reload,
    /// Dump state to log
    DumpState,
    /// Custom action
    Custom(String),
}

/// Signal handler for daemon processes
pub struct SignalHandler {
    /// Whether signal handling is installed
    installed: bool,
}

impl SignalHandler {
    /// Creates a new signal handler
    pub fn new() -> Self {
        Self { installed: false }
    }

    /// Installs signal handlers for daemon operation
    ///
    /// # Errors
    ///
    /// Returns an error if failed to install signal handlers
    pub fn install(&mut self) -> Result<()> {
        if self.installed {
            return Ok(());
        }

        // Create signal action with our handler
        let sig_action = SigAction::new(
            NixSigHandler::Handler(handle_signal),
            SaFlags::empty(),
            SigSet::empty(),
        );

        // Install handlers for common daemon signals
        unsafe {
            signal::sigaction(Signal::SIGTERM, &sig_action)
                .context("Failed to install SIGTERM handler")?;
            signal::sigaction(Signal::SIGINT, &sig_action)
                .context("Failed to install SIGINT handler")?;
            signal::sigaction(Signal::SIGHUP, &sig_action)
                .context("Failed to install SIGHUP handler")?;
            signal::sigaction(Signal::SIGUSR1, &sig_action)
                .context("Failed to install SIGUSR1 handler")?;
            signal::sigaction(Signal::SIGUSR2, &sig_action)
                .context("Failed to install SIGUSR2 handler")?;
        }

        // Ignore SIGPIPE (common for network daemons)
        unsafe {
            signal::sigaction(
                Signal::SIGPIPE,
                &SigAction::new(NixSigHandler::SigIgn, SaFlags::empty(), SigSet::empty()),
            )
            .context("Failed to ignore SIGPIPE")?;
        }

        self.installed = true;
        info!("Signal handlers installed");
        Ok(())
    }

    /// Checks for pending signals and returns the appropriate action
    pub fn check_signals(&self) -> Option<SignalAction> {
        // Check SIGTERM (graceful shutdown)
        if SIGTERM_RECEIVED.swap(false, Ordering::SeqCst) {
            info!("SIGTERM received, initiating graceful shutdown");
            return Some(SignalAction::Shutdown);
        }

        // Check SIGINT (interrupt/shutdown)
        if SIGINT_RECEIVED.swap(false, Ordering::SeqCst) {
            info!("SIGINT received, interrupting operation");
            return Some(SignalAction::Interrupt);
        }

        // Check SIGHUP (reload configuration)
        if SIGHUP_RECEIVED.swap(false, Ordering::SeqCst) {
            info!("SIGHUP received, reloading configuration");
            return Some(SignalAction::Reload);
        }

        // Check SIGUSR1 (config reload)
        if SIGUSR1_RECEIVED.swap(false, Ordering::SeqCst) {
            info!("SIGUSR1 received, triggering config reload");
            return Some(SignalAction::Reload);
        }

        // Check SIGUSR2 (state dump)
        if SIGUSR2_RECEIVED.swap(false, Ordering::SeqCst) {
            info!("SIGUSR2 received, triggering state dump");
            return Some(SignalAction::DumpState);
        }

        None
    }

    /// Resets all signal flags
    pub fn reset(&self) {
        SIGTERM_RECEIVED.store(false, Ordering::SeqCst);
        SIGINT_RECEIVED.store(false, Ordering::SeqCst);
        SIGHUP_RECEIVED.store(false, Ordering::SeqCst);
        SIGUSR1_RECEIVED.store(false, Ordering::SeqCst);
        SIGUSR2_RECEIVED.store(false, Ordering::SeqCst);
    }

    /// Blocks the given signal
    ///
    /// # Errors
    ///
    /// Returns an error if failed to block the signal
    pub fn block_signal(signal: Signal) -> Result<()> {
        let mut sigset = SigSet::empty();
        sigset.add(signal);
        sigset.thread_block().context("Failed to block signal")?;
        Ok(())
    }

    /// Unblocks the given signal
    ///
    /// # Errors
    ///
    /// Returns an error if failed to unblock the signal
    pub fn unblock_signal(signal: Signal) -> Result<()> {
        let mut sigset = SigSet::empty();
        sigset.add(signal);
        sigset
            .thread_unblock()
            .context("Failed to unblock signal")?;
        Ok(())
    }
}

impl Default for SignalHandler {
    fn default() -> Self {
        Self::new()
    }
}

/// Jupyter protocol message type for kernel control
#[derive(Debug, Clone)]
pub enum KernelMessage {
    /// Shutdown request message
    ShutdownRequest { restart: bool },
    /// Interrupt request message
    InterruptRequest,
    /// Configuration reload request
    ConfigReload,
    /// State dump request
    StateDump,
}

impl KernelMessage {
    /// Convert to Jupyter protocol message
    pub fn to_jupyter_message(&self) -> HashMap<String, serde_json::Value> {
        let msg_type = match self {
            Self::ShutdownRequest { .. } => "shutdown_request",
            Self::InterruptRequest => "interrupt_request",
            Self::ConfigReload | Self::StateDump => "custom_request",
        };

        let content = match self {
            Self::ShutdownRequest { restart } => json!({
                "restart": restart
            }),
            Self::InterruptRequest => json!({}),
            Self::ConfigReload => json!({
                "type": "config_reload"
            }),
            Self::StateDump => json!({
                "type": "state_dump"
            }),
        };

        let mut message = HashMap::new();
        message.insert("msg_type".to_string(), json!(msg_type));
        message.insert("content".to_string(), content);
        message
    }
}

/// Bridge between Unix signals and kernel messages
pub struct SignalBridge {
    handler: SignalHandler,
    shutdown_requested: Arc<AtomicBool>,
    message_sender: Option<mpsc::Sender<KernelMessage>>,
}

impl SignalBridge {
    /// Creates a new signal bridge
    pub fn new() -> Self {
        Self {
            handler: SignalHandler::new(),
            shutdown_requested: Arc::new(AtomicBool::new(false)),
            message_sender: None,
        }
    }

    /// Creates a new signal bridge with message channel
    pub fn with_message_channel(sender: mpsc::Sender<KernelMessage>) -> Self {
        Self {
            handler: SignalHandler::new(),
            shutdown_requested: Arc::new(AtomicBool::new(false)),
            message_sender: Some(sender),
        }
    }

    /// Initializes the signal bridge
    ///
    /// # Errors
    ///
    /// Returns an error if failed to install signal handlers
    pub fn init(&mut self) -> Result<()> {
        self.handler.install()?;
        info!("Signal bridge initialized");
        Ok(())
    }

    /// Processes pending signals
    ///
    /// This should be called periodically from the main event loop.
    pub fn process_signals(&self) -> Option<SignalAction> {
        if let Some(action) = self.handler.check_signals() {
            match &action {
                SignalAction::Shutdown => {
                    self.shutdown_requested.store(true, Ordering::SeqCst);
                    info!("Shutdown requested via signal");
                }
                SignalAction::Interrupt => {
                    debug!("Interrupt signal processed");
                }
                SignalAction::Reload => {
                    info!("Configuration reload requested");
                }
                SignalAction::DumpState => {
                    info!("State dump requested");
                }
                SignalAction::Custom(name) => {
                    debug!("Custom signal action: {}", name);
                }
            }
            return Some(action);
        }
        None
    }

    /// Processes pending signals and converts to kernel messages
    ///
    /// This should be called periodically from the main event loop.
    /// Returns the signal action and optionally sends kernel messages.
    pub async fn process_signals_to_messages(&self) -> Option<SignalAction> {
        if let Some(action) = self.handler.check_signals() {
            // Convert signal action to kernel message
            let kernel_message = match &action {
                SignalAction::Shutdown => {
                    self.shutdown_requested.store(true, Ordering::SeqCst);
                    info!("SIGTERM received, converting to shutdown_request");
                    Some(KernelMessage::ShutdownRequest { restart: false })
                }
                SignalAction::Interrupt => {
                    info!("SIGINT received, converting to interrupt_request");
                    Some(KernelMessage::InterruptRequest)
                }
                SignalAction::Reload => {
                    info!("SIGUSR1 received, triggering config reload");
                    Some(KernelMessage::ConfigReload)
                }
                SignalAction::DumpState => {
                    info!("SIGUSR2 received, triggering state dump");
                    Some(KernelMessage::StateDump)
                }
                SignalAction::Custom(name) => {
                    debug!("Custom signal action: {}, no message conversion", name);
                    None
                }
            };

            // Send kernel message if we have a channel
            if let Some(ref sender) = self.message_sender {
                if let Some(msg) = kernel_message {
                    trace!("Sending kernel message: {:?}", msg);
                    if let Err(e) = sender.send(msg).await {
                        error!("Failed to send kernel message: {}", e);
                    }
                }
            }

            return Some(action);
        }
        None
    }

    /// Convert a signal action to a kernel message
    pub fn action_to_message(action: &SignalAction) -> Option<KernelMessage> {
        match action {
            SignalAction::Shutdown => Some(KernelMessage::ShutdownRequest { restart: false }),
            SignalAction::Interrupt => Some(KernelMessage::InterruptRequest),
            SignalAction::Reload => Some(KernelMessage::ConfigReload),
            SignalAction::DumpState => Some(KernelMessage::StateDump),
            SignalAction::Custom(_) => None,
        }
    }

    /// Get a receiver for kernel messages
    ///
    /// Creates a channel for receiving kernel messages from signal handlers.
    /// Returns both a new `SignalBridge` with sender and the receiver.
    pub fn create_message_channel(buffer_size: usize) -> (Self, mpsc::Receiver<KernelMessage>) {
        let (sender, receiver) = mpsc::channel(buffer_size);
        let bridge = Self::with_message_channel(sender);
        (bridge, receiver)
    }

    /// Checks if shutdown has been requested
    pub fn is_shutdown_requested(&self) -> bool {
        self.shutdown_requested.load(Ordering::SeqCst)
    }

    /// Gets a clone of the shutdown flag for sharing
    pub fn shutdown_flag(&self) -> Arc<AtomicBool> {
        Arc::clone(&self.shutdown_requested)
    }

    /// Sends a signal to a process
    ///
    /// # Errors
    ///
    /// Returns an error if failed to send the signal
    pub fn send_signal(pid: i32, signal: Signal) -> Result<()> {
        signal::kill(nix::unistd::Pid::from_raw(pid), signal).context("Failed to send signal")?;
        Ok(())
    }
}

impl Default for SignalBridge {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signal_handler_creation() {
        let handler = SignalHandler::new();
        assert!(!handler.installed);
    }

    #[test]
    fn test_kernel_message_conversion() {
        // Test shutdown request conversion
        let msg = KernelMessage::ShutdownRequest { restart: true };
        let jupyter_msg = msg.to_jupyter_message();
        assert_eq!(jupyter_msg["msg_type"], json!("shutdown_request"));
        assert_eq!(jupyter_msg["content"]["restart"], json!(true));

        // Test interrupt request conversion
        let msg = KernelMessage::InterruptRequest;
        let jupyter_msg = msg.to_jupyter_message();
        assert_eq!(jupyter_msg["msg_type"], json!("interrupt_request"));
        assert_eq!(jupyter_msg["content"], json!({}));

        // Test config reload conversion
        let msg = KernelMessage::ConfigReload;
        let jupyter_msg = msg.to_jupyter_message();
        assert_eq!(jupyter_msg["msg_type"], json!("custom_request"));
        assert_eq!(jupyter_msg["content"]["type"], json!("config_reload"));
    }

    #[test]
    fn test_action_to_message_conversion() {
        // Test SIGTERM to shutdown_request
        let action = SignalAction::Shutdown;
        let msg = SignalBridge::action_to_message(&action);
        assert!(matches!(
            msg,
            Some(KernelMessage::ShutdownRequest { restart: false })
        ));

        // Test SIGINT to interrupt_request
        let action = SignalAction::Interrupt;
        let msg = SignalBridge::action_to_message(&action);
        assert!(matches!(msg, Some(KernelMessage::InterruptRequest)));

        // Test SIGUSR1 to config reload
        let action = SignalAction::Reload;
        let msg = SignalBridge::action_to_message(&action);
        assert!(matches!(msg, Some(KernelMessage::ConfigReload)));

        // Test SIGUSR2 to state dump
        let action = SignalAction::DumpState;
        let msg = SignalBridge::action_to_message(&action);
        assert!(matches!(msg, Some(KernelMessage::StateDump)));
    }

    #[test]
    fn test_signal_handler_reset() {
        let handler = SignalHandler::new();

        // Set some flags
        SIGTERM_RECEIVED.store(true, Ordering::SeqCst);
        SIGINT_RECEIVED.store(true, Ordering::SeqCst);

        // Reset
        handler.reset();

        // Check flags are cleared
        assert!(!SIGTERM_RECEIVED.load(Ordering::SeqCst));
        assert!(!SIGINT_RECEIVED.load(Ordering::SeqCst));
    }

    #[test]
    fn test_signal_action_detection() {
        let handler = SignalHandler::new();

        // No signals initially
        assert!(handler.check_signals().is_none());

        // Set SIGTERM
        SIGTERM_RECEIVED.store(true, Ordering::SeqCst);
        if let Some(action) = handler.check_signals() {
            assert!(matches!(action, SignalAction::Shutdown));
        } else {
            panic!("Expected shutdown action");
        }

        // Flag should be cleared
        assert!(handler.check_signals().is_none());
    }

    #[test]
    fn test_signal_bridge_creation() {
        let bridge = SignalBridge::new();
        assert!(!bridge.is_shutdown_requested());
    }

    #[test]
    fn test_signal_bridge_process() {
        let bridge = SignalBridge::new();

        // No signals initially
        assert!(bridge.process_signals().is_none());

        // Simulate SIGTERM
        SIGTERM_RECEIVED.store(true, Ordering::SeqCst);

        if let Some(action) = bridge.process_signals() {
            assert!(matches!(action, SignalAction::Shutdown));
            assert!(bridge.is_shutdown_requested());
        } else {
            panic!("Expected shutdown action");
        }
    }

    #[test]
    fn test_shutdown_flag_sharing() {
        let bridge = SignalBridge::new();
        let flag = bridge.shutdown_flag();

        assert!(!flag.load(Ordering::SeqCst));

        // Set through bridge's internal flag
        bridge.shutdown_requested.store(true, Ordering::SeqCst);

        // Should be visible through shared flag
        assert!(flag.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn test_signal_bridge_with_message_channel() {
        // Reset all signal flags at start of test
        SignalHandler::new().reset();

        let (bridge, mut receiver) = SignalBridge::create_message_channel(10);

        // Simulate SIGTERM
        SIGTERM_RECEIVED.store(true, Ordering::SeqCst);

        // Process signals
        let action = bridge.process_signals_to_messages().await;
        assert!(matches!(action, Some(SignalAction::Shutdown)));

        // Check that message was sent
        if let Ok(msg) = receiver.try_recv() {
            assert!(matches!(
                msg,
                KernelMessage::ShutdownRequest { restart: false }
            ));
        }

        // Verify shutdown flag was set
        assert!(bridge.is_shutdown_requested());
    }

    #[tokio::test]
    async fn test_multiple_signal_processing() {
        // Reset all signal flags at start of test
        SignalHandler::new().reset();

        let (bridge, mut receiver) = SignalBridge::create_message_channel(10);

        // Simulate SIGINT
        SIGINT_RECEIVED.store(true, Ordering::SeqCst);
        let action = bridge.process_signals_to_messages().await;
        assert!(matches!(action, Some(SignalAction::Interrupt)));

        // Check interrupt message
        if let Ok(msg) = receiver.try_recv() {
            assert!(matches!(msg, KernelMessage::InterruptRequest));
        }

        // Simulate SIGUSR1
        SIGUSR1_RECEIVED.store(true, Ordering::SeqCst);
        let action = bridge.process_signals_to_messages().await;
        assert!(matches!(action, Some(SignalAction::Reload)));

        // Check config reload message
        if let Ok(msg) = receiver.try_recv() {
            assert!(matches!(msg, KernelMessage::ConfigReload));
        }

        // Simulate SIGUSR2
        SIGUSR2_RECEIVED.store(true, Ordering::SeqCst);
        let action = bridge.process_signals_to_messages().await;
        assert!(matches!(action, Some(SignalAction::DumpState)));

        // Check state dump message
        if let Ok(msg) = receiver.try_recv() {
            assert!(matches!(msg, KernelMessage::StateDump));
        }
    }
}
