//! ABOUTME: Language-agnostic workflow parameter conversion utilities
//! ABOUTME: Provides common conversion functions used by all script engines

// Re-export for backward compatibility and convenience
pub use crate::workflow_conversion_core::{
    json_to_workflow_params, parse_error_strategy, WorkflowParams,
};
