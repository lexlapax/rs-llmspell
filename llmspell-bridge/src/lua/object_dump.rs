//! Advanced object dumping utilities for Lua values
//!
//! Provides comprehensive value inspection with configurable formatting,
//! depth limiting, and type-specific pretty printing.

use mlua::{Lua, Result as LuaResult, Table, Value};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Write;

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
    pub fn compact() -> Self {
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
    pub fn verbose() -> Self {
        Self {
            max_depth: 20,
            indent_size: 4,
            max_string_length: 1000,
            max_array_elements: 200,
            max_table_pairs: 200,
            show_types: true,
            show_addresses: true,
            compact_mode: false,
        }
    }

    /// Create options for debugging specific depth
    #[must_use]
    pub fn depth(depth: usize) -> Self {
        Self {
            max_depth: depth,
            ..Self::default()
        }
    }
}

/// Context for tracking dumping state
#[derive(Debug)]
struct DumpContext {
    options: DumpOptions,
    current_depth: usize,
    visited_tables: HashMap<*const u8, usize>, // Track circular references
    output: String,
}

impl DumpContext {
    fn new(options: DumpOptions) -> Self {
        Self {
            options,
            current_depth: 0,
            visited_tables: HashMap::new(),
            output: String::new(),
        }
    }

    fn indent(&self) -> String {
        if self.options.compact_mode {
            String::new()
        } else {
            " ".repeat(self.current_depth * self.options.indent_size)
        }
    }

    fn newline(&self) -> &'static str {
        if self.options.compact_mode {
            " "
        } else {
            "\n"
        }
    }

    fn write_indent(&mut self) {
        if !self.options.compact_mode {
            self.output.push_str(&self.indent());
        }
    }
}

/// Dump a Lua value with advanced formatting options
pub fn dump_value(value: &Value, options: &DumpOptions) -> String {
    let mut context = DumpContext::new(options.clone());
    dump_value_impl(value, &mut context);
    context.output
}

/// Dump a Lua value with a label
pub fn dump_labeled_value(value: &Value, label: &str, options: &DumpOptions) -> String {
    let dumped = dump_value(value, options);
    if options.compact_mode {
        format!("{}: {}", label, dumped)
    } else {
        format!("{}:\n{}", label, dumped)
    }
}

fn dump_value_impl(value: &Value, context: &mut DumpContext) {
    if context.current_depth > context.options.max_depth {
        context.output.push_str("...");
        return;
    }

    match value {
        Value::Nil => dump_nil(context),
        Value::Boolean(b) => dump_boolean(*b, context),
        Value::Integer(i) => dump_integer(*i, context),
        Value::Number(n) => dump_number(*n, context),
        Value::String(s) => dump_string(s, context),
        Value::Table(t) => {
            if let Err(_) = dump_table(t, context) {
                context.output.push_str("<table dump error>");
            }
        }
        Value::Function(_) => dump_function(context),
        Value::Thread(_) => dump_thread(context),
        Value::UserData(_) => dump_userdata(context),
        Value::LightUserData(_) => dump_light_userdata(context),
        Value::Error(e) => dump_error(e, context),
    }
}

fn dump_nil(context: &mut DumpContext) {
    if context.options.show_types {
        context.output.push_str("nil (nil)");
    } else {
        context.output.push_str("nil");
    }
}

fn dump_boolean(value: bool, context: &mut DumpContext) {
    if context.options.show_types {
        let _ = write!(context.output, "{} (boolean)", value);
    } else {
        let _ = write!(context.output, "{}", value);
    }
}

fn dump_integer(value: i64, context: &mut DumpContext) {
    if context.options.show_types {
        let _ = write!(context.output, "{} (integer)", value);
    } else {
        let _ = write!(context.output, "{}", value);
    }
}

fn dump_number(value: f64, context: &mut DumpContext) {
    if context.options.show_types {
        let _ = write!(context.output, "{} (number)", value);
    } else {
        let _ = write!(context.output, "{}", value);
    }
}

fn dump_string(value: &mlua::String, context: &mut DumpContext) {
    match value.to_str() {
        Ok(s) => {
            let truncated = if s.len() > context.options.max_string_length {
                format!("{}...", &s[..context.options.max_string_length - 3])
            } else {
                s.to_string()
            };

            let escaped = truncated.replace('\\', "\\\\").replace('"', "\\\"");

            if context.options.show_types {
                let _ = write!(
                    context.output,
                    "\"{}\" (string, {} chars)",
                    escaped,
                    s.len()
                );
            } else {
                let _ = write!(context.output, "\"{}\"", escaped);
            }
        }
        Err(_) => {
            context.output.push_str("<invalid UTF-8 string>");
        }
    }
}

fn dump_table(table: &Table, context: &mut DumpContext) -> LuaResult<()> {
    // Check for circular references
    let table_ptr = table.to_pointer() as *const u8;
    if let Some(first_depth) = context.visited_tables.get(&table_ptr) {
        let _ = write!(
            context.output,
            "<circular reference to depth {}>",
            first_depth
        );
        return Ok(());
    }

    context
        .visited_tables
        .insert(table_ptr, context.current_depth);
    context.current_depth += 1;

    // Check if it's an array-like table
    let (is_array, array_length) = check_array_like(table)?;

    if is_array && array_length > 0 {
        dump_array_table(table, array_length, context)?;
    } else {
        dump_hash_table(table, context)?;
    }

    context.current_depth -= 1;
    context.visited_tables.remove(&table_ptr);

    Ok(())
}

fn check_array_like(table: &Table) -> LuaResult<(bool, usize)> {
    let mut max_index = 0;
    let mut is_array = true;

    for pair in table.clone().pairs::<Value, Value>() {
        let (key, _) = pair?;
        match key {
            Value::Integer(i) if i > 0 => {
                max_index = max_index.max(i as usize);
            }
            _ => {
                is_array = false;
                break;
            }
        }
    }

    Ok((is_array, max_index))
}

fn dump_array_table(table: &Table, length: usize, context: &mut DumpContext) -> LuaResult<()> {
    let type_info = if context.options.show_types {
        format!(" (array, {} elements)", length)
    } else {
        String::new()
    };

    if context.options.compact_mode {
        context.output.push('[');
        let display_length = context.options.max_array_elements.min(length);

        for i in 1..=display_length {
            if i > 1 {
                context.output.push_str(", ");
            }
            let value: Value = table.get(i)?;
            dump_value_impl(&value, context);
        }

        if length > display_length {
            context.output.push_str(", ...");
        }

        context.output.push(']');
        context.output.push_str(&type_info);
    } else {
        let _ = write!(context.output, "[{}{}", type_info, context.newline());
        let display_length = context.options.max_array_elements.min(length);

        for i in 1..=display_length {
            context.write_indent();
            let _ = write!(context.output, "[{}] = ", i);
            let value: Value = table.get(i)?;
            dump_value_impl(&value, context);
            context.output.push_str(&format!(",{}", context.newline()));
        }

        if length > display_length {
            context.write_indent();
            let _ = write!(
                context.output,
                "... {} more elements{}",
                length - display_length,
                context.newline()
            );
        }

        // Decrease indentation for closing bracket
        if context.current_depth > 0 {
            context.current_depth -= 1;
            context.write_indent();
            context.current_depth += 1;
        }
        context.output.push(']');
    }

    Ok(())
}

fn dump_hash_table(table: &Table, context: &mut DumpContext) -> LuaResult<()> {
    let mut pairs_count = 0;
    let mut pairs_vec = Vec::new();

    // Collect pairs for counting and sorting
    for pair in table.clone().pairs::<Value, Value>() {
        pairs_vec.push(pair?);
        pairs_count += 1;
    }

    let type_info = if context.options.show_types {
        format!(" (table, {} pairs)", pairs_count)
    } else {
        String::new()
    };

    if context.options.compact_mode {
        context.output.push('{');
        let display_count = context.options.max_table_pairs.min(pairs_count);

        for (i, (key, value)) in pairs_vec.iter().take(display_count).enumerate() {
            if i > 0 {
                context.output.push_str(", ");
            }
            dump_table_key(key, context);
            context.output.push_str(" = ");
            dump_value_impl(value, context);
        }

        if pairs_count > display_count {
            context.output.push_str(", ...");
        }

        context.output.push('}');
        context.output.push_str(&type_info);
    } else {
        let _ = write!(context.output, "{{{}{}", type_info, context.newline());
        let display_count = context.options.max_table_pairs.min(pairs_count);

        for (key, value) in pairs_vec.iter().take(display_count) {
            context.write_indent();
            dump_table_key(key, context);
            context.output.push_str(" = ");
            dump_value_impl(value, context);
            context.output.push_str(&format!(",{}", context.newline()));
        }

        if pairs_count > display_count {
            context.write_indent();
            let _ = write!(
                context.output,
                "... {} more pairs{}",
                pairs_count - display_count,
                context.newline()
            );
        }

        // Decrease indentation for closing brace
        if context.current_depth > 0 {
            context.current_depth -= 1;
            context.write_indent();
            context.current_depth += 1;
        }
        context.output.push('}');
    }

    Ok(())
}

fn dump_table_key(key: &Value, context: &mut DumpContext) {
    match key {
        Value::String(s) => {
            if let Ok(str_val) = s.to_str() {
                // Check if it's a valid identifier
                if str_val.chars().all(|c| c.is_alphanumeric() || c == '_')
                    && str_val
                        .chars()
                        .next()
                        .map_or(false, |c| c.is_alphabetic() || c == '_')
                {
                    context.output.push_str(str_val);
                } else {
                    let _ = write!(context.output, "[{}]", str_val);
                }
            } else {
                context.output.push_str("[<invalid string>]");
            }
        }
        _ => {
            context.output.push('[');
            dump_value_impl(key, context);
            context.output.push(']');
        }
    }
}

fn dump_function(context: &mut DumpContext) {
    if context.options.show_types {
        context.output.push_str("<function> (function)");
    } else {
        context.output.push_str("<function>");
    }
}

fn dump_thread(context: &mut DumpContext) {
    if context.options.show_types {
        context.output.push_str("<thread> (thread)");
    } else {
        context.output.push_str("<thread>");
    }
}

fn dump_userdata(context: &mut DumpContext) {
    if context.options.show_types {
        context.output.push_str("<userdata> (userdata)");
    } else {
        context.output.push_str("<userdata>");
    }
}

fn dump_light_userdata(context: &mut DumpContext) {
    if context.options.show_types {
        context.output.push_str("<lightuserdata> (lightuserdata)");
    } else {
        context.output.push_str("<lightuserdata>");
    }
}

fn dump_error(error: &mlua::Error, context: &mut DumpContext) {
    if context.options.show_types {
        let _ = write!(context.output, "<error: {}> (error)", error);
    } else {
        let _ = write!(context.output, "<error: {}>", error);
    }
}

/// Create Lua functions for object dumping
pub fn create_dump_functions(lua: &Lua) -> LuaResult<Table> {
    let dump_table = lua.create_table()?;

    // dump(value, [options])
    let dump_fn = lua.create_function(|_lua, (value, options): (Value, Option<Table>)| {
        let dump_options = if let Some(opts) = options {
            DumpOptions {
                max_depth: opts.get("max_depth").unwrap_or(10),
                indent_size: opts.get("indent_size").unwrap_or(2),
                max_string_length: opts.get("max_string_length").unwrap_or(200),
                max_array_elements: opts.get("max_array_elements").unwrap_or(50),
                max_table_pairs: opts.get("max_table_pairs").unwrap_or(50),
                show_types: opts.get("show_types").unwrap_or(true),
                show_addresses: opts.get("show_addresses").unwrap_or(false),
                compact_mode: opts.get("compact_mode").unwrap_or(false),
            }
        } else {
            DumpOptions::default()
        };

        Ok(dump_value(&value, &dump_options))
    })?;
    dump_table.set("dump", dump_fn)?;

    // dumpCompact(value)
    let dump_compact_fn =
        lua.create_function(|_lua, value: Value| Ok(dump_value(&value, &DumpOptions::compact())))?;
    dump_table.set("dumpCompact", dump_compact_fn)?;

    // dumpVerbose(value)
    let dump_verbose_fn =
        lua.create_function(|_lua, value: Value| Ok(dump_value(&value, &DumpOptions::verbose())))?;
    dump_table.set("dumpVerbose", dump_verbose_fn)?;

    // dumpWithLabel(value, label, [options])
    let dump_labeled_fn = lua.create_function(
        |_lua, (value, label, options): (Value, String, Option<Table>)| {
            let dump_options = if let Some(opts) = options {
                DumpOptions {
                    max_depth: opts.get("max_depth").unwrap_or(10),
                    indent_size: opts.get("indent_size").unwrap_or(2),
                    max_string_length: opts.get("max_string_length").unwrap_or(200),
                    max_array_elements: opts.get("max_array_elements").unwrap_or(50),
                    max_table_pairs: opts.get("max_table_pairs").unwrap_or(50),
                    show_types: opts.get("show_types").unwrap_or(true),
                    show_addresses: opts.get("show_addresses").unwrap_or(false),
                    compact_mode: opts.get("compact_mode").unwrap_or(false),
                }
            } else {
                DumpOptions::default()
            };

            Ok(dump_labeled_value(&value, &label, &dump_options))
        },
    )?;
    dump_table.set("dumpWithLabel", dump_labeled_fn)?;

    Ok(dump_table)
}

#[cfg(test)]
mod tests {
    use super::*;
    use mlua::Lua;

    #[test]
    fn test_dump_options() {
        let default = DumpOptions::default();
        assert_eq!(default.max_depth, 10);
        assert_eq!(default.indent_size, 2);

        let compact = DumpOptions::compact();
        assert_eq!(compact.max_depth, 3);
        assert!(compact.compact_mode);

        let verbose = DumpOptions::verbose();
        assert_eq!(verbose.max_depth, 20);
        assert!(verbose.show_addresses);
    }

    #[test]
    fn test_dump_primitives() -> LuaResult<()> {
        let lua = Lua::new();
        let options = DumpOptions::default();

        // Test nil
        let nil_val = Value::Nil;
        let dump = dump_value(&nil_val, &options);
        assert!(dump.contains("nil"));

        // Test boolean
        let bool_val = Value::Boolean(true);
        let dump = dump_value(&bool_val, &options);
        assert!(dump.contains("true"));

        // Test number
        let num_val = Value::Integer(42);
        let dump = dump_value(&num_val, &options);
        assert!(dump.contains("42"));

        // Test string
        let str_val = lua.create_string("hello")?;
        let dump = dump_value(&Value::String(str_val), &options);
        assert!(dump.contains("hello"));

        Ok(())
    }

    #[test]
    fn test_dump_table() -> LuaResult<()> {
        let lua = Lua::new();
        let options = DumpOptions::default();

        // Create a test table
        let table = lua.create_table()?;
        table.set("name", "test")?;
        table.set("value", 42)?;
        table.set(1, "first")?;
        table.set(2, "second")?;

        let dump = dump_value(&Value::Table(table), &options);
        assert!(dump.contains("name"));
        assert!(dump.contains("test"));
        assert!(dump.contains("42"));

        Ok(())
    }

    #[test]
    fn test_compact_vs_verbose() -> LuaResult<()> {
        let lua = Lua::new();

        let table = lua.create_table()?;
        table.set("key", "value")?;

        let compact = dump_value(&Value::Table(table.clone()), &DumpOptions::compact());
        let verbose = dump_value(&Value::Table(table), &DumpOptions::verbose());

        // Compact should be shorter and not contain newlines
        assert!(compact.len() < verbose.len());
        assert!(!compact.contains('\n'));
        assert!(verbose.contains('\n'));

        Ok(())
    }
}
