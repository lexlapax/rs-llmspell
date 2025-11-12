//! Unified State Management System for Kernel
//!
//! This module provides a unified state system that consolidates execution,
//! session, and debug state with pluggable storage backends.
//!
//! Also includes comprehensive state-persistence and storage functionality
//! consolidated from internal state management and llmspell-storage.

use anyhow::Result;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::{debug, info, instrument, warn};

// Import state traits from core
pub use llmspell_core::state::{StateError, StateResult, StateScope};

// Original kernel state modules
pub mod circuit_breaker;
pub mod kernel_backends; // Original kernel storage backends enum
pub mod persistence;
pub mod types;

// Consolidated state-persistence modules

/// Agent-specific state management with conversation history and metadata
pub mod agent_state;

/// Adapter layer for abstracting different storage backend implementations
pub mod backend_adapter;

/// Comprehensive storage backends from consolidated crates
pub mod backends;

/// Backup and restore functionality for state persistence
pub mod backup;

/// Configuration management for state storage and persistence
pub mod config;

/// Hook system for state change notifications and event handling
pub mod hooks;

/// State-memory synchronization hook (Phase 13.7.4)
pub mod memory_hook;

/// Cryptographic key management for secure state storage
pub mod key_manager;

/// High-level state management coordination and lifecycle control
pub mod manager;

/// Schema migration system for evolving state structures
pub mod migration;

/// Performance monitoring and optimization for state operations
pub mod performance;

/// Schema definition and versioning for state structures
pub mod schema;

/// Sensitive data handling and redaction for privacy compliance
pub mod sensitive_data;

/// Testing utilities and helpers for session state validation
pub mod session_test;
pub mod vector_storage;

// Re-export original kernel state types
pub use circuit_breaker::CircuitBreaker;
pub use persistence::StatePersistence;
pub use types::{DebugState, ExecutionState, SessionState};

// Re-export consolidated state-persistence types (needed by sessions)
pub use agent_state::{
    AgentMetadata, AgentStateData, MessageRole, PersistentAgentState, ToolUsageStats,
};
pub use backend_adapter::StateStorageAdapter;
pub use config::{
    BackupConfig, EncryptionConfig, PerformanceConfig, PersistenceConfig, RocksDBConfig,
    StorageBackendType,
};
pub use manager::{HookReplayManager, SerializedHookExecution, StateManager};
pub use sensitive_data::{RedactSensitiveData, SensitiveDataConfig, SensitiveDataProtector};

// Re-export original kernel storage types
pub use kernel_backends::{MemoryBackend as KernelMemoryBackend, StorageBackend, VectorBackend};

// Re-export comprehensive storage backends
pub use backends::MemoryBackend;

// Re-export vector storage types
pub use vector_storage::{
    DistanceMetric, HNSWConfig, HNSWStorage, NamespaceStats, ScopedStats, StorageStats,
    VectorEntry, VectorQuery, VectorResult, VectorStorage,
};

// Migration and schema types
pub use migration::{
    DataTransformer, MigrationConfig, MigrationEngine, MigrationResult, MigrationStatus,
    ValidationLevel, ValidationResult,
};
pub use schema::{
    CompatibilityChecker, CompatibilityResult, EnhancedStateSchema, MigrationPlan,
    MigrationPlanner, SchemaRegistry, SchemaVersion, SemanticVersion,
};

// Performance optimization types
pub use performance::{FastPathConfig, FastPathManager, StateClass};

// State-memory synchronization (Phase 13.7.4)
pub use memory_hook::StateMemoryHook;

/// Unified kernel state that combines execution, session, and debug state
#[derive(Clone)]
pub struct KernelState {
    /// Execution state from execution bridge
    execution: Arc<RwLock<ExecutionState>>,
    /// Session state from sessions module
    session: Arc<RwLock<SessionState>>,
    /// Debug state from debug coordinator
    debug: Arc<RwLock<DebugState>>,
    /// Storage backend for persistence
    backend: Arc<RwLock<StorageBackend>>,
    /// Circuit breaker for resource protection
    circuit_breaker: Arc<CircuitBreaker>,
    /// Performance metrics
    metrics: Arc<RwLock<StateMetrics>>,
}

/// Performance metrics for state operations
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StateMetrics {
    /// Total number of state reads
    pub reads: u64,
    /// Total number of state writes
    pub writes: u64,
    /// Average read latency in microseconds
    pub avg_read_latency_us: u64,
    /// Average write latency in microseconds
    pub avg_write_latency_us: u64,
    /// Number of persistence operations
    pub persistence_ops: u64,
    /// Number of circuit breaker trips
    pub circuit_breaker_trips: u64,
    /// Total number of read errors
    pub read_errors: u64,
    /// Total number of write errors
    pub write_errors: u64,
    /// Total number of persistence errors
    pub persistence_errors: u64,
    /// Timestamp of last error
    #[serde(skip)]
    pub last_error_at: Option<Instant>,
    /// Last update timestamp (skipped in serialization)
    #[serde(skip)]
    pub last_update: Option<Instant>,
}

impl KernelState {
    /// Create a new kernel state with the specified backend
    ///
    /// # Errors
    ///
    /// Returns an error if state initialization fails
    #[instrument(level = "info", skip_all)]
    pub fn new(backend: StorageBackend) -> Result<Self> {
        info!("Initializing kernel state with {:?} backend", backend);

        let circuit_breaker = Arc::new(CircuitBreaker::new(
            10,                      // failure threshold
            Duration::from_secs(60), // reset timeout
            Duration::from_secs(30), // half-open test delay
        ));

        let state = Self {
            execution: Arc::new(RwLock::new(ExecutionState::default())),
            session: Arc::new(RwLock::new(SessionState::default())),
            debug: Arc::new(RwLock::new(DebugState::default())),
            backend: Arc::new(RwLock::new(backend)),
            circuit_breaker,
            metrics: Arc::new(RwLock::new(StateMetrics::default())),
        };

        // Try to restore from persistence if available
        if let Err(e) = state.restore() {
            info!("Failed to restore state from persistence: {}", e);
            // Continue with fresh state
        }

        Ok(state)
    }

    /// Get execution state
    pub fn execution(&self) -> Arc<RwLock<ExecutionState>> {
        self.record_read();
        self.execution.clone()
    }

    /// Get session state
    pub fn session(&self) -> Arc<RwLock<SessionState>> {
        self.record_read();
        self.session.clone()
    }

    /// Get debug state
    pub fn debug(&self) -> Arc<RwLock<DebugState>> {
        self.record_read();
        self.debug.clone()
    }

    /// Update execution state
    ///
    /// # Errors
    ///
    /// Returns an error if the update fails or circuit breaker is open
    #[instrument(level = "debug", skip_all)]
    pub fn update_execution<F>(&self, updater: F) -> Result<()>
    where
        F: FnOnce(&mut ExecutionState) -> Result<()>,
    {
        self.circuit_breaker.call(|| {
            let start = Instant::now();
            let mut state = self.execution.write();
            let result = updater(&mut state);
            self.record_write(start.elapsed());
            result
        })
    }

    /// Update session state
    ///
    /// # Errors
    ///
    /// Returns an error if the update fails or circuit breaker is open
    #[instrument(level = "debug", skip_all)]
    pub fn update_session<F>(&self, updater: F) -> Result<()>
    where
        F: FnOnce(&mut SessionState) -> Result<()>,
    {
        self.circuit_breaker.call(|| {
            let start = Instant::now();
            let mut state = self.session.write();
            let result = updater(&mut state);
            self.record_write(start.elapsed());
            result
        })
    }

    /// Update debug state
    ///
    /// # Errors
    ///
    /// Returns an error if the update fails or circuit breaker is open
    #[instrument(level = "debug", skip_all)]
    pub fn update_debug<F>(&self, updater: F) -> Result<()>
    where
        F: FnOnce(&mut DebugState) -> Result<()>,
    {
        self.circuit_breaker.call(|| {
            let start = Instant::now();
            let mut state = self.debug.write();
            let result = updater(&mut state);
            self.record_write(start.elapsed());
            result
        })
    }

    /// Persist current state to backend
    ///
    /// # Errors
    ///
    /// Returns an error if persistence fails
    #[instrument(level = "info", skip_all)]
    pub fn persist(&self) -> Result<()> {
        info!("Persisting kernel state");
        let start = Instant::now();

        self.circuit_breaker.call(|| {
            let mut backend = self.backend.write();

            // Serialize and store each state component
            backend.store_execution(&self.execution.read())?;
            backend.store_session(&self.session.read())?;
            backend.store_debug(&self.debug.read())?;

            let mut metrics = self.metrics.write();
            metrics.persistence_ops += 1;

            debug!("State persisted in {:?}", start.elapsed());
            Ok(())
        })
    }

    /// Restore state from backend
    ///
    /// # Errors
    ///
    /// Returns an error if restoration fails
    #[instrument(level = "info", skip_all)]
    pub fn restore(&self) -> Result<()> {
        info!("Restoring kernel state");
        let start = Instant::now();

        self.circuit_breaker.call(|| {
            let backend = self.backend.read();

            // Restore each state component
            if let Some(exec_state) = backend.load_execution()? {
                *self.execution.write() = exec_state;
            }

            if let Some(session_state) = backend.load_session()? {
                *self.session.write() = session_state;
            }

            if let Some(debug_state) = backend.load_debug()? {
                *self.debug.write() = debug_state;
            }

            debug!("State restored in {:?}", start.elapsed());
            Ok(())
        })
    }

    /// Clear all state
    ///
    /// # Errors
    ///
    /// Returns an error if clearing fails
    pub fn clear(&self) -> Result<()> {
        info!("Clearing all kernel state");

        *self.execution.write() = ExecutionState::default();
        *self.session.write() = SessionState::default();
        *self.debug.write() = DebugState::default();

        self.backend.write().clear()?;

        Ok(())
    }

    /// Get current metrics
    pub fn metrics(&self) -> StateMetrics {
        self.metrics.read().clone()
    }

    /// Record a read operation
    fn record_read(&self) {
        let mut metrics = self.metrics.write();
        metrics.reads += 1;
        metrics.last_update = Some(Instant::now());
    }

    /// Record a write operation
    fn record_write(&self, duration: Duration) {
        let mut metrics = self.metrics.write();
        metrics.writes += 1;

        let latency_us = u64::try_from(duration.as_micros()).unwrap_or(u64::MAX);
        if metrics.avg_write_latency_us == 0 {
            metrics.avg_write_latency_us = latency_us;
        } else {
            // Simple moving average
            metrics.avg_write_latency_us = u64::midpoint(metrics.avg_write_latency_us, latency_us);
        }

        metrics.last_update = Some(Instant::now());
    }

    /// Check if circuit breaker is open
    pub fn is_circuit_open(&self) -> bool {
        self.circuit_breaker.is_open()
    }

    /// Reset circuit breaker
    pub fn reset_circuit(&self) {
        self.circuit_breaker.reset();
    }

    /// Record a read error
    pub fn record_read_error(&self) {
        let mut metrics = self.metrics.write();
        metrics.read_errors += 1;
        metrics.last_error_at = Some(Instant::now());
        metrics.last_update = Some(Instant::now());
    }

    /// Record a write error
    pub fn record_write_error(&self) {
        let mut metrics = self.metrics.write();
        metrics.write_errors += 1;
        metrics.last_error_at = Some(Instant::now());
        metrics.last_update = Some(Instant::now());
    }

    /// Record a persistence error
    pub fn record_persistence_error(&self) {
        let mut metrics = self.metrics.write();
        metrics.persistence_errors += 1;
        metrics.last_error_at = Some(Instant::now());
        metrics.last_update = Some(Instant::now());
    }

    /// Get error rate per minute based on recent errors
    pub fn get_error_rate_per_minute(&self) -> f64 {
        let metrics = self.metrics.read();
        let total_errors = metrics.read_errors + metrics.write_errors + metrics.persistence_errors;

        // Simple calculation: assume uniform distribution over uptime
        // In production, would use sliding window
        if let Some(last_error) = metrics.last_error_at {
            let elapsed_minutes = last_error.elapsed().as_secs_f64() / 60.0;
            if elapsed_minutes > 0.0 {
                total_errors as f64 / elapsed_minutes
            } else {
                0.0
            }
        } else {
            0.0
        }
    }

    /// Get the current session ID
    pub fn session_id(&self) -> String {
        self.session
            .read()
            .session_id
            .clone()
            .unwrap_or_else(|| "default".to_string())
    }

    /// Get the current execution count
    pub fn execution_count(&self) -> u64 {
        self.execution.read().execution_count as u64
    }

    /// Create a snapshot of current state for serialization
    pub fn snapshot(&self) -> serde_json::Value {
        let execution = self.execution.read();
        let session = self.session.read();
        let debug = self.debug.read();
        let metrics = self.metrics();

        json!({
            "execution": {
                "count": execution.execution_count,
                "current_execution_id": execution.current_execution_id,
                "status": format!("{execution_status:?}", execution_status=execution.status),
                "current_code": execution.current_code,
                "total_execution_time_secs": execution.total_execution_time.as_secs(),
                "history_count": execution.history.len(),
            },
            "session": {
                "id": session.session_id,
                "created_at": session.created_at.map(|t| format!("{t:?}")),
                "last_activity": session.last_activity.map(|t| format!("{t:?}")),
                "metadata": session.metadata,
                "breakpoint_count": session.breakpoints.len(),
            },
            "debug": {
                "enabled": debug.enabled,
                "mode": format!("{debug_mode:?}", debug_mode=debug.mode),
                "stack_frames": debug.stack_frames.len(),
                "variable_scopes": debug.variables.len(),
                "watches": debug.watches.len(),
            },
            "metrics": metrics,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kernel_state_creation() {
        let backend = StorageBackend::Memory(Box::new(kernel_backends::MemoryBackend::new()));
        let state = KernelState::new(backend).unwrap();

        assert!(!state.is_circuit_open());
        assert_eq!(state.metrics().reads, 0);
        assert_eq!(state.metrics().writes, 0);
    }

    #[test]
    fn test_state_updates() {
        let backend = StorageBackend::Memory(Box::new(kernel_backends::MemoryBackend::new()));
        let state = KernelState::new(backend).unwrap();

        // Update execution state
        state
            .update_execution(|exec| {
                exec.increment_counter();
                Ok(())
            })
            .unwrap();

        assert_eq!(state.metrics().writes, 1);
    }

    #[test]
    fn test_state_persistence() {
        let backend = StorageBackend::Memory(Box::new(kernel_backends::MemoryBackend::new()));
        let state = KernelState::new(backend).unwrap();

        // Update and persist
        state
            .update_session(|session| {
                session.set_id("test-session");
                Ok(())
            })
            .unwrap();

        state.persist().unwrap();
        assert!(state.metrics().persistence_ops > 0);
    }
}
