//! ABOUTME: Migration source and target traits
//! ABOUTME: Defines generic interface for migration between storage backends

use anyhow::Result;
use async_trait::async_trait;

/// Migration source - backend to migrate FROM
#[async_trait]
pub trait MigrationSource: Send + Sync {
    /// List all keys for a component with given prefix
    ///
    /// # Arguments
    /// * `component` - Component name ("agent_state", "workflow_state", "sessions")
    ///
    /// # Returns
    /// * `Result<Vec<String>>` - List of keys matching the component
    async fn list_keys(&self, component: &str) -> Result<Vec<String>>;

    /// Get value for a specific key
    ///
    /// # Arguments
    /// * `component` - Component name
    /// * `key` - Key to retrieve
    ///
    /// # Returns
    /// * `Result<Option<Vec<u8>>>` - Value bytes or None if not found
    async fn get_value(&self, component: &str, key: &str) -> Result<Option<Vec<u8>>>;

    /// Count total records for a component
    ///
    /// # Arguments
    /// * `component` - Component name
    ///
    /// # Returns
    /// * `Result<usize>` - Total count
    async fn count(&self, component: &str) -> Result<usize>;
}

/// Migration target - backend to migrate TO
#[async_trait]
pub trait MigrationTarget: Send + Sync {
    /// Store a key-value pair
    ///
    /// # Arguments
    /// * `component` - Component name
    /// * `key` - Key to store
    /// * `value` - Value bytes
    ///
    /// # Returns
    /// * `Result<()>` - Success or error
    async fn store(&self, component: &str, key: &str, value: &[u8]) -> Result<()>;

    /// Get value for a specific key (for validation)
    ///
    /// # Arguments
    /// * `component` - Component name
    /// * `key` - Key to retrieve
    ///
    /// # Returns
    /// * `Result<Option<Vec<u8>>>` - Value bytes or None if not found
    async fn get_value(&self, component: &str, key: &str) -> Result<Option<Vec<u8>>>;

    /// Count total records for a component
    ///
    /// # Arguments
    /// * `component` - Component name
    ///
    /// # Returns
    /// * `Result<usize>` - Total count
    async fn count(&self, component: &str) -> Result<usize>;

    /// Delete a key (for cleanup on failure)
    ///
    /// # Arguments
    /// * `component` - Component name
    /// * `key` - Key to delete
    ///
    /// # Returns
    /// * `Result<()>` - Success or error
    async fn delete(&self, component: &str, key: &str) -> Result<()>;
}
