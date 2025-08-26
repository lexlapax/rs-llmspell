//! Performance benchmarks for llmspell-core
//!
//! Uses criterion to measure performance of core operations

// Benchmark for llmspell-core

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use llmspell_core::{
    traits::{
        agent::ConversationMessage,
        workflow::{RetryPolicy, WorkflowStep},
    },
    types::{AgentInput, AgentOutput, OutputMetadata},
    ComponentId, ComponentMetadata, ExecutionContext, LLMSpellError, Version,
};
use std::time::Duration;

fn bench_component_id_generation(c: &mut Criterion) {
    let mut group = c.benchmark_group("ComponentId");

    // Benchmark generation from different name lengths
    for name_len in [10, 50, 100, 500].iter() {
        let name = "a".repeat(*name_len);
        group.bench_with_input(BenchmarkId::new("from_name", name_len), name_len, |b, _| {
            b.iter(|| ComponentId::from_name(black_box(&name)));
        });
    }

    // Benchmark new() vs from_name()
    group.bench_function("new", |b| b.iter(ComponentId::new));

    group.finish();
}

fn bench_version_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("Version");

    let v1 = Version::new(1, 2, 3);
    let v2 = Version::new(1, 3, 0);
    let v3 = Version::new(2, 0, 0);

    group.bench_function("creation", |b| {
        b.iter(|| Version::new(black_box(1), black_box(2), black_box(3)))
    });

    group.bench_function("comparison", |b| {
        b.iter(|| {
            let _ = black_box(&v1) < black_box(&v2);
            let _ = black_box(&v2) < black_box(&v3);
        })
    });

    group.bench_function("compatibility_check", |b| {
        b.iter(|| {
            let _ = v1.is_compatible_with(black_box(&v2));
            let _ = v1.is_compatible_with(black_box(&v3));
        })
    });

    group.bench_function("to_string", |b| b.iter(|| v1.to_string()));

    group.finish();
}

fn bench_serialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("Serialization");

    // ComponentId serialization
    let component_id = ComponentId::from_name("test-component");
    group.bench_function("ComponentId_serialize", |b| {
        b.iter(|| serde_json::to_string(black_box(&component_id)).unwrap())
    });

    group.bench_function("ComponentId_deserialize", |b| {
        let json = serde_json::to_string(&component_id).unwrap();
        b.iter(|| {
            let _: ComponentId = serde_json::from_str(black_box(&json)).unwrap();
        })
    });

    // ComponentMetadata serialization
    let metadata =
        ComponentMetadata::new("test-component".to_string(), "A test component".to_string());

    group.bench_function("ComponentMetadata_serialize", |b| {
        b.iter(|| serde_json::to_string(black_box(&metadata)).unwrap())
    });

    group.bench_function("ComponentMetadata_deserialize", |b| {
        let json = serde_json::to_string(&metadata).unwrap();
        b.iter(|| {
            let _: ComponentMetadata = serde_json::from_str(black_box(&json)).unwrap();
        })
    });

    // AgentInput with context
    let context = ExecutionContext::new()
        .with_data("key1".to_string(), serde_json::json!("value1"))
        .with_data("key2".to_string(), serde_json::json!(42))
        .with_data("key3".to_string(), serde_json::json!({"nested": "value"}));
    let input = AgentInput::text("test prompt").with_context(context);

    group.bench_function("AgentInput_serialize", |b| {
        b.iter(|| serde_json::to_string(black_box(&input)).unwrap())
    });

    group.bench_function("AgentInput_deserialize", |b| {
        let json = serde_json::to_string(&input).unwrap();
        b.iter(|| {
            let _: AgentInput = serde_json::from_str(black_box(&json)).unwrap();
        })
    });

    group.finish();
}

fn bench_agent_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("Agent");

    // AgentInput creation and context manipulation
    group.bench_function("AgentInput_creation", |b| {
        b.iter(|| AgentInput::text(black_box("test prompt")))
    });

    group.bench_function("AgentInput_with_context", |b| {
        b.iter(|| {
            let context = ExecutionContext::new()
                .with_data("key1".to_string(), serde_json::json!("value1"))
                .with_data("key2".to_string(), serde_json::json!(42))
                .with_data("key3".to_string(), serde_json::json!(true));
            AgentInput::text("test").with_context(context)
        })
    });

    let context = ExecutionContext::new()
        .with_data("key1".to_string(), serde_json::json!("value1"))
        .with_data("key2".to_string(), serde_json::json!(42))
        .with_data("key3".to_string(), serde_json::json!(true));
    let input = AgentInput::text("test").with_context(context);

    group.bench_function("AgentInput_get_context", |b| {
        b.iter(|| {
            if let Some(ctx) = &input.context {
                let _ = ctx.data.get(black_box("key1"));
                let _ = ctx.data.get(black_box("key2"));
                let _ = ctx.data.get(black_box("nonexistent"));
            }
        })
    });

    // AgentOutput operations
    group.bench_function("AgentOutput_creation", |b| {
        b.iter(|| AgentOutput::text(black_box("result")))
    });

    group.bench_function("AgentOutput_with_metadata", |b| {
        b.iter(|| {
            let metadata = OutputMetadata {
                confidence: Some(0.95),
                token_count: Some(150),
                model: Some("gpt-4".to_string()),
                ..Default::default()
            };

            AgentOutput::text("result").with_metadata(metadata)
        })
    });

    // ConversationMessage operations
    group.bench_function("ConversationMessage_creation", |b| {
        b.iter(|| {
            let _ = ConversationMessage::system(black_box("System prompt".to_string()));
            let _ = ConversationMessage::user(black_box("User message".to_string()));
            let _ = ConversationMessage::assistant(black_box("Assistant response".to_string()));
        })
    });

    group.finish();
}

fn bench_error_handling(c: &mut Criterion) {
    let mut group = c.benchmark_group("Error");

    // Error creation
    group.bench_function("create_validation_error", |b| {
        b.iter(|| LLMSpellError::Validation {
            message: black_box("Invalid input".to_string()),
            field: Some(black_box("email".to_string())),
        })
    });

    group.bench_function("create_component_error", |b| {
        b.iter(|| LLMSpellError::Component {
            message: black_box("Component failed".to_string()),
            source: None,
        })
    });

    // Error property checks
    let errors = vec![
        LLMSpellError::Validation {
            message: "Invalid".to_string(),
            field: None,
        },
        LLMSpellError::Network {
            message: "Connection failed".to_string(),
            source: None,
        },
        LLMSpellError::Timeout {
            message: "Operation timed out".to_string(),
            duration_ms: Some(5000),
        },
    ];

    group.bench_function("error_severity_check", |b| {
        b.iter(|| {
            for err in &errors {
                let _ = black_box(err.severity());
            }
        })
    });

    group.bench_function("error_retryability_check", |b| {
        b.iter(|| {
            for err in &errors {
                let _ = black_box(err.is_retryable());
            }
        })
    });

    group.bench_function("error_category_check", |b| {
        b.iter(|| {
            for err in &errors {
                let _ = black_box(err.category());
            }
        })
    });

    group.finish();
}

fn bench_workflow_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("Workflow");

    let component_id = ComponentId::from_name("test-component");

    // WorkflowStep creation
    group.bench_function("WorkflowStep_creation", |b| {
        b.iter(|| WorkflowStep::new(black_box("step-name".to_string()), black_box(component_id)))
    });

    // WorkflowStep with dependencies
    group.bench_function("WorkflowStep_with_dependencies", |b| {
        let dep1 = ComponentId::from_name("dep1");
        let dep2 = ComponentId::from_name("dep2");
        let dep3 = ComponentId::from_name("dep3");

        b.iter(|| {
            WorkflowStep::new("step".to_string(), component_id)
                .with_dependency(black_box(dep1))
                .with_dependency(black_box(dep2))
                .with_dependency(black_box(dep3))
                .with_retry(RetryPolicy::default())
                .with_timeout(Duration::from_secs(30))
        })
    });

    group.finish();
}

fn bench_memory_usage(c: &mut Criterion) {
    let mut group = c.benchmark_group("Memory");

    // Measure allocation overhead for common types
    group.bench_function("ComponentId_size", |b| {
        b.iter(|| {
            let ids: Vec<ComponentId> = (0..1000)
                .map(|i| ComponentId::from_name(&format!("component-{}", i)))
                .collect();
            black_box(ids);
        })
    });

    group.bench_function("AgentInput_with_large_context", |b| {
        b.iter(|| {
            let mut context = ExecutionContext::new();
            for i in 0..100 {
                context = context.with_data(
                    format!("key{}", i),
                    serde_json::json!({"data": "x".repeat(100)}),
                );
            }
            let input = AgentInput::text("test").with_context(context);
            black_box(input);
        })
    });

    group.finish();
}

// Concurrent access benchmarks
fn bench_concurrent_operations(c: &mut Criterion) {
    use std::sync::Arc;
    use std::thread;

    let mut group = c.benchmark_group("Concurrent");

    // ComponentId generation from multiple threads
    group.bench_function("ComponentId_concurrent_generation", |b| {
        b.iter(|| {
            let handles: Vec<_> = (0..4)
                .map(|i| {
                    thread::spawn(move || {
                        for j in 0..25 {
                            let _ = ComponentId::from_name(&format!("component-{}-{}", i, j));
                        }
                    })
                })
                .collect();

            for handle in handles {
                handle.join().unwrap();
            }
        })
    });

    // Shared metadata access
    group.bench_function("ComponentMetadata_shared_access", |b| {
        let metadata = Arc::new(ComponentMetadata::new(
            "shared".to_string(),
            "Shared metadata".to_string(),
        ));

        b.iter(|| {
            let handles: Vec<_> = (0..4)
                .map(|_| {
                    let metadata_clone = Arc::clone(&metadata);
                    thread::spawn(move || {
                        for _ in 0..25 {
                            let _ = black_box(&metadata_clone.name);
                            let _ = black_box(&metadata_clone.version);
                            let _ = black_box(metadata_clone.id);
                        }
                    })
                })
                .collect();

            for handle in handles {
                handle.join().unwrap();
            }
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_component_id_generation,
    bench_version_operations,
    bench_serialization,
    bench_agent_operations,
    bench_error_handling,
    bench_workflow_operations,
    bench_memory_usage,
    bench_concurrent_operations
);
criterion_main!(benches);
