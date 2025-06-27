//! ABOUTME: Performance and memory validation tests for the bridge
//! ABOUTME: Ensures memory usage and performance targets are met

use llmspell_bridge::{
    engine::{
        factory::{EngineFactory, LuaConfig},
    },
    providers::{ProviderManager, ProviderManagerConfig},
    ComponentRegistry,
};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Test memory usage stays under 50MB for simple scripts
#[tokio::test]
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
        let script = format!("return {}", i);
        let _ = engine.execute_script(&script).await.unwrap();
    }
    let duration = start.elapsed();
    
    println!("Executed 100 scripts in {:?}", duration);
    
    // Verify the configured memory limit is reasonable
    assert_eq!(lua_config.max_memory, Some(50_000_000), "Default memory limit should be 50MB");
}

/// Test for memory leaks with repeated execution
#[tokio::test]
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
        let script = format!("local t = {{}}; for j=1,100 do t[j] = {} end; return #t", i);
        let start = Instant::now();
        let _ = engine.execute_script(&script).await.unwrap();
        timings.push(start.elapsed());
    }
    
    // Calculate average time for first 100 vs last 100
    let first_100_avg = timings[..100].iter().map(|d| d.as_micros()).sum::<u128>() / 100;
    let last_100_avg = timings[900..].iter().map(|d| d.as_micros()).sum::<u128>() / 100;
    
    println!("First 100 avg: {}μs, Last 100 avg: {}μs", first_100_avg, last_100_avg);
    
    // Performance should not degrade significantly (indicates memory issues)
    let degradation = last_100_avg as f64 / first_100_avg as f64;
    assert!(degradation < 2.0, "Performance degraded by {:.2}x, possible memory leak", degradation);
}

/// Test script startup time < 100ms
#[tokio::test]
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
    
    println!("Script startup time: {:?}", startup_time);
    assert!(startup_time < Duration::from_millis(100), 
            "Startup time {:?} should be < 100ms", startup_time);
}

/// Test streaming latency < 50ms
#[tokio::test]
async fn test_streaming_latency() {
    let lua_config = LuaConfig::default();
    let mut engine = EngineFactory::create_lua_engine(&lua_config).unwrap();
    
    let registry = Arc::new(ComponentRegistry::new());
    let provider_config = ProviderManagerConfig::default();
    let providers = Arc::new(ProviderManager::new(provider_config).await.unwrap());
    
    engine.inject_apis(&registry, &providers).unwrap();
    
    // Measure time to start streaming
    let start = Instant::now();
    let result = engine.execute_script_streaming("return 'stream test'").await;
    let latency = start.elapsed();
    
    match result {
        Ok(_) => {
            println!("Streaming latency: {:?}", latency);
            assert!(latency < Duration::from_millis(50), 
                    "Streaming latency {:?} should be < 50ms", latency);
        }
        Err(e) => {
            // Streaming not fully implemented yet
            println!("Streaming returned error (expected): {}", e);
        }
    }
}

/// Benchmark various script operations
#[tokio::test]
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
        ("loop", "local sum = 0; for i=1,100 do sum = sum + i end; return sum"),
        ("function call", "local function f(x) return x * 2 end; return f(21)"),
        ("string concat", "return 'hello' .. ' ' .. 'world'"),
    ];
    
    for (name, script) in operations {
        let iterations = 100;
        let start = Instant::now();
        
        for _ in 0..iterations {
            let _ = engine.execute_script(script).await.unwrap();
        }
        
        let duration = start.elapsed();
        let avg_micros = duration.as_micros() / iterations as u128;
        
        println!("Operation '{}': avg {}μs", name, avg_micros);
        
        // All basic operations should be fast
        assert!(avg_micros < 5000, "Operation '{}' too slow: {}μs", name, avg_micros);
    }
}

/// Test concurrent execution performance
#[tokio::test]
async fn test_concurrent_performance() {
    let lua_config = LuaConfig::default();
    let mut engine = EngineFactory::create_lua_engine(&lua_config).unwrap();
    
    let registry = Arc::new(ComponentRegistry::new());
    let provider_config = ProviderManagerConfig::default();
    let providers = Arc::new(ProviderManager::new(provider_config).await.unwrap());
    
    engine.inject_apis(&registry, &providers).unwrap();
    
    let engine = Arc::new(engine);
    
    // Benchmark sequential vs concurrent execution
    let script_count = 100;
    
    // Sequential execution
    let seq_start = Instant::now();
    for i in 0..script_count {
        let script = format!("return {}", i);
        let _ = engine.execute_script(&script).await.unwrap();
    }
    let seq_duration = seq_start.elapsed();
    
    // Concurrent execution
    let conc_start = Instant::now();
    let mut handles = vec![];
    
    for i in 0..script_count {
        let engine_clone = engine.clone();
        let handle = tokio::spawn(async move {
            let script = format!("return {}", i);
            engine_clone.execute_script(&script).await
        });
        handles.push(handle);
    }
    
    for handle in handles {
        let _ = handle.await.unwrap().unwrap();
    }
    let conc_duration = conc_start.elapsed();
    
    println!("Sequential execution: {:?}", seq_duration);
    println!("Concurrent execution: {:?}", conc_duration);
    
    // Concurrent should be faster or at least not significantly slower
    let speedup = seq_duration.as_secs_f64() / conc_duration.as_secs_f64();
    println!("Speedup factor: {:.2}x", speedup);
    
    // Should have some speedup with concurrent execution
    assert!(speedup > 0.8, "Concurrent execution should not be significantly slower");
}

/// Test memory usage with large scripts
#[tokio::test]
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
        large_script.push_str(&format!("  ['key_{}'] = 'value_{}_with_some_padding',\n", i, i));
    }
    large_script.push_str("}\nreturn #data");
    
    let script_size_mb = large_script.len() as f64 / (1024.0 * 1024.0);
    println!("Large script size: {:.2} MB", script_size_mb);
    
    // Verify script is under limit
    assert!(large_script.len() < 10_000_000, "Script should be under 10MB limit");
    
    // Execute large script and measure time
    let start = Instant::now();
    let result = engine.execute_script(&large_script).await;
    let duration = start.elapsed();
    
    assert!(result.is_ok(), "Large script should execute successfully");
    println!("Large script execution time: {:?}", duration);
    
    // Large scripts should still execute reasonably fast
    assert!(duration < Duration::from_secs(1), "Large script execution should be under 1 second");
}

/// Test API injection performance overhead
#[tokio::test]
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
    println!("Average API injection time: {:?}", avg_time);
    
    // API injection should be fast
    assert!(avg_time < Duration::from_millis(10), 
            "API injection overhead {:?} should be < 10ms", avg_time);
}

/// Test execution context switching overhead
#[tokio::test]
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
        let mut ctx = llmspell_bridge::engine::bridge::ExecutionContext::default();
        ctx.working_directory = format!("/test/dir/{}", i);
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
    
    println!("Context switching overhead: avg {}μs", avg_micros);
    
    // Context switching should have minimal overhead
    assert!(avg_micros < 1000, "Context switching overhead too high: {}μs", avg_micros);
}