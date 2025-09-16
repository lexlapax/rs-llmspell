//! ABOUTME: Common test utilities for session management integration tests
//! ABOUTME: Provides helper functions and fixtures for consistent test setup

use anyhow::Result;
use llmspell_events::bus::EventBus;
use llmspell_hooks::{HookExecutor, HookRegistry};
use llmspell_kernel::sessions::{
    CreateSessionOptions, SessionId, SessionManager, SessionManagerConfig,
};
use llmspell_state_persistence::StateManager;
use llmspell_storage::{MemoryBackend, StorageBackend};
use std::sync::Arc;
use tempfile::TempDir;

/// Test fixture containing all session management dependencies
pub struct TestFixture {
    pub session_manager: Arc<SessionManager>,
    _state_manager: Arc<StateManager>,
    _storage_backend: Arc<dyn StorageBackend>,
    _event_bus: Arc<EventBus>,
    _hook_registry: Arc<HookRegistry>,
    _hook_executor: Arc<HookExecutor>,
    _temp_dir: Option<TempDir>,
}

impl TestFixture {
    /// Create a new test fixture with memory backend
    pub async fn new() -> Result<Self> {
        Self::with_config(SessionManagerConfig::default()).await
    }

    /// Create a new test fixture with custom configuration
    pub async fn with_config(config: SessionManagerConfig) -> Result<Self> {
        let storage_backend = Arc::new(MemoryBackend::new());
        let state_manager = Arc::new(StateManager::new().await?);
        let event_bus = Arc::new(EventBus::new());
        let hook_registry = Arc::new(HookRegistry::new());
        let hook_executor = Arc::new(HookExecutor::new());

        let session_manager = Arc::new(SessionManager::new(
            state_manager.clone(),
            storage_backend.clone(),
            hook_registry.clone(),
            hook_executor.clone(),
            &event_bus,
            config,
        )?);

        Ok(Self {
            session_manager,
            _state_manager: state_manager,
            _storage_backend: storage_backend,
            _event_bus: event_bus,
            _hook_registry: hook_registry,
            _hook_executor: hook_executor,
            _temp_dir: None,
        })
    }


    /// Create a test session with custom options
    pub async fn create_test_session_with_options(
        &self,
        options: CreateSessionOptions,
    ) -> Result<SessionId> {
        Ok(self.session_manager.create_session(options).await?)
    }

    /// Create multiple test sessions
    pub async fn create_test_sessions(&self, count: usize) -> Result<Vec<SessionId>> {
        let mut sessions = Vec::new();
        for i in 0..count {
            let options = CreateSessionOptions {
                name: Some(format!("Test Session {}", i)),
                tags: vec!["test".to_string(), format!("session-{}", i)],
                ..Default::default()
            };
            sessions.push(self.create_test_session_with_options(options).await?);
        }
        Ok(sessions)
    }
}

/// Helper for timing operations
pub struct Timer {
    start: std::time::Instant,
}

impl Timer {
    pub fn start() -> Self {
        Self {
            start: std::time::Instant::now(),
        }
    }

    pub fn elapsed_ms(&self) -> u128 {
        self.start.elapsed().as_millis()
    }

    pub fn assert_under_ms(&self, max_ms: u128) {
        let elapsed = self.elapsed_ms();
        assert!(
            elapsed < max_ms,
            "Operation took {}ms, expected under {}ms",
            elapsed,
            max_ms
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn test_fixture_creation() {
        let fixture = TestFixture::new().await.expect("Failed to create fixture");
        assert!(fixture.session_manager.list_sessions(Default::default()).await.unwrap().is_empty());
    }
    #[tokio::test]
    async fn test_create_test_sessions() {
        let fixture = TestFixture::new().await.expect("Failed to create fixture");
        let sessions = fixture
            .create_test_sessions(3)
            .await
            .expect("Failed to create sessions");
        
        assert_eq!(sessions.len(), 3);
        let listed = fixture.session_manager.list_sessions(Default::default()).await.unwrap();
        assert_eq!(listed.len(), 3);
    }
    #[tokio::test]
    async fn test_timer() {
        let timer = Timer::start();
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        assert!(timer.elapsed_ms() >= 10);
        timer.assert_under_ms(100);
    }
}