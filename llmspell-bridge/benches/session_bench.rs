//! ABOUTME: Performance benchmarks for Session and Artifact operations
//! ABOUTME: Validates <50ms session operations target from Phase 6 requirements

// Benchmark for llmspell-bridge sessions

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use llmspell_bridge::engine::LuaConfig;
use llmspell_bridge::lua::engine::LuaEngine;
use llmspell_bridge::ScriptEngineBridge;
use tokio::runtime::Runtime;

/// Benchmark session creation performance
fn bench_session_creation(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let lua_config = LuaConfig::default();

    let lua_code = r#"
        return Session.create({
            name = "benchmark_session",
            description = "Performance test session"
        })
    "#;

    c.bench_function("session_creation", |b| {
        b.to_async(&rt).iter(|| async {
            let engine = LuaEngine::new(&lua_config).unwrap();
            engine.execute_script(lua_code).await.unwrap()
        });
    });
}

/// Benchmark session save/load performance
fn bench_session_persistence(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let lua_config = LuaConfig::default();

    // Pre-create session
    let session_id = rt.block_on(async {
        let engine = LuaEngine::new(&lua_config).unwrap();
        let create_code = r#"
            return Session.create({
                name = "persistence_benchmark"
            })
        "#;
        engine.execute_script(create_code).await.unwrap()
    });

    let save_code = format!(
        r#"
        Session.save("{}")
        return true
    "#,
        session_id.output.as_str().unwrap()
    );

    let load_code = format!(
        r#"
        return Session.load("{}")
    "#,
        session_id.output.as_str().unwrap()
    );

    c.bench_function("session_save", |b| {
        b.to_async(&rt).iter(|| async {
            let engine = LuaEngine::new(&lua_config).unwrap();
            engine.execute_script(&save_code).await.unwrap()
        });
    });

    c.bench_function("session_load", |b| {
        b.to_async(&rt).iter(|| async {
            let engine = LuaEngine::new(&lua_config).unwrap();
            engine.execute_script(&load_code).await.unwrap()
        });
    });
}

/// Benchmark artifact storage performance
fn bench_artifact_operations(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let lua_config = LuaConfig::default();

    // Pre-create session
    let session_id = rt.block_on(async {
        let engine = LuaEngine::new(&lua_config).unwrap();
        let create_code = r#"
            return Session.create({
                name = "artifact_benchmark"
            })
        "#;
        engine.execute_script(create_code).await.unwrap()
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
        session_id.output.as_str().unwrap()
    );

    let list_code = format!(
        r#"
        return Artifact.list("{}")
    "#,
        session_id.output.as_str().unwrap()
    );

    let query_code = format!(
        r#"
        return Artifact.query({{
            session_id = "{}",
            type = "user_input",
            limit = 10
        }})
    "#,
        session_id.output.as_str().unwrap()
    );

    c.bench_function("artifact_store", |b| {
        b.to_async(&rt).iter(|| async {
            let engine = LuaEngine::new(&lua_config).unwrap();
            engine.execute_script(&store_code).await.unwrap()
        });
    });

    c.bench_function("artifact_list", |b| {
        b.to_async(&rt).iter(|| async {
            let engine = LuaEngine::new(&lua_config).unwrap();
            engine.execute_script(&list_code).await.unwrap()
        });
    });

    c.bench_function("artifact_query", |b| {
        b.to_async(&rt).iter(|| async {
            let engine = LuaEngine::new(&lua_config).unwrap();
            engine.execute_script(&query_code).await.unwrap()
        });
    });
}

/// Benchmark batch operations for scalability
fn bench_batch_operations(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let lua_config = LuaConfig::default();

    for batch_size in &[1, 10, 50, 100] {
        let lua_code = format!(
            r#"
            local session_ids = {{}}
            for i = 1, {batch_size} do
                local id = Session.create({{
                    name = "batch_session_" .. i,
                    description = "Batch operation test"
                }})
                table.insert(session_ids, id)
            end
            return session_ids
        "#
        );

        c.bench_with_input(
            BenchmarkId::new("batch_session_creation", batch_size),
            batch_size,
            |b, &_size| {
                b.to_async(&rt).iter(|| async {
                    let engine = LuaEngine::new(&lua_config).unwrap();
                    engine.execute_script(&lua_code).await.unwrap()
                });
            },
        );
    }

    // Benchmark batch artifact storage
    let session_id = rt.block_on(async {
        let engine = LuaEngine::new(&lua_config).unwrap();
        engine
            .execute_script(r#"return Session.create({ name = "batch_artifact_test" })"#)
            .await
            .unwrap()
    });

    for batch_size in &[10, 50, 100, 500] {
        let store_batch_code = format!(
            r#"
            local artifacts = {{}}
            for i = 1, {} do
                local id = Artifact.store(
                    "{}",
                    "batch_data",
                    "batch_" .. i .. ".json",
                    '{{"index": ' .. i .. ', "data": "test"}}',
                    {{ category = "batch" }}
                )
                table.insert(artifacts, id)
            end
            return artifacts
        "#,
            batch_size,
            session_id.output.as_str().unwrap()
        );

        c.bench_with_input(
            BenchmarkId::new("batch_artifact_storage", batch_size),
            batch_size,
            |b, &_size| {
                b.to_async(&rt).iter(|| async {
                    let engine = LuaEngine::new(&lua_config).unwrap();
                    engine.execute_script(&store_batch_code).await.unwrap()
                });
            },
        );
    }
}

/// Target validation benchmarks - ensure we meet Phase 6 requirements
fn bench_target_validation(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let lua_config = LuaConfig::default();

    // Validate <50ms for session creation
    c.bench_function("target_session_create_50ms", |b| {
        b.to_async(&rt).iter(|| async {
            let engine = LuaEngine::new(&lua_config).unwrap();
            engine
                .execute_script(r#"return Session.create({ name = "perf_test" })"#)
                .await
                .unwrap()
        });
    });

    // Pre-create session for save/load tests
    let session_id = rt.block_on(async {
        let engine = LuaEngine::new(&lua_config).unwrap();
        engine
            .execute_script(r#"return Session.create({ name = "save_load_test" })"#)
            .await
            .unwrap()
    });

    // Validate <50ms for session save
    let save_code = format!(r#"Session.save("{}")"#, session_id.output.as_str().unwrap());
    c.bench_function("target_session_save_50ms", |b| {
        b.to_async(&rt).iter(|| async {
            let engine = LuaEngine::new(&lua_config).unwrap();
            engine.execute_script(&save_code).await.unwrap()
        });
    });

    // Validate <50ms for session load
    let load_code = format!(
        r#"return Session.load("{}")"#,
        session_id.output.as_str().unwrap()
    );
    c.bench_function("target_session_load_50ms", |b| {
        b.to_async(&rt).iter(|| async {
            let engine = LuaEngine::new(&lua_config).unwrap();
            engine.execute_script(&load_code).await.unwrap()
        });
    });

    // Validate <5ms for artifact storage (text/JSON)
    let store_text_code = format!(
        r#"
        return Artifact.store(
            "{}",
            "text",
            "quick_text.txt",
            "Small text content for <5ms validation",
            {{}}
        )
    "#,
        session_id.output.as_str().unwrap()
    );

    c.bench_function("target_artifact_store_text_5ms", |b| {
        b.to_async(&rt).iter(|| async {
            let engine = LuaEngine::new(&lua_config).unwrap();
            engine.execute_script(&store_text_code).await.unwrap()
        });
    });
}

criterion_group!(
    benches,
    bench_session_creation,
    bench_session_persistence,
    bench_artifact_operations,
    bench_batch_operations,
    bench_target_validation
);
criterion_main!(benches);
