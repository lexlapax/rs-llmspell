//! Unified Lua output handling: capture, formatting, and inspection
//!
//! This module combines output capture (for print/io.write) and object dumping
//! (for value inspection) into a single cohesive system for handling all Lua
//! output operations.

use crate::diagnostics_bridge::DiagnosticsBridge;
use mlua::{Lua, MultiValue, Result as LuaResult, Table, Value};
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Write;
use std::sync::Arc;

// ============================================================================
// Output Capture System (formerly output_capture.rs)
// ============================================================================

/// Console output collector for capturing Lua `print()` and `io.write()` output
#[derive(Debug, Clone)]
pub struct ConsoleCapture {
    /// Captured output lines
    lines: Arc<Mutex<Vec<String>>>,
    /// Optional diagnostics bridge for routing to diagnostics system
    diagnostics_bridge: Option<Arc<DiagnosticsBridge>>,
}

impl ConsoleCapture {
    /// Create a new console capture
    #[must_use]
    pub fn new() -> Self {
        Self {
            lines: Arc::new(Mutex::new(Vec::new())),
            diagnostics_bridge: None,
        }
    }

    /// Create a console capture with diagnostics bridge
    #[must_use]
    pub fn with_diagnostics_bridge(bridge: Arc<DiagnosticsBridge>) -> Self {
        Self {
            lines: Arc::new(Mutex::new(Vec::new())),
            diagnostics_bridge: Some(bridge),
        }
    }

    /// Get captured lines
    #[must_use]
    pub fn get_lines(&self) -> Vec<String> {
        self.lines.lock().clone()
    }

    /// Clear captured lines
    pub fn clear(&self) {
        self.lines.lock().clear();
    }

    /// Add a line to the capture
    fn add_line(&self, line: String) {
        // Route to diagnostics system if available
        if let Some(bridge) = &self.diagnostics_bridge {
            bridge.log("info", &line, Some("lua.print"));
        }

        // Also capture locally
        self.lines.lock().push(line);
    }
}

impl Default for ConsoleCapture {
    fn default() -> Self {
        Self::new()
    }
}

/// Override Lua's `print()` function to capture output
///
/// # Errors
///
/// Returns an error if function creation or global setting fails
pub fn override_print(lua: &Lua, capture: Arc<ConsoleCapture>) -> LuaResult<()> {
    // Create a custom print function
    let print_fn = lua.create_function(move |_lua, args: MultiValue| {
        let mut output = Vec::new();

        // Convert all arguments to strings, similar to Lua's default print
        for value in args {
            let str_val = match value {
                Value::Nil => "nil".to_string(),
                Value::Boolean(b) => b.to_string(),
                Value::Integer(i) => i.to_string(),
                Value::Number(n) => {
                    // Format number similarly to Lua
                    #[allow(clippy::float_cmp)]
                    if n.floor() == n && n.is_finite() {
                        format!("{n:.0}")
                    } else {
                        n.to_string()
                    }
                }
                Value::String(s) => s.to_str()?.to_string(),
                Value::Table(_) => "table".to_string(),
                Value::Function(_) => "function".to_string(),
                Value::Thread(_) => "thread".to_string(),
                Value::UserData(_) | Value::LightUserData(_) => "userdata".to_string(),
                Value::Error(e) => format!("error: {e}"),
            };
            output.push(str_val);
        }

        // Join with tabs like Lua's print
        let line = output.join("\t");

        // Capture the output
        capture.add_line(line.clone());

        // Also print to stdout for immediate feedback
        println!("{line}");

        Ok(())
    })?;

    // Override the global print function
    lua.globals().set("print", print_fn)?;

    Ok(())
}

/// Override other console functions (io.write, etc.)
///
/// # Errors
///
/// Returns an error if I/O function override fails
pub fn override_io_functions(lua: &Lua, capture: Arc<ConsoleCapture>) -> LuaResult<()> {
    // Get the io table
    let io_table: mlua::Table = lua.globals().get("io")?;

    // Create custom io.write function
    let io_write = lua.create_function(move |lua, args: MultiValue| {
        let mut output = String::new();

        for value in args {
            let str_val = match value {
                Value::String(s) => s.to_str()?.to_string(),
                Value::Integer(i) => i.to_string(),
                Value::Number(n) => n.to_string(),
                _ => continue, // io.write only accepts strings and numbers
            };
            output.push_str(&str_val);
        }

        // io.write doesn't add newline
        if !output.is_empty() {
            capture.add_line(output.clone());
            print!("{output}");
        }

        // Return io.stdout as Lua does
        // Get io table from globals instead of capturing it
        let io: mlua::Table = lua.globals().get("io")?;
        Ok(Value::Table(io))
    })?;

    io_table.set("write", io_write)?;

    Ok(())
}

/// Install all output capture overrides
///
/// # Errors
///
/// Returns an error if output capture installation fails
pub fn install_output_capture(
    lua: &Lua,
    diagnostics_bridge: Option<Arc<DiagnosticsBridge>>,
) -> LuaResult<Arc<ConsoleCapture>> {
    let capture = diagnostics_bridge.map_or_else(
        || Arc::new(ConsoleCapture::new()),
        |bridge| Arc::new(ConsoleCapture::with_diagnostics_bridge(bridge)),
    );

    override_print(lua, capture.clone())?;
    override_io_functions(lua, capture.clone())?;

    Ok(capture)
}

// ============================================================================
// Object Dumping System (formerly object_dump.rs)
// ============================================================================

/// Object dumping configuration options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DumpOptions {
    /// Maximum depth to traverse nested structures
    pub max_depth: usize,
    /// Number of spaces per indentation level
    pub indent_size: usize,
    /// Maximum string length before truncation
    pub max_string_length: usize,
    /// Maximum array elements to show
    pub max_array_elements: usize,
    /// Maximum table pairs to show
    pub max_table_pairs: usize,
    /// Whether to show types alongside values
    pub show_types: bool,
    /// Whether to show table addresses/references
    pub show_addresses: bool,
    /// Custom formatting for specific types
    pub compact_mode: bool,
}

impl Default for DumpOptions {
    fn default() -> Self {
        Self {
            max_depth: 10,
            indent_size: 2,
            max_string_length: 200,
            max_array_elements: 50,
            max_table_pairs: 50,
            show_types: true,
            show_addresses: false,
            compact_mode: false,
        }
    }
}

impl DumpOptions {
    /// Create compact dumping options for one-liners
    #[must_use]
    pub const fn compact() -> Self {
        Self {
            max_depth: 3,
            indent_size: 0,
            max_string_length: 50,
            max_array_elements: 10,
            max_table_pairs: 10,
            show_types: false,
            show_addresses: false,
            compact_mode: true,
        }
    }

    /// Create verbose dumping options for detailed inspection
    #[must_use]
    pub const fn verbose() -> Self {
        Self {
            max_depth: 20,
            indent_size: 4,
            max_string_length: 1000,
            max_array_elements: 100,
            max_table_pairs: 100,
            show_types: true,
            show_addresses: true,
            compact_mode: false,
        }
    }
}

/// State for recursive dumping
struct DumpState {
    visited: HashMap<*const std::ffi::c_void, usize>,
    next_ref_id: usize,
}

impl DumpState {
    fn new() -> Self {
        Self {
            visited: HashMap::new(),
            next_ref_id: 1,
        }
    }

    fn get_or_create_ref(&mut self, ptr: *const std::ffi::c_void) -> (bool, usize) {
        if let Some(&ref_id) = self.visited.get(&ptr) {
            (true, ref_id)
        } else {
            let ref_id = self.next_ref_id;
            self.next_ref_id += 1;
            self.visited.insert(ptr, ref_id);
            (false, ref_id)
        }
    }
}

/// Dump a Lua value with the given options
///
/// # Errors
///
/// Returns an error if value traversal fails
#[must_use]
pub fn dump_value(value: &Value, options: &DumpOptions) -> String {
    let mut state = DumpState::new();
    dump_value_recursive(value, options, &mut state, 0)
}

/// Dump a labeled value
#[must_use]
pub fn dump_labeled_value(value: &Value, label: &str, options: &DumpOptions) -> String {
    if options.compact_mode {
        format!("{label}: {}", dump_value(value, options))
    } else {
        format!("{label}:\n{}", dump_value(value, options))
    }
}

/// Simple value formatting for quick debug output
///
/// This is a convenience function that uses compact options.
/// Replaces the old `value_to_debug_string` and `format_lua_value` functions.
#[must_use]
pub fn format_simple(value: &Value) -> String {
    dump_value(value, &DumpOptions::compact())
}

/// Recursive value dumping implementation
fn dump_value_recursive(
    value: &Value,
    options: &DumpOptions,
    state: &mut DumpState,
    depth: usize,
) -> String {
    if depth > options.max_depth {
        return "...".to_string();
    }

    match value {
        Value::Nil => "nil".to_string(),
        Value::Boolean(b) => b.to_string(),
        Value::Integer(i) => {
            if options.show_types {
                format!("{i} (integer)")
            } else {
                i.to_string()
            }
        }
        Value::Number(n) => {
            #[allow(clippy::float_cmp)]
            let formatted = if n.floor() == *n && n.is_finite() {
                format!("{n:.0}")
            } else {
                format!("{n}")
            };
            if options.show_types {
                format!("{formatted} (number)")
            } else {
                formatted
            }
        }
        Value::String(s) => {
            let str_val = s.to_str().unwrap_or("<invalid utf8>");
            let truncated = if str_val.len() > options.max_string_length {
                format!(
                    "{}... ({} chars)",
                    &str_val[..options.max_string_length],
                    str_val.len()
                )
            } else {
                str_val.to_string()
            };
            if options.show_types {
                format!("\"{truncated}\" (string)")
            } else {
                format!("\"{truncated}\"")
            }
        }
        Value::Table(t) => dump_table(t, options, state, depth),
        Value::Function(_) => {
            if options.show_types {
                "function".to_string()
            } else {
                "<function>".to_string()
            }
        }
        Value::Thread(_) => {
            if options.show_types {
                "thread".to_string()
            } else {
                "<thread>".to_string()
            }
        }
        Value::UserData(_) => {
            if options.show_types {
                "userdata".to_string()
            } else {
                "<userdata>".to_string()
            }
        }
        Value::LightUserData(_) => {
            if options.show_types {
                "lightuserdata".to_string()
            } else {
                "<lightuserdata>".to_string()
            }
        }
        Value::Error(e) => format!("error: {e}"),
    }
}

/// Dump a Lua table
fn dump_table(table: &Table, options: &DumpOptions, state: &mut DumpState, depth: usize) -> String {
    // Get table pointer for cycle detection
    let ptr = table.to_pointer();
    let (seen_before, ref_id) = state.get_or_create_ref(ptr);

    if seen_before {
        return format!("<table #{ref_id}>"); // Circular reference
    }

    let mut result = String::new();

    // Check if this is an array-like table
    let (is_array, array_len) = check_if_array(table);

    if options.compact_mode {
        if is_array {
            result.push('[');
            for i in 1..=array_len.min(options.max_array_elements) {
                if i > 1 {
                    result.push_str(", ");
                }
                if let Ok(val) = table.get::<_, Value>(i) {
                    result.push_str(&dump_value_recursive(&val, options, state, depth + 1));
                }
            }
            if array_len > options.max_array_elements {
                write!(
                    result,
                    ", ... {} more",
                    array_len - options.max_array_elements
                )
                .unwrap();
            }
            result.push(']');
        } else {
            result.push('{');
            for (count, (k, v)) in table.clone().pairs::<Value, Value>().flatten().enumerate() {
                if count > 0 {
                    result.push_str(", ");
                }
                if count >= options.max_table_pairs {
                    result.push_str("...");
                    break;
                }
                let key_str = dump_value_recursive(&k, options, state, depth + 1);
                let val_str = dump_value_recursive(&v, options, state, depth + 1);
                write!(result, "{key_str}={val_str}").unwrap();
            }
            result.push('}');
        }
    } else {
        // Pretty-printed format
        let indent = " ".repeat(options.indent_size * depth);
        let inner_indent = " ".repeat(options.indent_size * (depth + 1));

        if options.show_addresses {
            write!(result, "table #{ref_id} ").unwrap();
        }

        if is_array {
            result.push_str("[\n");
            for i in 1..=array_len.min(options.max_array_elements) {
                if let Ok(val) = table.get::<_, Value>(i) {
                    writeln!(
                        result,
                        "{inner_indent}[{i}] = {},",
                        dump_value_recursive(&val, options, state, depth + 1)
                    )
                    .unwrap();
                }
            }
            if array_len > options.max_array_elements {
                writeln!(
                    result,
                    "{inner_indent}... {} more elements",
                    array_len - options.max_array_elements
                )
                .unwrap();
            }
            write!(result, "{indent}]").unwrap();
        } else {
            result.push_str("{\n");
            for (count, (k, v)) in table.clone().pairs::<Value, Value>().flatten().enumerate() {
                if count >= options.max_table_pairs {
                    writeln!(result, "{inner_indent}... more pairs").unwrap();
                    break;
                }
                let key_str = match k {
                    Value::String(s) => {
                        let str_val = s.to_str().unwrap_or("<invalid>");
                        if is_valid_identifier(str_val) {
                            str_val.to_string()
                        } else {
                            format!("[\"{str_val}\"]")
                        }
                    }
                    _ => format!("[{}]", dump_value_recursive(&k, options, state, depth + 1)),
                };
                writeln!(
                    result,
                    "{inner_indent}{key_str} = {},",
                    dump_value_recursive(&v, options, state, depth + 1)
                )
                .unwrap();
            }
            write!(result, "{indent}}}").unwrap();
        }
    }

    result
}

/// Check if a table is array-like
fn check_if_array(table: &Table) -> (bool, usize) {
    let mut max_index = 0;
    let mut is_array = true;

    for (k, _) in table.clone().pairs::<Value, Value>().flatten() {
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

    // Additional check: ensure indices are contiguous
    if is_array && max_index > 0 {
        for i in 1..=max_index {
            if table.get::<_, Value>(i).is_err() {
                is_array = false;
                break;
            }
        }
    }

    (is_array, max_index)
}

/// Check if a string is a valid Lua identifier
fn is_valid_identifier(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }

    let first_char = s.chars().next().unwrap();
    if !first_char.is_ascii_alphabetic() && first_char != '_' {
        return false;
    }

    s.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
}

// ============================================================================
// Stack Trace System (formerly stacktrace.rs)
// ============================================================================

use crate::execution_bridge::StackFrame;
use crate::execution_bridge::Variable;
use mlua::Function;

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

/// Variable capture configuration
#[derive(Debug, Clone, Default)]
pub struct CaptureConfig {
    /// Whether to capture local variables
    pub locals: bool,
    /// Whether to capture upvalues
    pub upvalues: bool,
    /// Whether to capture global variables
    pub globals: bool,
}

/// Stack trace collection options
#[derive(Debug, Clone)]
pub struct StackTraceOptions {
    /// Maximum stack depth to capture
    pub max_depth: usize,
    /// Variable capture configuration
    pub capture: CaptureConfig,
    /// Whether to include source information
    pub include_source: bool,
}

impl Default for StackTraceOptions {
    fn default() -> Self {
        Self {
            max_depth: 50,
            capture: CaptureConfig {
                locals: false, // Only at trace level
                upvalues: false,
                globals: false,
            },
            include_source: true,
        }
    }
}

impl StackTraceOptions {
    /// Create options for error-level capture (minimal)
    #[must_use]
    pub const fn for_error() -> Self {
        Self {
            max_depth: 20,
            capture: CaptureConfig {
                locals: false,
                upvalues: false,
                globals: false,
            },
            include_source: true,
        }
    }

    /// Create options for trace-level capture (comprehensive)
    #[must_use]
    pub const fn for_trace() -> Self {
        Self {
            max_depth: 50,
            capture: CaptureConfig {
                locals: true,
                upvalues: true,
                globals: false,
            },
            include_source: true,
        }
    }

    /// Create options for debugging (full capture including globals)
    #[must_use]
    pub const fn for_debug() -> Self {
        Self {
            max_depth: 100,
            capture: CaptureConfig {
                locals: true,
                upvalues: true,
                globals: true,
            },
            include_source: true,
        }
    }
}

/// Capture a stack trace from the current Lua execution point
#[must_use]
pub fn capture_stack_trace(lua: &Lua, options: &StackTraceOptions) -> StackTrace {
    match capture_stack_trace_impl(lua, options) {
        Ok(trace) => trace,
        Err(e) => StackTrace {
            frames: vec![],
            max_depth: options.max_depth,
            truncated: false,
            error: Some(format!("Stack trace capture failed: {e}")),
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

        let Ok(info_table) = info_result else { break }; // No more frames

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
    let name = info
        .get::<_, Option<String>>("name")?
        .unwrap_or_else(|| "<anonymous>".to_string());
    let source = if options.include_source {
        info.get::<_, Option<String>>("source")?
            .unwrap_or_else(|| "<unknown>".to_string())
    } else {
        "<unknown>".to_string()
    };
    let line = info.get::<_, Option<i32>>("currentline")?.unwrap_or(-1);
    let what = info
        .get::<_, String>("what")
        .unwrap_or_else(|_| "unknown".to_string());

    // Capture local variables if requested
    let locals = if options.capture.locals {
        capture_locals(lua, level)?
    } else {
        Vec::new()
    };

    // Determine if this is user code
    let is_user_code = !source.starts_with('@') && what != "C";

    Ok(StackFrame {
        id: format!("frame_{}", level - 1),
        name,
        source,
        #[allow(clippy::cast_sign_loss)]
        line: if line > 0 { line as u32 } else { 0 },
        column: None,
        locals,
        is_user_code,
    })
}

fn capture_locals(lua: &Lua, level: i32) -> LuaResult<Vec<Variable>> {
    let debug_table: Table = lua.globals().get("debug")?;
    let getlocal_fn: Function = debug_table.get("getlocal")?;

    let mut locals = Vec::new();
    let mut local_index = 1;

    loop {
        let result: LuaResult<(Option<String>, Value)> = getlocal_fn.call((level, local_index));

        match result {
            Ok((Some(name), value)) if !name.starts_with('(') => {
                // Skip internal variables starting with '('
                locals.push(Variable {
                    name,
                    value: format_simple(&value),
                    var_type: match value {
                        Value::Nil => "nil".to_string(),
                        Value::Boolean(_) => "boolean".to_string(),
                        Value::Integer(_) => "integer".to_string(),
                        Value::Number(_) => "number".to_string(),
                        Value::String(_) => "string".to_string(),
                        Value::Table(_) => "table".to_string(),
                        Value::Function(_) => "function".to_string(),
                        Value::Thread(_) => "thread".to_string(),
                        Value::UserData(_) | Value::LightUserData(_) => "userdata".to_string(),
                        Value::Error(_) => "error".to_string(),
                    },
                    has_children: matches!(value, Value::Table(_) | Value::UserData(_)),
                    reference: None,
                });
            }
            Ok((None, _)) | Err(_) => break, // No more locals
            _ => {}                          // Skip internal variables
        }

        local_index += 1;
    }

    Ok(locals)
}

/// Capture upvalues (closure variables) for a function at the given stack level
///
/// # Errors
///
/// Returns an error if the Lua debug API calls fail or if the function info cannot be retrieved.
pub fn capture_upvalues(lua: &Lua, level: i32) -> LuaResult<Vec<Variable>> {
    let debug_table: Table = lua.globals().get("debug")?;

    // First get the function at this level
    let getinfo_fn: Function = debug_table.get("getinfo")?;
    let info: Table = getinfo_fn.call((level, "f"))?;
    let func: Option<Function> = info.get("func").ok();

    let Some(func) = func else {
        return Ok(Vec::new());
    };

    let getupvalue_fn: Function = debug_table.get("getupvalue")?;
    let mut upvalues = Vec::new();
    let mut upvalue_index = 1;

    loop {
        let result: LuaResult<(Option<String>, Value)> = getupvalue_fn.call((&func, upvalue_index));

        match result {
            Ok((Some(name), value)) if !name.starts_with('(') => {
                // Skip internal upvalues starting with '('
                upvalues.push(Variable {
                    name,
                    value: format_simple(&value),
                    var_type: match value {
                        Value::Nil => "nil".to_string(),
                        Value::Boolean(_) => "boolean".to_string(),
                        Value::Integer(_) => "integer".to_string(),
                        Value::Number(_) => "number".to_string(),
                        Value::String(_) => "string".to_string(),
                        Value::Table(_) => "table".to_string(),
                        Value::Function(_) => "function".to_string(),
                        Value::Thread(_) => "thread".to_string(),
                        Value::UserData(_) | Value::LightUserData(_) => "userdata".to_string(),
                        Value::Error(_) => "error".to_string(),
                    },
                    has_children: matches!(value, Value::Table(_) | Value::UserData(_)),
                    reference: None,
                });
            }
            Ok((None, _)) | Err(_) => break, // No more upvalues
            _ => {}                          // Skip internal upvalues
        }

        upvalue_index += 1;
    }

    Ok(upvalues)
}

/// Capture global variables from the Lua environment
///
/// # Errors
///
/// Returns an error if accessing the Lua globals table fails or if iterating through the table encounters an error.
pub fn capture_globals(lua: &Lua) -> LuaResult<Vec<Variable>> {
    let globals_table = lua.globals();
    let mut globals = Vec::new();

    // Iterate through all global variables
    for (name, value) in globals_table.pairs::<String, Value>().flatten() {
        // Filter to common Lua globals and user-defined globals
        if name.starts_with('_')
            || !name.starts_with("package.")
            || [
                "print",
                "require",
                "module",
                "string",
                "table",
                "math",
                "io",
                "os",
                "debug",
                "coroutine",
                "bit",
                "jit",
                "load",
                "loadfile",
                "dofile",
                "pcall",
                "xpcall",
                "error",
                "assert",
                "type",
                "next",
                "pairs",
                "ipairs",
                "getmetatable",
                "setmetatable",
                "rawget",
                "rawset",
                "rawequal",
                "tonumber",
                "tostring",
                "select",
                "unpack",
                "collectgarbage",
            ]
            .contains(&name.as_str())
        {
            globals.push(Variable {
                name: name.clone(),
                value: format_simple(&value),
                var_type: match value {
                    Value::Nil => "nil".to_string(),
                    Value::Boolean(_) => "boolean".to_string(),
                    Value::Integer(_) => "integer".to_string(),
                    Value::Number(_) => "number".to_string(),
                    Value::String(_) => "string".to_string(),
                    Value::Table(_) => "table".to_string(),
                    Value::Function(_) => "function".to_string(),
                    Value::Thread(_) => "thread".to_string(),
                    Value::UserData(_) | Value::LightUserData(_) => "userdata".to_string(),
                    Value::Error(_) => "error".to_string(),
                },
                has_children: matches!(value, Value::Table(_) | Value::UserData(_)),
                reference: None,
            });
        }
    }

    Ok(globals)
}

impl StackTrace {
    /// Format the stack trace as a readable string
    #[must_use]
    pub fn format(&self) -> String {
        if let Some(error) = &self.error {
            return format!("Stack trace error: {error}");
        }

        if self.frames.is_empty() {
            return "No stack frames available".to_string();
        }

        let mut output = String::from("Stack trace:\n");

        for (i, frame) in self.frames.iter().enumerate() {
            let frame_header = if frame.name.is_empty() || frame.name == "<anonymous>" {
                format!("  #{i}: in <unknown>")
            } else {
                format!("  #{i}: in function '{}'", frame.name)
            };

            output.push_str(&frame_header);

            // Add source information
            if frame.source != "<unknown>" && frame.line > 0 {
                let _ = write!(output, " ({}:{})", frame.source, frame.line);
            } else if frame.source != "<unknown>" {
                let _ = write!(output, " ({})", frame.source);
            }

            output.push('\n');

            // Add local variables if available
            if !frame.locals.is_empty() {
                output.push_str("    Locals:\n");
                for var in &frame.locals {
                    let _ = writeln!(
                        output,
                        "      {} = {} ({})",
                        var.name, var.value, var.var_type
                    );
                }
            }
        }

        if self.truncated {
            let _ = writeln!(output, "  ... (truncated at {} frames)", self.max_depth);
        }

        output
    }

    /// Format as JSON string
    ///
    /// # Errors
    ///
    /// Returns an error if JSON serialization fails
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Get frame count
    #[must_use]
    pub const fn frame_count(&self) -> usize {
        self.frames.len()
    }

    /// Check if the trace was successful
    #[must_use]
    pub const fn is_successful(&self) -> bool {
        self.error.is_none()
    }

    /// Get the top frame (most recent)
    #[must_use]
    pub fn top_frame(&self) -> Option<&StackFrame> {
        self.frames.first()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_console_capture() {
        let capture = ConsoleCapture::new();

        capture.add_line("Line 1".to_string());
        capture.add_line("Line 2".to_string());

        let lines = capture.get_lines();
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0], "Line 1");
        assert_eq!(lines[1], "Line 2");

        capture.clear();
        assert!(capture.get_lines().is_empty());
    }

    #[test]
    fn test_print_override() -> LuaResult<()> {
        let lua = Lua::new();
        let capture = Arc::new(ConsoleCapture::new());

        override_print(&lua, capture.clone())?;

        // Test the overridden print
        lua.load(
            r#"
            print("Hello", "World")
            print(42, true, nil)
        "#,
        )
        .exec()?;

        let lines = capture.get_lines();
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0], "Hello\tWorld");
        assert_eq!(lines[1], "42\ttrue\tnil");

        Ok(())
    }

    #[test]
    fn test_dump_options() {
        let compact = DumpOptions::compact();
        assert!(compact.compact_mode);
        assert_eq!(compact.max_depth, 3);

        let verbose = DumpOptions::verbose();
        assert!(!verbose.compact_mode);
        assert_eq!(verbose.max_depth, 20);
        assert!(verbose.show_addresses);
    }

    #[test]
    fn test_value_dumping() {
        let options = DumpOptions::default();

        assert_eq!(dump_value(&Value::Nil, &options), "nil");
        assert_eq!(dump_value(&Value::Boolean(true), &options), "true");
        assert_eq!(dump_value(&Value::Integer(42), &options), "42 (integer)");
    }

    #[test]
    fn test_identifier_validation() {
        assert!(is_valid_identifier("hello"));
        assert!(is_valid_identifier("_test"));
        assert!(is_valid_identifier("var123"));
        assert!(!is_valid_identifier("123var"));
        assert!(!is_valid_identifier("hello-world"));
        assert!(!is_valid_identifier(""));
    }
}
