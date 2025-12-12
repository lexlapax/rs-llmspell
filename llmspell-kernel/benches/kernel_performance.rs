//! ABOUTME: Performance benchmarks for Phase 10 kernel operations
//! ABOUTME: Validates targets: <2s cold start, <5ms message handling, <10ms tool invocation
//!
//! Performance Targets (from Phase 10 Success Criteria):
//! - Kernel startup (embedded mode): <2s cold start, <100ms warm start
//! - Message handling (InProcess transport): <5ms request→reply roundtrip
//! - Tool invocation (ComponentRegistry): <10ms for simple tools (calculator)
//! - Memory overhead: <50MB baseline, <100MB with tools loaded

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use llmspell_bridge::ScriptRuntime;
use llmspell_config::LLMSpellConfig;
use llmspell_kernel::api::{start_embedded_kernel_with_executor, KernelExecutionMode};
use serde_json::json;
use std::sync::Arc;
use std::time::Duration;
use tokio::runtime::Runtime;
use tokio::sync::Mutex;

/// Helper to create a runtime for async benchmarks (multi-threaded)
fn create_bench_runtime() -> Runtime {
    Runtime::new().expect("Failed to create tokio runtime")
}

/// Benchmark kernel startup performance
/// Target: <2s cold start (includes ScriptRuntime + Kernel creation)
fn bench_kernel_startup(c: &mut Criterion) {
    let mut group = c.benchmark_group("kernel_startup");

    // Set longer measurement time for slow operations
    group.measurement_time(Duration::from_secs(10));
    group.sample_size(10); // Fewer samples for slow operations

    // Cold start: create fresh runtime + kernel from scratch
    group.bench_function("embedded_cold_start", |b| {
        b.to_async(create_bench_runtime()).iter(|| async {
            let config = LLMSpellConfig::default();
            let runtime = ScriptRuntime::new(config.clone())
                .await
                .expect("Failed to create runtime");
            let executor = Arc::new(runtime);
            let kernel_handle = start_embedded_kernel_with_executor(
                config,
                executor,
                KernelExecutionMode::Transport,
            )
            .await
            .expect("Failed to start kernel");
            black_box(kernel_handle)
        });
    });

    group.finish();
}

/// Benchmark message handling performance via InProcess transport
/// Target: <5ms for tool_request → tool_reply roundtrip
fn bench_message_handling(c: &mut Criterion) {
    let mut group = c.benchmark_group("message_handling");
    group.sample_size(50); // Good sample size for fast operations

    // Create kernel once for all iterations (warm kernel)
    let rt = create_bench_runtime();
    let kernel_handle = rt.block_on(async {
        let config = LLMSpellConfig::default();
        let runtime = ScriptRuntime::new(config.clone())
            .await
            .expect("Failed to create runtime");
        let executor = Arc::new(runtime);
        start_embedded_kernel_with_executor(config, executor, KernelExecutionMode::Transport)
            .await
            .expect("Failed to start kernel")
    });
    let shared_handle = Arc::new(Mutex::new(kernel_handle));

    // InProcess transport roundtrip: tool_request for list → tool_reply
    group.bench_function("inprocess_tool_list_roundtrip", |b| {
        b.to_async(create_bench_runtime()).iter(|| {
            let handle = shared_handle.clone();
            async move {
                let request = json!({
                    "command": "list",
                });
                let response = handle
                    .lock()
                    .await
                    .send_tool_request(request)
                    .await
                    .expect("Failed to send tool request");
                black_box(response)
            }
        });
    });

    group.finish();
}

/// Benchmark tool invocation performance
/// Target: <10ms for calculator (registry lookup + execution + reply)
fn bench_tool_invocation(c: &mut Criterion) {
    let mut group = c.benchmark_group("tool_invocation");
    group.sample_size(50);

    // Create kernel once with all tools registered
    let rt = create_bench_runtime();
    let kernel_handle = rt.block_on(async {
        let config = LLMSpellConfig::default();
        let runtime = ScriptRuntime::new(config.clone())
            .await
            .expect("Failed to create runtime");
        let executor = Arc::new(runtime);
        start_embedded_kernel_with_executor(config, executor, KernelExecutionMode::Transport)
            .await
            .expect("Failed to start kernel")
    });
    let shared_handle = Arc::new(Mutex::new(kernel_handle));

    // Calculator tool: registry lookup + execute simple expression
    group.bench_function("calculator_simple_expression", |b| {
        b.to_async(create_bench_runtime()).iter(|| {
            let handle = shared_handle.clone();
            async move {
                let request = json!({
                    "command": "invoke",
                    "name": "calculator",
                    "params": {
                        "expression": "2 + 2"
                    }
                });
                let response = handle
                    .lock()
                    .await
                    .send_tool_request(request)
                    .await
                    .expect("Failed to invoke calculator");
                black_box(response)
            }
        });
    });

    // Calculator tool: more complex expression
    group.bench_function("calculator_complex_expression", |b| {
        b.to_async(create_bench_runtime()).iter(|| {
            let handle = shared_handle.clone();
            async move {
                let request = json!({
                    "command": "invoke",
                    "name": "calculator",
                    "params": {
                        "expression": "(10 + 5) * 3 - 8 / 2"
                    }
                });
                let response = handle
                    .lock()
                    .await
                    .send_tool_request(request)
                    .await
                    .expect("Failed to invoke calculator");
                black_box(response)
            }
        });
    });

    // Tool info lookup: registry metadata retrieval
    group.bench_function("tool_info_lookup", |b| {
        b.to_async(create_bench_runtime()).iter(|| {
            let handle = shared_handle.clone();
            async move {
                let request = json!({
                    "command": "info",
                    "name": "calculator"
                });
                let response = handle
                    .lock()
                    .await
                    .send_tool_request(request)
                    .await
                    .expect("Failed to get tool info");
                black_box(response)
            }
        });
    });

    // Tool search: registry filtering
    group.bench_function("tool_search", |b| {
        b.to_async(create_bench_runtime()).iter(|| {
            let handle = shared_handle.clone();
            async move {
                let request = json!({
                    "command": "search",
                    "query": "calc"
                });
                let response = handle
                    .lock()
                    .await
                    .send_tool_request(request)
                    .await
                    .expect("Failed to search tools");
                black_box(response)
            }
        });
    });

    group.finish();
}

/// Benchmark tool registry operations
/// Target: <1ms for registry lookup (HashMap get)
fn bench_registry_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("registry_operations");
    group.sample_size(100);

    // Create kernel once
    let rt = create_bench_runtime();
    let kernel_handle = rt.block_on(async {
        let config = LLMSpellConfig::default();
        let runtime = ScriptRuntime::new(config.clone())
            .await
            .expect("Failed to create runtime");
        let executor = Arc::new(runtime);
        start_embedded_kernel_with_executor(config, executor, KernelExecutionMode::Transport)
            .await
            .expect("Failed to start kernel")
    });
    let shared_handle = Arc::new(Mutex::new(kernel_handle));

    // List all tools: registry iteration
    group.bench_function("list_all_tools", |b| {
        b.to_async(create_bench_runtime()).iter(|| {
            let handle = shared_handle.clone();
            async move {
                let request = json!({
                    "command": "list",
                });
                let response = handle
                    .lock()
                    .await
                    .send_tool_request(request)
                    .await
                    .expect("Failed to list tools");
                black_box(response)
            }
        });
    });

    // List tools with category filter
    group.bench_function("list_tools_filtered", |b| {
        b.to_async(create_bench_runtime()).iter(|| {
            let handle = shared_handle.clone();
            async move {
                let request = json!({
                    "command": "list",
                    "category": "utility"
                });
                let response = handle
                    .lock()
                    .await
                    .send_tool_request(request)
                    .await
                    .expect("Failed to list filtered tools");
                black_box(response)
            }
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_kernel_startup,
    bench_message_handling,
    bench_tool_invocation,
    bench_registry_operations
);
criterion_main!(benches);
