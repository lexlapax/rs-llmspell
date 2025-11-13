//! State-Memory synchronization hook (Phase 13.7.4)
//!
//! Captures state transitions as procedural memory patterns to learn user behaviors.
//! Example: User repeatedly setting `config.theme: "dark"` creates a learned pattern.

use crate::hooks::{Hook, HookContext, HookResult};
use anyhow::Result;
use llmspell_memory::MemoryManager;
use std::sync::Arc;
use tracing::{debug, error, trace, warn};

/// Hook that captures state transitions as procedural memory patterns
///
/// Tracks state changes and records transition frequencies. When a transition
/// occurs ≥3 times, it becomes a learned pattern (e.g., user preference).
///
/// **Architecture**: Opt-in design - only active when `memory_manager` present.
pub struct StateMemoryHook {
    memory_manager: Arc<dyn MemoryManager>,
    /// Minimum frequency to log pattern detection (default: 3)
    pattern_threshold: u32,
}

impl std::fmt::Debug for StateMemoryHook {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StateMemoryHook")
            .field("memory_manager", &"Arc<dyn MemoryManager>")
            .field("pattern_threshold", &self.pattern_threshold)
            .finish()
    }
}

impl StateMemoryHook {
    /// Create new state-memory synchronization hook
    pub fn new(memory_manager: Arc<dyn MemoryManager>) -> Self {
        Self {
            memory_manager,
            pattern_threshold: 3,
        }
    }

    /// Create with custom pattern threshold (for testing)
    #[cfg(test)]
    pub fn with_threshold(memory_manager: Arc<dyn MemoryManager>, threshold: u32) -> Self {
        Self {
            memory_manager,
            pattern_threshold: threshold,
        }
    }
}

#[async_trait::async_trait]
impl Hook for StateMemoryHook {
    async fn execute(&self, ctx: &mut HookContext) -> Result<HookResult> {
        // Extract state change data from context
        let Some(scope) = ctx.data.get("scope").and_then(|v| v.as_str()) else {
            warn!("StateMemoryHook: scope not in context, skipping");
            return Ok(HookResult::Continue);
        };

        let Some(key) = ctx.data.get("key").and_then(|v| v.as_str()) else {
            warn!("StateMemoryHook: key not in context, skipping");
            return Ok(HookResult::Continue);
        };

        // Get old_value (may be None for initial set)
        let old_value_str;
        let old_value = if let Some(v) = ctx.data.get("old_value") {
            if let Some(s) = v.as_str() {
                Some(s)
            } else if let Ok(stringified) = serde_json::to_string(v) {
                old_value_str = stringified;
                Some(old_value_str.as_str())
            } else {
                None
            }
        } else {
            None
        };

        // Get new_value (required for pattern tracking)
        let new_value_owned;
        let new_value_str = if let Some(v) = ctx.data.get("new_value") {
            if let Some(s) = v.as_str() {
                s
            } else if let Ok(stringified) = serde_json::to_string(v) {
                new_value_owned = stringified;
                new_value_owned.as_str()
            } else {
                warn!("StateMemoryHook: new_value not serializable, skipping");
                return Ok(HookResult::Continue);
            }
        } else {
            warn!("StateMemoryHook: new_value not in context, skipping");
            return Ok(HookResult::Continue);
        };

        // Record transition in procedural memory
        match self
            .memory_manager
            .procedural()
            .record_transition(scope, key, old_value, new_value_str)
            .await
        {
            Ok(frequency) => {
                trace!(
                    "StateMemoryHook: Recorded transition {scope}:{key} → {new_value_str} (freq={})",
                    frequency
                );

                // Log when pattern threshold is reached
                if frequency == self.pattern_threshold {
                    debug!(
                        "StateMemoryHook: Pattern detected! {scope}:{key} → {new_value_str} occurred {} times",
                        frequency
                    );
                }
            }
            Err(e) => {
                error!(
                    "StateMemoryHook: Failed to record transition {scope}:{key}: {}",
                    e
                );
                return Err(e.into());
            }
        }

        Ok(HookResult::Continue)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hooks::{ComponentId, ComponentType, HookPoint};
    use llmspell_memory::DefaultMemoryManager;
    use serde_json::json;

    #[tokio::test]
    async fn test_state_memory_hook_tracks_transitions() {
        let memory = DefaultMemoryManager::new_in_memory()
            .await
            .expect("Failed to create memory manager");
        let memory_arc = Arc::new(memory);
        let hook = StateMemoryHook::new(memory_arc.clone());

        let component_id = ComponentId::new(ComponentType::System, "test".to_string());
        let mut ctx = HookContext::new(HookPoint::SystemStartup, component_id.clone());

        // First transition: light → dark
        ctx.data.insert("scope".to_string(), json!("global"));
        ctx.data.insert("key".to_string(), json!("theme"));
        ctx.data.insert("old_value".to_string(), json!("light"));
        ctx.data.insert("new_value".to_string(), json!("dark"));

        let result = hook.execute(&mut ctx).await;
        assert!(result.is_ok());

        // Second transition: light → dark
        let result = hook.execute(&mut ctx).await;
        assert!(result.is_ok());

        // Third transition: light → dark
        let result = hook.execute(&mut ctx).await;
        assert!(result.is_ok());

        // Verify pattern frequency
        let freq = memory_arc
            .procedural()
            .get_pattern_frequency("global", "theme", "dark")
            .await
            .expect("Failed to get frequency");
        assert_eq!(freq, 3);
    }

    #[tokio::test]
    async fn test_state_memory_hook_pattern_threshold() {
        let memory = DefaultMemoryManager::new_in_memory()
            .await
            .expect("Failed to create memory manager");
        let memory_arc = Arc::new(memory);
        let hook = StateMemoryHook::with_threshold(memory_arc.clone(), 2);

        let component_id = ComponentId::new(ComponentType::System, "test".to_string());
        let mut ctx = HookContext::new(HookPoint::SystemStartup, component_id.clone());

        ctx.data.insert("scope".to_string(), json!("session:x"));
        ctx.data.insert("key".to_string(), json!("lang"));
        ctx.data.insert("new_value".to_string(), json!("rust"));

        // Transition 1
        hook.execute(&mut ctx).await.unwrap();
        // Transition 2 - should trigger pattern log
        hook.execute(&mut ctx).await.unwrap();

        let freq = memory_arc
            .procedural()
            .get_pattern_frequency("session:x", "lang", "rust")
            .await
            .unwrap();
        assert_eq!(freq, 2);
    }

    #[tokio::test]
    async fn test_state_memory_hook_missing_data() {
        let memory = DefaultMemoryManager::new_in_memory()
            .await
            .expect("Failed to create memory manager");
        let memory_arc = Arc::new(memory);
        let hook = StateMemoryHook::new(memory_arc);

        let component_id = ComponentId::new(ComponentType::System, "test".to_string());
        let mut ctx = HookContext::new(HookPoint::SystemStartup, component_id.clone());

        // Missing scope
        ctx.data.insert("key".to_string(), json!("test"));
        ctx.data.insert("new_value".to_string(), json!("value"));

        let result = hook.execute(&mut ctx).await;
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), HookResult::Continue));

        // Missing key
        let mut ctx2 = HookContext::new(HookPoint::SystemStartup, component_id.clone());
        ctx2.data.insert("scope".to_string(), json!("global"));
        ctx2.data.insert("new_value".to_string(), json!("value"));

        let result = hook.execute(&mut ctx2).await;
        assert!(result.is_ok());

        // Missing new_value
        let mut ctx3 = HookContext::new(HookPoint::SystemStartup, component_id.clone());
        ctx3.data.insert("scope".to_string(), json!("global"));
        ctx3.data.insert("key".to_string(), json!("test"));

        let result = hook.execute(&mut ctx3).await;
        assert!(result.is_ok());
    }
}
