//! ABOUTME: Composition patterns for agents and tools
//! ABOUTME: Enables complex workflows through agent and tool orchestration

pub mod tool_composition;

// Re-export main composition types
pub use tool_composition::{
    CompositionError, CompositionErrorStrategy, CompositionMetrics, CompositionResult,
    CompositionStep, ConditionType, DataFlow, DataTransform, ExecutionCondition, OutputTransform,
    RetryConfig, StepErrorStrategy, StepMetrics, StepResult, ToolComposition, ToolProvider,
};
