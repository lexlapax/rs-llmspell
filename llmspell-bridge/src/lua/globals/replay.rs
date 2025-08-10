// ABOUTME: Lua bindings for hook replay functionality including scheduling and comparisons
// ABOUTME: Provides scripting access to advanced replay capabilities for debugging

use llmspell_core::LLMSpellError;
use llmspell_hooks::replay::{
    HookResultComparator, ParameterModification, ReplayConfig, ReplayMode, ReplaySchedule,
};
use mlua::{
    Error as LuaError, FromLua, Lua, Result as LuaResult, Table, UserData, UserDataMethods, Value,
};
use serde_json;
use std::time::Duration;

/// Lua wrapper for `ReplayMode`
#[derive(Debug, Clone)]
pub struct LuaReplayMode(pub ReplayMode);

impl<'lua> FromLua<'lua> for LuaReplayMode {
    fn from_lua(value: Value<'lua>, _lua: &'lua Lua) -> LuaResult<Self> {
        match value {
            Value::UserData(ud) => Ok(ud.borrow::<Self>()?.clone()),
            _ => Err(LuaError::FromLuaConversionError {
                from: value.type_name(),
                to: "LuaReplayMode",
                message: Some("expected LuaReplayMode userdata".to_string()),
            }),
        }
    }
}

impl UserData for LuaReplayMode {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("name", |_, this, ()| match this.0 {
            ReplayMode::Exact => Ok("exact"),
            ReplayMode::Modified => Ok("modified"),
            ReplayMode::Simulate => Ok("simulate"),
            ReplayMode::Debug => Ok("debug"),
        });
    }
}

/// Lua wrapper for `ParameterModification`
#[derive(Debug, Clone)]
pub struct LuaParameterModification(pub ParameterModification);

impl UserData for LuaParameterModification {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("get_path", |_, this, ()| Ok(this.0.path.clone()));

        methods.add_method("get_value", |lua, this, ()| {
            let json_str = serde_json::to_string(&this.0.value).map_err(LuaError::external)?;
            lua.load(&json_str).eval::<Value>()
        });

        methods.add_method("is_enabled", |_, this, ()| Ok(this.0.enabled));

        methods.add_method_mut("set_enabled", |_, this, enabled: bool| {
            this.0.enabled = enabled;
            Ok(())
        });
    }
}

/// Lua wrapper for `ReplayConfig`
#[derive(Debug, Clone)]
pub struct LuaReplayConfig(pub ReplayConfig);

impl UserData for LuaReplayConfig {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("get_mode", |_, this, ()| Ok(LuaReplayMode(this.0.mode)));

        methods.add_method("should_compare_results", |_, this, ()| {
            Ok(this.0.compare_results)
        });

        methods.add_method("get_timeout_seconds", |_, this, ()| {
            Ok(this.0.timeout.as_secs())
        });

        methods.add_method("should_stop_on_error", |_, this, ()| {
            Ok(this.0.stop_on_error)
        });

        methods.add_method("get_modifications", |_, this, ()| {
            Ok(this
                .0
                .modifications
                .clone()
                .into_iter()
                .map(LuaParameterModification)
                .collect::<Vec<_>>())
        });

        methods.add_method_mut(
            "add_modification",
            |lua, this, (path, value, enabled): (String, Value, Option<bool>)| {
                let json_value = lua_value_to_json(lua, value)?;
                this.0.modifications.push(ParameterModification {
                    path,
                    value: json_value,
                    enabled: enabled.unwrap_or(true),
                });
                Ok(())
            },
        );
    }
}

/// Lua wrapper for `ReplaySchedule`
#[derive(Debug, Clone)]
pub struct LuaReplaySchedule(pub ReplaySchedule);

impl UserData for LuaReplaySchedule {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("type_name", |_, this, ()| match &this.0 {
            ReplaySchedule::Once { .. } => Ok("once"),
            ReplaySchedule::At { .. } => Ok("at"),
            ReplaySchedule::Interval { .. } => Ok("interval"),
            ReplaySchedule::Cron { .. } => Ok("cron"),
        });
    }
}

/// Convert Lua value to JSON
#[allow(clippy::only_used_in_recursion)]
fn lua_value_to_json(lua: &Lua, value: Value) -> LuaResult<serde_json::Value> {
    match value {
        Value::Nil => Ok(serde_json::Value::Null),
        Value::Boolean(b) => Ok(serde_json::Value::Bool(b)),
        Value::Integer(i) => Ok(serde_json::Value::Number(i.into())),
        Value::Number(n) => serde_json::Number::from_f64(n)
            .map(serde_json::Value::Number)
            .ok_or_else(|| LuaError::external("Invalid number")),
        Value::String(s) => Ok(serde_json::Value::String(s.to_str()?.to_string())),
        Value::Table(t) => {
            let mut is_array = true;
            let mut expected_index = 1;

            for pair in t.clone().pairs::<Value, Value>() {
                let (k, _) = pair?;
                if let Value::Integer(i) = k {
                    if i != expected_index {
                        is_array = false;
                        break;
                    }
                    expected_index += 1;
                } else {
                    is_array = false;
                    break;
                }
            }

            if is_array {
                let mut arr = Vec::new();
                for pair in t.pairs::<i64, Value>() {
                    let (_, v) = pair?;
                    arr.push(lua_value_to_json(lua, v)?);
                }
                Ok(serde_json::Value::Array(arr))
            } else {
                let mut map = serde_json::Map::new();
                for pair in t.pairs::<String, Value>() {
                    let (k, v) = pair?;
                    map.insert(k, lua_value_to_json(lua, v)?);
                }
                Ok(serde_json::Value::Object(map))
            }
        }
        _ => Err(LuaError::external("Unsupported Lua value type")),
    }
}

/// Inject replay global into Lua environment
///
/// # Errors
///
/// Returns an error if:
/// - Replay API creation fails
/// - Global injection fails
pub fn inject_replay_global(lua: &Lua) -> Result<(), LLMSpellError> {
    let replay = create_replay_api(lua).map_err(|e| LLMSpellError::Internal {
        message: e.to_string(),
        source: None,
    })?;

    lua.globals()
        .set("Replay", replay)
        .map_err(|e| LLMSpellError::Internal {
            message: e.to_string(),
            source: None,
        })?;

    Ok(())
}

/// Create the replay API table
///
/// # Errors
///
/// Returns an error if:
/// - Table creation fails
/// - Method injection fails
pub fn create_replay_api(lua: &Lua) -> LuaResult<Table> {
    let replay = lua.create_table()?;

    // ReplayMode constructors
    let modes = lua.create_table()?;
    modes.set("exact", LuaReplayMode(ReplayMode::Exact))?;
    modes.set("modified", LuaReplayMode(ReplayMode::Modified))?;
    modes.set("simulate", LuaReplayMode(ReplayMode::Simulate))?;
    modes.set("debug", LuaReplayMode(ReplayMode::Debug))?;
    replay.set("modes", modes)?;

    // Create a default config
    replay.set(
        "create_config",
        lua.create_function(|_, mode: Option<LuaReplayMode>| {
            let config = ReplayConfig {
                mode: mode.map_or(ReplayMode::Exact, |m| m.0),
                ..Default::default()
            };
            Ok(LuaReplayConfig(config))
        })?,
    )?;

    // Create parameter modification
    replay.set(
        "create_modification",
        lua.create_function(
            |lua, (path, value, enabled): (String, Value, Option<bool>)| {
                let json_value = lua_value_to_json(lua, value)?;
                Ok(LuaParameterModification(ParameterModification {
                    path,
                    value: json_value,
                    enabled: enabled.unwrap_or(true),
                }))
            },
        )?,
    )?;

    // Create schedule types
    let schedules = lua.create_table()?;

    schedules.set(
        "once",
        lua.create_function(|_, delay_seconds: f64| {
            Ok(LuaReplaySchedule(ReplaySchedule::Once {
                delay: Duration::from_secs_f64(delay_seconds),
            }))
        })?,
    )?;

    schedules.set(
        "interval",
        lua.create_function(
            |_, (initial_delay, interval, max_executions): (f64, f64, Option<usize>)| {
                Ok(LuaReplaySchedule(ReplaySchedule::Interval {
                    initial_delay: Duration::from_secs_f64(initial_delay),
                    interval: Duration::from_secs_f64(interval),
                    max_executions,
                }))
            },
        )?,
    )?;

    schedules.set(
        "cron",
        lua.create_function(|_, expression: String| {
            Ok(LuaReplaySchedule(ReplaySchedule::Cron { expression }))
        })?,
    )?;

    replay.set("schedules", schedules)?;

    // Create comparator
    replay.set(
        "create_comparator",
        lua.create_function(|_, ()| Ok(LuaHookResultComparator))?,
    )?;

    Ok(replay)
}

/// Lua wrapper for `HookResultComparator`
#[derive(Debug)]
pub struct LuaHookResultComparator;

impl UserData for LuaHookResultComparator {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method(
            "compare_json",
            |lua, _this, (original, replayed): (Value, Value)| {
                let orig_json = lua_value_to_json(lua, original)?;
                let repl_json = lua_value_to_json(lua, replayed)?;

                // Convert to HookResult for comparison
                let orig_result = llmspell_hooks::HookResult::Modified(orig_json);
                let repl_result = llmspell_hooks::HookResult::Modified(repl_json);

                let comparator = HookResultComparator::new();
                let comparison = comparator.compare(&orig_result, &repl_result);

                // Convert comparison result to Lua table
                let result = lua.create_table()?;
                result.set("identical", comparison.identical)?;
                result.set("similarity_score", comparison.similarity_score)?;
                result.set("summary", comparison.summary)?;

                if !comparison.differences.is_empty() {
                    let diffs = lua.create_table()?;
                    for (i, diff) in comparison.differences.iter().enumerate() {
                        let diff_table = lua.create_table()?;
                        diff_table.set("path", diff.path.clone())?;
                        diff_table.set("description", diff.description.clone())?;
                        diffs.set(i + 1, diff_table)?;
                    }
                    result.set("differences", diffs)?;
                }

                Ok(result)
            },
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_lua_replay_api_creation() {
        let lua = Lua::new();
        let result = create_replay_api(&lua);
        assert!(result.is_ok());
    }
    #[test]
    fn test_lua_value_to_json() {
        let lua = Lua::new();

        // Test nil
        let nil = Value::Nil;
        let json = lua_value_to_json(&lua, nil).unwrap();
        assert_eq!(json, serde_json::Value::Null);

        // Test boolean
        let bool_val = Value::Boolean(true);
        let json = lua_value_to_json(&lua, bool_val).unwrap();
        assert_eq!(json, serde_json::Value::Bool(true));

        // Test number
        let num_val = Value::Number(42.5);
        let json = lua_value_to_json(&lua, num_val).unwrap();
        assert_eq!(json, serde_json::json!(42.5));
    }
}
