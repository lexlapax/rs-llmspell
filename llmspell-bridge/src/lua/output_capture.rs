//! Output capture for Lua `print()` and other console functions
//!
//! Routes Lua output through the debug infrastructure for capture and analysis.

use crate::debug_bridge::DebugBridge;
use mlua::{Lua, MultiValue, Result as LuaResult, Value};
use parking_lot::Mutex;
use std::sync::Arc;
use tracing::{debug, instrument, trace};

/// Console output collector
#[derive(Clone)]
pub struct ConsoleCapture {
    /// Captured output lines
    lines: Arc<Mutex<Vec<String>>>,
    /// Optional debug bridge for routing to debug system
    debug_bridge: Option<Arc<DebugBridge>>,
    /// Optional callback for real-time output streaming
    output_callback: Arc<parking_lot::RwLock<Option<Arc<dyn Fn(&str) + Send + Sync>>>>,
}

impl std::fmt::Debug for ConsoleCapture {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ConsoleCapture")
            .field("lines", &self.lines)
            .field("debug_bridge", &self.debug_bridge)
            .field("output_callback", &self.output_callback.read().as_ref().map(|_| "Callback"))
            .finish()
    }
}

impl ConsoleCapture {
    /// Create a new console capture
    #[must_use]
    pub fn new() -> Self {
        Self {
            lines: Arc::new(Mutex::new(Vec::new())),
            debug_bridge: None,
            output_callback: Arc::new(parking_lot::RwLock::new(None)),
        }
    }

    /// Create a console capture with debug bridge
    #[must_use]
    pub fn with_debug_bridge(bridge: Arc<DebugBridge>) -> Self {
        Self {
            lines: Arc::new(Mutex::new(Vec::new())),
            debug_bridge: Some(bridge),
            output_callback: Arc::new(parking_lot::RwLock::new(None)),
        }
    }

    /// Set output callback
    /// Set output callback
    pub fn set_output_callback(&self, callback: Arc<dyn Fn(&str) + Send + Sync + 'static>) {
        *self.output_callback.write() = Some(callback);
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
        // Route to debug system if available
        if let Some(bridge) = &self.debug_bridge {
            bridge.log("info", &line, Some("lua.print"));
        }

        // Also capture locally
        // Also capture locally
        self.lines.lock().push(line.clone());

        // Stream via callback if configured
        if let Some(callback) = self.output_callback.read().as_ref() {
            callback(&line);
        }
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
#[instrument(level = "trace", skip(lua, capture))]
pub fn override_print(lua: &Lua, capture: Arc<ConsoleCapture>) -> LuaResult<()> {
    trace!("Overriding Lua print function");
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
#[instrument(level = "debug", skip(lua, debug_bridge), fields(
    has_debug_bridge = debug_bridge.is_some()
))]
pub fn install_output_capture(
    lua: &Lua,
    debug_bridge: Option<Arc<DebugBridge>>,
) -> LuaResult<Arc<ConsoleCapture>> {
    debug!("Installing Lua output capture");
    let capture = debug_bridge.map_or_else(
        || Arc::new(ConsoleCapture::new()),
        |bridge| Arc::new(ConsoleCapture::with_debug_bridge(bridge)),
    );

    override_print(lua, capture.clone())?;
    override_io_functions(lua, capture.clone())?;

    Ok(capture)
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
}
