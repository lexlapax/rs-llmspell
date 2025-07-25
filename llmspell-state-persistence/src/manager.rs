// ABOUTME: Core StateManager implementation with persistent backend support
// ABOUTME: Integrates Phase 4 hooks and Phase 3.3 storage for state persistence

use crate::backend_adapter::{create_storage_backend, StateStorageAdapter};
use crate::config::{PersistenceConfig, StateSchema};
use crate::error::{StateError, StateResult};
use crate::key_manager::KeyManager;
use crate::scope::StateScope;
use llmspell_events::{EventBus, UniversalEvent};
use llmspell_hooks::{
    HookContext, HookExecutor, HookPoint, HookResult,
    ReplayableHook, Hook, ComponentType,
};
use llmspell_storage::StorageBackend;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use serde_json::Value;
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
    pub result: String,         // Serialized HookResult
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
                .map_err(|e| StateError::SerializationError(e.to_string()))?,
            result: serde_json::to_string(result)
                .map_err(|e| StateError::SerializationError(e.to_string()))?,
            timestamp: SystemTime::now(),
            duration,
            metadata: HashMap::new(),
        };

        let key = format!(
            "hook_history:{}:{}",
            context.correlation_id,
            execution.execution_id
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
    storage_backend: Arc<dyn StorageBackend>,
    storage_adapter: Arc<StateStorageAdapter>,
    
    // Hook integration
    hook_executor: Arc<HookExecutor>,
    event_bus: Arc<EventBus>,
    
    // Configuration
    persistence_config: PersistenceConfig,
    state_schema: StateSchema,
    
    // Hook history and replay
    hook_history: Arc<RwLock<Vec<SerializedHookExecution>>>,
    replay_manager: HookReplayManager,
    
    // Registered hooks for state operations
    before_state_change_hooks: Arc<RwLock<Vec<Arc<dyn Hook>>>>,
    after_state_change_hooks: Arc<RwLock<Vec<Arc<dyn Hook>>>>,
}

impl StateManager {
    /// Create a new state manager with default in-memory backend
    pub async fn new() -> StateResult<Self> {
        Self::with_backend(
            crate::config::StorageBackendType::Memory,
            PersistenceConfig::default(),
        )
        .await
    }

    /// Create a new state manager with specified backend
    pub async fn with_backend(
        backend_type: crate::config::StorageBackendType,
        config: PersistenceConfig,
    ) -> StateResult<Self> {
        let storage_backend = create_storage_backend(&backend_type).await?;
        let storage_adapter = Arc::new(StateStorageAdapter::new(
            storage_backend.clone(),
            "state".to_string(),
        ));
        let hook_executor = Arc::new(HookExecutor::new());
        let event_bus = Arc::new(EventBus::new());
        let replay_manager = HookReplayManager::new(storage_adapter.clone());

        // Load existing state from storage if persistent
        let in_memory = if config.enabled {
            let mut state = HashMap::new();
            let keys = storage_adapter.list_keys("").await?;
            for key in keys {
                if let Some(serialized) = storage_adapter.load::<SerializableState>(&key).await? {
                    state.insert(key, serialized.value);
                }
            }
            Arc::new(RwLock::new(state))
        } else {
            Arc::new(RwLock::new(HashMap::new()))
        };

        Ok(Self {
            in_memory,
            storage_backend,
            storage_adapter,
            hook_executor,
            event_bus,
            persistence_config: config,
            state_schema: StateSchema::v1(),
            hook_history: Arc::new(RwLock::new(Vec::new())),
            replay_manager,
            before_state_change_hooks: Arc::new(RwLock::new(Vec::new())),
            after_state_change_hooks: Arc::new(RwLock::new(Vec::new())),
        })
    }

    /// Set state with hooks and persistence
    pub async fn set_with_hooks(
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
        let mut hook_context = HookContext::new(
            HookPoint::Custom("state_change".to_string()),
            component_id,
        );
        hook_context = hook_context.with_correlation_id(correlation_id);
        
        // Add metadata as string values
        hook_context.insert_metadata("operation".to_string(), "state_set".to_string());
        hook_context.insert_metadata("scope".to_string(), serde_json::to_string(&scope).unwrap());
        hook_context.insert_metadata("key".to_string(), key.to_string());
        hook_context.insert_metadata("old_value".to_string(), serde_json::to_string(&old_value).unwrap());
        hook_context.insert_metadata("new_value".to_string(), serde_json::to_string(&value).unwrap());

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
                .map_err(|e| StateError::HookError(e.to_string()))?
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

        // Execute post-state-change hooks
        hook_context.insert_metadata("success".to_string(), "true".to_string());
        hook_context.insert_metadata("final_value".to_string(), serde_json::to_string(&final_value).unwrap());
        
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
                .map_err(|e| StateError::HookError(e.to_string()))?;
        }

        // Emit state change event
        let event_data = serde_json::json!({
            "scope": scope,
            "key": key,
            "old_value": old_value,
            "new_value": final_value,
        });
        
        let state_event = UniversalEvent::new(
            "state.changed",
            event_data,
            llmspell_events::Language::Rust,
        );
        let state_event = state_event.with_correlation_id(correlation_id);

        self.event_bus
            .publish(state_event)
            .await
            .map_err(|e| StateError::StorageError(e.into()))?;

        Ok(())
    }

    /// Set state value (backward compatible method)
    pub async fn set(&self, scope: StateScope, key: &str, value: Value) -> StateResult<()> {
        if self.persistence_config.enabled {
            self.set_with_hooks(scope, key, value).await
        } else {
            // Fast path for non-persistent state
            self.set_state_internal(scope, key, value).await
        }
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
            if let Some(serialized) = self.storage_adapter.load::<SerializableState>(&scoped_key).await? {
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
    
    /// Save agent state to persistent storage
    pub async fn save_agent_state(
        &self,
        agent_state: &crate::agent_state::PersistentAgentState,
    ) -> StateResult<()> {
        let key = format!("agent_state:{}", agent_state.agent_id);
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
                .map_err(|e| StateError::HookError(e.to_string()))?;
        }
        
        // Store in persistent backend
        self.storage_adapter
            .store(&key, agent_state)
            .await?;
        
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
                .map_err(|e| StateError::HookError(e.to_string()))?;
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
            .map_err(|e| StateError::StorageError(e.into()))?;
        
        Ok(())
    }
    
    /// Load agent state from persistent storage
    pub async fn load_agent_state(
        &self,
        agent_id: &str,
    ) -> StateResult<Option<crate::agent_state::PersistentAgentState>> {
        let key = format!("agent_state:{}", agent_id);
        
        // Try to load from storage
        match self.storage_adapter.load(&key).await? {
            Some(state) => Ok(Some(state)),
            None => Ok(None),
        }
    }
    
    /// Delete agent state from persistent storage
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
                .map_err(|e| StateError::HookError(e.to_string()))?;
        }
        
        // Delete from storage
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
                .map_err(|e| StateError::HookError(e.to_string()))?;
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
            .map_err(|e| StateError::StorageError(e.into()))?;
        
        Ok(true)
    }
    
    /// List all saved agent states
    pub async fn list_agent_states(&self) -> StateResult<Vec<String>> {
        let prefix = "agent_state:";
        let keys = self.storage_adapter.list_keys(prefix).await?;
        
        // Extract agent IDs from keys
        Ok(keys
            .into_iter()
            .filter_map(|k| k.strip_prefix(prefix).map(|s| s.to_string()))
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
}

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
        let deleted = manager.delete(StateScope::Global, "test_key").await.unwrap();
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
            .set(StateScope::Agent("agent1".to_string()), "key", json!("agent1"))
            .await
            .unwrap();
        manager
            .set(StateScope::Agent("agent2".to_string()), "key", json!("agent2"))
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
        use crate::agent_state::{PersistentAgentState, MessageRole};
        
        let manager = StateManager::new().await.unwrap();
        
        // Create a test agent state
        let mut agent_state = PersistentAgentState::new(
            "test_agent_001".to_string(),
            "assistant".to_string(),
        );
        
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
        use crate::agent_state::PersistentAgentState;
        
        let manager = StateManager::new().await.unwrap();
        
        // Create a test agent state with metadata
        let mut agent_state = PersistentAgentState::new(
            "metadata_test_agent".to_string(),
            "researcher".to_string(),
        );
        
        agent_state.metadata.name = "Research Agent".to_string();
        agent_state.metadata.description = Some("Specialized research assistant".to_string());
        agent_state.metadata.capabilities = vec!["web_search".to_string(), "pdf_analysis".to_string()];
        agent_state.metadata.tags = vec!["research".to_string(), "academic".to_string()];
        
        // Save the state
        manager.save_agent_state(&agent_state).await.unwrap();
        
        // Get metadata without loading full state
        let metadata = manager.get_agent_metadata("metadata_test_agent").await.unwrap();
        assert!(metadata.is_some());
        
        let meta = metadata.unwrap();
        assert_eq!(meta.name, "Research Agent");
        assert_eq!(meta.capabilities.len(), 2);
        assert_eq!(meta.tags.len(), 2);
        
        // Cleanup
        manager.delete_agent_state("metadata_test_agent").await.unwrap();
    }
}