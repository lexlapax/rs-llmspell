//! ABOUTME: Session replay engine for reconstructing session history from events and hooks
//! ABOUTME: Leverages existing replay infrastructure from llmspell-hooks and llmspell-state-persistence

mod hook_replay_bridge;
pub mod session_adapter;

#[cfg(test)]
mod tests;

pub use hook_replay_bridge::HookReplayBridge;

use crate::{Result, SessionId};
use llmspell_events::EventBus;
use llmspell_hooks::replay::ReplayManager;
use llmspell_state_persistence::manager::HookReplayManager;
use llmspell_storage::StorageBackend;
use session_adapter::SessionReplayAdapter;
use std::sync::Arc;

/// Session replay engine integrating existing replay infrastructure
#[derive(Clone)]
pub struct ReplayEngine {
    /// Core replay manager from llmspell-hooks
    #[allow(dead_code)]
    replay_manager: Arc<ReplayManager>,
    /// Hook replay manager from llmspell-state-persistence
    #[allow(dead_code)]
    hook_replay_manager: Arc<HookReplayManager>,
    /// Session storage backend
    #[allow(dead_code)]
    storage_backend: Arc<dyn StorageBackend>,
    /// Event bus for publishing replay events
    #[allow(dead_code)]
    event_bus: Arc<EventBus>,
    /// Session-specific replay adapter
    session_adapter: Arc<SessionReplayAdapter>,
}

impl ReplayEngine {
    /// Create new replay engine with existing infrastructure
    pub fn new(
        replay_manager: Arc<ReplayManager>,
        hook_replay_manager: Arc<HookReplayManager>,
        storage_backend: Arc<dyn StorageBackend>,
        event_bus: Arc<EventBus>,
    ) -> Self {
        let session_adapter = Arc::new(SessionReplayAdapter::new(
            replay_manager.clone(),
            hook_replay_manager.clone(),
            storage_backend.clone(),
            event_bus.clone(),
        ));

        Self {
            replay_manager,
            hook_replay_manager,
            storage_backend,
            event_bus,
            session_adapter,
        }
    }

    /// Get the session replay adapter
    pub fn session_adapter(&self) -> &Arc<SessionReplayAdapter> {
        &self.session_adapter
    }

    /// Check if a session can be replayed
    pub async fn can_replay_session(&self, session_id: &SessionId) -> Result<bool> {
        self.session_adapter.can_replay_session(session_id).await
    }

    /// Replay a session
    pub async fn replay_session(
        &self,
        session_id: &SessionId,
        config: session_adapter::SessionReplayConfig,
    ) -> Result<session_adapter::SessionReplayResult> {
        self.session_adapter
            .replay_session(session_id, config)
            .await
    }

    /// Get session timeline
    pub async fn get_session_timeline(
        &self,
        session_id: &SessionId,
    ) -> Result<Vec<llmspell_state_persistence::manager::SerializedHookExecution>> {
        self.session_adapter.get_session_timeline(session_id).await
    }

    /// Get replay status for a session
    pub fn get_replay_status(
        &self,
        session_id: &SessionId,
    ) -> Option<session_adapter::SessionReplayStatus> {
        self.session_adapter.get_replay_status(session_id)
    }

    /// Stop session replay
    pub fn stop_replay(&self, session_id: &SessionId) -> Result<()> {
        self.session_adapter.stop_replay(session_id)
    }

    /// Get all active replays
    pub fn get_all_active_replays(&self) -> Vec<session_adapter::SessionReplayStatus> {
        self.session_adapter.get_all_active_replays()
    }

    /// Query hook executions for a specific session
    pub async fn query_session_hooks(
        &self,
        session_id: &SessionId,
        filter: session_adapter::SessionHookFilter,
    ) -> Result<Vec<llmspell_state_persistence::manager::SerializedHookExecution>> {
        self.session_adapter
            .query_session_hooks(session_id, filter)
            .await
    }

    /// Get session replay metadata
    pub async fn get_session_replay_metadata(
        &self,
        session_id: &SessionId,
    ) -> Result<session_adapter::SessionReplayMetadata> {
        self.session_adapter
            .get_session_replay_metadata(session_id)
            .await
    }

    /// List all sessions that can be replayed
    pub async fn list_replayable_sessions(&self) -> Result<Vec<SessionId>> {
        self.session_adapter.list_replayable_sessions().await
    }
}

impl Default for ReplayEngine {
    fn default() -> Self {
        // Minimal stub implementation for task 6.4.1
        // This creates a non-functional but valid ReplayEngine for compilation
        // Real functionality will be implemented in subsequent tasks
        let stub_storage = Arc::new(llmspell_storage::MemoryBackend::new());
        let stub_event_bus = Arc::new(EventBus::new());
        let stub_state_adapter = Arc::new(
            llmspell_state_persistence::backend_adapter::StateStorageAdapter::new(
                stub_storage.clone(),
                "stub".to_string(),
            ),
        );
        let stub_hook_replay_manager = Arc::new(
            llmspell_state_persistence::manager::HookReplayManager::new(stub_state_adapter),
        );

        // Create stub components using the bridge adapter
        let hook_replay_bridge = Arc::new(HookReplayBridge::new(stub_hook_replay_manager.clone()));
        let hooks_storage_backend =
            Arc::new(llmspell_hooks::persistence::InMemoryStorageBackend::new());
        let stub_replay_manager = Arc::new(llmspell_hooks::replay::ReplayManager::new(
            Arc::new(llmspell_hooks::persistence::HookPersistenceManager::new(
                hook_replay_bridge,
            )),
            hooks_storage_backend,
        ));

        Self::new(
            stub_replay_manager,
            stub_hook_replay_manager,
            stub_storage,
            stub_event_bus,
        )
    }
}
