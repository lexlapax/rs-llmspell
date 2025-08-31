//! Script-agnostic breakpoint condition evaluator for debugging
//!
//! This module provides trait definitions for evaluating breakpoint conditions
//! in any supported script language. Implementations are provided in the
//! respective language modules (lua/, js/, python/, etc.).

use crate::execution_bridge::Breakpoint;
use crate::execution_context::SharedExecutionContext;
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Script-agnostic compiled condition representation
#[derive(Debug, Clone)]
pub struct CompiledCondition {
    /// Original expression text
    pub expression: String,
    /// Script-specific compiled bytecode or AST
    pub compiled_data: Option<Vec<u8>>,
    /// Metadata for script engine
    pub metadata: HashMap<String, JsonValue>,
}

/// Debug context for condition evaluation
pub trait DebugContext: Send + Sync {
    /// Get all available variables as JSON values
    fn get_variables(&self) -> HashMap<String, JsonValue>;

    /// Get a specific variable by name
    fn get_variable(&self, name: &str) -> Option<JsonValue>;

    /// Get current execution location
    fn get_location(&self) -> Option<(String, u32)>; // (source, line)
}

/// Script-agnostic condition evaluator trait
pub trait ConditionEvaluator: Send + Sync {
    /// Compile a condition expression for efficient repeated evaluation
    ///
    /// # Arguments
    /// * `expression` - The condition expression to compile
    ///
    /// # Returns
    /// * `Ok(CompiledCondition)` - Successfully compiled condition
    ///
    /// # Errors
    /// * Returns an error if the expression has invalid syntax
    /// * Returns an error if compilation fails
    fn compile_condition(&self, expression: &str) -> Result<CompiledCondition, Box<dyn Error>>;

    /// Evaluate a condition expression in the context of debugging
    ///
    /// # Arguments  
    /// * `expression` - The condition expression to evaluate
    /// * `compiled` - Optional pre-compiled condition for efficiency
    /// * `context` - Debug context providing variables and location
    ///
    /// # Returns
    /// * `Ok(true)` - Condition evaluates to true (should break)
    /// * `Ok(false)` - Condition evaluates to false (should not break)  
    ///
    /// # Errors
    /// * Returns an error if evaluation fails (defaults to breaking for safety)
    /// * Returns an error if context variables cannot be accessed
    fn evaluate_condition(
        &self,
        expression: &str,
        compiled: Option<&CompiledCondition>,
        context: &dyn DebugContext,
    ) -> Result<bool, Box<dyn Error>>;

    /// Evaluate a breakpoint condition in the slow path
    ///
    /// This is the main entry point called from the debug hooks.
    /// It handles caching, error recovery, and result logging.
    fn evaluate_breakpoint(&self, breakpoint: &Breakpoint, context: &dyn DebugContext) -> bool {
        // If no condition, always break
        let Some(ref condition_expr) = breakpoint.condition else {
            return true;
        };

        // Attempt to evaluate the condition
        match self.evaluate_condition(condition_expr, None, context) {
            Ok(result) => result,
            Err(e) => {
                // Log error but don't block execution - break for safety
                tracing::warn!(
                    "Failed to evaluate breakpoint condition at {}:{}: {}",
                    breakpoint.source,
                    breakpoint.line,
                    e
                );
                true
            }
        }
    }
}

/// Implementation of `DebugContext` using `SharedExecutionContext`
pub struct SharedDebugContext {
    shared_context: Arc<RwLock<SharedExecutionContext>>,
}

impl SharedDebugContext {
    /// Create a new `SharedDebugContext`
    #[must_use]
    pub const fn new(shared_context: Arc<RwLock<SharedExecutionContext>>) -> Self {
        Self { shared_context }
    }

    /// Read variables from shared context synchronously
    fn read_variables_sync(&self) -> HashMap<String, JsonValue> {
        // Use block_on pattern similar to existing code
        crate::lua::sync_utils::block_on_async(
            "read_debug_context_variables",
            {
                let context = self.shared_context.clone();
                async move {
                    let ctx = context.read().await;
                    Ok::<_, std::io::Error>(ctx.variables.clone())
                }
            },
            None,
        )
        .unwrap_or_else(|e| {
            tracing::warn!("Failed to read debug context variables: {}", e);
            HashMap::new()
        })
    }
}

impl DebugContext for SharedDebugContext {
    fn get_variables(&self) -> HashMap<String, JsonValue> {
        self.read_variables_sync()
    }

    fn get_variable(&self, name: &str) -> Option<JsonValue> {
        self.get_variables().get(name).cloned()
    }

    fn get_location(&self) -> Option<(String, u32)> {
        crate::lua::sync_utils::block_on_async(
            "read_debug_context_location",
            {
                let context = self.shared_context.clone();
                async move {
                    let ctx = context.read().await;
                    Ok::<_, std::io::Error>(
                        ctx.location
                            .as_ref()
                            .map(|loc| (loc.source.clone(), loc.line)),
                    )
                }
            },
            None,
        )
        .unwrap_or_else(|e| {
            tracing::warn!("Failed to read debug context location: {}", e);
            None
        })
    }
}

/// Factory for creating script-specific condition evaluators
pub trait ConditionEvaluatorFactory {
    /// Create a condition evaluator for the specified script engine
    fn create_evaluator(&self) -> Box<dyn ConditionEvaluator>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shared_debug_context_creation() {
        let shared_context = Arc::new(RwLock::new(SharedExecutionContext::new()));
        let debug_context = SharedDebugContext::new(shared_context);

        // Should not panic
        let _vars = debug_context.get_variables();
    }
}
