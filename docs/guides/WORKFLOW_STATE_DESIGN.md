# Workflow State Management Design

## Overview

This document outlines the state management system for rs-llmspell workflows, preparing for Phase 5's persistent state implementation while providing immediate in-memory functionality.

## Current Implementation (Phase 3.3.19)

### In-Memory State Global

The `State` global object provides thread-safe in-memory state management:

```lua
-- Basic usage
State.set("key", "value")
local value = State.get("key")
State.delete("key")
local keys = State.list()
```

### Workflow State Infrastructure

We've created the foundation for workflow-integrated state management:

1. **State Manager** (`WorkflowStateManager`): Thread-safe state storage
2. **State Scoping** (`StateScope`): Isolation between workflows and steps
3. **State Access** (`WorkflowStateAccessor`): Convenient API for workflows
4. **State Builder** (`StateBuilder`): Integration trait for workflow builders

### State Scoping

State can be scoped at multiple levels:

```rust
pub enum StateScope {
    Global,                                    // Shared across all workflows
    Workflow(String),                         // Workflow-specific
    Step { workflow_id: String, step_name: String }, // Step-specific
    Custom(String),                           // Custom namespaces
}
```

### Thread Safety

All state operations use `parking_lot::RwLock` for efficient concurrent access:

- Multiple readers can access state simultaneously
- Writers have exclusive access
- Optimized for read-heavy workloads

## Integration with Workflows

### Sequential Workflows

Steps can read and write shared state:

```rust
// In step execution
if let Some(state) = context.state_accessor {
    // Read previous step's output
    let data = state.get("processed_data");
    
    // Write for next step
    state.set("step_result", result);
}
```

### Parallel Workflows

Thread-safe state access for concurrent branches:

```rust
// Each branch can safely update shared state
state.set("branch_1_done", true);
let all_done = state.get("branch_1_done").is_some() 
    && state.get("branch_2_done").is_some();
```

### Conditional Workflows

State can influence branching decisions:

```rust
// Check state to determine branch
let should_retry = state.get("retry_count")
    .and_then(|v| v.as_u64())
    .map(|count| count < 3)
    .unwrap_or(true);
```

### Loop Workflows

State tracks iteration progress:

```rust
// Update iteration counter
let iteration = state.get("iteration")
    .and_then(|v| v.as_u64())
    .unwrap_or(0);
state.set("iteration", iteration + 1);
```

## API Design

### Workflow State API

```rust
// Create state manager
let state_manager = WorkflowStateManager::new();

// Get workflow-specific accessor
let state = state_manager.workflow_state("workflow_123".to_string());

// Workflow-scoped operations
state.set("config", json!({"retries": 3}));
let config = state.get("config");

// Access global state
state.global().set("api_key", json!("sk-123"));

// Access step state
let step_state = state.step("process_data".to_string());
step_state.set("progress", json!(0.5));
```

### Script API (Lua)

```lua
-- Global state
State.set("key", "value")
local value = State.get("key")

-- In workflows (future)
workflow.state.set("step_data", result)
local data = workflow.state.get("step_data")

-- Step-specific (future)
step.state.set("progress", 0.5)
```

## Performance Considerations

### Current Performance

- State access: <1ms (in-memory)
- Concurrent reads: No blocking
- Write operations: Minimal lock contention
- Memory usage: Proportional to stored data

### Optimization Strategies

1. **Read-Heavy Optimization**: Using RwLock for concurrent reads
2. **Scoped Keys**: Efficient key prefixing for namespace isolation
3. **Lazy Cloning**: Values cloned only when needed
4. **Batch Operations**: Future API for bulk updates

## Phase 5 Integration Points

### Persistent Storage

Phase 5 will add:
- Sled/RocksDB backends
- Automatic persistence
- Write-ahead logging
- Crash recovery

### State Migrations

```rust
// Future migration API
pub trait StateMigration {
    fn version(&self) -> u32;
    fn migrate(&self, state: &mut StateManager) -> Result<()>;
}
```

### Distributed State

- State synchronization across nodes
- Conflict resolution strategies
- Eventually consistent updates
- Partition tolerance

## Security Considerations

### Access Control

- Scoped isolation prevents cross-workflow interference
- No direct file system access (until Phase 5)
- JSON serialization prevents code injection

### Resource Limits

Future implementations will enforce:
- Maximum state size per workflow
- Key count limits
- Value size restrictions
- TTL for temporary state

## Migration Path

### From In-Memory to Persistent

When Phase 5 arrives:
1. Existing State API remains unchanged
2. Add persistence configuration
3. Automatic migration of in-memory data
4. Gradual rollout with feature flags

### Backward Compatibility

- In-memory mode remains available
- Same API surface
- Performance characteristics preserved
- Opt-in persistence

## Best Practices

### State Design

1. **Minimize State**: Store only essential data
2. **Clear Naming**: Use descriptive keys
3. **Scope Appropriately**: Use workflow/step scopes
4. **Clean Up**: Remove unused state

### Concurrency

1. **Atomic Updates**: Use compare-and-swap patterns
2. **Avoid Races**: Design for concurrent access
3. **Batch Updates**: Reduce lock contention
4. **Read Caching**: Cache frequently accessed values

### Error Handling

1. **Graceful Defaults**: Handle missing keys
2. **Type Safety**: Validate JSON values
3. **Logging**: Track state changes
4. **Recovery**: Design for state loss

## Future Enhancements

### Phase 4 Integration

- Hook system integration for state change events
- Event emission on state updates
- State change triggers

### Phase 5 Features

- Persistent storage backends
- State versioning and history
- Distributed state sync
- Backup and restore
- Migration tools

### Beyond Phase 5

- GraphQL API for state queries
- State visualization tools
- Time-travel debugging
- State analytics

## Conclusion

The workflow state management system provides a solid foundation for stateful workflow execution. The current in-memory implementation offers full functionality with excellent performance, while the architecture is designed to seamlessly integrate persistent storage in Phase 5.

Key achievements:
- Thread-safe state management
- Scoped isolation
- Clean API design
- Performance optimization
- Future-proof architecture

This infrastructure enables sophisticated workflow patterns while maintaining simplicity and performance.