//! Lua-specific implementation of condition evaluation
//!
//! This module provides the Lua implementation of the `ConditionEvaluator` trait,
//! handling Lua-specific compilation and evaluation of breakpoint conditions.

use crate::condition_evaluator::{CompiledCondition, ConditionEvaluator, DebugContext};
use mlua::{Lua, Value};
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::error::Error;
use tracing::{debug, warn};

/// Lua-specific condition evaluator
pub struct LuaConditionEvaluator;

impl Default for LuaConditionEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

impl LuaConditionEvaluator {
    /// Create a new Lua condition evaluator
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Evaluate condition with a Lua instance (main entry point)
    ///
    /// # Errors
    /// * Returns an error if the expression cannot be evaluated
    /// * Returns an error if context variables cannot be set in Lua
    /// * Returns an error if the Lua evaluation fails
    pub fn evaluate_condition_with_lua(
        &self,
        expression: &str,
        compiled: Option<&CompiledCondition>,
        context: &dyn DebugContext,
        lua: &Lua,
    ) -> Result<bool, Box<dyn Error>> {
        // Use compiled condition if available, otherwise use raw expression
        let expr = compiled.map_or(expression, |compiled| &compiled.expression);

        debug!("Evaluating Lua condition: {}", expr);

        // Set up context variables in Lua global scope
        Self::setup_context_variables(context, lua)?;

        // Evaluate the expression
        match lua.load(expr).eval::<Value>() {
            Ok(value) => {
                // Convert Lua value to boolean
                let result = Self::lua_value_to_bool(&value);
                debug!("Condition '{}' evaluated to: {}", expr, result);
                Ok(result)
            }
            Err(e) => {
                warn!("Failed to evaluate Lua condition '{}': {}", expr, e);
                Err(Box::new(e))
            }
        }
    }

    /// Set up context variables in Lua global scope
    ///
    /// # Errors
    /// * Returns an error if JSON to Lua conversion fails
    /// * Returns an error if setting global variables fails
    fn setup_context_variables(
        context: &dyn DebugContext,
        lua: &Lua,
    ) -> Result<(), Box<dyn Error>> {
        let variables = context.get_variables();

        for (name, value) in variables {
            // Convert JSON value to Lua value
            let lua_value = json_to_lua_value(lua, &value)?;
            lua.globals().set(name, lua_value)?;
        }

        // Set up special context variables
        if let Some((source, line)) = context.get_location() {
            lua.globals().set("__source", source)?;
            lua.globals().set("__line", line)?;
        }

        Ok(())
    }

    /// Convert Lua value to boolean
    fn lua_value_to_bool(value: &Value) -> bool {
        match value {
            Value::Nil => false,
            Value::Boolean(b) => *b,
            Value::Integer(i) => *i != 0,
            Value::Number(n) => *n != 0.0,
            Value::String(s) => !s.to_str().map_or(true, str::is_empty),
            Value::Table(_)
            | Value::Function(_)
            | Value::Thread(_)
            | Value::UserData(_)
            | Value::LightUserData(_)
            | Value::Error(_) => true,
        }
    }
}

impl ConditionEvaluator for LuaConditionEvaluator {
    fn compile_condition(&self, expression: &str) -> Result<CompiledCondition, Box<dyn Error>> {
        debug!("Compiling Lua condition: {}", expression);

        // For now, we can't pre-compile without a Lua instance
        // Just validate the expression syntax would be valid Lua
        if expression.trim() == "" {
            Err("Empty expression".into())
        } else {
            // Basic validation passed
            let mut metadata = HashMap::new();
            metadata.insert("language".to_string(), JsonValue::String("lua".to_string()));
            metadata.insert(
                "expression_type".to_string(),
                JsonValue::String("condition".to_string()),
            );

            Ok(CompiledCondition {
                expression: expression.to_string(),
                compiled_data: None, // Lua doesn't expose bytecode directly
                metadata,
            })
        }
    }

    fn evaluate_condition(
        &self,
        expression: &str,
        _compiled: Option<&CompiledCondition>,
        _context: &dyn DebugContext,
    ) -> Result<bool, Box<dyn Error>> {
        // This method can't be implemented without a Lua instance
        // It should be called through evaluate_condition_with_lua
        Err(format!("Use evaluate_condition_with_lua for expression: {expression}").into())
    }
}

/// Convert JSON value to Lua value
fn json_to_lua_value<'lua>(
    lua: &'lua Lua,
    json: &JsonValue,
) -> Result<Value<'lua>, Box<dyn Error>> {
    match json {
        JsonValue::Null => Ok(Value::Nil),
        JsonValue::Bool(b) => Ok(Value::Boolean(*b)),
        JsonValue::Number(n) => Ok(n
            .as_i64()
            .map(Value::Integer)
            .or_else(|| n.as_f64().map(Value::Number))
            .unwrap_or(Value::Nil)),
        JsonValue::String(s) => Ok(Value::String(lua.create_string(s)?)),
        JsonValue::Array(arr) => {
            let table = lua.create_table()?;
            for (i, v) in arr.iter().enumerate() {
                table.set(i + 1, json_to_lua_value(lua, v)?)?;
            }
            Ok(Value::Table(table))
        }
        JsonValue::Object(obj) => {
            let table = lua.create_table()?;
            for (k, v) in obj {
                table.set(k.as_str(), json_to_lua_value(lua, v)?)?;
            }
            Ok(Value::Table(table))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    struct MockDebugContext {
        variables: HashMap<String, JsonValue>,
        location: Option<(String, u32)>,
    }

    impl MockDebugContext {
        fn new() -> Self {
            Self {
                variables: HashMap::new(),
                location: None,
            }
        }

        fn with_variable(mut self, name: &str, value: JsonValue) -> Self {
            self.variables.insert(name.to_string(), value);
            self
        }

        fn with_location(mut self, source: &str, line: u32) -> Self {
            self.location = Some((source.to_string(), line));
            self
        }
    }

    impl DebugContext for MockDebugContext {
        fn get_variables(&self) -> HashMap<String, JsonValue> {
            self.variables.clone()
        }

        fn get_variable(&self, name: &str) -> Option<JsonValue> {
            self.variables.get(name).cloned()
        }

        fn get_location(&self) -> Option<(String, u32)> {
            self.location.clone()
        }
    }

    #[test]
    fn test_condition_compilation() {
        let evaluator = LuaConditionEvaluator::new();

        // Test valid condition
        let result = evaluator.compile_condition("x > 10");
        assert!(result.is_ok());

        let compiled = result.unwrap();
        assert_eq!(compiled.expression, "x > 10");
        assert_eq!(
            compiled.metadata.get("language"),
            Some(&JsonValue::String("lua".to_string()))
        );

        // Test empty condition
        let result = evaluator.compile_condition("");
        assert!(result.is_err());
    }

    #[test]
    fn test_condition_evaluation() {
        let lua = Lua::new();
        let evaluator = LuaConditionEvaluator::new();

        // Create context with variables
        let context = MockDebugContext::new()
            .with_variable("x", JsonValue::from(15))
            .with_variable("y", JsonValue::from(5));

        // Test simple condition
        let result = evaluator.evaluate_condition_with_lua("x > 10", None, &context, &lua);
        assert!(result.is_ok());
        assert!(result.unwrap());

        // Test false condition
        let result = evaluator.evaluate_condition_with_lua("x < 10", None, &context, &lua);
        assert!(result.is_ok());
        assert!(!result.unwrap());

        // Test complex condition
        let result =
            evaluator.evaluate_condition_with_lua("x > y and x < 20", None, &context, &lua);
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_location_context() {
        let lua = Lua::new();
        let evaluator = LuaConditionEvaluator::new();

        // Create context with location
        let context = MockDebugContext::new().with_location("test.lua", 42);

        // Test condition using location variables
        let result = evaluator.evaluate_condition_with_lua("__line == 42", None, &context, &lua);
        assert!(result.is_ok());
        assert!(result.unwrap());

        let result =
            evaluator.evaluate_condition_with_lua("__source == 'test.lua'", None, &context, &lua);
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_json_to_lua_conversion() {
        let lua = Lua::new();

        // Test various JSON types
        let test_cases = [
            (JsonValue::Null, Value::Nil),
            (JsonValue::Bool(true), Value::Boolean(true)),
            (JsonValue::from(42), Value::Integer(42)),
            (
                JsonValue::from(std::f64::consts::PI),
                Value::Number(std::f64::consts::PI),
            ),
        ];

        for (json_val, expected_type) in test_cases {
            let lua_val = json_to_lua_value(&lua, &json_val).unwrap();
            assert_eq!(
                std::mem::discriminant(&lua_val),
                std::mem::discriminant(&expected_type)
            );
        }
    }

    #[test]
    fn test_lua_value_to_bool() {
        // Test boolean conversion
        assert!(!LuaConditionEvaluator::lua_value_to_bool(&Value::Nil));
        assert!(LuaConditionEvaluator::lua_value_to_bool(&Value::Boolean(
            true
        )));
        assert!(!LuaConditionEvaluator::lua_value_to_bool(&Value::Boolean(
            false
        )));
        assert!(LuaConditionEvaluator::lua_value_to_bool(&Value::Integer(1)));
        assert!(!LuaConditionEvaluator::lua_value_to_bool(&Value::Integer(
            0
        )));
        assert!(LuaConditionEvaluator::lua_value_to_bool(&Value::Number(
            1.0
        )));
        assert!(!LuaConditionEvaluator::lua_value_to_bool(&Value::Number(
            0.0
        )));
    }
}
