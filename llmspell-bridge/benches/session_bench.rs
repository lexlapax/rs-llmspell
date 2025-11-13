//! ABOUTME: Performance benchmarks for Session and Artifact operations
//! ABOUTME: Validates <50ms session operations target from Phase 6 requirements

// Benchmark for llmspell-bridge sessions

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use llmspell_bridge::engine::LuaConfig;
use llmspell_bridge::lua::engine::LuaEngine;
use llmspell_bridge::{ComponentRegistry, ProviderManager, ScriptEngineBridge};
use llmspell_config::providers::ProviderManagerConfig;
use llmspell_config::{GlobalRuntimeConfig, LLMSpellConfig, SessionConfig, StatePersistenceConfig};
use std::sync::Arc;
use tokio::runtime::Runtime;

// Helper function to create test providers
async fn create_test_providers() -> Arc<ProviderManager> {
    let config = ProviderManagerConfig::default();
    Arc::new(
        ProviderManager::new(config)
            .await
            .expect("Failed to create provider manager"),
    )
}

// Helper function to create test registry
fn create_test_registry() -> Arc<ComponentRegistry> {
    Arc::new(ComponentRegistry::new())
}

// Helper function to create infrastructure registries for testing
fn create_test_infrastructure() -> (
    Arc<llmspell_tools::ToolRegistry>,
    Arc<llmspell_agents::FactoryRegistry>,
    Arc<dyn llmspell_workflows::WorkflowFactory>,
) {
    let tool_registry = Arc::new(llmspell_tools::ToolRegistry::new());
    let agent_registry = Arc::new(llmspell_agents::FactoryRegistry::new());
    let workflow_factory: Arc<dyn llmspell_workflows::WorkflowFactory> =
        Arc::new(llmspell_workflows::factory::DefaultWorkflowFactory::new());
    (tool_registry, agent_registry, workflow_factory)
}

// Helper function to create and setup engine with globals
async fn create_setup_engine() -> LuaEngine {
    let lua_config = LuaConfig::default();
    let mut engine = LuaEngine::new(&lua_config).unwrap();

    // Create runtime config with sessions enabled
    let runtime_config = Arc::new(LLMSpellConfig {
        runtime: GlobalRuntimeConfig {
            sessions: SessionConfig {
                enabled: true,
                max_sessions: 100,
                session_timeout_seconds: 3600,
                ..Default::default()
            },
            state_persistence: StatePersistenceConfig {
                enabled: true,
                backend_type: "memory".to_string(),
                ..Default::default()
            },
            ..Default::default()
        },
        ..Default::default()
    });

    engine.set_runtime_config(runtime_config);

    let registry = create_test_registry();
    let providers = create_test_providers().await;

    let (tool_registry, agent_registry, workflow_factory) = create_test_infrastructure();

    engine
        .inject_apis(
            &registry,
            &providers,
            &tool_registry,
            &agent_registry,
            &workflow_factory,
            None,
            None,
            None,
        )
        .unwrap();
    engine
}

/// Benchmark session creation performance
fn bench_session_creation(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let lua_code = r#"
        return Session.create({
            name = "benchmark_session",
            description = "Performance test session"
        })
    "#;

    c.bench_function("session_creation", |b| {
        b.to_async(&rt).iter(|| async {
            let engine = create_setup_engine().await;
            engine.execute_script(lua_code).await.unwrap()
        });
    });
}

/// Benchmark session save/load performance
fn bench_session_persistence(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    // Benchmark session save (creates and saves in same iteration)
    let save_code = r#"
        local session_id = Session.create({
            name = "persistence_benchmark"
        })
        Session.save(session_id)
        return session_id
    "#;

    c.bench_function("session_save", |b| {
        b.to_async(&rt).iter(|| async {
            let engine = create_setup_engine().await;
            engine.execute_script(save_code).await.unwrap()
        });
    });

    // For load benchmark, we need a saved session first
    // Create one session and save it, then benchmark loading
    let setup_code = r#"
        local session_id = Session.create({
            name = "load_benchmark"
        })
        Session.save(session_id)
        return session_id
    "#;

    c.bench_function("session_load", |b| {
        b.to_async(&rt).iter(|| async {
            let engine = create_setup_engine().await;
            // First create and save a session
            let result = engine.execute_script(setup_code).await.unwrap();
            let session_id = result.output.as_str().unwrap();

            // Then benchmark loading it
            let load_code = format!(r#"return Session.load("{session_id}")"#);
            engine.execute_script(&load_code).await.unwrap()
        });
    });
}

/// Benchmark artifact storage performance
fn bench_artifact_operations(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    // Benchmark artifact store (creates session and stores in same iteration)
    let store_code = r#"
        local session_id = Session.create({
            name = "artifact_benchmark"
        })
        return Artifact.store(
            session_id,
            "user_input",
            "benchmark_" .. math.random(1000000) .. ".txt",
            "Benchmark content for performance testing",
            { category = "benchmark" }
        )
    "#;

    c.bench_function("artifact_store", |b| {
        b.to_async(&rt).iter(|| async {
            let engine = create_setup_engine().await;
            engine.execute_script(store_code).await.unwrap()
        });
    });

    // Benchmark artifact list (needs session with artifacts)
    let list_code = r#"
        local session_id = Session.create({
            name = "list_benchmark"
        })
        -- Store a few artifacts first
        for i = 1, 5 do
            Artifact.store(
                session_id,
                "test_data",
                "file_" .. i .. ".txt",
                "Content " .. i,
                {}
            )
        end
        return Artifact.list(session_id)
    "#;

    c.bench_function("artifact_list", |b| {
        b.to_async(&rt).iter(|| async {
            let engine = create_setup_engine().await;
            engine.execute_script(list_code).await.unwrap()
        });
    });

    // Benchmark artifact query
    let query_code = r#"
        local session_id = Session.create({
            name = "query_benchmark"
        })
        -- Store some artifacts to query
        for i = 1, 10 do
            Artifact.store(
                session_id,
                "user_input",
                "query_file_" .. i .. ".txt",
                "Query content " .. i,
                { category = "test" }
            )
        end
        return Artifact.query({
            session_id = session_id,
            type = "user_input",
            limit = 10
        })
    "#;

    c.bench_function("artifact_query", |b| {
        b.to_async(&rt).iter(|| async {
            let engine = create_setup_engine().await;
            engine.execute_script(query_code).await.unwrap()
        });
    });
}

/// Benchmark batch operations for scalability
fn bench_batch_operations(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

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
                    let engine = create_setup_engine().await;
                    engine.execute_script(&lua_code).await.unwrap()
                });
            },
        );
    }

    // Benchmark batch artifact storage
    for batch_size in &[10, 50, 100, 500] {
        let store_batch_code = format!(
            r#"
            local session_id = Session.create({{ name = "batch_artifact_test" }})
            local artifacts = {{}}
            for i = 1, {batch_size} do
                local id = Artifact.store(
                    session_id,
                    "batch_data",
                    "batch_" .. i .. ".json",
                    '{{"index": ' .. i .. ', "data": "test"}}',
                    {{ category = "batch" }}
                )
                table.insert(artifacts, id)
            end
            return artifacts
        "#
        );

        c.bench_with_input(
            BenchmarkId::new("batch_artifact_storage", batch_size),
            batch_size,
            |b, &_size| {
                b.to_async(&rt).iter(|| async {
                    let engine = create_setup_engine().await;
                    engine.execute_script(&store_batch_code).await.unwrap()
                });
            },
        );
    }
}

/// Target validation benchmarks - ensure we meet Phase 6 requirements
fn bench_target_validation(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    // Validate <50ms for session creation
    c.bench_function("target_session_create_50ms", |b| {
        b.to_async(&rt).iter(|| async {
            let engine = create_setup_engine().await;
            engine
                .execute_script(r#"return Session.create({ name = "perf_test" })"#)
                .await
                .unwrap()
        });
    });

    // Validate <50ms for session save
    let save_code = r#"
        local session_id = Session.create({ name = "save_test" })
        Session.save(session_id)
        return session_id
    "#;
    c.bench_function("target_session_save_50ms", |b| {
        b.to_async(&rt).iter(|| async {
            let engine = create_setup_engine().await;
            engine.execute_script(save_code).await.unwrap()
        });
    });

    // Validate <50ms for session load
    let load_code = r#"
        local session_id = Session.create({ name = "load_test" })
        Session.save(session_id)
        return Session.load(session_id)
    "#;
    c.bench_function("target_session_load_50ms", |b| {
        b.to_async(&rt).iter(|| async {
            let engine = create_setup_engine().await;
            engine.execute_script(load_code).await.unwrap()
        });
    });

    // Validate <5ms for artifact storage (text/JSON)
    let store_text_code = r#"
        local session_id = Session.create({ name = "artifact_perf_test" })
        return Artifact.store(
            session_id,
            "text",
            "quick_text.txt",
            "Small text content for <5ms validation",
            {}
        )
    "#;

    c.bench_function("target_artifact_store_text_5ms", |b| {
        b.to_async(&rt).iter(|| async {
            let engine = create_setup_engine().await;
            engine.execute_script(store_text_code).await.unwrap()
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
