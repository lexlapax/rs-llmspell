//! Breakpoint condition evaluator for slow path execution
//!
//! Evaluates breakpoint conditions only in the slow path after the fast path
//! confirms a potential breakpoint hit. Uses cached compiled conditions and
//! batched context variables for efficient evaluation.

use crate::execution_bridge::Breakpoint;
use crate::execution_context::SharedExecutionContext;
use crate::lua::debug_cache::{CompiledCondition, ContextBatcher, DebugStateCache};
use mlua::{Lua, Result as LuaResult, Value};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Condition evaluator for breakpoint conditions
pub struct ConditionEvaluator;

impl ConditionEvaluator {
    /// Pre-compile a condition expression for faster evaluation
    ///
    /// # Errors
    ///
    /// Returns an error if the condition cannot be compiled
    pub fn compile_condition(lua: &Lua, expression: &str) -> LuaResult<CompiledCondition> {
        // Create a Lua expression that returns a boolean
        let lua_expr = format!("return ({expression})");

        // Try to compile it to catch syntax errors early
        let chunk = lua.load(&lua_expr);
        let compiled = chunk.into_function()?;

        // Store the compiled bytecode
        let bytecode = compiled.dump(false);

        Ok(CompiledCondition {
            expression: expression.to_string(),
            compiled_chunk: Some(bytecode),
        })
    }

    /// Evaluate a breakpoint condition in the slow path
    ///
    /// This is called ONLY after `might_break_at()` returns true and we've
    /// confirmed there's a breakpoint with a condition at this location.
    pub fn evaluate_in_slow_path(
        breakpoint: &Breakpoint,
        cache: &DebugStateCache,
        _context: &ContextBatcher,
        shared_context: Arc<RwLock<SharedExecutionContext>>,
        lua: &Lua,
    ) -> bool {
        // If no condition, always break
        let Some(ref condition_expr) = breakpoint.condition else {
            return true;
        };

        let source = &breakpoint.source;
        let line = breakpoint.line;

        // Check cache first
        if let Some((result, gen)) = cache.get_cached_condition(source, line) {
            if gen == cache.generation() {
                return result; // Use cached result
            }
        }

        // Get the compiled condition from cache
        let compiled_condition = cache.get_condition(source, line);

        // Evaluate the condition
        let result =
            match Self::evaluate_condition(lua, condition_expr, compiled_condition, shared_context)
            {
                Ok(passes) => passes,
                Err(e) => {
                    // Log error but don't block execution
                    tracing::warn!(
                        "Failed to evaluate breakpoint condition at {}:{}: {}",
                        source,
                        line,
                        e
                    );
                    // On error, we break to be safe
                    true
                }
            };

        // Cache the result
        cache.cache_condition_result(source, line, result);
        result
    }

    /// Evaluate a condition expression using Lua
    fn evaluate_condition(
        lua: &Lua,
        expression: &str,
        compiled: Option<Arc<CompiledCondition>>,
        shared_context: Arc<RwLock<SharedExecutionContext>>,
    ) -> LuaResult<bool> {
        // First, inject variables from shared context into Lua globals
        // Only inject if we're in an async context (not in unit tests)
        if tokio::runtime::Handle::try_current().is_ok() {
            Self::inject_context_variables(lua, shared_context)?;
        }

        // Use compiled chunk if available, otherwise compile on the fly
        let result = if let Some(compiled) = compiled {
            if let Some(ref bytecode) = compiled.compiled_chunk {
                // Load from bytecode
                let chunk = lua
                    .load(bytecode.as_slice())
                    .set_mode(mlua::ChunkMode::Binary);
                chunk.eval::<Value>()?
            } else {
                // Fallback to text evaluation
                let expr = format!("return ({expression})");
                lua.load(&expr).eval::<Value>()?
            }
        } else {
            // No compiled version, evaluate directly
            let expr = format!("return ({expression})");
            lua.load(&expr).eval::<Value>()?
        };

        // Convert to boolean
        Ok(match result {
            Value::Boolean(b) => b,
            Value::Nil => false,
            Value::Number(n) => n != 0.0,
            Value::Integer(i) => i != 0,
            _ => true, // Non-nil values are truthy in Lua
        })
    }

    /// Inject context variables into Lua globals for condition evaluation
    fn inject_context_variables(
        lua: &Lua,
        shared_context: Arc<RwLock<SharedExecutionContext>>,
    ) -> LuaResult<()> {
        // Use block_on to read context synchronously
        let variables = crate::lua::sync_utils::block_on_async(
            "read_context_variables",
            async move {
                let ctx = shared_context.read().await;
                Ok::<_, std::io::Error>(ctx.variables.clone())
            },
            None,
        )
        .map_err(|e| mlua::Error::ExternalError(Arc::new(e)))?;

        // Inject each variable into Lua globals
        let globals = lua.globals();
        for (name, value) in variables {
            // Convert JSON value to Lua value
            let lua_value = serde_json_to_lua(lua, &value)?;
            globals.set(name, lua_value)?;
        }

        Ok(())
    }
}

/// Convert a `serde_json` Value to a Lua Value
fn serde_json_to_lua<'lua>(lua: &'lua Lua, json: &serde_json::Value) -> LuaResult<Value<'lua>> {
    match json {
        serde_json::Value::Null => Ok(Value::Nil),
        serde_json::Value::Bool(b) => Ok(Value::Boolean(*b)),
        serde_json::Value::Number(n) => Ok(n
            .as_i64()
            .map(Value::Integer)
            .or_else(|| n.as_f64().map(Value::Number))
            .unwrap_or(Value::Nil)),
        serde_json::Value::String(s) => Ok(Value::String(lua.create_string(s)?)),
        serde_json::Value::Array(arr) => {
            let table = lua.create_table()?;
            for (i, v) in arr.iter().enumerate() {
                table.set(i + 1, serde_json_to_lua(lua, v)?)?;
            }
            Ok(Value::Table(table))
        }
        serde_json::Value::Object(obj) => {
            let table = lua.create_table()?;
            for (k, v) in obj {
                table.set(k.as_str(), serde_json_to_lua(lua, v)?)?;
            }
            Ok(Value::Table(table))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compile_condition() {
        let lua = Lua::new();

        // Valid condition
        let condition = ConditionEvaluator::compile_condition(&lua, "x > 10").unwrap();
        assert_eq!(condition.expression, "x > 10");
        assert!(condition.compiled_chunk.is_some());

        // Invalid condition should fail
        let result = ConditionEvaluator::compile_condition(&lua, "x >>>> 10");
        assert!(result.is_err());
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_evaluate_simple_conditions() {
        let lua = Lua::new();
        let shared_context = Arc::new(RwLock::new(SharedExecutionContext::new()));

        // Set up a variable in Lua
        lua.globals().set("x", 15).unwrap();

        // Test various conditions
        let test_cases = vec![
            ("x > 10", true),
            ("x < 10", false),
            ("x == 15", true),
            ("x ~= 15", false),
            ("x >= 15", true),
            ("true", true),
            ("false", false),
            ("nil", false),
        ];

        for (expr, expected) in test_cases {
            let result =
                ConditionEvaluator::evaluate_condition(&lua, expr, None, shared_context.clone())
                    .unwrap();
            assert_eq!(result, expected, "Expression '{expr}' failed");
        }
    }
}
