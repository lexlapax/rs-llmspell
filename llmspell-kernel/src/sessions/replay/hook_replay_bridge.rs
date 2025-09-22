//! ABOUTME: Bridge adapter to connect state persistence `HookReplayManager` to hooks `HookReplayManager` trait
//! ABOUTME: Temporary bridge for Task 6.4.1 - will be refactored in subsequent tasks

use crate::state::manager::HookReplayManager as StateHookReplayManager;
use anyhow::Result;
use async_trait::async_trait;
use llmspell_hooks::persistence::HookReplayManager as HooksHookReplayManager;
use llmspell_hooks::{HookContext, HookResult, ReplayableHook};
use std::sync::Arc;
use std::time::Duration;
use uuid::Uuid;

/// Bridge adapter that implements `hooks::HookReplayManager` using `state-persistence::HookReplayManager`
#[derive(Debug)]
pub struct HookReplayBridge {
    state_replay_manager: Arc<StateHookReplayManager>,
}

impl HookReplayBridge {
    /// Create a new hook replay bridge
    pub fn new(state_replay_manager: Arc<StateHookReplayManager>) -> Self {
        Self {
            state_replay_manager,
        }
    }
}

#[async_trait]
impl HooksHookReplayManager for HookReplayBridge {
    async fn persist_hook_execution(
        &self,
        hook: &dyn ReplayableHook,
        context: &HookContext,
        result: &HookResult,
        duration: Duration,
    ) -> Result<()> {
        // Convert from anyhow::Result to StateResult and back
        self.state_replay_manager
            .persist_hook_execution(hook, context, result, duration)
            .await
            .map_err(|e| anyhow::anyhow!("State persistence error: {}", e))
    }

    async fn get_hook_executions_by_correlation(
        &self,
        correlation_id: Uuid,
    ) -> Result<Vec<llmspell_hooks::persistence::SerializedHookExecution>> {
        let state_executions = self
            .state_replay_manager
            .get_hook_executions_by_correlation(correlation_id)
            .await
            .map_err(|e| anyhow::anyhow!("State persistence error: {}", e))?;

        // Convert from state-persistence SerializedHookExecution to hooks SerializedHookExecution
        let hooks_executions = state_executions
            .into_iter()
            .map(
                |exec| llmspell_hooks::persistence::SerializedHookExecution {
                    hook_id: exec.hook_id,
                    execution_id: exec.execution_id,
                    correlation_id: exec.correlation_id,
                    hook_context: exec.hook_context,
                    result: exec.result,
                    timestamp: exec.timestamp,
                    duration: exec.duration,
                    metadata: exec.metadata,
                },
            )
            .collect();

        Ok(hooks_executions)
    }
}
