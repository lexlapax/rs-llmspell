// ABOUTME: Tool initialization performance benchmarks for Task 2.10.3
// ABOUTME: Measures tool creation time to ensure <10ms requirement is met

// Benchmark file

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use llmspell_security::sandbox::{FileSandbox, SandboxContext};
use llmspell_tools::{
    data::{CsvAnalyzerTool, JsonProcessorTool},
    fs::{
        ArchiveHandlerTool, FileConverterTool, FileOperationsTool, FileSearchTool, FileWatcherTool,
    },
    system::{EnvironmentReaderTool, ProcessExecutorTool, ServiceCheckerTool, SystemMonitorTool},
    util::{
        Base64EncoderTool, CalculatorTool, DataValidationTool, DateTimeHandlerTool,
        DiffCalculatorTool, HashCalculatorTool, TemplateEngineTool, TextManipulatorTool,
        UuidGeneratorTool,
    },
    GraphQLQueryTool, HttpRequestTool, WebSearchTool,
};
use std::sync::Arc;
use tempfile::TempDir;

fn create_test_sandbox() -> Arc<FileSandbox> {
    let temp_dir = TempDir::new().unwrap();
    let context = SandboxContext::new(
        "benchmark-sandbox".to_string(),
        llmspell_core::traits::tool::SecurityRequirements::default()
            .with_file_access(temp_dir.path().to_str().unwrap()),
        Default::default(),
    );
    Arc::new(FileSandbox::new(context).unwrap())
}

fn bench_utility_tools_init(c: &mut Criterion) {
    let mut group = c.benchmark_group("utility_tools_init");

    // Target: <10ms initialization for all tools
    group.bench_function("base64_encoder", |b| {
        b.iter(|| {
            let tool = Base64EncoderTool::new();
            black_box(tool)
        })
    });

    group.bench_function("calculator", |b| {
        b.iter(|| {
            let tool = CalculatorTool::new();
            black_box(tool)
        })
    });

    group.bench_function("data_validation", |b| {
        b.iter(|| {
            let tool = DataValidationTool::new();
            black_box(tool)
        })
    });

    group.bench_function("date_time_handler", |b| {
        b.iter(|| {
            let tool = DateTimeHandlerTool::new();
            black_box(tool)
        })
    });

    group.bench_function("diff_calculator", |b| {
        b.iter(|| {
            let tool = DiffCalculatorTool::new();
            black_box(tool)
        })
    });

    group.bench_function("hash_calculator", |b| {
        b.iter(|| {
            let tool = HashCalculatorTool::new(Default::default());
            black_box(tool)
        })
    });

    group.bench_function("template_engine", |b| {
        b.iter(|| {
            let tool = TemplateEngineTool::new();
            black_box(tool)
        })
    });

    group.bench_function("text_manipulator", |b| {
        b.iter(|| {
            let tool = TextManipulatorTool::new(Default::default());
            black_box(tool)
        })
    });

    group.bench_function("uuid_generator", |b| {
        b.iter(|| {
            let tool = UuidGeneratorTool::new(Default::default());
            black_box(tool)
        })
    });

    group.finish();
}

fn bench_data_tools_init(c: &mut Criterion) {
    let mut group = c.benchmark_group("data_tools_init");

    group.bench_function("csv_analyzer", |b| {
        b.iter(|| {
            let tool = CsvAnalyzerTool::new(Default::default());
            black_box(tool)
        })
    });

    group.bench_function("json_processor", |b| {
        b.iter(|| {
            let tool = JsonProcessorTool::new(Default::default());
            black_box(tool)
        })
    });

    group.bench_function("graphql_query", |b| {
        b.iter(|| {
            let tool =
                GraphQLQueryTool::new(Default::default()).expect("Failed to create GraphQL tool");
            black_box(tool)
        })
    });

    group.bench_function("http_request", |b| {
        b.iter(|| {
            let tool =
                HttpRequestTool::new(Default::default()).expect("Failed to create HTTP tool");
            black_box(tool)
        })
    });

    group.finish();
}

fn bench_file_system_tools_init(c: &mut Criterion) {
    let mut group = c.benchmark_group("file_system_tools_init");

    group.bench_function("archive_handler", |b| {
        b.iter(|| {
            let tool = ArchiveHandlerTool::new();
            black_box(tool)
        })
    });

    group.bench_function("file_converter", |b| {
        let sandbox = create_test_sandbox();
        b.iter(|| {
            let tool = FileConverterTool::new(Default::default(), sandbox.clone());
            black_box(tool)
        })
    });

    group.bench_function("file_operations", |b| {
        b.iter(|| {
            let tool = FileOperationsTool::new(Default::default());
            black_box(tool)
        })
    });

    group.bench_function("file_search", |b| {
        let sandbox = create_test_sandbox();
        b.iter(|| {
            let tool = FileSearchTool::new(Default::default(), sandbox.clone());
            black_box(tool)
        })
    });

    group.bench_function("file_watcher", |b| {
        let sandbox = create_test_sandbox();
        b.iter(|| {
            let tool = FileWatcherTool::new(Default::default(), sandbox.clone());
            black_box(tool)
        })
    });

    group.finish();
}

fn bench_system_tools_init(c: &mut Criterion) {
    let mut group = c.benchmark_group("system_tools_init");

    group.bench_function("environment_reader", |b| {
        b.iter(|| {
            let tool = EnvironmentReaderTool::new(Default::default());
            black_box(tool)
        })
    });

    group.bench_function("process_executor", |b| {
        b.iter(|| {
            let tool = ProcessExecutorTool::new(Default::default());
            black_box(tool)
        })
    });

    group.bench_function("service_checker", |b| {
        b.iter(|| {
            let tool = ServiceCheckerTool::new(Default::default());
            black_box(tool)
        })
    });

    group.bench_function("system_monitor", |b| {
        b.iter(|| {
            let tool = SystemMonitorTool::new(Default::default());
            black_box(tool)
        })
    });

    group.finish();
}

fn bench_search_tools_init(c: &mut Criterion) {
    let mut group = c.benchmark_group("search_tools_init");

    group.bench_function("web_search", |b| {
        b.iter(|| {
            let tool = WebSearchTool::new(Default::default());
            black_box(tool)
        })
    });

    group.finish();
}

fn bench_all_tools_sequential(c: &mut Criterion) {
    let mut group = c.benchmark_group("sequential_initialization");

    // Benchmark creating all tools sequentially to simulate startup
    group.bench_function("all_tools_startup", |b| {
        let sandbox = create_test_sandbox();
        b.iter(|| {
            // Utility tools
            let _base64 = Base64EncoderTool::new();
            let _calc = CalculatorTool::new();
            let _data_val = DataValidationTool::new();
            let _datetime = DateTimeHandlerTool::new();
            let _diff = DiffCalculatorTool::new();
            let _hash = HashCalculatorTool::new(Default::default());
            let _template = TemplateEngineTool::new();
            let _text = TextManipulatorTool::new(Default::default());
            let _uuid = UuidGeneratorTool::new(Default::default());

            // Data tools
            let _csv = CsvAnalyzerTool::new(Default::default());
            let _json = JsonProcessorTool::new(Default::default());
            let _graphql =
                GraphQLQueryTool::new(Default::default()).expect("GraphQL creation failed");
            let _http = HttpRequestTool::new(Default::default()).expect("HTTP creation failed");

            // File system tools
            let _archive = ArchiveHandlerTool::new();
            let _file_conv = FileConverterTool::new(Default::default(), sandbox.clone());
            let _file_ops = FileOperationsTool::new(Default::default());
            let _file_search = FileSearchTool::new(Default::default(), sandbox.clone());
            let _file_watch = FileWatcherTool::new(Default::default(), sandbox.clone());

            // System tools
            let _env = EnvironmentReaderTool::new(Default::default());
            let _proc = ProcessExecutorTool::new(Default::default());
            let _service = ServiceCheckerTool::new(Default::default());
            let _system = SystemMonitorTool::new(Default::default());

            // Search tools
            let _web_search = WebSearchTool::new(Default::default());

            black_box(())
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_utility_tools_init,
    bench_data_tools_init,
    bench_file_system_tools_init,
    bench_system_tools_init,
    bench_search_tools_init,
    bench_all_tools_sequential
);
criterion_main!(benches);
