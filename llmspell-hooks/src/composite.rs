// ABOUTME: CompositeHook implementation for combining multiple hooks with different patterns
// ABOUTME: Provides Sequential, Parallel, FirstMatch, and Voting composition strategies

use crate::context::HookContext;
use crate::result::HookResult;
use crate::traits::{ArcHook, Hook, ComposableHook};
use crate::types::{HookMetadata, Priority};
use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;
use tracing::{debug, info};

/// Composition pattern for combining hooks
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompositionPattern {
    /// Execute hooks in sequence, stop on first non-Continue result
    Sequential,
    /// Execute all hooks in parallel, aggregate results
    Parallel,
    /// Execute until first matching hook (non-Continue result)
    FirstMatch,
    /// Execute all hooks and use voting to determine result
    Voting { threshold: f64 },
}

/// A composite hook that combines multiple hooks
pub struct CompositeHook {
    pattern: CompositionPattern,
    hooks: Vec<ArcHook>,
    metadata: HookMetadata,
}

impl CompositeHook {
    /// Create a new composite hook
    pub fn new(name: &str, pattern: CompositionPattern) -> Self {
        Self {
            pattern,
            hooks: Vec::new(),
            metadata: HookMetadata {
                name: name.to_string(),
                description: Some(format!("Composite hook using {:?} pattern", pattern)),
                ..Default::default()
            },
        }
    }

    /// Add a hook to the composite
    pub fn add_hook(mut self, hook: ArcHook) -> Self {
        self.hooks.push(hook);
        self
    }

    /// Add multiple hooks
    pub fn add_hooks(mut self, hooks: Vec<ArcHook>) -> Self {
        self.hooks.extend(hooks);
        self
    }

    /// Set metadata
    pub fn with_metadata(mut self, metadata: HookMetadata) -> Self {
        self.metadata = metadata;
        self
    }

    /// Get the number of hooks
    pub fn len(&self) -> usize {
        self.hooks.len()
    }

    /// Check if composite is empty
    pub fn is_empty(&self) -> bool {
        self.hooks.is_empty()
    }

    /// Execute hooks sequentially
    async fn execute_sequential(&self, context: &mut HookContext) -> Result<HookResult> {
        debug!(
            "Executing {} hooks sequentially for composite '{}'",
            self.hooks.len(),
            self.metadata.name
        );

        for (idx, hook) in self.hooks.iter().enumerate() {
            let result = hook.execute(context).await?;
            
            debug!(
                "Sequential hook {} returned: {:?}",
                idx,
                result.description()
            );

            // Stop on first non-Continue result
            if !matches!(result, HookResult::Continue) {
                return Ok(result);
            }
        }

        Ok(HookResult::Continue)
    }

    /// Execute hooks in parallel
    async fn execute_parallel(&self, context: &mut HookContext) -> Result<HookResult> {
        use futures::future::join_all;

        debug!(
            "Executing {} hooks in parallel for composite '{}'",
            self.hooks.len(),
            self.metadata.name
        );

        // Clone context for each parallel execution
        let futures = self.hooks.iter().map(|hook| {
            let mut ctx_clone = context.clone();
            async move { hook.execute(&mut ctx_clone).await }
        });

        let results = join_all(futures).await;

        // Check for errors first
        for result in &results {
            if let Err(e) = result {
                return Err(anyhow::anyhow!("Parallel hook failed: {}", e));
            }
        }

        // Aggregate results - priority order:
        // 1. Cancel (stops everything)
        // 2. Replace (replaces result)
        // 3. Redirect (changes flow)
        // 4. Modified (modifies data)
        // 5. Other special results
        // 6. Continue (default)
        
        for result in results {
            let result = result?;
            match result {
                HookResult::Cancel(_) => return Ok(result),
                _ => continue,
            }
        }

        for result in results {
            let result = result?;
            match result {
                HookResult::Replace(_) => return Ok(result),
                _ => continue,
            }
        }

        for result in results {
            let result = result?;
            match result {
                HookResult::Redirect(_) => return Ok(result),
                _ => continue,
            }
        }

        for result in results {
            let result = result?;
            match result {
                HookResult::Modified(_) => return Ok(result),
                _ => continue,
            }
        }

        Ok(HookResult::Continue)
    }

    /// Execute until first match
    async fn execute_first_match(&self, context: &mut HookContext) -> Result<HookResult> {
        debug!(
            "Executing first-match pattern for composite '{}'",
            self.metadata.name
        );

        for (idx, hook) in self.hooks.iter().enumerate() {
            let result = hook.execute(context).await?;
            
            debug!(
                "First-match hook {} returned: {:?}",
                idx,
                result.description()
            );

            // Return first non-Continue result
            if !matches!(result, HookResult::Continue) {
                return Ok(result);
            }
        }

        Ok(HookResult::Continue)
    }

    /// Execute with voting
    async fn execute_voting(&self, context: &mut HookContext, threshold: f64) -> Result<HookResult> {
        use std::collections::HashMap;

        debug!(
            "Executing voting pattern (threshold: {:.2}) for composite '{}'",
            threshold, self.metadata.name
        );

        let mut vote_counts: HashMap<String, usize> = HashMap::new();
        let mut results = Vec::new();

        // Collect all results
        for hook in &self.hooks {
            let result = hook.execute(context).await?;
            let vote_key = format!("{:?}", result);
            *vote_counts.entry(vote_key).or_insert(0) += 1;
            results.push(result);
        }

        let total_votes = self.hooks.len();
        let required_votes = (total_votes as f64 * threshold).ceil() as usize;

        // Find winning result
        for (vote_key, count) in vote_counts {
            if count >= required_votes {
                // Find the first result matching this vote
                for result in results {
                    if format!("{:?}", result) == vote_key {
                        info!(
                            "Voting winner: {} with {}/{} votes (>= {} required)",
                            result.description(),
                            count,
                            total_votes,
                            required_votes
                        );
                        return Ok(result);
                    }
                }
            }
        }

        // No clear winner, default to Continue
        debug!("No voting winner, defaulting to Continue");
        Ok(HookResult::Continue)
    }
}

#[async_trait]
impl Hook for CompositeHook {
    async fn execute(&self, context: &mut HookContext) -> Result<HookResult> {
        if self.hooks.is_empty() {
            return Ok(HookResult::Continue);
        }

        match self.pattern {
            CompositionPattern::Sequential => self.execute_sequential(context).await,
            CompositionPattern::Parallel => self.execute_parallel(context).await,
            CompositionPattern::FirstMatch => self.execute_first_match(context).await,
            CompositionPattern::Voting { threshold } => self.execute_voting(context, threshold).await,
        }
    }

    fn metadata(&self) -> HookMetadata {
        self.metadata.clone()
    }

    fn should_execute(&self, context: &HookContext) -> bool {
        // Execute if any child hook should execute
        self.hooks.iter().any(|hook| hook.should_execute(context))
    }
}

impl ComposableHook for CompositeHook {
    fn can_compose_with(&self, other: &dyn Hook) -> bool {
        // Composite hooks can compose with any hook
        self.hooks.iter().all(|hook| {
            if let Some(composable) = hook.as_any().downcast_ref::<dyn ComposableHook>() {
                composable.can_compose_with(other)
            } else {
                true
            }
        })
    }

    fn composition_priority(&self) -> i32 {
        // Use the highest priority among child hooks
        self.hooks
            .iter()
            .filter_map(|hook| {
                hook.as_any()
                    .downcast_ref::<dyn ComposableHook>()
                    .map(|c| c.composition_priority())
            })
            .min()
            .unwrap_or(0)
    }
}

/// Builder for composite hooks
pub struct CompositeHookBuilder {
    name: String,
    pattern: CompositionPattern,
    hooks: Vec<ArcHook>,
    priority: Priority,
    tags: Vec<String>,
}

impl CompositeHookBuilder {
    pub fn new(name: &str, pattern: CompositionPattern) -> Self {
        Self {
            name: name.to_string(),
            pattern,
            hooks: Vec::new(),
            priority: Priority::NORMAL,
            tags: Vec::new(),
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

    pub fn with_priority(mut self, priority: Priority) -> Self {
        self.priority = priority;
        self
    }

    pub fn with_tag(mut self, tag: &str) -> Self {
        self.tags.push(tag.to_string());
        self
    }

    pub fn build(self) -> CompositeHook {
        let metadata = HookMetadata {
            name: self.name.clone(),
            description: Some(format!("Composite hook using {:?} pattern", self.pattern)),
            priority: self.priority,
            tags: self.tags,
            ..Default::default()
        };

        CompositeHook {
            pattern: self.pattern,
            hooks: self.hooks,
            metadata,
        }
    }
}

// Helper trait to enable downcasting
trait AsAny: Hook {
    fn as_any(&self) -> &dyn std::any::Any;
}

impl<T: Hook + 'static> AsAny for T {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[cfg(test)]
#[cfg_attr(test_category = "hook")]
mod tests {
    use super::*;
    use crate::traits::FnHook;
    use crate::types::{ComponentId, ComponentType, HookPoint};
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[cfg_attr(test_category = "unit")]
    #[tokio::test]
    async fn test_sequential_composition() {
        let counter = Arc::new(AtomicUsize::new(0));
        let counter1 = counter.clone();
        let counter2 = counter.clone();

        let hook1 = FnHook::new("hook1", move |_ctx| {
            counter1.fetch_add(1, Ordering::SeqCst);
            Ok(HookResult::Continue)
        });

        let hook2 = FnHook::new("hook2", move |_ctx| {
            counter2.fetch_add(10, Ordering::SeqCst);
            Ok(HookResult::Modified(serde_json::json!({"modified": true})))
        });

        let composite = CompositeHook::new("test_sequential", CompositionPattern::Sequential)
            .add_hook(Arc::new(hook1))
            .add_hook(Arc::new(hook2));

        let component_id = ComponentId::new(ComponentType::System, "test".to_string());
        let mut context = HookContext::new(HookPoint::SystemStartup, component_id);

        let result = composite.execute(&mut context).await.unwrap();

        // Should stop at first non-Continue result
        assert!(matches!(result, HookResult::Modified(_)));
        assert_eq!(counter.load(Ordering::SeqCst), 11); // Both executed
    }

    #[cfg_attr(test_category = "unit")]
    #[tokio::test]
    async fn test_parallel_composition() {
        let hook1 = FnHook::new("hook1", |_ctx| Ok(HookResult::Continue));
        
        let hook2 = FnHook::new("hook2", |_ctx| {
            Ok(HookResult::Modified(serde_json::json!({"value": 42})))
        });

        let hook3 = FnHook::new("hook3", |_ctx| Ok(HookResult::Continue));

        let composite = CompositeHook::new("test_parallel", CompositionPattern::Parallel)
            .add_hooks(vec![
                Arc::new(hook1),
                Arc::new(hook2),
                Arc::new(hook3),
            ]);

        let component_id = ComponentId::new(ComponentType::System, "test".to_string());
        let mut context = HookContext::new(HookPoint::SystemStartup, component_id);

        let result = composite.execute(&mut context).await.unwrap();

        // Should return the Modified result (highest priority non-Continue)
        assert!(matches!(result, HookResult::Modified(_)));
    }

    #[cfg_attr(test_category = "unit")]
    #[tokio::test]
    async fn test_first_match_composition() {
        let counter = Arc::new(AtomicUsize::new(0));
        let counter1 = counter.clone();
        let counter2 = counter.clone();
        let counter3 = counter.clone();

        let hook1 = FnHook::new("hook1", move |_ctx| {
            counter1.fetch_add(1, Ordering::SeqCst);
            Ok(HookResult::Continue)
        });

        let hook2 = FnHook::new("hook2", move |_ctx| {
            counter2.fetch_add(10, Ordering::SeqCst);
            Ok(HookResult::Cancel("Found match".to_string()))
        });

        let hook3 = FnHook::new("hook3", move |_ctx| {
            counter3.fetch_add(100, Ordering::SeqCst);
            Ok(HookResult::Continue)
        });

        let composite = CompositeHook::new("test_first_match", CompositionPattern::FirstMatch)
            .add_hook(Arc::new(hook1))
            .add_hook(Arc::new(hook2))
            .add_hook(Arc::new(hook3));

        let component_id = ComponentId::new(ComponentType::System, "test".to_string());
        let mut context = HookContext::new(HookPoint::SystemStartup, component_id);

        let result = composite.execute(&mut context).await.unwrap();

        assert!(matches!(result, HookResult::Cancel(_)));
        assert_eq!(counter.load(Ordering::SeqCst), 11); // First two executed
    }

    #[cfg_attr(test_category = "unit")]
    #[tokio::test]
    async fn test_voting_composition() {
        let hook1 = FnHook::new("hook1", |_ctx| Ok(HookResult::Continue));
        let hook2 = FnHook::new("hook2", |_ctx| Ok(HookResult::Continue));
        let hook3 = FnHook::new("hook3", |_ctx| Ok(HookResult::Cancel("Veto".to_string())));
        let hook4 = FnHook::new("hook4", |_ctx| Ok(HookResult::Continue));

        let composite = CompositeHook::new("test_voting", CompositionPattern::Voting { threshold: 0.5 })
            .add_hooks(vec![
                Arc::new(hook1),
                Arc::new(hook2),
                Arc::new(hook3),
                Arc::new(hook4),
            ]);

        let component_id = ComponentId::new(ComponentType::System, "test".to_string());
        let mut context = HookContext::new(HookPoint::SystemStartup, component_id);

        let result = composite.execute(&mut context).await.unwrap();

        // 3 out of 4 voted Continue (75% > 50% threshold)
        assert!(matches!(result, HookResult::Continue));
    }

    #[cfg_attr(test_category = "unit")]
    #[tokio::test]
    async fn test_empty_composite() {
        let composite = CompositeHook::new("empty", CompositionPattern::Sequential);
        
        assert!(composite.is_empty());
        assert_eq!(composite.len(), 0);

        let component_id = ComponentId::new(ComponentType::System, "test".to_string());
        let mut context = HookContext::new(HookPoint::SystemStartup, component_id);

        let result = composite.execute(&mut context).await.unwrap();
        assert!(matches!(result, HookResult::Continue));
    }

    #[cfg_attr(test_category = "unit")]
    #[tokio::test]
    async fn test_composite_builder() {
        let hook1 = FnHook::new("hook1", |_ctx| Ok(HookResult::Continue));
        let hook2 = FnHook::new("hook2", |_ctx| Ok(HookResult::Continue));

        let composite = CompositeHookBuilder::new("built_composite", CompositionPattern::Sequential)
            .add_hook(hook1)
            .add_arc_hook(Arc::new(hook2))
            .with_priority(Priority::HIGH)
            .with_tag("test")
            .with_tag("composite")
            .build();

        assert_eq!(composite.metadata.name, "built_composite");
        assert_eq!(composite.metadata.priority, Priority::HIGH);
        assert_eq!(composite.metadata.tags.len(), 2);
        assert_eq!(composite.len(), 2);
    }

    #[cfg_attr(test_category = "unit")]
    #[tokio::test]
    async fn test_nested_composite() {
        let hook1 = FnHook::new("hook1", |_ctx| Ok(HookResult::Continue));
        let hook2 = FnHook::new("hook2", |_ctx| Ok(HookResult::Continue));

        let inner_composite = CompositeHook::new("inner", CompositionPattern::Sequential)
            .add_hook(Arc::new(hook1))
            .add_hook(Arc::new(hook2));

        let hook3 = FnHook::new("hook3", |_ctx| {
            Ok(HookResult::Modified(serde_json::json!({"nested": true})))
        });

        let outer_composite = CompositeHook::new("outer", CompositionPattern::Sequential)
            .add_hook(Arc::new(inner_composite))
            .add_hook(Arc::new(hook3));

        let component_id = ComponentId::new(ComponentType::System, "test".to_string());
        let mut context = HookContext::new(HookPoint::SystemStartup, component_id);

        let result = outer_composite.execute(&mut context).await.unwrap();
        assert!(matches!(result, HookResult::Modified(_)));
    }
}