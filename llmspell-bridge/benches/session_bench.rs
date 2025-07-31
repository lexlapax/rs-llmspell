//! ABOUTME: Performance benchmarks for Session and Artifact operations
//! ABOUTME: Validates <50ms session operations target from Phase 6 requirements

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use llmspell_bridge::lua::engine::LuaEngine;
use llmspell_bridge::runtime::GlobalRuntimeConfig;
use llmspell_bridge::ComponentRegistry;
use std::sync::Arc;
use tokio::runtime::Runtime;

/// Benchmark session creation performance
fn bench_session_creation(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut runtime_config = GlobalRuntimeConfig::default();
    runtime_config.runtime.sessions.enabled = true;
    runtime_config.runtime.sessions.storage_backend = "memory".to_string();
    runtime_config.runtime.sessions.max_sessions = 1000;

    let registry = Arc::new(ComponentRegistry::new());
    let engine = rt.block_on(async { LuaEngine::new(registry, runtime_config).await.unwrap() });

    let lua_code = r#"
        return Session.create({
            name = "benchmark_session",
            description = "Performance test session"
        })
    "#;

    c.bench_function("session_creation", |b| {
        b.to_async(&rt).iter(|| async {
            let mut engine_clone = engine.clone();
            engine_clone.execute(lua_code).await.unwrap()
        });
    });
}

/// Benchmark session save/load performance
fn bench_session_persistence(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut runtime_config = GlobalRuntimeConfig::default();
    runtime_config.runtime.sessions.enabled = true;
    runtime_config.runtime.sessions.storage_backend = "memory".to_string();

    let registry = Arc::new(ComponentRegistry::new());
    let mut engine = rt.block_on(async { LuaEngine::new(registry, runtime_config).await.unwrap() });

    // Pre-create session
    let session_id = rt.block_on(async {
        let create_code = r#"
            return Session.create({
                name = "persistence_benchmark"
            })
        "#;
        engine.execute(create_code).await.unwrap()
    });

    let save_code = format!(
        r#"
        Session.save("{}")
        return true
    "#,
        session_id.as_str().unwrap()
    );

    let load_code = format!(
        r#"
        return Session.load("{}")
    "#,
        session_id.as_str().unwrap()
    );

    c.bench_function("session_save", |b| {
        b.to_async(&rt).iter(|| async {
            let mut engine_clone = engine.clone();
            engine_clone.execute(&save_code).await.unwrap()
        });
    });

    c.bench_function("session_load", |b| {
        b.to_async(&rt).iter(|| async {
            let mut engine_clone = engine.clone();
            engine_clone.execute(&load_code).await.unwrap()
        });
    });
}

/// Benchmark artifact storage performance
fn bench_artifact_operations(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut runtime_config = GlobalRuntimeConfig::default();
    runtime_config.runtime.sessions.enabled = true;
    runtime_config.runtime.sessions.storage_backend = "memory".to_string();

    let registry = Arc::new(ComponentRegistry::new());
    let mut engine = rt.block_on(async { LuaEngine::new(registry, runtime_config).await.unwrap() });

    // Pre-create session
    let session_id = rt.block_on(async {
        let create_code = r#"
            return Session.create({
                name = "artifact_benchmark"
            })
        "#;
        engine.execute(create_code).await.unwrap()
    });

    let store_code = format!(
        r#"
        return Artifact.store(
            "{}",
            "user_input",
            "benchmark_" .. math.random(1000000) .. ".txt",
            "Benchmark content for performance testing",
            {{ category = "benchmark" }}
        )
    "#,
        session_id.as_str().unwrap()
    );

    let list_code = format!(
        r#"
        return Artifact.list("{}")
    "#,
        session_id.as_str().unwrap()
    );

    let query_code = format!(
        r#"
        return Artifact.query({{
            session_id = "{}",
            type = "user_input",
            limit = 10
        }})
    "#,
        session_id.as_str().unwrap()
    );

    c.bench_function("artifact_store", |b| {
        b.to_async(&rt).iter(|| async {
            let mut engine_clone = engine.clone();
            engine_clone.execute(&store_code).await.unwrap()
        });
    });

    c.bench_function("artifact_list", |b| {
        b.to_async(&rt).iter(|| async {
            let mut engine_clone = engine.clone();
            engine_clone.execute(&list_code).await.unwrap()
        });
    });

    c.bench_function("artifact_query", |b| {
        b.to_async(&rt).iter(|| async {
            let mut engine_clone = engine.clone();
            engine_clone.execute(&query_code).await.unwrap()
        });
    });
}

/// Benchmark batch operations for scalability
fn bench_batch_operations(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut runtime_config = GlobalRuntimeConfig::default();
    runtime_config.runtime.sessions.enabled = true;
    runtime_config.runtime.sessions.storage_backend = "memory".to_string();
    runtime_config.runtime.sessions.max_sessions = 1000;

    let registry = Arc::new(ComponentRegistry::new());

    for batch_size in [1, 10, 50, 100].iter() {
        let mut engine = rt.block_on(async {
            LuaEngine::new(registry.clone(), runtime_config.clone())
                .await
                .unwrap()
        });

        let lua_code = format!(
            r#"
            local session_ids = {{}}
            for i = 1, {} do
                local id = Session.create({{
                    name = "batch_session_" .. i,
                    description = "Batch operation test"
                }})
                table.insert(session_ids, id)
            end
            return session_ids
        "#,
            batch_size
        );

        c.bench_with_input(
            BenchmarkId::new("batch_session_creation", batch_size),
            batch_size,
            |b, &_size| {
                b.to_async(&rt).iter(|| async {
                    let mut engine_clone = engine.clone();
                    engine_clone.execute(&lua_code).await.unwrap()
                });
            },
        );
    }
}

criterion_group!(
    benches,
    bench_session_creation,
    bench_session_persistence,
    bench_artifact_operations,
    bench_batch_operations
);
criterion_main!(benches);
