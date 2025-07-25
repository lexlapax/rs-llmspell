# Phase 5: Persistent State Management - Design Document

**Version**: 1.0  
**Date**: July 2025  
**Status**: Implementation Ready  
**Phase**: 5 (Persistent State Management)  
**Timeline**: Weeks 19-20  
**Priority**: MEDIUM (Production Important)  
**Dependencies**: Phase 4 Hook System (COMPLETE), Phase 3.3 llmspell-storage (AVAILABLE)

> **📋 Detailed Implementation Guide**: This document provides complete specifications for implementing Phase 5 persistent state management for rs-llmspell, leveraging Phase 4's ReplayableHook trait and Phase 3.3's llmspell-storage infrastructure.

---

## Phase Overview

### Goal
Implement persistent state storage with sled/rocksdb backend, enabling state persistence across application restarts, hook history replay, and state timeline reconstruction.

### Core Principles
- **Hook Integration First**: Leverage Phase 4's ReplayableHook trait and event correlation for state change notifications
- **Storage Abstraction**: Build on Phase 3.3's llmspell-storage infrastructure with StorageSerialize trait
- **Backward Compatibility**: Maintain existing State API surface while adding persistence
- **Performance Preservation**: <5ms persistence overhead, maintain current in-memory performance characteristics

### Success Criteria
- [ ] Agent state persists across application restarts
- [ ] State can be serialized and restored correctly with full fidelity
- [ ] Multiple agents have independent, isolated persistent state
- [ ] State migrations work seamlessly for schema changes with rollback capability
- [ ] Backup/restore operations functional with integrity verification
- [ ] **Hook history is persisted and replayable** (Phase 4 integration)
- [ ] **State changes trigger appropriate hooks** (Phase 4 integration)
- [ ] **Event correlation IDs link state changes** (Phase 4 integration)

---

## 1. Implementation Specifications

### 1.1 Enhanced StateManager with Persistent Backend

**Core StateManager Enhancement:**

```rust
// Enhanced StateManager leveraging llmspell-storage
use llmspell_storage::{StorageBackend, StorageSerialize, StorageDeserialize};
use llmspell_hooks::{ReplayableHook, HookExecutor, HookContext};
use llmspell_events::{UniversalEvent, EventBus};

pub struct StateManager {
    // Existing in-memory state (Phase 3.3)
    in_memory: Arc<RwLock<HashMap<String, serde_json::Value>>>,
    
    // New persistent backend (Phase 5)
    storage_backend: Arc<dyn StorageBackend>,
    hook_executor: Arc<HookExecutor>,
    event_bus: Arc<EventBus>,
    
    // State persistence configuration
    persistence_config: PersistenceConfig,
    state_schema: StateSchema,
    
    // Hook history and replay (Phase 4 integration)
    hook_history: Arc<RwLock<Vec<SerializedHookExecution>>>,
    replay_manager: HookReplayManager,
    
    // Event correlation for timeline reconstruction
    correlation_tracker: EventCorrelationTracker,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistenceConfig {
    pub enabled: bool,
    pub backend_type: StorageBackendType,
    pub flush_interval: Duration,
    pub compression: bool,
    pub encryption: Option<EncryptionConfig>,
    pub backup_retention: Duration,
}

#[derive(Debug, Clone)]
pub enum StorageBackendType {
    Memory,           // Phase 3.3 compatibility
    Sled(SledConfig), // Development/testing
    RocksDB(RocksDBConfig), // Production
}

impl StateManager {
    // Backward compatible constructor (Phase 3.3 compatibility)
    pub fn new() -> Self {
        Self::with_backend(StorageBackendType::Memory, PersistenceConfig::default())
    }
    
    // New persistent constructor
    pub fn with_backend(backend_type: StorageBackendType, config: PersistenceConfig) -> Self {
        let storage_backend = Self::create_storage_backend(backend_type, &config)?;
        let hook_executor = Arc::new(HookExecutor::new());
        let event_bus = Arc::new(EventBus::new());
        
        Self {
            in_memory: Arc::new(RwLock::new(HashMap::new())),
            storage_backend,
            hook_executor,
            event_bus,
            persistence_config: config,
            state_schema: StateSchema::v1(),
            hook_history: Arc::new(RwLock::new(Vec::new())),
            replay_manager: HookReplayManager::new(),
            correlation_tracker: EventCorrelationTracker::new(),
        }
    }
    
    // Enhanced state operations with hook integration
    pub async fn set_with_hooks(&self, scope: StateScope, key: &str, value: serde_json::Value) -> Result<()> {
        let correlation_id = uuid::Uuid::new_v4();
        
        // Create hook context for state change
        let hook_context = HookContext::new()
            .with_correlation_id(correlation_id)
            .with_operation("state_set")
            .with_metadata("scope", scope.to_string())
            .with_metadata("key", key);
        
        // Execute pre-state-change hooks
        let pre_result = self.hook_executor.execute_hooks(
            HookPoint::Custom("before_state_change"),
            hook_context.clone()
        ).await?;
        
        // Handle hook results (Continue, Modified, Cancel, etc.)
        let final_value = match pre_result.aggregate_result() {
            HookResult::Continue => value,
            HookResult::Modified(new_data) => new_data,
            HookResult::Cancel => return Ok(()),
            _ => value,
        };
        
        // Perform state update (in-memory + persistent)
        self.set_state_internal(scope, key, final_value.clone()).await?;
        
        // Execute post-state-change hooks
        let post_context = hook_context.with_result(final_value.clone());
        self.hook_executor.execute_hooks(
            HookPoint::Custom("after_state_change"),
            post_context
        ).await?;
        
        // Emit state change event for correlation
        let state_event = UniversalEvent::new("state.changed")
            .with_correlation_id(correlation_id)
            .with_data(json!({
                "scope": scope.to_string(),
                "key": key,
                "value": final_value
            }));
        
        self.event_bus.publish(state_event).await?;
        
        Ok(())
    }
    
    // Internal state update with persistence
    async fn set_state_internal(&self, scope: StateScope, key: &str, value: serde_json::Value) -> Result<()> {
        let scoped_key = self.create_scoped_key(scope, key);
        
        // Update in-memory state first (fast path)
        {
            let mut memory = self.in_memory.write();
            memory.insert(scoped_key.clone(), value.clone());
        }
        
        // Persist to backend if enabled
        if self.persistence_config.enabled {
            let serialized_state = SerializableState {
                key: scoped_key,
                value,
                timestamp: SystemTime::now(),
                schema_version: self.state_schema.version,
            };
            
            self.storage_backend.store(&serialized_state.key, &serialized_state).await?;
        }
        
        Ok(())
    }
}
```

### 1.2 Hook History Persistence (Phase 4 Integration)

**Hook Replay Management:**

```rust
use llmspell_hooks::{ReplayableHook, HookContext, HookResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializedHookExecution {
    pub hook_id: String,
    pub execution_id: uuid::Uuid,
    pub hook_point: String,
    pub context: SerializedHookContext,
    pub result: SerializedHookResult,
    pub timestamp: SystemTime,
    pub correlation_id: uuid::Uuid,
    pub execution_duration: Duration,
}

pub struct HookReplayManager {
    storage_backend: Arc<dyn StorageBackend>,
    replay_filters: Vec<ReplayFilter>,
}

impl HookReplayManager {
    pub async fn persist_hook_execution(
        &self,
        hook: &dyn ReplayableHook,
        context: &HookContext,
        result: &HookResult
    ) -> Result<()> {
        let execution = SerializedHookExecution {
            hook_id: hook.id().to_string(),
            execution_id: uuid::Uuid::new_v4(),
            hook_point: context.hook_point().to_string(),
            context: SerializedHookContext::from(context),
            result: SerializedHookResult::from(result),
            timestamp: SystemTime::now(),
            correlation_id: context.correlation_id(),
            execution_duration: context.execution_duration(),
        };
        
        let key = format!("hook_history:{}:{}", 
            context.correlation_id(), 
            execution.execution_id
        );
        
        self.storage_backend.store(&key, &execution).await?;
        Ok(())
    }
    
    pub async fn replay_hooks_for_correlation(
        &self,
        correlation_id: uuid::Uuid,
        target_state_manager: &StateManager
    ) -> Result<StateReplayResult> {
        // Retrieve all hook executions for correlation ID
        let hook_executions = self.get_hook_executions_by_correlation(correlation_id).await?;
        
        // Sort by timestamp for proper replay order
        let mut sorted_executions = hook_executions;
        sorted_executions.sort_by_key(|e| e.timestamp);
        
        let mut replay_result = StateReplayResult::new(correlation_id);
        
        // Replay each hook execution
        for execution in sorted_executions {
            match self.replay_single_execution(&execution, target_state_manager).await {
                Ok(result) => replay_result.add_success(execution.execution_id, result),
                Err(e) => replay_result.add_failure(execution.execution_id, e),
            }
        }
        
        Ok(replay_result)
    }
}
```

### 1.3 Agent State Serialization (StorageSerialize Extension)

**Agent Persistence Integration:**

```rust
use llmspell_storage::{StorageSerialize, StorageDeserialize};
use llmspell_agents::{Agent, AgentState, AgentMetadata};

#[derive(Debug, Serialize, Deserialize)]
pub struct PersistentAgentState {
    pub agent_id: String,
    pub agent_type: String,
    pub state: AgentState,
    pub metadata: AgentMetadata,
    pub creation_time: SystemTime,
    pub last_modified: SystemTime,
    pub schema_version: u32,
    
    // Hook integration fields (Phase 4)
    pub hook_registrations: Vec<String>,
    pub last_hook_execution: Option<SystemTime>,
    pub correlation_context: Option<uuid::Uuid>,
}

impl StorageSerialize for PersistentAgentState {
    fn serialize_for_storage(&self) -> Result<Vec<u8>, llmspell_storage::StorageError> {
        bincode::serialize(self)
            .map_err(|e| llmspell_storage::StorageError::SerializationFailed(e.to_string()))
    }
    
    fn storage_key(&self) -> String {
        format!("agent_state:{}", self.agent_id)
    }
    
    fn storage_namespace(&self) -> String {
        "agents".to_string()
    }
}

impl StorageDeserialize for PersistentAgentState {
    fn deserialize_from_storage(data: &[u8]) -> Result<Self, llmspell_storage::StorageError> 
    where 
        Self: Sized 
    {
        bincode::deserialize(data)
            .map_err(|e| llmspell_storage::StorageError::DeserializationFailed(e.to_string()))
    }
}

// Agent trait extension for persistence
#[async_trait]
pub trait PersistentAgent: Agent {
    async fn save_state(&self, state_manager: &StateManager) -> Result<()>;
    async fn load_state(&mut self, state_manager: &StateManager) -> Result<()>;
    async fn delete_state(&self, state_manager: &StateManager) -> Result<()>;
}

impl<T: Agent> PersistentAgent for T {
    async fn save_state(&self, state_manager: &StateManager) -> Result<()> {
        let persistent_state = PersistentAgentState {
            agent_id: self.id().to_string(),
            agent_type: self.agent_type().to_string(),
            state: self.state().clone(),
            metadata: self.metadata().clone(),
            creation_time: self.creation_time(),
            last_modified: SystemTime::now(),
            schema_version: 1,
            hook_registrations: self.get_hook_registrations(),
            last_hook_execution: self.last_hook_execution(),
            correlation_context: self.current_correlation_id(),
        };
        
        state_manager.storage_backend
            .store(&persistent_state.storage_key(), &persistent_state)
            .await?;
        
        Ok(())
    }
    
    async fn load_state(&mut self, state_manager: &StateManager) -> Result<()> {
        let key = format!("agent_state:{}", self.id());
        
        match state_manager.storage_backend.load::<PersistentAgentState>(&key).await {
            Ok(persistent_state) => {
                self.set_state(persistent_state.state);
                self.set_metadata(persistent_state.metadata);
                self.restore_hook_registrations(persistent_state.hook_registrations).await?;
                self.set_correlation_context(persistent_state.correlation_context);
                Ok(())
            }
            Err(llmspell_storage::StorageError::NotFound) => {
                // No saved state - use defaults
                Ok(())
            }
            Err(e) => Err(anyhow::anyhow!("Failed to load agent state: {}", e))
        }
    }
    
    async fn delete_state(&self, state_manager: &StateManager) -> Result<()> {
        let key = format!("agent_state:{}", self.id());
        state_manager.storage_backend.delete(&key).await?;
        Ok(())
    }
}
```

### 1.4 State Migration and Versioning

**Schema Migration System:**

```rust
#[derive(Debug, Clone)]
pub struct StateSchema {
    pub version: u32,
    pub migrations: Vec<Box<dyn StateMigration>>,
}

#[async_trait]
pub trait StateMigration: Send + Sync {
    fn from_version(&self) -> u32;
    fn to_version(&self) -> u32;
    async fn migrate(&self, state: &mut StateManager) -> Result<MigrationResult>;
    async fn rollback(&self, state: &mut StateManager) -> Result<()>;
    fn is_destructive(&self) -> bool;
}

pub struct MigrationManager {
    current_schema: StateSchema,
    storage_backend: Arc<dyn StorageBackend>,
    hook_executor: Arc<HookExecutor>,
}

impl MigrationManager {
    pub async fn migrate_to_version(&self, target_version: u32, state_manager: &mut StateManager) -> Result<MigrationResult> {
        let current_version = self.get_current_version(state_manager).await?;
        
        if current_version == target_version {
            return Ok(MigrationResult::NoMigrationNeeded);
        }
        
        // Create migration plan
        let migration_plan = self.create_migration_plan(current_version, target_version)?;
        
        // Execute pre-migration hooks
        let hook_context = HookContext::new()
            .with_operation("state_migration")
            .with_metadata("from_version", current_version)
            .with_metadata("to_version", target_version);
        
        self.hook_executor.execute_hooks(
            HookPoint::Custom("before_state_migration"),
            hook_context.clone()
        ).await?;
        
        // Create backup before migration
        let backup_id = self.create_backup(state_manager).await?;
        
        let mut migration_result = MigrationResult::new(current_version, target_version);
        
        // Execute migrations in order
        for migration in migration_plan.migrations {
            match migration.migrate(state_manager).await {
                Ok(result) => migration_result.add_step_success(migration.to_version(), result),
                Err(e) => {
                    // Rollback on failure
                    self.rollback_to_backup(backup_id, state_manager).await?;
                    migration_result.add_step_failure(migration.to_version(), e);
                    return Ok(migration_result);
                }
            }
        }
        
        // Update schema version
        self.set_current_version(target_version, state_manager).await?;
        
        // Execute post-migration hooks
        let post_context = hook_context.with_result(migration_result.clone());
        self.hook_executor.execute_hooks(
            HookPoint::Custom("after_state_migration"),
            post_context
        ).await?;
        
        Ok(migration_result)
    }
}

// Example migration implementation
pub struct V1ToV2Migration;

#[async_trait]
impl StateMigration for V1ToV2Migration {
    fn from_version(&self) -> u32 { 1 }
    fn to_version(&self) -> u32 { 2 }
    
    async fn migrate(&self, state: &mut StateManager) -> Result<MigrationResult> {
        // Add new fields, transform data structures, etc.
        // Example: Convert string timestamps to structured DateTime
        let all_keys = state.get_all_keys().await?;
        
        for key in all_keys {
            if let Some(value) = state.get_raw(&key).await? {
                let migrated_value = self.migrate_value(value)?;
                state.set_raw(&key, migrated_value).await?;
            }
        }
        
        Ok(MigrationResult::Success)
    }
    
    async fn rollback(&self, state: &mut StateManager) -> Result<()> {
        // Implement rollback logic
        Ok(())
    }
    
    fn is_destructive(&self) -> bool { false }
}
```

---

## 2. Architectural Considerations

### 2.1 Phase 4 Hook System Integration

**ReplayableHook Integration Points:**

The state management system integrates deeply with Phase 4's hook system:

1. **State Change Hooks**: Every state modification triggers `before_state_change` and `after_state_change` hooks
2. **Hook History Persistence**: All ReplayableHook executions are automatically persisted with correlation IDs
3. **State Timeline Reconstruction**: Event correlation enables complete state change timeline visualization
4. **Migration Hooks**: State schema migrations trigger specialized hooks for monitoring and validation

**Event Correlation Integration:**

```rust
// Integration with Phase 4 UniversalEvent system
pub struct StateChangeEvent {
    base_event: UniversalEvent,
    scope: StateScope,
    key: String,
    old_value: Option<serde_json::Value>,
    new_value: serde_json::Value,
    triggered_by: String, // Hook, Agent, User, etc.
}

impl StateChangeEvent {
    pub fn emit_to_phase4_event_bus(&self, event_bus: &EventBus) -> Result<()> {
        let universal_event = UniversalEvent::new("state.changed")
            .with_correlation_id(self.base_event.correlation_id())
            .with_data(json!({
                "scope": self.scope.to_string(),
                "key": self.key,
                "old_value": self.old_value,
                "new_value": self.new_value,
                "triggered_by": self.triggered_by
            }))
            .with_metadata("state_change_type", "persistent");
            
        event_bus.publish(universal_event)
    }
}
```

### 2.2 Phase 3.3 Storage Infrastructure Leverage

**llmspell-storage Integration:**

Phase 5 builds directly on Phase 3.3's storage infrastructure:

```rust
// Leverage existing StorageBackend trait
use llmspell_storage::{
    StorageBackend, 
    MemoryBackend, 
    SledBackend, 
    StorageSerialize,
    StorageConfig
};

pub struct StateStorageAdapter {
    backend: Arc<dyn StorageBackend>,
    config: StorageConfig,
}

impl StateStorageAdapter {
    pub fn new_with_existing_backend(backend: Arc<dyn StorageBackend>) -> Self {
        Self {
            backend,
            config: StorageConfig::default(),
        }
    }
    
    pub async fn from_config(config: StorageConfig) -> Result<Self> {
        let backend = match config.backend_type.as_str() {
            "memory" => Arc::new(MemoryBackend::new()) as Arc<dyn StorageBackend>,
            "sled" => Arc::new(SledBackend::new(&config.path).await?) as Arc<dyn StorageBackend>,
            "rocksdb" => Arc::new(RocksDBBackend::new(&config.path).await?) as Arc<dyn StorageBackend>,
            _ => return Err(anyhow::anyhow!("Unsupported storage backend: {}", config.backend_type))
        };
        
        Ok(Self { backend, config })
    }
}
```

### 2.3 Backward Compatibility Strategy

**API Preservation:**

The existing State API from Phase 3.3 remains completely unchanged:

```rust
// Existing API (Phase 3.3) continues to work
impl StateManager {
    // These methods maintain exact same signatures
    pub fn set(&self, key: &str, value: serde_json::Value) -> Result<()> {
        // Internally routes to set_with_hooks for persistence
        block_on(self.set_with_hooks(StateScope::Global, key, value))
    }
    
    pub fn get(&self, key: &str) -> Option<serde_json::Value> {
        // Existing implementation unchanged
        let memory = self.in_memory.read();
        memory.get(key).cloned()
    }
    
    // Workflow-specific methods remain the same
    pub fn workflow_state(&self, workflow_id: String) -> WorkflowStateAccessor {
        WorkflowStateAccessor::new(self.clone(), StateScope::Workflow(workflow_id))
    }
}
```

### 2.4 Performance Architecture

**Multi-Tier Performance Strategy:**

1. **In-Memory Fast Path**: All reads from memory cache (Phase 3.3 performance preserved)
2. **Async Persistence**: Write-through to storage backend without blocking reads
3. **Batch Operations**: Group multiple state changes for efficient persistence
4. **Smart Flushing**: Configurable flush intervals with immediate flush for critical changes

```rust
pub struct PerformanceConfig {
    pub batch_size: usize,
    pub flush_interval: Duration,
    pub immediate_flush_patterns: Vec<String>, // Keys that flush immediately
    pub cache_size_limit: usize,
    pub compression_threshold: usize,
}

impl StateManager {
    async fn optimized_batch_set(&self, changes: Vec<StateChange>) -> Result<()> {
        // Group changes by backend for efficient batch operations
        let grouped_changes = self.group_changes_by_backend(changes);
        
        // Execute all changes in parallel
        let futures: Vec<_> = grouped_changes.into_iter()
            .map(|(backend, changes)| self.execute_batch_on_backend(backend, changes))
            .collect();
            
        futures::future::try_join_all(futures).await?;
        Ok(())
    }
}
```

---

## 3. Testing Strategy

### 3.1 Persistence Integration Tests

**Core Persistence Validation:**

```rust
#[cfg(test)]
mod persistence_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_state_persistence_across_restarts() {
        // Create state manager with persistent backend
        let temp_dir = tempfile::tempdir().unwrap();
        let config = PersistenceConfig {
            enabled: true,
            backend_type: StorageBackendType::Sled(SledConfig {
                path: temp_dir.path().to_path_buf(),
            }),
            ..Default::default()
        };
        
        let state_manager = StateManager::with_backend(config.backend_type.clone(), config.clone());
        
        // Set state and verify in-memory
        state_manager.set_with_hooks(StateScope::Global, "test_key", json!("test_value")).await.unwrap();
        assert_eq!(state_manager.get("test_key"), Some(json!("test_value")));
        
        // Simulate restart by creating new state manager with same backend
        drop(state_manager);
        let restored_state_manager = StateManager::with_backend(config.backend_type, config);
        
        // Verify state persisted
        assert_eq!(restored_state_manager.get("test_key"), Some(json!("test_value")));
    }
    
    #[tokio::test]
    async fn test_agent_state_isolation() {
        let state_manager = create_test_state_manager().await;
        
        // Create multiple agents with different states
        let agent1_state = create_test_agent_state("agent1", json!({"config": "value1"}));
        let agent2_state = create_test_agent_state("agent2", json!({"config": "value2"}));
        
        // Save both agent states
        state_manager.storage_backend.store(&agent1_state.storage_key(), &agent1_state).await.unwrap();
        state_manager.storage_backend.store(&agent2_state.storage_key(), &agent2_state).await.unwrap();
        
        // Verify isolation
        let loaded_agent1: PersistentAgentState = state_manager.storage_backend
            .load(&agent1_state.storage_key()).await.unwrap();
        let loaded_agent2: PersistentAgentState = state_manager.storage_backend
            .load(&agent2_state.storage_key()).await.unwrap();
            
        assert_ne!(loaded_agent1.state, loaded_agent2.state);
        assert_eq!(loaded_agent1.agent_id, "agent1");
        assert_eq!(loaded_agent2.agent_id, "agent2");
    }
}
```

### 3.2 Hook Integration Testing

**Phase 4 Integration Validation:**

```rust
#[tokio::test]
async fn test_hook_history_persistence() {
    let state_manager = create_test_state_manager().await;
    let correlation_id = uuid::Uuid::new_v4();
    
    // Register test hook that implements ReplayableHook
    let test_hook = TestReplayableHook::new("test_hook");
    state_manager.hook_executor.register_hook(
        HookPoint::Custom("before_state_change"),
        test_hook.clone()
    ).await.unwrap();
    
    // Perform state change that triggers hook
    state_manager.set_with_hooks(StateScope::Global, "test_key", json!("test_value")).await.unwrap();
    
    // Verify hook execution was persisted
    let hook_executions = state_manager.replay_manager
        .get_hook_executions_by_correlation(correlation_id).await.unwrap();
    
    assert!(!hook_executions.is_empty());
    assert_eq!(hook_executions[0].hook_id, "test_hook");
    assert_eq!(hook_executions[0].correlation_id, correlation_id);
}

#[tokio::test]
async fn test_state_timeline_reconstruction() {
    let state_manager = create_test_state_manager().await;
    let correlation_id = uuid::Uuid::new_v4();
    
    // Perform series of state changes
    let changes = vec![
        ("key1", json!("value1")),
        ("key2", json!("value2")),
        ("key1", json!("updated_value1")),
    ];
    
    for (key, value) in changes {
        state_manager.set_with_hooks(StateScope::Global, key, value).await.unwrap();
    }
    
    // Reconstruct timeline
    let timeline = state_manager.correlation_tracker
        .reconstruct_timeline(correlation_id).await.unwrap();
    
    assert_eq!(timeline.events.len(), 3);
    assert_eq!(timeline.events[0].key, "key1");
    assert_eq!(timeline.events[2].key, "key1");
    assert_eq!(timeline.events[2].new_value, json!("updated_value1"));
}
```

### 3.3 Migration Testing

**Schema Migration Validation:**

```rust
#[tokio::test]
async fn test_schema_migration_with_rollback() {
    let state_manager = create_test_state_manager().await;
    let migration_manager = MigrationManager::new(state_manager.clone());
    
    // Set up v1 data
    state_manager.set_raw("timestamp", json!("2023-01-01T00:00:00Z")).await.unwrap();
    
    // Create migration that might fail
    let migration = FailingV1ToV2Migration::new();
    
    // Execute migration (should fail and rollback)
    let result = migration_manager.migrate_to_version(2, &mut state_manager).await.unwrap();
    
    assert!(result.failed());
    assert_eq!(migration_manager.get_current_version(&state_manager).await.unwrap(), 1);
    
    // Verify rollback worked
    assert_eq!(state_manager.get_raw("timestamp").await.unwrap(), Some(json!("2023-01-01T00:00:00Z")));
}
```

---

## 4. Migration and Integration

### 4.1 Phase 3.3 Migration Path

**Seamless Migration Strategy:**

1. **Phase 3.3 Compatibility**: All existing State API calls continue to work unchanged
2. **Opt-in Persistence**: Persistence is disabled by default, can be enabled per component
3. **Gradual Migration**: Components can be migrated to persistent state one at a time
4. **Performance Preservation**: In-memory performance characteristics maintained

```rust
// Migration helper for existing components
pub struct StateMigrationHelper;

impl StateMigrationHelper {
    pub async fn migrate_component_to_persistent(
        component_name: &str,
        old_state_manager: &StateManager,
        new_state_manager: &StateManager
    ) -> Result<MigrationSummary> {
        let component_keys = old_state_manager.get_keys_for_component(component_name).await?;
        let mut summary = MigrationSummary::new(component_name);
        
        for key in component_keys {
            if let Some(value) = old_state_manager.get(&key) {
                new_state_manager.set_with_hooks(StateScope::Global, &key, value).await?;
                summary.add_migrated_key(key);
            }
        }
        
        Ok(summary)
    }
}
```

### 4.2 Configuration Integration

**Unified Configuration System:**

```rust
// Extension to existing configuration system
#[derive(Debug, Deserialize)]
pub struct StateConfig {
    pub persistence: PersistenceConfig,
    pub migration: MigrationConfig,
    pub backup: BackupConfig,
    pub performance: PerformanceConfig,
}

impl Default for StateConfig {
    fn default() -> Self {
        Self {
            persistence: PersistenceConfig {
                enabled: false, // Disabled by default for backward compatibility
                backend_type: StorageBackendType::Memory,
                flush_interval: Duration::from_secs(5),
                compression: true,
                encryption: None,
                backup_retention: Duration::from_days(7),
            },
            migration: MigrationConfig::default(),
            backup: BackupConfig::default(),
            performance: PerformanceConfig::default(),
        }
    }
}
```

---

## 5. Performance and Security

### 5.1 Performance Targets

**Quantified Performance Goals:**

1. **State Read Operations**: <1ms (maintained from Phase 3.3)
2. **State Write Operations**: <5ms (including persistence)
3. **Hook Execution Overhead**: <2ms per state change
4. **Migration Operations**: <100ms per 1000 state entries
5. **Memory Usage**: <10% increase over Phase 3.3 baseline

**Performance Monitoring Integration:**

```rust
use llmspell_hooks::builtin::MetricsHook;

pub struct StatePerformanceMonitor {
    metrics_hook: Arc<MetricsHook>,
    performance_tracker: PerformanceTracker,
}

impl StatePerformanceMonitor {
    pub async fn track_operation<F, R>(&self, operation: &str, f: F) -> Result<R>
    where
        F: Future<Output = Result<R>>,
    {
        let start = Instant::now();
        let result = f.await;
        let duration = start.elapsed();
        
        self.metrics_hook.record_duration(
            format!("state_operation_{}", operation),
            duration
        ).await;
        
        if duration > self.performance_tracker.warning_threshold(operation) {
            tracing::warn!(
                operation = operation,
                duration_ms = duration.as_millis(),
                "State operation exceeded performance threshold"
            );
        }
        
        result
    }
}
```

### 5.2 Security Considerations

**Data Protection Strategy:**

1. **Encryption at Rest**: Optional encryption for sensitive state data
2. **Access Control**: State scoping prevents unauthorized access
3. **Audit Logging**: All state changes logged with correlation IDs
4. **Backup Security**: Encrypted backups with integrity verification

```rust
#[derive(Debug, Clone)]
pub struct EncryptionConfig {
    pub algorithm: EncryptionAlgorithm,
    pub key_derivation: KeyDerivationConfig,
    pub rotation_interval: Duration,
}

#[derive(Debug, Clone)]
pub enum EncryptionAlgorithm {
    ChaCha20Poly1305,
    AES256GCM,
}

pub struct StateSecurityManager {
    encryption_key: Arc<RwLock<EncryptionKey>>,
    access_controller: StateAccessController,
    audit_logger: AuditLogger,
}

impl StateSecurityManager {
    pub async fn encrypt_state_value(&self, value: &serde_json::Value) -> Result<EncryptedValue> {
        let key = self.encryption_key.read();
        let serialized = serde_json::to_vec(value)?;
        let encrypted = key.encrypt(&serialized)?;
        
        Ok(EncryptedValue {
            ciphertext: encrypted,
            algorithm: self.get_algorithm(),
            created_at: SystemTime::now(),
        })
    }
    
    pub async fn audit_state_access(&self, operation: &str, scope: &StateScope, key: &str) -> Result<()> {
        let audit_entry = AuditEntry {
            operation: operation.to_string(),
            scope: scope.clone(),
            key: key.to_string(),
            timestamp: SystemTime::now(),
            user_context: self.get_current_user_context(),
        };
        
        self.audit_logger.log_entry(audit_entry).await
    }
}
```

---

## 6. Phase 6 Preparation

### 6.1 Session Boundary Hooks

**Preparation for Session Management:**

```rust
// Hook points for Phase 6 session management
pub enum SessionHookPoint {
    SessionStart,
    SessionEnd,
    SessionPause,
    SessionResume,
    ArtifactCollected,
    ArtifactDeleted,
}

// State management extensions for sessions
impl StateManager {
    pub async fn create_session_scope(&self, session_id: &str) -> Result<SessionStateAccessor> {
        let scope = StateScope::Custom(format!("session:{}", session_id));
        
        // Execute session start hooks
        let hook_context = HookContext::new()
            .with_operation("session_start")
            .with_metadata("session_id", session_id);
            
        self.hook_executor.execute_hooks(
            HookPoint::Custom("session_start"),
            hook_context
        ).await?;
        
        Ok(SessionStateAccessor::new(self.clone(), scope))
    }
    
    pub async fn archive_session_state(&self, session_id: &str) -> Result<SessionArchive> {
        let scope = StateScope::Custom(format!("session:{}", session_id));
        let session_data = self.get_all_in_scope(scope).await?;
        
        let archive = SessionArchive {
            session_id: session_id.to_string(),
            state_data: session_data,
            created_at: SystemTime::now(),
            correlation_ids: self.get_correlation_ids_for_session(session_id).await?,
        };
        
        // Store archive for Phase 6 artifact management
        self.storage_backend.store(
            &format!("session_archive:{}", session_id),
            &archive
        ).await?;
        
        Ok(archive)
    }
}
```

### 6.2 Artifact Collection Hooks

**Framework for Automatic Artifact Collection:**

```rust
// Prepare artifact collection infrastructure for Phase 6
pub trait ArtifactCollector: Send + Sync {
    async fn collect_artifacts(&self, context: &HookContext) -> Result<Vec<Artifact>>;
    fn artifact_types(&self) -> Vec<ArtifactType>;
}

pub struct StateArtifactCollector {
    state_manager: Arc<StateManager>,
}

impl ArtifactCollector for StateArtifactCollector {
    async fn collect_artifacts(&self, context: &HookContext) -> Result<Vec<Artifact>> {
        let session_id = context.get_metadata("session_id")
            .ok_or_else(|| anyhow::anyhow!("No session_id in context"))?;
            
        let state_snapshot = self.state_manager.create_state_snapshot(session_id).await?;
        
        Ok(vec![
            Artifact {
                artifact_type: ArtifactType::StateSnapshot,
                content: ArtifactContent::Structured(state_snapshot),
                metadata: ArtifactMetadata::new()
                    .with_session_id(session_id)
                    .with_timestamp(SystemTime::now()),
            }
        ])
    }
    
    fn artifact_types(&self) -> Vec<ArtifactType> {
        vec![ArtifactType::StateSnapshot, ArtifactType::HookHistory]
    }
}
```

---

This design document provides comprehensive specifications for Phase 5 implementation, leveraging all available infrastructure from Phases 3.3 and 4 while preparing integration points for Phase 6. The implementation maintains backward compatibility while adding powerful persistence capabilities with hook integration and event correlation.