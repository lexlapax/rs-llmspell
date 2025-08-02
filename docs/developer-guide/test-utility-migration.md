# Test Utility Migration Guide

This guide documents the test utility consolidation performed in Phase 7, Step 7 of the implementation.

## Overview

We've consolidated duplicate test utilities from across the codebase into the `llmspell-testing` crate. This provides:

1. Centralized test helpers for all crates
2. Consistent testing patterns
3. Reduced code duplication
4. Easier maintenance

## Migration Summary

### Crates Using llmspell-testing

The following crates have been migrated to use centralized test utilities:

- **llmspell-agents**: Uses `agent_helpers::create_test_agent()`
- **llmspell-bridge**: Uses `bridge_helpers::create_test_context()`
- **llmspell-events**: Uses `event_helpers::create_test_event_bus()`
- **llmspell-hooks**: Uses `hook_helpers::create_test_hook_manager()`
- **llmspell-sessions**: Uses centralized helpers (kept TestFixture for specialized needs)
- **llmspell-state-persistence**: Uses some centralized helpers (kept local helpers due to circular deps)
- **llmspell-tools**: Uses `tool_helpers::create_test_tool()`
- **llmspell-workflows**: Uses `workflow_helpers::create_test_steps()`

### Foundational Crates (Cannot Use llmspell-testing)

Due to circular dependency constraints, these crates cannot depend on llmspell-testing:

- **llmspell-core**: Core traits and types
- **llmspell-utils**: Basic utilities
- **llmspell-storage**: Storage abstractions
- **llmspell-security**: Security primitives
- **llmspell-config**: Configuration types
- **llmspell-state-traits**: State trait definitions
- **llmspell-cli**: CLI binary (doesn't need test utilities)

### Other Crates

- **llmspell-providers**: Has minimal tests, doesn't use shared helpers

## Helper Modules

The `llmspell-testing` crate provides these helper modules:

### tool_helpers
```rust
pub fn create_test_tool() -> Box<dyn Tool>
pub fn create_test_tool_input(params: Vec<(&str, &str)>) -> AgentInput
pub fn create_test_tool_with_function<F>(name: &str, f: F) -> Box<dyn Tool>
```

### agent_helpers
```rust
pub fn create_test_agent() -> Box<dyn Agent>
pub fn create_test_context() -> ProviderContext
pub fn create_test_memory() -> MemoryProvider
```

### state_helpers
```rust
pub async fn create_test_state_manager() -> Arc<StateManager>
pub async fn create_test_memory_backend() -> Arc<dyn StorageBackend>
```

### hook_helpers
```rust
pub async fn create_test_hook_manager() -> Arc<HookManager>
pub fn create_test_hook_config() -> HookConfig
```

### event_helpers
```rust
pub async fn create_test_event_bus() -> Arc<EventBus>
pub fn create_test_event(event_type: &str) -> Event
```

### workflow_helpers
```rust
pub fn create_test_steps(count: usize) -> Vec<WorkflowStep>
pub fn create_test_workflow(name: &str) -> Box<dyn Workflow>
```

### bridge_helpers
```rust
pub async fn create_test_context() -> GlobalContext
pub async fn create_test_registry() -> Arc<ComponentRegistry>
```

## Migration Process

To migrate your tests to use centralized utilities:

1. Add `llmspell-testing` to dev-dependencies:
   ```toml
   [dev-dependencies]
   llmspell-testing = { path = "../llmspell-testing", features = ["test-utilities"] }
   ```

2. Replace local test helper functions with imports:
   ```rust
   // Before
   fn create_test_tool() -> Box<dyn Tool> {
       Box::new(MockTool::new())
   }
   
   // After
   use llmspell_testing::tool_helpers::create_test_tool;
   ```

3. Remove duplicate test utility functions from your crate

## Patterns That Remain Local

Some patterns couldn't be consolidated due to:

1. **Circular Dependencies**: Foundational crates cannot depend on llmspell-testing
2. **Domain-Specific Needs**: Some test fixtures are too specialized (e.g., `TestFixture` in sessions)
3. **Performance Tests**: Benchmark-specific utilities often need to remain local

## Common Remaining Patterns

Analysis shows these patterns still exist outside llmspell-testing:

- `create_test_context`: 11 locations (mostly in foundational crates)
- `create_test_event`: 6 locations (in crates that can't use llmspell-testing)
- `create_test_manager`: 3 locations
- `create_test_tool`: 4 locations (in foundational crates)
- `create_test_state`: 4 locations

Total: 88 test helper functions remain outside llmspell-testing, primarily in foundational crates.

## Best Practices

1. **New Crates**: Always use `llmspell-testing` for test utilities unless you're creating a foundational crate
2. **Existing Tests**: Gradually migrate to centralized helpers when modifying tests
3. **Custom Helpers**: Only create local helpers when they're truly domain-specific
4. **Documentation**: Document why a test helper remains local if it can't be consolidated

## Future Improvements

1. Consider creating a `llmspell-testing-core` with minimal dependencies for foundational crates
2. Add more specialized test fixtures for common scenarios
3. Expand property-based testing generators
4. Add performance testing utilities