# Test Categorization Guide

This guide explains the test categorization system used in llmspell-testing.

## Overview

Tests in llmspell are categorized across multiple dimensions to enable fine-grained selection and execution. This allows developers to run specific subsets of tests based on their needs.

## Category Dimensions

### 1. Speed Categories

- **Fast** (`Speed::Fast`): Tests completing in <100ms
- **Slow** (`Speed::Slow`): Tests completing in <5s
- **Very Slow** (`Speed::VerySlow`): Tests taking >5s

### 2. Scope Categories

- **Unit** (`Scope::Unit`): Isolated tests of individual functions/methods
- **Integration** (`Scope::Integration`): Tests verifying component interactions
- **E2E** (`Scope::E2E`): End-to-end scenario tests

### 3. Component Categories

- **Core** (`Component::Core`): Core traits and types
- **Agents** (`Component::Agents`): Agent functionality
- **Tools** (`Component::Tools`): Tool implementations
- **Workflows** (`Component::Workflows`): Workflow patterns
- **Bridge** (`Component::Bridge`): Lua/JavaScript integration
- **State** (`Component::State`): State persistence
- **Events** (`Component::Events`): Event system
- **Hooks** (`Component::Hooks`): Hook system
- **Security** (`Component::Security`): Security features
- **Utils** (`Component::Utils`): Utility functions

### 4. Priority Levels

- **Critical** (`Priority::Critical`): Must pass for release
- **High** (`Priority::High`): Should pass for release
- **Medium** (`Priority::Medium`): Good to have passing
- **Low** (`Priority::Low`): Nice to have passing

### 5. Stability Categories

- **Stable** (`Stability::Stable`): Reliable tests
- **Flaky** (`Stability::Flaky`): Occasionally failing tests
- **Experimental** (`Stability::Experimental`): New or experimental tests

### 6. External Dependencies

- **Network**: Requires network access
- **LLM**: Requires specific LLM provider
- **Database**: Requires database connection
- **FileSystem**: Requires file system access
- **EnvVar**: Requires specific environment variables
- **Service**: Requires external service

## Using Test Categories

### Method 1: Using Macros (Recommended)

```rust
use llmspell_testing::{test_category, requires_network, slow_test};

#[test]
#[test_category(unit)]
fn test_basic_functionality() {
    // Fast unit test
}

#[test]
#[test_category(integration)]
#[requires_network!()]
#[slow_test!()]
fn test_api_integration() {
    // Slow integration test requiring network
}

#[test]
#[test_category(agent)]
#[requires_llm!("openai")]
fn test_openai_agent() {
    // Test requiring OpenAI
}
```

### Method 2: Using Attributes API

```rust
use llmspell_testing::attributes::*;

fn categorize_test() -> TestCategory {
    TestCategory::new(Speed::Slow, Scope::Integration)
        .with_component(Component::Agents)
        .with_priority(Priority::High)
        .with_dependency(Dependency::Network)
        .with_tag("regression")
}
```

### Method 3: Using Predefined Categories

```rust
use llmspell_testing::tests::categories::*;

let category = agent_test_category()
    .with_dependency(Dependency::LLM("anthropic".to_string()));
```

## Running Categorized Tests

### Using the Test Runner

```bash
# Run only fast unit tests
llmspell-test run unit --filter fast

# Run integration tests without network
llmspell-test run integration --offline

# Run critical tests only
llmspell-test run all --priority critical

# Run stable tests only (exclude flaky/experimental)
llmspell-test run all --stable-only
```

### Using Cargo Features

```bash
# Run unit tests
cargo test -p llmspell-testing --features unit-tests

# Run network tests
cargo test -p llmspell-testing --features network-tests

# Run slow tests
cargo test -p llmspell-testing --features slow-tests

# Run component-specific tests
cargo test -p llmspell-testing --features agent-tests,state-tests
```

## Test Organization Best Practices

### 1. Categorize Appropriately

- Be honest about test speed
- Mark flaky tests appropriately
- Set correct priority levels
- Document external dependencies

### 2. Use Multiple Categories

Tests often belong to multiple categories:

```rust
#[test]
#[test_category(integration)]
#[test_category(agent)]
#[requires_network!()]
#[slow_test!()]
fn test_agent_network_operation() {
    // Test implementation
}
```

### 3. Group Related Tests

Use the `categorized_test_module!` macro for grouping:

```rust
categorized_test_module! {
    name: state_persistence_tests,
    category: state,
    tags: [integration, slow],
    
    tests {
        #[test]
        fn test_save_state() {
            // Test implementation
        }
        
        #[test]
        fn test_load_state() {
            // Test implementation
        }
    }
}
```

## CI/CD Integration

### Running Tests in CI

```yaml
# Run stable, high-priority tests
- run: llmspell-test run all --stable-only --priority high

# Run offline tests only
- run: llmspell-test run all --offline

# Run specific component tests
- run: cargo test -p llmspell-testing --features core-tests,utils-tests
```

### Conditional Test Execution

Use the `skip_if!` macro for runtime conditions:

```rust
#[test]
fn test_with_conditions() {
    skip_if!(ci);  // Skip in CI
    skip_if!(no_network);  // Skip if no network
    skip_if!(env_not_set: "API_KEY");  // Skip if env var not set
    
    // Test implementation
}
```

## Migration Guide

### For Existing Tests

1. Add appropriate category macros:
   ```rust
   #[test]
   #[test_category(unit)]  // Add this
   fn existing_test() {
       // ...
   }
   ```

2. Mark slow tests:
   ```rust
   #[test]
   #[slow_test!()]  // Add if test takes >5s
   fn slow_existing_test() {
       // ...
   }
   ```

3. Document dependencies:
   ```rust
   #[test]
   #[requires_network!()]  // Add if needs network
   fn network_test() {
       // ...
   }
   ```

### For New Tests

1. Always categorize new tests
2. Set appropriate priority
3. Document external dependencies
4. Consider stability (mark experimental if new)

## Examples

### Example 1: Basic Unit Test

```rust
#[test]
#[test_category(unit)]
fn test_component_creation() {
    let component = Component::new();
    assert!(component.is_valid());
}
```

### Example 2: Integration Test with Dependencies

```rust
#[test]
#[test_category(integration)]
#[requires_network!()]
#[requires_llm!("openai")]
#[slow_test!()]
fn test_llm_integration() {
    skip_if!(env_not_set: "OPENAI_API_KEY");
    
    // Test implementation
}
```

### Example 3: Flaky Test

```rust
#[test]
#[test_category(integration)]
#[flaky_test!()]
fn test_occasional_timeout() {
    // Test that sometimes fails due to timeouts
}
```

### Example 4: Security Test

```rust
use llmspell_testing::tests::categories::security_test_category;

#[test]
fn test_input_validation() {
    let category = security_test_category()
        .with_priority(Priority::Critical);
    
    // Security test implementation
}
```

## Troubleshooting

### Tests Not Running

1. Check feature flags are enabled
2. Verify category matches filter
3. Check for `#[ignore]` attributes
4. Ensure dependencies are available

### Category Conflicts

If a test has conflicting categories:
- Speed takes precedence (Fast < Slow < VerySlow)
- Stability affects reliability (Stable > Flaky > Experimental)
- Priority affects importance (Critical > High > Medium > Low)

### Performance Impact

The categorization system has minimal runtime overhead:
- Categories are evaluated at compile time where possible
- Filters are applied before test execution
- No impact on test execution speed