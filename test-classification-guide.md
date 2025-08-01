# Test Classification Guide for rs-llmspell

## Overview
This guide defines the test categorization system for rs-llmspell. All tests must be properly categorized to enable selective test execution and maintain CI performance.

## Core Test Categories

### 1. Unit Tests (`#[cfg_attr(test_category = "unit")]`)
- **Location**: `src/**/*.rs` files (inline with code)
- **Characteristics**:
  - Fast execution (<100ms per test)
  - No external dependencies
  - No network calls
  - No file I/O (except test fixtures)
  - Isolated component testing
- **Example**:
  ```rust
  #[test]
  #[cfg_attr(test_category = "unit")]
  fn test_agent_metadata() {
      let metadata = ComponentMetadata::new("test-agent");
      assert_eq!(metadata.name(), "test-agent");
  }
  ```

### 2. Integration Tests (`#[cfg_attr(test_category = "integration")]`)
- **Location**: `tests/**/*.rs` files
- **Characteristics**:
  - Cross-component testing
  - May use multiple crates
  - No external services
  - Mocked external dependencies
  - <5s execution time
- **Example**:
  ```rust
  #[tokio::test]
  #[cfg_attr(test_category = "integration")]
  async fn test_agent_tool_interaction() {
      let agent = create_test_agent();
      let tool = create_mock_tool();
      // Test interaction between components
  }
  ```

### 3. External Tests (`#[cfg_attr(test_category = "external")]`)
- **Location**: `tests/**/*_integration.rs` or `tests/**/*_external.rs`
- **Characteristics**:
  - Requires network access
  - Calls real APIs (OpenAI, Anthropic, etc.)
  - Requires credentials/API keys
  - May be slow (>5s)
  - Should be marked with `#[ignore]` by default
- **Example**:
  ```rust
  #[tokio::test]
  #[cfg_attr(test_category = "external")]
  #[ignore] // Run with --ignored flag
  async fn test_openai_provider() {
      let api_key = std::env::var("OPENAI_API_KEY").unwrap();
      let provider = OpenAIProvider::new(api_key);
      // Test real API calls
  }
  ```

### 4. Benchmark Tests (`#[cfg_attr(test_category = "benchmark")]`)
- **Location**: `benches/**/*.rs` files
- **Characteristics**:
  - Measures performance metrics (speed, throughput, memory)
  - Uses Criterion framework
  - Requires multiple iterations for statistical accuracy
  - Should run in isolated environments
  - Not part of regular test suite
  - Validates performance requirements
- **Example**:
  ```rust
  use criterion::{black_box, criterion_group, criterion_main, Criterion};
  
  fn bench_tool_initialization(c: &mut Criterion) {
      c.bench_function("calculator_tool_init", |b| {
          b.iter(|| {
              let tool = CalculatorTool::new();
              black_box(tool)
          })
      });
  }
  
  criterion_group!(benches, bench_tool_initialization);
  criterion_main!(benches);
  ```

## Component-Specific Categories

Tests should also include component categories when applicable:

### Tool Tests (`#[cfg_attr(test_category = "tool")]`)
- Tests for tool implementations
- Located in `llmspell-tools/tests/`

### Agent Tests (`#[cfg_attr(test_category = "agent")]`)
- Tests for agent functionality
- Located in `llmspell-agents/tests/`

### Workflow Tests (`#[cfg_attr(test_category = "workflow")]`)
- Tests for workflow patterns
- Located in `llmspell-workflows/tests/`

### Bridge Tests (`#[cfg_attr(test_category = "bridge")]`)
- Tests for Lua/JavaScript integration
- Located in `llmspell-bridge/tests/`

### Security Tests (`#[cfg_attr(test_category = "security")]`)
- Security-specific tests
- Input validation, sandboxing, etc.

### Performance Tests (`#[cfg_attr(test_category = "performance")]`)
- Performance validation tests (not benchmarks)
- Tests that verify performance requirements are met
- Different from benchmarks - these are pass/fail tests

## Multiple Categories

Tests can have multiple categories:

```rust
#[test]
#[cfg_attr(test_category = "unit")]
#[cfg_attr(test_category = "tool")]
#[cfg_attr(test_category = "security")]
fn test_tool_input_validation() {
    // Fast unit test for tool security
}
```

## Test Execution Commands

### Fast Test Suite (Unit + Integration)
```bash
cargo test -p llmspell-testing --features fast-tests
# Should complete in <35 seconds
# Ideal for CI/PR checks and development feedback
```

### Comprehensive Test Suite (All but External)
```bash
cargo test -p llmspell-testing --features comprehensive-tests
# Includes unit, integration, component, security, and performance tests
# Excludes external tests requiring network/API keys
```

### External Test Suite (Network/API Tests)
```bash
cargo test -p llmspell-testing --features external-tests -- --ignored
# Requires environment variables for API keys (OPENAI_API_KEY, etc.)
# Run manually or in nightly CI builds
```

### All Tests (Complete Suite)
```bash
cargo test -p llmspell-testing --features all-tests -- --include-ignored
# Runs everything including external tests
# Use for release validation
```

### Benchmark Tests
```bash
cargo bench -p llmspell-testing --features benchmark-tests
# Performance measurement, not pass/fail tests
# Run in isolated environment for accurate results
```

### Component-Specific Tests
```bash
cargo test -p llmspell-testing --features tool-tests
cargo test -p llmspell-testing --features agent-tests  
cargo test -p llmspell-testing --features workflow-tests
cargo test -p llmspell-testing --features bridge-tests
```

### Security/Performance Tests
```bash
cargo test -p llmspell-testing --features security-tests
cargo test -p llmspell-testing --features performance-tests
```

## Migration Guidelines

### For Existing Tests Without Categories

1. **Identify test type**:
   - Does it measure performance? → `benchmark`
   - Does it access network? → `external`
   - Does it test multiple components? → `integration`
   - Is it fast and isolated? → `unit`

2. **Add appropriate attributes**:
   ```rust
   // Before
   #[test]
   fn my_test() { ... }
   
   // After
   #[test]
   #[cfg_attr(test_category = "unit")]
   fn my_test() { ... }
   ```

3. **Add component category if applicable**:
   ```rust
   #[test]
   #[cfg_attr(test_category = "integration")]
   #[cfg_attr(test_category = "tool")]
   fn test_json_processor_integration() { ... }
   ```

4. **Mark external tests as ignored**:
   ```rust
   #[tokio::test]
   #[cfg_attr(test_category = "external")]
   #[ignore]
   async fn test_real_api_call() { ... }
   ```

## CI Configuration

The CI pipeline should be configured to:

1. **Default PR checks**: Run only unit and integration tests
2. **Nightly builds**: Run all tests including external
3. **Release builds**: Run comprehensive test suite

## Best Practices

1. **Keep unit tests fast**: Aim for <100ms per test
2. **Mock external dependencies**: Use mocks for integration tests
3. **Document test requirements**: Note any special setup needed
4. **Use descriptive names**: Make test purpose clear from the name
5. **Group related tests**: Use modules to organize tests
6. **Clean up resources**: Ensure tests don't leave artifacts