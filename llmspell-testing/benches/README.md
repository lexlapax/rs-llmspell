# llmspell-testing Benchmarks

Performance benchmarks for the llmspell framework.

## Overview

This directory contains comprehensive performance benchmarks using Criterion.rs to measure and track performance across the llmspell framework.

## Benchmark Categories

### Core Performance
- `minimal_test` - Minimal benchmark for testing setup
- `event_throughput` - Event system throughput measurements
- `event_throughput_simple` - Simplified event throughput tests

### Hook System
- `hook_overhead` - Hook execution overhead measurements
- `hook_overhead_simple` - Simplified hook overhead tests
- `circuit_breaker` - Circuit breaker performance under load

### State Persistence
- `state_persistence` - State persistence operation benchmarks
- `state_operations` - Individual state operation performance
- `migration_performance` - State migration speed tests
- `integrated_overhead` - System-wide integration overhead

### Cross-Language
- `cross_language` - Lua/JavaScript bridge performance

## Running Benchmarks

### Run All Benchmarks
```bash
cargo bench -p llmspell-testing
```

### Run Specific Benchmark
```bash
cargo bench -p llmspell-testing <benchmark_name>

# Examples:
cargo bench -p llmspell-testing state_persistence
cargo bench -p llmspell-testing hook_overhead
```

### Run with Baseline Comparison
```bash
# Save baseline
cargo bench -p llmspell-testing -- --save-baseline my_baseline

# Compare against baseline
cargo bench -p llmspell-testing -- --baseline my_baseline
```

### Generate HTML Reports
Criterion automatically generates HTML reports in `target/criterion/`.

## Performance Targets

Key performance targets based on architecture requirements:

- **Hook Overhead**: <1% for simple hooks, <5% for complex hooks
- **Event Throughput**: >90K events/second
- **State Operations**: <10ms latency at 99th percentile
- **Circuit Breaker**: <1ms decision time
- **Migration**: <1ms per transformation
- **Cross-Language**: <5ms bridge overhead

## Benchmark Development

### Adding New Benchmarks

1. Create a new file in `benches/` directory
2. Add the benchmark to `Cargo.toml`:
   ```toml
   [[bench]]
   name = "your_benchmark"
   harness = false
   ```
3. Use Criterion for consistent measurements:
   ```rust
   use criterion::{criterion_group, criterion_main, Criterion};
   
   fn bench_your_feature(c: &mut Criterion) {
       c.bench_function("your_test", |b| {
           b.iter(|| {
               // Your benchmark code
           });
       });
   }
   
   criterion_group!(benches, bench_your_feature);
   criterion_main!(benches);
   ```

### Best Practices

1. **Isolate What You Measure**: Benchmark only the code you're interested in
2. **Use Realistic Data**: Test with production-like data sizes and patterns
3. **Warm Up**: Let the JIT compiler optimize before measuring
4. **Multiple Iterations**: Criterion handles this automatically
5. **Document Targets**: Include expected performance in comments

## Continuous Performance Tracking

The benchmarks are designed to be run in CI to catch performance regressions:

```yaml
# In GitHub Actions
- name: Run Benchmarks
  run: cargo bench -p llmspell-testing -- --output-format bencher | tee output.txt
```

## Historical Data

Benchmark results are stored in:
- `target/criterion/` - Local results and HTML reports
- CI artifacts - For regression tracking

## Scripts

- `run-performance-tests.sh` - Run all performance tests with analysis
- `run-simple-tests.sh` - Quick performance smoke tests

## Troubleshooting

### Benchmarks Taking Too Long
- Reduce sample size in the benchmark group
- Use `--profile-time` flag to limit profiling time
- Run specific benchmarks instead of all

### Inconsistent Results
- Ensure system is idle during benchmarking
- Disable CPU frequency scaling
- Run with `--noplot` to skip HTML generation
- Use `taskset` to pin to specific CPU cores

### Out of Memory
- Reduce data sizes in benchmarks
- Run benchmarks individually
- Increase system swap space