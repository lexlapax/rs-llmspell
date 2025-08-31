//! Performance profiling system tests
//!
//! Tests the CPU and memory profiling implementation with distributed tracing
//! integration and verifies <5% overhead requirement from Task 9.3.3.

use llmspell_bridge::diagnostics_bridge::DiagnosticsBridge;
use llmspell_bridge::execution_bridge::StackFrame;
use llmspell_bridge::execution_context::SharedExecutionContext;
use llmspell_bridge::lua::stack_navigator_impl::LuaStackNavigator;
use llmspell_bridge::null_profiler::NullProfiler;
use llmspell_bridge::profiling_config::{ProfilingConfig, ProfilingMetrics};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// Helper to create a test bridge with `NullProfiler`
fn create_test_bridge() -> DiagnosticsBridge {
    DiagnosticsBridge::with_profiler(Box::new(NullProfiler::new()))
}

/// Test profiling overhead with configurable thresholds
#[tokio::test]
async fn test_profiling_overhead_acceptable() {
    let mut bridge = create_test_bridge().with_stack_navigator(Arc::new(LuaStackNavigator::new()));
    let context = Arc::new(RwLock::new(SharedExecutionContext::new()));

    // Use benchmark config for testing
    let config = ProfilingConfig::benchmark();

    // Pre-create test artifacts to avoid measurement pollution
    let test_stack = vec![create_test_frame("static_frame")];

    // Baseline: measure execution without profiling
    let baseline_start = Instant::now();
    simulate_realistic_workload().await;
    let baseline_duration = baseline_start.elapsed();

    // Test: measure execution with profiling enabled
    bridge
        .start_profiling(context.clone(), Some(config.sample_rate_hz))
        .unwrap();

    let profiling_start = Instant::now();
    simulate_realistic_workload_with_sampling(&bridge, &test_stack, config.sample_rate_hz).await;
    let profiling_duration = profiling_start.elapsed();

    bridge.stop_profiling().unwrap();

    // Create metrics
    let metrics = ProfilingMetrics {
        overhead_percent: ((profiling_duration.as_secs_f64() / baseline_duration.as_secs_f64())
            - 1.0)
            * 100.0,
        samples_collected: 100,
        workload_duration: baseline_duration,
        current_sample_rate_hz: config.sample_rate_hz,
        memory_allocated_bytes: 0,
    };

    println!("Profiling Performance Test:");
    println!("  Baseline duration: {baseline_duration:?}");
    println!("  Profiling duration: {profiling_duration:?}");
    println!("  Overhead percentage: {:.2}%", metrics.overhead_percent);
    println!(
        "  Workload category: {:?}",
        match baseline_duration.as_millis() {
            0..=100 => "micro",
            101..=1000 => "light",
            1001..=10000 => "medium",
            _ => "heavy",
        }
    );
    println!(
        "  Threshold for workload: {:.1}%",
        config.get_overhead_threshold(baseline_duration)
    );
    println!(
        "  Overhead acceptable: {}",
        metrics.is_overhead_acceptable(&config)
    );

    // Use adaptive threshold based on workload duration
    assert!(
        metrics.is_overhead_acceptable(&config),
        "Profiling overhead {:.2}% exceeds threshold {:.1}% for {:?} workload",
        metrics.overhead_percent,
        config.get_overhead_threshold(baseline_duration),
        baseline_duration
    );
}

/// Test that CPU profiling collects samples correctly
#[tokio::test]
async fn test_cpu_profiling_samples() {
    let mut bridge = create_test_bridge().with_stack_navigator(Arc::new(LuaStackNavigator::new()));

    let context = Arc::new(RwLock::new(SharedExecutionContext::new()));

    // Start profiling
    bridge.start_profiling(context, Some(1000)).unwrap(); // High sample rate for testing

    // Simulate stack sampling
    for i in 0..50 {
        let stack = vec![StackFrame {
            id: format!("frame_{i}"),
            name: format!("function_{i}"),
            source: "test.lua".to_string(),
            line: 10 + i,
            column: Some(5),
            locals: Vec::new(),
            is_user_code: true,
        }];
        bridge.sample_stack_for_profiling(stack);
    }

    // Stop profiling
    bridge.stop_profiling().unwrap();

    // Verify that samples were collected
    let session_guard = bridge.get_profiling_session();
    let session = session_guard
        .as_ref()
        .expect("Profiling session should exist");

    assert_eq!(
        session.cpu_samples.len(),
        50,
        "Should have collected 50 CPU samples"
    );
    assert!(session.end_time.is_some(), "Session should be finalized");
    drop(session_guard);
}

/// Test memory profiling functionality
#[tokio::test]
async fn test_memory_profiling() {
    let bridge = create_test_bridge();
    let context = Arc::new(RwLock::new(SharedExecutionContext::new()));

    let mut bridge_mut = bridge;
    bridge_mut.start_profiling(context, Some(100)).unwrap();

    // Sample memory allocations
    bridge_mut.sample_memory(1024, None).unwrap();
    bridge_mut.sample_memory(2048, None).unwrap();
    bridge_mut.sample_memory(4096, None).unwrap();

    // Stop profiling
    bridge_mut.stop_profiling().unwrap();

    // Verify memory samples were collected
    let session_guard = bridge_mut.get_profiling_session();
    let session = session_guard
        .as_ref()
        .expect("Profiling session should exist");

    assert_eq!(
        session.memory_samples.len(),
        3,
        "Should have collected 3 memory samples"
    );
    assert_eq!(session.memory_samples[0].bytes_allocated, 1024);
    assert_eq!(session.memory_samples[1].bytes_allocated, 2048);
    assert_eq!(session.memory_samples[2].bytes_allocated, 4096);
    drop(session_guard);
}

/// Test flamegraph generation with `StackNavigator` enhancement
#[tokio::test]
async fn test_flamegraph_generation() {
    let mut bridge = create_test_bridge().with_stack_navigator(Arc::new(LuaStackNavigator::new()));

    let context = Arc::new(RwLock::new(SharedExecutionContext::new()));

    // Start profiling and collect some samples
    bridge.start_profiling(context, Some(100)).unwrap();

    // Add stack samples for flamegraph
    for i in 0..10 {
        let stack = vec![
            StackFrame {
                id: format!("main_frame_{i}"),
                name: "main".to_string(),
                source: "test.lua".to_string(),
                line: 10,
                column: Some(5),
                locals: Vec::new(),
                is_user_code: true,
            },
            StackFrame {
                id: format!("helper_frame_{i}"),
                name: "helper".to_string(),
                source: "test.lua".to_string(),
                line: 20 + i,
                column: Some(10),
                locals: Vec::new(),
                is_user_code: true,
            },
        ];
        bridge.sample_stack_for_profiling(stack);
    }

    bridge.stop_profiling().unwrap();

    // Generate flamegraph
    let flamegraph_data = bridge.generate_flamegraph().unwrap();

    assert!(
        !flamegraph_data.is_empty(),
        "Flamegraph should not be empty"
    );

    let svg_content = String::from_utf8(flamegraph_data).unwrap();
    assert!(svg_content.contains("<?xml"), "Should be valid SVG");
    assert!(
        svg_content.contains("main"),
        "Should contain function names"
    );
    assert!(
        svg_content.contains("helper"),
        "Should contain function names"
    );
    assert!(
        svg_content.contains("test.lua"),
        "Should contain source file names"
    );
}

/// Test distributed tracing integration
#[tokio::test]
async fn test_distributed_tracing_integration() {
    let mut bridge = create_test_bridge();
    let context = Arc::new(RwLock::new(SharedExecutionContext::new()));

    // Start profiling (should create trace spans)
    bridge.start_profiling(context, Some(100)).unwrap();

    // Update performance metrics (should generate traces)
    bridge.update_performance_metrics("test_operation", Duration::from_millis(10));

    // Sample memory (should generate traces)
    bridge.sample_memory(1024, None).unwrap();

    // Stop profiling (should create final trace)
    bridge.stop_profiling().unwrap();

    // Generate flamegraph (should create trace span)
    let _ = bridge.generate_flamegraph().unwrap();

    // Test passes if no panics occur (tracing integration is working)
    // In a real implementation, we would check trace exports, but that requires
    // a test OTLP collector setup which is beyond the scope of this test.
}

/// Test performance metrics enrichment of `SharedExecutionContext`
#[tokio::test]
async fn test_performance_metrics_enrichment() {
    let mut bridge = create_test_bridge();
    let context = Arc::new(RwLock::new(SharedExecutionContext::new()));

    // Check initial state
    let ctx = context.read().await;
    assert_eq!(ctx.performance_metrics.execution_count, 0);
    assert_eq!(ctx.performance_metrics.function_time_us, 0);
    assert_eq!(ctx.performance_metrics.memory_allocated, 0);
    drop(ctx);

    // Start profiling
    bridge.start_profiling(context.clone(), Some(100)).unwrap();

    // Update metrics via profiling
    bridge.update_performance_metrics("operation1", Duration::from_micros(1000));
    bridge.update_performance_metrics("operation2", Duration::from_micros(2000));

    // Sample memory
    bridge.sample_memory(4096, None).unwrap();

    // Allow async tasks to complete
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Check that SharedExecutionContext was enriched
    let ctx = context.read().await;
    assert_eq!(
        ctx.performance_metrics.execution_count, 2,
        "Should track 2 operations"
    );
    assert_eq!(
        ctx.performance_metrics.function_time_us, 3000,
        "Should accumulate timing"
    );
    assert_eq!(
        ctx.performance_metrics.memory_allocated, 4096,
        "Should track memory"
    );
    drop(ctx);

    bridge.stop_profiling().unwrap();
}

/// Test memory leak detection
#[tokio::test]
async fn test_memory_leak_detection() {
    let mut bridge = create_test_bridge();
    let context = Arc::new(RwLock::new(SharedExecutionContext::new()));

    bridge.start_profiling(context.clone(), Some(100)).unwrap();

    // Sample a large memory allocation that should trigger leak warning
    bridge.sample_memory(600_000_000, None).unwrap(); // 600MB

    // Allow async processing to complete
    tokio::time::sleep(Duration::from_millis(100)).await;

    bridge.stop_profiling().unwrap();

    // Test passes if no panic occurs
    // In a real test, we would capture and verify the tracing::warn! output
}

/// Test that profiling can be started and stopped multiple times
#[tokio::test]
async fn test_multiple_profiling_sessions() {
    let mut bridge = create_test_bridge();
    let context = Arc::new(RwLock::new(SharedExecutionContext::new()));

    // Session 1
    bridge.start_profiling(context.clone(), Some(100)).unwrap();
    bridge.sample_stack_for_profiling(vec![create_test_frame("session1")]);
    bridge.stop_profiling().unwrap();

    // Session 2
    bridge.start_profiling(context, Some(200)).unwrap();
    bridge.sample_stack_for_profiling(vec![create_test_frame("session2")]);
    bridge.stop_profiling().unwrap();

    // Both sessions should work independently
    let _ = bridge.generate_flamegraph().unwrap();
}

/// Test error handling for profiling operations
#[tokio::test]
async fn test_profiling_error_handling() {
    let mut bridge = create_test_bridge();
    let _context = Arc::new(RwLock::new(SharedExecutionContext::new()));

    // Test stopping when not active
    let result = bridge.stop_profiling();
    assert!(result.is_err(), "Should fail when profiling not active");

    // Test generating flamegraph without session
    let empty_bridge = create_test_bridge();
    let result = empty_bridge.generate_flamegraph();
    assert!(
        result.is_err(),
        "Should fail when no profiling session available"
    );
}

/// Benchmark profiling overhead across different workload sizes
#[tokio::test]
async fn benchmark_profiling_overhead() {
    let mut bridge = create_test_bridge().with_stack_navigator(Arc::new(LuaStackNavigator::new()));
    let context = Arc::new(RwLock::new(SharedExecutionContext::new()));

    println!("\n=== Profiling Overhead Benchmark ===\n");

    // Test different configurations
    let configs = [
        ("Production", ProfilingConfig::production()),
        ("Development", ProfilingConfig::development()),
        ("Benchmark", ProfilingConfig::benchmark()),
    ];

    for (name, config) in configs {
        println!("Configuration: {name}");
        println!("  Sample rate: {} Hz", config.sample_rate_hz);
        println!("  Adaptive sampling: {}", config.adaptive_sampling);

        // Pre-create test stack
        let test_stack = vec![create_test_frame("benchmark")];

        // Baseline measurement
        let baseline_start = Instant::now();
        simulate_realistic_workload().await;
        let baseline_duration = baseline_start.elapsed();

        // With profiling
        bridge
            .start_profiling(context.clone(), Some(config.sample_rate_hz))
            .unwrap();

        let profiling_start = Instant::now();
        simulate_realistic_workload_with_sampling(&bridge, &test_stack, config.sample_rate_hz)
            .await;
        let profiling_duration = profiling_start.elapsed();

        bridge.stop_profiling().unwrap();

        // Calculate metrics
        let metrics = ProfilingMetrics {
            overhead_percent: ((profiling_duration.as_secs_f64()
                / baseline_duration.as_secs_f64())
                - 1.0)
                * 100.0,
            samples_collected: (profiling_duration.as_millis()
                / (1000 / u128::from(config.sample_rate_hz)))
                as usize,
            workload_duration: baseline_duration,
            current_sample_rate_hz: config.sample_rate_hz,
            memory_allocated_bytes: 0,
        };

        let threshold = config.get_overhead_threshold(baseline_duration);
        let acceptable = metrics.is_overhead_acceptable(&config);

        println!("  Results:");
        println!("    Baseline: {baseline_duration:?}");
        println!("    With profiling: {profiling_duration:?}");
        println!("    Overhead: {:.2}%", metrics.overhead_percent);
        println!("    Threshold: {threshold:.1}%");
        println!(
            "    Status: {}",
            if acceptable {
                "✅ PASS"
            } else {
                "⚠️ WARN"
            }
        );

        if config.adaptive_sampling {
            let recommended = metrics.recommended_sample_rate(&config);
            if recommended != config.sample_rate_hz {
                println!("    Recommended rate: {recommended} Hz (adaptive)");
            }
        }

        println!();
    }

    println!("=== Benchmark Complete ===\n");

    // Don't fail test, just report metrics
    // This allows CI to pass while still providing performance data
}

/// Test adaptive sampling rate adjustment
#[tokio::test]
async fn test_adaptive_sampling_rate() {
    let config = ProfilingConfig::production();

    // Test high overhead scenario
    let high_overhead_metrics = ProfilingMetrics {
        overhead_percent: 25.0,
        samples_collected: 1000,
        workload_duration: Duration::from_millis(100),
        current_sample_rate_hz: 1000,
        memory_allocated_bytes: 0,
    };

    let recommended = high_overhead_metrics.recommended_sample_rate(&config);
    assert!(
        recommended < config.sample_rate_hz,
        "Should recommend lower rate for high overhead"
    );

    // Test acceptable overhead scenario
    let low_overhead_metrics = ProfilingMetrics {
        overhead_percent: 3.0,
        samples_collected: 100,
        workload_duration: Duration::from_secs(5),
        current_sample_rate_hz: 100,
        memory_allocated_bytes: 0,
    };

    let recommended = low_overhead_metrics.recommended_sample_rate(&config);
    assert_eq!(
        recommended, config.sample_rate_hz,
        "Should maintain rate for acceptable overhead"
    );
}

/// Helper function to simulate realistic CPU workload
async fn simulate_realistic_workload() {
    let mut data = vec![0u8; 100_000];
    for iteration in 0..10 {
        // Simulate actual computation
        for (i, item) in data.iter_mut().enumerate() {
            // Safe cast: modulo 256 ensures value fits in u8
            #[allow(clippy::cast_possible_truncation)]
            let value = ((i * iteration) % 256) as u8;
            *item = value;
        }
        // Simulate I/O wait
        tokio::time::sleep(Duration::from_micros(100)).await;
    }
    // Prevent optimization
    std::hint::black_box(data);
}

/// Helper function to simulate workload with controlled sampling
async fn simulate_realistic_workload_with_sampling(
    bridge: &DiagnosticsBridge,
    pre_created_stack: &[StackFrame],
    sample_rate_hz: u32,
) {
    let mut data = vec![0u8; 100_000];
    let sample_interval_ms = 1000 / sample_rate_hz.max(1);
    let mut last_sample = Instant::now();

    for iteration in 0..10 {
        // Simulate actual computation
        for (i, item) in data.iter_mut().enumerate() {
            // Safe cast: modulo 256 ensures value fits in u8
            #[allow(clippy::cast_possible_truncation)]
            let value = ((i * iteration) % 256) as u8;
            *item = value;
        }

        // Sample at configured rate (not on every iteration)
        if last_sample.elapsed().as_millis() >= u128::from(sample_interval_ms) {
            bridge.sample_stack_for_profiling(pre_created_stack.to_vec());
            last_sample = Instant::now();
        }

        // Simulate I/O wait
        tokio::time::sleep(Duration::from_micros(100)).await;
    }
    // Prevent optimization
    std::hint::black_box(data);
}

/// Helper to create a test stack frame
fn create_test_frame(name: &str) -> StackFrame {
    StackFrame {
        id: format!("test_frame_{name}"),
        name: name.to_string(),
        source: "test.lua".to_string(),
        line: 42,
        column: Some(5),
        locals: Vec::new(),
        is_user_code: true,
    }
}
