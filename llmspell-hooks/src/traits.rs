// ABOUTME: Core trait definitions for the hook system including Hook, HookAdapter, and ReplayableHook
// ABOUTME: Provides the foundation for language-agnostic hook execution and future persistence support

use crate::context::HookContext;
use crate::result::HookResult;
use crate::types::HookMetadata;
use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;

/// Base hook trait for all hooks in the system
#[async_trait]
pub trait Hook: Send + Sync {
    /// Execute the hook with the given context
    async fn execute(&self, context: &mut HookContext) -> Result<HookResult>;

    /// Get metadata about this hook
    fn metadata(&self) -> HookMetadata {
        HookMetadata::default()
    }

    /// Check if this hook should be executed for the given context
    fn should_execute(&self, _context: &HookContext) -> bool {
        true
    }

    /// Get self as Any for downcasting
    fn as_any(&self) -> &dyn std::any::Any;
}

/// Language adapter for cross-language hook support
pub trait HookAdapter: Send + Sync {
    /// The language-specific context type
    type Context;

    /// The language-specific result type
    type Result;

    /// Adapt the universal context to language-specific format
    fn adapt_context(&self, ctx: &HookContext) -> Self::Context;

    /// Adapt the language-specific result to universal format
    fn adapt_result(&self, result: Self::Result) -> HookResult;

    /// Get the error from a language-specific result if any
    fn extract_error(&self, _result: &Self::Result) -> Option<String> {
        None
    }
}

/// Trait for hooks that can be replayed from persisted state (Phase 5 prep)
#[async_trait]
pub trait ReplayableHook: Hook {
    /// Check if this hook supports replay
    fn is_replayable(&self) -> bool {
        true
    }

    /// Serialize the context for persistence
    fn serialize_context(&self, ctx: &HookContext) -> Result<Vec<u8>> {
        Ok(serde_json::to_vec(ctx)?)
    }

    /// Deserialize the context from persistence
    fn deserialize_context(&self, data: &[u8]) -> Result<HookContext> {
        Ok(serde_json::from_slice(data)?)
    }

    /// Get a replay identifier for this hook
    fn replay_id(&self) -> String {
        format!("{}:{}", self.metadata().name, self.metadata().version)
    }
}

/// Trait for hooks that can be composed
pub trait ComposableHook: Hook {
    /// Check if this hook can be composed with another
    fn can_compose_with(&self, _other: &dyn Hook) -> bool {
        true
    }

    /// Get the composition priority
    fn composition_priority(&self) -> i32 {
        0
    }
}

/// Trait for hooks that track metrics
#[async_trait]
pub trait MetricHook: Hook {
    /// Record metrics before execution
    async fn record_pre_execution(&self, _context: &HookContext) -> Result<()> {
        Ok(())
    }

    /// Record metrics after execution
    async fn record_post_execution(
        &self,
        _context: &HookContext,
        _result: &HookResult,
        _duration: std::time::Duration,
    ) -> Result<()> {
        Ok(())
    }
}

/// A wrapper to make closures into hooks
pub struct FnHook<F> {
    func: F,
    metadata: HookMetadata,
}

impl<F> FnHook<F>
where
    F: Fn(&mut HookContext) -> Result<HookResult> + Send + Sync + 'static,
{
    pub fn new(name: &str, func: F) -> Self {
        Self {
            func,
            metadata: HookMetadata {
                name: name.to_string(),
                ..Default::default()
            },
        }
    }

    pub fn with_metadata(mut self, metadata: HookMetadata) -> Self {
        self.metadata = metadata;
        self
    }
}

#[async_trait]
impl<F> Hook for FnHook<F>
where
    F: Fn(&mut HookContext) -> Result<HookResult> + Send + Sync + 'static,
{
    async fn execute(&self, context: &mut HookContext) -> Result<HookResult> {
        (self.func)(context)
    }

    fn metadata(&self) -> HookMetadata {
        self.metadata.clone()
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Type alias for a boxed hook
pub type BoxedHook = Box<dyn Hook>;

/// Type alias for an arc'd hook
pub type ArcHook = Arc<dyn Hook>;

/// Extension trait for hooks
pub trait HookExt: Hook {
    /// Convert to a boxed hook
    fn boxed(self) -> BoxedHook
    where
        Self: Sized + 'static,
    {
        Box::new(self)
    }

    /// Convert to an arc'd hook
    fn arc(self) -> ArcHook
    where
        Self: Sized + 'static,
    {
        Arc::new(self)
    }
}

impl<H: Hook + ?Sized> HookExt for H {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{ComponentId, ComponentType, HookPoint};

    struct TestHook {
        name: String,
    }

    #[async_trait]
    impl Hook for TestHook {
        async fn execute(&self, _context: &mut HookContext) -> Result<HookResult> {
            Ok(HookResult::Continue)
        }

        fn metadata(&self) -> HookMetadata {
            HookMetadata {
                name: self.name.clone(),
                ..Default::default()
            }
        }

        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
    }

    #[tokio::test]
    async fn test_basic_hook() {
        let hook = TestHook {
            name: "test_hook".to_string(),
        };

        let component_id = ComponentId::new(ComponentType::System, "test".to_string());
        let mut context = HookContext::new(HookPoint::SystemStartup, component_id);

        let result = hook.execute(&mut context).await.unwrap();
        assert!(matches!(result, HookResult::Continue));
        assert_eq!(hook.metadata().name, "test_hook");
    }

    #[tokio::test]
    async fn test_fn_hook() {
        let hook = FnHook::new("function_hook", |ctx: &mut HookContext| {
            ctx.insert_metadata("executed".to_string(), "true".to_string());
            Ok(HookResult::Continue)
        });

        let component_id = ComponentId::new(ComponentType::Agent, "test".to_string());
        let mut context = HookContext::new(HookPoint::BeforeAgentExecution, component_id);

        let result = hook.execute(&mut context).await.unwrap();
        assert!(matches!(result, HookResult::Continue));
        assert_eq!(context.get_metadata("executed"), Some("true"));
    }

    struct TestAdapter;

    impl HookAdapter for TestAdapter {
        type Context = String;
        type Result = bool;

        fn adapt_context(&self, ctx: &HookContext) -> Self::Context {
            format!("Hook: {:?}", ctx.point)
        }

        fn adapt_result(&self, result: Self::Result) -> HookResult {
            if result {
                HookResult::Continue
            } else {
                HookResult::Cancel("Adapter returned false".to_string())
            }
        }
    }

    #[test]
    fn test_hook_adapter() {
        let adapter = TestAdapter;
        let component_id = ComponentId::new(ComponentType::Tool, "test".to_string());
        let context = HookContext::new(HookPoint::BeforeToolExecution, component_id);

        let adapted_ctx = adapter.adapt_context(&context);
        assert_eq!(adapted_ctx, "Hook: BeforeToolExecution");

        let result1 = adapter.adapt_result(true);
        assert!(matches!(result1, HookResult::Continue));

        let result2 = adapter.adapt_result(false);
        assert!(matches!(result2, HookResult::Cancel(_)));
    }

    struct TestReplayableHook;

    #[async_trait]
    impl Hook for TestReplayableHook {
        async fn execute(&self, _context: &mut HookContext) -> Result<HookResult> {
            Ok(HookResult::Continue)
        }

        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
    }

    #[async_trait]
    impl ReplayableHook for TestReplayableHook {
        fn replay_id(&self) -> String {
            "test_replayable:1.0.0".to_string()
        }
    }

    #[tokio::test]
    async fn test_replayable_hook() {
        let hook = TestReplayableHook;
        let component_id = ComponentId::new(ComponentType::System, "test".to_string());
        let context = HookContext::new(HookPoint::SystemStartup, component_id);

        // Test serialization
        let serialized = hook.serialize_context(&context).unwrap();
        let deserialized = hook.deserialize_context(&serialized).unwrap();

        assert_eq!(deserialized.point, context.point);
        assert_eq!(hook.replay_id(), "test_replayable:1.0.0");
        assert!(hook.is_replayable());
    }

    #[test]
    fn test_hook_extensions() {
        let hook = TestHook {
            name: "extension_test".to_string(),
        };

        let boxed: BoxedHook = hook.boxed();
        assert_eq!(boxed.metadata().name, "extension_test");
    }
}
