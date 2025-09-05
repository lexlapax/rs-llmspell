//! Benchmark comparing in-process kernel vs direct ScriptRuntime execution
//! 
//! Measures overhead introduced by the kernel architecture:
//! - OLD: Direct ScriptRuntime::execute_script()
//! - NEW: InProcessKernel → GenericKernel → ScriptRuntime

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use llmspell_bridge::runtime::ScriptRuntime;
use llmspell_cli::kernel_client::{InProcessKernel, KernelConnectionTrait};
use llmspell_config::LLMSpellConfig;
use std::sync::Arc;
use tokio::runtime::Runtime;

fn create_test_config() -> Arc<LLMSpellConfig> {
    let mut config = LLMSpellConfig::default();
    config.default_engine = "lua".to_string();
    // Disable debug to measure pure execution overhead
    config.debug.enabled = false;
    Arc::new(config)
}

fn benchmark_direct_execution(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let config = create_test_config();
    
    c.bench_function("direct_scriptruntime_simple", |b| {
        let runtime = rt.block_on(async {
            ScriptRuntime::new_with_lua((*config).clone()).await.unwrap()
        });
        let runtime_arc = Arc::new(tokio::sync::RwLock::new(runtime));
        
        b.to_async(&rt).iter(|| {
            let runtime = runtime_arc.clone();
            async move {
                let runtime = runtime.write().await;
                let result = runtime.execute_script(black_box("return 42")).await.unwrap();
                black_box(result);
            }
        });
    });
    
    c.bench_function("direct_scriptruntime_loop", |b| {
        let runtime = rt.block_on(async {
            ScriptRuntime::new_with_lua((*config).clone()).await.unwrap()
        });
        let runtime_arc = Arc::new(tokio::sync::RwLock::new(runtime));
        
        b.to_async(&rt).iter(|| {
            let runtime = runtime_arc.clone();
            async move {
                let runtime = runtime.write().await;
                let result = runtime.execute_script(black_box(
                    "local sum = 0; for i = 1, 100 do sum = sum + i end; return sum"
                )).await.unwrap();
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

fn benchmark_kernel_execution(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let config = create_test_config();
    
    c.bench_function("kernel_inprocess_simple", |b| {
        let kernel = rt.block_on(async {
            InProcessKernel::new(config.clone()).await.unwrap()
        });
        let kernel_arc = Arc::new(tokio::sync::RwLock::new(kernel));
        
        b.to_async(&rt).iter(|| {
            let kernel = kernel_arc.clone();
            async move {
                let mut kernel = kernel.write().await;
                let result = kernel.execute(black_box("return 42")).await.unwrap();
                black_box(result);
            }
        });
    });
    
    c.bench_function("kernel_inprocess_loop", |b| {
        let kernel = rt.block_on(async {
            InProcessKernel::new(config.clone()).await.unwrap()
        });
        let kernel_arc = Arc::new(tokio::sync::RwLock::new(kernel));
        
        b.to_async(&rt).iter(|| {
            let kernel = kernel_arc.clone();
            async move {
                let mut kernel = kernel.write().await;
                let result = kernel.execute(black_box(
                    "local sum = 0; for i = 1, 100 do sum = sum + i end; return sum"
                )).await.unwrap();
                black_box(result);
            }
        });
    });
    
    c.bench_function("kernel_inprocess_function", |b| {
        let kernel = rt.block_on(async {
            InProcessKernel::new(config.clone()).await.unwrap()
        });
        let kernel_arc = Arc::new(tokio::sync::RwLock::new(kernel));
        
        b.to_async(&rt).iter(|| {
            let kernel = kernel_arc.clone();
            async move {
                let mut kernel = kernel.write().await;
                let result = kernel.execute(black_box(
                    "function fib(n) if n <= 1 then return n else return fib(n-1) + fib(n-2) end end; return fib(10)"
                )).await.unwrap();
                black_box(result);
            }
        });
    });
}

fn benchmark_overhead_comparison(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let config = create_test_config();
    
    // Measure initialization overhead
    c.bench_function("init_direct_runtime", |b| {
        b.to_async(&rt).iter(|| {
            let config = config.clone();
            async move {
                let runtime = ScriptRuntime::new_with_lua((*config).clone()).await.unwrap();
                black_box(runtime);
            }
        });
    });
    
    c.bench_function("init_kernel_inprocess", |b| {
        b.to_async(&rt).iter(|| {
            let config = config.clone();
            async move {
                let kernel = InProcessKernel::new(config).await.unwrap();
                black_box(kernel);
            }
        });
    });
    
    // Measure execution overhead with pre-initialized instances
    let mut group = c.benchmark_group("execution_overhead");
    
    // Test with minimal script
    group.bench_function("minimal_direct", |b| {
        let runtime = rt.block_on(async {
            ScriptRuntime::new_with_lua((*config).clone()).await.unwrap()
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
    
    group.bench_function("minimal_kernel", |b| {
        let kernel = rt.block_on(async {
            InProcessKernel::new(config.clone()).await.unwrap()
        });
        let kernel_arc = Arc::new(tokio::sync::RwLock::new(kernel));
        
        b.to_async(&rt).iter(|| {
            let kernel = kernel_arc.clone();
            async move {
                let mut kernel = kernel.write().await;
                let result = kernel.execute("return 1").await.unwrap();
                black_box(result);
            }
        });
    });
    
    group.finish();
}

criterion_group!(
    benches, 
    benchmark_direct_execution,
    benchmark_kernel_execution,
    benchmark_overhead_comparison
);
criterion_main!(benches);