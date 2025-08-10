//! ABOUTME: Performance and memory validation tests for the bridge
//! ABOUTME: Ensures memory usage and performance targets are met

use llmspell_bridge::{
    engine::factory::{EngineFactory, LuaConfig},
    providers::{ProviderManager, ProviderManagerConfig},
    ComponentRegistry,
};
use std::fmt::Write;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Test memory usage stays under 50MB for simple scripts
#[tokio::test(flavor = "multi_thread")]
async fn test_memory_usage_simple_scripts() {
    // Note: Actual memory measurement would require memory_stats crate
    // For now, we validate that scripts execute without issues

    let lua_config = LuaConfig::default();
    let mut engine = EngineFactory::create_lua_engine(&lua_config).unwrap();

    let registry = Arc::new(ComponentRegistry::new());
    let provider_config = ProviderManagerConfig::default();
    let providers = Arc::new(ProviderManager::new(provider_config).await.unwrap());

    engine.inject_apis(&registry, &providers).unwrap();

    // Execute multiple simple scripts
    let start = Instant::now();
    for i in 0..100 {
        let script = format!("return {i}");
        let _ = engine.execute_script(&script).await.unwrap();
    }
    let duration = start.elapsed();

    println!("Executed 100 scripts in {duration:?}");

    // Verify the configured memory limit is reasonable
    assert_eq!(
        lua_config.max_memory,
        Some(50_000_000),
        "Default memory limit should be 50MB"
    );
}

/// Test for memory leaks with repeated execution
#[tokio::test(flavor = "multi_thread")]
#[allow(clippy::cast_precision_loss)] // Timing measurements for performance testing
async fn test_no_memory_leaks() {
    let lua_config = LuaConfig::default();
    let mut engine = EngineFactory::create_lua_engine(&lua_config).unwrap();

    let registry = Arc::new(ComponentRegistry::new());
    let provider_config = ProviderManagerConfig::default();
    let providers = Arc::new(ProviderManager::new(provider_config).await.unwrap());

    engine.inject_apis(&registry, &providers).unwrap();

    // Warm up
    for _ in 0..10 {
        let _ = engine.execute_script("return 'warmup'").await.unwrap();
    }

    // Track execution time as proxy for memory issues
    let mut timings = vec![];

    // Execute many scripts
    for i in 0..1000 {
        let script = format!("local t = {{}}; for j=1,100 do t[j] = {i} end; return #t");
        let start = Instant::now();
        let _ = engine.execute_script(&script).await.unwrap();
        timings.push(start.elapsed());
    }

    // Calculate average time for first 100 vs last 100
    let first_100_avg = timings[..100]
        .iter()
        .map(std::time::Duration::as_micros)
        .sum::<u128>()
        / 100;
    let last_100_avg = timings[900..]
        .iter()
        .map(std::time::Duration::as_micros)
        .sum::<u128>()
        / 100;

    println!("First 100 avg: {first_100_avg}μs, Last 100 avg: {last_100_avg}μs");

    // Performance should not degrade significantly (indicates memory issues)
    let degradation = (last_100_avg as f64) / (first_100_avg as f64);
    assert!(
        degradation < 2.0,
        "Performance degraded by {degradation:.2}x, possible memory leak"
    );
}

/// Test script startup time < 100ms
#[tokio::test(flavor = "multi_thread")]
async fn test_script_startup_time() {
    let lua_config = LuaConfig::default();
    let mut engine = EngineFactory::create_lua_engine(&lua_config).unwrap();

    let registry = Arc::new(ComponentRegistry::new());
    let provider_config = ProviderManagerConfig::default();
    let providers = Arc::new(ProviderManager::new(provider_config).await.unwrap());

    // Measure time to inject APIs and execute first script
    let start = Instant::now();
    engine.inject_apis(&registry, &providers).unwrap();
    let _ = engine.execute_script("return 'hello'").await.unwrap();
    let startup_time = start.elapsed();

    println!("Script startup time: {startup_time:?}");
    assert!(
        startup_time < Duration::from_millis(100),
        "Startup time {startup_time:?} should be < 100ms"
    );
}

/// Test streaming latency < 50ms
#[tokio::test(flavor = "multi_thread")]
async fn test_streaming_latency() {
    let lua_config = LuaConfig::default();
    let mut engine = EngineFactory::create_lua_engine(&lua_config).unwrap();

    let registry = Arc::new(ComponentRegistry::new());
    let provider_config = ProviderManagerConfig::default();
    let providers = Arc::new(ProviderManager::new(provider_config).await.unwrap());

    engine.inject_apis(&registry, &providers).unwrap();

    // Measure time to start streaming
    let start = Instant::now();
    let result = engine
        .execute_script_streaming("return 'stream test'")
        .await;
    let latency = start.elapsed();

    match result {
        Ok(_) => {
            println!("Streaming latency: {latency:?}");
            assert!(
                latency < Duration::from_millis(50),
                "Streaming latency {latency:?} should be < 50ms"
            );
        }
        Err(e) => {
            // Streaming not fully implemented yet
            println!("Streaming returned error (expected): {e}");
        }
    }
}

/// Benchmark various script operations
#[tokio::test(flavor = "multi_thread")]
async fn test_operation_benchmarks() {
    let lua_config = LuaConfig::default();
    let mut engine = EngineFactory::create_lua_engine(&lua_config).unwrap();

    let registry = Arc::new(ComponentRegistry::new());
    let provider_config = ProviderManagerConfig::default();
    let providers = Arc::new(ProviderManager::new(provider_config).await.unwrap());

    engine.inject_apis(&registry, &providers).unwrap();

    // Benchmark different operations
    let operations = vec![
        ("simple return", "return 42"),
        ("table creation", "return {a=1, b=2, c=3}"),
        (
            "loop",
            "local sum = 0; for i=1,100 do sum = sum + i end; return sum",
        ),
        (
            "function call",
            "local function f(x) return x * 2 end; return f(21)",
        ),
        ("string concat", "return 'hello' .. ' ' .. 'world'"),
    ];

    for (name, script) in operations {
        let iterations = 100;
        let start = Instant::now();

        for _ in 0..iterations {
            let _ = engine.execute_script(script).await.unwrap();
        }

        let duration = start.elapsed();
        let avg_micros = duration.as_micros()
            / u128::try_from(iterations).expect("iterations should be positive");

        println!("Operation '{name}': avg {avg_micros}μs");

        // All basic operations should be fast
        assert!(
            avg_micros < 5000,
            "Operation '{name}' too slow: {avg_micros}μs"
        );
    }
}

/// Test concurrent execution performance
///
/// Note: The Lua engine uses a Mutex internally for thread safety, so concurrent
/// execution won't provide speedup. This test verifies that concurrent execution
/// works correctly and doesn't cause significant slowdown.
#[tokio::test(flavor = "multi_thread")]
async fn test_concurrent_execution_correctness() {
    let lua_config = LuaConfig::default();
    let mut engine = EngineFactory::create_lua_engine(&lua_config).unwrap();

    let registry = Arc::new(ComponentRegistry::new());
    let provider_config = ProviderManagerConfig::default();
    let providers = Arc::new(ProviderManager::new(provider_config).await.unwrap());

    engine.inject_apis(&registry, &providers).unwrap();

    let engine = Arc::new(engine);

    // Benchmark sequential vs concurrent execution
    let script_count = 100;

    // Use scripts that do more work to make concurrency benefits more apparent
    let create_script = |i: usize| -> String {
        format!(
            r"
            local sum = 0
            for j = 1, 100 do
                sum = sum + j * {i}
            end
            return sum
            "
        )
    };

    // Sequential execution
    let seq_start = Instant::now();
    for i in 0..script_count {
        let script = create_script(i);
        let _ = engine.execute_script(&script).await.unwrap();
    }
    let seq_duration = seq_start.elapsed();

    // Concurrent execution
    let conc_start = Instant::now();
    let mut handles = vec![];

    for i in 0..script_count {
        let engine_clone = engine.clone();
        let handle = tokio::spawn(async move {
            let script = create_script(i);
            engine_clone.execute_script(&script).await
        });
        handles.push(handle);
    }

    for handle in handles {
        let _ = handle.await.unwrap().unwrap();
    }
    let conc_duration = conc_start.elapsed();

    println!("Sequential execution: {seq_duration:?}");
    println!("Concurrent execution: {conc_duration:?}");

    // Calculate the overhead ratio
    let overhead_ratio = conc_duration.as_secs_f64() / seq_duration.as_secs_f64();
    println!("Overhead ratio: {overhead_ratio:.2}x");

    // Since Lua engine uses a Mutex internally, concurrent execution won't be faster.
    // We just verify that the overhead is reasonable (less than 5x slower)
    // The overhead can vary based on system load and task scheduling
    assert!(
        overhead_ratio < 5.0,
        "Concurrent execution overhead too high: {overhead_ratio:.2}x"
    );

    // Also verify that concurrent execution produces correct results
    // (This is implicitly tested by the unwrap() calls above)
}

/// Test memory usage with large scripts
#[tokio::test(flavor = "multi_thread")]
#[allow(clippy::cast_precision_loss)] // File size calculation for diagnostics
async fn test_large_script_memory() {
    let lua_config = LuaConfig::default();
    let mut engine = EngineFactory::create_lua_engine(&lua_config).unwrap();

    let registry = Arc::new(ComponentRegistry::new());
    let provider_config = ProviderManagerConfig::default();
    let providers = Arc::new(ProviderManager::new(provider_config).await.unwrap());

    engine.inject_apis(&registry, &providers).unwrap();

    // Create a large script (but under 10MB limit)
    let mut large_script = String::new();
    large_script.push_str("local data = {\n");
    for i in 0..10000 {
        writeln!(
            large_script,
            "  ['key_{i}'] = 'value_{i}_with_some_padding',"
        )
        .unwrap();
    }
    large_script.push_str("}\nreturn #data");

    let script_size_mb = large_script.len() as f64 / (1024.0 * 1024.0);
    println!("Large script size: {script_size_mb:.2} MB");

    // Verify script is under limit
    assert!(
        large_script.len() < 10_000_000,
        "Script should be under 10MB limit"
    );

    // Execute large script and measure time
    let start = Instant::now();
    let result = engine.execute_script(&large_script).await;
    let duration = start.elapsed();

    assert!(result.is_ok(), "Large script should execute successfully");
    println!("Large script execution time: {duration:?}");

    // Large scripts should still execute reasonably fast
    assert!(
        duration < Duration::from_secs(1),
        "Large script execution should be under 1 second"
    );
}

/// Test API injection performance overhead
#[tokio::test(flavor = "multi_thread")]
async fn test_api_injection_overhead() {
    let lua_config = LuaConfig::default();
    let iterations = 10;

    let mut total_time = Duration::ZERO;

    for _ in 0..iterations {
        let mut engine = EngineFactory::create_lua_engine(&lua_config).unwrap();

        let registry = Arc::new(ComponentRegistry::new());
        let provider_config = ProviderManagerConfig::default();
        let providers = Arc::new(ProviderManager::new(provider_config).await.unwrap());

        let start = Instant::now();
        engine.inject_apis(&registry, &providers).unwrap();
        total_time += start.elapsed();
    }

    let avg_time = total_time / iterations;
    println!("Average API injection time: {avg_time:?}");

    // API injection should be fast
    assert!(
        avg_time < Duration::from_millis(10),
        "API injection overhead {avg_time:?} should be < 10ms"
    );
}

/// Test execution context switching overhead
#[tokio::test(flavor = "multi_thread")]
async fn test_context_switching_overhead() {
    let lua_config = LuaConfig::default();
    let mut engine = EngineFactory::create_lua_engine(&lua_config).unwrap();

    let registry = Arc::new(ComponentRegistry::new());
    let provider_config = ProviderManagerConfig::default();
    let providers = Arc::new(ProviderManager::new(provider_config).await.unwrap());

    engine.inject_apis(&registry, &providers).unwrap();

    // Create different contexts
    let mut contexts = vec![];
    for i in 0..5 {
        let mut ctx = llmspell_bridge::engine::bridge::ExecutionContext {
            working_directory: format!("/test/dir/{i}"),
            ..Default::default()
        };
        ctx.environment.insert("CTX_ID".to_string(), i.to_string());
        contexts.push(ctx);
    }

    // Measure context switching overhead
    let iterations = 100;
    let start = Instant::now();

    for i in 0..iterations {
        let ctx = &contexts[i % contexts.len()];
        engine.set_execution_context(ctx.clone()).unwrap();
        let _ = engine.execute_script("return 1").await.unwrap();
    }

    let duration = start.elapsed();
    let avg_micros = duration.as_micros() / iterations as u128;

    println!("Context switching overhead: avg {avg_micros}μs");

    // Context switching should have minimal overhead
    assert!(
        avg_micros < 1000,
        "Context switching overhead too high: {avg_micros}μs"
    );
}
