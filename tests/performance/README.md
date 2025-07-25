# LLMSpell Performance Test Suite

This directory contains comprehensive performance tests for the LLMSpell hook and event system.

## Test Categories

### 1. Hook Overhead (`hook_overhead.rs`)
Tests the performance overhead of hooks on agent, tool, and workflow operations.

**Key Metrics:**
- Hook registration time: <0.1ms target
- Hook execution overhead: <5% target
- Memory usage per hook: minimal

**Test Scenarios:**
- Agent execution with/without hooks
- Tool execution with multiple hooks
- Workflow execution with stage hooks
- Hook registration performance

### 2. Event Throughput (`event_throughput.rs`)
Validates the event system can handle 100K+ events per second.

**Key Metrics:**
- Publishing throughput: >100K events/sec
- End-to-end throughput: >100K events/sec
- Subscription overhead: minimal
- Pattern matching performance

**Test Scenarios:**
- Basic event publishing (1K, 10K, 100K events)
- Concurrent publishers (1000 publishers)
- Event correlation chains
- High-frequency event scenarios
- Memory usage under load

### 3. Circuit Breaker (`circuit_breaker.rs`)
Tests circuit breaker effectiveness under various failure conditions.

**Key Metrics:**
- Failure detection: 5 failures trigger
- Recovery time: configurable timeout
- Rejection rate when open: 100%
- Performance impact: minimal

**Test Scenarios:**
- Normal load (no failures)
- Failure detection and opening
- Recovery (half-open to closed)
- Performance monitor integration
- Concurrent load handling

### 4. Cross-Language Bridge (`cross_language.rs`)
Measures overhead of cross-language operations (Lua↔Rust↔JavaScript).

**Key Metrics:**
- Bridge overhead: <10% target
- Serialization time: <1ms per event
- Lua hook overhead: minimal
- Event propagation latency

**Test Scenarios:**
- Lua→Rust hook execution
- Cross-language event propagation
- UniversalEvent serialization
- JavaScript bridge simulation
- Multi-language coordination

## Running Performance Tests

### Run All Tests
```bash
cd tests/performance
cargo bench
```

### Run Specific Test
```bash
cargo bench hook_overhead
cargo bench event_throughput
cargo bench circuit_breaker
cargo bench cross_language
```

### Generate HTML Reports
```bash
cargo bench -- --plotting-backend gnuplot
```

Reports will be generated in `target/criterion/`.

### Run with Profiling
```bash
CARGO_PROFILE_BENCH_DEBUG=true cargo bench -- --profile-time=10
```

## Performance Targets

| Metric | Target | Test |
|--------|--------|------|
| Hook Overhead | <5% | hook_overhead |
| Event Throughput | >100K/sec | event_throughput |
| Circuit Breaker Response | <5ms | circuit_breaker |
| Cross-Language Overhead | <10% | cross_language |
| Memory Leaks | None | All tests |

## CI Integration

These tests are automatically run in CI with the following checks:
- All benchmarks must complete successfully
- Performance regression detection (>10% slowdown fails)
- Memory leak detection via valgrind
- Results stored for historical comparison

## Interpreting Results

### Hook Overhead
```
hook_overhead_percentage/
                        time:   [1.234 ms 1.245 ms 1.256 ms]
                        change: [-2.1% +0.5% +3.2%] (p = 0.72 > 0.05)
```
- Time shows [min, median, max] for the benchmark
- Change shows performance change from baseline
- p-value >0.05 means no significant change

### Event Throughput
```
event_publishing/100000 time:   [850.23 ms 862.45 ms 875.12 ms]
                        thrpt:  [114.3K 116.0K 117.6K]
```
- Shows both time and throughput (events/sec)
- Target is >100K events/sec

### Memory Usage
Run with valgrind to check for leaks:
```bash
valgrind --leak-check=full --show-leak-kinds=all \
    cargo test --test performance -- --nocapture
```

## Adding New Performance Tests

1. Create new test file in `tests/performance/`
2. Add benchmark groups using criterion
3. Include verification of performance targets
4. Update this README with test description
5. Add to CI performance test suite

## Troubleshooting

### Tests Timing Out
- Reduce iteration counts for initial testing
- Check for infinite loops in async code
- Ensure proper tokio runtime usage

### High Variance in Results
- Close other applications
- Run with CPU governor set to performance
- Use `--sample-size` to increase iterations

### Memory Issues
- Use `valgrind` for leak detection
- Check for Arc cycles
- Ensure proper cleanup in benchmarks