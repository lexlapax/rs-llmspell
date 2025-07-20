//! ABOUTME: Performance benchmarks for workflow bridge operations
//! ABOUTME: Measures overhead of bridge operations to ensure <10ms requirement

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use llmspell_bridge::{workflow_bridge::WorkflowBridge, ComponentRegistry};
use serde_json::json;
use std::sync::Arc;
use tokio::runtime::Runtime;

fn benchmark_workflow_creation(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let registry = Arc::new(ComponentRegistry::new());
    let bridge = WorkflowBridge::new(registry);

    c.bench_function("workflow_creation_sequential", |b| {
        b.iter(|| {
            rt.block_on(async {
                let params = json!({
                    "name": "test_workflow",
                    "steps": [
                        {"name": "step1", "tool": "mock_tool"}
                    ]
                });
                let id = bridge
                    .create_workflow("sequential", black_box(params))
                    .await
                    .unwrap();
                black_box(id);
            })
        })
    });

    c.bench_function("workflow_creation_parallel", |b| {
        b.iter(|| {
            rt.block_on(async {
                let params = json!({
                    "name": "test_workflow",
                    "branches": [
                        {"name": "branch1", "steps": []}
                    ],
                    "max_concurrency": 2
                });
                let id = bridge
                    .create_workflow("parallel", black_box(params))
                    .await
                    .unwrap();
                black_box(id);
            })
        })
    });
}

fn benchmark_workflow_discovery(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let registry = Arc::new(ComponentRegistry::new());
    let bridge = WorkflowBridge::new(registry);

    c.bench_function("list_workflow_types", |b| {
        b.iter(|| {
            rt.block_on(async {
                let types = bridge.list_workflow_types().await;
                black_box(types);
            })
        })
    });

    c.bench_function("get_workflow_info", |b| {
        b.iter(|| {
            rt.block_on(async {
                let info = bridge
                    .get_workflow_info(&"sequential".to_string())
                    .await
                    .unwrap();
                black_box(info);
            })
        })
    });
}

fn benchmark_parameter_conversion(c: &mut Criterion) {
    use llmspell_bridge::workflow_conversion_core::json_to_workflow_params;

    c.bench_function("json_to_workflow_params", |b| {
        let json_params = json!({
            "name": "test_workflow",
            "steps": [
                {"name": "step1", "tool": "tool1"},
                {"name": "step2", "agent": "agent1"}
            ],
            "error_strategy": "continue"
        });

        b.iter(|| {
            let params = json_to_workflow_params("sequential", black_box(&json_params)).unwrap();
            black_box(params);
        })
    });
}

fn benchmark_workflow_execution(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let registry = Arc::new(ComponentRegistry::new());
    let bridge = WorkflowBridge::new(registry);

    // Create a workflow once for execution benchmarks
    let workflow_id = rt.block_on(async {
        let params = json!({
            "name": "bench_workflow",
            "steps": [
                {"name": "step1", "tool": "mock_tool"}
            ]
        });
        bridge.create_workflow("sequential", params).await.unwrap()
    });

    c.bench_function("workflow_execution", |b| {
        b.iter(|| {
            rt.block_on(async {
                let input = json!({"test": "data"});
                let result = bridge
                    .execute_workflow(&workflow_id, black_box(input))
                    .await
                    .unwrap();
                black_box(result);
            })
        })
    });
}

fn benchmark_bridge_overhead(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let registry = Arc::new(ComponentRegistry::new());
    let bridge = WorkflowBridge::new(registry);

    // Measure overhead of bridge operations vs direct workflow operations
    c.bench_function("bridge_overhead_complete_cycle", |b| {
        b.iter(|| {
            rt.block_on(async {
                // Complete cycle: create, execute, get status
                let params = json!({
                    "name": "overhead_test",
                    "steps": [{"name": "step1", "tool": "mock"}]
                });

                let start = std::time::Instant::now();

                // Create workflow
                let id = bridge.create_workflow("sequential", params).await.unwrap();

                // Execute workflow
                let result = bridge.execute_workflow(&id, json!({})).await.unwrap();

                // Get execution history instead of status
                let history = bridge.get_execution_history().await;

                let duration = start.elapsed();

                black_box((id, result, history, duration));
            })
        })
    });
}

criterion_group!(
    benches,
    benchmark_workflow_creation,
    benchmark_workflow_discovery,
    benchmark_parameter_conversion,
    benchmark_workflow_execution,
    benchmark_bridge_overhead
);
criterion_main!(benches);
