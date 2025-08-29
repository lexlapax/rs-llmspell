# llmspell-state-traits

**Core trait definitions for state management**

**🔗 Navigation**: [← Rust API](README.md) | [Crate Docs](https://docs.rs/llmspell-state-traits) | [Source](../../../../llmspell-state-traits)

---

## Overview

`llmspell-state-traits` defines the fundamental traits and interfaces for state management across LLMSpell. These traits provide the contract that all state implementations must follow, enabling pluggable state backends and consistent behavior across the system.

**Key Features:**
- 🎯 Core state operation traits
- 🔄 Async-first design
- 🏢 Scope and isolation traits
- 📝 Serialization traits
- 🔒 Thread-safe abstractions
- 📊 Observability interfaces
- 🧩 Composable state layers
- ⚡ Zero-cost abstractions

## Core Traits

### State Trait

The fundamental trait for all state operations:

```rust
use async_trait::async_trait;
use serde_json::Value;
use std::error::Error;

#[async_trait]
pub trait State: Send + Sync {
    /// Get a value from state
    async fn get(&self, key: &str) -> Result<Option<Value>, Box<dyn Error>>;
    
    /// Set a value in state
    async fn set(&self, key: &str, value: Value) -> Result<(), Box<dyn Error>>;
    
    /// Delete a value from state
    async fn delete(&self, key: &str) -> Result<(), Box<dyn Error>>;
    
    /// Check if a key exists
    async fn exists(&self, key: &str) -> Result<bool, Box<dyn Error>>;
    
    /// Get multiple values
    async fn get_many(&self, keys: &[String]) -> Result<Vec<Option<Value>>, Box<dyn Error>> {
        let mut results = Vec::with_capacity(keys.len());
        for key in keys {
            results.push(self.get(key).await?);
        }
        Ok(results)
    }
    
    /// Set multiple values
    async fn set_many(&self, items: Vec<(String, Value)>) -> Result<(), Box<dyn Error>> {
        for (key, value) in items {
            self.set(&key, value).await?;
        }
        Ok(())
    }
}
```

### ScopedState Trait

Enable hierarchical state organization:

```rust
#[async_trait]
pub trait ScopedState: State {
    /// Create a scoped view of the state
    async fn scope(&self, scope: &str) -> Result<Box<dyn ScopedState>, Box<dyn Error>>;
    
    /// Get the current scope path
    fn scope_path(&self) -> &str;
    
    /// Get parent scope (if any)
    async fn parent(&self) -> Option<Box<dyn ScopedState>>;
    
    /// List child scopes
    async fn children(&self) -> Result<Vec<String>, Box<dyn Error>>;
    
    /// Delete this scope and all children
    async fn delete_scope(&self) -> Result<(), Box<dyn Error>>;
}
```

### PersistentState Trait

Add persistence capabilities to state:

```rust
#[async_trait]
pub trait PersistentState: State {
    /// Persist current state to storage
    async fn persist(&self) -> Result<(), Box<dyn Error>>;
    
    /// Load state from storage
    async fn load(&self) -> Result<(), Box<dyn Error>>;
    
    /// Get persistence metadata
    async fn persistence_info(&self) -> Result<PersistenceInfo, Box<dyn Error>>;
    
    /// Configure persistence settings
    async fn configure_persistence(&self, config: PersistenceConfig) -> Result<(), Box<dyn Error>>;
}

#[derive(Debug, Clone)]
pub struct PersistenceInfo {
    pub backend: String,
    pub last_persisted: Option<SystemTime>,
    pub persistence_path: Option<PathBuf>,
    pub size_bytes: u64,
    pub entry_count: usize,
}

#[derive(Debug, Clone)]
pub struct PersistenceConfig {
    pub auto_persist: bool,
    pub persist_interval: Duration,
    pub compression: bool,
    pub encryption: bool,
}
```

### TransactionalState Trait

Support transactional operations:

```rust
#[async_trait]
pub trait TransactionalState: State {
    /// Begin a new transaction
    async fn begin(&self) -> Result<TransactionHandle, Box<dyn Error>>;
    
    /// Commit a transaction
    async fn commit(&self, handle: TransactionHandle) -> Result<(), Box<dyn Error>>;
    
    /// Rollback a transaction
    async fn rollback(&self, handle: TransactionHandle) -> Result<(), Box<dyn Error>>;
    
    /// Execute operations in a transaction
    async fn transact<F, R>(&self, f: F) -> Result<R, Box<dyn Error>>
    where
        F: FnOnce(&dyn State) -> Pin<Box<dyn Future<Output = Result<R, Box<dyn Error>>> + Send>> + Send,
        R: Send;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TransactionHandle(pub Uuid);
```

### ObservableState Trait

Enable state change notifications:

```rust
#[async_trait]
pub trait ObservableState: State {
    /// Subscribe to state changes
    async fn subscribe(&self, pattern: &str) -> Result<StateSubscription, Box<dyn Error>>;
    
    /// Unsubscribe from state changes
    async fn unsubscribe(&self, subscription: StateSubscription) -> Result<(), Box<dyn Error>>;
    
    /// Register a change handler
    async fn on_change<F>(&self, pattern: &str, handler: F) -> Result<HandlerId, Box<dyn Error>>
    where
        F: Fn(StateChangeEvent) + Send + Sync + 'static;
    
    /// Remove a change handler
    async fn remove_handler(&self, id: HandlerId) -> Result<(), Box<dyn Error>>;
}

#[derive(Debug, Clone)]
pub struct StateChangeEvent {
    pub key: String,
    pub old_value: Option<Value>,
    pub new_value: Option<Value>,
    pub operation: StateOperation,
    pub timestamp: SystemTime,
}

#[derive(Debug, Clone, Copy)]
pub enum StateOperation {
    Set,
    Delete,
    Clear,
}
```

## Query Traits

### QueryableState Trait

Advanced querying capabilities:

```rust
#[async_trait]
pub trait QueryableState: State {
    /// Find keys matching a pattern
    async fn keys(&self, pattern: &str) -> Result<Vec<String>, Box<dyn Error>>;
    
    /// Query values matching criteria
    async fn query(&self, query: StateQuery) -> Result<Vec<(String, Value)>, Box<dyn Error>>;
    
    /// Count matching entries
    async fn count(&self, query: StateQuery) -> Result<usize, Box<dyn Error>>;
    
    /// Aggregate values
    async fn aggregate(&self, aggregation: StateAggregation) -> Result<Value, Box<dyn Error>>;
}

#[derive(Debug, Clone)]
pub struct StateQuery {
    pub key_pattern: Option<String>,
    pub value_filter: Option<ValueFilter>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    pub order_by: Option<OrderBy>,
}

#[derive(Debug, Clone)]
pub enum ValueFilter {
    Equals(Value),
    Contains(String),
    GreaterThan(Value),
    LessThan(Value),
    Between(Value, Value),
    In(Vec<Value>),
    JsonPath(String),
}

#[derive(Debug, Clone)]
pub struct StateAggregation {
    pub operation: AggregationOp,
    pub field: Option<String>,
    pub group_by: Option<String>,
}

#[derive(Debug, Clone)]
pub enum AggregationOp {
    Count,
    Sum,
    Avg,
    Min,
    Max,
}
```

### IndexedState Trait

Support for indexed access:

```rust
#[async_trait]
pub trait IndexedState: QueryableState {
    /// Create an index on a field
    async fn create_index(&self, index: IndexDefinition) -> Result<(), Box<dyn Error>>;
    
    /// Drop an index
    async fn drop_index(&self, name: &str) -> Result<(), Box<dyn Error>>;
    
    /// List all indexes
    async fn list_indexes(&self) -> Result<Vec<IndexInfo>, Box<dyn Error>>;
    
    /// Query using an index
    async fn indexed_query(&self, index: &str, query: IndexQuery) -> Result<Vec<(String, Value)>, Box<dyn Error>>;
}

#[derive(Debug, Clone)]
pub struct IndexDefinition {
    pub name: String,
    pub field_path: String,
    pub index_type: IndexType,
    pub unique: bool,
}

#[derive(Debug, Clone)]
pub enum IndexType {
    Hash,
    BTree,
    FullText,
    Geospatial,
}
```

## Serialization Traits

### SerializableState Trait

Enable state serialization:

```rust
pub trait SerializableState: State {
    /// Serialize state to bytes
    fn serialize(&self) -> Result<Vec<u8>, Box<dyn Error>>;
    
    /// Deserialize state from bytes
    fn deserialize(&mut self, data: &[u8]) -> Result<(), Box<dyn Error>>;
    
    /// Export state to a specific format
    fn export(&self, format: ExportFormat) -> Result<String, Box<dyn Error>>;
    
    /// Import state from a specific format
    fn import(&mut self, data: &str, format: ExportFormat) -> Result<(), Box<dyn Error>>;
}

#[derive(Debug, Clone, Copy)]
pub enum ExportFormat {
    Json,
    Yaml,
    Toml,
    MessagePack,
    Protobuf,
}
```

## Composition Traits

### LayeredState Trait

Enable state layering and composition:

```rust
pub trait LayeredState: State {
    /// Add a layer on top
    fn push_layer(&mut self, layer: Box<dyn State>);
    
    /// Remove the top layer
    fn pop_layer(&mut self) -> Option<Box<dyn State>>;
    
    /// Get number of layers
    fn layer_count(&self) -> usize;
    
    /// Access a specific layer
    fn layer(&self, index: usize) -> Option<&dyn State>;
    
    /// Merge layers
    fn flatten(&self) -> Result<Box<dyn State>, Box<dyn Error>>;
}
```

### CachedState Trait

Add caching capabilities:

```rust
pub trait CachedState: State {
    /// Configure cache settings
    fn configure_cache(&mut self, config: CacheConfig);
    
    /// Get cache statistics
    fn cache_stats(&self) -> CacheStats;
    
    /// Clear the cache
    fn clear_cache(&mut self);
    
    /// Warm up the cache
    async fn warm_cache(&mut self, keys: &[String]) -> Result<(), Box<dyn Error>>;
}

#[derive(Debug, Clone)]
pub struct CacheConfig {
    pub max_size: usize,
    pub ttl: Option<Duration>,
    pub eviction_policy: EvictionPolicy,
}

#[derive(Debug, Clone, Copy)]
pub enum EvictionPolicy {
    LRU,
    LFU,
    FIFO,
    Random,
}

#[derive(Debug, Default)]
pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub evictions: u64,
    pub size: usize,
}
```

## Type-Safe State Traits

### TypedState Trait

Type-safe state operations:

```rust
pub trait TypedState {
    /// Get a typed value
    async fn get_typed<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>, Box<dyn Error>>;
    
    /// Set a typed value
    async fn set_typed<T: Serialize>(&self, key: &str, value: &T) -> Result<(), Box<dyn Error>>;
    
    /// Get with default
    async fn get_or_default<T: DeserializeOwned + Default>(&self, key: &str) -> Result<T, Box<dyn Error>> {
        Ok(self.get_typed(key).await?.unwrap_or_default())
    }
    
    /// Update a value
    async fn update<T, F>(&self, key: &str, f: F) -> Result<T, Box<dyn Error>>
    where
        T: DeserializeOwned + Serialize,
        F: FnOnce(Option<T>) -> T;
}
```

## Utility Traits

### MigratableState Trait

Support state schema migration:

```rust
#[async_trait]
pub trait MigratableState: State {
    /// Get current schema version
    async fn schema_version(&self) -> Result<Version, Box<dyn Error>>;
    
    /// Migrate to a new version
    async fn migrate(&self, target_version: Version) -> Result<(), Box<dyn Error>>;
    
    /// Register a migration
    fn register_migration(&mut self, migration: Box<dyn Migration>);
    
    /// List available migrations
    fn available_migrations(&self) -> Vec<&dyn Migration>;
}

pub trait Migration: Send + Sync {
    fn version(&self) -> Version;
    fn description(&self) -> &str;
    async fn up(&self, state: &dyn State) -> Result<(), Box<dyn Error>>;
    async fn down(&self, state: &dyn State) -> Result<(), Box<dyn Error>>;
}
```

### AuditableState Trait

Add audit logging to state:

```rust
#[async_trait]
pub trait AuditableState: State {
    /// Get audit log for a key
    async fn audit_log(&self, key: &str) -> Result<Vec<AuditEntry>, Box<dyn Error>>;
    
    /// Get full audit trail
    async fn full_audit_log(&self) -> Result<Vec<AuditEntry>, Box<dyn Error>>;
    
    /// Configure audit settings
    fn configure_audit(&mut self, config: AuditConfig);
}

#[derive(Debug, Clone)]
pub struct AuditEntry {
    pub timestamp: SystemTime,
    pub key: String,
    pub operation: StateOperation,
    pub old_value: Option<Value>,
    pub new_value: Option<Value>,
    pub user: Option<String>,
    pub metadata: HashMap<String, Value>,
}

#[derive(Debug, Clone)]
pub struct AuditConfig {
    pub enabled: bool,
    pub include_values: bool,
    pub retention: Duration,
}
```

## Implementation Helpers

### StateWrapper

Base implementation helper:

```rust
pub struct StateWrapper<S: State> {
    inner: S,
    middleware: Vec<Box<dyn StateMiddleware>>,
}

impl<S: State> StateWrapper<S> {
    pub fn new(state: S) -> Self {
        Self {
            inner: state,
            middleware: Vec::new(),
        }
    }
    
    pub fn with_middleware(mut self, middleware: Box<dyn StateMiddleware>) -> Self {
        self.middleware.push(middleware);
        self
    }
}

#[async_trait]
pub trait StateMiddleware: Send + Sync {
    async fn before_get(&self, key: &str) -> Result<(), Box<dyn Error>>;
    async fn after_get(&self, key: &str, value: &Option<Value>) -> Result<(), Box<dyn Error>>;
    async fn before_set(&self, key: &str, value: &Value) -> Result<(), Box<dyn Error>>;
    async fn after_set(&self, key: &str, value: &Value) -> Result<(), Box<dyn Error>>;
}
```

## Usage Examples

### Implementing a Custom State Backend

```rust
use llmspell_state_traits::{State, ScopedState};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::RwLock;

pub struct InMemoryState {
    data: RwLock<HashMap<String, Value>>,
    scope: String,
}

#[async_trait]
impl State for InMemoryState {
    async fn get(&self, key: &str) -> Result<Option<Value>, Box<dyn Error>> {
        Ok(self.data.read()?.get(&self.scoped_key(key)).cloned())
    }
    
    async fn set(&self, key: &str, value: Value) -> Result<(), Box<dyn Error>> {
        self.data.write()?.insert(self.scoped_key(key), value);
        Ok(())
    }
    
    async fn delete(&self, key: &str) -> Result<(), Box<dyn Error>> {
        self.data.write()?.remove(&self.scoped_key(key));
        Ok(())
    }
    
    async fn exists(&self, key: &str) -> Result<bool, Box<dyn Error>> {
        Ok(self.data.read()?.contains_key(&self.scoped_key(key)))
    }
}

impl InMemoryState {
    fn scoped_key(&self, key: &str) -> String {
        if self.scope.is_empty() {
            key.to_string()
        } else {
            format!("{}::{}", self.scope, key)
        }
    }
}
```

### Composing State Layers

```rust
use llmspell_state_traits::{State, CachedState, AuditableState};

pub struct CompositeState {
    cache: Box<dyn CachedState>,
    persistent: Box<dyn State>,
    audit: Box<dyn AuditableState>,
}

#[async_trait]
impl State for CompositeState {
    async fn get(&self, key: &str) -> Result<Option<Value>, Box<dyn Error>> {
        // Try cache first
        if let Some(value) = self.cache.get(key).await? {
            return Ok(Some(value));
        }
        
        // Fall back to persistent storage
        let value = self.persistent.get(key).await?;
        
        // Update cache if found
        if let Some(ref val) = value {
            self.cache.set(key, val.clone()).await?;
        }
        
        Ok(value)
    }
    
    async fn set(&self, key: &str, value: Value) -> Result<(), Box<dyn Error>> {
        // Update all layers
        self.cache.set(key, value.clone()).await?;
        self.persistent.set(key, value.clone()).await?;
        self.audit.set(key, value).await?;
        Ok(())
    }
}
```

### Type-Safe State Operations

```rust
use llmspell_state_traits::TypedState;
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
struct UserProfile {
    id: String,
    name: String,
    preferences: HashMap<String, String>,
}

async fn typed_operations(state: &dyn TypedState) -> Result<(), Box<dyn Error>> {
    let profile = UserProfile {
        id: "user123".to_string(),
        name: "Alice".to_string(),
        preferences: HashMap::new(),
    };
    
    // Set typed value
    state.set_typed("profile:user123", &profile).await?;
    
    // Get typed value
    if let Some(loaded) = state.get_typed::<UserProfile>("profile:user123").await? {
        println!("Loaded profile: {:?}", loaded);
    }
    
    // Update with closure
    let updated = state.update("profile:user123", |p: Option<UserProfile>| {
        let mut profile = p.unwrap_or_default();
        profile.preferences.insert("theme".to_string(), "dark".to_string());
        profile
    }).await?;
    
    Ok(())
}
```

## Testing Utilities

```rust
#[cfg(test)]
pub mod test_utils {
    use super::*;
    
    /// Create a test state implementation
    pub fn test_state() -> impl State {
        InMemoryState::new()
    }
    
    /// Assert state behavior compliance
    pub async fn assert_state_compliant<S: State>(state: S) {
        // Test basic operations
        state.set("test_key", json!("test_value")).await.unwrap();
        assert_eq!(
            state.get("test_key").await.unwrap(),
            Some(json!("test_value"))
        );
        
        // Test existence check
        assert!(state.exists("test_key").await.unwrap());
        
        // Test deletion
        state.delete("test_key").await.unwrap();
        assert!(!state.exists("test_key").await.unwrap());
        
        // Test batch operations
        state.set_many(vec![
            ("key1".to_string(), json!(1)),
            ("key2".to_string(), json!(2)),
        ]).await.unwrap();
        
        let values = state.get_many(&["key1".to_string(), "key2".to_string()])
            .await.unwrap();
        assert_eq!(values, vec![Some(json!(1)), Some(json!(2))]);
    }
}
```

## Performance Guidelines

1. **Async Operations**: All trait methods are async for non-blocking I/O
2. **Batch Operations**: Use `get_many`/`set_many` for bulk operations
3. **Caching**: Implement `CachedState` for frequently accessed data
4. **Indexing**: Use `IndexedState` for large datasets
5. **Transactions**: Group related operations in transactions

## Related Documentation

- [llmspell-state-persistence](llmspell-state-persistence.md) - Persistent state implementation
- [llmspell-sessions](llmspell-sessions.md) - Session-scoped state
- [llmspell-core](llmspell-core.md) - Core traits and types