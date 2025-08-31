# llmspell-testing

Comprehensive test suite and testing utilities for the rs-llmspell framework.

## Overview

This crate provides:
1. **Unified Test Suite**: All tests for the llmspell workspace organized by category
2. **Test Runner CLI**: Simple command-line interface for test discovery and execution
3. **Testing Utilities**: Mocks, generators, fixtures, and benchmarks for testing
4. **Performance Benchmarks**: Criterion-based benchmarks for performance tracking

## Test Organization

Tests are organized into categories for easy discovery and selective execution:

### Test Categories

- **unit** - Fast, isolated unit tests for individual components
- **integration** - Cross-crate integration tests
- **agent** - Agent-specific functionality tests
- **scenario** - End-to-end scenario tests simulating real usage
- **lua** - Lua scripting bridge tests
- **performance** - Performance benchmarks (Criterion)

## Installation

To use the test runner CLI:

```bash
# Install the test runner
cargo install --path llmspell-testing --features test-runner

# Or run directly from the workspace
cargo run -p llmspell-testing --features test-runner --bin llmspell-test -- --help
```

## Using the Test Runner

### List Available Categories

```bash
# Show all test categories
llmspell-test list

# Show detailed information
llmspell-test list --detailed
```

### Run Tests

```bash
# Run all tests
llmspell-test run all

# Run specific categories
llmspell-test run unit integration

# Run with filter
llmspell-test run unit --filter test_state

# Run in release mode
llmspell-test run all --release

# Generate coverage report
llmspell-test run all --coverage

# Don't capture test output
llmspell-test run unit --nocapture
```

### Run Benchmarks

```bash
# Run all benchmarks
llmspell-test bench

# Run specific benchmark
llmspell-test bench hook_overhead

# Save baseline for comparison
llmspell-test bench --save my-baseline

# Compare with baseline
llmspell-test bench --baseline my-baseline
```

### Get Category Information

```bash
# Show information about a specific category
llmspell-test info unit
```

## Running Tests with Cargo

You can also run tests directly with cargo:

```bash
# Run unit tests
cargo test -p llmspell-testing --features unit-tests

# Run integration tests
cargo test -p llmspell-testing --features integration-tests

# Run all tests
cargo test -p llmspell-testing --features all-tests

# Run benchmarks
cargo bench -p llmspell-testing
```

## Performance Benchmarks

Performance benchmarks have been integrated into llmspell-testing and use Criterion for statistical analysis:

```bash
# Run benchmarks with test runner
llmspell-test bench

# Or with cargo directly
cargo bench -p llmspell-testing

# View results
open target/criterion/report/index.html
```

Benchmark results are saved in `target/criterion` and can be compared across runs.

## Unified Test Execution Framework

The `test_framework` module provides a unified execution engine for tests and benchmarks with automatic workload adaptation:

### TestExecutor Trait

```rust
use llmspell_testing::test_framework::{
    TestExecutor, ExecutionContext, ExecutionMode, WorkloadClass
};
use async_trait::async_trait;

#[async_trait]
impl TestExecutor for MyTestExecutor {
    type Config = MyConfig;
    type Result = MyResult;
    
    async fn execute(&self, context: ExecutionContext<Self::Config>) -> Self::Result {
        let workload = self.adapt_workload(context.mode);
        let event_count = workload.event_count();
        
        // Test logic with automatic telemetry collection
        context.telemetry.record_metric("events", event_count as f64);
        
        // Implementation adapts based on execution mode
        MyResult::new(event_count)
    }
    
    fn default_config(&self) -> Self::Config {
        MyConfig::default()
    }
    
    fn adapt_workload(&self, mode: ExecutionMode) -> WorkloadClass {
        match mode {
            ExecutionMode::Test => WorkloadClass::Small,    // 1K items
            ExecutionMode::Bench => WorkloadClass::Large,   // 100K items  
            ExecutionMode::Stress => WorkloadClass::Stress, // 1M items
            ExecutionMode::CI => WorkloadClass::Medium,     // 10K items
        }
    }
}
```

### Workload Classes

- **Micro**: <100ms, 100 items
- **Small**: <1s, 1K items  
- **Medium**: <10s, 10K items
- **Large**: <60s, 100K items
- **Stress**: Unlimited, 1M+ items

### Criterion Integration

```rust
use llmspell_testing::test_framework::adapters::CriterionAdapter;

fn bench_my_test(c: &mut Criterion) {
    let executor = MyTestExecutor::new();
    CriterionAdapter::new(executor)
        .with_workload(WorkloadClass::Large)
        .bench(c, "my_test");
}
```

### Built-in Telemetry

All executions automatically collect metrics:

```rust
let context = ExecutionContext::test_default(config);
let result = executor.execute(context).await;

// Metrics are automatically collected
let snapshot = context.telemetry.snapshot();
println!("Metrics: {:?}", snapshot.values);
```

## Test Utilities

The crate provides several utilities for writing tests:

### Mocks

```rust
use llmspell_testing::mocks::MockBaseAgent;

let mut mock = MockBaseAgent::new();
mock.expect_execute()
    .returning(|_, _| Ok(AgentOutput::text("test response")));
```

### Fixtures

```rust
use llmspell_testing::fixtures::load_fixture;

let test_data = load_fixture("test_data.json")?;
```

### Generators

```rust
use llmspell_testing::generators::{agent_id_strategy, component_id_strategy};
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_with_random_ids(
        agent_id in agent_id_strategy(),
        component_id in component_id_strategy()
    ) {
        // Test with generated IDs
    }
}
```

## Directory Structure

```
llmspell-testing/
├── src/
│   ├── lib.rs           # Main library exports
│   ├── benchmarks.rs    # Benchmark utilities
│   ├── fixtures.rs      # Test fixtures and data
│   ├── generators.rs    # Property-based test generators
│   ├── mocks.rs         # Mock implementations
│   ├── runner/          # Test runner implementation
│   └── bin/
│       └── test-runner.rs  # CLI binary
├── tests/
│   ├── unit/            # Unit tests
│   ├── integration/     # Integration tests
│   ├── agents/          # Agent tests
│   ├── scenarios/       # Scenario tests
│   └── lua/             # Lua tests
├── benches/             # Performance benchmarks
└── fixtures/            # Test data files
    ├── data/            # JSON/YAML test data
    └── lua/             # Lua test scripts
```

## Adding New Tests

1. Determine the appropriate category for your test
2. Create or update the test file in the corresponding directory
3. If adding a new module, update the category's `mod.rs` file
4. Run the test to ensure it passes:
   ```bash
   cargo test -p llmspell-testing --features <category>-tests your_test_name
   ```

## CI/CD Integration

The test runner integrates seamlessly with CI/CD pipelines:

```yaml
# GitHub Actions example
- name: Run tests
  run: |
    cargo install --path llmspell-testing --features test-runner
    llmspell-test run all --format junit > test-results.xml

# Or use scripts
- name: Run tests
  run: ./scripts/run-llmspell-tests.sh all
```

## Scripts Integration

The following scripts use llmspell-testing:

- `scripts/run-llmspell-tests.sh` - Convenience wrapper
- `scripts/test-by-tag.sh` - Legacy script (delegates to test runner)
- `scripts/quality-check.sh` - Includes test execution
- `scripts/test-coverage.sh` - Coverage reporting

## Contributing

When contributing tests:
1. Follow the existing organization structure
2. Add appropriate documentation
3. Use provided utilities where applicable
4. Ensure tests are deterministic and isolated
5. Add fixtures to the `fixtures/` directory
6. Update this README if adding new categories or utilities

## Troubleshooting

### Tests Not Found

If tests aren't discovered:
1. Ensure the correct feature is enabled
2. Check that test files are in the expected location
3. Verify test function names start with `test_`

### Performance Issues

For faster test execution:
1. Run specific categories instead of `all`
2. Use `--jobs` to control parallelism
3. Skip slow tests with environment variables

### Coverage Reports

If coverage fails:
1. Install `cargo-tarpaulin`: `cargo install cargo-tarpaulin`
2. Ensure all crates are built with coverage flags
3. Check available disk space for report generation

## Future Enhancements

- [x] Unified test runner CLI (Task 5.7.3) ✅
- [ ] Test categorization attributes (Task 5.7.4)
- [x] Performance benchmark integration (Task 5.7.2) ✅
- [x] Coverage reporting integration ✅
- [ ] Test result visualization