//! ABOUTME: Performance benchmarks for web tools
//! ABOUTME: Measures initialization time and basic operations for external integration tools

// Benchmark file

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use llmspell_core::{types::AgentInput, BaseAgent};
use llmspell_tools::{
    ApiTesterTool, SitemapCrawlerTool, UrlAnalyzerTool, WebScraperTool, WebhookCallerTool,
    WebpageMonitorTool,
};
use serde_json::json;
use std::collections::HashMap;

fn create_test_input(url: &str) -> AgentInput {
    let mut params = HashMap::new();
    params.insert(
        "parameters".to_string(),
        json!({
            "input": url
        }),
    );

    let mut input = AgentInput::text("");
    input.parameters = params;
    input
}

fn benchmark_tool_initialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("tool_initialization");

    group.bench_function("ApiTesterTool::new", |b| {
        b.iter(|| {
            let _ = black_box(ApiTesterTool::new());
        });
    });

    group.bench_function("WebScraperTool::default", |b| {
        b.iter(|| {
            let _ = black_box(WebScraperTool::default());
        });
    });

    group.bench_function("UrlAnalyzerTool::new", |b| {
        b.iter(|| {
            let _ = black_box(UrlAnalyzerTool::new());
        });
    });

    group.bench_function("WebhookCallerTool::new", |b| {
        b.iter(|| {
            let _ = black_box(WebhookCallerTool::new());
        });
    });

    group.bench_function("WebpageMonitorTool::new", |b| {
        b.iter(|| {
            let _ = black_box(WebpageMonitorTool::new());
        });
    });

    group.bench_function("SitemapCrawlerTool::new", |b| {
        b.iter(|| {
            let _ = black_box(SitemapCrawlerTool::new());
        });
    });

    group.finish();
}

fn benchmark_input_validation(c: &mut Criterion) {
    let mut group = c.benchmark_group("input_validation");

    let input = create_test_input("https://example.com");
    let runtime = tokio::runtime::Runtime::new().unwrap();

    group.bench_function("ApiTesterTool::validate_input", |b| {
        let tool = ApiTesterTool::new();
        b.iter(|| {
            runtime.block_on(async {
                let _ = black_box(tool.validate_input(&input).await);
            });
        });
    });

    group.bench_function("UrlAnalyzerTool::validate_input", |b| {
        let tool = UrlAnalyzerTool::new();
        b.iter(|| {
            runtime.block_on(async {
                let _ = black_box(tool.validate_input(&input).await);
            });
        });
    });

    group.finish();
}

fn benchmark_schema_generation(c: &mut Criterion) {
    use llmspell_core::Tool;

    let mut group = c.benchmark_group("schema_generation");

    group.bench_function("ApiTesterTool::schema", |b| {
        let tool = ApiTesterTool::new();
        b.iter(|| {
            let _ = black_box(tool.schema());
        });
    });

    group.bench_function("WebScraperTool::schema", |b| {
        let tool = WebScraperTool::default();
        b.iter(|| {
            let _ = black_box(tool.schema());
        });
    });

    group.bench_function("UrlAnalyzerTool::schema", |b| {
        let tool = UrlAnalyzerTool::new();
        b.iter(|| {
            let _ = black_box(tool.schema());
        });
    });

    group.finish();
}

fn benchmark_memory_usage(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_footprint");

    group.bench_function("create_100_tools", |b| {
        b.iter(|| {
            let tools: Vec<Box<dyn BaseAgent>> = (0..100)
                .map(|i| match i % 6 {
                    0 => Box::new(ApiTesterTool::new()) as Box<dyn BaseAgent>,
                    1 => Box::new(WebScraperTool::default()) as Box<dyn BaseAgent>,
                    2 => Box::new(UrlAnalyzerTool::new()) as Box<dyn BaseAgent>,
                    3 => Box::new(WebhookCallerTool::new()) as Box<dyn BaseAgent>,
                    4 => Box::new(WebpageMonitorTool::new()) as Box<dyn BaseAgent>,
                    _ => Box::new(SitemapCrawlerTool::new()) as Box<dyn BaseAgent>,
                })
                .collect();
            black_box(tools);
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    benchmark_tool_initialization,
    benchmark_input_validation,
    benchmark_schema_generation,
    benchmark_memory_usage
);

criterion_main!(benches);
