// ABOUTME: Parallel hook pattern implementation with result aggregation
// ABOUTME: Executes all hooks concurrently and aggregates results by priority

use crate::composite::{CompositeHook, CompositionPattern};
use crate::context::HookContext;
use crate::result::HookResult;
use crate::traits::{ArcHook, Hook};
use crate::types::HookMetadata;
use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, warn};

/// A parallel hook that executes hooks concurrently
pub struct ParallelHook {
    inner: CompositeHook,
    timeout: Option<Duration>,
    continue_on_error: bool,
}

impl ParallelHook {
    /// Create a new parallel hook
    pub fn new(name: &str) -> Self {
        Self {
            inner: CompositeHook::new(name, CompositionPattern::Parallel),
            timeout: None,
            continue_on_error: false,
        }
    }

    /// Set execution timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Set whether to continue on error
    pub fn continue_on_error(mut self, continue_on_error: bool) -> Self {
        self.continue_on_error = continue_on_error;
        self
    }

    /// Add a hook to execute in parallel
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
    pub fn builder(name: &str) -> ParallelHookBuilder {
        ParallelHookBuilder::new(name)
    }
}

#[async_trait]
impl Hook for ParallelHook {
    async fn execute(&self, context: &mut HookContext) -> Result<HookResult> {
        if let Some(timeout) = self.timeout {
            match tokio::time::timeout(timeout, self.inner.execute(context)).await {
                Ok(result) => result,
                Err(_) => {
                    warn!(
                        "Parallel hook '{}' timed out after {:?}",
                        self.inner.metadata().name,
                        timeout
                    );
                    Err(anyhow::anyhow!("Parallel execution timed out"))
                }
            }
        } else {
            self.inner.execute(context).await
        }
    }

    fn metadata(&self) -> HookMetadata {
        self.inner.metadata()
    }

    fn should_execute(&self, context: &HookContext) -> bool {
        self.inner.should_execute(context)
    }
}

/// Builder for parallel hooks
pub struct ParallelHookBuilder {
    name: String,
    hooks: Vec<ArcHook>,
    metadata: Option<HookMetadata>,
    timeout: Option<Duration>,
    continue_on_error: bool,
    max_concurrency: Option<usize>,
}

impl ParallelHookBuilder {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            hooks: Vec::new(),
            metadata: None,
            timeout: None,
            continue_on_error: false,
            max_concurrency: None,
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

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    pub fn continue_on_error(mut self, continue_on_error: bool) -> Self {
        self.continue_on_error = continue_on_error;
        self
    }

    pub fn with_max_concurrency(mut self, max: usize) -> Self {
        self.max_concurrency = Some(max);
        self
    }

    pub fn build(self) -> ParallelHook {
        let mut hook = ParallelHook::new(&self.name)
            .continue_on_error(self.continue_on_error);
        
        if let Some(metadata) = self.metadata {
            hook = hook.with_metadata(metadata);
        }
        
        if let Some(timeout) = self.timeout {
            hook = hook.with_timeout(timeout);
        }
        
        hook = hook.add_hooks(self.hooks);
        hook
    }
}

/// Result aggregator for parallel execution
#[derive(Debug)]
pub struct ParallelResultAggregator {
    results: Vec<HookResult>,
}

impl ParallelResultAggregator {
    pub fn new() -> Self {
        Self {
            results: Vec::new(),
        }
    }

    pub fn add_result(&mut self, result: HookResult) {
        self.results.push(result);
    }

    pub fn aggregate(self) -> HookResult {
        // Priority order for aggregation:
        // 1. Cancel (highest priority - stops everything)
        // 2. Replace (replaces the entire result)
        // 3. Redirect (changes the flow)
        // 4. Fork (spawns parallel operations)
        // 5. Retry (requests retry)
        // 6. Modified (modifies data)
        // 7. Cache (caching directive)
        // 8. Skipped (hook was skipped)
        // 9. Continue (default)

        for result in &self.results {
            if matches!(result, HookResult::Cancel(_)) {
                return result.clone();
            }
        }

        for result in &self.results {
            if matches!(result, HookResult::Replace(_)) {
                return result.clone();
            }
        }

        for result in &self.results {
            if matches!(result, HookResult::Redirect(_)) {
                return result.clone();
            }
        }

        for result in &self.results {
            if matches!(result, HookResult::Fork { .. }) {
                return result.clone();
            }
        }

        for result in &self.results {
            if matches!(result, HookResult::Retry { .. }) {
                return result.clone();
            }
        }

        for result in &self.results {
            if matches!(result, HookResult::Modified(_)) {
                return result.clone();
            }
        }

        for result in &self.results {
            if matches!(result, HookResult::Cache { .. }) {
                return result.clone();
            }
        }

        HookResult::Continue
    }
}

impl Default for ParallelResultAggregator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::FnHook;
    use crate::types::{ComponentId, ComponentType, HookPoint};
    use std::sync::atomic::{AtomicUsize, Ordering};
    use tokio::time::sleep;
    #[tokio::test]
    async fn test_parallel_execution() {
        let counter = Arc::new(AtomicUsize::new(0));
        let c1 = counter.clone();
        let c2 = counter.clone();
        let c3 = counter.clone();

        let hook = ParallelHook::new("test_parallel")
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
        
        // All hooks should have executed
        assert_eq!(counter.load(Ordering::SeqCst), 111);
    }
    #[tokio::test]
    async fn test_parallel_timeout() {
        let hook = ParallelHook::builder("test_timeout")
            .add_hook(FnHook::new("fast", |_| Ok(HookResult::Continue)))
            .add_hook(FnHook::new("slow", |_| async {
                sleep(Duration::from_millis(200)).await;
                Ok(HookResult::Continue)
            }))
            .with_timeout(Duration::from_millis(50))
            .build();

        let component_id = ComponentId::new(ComponentType::System, "test".to_string());
        let mut context = HookContext::new(HookPoint::SystemStartup, component_id);

        let result = hook.execute(&mut context).await;
        assert!(result.is_err());
    }
    #[test]
    fn test_result_aggregator() {
        let mut aggregator = ParallelResultAggregator::new();
        
        aggregator.add_result(HookResult::Continue);
        aggregator.add_result(HookResult::Modified(serde_json::json!({"value": 1})));
        aggregator.add_result(HookResult::Continue);
        
        let result = aggregator.aggregate();
        assert!(matches!(result, HookResult::Modified(_)));
        
        // Test priority ordering
        let mut aggregator = ParallelResultAggregator::new();
        aggregator.add_result(HookResult::Modified(serde_json::json!({"value": 1})));
        aggregator.add_result(HookResult::Cancel("Stop".to_string()));
        aggregator.add_result(HookResult::Replace(serde_json::json!({"value": 2})));
        
        let result = aggregator.aggregate();
        assert!(matches!(result, HookResult::Cancel(_))); // Cancel has highest priority
    }
}