// ABOUTME: Enhanced replay manager with parameter modification and result comparison
// ABOUTME: Provides advanced replay capabilities for debugging and what-if analysis

use super::{
    BatchReplayRequest, BatchReplayResponse, HookResultComparator, ParameterModification,
    ReplayConfig, ReplayMode, ReplayResult, ReplayScheduler,
};
use crate::context::HookContext;
use crate::persistence::{HookPersistenceManager, StorageBackend};
use crate::result::HookResult;
use crate::traits::ReplayableHook;
use anyhow::{Context, Result};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};
use tracing::{debug, error};
use uuid::Uuid;

/// Replay request for a single hook execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayRequest {
    /// Execution ID to replay
    pub execution_id: Uuid,
    /// Replay configuration
    pub config: ReplayConfig,
    /// Optional correlation ID for tracking
    pub correlation_id: Option<Uuid>,
}

/// Replay response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayResponse {
    /// Replay result
    pub result: ReplayResult,
    /// Any warnings during replay
    pub warnings: Vec<String>,
}

/// Enhanced replay manager
pub struct ReplayManager {
    /// Hook persistence manager
    #[allow(dead_code)]
    persistence_manager: Arc<HookPersistenceManager>,
    /// Storage backend
    storage_backend: Arc<dyn StorageBackend>,
    /// Registered hooks for replay
    hook_registry: Arc<RwLock<HashMap<String, Arc<dyn ReplayableHook>>>>,
    /// Result comparator
    comparator: HookResultComparator,
    /// Replay scheduler
    scheduler: Arc<ReplayScheduler>,
    /// Active replays
    active_replays: Arc<RwLock<HashMap<Uuid, ReplayStatus>>>,
}

/// Status of an active replay
#[derive(Debug, Clone)]
struct ReplayStatus {
    #[allow(dead_code)]
    request: ReplayRequest,
    start_time: Instant,
    status: ReplayState,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ReplayState {
    Running,
    Completed,
    Failed(String),
    Cancelled,
}

impl ReplayManager {
    /// Create a new replay manager
    pub fn new(
        persistence_manager: Arc<HookPersistenceManager>,
        storage_backend: Arc<dyn StorageBackend>,
    ) -> Self {
        Self {
            persistence_manager,
            storage_backend,
            hook_registry: Arc::new(RwLock::new(HashMap::new())),
            comparator: HookResultComparator::new(),
            scheduler: Arc::new(ReplayScheduler::new()),
            active_replays: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a hook for replay
    pub fn register_hook(&self, hook_id: String, hook: Arc<dyn ReplayableHook>) {
        self.hook_registry.write().insert(hook_id, hook);
    }

    /// Replay a single hook execution
    pub async fn replay(&self, request: ReplayRequest) -> Result<ReplayResponse> {
        let replay_id = Uuid::new_v4();

        // Track active replay
        {
            let mut active = self.active_replays.write();
            active.insert(
                replay_id,
                ReplayStatus {
                    request: request.clone(),
                    start_time: Instant::now(),
                    status: ReplayState::Running,
                },
            );
        }

        // Execute replay
        let result = self.execute_replay(replay_id, request).await;

        // Update status
        {
            let mut active = self.active_replays.write();
            if let Some(status) = active.get_mut(&replay_id) {
                status.status = match &result {
                    Ok(_) => ReplayState::Completed,
                    Err(e) => ReplayState::Failed(e.to_string()),
                };
            }
        }

        result
    }

    /// Execute the actual replay
    async fn execute_replay(
        &self,
        replay_id: Uuid,
        request: ReplayRequest,
    ) -> Result<ReplayResponse> {
        let start_time = SystemTime::now();
        let mut warnings = Vec::new();

        // Load original execution
        let original = self
            .storage_backend
            .load_execution(&request.execution_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Execution not found: {}", request.execution_id))?;

        // Get the hook
        let hook = self
            .get_hook(&original.hook_id)?
            .ok_or_else(|| anyhow::anyhow!("Hook not registered: {}", original.hook_id))?;

        // Deserialize and modify context
        let mut context = self.deserialize_context(&hook, &original.hook_context)?;

        // Apply parameter modifications
        let applied_modifications =
            self.apply_modifications(&mut context, &request.config.modifications, &mut warnings)?;

        // Execute based on mode
        let hook_result = match request.config.mode {
            ReplayMode::Exact => {
                // Execute with original context
                self.execute_hook_with_timeout(&hook, &mut context, request.config.timeout)
                    .await
            }
            ReplayMode::Modified => {
                // Execute with modified context
                self.execute_hook_with_timeout(&hook, &mut context, request.config.timeout)
                    .await
            }
            ReplayMode::Simulate => {
                // Don't actually execute, just return original result
                let result: HookResult = serde_json::from_str(&original.result)
                    .context("Failed to deserialize original result")?;
                Ok(result)
            }
            ReplayMode::Debug => {
                // Execute with debug logging enabled
                debug!("Replay debug mode - Context: {:?}", context);
                self.execute_hook_with_timeout(&hook, &mut context, request.config.timeout)
                    .await
            }
        };

        let duration = start_time.elapsed().unwrap_or_default();

        // Compare results if requested
        let comparison = if request.config.compare_results {
            let original_result: HookResult = serde_json::from_str(&original.result)?;
            Some(self.comparator.compare(
                &original_result,
                hook_result.as_ref().unwrap_or(&HookResult::Continue),
            ))
        } else {
            None
        };

        // Build result
        let result = ReplayResult {
            replay_id,
            original_execution_id: request.execution_id,
            hook_name: original.hook_id.clone(),
            start_time,
            duration,
            hook_result: hook_result.map_err(|e| e.to_string()),
            comparison,
            applied_modifications,
            metadata: HashMap::new(),
        };

        Ok(ReplayResponse { result, warnings })
    }

    /// Apply parameter modifications to context
    fn apply_modifications(
        &self,
        context: &mut HookContext,
        modifications: &[ParameterModification],
        warnings: &mut Vec<String>,
    ) -> Result<Vec<ParameterModification>> {
        let mut applied = Vec::new();

        for modification in modifications {
            if !modification.enabled {
                continue;
            }

            match self.apply_single_modification(context, modification) {
                Ok(()) => applied.push(modification.clone()),
                Err(e) => {
                    warnings.push(format!(
                        "Failed to apply modification to {}: {}",
                        modification.path, e
                    ));
                    if applied.is_empty() {
                        // Fail if no modifications succeeded yet
                        return Err(e);
                    }
                }
            }
        }

        Ok(applied)
    }

    /// Apply a single parameter modification
    fn apply_single_modification(
        &self,
        context: &mut HookContext,
        modification: &ParameterModification,
    ) -> Result<()> {
        let parts: Vec<&str> = modification.path.split('.').collect();

        match parts.as_slice() {
            ["context", "data", key] => {
                context
                    .data
                    .insert(key.to_string(), modification.value.clone());
                Ok(())
            }
            ["context", "metadata", key] => {
                if let Value::String(value) = &modification.value {
                    context.metadata.insert(key.to_string(), value.clone());
                    Ok(())
                } else {
                    Err(anyhow::anyhow!("Metadata values must be strings"))
                }
            }
            _ => Err(anyhow::anyhow!(
                "Unsupported modification path: {}",
                modification.path
            )),
        }
    }

    /// Execute hook with timeout
    async fn execute_hook_with_timeout(
        &self,
        hook: &Arc<dyn ReplayableHook>,
        context: &mut HookContext,
        timeout: Duration,
    ) -> Result<HookResult> {
        tokio::time::timeout(timeout, hook.execute(context))
            .await
            .context("Hook execution timed out")?
    }

    /// Batch replay multiple executions
    pub async fn batch_replay(&self, request: BatchReplayRequest) -> Result<BatchReplayResponse> {
        let start_time = Instant::now();
        let mut results = Vec::new();
        let mut success_count = 0;
        let mut failure_count = 0;

        if request.parallel && request.max_concurrent > 1 {
            // Parallel execution
            use futures::stream::{self, StreamExt};

            let futures = request.executions.into_iter().map(|execution| {
                let config = request.config.clone();
                let execution_id = execution.execution_id;
                async move {
                    self.replay(ReplayRequest {
                        execution_id,
                        config,
                        correlation_id: None,
                    })
                    .await
                }
            });

            let mut stream = stream::iter(futures).buffer_unordered(request.max_concurrent);

            while let Some(result) = stream.next().await {
                match result {
                    Ok(response) => {
                        if response.result.hook_result.is_ok() {
                            success_count += 1;
                        } else {
                            failure_count += 1;
                        }
                        results.push(response.result);
                    }
                    Err(e) => {
                        failure_count += 1;
                        error!("Batch replay error: {}", e);
                        if request.config.stop_on_error {
                            break;
                        }
                    }
                }
            }
        } else {
            // Sequential execution
            for execution in request.executions {
                match self
                    .replay(ReplayRequest {
                        execution_id: execution.execution_id,
                        config: request.config.clone(),
                        correlation_id: None,
                    })
                    .await
                {
                    Ok(response) => {
                        if response.result.hook_result.is_ok() {
                            success_count += 1;
                        } else {
                            failure_count += 1;
                        }
                        results.push(response.result);
                    }
                    Err(e) => {
                        failure_count += 1;
                        error!("Batch replay error: {}", e);
                        if request.config.stop_on_error {
                            break;
                        }
                    }
                }
            }
        }

        Ok(BatchReplayResponse {
            results,
            total_duration: start_time.elapsed(),
            success_count,
            failure_count,
            metadata: HashMap::new(),
        })
    }

    /// Schedule a replay for later execution
    pub async fn schedule_replay(
        &self,
        request: ReplayRequest,
        schedule: crate::replay::ReplaySchedule,
    ) -> Result<Uuid> {
        self.scheduler.schedule(request, schedule).await
    }

    /// Get active replay status
    pub fn get_active_replays(&self) -> Vec<(Uuid, ReplayState)> {
        self.active_replays
            .read()
            .iter()
            .map(|(id, status)| (*id, status.status.clone()))
            .collect()
    }

    /// Cancel an active replay
    pub fn cancel_replay(&self, replay_id: Uuid) -> Result<()> {
        let mut active = self.active_replays.write();
        if let Some(status) = active.get_mut(&replay_id) {
            if status.status == ReplayState::Running {
                status.status = ReplayState::Cancelled;
                Ok(())
            } else {
                Err(anyhow::anyhow!("Replay is not running"))
            }
        } else {
            Err(anyhow::anyhow!("Replay not found"))
        }
    }

    /// Get a hook from the registry
    fn get_hook(&self, hook_id: &str) -> Result<Option<Arc<dyn ReplayableHook>>> {
        Ok(self.hook_registry.read().get(hook_id).cloned())
    }

    /// Deserialize hook context
    fn deserialize_context(
        &self,
        hook: &Arc<dyn ReplayableHook>,
        context_bytes: &[u8],
    ) -> Result<HookContext> {
        hook.deserialize_context(context_bytes)
    }

    /// Clean up completed replays
    pub fn cleanup_completed_replays(&self, older_than: Duration) {
        let cutoff = Instant::now() - older_than;
        self.active_replays.write().retain(|_, status| {
            status.start_time > cutoff || status.status == ReplayState::Running
        });
    }
}

#[cfg(test)]
#[cfg_attr(test_category = "hook")]
mod tests {
    use super::*;
    use crate::{ComponentId, ComponentType, HookPoint};

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_parameter_modification_parsing() {
        let modification = ParameterModification {
            path: "context.data.test_key".to_string(),
            value: serde_json::json!("test_value"),
            enabled: true,
        };

        let component_id = ComponentId::new(ComponentType::System, "test".to_string());
        let _context = HookContext::new(HookPoint::SystemStartup, component_id);

        // This test verifies the path parsing logic
        assert_eq!(
            modification.path.split('.').collect::<Vec<_>>(),
            vec!["context", "data", "test_key"]
        );
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_replay_config_builder() {
        let config = ReplayConfig {
            mode: ReplayMode::Modified,
            modifications: vec![ParameterModification {
                path: "context.data.value".to_string(),
                value: serde_json::json!(42),
                enabled: true,
            }],
            compare_results: true,
            timeout: Duration::from_secs(60),
            stop_on_error: false,
            tags: vec!["test".to_string()],
        };

        assert_eq!(config.mode, ReplayMode::Modified);
        assert_eq!(config.modifications.len(), 1);
        assert!(config.compare_results);
    }
}
