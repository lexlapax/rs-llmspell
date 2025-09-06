//! Benchmark comparing kernel vs direct ScriptRuntime execution
//!
//! Measures overhead introduced by the kernel architecture:
//! - Direct ScriptRuntime::execute_script()
//! - External kernel via ZeroMQ (TODO: implement when ZmqKernelClient is ready)

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use llmspell_bridge::runtime::ScriptRuntime;
// use llmspell_cli::kernel_client::KernelConnectionTrait; // TODO: Re-enable when ZmqKernelClient is ready
use llmspell_config::LLMSpellConfig;
use std::sync::Arc;
use tokio::runtime::Runtime;

fn create_test_config() -> Arc<LLMSpellConfig> {
    Arc::new(LLMSpellConfig {
        default_engine: "lua".to_string(),
        // Disable debug to measure pure execution overhead
        debug: llmspell_config::DebugConfig {
            enabled: false,
            ..Default::default()
        },
        ..LLMSpellConfig::default()
    })
}

fn benchmark_direct_execution(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let config = create_test_config();

    c.bench_function("direct_scriptruntime_simple", |b| {
        let runtime = rt.block_on(async {
            ScriptRuntime::new_with_lua((*config).clone())
                .await
                .unwrap()
        });
        let runtime_arc = Arc::new(tokio::sync::RwLock::new(runtime));

        b.to_async(&rt).iter(|| {
            let runtime = runtime_arc.clone();
            async move {
                let runtime = runtime.write().await;
                let result = runtime
                    .execute_script(black_box("return 42"))
                    .await
                    .unwrap();
                black_box(result);
            }
        });
    });

    c.bench_function("direct_scriptruntime_loop", |b| {
        let runtime = rt.block_on(async {
            ScriptRuntime::new_with_lua((*config).clone())
                .await
                .unwrap()
        });
        let runtime_arc = Arc::new(tokio::sync::RwLock::new(runtime));

        b.to_async(&rt).iter(|| {
            let runtime = runtime_arc.clone();
            async move {
                let runtime = runtime.write().await;
                let result = runtime
                    .execute_script(black_box(
                        "local sum = 0; for i = 1, 100 do sum = sum + i end; return sum",
                    ))
                    .await
                    .unwrap();
                black_box(result);
            }
        });
    });

    c.bench_function("direct_scriptruntime_function", |b| {
        let runtime = rt.block_on(async {
            ScriptRuntime::new_with_lua((*config).clone()).await.unwrap()
        });
        let runtime_arc = Arc::new(tokio::sync::RwLock::new(runtime));

        b.to_async(&rt).iter(|| {
            let runtime = runtime_arc.clone();
            async move {
                let runtime = runtime.write().await;
                let result = runtime.execute_script(black_box(
                    "function fib(n) if n <= 1 then return n else return fib(n-1) + fib(n-2) end end; return fib(10)"
                )).await.unwrap();
                black_box(result);
            }
        });
    });
}

fn benchmark_kernel_execution(_c: &mut Criterion) {
    // TODO: Implement when ZmqKernelClient is ready
    // This will benchmark external kernel communication overhead
    /*
    let rt = Runtime::new().unwrap();
    let config = create_test_config();

    c.bench_function("kernel_external_simple", |b| {
        // Start external kernel process
        // Connect via ZmqKernelClient
        // Benchmark execution
    });
    */
    // TODO: Add more kernel benchmarks
}

fn benchmark_overhead_comparison(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let config = create_test_config();

    // Measure initialization overhead
    c.bench_function("init_direct_runtime", |b| {
        b.to_async(&rt).iter(|| {
            let config = config.clone();
            async move {
                let runtime = ScriptRuntime::new_with_lua((*config).clone())
                    .await
                    .unwrap();
                black_box(runtime);
            }
        });
    });

    // TODO: Add external kernel initialization benchmark when ZmqKernelClient is ready
    /*
    c.bench_function("init_kernel_external", |b| {
        b.to_async(&rt).iter(|| {
            let config = config.clone();
            async move {
                // Start kernel process and connect
            }
        });
    });
    */

    // Measure execution overhead with pre-initialized instances
    let mut group = c.benchmark_group("execution_overhead");

    // Test with minimal script
    group.bench_function("minimal_direct", |b| {
        let runtime = rt.block_on(async {
            ScriptRuntime::new_with_lua((*config).clone())
                .await
                .unwrap()
        });
        let runtime_arc = Arc::new(tokio::sync::RwLock::new(runtime));

        b.to_async(&rt).iter(|| {
            let runtime = runtime_arc.clone();
            async move {
                let runtime = runtime.write().await;
                let result = runtime.execute_script("return 1").await.unwrap();
                black_box(result);
            }
        });
    });

    // TODO: Add external kernel benchmark when ZmqKernelClient is ready
    /*
    group.bench_function("minimal_kernel_external", |b| {
        // Connect to external kernel
        // Benchmark minimal execution
    });
    */

    group.finish();
}

criterion_group!(
    benches,
    benchmark_direct_execution,
    benchmark_kernel_execution,
    benchmark_overhead_comparison
);
criterion_main!(benches);
