//! ABOUTME: State builder pattern for workflow integration
//! ABOUTME: Allows workflows to opt-in to state management

use super::WorkflowStateManager;

/// Builder extension for adding state management to workflows
pub trait StateBuilder {
    /// Enable state management for this workflow
    fn with_state_manager(self, manager: WorkflowStateManager) -> Self;

    /// Create a new state manager and enable it for this workflow
    fn with_state(self) -> Self
    where
        Self: Sized,
    {
        self.with_state_manager(WorkflowStateManager::new())
    }
}

/// Add state management to workflow builders
#[allow(unused_macros)]
macro_rules! impl_state_builder {
    ($builder:ty) => {
        impl StateBuilder for $builder {
            fn with_state_manager(mut self, manager: WorkflowStateManager) -> Self {
                // This will be stored in the builder and passed to the workflow
                // when built. For now, we store it in the builder's state.
                // Each builder will need to add a state_manager field.
                self
            }
        }
    };
}

// TODO: Once workflow builders add state_manager fields, uncomment these:
// impl_state_builder!(crate::sequential::SequentialWorkflowBuilder);
// impl_state_builder!(crate::conditional::ConditionalWorkflowBuilder);
// impl_state_builder!(crate::r#loop::LoopWorkflowBuilder);
// impl_state_builder!(crate::parallel::ParallelWorkflowBuilder);
