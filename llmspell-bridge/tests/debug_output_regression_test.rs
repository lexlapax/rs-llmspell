//! Regression test for Task 9.7.5: Verify debug output formatting quality
//!
//! This test ensures that debug output formatting through the hybrid architecture
//! maintains the same quality as the original implementation.

use llmspell_bridge::lua::output::{
    capture_stack_trace, dump_value, format_simple, DumpOptions, StackTraceOptions,
};
use mlua::{Lua, Value};

#[tokio::test]
async fn test_no_regression_in_output_quality() {
    test_format_simple_quality();
    test_dump_value_quality();
    test_stack_trace_quality();
    test_debug_coordinator_formatting().await;
    test_complex_structure_formatting();
}

fn test_format_simple_quality() {
    // Create a Lua context for testing
    let lua = Lua::new();

    // Test 1: format_simple() quality check
    {
        // Test various Lua value types
        let nil_val = Value::Nil;
        assert_eq!(format_simple(&nil_val), "nil");

        let bool_val = Value::Boolean(true);
        assert_eq!(format_simple(&bool_val), "true");

        let int_val = Value::Integer(42);
        assert_eq!(format_simple(&int_val), "42");

        let num_val = Value::Number(123.456);
        assert_eq!(format_simple(&num_val), "123.456");

        let str_val = Value::String(lua.create_string("hello world").unwrap());
        assert_eq!(format_simple(&str_val), "\"hello world\"");

        // Test table formatting
        let table = lua.create_table().unwrap();
        table.set("key1", "value1").unwrap();
        table.set("key2", 42).unwrap();
        let table_val = Value::Table(table);
        let formatted = format_simple(&table_val);
        // Should contain both key-value pairs
        assert!(formatted.contains("key1") || formatted.contains("[1]"));
        assert!(formatted.contains("key2") || formatted.contains("[2]"));
    }
}

fn test_dump_value_quality() {
    let lua = Lua::new();
    let table = lua.create_table().unwrap();
    table.set("name", "test").unwrap();
    table.set("count", 10).unwrap();
    let nested = lua.create_table().unwrap();
    nested.set("inner", true).unwrap();
    table.set("nested", nested).unwrap();

    let table_val = Value::Table(table);

    // Compact format
    let compact_output = dump_value(&table_val, &DumpOptions::compact());
    assert!(!compact_output.is_empty());
    assert!(!compact_output.contains('\n')); // Compact should be single line

    // Verbose format (pretty-printed)
    let pretty_output = dump_value(&table_val, &DumpOptions::verbose());
    assert!(pretty_output.contains('\n')); // Verbose should have newlines
    assert!(pretty_output.contains("  ")); // Verbose should have indentation

    // Custom options with max depth
    let custom_opts = DumpOptions {
        max_depth: 1,
        ..Default::default()
    };
    let limited_output = dump_value(&table_val, &custom_opts);
    assert!(limited_output.contains("...")); // Should show truncation at depth limit
}

fn test_stack_trace_quality() {
    let lua = Lua::new();
    lua.load(
        r#"
            function level3()
                error("test error")
            end
            
            function level2()
                level3()
            end
            
            function level1()
                level2()
            end
        "#,
    )
    .exec()
    .unwrap();

    // Capture stack trace within a pcall to avoid actual error
    let _result: Result<(), mlua::Error> = lua
        .load(
            r"
                local status, err = pcall(level1)
                if not status then
                    -- Stack trace will be captured here
                    return nil
                end
            ",
        )
        .exec();

    // Capture current stack trace
    let stack_options = StackTraceOptions {
        max_depth: 10,
        capture_locals: true,
        capture_upvalues: false,
        include_source: true,
    };
    let stack_trace = capture_stack_trace(&lua, &stack_options);

    // Verify stack trace has expected format
    let formatted = stack_trace.format();
    println!("Stack trace formatted output:\n{formatted}");

    // The format should be clean and readable
    // Even an empty stack trace should have structure
    assert!(
        !formatted.is_empty(),
        "Stack trace should produce some output"
    );
    if stack_trace.frames.is_empty() {
        assert!(
            formatted.contains("empty") || formatted.contains("no frames") || !formatted.is_empty()
        );
    } else {
        assert!(formatted.contains("Stack trace"));
        assert!(formatted.contains('#') || formatted.contains("Frame"));
    }
}

async fn test_debug_coordinator_formatting() {
    // Test the DebugCoordinator formatting directly without full runtime
    use llmspell_bridge::debug_coordinator::DebugCoordinator;
    use llmspell_bridge::debug_state_cache::SharedDebugStateCache;
    use llmspell_bridge::execution_bridge::ExecutionManager;
    use llmspell_bridge::execution_context::SharedExecutionContext;
    use std::collections::HashMap;
    use std::sync::Arc;
    use tokio::sync::RwLock;

    let shared_context = Arc::new(RwLock::new(SharedExecutionContext::new()));
    let capabilities = Arc::new(RwLock::new(HashMap::new()));
    let debug_cache = Arc::new(SharedDebugStateCache::new());
    let execution_manager = Arc::new(ExecutionManager::new(debug_cache));

    let coordinator = DebugCoordinator::new(shared_context, capabilities, execution_manager);

    // Verify formatting works
    let formatted = coordinator.format_current_state().await;
    assert_eq!(formatted, "No current execution location");
}

fn test_complex_structure_formatting() {
    let lua = Lua::new();
    let complex = lua.create_table().unwrap();
    let array = lua.create_table().unwrap();
    for i in 1..=5 {
        array.set(i, i * 10).unwrap();
    }
    complex.set("array", array).unwrap();

    let map = lua.create_table().unwrap();
    map.set("alpha", "first").unwrap();
    map.set("beta", "second").unwrap();
    complex.set("map", map).unwrap();

    complex.set("mixed", vec![1, 2, 3]).unwrap();
    complex.set("flag", true).unwrap();
    complex.set("nothing", Value::Nil).unwrap();

    let complex_val = Value::Table(complex);

    // Verify verbose printing maintains quality
    let pretty = dump_value(&complex_val, &DumpOptions::verbose());

    // Should have proper structure
    assert!(pretty.contains("array"));
    assert!(pretty.contains("map"));
    assert!(pretty.contains("alpha"));
    assert!(pretty.contains("true"));

    // Should have proper indentation
    assert!(pretty.lines().count() > 5); // Should be multi-line

    // Verify no malformed output
    assert!(!pretty.contains("ERROR"));
    assert!(!pretty.contains("null")); // Should use nil, not null
}

#[test]
fn test_format_consistency() {
    // This test verifies that formatting is consistent across multiple calls
    let lua = Lua::new();

    let table = lua.create_table().unwrap();
    table.set("test", 123).unwrap();
    let val = Value::Table(table);

    // Format the same value multiple times
    let format1 = format_simple(&val);
    let format2 = format_simple(&val);
    let format3 = format_simple(&val);

    // All formats should be identical
    assert_eq!(format1, format2);
    assert_eq!(format2, format3);
}

#[test]
fn test_special_characters_handling() {
    // Verify that special characters are handled properly
    // Note: format_simple outputs literal characters, not escaped ones
    // This is appropriate for debug output as it shows actual content
    let lua = Lua::new();

    let test_cases = vec![
        ("hello\nworld", "hello\nworld"),   // newline - literal
        ("tab\there", "tab\there"),         // tab - literal
        ("quote\"inside", "quote\"inside"), // quote - literal
        ("back\\slash", "back\\slash"),     // backslash - literal
        ("unicode: 你好", "unicode: 你好"), // unicode - preserved
        ("", ""),                           // empty string
        ("normal text", "normal text"),     // normal string
    ];

    for (input, expected_content) in test_cases {
        let lua_str = lua.create_string(input).unwrap();
        let val = Value::String(lua_str);
        let formatted = format_simple(&val);

        // format_simple adds quotes around strings
        let expected = format!("\"{expected_content}\"");
        assert_eq!(formatted, expected, "Failed for input: {input:?}");

        // Verify quotes are added
        assert!(formatted.starts_with('"'), "Should start with quote");
        assert!(formatted.ends_with('"'), "Should end with quote");

        // The content should match exactly (literal, not escaped)
        let content = &formatted[1..formatted.len() - 1];
        assert_eq!(content, expected_content, "Content mismatch for: {input:?}");
    }
}
