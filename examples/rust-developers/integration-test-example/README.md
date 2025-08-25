# Integration Test Example

**Complexity Level:** ADVANCED  
**Time to Complete:** ~5 seconds compilation + execution  

## Overview

This example demonstrates comprehensive testing strategies for LLMSpell components including unit testing, integration testing, mocking patterns, and test fixtures. You'll learn professional testing practices for robust component development.

## Key Concepts

- **Unit Testing** - Testing individual components in isolation
- **Integration Testing** - Testing component interactions and workflows
- **Mock Objects** - Test doubles for controlled testing environments
- **Test Fixtures** - Reusable test data and setup patterns
- **Error Testing** - Comprehensive error condition coverage

## What You'll Learn

- Creating testable LLMSpell tools and agents
- Writing unit tests with tokio test framework
- Building mock objects for controlled testing
- Testing error conditions and edge cases
- Integration testing patterns for component interactions

## Testing Components

### EchoTool
- **Purpose:** Simple tool for testing fundamental patterns
- **Features:** Call logging, timestamp generation, parameter echoing
- **Testing Focus:** Basic tool functionality and validation

### MockAgent
- **Purpose:** Controllable agent for integration testing
- **Features:** Predefined responses, call counting, response cycling
- **Testing Focus:** Agent behavior simulation and interaction testing

### TestFixture
- **Purpose:** Reusable test data and setup
- **Features:** Standard test inputs, expected outputs, test scenarios
- **Testing Focus:** Consistent test data across test suites

## Test Categories

### 1. Unit Tests
- **test_echo_tool_basic_functionality** - Core tool operation
- **test_echo_tool_empty_input** - Edge case handling
- **test_echo_tool_parameter_input** - Parameter processing
- **test_echo_tool_call_logging** - Call tracking functionality
- **test_tool_schema** - Schema generation and validation

### 2. Mock Testing
- **test_mock_agent_responses** - Mock agent behavior
- **Controlled Responses** - Predefined agent responses
- **Call Counting** - Interaction tracking and verification

### 3. Integration Tests
- **test_integration_scenario** - End-to-end workflow testing
- **test_integration_scenario_error** - Error propagation testing
- **Component Interaction** - Multi-component workflow verification

### 4. Test Fixtures
- **test_test_fixture** - Reusable test data validation
- **Consistent Data** - Standardized test inputs and outputs

## How to Run

### Run the Demo
```bash
cd integration-test-example
cargo run
```

### Run the Tests
```bash
cargo test
cargo test -- --nocapture  # To see println! output
```

## Expected Output

### Demo Output
- Testing demonstrations with unit tests, integration tests, and mocking
- Mock agent interaction examples
- Error handling demonstrations

### Test Results
- 9 passing unit tests covering all testing patterns
- Zero failures across all test categories
- Comprehensive coverage of success and error scenarios

## Testing Patterns Demonstrated

### Unit Testing Patterns
- **Isolated Testing** - Testing components without dependencies
- **Edge Case Coverage** - Empty inputs, invalid parameters
- **State Verification** - Call logs, counters, timestamps

### Mock Testing Patterns
- **Behavior Simulation** - Predefined responses and behaviors
- **Interaction Verification** - Call counting and parameter tracking
- **Controlled Environments** - Deterministic testing conditions

### Integration Testing Patterns
- **Workflow Testing** - End-to-end scenario validation
- **Error Propagation** - Error handling across component boundaries
- **Component Interaction** - Multi-component collaboration testing

### Test Fixture Patterns
- **Reusable Test Data** - Standard inputs and expected outputs
- **Setup Consistency** - Uniform test environment preparation
- **Test Organization** - Structured test data management

## Testing Architecture

- **llmspell-testing Crate** - Comprehensive testing utilities
- **tokio Test Framework** - Async testing with `#[tokio::test]`
- **Mock Objects** - Test doubles for controlled scenarios
- **Test Fixtures** - Reusable test data and scenarios

## Professional Testing Practices

- **Test Organization** - Structured test modules and naming
- **Coverage Goals** - Comprehensive success and error case coverage
- **Async Testing** - Proper async test patterns with tokio
- **Error Testing** - Deliberate error condition validation

## Next Steps

After completing this example:
- Apply testing patterns to your own LLMSpell components
- Explore advanced testing with external dependencies
- Learn performance testing and benchmarking techniques
- Study test-driven development (TDD) workflows