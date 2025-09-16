//! ABOUTME: Temporary compatibility layer for kernel integration during Phase 9.4a.2 migration
//! ABOUTME: Provides bridge between old kernel API expectations and new comprehensive sessions
//! TODO: Remove this file in Task 9.4a.2.6 when kernel is adapted to new API

use crate::sessions::{SessionManager, SessionManagerConfig, SessionConfig, SessionId, Result, SessionError};
use crate::state::KernelState;
use anyhow::Result as AnyhowResult;
use std::sync::Arc;

/// Temporary trait for kernel integration - matches old API expectations
pub trait KernelSessionIntegration {
    /// Handle kernel message in session context
    ///
    /// # Errors
    ///
    /// Returns an error if message handling fails
    fn handle_kernel_message(&self, msg: serde_json::Value) -> AnyhowResult<()>;

    /// Apply session policies to message
    ///
    /// # Errors
    ///
    /// Returns an error if policy evaluation fails
    fn apply_policies(&self, msg: &serde_json::Value) -> AnyhowResult<bool>;

    /// Track message for correlation
    fn track_message(&self, msg: &serde_json::Value);
}

impl SessionManager {
    /// **TEMPORARY COMPATIBILITY**: Simplified constructor matching old API
    ///
    /// This creates a SessionManager with default dependencies for kernel compatibility.
    /// TODO: Remove in Task 9.4a.2.6 when kernel uses proper constructor
    ///
    /// # Errors
    ///
    /// Returns an error if default dependencies cannot be created
    pub fn new_legacy(config: SessionConfig) -> Result<Self> {
        // Create default dependencies for compatibility
        use llmspell_state_persistence::StateManager;
        use llmspell_storage::MemoryBackend;
        use llmspell_hooks::{HookExecutor, HookRegistry};
        use llmspell_events::bus::EventBus;

        // This is a blocking async call - not ideal but needed for compatibility
        let state_manager = Arc::new(
            tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(async {
                    StateManager::new().await
                })
            })
            .map_err(|e| SessionError::Configuration(format!("Failed to create state manager: {}", e)))?
        );

        let storage_backend = Arc::new(MemoryBackend::new());
        let hook_registry = Arc::new(HookRegistry::new());
        let hook_executor = Arc::new(HookExecutor::new());
        let event_bus = Arc::new(EventBus::new());

        // Convert SessionConfig to SessionManagerConfig
        let manager_config = SessionManagerConfig::default(); // TODO: Convert properly
        #[allow(unused_variables)]
        let _config = config; // Suppress warning for compatibility parameter

        Self::new(
            state_manager,
            storage_backend,
            hook_registry,
            hook_executor,
            &event_bus,
            manager_config,
        )
    }

    /// **TEMPORARY COMPATIBILITY**: Set kernel state reference
    /// TODO: Remove in Task 9.4a.2.6 when kernel uses proper integration
    pub fn set_kernel_state(&mut self, _state: Arc<KernelState>) {
        // For now, this is a no-op since the new SessionManager doesn't use KernelState directly
        // The new architecture uses StateManager instead
        // TODO: Integrate properly when kernel is adapted
    }

    /// **TEMPORARY COMPATIBILITY**: Create session with old signature (blocking)
    /// TODO: Remove in Task 9.4a.2.6 when kernel uses proper session creation
    pub fn create_session_legacy(&self, owner: Option<String>) -> Result<SessionId> {
        use crate::sessions::types::CreateSessionOptions;

        let options = CreateSessionOptions::builder()
            .created_by(owner.unwrap_or_default())
            .build();

        // Use blocking call for compatibility
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                self.create_session(options).await
            })
        })
    }
}

/// **TEMPORARY COMPATIBILITY**: Implement old kernel integration trait
impl KernelSessionIntegration for SessionManager {
    fn handle_kernel_message(&self, _msg: serde_json::Value) -> AnyhowResult<()> {
        // TODO: Implement proper message handling when kernel is adapted
        // For now, this is a no-op to allow compilation
        Ok(())
    }

    fn apply_policies(&self, _msg: &serde_json::Value) -> AnyhowResult<bool> {
        // TODO: Implement proper policy application when kernel is adapted
        // For now, allow all messages
        Ok(true)
    }

    fn track_message(&self, _msg: &serde_json::Value) {
        // TODO: Implement proper message tracking when kernel is adapted
        // For now, this is a no-op
    }
}