//! ABOUTME: Hook infrastructure for workflows (preparing for Phase 4)
//! ABOUTME: Defines hook points and interfaces for future implementation

pub mod builder;
pub mod integration;
pub mod lifecycle;
pub mod types;

pub use builder::HookBuilder;
pub use integration::{
    HookableWorkflowExecution, WorkflowExecutionPhase, WorkflowExecutor, WorkflowHookContext,
    WorkflowLifecycleConfig,
};
pub use lifecycle::{HookPoint, WithHooks, WorkflowHooks};
pub use types::{HookContext, HookError, HookResult, StepContext};

/// Hook function type - will be expanded in Phase 4
pub type HookFn = Box<dyn Fn(&HookContext) -> HookResult + Send + Sync>;
