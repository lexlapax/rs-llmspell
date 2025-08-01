//! ABOUTME: Session manager orchestration providing centralized session lifecycle management
//! ABOUTME: Coordinates with state persistence, storage backends, hooks, and events

use crate::{
    artifact::{
        access::AccessType, ArtifactId, ArtifactMetadata, ArtifactQuery, ArtifactStorage,
        ArtifactStorageOps, ArtifactType, SessionArtifact,
    },
    config::SessionManagerConfig,
    events::{create_correlated_event, create_session_event, SessionEventType},
    hooks::{
        register_artifact_collectors, ArtifactCollectionProcessor, CollectorConfig,
        SessionHookContextHelper,
    },
    replay::ReplayEngine,
    security::SessionSecurityManager,
    session::{Session, SessionSnapshot},
    types::{CreateSessionOptions, SessionQuery, SessionSortBy},
    Result, SessionError, SessionId, SessionMetadata,
};
use chrono::{DateTime, Utc};
use llmspell_events::{bus::EventBus, correlation::EventCorrelationTracker};
use llmspell_hooks::{HookExecutor, HookPoint, HookRegistry, LoggingHook, MetricsHook};
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
    /// Artifact collection processor
    #[allow(dead_code)]
    artifact_collector: Option<Arc<ArtifactCollectionProcessor>>,
    /// Manager configuration
    config: SessionManagerConfig,
    /// Session security and isolation manager
    security_manager: Arc<RwLock<SessionSecurityManager>>,
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

        let artifact_storage = Arc::new(ArtifactStorage::with_backend(storage_backend.clone()));

        // Create artifact collection processor if configured
        let artifact_collector = if config.hook_config.enable_artifact_collection {
            let collector_config = CollectorConfig::default();
            register_artifact_collectors(&hook_registry, &collector_config)?;
            Some(Arc::new(ArtifactCollectionProcessor::new(
                artifact_storage.clone(),
                collector_config,
            )))
        } else {
            None
        };

        // Create replay infrastructure components
        let state_storage_adapter = Arc::new(
            llmspell_state_persistence::backend_adapter::StateStorageAdapter::new(
                storage_backend.clone(),
                "sessions".to_string(),
            ),
        );
        let hook_replay_manager =
            Arc::new(llmspell_state_persistence::manager::HookReplayManager::new(
                state_storage_adapter.clone(),
            ));

        // Create bridge adapter for type compatibility
        let hook_replay_bridge = Arc::new(crate::replay::HookReplayBridge::new(
            hook_replay_manager.clone(),
        ));

        // Create hooks storage backend (separate from main storage)
        let hooks_storage_backend =
            Arc::new(llmspell_hooks::persistence::InMemoryStorageBackend::new());

        // Create replay manager with proper storage
        let replay_manager = Arc::new(llmspell_hooks::replay::ReplayManager::new(
            Arc::new(llmspell_hooks::persistence::HookPersistenceManager::new(
                hook_replay_bridge,
            )),
            hooks_storage_backend,
        ));

        // Create replay engine with properly configured storage
        let replay_engine = Arc::new(ReplayEngine::new(
            replay_manager,
            hook_replay_manager,
            storage_backend.clone(),
            event_bus.clone(),
        ));

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
            artifact_storage,
            replay_engine,
            artifact_collector,
            security_manager: Arc::new(RwLock::new(SessionSecurityManager::new(true))), // Strict isolation by default
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

        // Create session created event with correlation
        let session_event = create_session_event(
            session_id,
            SessionEventType::Created,
            serde_json::json!({
                "name": options.name.clone(),
                "created_by": options.created_by.clone(),
                "tags": options.tags.clone(),
            }),
        );

        // Track the event and add correlation context
        self.correlation_tracker
            .track_event(session_event.event.clone());
        self.correlation_tracker
            .add_context(session_event.correlation_context.clone());

        // Store the correlation ID for the session (for replay)
        let session_correlation_key = format!("session_correlation:{}", session_id);
        let correlation_data = serde_json::json!({
            "session_id": session_id.to_string(),
            "correlation_id": session_event.correlation_context.correlation_id.to_string(),
            "created_at": chrono::Utc::now(),
        });
        let correlation_bytes = serde_json::to_vec(&correlation_data).map_err(|e| {
            SessionError::general(format!("Failed to serialize correlation data: {}", e))
        })?;
        self.storage_backend
            .set(&session_correlation_key, correlation_bytes)
            .await
            .map_err(|e| SessionError::Storage(e.to_string()))?;

        // Fire session start hooks
        if self.config.hook_config.enable_lifecycle_hooks {
            let hooks = self.hook_registry.get_hooks(&HookPoint::SessionStart);

            for hook in hooks {
                let mut hook_ctx = SessionHookContextHelper::create_lifecycle_context(
                    HookPoint::SessionStart,
                    &session,
                    "session-manager",
                )
                .await;

                if let Err(e) = hook.execute(&mut hook_ctx).await {
                    warn!("Session start hook failed: {e}");
                }

                // Process any collected artifacts
                if let Some(ref collector) = self.artifact_collector {
                    if ArtifactCollectionProcessor::should_process_hook_point(&hook_ctx.point) {
                        let _ = collector.process_hook_context(&hook_ctx, &session_id).await;
                    }
                }
            }
        }

        // Store in active sessions
        self.active_sessions
            .write()
            .await
            .insert(session_id, session.clone());

        // Register session with security manager
        self.security_manager
            .write()
            .await
            .register_session(&session_id);

        // Initialize state scope
        self.state_manager
            .set(
                StateScope::Session(session_id.to_string()),
                "created_at",
                serde_json::json!(chrono::Utc::now()),
            )
            .await
            .map_err(SessionError::State)?;

        // Publish session created event with correlation
        if self.config.event_config.enable_session_events {
            // Create started event as child of created event
            let started_event = create_correlated_event(
                session_id,
                SessionEventType::Started,
                serde_json::json!({
                    "hook_count": self.hook_registry.get_hooks(&HookPoint::SessionStart).len(),
                }),
                &session_event,
            );

            self.correlation_tracker
                .track_event(started_event.event.clone());

            // Add link between created and started events
            let link = session_event.link_to(
                &started_event,
                llmspell_events::correlation::EventRelationship::CausedBy,
            );
            self.correlation_tracker.add_link(link);

            if let Err(e) = self.event_bus.publish(started_event.event).await {
                warn!("Failed to publish session started event: {e}");
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

            for hook in hooks {
                let mut hook_ctx = SessionHookContextHelper::create_lifecycle_context(
                    HookPoint::SessionCheckpoint,
                    &session,
                    "session-manager",
                )
                .await;
                hook_ctx
                    .data
                    .insert("action".to_string(), serde_json::json!("suspend"));

                if let Err(e) = hook.execute(&mut hook_ctx).await {
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

        // Publish event with correlation
        if self.config.event_config.enable_session_events {
            // Get the session's correlation context
            let context = self
                .correlation_tracker
                .get_context(session_id.as_uuid())
                .unwrap_or_else(|| {
                    llmspell_events::correlation::CorrelationContext::new_root()
                        .with_metadata("session_id", session_id.to_string())
                });

            let suspend_event = crate::events::SessionEvent::new(
                *session_id,
                SessionEventType::Suspended,
                serde_json::json!({
                    "status": session.status().await.to_string(),
                }),
                context,
            );

            self.correlation_tracker
                .track_event(suspend_event.event.clone());

            if let Err(e) = self.event_bus.publish(suspend_event.event).await {
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

            for hook in hooks {
                let mut hook_ctx = SessionHookContextHelper::create_lifecycle_context(
                    HookPoint::SessionRestore,
                    &session,
                    "session-manager",
                )
                .await;
                hook_ctx
                    .data
                    .insert("action".to_string(), serde_json::json!("resume"));

                if let Err(e) = hook.execute(&mut hook_ctx).await {
                    warn!("Session resume hook failed: {e}");
                }
            }
        }

        // Resume the session
        session.resume().await?;

        // Publish event with correlation
        if self.config.event_config.enable_session_events {
            // Get the session's correlation context
            let context = self
                .correlation_tracker
                .get_context(session_id.as_uuid())
                .unwrap_or_else(|| {
                    llmspell_events::correlation::CorrelationContext::new_root()
                        .with_metadata("session_id", session_id.to_string())
                });

            let resume_event = crate::events::SessionEvent::new(
                *session_id,
                SessionEventType::Resumed,
                serde_json::json!({
                    "status": session.status().await.to_string(),
                }),
                context,
            );

            self.correlation_tracker
                .track_event(resume_event.event.clone());

            if let Err(e) = self.event_bus.publish(resume_event.event).await {
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

            for hook in hooks {
                let mut hook_ctx = SessionHookContextHelper::create_lifecycle_context(
                    HookPoint::SessionEnd,
                    &session,
                    "session-manager",
                )
                .await;
                hook_ctx
                    .data
                    .insert("action".to_string(), serde_json::json!("complete"));

                if let Err(e) = hook.execute(&mut hook_ctx).await {
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

        // Publish event with correlation
        if self.config.event_config.enable_session_events {
            // Get the session's correlation context
            let context = self
                .correlation_tracker
                .get_context(session_id.as_uuid())
                .unwrap_or_else(|| {
                    llmspell_events::correlation::CorrelationContext::new_root()
                        .with_metadata("session_id", session_id.to_string())
                });

            let complete_event = crate::events::SessionEvent::new(
                *session_id,
                SessionEventType::Completed,
                serde_json::json!({
                    "duration_ms": session.metadata.read().await.duration()
                        .map_or(0, |d| d.num_milliseconds()),
                    "artifact_count": session.metadata.read().await.artifact_count,
                    "operation_count": session.metadata.read().await.operation_count,
                }),
                context,
            );

            self.correlation_tracker
                .track_event(complete_event.event.clone());

            if let Err(e) = self.event_bus.publish(complete_event.event).await {
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

            for hook in hooks {
                let mut hook_ctx = SessionHookContextHelper::create_lifecycle_context(
                    HookPoint::SessionSave,
                    session,
                    "session-manager",
                )
                .await;
                hook_ctx
                    .data
                    .insert("action".to_string(), serde_json::json!("save"));

                if let Err(e) = hook.execute(&mut hook_ctx).await {
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

        // Store session metadata for replay (JSON format)
        // This includes the correlation_id for session replay functionality
        let session_correlation_key = format!("session_correlation:{}", session_id);
        if let Ok(Some(correlation_bytes)) =
            self.storage_backend.get(&session_correlation_key).await
        {
            if let Ok(correlation_data) =
                serde_json::from_slice::<serde_json::Value>(&correlation_bytes)
            {
                if let Some(correlation_id_str) = correlation_data
                    .get("correlation_id")
                    .and_then(|v| v.as_str())
                {
                    let metadata_key = format!("session_metadata:{}", session_id);
                    let metadata = serde_json::json!({
                        "id": session_id.to_string(),
                        "name": snapshot.metadata.name,
                        "status": snapshot.metadata.status,
                        "correlation_id": correlation_id_str,
                        "created_at": snapshot.metadata.created_at,
                        "updated_at": snapshot.metadata.updated_at,
                    });
                    let metadata_bytes = serde_json::to_vec(&metadata)?;
                    self.storage_backend
                        .set(&metadata_key, metadata_bytes)
                        .await
                        .map_err(|e| SessionError::Storage(e.to_string()))?;
                }
            }
        }

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

        // Unregister from security manager
        self.security_manager
            .write()
            .await
            .unregister_session(session_id);

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

        // Clean up session replay metadata for deleted sessions
        self.cleanup_session_replay_metadata(cutoff).await?;

        Ok(())
    }

    /// Clean up old session replay metadata
    async fn cleanup_session_replay_metadata(&self, cutoff: DateTime<Utc>) -> Result<()> {
        // List all session_metadata keys
        let prefix = "session_metadata:";
        let keys = self
            .storage_backend
            .list_keys(prefix)
            .await
            .map_err(|e| SessionError::Storage(e.to_string()))?;

        let mut to_delete = Vec::new();

        for key in keys {
            // Load metadata to check updated_at
            if let Ok(Some(metadata_bytes)) = self.storage_backend.get(&key).await {
                if let Ok(metadata) = serde_json::from_slice::<serde_json::Value>(&metadata_bytes) {
                    if let Some(updated_at_str) =
                        metadata.get("updated_at").and_then(|v| v.as_str())
                    {
                        if let Ok(updated_at) = DateTime::parse_from_rfc3339(updated_at_str) {
                            let updated_at_utc = updated_at.with_timezone(&Utc);
                            if updated_at_utc < cutoff {
                                to_delete.push(key);
                            }
                        }
                    }
                }
            }
        }

        // Delete old metadata
        let to_delete_count = to_delete.len();
        for key in to_delete {
            if let Err(e) = self.storage_backend.delete(&key).await {
                warn!("Failed to delete old session metadata {}: {}", key, e);
            }
        }

        if to_delete_count > 0 {
            info!(
                "Cleaned up {} old session replay metadata entries",
                to_delete_count
            );
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

    // ===== Public Artifact API =====

    /// Store a user-provided artifact in a session
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Session not found
    /// - Artifact exceeds size limits
    /// - Storage backend fails
    /// - Artifact validation fails
    pub async fn store_artifact(
        &self,
        session_id: &SessionId,
        artifact_type: ArtifactType,
        name: String,
        content: Vec<u8>,
        metadata: Option<HashMap<String, serde_json::Value>>,
    ) -> Result<ArtifactId> {
        // Verify session exists and is active
        let session = self.get_session(session_id).await?;
        let status = session.status().await;
        if !status.is_active() {
            return Err(SessionError::InvalidOperation {
                reason: format!("Cannot store artifacts in {status:?} session"),
            });
        }

        // Get next sequence number for this session
        let sequence = session.increment_operation_count().await?;

        // Create the artifact with metadata
        let mut artifact = SessionArtifact::create_with_metadata(
            *session_id,
            sequence,
            artifact_type.clone(),
            name.clone(),
            content,
            None, // created_by will be set by session metadata if available
        )?;

        // Add custom metadata if provided
        if let Some(custom_metadata) = metadata {
            for (key, value) in custom_metadata {
                match key.as_str() {
                    // Special handling for mime_type - set it on the artifact metadata directly
                    "mime_type" => {
                        if let Some(mime_type_str) = value.as_str() {
                            artifact.metadata.mime_type = mime_type_str.to_string();
                        }
                    }
                    // Special handling for tags - set them on the artifact metadata directly
                    "tags" => {
                        if let Some(tags_array) = value.as_array() {
                            artifact.metadata.tags = tags_array
                                .iter()
                                .filter_map(|v| v.as_str().map(String::from))
                                .collect();
                        }
                    }
                    // Everything else goes to custom metadata
                    _ => {
                        artifact.metadata.custom.insert(key, value);
                    }
                }
            }
        }

        // Store the artifact
        let artifact_id = self.artifact_storage.store_artifact(&artifact).await?;

        // Update session metadata
        session.increment_artifact_count().await?;

        // Fire artifact creation hooks if enabled
        if self.config.hook_config.enable_artifact_collection {
            let hooks = self.hook_registry.get_hooks(&HookPoint::AfterToolExecution);

            for hook in hooks {
                let mut hook_ctx = SessionHookContextHelper::create_artifact_context(
                    &session,
                    "store_artifact",
                    &artifact_id.to_string(),
                    "session-manager",
                )
                .await;
                hook_ctx.data.insert(
                    "artifact_type".to_string(),
                    serde_json::json!(artifact_type.to_string()),
                );
                hook_ctx
                    .data
                    .insert("name".to_string(), serde_json::json!(name));
                hook_ctx.data.insert(
                    "size".to_string(),
                    serde_json::json!(artifact.metadata.size),
                );

                if let Err(e) = hook.execute(&mut hook_ctx).await {
                    warn!("Artifact storage hook failed: {e}");
                }
            }
        }

        // Publish event with correlation
        if self.config.event_config.enable_session_events {
            // Get the session's correlation context
            let correlation_ctx = self
                .correlation_tracker
                .get_context(session_id.as_uuid())
                .unwrap_or_else(|| {
                    llmspell_events::correlation::CorrelationContext::new_root()
                        .with_metadata("session_id", session_id.to_string())
                        .with_tag("artifact_operation")
                });

            let artifact_event = crate::events::SessionEvent::new(
                *session_id,
                SessionEventType::ArtifactStored,
                serde_json::json!({
                    "artifact_id": artifact_id.to_string(),
                    "artifact_type": artifact_type.to_string(),
                    "name": name,
                    "size": artifact.metadata.size,
                }),
                correlation_ctx,
            );

            self.correlation_tracker
                .track_event(artifact_event.event.clone());

            if let Err(e) = self.event_bus.publish(artifact_event.event).await {
                warn!("Failed to publish artifact stored event: {e}");
            }
        }

        info!("Stored user artifact {artifact_id} in session {session_id}");
        Ok(artifact_id)
    }

    /// Retrieve an artifact by ID
    ///
    /// # Errors
    ///
    /// Returns an error if the artifact is not found or retrieval fails
    pub async fn get_artifact(
        &self,
        session_id: &SessionId,
        artifact_id: &ArtifactId,
    ) -> Result<SessionArtifact> {
        // Check access permission
        let has_permission = self
            .artifact_storage
            .access_control_manager()
            .check_permission(artifact_id, session_id, AccessType::Read)
            .await?;

        if !has_permission {
            return Err(SessionError::AccessDenied {
                message: format!(
                    "Session {} does not have read permission for artifact {}",
                    session_id,
                    artifact_id.storage_key()
                ),
            });
        }

        // Retrieve the artifact
        self.artifact_storage
            .get_artifact(artifact_id)
            .await?
            .ok_or_else(|| SessionError::ArtifactNotFound {
                id: artifact_id.to_string(),
            })
    }

    /// Retrieve artifact content only (without metadata)
    ///
    /// # Errors
    ///
    /// Returns an error if the artifact is not found or content retrieval fails
    pub async fn get_artifact_content(
        &self,
        session_id: &SessionId,
        artifact_id: &ArtifactId,
    ) -> Result<Vec<u8>> {
        let artifact = self.get_artifact(session_id, artifact_id).await?;
        artifact.get_content()
    }

    /// List all artifacts for a session
    ///
    /// # Errors
    ///
    /// Returns an error if the session is not found or listing fails
    pub async fn list_artifacts(&self, session_id: &SessionId) -> Result<Vec<ArtifactMetadata>> {
        // Verify session exists
        self.get_session(session_id).await?;

        // For now, only allow sessions to list their own artifacts
        // Cross-session access control for listing will be implemented in a future enhancement
        self.artifact_storage
            .list_session_artifacts(session_id)
            .await
    }

    /// Delete an artifact
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Artifact not found
    /// - Session is not active
    /// - Deletion fails
    pub async fn delete_artifact(
        &self,
        requesting_session_id: &SessionId,
        artifact_id: &ArtifactId,
    ) -> Result<()> {
        // Check access permission for deletion (requires admin access)
        let has_permission = self
            .artifact_storage
            .access_control_manager()
            .check_permission(artifact_id, requesting_session_id, AccessType::Delete)
            .await?;

        if !has_permission {
            return Err(SessionError::AccessDenied {
                message: format!(
                    "Session {} does not have delete permission for artifact {}",
                    requesting_session_id,
                    artifact_id.storage_key()
                ),
            });
        }

        // Verify session exists and is active
        let session = self.get_session(&artifact_id.session_id).await?;
        let status = session.status().await;
        if !status.is_active() {
            return Err(SessionError::InvalidOperation {
                reason: format!("Cannot delete artifacts from {status:?} session"),
            });
        }

        // Delete the artifact
        let deleted = self.artifact_storage.delete_artifact(artifact_id).await?;
        if !deleted {
            return Err(SessionError::ArtifactNotFound {
                id: artifact_id.to_string(),
            });
        }

        // Update session metadata
        session.decrement_artifact_count().await?;

        // Publish event with correlation
        if self.config.event_config.enable_session_events {
            // Get the session's correlation context
            let artifact_context = self
                .correlation_tracker
                .get_context(artifact_id.session_id.as_uuid())
                .unwrap_or_else(|| {
                    llmspell_events::correlation::CorrelationContext::new_root()
                        .with_metadata("session_id", artifact_id.session_id.to_string())
                        .with_tag("artifact_operation")
                });

            let delete_event = crate::events::SessionEvent::new(
                artifact_id.session_id,
                SessionEventType::ArtifactDeleted,
                serde_json::json!({
                    "artifact_id": artifact_id.to_string(),
                }),
                artifact_context,
            );

            self.correlation_tracker
                .track_event(delete_event.event.clone());

            if let Err(e) = self.event_bus.publish(delete_event.event).await {
                warn!("Failed to publish artifact deleted event: {e}");
            }
        }

        info!("Deleted artifact {artifact_id}");
        Ok(())
    }

    /// Query artifacts with filtering and pagination
    ///
    /// # Errors
    ///
    /// Returns an error if the query fails
    pub async fn query_artifacts(&self, query: ArtifactQuery) -> Result<Vec<ArtifactMetadata>> {
        // For now, querying is limited to the session specified in the query
        // Cross-session access control for querying will be implemented in a future enhancement
        self.artifact_storage.query_artifacts(query).await
    }

    /// Grant permission for another session to access an artifact
    ///
    /// # Arguments
    ///
    /// * `granting_session_id` - The session granting the permission (must have admin access)
    /// * `artifact_id` - The artifact to grant access to
    /// * `target_session_id` - The session to grant access to
    /// * `permission` - The level of permission to grant
    ///
    /// # Errors
    ///
    /// Returns an error if the granting session doesn't have admin access or the operation fails
    pub async fn grant_artifact_permission(
        &self,
        granting_session_id: &SessionId,
        artifact_id: &ArtifactId,
        target_session_id: SessionId,
        permission: crate::artifact::access::Permission,
    ) -> Result<()> {
        self.artifact_storage
            .access_control_manager()
            .grant_permission(
                artifact_id,
                target_session_id,
                permission,
                *granting_session_id,
            )
            .await
    }

    /// Revoke permission for another session to access an artifact
    ///
    /// # Arguments
    ///
    /// * `revoking_session_id` - The session revoking the permission (must have admin access)
    /// * `artifact_id` - The artifact to revoke access from
    /// * `target_session_id` - The session to revoke access from
    ///
    /// # Errors
    ///
    /// Returns an error if the revoking session doesn't have admin access or the operation fails
    pub async fn revoke_artifact_permission(
        &self,
        revoking_session_id: &SessionId,
        artifact_id: &ArtifactId,
        target_session_id: &SessionId,
    ) -> Result<()> {
        self.artifact_storage
            .access_control_manager()
            .revoke_permission(artifact_id, target_session_id, *revoking_session_id)
            .await
    }

    /// Get access control list for an artifact
    ///
    /// # Arguments
    ///
    /// * `requesting_session_id` - The session requesting the ACL (must have admin access)
    /// * `artifact_id` - The artifact to get the ACL for
    ///
    /// # Errors
    ///
    /// Returns an error if the requesting session doesn't have admin access
    pub async fn get_artifact_acl(
        &self,
        requesting_session_id: &SessionId,
        artifact_id: &ArtifactId,
    ) -> Result<crate::artifact::access::AccessControlList> {
        // Check if the requesting session has admin access to view the ACL
        let has_permission = self
            .artifact_storage
            .access_control_manager()
            .check_permission(
                artifact_id,
                requesting_session_id,
                AccessType::ChangePermissions,
            )
            .await?;

        if !has_permission {
            return Err(SessionError::AccessDenied {
                message: format!(
                    "Session {} does not have permission to view ACL for artifact {}",
                    requesting_session_id,
                    artifact_id.storage_key()
                ),
            });
        }

        self.artifact_storage
            .access_control_manager()
            .get_acl(artifact_id)
            .await
    }

    /// Get audit log for an artifact
    ///
    /// # Arguments
    ///
    /// * `requesting_session_id` - The session requesting the audit log (must have admin access)
    /// * `artifact_id` - The artifact to get the audit log for
    ///
    /// # Errors
    ///
    /// Returns an error if the requesting session doesn't have admin access
    pub async fn get_artifact_audit_log(
        &self,
        requesting_session_id: &SessionId,
        artifact_id: &ArtifactId,
    ) -> Result<Vec<crate::artifact::access::AccessAuditEntry>> {
        // Check if the requesting session has admin access to view the audit log
        let has_permission = self
            .artifact_storage
            .access_control_manager()
            .check_permission(
                artifact_id,
                requesting_session_id,
                AccessType::ChangePermissions,
            )
            .await?;

        if !has_permission {
            return Err(SessionError::AccessDenied {
                message: format!(
                    "Session {} does not have permission to view audit log for artifact {}",
                    requesting_session_id,
                    artifact_id.storage_key()
                ),
            });
        }

        Ok(self
            .artifact_storage
            .access_control_manager()
            .get_audit_log(artifact_id)
            .await)
    }

    /// Store a file as an artifact
    ///
    /// # Errors
    ///
    /// Returns an error if file reading or storage fails
    pub async fn store_file_artifact(
        &self,
        session_id: &SessionId,
        file_path: &std::path::Path,
        artifact_type: ArtifactType,
        metadata: Option<HashMap<String, serde_json::Value>>,
    ) -> Result<ArtifactId> {
        // Read file content
        let content = tokio::fs::read(file_path).await.map_err(|e| {
            SessionError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to read file: {e}"),
            ))
        })?;

        // Use file name as artifact name
        let name = file_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unnamed")
            .to_string();

        // Store as artifact
        self.store_artifact(session_id, artifact_type, name, content, metadata)
            .await
    }

    // MARK: - Replay Methods

    /// Get the replay engine for direct access
    pub fn replay_engine(&self) -> &Arc<ReplayEngine> {
        &self.replay_engine
    }

    /// Check if a session can be replayed
    pub async fn can_replay_session(&self, session_id: &SessionId) -> Result<bool> {
        self.replay_engine.can_replay_session(session_id).await
    }

    /// Replay a session using the existing replay infrastructure
    pub async fn replay_session(
        &self,
        session_id: &SessionId,
        config: crate::replay::session_adapter::SessionReplayConfig,
    ) -> Result<crate::replay::session_adapter::SessionReplayResult> {
        self.replay_engine.replay_session(session_id, config).await
    }

    /// Get the timeline of events for a session
    pub async fn get_session_timeline(
        &self,
        session_id: &SessionId,
    ) -> Result<Vec<llmspell_state_persistence::manager::SerializedHookExecution>> {
        self.replay_engine.get_session_timeline(session_id).await
    }

    /// Get replay status for a session
    pub fn get_replay_status(
        &self,
        session_id: &SessionId,
    ) -> Option<crate::replay::session_adapter::SessionReplayStatus> {
        self.replay_engine.get_replay_status(session_id)
    }

    /// Stop session replay
    pub fn stop_replay(&self, session_id: &SessionId) -> Result<()> {
        self.replay_engine.stop_replay(session_id)
    }

    /// Get all active replays
    pub fn get_all_active_replays(
        &self,
    ) -> Vec<crate::replay::session_adapter::SessionReplayStatus> {
        self.replay_engine.get_all_active_replays()
    }

    /// Query hook executions for a specific session
    pub async fn query_session_hooks(
        &self,
        session_id: &SessionId,
        filter: crate::replay::session_adapter::SessionHookFilter,
    ) -> Result<Vec<llmspell_state_persistence::manager::SerializedHookExecution>> {
        self.replay_engine
            .query_session_hooks(session_id, filter)
            .await
    }

    /// Get session replay metadata
    pub async fn get_session_replay_metadata(
        &self,
        session_id: &SessionId,
    ) -> Result<crate::replay::session_adapter::SessionReplayMetadata> {
        self.replay_engine
            .get_session_replay_metadata(session_id)
            .await
    }

    /// List all sessions that can be replayed
    pub async fn list_replayable_sessions(&self) -> Result<Vec<SessionId>> {
        self.replay_engine.list_replayable_sessions().await
    }
}

#[cfg(test)]
#[cfg_attr(test_category = "session")]
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

    #[cfg_attr(test_category = "unit")]
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

    #[cfg_attr(test_category = "unit")]
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

    #[cfg_attr(test_category = "unit")]
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

    #[cfg_attr(test_category = "unit")]
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

    #[cfg_attr(test_category = "unit")]
    #[tokio::test]
    async fn test_user_artifact_storage() {
        let manager = create_test_manager().await;

        // Create a session
        let session_id = manager
            .create_session(CreateSessionOptions::default())
            .await
            .unwrap();

        // Store a user artifact
        let content = b"Hello, this is my data!".to_vec();
        let artifact_id = manager
            .store_artifact(
                &session_id,
                ArtifactType::UserInput,
                "my_data.txt".to_string(),
                content.clone(),
                None,
            )
            .await
            .unwrap();

        // Retrieve the artifact
        let artifact = manager
            .get_artifact(&session_id, &artifact_id)
            .await
            .unwrap();
        assert_eq!(artifact.metadata.name, "my_data.txt");
        assert_eq!(artifact.metadata.artifact_type, ArtifactType::UserInput);
        assert_eq!(artifact.get_content().unwrap(), content);

        // List artifacts
        let artifacts = manager.list_artifacts(&session_id).await.unwrap();
        assert_eq!(artifacts.len(), 1);
        assert_eq!(artifacts[0].name, "my_data.txt");

        // Delete the artifact
        manager
            .delete_artifact(&session_id, &artifact_id)
            .await
            .unwrap();

        // Verify it's deleted
        let result = manager.get_artifact(&session_id, &artifact_id).await;
        assert!(result.is_err());
    }

    #[cfg_attr(test_category = "unit")]
    #[tokio::test]
    async fn test_artifact_with_metadata() {
        let manager = create_test_manager().await;

        // Create a session
        let session_id = manager
            .create_session(CreateSessionOptions::default())
            .await
            .unwrap();

        // Create custom metadata
        let mut metadata = HashMap::new();
        metadata.insert("author".to_string(), serde_json::json!("John Doe"));
        metadata.insert("version".to_string(), serde_json::json!("1.0.0"));
        metadata.insert(
            "tags".to_string(),
            serde_json::json!(["important", "dataset"]),
        );

        // Store artifact with metadata
        let content = b"Dataset content".to_vec();
        let artifact_id = manager
            .store_artifact(
                &session_id,
                ArtifactType::UserInput,
                "dataset.csv".to_string(),
                content,
                Some(metadata.clone()),
            )
            .await
            .unwrap();

        // Retrieve and verify metadata
        let artifact = manager
            .get_artifact(&session_id, &artifact_id)
            .await
            .unwrap();
        assert_eq!(
            artifact.metadata.custom.get("author").unwrap(),
            &serde_json::json!("John Doe")
        );
        assert_eq!(
            artifact.metadata.custom.get("version").unwrap(),
            &serde_json::json!("1.0.0")
        );
    }

    #[cfg_attr(test_category = "unit")]
    #[tokio::test]
    async fn test_store_file_artifact() {
        let manager = create_test_manager().await;

        // Create a temporary file
        let temp_dir = tempfile::tempdir().unwrap();
        let file_path = temp_dir.path().join("test_file.json");
        let content = r#"{"key": "value", "number": 42}"#;
        std::fs::write(&file_path, content).unwrap();

        // Create a session
        let session_id = manager
            .create_session(CreateSessionOptions::default())
            .await
            .unwrap();

        // Store file as artifact
        let artifact_id = manager
            .store_file_artifact(&session_id, &file_path, ArtifactType::UserInput, None)
            .await
            .unwrap();

        // Retrieve and verify
        let artifact = manager
            .get_artifact(&session_id, &artifact_id)
            .await
            .unwrap();
        assert_eq!(artifact.metadata.name, "test_file.json");
        assert_eq!(artifact.get_content().unwrap(), content.as_bytes());
    }

    #[cfg_attr(test_category = "unit")]
    #[tokio::test]
    async fn test_artifact_operations_on_inactive_session() {
        let manager = create_test_manager().await;

        // Create and suspend a session
        let session_id = manager
            .create_session(CreateSessionOptions::default())
            .await
            .unwrap();
        manager.suspend_session(&session_id).await.unwrap();

        // Try to store artifact - should fail
        let result = manager
            .store_artifact(
                &session_id,
                ArtifactType::UserInput,
                "test.txt".to_string(),
                b"content".to_vec(),
                None,
            )
            .await;
        assert!(result.is_err());

        // Resume session and try again
        manager.resume_session(&session_id).await.unwrap();
        let artifact_id = manager
            .store_artifact(
                &session_id,
                ArtifactType::UserInput,
                "test.txt".to_string(),
                b"content".to_vec(),
                None,
            )
            .await
            .unwrap();

        // Suspend again and try to delete - should fail
        manager.suspend_session(&session_id).await.unwrap();
        let result = manager.delete_artifact(&session_id, &artifact_id).await;
        assert!(result.is_err());
    }

    #[cfg_attr(test_category = "unit")]
    #[tokio::test]
    async fn test_query_artifacts() {
        let manager = create_test_manager().await;

        // Create a session
        let session_id = manager
            .create_session(CreateSessionOptions::default())
            .await
            .unwrap();

        // Store multiple artifacts
        for i in 0..3 {
            manager
                .store_artifact(
                    &session_id,
                    ArtifactType::UserInput,
                    format!("file_{}.txt", i),
                    format!("content {}", i).into_bytes(),
                    None,
                )
                .await
                .unwrap();
        }

        // Query artifacts
        let query = ArtifactQuery {
            session_id: Some(session_id),
            artifact_type: Some(ArtifactType::UserInput),
            ..Default::default()
        };
        let artifacts = manager.query_artifacts(query).await.unwrap();
        assert_eq!(artifacts.len(), 3);
    }
}
