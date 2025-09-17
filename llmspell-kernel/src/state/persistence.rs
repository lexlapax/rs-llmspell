//! State persistence layer

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::SystemTime;
use tracing::{debug, info, instrument};
use tracing::field::Empty;

use super::types::{DebugState, ExecutionState, SessionState};

/// State persistence trait for async storage operations
#[async_trait]
pub trait StatePersistence: Send + Sync {
    /// Save state to storage
    async fn save_state(&self, state: &KernelStateSnapshot) -> Result<()>;

    /// Load state from storage
    async fn load_state(&self) -> Result<Option<KernelStateSnapshot>>;

    /// Check if state exists
    async fn state_exists(&self) -> Result<bool>;

    /// Delete state from storage
    async fn delete_state(&self) -> Result<()>;

    /// List available state snapshots
    async fn list_snapshots(&self) -> Result<Vec<SnapshotInfo>>;

    /// Save a named snapshot
    async fn save_snapshot(&self, name: &str, state: &KernelStateSnapshot) -> Result<()>;

    /// Load a named snapshot
    async fn load_snapshot(&self, name: &str) -> Result<Option<KernelStateSnapshot>>;
}

/// Complete kernel state snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KernelStateSnapshot {
    /// Execution state
    pub execution: ExecutionState,
    /// Session state
    pub session: SessionState,
    /// Debug state
    pub debug: DebugState,
    /// Timestamp of snapshot
    pub timestamp: SystemTime,
    /// Snapshot metadata
    pub metadata: HashMap<String, String>,
}

impl KernelStateSnapshot {
    /// Create a new snapshot
    pub fn new(execution: ExecutionState, session: SessionState, debug: DebugState) -> Self {
        Self {
            execution,
            session,
            debug,
            timestamp: SystemTime::now(),
            metadata: HashMap::new(),
        }
    }

    /// Add metadata to snapshot
    #[must_use]
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }
}

/// Information about a state snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotInfo {
    /// Snapshot name
    pub name: String,
    /// Creation timestamp
    pub timestamp: SystemTime,
    /// Size in bytes
    pub size: u64,
    /// Metadata
    pub metadata: HashMap<String, String>,
}

/// File-based state persistence implementation
pub struct FilePersistence {
    /// Base directory for state files
    base_path: PathBuf,
    /// File extension for state files
    extension: String,
}

impl FilePersistence {
    /// Create a new file persistence
    ///
    /// # Errors
    ///
    /// Returns an error if the directory cannot be created
    pub fn new(base_path: PathBuf) -> Result<Self> {
        // Ensure directory exists
        std::fs::create_dir_all(&base_path)?;

        Ok(Self {
            base_path,
            extension: "state".to_string(),
        })
    }

    /// Get the path for the main state file
    fn state_path(&self) -> PathBuf {
        self.base_path.join(format!("kernel.{}", self.extension))
    }

    /// Get the path for a snapshot file
    fn snapshot_path(&self, name: &str) -> PathBuf {
        self.base_path
            .join("snapshots")
            .join(format!("{}.{}", name, self.extension))
    }
}

#[async_trait]
impl StatePersistence for FilePersistence {
    #[instrument(skip_all)]
    async fn save_state(&self, state: &KernelStateSnapshot) -> Result<()> {
        let path = self.state_path();
        let data = serde_json::to_vec_pretty(state)?;

        tokio::fs::write(&path, data).await?;
        info!("State saved to {:?}", path);

        Ok(())
    }

    #[instrument(skip_all)]
    async fn load_state(&self) -> Result<Option<KernelStateSnapshot>> {
        let path = self.state_path();

        if !path.exists() {
            return Ok(None);
        }

        let data = tokio::fs::read(&path).await?;
        let state = serde_json::from_slice(&data)?;

        info!("State loaded from {:?}", path);
        Ok(Some(state))
    }

    #[instrument(level = "trace", skip_all)]
    async fn state_exists(&self) -> Result<bool> {
        Ok(self.state_path().exists())
    }

    #[instrument(level = "debug", skip_all)]
    async fn delete_state(&self) -> Result<()> {
        let path = self.state_path();

        if path.exists() {
            tokio::fs::remove_file(&path).await?;
            info!("State deleted from {:?}", path);
        }

        Ok(())
    }

    #[instrument(level = "debug", skip_all, fields(snapshot_count = Empty))]
    async fn list_snapshots(&self) -> Result<Vec<SnapshotInfo>> {
        let snapshots_dir = self.base_path.join("snapshots");

        if !snapshots_dir.exists() {
            return Ok(Vec::new());
        }

        let mut snapshots = Vec::new();
        let mut entries = tokio::fs::read_dir(&snapshots_dir).await?;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some(&self.extension) {
                if let Ok(metadata) = entry.metadata().await {
                    if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                        snapshots.push(SnapshotInfo {
                            name: name.to_string(),
                            timestamp: metadata.modified()?,
                            size: metadata.len(),
                            metadata: HashMap::new(),
                        });
                    }
                }
            }
        }

        tracing::Span::current().record("snapshot_count", snapshots.len());
        debug!("Found {} snapshots", snapshots.len());
        Ok(snapshots)
    }

    #[instrument(level = "info", skip(self, state), fields(snapshot_name = %name))]
    async fn save_snapshot(&self, name: &str, state: &KernelStateSnapshot) -> Result<()> {
        let path = self.snapshot_path(name);

        // Ensure snapshots directory exists
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        let data = serde_json::to_vec_pretty(state)?;
        tokio::fs::write(&path, data).await?;

        info!("Snapshot '{}' saved to {:?}", name, path);
        Ok(())
    }

    #[instrument(level = "info", skip_all, fields(snapshot_name = %name))]
    async fn load_snapshot(&self, name: &str) -> Result<Option<KernelStateSnapshot>> {
        let path = self.snapshot_path(name);

        if !path.exists() {
            return Ok(None);
        }

        let data = tokio::fs::read(&path).await?;
        let state = serde_json::from_slice(&data)?;

        info!("Snapshot '{}' loaded from {:?}", name, path);
        Ok(Some(state))
    }
}

/// Memory-based state persistence (for testing)
pub struct MemoryPersistence {
    /// In-memory storage
    storage: tokio::sync::RwLock<HashMap<String, KernelStateSnapshot>>,
}

impl MemoryPersistence {
    /// Create a new memory persistence
    pub fn new() -> Self {
        Self {
            storage: tokio::sync::RwLock::new(HashMap::new()),
        }
    }
}

impl Default for MemoryPersistence {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl StatePersistence for MemoryPersistence {
    #[instrument(level = "trace", skip_all)]
    async fn save_state(&self, state: &KernelStateSnapshot) -> Result<()> {
        let mut storage = self.storage.write().await;
        storage.insert("main".to_string(), state.clone());
        Ok(())
    }

    #[instrument(level = "trace", skip_all)]
    async fn load_state(&self) -> Result<Option<KernelStateSnapshot>> {
        let storage = self.storage.read().await;
        Ok(storage.get("main").cloned())
    }

    #[instrument(level = "trace", skip_all)]
    async fn state_exists(&self) -> Result<bool> {
        let storage = self.storage.read().await;
        Ok(storage.contains_key("main"))
    }

    #[instrument(level = "trace", skip_all)]
    async fn delete_state(&self) -> Result<()> {
        let mut storage = self.storage.write().await;
        storage.remove("main");
        Ok(())
    }

    async fn list_snapshots(&self) -> Result<Vec<SnapshotInfo>> {
        let storage = self.storage.read().await;
        let snapshots = storage
            .iter()
            .filter(|(k, _)| k != &"main")
            .map(|(name, snapshot)| SnapshotInfo {
                name: name.clone(),
                timestamp: snapshot.timestamp,
                size: 0, // Not applicable for memory storage
                metadata: snapshot.metadata.clone(),
            })
            .collect::<Vec<_>>();

        tracing::Span::current().record("snapshot_count", snapshots.len());
        Ok(snapshots)
    }

    #[instrument(level = "trace", skip(self, state), fields(snapshot_name = %name))]
    async fn save_snapshot(&self, name: &str, state: &KernelStateSnapshot) -> Result<()> {
        let mut storage = self.storage.write().await;
        storage.insert(name.to_string(), state.clone());
        Ok(())
    }

    #[instrument(level = "trace", skip_all, fields(snapshot_name = %name))]
    async fn load_snapshot(&self, name: &str) -> Result<Option<KernelStateSnapshot>> {
        let storage = self.storage.read().await;
        Ok(storage.get(name).cloned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_memory_persistence() {
        let persistence = MemoryPersistence::new();

        let snapshot = KernelStateSnapshot::new(
            ExecutionState::default(),
            SessionState::default(),
            DebugState::default(),
        );

        // Save and load state
        persistence.save_state(&snapshot).await.unwrap();
        assert!(persistence.state_exists().await.unwrap());

        let loaded = persistence.load_state().await.unwrap().unwrap();
        assert_eq!(
            loaded.execution.execution_count,
            snapshot.execution.execution_count
        );

        // Delete state
        persistence.delete_state().await.unwrap();
        assert!(!persistence.state_exists().await.unwrap());
    }

    #[tokio::test]
    async fn test_snapshots() {
        let persistence = MemoryPersistence::new();

        let snapshot = KernelStateSnapshot::new(
            ExecutionState::default(),
            SessionState::default(),
            DebugState::default(),
        );

        // Save snapshots
        persistence.save_snapshot("test1", &snapshot).await.unwrap();
        persistence.save_snapshot("test2", &snapshot).await.unwrap();

        // List snapshots
        let snapshots = persistence.list_snapshots().await.unwrap();
        assert_eq!(snapshots.len(), 2);

        // Load snapshot
        let loaded = persistence.load_snapshot("test1").await.unwrap().unwrap();
        assert_eq!(
            loaded.execution.execution_count,
            snapshot.execution.execution_count
        );
    }
}
