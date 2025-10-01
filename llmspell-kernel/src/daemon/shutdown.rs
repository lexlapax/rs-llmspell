//! # Graceful Shutdown Coordinator
//!
//! This module provides graceful shutdown coordination for the daemon,
//! ensuring active operations complete, state is preserved, and clients
//! are notified before shutdown.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{broadcast, RwLock};
use tokio::time::timeout;
use tracing::{debug, error, info, warn};

use crate::io::manager::IOPubMessage;
use crate::io::router::{MessageDestination, MessageRouter};
use crate::state::KernelState;

/// Shutdown configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShutdownConfig {
    /// Timeout for graceful shutdown in seconds
    pub grace_period_secs: u64,
    /// Path to save state on shutdown
    pub state_save_path: PathBuf,
    /// Whether to notify clients before shutdown
    pub notify_clients: bool,
    /// Maximum time to wait for active operations
    pub operation_timeout_secs: u64,
    /// Enable state preservation
    pub preserve_state: bool,
}

impl Default for ShutdownConfig {
    fn default() -> Self {
        Self {
            grace_period_secs: 5,
            state_save_path: PathBuf::from("~/.llmspell/kernel_state.json"),
            notify_clients: true,
            operation_timeout_secs: 10,
            preserve_state: true,
        }
    }
}

/// Shutdown phase tracking
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ShutdownPhase {
    /// Normal operation
    Running,
    /// Shutdown initiated, stopping new requests
    Initiated,
    /// Waiting for active operations to complete
    WaitingForOperations,
    /// Saving state to disk
    SavingState,
    /// Notifying clients
    NotifyingClients,
    /// Cleanup phase
    Cleanup,
    /// Shutdown complete
    Complete,
}

/// Shutdown statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ShutdownStats {
    /// Time shutdown was initiated (as timestamp)
    pub initiated_at_ms: Option<u64>,
    /// Operations completed during shutdown
    pub operations_completed: u64,
    /// Operations cancelled during shutdown
    pub operations_cancelled: u64,
    /// Clients notified
    pub clients_notified: u64,
    /// Whether state was successfully saved
    pub state_saved: bool,
    /// Total shutdown duration
    pub duration_ms: Option<u64>,
}

/// Graceful shutdown coordinator
pub struct ShutdownCoordinator {
    /// Configuration
    config: ShutdownConfig,
    /// Current shutdown phase
    phase: Arc<RwLock<ShutdownPhase>>,
    /// Shutdown requested flag
    shutdown_requested: Arc<AtomicBool>,
    /// Active operation counter
    active_operations: Arc<AtomicU64>,
    /// Shutdown statistics
    stats: Arc<RwLock<ShutdownStats>>,
    /// Broadcast channel for shutdown events
    shutdown_tx: broadcast::Sender<ShutdownPhase>,
    /// Message router for client notifications
    message_router: Option<Arc<MessageRouter>>,
    /// Kernel state for preservation
    kernel_state: Option<Arc<KernelState>>,
    /// Track when shutdown started for duration calculation
    shutdown_start: Option<Instant>,
}

impl ShutdownCoordinator {
    /// Create new shutdown coordinator
    pub fn new(config: ShutdownConfig) -> Self {
        let (shutdown_tx, _) = broadcast::channel(16);

        Self {
            config,
            phase: Arc::new(RwLock::new(ShutdownPhase::Running)),
            shutdown_requested: Arc::new(AtomicBool::new(false)),
            active_operations: Arc::new(AtomicU64::new(0)),
            stats: Arc::new(RwLock::new(ShutdownStats::default())),
            shutdown_tx,
            message_router: None,
            kernel_state: None,
            shutdown_start: None,
        }
    }

    /// Set message router for client notifications
    pub fn set_message_router(&mut self, router: Arc<MessageRouter>) {
        self.message_router = Some(router);
    }

    /// Set kernel state for preservation
    pub fn set_kernel_state(&mut self, state: Arc<KernelState>) {
        self.kernel_state = Some(state);
    }

    /// Get shutdown receiver for monitoring
    pub fn subscribe(&self) -> broadcast::Receiver<ShutdownPhase> {
        self.shutdown_tx.subscribe()
    }

    /// Increment active operation counter
    pub fn begin_operation(&self) {
        self.active_operations.fetch_add(1, Ordering::SeqCst);
    }

    /// Decrement active operation counter
    pub fn end_operation(&self) {
        let prev = self.active_operations.fetch_sub(1, Ordering::SeqCst);
        if prev == 0 {
            error!("Operation counter underflow!");
        }
    }

    /// Get active operation count
    pub fn active_operation_count(&self) -> u64 {
        self.active_operations.load(Ordering::SeqCst)
    }

    /// Check if shutdown is requested
    pub fn is_shutdown_requested(&self) -> bool {
        self.shutdown_requested.load(Ordering::SeqCst)
    }

    /// Check if accepting new requests
    pub async fn is_accepting_requests(&self) -> bool {
        let phase = *self.phase.read().await;
        phase == ShutdownPhase::Running
    }

    /// Initiate graceful shutdown
    ///
    /// # Errors
    ///
    /// Returns an error if the shutdown sequence fails
    ///
    /// # Panics
    ///
    /// May panic if system time is before UNIX epoch
    pub async fn initiate_shutdown(&self) -> Result<()> {
        // Check if already shutting down
        let mut phase = self.phase.write().await;
        if *phase != ShutdownPhase::Running {
            info!("Shutdown already in progress, phase: {:?}", *phase);
            return Ok(());
        }

        info!("Initiating graceful shutdown");
        *phase = ShutdownPhase::Initiated;
        drop(phase);

        // Update stats and track start time
        let now = Instant::now();
        let self_mut = std::ptr::from_ref(self).cast_mut();
        unsafe {
            (*self_mut).shutdown_start = Some(now);
        }

        let mut stats = self.stats.write().await;
        stats.initiated_at_ms = Some(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
        );
        drop(stats);

        // Set shutdown flag
        self.shutdown_requested.store(true, Ordering::SeqCst);

        // Broadcast phase change
        let _ = self.shutdown_tx.send(ShutdownPhase::Initiated);

        // Execute shutdown sequence with timeout
        let grace_period = Duration::from_secs(self.config.grace_period_secs);

        if let Ok(result) = timeout(grace_period, self.execute_shutdown_sequence()).await {
            result?;
        } else {
            warn!("Graceful shutdown timeout, forcing shutdown");
            self.force_shutdown().await?;
        }

        Ok(())
    }

    /// Execute the shutdown sequence
    async fn execute_shutdown_sequence(&self) -> Result<()> {
        // Phase 1: Stop accepting new requests (already done in initiate_shutdown)

        // Phase 2: Wait for active operations
        self.wait_for_operations().await?;

        // Phase 3: Save state if configured
        if self.config.preserve_state {
            self.save_state().await?;
        }

        // Phase 4: Notify clients if configured
        if self.config.notify_clients {
            self.notify_clients().await?;
        }

        // Phase 5: Cleanup
        self.cleanup().await?;

        // Update final phase
        *self.phase.write().await = ShutdownPhase::Complete;
        let _ = self.shutdown_tx.send(ShutdownPhase::Complete);

        // Update stats
        let mut stats = self.stats.write().await;
        if let Some(started) = self.shutdown_start {
            stats.duration_ms = Some(started.elapsed().as_millis() as u64);
        }

        info!("Graceful shutdown complete. Stats: {:?}", *stats);
        Ok(())
    }

    /// Wait for active operations to complete
    async fn wait_for_operations(&self) -> Result<()> {
        *self.phase.write().await = ShutdownPhase::WaitingForOperations;
        let _ = self.shutdown_tx.send(ShutdownPhase::WaitingForOperations);

        let timeout_duration = Duration::from_secs(self.config.operation_timeout_secs);
        let start = Instant::now();

        while self.active_operations.load(Ordering::SeqCst) > 0 {
            if start.elapsed() > timeout_duration {
                let remaining = self.active_operations.load(Ordering::SeqCst);
                warn!("Timeout waiting for {} operations to complete", remaining);

                // Update stats
                let mut stats = self.stats.write().await;
                stats.operations_cancelled = remaining;

                break;
            }

            debug!(
                "Waiting for {} active operations to complete",
                self.active_operations.load(Ordering::SeqCst)
            );

            tokio::time::sleep(Duration::from_millis(100)).await;
        }

        let mut stats = self.stats.write().await;
        stats.operations_completed = stats
            .operations_cancelled
            .saturating_sub(self.active_operations.load(Ordering::SeqCst));

        info!("Operations phase complete");
        Ok(())
    }

    /// Save kernel state to disk
    async fn save_state(&self) -> Result<()> {
        *self.phase.write().await = ShutdownPhase::SavingState;
        let _ = self.shutdown_tx.send(ShutdownPhase::SavingState);

        info!("Saving kernel state to {:?}", self.config.state_save_path);

        if let Some(ref _state) = self.kernel_state {
            // Expand home directory if needed
            let save_path = if self.config.state_save_path.starts_with("~") {
                let home = std::env::var("HOME").context("HOME not set")?;
                PathBuf::from(home).join(
                    self.config
                        .state_save_path
                        .strip_prefix("~/")
                        .unwrap_or(&self.config.state_save_path),
                )
            } else {
                self.config.state_save_path.clone()
            };

            // Create parent directory if needed
            if let Some(parent) = save_path.parent() {
                std::fs::create_dir_all(parent).context("Failed to create state directory")?;
            }

            // Save current state as JSON
            // For now, just save basic info - full state serialization can be added later
            let state_json = serde_json::json!({
                "timestamp": std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis(),
                "shutdown_reason": "graceful",
                "active_operations": self.active_operations.load(Ordering::SeqCst)
            });
            let json =
                serde_json::to_string_pretty(&state_json).context("Failed to serialize state")?;

            std::fs::write(&save_path, json)
                .with_context(|| format!("Failed to write state to {}", save_path.display()))?;

            let mut stats = self.stats.write().await;
            stats.state_saved = true;

            info!("State saved successfully");
        } else {
            debug!("No kernel state to save");
        }

        Ok(())
    }

    /// Notify connected clients about shutdown
    async fn notify_clients(&self) -> Result<()> {
        *self.phase.write().await = ShutdownPhase::NotifyingClients;
        let _ = self.shutdown_tx.send(ShutdownPhase::NotifyingClients);

        info!("Notifying clients about shutdown");

        if let Some(ref router) = self.message_router {
            // Create shutdown notification message
            let notification = IOPubMessage {
                header: crate::io::manager::MessageHeader::new("shutdown_notification", "kernel"),
                parent_header: None,
                metadata: HashMap::new(),
                content: {
                    let mut content = HashMap::new();
                    content.insert("restart".to_string(), serde_json::json!(false));
                    content.insert("status".to_string(), serde_json::json!("shutdown"));
                    content.insert(
                        "message".to_string(),
                        serde_json::json!("Kernel is shutting down gracefully"),
                    );
                    content
                },
            };

            // Broadcast to all clients
            router
                .route_message(notification, MessageDestination::Broadcast)
                .await?;

            // Get client count for stats
            let clients = router.active_client_count();
            let mut stats = self.stats.write().await;
            stats.clients_notified = clients as u64;

            info!("Notified {} clients", clients);
        } else {
            debug!("No message router configured for client notification");
        }

        Ok(())
    }

    /// Perform cleanup operations
    async fn cleanup(&self) -> Result<()> {
        *self.phase.write().await = ShutdownPhase::Cleanup;
        let _ = self.shutdown_tx.send(ShutdownPhase::Cleanup);

        info!("Performing cleanup");

        // Additional cleanup operations can be added here
        // - Close database connections
        // - Flush logs
        // - Release resources

        Ok(())
    }

    /// Force immediate shutdown
    ///
    /// # Errors
    ///
    /// Returns an error if state saving fails during forced shutdown
    pub async fn force_shutdown(&self) -> Result<()> {
        error!("Forcing immediate shutdown");

        // Cancel all operations
        let cancelled = self.active_operations.swap(0, Ordering::SeqCst);

        // Update stats
        let mut stats = self.stats.write().await;
        stats.operations_cancelled = cancelled;

        // Try to save state even in forced shutdown
        if self.config.preserve_state {
            if let Err(e) = self.save_state().await {
                error!("Failed to save state during forced shutdown: {}", e);
            }
        }

        // Update phase
        *self.phase.write().await = ShutdownPhase::Complete;
        let _ = self.shutdown_tx.send(ShutdownPhase::Complete);

        Ok(())
    }

    /// Get current shutdown phase
    pub async fn current_phase(&self) -> ShutdownPhase {
        *self.phase.read().await
    }

    /// Get shutdown statistics
    pub async fn get_stats(&self) -> ShutdownStats {
        self.stats.read().await.clone()
    }
}

/// Shutdown guard for automatic operation tracking
pub struct OperationGuard {
    coordinator: Arc<ShutdownCoordinator>,
}

impl OperationGuard {
    /// Create new operation guard
    pub fn new(coordinator: Arc<ShutdownCoordinator>) -> Self {
        coordinator.begin_operation();
        Self { coordinator }
    }
}

impl Drop for OperationGuard {
    fn drop(&mut self) {
        self.coordinator.end_operation();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_shutdown_coordinator_creation() {
        let config = ShutdownConfig::default();
        let coordinator = ShutdownCoordinator::new(config);

        assert_eq!(coordinator.active_operation_count(), 0);
        assert!(!coordinator.is_shutdown_requested());
        assert!(coordinator.is_accepting_requests().await);
    }

    #[tokio::test]
    async fn test_operation_tracking() {
        let coordinator = Arc::new(ShutdownCoordinator::new(ShutdownConfig::default()));

        // Track operations with guards
        {
            let _guard1 = OperationGuard::new(coordinator.clone());
            assert_eq!(coordinator.active_operation_count(), 1);

            {
                let _guard2 = OperationGuard::new(coordinator.clone());
                assert_eq!(coordinator.active_operation_count(), 2);
            }

            assert_eq!(coordinator.active_operation_count(), 1);
        }

        assert_eq!(coordinator.active_operation_count(), 0);
    }

    #[tokio::test]
    async fn test_shutdown_phases() {
        let config = ShutdownConfig {
            preserve_state: false,
            notify_clients: false,
            grace_period_secs: 1,
            ..Default::default()
        };

        let coordinator = Arc::new(ShutdownCoordinator::new(config));

        // Subscribe to phase changes
        let mut rx = coordinator.subscribe();

        // Start shutdown
        let shutdown_handle = tokio::spawn({
            let coordinator = coordinator.clone();
            async move { coordinator.initiate_shutdown().await }
        });

        // Check phase transitions
        let phase = rx.recv().await.unwrap();
        assert_eq!(phase, ShutdownPhase::Initiated);

        // Wait for completion
        shutdown_handle.await.unwrap().unwrap();

        assert_eq!(coordinator.current_phase().await, ShutdownPhase::Complete);
    }

    #[tokio::test]
    async fn test_graceful_shutdown_with_timeout() {
        let config = ShutdownConfig {
            grace_period_secs: 1,
            operation_timeout_secs: 1,
            preserve_state: false,
            notify_clients: false,
            ..Default::default()
        };

        let coordinator = Arc::new(ShutdownCoordinator::new(config));

        // Add an operation that won't complete
        coordinator.begin_operation();

        // Initiate shutdown
        let result = coordinator.initiate_shutdown().await;
        assert!(result.is_ok());

        // Check that operation was cancelled
        let stats = coordinator.get_stats().await;
        assert_eq!(stats.operations_cancelled, 1);
    }

    #[tokio::test]
    async fn test_multiple_shutdown_requests() {
        let config = ShutdownConfig {
            preserve_state: false,
            notify_clients: false,
            ..Default::default()
        };

        let coordinator = Arc::new(ShutdownCoordinator::new(config));

        // Start first shutdown
        let handle1 = tokio::spawn({
            let coordinator = coordinator.clone();
            async move { coordinator.initiate_shutdown().await }
        });

        // Small delay
        tokio::time::sleep(Duration::from_millis(10)).await;

        // Try second shutdown (should be ignored)
        let handle2 = tokio::spawn({
            let coordinator = coordinator.clone();
            async move { coordinator.initiate_shutdown().await }
        });

        // Both should complete successfully
        handle1.await.unwrap().unwrap();
        handle2.await.unwrap().unwrap();

        assert_eq!(coordinator.current_phase().await, ShutdownPhase::Complete);
    }
}
