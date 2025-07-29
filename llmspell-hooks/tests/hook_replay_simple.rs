// ABOUTME: Simple test demonstrating hook replay functionality
// ABOUTME: Shows basic usage of ReplayableHook trait with replay manager components

use anyhow::Result;
use async_trait::async_trait;
use llmspell_hooks::{
    persistence::{HookReplayManager, SerializedHookExecution},
    replay::{HookResultComparator, ParameterModification},
    traits::{Hook, ReplayableHook},
    ComponentId, ComponentType, HookContext, HookPoint, HookResult,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use uuid::Uuid;

/// Simple test hook that doubles input values
#[derive(Debug, Clone, Serialize, Deserialize)]
struct DoublerHook {
    name: String,
}

#[async_trait]
impl Hook for DoublerHook {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    async fn execute(&self, context: &mut HookContext) -> Result<HookResult> {
        let input = context
            .data
            .get("value")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);

        let result = input * 2.0;
        context.data.insert("result".to_string(), json!(result));

        Ok(HookResult::Modified(json!({
            "input": input,
            "output": result
        })))
    }
}

#[async_trait]
impl ReplayableHook for DoublerHook {
    fn replay_id(&self) -> String {
        self.name.clone()
    }
}

/// Simple in-memory replay manager for testing
struct TestReplayManager {
    executions: Arc<parking_lot::RwLock<HashMap<Uuid, SerializedHookExecution>>>,
}

impl TestReplayManager {
    fn new() -> Self {
        Self {
            executions: Arc::new(parking_lot::RwLock::new(HashMap::new())),
        }
    }

    async fn replay_hook_execution(
        &self,
        execution_id: &Uuid,
        hook: &dyn ReplayableHook,
        modifications: &[ParameterModification],
    ) -> Result<HookResult> {
        let execution = self
            .executions
            .read()
            .get(execution_id)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("Execution not found"))?;

        let mut context = hook.deserialize_context(&execution.hook_context)?;

        // Apply modifications
        for modification in modifications {
            if modification.enabled && modification.path == "context.data.value" {
                context
                    .data
                    .insert("value".to_string(), modification.value.clone());
            }
        }

        hook.execute(&mut context).await
    }
}

#[async_trait]
impl HookReplayManager for TestReplayManager {
    async fn persist_hook_execution(
        &self,
        hook: &dyn ReplayableHook,
        context: &HookContext,
        result: &HookResult,
        duration: Duration,
    ) -> Result<()> {
        let execution = SerializedHookExecution {
            hook_id: hook.replay_id(),
            execution_id: Uuid::new_v4(),
            correlation_id: Uuid::new_v4(),
            hook_context: hook.serialize_context(context)?,
            result: serde_json::to_string(result)?,
            timestamp: SystemTime::now(),
            duration,
            metadata: HashMap::new(),
        };

        self.executions
            .write()
            .insert(execution.execution_id, execution);
        Ok(())
    }

    async fn get_hook_executions_by_correlation(
        &self,
        correlation_id: Uuid,
    ) -> Result<Vec<SerializedHookExecution>> {
        Ok(self
            .executions
            .read()
            .values()
            .filter(|e| e.correlation_id == correlation_id)
            .cloned()
            .collect())
    }
}

#[tokio::test]
async fn test_basic_hook_replay() -> Result<()> {
    // Create hook and replay manager
    let hook = DoublerHook {
        name: "test_doubler".to_string(),
    };
    let replay_manager = TestReplayManager::new();

    // Create context with initial value
    let component_id = ComponentId::new(ComponentType::Tool, "test_tool".to_string());
    let mut context = HookContext::new(HookPoint::BeforeToolExecution, component_id);
    context.data.insert("value".to_string(), json!(5.0));

    // Execute and persist
    let result = hook.execute(&mut context).await?;
    replay_manager
        .persist_hook_execution(&hook, &context, &result, Duration::from_millis(1))
        .await?;

    // Get the execution ID
    let execution_id = replay_manager
        .executions
        .read()
        .keys()
        .next()
        .cloned()
        .unwrap();

    // Replay with same values
    let replay_result = replay_manager
        .replay_hook_execution(&execution_id, &hook, &[])
        .await?;

    // Compare results
    let comparator = HookResultComparator::new();
    let comparison = comparator.compare(&result, &replay_result);
    assert_eq!(comparison.similarity_score, 1.0);

    // Replay with modified value
    let modifications = vec![ParameterModification {
        path: "context.data.value".to_string(),
        value: json!(10.0),
        enabled: true,
    }];
    let modified_result = replay_manager
        .replay_hook_execution(&execution_id, &hook, &modifications)
        .await?;

    // Verify the modification worked
    if let HookResult::Modified(data) = modified_result {
        assert_eq!(data["input"], 10.0);
        assert_eq!(data["output"], 20.0);
    } else {
        panic!("Expected Modified result");
    }

    Ok(())
}

#[tokio::test]
async fn test_parameter_modification() -> Result<()> {
    let modification = ParameterModification {
        path: "context.data.test_key".to_string(),
        value: json!("test_value"),
        enabled: true,
    };

    // Verify the path parsing logic
    assert_eq!(
        modification.path.split('.').collect::<Vec<_>>(),
        vec!["context", "data", "test_key"]
    );

    Ok(())
}
