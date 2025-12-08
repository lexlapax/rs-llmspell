//! ABOUTME: End-to-end template infrastructure overhead benchmarks (Task 13.14.1)
//! ABOUTME: Measures template lookup, context assembly, and infrastructure overhead (<2ms target)

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use llmspell_bridge::{ContextBridge, MemoryProvider};
use llmspell_memory::{DefaultMemoryManager, EpisodicEntry, MemoryManager};
use llmspell_templates::{ExecutionContext, TemplateParams, TemplateRegistry};
use serde_json::json;
use std::sync::Arc;
use tokio::runtime::Runtime;
use tracing::info;

/// Benchmark template registry lookup overhead
fn template_lookup_benchmark(c: &mut Criterion) {
    info!("Starting template_lookup benchmark");

    let registry = TemplateRegistry::with_builtin_templates().expect("Failed to load templates");

    c.bench_function("template_lookup", |b| {
        b.iter(|| {
            // Benchmark template lookup overhead
            let _template = registry
                .get(black_box("research-assistant"))
                .expect("Template not found");
        });
    });
}

/// Benchmark execution context creation with memory infrastructure
fn context_creation_benchmark(c: &mut Criterion) {
    info!("Starting context_creation benchmark");

    let rt = Runtime::new().unwrap();
    let memory_manager = rt.block_on(async {
        Arc::new(
            DefaultMemoryManager::new_in_memory()
                .await
                .expect("Failed to create memory manager"),
        )
    });

    c.bench_function("context_creation_with_memory", |b| {
        let mm = memory_manager.clone();
        b.iter(|| {
            let context_bridge =
                Arc::new(ContextBridge::new(MemoryProvider::new_eager(mm.clone())));
            let _context = ExecutionContext::builder()
                .with_memory_manager(mm.clone())
                .with_context_bridge(context_bridge)
                .build();
        });
    });
}

/// Benchmark template parameter parsing overhead
fn param_parsing_benchmark(c: &mut Criterion) {
    info!("Starting param_parsing benchmark");

    c.bench_function("param_parsing", |b| {
        b.iter(|| {
            let mut params = TemplateParams::new();
            params.insert("session_id", json!("bench-session"));
            params.insert("memory_enabled", json!(true));
            params.insert("context_budget", json!(2000));
            params.insert("query", json!("What is Rust?"));

            // Extract parameters (typical template startup cost)
            let _session_id: Option<String> = params.get_optional("session_id").unwrap_or(None);
            let _memory_enabled: bool = params
                .get_optional("memory_enabled")
                .unwrap_or(Some(false))
                .unwrap_or(false);
            let _context_budget: usize = params
                .get_optional("context_budget")
                .unwrap_or(Some(2000))
                .unwrap_or(2000);
            let _query: String = params.get("query").unwrap();

            black_box(_session_id);
        });
    });
}

/// Benchmark end-to-end infrastructure overhead (lookup + context + params)
fn template_infrastructure_overhead_benchmark(c: &mut Criterion) {
    info!("Starting template_infrastructure_overhead benchmark (target: <2ms)");

    let rt = Runtime::new().unwrap();
    let registry = TemplateRegistry::with_builtin_templates().expect("Failed to load templates");
    let memory_manager = rt.block_on(async {
        Arc::new(
            DefaultMemoryManager::new_in_memory()
                .await
                .expect("Failed to create memory manager"),
        )
    });

    c.bench_function("template_infrastructure_overhead", |b| {
        let mm = memory_manager.clone();
        b.iter(|| {
            // 1. Template lookup
            let _template = registry
                .get(black_box("research-assistant"))
                .expect("Template not found");

            // 2. Context creation
            let context_bridge =
                Arc::new(ContextBridge::new(MemoryProvider::new_eager(mm.clone())));
            let _context = ExecutionContext::builder()
                .with_memory_manager(mm.clone())
                .with_context_bridge(context_bridge)
                .build();

            // 3. Parameter parsing
            let mut params = TemplateParams::new();
            params.insert("session_id", json!("bench-session"));
            params.insert("memory_enabled", json!(true));
            params.insert("query", json!("What is Rust?"));

            black_box(params);
        });
    });
}

/// Benchmark template execution with memory-enabled context (infrastructure only)
fn memory_enabled_context_assembly_benchmark(c: &mut Criterion) {
    info!("Starting memory_enabled_context_assembly benchmark");

    let rt = Runtime::new().unwrap();
    let memory_manager = rt.block_on(async {
        let mm = Arc::new(
            DefaultMemoryManager::new_in_memory()
                .await
                .expect("Failed to create memory manager"),
        );

        // Preload some episodic entries
        for i in 0..100 {
            let entry = EpisodicEntry::new(
                "bench-session".to_string(),
                if i % 2 == 0 { "user" } else { "assistant" }.to_string(),
                format!("Context message {} about Rust programming", i),
            );
            mm.episodic().add(entry).await.unwrap();
        }

        mm
    });

    let context_bridge = Arc::new(ContextBridge::new(MemoryProvider::new_eager(
        memory_manager.clone(),
    )));

    c.bench_function("memory_enabled_context_retrieval", |b| {
        let cb = context_bridge.clone();
        b.to_async(&rt).iter(|| {
            let cb = cb.clone();
            async move {
                // Benchmark context retrieval with memory
                let _context = cb
                    .assemble(
                        black_box("What is Rust ownership?"),
                        black_box("episodic"),
                        black_box(1000),
                        Some(black_box("bench-session")),
                    )
                    .await
                    .unwrap();
            }
        });
    });
}

criterion_group!(
    benches,
    template_lookup_benchmark,
    context_creation_benchmark,
    param_parsing_benchmark,
    template_infrastructure_overhead_benchmark,
    memory_enabled_context_assembly_benchmark
);
criterion_main!(benches);
