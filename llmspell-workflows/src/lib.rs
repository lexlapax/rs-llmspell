//! ABOUTME: llmspell-workflows implementation crate
//! ABOUTME: Provides workflow patterns for orchestrating agents and tools

pub mod basic;

// Re-export main functionality for convenience
pub use basic::{
    BasicErrorStrategy, BasicSequentialWorkflow, BasicStepResult, BasicStepType,
    BasicWorkflowConfig, BasicWorkflowInput, BasicWorkflowOutput, BasicWorkflowState,
    BasicWorkflowStatus, BasicWorkflowStep,
};

// Re-export traits
pub use basic::traits::BasicWorkflow;
