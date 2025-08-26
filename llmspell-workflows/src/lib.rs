//! ABOUTME: llmspell-workflows implementation crate
//! ABOUTME: Provides workflow patterns for orchestrating agents and tools

pub mod adapters;
pub mod conditional;
pub mod conditions;
pub mod error_handling;
pub mod executor;
pub mod factory;
pub mod hooks;
/// Loop workflow for iterative execution patterns
pub mod r#loop;
/// Parallel workflow for concurrent execution of multiple steps
pub mod parallel;
pub mod result;
pub mod sequential;
pub mod shared_state;
pub mod state;
pub mod step_executor;
pub mod traits;
pub mod types;

// Test utilities (only available during testing)
#[cfg(test)]
pub mod test_utils;

// Re-export main functionality for convenience
pub use types::{
    StepExecutionContext, WorkflowConfig, WorkflowInput, WorkflowOutput, WorkflowState,
};

pub use traits::{ErrorStrategy, StepResult, StepType, WorkflowStatus, WorkflowStep};

pub use sequential::{SequentialWorkflow, SequentialWorkflowBuilder};

pub use conditional::{
    BranchExecutionResult, ConditionalBranch, ConditionalWorkflow, ConditionalWorkflowBuilder,
    ConditionalWorkflowConfig, ConditionalWorkflowResult,
};

pub use conditions::{Condition, ConditionEvaluationContext, ConditionEvaluator, ConditionResult};

pub use error_handling::{
    ErrorAction, ErrorHandler, RecoveryAction, WorkflowErrorAnalysis, WorkflowErrorType,
};

// Re-export state components (both memory and persistent)
pub use state::{
    ExecutionStats, PersistentWorkflowState, PersistentWorkflowStateManager, RetryStatistics,
    StateManager, StepStatistics, WorkflowCheckpoint, WorkflowExecutionStats,
    WorkflowStatePersistence,
};
pub use step_executor::StepExecutor;

pub use r#loop::{
    BreakCondition, LoopConfig, LoopIterator, LoopWorkflow, LoopWorkflowBuilder,
    LoopWorkflowResult, ResultAggregation,
};

pub use parallel::{
    BranchResult, ParallelBranch, ParallelConfig, ParallelWorkflow, ParallelWorkflowBuilder,
    ParallelWorkflowResult,
};

// Re-export adapters for workflow-agent integration
pub use adapters::prelude::{
    agent_to_workflow_input, agent_to_workflow_output, workflow_to_agent_input,
    workflow_to_agent_output,
};
pub use adapters::{WorkflowInputAdapter, WorkflowOutputAdapter};

// Re-export factory types
pub use factory::{
    DefaultWorkflowFactory, TemplateWorkflowFactory, WorkflowFactory, WorkflowParams,
    WorkflowTemplate, WorkflowType,
};

// Re-export executor types
pub use executor::{
    DefaultWorkflowExecutor, ExecutionContext, ExecutionHook, ExecutionMetrics, WorkflowExecutor,
};

// Re-export unified result types
pub use result::{
    WorkflowError, WorkflowResult, WorkflowResultExt, WorkflowType as ResultWorkflowType,
};
