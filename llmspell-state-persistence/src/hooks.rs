// ABOUTME: State change hook definitions and integration
// ABOUTME: Provides hook points for state operations

use async_trait::async_trait;
use llmspell_hooks::{Hook, HookContext, HookMetadata, HookResult, Language, Priority};
use serde_json::Value;

/// State change event for hooks
#[derive(Debug, Clone)]
pub struct StateChangeEvent {
    pub scope: crate::scope::StateScope,
    pub key: String,
    pub old_value: Option<Value>,
    pub new_value: Option<Value>,
    pub operation: StateOperation,
    pub correlation_id: uuid::Uuid,
    pub timestamp: std::time::SystemTime,
}

#[derive(Debug, Clone)]
pub enum StateOperation {
    Set,
    Delete,
    Clear,
}

/// Built-in state validation hook
pub struct StateValidationHook;

#[async_trait]
impl Hook for StateValidationHook {
    async fn execute(&self, context: &mut HookContext) -> anyhow::Result<HookResult> {
        // Validate state value size
        if let Some(new_value) = context.get_metadata("new_value") {
            if let Ok(value) = serde_json::from_str::<Value>(new_value) {
                let size = serde_json::to_string(&value)?.len();
                if size > 1_000_000 {
                    // 1MB limit
                    return Ok(HookResult::Cancel(
                        "State value exceeds size limit".to_string(),
                    ));
                }
            }
        }
        Ok(HookResult::Continue)
    }

    fn metadata(&self) -> HookMetadata {
        HookMetadata {
            name: "state_validation".to_string(),
            description: Some("Validates state values before storage".to_string()),
            version: "1.0.0".to_string(),
            priority: Priority::HIGH,
            language: Language::Native,
            tags: vec!["state".to_string(), "validation".to_string()],
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Built-in state audit hook
pub struct StateAuditHook;

#[async_trait]
impl Hook for StateAuditHook {
    async fn execute(&self, context: &mut HookContext) -> anyhow::Result<HookResult> {
        // Log state changes for audit trail
        let scope = context.get_metadata("scope").unwrap_or("unknown");
        let key = context.get_metadata("key").unwrap_or("unknown");
        let operation = context.get_metadata("operation").unwrap_or("unknown");

        tracing::info!(
            "State operation: {} on {}/{} by component {:?}",
            operation,
            scope,
            key,
            context.component_id
        );

        Ok(HookResult::Continue)
    }

    fn metadata(&self) -> HookMetadata {
        HookMetadata {
            name: "state_audit".to_string(),
            description: Some("Logs state changes for audit trail".to_string()),
            version: "1.0.0".to_string(),
            priority: Priority::LOW,
            language: Language::Native,
            tags: vec!["state".to_string(), "audit".to_string()],
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Built-in state cache invalidation hook
pub struct StateCacheHook;

#[async_trait]
impl Hook for StateCacheHook {
    async fn execute(&self, context: &mut HookContext) -> anyhow::Result<HookResult> {
        // In a real implementation, this would invalidate caches
        let key = context.get_metadata("key").unwrap_or("unknown");
        tracing::debug!("Cache invalidated for key: {}", key);
        Ok(HookResult::Continue)
    }

    fn metadata(&self) -> HookMetadata {
        HookMetadata {
            name: "state_cache".to_string(),
            description: Some("Invalidates caches on state changes".to_string()),
            version: "1.0.0".to_string(),
            priority: Priority::NORMAL,
            language: Language::Native,
            tags: vec!["state".to_string(), "cache".to_string()],
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Helper to aggregate hook results
pub fn aggregate_hook_results(results: &[HookResult]) -> HookResult {
    // If any hook cancels, the operation is cancelled
    for result in results {
        if let HookResult::Cancel(reason) = result {
            return HookResult::Cancel(reason.clone());
        }
    }

    // If any hook modifies data, use the last modification
    for result in results.iter().rev() {
        if let HookResult::Modified(data) = result {
            return HookResult::Modified(data.clone());
        }
    }

    // Otherwise continue
    HookResult::Continue
}
