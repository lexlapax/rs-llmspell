//! ABOUTME: Test utilities and mock implementations for workflow testing
//! ABOUTME: Provides MockStateAccess and other test helpers for state-based workflow testing

use async_trait::async_trait;
use llmspell_core::{traits::state::StateAccess, Result};
use parking_lot::RwLock;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

/// Mock implementation of StateAccess for testing
///
/// This provides an in-memory state store that can be used for testing
/// state-based workflows without requiring a full StateManager setup.
///
/// # Examples
///
/// ```rust
/// use llmspell_workflows::test_utils::MockStateAccess;
/// use std::sync::Arc;
///
/// let mock_state = Arc::new(MockStateAccess::new());
/// // Use in ExecutionContext for workflow testing
/// ```
#[derive(Debug)]
pub struct MockStateAccess {
    /// In-memory storage for state data
    storage: Arc<RwLock<HashMap<String, Value>>>,
    /// Track read operations for verification
    reads: Arc<RwLock<Vec<String>>>,
    /// Track write operations for verification
    writes: Arc<RwLock<Vec<(String, Value)>>>,
    /// Track delete operations for verification
    deletes: Arc<RwLock<Vec<String>>>,
    /// Simulate failures for error testing
    fail_reads: Arc<RwLock<Vec<String>>>,
    fail_writes: Arc<RwLock<Vec<String>>>,
    fail_deletes: Arc<RwLock<Vec<String>>>,
}

impl MockStateAccess {
    /// Create a new mock state access instance
    #[must_use]
    pub fn new() -> Self {
        Self {
            storage: Arc::new(RwLock::new(HashMap::new())),
            reads: Arc::new(RwLock::new(Vec::new())),
            writes: Arc::new(RwLock::new(Vec::new())),
            deletes: Arc::new(RwLock::new(Vec::new())),
            fail_reads: Arc::new(RwLock::new(Vec::new())),
            fail_writes: Arc::new(RwLock::new(Vec::new())),
            fail_deletes: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Get all stored keys for verification
    #[must_use]
    pub fn get_all_keys(&self) -> Vec<String> {
        self.storage.read().keys().cloned().collect()
    }

    /// Get all values stored for verification
    #[must_use]
    pub fn get_all_values(&self) -> HashMap<String, Value> {
        self.storage.read().clone()
    }

    /// Get tracked read operations
    #[must_use]
    pub fn get_reads(&self) -> Vec<String> {
        self.reads.read().clone()
    }

    /// Get tracked write operations
    #[must_use]
    pub fn get_writes(&self) -> Vec<(String, Value)> {
        self.writes.read().clone()
    }

    /// Get tracked delete operations
    #[must_use]
    pub fn get_deletes(&self) -> Vec<String> {
        self.deletes.read().clone()
    }

    /// Clear all tracking data
    pub fn clear_tracking(&self) {
        self.reads.write().clear();
        self.writes.write().clear();
        self.deletes.write().clear();
    }

    /// Clear all stored data
    pub fn clear_storage(&self) {
        self.storage.write().clear();
    }

    /// Configure read operations to fail for specific keys
    pub fn set_read_failures(&self, keys: Vec<String>) {
        *self.fail_reads.write() = keys;
    }

    /// Configure write operations to fail for specific keys
    pub fn set_write_failures(&self, keys: Vec<String>) {
        *self.fail_writes.write() = keys;
    }

    /// Configure delete operations to fail for specific keys
    pub fn set_delete_failures(&self, keys: Vec<String>) {
        *self.fail_deletes.write() = keys;
    }

    /// Check if a key count matches expected value
    #[must_use]
    pub fn has_key_count(&self, expected: usize) -> bool {
        self.storage.read().len() == expected
    }

    /// Check if a specific key exists
    #[must_use]
    pub fn has_key(&self, key: &str) -> bool {
        self.storage.read().contains_key(key)
    }

    /// Get value by key for testing
    #[must_use]
    pub fn get_value(&self, key: &str) -> Option<Value> {
        self.storage.read().get(key).cloned()
    }

    /// Check if read was performed on specific key
    #[must_use]
    pub fn was_read(&self, key: &str) -> bool {
        self.reads.read().contains(&key.to_string())
    }

    /// Check if write was performed on specific key
    #[must_use]
    pub fn was_written(&self, key: &str) -> bool {
        self.writes.read().iter().any(|(k, _)| k == key)
    }

    /// Check if delete was performed on specific key
    #[must_use]
    pub fn was_deleted(&self, key: &str) -> bool {
        self.deletes.read().contains(&key.to_string())
    }

    /// Get the number of operations performed
    #[must_use]
    pub fn operation_counts(&self) -> (usize, usize, usize) {
        (
            self.reads.read().len(),
            self.writes.read().len(),
            self.deletes.read().len(),
        )
    }
}

impl Default for MockStateAccess {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl StateAccess for MockStateAccess {
    async fn read(&self, key: &str) -> Result<Option<Value>> {
        // Track the read operation
        self.reads.write().push(key.to_string());

        // Check if this read should fail
        if self.fail_reads.read().contains(&key.to_string()) {
            return Err(llmspell_core::LLMSpellError::Storage {
                message: format!("Mock read failure for key: {key}"),
                operation: Some("read".to_string()),
                source: None,
            });
        }

        // Perform the read
        Ok(self.storage.read().get(key).cloned())
    }

    async fn write(&self, key: &str, value: Value) -> Result<()> {
        // Track the write operation
        self.writes.write().push((key.to_string(), value.clone()));

        // Check if this write should fail
        if self.fail_writes.read().contains(&key.to_string()) {
            return Err(llmspell_core::LLMSpellError::Storage {
                message: format!("Mock write failure for key: {key}"),
                operation: Some("write".to_string()),
                source: None,
            });
        }

        // Perform the write
        self.storage.write().insert(key.to_string(), value);
        Ok(())
    }

    async fn delete(&self, key: &str) -> Result<bool> {
        // Track the delete operation
        self.deletes.write().push(key.to_string());

        // Check if this delete should fail
        if self.fail_deletes.read().contains(&key.to_string()) {
            return Err(llmspell_core::LLMSpellError::Storage {
                message: format!("Mock delete failure for key: {key}"),
                operation: Some("delete".to_string()),
                source: None,
            });
        }

        // Perform the delete
        Ok(self.storage.write().remove(key).is_some())
    }

    async fn list_keys(&self, prefix: &str) -> Result<Vec<String>> {
        // Track list operation as a read
        self.reads.write().push(format!("list:{prefix}"));

        // Filter keys by prefix
        let keys: Vec<String> = self
            .storage
            .read()
            .keys()
            .filter(|k| k.starts_with(prefix))
            .cloned()
            .collect();

        Ok(keys)
    }
}

/// Create a test ExecutionContext with mock state
///
/// # Errors
///
/// Returns an error if ExecutionContext creation fails
pub fn create_test_execution_context() -> llmspell_core::ExecutionContext {
    let mock_state = Arc::new(MockStateAccess::new());
    llmspell_core::execution_context::ExecutionContextBuilder::new()
        .scope(llmspell_core::execution_context::ContextScope::Global)
        .state(mock_state)
        .build()
}

/// Create a test ExecutionContext with shared mock state
///
/// This allows tests to verify state operations performed by workflows
pub fn create_test_execution_context_with_mock(
    mock_state: Arc<MockStateAccess>,
) -> llmspell_core::ExecutionContext {
    llmspell_core::execution_context::ExecutionContextBuilder::new()
        .scope(llmspell_core::execution_context::ContextScope::Global)
        .state(mock_state)
        .build()
}

/// Create a test ExecutionContext without state (for testing None case)
#[must_use]
pub fn create_test_execution_context_no_state() -> llmspell_core::ExecutionContext {
    llmspell_core::ExecutionContext::default()
}

/// Generate large test data for performance testing
#[must_use]
pub fn generate_large_test_data(size_mb: usize) -> Value {
    let target_bytes = size_mb * 1024 * 1024;
    let chunk_size = 1000; // Create strings of ~1KB each
    let num_chunks = target_bytes / chunk_size;

    let mut data = Vec::new();
    for i in 0..num_chunks {
        data.push(Value::String("x".repeat(chunk_size)));

        // Add some variety every 100 chunks
        if i.is_multiple_of(100) {
            data.push(Value::Object({
                let mut obj = serde_json::Map::new();
                obj.insert("chunk_id".to_string(), Value::Number(i.into()));
                obj.insert(
                    "timestamp".to_string(),
                    Value::String(chrono::Utc::now().to_rfc3339()),
                );
                obj
            }));
        }
    }

    Value::Array(data)
}

/// Performance measurement utilities
pub struct PerformanceMeasurement {
    start_time: std::time::Instant,
    memory_start: Option<usize>,
}

impl PerformanceMeasurement {
    /// Start a new performance measurement
    #[must_use]
    pub fn start() -> Self {
        Self {
            start_time: std::time::Instant::now(),
            memory_start: get_memory_usage(),
        }
    }

    /// Finish measurement and return duration and memory delta
    #[must_use]
    pub fn finish(self) -> (std::time::Duration, Option<isize>) {
        let duration = self.start_time.elapsed();
        let memory_delta = if let (Some(start), Some(end)) = (self.memory_start, get_memory_usage())
        {
            Some(end as isize - start as isize)
        } else {
            None
        };
        (duration, memory_delta)
    }
}

/// Get current memory usage (best effort, platform dependent)
fn get_memory_usage() -> Option<usize> {
    #[cfg(target_os = "linux")]
    {
        use std::fs;
        let status = fs::read_to_string("/proc/self/status").ok()?;
        for line in status.lines() {
            if line.starts_with("VmRSS:") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    return parts[1].parse::<usize>().ok().map(|kb| kb * 1024);
                }
            }
        }
        None
    }
    #[cfg(target_os = "macos")]
    {
        // On macOS, we could use mach APIs but for simplicity, just return None
        // Real benchmarks should use dedicated tools
        None
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_state_access_basic_operations() {
        let mock_state = MockStateAccess::new();

        // Test write
        let test_value = serde_json::json!({"test": "data"});
        mock_state
            .write("test_key", test_value.clone())
            .await
            .unwrap();

        // Verify tracking
        assert!(mock_state.was_written("test_key"));
        assert_eq!(mock_state.get_writes().len(), 1);

        // Test read
        let read_value = mock_state.read("test_key").await.unwrap();
        assert_eq!(read_value, Some(test_value));
        assert!(mock_state.was_read("test_key"));

        // Test delete
        let deleted = mock_state.delete("test_key").await.unwrap();
        assert!(deleted);
        assert!(mock_state.was_deleted("test_key"));

        // Verify key was removed
        let read_after_delete = mock_state.read("test_key").await.unwrap();
        assert_eq!(read_after_delete, None);
    }

    #[tokio::test]
    async fn test_mock_state_access_list_keys() {
        let mock_state = MockStateAccess::new();

        // Write several keys with different prefixes
        mock_state
            .write("workflow:123:step1", serde_json::json!("data1"))
            .await
            .unwrap();
        mock_state
            .write("workflow:123:step2", serde_json::json!("data2"))
            .await
            .unwrap();
        mock_state
            .write("workflow:456:step1", serde_json::json!("data3"))
            .await
            .unwrap();
        mock_state
            .write("agent:789:config", serde_json::json!("data4"))
            .await
            .unwrap();

        // Test prefix filtering
        let workflow_123_keys = mock_state.list_keys("workflow:123").await.unwrap();
        assert_eq!(workflow_123_keys.len(), 2);
        assert!(workflow_123_keys.contains(&"workflow:123:step1".to_string()));
        assert!(workflow_123_keys.contains(&"workflow:123:step2".to_string()));

        // Test different prefix
        let agent_keys = mock_state.list_keys("agent:").await.unwrap();
        assert_eq!(agent_keys.len(), 1);
        assert!(agent_keys.contains(&"agent:789:config".to_string()));

        // Verify list operation was tracked
        assert!(mock_state
            .get_reads()
            .iter()
            .any(|r| r.starts_with("list:")));
    }

    #[tokio::test]
    async fn test_mock_state_access_failure_simulation() {
        let mock_state = MockStateAccess::new();

        // Configure read failure
        mock_state.set_read_failures(vec!["fail_key".to_string()]);

        // Test that read fails
        let result = mock_state.read("fail_key").await;
        assert!(result.is_err());

        // Test that other reads still work
        mock_state
            .write("good_key", serde_json::json!("test"))
            .await
            .unwrap();
        let result = mock_state.read("good_key").await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_execution_context_creation() {
        let context = create_test_execution_context();
        assert!(context.state.is_some());

        let context_no_state = create_test_execution_context_no_state();
        assert!(context_no_state.state.is_none());
    }

    #[test]
    fn test_large_data_generation() {
        let large_data = generate_large_test_data(1); // 1MB
        let serialized = serde_json::to_string(&large_data).unwrap();

        // Should be roughly 1MB (allowing for JSON overhead)
        assert!(serialized.len() > 900_000); // At least 900KB
        assert!(serialized.len() < 1_500_000); // Less than 1.5MB
    }

    #[test]
    fn test_performance_measurement() {
        let measurement = PerformanceMeasurement::start();

        // Simulate some work
        std::thread::sleep(std::time::Duration::from_millis(10));

        let (duration, _memory_delta) = measurement.finish();
        assert!(duration >= std::time::Duration::from_millis(10));
    }
}
