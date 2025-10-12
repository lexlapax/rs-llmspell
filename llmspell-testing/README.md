# llmspell-testing

Comprehensive test suite and testing utilities for the rs-llmspell framework.

## Overview

This crate provides:
1. **Unified Test Suite**: All tests for the llmspell workspace organized by category
2. **Testing Utilities**: Mocks, generators, fixtures, and benchmarks for testing
3. **Performance Benchmarks**: Criterion-based benchmarks for performance tracking

## Test Organization

Tests are organized into categories for easy discovery and selective execution:

### Test Categories

- **unit** - Fast, isolated unit tests for individual components
- **integration** - Cross-crate integration tests
- **agent** - Agent-specific functionality tests
- **scenario** - End-to-end scenario tests simulating real usage
- **lua** - Lua scripting bridge tests
- **performance** - Performance benchmarks (Criterion)

## Running Tests

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
# Run all benchmarks
cargo bench -p llmspell-testing

# View results
open target/criterion/report/index.html
```

Benchmark results are saved in `target/criterion` and can be compared across runs.

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
│   └── mocks.rs         # Mock implementations
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

Tests can be integrated into CI/CD pipelines:

```yaml
# GitHub Actions example
- name: Run tests
  run: cargo test --workspace

# Or use scripts
- name: Run tests
  run: ./scripts/testing/test-by-tag.sh all
```

## Scripts Integration

The following scripts use llmspell-testing:

- `scripts/testing/test-by-tag.sh` - Run tests by category tag
- `scripts/quality/quality-check.sh` - Includes test execution
- `scripts/testing/test-coverage.sh` - Coverage reporting

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

- [x] Test categorization attributes ✅
- [x] Performance benchmark integration ✅
- [x] Coverage reporting integration ✅
- [ ] Test result visualization