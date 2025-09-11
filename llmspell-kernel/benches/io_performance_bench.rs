//! Performance benchmarks for IO subsystem
//!
//! Measures the performance improvements from buffered IO:
//! - Direct println!: ~1μs per call
//! - `IOContext` with batching: ~100ns amortized
//! - Target: 10x improvement for script with heavy output

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use llmspell_core::io::{BufferedStream, IOContext, IOPerformanceHints, IOStream, MockStream};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Measure direct println! performance
fn bench_direct_println(c: &mut Criterion) {
    c.bench_function("direct_println", |b| {
        b.iter(|| {
            for i in 0..100 {
                // Using print! to avoid actual console output during benchmarks
                let _ = black_box(format!("Line {i}"));
            }
        });
    });
}

/// Measure unbuffered `IOContext` performance
fn bench_unbuffered_io_context(c: &mut Criterion) {
    c.bench_function("unbuffered_io_context", |b| {
        let mock = Arc::new(MockStream::new());
        let io_context = IOContext::new(
            mock.clone(),
            Arc::new(MockStream::new()),
            Arc::new(llmspell_core::io::NullInput),
            Arc::new(llmspell_core::io::NoOpSignalHandler),
            IOPerformanceHints::default(),
        );

        b.iter(|| {
            for i in 0..100 {
                let _ = io_context.stdout.write_line(&format!("Line {i}"));
            }
            let _ = io_context.stdout.flush();
        });

        // Clear the mock stream for next iteration
        mock.lines.lock().unwrap().clear();
    });
}

/// Measure buffered `IOContext` performance with different batch sizes
fn bench_buffered_io_context(c: &mut Criterion) {
    let mut group = c.benchmark_group("buffered_io_context");

    for batch_size in [10, 50, 100, 200] {
        group.bench_with_input(
            BenchmarkId::from_parameter(batch_size),
            &batch_size,
            |b, &batch_size| {
                let mock = Arc::new(MockStream::new());
                let buffered = Arc::new(BufferedStream::with_interval(
                    mock.clone(),
                    batch_size,
                    Duration::from_millis(100),
                ));

                b.iter(|| {
                    for i in 0..100 {
                        let _ = buffered.write_line(&format!("Line {i}"));
                    }
                    let _ = buffered.flush();
                });

                // Clear the mock stream for next iteration
                mock.lines.lock().unwrap().clear();
            },
        );
    }

    group.finish();
}

/// Measure `IOContext` pool performance
fn bench_io_context_pool(c: &mut Criterion) {
    use llmspell_core::io::IOContextPool;

    c.bench_function("io_context_pool_acquire_release", |b| {
        let pool = IOContextPool::new(10);

        b.iter(|| {
            let context = pool.acquire();
            black_box(&context);
            pool.release(context);
        });
    });
}

/// Compare throughput: direct vs buffered for heavy output scenario
fn bench_heavy_output_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("heavy_output_throughput");

    // Baseline: Direct write to Vec (simulating println!)
    group.bench_function("baseline_direct", |b| {
        b.iter(|| {
            let mut output = Vec::with_capacity(10000);
            for i in 0..1000 {
                let line =
                    format!("Line {i}: Some longer output text that simulates real log data\n");
                output.extend_from_slice(line.as_bytes());
            }
            black_box(output);
        });
    });

    // Unbuffered IOContext
    group.bench_function("unbuffered", |b| {
        let mock = Arc::new(MockStream::new());
        let io_context = IOContext::new(
            mock.clone(),
            Arc::new(MockStream::new()),
            Arc::new(llmspell_core::io::NullInput),
            Arc::new(llmspell_core::io::NoOpSignalHandler),
            IOPerformanceHints::default(),
        );

        b.iter(|| {
            for i in 0..1000 {
                let _ = io_context.stdout.write_line(&format!(
                    "Line {i}: Some longer output text that simulates real log data"
                ));
            }
            let _ = io_context.stdout.flush();
            mock.lines.lock().unwrap().clear();
        });
    });

    // Buffered IOContext with optimal batch size
    group.bench_function("buffered_optimal", |b| {
        let mock = Arc::new(MockStream::new());
        let buffered = Arc::new(BufferedStream::with_interval(
            mock.clone(),
            100, // Batch size
            Duration::from_millis(50),
        ));

        b.iter(|| {
            for i in 0..1000 {
                let _ = buffered.write_line(&format!(
                    "Line {i}: Some longer output text that simulates real log data"
                ));
            }
            let _ = buffered.flush();
            mock.lines.lock().unwrap().clear();
        });
    });

    group.finish();
}

/// Measure latency: time to first output
fn bench_latency_to_first_output(c: &mut Criterion) {
    let mut group = c.benchmark_group("latency_to_first_output");

    // Unbuffered - immediate output
    group.bench_function("unbuffered", |b| {
        let mock = Arc::new(MockStream::new());

        b.iter(|| {
            let start = Instant::now();
            let _ = mock.write_line("First output");
            let _ = mock.flush();
            black_box(start.elapsed());
            mock.lines.lock().unwrap().clear();
        });
    });

    // Buffered with small batch - low latency config
    group.bench_function("buffered_low_latency", |b| {
        let mock = Arc::new(MockStream::new());
        let buffered = Arc::new(BufferedStream::with_interval(
            mock.clone(),
            1, // Batch size of 1 for low latency
            Duration::from_millis(10),
        ));

        b.iter(|| {
            let start = Instant::now();
            let _ = buffered.write_line("First output");
            let _ = buffered.flush();
            black_box(start.elapsed());
            mock.lines.lock().unwrap().clear();
        });
    });

    group.finish();
}

/// Measure memory usage patterns
fn bench_memory_usage(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_usage");

    // Measure allocation patterns for different batch sizes
    for batch_size in [10, 100, 1000] {
        group.bench_with_input(
            BenchmarkId::new("batch_size", batch_size),
            &batch_size,
            |b, &batch_size| {
                b.iter(|| {
                    let mock = Arc::new(MockStream::new());
                    let buffered =
                        BufferedStream::with_interval(mock, batch_size, Duration::from_millis(100));

                    // Simulate workload
                    for i in 0..batch_size {
                        let _ = buffered.write_line(&format!("Line {i}"));
                    }
                    let _ = buffered.flush();

                    black_box(buffered);
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_direct_println,
    bench_unbuffered_io_context,
    bench_buffered_io_context,
    bench_io_context_pool,
    bench_heavy_output_throughput,
    bench_latency_to_first_output,
    bench_memory_usage
);

criterion_main!(benches);
