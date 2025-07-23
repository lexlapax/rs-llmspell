//! Performance benchmarks for JSON API

#![cfg(feature = "lua")]

use llmspell_bridge::lua::globals::json::inject_json_global;
use std::time::Instant;

/// Benchmark JSON parsing performance
#[test]
fn test_json_parse_performance() {
    let lua = mlua::Lua::new();
    inject_json_global(&lua).unwrap();

    // Small JSON (typical tool output)
    let small_json = r#"{"success":true,"result":{"uuid":"123e4567-e89b-12d3-a456-426614174000","version":"v4"}}"#;

    // Medium JSON (complex tool output)
    let medium_json = r#"{
        "success": true,
        "result": {
            "data": [1, 2, 3, 4, 5, 6, 7, 8, 9, 10],
            "metadata": {
                "count": 10,
                "type": "array",
                "processed": true
            },
            "nested": {
                "level1": {
                    "level2": {
                        "value": "deep"
                    }
                }
            }
        }
    }"#;

    // Large JSON (stress test)
    let large_data: Vec<i32> = (0..1000).collect();
    let large_json = serde_json::to_string(&serde_json::json!({
        "success": true,
        "result": {
            "data": large_data,
            "metadata": {
                "count": 1000
            }
        }
    }))
    .unwrap();

    // Benchmark small JSON parsing
    let start = Instant::now();
    let iterations = 1000;
    for _ in 0..iterations {
        lua.load(&format!(
            r#"
            local parsed = JSON.parse('{}')
            assert(parsed.success == true)
        "#,
            small_json.replace('\\', "\\\\").replace('"', r#"\""#)
        ))
        .exec()
        .unwrap();
    }
    let small_duration = start.elapsed();
    let small_per_op = small_duration.as_micros() as f64 / iterations as f64;

    // Benchmark medium JSON parsing
    let start = Instant::now();
    let iterations = 100;
    for _ in 0..iterations {
        lua.load(&format!(
            r#"
            local parsed = JSON.parse('{}')
            assert(parsed.success == true)
        "#,
            medium_json.replace('\n', "").replace('"', r#"\""#)
        ))
        .exec()
        .unwrap();
    }
    let medium_duration = start.elapsed();
    let medium_per_op = medium_duration.as_micros() as f64 / iterations as f64;

    // Benchmark large JSON parsing
    let start = Instant::now();
    let iterations = 10;
    for _ in 0..iterations {
        lua.load(&format!(
            r#"
            local parsed = JSON.parse('{}')
            assert(parsed.success == true)
        "#,
            large_json.replace('\\', "\\\\").replace('"', r#"\""#)
        ))
        .exec()
        .unwrap();
    }
    let large_duration = start.elapsed();
    let large_per_op = large_duration.as_micros() as f64 / iterations as f64;

    println!("JSON Parse Performance:");
    println!("  Small JSON (~100 bytes):  {:.2} μs/op", small_per_op);
    println!("  Medium JSON (~300 bytes): {:.2} μs/op", medium_per_op);
    println!("  Large JSON (~5KB):        {:.2} μs/op", large_per_op);

    // Assert performance requirements (<1ms for typical operations)
    assert!(small_per_op < 1000.0, "Small JSON parsing should be <1ms");
    assert!(medium_per_op < 1000.0, "Medium JSON parsing should be <1ms");
}

/// Benchmark JSON stringify performance
#[test]
fn test_json_stringify_performance() {
    let lua = mlua::Lua::new();
    inject_json_global(&lua).unwrap();

    // Setup test data in Lua
    lua.load(
        r#"
        small_data = {success = true, result = {uuid = "123e4567-e89b-12d3-a456-426614174000"}}
        medium_data = {
            success = true,
            result = {
                data = {1, 2, 3, 4, 5, 6, 7, 8, 9, 10},
                metadata = {count = 10, type = "array", processed = true},
                nested = {level1 = {level2 = {value = "deep"}}}
            }
        }
        large_data = {success = true, result = {data = {}}}
        for i = 1, 1000 do
            table.insert(large_data.result.data, i)
        end
    "#,
    )
    .exec()
    .unwrap();

    // Benchmark small data stringify
    let start = Instant::now();
    let iterations = 1000;
    for _ in 0..iterations {
        lua.load(
            r#"
            local json_str = JSON.stringify(small_data)
            assert(json_str ~= nil)
        "#,
        )
        .exec()
        .unwrap();
    }
    let small_duration = start.elapsed();
    let small_per_op = small_duration.as_micros() as f64 / iterations as f64;

    // Benchmark medium data stringify
    let start = Instant::now();
    let iterations = 100;
    for _ in 0..iterations {
        lua.load(
            r#"
            local json_str = JSON.stringify(medium_data)
            assert(json_str ~= nil)
        "#,
        )
        .exec()
        .unwrap();
    }
    let medium_duration = start.elapsed();
    let medium_per_op = medium_duration.as_micros() as f64 / iterations as f64;

    // Benchmark large data stringify
    let start = Instant::now();
    let iterations = 10;
    for _ in 0..iterations {
        lua.load(
            r#"
            local json_str = JSON.stringify(large_data)
            assert(json_str ~= nil)
        "#,
        )
        .exec()
        .unwrap();
    }
    let large_duration = start.elapsed();
    let large_per_op = large_duration.as_micros() as f64 / iterations as f64;

    println!("\nJSON Stringify Performance:");
    println!("  Small data:  {:.2} μs/op", small_per_op);
    println!("  Medium data: {:.2} μs/op", medium_per_op);
    println!("  Large data:  {:.2} μs/op", large_per_op);

    // Assert performance requirements (<1ms for typical operations)
    assert!(small_per_op < 1000.0, "Small data stringify should be <1ms");
    assert!(
        medium_per_op < 1000.0,
        "Medium data stringify should be <1ms"
    );
}

/// Benchmark roundtrip performance (parse -> stringify -> parse)
#[test]
fn test_json_roundtrip_performance() {
    let lua = mlua::Lua::new();
    inject_json_global(&lua).unwrap();

    let tool_output = r#"{"success":true,"result":{"operation":"completed","data":{"items":[1,2,3],"metadata":{"count":3,"type":"array"}}}}"#;

    let start = Instant::now();
    let iterations = 100;

    for _ in 0..iterations {
        lua.load(&format!(
            r#"
            local original = '{}'
            local parsed = JSON.parse(original)
            local stringified = JSON.stringify(parsed)
            local reparsed = JSON.parse(stringified)
            assert(reparsed.success == parsed.success)
            assert(reparsed.result.operation == parsed.result.operation)
        "#,
            tool_output.replace('\\', "\\\\").replace('"', r#"\""#)
        ))
        .exec()
        .unwrap();
    }

    let duration = start.elapsed();
    let per_op = duration.as_micros() as f64 / iterations as f64;

    println!("\nJSON Roundtrip Performance:");
    println!("  Parse->Stringify->Parse: {:.2} μs/op", per_op);

    // Assert performance requirement
    assert!(per_op < 1000.0, "Roundtrip should be <1ms");
}
