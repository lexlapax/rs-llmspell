//! Lua-specific Diagnostics global implementation (exposed as Console)
//!
//! Provides comprehensive diagnostics capabilities for Lua scripts including
//! logging, performance profiling, and diagnostic utilities.

use crate::diagnostics_bridge::DiagnosticsBridge;
use crate::globals::GlobalContext;
use crate::lua::output::{
    capture_stack_trace, dump_labeled_value, dump_value, DumpOptions, StackTraceOptions,
};
use llmspell_utils::debug::FilterPattern;
use mlua::{Lua, UserData, UserDataFields, UserDataMethods, Value};
use std::sync::Arc;

/// Timer handle for Lua
struct LuaTimer {
    bridge: Arc<DiagnosticsBridge>,
    id: String,
}

impl UserData for LuaTimer {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        // Stop the timer and return duration in milliseconds
        methods.add_method_mut("stop", |_, this, ()| Ok(this.bridge.stop_timer(&this.id)));

        // Record a lap
        methods.add_method("lap", |_, this, name: String| {
            Ok(this.bridge.lap_timer(&this.id, &name))
        });

        // Get elapsed time without stopping
        methods.add_method("elapsed", |_, this, ()| {
            Ok(this.bridge.elapsed_timer(&this.id))
        });
    }

    fn add_fields<'lua, F: UserDataFields<'lua, Self>>(fields: &mut F) {
        // Expose timer ID for event recording
        fields.add_field_method_get("id", |_, this| Ok(this.id.clone()));
    }
}

/// Inject Console global into Lua environment (diagnostics)
///
/// # Errors
///
/// Returns an error if:
/// - Lua table creation fails
/// - Function binding fails
#[allow(clippy::too_many_lines)]
pub fn inject_diagnostics_global(
    lua: &Lua,
    _context: &GlobalContext,
    bridge: &Arc<DiagnosticsBridge>,
) -> mlua::Result<()> {
    let console_table = lua.create_table()?;

    // Console.log(message) or Console.log(level, message, [module])
    let bridge_clone = bridge.clone();
    let log_fn = lua.create_function(
        move |_, (level, message, module): (String, String, Option<String>)| {
            bridge_clone.log(&level, &message, module.as_deref());
            Ok(())
        },
    )?;
    console_table.set("log", log_fn)?;

    // Debug.trace(message, [module])
    let bridge_clone = bridge.clone();
    let trace_fn = lua.create_function(move |_, (message, module): (String, Option<String>)| {
        bridge_clone.log("trace", &message, module.as_deref());
        Ok(())
    })?;
    console_table.set("trace", trace_fn)?;

    // Debug.debug(message, [module])
    let bridge_clone = bridge.clone();
    let debug_fn = lua.create_function(move |_, (message, module): (String, Option<String>)| {
        bridge_clone.log("debug", &message, module.as_deref());
        Ok(())
    })?;
    console_table.set("debug", debug_fn)?;

    // Debug.info(message, [module])
    let bridge_clone = bridge.clone();
    let info_fn = lua.create_function(move |_, (message, module): (String, Option<String>)| {
        bridge_clone.log("info", &message, module.as_deref());
        Ok(())
    })?;
    console_table.set("info", info_fn)?;

    // Debug.warn(message, [module])
    let bridge_clone = bridge.clone();
    let warn_fn = lua.create_function(move |_, (message, module): (String, Option<String>)| {
        bridge_clone.log("warn", &message, module.as_deref());
        Ok(())
    })?;
    console_table.set("warn", warn_fn)?;

    // Debug.error(message, [module])
    let bridge_clone = bridge.clone();
    let error_fn = lua.create_function(move |_, (message, module): (String, Option<String>)| {
        bridge_clone.log("error", &message, module.as_deref());
        Ok(())
    })?;
    console_table.set("error", error_fn)?;

    // Debug.logWithData(level, message, data, [module])
    let bridge_clone = bridge.clone();
    let log_with_data_fn = lua.create_function(
        move |_lua, (level, message, data, module): (String, String, Value, Option<String>)| {
            // Convert Lua value to JSON
            let json_value = lua_value_to_json(&data)?;
            bridge_clone.log_with_metadata(&level, &message, module.as_deref(), json_value);
            Ok(())
        },
    )?;
    console_table.set("logWithData", log_with_data_fn)?;

    // Debug.timer(name) - returns a timer object
    let bridge_clone = bridge.clone();
    let timer_fn = lua.create_function(move |_lua, name: String| {
        let timer_id = bridge_clone.start_timer(&name);

        let timer = LuaTimer {
            bridge: bridge_clone.clone(),
            id: timer_id,
        };

        Ok(timer)
    })?;
    console_table.set("timer", timer_fn)?;

    // Debug.setLevel(level)
    let bridge_clone = bridge.clone();
    let set_level_fn =
        lua.create_function(move |_, level: String| Ok(bridge_clone.set_level(&level)))?;
    console_table.set("setLevel", set_level_fn)?;

    // Debug.getLevel()
    let bridge_clone = bridge.clone();
    let get_level_fn = lua.create_function(move |_, ()| Ok(bridge_clone.get_level()))?;
    console_table.set("getLevel", get_level_fn)?;

    // Debug.setEnabled(enabled)
    let bridge_clone = bridge.clone();
    let set_enabled_fn = lua.create_function(move |_, enabled: bool| {
        bridge_clone.set_enabled(enabled);
        Ok(())
    })?;
    console_table.set("setEnabled", set_enabled_fn)?;

    // Debug.isEnabled()
    let bridge_clone = bridge.clone();
    let is_enabled_fn = lua.create_function(move |_, ()| Ok(bridge_clone.is_enabled()))?;
    console_table.set("isEnabled", is_enabled_fn)?;

    // Debug.addModuleFilter(pattern, enabled)
    let bridge_clone = bridge.clone();
    let add_filter_fn = lua.create_function(move |_, (pattern, enabled): (String, bool)| {
        bridge_clone.add_module_filter(&pattern, enabled);
        Ok(())
    })?;
    console_table.set("addModuleFilter", add_filter_fn)?;

    // Debug.clearModuleFilters()
    let bridge_clone = bridge.clone();
    let clear_filters_fn = lua.create_function(move |_, ()| {
        bridge_clone.clear_module_filters();
        Ok(())
    })?;
    console_table.set("clearModuleFilters", clear_filters_fn)?;

    // Debug.removeModuleFilter(pattern)
    let bridge_clone = bridge.clone();
    let remove_filter_fn = lua.create_function(move |_, pattern: String| {
        Ok(bridge_clone.remove_module_filter(&pattern))
    })?;
    console_table.set("removeModuleFilter", remove_filter_fn)?;

    // Debug.setDefaultFilterEnabled(enabled)
    let bridge_clone = bridge.clone();
    let set_default_fn = lua.create_function(move |_, enabled: bool| {
        bridge_clone.set_default_filter_enabled(enabled);
        Ok(())
    })?;
    console_table.set("setDefaultFilterEnabled", set_default_fn)?;

    // Debug.addAdvancedFilter(pattern, pattern_type, enabled)
    let bridge_clone = bridge.clone();
    let add_advanced_filter_fn = lua.create_function(
        move |_, (pattern, pattern_type, enabled): (String, String, bool)| {
            Ok(bridge_clone.add_filter_rule(&pattern, &pattern_type, enabled))
        },
    )?;
    console_table.set("addAdvancedFilter", add_advanced_filter_fn)?;

    // Debug.getFilterSummary()
    let bridge_clone = bridge.clone();
    let filter_summary_fn = lua.create_function(move |lua, ()| {
        let summary = bridge_clone.get_filter_summary();

        let table = lua.create_table()?;
        table.set("default_enabled", summary.default_enabled)?;
        table.set("total_rules", summary.total_rules)?;

        let rules_table = lua.create_table()?;
        for (i, rule) in summary.rules.iter().enumerate() {
            let rule_entry = lua.create_table()?;

            let (pattern_str, pattern_type) = match &rule.pattern {
                FilterPattern::Exact(p) => (p.clone(), "exact"),
                FilterPattern::Wildcard(p) => (p.clone(), "wildcard"),
                FilterPattern::Regex(p) => (p.clone(), "regex"),
                FilterPattern::Hierarchical(p) => (p.clone(), "hierarchical"),
            };

            rule_entry.set("pattern", pattern_str)?;
            rule_entry.set("pattern_type", pattern_type)?;
            rule_entry.set("enabled", rule.enabled)?;

            if let Some(desc) = &rule.description {
                rule_entry.set("description", desc.clone())?;
            }

            rules_table.set(i + 1, rule_entry)?;
        }
        table.set("rules", rules_table)?;

        Ok(table)
    })?;
    console_table.set("getFilterSummary", filter_summary_fn)?;

    // Debug.getCapturedEntries([limit])
    let bridge_clone = bridge.clone();
    let get_entries_fn = lua.create_function(move |lua, limit: Option<usize>| {
        let entries = bridge_clone.get_captured_entries(limit);

        let table = lua.create_table()?;
        for (i, entry) in entries.into_iter().enumerate() {
            let entry_table = lua.create_table()?;
            entry_table.set("timestamp", entry.timestamp)?;
            entry_table.set("level", entry.level)?;
            entry_table.set("message", entry.message)?;
            if let Some(module) = entry.module {
                entry_table.set("module", module)?;
            }
            if let Some(metadata) = entry.metadata {
                let meta_value = json_to_lua_value(lua, &metadata)?;
                entry_table.set("metadata", meta_value)?;
            }
            table.set(i + 1, entry_table)?;
        }
        Ok(table)
    })?;
    console_table.set("getCapturedEntries", get_entries_fn)?;

    // Debug.clearCaptured()
    let bridge_clone = bridge.clone();
    let clear_captured_fn = lua.create_function(move |_, ()| {
        bridge_clone.clear_captured();
        Ok(())
    })?;
    console_table.set("clearCaptured", clear_captured_fn)?;

    // Debug.performanceReport()
    let bridge_clone = bridge.clone();
    let perf_report_fn =
        lua.create_function(move |_, ()| Ok(bridge_clone.generate_performance_report()))?;
    console_table.set("performanceReport", perf_report_fn)?;

    // Debug.dump(value, [label]) - Enhanced Lua-specific dumping
    let dump_fn = lua.create_function(move |_lua, (value, label): (Value, Option<String>)| {
        let options = DumpOptions::default();
        label.map_or_else(
            || Ok(dump_value(&value, &options)),
            |label_str| Ok(dump_labeled_value(&value, &label_str, &options)),
        )
    })?;
    console_table.set("dump", dump_fn)?;

    // Debug.dumpCompact(value, [label]) - Compact one-liner format
    let dump_compact_fn =
        lua.create_function(move |_lua, (value, label): (Value, Option<String>)| {
            let options = DumpOptions::compact();
            label.map_or_else(
                || Ok(dump_value(&value, &options)),
                |label_str| Ok(dump_labeled_value(&value, &label_str, &options)),
            )
        })?;
    console_table.set("dumpCompact", dump_compact_fn)?;

    // Debug.dumpVerbose(value, [label]) - Detailed inspection format
    let dump_verbose_fn =
        lua.create_function(move |_lua, (value, label): (Value, Option<String>)| {
            let options = DumpOptions::verbose();
            label.map_or_else(
                || Ok(dump_value(&value, &options)),
                |label_str| Ok(dump_labeled_value(&value, &label_str, &options)),
            )
        })?;
    console_table.set("dumpVerbose", dump_verbose_fn)?;

    // Debug.dumpWithOptions(value, options, [label]) - Fully configurable
    let dump_with_options_fn = lua.create_function(
        move |_lua, (value, options_table, label): (Value, mlua::Table, Option<String>)| {
            let options = DumpOptions {
                max_depth: options_table.get("max_depth").unwrap_or(10),
                indent_size: options_table.get("indent_size").unwrap_or(2),
                max_string_length: options_table.get("max_string_length").unwrap_or(200),
                max_array_elements: options_table.get("max_array_elements").unwrap_or(50),
                max_table_pairs: options_table.get("max_table_pairs").unwrap_or(50),
                show_types: options_table.get("show_types").unwrap_or(true),
                show_addresses: options_table.get("show_addresses").unwrap_or(false),
                compact_mode: options_table.get("compact_mode").unwrap_or(false),
            };

            label.map_or_else(
                || Ok(dump_value(&value, &options)),
                |label_str| Ok(dump_labeled_value(&value, &label_str, &options)),
            )
        },
    )?;
    console_table.set("dumpWithOptions", dump_with_options_fn)?;

    // Debug.memoryStats()
    let bridge_clone = bridge.clone();
    let mem_stats_fn = lua.create_function(move |lua, ()| {
        let stats = bridge_clone.get_memory_stats();

        let table = lua.create_table()?;
        table.set("used_bytes", stats.used_bytes)?;
        table.set("allocated_bytes", stats.allocated_bytes)?;
        table.set("resident_bytes", stats.resident_bytes)?;
        table.set("collections", stats.collections)?;
        Ok(table)
    })?;
    console_table.set("memoryStats", mem_stats_fn)?;

    // Debug.jsonReport()
    let bridge_clone = bridge.clone();
    let json_report_fn =
        lua.create_function(move |_, ()| match bridge_clone.generate_json_report() {
            Ok(json) => Ok(json),
            Err(e) => Err(mlua::Error::RuntimeError(e)),
        })?;
    console_table.set("jsonReport", json_report_fn)?;

    // Debug.flameGraph()
    let bridge_clone = bridge.clone();
    let flame_graph_fn =
        lua.create_function(move |_, ()| Ok(bridge_clone.generate_flame_graph()))?;
    console_table.set("flameGraph", flame_graph_fn)?;

    // Debug.memorySnapshot()
    let bridge_clone = bridge.clone();
    let memory_snapshot_fn = lua.create_function(move |lua, ()| {
        let snapshot = bridge_clone.get_memory_snapshot();

        let table = lua.create_table()?;
        table.set("timestamp_secs", snapshot.timestamp_secs)?;
        table.set("active_trackers", snapshot.active_trackers)?;

        if let Some(delta) = snapshot.total_memory_delta_bytes {
            table.set("total_memory_delta_bytes", delta)?;
        }

        let usage_table = lua.create_table()?;
        for (name, info) in snapshot.tracker_memory_usage {
            let info_table = lua.create_table()?;
            if let Some(start) = info.start_bytes {
                info_table.set("start_bytes", start)?;
            }
            if let Some(end) = info.end_bytes {
                info_table.set("end_bytes", end)?;
            }
            if let Some(delta) = info.delta_bytes {
                info_table.set("delta_bytes", delta)?;
            }
            usage_table.set(name, info_table)?;
        }
        table.set("tracker_memory_usage", usage_table)?;

        Ok(table)
    })?;
    console_table.set("memorySnapshot", memory_snapshot_fn)?;

    // Debug.recordEvent(timer_id, event_name, [metadata])
    let bridge_clone = bridge.clone();
    let record_event_fn = lua.create_function(
        move |_lua, (timer_id, event_name, metadata): (String, String, Option<Value>)| {
            let json_metadata = if let Some(meta) = metadata {
                Some(lua_value_to_json(&meta)?)
            } else {
                None
            };

            Ok(bridge_clone.record_event(&timer_id, &event_name, json_metadata))
        },
    )?;
    console_table.set("recordEvent", record_event_fn)?;

    // Debug.stackTrace([options])
    let bridge_clone = bridge.clone();
    let stack_trace_fn = lua.create_function(move |lua, options: Option<mlua::Table>| {
        let trace_options = options.map_or_else(
            || bridge_clone.stack_trace_options_for_level(&bridge_clone.get_level()),
            |opts| StackTraceOptions {
                max_depth: opts.get("max_depth").unwrap_or(50),
                capture_locals: opts.get("capture_locals").unwrap_or(false),
                capture_upvalues: opts.get("capture_upvalues").unwrap_or(false),
                include_source: opts.get("include_source").unwrap_or(true),
            },
        );

        let trace = capture_stack_trace(lua, &trace_options);
        Ok(trace.format())
    })?;
    console_table.set("stackTrace", stack_trace_fn)?;

    // Debug.stackTraceJson([options])
    let bridge_clone = bridge.clone();
    let stack_trace_json_fn = lua.create_function(move |lua, options: Option<mlua::Table>| {
        let trace_options = options.map_or_else(
            || bridge_clone.stack_trace_options_for_level(&bridge_clone.get_level()),
            |opts| StackTraceOptions {
                max_depth: opts.get("max_depth").unwrap_or(50),
                capture_locals: opts.get("capture_locals").unwrap_or(false),
                capture_upvalues: opts.get("capture_upvalues").unwrap_or(false),
                include_source: opts.get("include_source").unwrap_or(true),
            },
        );

        let trace = capture_stack_trace(lua, &trace_options);
        match trace.to_json() {
            Ok(json) => Ok(json),
            Err(e) => Err(mlua::Error::RuntimeError(format!(
                "JSON serialization failed: {e}"
            ))),
        }
    })?;
    console_table.set("stackTraceJson", stack_trace_json_fn)?;

    // Set the Console global (diagnostics)
    lua.globals().set("Console", console_table)?;

    Ok(())
}

/// Convert Lua value to JSON
fn lua_value_to_json(value: &Value) -> mlua::Result<serde_json::Value> {
    match value {
        Value::Boolean(b) => Ok(serde_json::Value::Bool(*b)),
        Value::Integer(i) => Ok(serde_json::Value::Number((*i).into())),
        Value::Number(n) => serde_json::Number::from_f64(*n)
            .map_or(Ok(serde_json::Value::Null), |num| {
                Ok(serde_json::Value::Number(num))
            }),
        Value::String(s) => Ok(serde_json::Value::String(s.to_str()?.to_string())),
        Value::Table(t) => {
            // Check if it's an array-like table
            let mut is_array = true;
            let mut max_index = 0;

            for pair in t.clone().pairs::<Value, Value>() {
                let (k, _) = pair?;
                match k {
                    Value::Integer(i) if i > 0 => {
                        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                        {
                            max_index = max_index.max(i as usize);
                        }
                    }
                    _ => {
                        is_array = false;
                        break;
                    }
                }
            }

            if is_array && max_index > 0 {
                let mut arr = Vec::new();
                for i in 1..=max_index {
                    let val = t.get::<_, Value>(i)?;
                    arr.push(lua_value_to_json(&val)?);
                }
                Ok(serde_json::Value::Array(arr))
            } else {
                let mut map = serde_json::Map::new();
                for pair in t.clone().pairs::<String, Value>() {
                    let (k, v) = pair?;
                    map.insert(k, lua_value_to_json(&v)?);
                }
                Ok(serde_json::Value::Object(map))
            }
        }
        _ => Ok(serde_json::Value::Null),
    }
}

/// Convert JSON to Lua value
fn json_to_lua_value<'lua>(lua: &'lua Lua, value: &serde_json::Value) -> mlua::Result<Value<'lua>> {
    match value {
        serde_json::Value::Null => Ok(Value::Nil),
        serde_json::Value::Bool(b) => Ok(Value::Boolean(*b)),
        serde_json::Value::Number(n) => n.as_i64().map_or_else(
            || n.as_f64().map_or(Ok(Value::Nil), |f| Ok(Value::Number(f))),
            |i| Ok(Value::Integer(i)),
        ),
        serde_json::Value::String(s) => Ok(Value::String(lua.create_string(s)?)),
        serde_json::Value::Array(arr) => {
            let table = lua.create_table()?;
            for (i, v) in arr.iter().enumerate() {
                table.set(i + 1, json_to_lua_value(lua, v)?)?;
            }
            Ok(Value::Table(table))
        }
        serde_json::Value::Object(map) => {
            let table = lua.create_table()?;
            for (k, v) in map {
                table.set(k.as_str(), json_to_lua_value(lua, v)?)?;
            }
            Ok(Value::Table(table))
        }
    }
}
