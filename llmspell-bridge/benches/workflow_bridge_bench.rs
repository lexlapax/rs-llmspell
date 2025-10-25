//! ABOUTME: Performance benchmarks for workflow bridge operations
//! ABOUTME: Measures overhead of bridge operations to ensure <10ms requirement
//!
//! **NOTE**: Most workflow benchmarks removed due to `GlobalContext` initialization
//! complexity. Only pure conversion logic benchmarks remain. Workflow execution
//! benchmarks can be added when proper `GlobalContext` setup helpers are available.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use serde_json::json;

fn benchmark_parameter_conversion(c: &mut Criterion) {
    use llmspell_bridge::conversion::json_to_workflow_params;

    c.bench_function("json_to_workflow_params", |b| {
        let json_params = json!({
            "name": "test_workflow",
            "type": "sequential",
            "steps": [
                {"name": "step1", "tool": "tool1"},
                {"name": "step2", "agent": "agent1"}
            ],
            "error_strategy": "continue"
        });

        b.iter(|| {
            let params = json_to_workflow_params(black_box(json_params.clone())).unwrap();
            black_box(params);
        });
    });
}

criterion_group!(benches, benchmark_parameter_conversion);
criterion_main!(benches);
