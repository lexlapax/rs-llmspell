//! # Signal Handling for Daemons
//!
//! Provides signal handling infrastructure for daemon processes, converting
//! Unix signals to appropriate kernel messages for graceful shutdown and
//! runtime operations.

use anyhow::{Context, Result};
use nix::sys::signal::{self, SaFlags, SigAction, SigHandler as NixSigHandler, SigSet, Signal};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tracing::{debug, info};

/// Atomic flags for signal handling
static SIGTERM_RECEIVED: AtomicBool = AtomicBool::new(false);
static SIGINT_RECEIVED: AtomicBool = AtomicBool::new(false);
static SIGHUP_RECEIVED: AtomicBool = AtomicBool::new(false);
static SIGUSR1_RECEIVED: AtomicBool = AtomicBool::new(false);
static SIGUSR2_RECEIVED: AtomicBool = AtomicBool::new(false);

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

        // Check SIGUSR1 (custom action - dump state)
        if SIGUSR1_RECEIVED.swap(false, Ordering::SeqCst) {
            info!("SIGUSR1 received, dumping state");
            return Some(SignalAction::DumpState);
        }

        // Check SIGUSR2 (custom action)
        if SIGUSR2_RECEIVED.swap(false, Ordering::SeqCst) {
            info!("SIGUSR2 received, executing custom action");
            return Some(SignalAction::Custom("usr2".to_string()));
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

/// Bridge between Unix signals and kernel messages
pub struct SignalBridge {
    handler: SignalHandler,
    shutdown_requested: Arc<AtomicBool>,
}

impl SignalBridge {
    /// Creates a new signal bridge
    pub fn new() -> Self {
        Self {
            handler: SignalHandler::new(),
            shutdown_requested: Arc::new(AtomicBool::new(false)),
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
}
