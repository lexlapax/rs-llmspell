// ABOUTME: Hook replay functionality for debugging and testing
// ABOUTME: Allows reconstruction and replay of hook executions from storage

use crate::persistence::SerializedHookExecution;
use crate::result::HookResult;
use crate::traits::ReplayableHook;
use anyhow::{Context as AnyhowContext, Result};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::{debug, info};

/// Options for replaying hook executions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayOptions {
    /// Whether to modify parameters during replay
    pub modify_parameters: bool,
    /// Custom parameters to use during replay
    pub custom_parameters: Option<serde_json::Value>,
    /// Whether to simulate the original timing
    pub simulate_timing: bool,
    /// Whether to skip side effects
    pub dry_run: bool,
    /// Maximum executions to replay
    pub max_executions: Option<usize>,
    /// Filter by hook types
    pub hook_type_filter: Option<Vec<String>>,
}

impl Default for ReplayOptions {
    fn default() -> Self {
        Self {
            modify_parameters: false,
            custom_parameters: None,
            simulate_timing: false,
            dry_run: true,
            max_executions: None,
            hook_type_filter: None,
        }
    }
}

/// Hook replay engine for reconstructing executions
pub struct HookReplayEngine {
    /// Replay statistics
    stats: ReplayStatistics,
}

#[derive(Debug, Default)]
struct ReplayStatistics {
    total_replayed: u64,
    successful_replays: u64,
    failed_replays: u64,
    total_replay_time: Duration,
}

impl HookReplayEngine {
    /// Create a new replay engine
    pub fn new() -> Self {
        Self {
            stats: ReplayStatistics::default(),
        }
    }

    /// Replay a single hook execution
    pub async fn replay_execution(
        &mut self,
        hook: &dyn ReplayableHook,
        execution: &SerializedHookExecution,
        options: &ReplayOptions,
    ) -> Result<HookResult> {
        info!(
            hook_id = execution.hook_id,
            execution_id = %execution.execution_id,
            "Replaying hook execution"
        );

        let start = std::time::Instant::now();

        // Deserialize the hook context
        let mut context = hook
            .deserialize_context(&execution.hook_context)
            .context("Failed to deserialize hook context")?;

        // Modify parameters if requested
        if options.modify_parameters {
            if let Some(ref params) = options.custom_parameters {
                context.insert_data("custom_params".to_string(), params.clone());
            }
        }

        // Simulate timing if requested
        if options.simulate_timing {
            debug!(
                "Simulating original execution time: {:?}",
                execution.duration
            );
            tokio::time::sleep(execution.duration).await;
        }

        // Execute the hook
        let result = if options.dry_run {
            // In dry run mode, return the original result
            serde_json::from_str(&execution.result)
                .context("Failed to deserialize original result")?
        } else {
            // Actually execute the hook
            hook.execute(&mut context).await?
        };

        // Update statistics
        self.stats.total_replayed += 1;
        match &result {
            HookResult::Continue | HookResult::Modified(_) | HookResult::Replace(_) => {
                self.stats.successful_replays += 1;
            }
            _ => {}
        }
        self.stats.total_replay_time += start.elapsed();

        Ok(result)
    }

    /// Replay multiple hook executions
    pub async fn replay_executions(
        &mut self,
        hooks: &[(Box<dyn ReplayableHook>, SerializedHookExecution)],
        options: &ReplayOptions,
    ) -> Result<Vec<HookResult>> {
        let mut results = Vec::new();
        let max = options.max_executions.unwrap_or(hooks.len());

        for (hook, execution) in hooks.iter().take(max) {
            // Apply hook type filter if specified
            if let Some(ref filter) = options.hook_type_filter {
                if !filter.contains(&execution.hook_id) {
                    continue;
                }
            }

            match self
                .replay_execution(hook.as_ref(), execution, options)
                .await
            {
                Ok(result) => results.push(result),
                Err(e) => {
                    self.stats.failed_replays += 1;
                    return Err(e.context(format!("Failed to replay hook: {}", execution.hook_id)));
                }
            }
        }

        Ok(results)
    }

    /// Get replay statistics
    pub fn get_statistics(&self) -> (u64, u64, u64, Duration) {
        (
            self.stats.total_replayed,
            self.stats.successful_replays,
            self.stats.failed_replays,
            self.stats.total_replay_time,
        )
    }

    /// Reset statistics
    pub fn reset_statistics(&mut self) {
        self.stats = ReplayStatistics::default();
    }
}

impl Default for HookReplayEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[cfg_attr(test_category = "hook")]
mod tests {
    use super::*;

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_replay_options() {
        let options = ReplayOptions::default();
        assert!(!options.modify_parameters);
        assert!(options.dry_run);
        assert!(!options.simulate_timing);
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_replay_engine_creation() {
        let engine = HookReplayEngine::new();
        let (total, success, failed, _) = engine.get_statistics();
        assert_eq!(total, 0);
        assert_eq!(success, 0);
        assert_eq!(failed, 0);
    }
}
