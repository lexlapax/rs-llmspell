// ABOUTME: Performance test for cross-language bridge overhead
// ABOUTME: Validates performance of script runtime and cross-language operations

#![cfg_attr(test_category = "benchmark")]

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use llmspell_bridge::{RuntimeConfig, ScriptRuntime};
use llmspell_events::{Language, UniversalEvent};
use mlua::{Lua, Result as LuaResult};
use tokio::runtime::Runtime;

/// Benchmark basic Lua script execution
fn bench_lua_script_execution(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("lua_script_basic_execution", |b| {
        b.iter(|| {
            let _ = rt.block_on(async {
                let lua = Lua::new();

                let result: LuaResult<i32> = lua
                    .load(
                        r#"
                    local sum = 0
                    for i = 1, 100 do
                        sum = sum + i
                    end
                    return sum
                "#,
                    )
                    .eval_async()
                    .await;

                black_box(result)
            });
        });
    });
}

/// Benchmark Lua table operations (simulating component data)
fn bench_lua_table_operations(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("lua_table_operations", |b| {
        b.iter(|| {
            let _ = rt.block_on(async {
                let lua = Lua::new();

                let result: LuaResult<String> = lua
                    .load(
                        r#"
                    local agent = {
                        id = "test-agent",
                        type = "BasicAgent",
                        config = {
                            temperature = 0.7,
                            max_tokens = 1000
                        },
                        state = {
                            active = true,
                            conversation = {}
                        }
                    }
                    
                    -- Simulate some operations
                    for i = 1, 10 do
                        agent.state.conversation[i] = "message-" .. i
                    end
                    
                    return agent.id .. "-processed"
                "#,
                    )
                    .eval_async()
                    .await;

                black_box(result)
            });
        });
    });
}

/// Benchmark ScriptRuntime initialization overhead
fn bench_script_runtime_initialization(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("script_runtime_initialization", |b| {
        b.iter(|| {
            let _ = rt.block_on(async {
                let config = RuntimeConfig::default();
                let runtime_result = ScriptRuntime::new_with_lua(config).await;
                black_box(runtime_result)
            });
        });
    });
}

/// Benchmark event creation and serialization across languages
fn bench_cross_language_event_serialization(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("cross_language_event_serialization", |b| {
        b.iter(|| {
            rt.block_on(async {
                // Create events in different languages
                let rust_event = UniversalEvent::new(
                    "rust.test.event",
                    serde_json::json!({
                        "source": "rust",
                        "data": "test-data",
                        "timestamp": chrono::Utc::now().timestamp()
                    }),
                    Language::Rust,
                );

                let lua_event = UniversalEvent::new(
                    "lua.test.event",
                    serde_json::json!({
                        "source": "lua",
                        "script": "test-script.lua",
                        "result": "success"
                    }),
                    Language::Lua,
                );

                // Simulate serialization overhead
                let rust_serialized = serde_json::to_string(&rust_event).unwrap();
                let lua_serialized = serde_json::to_string(&lua_event).unwrap();

                black_box((rust_serialized, lua_serialized))
            });
        });
    });
}

/// Benchmark Lua coroutine overhead (simulating async operations)
fn bench_lua_coroutine_overhead(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("lua_coroutine_overhead", |b| {
        b.iter(|| {
            let _ = rt.block_on(async {
                let lua = Lua::new();

                let result: LuaResult<String> = lua
                    .load(
                        r#"
                    local co = coroutine.create(function()
                        local result = ""
                        for i = 1, 10 do
                            coroutine.yield("step-" .. i)
                            result = result .. i .. ","
                        end
                        return result
                    end)
                    
                    local final_result = ""
                    while coroutine.status(co) ~= "dead" do
                        local success, value = coroutine.resume(co)
                        if success then
                            final_result = final_result .. (value or "")
                        end
                    end
                    
                    return final_result
                "#,
                    )
                    .eval_async()
                    .await;

                black_box(result)
            });
        });
    });
}

/// Calculate cross-language overhead
fn calculate_cross_language_overhead(_c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    println!("\n=== Cross-Language Overhead Analysis ===");

    rt.block_on(async {
        // Baseline: Pure Rust operations
        let start = tokio::time::Instant::now();
        for _ in 0..1000 {
            let data = serde_json::json!({
                "test": "data",
                "iteration": 42,
                "timestamp": chrono::Utc::now().timestamp()
            });
            let _ = serde_json::to_string(&data);
        }
        let rust_baseline = start.elapsed();

        // With Lua script execution
        let lua = Lua::new();
        let start = tokio::time::Instant::now();
        for i in 0..1000 {
            let script = format!(
                r#"
                local data = {{
                    test = "data",
                    iteration = {},
                    processed = true
                }}
                return data.test .. "-" .. data.iteration
            "#,
                i
            );

            let _: LuaResult<String> = lua.load(&script).eval_async().await;
        }
        let lua_execution = start.elapsed();

        let overhead_ns = lua_execution
            .as_nanos()
            .saturating_sub(rust_baseline.as_nanos());
        let overhead_percent = (overhead_ns as f64 / rust_baseline.as_nanos() as f64) * 100.0;

        println!("Pure Rust operations: {:?}", rust_baseline);
        println!("With Lua execution: {:?}", lua_execution);
        println!("Cross-language overhead: {:.2}%", overhead_percent);
        println!("Target: <10% (acceptable for cross-language operations)");
        println!(
            "Status: {}",
            if overhead_percent < 10.0 {
                "PASS ✅"
            } else if overhead_percent < 25.0 {
                "ACCEPTABLE ⚠️"
            } else {
                "NEEDS OPTIMIZATION ❌"
            }
        );

        // Also test script runtime initialization overhead
        println!("\n--- Script Runtime Initialization Overhead ---");

        let start = tokio::time::Instant::now();
        for _ in 0..10 {
            let config = RuntimeConfig::default();
            let _ = ScriptRuntime::new_with_lua(config).await;
        }
        let runtime_init = start.elapsed();

        let per_init = runtime_init.as_millis() / 10;
        println!("Runtime initialization (avg): {}ms", per_init);
        println!(
            "Initialization status: {}",
            if per_init < 100 {
                "FAST ✅"
            } else if per_init < 500 {
                "ACCEPTABLE ⚠️"
            } else {
                "SLOW ❌"
            }
        );
    });
}

criterion_group!(
    benches,
    bench_lua_script_execution,
    bench_lua_table_operations,
    bench_script_runtime_initialization,
    bench_cross_language_event_serialization,
    bench_lua_coroutine_overhead,
    calculate_cross_language_overhead
);
criterion_main!(benches);
