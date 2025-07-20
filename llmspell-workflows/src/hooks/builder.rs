//! ABOUTME: Hook builder pattern for workflow integration
//! ABOUTME: Prepares workflows for Phase 4 hook system without breaking changes

/// Builder extension for adding hooks to workflows
pub trait HookBuilder {
    /// Add a logging hook that works now (before Phase 4)
    fn with_logging_hooks(self) -> Self;

    /// Prepare hook points for Phase 4 (no-op for now)
    fn with_lifecycle_hooks(self) -> Self;
}

/// Add hook preparation to workflow builders
macro_rules! impl_hook_builder {
    ($builder:ty) => {
        impl HookBuilder for $builder {
            fn with_logging_hooks(self) -> Self {
                // These will be added to the workflow when built
                // For now, just mark that hooks are requested
                self
            }

            fn with_lifecycle_hooks(self) -> Self {
                // Placeholder for Phase 4
                self
            }
        }
    };
}

// Apply to all workflow builders
impl_hook_builder!(crate::sequential::SequentialWorkflowBuilder);
impl_hook_builder!(crate::conditional::ConditionalWorkflowBuilder);
impl_hook_builder!(crate::r#loop::LoopWorkflowBuilder);
impl_hook_builder!(crate::parallel::ParallelWorkflowBuilder);
