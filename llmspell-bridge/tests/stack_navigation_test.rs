//! Tests for stack navigation system (Task 9.2.9)
//!
//! These tests verify that:
//! - Stack navigation operates on cached frames from `SharedExecutionContext`
//! - Frame switching requires no hook operations
//! - Navigation is instant (<1ms)
//! - Lua-specific formatting works correctly

use llmspell_bridge::{
    debug_state_cache::{DebugStateCache, SharedDebugStateCache},
    execution_bridge::StackFrame,
    execution_context::SharedExecutionContext,
    lua::stack_navigator_impl::LuaStackNavigator,
    stack_navigator::{SharedStackNavigator, StackNavigator},
};
use std::sync::Arc;
use std::time::Instant;

/// Test basic stack navigator creation
#[test]
fn test_stack_navigator_creation() {
    let navigator = SharedStackNavigator::new();
    assert_eq!(navigator.get_current_frame(), 0);
}

/// Test frame navigation
#[test]
fn test_frame_navigation() {
    let navigator = SharedStackNavigator::new();

    let stack = vec![
        StackFrame {
            id: "frame0".to_string(),
            name: "main".to_string(),
            source: "main.lua".to_string(),
            line: 1,
            column: None,
            locals: Vec::new(),
            is_user_code: true,
        },
        StackFrame {
            id: "frame1".to_string(),
            name: "helper".to_string(),
            source: "helper.lua".to_string(),
            line: 10,
            column: Some(5),
            locals: Vec::new(),
            is_user_code: true,
        },
        StackFrame {
            id: "frame2".to_string(),
            name: "util".to_string(),
            source: "util.lua".to_string(),
            line: 20,
            column: None,
            locals: Vec::new(),
            is_user_code: true,
        },
    ];

    // Navigate to frame 1
    let result = navigator.navigate_to_frame_common(1, &stack);
    assert!(result.is_ok());

    let frame = result.unwrap();
    assert_eq!(frame.name, "helper");
    assert_eq!(frame.line, 10);
    assert_eq!(navigator.get_current_frame(), 1);

    // Navigate to frame 2
    let result = navigator.navigate_to_frame_common(2, &stack);
    assert!(result.is_ok());
    assert_eq!(navigator.get_current_frame(), 2);

    // Try invalid navigation
    let result = navigator.navigate_to_frame_common(10, &stack);
    assert!(result.is_err());
}

/// Test frame formatting
#[test]
fn test_frame_formatting() {
    let navigator = SharedStackNavigator::new();

    // Test frame with column
    let frame1 = StackFrame {
        id: "frame1".to_string(),
        name: "function1".to_string(),
        source: "test.lua".to_string(),
        line: 10,
        column: Some(5),
        locals: Vec::new(),
        is_user_code: true,
    };

    let formatted = navigator.format_frame_common(&frame1);
    assert_eq!(formatted, "test.lua:10:5 in function1");

    // Test frame without column
    let frame2 = StackFrame {
        id: "frame2".to_string(),
        name: "function2".to_string(),
        source: "test.lua".to_string(),
        line: 20,
        column: None,
        locals: Vec::new(),
        is_user_code: true,
    };

    let formatted = navigator.format_frame_common(&frame2);
    assert_eq!(formatted, "test.lua:20 in function2");
}

/// Test stack trace formatting
#[test]
fn test_stack_trace_formatting() {
    let navigator = SharedStackNavigator::new();

    let stack = vec![
        StackFrame {
            id: "frame0".to_string(),
            name: "main".to_string(),
            source: "main.lua".to_string(),
            line: 1,
            column: None,
            locals: Vec::new(),
            is_user_code: true,
        },
        StackFrame {
            id: "frame1".to_string(),
            name: "helper".to_string(),
            source: "helper.lua".to_string(),
            line: 10,
            column: Some(5),
            locals: Vec::new(),
            is_user_code: true,
        },
    ];

    let trace = navigator.format_stack_trace_common(&stack, 0);
    assert!(trace.contains("> [0] main.lua:1 in main"));
    assert!(trace.contains("  [1] helper.lua:10:5 in helper"));
}

/// Test integration with `SharedExecutionContext`
#[test]
fn test_execution_context_integration() {
    let navigator = SharedStackNavigator::new();
    let mut context = SharedExecutionContext::new();

    // Add stack frames to context
    context.stack = vec![
        StackFrame {
            id: "frame0".to_string(),
            name: "main".to_string(),
            source: "main.lua".to_string(),
            line: 1,
            column: None,
            locals: Vec::new(),
            is_user_code: true,
        },
        StackFrame {
            id: "frame1".to_string(),
            name: "process".to_string(),
            source: "process.lua".to_string(),
            line: 15,
            column: Some(3),
            locals: Vec::new(),
            is_user_code: true,
        },
    ];

    // Add variables to context
    context
        .variables
        .insert("x".to_string(), serde_json::json!(42));
    context
        .variables
        .insert("name".to_string(), serde_json::json!("test"));

    // Navigate using cached stack
    let result = navigator.navigate_to_frame_common(1, &context.stack);
    assert!(result.is_ok());

    // Get variables for frame (currently returns all variables)
    let vars = navigator.get_frame_variables_common(&context.stack[1], &context);
    assert_eq!(vars.get("x"), Some(&serde_json::json!(42)));
    assert_eq!(vars.get("name"), Some(&serde_json::json!("test")));
}

/// Test Lua-specific stack navigator
#[test]
fn test_lua_stack_navigator() {
    let navigator = LuaStackNavigator::new();

    // Test Lua-specific frame formatting
    let main_chunk = StackFrame {
        id: "frame1".to_string(),
        name: "(main chunk)".to_string(),
        source: "test.lua".to_string(),
        line: 1,
        column: None,
        locals: Vec::new(),
        is_user_code: true,
    };

    let formatted = navigator.format_frame(&main_chunk);
    assert!(formatted.contains("[main]"));

    // Test tail call formatting
    let tail_call = StackFrame {
        id: "frame2".to_string(),
        name: "(tail call)".to_string(),
        source: "test.lua".to_string(),
        line: 10,
        column: None,
        locals: Vec::new(),
        is_user_code: true,
    };

    let formatted = navigator.format_frame(&tail_call);
    assert!(formatted.contains("[tail]"));

    // Test regular function
    let regular = StackFrame {
        id: "frame3".to_string(),
        name: "my_function".to_string(),
        source: "test.lua".to_string(),
        line: 20,
        column: Some(5),
        locals: Vec::new(),
        is_user_code: true,
    };

    let formatted = navigator.format_frame(&regular);
    assert!(formatted.contains("test.lua:20:5 in my_function"));
}

/// Test integration with `DebugStateCache`
#[test]
fn test_debug_cache_integration() {
    let cache = Arc::new(SharedDebugStateCache::new());

    // Test navigation state in cache
    assert_eq!(cache.get_current_frame_index(), 0);

    cache.set_current_frame_index(3);
    assert_eq!(cache.get_current_frame_index(), 3);

    cache.set_current_frame_index(1);
    assert_eq!(cache.get_current_frame_index(), 1);
}

/// Test navigation performance
#[test]
fn test_navigation_performance() {
    let navigator = SharedStackNavigator::new();

    // Create a large stack
    let mut stack = Vec::new();
    for i in 0..100u32 {
        stack.push(StackFrame {
            id: format!("frame{i}"),
            name: format!("function{i}"),
            source: format!("file{i}.lua"),
            line: i,
            column: if i % 2 == 0 { Some(i) } else { None },
            locals: Vec::new(),
            is_user_code: true,
        });
    }

    // Measure navigation performance
    let start = Instant::now();

    // Navigate through all frames
    for i in 0..100usize {
        let result = navigator.navigate_to_frame_common(i, &stack);
        assert!(result.is_ok());
    }

    let elapsed = start.elapsed();

    // Navigation should be instant (<1ms for 100 navigations)
    println!("Navigation time for 100 frames: {elapsed:?}");
    assert!(elapsed.as_millis() < 1);
}

/// Test zero overhead for read operations
#[test]
fn test_zero_overhead_reads() {
    let navigator = SharedStackNavigator::new();
    let context = SharedExecutionContext::new();

    // Create a frame
    let frame = StackFrame {
        id: "frame1".to_string(),
        name: "test".to_string(),
        source: "test.lua".to_string(),
        line: 10,
        column: None,
        locals: Vec::new(),
        is_user_code: true,
    };

    // Measure read performance
    let start = Instant::now();

    // Perform 10000 read operations
    for _ in 0..10000 {
        let _ = navigator.get_current_frame();
        let _ = navigator.format_frame_common(&frame);
        let _ = navigator.get_frame_variables_common(&frame, &context);
    }

    let elapsed = start.elapsed();

    // Should be very fast (< 10ms for 30000 operations)
    println!("Read operations time (30000 ops): {elapsed:?}");
    assert!(elapsed.as_millis() < 10);
}

/// Test Lua stack trace with various frame types
#[test]
fn test_lua_stack_trace_comprehensive() {
    let navigator = LuaStackNavigator::new();

    let stack = vec![
        StackFrame {
            id: "frame0".to_string(),
            name: "(main chunk)".to_string(),
            source: "[string \"local x = 1\"]".to_string(),
            line: 1,
            column: None,
            locals: Vec::new(),
            is_user_code: true,
        },
        StackFrame {
            id: "frame1".to_string(),
            name: "process_data".to_string(),
            source: "processor.lua".to_string(),
            line: 42,
            column: Some(8),
            locals: Vec::new(),
            is_user_code: true,
        },
        StackFrame {
            id: "frame2".to_string(),
            name: "(tail call)".to_string(),
            source: "utils.lua".to_string(),
            line: 100,
            column: None,
            locals: Vec::new(),
            is_user_code: true,
        },
        StackFrame {
            id: "frame3".to_string(),
            name: "(C)".to_string(),
            source: "[C]".to_string(),
            line: 0,
            column: None,
            locals: Vec::new(),
            is_user_code: false,
        },
    ];

    let trace = navigator.format_stack_trace(&stack, 1);

    // Verify formatting
    assert!(trace.contains("[main]"));
    assert!(trace.contains("→ [1]")); // Current frame marker
    assert!(trace.contains("[tail]"));
    assert!(trace.contains("processor.lua:42:8"));
}

/// Test that navigation doesn't require hooks
#[test]
fn test_no_hook_requirement() {
    // This test verifies that stack navigation is purely read-only
    // and doesn't require any Lua hooks or execution context modifications

    let navigator = SharedStackNavigator::new();
    let cache = Arc::new(SharedDebugStateCache::new());

    // Create stack without any Lua engine or hooks
    let stack = vec![StackFrame {
        id: "frame1".to_string(),
        name: "test".to_string(),
        source: "test.lua".to_string(),
        line: 1,
        column: None,
        locals: Vec::new(),
        is_user_code: true,
    }];

    // All operations should work without hooks
    let result = navigator.navigate_to_frame_common(0, &stack);
    assert!(result.is_ok());

    let formatted = navigator.format_frame_common(&stack[0]);
    assert!(!formatted.is_empty());

    // Cache operations also work without hooks
    cache.set_current_frame_index(0);
    assert_eq!(cache.get_current_frame_index(), 0);
}
