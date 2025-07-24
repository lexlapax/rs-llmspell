// ABOUTME: Sequential hook pattern implementation with early termination support
// ABOUTME: Executes hooks in order, stopping at the first non-Continue result

use crate::composite::{CompositeHook, CompositionPattern};
use crate::context::HookContext;
use crate::result::HookResult;
use crate::traits::{ArcHook, Hook};
use crate::types::HookMetadata;
use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;
use tracing::debug;

/// A sequential hook that executes hooks in order
pub struct SequentialHook {
    inner: CompositeHook,
}

impl SequentialHook {
    /// Create a new sequential hook
    pub fn new(name: &str) -> Self {
        Self {
            inner: CompositeHook::new(name, CompositionPattern::Sequential),
        }
    }

    /// Add a hook to the sequence
    pub fn add_hook(mut self, hook: impl Hook + 'static) -> Self {
        self.inner = self.inner.add_hook(Arc::new(hook));
        self
    }

    /// Add an Arc'd hook
    pub fn add_arc_hook(mut self, hook: ArcHook) -> Self {
        self.inner = self.inner.add_hook(hook);
        self
    }

    /// Add multiple hooks
    pub fn add_hooks(mut self, hooks: Vec<ArcHook>) -> Self {
        self.inner = self.inner.add_hooks(hooks);
        self
    }

    /// Set metadata
    pub fn with_metadata(mut self, metadata: HookMetadata) -> Self {
        self.inner = self.inner.with_metadata(metadata);
        self
    }

    /// Get the number of hooks
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Create a builder
    pub fn builder(name: &str) -> SequentialHookBuilder {
        SequentialHookBuilder::new(name)
    }
}

#[async_trait]
impl Hook for SequentialHook {
    async fn execute(&self, context: &mut HookContext) -> Result<HookResult> {
        self.inner.execute(context).await
    }

    fn metadata(&self) -> HookMetadata {
        self.inner.metadata()
    }

    fn should_execute(&self, context: &HookContext) -> bool {
        self.inner.should_execute(context)
    }
}

/// Builder for sequential hooks
pub struct SequentialHookBuilder {
    name: String,
    hooks: Vec<ArcHook>,
    metadata: Option<HookMetadata>,
    stop_on_error: bool,
    stop_on_modified: bool,
}

impl SequentialHookBuilder {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            hooks: Vec::new(),
            metadata: None,
            stop_on_error: true,
            stop_on_modified: true,
        }
    }

    pub fn add_hook(mut self, hook: impl Hook + 'static) -> Self {
        self.hooks.push(Arc::new(hook));
        self
    }

    pub fn add_arc_hook(mut self, hook: ArcHook) -> Self {
        self.hooks.push(hook);
        self
    }

    pub fn with_metadata(mut self, metadata: HookMetadata) -> Self {
        self.metadata = Some(metadata);
        self
    }

    pub fn stop_on_error(mut self, stop: bool) -> Self {
        self.stop_on_error = stop;
        self
    }

    pub fn stop_on_modified(mut self, stop: bool) -> Self {
        self.stop_on_modified = stop;
        self
    }

    pub fn build(self) -> SequentialHook {
        let mut hook = SequentialHook::new(&self.name);
        
        if let Some(metadata) = self.metadata {
            hook = hook.with_metadata(metadata);
        }
        
        hook = hook.add_hooks(self.hooks);
        hook
    }
}

/// Advanced sequential hook with conditional execution
pub struct ConditionalSequentialHook {
    base: SequentialHook,
    conditions: Vec<Box<dyn Fn(&HookContext) -> bool + Send + Sync>>,
}

impl ConditionalSequentialHook {
    pub fn new(name: &str) -> Self {
        Self {
            base: SequentialHook::new(name),
            conditions: Vec::new(),
        }
    }

    pub fn add_hook_with_condition<F>(
        mut self,
        hook: impl Hook + 'static,
        condition: F,
    ) -> Self
    where
        F: Fn(&HookContext) -> bool + Send + Sync + 'static,
    {
        self.base = self.base.add_hook(hook);
        self.conditions.push(Box::new(condition));
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::FnHook;
    use crate::types::{ComponentId, ComponentType, HookPoint};
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[tokio::test]
    async fn test_sequential_execution() {
        let counter = Arc::new(AtomicUsize::new(0));
        let c1 = counter.clone();
        let c2 = counter.clone();
        let c3 = counter.clone();

        let hook = SequentialHook::new("test_seq")
            .add_hook(FnHook::new("h1", move |_| {
                c1.fetch_add(1, Ordering::SeqCst);
                Ok(HookResult::Continue)
            }))
            .add_hook(FnHook::new("h2", move |_| {
                c2.fetch_add(10, Ordering::SeqCst);
                Ok(HookResult::Continue)
            }))
            .add_hook(FnHook::new("h3", move |_| {
                c3.fetch_add(100, Ordering::SeqCst);
                Ok(HookResult::Continue)
            }));

        let component_id = ComponentId::new(ComponentType::System, "test".to_string());
        let mut context = HookContext::new(HookPoint::SystemStartup, component_id);

        let result = hook.execute(&mut context).await.unwrap();
        assert!(matches!(result, HookResult::Continue));
        assert_eq!(counter.load(Ordering::SeqCst), 111);
    }

    #[tokio::test]
    async fn test_sequential_early_termination() {
        let counter = Arc::new(AtomicUsize::new(0));
        let c1 = counter.clone();
        let c2 = counter.clone();
        let c3 = counter.clone();

        let hook = SequentialHook::builder("test_early")
            .add_hook(FnHook::new("h1", move |_| {
                c1.fetch_add(1, Ordering::SeqCst);
                Ok(HookResult::Continue)
            }))
            .add_hook(FnHook::new("h2", move |_| {
                c2.fetch_add(10, Ordering::SeqCst);
                Ok(HookResult::Cancel("Stop here".to_string()))
            }))
            .add_hook(FnHook::new("h3", move |_| {
                c3.fetch_add(100, Ordering::SeqCst);
                Ok(HookResult::Continue)
            }))
            .build();

        let component_id = ComponentId::new(ComponentType::System, "test".to_string());
        let mut context = HookContext::new(HookPoint::SystemStartup, component_id);

        let result = hook.execute(&mut context).await.unwrap();
        assert!(matches!(result, HookResult::Cancel(_)));
        assert_eq!(counter.load(Ordering::SeqCst), 11); // Third hook not executed
    }
}