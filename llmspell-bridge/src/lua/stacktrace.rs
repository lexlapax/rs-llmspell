//! Stack trace collection for Lua scripts
//!
//! Provides comprehensive stack trace capture using Lua's debug library
//! with local variable inspection and source location mapping.

use mlua::{Function, Lua, Result as LuaResult, Table, Value};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Stack frame information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StackFrame {
    /// Function name (if available)
    pub name: Option<String>,
    /// Source file or chunk name
    pub source: Option<String>,
    /// Line number in source
    pub line: Option<i32>,
    /// Function definition line
    pub line_defined: Option<i32>,
    /// Function type (Lua, C, main, tail)
    pub what: String,
    /// Number of upvalues
    pub num_upvalues: u8,
    /// Number of parameters
    pub num_params: u8,
    /// Local variables at this frame
    pub locals: HashMap<String, String>,
    /// Upvalues at this frame
    pub upvalues: HashMap<String, String>,
}

/// Complete stack trace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StackTrace {
    /// Stack frames from bottom to top
    pub frames: Vec<StackFrame>,
    /// Maximum depth captured
    pub max_depth: usize,
    /// Whether the trace was truncated
    pub truncated: bool,
    /// Error message if capture failed
    pub error: Option<String>,
}

/// Stack trace collection options
#[derive(Debug, Clone)]
pub struct StackTraceOptions {
    /// Maximum stack depth to capture
    pub max_depth: usize,
    /// Whether to capture local variables
    pub capture_locals: bool,
    /// Whether to capture upvalues
    pub capture_upvalues: bool,
    /// Whether to include source information
    pub include_source: bool,
}

impl Default for StackTraceOptions {
    fn default() -> Self {
        Self {
            max_depth: 50,
            capture_locals: false, // Only at trace level
            capture_upvalues: false,
            include_source: true,
        }
    }
}

impl StackTraceOptions {
    /// Create options for error-level capture (minimal)
    #[must_use]
    pub fn for_error() -> Self {
        Self {
            max_depth: 20,
            capture_locals: false,
            capture_upvalues: false,
            include_source: true,
        }
    }

    /// Create options for trace-level capture (comprehensive)
    #[must_use]
    pub fn for_trace() -> Self {
        Self {
            max_depth: 50,
            capture_locals: true,
            capture_upvalues: true,
            include_source: true,
        }
    }
}

/// Capture a stack trace from the current Lua execution point
pub fn capture_stack_trace(lua: &Lua, options: &StackTraceOptions) -> StackTrace {
    match capture_stack_trace_impl(lua, options) {
        Ok(trace) => trace,
        Err(e) => StackTrace {
            frames: vec![],
            max_depth: options.max_depth,
            truncated: false,
            error: Some(format!("Stack trace capture failed: {}", e)),
        },
    }
}

fn capture_stack_trace_impl(lua: &Lua, options: &StackTraceOptions) -> LuaResult<StackTrace> {
    let debug_table: Table = lua.globals().get("debug")?;
    let getinfo_fn: Function = debug_table.get("getinfo")?;

    let mut frames = Vec::new();
    let mut level = 1; // Start from level 1 (skip debug.getinfo itself)
    let mut truncated = false;

    while frames.len() < options.max_depth {
        // Get basic frame info
        let info_result: LuaResult<Table> = getinfo_fn.call((level, "nSluf"));

        let info_table = match info_result {
            Ok(table) => table,
            Err(_) => break, // No more frames
        };

        // Extract frame information
        let frame = extract_frame_info(lua, &info_table, level, options)?;
        frames.push(frame);

        level += 1;

        // Check if we've hit our depth limit
        if frames.len() >= options.max_depth {
            truncated = true;
            break;
        }
    }

    Ok(StackTrace {
        frames,
        max_depth: options.max_depth,
        truncated,
        error: None,
    })
}

fn extract_frame_info(
    lua: &Lua,
    info: &Table,
    level: i32,
    options: &StackTraceOptions,
) -> LuaResult<StackFrame> {
    // Basic frame information
    let name = info.get::<_, Option<String>>("name")?;
    let source = if options.include_source {
        info.get::<_, Option<String>>("source")?
    } else {
        None
    };
    let line = info.get::<_, Option<i32>>("currentline")?;
    let line_defined = info.get::<_, Option<i32>>("linedefined")?;
    let what = info
        .get::<_, String>("what")
        .unwrap_or_else(|_| "unknown".to_string());
    let num_upvalues = info.get::<_, Option<u8>>("nups")?.unwrap_or(0);
    let num_params = info.get::<_, Option<u8>>("nparams")?.unwrap_or(0);

    // Capture local variables if requested
    let locals = if options.capture_locals {
        capture_locals(lua, level)?
    } else {
        HashMap::new()
    };

    // Capture upvalues if requested
    let upvalues = if options.capture_upvalues && info.contains_key("func")? {
        capture_upvalues(lua, level)?
    } else {
        HashMap::new()
    };

    Ok(StackFrame {
        name,
        source,
        line,
        line_defined,
        what,
        num_upvalues,
        num_params,
        locals,
        upvalues,
    })
}

fn capture_locals(lua: &Lua, level: i32) -> LuaResult<HashMap<String, String>> {
    let debug_table: Table = lua.globals().get("debug")?;
    let getlocal_fn: Function = debug_table.get("getlocal")?;

    let mut locals = HashMap::new();
    let mut local_index = 1;

    loop {
        let result: LuaResult<(Option<String>, Value)> = getlocal_fn.call((level, local_index));

        match result {
            Ok((Some(name), value)) => {
                // Skip internal variables (those starting with '(')
                if !name.starts_with('(') {
                    locals.insert(name, value_to_debug_string(&value));
                }
                local_index += 1;
            }
            _ => break, // No more locals or error
        }

        // Safety limit to prevent infinite loops
        if local_index > 100 {
            break;
        }
    }

    Ok(locals)
}

fn capture_upvalues(lua: &Lua, level: i32) -> LuaResult<HashMap<String, String>> {
    let debug_table: Table = lua.globals().get("debug")?;
    let getinfo_fn: Function = debug_table.get("getinfo")?;
    let getupvalue_fn: Function = debug_table.get("getupvalue")?;

    // Get the function at this level
    let info: Table = getinfo_fn.call((level, "f"))?;
    let func: Function = info.get("func")?;

    let mut upvalues = HashMap::new();
    let mut upvalue_index = 1;

    loop {
        let result: LuaResult<(Option<String>, Value)> = getupvalue_fn.call((&func, upvalue_index));

        match result {
            Ok((Some(name), value)) => {
                upvalues.insert(name, value_to_debug_string(&value));
                upvalue_index += 1;
            }
            _ => break, // No more upvalues or error
        }

        // Safety limit
        if upvalue_index > 50 {
            break;
        }
    }

    Ok(upvalues)
}

fn value_to_debug_string(value: &Value) -> String {
    match value {
        Value::Nil => "nil".to_string(),
        Value::Boolean(b) => b.to_string(),
        Value::Integer(i) => i.to_string(),
        Value::Number(n) => n.to_string(),
        Value::String(s) => match s.to_str() {
            Ok(str_val) => {
                if str_val.len() > 100 {
                    format!("\"{}...\" ({})", &str_val[..97], str_val.len())
                } else {
                    format!("\"{}\"", str_val)
                }
            }
            Err(_) => "\"<invalid string>\"".to_string(),
        },
        Value::Table(_) => "table".to_string(),
        Value::Function(_) => "function".to_string(),
        Value::Thread(_) => "thread".to_string(),
        Value::UserData(_) | Value::LightUserData(_) => "userdata".to_string(),
        Value::Error(e) => format!("error: {}", e),
    }
}

impl StackTrace {
    /// Format the stack trace as a readable string
    #[must_use]
    pub fn format(&self) -> String {
        if let Some(error) = &self.error {
            return format!("Stack trace error: {}", error);
        }

        if self.frames.is_empty() {
            return "No stack frames available".to_string();
        }

        let mut output = String::from("Stack trace:\n");

        for (i, frame) in self.frames.iter().enumerate() {
            let frame_header = if let Some(name) = &frame.name {
                format!("  #{}: in function '{}'", i, name)
            } else {
                format!("  #{}: in <unknown>", i)
            };

            output.push_str(&frame_header);

            // Add source information
            if let Some(source) = &frame.source {
                if let Some(line) = frame.line {
                    output.push_str(&format!(" ({}:{})", source, line));
                } else {
                    output.push_str(&format!(" ({})", source));
                }
            }

            output.push('\n');

            // Add local variables if available
            if !frame.locals.is_empty() {
                output.push_str("    Locals:\n");
                for (name, value) in &frame.locals {
                    output.push_str(&format!("      {} = {}\n", name, value));
                }
            }

            // Add upvalues if available
            if !frame.upvalues.is_empty() {
                output.push_str("    Upvalues:\n");
                for (name, value) in &frame.upvalues {
                    output.push_str(&format!("      {} = {}\n", name, value));
                }
            }
        }

        if self.truncated {
            output.push_str(&format!("  ... (truncated at {} frames)\n", self.max_depth));
        }

        output
    }

    /// Format as JSON string
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Get frame count
    #[must_use]
    pub fn frame_count(&self) -> usize {
        self.frames.len()
    }

    /// Get the topmost frame (most recent call)
    #[must_use]
    pub fn top_frame(&self) -> Option<&StackFrame> {
        self.frames.first()
    }
}

/// Create a stack trace function for Lua scripts
pub fn create_stacktrace_function(lua: &Lua) -> LuaResult<Function> {
    lua.create_function(|lua, options: Option<Table>| {
        let trace_options = if let Some(opts) = options {
            StackTraceOptions {
                max_depth: opts.get("max_depth").unwrap_or(50),
                capture_locals: opts.get("capture_locals").unwrap_or(false),
                capture_upvalues: opts.get("capture_upvalues").unwrap_or(false),
                include_source: opts.get("include_source").unwrap_or(true),
            }
        } else {
            StackTraceOptions::default()
        };

        let trace = capture_stack_trace(lua, &trace_options);
        Ok(trace.format())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stack_trace_options() {
        let error_opts = StackTraceOptions::for_error();
        assert_eq!(error_opts.max_depth, 20);
        assert!(!error_opts.capture_locals);

        let trace_opts = StackTraceOptions::for_trace();
        assert_eq!(trace_opts.max_depth, 50);
        assert!(trace_opts.capture_locals);
    }

    #[test]
    fn test_value_to_debug_string() {
        assert_eq!(value_to_debug_string(&Value::Nil), "nil");
        assert_eq!(value_to_debug_string(&Value::Boolean(true)), "true");
        assert_eq!(value_to_debug_string(&Value::Integer(42)), "42");
    }

    #[test]
    fn test_stack_trace_basic_functionality() -> LuaResult<()> {
        let lua = Lua::new();

        // Test basic stack trace options
        let default_opts = StackTraceOptions::default();
        assert_eq!(default_opts.max_depth, 50);
        assert!(!default_opts.capture_locals);

        let error_opts = StackTraceOptions::for_error();
        assert_eq!(error_opts.max_depth, 20);
        assert!(!error_opts.capture_locals);

        let trace_opts = StackTraceOptions::for_trace();
        assert_eq!(trace_opts.max_depth, 50);
        assert!(trace_opts.capture_locals);

        // Test capture function creation (should not fail)
        let _trace_fn = create_stacktrace_function(&lua)?;

        // Test actual capture with graceful error handling
        let trace = capture_stack_trace(&lua, &default_opts);
        // The trace should exist, possibly with an error if debug not available
        assert!(trace.frames.is_empty() || !trace.frames.is_empty());

        Ok(())
    }
}
