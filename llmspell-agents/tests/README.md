# Agent Testing Infrastructure

This directory contains the comprehensive testing infrastructure for the LLMSpell agent system. The testing framework provides utilities for unit testing, integration testing, and scenario-based testing of agent implementations.

## Overview

The testing infrastructure consists of several key components:

1. **Test Framework** (`src/testing/framework.rs`) - Core testing utilities and harness
2. **Mock Implementations** (`src/testing/mocks.rs`) - Configurable mock agents and tools
3. **Test Scenarios** (`src/testing/scenarios.rs`) - Pre-defined test scenarios
4. **Test Utilities** (`src/testing/utils.rs`) - Helper functions and data generators

## Test Categories

### Unit Tests

Individual component tests are located throughout the codebase in `#[cfg(test)]` modules. These test specific functionality in isolation.

### Integration Tests

- **`lifecycle_tests.rs`** - Tests agent lifecycle management, state transitions, and resource management
- **`communication_tests.rs`** - Tests agent-to-agent communication patterns
- **`integration_tests.rs`** - Tests factory, registry, and dependency injection
- **`scenario_tests.rs`** - Tests using pre-defined scenarios

## Using the Test Framework

### Creating a Test Harness

```rust
use llmspell_agents::testing::{TestConfig, TestHarness};

let config = TestConfig {
    timeout: Duration::from_secs(30),
    debug: true,
    record_interactions: true,
    profile_performance: true,
    validate_resources: true,
    metadata: HashMap::new(),
};

let harness = TestHarness::new(config);
```

### Using Mock Agents

```rust
use llmspell_agents::testing::{MockAgentBuilder, TestDoubles};

// Create a custom mock agent
let agent = MockAgentBuilder::new("test_agent")
    .agent_type("basic")
    .with_tool("calculator")
    .with_response(Some("hello".to_string()), "Hello response")
    .with_delay(Duration::from_millis(100))
    .build();

// Or use pre-defined test doubles
let echo_agent = TestDoubles::echo_agent("echo");
let failing_agent = TestDoubles::failing_agent("fail", "Error message");
let tool_agent = TestDoubles::tool_agent("tools", vec!["calc", "search"]);
```

### Running Test Scenarios

```rust
use llmspell_agents::testing::{TestScenarios, ScenarioRunner};

// Use pre-defined scenarios
let scenario = TestScenarios::echo_scenario();
let result = ScenarioRunner::run_scenario(&agent, &scenario).await?;

// Or create custom scenarios
let custom_scenario = ScenarioConfig {
    name: "Custom Test".to_string(),
    description: "Custom scenario".to_string(),
    inputs: vec![AgentInput::text("test")],
    expected_outputs: vec![ExpectedOutput::Contains("response")],
    timeout: Duration::from_secs(5),
};
```

## Test Utilities

### Data Generators

```rust
use llmspell_agents::testing::TestDataGenerator;

// Generate test inputs
let random_input = TestDataGenerator::random_input();
let media_input = TestDataGenerator::input_with_media();
let complex_input = TestDataGenerator::complex_input();

// Generate test contexts
let context = TestDataGenerator::context_with_metadata();
```

### Test Configurations

```rust
use llmspell_agents::testing::TestConfigs;

// Pre-configured agent setups
let basic_config = TestConfigs::basic_agent();
let tool_config = TestConfigs::tool_agent();
let limited_config = TestConfigs::limited_agent();
```

### Assertions

```rust
use llmspell_agents::testing::AgentAssertions;

// Assert output content
AgentAssertions::assert_output_contains(&output, "expected text")?;

// Assert tool calls
AgentAssertions::assert_tool_calls(&output, &["calculator", "search"])?;

// Assert performance
AgentAssertions::assert_execution_time(duration, max_duration)?;

// Assert resource usage
AgentAssertions::assert_resource_usage(&usage, &limits)?;
```

## Running Tests

```bash
# Run all tests
cargo test -p llmspell-agents

# Run specific test category
cargo test -p llmspell-agents lifecycle
cargo test -p llmspell-agents communication
cargo test -p llmspell-agents integration
cargo test -p llmspell-agents scenario

# Run with output
cargo test -p llmspell-agents -- --nocapture

# Run specific test
cargo test -p llmspell-agents test_agent_lifecycle
```

## Performance Testing

The framework includes performance measurement utilities:

```rust
use llmspell_agents::testing::PerformanceMeasure;

let measure = PerformanceMeasure::start("operation");
// ... perform operation ...
let duration = measure.end(); // Logs duration
```

## Test Reports

Generate test reports using:

```rust
use llmspell_agents::testing::TestReport;

let mut report = TestReport::new();
report.add_result("test1", true, duration, None);
report.add_result("test2", false, duration, Some("Error".to_string()));

println!("{}", report.summary());
println!("Pass rate: {:.2}%", report.pass_rate());
```

## Best Practices

1. **Use the Test Harness** - Provides consistent test execution and metrics
2. **Record Interactions** - Enable interaction recording for debugging
3. **Set Appropriate Timeouts** - Prevent tests from hanging
4. **Validate Resources** - Ensure agents respect resource limits
5. **Use Scenarios** - Test common usage patterns systematically
6. **Mock External Dependencies** - Use mock tools for predictable testing

## Adding New Tests

1. Choose the appropriate test file based on what you're testing
2. Use the test framework utilities for consistency
3. Add assertions to verify expected behavior
4. Document complex test scenarios
5. Run tests locally before committing

## Troubleshooting

- **Timeout Errors** - Increase test timeout in TestConfig
- **Resource Errors** - Check resource limits in agent configuration
- **State Errors** - Verify state machine transitions are valid
- **Mock Failures** - Ensure mock responses match expected inputs