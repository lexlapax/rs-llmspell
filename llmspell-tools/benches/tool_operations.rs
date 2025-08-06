// ABOUTME: Tool operation performance benchmarks for Task 2.10.3
// ABOUTME: Measures tool execution performance for common operations

// Benchmark file

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use llmspell_core::{traits::base_agent::BaseAgent, types::AgentInput, ExecutionContext};
use llmspell_tools::{
    data::JsonProcessorTool,
    util::{
        Base64EncoderTool, CalculatorTool, HashCalculatorTool, TextManipulatorTool,
        UuidGeneratorTool,
    },
};
use serde_json::json;
use tokio::runtime::Runtime;

fn bench_base64_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("base64_operations");
    let rt = Runtime::new().unwrap();

    let tool = Base64EncoderTool::new();
    let test_data = "Hello, World! This is a test string for base64 encoding.".repeat(100);

    group.bench_function("encode_1kb", |b| {
        b.iter(|| {
            let input = AgentInput::text("encode").with_parameter(
                "parameters",
                json!({
                    "operation": "encode",
                    "data": test_data
                }),
            );
            rt.block_on(async {
                let result = tool.execute(input, ExecutionContext::default()).await;
                black_box(result)
            })
        })
    });

    let encoded_data =
        "SGVsbG8sIFdvcmxkISBUaGlzIGlzIGEgdGVzdCBzdHJpbmcgZm9yIGJhc2U2NCBlbmNvZGluZy4=".repeat(100);

    group.bench_function("decode_1kb", |b| {
        b.iter(|| {
            let input = AgentInput::text("decode").with_parameter(
                "parameters",
                json!({
                    "operation": "decode",
                    "data": encoded_data
                }),
            );
            rt.block_on(async {
                let result = tool.execute(input, ExecutionContext::default()).await;
                black_box(result)
            })
        })
    });

    group.finish();
}

fn bench_calculator_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("calculator_operations");
    let rt = Runtime::new().unwrap();

    let tool = CalculatorTool::new();

    group.bench_function("simple_arithmetic", |b| {
        b.iter(|| {
            let input = AgentInput::text("calculate").with_parameter(
                "parameters",
                json!({
                    "expression": "2 + 3 * 4 - 1"
                }),
            );
            rt.block_on(async {
                let result = tool.execute(input, ExecutionContext::default()).await;
                black_box(result)
            })
        })
    });

    group.bench_function("complex_expression", |b| {
        b.iter(|| {
            let input = AgentInput::text("calculate").with_parameter(
                "parameters",
                json!({
                    "expression": "sqrt(16) + pow(2, 3) + sin(pi/2) + log(10)"
                }),
            );
            rt.block_on(async {
                let result = tool.execute(input, ExecutionContext::default()).await;
                black_box(result)
            })
        })
    });

    group.finish();
}

fn bench_hash_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("hash_operations");
    let rt = Runtime::new().unwrap();

    let tool = HashCalculatorTool::new(Default::default());

    let small_data = "Hello, World!";
    let medium_data = "A".repeat(1000); // 1KB
    let large_data = "B".repeat(100_000); // 100KB

    group.bench_function("md5_small", |b| {
        b.iter(|| {
            let input = AgentInput::text("hash").with_parameter(
                "parameters",
                json!({
                    "operation": "hash",
                    "algorithm": "md5",
                    "data": small_data
                }),
            );
            rt.block_on(async {
                let result = tool.execute(input, ExecutionContext::default()).await;
                black_box(result)
            })
        })
    });

    group.bench_function("sha256_medium", |b| {
        b.iter(|| {
            let input = AgentInput::text("hash").with_parameter(
                "parameters",
                json!({
                    "operation": "hash",
                    "algorithm": "sha256",
                    "data": medium_data
                }),
            );
            rt.block_on(async {
                let result = tool.execute(input, ExecutionContext::default()).await;
                black_box(result)
            })
        })
    });

    group.bench_function("sha256_large", |b| {
        b.iter(|| {
            let input = AgentInput::text("hash").with_parameter(
                "parameters",
                json!({
                    "operation": "hash",
                    "algorithm": "sha256",
                    "data": large_data
                }),
            );
            rt.block_on(async {
                let result = tool.execute(input, ExecutionContext::default()).await;
                black_box(result)
            })
        })
    });

    group.finish();
}

fn bench_text_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("text_operations");
    let rt = Runtime::new().unwrap();

    let tool = TextManipulatorTool::new(Default::default());
    let test_text = "Lorem ipsum dolor sit amet, consectetur adipiscing elit. ".repeat(1000);

    group.bench_function("uppercase_1kb", |b| {
        b.iter(|| {
            let input = AgentInput::text("transform").with_parameter(
                "parameters",
                json!({
                    "operation": "uppercase",
                    "text": test_text
                }),
            );
            rt.block_on(async {
                let result = tool.execute(input, ExecutionContext::default()).await;
                black_box(result)
            })
        })
    });

    group.bench_function("replace_operation", |b| {
        b.iter(|| {
            let input = AgentInput::text("transform").with_parameter(
                "parameters",
                json!({
                    "operation": "replace",
                    "text": test_text,
                    "pattern": "Lorem",
                    "replacement": "LOREM"
                }),
            );
            rt.block_on(async {
                let result = tool.execute(input, ExecutionContext::default()).await;
                black_box(result)
            })
        })
    });

    group.finish();
}

fn bench_json_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("json_operations");
    let rt = Runtime::new().unwrap();

    let tool = JsonProcessorTool::new(Default::default());

    // Create test JSON data
    let simple_json = json!({
        "name": "John",
        "age": 30,
        "city": "New York"
    });

    let complex_json = json!({
        "users": (0..100).map(|i| json!({
            "id": i,
            "name": format!("User {}", i),
            "email": format!("user{}@example.com", i),
            "profile": {
                "age": 20 + (i % 50),
                "city": "City",
                "tags": ["tag1", "tag2", "tag3"]
            }
        })).collect::<Vec<_>>()
    });

    group.bench_function("simple_query", |b| {
        b.iter(|| {
            let input = AgentInput::text("query").with_parameter(
                "parameters",
                json!({
                    "operation": "query",
                    "input": simple_json,
                    "query": ".name"
                }),
            );
            rt.block_on(async {
                let result = tool.execute(input, ExecutionContext::default()).await;
                black_box(result)
            })
        })
    });

    group.bench_function("complex_query", |b| {
        b.iter(|| {
            let input = AgentInput::text("query").with_parameter(
                "parameters",
                json!({
                    "operation": "query",
                    "input": complex_json,
                    "query": ".users[] | select(.profile.age > 30) | .name"
                }),
            );
            rt.block_on(async {
                let result = tool.execute(input, ExecutionContext::default()).await;
                black_box(result)
            })
        })
    });

    group.finish();
}

fn bench_uuid_generation(c: &mut Criterion) {
    let mut group = c.benchmark_group("uuid_operations");
    let rt = Runtime::new().unwrap();

    let tool = UuidGeneratorTool::new(Default::default());

    group.bench_function("uuid_v4_generation", |b| {
        b.iter(|| {
            let input = AgentInput::text("generate").with_parameter(
                "parameters",
                json!({
                    "operation": "generate",
                    "version": "v4"
                }),
            );
            rt.block_on(async {
                let result = tool.execute(input, ExecutionContext::default()).await;
                black_box(result)
            })
        })
    });

    group.bench_function("uuid_bulk_generation", |b| {
        b.iter(|| {
            let input = AgentInput::text("generate").with_parameter(
                "parameters",
                json!({
                    "operation": "bulk_generate",
                    "version": "v4",
                    "count": 100
                }),
            );
            rt.block_on(async {
                let result = tool.execute(input, ExecutionContext::default()).await;
                black_box(result)
            })
        })
    });

    group.finish();
}

fn bench_mixed_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("mixed_operations");
    let rt = Runtime::new().unwrap();

    // Simulate a realistic workflow using multiple tools
    group.bench_function("data_processing_workflow", |b| {
        let json_tool = JsonProcessorTool::new(Default::default());
        let hash_tool = HashCalculatorTool::new(Default::default());
        let text_tool = TextManipulatorTool::new(Default::default());

        b.iter(|| {
            rt.block_on(async {
                // 1. Process JSON data
                let json_input = AgentInput::text("query").with_parameter(
                    "parameters",
                    json!({
                        "operation": "query",
                        "input": {"data": "test value", "id": 123},
                        "query": ".data"
                    }),
                );
                let json_result = json_tool
                    .execute(json_input, ExecutionContext::default())
                    .await
                    .unwrap();

                // 2. Transform text
                let text_input = AgentInput::text("transform").with_parameter(
                    "parameters",
                    json!({
                        "operation": "uppercase",
                        "text": "test value"
                    }),
                );
                let text_result = text_tool
                    .execute(text_input, ExecutionContext::default())
                    .await
                    .unwrap();

                // 3. Hash the result
                let hash_input = AgentInput::text("hash").with_parameter(
                    "parameters",
                    json!({
                        "operation": "hash",
                        "algorithm": "sha256",
                        "data": "TEST VALUE"
                    }),
                );
                let hash_result = hash_tool
                    .execute(hash_input, ExecutionContext::default())
                    .await
                    .unwrap();

                black_box((json_result, text_result, hash_result))
            })
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_base64_operations,
    bench_calculator_operations,
    bench_hash_operations,
    bench_text_operations,
    bench_json_operations,
    bench_uuid_generation,
    bench_mixed_operations
);
criterion_main!(benches);
