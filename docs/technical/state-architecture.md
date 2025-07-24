# Workflow State Architecture

**Version**: Phase 3.3 Implementation  
**Status**: ✅ **CURRENT** - Thread-safe in-memory state management  
**Future**: Phase 5 will add persistent storage backends

> **🔧 IMPLEMENTATION STATUS**: This document describes the current in-memory state architecture (Phase 3.3) and planned persistent storage features (Phase 5). All current features are production-ready.

## State Scoping

State can be scoped at multiple levels:

```rust
pub enum StateScope {
    Global,                                    // Shared across all workflows
    Workflow(String),                         // Workflow-specific
    Step { workflow_id: String, step_name: String }, // Step-specific
    Custom(String),                           // Custom namespaces
}
```

## Thread Safety

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

## Phase 5 Integration Points (Future)

### Persistent Storage (Planned - Phase 5)

📋 **Phase 5 will add**:
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

### Distributed State (Planned - Phase 5+)

📋 **Future distributed features**:
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

- Same API surface
- Performance characteristics preserved
- Opt-in persistence

## Implementation Details

### WorkflowStateManager

Central state management with workflow isolation:

```rust
pub struct WorkflowStateManager {
    global_state: Arc<RwLock<HashMap<String, Value>>>,
    workflow_states: Arc<RwLock<HashMap<String, Arc<RwLock<HashMap<String, Value>>>>>>,
}
```

### StateScope Implementation

Efficient key prefixing for namespace isolation:

```rust
impl StateScope {
    pub fn prefix(&self) -> String {
        match self {
            StateScope::Global => String::new(),
            StateScope::Workflow(id) => format!("workflow:{}:", id),
            StateScope::Step { workflow_id, step_name } => {
                format!("workflow:{}:step:{}:", workflow_id, step_name)
            }
            StateScope::Custom(namespace) => format!("custom:{}:", namespace),
        }
    }
}
```

### Thread-Safe Operations

All operations use RAII guards:

```rust
pub fn set(&self, key: &str, value: Value) {
    let mut state = self.state.write();
    state.insert(self.make_key(key), value);
}

pub fn get(&self, key: &str) -> Option<Value> {
    let state = self.state.read();
    state.get(&self.make_key(key)).cloned()
}
```