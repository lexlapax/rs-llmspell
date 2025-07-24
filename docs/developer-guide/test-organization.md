# Test Organization Guide

✅ **Current Implementation**: This guide accurately reflects the current test organization and scripts. All referenced scripts exist and work as documented.

This guide explains how tests are organized in the rs-llmspell project and how to run specific categories of tests.

## Test Categories

Tests in rs-llmspell are organized into the following categories:

### 1. Unit Tests
- **Location**: In `src/` directories within `#[cfg(test)]` modules
- **Naming**: `test_unit_<component>_<behavior>`
- **Characteristics**: Fast, isolated, no I/O, no external dependencies
- **Run with**: `./scripts/test-by-tag.sh unit`

### 2. Integration Tests
- **Location**: In `tests/` directories
- **Naming**: `test_integration_<feature>_<scenario>`
- **Characteristics**: May use real files, may mock network calls
- **Run with**: `./scripts/test-by-tag.sh integration`

### 3. Tool Tests
- **Location**: In `llmspell-tools` package
- **Naming**: `test_tool_<toolname>_<operation>`
- **Characteristics**: Tool-specific functionality tests
- **Run with**: `./scripts/test-by-tag.sh tool`

### 4. Agent Tests
- **Location**: In `llmspell-agents` package (when implemented)
- **Naming**: `test_agent_<agentname>_<behavior>`
- **Characteristics**: Agent behavior and lifecycle tests
- **Run with**: `./scripts/test-by-tag.sh agent`

### 5. Bridge Tests
- **Location**: In `llmspell-bridge` package
- **Naming**: `test_bridge_<component>_<behavior>`
- **Characteristics**: Script engine and runtime integration
- **Run with**: `./scripts/test-by-tag.sh bridge`

### 6. LLM Tests
- **Location**: Any package testing LLM providers
- **Marking**: `#[ignore = "llm"]`
- **Characteristics**: Tests LLM provider integrations
- **Run with**: `./scripts/test-by-tag.sh llm`

### 7. Workflow Tests
- **Location**: Mixed (unit and integration)
- **Naming**: Contains "workflow" in the test name
- **Characteristics**: Tests workflow orchestration
- **Run with**: `./scripts/test-by-tag.sh workflow`

### 8. External Tests
- **Location**: Any test file
- **Marking**: `#[ignore = "external"]` or `#[ignore = "external,integration"]`
- **Characteristics**: Require network access or external services
- **Run with**: `./scripts/test-by-tag.sh external`

### 9. Slow Tests
- **Location**: Any test file
- **Marking**: `#[ignore = "slow"]`
- **Characteristics**: Take >1 second to run
- **Run with**: `./scripts/test-by-tag.sh slow`

### 10. Database Tests
- **Location**: Any test file using database functionality
- **Marking**: `#[ignore = "database"]`
- **Characteristics**: Require database connections
- **Run with**: `./scripts/test-by-tag.sh database`

## Test Naming Conventions

Follow these naming patterns for consistency:

```rust
// Unit test
#[test]
fn test_unit_response_builder_success() { }

// Integration test
#[test]
fn test_integration_file_tool_creates_directory() { }

// Tool test
#[test]
fn test_tool_calculator_division_by_zero() { }

// External test (requires network)
#[test]
#[ignore = "external,integration"]
fn test_integration_web_search_real_api() { }

// Slow test
#[test]
#[ignore = "slow"]
fn test_slow_large_file_processing() { }
```

## Using #[ignore] Attributes

The `#[ignore]` attribute can include a reason string for categorization:

```rust
#[test]
#[ignore = "external"]  // Single category
fn test_external_api() { }

#[test]
#[ignore = "external,integration"]  // Multiple categories
fn test_external_integration() { }

#[test]
#[ignore = "slow,tool"]  // Slow tool test
fn test_slow_tool_operation() { }

#[test]
#[ignore = "api_keys"]  // Requires API keys
fn test_with_credentials() { }
```

## Test Runner Scripts

### 1. Run Tests by Tag
```bash
# Run only unit tests
./scripts/test-by-tag.sh unit

# Run integration tests
./scripts/test-by-tag.sh integration

# Run tool tests
./scripts/test-by-tag.sh tool

# Run slow tests (single-threaded)
./scripts/test-by-tag.sh slow

# Run external tests
./scripts/test-by-tag.sh external

# Run all tests including ignored
./scripts/test-by-tag.sh all
```

### 2. Run Multiple Tags
```bash
# Run fast tool tests
./scripts/test-multiple-tags.sh "tool,fast"

# Run unit tests excluding slow ones
./scripts/test-multiple-tags.sh "unit,!slow"
```

### 3. List Tests by Tag
```bash
# See what unit tests exist
./scripts/list-tests-by-tag.sh unit

# See all ignored tests
./scripts/list-tests-by-tag.sh ignored

# Get test count summary
./scripts/list-tests-by-tag.sh all
```

## Environment Variables

### Skip Slow Tests in Quality Checks
```bash
# Run quality check without slow/external tests
SKIP_SLOW_TESTS=true ./scripts/quality-check.sh
```

### Custom Test Selection
```bash
# Future: Run tests matching tags
LLMSPELL_TEST_TAGS="tool,fast" cargo test

# Future: Skip tests matching tags
LLMSPELL_SKIP_TAGS="slow,external" cargo test
```

## CI/CD Integration

The test organization supports efficient CI/CD pipelines:

1. **Fast feedback**: Run unit tests first
2. **Parallel execution**: Run test categories in parallel jobs
3. **Conditional execution**: Skip external tests without credentials
4. **Nightly runs**: Full test suite including slow tests

Example GitHub Actions matrix:
```yaml
strategy:
  matrix:
    test-category: [unit, integration, tool, agent, workflow]
```

## Best Practices

1. **Always categorize tests** - Use appropriate naming and #[ignore] attributes
2. **Keep unit tests fast** - <100ms per test
3. **Mock external dependencies** - Unless explicitly testing external integration
4. **Document test requirements** - API keys, file permissions, etc.
5. **Run appropriate tests** - Use test-by-tag.sh during development

## Examples

### Adding a New Unit Test
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unit_my_component_handles_empty_input() {
        // Fast, isolated test
        let result = my_function("");
        assert!(result.is_err());
    }
}
```

### Adding an Integration Test
```rust
// In tests/my_integration_test.rs
#[tokio::test]
async fn test_integration_tools_work_together() {
    // May use real files, multiple components
    let file_tool = FileTool::new();
    let calc_tool = CalculatorTool::new();
    // Test interaction...
}
```

### Adding an External Test
```rust
#[tokio::test]
#[ignore = "external,slow"]
async fn test_external_real_api_endpoint() {
    // Requires network, may be slow
    let response = reqwest::get("https://api.example.com").await;
    // ...
}
```

## Tool Testing Patterns

Specific patterns for testing tools effectively:

### Standard Tool Test Pattern
```rust
#[tokio::test]
async fn test_tool_name() {
    let tool = ToolName::new();
    assert_eq!(tool.name(), "expected_name");
    
    let params = serde_json::json!({
        "param1": "value1",
        "param2": "value2"
    });
    
    let result = tool.execute(Value::Object(params)).await;
    assert!(result.is_ok());
    
    let response = result.unwrap();
    let output: serde_json::Value = serde_json::from_str(&response.output).unwrap();
    assert!(output["success"].as_bool().unwrap());
    // Additional assertions...
}
```

### Error Handling Pattern
```rust
// Test validates proper error response format
let result = tool.execute(Value::Object(invalid_params)).await;
assert!(result.is_ok()); // Tool returns Ok with error in output
let output: serde_json::Value = serde_json::from_str(&response.output).unwrap();
assert!(!output["success"].as_bool().unwrap());
assert!(output["error"].is_string());
```

### Performance Test Pattern
```rust
let start = Instant::now();
for _ in 0..10 {
    ToolName::new();
}
let avg_duration = start.elapsed() / 10;
assert!(avg_duration.as_millis() < 10);
```

### Tool Chaining Tests
Tests that validate tools working together:
- **File → Data → File**: file_operations → file_converter → file_search
- **System → Data → Utility**: environment_reader → text_manipulator → hash_calculator

## Special Testing Considerations

### Platform-Specific Tests
Some tests may behave differently on different platforms:
- Process execution commands
- File path handling  
- System monitoring metrics

### External Dependencies
Some tests require external resources:
- Network connectivity for service_checker HTTP tests
- File system access for file operations
- System permissions for process execution

### Media Processing Tools
Media processing tools (audio_processor, video_processor, image_processor) may have placeholder implementations:
- Tests should verify placeholder behavior is correct
- Actual processing may be deferred to future phases

## Troubleshooting

### Tests Take Too Long
1. Use `./scripts/test-by-tag.sh fast` for quick feedback
2. Set `SKIP_SLOW_TESTS=true` for quality checks
3. Run slow tests separately with `./scripts/test-by-tag.sh slow`

### Can't Find Specific Tests
1. Use `./scripts/list-tests-by-tag.sh <tag>` to discover tests
2. Check test naming follows conventions
3. Ensure tests have appropriate #[ignore] attributes

### External Tests Failing
1. Check required environment variables are set
2. Verify network connectivity
3. Run with `./scripts/test-by-tag.sh external` for detailed output