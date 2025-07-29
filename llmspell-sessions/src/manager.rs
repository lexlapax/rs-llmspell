//! ABOUTME: Session manager orchestration providing centralized session lifecycle management
//! ABOUTME: Coordinates with state persistence, storage backends, hooks, and events

use crate::{
    artifact::ArtifactStorage,
    config::SessionManagerConfig,
    replay::ReplayEngine,
    session::{Session, SessionSnapshot},
    types::{CreateSessionOptions, SessionQuery, SessionSortBy},
    Result, SessionError, SessionId, SessionMetadata,
};
use llmspell_events::{
    bus::EventBus,
    correlation::EventCorrelationTracker,
    universal_event::{Language, UniversalEvent},
};
use llmspell_hooks::{
    ComponentId, ComponentType, HookContext, HookExecutor, HookPoint, HookRegistry, LoggingHook,
    MetricsHook,
};
use llmspell_state_persistence::StateManager;
use llmspell_state_traits::StateScope;
use llmspell_storage::StorageBackend;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{interval, Duration};
use tracing::{debug, error, info, warn};

/// Core session manager orchestrating all session operations
#[derive(Clone)]
pub struct SessionManager {
    /// State persistence manager
    state_manager: Arc<StateManager>,
    /// Storage backend for session data
    storage_backend: Arc<dyn StorageBackend>,
    /// Hook registry for lifecycle events
    hook_registry: Arc<HookRegistry>,
    /// Hook executor for lifecycle events (future use)
    #[allow(dead_code)]
    hook_executor: Arc<HookExecutor>,
    /// Event bus for publishing session events
    event_bus: Arc<EventBus>,
    /// Correlation tracker for event relationships (future use)
    #[allow(dead_code)]
    correlation_tracker: Arc<EventCorrelationTracker>,
    /// Active sessions in memory
    active_sessions: Arc<RwLock<HashMap<SessionId, Session>>>,
    /// Artifact storage system (Phase 6.2)
    #[allow(dead_code)]
    artifact_storage: Arc<ArtifactStorage>,
    /// Session replay engine (Phase 6.4)
    #[allow(dead_code)]
    replay_engine: Arc<ReplayEngine>,
    /// Manager configuration
    config: SessionManagerConfig,
    /// Shutdown signal
    shutdown: Arc<RwLock<bool>>,
}

impl SessionManager {
    /// Create a new session manager with the given dependencies
    ///
    /// # Errors
    ///
    /// Returns an error if configuration is invalid or storage directory cannot be created
    pub fn new(
        state_manager: Arc<StateManager>,
        storage_backend: Arc<dyn StorageBackend>,
        hook_registry: Arc<HookRegistry>,
        hook_executor: Arc<HookExecutor>,
        event_bus: &Arc<EventBus>,
        config: SessionManagerConfig,
    ) -> Result<Self> {
        // Validate configuration
        if config.max_active_sessions == 0 {
            return Err(SessionError::Configuration(
                "max_active_sessions must be greater than 0".to_string(),
            ));
        }

        if config.compression_level > 9 {
            return Err(SessionError::Configuration(
                "compression_level must be between 1-9".to_string(),
            ));
        }

        // Create storage directories if needed
        if config.auto_persist {
            std::fs::create_dir_all(&config.storage_path).map_err(|e| {
                SessionError::Io(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Failed to create storage directory: {e}"),
                ))
            })?;
        }

        // Register built-in hooks for session lifecycle
        if config.hook_config.enable_lifecycle_hooks {
            Self::register_builtin_hooks(&hook_registry)?;
        }

        let manager = Self {
            state_manager,
            storage_backend,
            hook_registry,
            hook_executor,
            event_bus: event_bus.clone(),
            correlation_tracker: Arc::new(EventCorrelationTracker::new(
                llmspell_events::correlation::CorrelationConfig::default(),
            )),
            active_sessions: Arc::new(RwLock::new(HashMap::new())),
            artifact_storage: Arc::new(ArtifactStorage::new()),
            replay_engine: Arc::new(ReplayEngine::new()),
            config,
            shutdown: Arc::new(RwLock::new(false)),
        };

        // Start background tasks
        if manager.config.auto_persist {
            manager.start_auto_persist_task();
        }

        if manager.config.cleanup_config.enable_auto_cleanup {
            manager.start_cleanup_task();
        }

        Ok(manager)
    }

    /// Create a new session
    ///
    /// # Errors
    ///
    /// Returns an error if the maximum number of active sessions is reached
    pub async fn create_session(&self, options: CreateSessionOptions) -> Result<SessionId> {
        // Check session limits
        let active_count = self.active_sessions.read().await.len();
        if active_count >= self.config.max_active_sessions {
            return Err(SessionError::ResourceLimitExceeded {
                resource: "active_sessions".to_string(),
                message: format!(
                    "Maximum active sessions ({}) reached",
                    self.config.max_active_sessions
                ),
            });
        }

        // Create new session
        let session = Session::new(options.clone());
        let session_id = session.id().await;

        // Fire session start hooks
        if self.config.hook_config.enable_lifecycle_hooks {
            let hooks = self.hook_registry.get_hooks(&HookPoint::SessionStart);
            let component_id = ComponentId::new(
                ComponentType::Custom("SessionManager".to_string()),
                "session-manager".to_string(),
            );

            for hook in hooks {
                let mut context = HookContext::new(HookPoint::SessionStart, component_id.clone());
                context.data.insert(
                    "session_id".to_string(),
                    serde_json::json!(session_id.to_string()),
                );
                if let Some(ref name) = options.name {
                    context
                        .data
                        .insert("name".to_string(), serde_json::json!(name));
                }
                if let Some(ref created_by) = options.created_by {
                    context
                        .data
                        .insert("created_by".to_string(), serde_json::json!(created_by));
                }

                if let Err(e) = hook.execute(&mut context).await {
                    warn!("Session start hook failed: {e}");
                }
            }
        }

        // Store in active sessions
        self.active_sessions
            .write()
            .await
            .insert(session_id, session.clone());

        // Initialize state scope
        self.state_manager
            .set(
                StateScope::Session(session_id.to_string()),
                "created_at",
                serde_json::json!(chrono::Utc::now()),
            )
            .await
            .map_err(SessionError::State)?;

        // Publish session created event
        if self.config.event_config.enable_session_events {
            let event = UniversalEvent::new(
                "session.created",
                serde_json::json!({
                    "session_id": session_id.to_string(),
                }),
                Language::Rust,
            );

            if let Err(e) = self.event_bus.publish(event).await {
                warn!("Failed to publish session created event: {e}");
            }
        }

        info!("Created new session: {session_id}");
        Ok(session_id)
    }

    /// Get an active session
    ///
    /// # Errors
    ///
    /// Returns `SessionError::SessionNotFound` if the session does not exist
    pub async fn get_session(&self, session_id: &SessionId) -> Result<Session> {
        self.active_sessions
            .read()
            .await
            .get(session_id)
            .cloned()
            .ok_or_else(|| SessionError::SessionNotFound {
                id: session_id.to_string(),
            })
    }

    /// List sessions matching query
    ///
    /// # Errors
    ///
    /// Currently always succeeds, but returns Result for future error cases
    pub async fn list_sessions(&self, query: SessionQuery) -> Result<Vec<SessionMetadata>> {
        let sessions = self.active_sessions.read().await;
        let mut results = Vec::new();

        for session in sessions.values() {
            let metadata = session.metadata.read().await;

            // Apply filters
            if let Some(status) = query.status {
                if metadata.status != status {
                    continue;
                }
            }

            if let Some(ref created_by) = query.created_by {
                if metadata.created_by.as_ref() != Some(created_by) {
                    continue;
                }
            }

            if !query.tags.is_empty() {
                let has_all_tags = query.tags.iter().all(|tag| metadata.tags.contains(tag));
                if !has_all_tags {
                    continue;
                }
            }

            if let Some(ref parent_id) = query.parent_session_id {
                if metadata.parent_session_id.as_ref() != Some(parent_id) {
                    continue;
                }
            }

            if let Some(created_after) = query.created_after {
                if metadata.created_at < created_after {
                    continue;
                }
            }

            if let Some(created_before) = query.created_before {
                if metadata.created_at > created_before {
                    continue;
                }
            }

            if let Some(ref search_text) = query.search_text {
                let name_match = metadata
                    .name
                    .as_ref()
                    .is_some_and(|n| n.contains(search_text));
                let desc_match = metadata
                    .description
                    .as_ref()
                    .is_some_and(|d| d.contains(search_text));
                if !name_match && !desc_match {
                    continue;
                }
            }

            results.push(metadata.clone());
        }

        // Sort results
        results.sort_by(|a, b| match query.sort_by {
            SessionSortBy::CreatedAt => {
                if query.sort_desc {
                    b.created_at.cmp(&a.created_at)
                } else {
                    a.created_at.cmp(&b.created_at)
                }
            }
            SessionSortBy::UpdatedAt => {
                if query.sort_desc {
                    b.updated_at.cmp(&a.updated_at)
                } else {
                    a.updated_at.cmp(&b.updated_at)
                }
            }
            SessionSortBy::Name => {
                let a_name = a.name.as_deref().unwrap_or("");
                let b_name = b.name.as_deref().unwrap_or("");
                if query.sort_desc {
                    b_name.cmp(a_name)
                } else {
                    a_name.cmp(b_name)
                }
            }
            SessionSortBy::ArtifactCount => {
                if query.sort_desc {
                    b.artifact_count.cmp(&a.artifact_count)
                } else {
                    a.artifact_count.cmp(&b.artifact_count)
                }
            }
            SessionSortBy::OperationCount => {
                if query.sort_desc {
                    b.operation_count.cmp(&a.operation_count)
                } else {
                    a.operation_count.cmp(&b.operation_count)
                }
            }
        });

        // Apply pagination
        if let Some(offset) = query.offset {
            results = results.into_iter().skip(offset).collect();
        }

        if let Some(limit) = query.limit {
            results.truncate(limit);
        }

        Ok(results)
    }

    /// Suspend a session
    ///
    /// # Errors
    ///
    /// Returns an error if the session does not exist or cannot be suspended
    pub async fn suspend_session(&self, session_id: &SessionId) -> Result<()> {
        let session = self.get_session(session_id).await?;

        // Fire pre-suspend hooks
        if self.config.hook_config.enable_lifecycle_hooks {
            let hooks = self.hook_registry.get_hooks(&HookPoint::SessionCheckpoint);
            let component_id = ComponentId::new(
                ComponentType::Custom("SessionManager".to_string()),
                "session-manager".to_string(),
            );

            for hook in hooks {
                let mut context =
                    HookContext::new(HookPoint::SessionCheckpoint, component_id.clone());
                context.data.insert(
                    "session_id".to_string(),
                    serde_json::json!(session_id.to_string()),
                );
                context
                    .data
                    .insert("action".to_string(), serde_json::json!("suspend"));

                if let Err(e) = hook.execute(&mut context).await {
                    warn!("Session suspend hook failed: {e}");
                }
            }
        }

        // Suspend the session
        session.suspend().await?;

        // Save session state
        if self.config.auto_persist {
            self.save_session(&session).await?;
        }

        // Publish event
        if self.config.event_config.enable_session_events {
            let event = UniversalEvent::new(
                "session.suspended",
                serde_json::json!({
                    "session_id": session_id.to_string(),
                }),
                Language::Rust,
            );

            if let Err(e) = self.event_bus.publish(event).await {
                warn!("Failed to publish session suspended event: {e}");
            }
        }

        info!("Suspended session: {session_id}");
        Ok(())
    }

    /// Resume a session
    ///
    /// # Errors
    ///
    /// Returns an error if the session does not exist or cannot be resumed
    pub async fn resume_session(&self, session_id: &SessionId) -> Result<()> {
        let session = self.get_session(session_id).await?;

        // Fire pre-resume hooks
        if self.config.hook_config.enable_lifecycle_hooks {
            let hooks = self.hook_registry.get_hooks(&HookPoint::SessionRestore);
            let component_id = ComponentId::new(
                ComponentType::Custom("SessionManager".to_string()),
                "session-manager".to_string(),
            );

            for hook in hooks {
                let mut context = HookContext::new(HookPoint::SessionRestore, component_id.clone());
                context.data.insert(
                    "session_id".to_string(),
                    serde_json::json!(session_id.to_string()),
                );
                context
                    .data
                    .insert("action".to_string(), serde_json::json!("resume"));

                if let Err(e) = hook.execute(&mut context).await {
                    warn!("Session resume hook failed: {e}");
                }
            }
        }

        // Resume the session
        session.resume().await?;

        // Publish event
        if self.config.event_config.enable_session_events {
            let event = UniversalEvent::new(
                "session.resumed",
                serde_json::json!({
                    "session_id": session_id.to_string(),
                }),
                Language::Rust,
            );

            if let Err(e) = self.event_bus.publish(event).await {
                warn!("Failed to publish session resumed event: {e}");
            }
        }

        info!("Resumed session: {session_id}");
        Ok(())
    }

    /// Complete a session
    ///
    /// # Errors
    ///
    /// Returns an error if the session does not exist or cannot be completed
    pub async fn complete_session(&self, session_id: &SessionId) -> Result<()> {
        let session = self.get_session(session_id).await?;

        // Fire pre-complete hooks
        if self.config.hook_config.enable_lifecycle_hooks {
            let hooks = self.hook_registry.get_hooks(&HookPoint::SessionEnd);
            let component_id = ComponentId::new(
                ComponentType::Custom("SessionManager".to_string()),
                "session-manager".to_string(),
            );

            for hook in hooks {
                let mut context = HookContext::new(HookPoint::SessionEnd, component_id.clone());
                context.data.insert(
                    "session_id".to_string(),
                    serde_json::json!(session_id.to_string()),
                );
                context
                    .data
                    .insert("action".to_string(), serde_json::json!("complete"));

                if let Err(e) = hook.execute(&mut context).await {
                    warn!("Session end hook failed: {e}");
                }
            }
        }

        // Complete the session
        session.complete().await?;

        // Final save
        if self.config.auto_persist {
            self.save_session(&session).await?;
        }

        // Remove from active sessions
        self.active_sessions.write().await.remove(session_id);

        // Publish event
        if self.config.event_config.enable_session_events {
            let event = UniversalEvent::new(
                "session.completed",
                serde_json::json!({
                    "session_id": session_id.to_string(),
                }),
                Language::Rust,
            );

            if let Err(e) = self.event_bus.publish(event).await {
                warn!("Failed to publish session completed event: {e}");
            }
        }

        info!("Completed session: {session_id}");
        Ok(())
    }

    /// Save a session to storage
    ///
    /// # Errors
    ///
    /// Returns an error if serialization or storage fails
    pub async fn save_session(&self, session: &Session) -> Result<()> {
        let snapshot = session.snapshot().await;
        let session_id = snapshot.metadata.id;

        // Fire pre-save hooks
        if self.config.hook_config.enable_lifecycle_hooks {
            let hooks = self.hook_registry.get_hooks(&HookPoint::SessionSave);
            let component_id = ComponentId::new(
                ComponentType::Custom("SessionManager".to_string()),
                "session-manager".to_string(),
            );

            for hook in hooks {
                let mut context = HookContext::new(HookPoint::SessionSave, component_id.clone());
                context.data.insert(
                    "session_id".to_string(),
                    serde_json::json!(session_id.to_string()),
                );
                context
                    .data
                    .insert("action".to_string(), serde_json::json!("save"));

                if let Err(e) = hook.execute(&mut context).await {
                    warn!("Session save hook failed: {e}");
                }
            }
        }

        // Serialize snapshot
        let data = if self.config.enable_compression {
            let serialized = bincode::serialize(&snapshot)?;
            lz4_flex::compress_prepend_size(&serialized)
        } else {
            bincode::serialize(&snapshot)?
        };

        // Store in backend
        let key = format!("session:{session_id}");
        let data_len = data.len();
        self.storage_backend
            .set(&key, data)
            .await
            .map_err(|e| SessionError::Storage(e.to_string()))?;

        debug!("Saved session {session_id} ({data_len} bytes)");
        Ok(())
    }

    /// Load a session from storage
    ///
    /// # Errors
    ///
    /// Returns an error if the session is not found or deserialization fails
    pub async fn load_session(&self, session_id: &SessionId) -> Result<Session> {
        let key = format!("session:{session_id}");

        // Get from storage
        let data = self
            .storage_backend
            .get(&key)
            .await
            .map_err(|e| SessionError::Storage(e.to_string()))?
            .ok_or_else(|| SessionError::SessionNotFound {
                id: session_id.to_string(),
            })?;

        // Deserialize
        let snapshot: SessionSnapshot = if self.config.enable_compression {
            let decompressed = lz4_flex::decompress_size_prepended(&data)
                .map_err(|e| SessionError::Deserialization(e.to_string()))?;
            bincode::deserialize(&decompressed)?
        } else {
            bincode::deserialize(&data)?
        };

        // Check version compatibility
        if snapshot.version > crate::session::SNAPSHOT_VERSION {
            return Err(SessionError::Configuration(format!(
                "Session snapshot version {} is newer than supported version {}",
                snapshot.version,
                crate::session::SNAPSHOT_VERSION
            )));
        }

        // Restore session
        let session = Session::from_snapshot(snapshot);

        // Add to active sessions if not already there
        self.active_sessions
            .write()
            .await
            .entry(*session_id)
            .or_insert(session.clone());

        debug!("Loaded session {session_id} from storage");
        Ok(session)
    }

    /// Delete a session
    ///
    /// # Errors
    ///
    /// Returns an error if storage deletion or state cleanup fails
    pub async fn delete_session(&self, session_id: &SessionId) -> Result<()> {
        // Remove from active sessions
        self.active_sessions.write().await.remove(session_id);

        // Delete from storage
        let key = format!("session:{session_id}");
        self.storage_backend
            .delete(&key)
            .await
            .map_err(|e| SessionError::Storage(e.to_string()))?;

        // Clean up state
        self.state_manager
            .clear_scope(StateScope::Session(session_id.to_string()))
            .await
            .map_err(SessionError::State)?;

        info!("Deleted session: {session_id}");
        Ok(())
    }

    /// Start auto-persist background task
    fn start_auto_persist_task(&self) {
        let manager = self.clone();
        let interval_secs = self.config.persist_interval_secs;

        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(interval_secs));

            loop {
                interval.tick().await;

                if *manager.shutdown.read().await {
                    break;
                }

                let sessions = manager.active_sessions.read().await;
                for session in sessions.values() {
                    if let Err(e) = manager.save_session(session).await {
                        error!("Failed to auto-save session: {e}");
                    }
                }
            }
        });
    }

    /// Start cleanup background task
    fn start_cleanup_task(&self) {
        let manager = self.clone();
        let interval_secs = self.config.cleanup_config.cleanup_interval_secs;

        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(interval_secs));

            loop {
                interval.tick().await;

                if *manager.shutdown.read().await {
                    break;
                }

                if let Err(e) = manager.cleanup_old_sessions().await {
                    error!("Failed to cleanup old sessions: {e}");
                }
            }
        });
    }

    /// Clean up old sessions
    async fn cleanup_old_sessions(&self) -> Result<()> {
        let cutoff = chrono::Utc::now() - self.config.cleanup_config.delete_after;

        let sessions = self.active_sessions.read().await;
        let mut to_delete = Vec::new();

        for session in sessions.values() {
            let metadata = session.metadata.read().await;
            if metadata.status.is_terminal() && metadata.updated_at < cutoff {
                to_delete.push(metadata.id);
            }
        }

        drop(sessions);

        for session_id in to_delete {
            if self.config.cleanup_config.archive_before_delete {
                // TODO: Implement archiving
                debug!("Would archive session {session_id} before deletion");
            }
            self.delete_session(&session_id).await?;
        }

        Ok(())
    }

    /// Save all active sessions
    ///
    /// # Errors
    ///
    /// Returns an error if any session fails to save
    pub async fn save_all_active_sessions(&self) -> Result<()> {
        let sessions = self.active_sessions.read().await;
        let mut errors = Vec::new();

        for session in sessions.values() {
            if let Err(e) = self.save_session(session).await {
                errors.push(format!(
                    "Failed to save session {}: {e}",
                    session.id().await
                ));
            }
        }

        if !errors.is_empty() {
            return Err(SessionError::Storage(errors.join("; ")));
        }

        info!("Saved {} active sessions", sessions.len());
        Ok(())
    }

    /// Restore recent sessions
    ///
    /// # Errors
    ///
    /// Returns an error if session listing or restoration fails
    pub async fn restore_recent_sessions(&self, count: usize) -> Result<Vec<SessionId>> {
        // List all stored sessions
        let prefix = "session:";
        let keys = self
            .storage_backend
            .list_keys(prefix)
            .await
            .map_err(|e| SessionError::Storage(e.to_string()))?;

        // Sort by modification time (newest first)
        let mut session_infos = Vec::new();
        for key in keys {
            if let Some(session_id_str) = key.strip_prefix("session:") {
                if let Ok(session_id) = SessionId::from_str(session_id_str) {
                    // Get metadata to check updated_at
                    match self.load_session(&session_id).await {
                        Ok(session) => {
                            let metadata = session.metadata.read().await;
                            session_infos.push((session_id, metadata.updated_at));
                        }
                        Err(e) => {
                            warn!("Failed to load session metadata for {session_id}: {e}");
                        }
                    }
                }
            }
        }

        // Sort by updated_at (newest first)
        session_infos.sort_by(|a, b| b.1.cmp(&a.1));

        // Restore the most recent sessions
        let mut restored = Vec::new();
        for (session_id, _) in session_infos.into_iter().take(count) {
            match self.load_session(&session_id).await {
                Ok(_) => {
                    restored.push(session_id);
                }
                Err(e) => {
                    warn!("Failed to restore session {session_id}: {e}");
                }
            }
        }

        info!("Restored {} recent sessions", restored.len());
        Ok(restored)
    }

    /// Shutdown the manager
    pub async fn shutdown(&self) {
        *self.shutdown.write().await = true;
        info!("SessionManager shutdown initiated");
    }

    /// Register built-in hooks for session lifecycle events
    fn register_builtin_hooks(registry: &Arc<HookRegistry>) -> Result<()> {
        // Register hooks for all session lifecycle events
        let session_points = vec![
            HookPoint::SessionStart,
            HookPoint::SessionEnd,
            HookPoint::SessionCheckpoint,
            HookPoint::SessionRestore,
            HookPoint::SessionSave,
        ];

        for point in session_points {
            // Register logging hook
            registry
                .register(point.clone(), LoggingHook::new())
                .map_err(|e| {
                    SessionError::Configuration(format!("Failed to register logging hook: {e}"))
                })?;

            // Register metrics hook
            registry.register(point, MetricsHook::new()).map_err(|e| {
                SessionError::Configuration(format!("Failed to register metrics hook: {e}"))
            })?;
        }

        info!("Registered built-in hooks for session lifecycle events");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SessionStatus;
    use llmspell_storage::MemoryBackend;

    async fn create_test_manager() -> SessionManager {
        let state_manager = Arc::new(StateManager::new().await.unwrap());
        let storage_backend = Arc::new(MemoryBackend::new());
        let hook_registry = Arc::new(HookRegistry::new());
        let hook_executor = Arc::new(HookExecutor::new());
        let event_bus = Arc::new(EventBus::new());
        let config = SessionManagerConfig::default();

        SessionManager::new(
            state_manager,
            storage_backend,
            hook_registry,
            hook_executor,
            &event_bus,
            config,
        )
        .unwrap()
    }

    #[tokio::test]
    async fn test_create_session() {
        let manager = create_test_manager().await;

        let options = CreateSessionOptions {
            name: Some("Test Session".to_string()),
            ..Default::default()
        };

        let session_id = manager.create_session(options).await.unwrap();
        assert!(manager.get_session(&session_id).await.is_ok());
    }

    #[tokio::test]
    async fn test_session_lifecycle() {
        let manager = create_test_manager().await;

        // Create session
        let session_id = manager
            .create_session(CreateSessionOptions::default())
            .await
            .unwrap();

        // Suspend
        manager.suspend_session(&session_id).await.unwrap();
        let session = manager.get_session(&session_id).await.unwrap();
        assert_eq!(session.status().await, SessionStatus::Suspended);

        // Resume
        manager.resume_session(&session_id).await.unwrap();
        let session = manager.get_session(&session_id).await.unwrap();
        assert_eq!(session.status().await, SessionStatus::Active);

        // Complete
        manager.complete_session(&session_id).await.unwrap();
        assert!(manager.get_session(&session_id).await.is_err());
    }

    #[tokio::test]
    async fn test_save_load_session() {
        let manager = create_test_manager().await;

        // Create and save session
        let session_id = manager
            .create_session(CreateSessionOptions::default())
            .await
            .unwrap();
        let session = manager.get_session(&session_id).await.unwrap();
        manager.save_session(&session).await.unwrap();

        // Remove from active sessions
        manager.active_sessions.write().await.remove(&session_id);

        // Load it back
        let loaded = manager.load_session(&session_id).await.unwrap();
        assert_eq!(loaded.id().await, session_id);
    }

    #[tokio::test]
    async fn test_list_sessions() {
        let manager = create_test_manager().await;

        // Create multiple sessions
        for i in 0..3 {
            let options = CreateSessionOptions {
                name: Some(format!("Session {}", i)),
                tags: vec!["test".to_string()],
                ..Default::default()
            };
            manager.create_session(options).await.unwrap();
        }

        // Query all sessions
        let query = SessionQuery::default();
        let sessions = manager.list_sessions(query).await.unwrap();
        assert_eq!(sessions.len(), 3);

        // Query with filter
        let query = SessionQuery {
            tags: vec!["test".to_string()],
            ..Default::default()
        };
        let sessions = manager.list_sessions(query).await.unwrap();
        assert_eq!(sessions.len(), 3);
    }
}
