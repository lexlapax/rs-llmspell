//! ABOUTME: llmspell-workflows implementation crate
//! ABOUTME: Provides workflow patterns for orchestrating agents and tools

pub mod conditional;
pub mod conditions;
pub mod error_handling;
pub mod hooks;
pub mod r#loop;
pub mod parallel;
pub mod sequential;
pub mod shared_state;
pub mod state;
pub mod step_executor;
pub mod traits;
pub mod types;

// Re-export main functionality for convenience
pub use types::{
    StepExecutionContext, WorkflowConfig, WorkflowInput, WorkflowOutput, WorkflowState,
};

pub use traits::{ErrorStrategy, StepResult, StepType, WorkflowStatus, WorkflowStep};

pub use sequential::{SequentialWorkflow, SequentialWorkflowBuilder, SequentialWorkflowResult};

pub use conditional::{
    BranchExecutionResult, ConditionalBranch, ConditionalWorkflow, ConditionalWorkflowBuilder,
    ConditionalWorkflowConfig, ConditionalWorkflowResult,
};

pub use conditions::{Condition, ConditionEvaluationContext, ConditionEvaluator, ConditionResult};

pub use error_handling::{
    ErrorAction, ErrorHandler, RecoveryAction, WorkflowErrorAnalysis, WorkflowErrorType,
};
pub use state::{ExecutionStats, StateManager};
pub use step_executor::StepExecutor;

pub use r#loop::{
    BreakCondition, LoopConfig, LoopIterator, LoopWorkflow, LoopWorkflowBuilder,
    LoopWorkflowResult, ResultAggregation,
};

pub use parallel::{
    BranchResult, ParallelBranch, ParallelConfig, ParallelWorkflow, ParallelWorkflowBuilder,
    ParallelWorkflowResult,
};
