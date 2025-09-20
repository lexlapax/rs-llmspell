// ABOUTME: Storage backend for persisting hook executions with compression and archiving
// ABOUTME: Provides efficient storage operations with configurable backends

use crate::persistence::{HookMetadata, SerializedHookExecution};
use anyhow::{Context, Result};
use async_trait::async_trait;
use bytes::Bytes;
use flate2::{read::GzDecoder, write::GzEncoder, Compression};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::fs;
use tracing::{debug, info, warn};
use uuid::Uuid;

/// Storage backend trait for hook persistence
#[async_trait]
pub trait StorageBackend: Send + Sync + std::fmt::Debug {
    /// Store a hook execution
    async fn store_execution(
        &self,
        execution: &SerializedHookExecution,
        metadata: &HookMetadata,
    ) -> Result<()>;

    /// Load a hook execution by ID
    async fn load_execution(&self, execution_id: &Uuid) -> Result<Option<SerializedHookExecution>>;

    /// Load all executions for a correlation ID
    async fn load_executions_by_correlation(
        &self,
        correlation_id: &Uuid,
    ) -> Result<Vec<SerializedHookExecution>>;

    /// Delete an execution
    async fn delete_execution(&self, execution_id: &Uuid) -> Result<()>;

    /// Archive old executions
    async fn archive_executions(&self, older_than: SystemTime) -> Result<u64>;

    /// Get storage statistics
    async fn get_statistics(&self) -> Result<StorageStats>;

    /// Cleanup storage based on retention policies
    async fn cleanup(&self, max_size: Option<u64>, max_age: Option<Duration>) -> Result<u64>;
}

/// Storage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageStats {
    pub total_executions: u64,
    pub total_size_bytes: u64,
    pub compressed_size_bytes: u64,
    pub compression_ratio: f64,
    pub oldest_execution: Option<SystemTime>,
    pub newest_execution: Option<SystemTime>,
    pub executions_by_hook: HashMap<String, u64>,
}

/// Execution storage entry
type ExecutionEntry = (SerializedHookExecution, HookMetadata, Bytes);

/// In-memory storage backend (for testing and development)
#[derive(Debug)]
pub struct InMemoryStorageBackend {
    executions: Arc<RwLock<HashMap<Uuid, ExecutionEntry>>>,
    correlation_index: Arc<RwLock<HashMap<Uuid, Vec<Uuid>>>>,
    stats: Arc<RwLock<StorageStats>>,
}

impl InMemoryStorageBackend {
    pub fn new() -> Self {
        Self {
            executions: Arc::new(RwLock::new(HashMap::new())),
            correlation_index: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(RwLock::new(StorageStats {
                total_executions: 0,
                total_size_bytes: 0,
                compressed_size_bytes: 0,
                compression_ratio: 1.0,
                oldest_execution: None,
                newest_execution: None,
                executions_by_hook: HashMap::new(),
            })),
        }
    }

    fn compress_data(data: &[u8]) -> Result<Bytes> {
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder
            .write_all(data)
            .context("Failed to write to compressor")?;
        let compressed = encoder.finish().context("Failed to finish compression")?;
        Ok(Bytes::from(compressed))
    }

    fn decompress_data(data: &[u8]) -> Result<Vec<u8>> {
        let mut decoder = GzDecoder::new(data);
        let mut decompressed = Vec::new();
        decoder
            .read_to_end(&mut decompressed)
            .context("Failed to decompress data")?;
        Ok(decompressed)
    }
}

impl Default for InMemoryStorageBackend {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl StorageBackend for InMemoryStorageBackend {
    async fn store_execution(
        &self,
        execution: &SerializedHookExecution,
        metadata: &HookMetadata,
    ) -> Result<()> {
        // Serialize execution
        let serialized = serde_json::to_vec(execution).context("Failed to serialize execution")?;
        let original_size = serialized.len() as u64;

        // Compress data
        let compressed = Self::compress_data(&serialized)?;
        let compressed_size = compressed.len() as u64;

        // Update storage
        {
            let mut executions = self.executions.write();
            executions.insert(
                execution.execution_id,
                (execution.clone(), metadata.clone(), compressed),
            );
        }

        // Update correlation index
        {
            let mut index = self.correlation_index.write();
            index
                .entry(execution.correlation_id)
                .or_default()
                .push(execution.execution_id);
        }

        // Update statistics
        {
            let mut stats = self.stats.write();
            stats.total_executions += 1;
            stats.total_size_bytes += original_size;
            stats.compressed_size_bytes += compressed_size;
            #[allow(clippy::cast_precision_loss)]
            let compressed_f64 = stats.compressed_size_bytes as f64;
            #[allow(clippy::cast_precision_loss)]
            let total_f64 = stats.total_size_bytes as f64;
            stats.compression_ratio = compressed_f64 / total_f64;

            // Update timestamps
            if stats.oldest_execution.is_none()
                || execution.timestamp < stats.oldest_execution.unwrap()
            {
                stats.oldest_execution = Some(execution.timestamp);
            }
            stats.newest_execution = Some(execution.timestamp);

            // Update hook counts
            *stats
                .executions_by_hook
                .entry(execution.hook_id.clone())
                .or_insert(0) += 1;
        }

        debug!(
            execution_id = %execution.execution_id,
            hook_id = %execution.hook_id,
            original_size,
            compressed_size,
            "Stored hook execution"
        );

        Ok(())
    }

    async fn load_execution(&self, execution_id: &Uuid) -> Result<Option<SerializedHookExecution>> {
        let executions = self.executions.read();
        if let Some((execution, _, _compressed)) = executions.get(execution_id) {
            // For in-memory backend, we already have the deserialized execution
            Ok(Some(execution.clone()))
        } else {
            Ok(None)
        }
    }

    async fn load_executions_by_correlation(
        &self,
        correlation_id: &Uuid,
    ) -> Result<Vec<SerializedHookExecution>> {
        let index = self.correlation_index.read();
        if let Some(execution_ids) = index.get(correlation_id) {
            let executions = self.executions.read();
            let mut results = Vec::new();

            for exec_id in execution_ids {
                if let Some((execution, _, _)) = executions.get(exec_id) {
                    results.push(execution.clone());
                }
            }

            // Sort by timestamp
            results.sort_by_key(|e| e.timestamp);
            Ok(results)
        } else {
            Ok(Vec::new())
        }
    }

    async fn delete_execution(&self, execution_id: &Uuid) -> Result<()> {
        let removed = {
            let mut executions = self.executions.write();
            executions.remove(execution_id)
        };

        if let Some((execution, _, compressed)) = removed {
            // Update correlation index
            {
                let mut index = self.correlation_index.write();
                if let Some(exec_ids) = index.get_mut(&execution.correlation_id) {
                    exec_ids.retain(|id| id != execution_id);
                    if exec_ids.is_empty() {
                        index.remove(&execution.correlation_id);
                    }
                }
            }

            // Update statistics
            {
                let mut stats = self.stats.write();
                stats.total_executions = stats.total_executions.saturating_sub(1);

                let original_size = serde_json::to_vec(&execution)?.len() as u64;
                stats.total_size_bytes = stats.total_size_bytes.saturating_sub(original_size);
                stats.compressed_size_bytes = stats
                    .compressed_size_bytes
                    .saturating_sub(compressed.len() as u64);

                if stats.total_size_bytes > 0 {
                    #[allow(clippy::cast_precision_loss)]
                    let compressed_f64 = stats.compressed_size_bytes as f64;
                    #[allow(clippy::cast_precision_loss)]
                    let total_f64 = stats.total_size_bytes as f64;
                    stats.compression_ratio = compressed_f64 / total_f64;
                }

                if let Some(count) = stats.executions_by_hook.get_mut(&execution.hook_id) {
                    *count = count.saturating_sub(1);
                    if *count == 0 {
                        stats.executions_by_hook.remove(&execution.hook_id);
                    }
                }
            }
        }

        Ok(())
    }

    async fn archive_executions(&self, older_than: SystemTime) -> Result<u64> {
        let mut to_archive = Vec::new();

        {
            let executions = self.executions.read();
            for (id, (execution, _, _)) in executions.iter() {
                if execution.timestamp < older_than {
                    to_archive.push(*id);
                }
            }
        }

        let archived_count = to_archive.len() as u64;

        // In a real implementation, we would move these to archive storage
        // For now, we just delete them
        for id in to_archive {
            self.delete_execution(&id).await?;
        }

        info!(archived_count, "Archived old executions");
        Ok(archived_count)
    }

    async fn get_statistics(&self) -> Result<StorageStats> {
        Ok(self.stats.read().clone())
    }

    async fn cleanup(&self, max_size: Option<u64>, max_age: Option<Duration>) -> Result<u64> {
        let mut to_remove = Vec::new();
        let now = SystemTime::now();

        {
            let executions = self.executions.read();
            let mut entries: Vec<_> = executions
                .iter()
                .map(|(id, (exec, _, compressed))| (*id, exec.timestamp, compressed.len() as u64))
                .collect();

            // Sort by timestamp (oldest first)
            entries.sort_by_key(|(_, timestamp, _)| *timestamp);

            // Remove based on age
            if let Some(max_age) = max_age {
                let cutoff = now - max_age;
                for (id, timestamp, _) in &entries {
                    if *timestamp < cutoff {
                        to_remove.push(*id);
                    }
                }
            }

            // Remove based on size
            if let Some(max_size) = max_size {
                let stats = self.stats.read();
                let mut current_size = stats.compressed_size_bytes;

                for (id, _, size) in entries.iter().rev() {
                    if current_size <= max_size {
                        break;
                    }
                    if !to_remove.contains(id) {
                        to_remove.push(*id);
                        current_size -= size;
                    }
                }
            }
        }

        let removed_count = to_remove.len() as u64;

        for id in to_remove {
            self.delete_execution(&id).await?;
        }

        Ok(removed_count)
    }
}

/// File-based storage backend
#[derive(Debug)]
pub struct FileStorageBackend {
    root_path: PathBuf,
    compression_enabled: bool,
    index: Arc<RwLock<HashMap<Uuid, PathBuf>>>,
    correlation_index: Arc<RwLock<HashMap<Uuid, Vec<Uuid>>>>,
}

impl FileStorageBackend {
    pub async fn new(root_path: impl AsRef<Path>, compression_enabled: bool) -> Result<Self> {
        let root_path = root_path.as_ref().to_path_buf();

        // Create directory structure
        fs::create_dir_all(&root_path)
            .await
            .context("Failed to create storage directory")?;

        let executions_dir = root_path.join("executions");
        fs::create_dir_all(&executions_dir)
            .await
            .context("Failed to create executions directory")?;

        let archive_dir = root_path.join("archive");
        fs::create_dir_all(&archive_dir)
            .await
            .context("Failed to create archive directory")?;

        Ok(Self {
            root_path,
            compression_enabled,
            index: Arc::new(RwLock::new(HashMap::new())),
            correlation_index: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    fn get_execution_path(&self, execution_id: &Uuid) -> PathBuf {
        let id_str = execution_id.to_string();
        let prefix = &id_str[..2];
        let extension = if self.compression_enabled {
            "json.gz"
        } else {
            "json"
        };

        self.root_path
            .join("executions")
            .join(prefix)
            .join(format!("{}.{}", id_str, extension))
    }

    async fn write_execution_file(
        &self,
        path: &Path,
        execution: &SerializedHookExecution,
    ) -> Result<()> {
        // Create parent directory
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .await
                .context("Failed to create execution directory")?;
        }

        let data = serde_json::to_vec(execution).context("Failed to serialize execution")?;

        if self.compression_enabled {
            let compressed = InMemoryStorageBackend::compress_data(&data)?;
            fs::write(path, &compressed)
                .await
                .context("Failed to write compressed execution")?;
        } else {
            fs::write(path, &data)
                .await
                .context("Failed to write execution")?;
        }

        Ok(())
    }

    async fn read_execution_file(&self, path: &Path) -> Result<SerializedHookExecution> {
        let data = fs::read(path)
            .await
            .context("Failed to read execution file")?;

        let json_data = if self.compression_enabled {
            InMemoryStorageBackend::decompress_data(&data)?
        } else {
            data
        };

        serde_json::from_slice(&json_data).context("Failed to deserialize execution")
    }
}

#[async_trait]
impl StorageBackend for FileStorageBackend {
    async fn store_execution(
        &self,
        execution: &SerializedHookExecution,
        metadata: &HookMetadata,
    ) -> Result<()> {
        let path = self.get_execution_path(&execution.execution_id);

        // Write execution file
        self.write_execution_file(&path, execution).await?;

        // Update indices
        {
            let mut index = self.index.write();
            index.insert(execution.execution_id, path.clone());
        }

        {
            let mut corr_index = self.correlation_index.write();
            corr_index
                .entry(execution.correlation_id)
                .or_default()
                .push(execution.execution_id);
        }

        // Write metadata file
        let metadata_path = path.with_extension("metadata.json");
        let metadata_data = serde_json::to_vec_pretty(metadata)?;
        fs::write(&metadata_path, metadata_data)
            .await
            .context("Failed to write metadata")?;

        Ok(())
    }

    async fn load_execution(&self, execution_id: &Uuid) -> Result<Option<SerializedHookExecution>> {
        let path = {
            let index = self.index.read();
            index.get(execution_id).cloned()
        };

        if let Some(path) = path {
            match self.read_execution_file(&path).await {
                Ok(execution) => Ok(Some(execution)),
                Err(e) => {
                    warn!(
                        execution_id = %execution_id,
                        error = %e,
                        "Failed to load execution"
                    );
                    Ok(None)
                }
            }
        } else {
            Ok(None)
        }
    }

    async fn load_executions_by_correlation(
        &self,
        correlation_id: &Uuid,
    ) -> Result<Vec<SerializedHookExecution>> {
        let execution_ids = {
            let corr_index = self.correlation_index.read();
            corr_index.get(correlation_id).cloned()
        };

        if let Some(execution_ids) = execution_ids {
            let mut results = Vec::new();

            for exec_id in execution_ids {
                if let Some(execution) = self.load_execution(&exec_id).await? {
                    results.push(execution);
                }
            }

            // Sort by timestamp
            results.sort_by_key(|e| e.timestamp);
            Ok(results)
        } else {
            Ok(Vec::new())
        }
    }

    async fn delete_execution(&self, execution_id: &Uuid) -> Result<()> {
        let path = {
            let mut index = self.index.write();
            index.remove(execution_id)
        };

        if let Some(path) = path {
            // Remove from correlation index
            {
                let mut corr_index = self.correlation_index.write();
                // We need to scan all correlations to find this execution
                corr_index.retain(|_, exec_ids| {
                    exec_ids.retain(|id| id != execution_id);
                    !exec_ids.is_empty()
                });
            }

            // Delete files
            if path.exists() {
                fs::remove_file(&path)
                    .await
                    .context("Failed to delete execution file")?;
            }

            let metadata_path = path.with_extension("metadata.json");
            if metadata_path.exists() {
                fs::remove_file(&metadata_path)
                    .await
                    .context("Failed to delete metadata file")?;
            }
        }

        Ok(())
    }

    async fn archive_executions(&self, _older_than: SystemTime) -> Result<u64> {
        // Implementation would move old files to archive directory
        // For brevity, returning 0
        Ok(0)
    }

    async fn get_statistics(&self) -> Result<StorageStats> {
        // Implementation would scan files to gather statistics
        // For brevity, returning empty stats
        Ok(StorageStats {
            total_executions: self.index.read().len() as u64,
            total_size_bytes: 0,
            compressed_size_bytes: 0,
            compression_ratio: 1.0,
            oldest_execution: None,
            newest_execution: None,
            executions_by_hook: HashMap::new(),
        })
    }

    async fn cleanup(&self, _max_size: Option<u64>, _max_age: Option<Duration>) -> Result<u64> {
        // Implementation would scan and remove old/large files
        // For brevity, returning 0
        Ok(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::ComponentType;

    fn create_test_execution() -> (SerializedHookExecution, HookMetadata) {
        let execution = SerializedHookExecution {
            hook_id: "test_hook".to_string(),
            execution_id: Uuid::new_v4(),
            correlation_id: Uuid::new_v4(),
            hook_context: vec![1, 2, 3, 4],
            result: "HookResult::Continue".to_string(),
            timestamp: SystemTime::now(),
            duration: Duration::from_millis(100),
            metadata: HashMap::new(),
        };

        let metadata = HookMetadata::new(
            "test_hook".to_string(),
            ComponentType::Agent,
            "test_agent".to_string(),
        );

        (execution, metadata)
    }
    #[tokio::test]
    async fn test_in_memory_storage() {
        let backend = InMemoryStorageBackend::new();
        let (execution, metadata) = create_test_execution();

        // Store execution
        backend
            .store_execution(&execution, &metadata)
            .await
            .unwrap();

        // Load by ID
        let loaded = backend
            .load_execution(&execution.execution_id)
            .await
            .unwrap();
        assert!(loaded.is_some());
        assert_eq!(loaded.unwrap().hook_id, execution.hook_id);

        // Load by correlation
        let correlated = backend
            .load_executions_by_correlation(&execution.correlation_id)
            .await
            .unwrap();
        assert_eq!(correlated.len(), 1);

        // Check statistics
        let stats = backend.get_statistics().await.unwrap();
        assert_eq!(stats.total_executions, 1);
        assert!(stats.compression_ratio < 1.0); // Should be compressed

        // Delete execution
        backend
            .delete_execution(&execution.execution_id)
            .await
            .unwrap();

        let loaded = backend
            .load_execution(&execution.execution_id)
            .await
            .unwrap();
        assert!(loaded.is_none());
    }
    #[tokio::test]
    async fn test_compression() {
        // Use larger, repetitive data that compresses well
        let data = "Hello, world! This is a test of compression. ".repeat(100);
        let data_bytes = data.as_bytes();

        let compressed = InMemoryStorageBackend::compress_data(data_bytes).unwrap();
        let decompressed = InMemoryStorageBackend::decompress_data(&compressed).unwrap();

        assert_eq!(data_bytes, decompressed.as_slice());
        // With repetitive data, compression should reduce size significantly
        assert!(
            compressed.len() < data_bytes.len(),
            "Compressed size {} should be less than original size {}",
            compressed.len(),
            data_bytes.len()
        );

        // Also test that compression actually provides good ratio for this data
        let compression_ratio = compressed.len() as f64 / data_bytes.len() as f64;
        assert!(
            compression_ratio < 0.5,
            "Compression ratio {} should be less than 0.5",
            compression_ratio
        );
    }
    #[tokio::test]
    async fn test_cleanup_by_age() {
        let backend = InMemoryStorageBackend::new();

        // Create old execution
        let (mut old_execution, metadata) = create_test_execution();
        old_execution.timestamp = SystemTime::now() - Duration::from_secs(3600);
        backend
            .store_execution(&old_execution, &metadata)
            .await
            .unwrap();

        // Create recent execution
        let (recent_execution, metadata) = create_test_execution();
        backend
            .store_execution(&recent_execution, &metadata)
            .await
            .unwrap();

        // Cleanup executions older than 30 minutes
        let removed = backend
            .cleanup(None, Some(Duration::from_secs(1800)))
            .await
            .unwrap();

        assert_eq!(removed, 1);

        // Verify old execution was removed
        assert!(backend
            .load_execution(&old_execution.execution_id)
            .await
            .unwrap()
            .is_none());
        // Verify recent execution remains
        assert!(backend
            .load_execution(&recent_execution.execution_id)
            .await
            .unwrap()
            .is_some());
    }
}
