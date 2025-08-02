# Test Development Guide

**Version**: Post-7.1.6 Implementation  
**Status**: ✅ **CURRENT** - Reflects actual test architecture  
**Last Updated**: January 2025

> **🧪 COMPREHENSIVE TESTING GUIDE**: Complete guide to testing in rs-llmspell, covering categorization, organization, utilities, and best practices established in Task 7.1.6.

**🔗 Navigation**: [← Developer Guide](README.md) | [Documentation Hub](../README.md) | [CLAUDE.md Testing Guidelines](../../CLAUDE.md#testing-guidelines-critical---maintain-716-architecture)

---

## Overview

This guide documents the complete test architecture established in Task 7.1.6, which processed 536+ test files across the entire codebase and created a sophisticated but practical test categorization system.

**Key Achievements of 7.1.6:**
- ✅ All 536+ test files properly categorized
- ✅ Fast test suite: <35 seconds (unit + integration)  
- ✅ External tests properly isolated (35 tests)
- ✅ Test utilities consolidated in llmspell-testing
- ✅ Quality check scripts for development workflow
- ✅ Feature flag-based test execution

---

## Test Categorization System

### Basic Categories (Always Required)

Choose exactly one based on test characteristics:

```rust
#[test]
#[cfg_attr(test_category = "unit")]        // Fast, isolated, <5s total per crate
#[cfg_attr(test_category = "integration")] // Cross-component, <30s total per crate  
#[cfg_attr(test_category = "external")]    // External dependencies, can be slow
```

### Component Categories (Add One)

Add one category that matches your functionality:

```rust
#[cfg_attr(test_category = "tool")]        // Tool-related functionality
#[cfg_attr(test_category = "agent")]       // Agent-related functionality  
#[cfg_attr(test_category = "workflow")]    // Workflow-related functionality
#[cfg_attr(test_category = "bridge")]      // Script bridge functionality
#[cfg_attr(test_category = "hook")]        // Hook system functionality
#[cfg_attr(test_category = "event")]       // Event system functionality
#[cfg_attr(test_category = "session")]     // Session management functionality
#[cfg_attr(test_category = "state")]       // State management functionality
#[cfg_attr(test_category = "core")]        // Core trait/type functionality
#[cfg_attr(test_category = "util")]        // Utility functionality
```

### Specialized Categories (When Applicable)

```rust
#[cfg_attr(test_category = "security")]    // Security-related tests
#[cfg_attr(test_category = "performance")] // Performance/benchmark tests
```

### External Test Marking

For tests requiring external dependencies:

```rust
#[test]
#[cfg_attr(test_category = "external")]
#[cfg_attr(test_category = "tool")]
#[ignore = "external"]  // ← This makes it skippable in CI
async fn test_real_api_integration() {
    // Test with actual API calls, network, etc.
}
```

---

## Test Placement Rules

### Unit Tests: In `src/` Files
- **Location**: Within `#[cfg(test)]` modules in source files
- **Characteristics**: Fast, isolated, no I/O, no external dependencies
- **Performance**: Should complete in <5 seconds total per crate
- **Example**:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg_attr(test_category = "unit")]
    #[cfg_attr(test_category = "tool")]
    fn test_file_reader_basic_functionality() {
        let reader = FileReader::new();
        assert_eq!(reader.name(), "file_reader");
    }
}
```

### Integration Tests: In `tests/` Directories
- **Location**: Separate files in `tests/` directories
- **Characteristics**: Cross-component, external dependencies MUST be mocked
- **Performance**: Should complete in <30 seconds total per crate
- **Example**:
```rust
// In tests/file_tool_integration.rs
#[tokio::test]
#[cfg_attr(test_category = "integration")]
#[cfg_attr(test_category = "tool")]
async fn test_file_operations_work_together() {
    use llmspell_testing::tool_helpers::create_test_tool_input;
    
    let file_tool = FileTool::new();
    let input = create_test_tool_input(vec![("operation", "read"), ("path", "/tmp/test.txt")]);
    // Test interaction between multiple components
}
```

### External Tests: In `tests/` with `#[ignore = "external"]`
- **Location**: `tests/` directories with special marking
- **Characteristics**: Real API calls, network requests, slow operations
- **Performance**: Can be slow, skipped in CI by default
- **Example**:
```rust
#[tokio::test]
#[cfg_attr(test_category = "external")]
#[cfg_attr(test_category = "tool")]
#[ignore = "external"]
async fn test_real_web_request() {
    let response = reqwest::get("https://api.example.com").await;
    assert!(response.is_ok());
}
```

---

## Test Execution Commands

### Development Workflow Commands

**MANDATORY before commits:**
```bash
# Quick check (seconds) - formatting, clippy, compilation  
./scripts/quality-check-minimal.sh     

# Fast check (~1 min) - adds unit tests & docs
./scripts/quality-check-fast.sh        

# Full check (5+ min) - all tests & coverage
./scripts/quality-check.sh             
```

**During development:**
```bash
# Test specific components
./scripts/test-by-tag.sh unit         # Run only unit tests
./scripts/test-by-tag.sh tool         # Run tool tests
./scripts/test-by-tag.sh external     # Run external/network tests

# Skip slow tests in quality checks
SKIP_SLOW_TESTS=true ./scripts/quality-check.sh
```

### Feature-Based Test Execution

```bash
# Fast unit tests only
cargo test --features unit-tests              

# Integration tests only  
cargo test --features integration-tests       

# External tests only (requires --ignored)
cargo test --features external-tests --ignored 

# All tests (slow)
cargo test --features all-tests               
```

### Component-Specific Testing

```bash
# Run tests for specific crates
cargo test -p llmspell-tools
cargo test -p llmspell-workflows  
cargo test -p llmspell-bridge

# Run specific test categories
./scripts/test-by-tag.sh workflow
./scripts/test-by-tag.sh agent
./scripts/test-by-tag.sh state
```

---

## Test Utilities (MANDATORY - NO DUPLICATES)

### Using llmspell-testing Helpers

**ALWAYS use centralized helpers instead of creating your own:**

```rust
use llmspell_testing::{
    // Tool testing
    tool_helpers::{create_test_tool, create_test_tool_input, MockTool},
    
    // Agent testing  
    agent_helpers::{AgentTestBuilder, create_mock_provider_agent, TestProviderAgent},
    
    // Event testing
    event_helpers::{create_test_event, create_test_event_bus},
    
    // Workflow testing
    workflow_helpers::{create_test_workflow_step, create_test_sequential_workflow},
    
    // State testing
    state_helpers::{create_test_state_manager, create_test_memory_backend},
    
    // Hook testing
    hook_helpers::{create_test_hook_manager, create_test_hook_config},
    
    // Bridge testing
    bridge_helpers::{create_test_context, create_test_registry},
    
    // Common mocks
    mocks::{MockBaseAgent, MockProvider, MockTool},
};
```

### Adding llmspell-testing to Your Crate

Add to `Cargo.toml`:
```toml
[dev-dependencies]
llmspell-testing = { path = "../llmspell-testing", features = ["test-utilities"] }
```

### Foundational Crates (Cannot Use llmspell-testing)

Due to circular dependency constraints, these crates keep local helpers:

- **llmspell-core**: Core traits and types
- **llmspell-utils**: Basic utilities  
- **llmspell-storage**: Storage abstractions
- **llmspell-security**: Security primitives
- **llmspell-config**: Configuration types
- **llmspell-state-traits**: State trait definitions

**Note**: 88 test helper functions remain outside llmspell-testing, primarily in these foundational crates.

---

## Test Development Patterns

### Standard Tool Test Pattern

```rust
#[tokio::test]
#[cfg_attr(test_category = "unit")]
#[cfg_attr(test_category = "tool")]
async fn test_tool_calculator_basic_operations() {
    use llmspell_testing::tool_helpers::{create_test_tool, create_test_tool_input};
    
    let tool = create_test_tool("calculator", "Calculator tool", vec![
        ("operation", "string"),
        ("a", "number"), 
        ("b", "number")
    ]);
    
    let input = create_test_tool_input(vec![
        ("operation", "add"),
        ("a", "5"),
        ("b", "3")
    ]);
    
    let output = tool.execute(input, Default::default()).await.unwrap();
    
    // Verify tool call was made
    assert!(!output.tool_calls.is_empty());
    
    // Check result
    if let Some(result) = &output.tool_calls[0].result {
        assert!(result.success);
        // Additional assertions...
    }
}
```

### Agent Testing Pattern

```rust
#[tokio::test]
#[cfg_attr(test_category = "integration")]
#[cfg_attr(test_category = "agent")]
async fn test_agent_with_tools() {
    use llmspell_testing::agent_helpers::AgentTestBuilder;
    
    let agent = AgentTestBuilder::new("test-agent")
        .with_provider("mock")
        .with_model("test-model")
        .with_response("I performed the calculation: 8")
        .build()
        .await
        .unwrap();
    
    let input = AgentInput::text("Add 5 and 3");
    let output = agent.execute(input, ExecutionContext::default()).await.unwrap();
    
    assert!(output.text.contains("calculation"));
}
```

### Error Handling Test Pattern

```rust
#[test]
#[cfg_attr(test_category = "unit")]  
#[cfg_attr(test_category = "tool")]
fn test_tool_error_handling() {
    use llmspell_testing::tool_helpers::create_test_tool_input;
    
    let tool = FileTool::new();
    let invalid_input = create_test_tool_input(vec![
        ("operation", "invalid_op"),
        ("path", "")
    ]);
    
    let result = tool.execute(invalid_input, Default::default()).await;
    
    // Tool should return Ok with error in output, not Err
    assert!(result.is_ok());
    let output = result.unwrap();
    
    if let Some(result) = &output.tool_calls[0].result {
        assert!(!result.success);
        assert!(result.error.is_some());
    }
}
```

### External Dependency Test Pattern

```rust
#[tokio::test]
#[cfg_attr(test_category = "external")]
#[cfg_attr(test_category = "tool")]
#[ignore = "external"]
async fn test_real_web_request() {
    // Skip if no network or credentials
    if std::env::var("SKIP_EXTERNAL_TESTS").is_ok() {
        return;
    }
    
    let web_tool = WebSearchTool::new();
    let input = create_test_tool_input(vec![
        ("operation", "search"),
        ("query", "rust programming")
    ]);
    
    let output = web_tool.execute(input, Default::default()).await.unwrap();
    // Test real external behavior...
}
```

### Performance Test Pattern

```rust
#[test]
#[cfg_attr(test_category = "performance")]
#[cfg_attr(test_category = "tool")]
fn test_tool_creation_performance() {
    let start = std::time::Instant::now();
    
    for _ in 0..10 {
        let _tool = FileTool::new();
    }
    
    let avg_duration = start.elapsed() / 10;
    assert!(avg_duration.as_millis() < 10, "Tool creation too slow: {:?}", avg_duration);
}
```

---

## Test Organization Best Practices

### 1. Always Categorize Tests

**Bad:**
```rust
#[test]
fn test_something() {  // ❌ No categorization
    // ...
}
```

**Good:**
```rust
#[test]
#[cfg_attr(test_category = "unit")]      // ✅ Speed category
#[cfg_attr(test_category = "tool")]      // ✅ Component category  
fn test_tool_file_reader_basic() {
    // ...
}
```

### 2. Use Descriptive Test Names

Follow the pattern: `test_<category>_<component>_<behavior>`

```rust
#[test]
fn test_unit_file_reader_handles_empty_path() { }

#[test] 
fn test_integration_file_tool_creates_directory() { }

#[test]
fn test_external_web_search_real_api() { }
```

### 3. Group Related Tests

Use modules to organize related functionality:

```rust
#[cfg(test)]
mod file_operations {
    use super::*;
    
    #[test]
    #[cfg_attr(test_category = "unit")]
    #[cfg_attr(test_category = "tool")]
    fn test_read_file() { }
    
    #[test]
    #[cfg_attr(test_category = "unit")]
    #[cfg_attr(test_category = "tool")]
    fn test_write_file() { }
}
```

### 4. Mock External Dependencies

For integration tests, always mock external services:

```rust
#[tokio::test]
#[cfg_attr(test_category = "integration")]
#[cfg_attr(test_category = "tool")]
async fn test_web_tool_with_mock_server() {
    // Use wiremock or similar to mock HTTP responses
    let mock_server = MockServer::start().await;
    
    Mock::given(method("GET"))
        .and(path("/api/search"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "results": ["test result"]
        })))
        .mount(&mock_server)
        .await;
    
    // Test with mock URL
    let web_tool = WebSearchTool::with_base_url(&mock_server.uri());
    // ... test logic
}
```

### 5. Test State Transitions

For stateful components, verify all state machine paths:

```rust
#[tokio::test]
#[cfg_attr(test_category = "unit")]
#[cfg_attr(test_category = "agent")]
async fn test_agent_lifecycle() {
    let agent = create_test_agent();
    
    // Initial state
    assert_eq!(agent.state(), AgentState::Created);
    
    // Initialize
    agent.initialize().await.unwrap();
    assert_eq!(agent.state(), AgentState::Ready);
    
    // Execute
    let result = agent.execute(input).await.unwrap();
    assert_eq!(agent.state(), AgentState::Ready);
    
    // Shutdown
    agent.shutdown().await.unwrap();
    assert_eq!(agent.state(), AgentState::Stopped);
}
```

---

## Advanced Testing Patterns

### Property-Based Testing

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    #[cfg_attr(test_category = "unit")]
    #[cfg_attr(test_category = "util")]
    fn test_string_utils_roundtrip(s in ".*") {
        let encoded = string_utils::encode(&s);
        let decoded = string_utils::decode(&encoded).unwrap();
        prop_assert_eq!(s, decoded);
    }
}
```

### Async Test Patterns

```rust
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[cfg_attr(test_category = "integration")]
#[cfg_attr(test_category = "workflow")]
async fn test_parallel_workflow_execution() {
    // Test concurrent execution
}

#[tokio::test]
#[cfg_attr(test_category = "integration")]
#[cfg_attr(test_category = "workflow")]
async fn test_workflow_cancellation() {
    use tokio::time::{timeout, Duration};
    
    let workflow = create_long_running_workflow();
    let future = workflow.execute();
    
    // Test timeout behavior
    let result = timeout(Duration::from_millis(100), future).await;
    assert!(result.is_err()); // Should timeout
}
```

### Test Data Management

```rust
use tempfile::TempDir;

#[tokio::test]
#[cfg_attr(test_category = "integration")]
#[cfg_attr(test_category = "tool")]
async fn test_file_tool_with_temp_directory() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.txt");
    
    // Create test file
    std::fs::write(&test_file, "test content").unwrap();
    
    let file_tool = FileTool::new();
    let input = create_test_tool_input(vec![
        ("operation", "read"),
        ("path", test_file.to_str().unwrap())
    ]);
    
    let output = file_tool.execute(input, Default::default()).await.unwrap();
    
    // Verify file was read correctly
    // temp_dir automatically cleaned up when dropped
}
```

---

## CI/CD Integration

### GitHub Actions Example

```yaml
name: Test Suite

on: [push, pull_request]

jobs:
  fast-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      
      # Fast feedback loop
      - name: Run fast tests
        run: ./scripts/quality-check-fast.sh
  
  comprehensive-tests:
    runs-on: ubuntu-latest
    needs: fast-tests
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      
      # Full test suite
      - name: Run comprehensive tests
        run: ./scripts/quality-check.sh
  
  external-tests:
    runs-on: ubuntu-latest
    if: github.event_name == 'schedule' # Only run on nightly
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      
      # External dependency tests
      - name: Run external tests
        run: ./scripts/test-by-tag.sh external
        env:
          OPENAI_API_KEY: ${{ secrets.OPENAI_API_KEY }}
          ANTHROPIC_API_KEY: ${{ secrets.ANTHROPIC_API_KEY }}
```

### Test Matrix Strategy

```yaml
strategy:
  matrix:
    test-category: [unit, integration, tool, agent, workflow, bridge, state]
    
steps:
  - name: Run ${{ matrix.test-category }} tests
    run: ./scripts/test-by-tag.sh ${{ matrix.test-category }}
```

---

## Troubleshooting

### Common Issues

**Tests Take Too Long:**
1. Use `./scripts/test-by-tag.sh unit` for quick feedback
2. Set `SKIP_SLOW_TESTS=true` for quality checks  
3. Run slow tests separately: `./scripts/test-by-tag.sh external`

**Can't Find Specific Tests:**
1. Use `./scripts/list-tests-by-tag.sh <tag>` to discover tests
2. Check test naming follows conventions
3. Ensure tests have appropriate categorization

**External Tests Failing:**
1. Check required environment variables are set
2. Verify network connectivity
3. Use `#[ignore = "external"]` for CI skipping

**Compilation Errors in Tests:**
1. Run `./scripts/quality-check-minimal.sh` first
2. Check imports and dependencies
3. Verify test helper usage is correct

**Flaky Tests:**
1. Remove non-deterministic behavior
2. Use mocks instead of real external services
3. Add proper timeouts and error handling
4. Consider marking as `#[cfg_attr(test_category = "external")]`

### Debug Mode

Enable detailed test output:
```bash
# Run with output
cargo test -- --nocapture

# Run specific test with debug
RUST_LOG=debug cargo test test_specific_function -- --nocapture

# Run with timing
cargo test -- --nocapture --show-output
```

---

## Migration Guide

### From Pre-7.1.6 Tests

1. **Add categorization to existing tests:**
   ```rust
   #[test]
   #[cfg_attr(test_category = "unit")]    // Add this
   #[cfg_attr(test_category = "tool")]    // Add this
   fn existing_test() {
       // ...
   }
   ```

2. **Replace local test helpers:**
   ```rust
   // Before
   fn create_test_tool() -> Box<dyn Tool> {
       Box::new(MockTool::new())
   }
   
   // After
   use llmspell_testing::tool_helpers::create_test_tool;
   ```

3. **Update external test marking:**
   ```rust
   // Before
   #[ignore]
   fn test_external_api() { }
   
   // After
   #[cfg_attr(test_category = "external")]
   #[ignore = "external"]
   fn test_external_api() { }
   ```

### For New Crates

1. Always add `llmspell-testing` to dev-dependencies
2. Use centralized helpers from day one
3. Follow categorization patterns from this guide
4. Set up quality check scripts integration

---

## Summary

The test architecture established in Task 7.1.6 provides:

✅ **Fast Development Feedback**: Unit tests run in <35 seconds  
✅ **Reliable CI**: External tests properly isolated  
✅ **Consistent Patterns**: Centralized utilities in llmspell-testing  
✅ **Quality Automation**: Scripts for pre-commit validation  
✅ **Comprehensive Coverage**: 536+ files properly categorized  

**Key Commands to Remember:**
- `./scripts/quality-check-fast.sh` - Daily development workflow
- `./scripts/test-by-tag.sh unit` - Quick feedback during coding
- `./scripts/quality-check.sh` - Full validation before major commits
- Use `llmspell-testing` helpers - never duplicate test utilities

This architecture ensures maintainable, fast, and reliable testing across the entire rs-llmspell codebase.