# Test Execution Fix Documentation

## Issue Identified

The test categorization system using `#[cfg_attr(test_category = "category")]` syntax is incorrect. This pattern doesn't work in Rust because:

1. `cfg_attr` expects a condition (that evaluates to boolean) and then attributes to apply
2. The syntax `test_category = "category"` is not a valid cfg condition
3. This caused compilation errors across the codebase

## Correct Approach

We should use feature flags directly in the `cfg` attributes:

### Instead of:
```rust
#[cfg_attr(test_category = "unit")]
#[test]
fn my_test() { ... }
```

### Use:
```rust
#[cfg(feature = "unit-tests")]
#[test]
fn my_test() { ... }
```

## Implementation Strategy

1. **Keep existing test categorization** - The 536 test files that were categorized should have their `cfg_attr` lines removed or replaced
2. **Use feature-based compilation** - Tests are compiled only when their feature is enabled
3. **Centralize test execution** - Use the `llmspell-testing` crate as the primary test runner

## Updated Test Execution Commands

```bash
# Fast tests (unit + integration)
cargo test -p llmspell-testing --features "unit-tests,integration-tests"

# Comprehensive tests (everything except external)
cargo test -p llmspell-testing --features "comprehensive-tests"

# External tests
cargo test -p llmspell-testing --features "external-tests" -- --ignored

# All tests
cargo test -p llmspell-testing --features "all-tests" -- --include-ignored
```

## Next Actions

1. Remove invalid `cfg_attr` lines from test files
2. Instead organize tests by location and use feature flags in Cargo.toml
3. Update CI/CD to use the new test execution approach
4. Update documentation to reflect the correct approach