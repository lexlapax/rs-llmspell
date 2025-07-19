//! ABOUTME: Basic workflow implementations module
//! ABOUTME: Contains foundational workflow patterns with memory-based state management

pub mod error_handling;
pub mod sequential;
pub mod state;
pub mod step_executor;
pub mod traits;
pub mod types;

// Re-export main types and traits for convenience
pub use error_handling::{BasicErrorHandler, ErrorAction, WorkflowErrorAnalysis};
pub use sequential::BasicSequentialWorkflow;
pub use state::{BasicStateManager, ExecutionStats};
pub use step_executor::BasicStepExecutor;
pub use traits::{
    BasicErrorStrategy, BasicStepResult, BasicStepType, BasicWorkflow, BasicWorkflowStatus,
    BasicWorkflowStep,
};
pub use types::{
    BasicWorkflowConfig, BasicWorkflowInput, BasicWorkflowOutput, BasicWorkflowState,
    StepExecutionContext,
};
