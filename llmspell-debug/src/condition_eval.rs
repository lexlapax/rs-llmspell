//! Breakpoint condition evaluation - Layer 3 of three-layer architecture
//!
//! Provides condition evaluation for breakpoints using `SharedExecutionContext`

use crate::{Breakpoint, SharedExecutionContext};
use anyhow::{anyhow, Result};
use mlua::Lua;

/// Breakpoint condition evaluator using `SharedExecutionContext`
pub struct ConditionEvaluator;

impl ConditionEvaluator {
    /// Evaluate breakpoint condition with `SharedExecutionContext`
    ///
    /// # Errors
    ///
    /// Returns an error if the condition evaluation fails or if Lua operations fail
    pub fn should_break_with_context(
        breakpoint: &mut Breakpoint,
        lua: &Lua,
        context: &SharedExecutionContext,
    ) -> Result<bool> {
        // Use existing should_break() logic first
        if !breakpoint.should_break() {
            return Ok(false);
        }

        // Update hit counter (using existing fields)
        breakpoint.current_hits += 1;

        // Evaluate condition with SharedExecutionContext
        if let Some(condition) = &breakpoint.condition {
            match Self::evaluate_condition_with_context(lua, condition, context) {
                Ok(result) => Ok(result),
                Err(e) => {
                    // Use tracing for error logging (following diagnostics pattern)
                    tracing::error!(
                        "Breakpoint condition error at {}:{}: {}",
                        breakpoint.source,
                        breakpoint.line,
                        e
                    );
                    Ok(true) // Break anyway for safety
                }
            }
        } else {
            Ok(true) // No condition means always break
        }
    }

    /// Evaluate condition with `SharedExecutionContext` variables
    fn evaluate_condition_with_context(
        lua: &Lua,
        condition: &str,
        context: &SharedExecutionContext,
    ) -> Result<bool> {
        // Create safe evaluation environment
        let env = lua.create_table()?;

        // Use SharedExecutionContext variables instead of extracting locals
        for (name, value) in &context.variables {
            // Convert JSON value to Lua value
            let lua_value = Self::json_to_lua_value(lua, value)?;
            env.set(name.clone(), lua_value)?;
        }

        // Add current location context
        if let Some(location) = &context.location {
            env.set("__current_line__", location.line)?;
            env.set("__current_file__", location.source.clone())?;
        }

        // Add performance metrics context
        env.set(
            "__execution_count__",
            context.performance_metrics.execution_count,
        )?;
        env.set(
            "__function_time_us__",
            context.performance_metrics.function_time_us,
        )?;

        // Evaluate condition as Lua expression
        let chunk = lua.load(condition).set_environment(env);

        chunk
            .eval::<bool>()
            .map_err(|e| anyhow!("Condition evaluation failed: {}", e))
    }

    /// Convert JSON values from `SharedExecutionContext` to Lua values
    fn json_to_lua_value<'lua>(
        lua: &'lua Lua,
        json_value: &serde_json::Value,
    ) -> Result<mlua::Value<'lua>> {
        match json_value {
            serde_json::Value::Null => Ok(mlua::Value::Nil),
            serde_json::Value::Bool(b) => Ok(mlua::Value::Boolean(*b)),
            serde_json::Value::Number(n) => n
                .as_f64()
                .map_or(Ok(mlua::Value::Nil), |f| Ok(mlua::Value::Number(f))),
            serde_json::Value::String(s) => Ok(mlua::Value::String(lua.create_string(s)?)),
            serde_json::Value::Array(_) | serde_json::Value::Object(_) => {
                // For complex types, use string representation
                let formatted = serde_json::to_string_pretty(json_value)?;
                Ok(mlua::Value::String(lua.create_string(formatted)?))
            }
        }
    }
}

/// Condition template builder for common debugging patterns
pub struct ConditionTemplates;

impl ConditionTemplates {
    /// Create condition to break when variable equals value
    #[must_use]
    pub fn variable_equals(var_name: &str, value: &str) -> String {
        format!("{var_name} == {value}")
    }

    /// Create condition to break when variable is greater than value
    #[must_use]
    pub fn variable_greater_than(var_name: &str, value: &str) -> String {
        format!("{var_name} > {value}")
    }

    /// Create condition to break when variable is nil
    #[must_use]
    pub fn variable_is_nil(var_name: &str) -> String {
        format!("{var_name} == nil")
    }

    /// Create condition to break when execution count exceeds threshold
    #[must_use]
    pub fn execution_count_exceeds(threshold: u32) -> String {
        format!("__execution_count__ > {threshold}")
    }

    /// Create condition to break when function time exceeds threshold (microseconds)
    #[must_use]
    pub fn function_time_exceeds(threshold_us: u64) -> String {
        format!("__function_time_us__ > {threshold_us}")
    }

    /// Create condition to break at specific location
    #[must_use]
    pub fn at_location(file: &str, line: u32) -> String {
        format!("__current_file__ == \"{file}\" and __current_line__ == {line}")
    }
}

/// Condition validator for breakpoint conditions
pub struct ConditionValidator;

impl ConditionValidator {
    /// Validate a breakpoint condition syntax
    ///
    /// # Errors
    ///
    /// Returns an error if the condition syntax is invalid or references undefined variables
    pub fn validate_condition(
        lua: &Lua,
        condition: &str,
        context: &SharedExecutionContext,
    ) -> Result<()> {
        // Create temporary environment for validation
        let env = lua.create_table()?;

        // Add context variables for validation
        for (name, value) in &context.variables {
            match value {
                serde_json::Value::Bool(b) => env.set(name.clone(), *b)?,
                serde_json::Value::Number(n) => {
                    if let Some(f) = n.as_f64() {
                        env.set(name.clone(), f)?;
                    }
                }
                serde_json::Value::String(s) => env.set(name.clone(), s.clone())?,
                _ => env.set(name.clone(), mlua::Value::Nil)?,
            }
        }

        // Add location context
        if let Some(location) = &context.location {
            env.set("__current_line__", location.line)?;
            env.set("__current_file__", location.source.clone())?;
        }

        // Add performance metrics context
        env.set(
            "__execution_count__",
            context.performance_metrics.execution_count,
        )?;
        env.set(
            "__function_time_us__",
            context.performance_metrics.function_time_us,
        )?;

        // Try to load and evaluate the condition as a boolean
        let chunk = lua.load(condition).set_environment(env);
        chunk
            .eval::<bool>()
            .map_err(|e| anyhow!("Condition validation failed: {}", e))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SharedExecutionContext;

    #[tokio::test]
    async fn test_simple_condition_evaluation() {
        let lua = Lua::new();
        let mut context = SharedExecutionContext::new();

        // Add test variables
        context.variables.insert(
            "x".to_string(),
            serde_json::Value::Number(serde_json::Number::from(42)),
        );

        let mut breakpoint =
            Breakpoint::new("test.lua".to_string(), 10).with_condition("x > 40".to_string());

        let should_break =
            ConditionEvaluator::should_break_with_context(&mut breakpoint, &lua, &context).unwrap();
        assert!(should_break);
    }

    #[tokio::test]
    async fn test_condition_templates() {
        let condition = ConditionTemplates::variable_equals("name", "\"test\"");
        assert_eq!(condition, "name == \"test\"");

        let condition = ConditionTemplates::variable_greater_than("count", "10");
        assert_eq!(condition, "count > 10");

        let condition = ConditionTemplates::execution_count_exceeds(1000);
        assert_eq!(condition, "__execution_count__ > 1000");
    }

    #[tokio::test]
    async fn test_condition_validation() {
        let lua = Lua::new();
        let mut context = SharedExecutionContext::new();

        context.variables.insert(
            "x".to_string(),
            serde_json::Value::Number(serde_json::Number::from(10)),
        );

        // Valid condition should pass
        let result = ConditionValidator::validate_condition(&lua, "x > 5", &context);
        assert!(result.is_ok());

        // Invalid condition should fail
        let result = ConditionValidator::validate_condition(&lua, "invalid_var > 5", &context);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_json_to_lua_conversion() {
        let lua = Lua::new();

        // Test different JSON value types
        let bool_val = serde_json::Value::Bool(true);
        let lua_val = ConditionEvaluator::json_to_lua_value(&lua, &bool_val).unwrap();
        assert!(matches!(lua_val, mlua::Value::Boolean(true)));

        let num_val = serde_json::Value::Number(serde_json::Number::from(42));
        let lua_val = ConditionEvaluator::json_to_lua_value(&lua, &num_val).unwrap();
        if let mlua::Value::Number(n) = lua_val {
            assert!((n - 42.0).abs() < f64::EPSILON);
        } else {
            panic!("Expected number value");
        }
    }
}
