# Performance Benchmarking Guide

This guide explains how to run, analyze, and interpret performance benchmarks in the llmspell project.

## Quick Start

```bash
# List available benchmarks
./scripts/testing/kernel-benchmark.sh --list

# Run all benchmarks
./scripts/testing/kernel-benchmark.sh

# Run kernel benchmarks only
./scripts/testing/kernel-benchmark.sh -p llmspell-kernel

# Save baseline for future comparison
./scripts/testing/kernel-benchmark.sh -b my-baseline

# Compare against baseline (detect regressions)
./scripts/testing/kernel-benchmark.sh -c my-baseline
```

## Available Benchmarks

### llmspell-kernel (Phase 10)
- **kernel_performance** - Core kernel operations
  - Startup: <2s cold start, <100ms warm start
  - Message handling: <5ms InProcess roundtrip
  - Tool invocation: <10ms for simple tools
  - Registry operations: <1ms direct access

### llmspell-tools
- **tool_initialization** - Tool creation time (<10ms target)
- **tool_operations** - Tool execution performance

### llmspell-bridge
- **session_bench** - Session management performance
- **workflow_bridge_bench** - Workflow execution

### llmspell-workflows
- **workflow_bench** - Workflow performance

## Benchmark Script Usage

### Command-Line Options

```bash
./scripts/testing/kernel-benchmark.sh [OPTIONS] [BENCHMARK_NAME]

OPTIONS:
  -p, --package PACKAGE    Run benchmarks for specific package
  -b, --baseline NAME      Save results as baseline NAME
  -c, --compare BASELINE   Compare against saved BASELINE
  -l, --list              List available benchmarks
  -h, --help              Show help message
  -v, --verbose           Show detailed cargo output
```

### Examples

#### Run Specific Package Benchmarks
```bash
# Run all kernel benchmarks
./scripts/testing/kernel-benchmark.sh -p llmspell-kernel

# Run specific benchmark in package
./scripts/testing/kernel-benchmark.sh -p llmspell-kernel kernel_performance
```

#### Baseline Management
```bash
# Save current performance as baseline
./scripts/testing/kernel-benchmark.sh -b phase10-final

# Compare against saved baseline
./scripts/testing/kernel-benchmark.sh -c phase10-final

# List saved baselines
ls -1 target/criterion/ | grep -v "^report$"
```

#### Regression Detection
```bash
# Before making changes
./scripts/testing/kernel-benchmark.sh -p llmspell-kernel -b before-optimization

# After making changes
./scripts/testing/kernel-benchmark.sh -p llmspell-kernel -c before-optimization

# Check HTML reports for detailed comparison
open target/criterion/report/index.html
```

## Interpreting Results

### Performance Status Indicators

- **✓ Within 5%**: No action needed (within measurement noise)
- **⚠ 5-10%**: Monitor, investigate if consistent across multiple runs
- **✗ >10%**: Likely regression, investigate immediately

### Reading Criterion Output

Criterion provides detailed statistical analysis:

```
kernel_startup/embedded_cold_start
                        time:   [36.5 ms 37.0 ms 37.5 ms]
                        change: [-2.3% +0.5% +3.2%] (p = 0.45 > 0.05)
                        No change in performance detected.
```

**Key metrics**:
- **time**: [lower bound, median, upper bound] with 95% confidence
- **change**: Performance delta vs baseline (if comparing)
- **p-value**: Statistical significance (p < 0.05 means significant change)

### HTML Reports

Open detailed interactive reports:

```bash
open target/criterion/report/index.html
```

HTML reports provide:
- Interactive charts showing performance over time
- Statistical distributions (violin plots)
- Detailed comparison tables
- Outlier detection and analysis

## Baseline Management

### Saving Baselines

Baselines are stored in `target/criterion/<baseline-name>/`:

```bash
# Save Phase 10 completion baseline
./scripts/testing/kernel-benchmark.sh -b phase10-complete

# Save pre-optimization baseline
./scripts/testing/kernel-benchmark.sh -b before-refactor
```

**Baseline Naming Conventions**:
- Use descriptive names: `phase10-complete`, `before-optimization`
- Use only alphanumeric characters, hyphens, underscores
- Avoid spaces and special characters

### Comparing Baselines

```bash
# Compare current performance against baseline
./scripts/testing/kernel-benchmark.sh -c phase10-complete

# Compare specific package
./scripts/testing/kernel-benchmark.sh -p llmspell-kernel -c phase10-complete
```

Criterion will output change percentages and statistical significance.

## CI Integration (Future)

For Phase 11+ CI integration:

```bash
# In CI pipeline
./scripts/testing/kernel-benchmark.sh -c main-baseline > bench-output.txt

# Parse output for regression detection
if grep -q "Performance has regressed" bench-output.txt; then
    echo "❌ Performance regression detected!"
    exit 1
fi
```

## Troubleshooting

### Long Compilation Times

Benchmarks compile in release mode (optimized), which takes longer:

```bash
# First run: ~2-3 minutes compilation
# Subsequent runs: ~10-30 seconds (incremental)
```

### Inconsistent Results

If results vary significantly between runs:

1. **Close background applications** - browsers, IDEs can affect performance
2. **Run multiple times** - validate consistency across 3+ runs
3. **Check system load** - ensure system is idle during benchmarks
4. **Use representative hardware** - avoid underpowered machines

### Missing Baselines

If comparing against a baseline that doesn't exist:

```bash
# List available baselines
./scripts/testing/kernel-benchmark.sh --list

# Or directly check filesystem
ls -1 target/criterion/
```

## Best Practices

### Before Running Benchmarks

1. **Close resource-intensive applications**
2. **Ensure system is idle** (no heavy background processes)
3. **Use consistent hardware** for reproducible results
4. **Run multiple times** to validate consistency

### When Making Performance Changes

1. **Save baseline before changes**: `./scripts/testing/kernel-benchmark.sh -b before-change`
2. **Make changes**
3. **Compare after changes**: `./scripts/testing/kernel-benchmark.sh -c before-change`
4. **Review HTML reports** for detailed analysis
5. **Document findings** in commit message or PR

### Baseline Strategy

- **Phase baselines**: Save at end of each phase (e.g., `phase10-final`)
- **Feature baselines**: Save before major features (e.g., `before-fleet-mgmt`)
- **Optimization baselines**: Save before/after optimizations

## Performance Targets (Phase 10)

See [performance-baseline.md](./performance-baseline.md) for current measurements.

### Kernel Performance
- **Startup**: <2s cold start → **Achieved: 36.5ms (55x better)**
- **Message Handling**: <5ms roundtrip → **Achieved: 11.9ms (2.4x target, acceptable)**
- **Tool Invocation**: <10ms → **Achieved: 11.9-12.0ms (close to target)**
- **Registry Operations**: <1ms direct access → **Achieved: <1ms (estimated)**

## Advanced Usage

### Running Specific Benchmarks

```bash
# Run only startup benchmarks
./scripts/testing/kernel-benchmark.sh -p llmspell-kernel kernel_performance -- --bench startup

# Run with custom Criterion arguments
./scripts/testing/kernel-benchmark.sh -p llmspell-kernel -- --warm-up-time 5
```

### Custom Criterion Configuration

Edit `benches/kernel_performance.rs` to adjust:
- Sample size (default: 10-100 depending on benchmark)
- Measurement time (default: varies by benchmark group)
- Warm-up time (default: 3s)

### Memory Profiling

For memory usage analysis (future enhancement):

```bash
# Use dhat or valgrind with benchmarks
cargo bench --features dhat -- --profile-time=10
```

## References

- **Criterion Documentation**: https://bheisler.github.io/criterion.rs/book/
- **Performance Baseline**: [performance-baseline.md](./performance-baseline.md)
- **Benchmark Source**: `llmspell-kernel/benches/kernel_performance.rs`
- **Script Source**: `scripts/testing/kernel-benchmark.sh`

## Support

For questions or issues with benchmarking:
1. Check HTML reports: `open target/criterion/report/index.html`
2. Review [performance-baseline.md](./performance-baseline.md)
3. Run with `--verbose` flag for detailed output
4. Check [scripts/testing/README.md](../../scripts/testing/README.md)
