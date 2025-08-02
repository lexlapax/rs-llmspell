// ABOUTME: Performance benchmarks for workflow hook overhead
// ABOUTME: Validates <3% overhead requirement for hook execution

// Benchmark for workflow hook overhead

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use llmspell_testing::workflow_helpers::create_test_steps;
use llmspell_workflows::{
    conditional::{ConditionalBranch, ConditionalWorkflow},
    conditions::Condition,
    hooks::{WorkflowExecutor, WorkflowLifecycleConfig},
    parallel::{ParallelBranch, ParallelWorkflowBuilder},
    r#loop::LoopWorkflowBuilder,
    sequential::SequentialWorkflow,
    traits::{StepType, WorkflowStep},
};
use serde_json::json;
use std::sync::Arc;

fn benchmark_sequential_workflow(c: &mut Criterion) {
    let mut group = c.benchmark_group("sequential_workflow");
    group.sample_size(50); // Reduce sample size for faster benchmarks

    let rt = tokio::runtime::Runtime::new().unwrap();

    for step_count in [1, 5, 10, 20] {
        // Benchmark without hooks
        group.bench_with_input(
            BenchmarkId::new("without_hooks", step_count),
            &step_count,
            |b, &count| {
                b.to_async(&rt).iter(|| async move {
                    let workflow = SequentialWorkflow::builder("bench_sequential".to_string())
                        .add_steps(create_test_steps(count))
                        .build();

                    let result = workflow.execute().await.unwrap();
                    black_box(result);
                });
            },
        );

        // Benchmark with hooks
        group.bench_with_input(
            BenchmarkId::new("with_hooks", step_count),
            &step_count,
            |b, &count| {
                b.to_async(&rt).iter(|| async move {
                    let workflow_executor = Arc::new(WorkflowExecutor::new(
                        WorkflowLifecycleConfig::default(),
                        None,
                        None,
                    ));

                    let workflow = SequentialWorkflow::builder("bench_sequential".to_string())
                        .with_hooks(workflow_executor)
                        .add_steps(create_test_steps(count))
                        .build();

                    let result = workflow.execute().await.unwrap();
                    black_box(result);
                });
            },
        );
    }

    group.finish();
}

fn benchmark_conditional_workflow(c: &mut Criterion) {
    let mut group = c.benchmark_group("conditional_workflow");
    group.sample_size(50);

    let rt = tokio::runtime::Runtime::new().unwrap();

    for branch_count in [1, 3, 5, 10] {
        // Benchmark without hooks
        group.bench_with_input(
            BenchmarkId::new("without_hooks", branch_count),
            &branch_count,
            |b, &count| {
                b.to_async(&rt).iter(|| async move {
                    let mut builder = ConditionalWorkflow::builder("bench_conditional".to_string());

                    // Add branches (only first one will execute)
                    for i in 0..count {
                        let condition = if i == 0 {
                            Condition::Always
                        } else {
                            Condition::Never
                        };
                        let branch = ConditionalBranch::new(format!("branch_{}", i), condition)
                            .with_steps(create_test_steps(3));
                        builder = builder.add_branch(branch);
                    }

                    let workflow = builder.build();
                    let result = workflow.execute().await.unwrap();
                    black_box(result);
                });
            },
        );

        // Benchmark with hooks
        group.bench_with_input(
            BenchmarkId::new("with_hooks", branch_count),
            &branch_count,
            |b, &count| {
                b.to_async(&rt).iter(|| async move {
                    let workflow_executor = Arc::new(WorkflowExecutor::new(
                        WorkflowLifecycleConfig::default(),
                        None,
                        None,
                    ));

                    let mut builder = ConditionalWorkflow::builder("bench_conditional".to_string())
                        .with_hooks(workflow_executor);

                    for i in 0..count {
                        let condition = if i == 0 {
                            Condition::Always
                        } else {
                            Condition::Never
                        };
                        let branch = ConditionalBranch::new(format!("branch_{}", i), condition)
                            .with_steps(create_test_steps(3));
                        builder = builder.add_branch(branch);
                    }

                    let workflow = builder.build();
                    let result = workflow.execute().await.unwrap();
                    black_box(result);
                });
            },
        );
    }

    group.finish();
}

fn benchmark_loop_workflow(c: &mut Criterion) {
    let mut group = c.benchmark_group("loop_workflow");
    group.sample_size(30); // Lower sample size for loops

    let rt = tokio::runtime::Runtime::new().unwrap();

    for iteration_count in [5, 10, 20, 50] {
        // Benchmark without hooks
        group.bench_with_input(
            BenchmarkId::new("without_hooks", iteration_count),
            &iteration_count,
            |b, &count| {
                b.to_async(&rt).iter(|| async move {
                    let workflow = LoopWorkflowBuilder::new("bench_loop".to_string())
                        .with_range(0, count as i64, 1)
                        .add_step(WorkflowStep::new(
                            "loop_step".to_string(),
                            StepType::Custom {
                                function_name: "test".to_string(),
                                parameters: json!({}),
                            },
                        ))
                        .build()
                        .unwrap();

                    let result = workflow.execute().await.unwrap();
                    black_box(result);
                });
            },
        );

        // Benchmark with hooks
        group.bench_with_input(
            BenchmarkId::new("with_hooks", iteration_count),
            &iteration_count,
            |b, &count| {
                b.to_async(&rt).iter(|| async move {
                    let workflow_executor = Arc::new(WorkflowExecutor::new(
                        WorkflowLifecycleConfig::default(),
                        None,
                        None,
                    ));

                    let workflow = LoopWorkflowBuilder::new("bench_loop".to_string())
                        .with_range(0, count as i64, 1)
                        .add_step(WorkflowStep::new(
                            "loop_step".to_string(),
                            StepType::Custom {
                                function_name: "test".to_string(),
                                parameters: json!({}),
                            },
                        ))
                        .with_hooks(workflow_executor)
                        .build()
                        .unwrap();

                    let result = workflow.execute().await.unwrap();
                    black_box(result);
                });
            },
        );
    }

    group.finish();
}

fn benchmark_parallel_workflow(c: &mut Criterion) {
    let mut group = c.benchmark_group("parallel_workflow");
    group.sample_size(30); // Lower sample size for parallel execution

    let rt = tokio::runtime::Runtime::new().unwrap();

    for branch_count in [2, 4, 8, 16] {
        // Benchmark without hooks
        group.bench_with_input(
            BenchmarkId::new("without_hooks", branch_count),
            &branch_count,
            |b, &count| {
                b.to_async(&rt).iter(|| async move {
                    let mut builder = ParallelWorkflowBuilder::new("bench_parallel".to_string());

                    for i in 0..count {
                        let branch = ParallelBranch::new(format!("branch_{}", i)).add_step(
                            WorkflowStep::new(
                                format!("step_{}", i),
                                StepType::Custom {
                                    function_name: "test".to_string(),
                                    parameters: json!({"branch": i}),
                                },
                            ),
                        );
                        builder = builder.add_branch(branch);
                    }

                    let workflow = builder.build().unwrap();
                    let result = workflow.execute().await.unwrap();
                    black_box(result);
                });
            },
        );

        // Benchmark with hooks
        group.bench_with_input(
            BenchmarkId::new("with_hooks", branch_count),
            &branch_count,
            |b, &count| {
                b.to_async(&rt).iter(|| async move {
                    let workflow_executor = Arc::new(WorkflowExecutor::new(
                        WorkflowLifecycleConfig::default(),
                        None,
                        None,
                    ));

                    let mut builder = ParallelWorkflowBuilder::new("bench_parallel".to_string())
                        .with_hooks(workflow_executor);

                    for i in 0..count {
                        let branch = ParallelBranch::new(format!("branch_{}", i)).add_step(
                            WorkflowStep::new(
                                format!("step_{}", i),
                                StepType::Custom {
                                    function_name: "test".to_string(),
                                    parameters: json!({"branch": i}),
                                },
                            ),
                        );
                        builder = builder.add_branch(branch);
                    }

                    let workflow = builder.build().unwrap();
                    let result = workflow.execute().await.unwrap();
                    black_box(result);
                });
            },
        );
    }

    group.finish();
}

// Focused benchmark for overhead calculation
fn benchmark_hook_overhead(c: &mut Criterion) {
    let mut group = c.benchmark_group("hook_overhead_percentage");
    group.sample_size(100); // Higher sample size for accuracy

    let rt = tokio::runtime::Runtime::new().unwrap();

    // Sequential workflow with 10 steps
    group.bench_function("sequential_10_steps_baseline", |b| {
        b.to_async(&rt).iter(|| async {
            let workflow = SequentialWorkflow::builder("overhead_test".to_string())
                .add_steps(create_test_steps(10))
                .build();

            let result = workflow.execute().await.unwrap();
            black_box(result);
        });
    });

    group.bench_function("sequential_10_steps_with_hooks", |b| {
        b.to_async(&rt).iter(|| async {
            let workflow_executor = Arc::new(WorkflowExecutor::new(
                WorkflowLifecycleConfig::default(),
                None,
                None,
            ));

            let workflow = SequentialWorkflow::builder("overhead_test".to_string())
                .with_hooks(workflow_executor)
                .add_steps(create_test_steps(10))
                .build();

            let result = workflow.execute().await.unwrap();
            black_box(result);
        });
    });

    // Complex workflow scenario
    group.bench_function("complex_workflow_baseline", |b| {
        b.to_async(&rt).iter(|| async {
            // Create a workflow with conditional, loop, and sequential steps
            let branch1 = ConditionalBranch::new("main_branch".to_string(), Condition::Always)
                .with_steps(create_test_steps(5));

            let branch2 = ConditionalBranch::new("alt_branch".to_string(), Condition::Never)
                .with_steps(create_test_steps(3));

            let workflow = ConditionalWorkflow::builder("complex_test".to_string())
                .add_branch(branch1)
                .add_branch(branch2)
                .build();

            let result = workflow.execute().await.unwrap();
            black_box(result);
        });
    });

    group.bench_function("complex_workflow_with_hooks", |b| {
        b.to_async(&rt).iter(|| async {
            let workflow_executor = Arc::new(WorkflowExecutor::new(
                WorkflowLifecycleConfig::default(),
                None,
                None,
            ));

            let branch1 = ConditionalBranch::new("main_branch".to_string(), Condition::Always)
                .with_steps(create_test_steps(5));

            let branch2 = ConditionalBranch::new("alt_branch".to_string(), Condition::Never)
                .with_steps(create_test_steps(3));

            let workflow = ConditionalWorkflow::builder("complex_test".to_string())
                .with_hooks(workflow_executor)
                .add_branch(branch1)
                .add_branch(branch2)
                .build();

            let result = workflow.execute().await.unwrap();
            black_box(result);
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    benchmark_sequential_workflow,
    benchmark_conditional_workflow,
    benchmark_loop_workflow,
    benchmark_parallel_workflow,
    benchmark_hook_overhead
);
criterion_main!(benches);
