// ABOUTME: Hook persistence module providing storage and replay capabilities
// ABOUTME: Integrates with StateManager for automatic hook execution history

mod inspector;
mod replay;
mod replay_manager;
mod retention;
mod storage;
mod storage_backend;
#[cfg(test)]
#[cfg_attr(test_category = "hook")]
mod tests;

pub use inspector::{
    ComparisonResult, ExecutionAnalysis, ExecutionPattern, HookInspector, InspectionQuery,
    PatternType, ResultTypeFilter, TimeDistribution, TimeRange,
};
pub use replay::{HookReplayEngine, ReplayOptions};
pub use replay_manager::{
    BreakpointAction, BreakpointCondition, CapturedState, ReplayBreakpoint, ReplayError,
    ReplayErrorType, ReplayManager, ReplaySession, ReplaySessionConfig, ReplayTimeline,
    TimelineEntry,
};
pub use retention::{RetentionManager, RetentionPolicy, RetentionStatistics};
pub use storage::{HookMetadata, HookStorageAdapter, StorageStatistics};
pub use storage_backend::{
    FileStorageBackend, InMemoryStorageBackend, StorageBackend, StorageStats,
};

use crate::context::HookContext;
use crate::result::HookResult;
use crate::traits::ReplayableHook;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
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

/// Interface for hook replay management
#[async_trait::async_trait]
pub trait HookReplayManager: Send + Sync {
    /// Persist a hook execution
    async fn persist_hook_execution(
        &self,
        hook: &dyn ReplayableHook,
        context: &HookContext,
        result: &HookResult,
        duration: Duration,
    ) -> Result<()>;

    /// Get hook executions by correlation ID
    async fn get_hook_executions_by_correlation(
        &self,
        correlation_id: Uuid,
    ) -> Result<Vec<SerializedHookExecution>>;
}

/// Enhanced hook persistence manager
pub struct HookPersistenceManager {
    /// Underlying replay manager implementation
    #[allow(dead_code)]
    replay_manager: Arc<dyn HookReplayManager>,
    /// Hook-specific storage adapter
    storage_adapter: HookStorageAdapter,
    /// Retention policy manager
    retention_manager: RetentionManager,
    /// Storage backend for persistence
    storage_backend: Arc<dyn StorageBackend>,
}

impl HookPersistenceManager {
    /// Create a new hook persistence manager
    pub fn new(replay_manager: Arc<dyn HookReplayManager>) -> Self {
        let storage_adapter = HookStorageAdapter::new();
        let retention_manager = RetentionManager::default();
        let storage_backend = Arc::new(InMemoryStorageBackend::new());

        Self {
            replay_manager,
            storage_adapter,
            retention_manager,
            storage_backend,
        }
    }

    /// Create with a specific storage backend
    pub fn with_storage_backend(
        replay_manager: Arc<dyn HookReplayManager>,
        storage_backend: Arc<dyn StorageBackend>,
    ) -> Self {
        let storage_adapter = HookStorageAdapter::new();
        let retention_manager = RetentionManager::default();

        Self {
            replay_manager,
            storage_adapter,
            retention_manager,
            storage_backend,
        }
    }

    /// Persist a hook execution with metadata and retention policies
    pub async fn persist_execution(
        &self,
        hook: &dyn ReplayableHook,
        context: &HookContext,
        result: &HookResult,
        duration: Duration,
        metadata: HookMetadata,
    ) -> Result<()> {
        // Create serialized execution
        let execution = SerializedHookExecution {
            hook_id: hook.replay_id(),
            execution_id: Uuid::new_v4(),
            correlation_id: context.correlation_id,
            hook_context: hook.serialize_context(context)?,
            result: serde_json::to_string(result)?,
            timestamp: SystemTime::now(),
            duration,
            metadata: context.data.clone(),
        };

        // Store execution in backend
        self.storage_backend
            .store_execution(&execution, &metadata)
            .await?;

        // Store additional hook-specific metadata
        self.storage_adapter
            .store_metadata(&context.correlation_id, &metadata)
            .await?;

        // Apply retention policies
        self.retention_manager
            .apply_retention_policies(&hook.replay_id())
            .await?;

        // Cleanup based on retention policies
        let policy = self.retention_manager.get_policy(&hook.replay_id());
        if let Some(max_age) = policy.max_age {
            self.storage_backend.cleanup(None, Some(max_age)).await?;
        }

        Ok(())
    }

    /// Get hook executions with enhanced metadata
    pub async fn get_executions_with_metadata(
        &self,
        correlation_id: Uuid,
    ) -> Result<Vec<(SerializedHookExecution, HookMetadata)>> {
        // Load executions from storage backend
        let executions = self
            .storage_backend
            .load_executions_by_correlation(&correlation_id)
            .await?;

        let mut results = Vec::new();
        for execution in executions {
            if let Some(metadata) = self.storage_adapter.load_metadata(&correlation_id).await? {
                results.push((execution, metadata));
            }
        }

        Ok(results)
    }

    /// Get storage statistics
    pub async fn get_storage_statistics(&self) -> Result<StorageStats> {
        self.storage_backend.get_statistics().await
    }

    /// Archive old executions
    pub async fn archive_old_executions(&self, older_than: SystemTime) -> Result<u64> {
        self.storage_backend.archive_executions(older_than).await
    }

    /// Configure retention policy for a specific hook type
    pub fn configure_retention(&mut self, hook_id: &str, policy: RetentionPolicy) {
        self.retention_manager.set_policy(hook_id, policy);
    }
}
