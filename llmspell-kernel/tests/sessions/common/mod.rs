//! ABOUTME: Common test utilities for session management integration tests
//! ABOUTME: Provides helper functions and fixtures for consistent test setup

use anyhow::Result;
use llmspell_events::bus::EventBus;
use llmspell_hooks::{HookExecutor, HookRegistry};
use llmspell_sessions::{
    CreateSessionOptions, SessionId, SessionManager, SessionManagerConfig,
    SessionManagerConfigBuilder,
};
use llmspell_state_persistence::StateManager;
use llmspell_storage::{MemoryBackend, StorageBackend};
use std::sync::Arc;
use tempfile::TempDir;

/// Test fixture containing all session management dependencies
pub struct TestFixture {
    pub session_manager: Arc<SessionManager>,
    pub state_manager: Arc<StateManager>,
    pub storage_backend: Arc<dyn StorageBackend>,
    pub event_bus: Arc<EventBus>,
    pub hook_registry: Arc<HookRegistry>,
    pub hook_executor: Arc<HookExecutor>,
    pub _temp_dir: Option<TempDir>,
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
            state_manager,
            storage_backend,
            event_bus,
            hook_registry,
            hook_executor,
            _temp_dir: None,
        })
    }

    /// Create a new test fixture with file storage backend
    pub async fn with_file_storage() -> Result<Self> {
        let temp_dir = TempDir::new()?;
        let config = SessionManagerConfigBuilder::new()
            .storage_path(temp_dir.path())
            .build();

        let mut fixture = Self::with_config(config).await?;
        fixture._temp_dir = Some(temp_dir);
        Ok(fixture)
    }

    /// Create a test session with default options
    pub async fn create_test_session(&self) -> Result<SessionId> {
        self.create_test_session_with_options(CreateSessionOptions::default())
            .await
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

/// Create a minimal test configuration
pub fn minimal_test_config() -> SessionManagerConfig {
    SessionManagerConfigBuilder::new()
        .auto_persist(false)
        .max_active_sessions(10)
        .storage_path("/tmp/llmspell-test")
        .build()
}

/// Create a performance test configuration
pub fn performance_test_config() -> SessionManagerConfig {
    SessionManagerConfigBuilder::new()
        .auto_persist(false)
        .max_active_sessions(1000)
        .enable_compression(false) // Disable for performance tests
        .build()
}

/// Helper to create test artifacts
pub async fn create_test_artifact(
    fixture: &TestFixture,
    session_id: &SessionId,
    name: &str,
    content: &str,
) -> Result<llmspell_sessions::ArtifactId> {
    Ok(fixture
        .session_manager
        .store_artifact(
            session_id,
            llmspell_sessions::ArtifactType::UserInput,
            name.to_string(),
            content.as_bytes().to_vec(),
            Default::default(),
        )
        .await?)
}

/// Helper to verify session state
pub async fn assert_session_status(
    fixture: &TestFixture,
    session_id: &SessionId,
    expected_status: llmspell_sessions::SessionStatus,
) {
    let session = fixture
        .session_manager
        .get_session(session_id)
        .await
        .expect("Failed to get session");
    
    let status = session.status().await;
    assert_eq!(
        status, expected_status,
        "Expected session status {:?}, got {:?}",
        expected_status, status
    );
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
        assert!(fixture.session_manager.list_sessions().await.unwrap().is_empty());
    }
    #[tokio::test]
    async fn test_create_test_sessions() {
        let fixture = TestFixture::new().await.expect("Failed to create fixture");
        let sessions = fixture
            .create_test_sessions(3)
            .await
            .expect("Failed to create sessions");
        
        assert_eq!(sessions.len(), 3);
        let listed = fixture.session_manager.list_sessions().await.unwrap();
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