//! Interactive debugging infrastructure for `LLMSpell`
//!
//! This crate implements enhanced interactive debugging capabilities following the
//! three-layer architecture established in Phase 9.1:
//! - Bridge layer: Interactive debugging interface
//! - Global layer: Session management
//! - Language layer: Condition evaluation
//!
//! The architecture integrates with existing `ExecutionBridge` and uses unified types.

pub mod condition_eval;
pub mod interactive; // Layer 1: Interactive debugging interface
pub mod session_manager; // Session management // Breakpoint condition evaluation

// Re-export ExecutionBridge, ExecutionManager from llmspell-bridge
pub use llmspell_bridge::{
    execution_bridge::{
        Breakpoint, DebugCommand, DebugState, ExecutionLocation, ExecutionManager, PauseReason,
        StackFrame, Variable,
    },
    execution_context::SharedExecutionContext,
};

// Re-export main interactive debugging interface
pub use condition_eval::{ConditionEvaluator, ConditionTemplates, ConditionValidator};
pub use interactive::InteractiveDebugger;
pub use session_manager::{DebugSession, DebugSessionManager};
