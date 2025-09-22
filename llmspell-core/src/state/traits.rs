// ABOUTME: Core state management traits for rs-llmspell components
// ABOUTME: Provides consistent state persistence interfaces across tools, workflows, hooks, and agents

use super::{StateError, StateResult, StateScope};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// Core state management trait for all state operations
///
/// This trait provides the fundamental interface for state storage and retrieval
/// across all components in the rs-llmspell system. It supports hierarchical
/// scoping and type-safe serialization.
#[async_trait]
pub trait StateManager: Send + Sync + std::fmt::Debug {
    /// Store a value in the specified scope with the given key
    async fn set(&self, scope: StateScope, key: &str, value: Value) -> StateResult<()>;

    /// Retrieve a value from the specified scope with the given key
    async fn get(&self, scope: StateScope, key: &str) -> StateResult<Option<Value>>;

    /// Delete a value from the specified scope with the given key
    /// Returns true if the key existed and was deleted, false otherwise
    async fn delete(&self, scope: StateScope, key: &str) -> StateResult<bool>;

    /// List all keys within a scope
    async fn list_keys(&self, scope: StateScope) -> StateResult<Vec<String>>;

    /// Check if a key exists in the specified scope
    async fn exists(&self, scope: StateScope, key: &str) -> StateResult<bool>;

    /// Clear all values in a scope
    async fn clear_scope(&self, scope: StateScope) -> StateResult<()>;

    /// Get all key-value pairs in a scope
    async fn get_all_in_scope(&self, scope: StateScope) -> StateResult<HashMap<String, Value>>;

    /// Copy all values from one scope to another
    async fn copy_scope(&self, from_scope: StateScope, to_scope: StateScope) -> StateResult<usize>;

    /// Move all values from one scope to another (copy then clear source)
    async fn move_scope(&self, from_scope: StateScope, to_scope: StateScope) -> StateResult<usize>;
}

/// Enhanced state persistence trait for components with specialized requirements
///
/// This trait extends the basic StateManager with additional functionality
/// needed by components that require more sophisticated state management,
/// such as hooks, transactions, and batching operations.
#[async_trait]
pub trait StatePersistence: StateManager {
    /// Store multiple key-value pairs atomically within a scope
    async fn set_batch(&self, scope: StateScope, items: HashMap<String, Value>) -> StateResult<()> {
        for (key, value) in items {
            self.set(scope.clone(), &key, value).await?;
        }
        Ok(())
    }

    /// Retrieve multiple values by keys within a scope
    async fn get_batch(
        &self,
        scope: StateScope,
        keys: Vec<String>,
    ) -> StateResult<HashMap<String, Value>> {
        let mut result = HashMap::new();
        for key in keys {
            if let Some(value) = self.get(scope.clone(), &key).await? {
                result.insert(key, value);
            }
        }
        Ok(result)
    }

    /// Delete multiple keys within a scope
    async fn delete_batch(&self, scope: StateScope, keys: Vec<String>) -> StateResult<usize> {
        let mut count = 0;
        for key in keys {
            if self.delete(scope.clone(), &key).await? {
                count += 1;
            }
        }
        Ok(count)
    }

    /// Store a serialized value directly as JSON Value
    async fn set_json(&self, scope: StateScope, key: &str, value: Value) -> StateResult<()> {
        self.set(scope, key, value).await
    }

    /// Retrieve a value as JSON Value for manual deserialization
    async fn get_json(&self, scope: StateScope, key: &str) -> StateResult<Option<Value>> {
        self.get(scope, key).await
    }

    /// Store data with expiration (if supported by backend)
    async fn set_with_ttl(
        &self,
        scope: StateScope,
        key: &str,
        value: Value,
        _ttl_seconds: u64,
    ) -> StateResult<()> {
        // Default implementation ignores TTL - backends can override for actual TTL support
        self.set(scope, key, value).await
    }

    /// Increment a numeric value atomically
    async fn increment(&self, scope: StateScope, key: &str, delta: i64) -> StateResult<i64> {
        let current = self
            .get(scope.clone(), key)
            .await?
            .and_then(|v| v.as_i64())
            .unwrap_or(0);
        let new_value = current + delta;
        self.set(scope, key, Value::from(new_value)).await?;
        Ok(new_value)
    }

    /// Set a value only if the key doesn't exist
    async fn set_if_not_exists(
        &self,
        scope: StateScope,
        key: &str,
        value: Value,
    ) -> StateResult<bool> {
        if self.exists(scope.clone(), key).await? {
            Ok(false)
        } else {
            self.set(scope, key, value).await?;
            Ok(true)
        }
    }

    /// Compare and swap operation for atomic updates
    async fn compare_and_swap(
        &self,
        scope: StateScope,
        key: &str,
        expected: Option<Value>,
        new_value: Value,
    ) -> StateResult<bool> {
        let current = self.get(scope.clone(), key).await?;
        if current == expected {
            self.set(scope, key, new_value).await?;
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

/// Typed state operations for compile-time type safety
///
/// This trait provides typed serialization/deserialization operations
/// that work with concrete types rather than trait objects.
pub trait TypedStatePersistence: StatePersistence {
    /// Store typed data with automatic serialization
    #[allow(async_fn_in_trait)]
    async fn set_typed<T>(&self, scope: StateScope, key: &str, value: &T) -> StateResult<()>
    where
        T: Serialize + Send + Sync,
    {
        let json_value =
            serde_json::to_value(value).map_err(|e| StateError::serialization(e.to_string()))?;
        self.set(scope, key, json_value).await
    }

    /// Retrieve typed data with automatic deserialization
    #[allow(async_fn_in_trait)]
    async fn get_typed<T>(&self, scope: StateScope, key: &str) -> StateResult<Option<T>>
    where
        T: for<'de> Deserialize<'de> + Send,
    {
        match self.get(scope, key).await? {
            Some(value) => {
                let typed_value = serde_json::from_value(value)
                    .map_err(|e| StateError::serialization(e.to_string()))?;
                Ok(Some(typed_value))
            }
            None => Ok(None),
        }
    }
}

/// State persistence capabilities for component-specific data
///
/// This trait defines how different component types (tools, workflows, hooks, agents)
/// should expose their state for persistence. Components implement this trait to
/// provide their state data in a standardized format.
#[async_trait]
pub trait ComponentStatePersistence: Send + Sync {
    /// Get the component's unique identifier for state scoping
    fn component_id(&self) -> String;

    /// Get the component type for scoping (tool, workflow, hook, agent, etc.)
    fn component_type(&self) -> String;

    /// Extract the component's current state for persistence
    async fn extract_state(&self) -> StateResult<Value>;

    /// Restore the component's state from persisted data
    async fn restore_state(&mut self, state: Value) -> StateResult<()>;

    /// Get the component's state scope
    fn state_scope(&self) -> StateScope {
        match self.component_type().as_str() {
            "tool" => StateScope::Tool(self.component_id()),
            "workflow" => StateScope::Workflow(self.component_id()),
            "hook" => StateScope::Hook(self.component_id()),
            "agent" => StateScope::Agent(self.component_id()),
            _ => StateScope::Custom(format!("{}:{}", self.component_type(), self.component_id())),
        }
    }

    /// Get metadata about the component's state structure
    async fn state_metadata(&self) -> StateResult<ComponentStateMetadata> {
        Ok(ComponentStateMetadata {
            component_id: self.component_id(),
            component_type: self.component_type(),
            schema_version: 1,
            last_updated: std::time::SystemTime::now(),
            state_keys: vec![],
            custom_metadata: HashMap::new(),
        })
    }

    /// Validate that restored state is compatible with this component
    async fn validate_state(&self, state: &Value) -> StateResult<bool> {
        // Default implementation accepts any valid JSON
        Ok(state.is_object()
            || state.is_array()
            || state.is_string()
            || state.is_number()
            || state.is_boolean()
            || state.is_null())
    }

    /// Clear all persisted state for this component
    async fn clear_state(&mut self, state_manager: &dyn StateManager) -> StateResult<()> {
        let scope = self.state_scope();
        state_manager.clear_scope(scope).await
    }

    /// Create a backup of the component's state
    async fn backup_state(
        &self,
        state_manager: &dyn StateManager,
    ) -> StateResult<ComponentStateBackup> {
        let scope = self.state_scope();
        let all_data = state_manager.get_all_in_scope(scope).await?;
        let current_state = self.extract_state().await?;

        Ok(ComponentStateBackup {
            component_id: self.component_id(),
            component_type: self.component_type(),
            timestamp: std::time::SystemTime::now(),
            state_data: current_state,
            raw_storage_data: all_data,
            metadata: self.state_metadata().await?,
        })
    }

    /// Restore from a backup
    async fn restore_from_backup(
        &mut self,
        backup: ComponentStateBackup,
        state_persistence: &dyn StatePersistence,
    ) -> StateResult<()> {
        // Validate backup compatibility
        if backup.component_id != self.component_id() {
            return Err(StateError::invalid_format(format!(
                "Backup component ID '{}' doesn't match current ID '{}'",
                backup.component_id,
                self.component_id()
            )));
        }

        if backup.component_type != self.component_type() {
            return Err(StateError::invalid_format(format!(
                "Backup component type '{}' doesn't match current type '{}'",
                backup.component_type,
                self.component_type()
            )));
        }

        // Restore the component state
        self.restore_state(backup.state_data).await?;

        // Restore raw storage data if needed
        let scope = self.state_scope();
        state_persistence.clear_scope(scope.clone()).await?;
        state_persistence
            .set_batch(scope, backup.raw_storage_data)
            .await?;

        Ok(())
    }
}

/// Metadata about a component's state structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentStateMetadata {
    /// Unique identifier of the component
    pub component_id: String,

    /// Type of the component (tool, workflow, hook, agent, etc.)
    pub component_type: String,

    /// Schema version for state compatibility
    pub schema_version: u32,

    /// When the state was last updated
    pub last_updated: std::time::SystemTime,

    /// List of state keys used by this component
    pub state_keys: Vec<String>,

    /// Additional metadata specific to the component type
    pub custom_metadata: HashMap<String, Value>,
}

/// Complete backup of a component's state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentStateBackup {
    /// Unique identifier of the component
    pub component_id: String,

    /// Type of the component
    pub component_type: String,

    /// When the backup was created
    pub timestamp: std::time::SystemTime,

    /// The component's structured state data
    pub state_data: Value,

    /// Raw key-value data from storage
    pub raw_storage_data: HashMap<String, Value>,

    /// Metadata about the component state
    pub metadata: ComponentStateMetadata,
}

/// Transaction-like operations for state management
///
/// This trait provides atomic operations across multiple state changes,
/// useful for complex component state updates that must succeed or fail together.
#[async_trait]
pub trait StateTransaction: Send + Sync {
    /// Begin a new transaction
    async fn begin_transaction(&self) -> StateResult<TransactionId>;

    /// Commit a transaction, making all changes permanent
    async fn commit_transaction(&self, transaction_id: TransactionId) -> StateResult<()>;

    /// Rollback a transaction, discarding all changes
    async fn rollback_transaction(&self, transaction_id: TransactionId) -> StateResult<()>;

    /// Set a value within a transaction
    async fn set_in_transaction(
        &self,
        transaction_id: TransactionId,
        scope: StateScope,
        key: &str,
        value: Value,
    ) -> StateResult<()>;

    /// Delete a value within a transaction
    async fn delete_in_transaction(
        &self,
        transaction_id: TransactionId,
        scope: StateScope,
        key: &str,
    ) -> StateResult<()>;

    /// Check if transactions are supported by this implementation
    fn supports_transactions(&self) -> bool {
        false
    }
}

/// Transaction identifier for tracking state operations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TransactionId(pub uuid::Uuid);

impl TransactionId {
    /// Create a new transaction ID
    #[must_use]
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4())
    }
}

impl Default for TransactionId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for TransactionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// State observation and hooks for monitoring state changes
///
/// This trait allows components to register for notifications when
/// state changes occur, enabling reactive behavior and audit logging.
#[async_trait]
pub trait StateObserver: Send + Sync {
    /// Called when a state value is set
    async fn on_state_set(
        &self,
        scope: &StateScope,
        key: &str,
        old_value: Option<&Value>,
        new_value: &Value,
    ) -> StateResult<()>;

    /// Called when a state value is deleted
    async fn on_state_deleted(
        &self,
        scope: &StateScope,
        key: &str,
        old_value: Option<&Value>,
    ) -> StateResult<()>;

    /// Called when a scope is cleared
    async fn on_scope_cleared(&self, scope: &StateScope, cleared_count: usize) -> StateResult<()>;

    /// Get the observer's identifier
    fn observer_id(&self) -> String;

    /// Check if this observer is interested in changes to the given scope
    fn interested_in_scope(&self, scope: &StateScope) -> bool {
        // By default, observe all scopes
        let _ = scope;
        true
    }
}

/// Registry for state observers
#[async_trait]
pub trait StateObserverRegistry: Send + Sync {
    /// Register an observer for state changes
    async fn register_observer(&self, observer: Box<dyn StateObserver>) -> StateResult<()>;

    /// Unregister an observer by ID
    async fn unregister_observer(&self, observer_id: &str) -> StateResult<bool>;

    /// Notify all interested observers of a state change
    async fn notify_state_set(
        &self,
        scope: &StateScope,
        key: &str,
        old_value: Option<&Value>,
        new_value: &Value,
    ) -> StateResult<()>;

    /// Notify all interested observers of a state deletion
    async fn notify_state_deleted(
        &self,
        scope: &StateScope,
        key: &str,
        old_value: Option<&Value>,
    ) -> StateResult<()>;

    /// Notify all interested observers of a scope clear
    async fn notify_scope_cleared(
        &self,
        scope: &StateScope,
        cleared_count: usize,
    ) -> StateResult<()>;
}

/// Migration support for state schema changes
///
/// This trait provides functionality for migrating state data when
/// component schemas or storage formats change.
#[async_trait]
pub trait StateMigration: Send + Sync {
    /// Get the current schema version
    fn current_schema_version(&self) -> u32;

    /// Check if migration is needed for the given data
    async fn needs_migration(&self, data: &Value, from_version: u32) -> StateResult<bool>;

    /// Migrate data from one schema version to another
    async fn migrate(&self, data: Value, from_version: u32, to_version: u32) -> StateResult<Value>;

    /// Get all supported migration paths
    fn supported_migrations(&self) -> Vec<(u32, u32)>;

    /// Validate that migrated data is correct
    async fn validate_migration(
        &self,
        original: &Value,
        migrated: &Value,
        from_version: u32,
        to_version: u32,
    ) -> StateResult<bool>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // Mock implementation for testing
    #[derive(Debug)]
    struct MockStateManager {
        data: std::sync::Arc<tokio::sync::RwLock<HashMap<String, Value>>>,
    }

    impl MockStateManager {
        fn new() -> Self {
            Self {
                data: std::sync::Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            }
        }

        #[allow(clippy::unused_self)]
        fn scoped_key(&self, scope: &StateScope, key: &str) -> String {
            scope.storage_key(key)
        }
    }

    #[async_trait]
    impl StateManager for MockStateManager {
        async fn set(&self, scope: StateScope, key: &str, value: Value) -> StateResult<()> {
            let scoped_key = self.scoped_key(&scope, key);
            let mut data = self.data.write().await;
            data.insert(scoped_key, value);
            Ok(())
        }

        async fn get(&self, scope: StateScope, key: &str) -> StateResult<Option<Value>> {
            let scoped_key = self.scoped_key(&scope, key);
            let data = self.data.read().await;
            Ok(data.get(&scoped_key).cloned())
        }

        async fn delete(&self, scope: StateScope, key: &str) -> StateResult<bool> {
            let scoped_key = self.scoped_key(&scope, key);
            let mut data = self.data.write().await;
            Ok(data.remove(&scoped_key).is_some())
        }

        async fn list_keys(&self, scope: StateScope) -> StateResult<Vec<String>> {
            let prefix = scope.storage_key("");
            let data = self.data.read().await;
            let keys: Vec<String> = data
                .keys()
                .filter(|k| k.starts_with(&prefix))
                .filter_map(|k| k.strip_prefix(&prefix))
                .map(str::to_string)
                .collect();
            Ok(keys)
        }

        async fn exists(&self, scope: StateScope, key: &str) -> StateResult<bool> {
            let scoped_key = self.scoped_key(&scope, key);
            let data = self.data.read().await;
            Ok(data.contains_key(&scoped_key))
        }

        async fn clear_scope(&self, scope: StateScope) -> StateResult<()> {
            let prefix = scope.storage_key("");
            let mut data = self.data.write().await;
            let keys_to_remove: Vec<String> = data
                .keys()
                .filter(|k| k.starts_with(&prefix))
                .cloned()
                .collect();
            for key in keys_to_remove {
                data.remove(&key);
            }
            Ok(())
        }

        async fn get_all_in_scope(&self, scope: StateScope) -> StateResult<HashMap<String, Value>> {
            let prefix = scope.storage_key("");
            let data = self.data.read().await;
            let mut result = HashMap::new();
            for (key, value) in data.iter() {
                if key.starts_with(&prefix) {
                    if let Some(suffix) = key.strip_prefix(&prefix) {
                        result.insert(suffix.to_string(), value.clone());
                    }
                }
            }
            Ok(result)
        }

        async fn copy_scope(
            &self,
            from_scope: StateScope,
            to_scope: StateScope,
        ) -> StateResult<usize> {
            let values = self.get_all_in_scope(from_scope).await?;
            let count = values.len();
            for (key, value) in values {
                self.set(to_scope.clone(), &key, value).await?;
            }
            Ok(count)
        }

        async fn move_scope(
            &self,
            from_scope: StateScope,
            to_scope: StateScope,
        ) -> StateResult<usize> {
            let count = self.copy_scope(from_scope.clone(), to_scope).await?;
            self.clear_scope(from_scope).await?;
            Ok(count)
        }
    }

    #[async_trait]
    impl StatePersistence for MockStateManager {}

    impl TypedStatePersistence for MockStateManager {}
    #[tokio::test]
    async fn test_state_manager_basic() {
        let manager = MockStateManager::new();

        // Test set and get
        manager
            .set(StateScope::Global, "test", json!("value"))
            .await
            .unwrap();
        let value = manager.get(StateScope::Global, "test").await.unwrap();
        assert_eq!(value, Some(json!("value")));

        // Test exists
        assert!(manager.exists(StateScope::Global, "test").await.unwrap());
        assert!(!manager
            .exists(StateScope::Global, "nonexistent")
            .await
            .unwrap());

        // Test delete
        assert!(manager.delete(StateScope::Global, "test").await.unwrap());
        assert!(!manager.exists(StateScope::Global, "test").await.unwrap());
    }
    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct TestData {
        name: String,
        value: i32,
    }

    #[tokio::test]
    async fn test_state_persistence_typed() {
        let manager = MockStateManager::new();

        let data = TestData {
            name: "test".to_string(),
            value: 42,
        };

        // Test typed set and get using TypedStatePersistence trait
        TypedStatePersistence::set_typed(&manager, StateScope::Global, "typed_test", &data)
            .await
            .unwrap();
        let retrieved: Option<TestData> =
            TypedStatePersistence::get_typed(&manager, StateScope::Global, "typed_test")
                .await
                .unwrap();
        assert_eq!(retrieved, Some(data));
    }
    #[tokio::test]
    async fn test_batch_operations() {
        let manager = MockStateManager::new();

        // Test batch set
        let mut items = HashMap::new();
        items.insert("key1".to_string(), json!("value1"));
        items.insert("key2".to_string(), json!("value2"));
        items.insert("key3".to_string(), json!("value3"));

        manager.set_batch(StateScope::Global, items).await.unwrap();

        // Test batch get
        let keys = vec!["key1".to_string(), "key2".to_string(), "key3".to_string()];
        let result = manager.get_batch(StateScope::Global, keys).await.unwrap();

        assert_eq!(result.len(), 3);
        assert_eq!(result.get("key1"), Some(&json!("value1")));
        assert_eq!(result.get("key2"), Some(&json!("value2")));
        assert_eq!(result.get("key3"), Some(&json!("value3")));
    }
    #[tokio::test]
    async fn test_scope_operations() {
        let manager = MockStateManager::new();

        // Set up data in source scope
        manager
            .set(
                StateScope::User("alice".to_string()),
                "pref1",
                json!("value1"),
            )
            .await
            .unwrap();
        manager
            .set(
                StateScope::User("alice".to_string()),
                "pref2",
                json!("value2"),
            )
            .await
            .unwrap();

        // Test copy scope
        let copied = manager
            .copy_scope(
                StateScope::User("alice".to_string()),
                StateScope::User("bob".to_string()),
            )
            .await
            .unwrap();
        assert_eq!(copied, 2);

        // Verify both scopes have the data
        assert!(manager
            .exists(StateScope::User("alice".to_string()), "pref1")
            .await
            .unwrap());
        assert!(manager
            .exists(StateScope::User("bob".to_string()), "pref1")
            .await
            .unwrap());

        // Test move scope
        let moved = manager
            .move_scope(
                StateScope::User("bob".to_string()),
                StateScope::User("charlie".to_string()),
            )
            .await
            .unwrap();
        assert_eq!(moved, 2);

        // Verify source is cleared and destination has data
        assert!(!manager
            .exists(StateScope::User("bob".to_string()), "pref1")
            .await
            .unwrap());
        assert!(manager
            .exists(StateScope::User("charlie".to_string()), "pref1")
            .await
            .unwrap());
    }
    #[tokio::test]
    async fn test_transaction_id() {
        let id1 = TransactionId::new();
        let id2 = TransactionId::new();

        assert_ne!(id1, id2);
        assert!(!id1.to_string().is_empty());
    }
}
