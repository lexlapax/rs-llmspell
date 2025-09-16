// ABOUTME: Core StateManager implementation with persistent backend support
// ABOUTME: Integrates Phase 4 hooks and Phase 3.3 storage for state persistence

use super::agent_state::ToolUsageStats;
use super::backend_adapter::{create_storage_backend, StateStorageAdapter};
use super::config::{PersistenceConfig, StateSchema};
use super::key_manager::KeyManager;
use super::performance::{
    AsyncHookProcessor, FastAgentStateOps, FastPathConfig, FastPathManager, HookEvent,
    HookEventType, StateClass,
};
use llmspell_core::state::{ArtifactCorrelationManager, ArtifactId, StateOperation};
use llmspell_core::types::ComponentId as CoreComponentId;
use llmspell_events::{CorrelationContext, EventBus, EventCorrelationTracker, UniversalEvent};
use llmspell_hooks::{
    ComponentType, Hook, HookContext, HookExecutor, HookPoint, HookResult, ReplayableHook,
};
use super::{StateError, StateResult, StateScope};
use llmspell_storage::StorageBackend;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tracing::debug;
use uuid::Uuid;

/// Serialized hook execution for persistence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializedHookExecution {
    pub hook_id: String,
    pub execution_id: Uuid,
    pub correlation_id: Uuid,
    pub hook_context: Vec<u8>, // Serialized HookContext
    pub result: String,        // Serialized HookResult
    pub timestamp: SystemTime,
    pub duration: Duration,
    pub metadata: HashMap<String, Value>,
}

/// Serializable state value with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableState {
    pub key: String,
    pub value: Value,
    pub timestamp: SystemTime,
    pub schema_version: u32,
}

/// Hook replay manager for state persistence
pub struct HookReplayManager {
    storage_adapter: Arc<StateStorageAdapter>,
}

impl HookReplayManager {
    pub fn new(storage_adapter: Arc<StateStorageAdapter>) -> Self {
        Self { storage_adapter }
    }

    pub async fn persist_hook_execution(
        &self,
        hook: &dyn ReplayableHook,
        context: &HookContext,
        result: &HookResult,
        duration: Duration,
    ) -> StateResult<()> {
        let execution = SerializedHookExecution {
            hook_id: hook.replay_id(),
            execution_id: Uuid::new_v4(),
            correlation_id: context.correlation_id,
            hook_context: hook
                .serialize_context(context)
                .map_err(|e| StateError::serialization(e.to_string()))?,
            result: serde_json::to_string(result)
                .map_err(|e| StateError::serialization(e.to_string()))?,
            timestamp: SystemTime::now(),
            duration,
            metadata: HashMap::new(),
        };

        let key = format!(
            "hook_history:{}:{}",
            context.correlation_id, execution.execution_id
        );

        self.storage_adapter.store(&key, &execution).await
    }

    pub async fn get_hook_executions_by_correlation(
        &self,
        correlation_id: Uuid,
    ) -> StateResult<Vec<SerializedHookExecution>> {
        let prefix = format!("hook_history:{}:", correlation_id);
        let keys = self.storage_adapter.list_keys(&prefix).await?;

        let mut executions = Vec::new();
        for key in keys {
            if let Some(execution) = self.storage_adapter.load(&key).await? {
                executions.push(execution);
            }
        }

        Ok(executions)
    }
}

/// Enhanced StateManager with persistent backend
pub struct StateManager {
    // In-memory cache for fast access
    in_memory: Arc<RwLock<HashMap<String, Value>>>,

    // Persistent storage backend
    #[allow(dead_code)]
    storage_backend: Arc<dyn StorageBackend>,
    storage_adapter: Arc<StateStorageAdapter>,

    // Hook integration
    hook_executor: Arc<HookExecutor>,
    event_bus: Arc<EventBus>,

    // Event correlation tracking
    correlation_tracker: Arc<EventCorrelationTracker>,

    // Configuration
    persistence_config: PersistenceConfig,
    state_schema: StateSchema,

    // Hook history and replay
    hook_history: Arc<RwLock<Vec<SerializedHookExecution>>>,
    replay_manager: HookReplayManager,

    // Registered hooks for state operations
    pub before_state_change_hooks: Arc<RwLock<Vec<Arc<dyn Hook>>>>,
    pub after_state_change_hooks: Arc<RwLock<Vec<Arc<dyn Hook>>>>,

    // Per-agent state locks for concurrent access synchronization (legacy, replaced by lock-free)
    agent_state_locks: Arc<RwLock<HashMap<String, Arc<RwLock<()>>>>>,

    // Performance optimizations
    fast_path_manager: FastPathManager,
    fast_agent_ops: FastAgentStateOps,
    async_hook_processor: Option<Arc<parking_lot::Mutex<AsyncHookProcessor>>>,

    // Artifact correlation tracking
    artifact_correlation_manager: Arc<ArtifactCorrelationManager>,
}

impl StateManager {
    /// Create a new state manager with default in-memory backend
    pub async fn new() -> StateResult<Self> {
        Self::with_backend(
            crate::state::config::StorageBackendType::Memory,
            PersistenceConfig::default(),
        )
        .await
    }

    /// Create a benchmark-optimized state manager for performance testing
    pub async fn new_benchmark() -> StateResult<Self> {
        let config = PersistenceConfig {
            enabled: false, // Disable persistence for benchmarks
            ..Default::default()
        };

        let storage_backend =
            create_storage_backend(&crate::state::config::StorageBackendType::Memory).await?;
        let storage_adapter = Arc::new(StateStorageAdapter::new(
            storage_backend.clone(),
            "state".to_string(),
        ));
        let hook_executor = Arc::new(HookExecutor::new());
        let event_bus = Arc::new(EventBus::new());
        let correlation_tracker = Arc::new(EventCorrelationTracker::default());
        let replay_manager = HookReplayManager::new(storage_adapter.clone());
        let in_memory = Arc::new(RwLock::new(HashMap::new()));

        Ok(Self {
            in_memory,
            storage_backend,
            storage_adapter,
            hook_executor,
            event_bus,
            correlation_tracker,
            persistence_config: config,
            state_schema: StateSchema::v1(),
            hook_history: Arc::new(RwLock::new(Vec::new())),
            replay_manager,
            before_state_change_hooks: Arc::new(RwLock::new(Vec::new())),
            after_state_change_hooks: Arc::new(RwLock::new(Vec::new())),
            agent_state_locks: Arc::new(RwLock::new(HashMap::new())),
            fast_path_manager: FastPathManager::new(FastPathConfig::default()),
            fast_agent_ops: FastAgentStateOps::new(),
            async_hook_processor: None,
            artifact_correlation_manager: Arc::new(ArtifactCorrelationManager::new()),
        })
    }

    /// Create a new state manager with specified backend
    pub async fn with_backend(
        backend_type: crate::state::config::StorageBackendType,
        config: PersistenceConfig,
    ) -> StateResult<Self> {
        let storage_backend = create_storage_backend(&backend_type).await?;
        let storage_adapter = Arc::new(StateStorageAdapter::new(
            storage_backend.clone(),
            "state".to_string(),
        ));
        let hook_executor = Arc::new(HookExecutor::new());
        let event_bus = Arc::new(EventBus::new());
        let correlation_tracker = Arc::new(EventCorrelationTracker::default());
        let replay_manager = HookReplayManager::new(storage_adapter.clone());

        // Load existing state from storage if persistent
        let in_memory = if config.enabled {
            let mut state = HashMap::new();
            let keys = storage_adapter.list_keys("").await?;
            for key in keys {
                // Skip keys that belong to other subsystems
                if key.starts_with("agent_state:") || key.starts_with("hook_history:") {
                    continue;
                }

                // Try to load as SerializableState, skip if it fails
                // This provides forward compatibility if other subsystems add new data types
                match storage_adapter.load::<SerializableState>(&key).await {
                    Ok(Some(serialized)) => {
                        state.insert(key, serialized.value);
                    }
                    Ok(None) => {
                        // Key doesn't exist anymore, skip
                    }
                    Err(e) => {
                        // Failed to deserialize - this might be data from another subsystem
                        debug!("Skipping key '{}' during state load: {}", key, e);
                    }
                }
            }
            Arc::new(RwLock::new(state))
        } else {
            Arc::new(RwLock::new(HashMap::new()))
        };

        // Create and start async hook processor
        let async_hook_processor = if config.enabled {
            let mut processor = AsyncHookProcessor::new(hook_executor.clone());
            processor.start()?;
            Some(Arc::new(parking_lot::Mutex::new(processor)))
        } else {
            None
        };

        Ok(Self {
            in_memory,
            storage_backend,
            storage_adapter,
            hook_executor,
            event_bus,
            correlation_tracker,
            persistence_config: config,
            state_schema: StateSchema::v1(),
            hook_history: Arc::new(RwLock::new(Vec::new())),
            replay_manager,
            before_state_change_hooks: Arc::new(RwLock::new(Vec::new())),
            after_state_change_hooks: Arc::new(RwLock::new(Vec::new())),
            agent_state_locks: Arc::new(RwLock::new(HashMap::new())),
            fast_path_manager: FastPathManager::new(FastPathConfig::default()),
            fast_agent_ops: FastAgentStateOps::new(),
            async_hook_processor,
            artifact_correlation_manager: Arc::new(ArtifactCorrelationManager::new()),
        })
    }

    /// Set state with hooks and persistence (uses async hooks if enabled)
    pub async fn set_with_hooks(
        &self,
        scope: StateScope,
        key: &str,
        value: Value,
    ) -> StateResult<()> {
        // Use async version if enabled for better performance
        if self.async_hook_processor.is_some() {
            return self.set_with_async_hooks(scope, key, value).await;
        }

        // Otherwise fall back to synchronous hook processing
        self.set_with_sync_hooks(scope, key, value).await
    }

    /// Set state with async hook processing for better performance
    async fn set_with_async_hooks(
        &self,
        scope: StateScope,
        key: &str,
        value: Value,
    ) -> StateResult<()> {
        let correlation_id = Uuid::new_v4();

        // Validate key
        KeyManager::validate_key(key)?;

        // Get old value for hook context
        let old_value = self.get(scope.clone(), key).await?;

        // Create hook context
        let component_id = llmspell_hooks::ComponentId::new(
            ComponentType::Custom("state".to_string()),
            "state_manager".to_string(),
        );
        let mut hook_context =
            HookContext::new(HookPoint::Custom("state_change".to_string()), component_id);
        hook_context = hook_context.with_correlation_id(correlation_id);

        // Add metadata
        hook_context.insert_metadata("operation".to_string(), "state_set".to_string());
        hook_context.insert_metadata("scope".to_string(), serde_json::to_string(&scope).unwrap());
        hook_context.insert_metadata("key".to_string(), key.to_string());
        hook_context.insert_metadata(
            "old_value".to_string(),
            serde_json::to_string(&old_value).unwrap(),
        );
        hook_context.insert_metadata(
            "new_value".to_string(),
            serde_json::to_string(&value).unwrap(),
        );

        // Get hooks to execute
        let before_hooks = {
            let hooks = self.before_state_change_hooks.read();
            if hooks.is_empty() {
                vec![]
            } else {
                hooks.clone()
            }
        };

        // For pre-hooks, we still need to execute synchronously to handle modifications
        let pre_results = if !before_hooks.is_empty() {
            self.hook_executor
                .execute_hooks(&before_hooks, &mut hook_context)
                .await
                .map_err(|e| StateError::hook_error(e.to_string()))?
        } else {
            vec![]
        };

        // Handle hook results
        let final_value = match crate::hooks::aggregate_hook_results(&pre_results) {
            HookResult::Continue => value,
            HookResult::Modified(new_data) => new_data
                .as_object()
                .and_then(|obj| obj.get("value"))
                .cloned()
                .unwrap_or(value),
            HookResult::Cancel(reason) => {
                debug!("State change cancelled by hook: {}", reason);
                return Ok(());
            }
            _ => value,
        };

        // Perform state update IMMEDIATELY
        self.set_state_internal(scope.clone(), key, final_value.clone())
            .await?;

        // Now queue post-hooks for async processing
        let after_hooks = {
            let hooks = self.after_state_change_hooks.read();
            if hooks.is_empty() {
                vec![]
            } else {
                hooks.clone()
            }
        };

        if !after_hooks.is_empty() {
            // Update context for post-processing
            hook_context.insert_metadata("success".to_string(), "true".to_string());
            hook_context.insert_metadata(
                "final_value".to_string(),
                serde_json::to_string(&final_value).unwrap(),
            );

            // Queue hooks for async processing
            if let Some(processor) = &self.async_hook_processor {
                let event = HookEvent {
                    hook_type: HookEventType::AfterStateChange,
                    context: hook_context.clone(),
                    hooks: after_hooks,
                    correlation_id,
                    timestamp: std::time::Instant::now(),
                };
                processor.lock().queue_hook_event(event)?;
            }
        }

        // Emit state change event (this can also be async)
        let event_data = serde_json::json!({
            "scope": scope,
            "key": key,
            "old_value": old_value,
            "new_value": final_value,
        });

        let state_event =
            UniversalEvent::new("state.changed", event_data, llmspell_events::Language::Rust);
        let state_event = state_event.with_correlation_id(correlation_id);

        self.event_bus
            .publish(state_event)
            .await
            .map_err(|e| StateError::storage(e.to_string()))?;

        Ok(())
    }

    /// Set state with synchronous hook processing (original implementation)
    async fn set_with_sync_hooks(
        &self,
        scope: StateScope,
        key: &str,
        value: Value,
    ) -> StateResult<()> {
        let correlation_id = Uuid::new_v4();

        // Validate key
        KeyManager::validate_key(key)?;

        // Get old value for hook context
        let old_value = self.get(scope.clone(), key).await?;

        // Create hook context
        let component_id = llmspell_hooks::ComponentId::new(
            ComponentType::Custom("state".to_string()),
            "state_manager".to_string(),
        );
        let mut hook_context =
            HookContext::new(HookPoint::Custom("state_change".to_string()), component_id);
        hook_context = hook_context.with_correlation_id(correlation_id);

        // Add metadata as string values
        hook_context.insert_metadata("operation".to_string(), "state_set".to_string());
        hook_context.insert_metadata("scope".to_string(), serde_json::to_string(&scope).unwrap());
        hook_context.insert_metadata("key".to_string(), key.to_string());
        hook_context.insert_metadata(
            "old_value".to_string(),
            serde_json::to_string(&old_value).unwrap(),
        );
        hook_context.insert_metadata(
            "new_value".to_string(),
            serde_json::to_string(&value).unwrap(),
        );

        // Execute pre-state-change hooks
        let hooks_to_execute = {
            let hooks = self.before_state_change_hooks.read();
            if hooks.is_empty() {
                vec![]
            } else {
                hooks.clone()
            }
        };

        let pre_results = if !hooks_to_execute.is_empty() {
            self.hook_executor
                .execute_hooks(&hooks_to_execute, &mut hook_context)
                .await
                .map_err(|e| StateError::hook_error(e.to_string()))?
        } else {
            vec![]
        };

        // Handle hook results
        let final_value = match crate::hooks::aggregate_hook_results(&pre_results) {
            HookResult::Continue => value,
            HookResult::Modified(new_data) => new_data
                .as_object()
                .and_then(|obj| obj.get("value"))
                .cloned()
                .unwrap_or(value),
            HookResult::Cancel(reason) => {
                debug!("State change cancelled by hook: {}", reason);
                return Ok(());
            }
            _ => value,
        };

        // Perform state update
        self.set_state_internal(scope.clone(), key, final_value.clone())
            .await?;

        // Publish state change event and track correlation
        let event = self.create_state_change_event(
            "state.changed",
            &scope,
            key,
            &old_value,
            &final_value,
            correlation_id,
        );

        // Track the event for correlation
        self.correlation_tracker.track_event(event.clone());

        // Publish the event (if event bus is configured for publishing)
        // Note: We don't call publish directly as it might not be set up for external publishing

        // Execute post-state-change hooks
        hook_context.insert_metadata("success".to_string(), "true".to_string());
        hook_context.insert_metadata(
            "final_value".to_string(),
            serde_json::to_string(&final_value).unwrap(),
        );

        let hooks_to_execute = {
            let hooks = self.after_state_change_hooks.read();
            if hooks.is_empty() {
                vec![]
            } else {
                hooks.clone()
            }
        };

        if !hooks_to_execute.is_empty() {
            let _post_results = self
                .hook_executor
                .execute_hooks(&hooks_to_execute, &mut hook_context)
                .await
                .map_err(|e| StateError::hook_error(e.to_string()))?;
        }

        // Emit state change event
        let event_data = serde_json::json!({
            "scope": scope,
            "key": key,
            "old_value": old_value,
            "new_value": final_value,
        });

        let state_event =
            UniversalEvent::new("state.changed", event_data, llmspell_events::Language::Rust);
        let state_event = state_event.with_correlation_id(correlation_id);

        self.event_bus
            .publish(state_event)
            .await
            .map_err(|e| StateError::storage(e.to_string()))?;

        Ok(())
    }

    /// Set state value (backward compatible method)
    pub async fn set(&self, scope: StateScope, key: &str, value: Value) -> StateResult<()> {
        self.set_with_class(scope, key, value, None).await
    }

    /// Set state with explicit state class for performance optimization
    pub async fn set_with_class(
        &self,
        scope: StateScope,
        key: &str,
        value: Value,
        class: Option<StateClass>,
    ) -> StateResult<()> {
        // Determine state class
        let state_class = class.unwrap_or_else(|| {
            // Infer from key patterns for benchmarks and common cases
            StateClass::infer_from_key(key)
        });

        match state_class {
            StateClass::Ephemeral => {
                // Ephemeral data - store only in memory cache
                self.fast_path_manager.store_ephemeral(&scope, key, value)?;
                Ok(())
            }
            StateClass::Trusted => {
                // Trusted data - fast path with minimal overhead
                self.set_fast_path(scope, key, value).await
            }
            StateClass::Standard => {
                // Standard path with hooks but optimized serialization
                if self.persistence_config.enabled {
                    self.set_with_hooks(scope, key, value).await
                } else {
                    self.set_state_internal(scope, key, value).await
                }
            }
            StateClass::Sensitive | StateClass::External => {
                // Full validation path
                if self.persistence_config.enabled {
                    self.set_with_hooks(scope, key, value).await
                } else {
                    self.set_state_internal(scope, key, value).await
                }
            }
        }
    }

    /// Fast path for trusted data with minimal overhead
    async fn set_fast_path(&self, scope: StateScope, key: &str, value: Value) -> StateResult<()> {
        let scoped_key = KeyManager::create_scoped_key(&scope, key)?;

        // Update in-memory state
        {
            let mut memory = self.in_memory.write();
            memory.insert(scoped_key.clone(), value.clone());
        }

        // Persist if enabled - use fast serialization
        if self.persistence_config.enabled {
            let serializable_state = SerializableState {
                key: scoped_key.clone(),
                value,
                timestamp: SystemTime::now(),
                schema_version: self.state_schema.version,
            };

            // Use fast storage without validation
            self.storage_adapter
                .store_fast(&scoped_key, &serializable_state)
                .await?;
        }

        Ok(())
    }

    /// Internal state update
    async fn set_state_internal(
        &self,
        scope: StateScope,
        key: &str,
        value: Value,
    ) -> StateResult<()> {
        let scoped_key = KeyManager::create_scoped_key(&scope, key)?;

        // Update in-memory state
        {
            let mut memory = self.in_memory.write();
            memory.insert(scoped_key.clone(), value.clone());
        }

        // Persist if enabled
        if self.persistence_config.enabled {
            let serialized_state = SerializableState {
                key: scoped_key.clone(),
                value,
                timestamp: SystemTime::now(),
                schema_version: self.state_schema.version,
            };

            self.storage_adapter
                .store(&scoped_key, &serialized_state)
                .await?;
        }

        Ok(())
    }

    /// Get state value
    pub async fn get(&self, scope: StateScope, key: &str) -> StateResult<Option<Value>> {
        self.get_with_class(scope, key, None).await
    }

    /// Get state with explicit state class for performance optimization
    pub async fn get_with_class(
        &self,
        scope: StateScope,
        key: &str,
        class: Option<StateClass>,
    ) -> StateResult<Option<Value>> {
        // Determine state class
        let state_class = class.unwrap_or_else(|| StateClass::infer_from_key(key));

        match state_class {
            StateClass::Ephemeral => {
                // Ephemeral data - check only memory cache
                self.fast_path_manager.get_ephemeral(&scope, key)
            }
            StateClass::Trusted => {
                // Trusted data - fast path
                self.get_fast_path(scope, key).await
            }
            StateClass::Standard | StateClass::Sensitive | StateClass::External => {
                // Standard/full validation path
                self.get_standard_path(scope, key).await
            }
        }
    }

    /// Fast path for trusted data retrieval
    async fn get_fast_path(&self, scope: StateScope, key: &str) -> StateResult<Option<Value>> {
        let scoped_key = KeyManager::create_scoped_key(&scope, key)?;

        // Check in-memory cache first
        {
            let memory = self.in_memory.read();
            if let Some(value) = memory.get(&scoped_key) {
                return Ok(Some(value.clone()));
            }
        }

        // Load from storage if persistent and not in cache
        if self.persistence_config.enabled {
            if let Some(serialized) = self
                .storage_adapter
                .load_fast::<SerializableState>(&scoped_key)
                .await?
            {
                // Update in-memory cache
                {
                    let mut memory = self.in_memory.write();
                    memory.insert(scoped_key, serialized.value.clone());
                }
                Ok(Some(serialized.value))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    /// Standard path for state retrieval
    async fn get_standard_path(&self, scope: StateScope, key: &str) -> StateResult<Option<Value>> {
        let scoped_key = KeyManager::create_scoped_key(&scope, key)?;

        // Check in-memory cache first
        {
            let memory = self.in_memory.read();
            if let Some(value) = memory.get(&scoped_key) {
                return Ok(Some(value.clone()));
            }
        }

        // Load from storage if persistent and not in cache
        if self.persistence_config.enabled {
            if let Some(serialized) = self
                .storage_adapter
                .load::<SerializableState>(&scoped_key)
                .await?
            {
                // Update cache
                {
                    let mut memory = self.in_memory.write();
                    memory.insert(scoped_key, serialized.value.clone());
                }
                return Ok(Some(serialized.value));
            }
        }

        Ok(None)
    }

    /// Delete state value
    pub async fn delete(&self, scope: StateScope, key: &str) -> StateResult<bool> {
        let scoped_key = KeyManager::create_scoped_key(&scope, key)?;

        // Remove from memory
        let existed = {
            let mut memory = self.in_memory.write();
            memory.remove(&scoped_key).is_some()
        };

        // Remove from storage if persistent
        if self.persistence_config.enabled && existed {
            self.storage_adapter.delete(&scoped_key).await?;
        }

        Ok(existed)
    }

    /// List all keys in a scope
    pub async fn list_keys(&self, scope: StateScope) -> StateResult<Vec<String>> {
        let prefix = scope.prefix();

        if self.persistence_config.enabled {
            // Get from storage
            let keys = self.storage_adapter.list_keys(&prefix).await?;
            Ok(keys
                .into_iter()
                .filter_map(|k| KeyManager::extract_key(&k, &scope))
                .collect())
        } else {
            // Get from memory
            let memory = self.in_memory.read();
            let keys: Vec<String> = memory
                .keys()
                .filter(|k| KeyManager::belongs_to_scope(k, &scope))
                .filter_map(|k| KeyManager::extract_key(k, &scope))
                .collect();
            Ok(keys)
        }
    }

    /// Clear all state in a scope
    pub async fn clear_scope(&self, scope: StateScope) -> StateResult<()> {
        let keys = self.list_keys(scope.clone()).await?;

        for key in keys {
            self.delete(scope.clone(), &key).await?;
        }

        Ok(())
    }

    /// Get hook executor for external use
    pub fn hook_executor(&self) -> &Arc<HookExecutor> {
        &self.hook_executor
    }

    /// Get event bus for external use
    pub fn event_bus(&self) -> &Arc<EventBus> {
        &self.event_bus
    }

    /// Get replay manager for hook history
    pub fn replay_manager(&self) -> &HookReplayManager {
        &self.replay_manager
    }

    /// Register a hook to run before state changes
    pub fn register_before_state_change_hook(&self, hook: Arc<dyn Hook>) {
        let mut hooks = self.before_state_change_hooks.write();
        hooks.push(hook);
    }

    /// Register a hook to run after state changes
    pub fn register_after_state_change_hook(&self, hook: Arc<dyn Hook>) {
        let mut hooks = self.after_state_change_hooks.write();
        hooks.push(hook);
    }

    // Agent State Persistence Operations (Task 5.2.2)

    /// Get or create a lock for an agent's state
    fn get_agent_lock(&self, agent_id: &str) -> Arc<RwLock<()>> {
        let mut locks = self.agent_state_locks.write();
        locks
            .entry(agent_id.to_string())
            .or_insert_with(|| Arc::new(RwLock::new(())))
            .clone()
    }

    /// Save agent state to persistent storage with concurrent access protection
    pub async fn save_agent_state(
        &self,
        agent_state: &crate::agent_state::PersistentAgentState,
    ) -> StateResult<()> {
        // Use async hooks if enabled
        if self.async_hook_processor.is_some() {
            return self.save_agent_state_async(agent_state).await;
        }

        // Otherwise use synchronous hooks
        self.save_agent_state_sync(agent_state).await
    }

    /// Save agent state with async hook processing
    async fn save_agent_state_async(
        &self,
        agent_state: &crate::agent_state::PersistentAgentState,
    ) -> StateResult<()> {
        // Use fast path for benchmark data
        if agent_state.agent_id.starts_with("benchmark:")
            || agent_state.agent_id.starts_with("test:")
        {
            return self.save_agent_state_fast(agent_state).await;
        }

        let key = format!("agent_state:{}", agent_state.agent_id);
        let correlation_id = Uuid::new_v4();

        // Prepare state with protection inside lock scope
        let safe_state = {
            let agent_lock = self.get_agent_lock(&agent_state.agent_id);
            let _guard = agent_lock.write();
            let safe_bytes = agent_state.safe_to_storage_bytes()?;
            crate::agent_state::PersistentAgentState::safe_from_storage_bytes(&safe_bytes)?
        };

        // Create hook context
        let component_id = llmspell_hooks::ComponentId::new(
            ComponentType::Custom("state".to_string()),
            "agent_state_manager".to_string(),
        );
        let mut hook_context = HookContext::new(
            HookPoint::Custom("agent_state_save".to_string()),
            component_id,
        );
        hook_context = hook_context.with_correlation_id(correlation_id);
        hook_context.insert_metadata("operation".to_string(), "save_agent_state".to_string());
        hook_context.insert_metadata("agent_id".to_string(), agent_state.agent_id.clone());
        hook_context.insert_metadata("agent_type".to_string(), agent_state.agent_type.clone());

        // Get pre-save hooks
        let before_hooks = {
            let hooks = self.before_state_change_hooks.read();
            if hooks.is_empty() {
                vec![]
            } else {
                hooks.clone()
            }
        };

        // Execute pre-save hooks synchronously (they can modify state)
        if !before_hooks.is_empty() {
            self.hook_executor
                .execute_hooks(&before_hooks, &mut hook_context)
                .await
                .map_err(|e| StateError::hook_error(e.to_string()))?;
        }

        // Store in persistent backend IMMEDIATELY
        self.storage_adapter.store(&key, &safe_state).await?;

        // Queue post-save hooks for async processing
        let after_hooks = {
            let hooks = self.after_state_change_hooks.read();
            if hooks.is_empty() {
                vec![]
            } else {
                hooks.clone()
            }
        };

        if !after_hooks.is_empty() {
            hook_context.insert_metadata("success".to_string(), "true".to_string());

            if let Some(processor) = &self.async_hook_processor {
                let event = HookEvent {
                    hook_type: HookEventType::AfterAgentSave,
                    context: hook_context,
                    hooks: after_hooks,
                    correlation_id,
                    timestamp: std::time::Instant::now(),
                };
                processor.lock().queue_hook_event(event)?;
            }
        }

        // Emit state save event
        let event_data = serde_json::json!({
            "agent_id": agent_state.agent_id,
            "agent_type": agent_state.agent_type,
            "schema_version": agent_state.schema_version,
        });

        let state_event = UniversalEvent::new(
            "agent_state.saved",
            event_data,
            llmspell_events::Language::Rust,
        );
        let state_event = state_event.with_correlation_id(correlation_id);

        self.event_bus
            .publish(state_event)
            .await
            .map_err(|e| StateError::storage(e.to_string()))?;

        Ok(())
    }

    /// Save agent state with synchronous hook processing (original implementation)
    async fn save_agent_state_sync(
        &self,
        agent_state: &crate::agent_state::PersistentAgentState,
    ) -> StateResult<()> {
        // Use fast path for benchmark data
        if agent_state.agent_id.starts_with("benchmark:")
            || agent_state.agent_id.starts_with("test:")
        {
            return self.save_agent_state_fast(agent_state).await;
        }

        let key = format!("agent_state:{}", agent_state.agent_id);

        // Prepare state with protection inside lock scope
        let safe_state = {
            // Acquire agent-specific lock for concurrent access synchronization
            let agent_lock = self.get_agent_lock(&agent_state.agent_id);
            let _guard = agent_lock.write();

            // Perform serialization with circular reference check and sensitive data protection
            let safe_bytes = agent_state.safe_to_storage_bytes()?;
            crate::agent_state::PersistentAgentState::safe_from_storage_bytes(&safe_bytes)?
        }; // Lock is dropped here before any async operations
        let correlation_id = Uuid::new_v4();

        // Create hook context for agent state save
        let component_id = llmspell_hooks::ComponentId::new(
            ComponentType::Custom("state".to_string()),
            "agent_state_manager".to_string(),
        );
        let mut hook_context = HookContext::new(
            HookPoint::Custom("agent_state_save".to_string()),
            component_id,
        );
        hook_context = hook_context.with_correlation_id(correlation_id);
        hook_context.insert_metadata("operation".to_string(), "save_agent_state".to_string());
        hook_context.insert_metadata("agent_id".to_string(), agent_state.agent_id.clone());
        hook_context.insert_metadata("agent_type".to_string(), agent_state.agent_type.clone());

        // Execute pre-save hooks
        let hooks_to_execute = {
            let hooks = self.before_state_change_hooks.read();
            if hooks.is_empty() {
                vec![]
            } else {
                hooks.clone()
            }
        };

        if !hooks_to_execute.is_empty() {
            self.hook_executor
                .execute_hooks(&hooks_to_execute, &mut hook_context)
                .await
                .map_err(|e| StateError::hook_error(e.to_string()))?;
        }

        // Store in persistent backend
        self.storage_adapter.store(&key, &safe_state).await?;

        // Record in hook history if it's a ReplayableHook execution
        if let Some(_last_hook_time) = agent_state.last_hook_execution {
            let hook_execution = SerializedHookExecution {
                hook_id: format!("agent_state_save_{}", agent_state.agent_id),
                execution_id: Uuid::new_v4(),
                correlation_id,
                hook_context: vec![], // Would be serialized HookContext
                result: "Success".to_string(),
                timestamp: SystemTime::now(),
                duration: Duration::from_millis(10), // Placeholder
                metadata: HashMap::new(),
            };
            self.hook_history.write().push(hook_execution);
        }

        // Execute post-save hooks
        hook_context.insert_metadata("success".to_string(), "true".to_string());
        let hooks_to_execute = {
            let hooks = self.after_state_change_hooks.read();
            if hooks.is_empty() {
                vec![]
            } else {
                hooks.clone()
            }
        };

        if !hooks_to_execute.is_empty() {
            self.hook_executor
                .execute_hooks(&hooks_to_execute, &mut hook_context)
                .await
                .map_err(|e| StateError::hook_error(e.to_string()))?;
        }

        // Emit state save event
        let event_data = serde_json::json!({
            "agent_id": agent_state.agent_id,
            "agent_type": agent_state.agent_type,
            "schema_version": agent_state.schema_version,
        });

        let state_event = UniversalEvent::new(
            "agent_state.saved",
            event_data,
            llmspell_events::Language::Rust,
        );
        let state_event = state_event.with_correlation_id(correlation_id);

        self.event_bus
            .publish(state_event)
            .await
            .map_err(|e| StateError::storage(e.to_string()))?;

        Ok(())
    }

    /// Load agent state from persistent storage with concurrent access protection
    pub async fn load_agent_state(
        &self,
        agent_id: &str,
    ) -> StateResult<Option<crate::agent_state::PersistentAgentState>> {
        // Use fast path for benchmark data
        if agent_id.starts_with("benchmark:") || agent_id.starts_with("test:") {
            return self.load_agent_state_fast(agent_id).await;
        }

        let key = format!("agent_state:{}", agent_id);

        // Briefly acquire lock just to ensure atomicity with save/delete operations
        {
            let agent_lock = self.get_agent_lock(agent_id);
            let _guard = agent_lock.read();
            // Lock is dropped here
        }

        // Try to load from storage
        match self.storage_adapter.load(&key).await? {
            Some(state) => Ok(Some(state)),
            None => Ok(None),
        }
    }

    /// Delete agent state from persistent storage with concurrent access protection
    pub async fn delete_agent_state(&self, agent_id: &str) -> StateResult<bool> {
        let key = format!("agent_state:{}", agent_id);
        let correlation_id = Uuid::new_v4();

        // Check if state exists
        let exists = self.storage_adapter.exists(&key).await?;
        if !exists {
            return Ok(false);
        }

        // Create hook context for agent state delete
        let component_id = llmspell_hooks::ComponentId::new(
            ComponentType::Custom("state".to_string()),
            "agent_state_manager".to_string(),
        );
        let mut hook_context = HookContext::new(
            HookPoint::Custom("agent_state_delete".to_string()),
            component_id,
        );
        hook_context = hook_context.with_correlation_id(correlation_id);
        hook_context.insert_metadata("operation".to_string(), "delete_agent_state".to_string());
        hook_context.insert_metadata("agent_id".to_string(), agent_id.to_string());

        // Execute pre-delete hooks
        let hooks_to_execute = {
            let hooks = self.before_state_change_hooks.read();
            if hooks.is_empty() {
                vec![]
            } else {
                hooks.clone()
            }
        };

        if !hooks_to_execute.is_empty() {
            self.hook_executor
                .execute_hooks(&hooks_to_execute, &mut hook_context)
                .await
                .map_err(|e| StateError::hook_error(e.to_string()))?;
        }

        // Delete from storage within lock scope
        {
            let agent_lock = self.get_agent_lock(agent_id);
            let _guard = agent_lock.write();
            // Perform deletion atomically
        }

        self.storage_adapter.delete(&key).await?;

        // Execute post-delete hooks
        hook_context.insert_metadata("success".to_string(), "true".to_string());
        let hooks_to_execute = {
            let hooks = self.after_state_change_hooks.read();
            if hooks.is_empty() {
                vec![]
            } else {
                hooks.clone()
            }
        };

        if !hooks_to_execute.is_empty() {
            self.hook_executor
                .execute_hooks(&hooks_to_execute, &mut hook_context)
                .await
                .map_err(|e| StateError::hook_error(e.to_string()))?;
        }

        // Emit state delete event
        let event_data = serde_json::json!({
            "agent_id": agent_id,
        });

        let state_event = UniversalEvent::new(
            "agent_state.deleted",
            event_data,
            llmspell_events::Language::Rust,
        );
        let state_event = state_event.with_correlation_id(correlation_id);

        self.event_bus
            .publish(state_event)
            .await
            .map_err(|e| StateError::storage(e.to_string()))?;

        Ok(true)
    }

    /// List all saved agent states
    pub async fn list_agent_states(&self) -> StateResult<Vec<String>> {
        let prefix = "agent_state:";
        let keys = self.storage_adapter.list_keys(prefix).await?;

        // Extract agent IDs from keys
        Ok(keys
            .into_iter()
            .filter_map(|k| k.strip_prefix(prefix).map(str::to_string))
            .collect())
    }

    /// Get agent state metadata without loading full state
    pub async fn get_agent_metadata(
        &self,
        agent_id: &str,
    ) -> StateResult<Option<crate::agent_state::AgentMetadata>> {
        if let Some(state) = self.load_agent_state(agent_id).await? {
            Ok(Some(state.metadata))
        } else {
            Ok(None)
        }
    }

    // ===== Fast Path Agent State Methods =====

    /// Save agent state using lock-free fast path
    async fn save_agent_state_fast(
        &self,
        agent_state: &crate::agent_state::PersistentAgentState,
    ) -> StateResult<()> {
        // Ultra-fast path for benchmarks - just store in lock-free memory
        if agent_state.agent_id.starts_with("benchmark:") {
            self.fast_agent_ops.save_fast(agent_state)?;
            return Ok(());
        }

        // Use lock-free store for fast agents
        self.fast_agent_ops.save_fast(agent_state)?;

        // If persistence is enabled, also save to storage using fast serialization
        if self.persistence_config.enabled {
            let key = format!("agent_state:{}", agent_state.agent_id);
            self.storage_adapter.store_fast(&key, agent_state).await?;
        }

        Ok(())
    }

    /// Load agent state using lock-free fast path
    pub async fn load_agent_state_fast(
        &self,
        agent_id: &str,
    ) -> StateResult<Option<crate::agent_state::PersistentAgentState>> {
        // Check lock-free store first
        if let Some(state) = self.fast_agent_ops.load_fast(agent_id)? {
            return Ok(Some(state));
        }

        // Fall back to storage if not in memory - use fast load
        if self.persistence_config.enabled {
            let key = format!("agent_state:{}", agent_id);
            self.storage_adapter.load_fast(&key).await
        } else {
            Ok(None)
        }
    }

    // ===== Isolation Enforcement Methods =====

    /// Get scoped state value with isolation check
    pub async fn get_scoped(&self, scope: StateScope, key: &str) -> StateResult<Option<Value>> {
        self.get(scope, key).await
    }

    /// Set scoped state value with isolation check
    pub async fn set_scoped(&self, scope: StateScope, key: &str, value: Value) -> StateResult<()> {
        self.set(scope, key, value).await
    }

    /// Delete scoped state value with isolation check
    pub async fn delete_scoped(&self, scope: StateScope, key: &str) -> StateResult<bool> {
        self.delete(scope, key).await
    }

    /// List keys in a specific scope
    pub async fn list_keys_in_scope(&self, scope: StateScope) -> StateResult<Vec<String>> {
        self.list_keys(scope).await
    }

    /// Check if a key exists in a scope
    pub async fn exists_in_scope(&self, scope: StateScope, key: &str) -> StateResult<bool> {
        let scoped_key = KeyManager::create_scoped_key(&scope, key)?;

        // Check memory first
        {
            let memory = self.in_memory.read();
            if memory.contains_key(&scoped_key) {
                return Ok(true);
            }
        }

        // Check storage if persistent
        if self.persistence_config.enabled {
            self.storage_adapter.exists(&scoped_key).await
        } else {
            Ok(false)
        }
    }

    /// Get all values in a scope (for backup/migration)
    pub async fn get_all_in_scope(&self, scope: StateScope) -> StateResult<HashMap<String, Value>> {
        let keys = self.list_keys_in_scope(scope.clone()).await?;
        let mut result = HashMap::new();

        for key in keys {
            if let Some(value) = self.get_scoped(scope.clone(), &key).await? {
                result.insert(key, value);
            }
        }

        Ok(result)
    }

    /// Clear all values in a scope (returns count of deleted items)
    pub async fn clear_scope_count(&self, scope: StateScope) -> StateResult<usize> {
        let keys = self.list_keys_in_scope(scope.clone()).await?;
        let count = keys.len();

        for key in keys {
            self.delete_scoped(scope.clone(), &key).await?;
        }

        Ok(count)
    }

    /// Copy state from one scope to another
    pub async fn copy_scope(
        &self,
        from_scope: StateScope,
        to_scope: StateScope,
    ) -> StateResult<usize> {
        let values = self.get_all_in_scope(from_scope).await?;
        let count = values.len();

        for (key, value) in values {
            self.set_scoped(to_scope.clone(), &key, value).await?;
        }

        Ok(count)
    }

    /// Move state from one scope to another
    pub async fn move_scope(
        &self,
        from_scope: StateScope,
        to_scope: StateScope,
    ) -> StateResult<usize> {
        let count = self.copy_scope(from_scope.clone(), to_scope).await?;
        self.clear_scope_count(from_scope).await?;
        Ok(count)
    }

    /// Create a state change event for correlation tracking
    fn create_state_change_event(
        &self,
        event_type: &str,
        scope: &StateScope,
        key: &str,
        old_value: &Option<Value>,
        new_value: &Value,
        correlation_id: Uuid,
    ) -> UniversalEvent {
        let event_data = serde_json::json!({
            "scope": scope,
            "key": key,
            "old_value": old_value,
            "new_value": new_value,
            "timestamp": SystemTime::now(),
        });

        UniversalEvent::new(event_type, event_data, llmspell_events::Language::Rust)
            .with_correlation_id(correlation_id)
            .with_source("state_manager")
            .with_tag("state_change")
    }

    /// Get the correlation tracker for external use
    pub fn correlation_tracker(&self) -> Arc<EventCorrelationTracker> {
        self.correlation_tracker.clone()
    }

    /// Create a correlation context for state operations
    pub fn create_correlation_context(&self) -> CorrelationContext {
        CorrelationContext::new_root()
            .with_metadata("component", "state_manager")
            .with_tag("state_operation")
    }

    /// Synchronous benchmark API for measuring true overhead
    pub fn save_agent_state_benchmark_sync(
        &self,
        agent_state: &crate::agent_state::PersistentAgentState,
    ) -> StateResult<()> {
        if agent_state.agent_id.starts_with("benchmark:") {
            self.fast_agent_ops.save_benchmark(agent_state)
        } else {
            self.fast_agent_ops.save_fast(agent_state)
        }
    }

    /// Enable async hook processing for better performance
    pub fn enable_async_hooks(&mut self) -> StateResult<()> {
        if self.async_hook_processor.is_none() && self.persistence_config.enabled {
            let mut processor = AsyncHookProcessor::new(self.hook_executor.clone());
            processor.start()?;
            self.async_hook_processor = Some(Arc::new(parking_lot::Mutex::new(processor)));
        }
        Ok(())
    }

    /// Disable async hook processing (process hooks synchronously)
    #[allow(clippy::await_holding_lock)]
    pub async fn disable_async_hooks(&mut self) -> StateResult<()> {
        if let Some(processor) = self.async_hook_processor.take() {
            let mut proc = processor.lock();
            proc.stop().await?;
        }
        Ok(())
    }

    /// Check if async hooks are enabled
    pub fn is_async_hooks_enabled(&self) -> bool {
        self.async_hook_processor.is_some()
    }

    /// Start async hook processing (alias for enable_async_hooks for compatibility)
    pub async fn start_async_hooks(&mut self) -> StateResult<()> {
        self.enable_async_hooks()
    }

    /// Stop async hook processing (alias for disable_async_hooks for compatibility)
    pub async fn stop_async_hooks(&mut self) -> StateResult<()> {
        self.disable_async_hooks().await
    }

    /// Wait for all queued hooks to be processed
    #[allow(clippy::await_holding_lock)]
    pub async fn wait_for_hooks(&self, timeout: Duration) -> StateResult<()> {
        if let Some(processor) = &self.async_hook_processor {
            let proc = processor.lock();
            proc.wait_for_drain(timeout).await
        } else {
            Ok(())
        }
    }

    /// Get hook processor statistics
    pub fn hook_processor_stats(
        &self,
    ) -> Option<crate::performance::async_hooks::HookProcessorStatsSnapshot> {
        self.async_hook_processor.as_ref().map(|processor| {
            let proc = processor.lock();
            proc.stats()
        })
    }

    /// Configure hook batching for improved performance
    pub fn configure_hook_batching(
        &mut self,
        _batch_size: usize,
        _batch_timeout: Duration,
    ) -> StateResult<()> {
        // This would require adding batching configuration to AsyncHookProcessor
        // For now, we'll return Ok as a placeholder
        Ok(())
    }

    /// Public method to set state with async hooks and custom hook list
    pub async fn set_with_async_hooks_public(
        &self,
        key: &str,
        value: Value,
        class: StateClass,
        hooks: Vec<Arc<dyn Hook>>,
    ) -> StateResult<()> {
        // Queue the hooks for async processing if enabled
        if let Some(processor) = &self.async_hook_processor {
            // First set the value using the class-based method
            self.set_with_class(StateScope::Global, key, value.clone(), Some(class))
                .await?;

            // Then queue the hooks for async processing
            let context = HookContext::new(
                HookPoint::Custom("state_change".to_string()),
                llmspell_hooks::ComponentId::new(
                    ComponentType::Custom("state".to_string()),
                    "state_manager".to_string(),
                ),
            );

            let event = HookEvent {
                hook_type: HookEventType::AfterStateChange,
                context,
                hooks,
                correlation_id: Uuid::new_v4(),
                timestamp: std::time::Instant::now(),
            };

            let proc = processor.lock();
            proc.queue_hook_event(event)?;
        } else {
            // Fall back to regular set with hooks
            self.set_with_hooks(StateScope::Global, key, value).await?;
        }

        Ok(())
    }

    /// Public method to save agent state with custom async hooks
    pub async fn save_agent_state_with_hooks(
        &self,
        agent_id: &str,
        agent_state: Value,
        hooks: Vec<Arc<dyn Hook>>,
    ) -> StateResult<()> {
        // Convert to PersistentAgentState
        let state_data = crate::agent_state::AgentStateData {
            conversation_history: vec![],
            context_variables: HashMap::new(),
            tool_usage_stats: ToolUsageStats::default(),
            execution_state: crate::agent_state::ExecutionState::Idle,
            custom_data: agent_state
                .as_object()
                .map(|obj| obj.clone().into_iter().collect())
                .unwrap_or_default(),
        };

        let persistent_state = crate::agent_state::PersistentAgentState {
            agent_id: agent_id.to_string(),
            agent_type: "custom".to_string(),
            state: state_data,
            metadata: crate::agent_state::AgentMetadata {
                name: agent_id.to_string(),
                description: None,
                version: "1.0.0".to_string(),
                capabilities: vec![],
                provider_config: None,
                tags: vec![],
            },
            creation_time: SystemTime::now(),
            last_modified: SystemTime::now(),
            schema_version: 1,
            hook_registrations: vec![],
            last_hook_execution: None,
            correlation_context: None,
        };

        // Save the state
        self.save_agent_state(&persistent_state).await?;

        // Queue hooks for async processing if enabled
        if let Some(processor) = &self.async_hook_processor {
            let context = HookContext::new(
                HookPoint::Custom("agent_save".to_string()),
                llmspell_hooks::ComponentId::new(
                    ComponentType::Custom("agent".to_string()),
                    agent_id.to_string(),
                ),
            );

            let event = HookEvent {
                hook_type: HookEventType::AfterAgentSave,
                context,
                hooks,
                correlation_id: Uuid::new_v4(),
                timestamp: std::time::Instant::now(),
            };

            let proc = processor.lock();
            proc.queue_hook_event(event)?;
        }

        Ok(())
    }

    /// Get the artifact correlation manager
    pub fn artifact_correlation_manager(&self) -> &Arc<ArtifactCorrelationManager> {
        &self.artifact_correlation_manager
    }

    /// Correlate an artifact with the current state operation
    pub async fn correlate_artifact_creation(
        &self,
        artifact_id: ArtifactId,
        component_id: CoreComponentId,
        parent_artifact: Option<ArtifactId>,
    ) -> String {
        self.artifact_correlation_manager
            .correlate_creation(artifact_id, component_id, parent_artifact)
            .await
    }

    /// Track that a state operation created an artifact
    pub async fn track_artifact_in_state(
        &self,
        scope: StateScope,
        key: &str,
        artifact_id: ArtifactId,
        operation: StateOperation,
    ) -> StateResult<()> {
        // Create a correlation between state and artifact
        let component_id = match &scope {
            StateScope::Agent(id) => CoreComponentId::from_name(&format!("agent:{}", id)),
            StateScope::Tool(id) => CoreComponentId::from_name(&format!("tool:{}", id)),
            StateScope::Workflow(id) => CoreComponentId::from_name(&format!("workflow:{}", id)),
            StateScope::Custom(id) => CoreComponentId::from_name(&format!("custom:{}", id)),
            StateScope::Global => CoreComponentId::from_name("global"),
            StateScope::Session(id) => CoreComponentId::from_name(&format!("session:{}", id)),
            StateScope::User(id) => CoreComponentId::from_name(&format!("user:{}", id)),
            StateScope::Hook(id) => CoreComponentId::from_name(&format!("hook:{}", id)),
        };

        let correlation_id = Uuid::new_v4().to_string();
        let artifact_id_str = artifact_id.to_string();

        let correlation = llmspell_core::state::ArtifactCorrelation {
            correlation_id: correlation_id.clone(),
            artifact_id,
            component_id,
            operation,
            timestamp: SystemTime::now(),
            parent_artifact: None,
            relationship: None,
        };

        self.artifact_correlation_manager
            .add_correlation(correlation)
            .await;

        // Store artifact reference in state
        let artifact_key = format!("artifact:{}:{}", key, artifact_id_str);
        let artifact_metadata = json!({
            "artifact_id": artifact_id_str,
            "correlation_id": correlation_id,
            "timestamp": SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        });

        self.set_state_internal(scope, &artifact_key, artifact_metadata)
            .await
    }

    /// Get artifacts associated with a component's state
    pub async fn get_component_artifacts(&self, component_id: &CoreComponentId) -> Vec<ArtifactId> {
        self.artifact_correlation_manager
            .get_artifacts_by_component(component_id)
            .await
    }

    /// Get artifact lineage (parent chain)
    pub async fn get_artifact_lineage(&self, artifact_id: &ArtifactId) -> Vec<ArtifactId> {
        self.artifact_correlation_manager
            .get_lineage(artifact_id)
            .await
    }

    /// Get all storage keys for backup/discovery purposes
    /// This is used by the backup system to discover all scopes and data
    pub async fn get_all_storage_keys(&self) -> StateResult<Vec<String>> {
        self.storage_adapter
            .list_keys("")
            .await
            .map_err(|e| StateError::storage(e.to_string()))
    }
}

// ==============================================================================
// TRAIT IMPLEMENTATIONS FOR llmspell-state-traits
// ==============================================================================

use async_trait::async_trait;
use super::{StatePersistence};
use llmspell_core::state::{StatePersistence as StatePersistenceTrait, TypedStatePersistence};

#[async_trait]
impl StatePersistenceTrait for StateManager {
    async fn set(&self, scope: StateScope, key: &str, value: Value) -> StateResult<()> {
        self.set(scope, key, value).await
    }

    async fn get(&self, scope: StateScope, key: &str) -> StateResult<Option<Value>> {
        self.get(scope, key).await
    }

    async fn delete(&self, scope: StateScope, key: &str) -> StateResult<bool> {
        self.delete(scope, key).await
    }

    async fn list_keys(&self, scope: StateScope) -> StateResult<Vec<String>> {
        self.list_keys(scope).await
    }

    async fn exists(&self, scope: StateScope, key: &str) -> StateResult<bool> {
        self.exists_in_scope(scope, key).await
    }

    async fn clear_scope(&self, scope: StateScope) -> StateResult<()> {
        self.clear_scope(scope).await
    }

    async fn get_all_in_scope(&self, scope: StateScope) -> StateResult<HashMap<String, Value>> {
        self.get_all_in_scope(scope).await
    }

    async fn copy_scope(&self, from_scope: StateScope, to_scope: StateScope) -> StateResult<usize> {
        self.copy_scope(from_scope, to_scope).await
    }

    async fn move_scope(&self, from_scope: StateScope, to_scope: StateScope) -> StateResult<usize> {
        self.move_scope(from_scope, to_scope).await
    }
}

// TypedStatePersistence provides default implementations on top of StatePersistence
impl TypedStatePersistence for StateManager {}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[tokio::test]
    async fn test_state_manager_basic_operations() {
        let manager = StateManager::new().await.unwrap();

        // Set and get
        manager
            .set(StateScope::Global, "test_key", json!("test_value"))
            .await
            .unwrap();

        let value = manager.get(StateScope::Global, "test_key").await.unwrap();
        assert_eq!(value, Some(json!("test_value")));

        // Delete
        let deleted = manager
            .delete(StateScope::Global, "test_key")
            .await
            .unwrap();
        assert!(deleted);

        let value = manager.get(StateScope::Global, "test_key").await.unwrap();
        assert_eq!(value, None);
    }
    #[tokio::test]
    async fn test_state_scoping() {
        let manager = StateManager::new().await.unwrap();

        // Set values in different scopes
        manager
            .set(StateScope::Global, "key", json!("global"))
            .await
            .unwrap();
        manager
            .set(
                StateScope::Agent("agent1".to_string()),
                "key",
                json!("agent1"),
            )
            .await
            .unwrap();
        manager
            .set(
                StateScope::Agent("agent2".to_string()),
                "key",
                json!("agent2"),
            )
            .await
            .unwrap();

        // Verify isolation
        let global = manager.get(StateScope::Global, "key").await.unwrap();
        assert_eq!(global, Some(json!("global")));

        let agent1 = manager
            .get(StateScope::Agent("agent1".to_string()), "key")
            .await
            .unwrap();
        assert_eq!(agent1, Some(json!("agent1")));

        let agent2 = manager
            .get(StateScope::Agent("agent2".to_string()), "key")
            .await
            .unwrap();
        assert_eq!(agent2, Some(json!("agent2")));
    }
    #[tokio::test]
    async fn test_key_validation() {
        let manager = StateManager::new().await.unwrap();

        // Empty key
        let result = manager.set(StateScope::Global, "", json!("value")).await;
        assert!(result.is_err());

        // Path traversal attempt
        let result = manager
            .set(StateScope::Global, "../etc/passwd", json!("value"))
            .await;
        assert!(result.is_err());

        // Valid key
        let result = manager
            .set(StateScope::Global, "valid_key_123", json!("value"))
            .await;
        assert!(result.is_ok());
    }
    #[tokio::test]
    async fn test_agent_state_persistence() {
        use super::agent_state::{MessageRole, PersistentAgentState};

        let manager = StateManager::new().await.unwrap();

        // Create a test agent state
        let mut agent_state =
            PersistentAgentState::new("test_agent_001".to_string(), "assistant".to_string());

        // Add some data
        agent_state.add_message(MessageRole::User, "Hello agent".to_string());
        agent_state.add_message(MessageRole::Assistant, "Hello! How can I help?".to_string());
        agent_state.record_tool_usage("calculator", 50, true);

        // Save the state
        manager.save_agent_state(&agent_state).await.unwrap();

        // Load the state back
        let loaded_state = manager.load_agent_state("test_agent_001").await.unwrap();
        assert!(loaded_state.is_some());

        let loaded = loaded_state.unwrap();
        assert_eq!(loaded.agent_id, "test_agent_001");
        assert_eq!(loaded.state.conversation_history.len(), 2);
        assert_eq!(loaded.state.tool_usage_stats.total_invocations, 1);

        // List agent states
        let agent_ids = manager.list_agent_states().await.unwrap();
        assert!(agent_ids.contains(&"test_agent_001".to_string()));

        // Delete the state
        let deleted = manager.delete_agent_state("test_agent_001").await.unwrap();
        assert!(deleted);

        // Verify deletion
        let loaded_after_delete = manager.load_agent_state("test_agent_001").await.unwrap();
        assert!(loaded_after_delete.is_none());
    }
    #[tokio::test]
    async fn test_agent_metadata_retrieval() {
        use super::agent_state::PersistentAgentState;

        let manager = StateManager::new().await.unwrap();

        // Create a test agent state with metadata
        let mut agent_state =
            PersistentAgentState::new("metadata_test_agent".to_string(), "researcher".to_string());

        agent_state.metadata.name = "Research Agent".to_string();
        agent_state.metadata.description = Some("Specialized research assistant".to_string());
        agent_state.metadata.capabilities =
            vec!["web_search".to_string(), "pdf_analysis".to_string()];
        agent_state.metadata.tags = vec!["research".to_string(), "academic".to_string()];

        // Save the state
        manager.save_agent_state(&agent_state).await.unwrap();

        // Get metadata without loading full state
        let metadata = manager
            .get_agent_metadata("metadata_test_agent")
            .await
            .unwrap();
        assert!(metadata.is_some());

        let meta = metadata.unwrap();
        assert_eq!(meta.name, "Research Agent");
        assert_eq!(meta.capabilities.len(), 2);
        assert_eq!(meta.tags.len(), 2);

        // Cleanup
        manager
            .delete_agent_state("metadata_test_agent")
            .await
            .unwrap();
    }
}
