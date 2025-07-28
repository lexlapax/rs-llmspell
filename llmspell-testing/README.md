# llmspell-testing

Comprehensive test suite and testing utilities for rs-llmspell.

## Overview

This crate serves two purposes:

1. **Test Suite**: Consolidates all tests from across the llmspell workspace into a single, well-organized location
2. **Test Utilities**: Provides mocks, fixtures, generators, and benchmarks for testing llmspell applications

## Test Organization

Tests are organized into categories for easy discovery and selective execution:

### Test Categories

- **`unit`** - Unit tests for individual components and functions
- **`integration`** - Cross-crate integration tests verifying component interactions
- **`agents`** - Agent-specific tests including isolation, lifecycle, and behavior
- **`scenarios`** - End-to-end scenario tests simulating real-world usage
- **`lua`** - Lua scripting tests for bridge functionality
- **`performance`** - Performance benchmarks (kept separate from regular tests)

## Running Tests

### Run All Tests
```bash
cargo test -p llmspell-testing --features all-tests
```

### Run Specific Category
```bash
# Unit tests only
cargo test -p llmspell-testing --features unit-tests

# Integration tests only
cargo test -p llmspell-testing --features integration-tests

# Agent tests only
cargo test -p llmspell-testing --features agent-tests

# Scenario tests only
cargo test -p llmspell-testing --features scenario-tests

# Lua tests only
cargo test -p llmspell-testing --features lua-tests
```

### Run Multiple Categories
```bash
cargo test -p llmspell-testing --features "unit-tests,integration-tests"
```

### Run Tests with Output
```bash
cargo test -p llmspell-testing --features all-tests -- --nocapture
```

### Run Specific Test
```bash
cargo test -p llmspell-testing --features agent-tests test_strict_isolation
```

## Performance Benchmarks

Performance benchmarks are managed separately and will be integrated in Task 5.7.2.

Currently, run benchmarks from the performance crate:
```bash
cd tests/performance
cargo bench
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
├── src/              # Test utilities (mocks, generators, etc.)
├── tests/            # Organized test suites
│   ├── unit/         # Unit tests by crate
│   ├── integration/  # Integration tests
│   ├── agents/       # Agent-specific tests
│   ├── scenarios/    # End-to-end scenarios
│   └── lua/          # Lua scripting tests
├── fixtures/         # Test data and fixtures
│   ├── data/         # JSON/YAML test data
│   └── lua/          # Lua test scripts
└── benches/          # Performance benchmarks (Task 5.7.2)
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

The test suite is designed to integrate seamlessly with CI/CD pipelines. See Task 5.7.6 for CI/CD updates.

## Contributing

When contributing tests:
1. Follow the existing organization structure
2. Add appropriate documentation
3. Use provided utilities where applicable
4. Ensure tests are deterministic and isolated
5. Add fixtures to the `fixtures/` directory
6. Update this README if adding new categories or utilities

## Future Enhancements

- [ ] Unified test runner CLI (Task 5.7.3)
- [ ] Test categorization attributes (Task 5.7.4)
- [ ] Performance benchmark integration (Task 5.7.2)
- [ ] Coverage reporting integration
- [ ] Test result visualization