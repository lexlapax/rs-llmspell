//! ABOUTME: StateAccess trait for universal state management across all components
//! ABOUTME: Provides abstraction for persistent state operations used by workflows, agents, and tools

use crate::Result;
use async_trait::async_trait;
use serde_json::Value;
use std::fmt::Debug;

/// Universal state access trait for all components
/// 
/// This trait provides a clean abstraction for state operations that can be
/// implemented by various backends (in-memory, persistent, distributed, etc.).
/// It's designed to be the primary data bus for component communication,
/// following patterns from Google ADK, Temporal, and Airflow.
/// 
/// # Key Design Decisions
/// 
/// - **Simple Key-Value Model**: Uses string keys and JSON values for maximum flexibility
/// - **Async Operations**: All operations are async to support various backends
/// - **Prefix-Based Organization**: Keys can use prefixes like "workflow:id:step" for organization
/// - **Optional Implementation**: Components work with `Option<Arc<dyn StateAccess>>`
/// 
/// # Usage Patterns
/// 
/// ```ignore
/// // Workflows write outputs to state
/// context.state.write("workflow:main:ux_design", design_output).await?;
/// 
/// // Agents access workflow outputs
/// let design = context.state.read("workflow:main:ux_design").await?;
/// 
/// // Tools can share results
/// context.state.write("tool:analyzer:metrics", metrics).await?;
/// ```
#[async_trait]
pub trait StateAccess: Send + Sync + Debug {
    /// Read a value from state
    /// 
    /// Returns None if the key doesn't exist.
    /// 
    /// # Arguments
    /// * `key` - The state key to read from
    /// 
    /// # Example
    /// ```ignore
    /// if let Some(value) = state.read("user:preferences").await? {
    ///     // Process the value
    /// }
    /// ```
    async fn read(&self, key: &str) -> Result<Option<Value>>;

    /// Write a value to state
    /// 
    /// Overwrites any existing value at the key.
    /// 
    /// # Arguments
    /// * `key` - The state key to write to
    /// * `value` - The JSON value to store
    /// 
    /// # Example
    /// ```ignore
    /// state.write("workflow:result", json!({
    ///     "status": "completed",
    ///     "output": data
    /// })).await?;
    /// ```
    async fn write(&self, key: &str, value: Value) -> Result<()>;

    /// Delete a value from state
    /// 
    /// Returns true if the key existed and was deleted, false otherwise.
    /// 
    /// # Arguments
    /// * `key` - The state key to delete
    /// 
    /// # Example
    /// ```ignore
    /// if state.delete("temp:data").await? {
    ///     println!("Temporary data cleaned up");
    /// }
    /// ```
    async fn delete(&self, key: &str) -> Result<bool>;

    /// List all keys matching a prefix
    /// 
    /// Useful for discovering related state entries.
    /// 
    /// # Arguments
    /// * `prefix` - The key prefix to search for (empty string returns all keys)
    /// 
    /// # Example
    /// ```ignore
    /// // Get all workflow outputs
    /// let workflow_keys = state.list_keys("workflow:main:").await?;
    /// for key in workflow_keys {
    ///     let value = state.read(&key).await?;
    ///     // Process each workflow output
    /// }
    /// ```
    async fn list_keys(&self, prefix: &str) -> Result<Vec<String>>;

    /// Check if a key exists without reading its value
    /// 
    /// More efficient than read() when you only need existence check.
    /// 
    /// # Arguments
    /// * `key` - The state key to check
    /// 
    /// # Default Implementation
    /// Uses read() to check existence, but backends can override for efficiency.
    async fn exists(&self, key: &str) -> Result<bool> {
        Ok(self.read(key).await?.is_some())
    }

    /// Atomically write multiple key-value pairs
    /// 
    /// Either all writes succeed or none do (if backend supports transactions).
    /// 
    /// # Arguments
    /// * `entries` - Vector of (key, value) pairs to write
    /// 
    /// # Default Implementation
    /// Writes sequentially; backends can override for atomic batch operations.
    async fn write_batch(&self, entries: Vec<(String, Value)>) -> Result<()> {
        for (key, value) in entries {
            self.write(&key, value).await?;
        }
        Ok(())
    }

    /// Read multiple values in a single operation
    /// 
    /// Returns a vector of optional values in the same order as the keys.
    /// 
    /// # Arguments
    /// * `keys` - Slice of keys to read
    /// 
    /// # Default Implementation
    /// Reads sequentially; backends can override for batch efficiency.
    async fn read_batch(&self, keys: &[String]) -> Result<Vec<Option<Value>>> {
        let mut results = Vec::with_capacity(keys.len());
        for key in keys {
            results.push(self.read(key).await?);
        }
        Ok(results)
    }

    /// Clear all keys with a given prefix
    /// 
    /// Useful for cleanup operations.
    /// 
    /// # Arguments
    /// * `prefix` - The key prefix to clear
    /// 
    /// # Returns
    /// Number of keys deleted
    /// 
    /// # Default Implementation
    /// Lists keys and deletes them individually; backends can override.
    async fn clear_prefix(&self, prefix: &str) -> Result<usize> {
        let keys = self.list_keys(prefix).await?;
        let count = keys.len();
        for key in keys {
            self.delete(&key).await?;
        }
        Ok(count)
    }
}