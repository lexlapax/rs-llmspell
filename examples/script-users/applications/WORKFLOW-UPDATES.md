# Workflow API Updates - Phase 7.3.12.8

## Overview
This document outlines the new workflow capabilities added in Phase 7.3.12.8, including parallel workflows, conditional workflows with unified state, and loop workflows.

## 1. Parallel Workflows ✅
Execute multiple steps concurrently with automatic result aggregation.

### API
```lua
local workflow = Workflow.builder()
    :name("parallel_example")
    :parallel()
    :max_concurrency(3)  -- Optional: limit concurrent executions
    :add_step({ ... })
    :add_step({ ... })
    :build()
```

### Implemented In
- **content-creator**: Nested parallel quality checks after sequential content generation
  - Performance: 3 agents sequential (16.3s) → 2 agents parallel (43ms)

## 2. Conditional Workflows ✅
Branch execution based on conditions with full state integration.

### API
```lua
local workflow = Workflow.builder()
    :name("conditional_example")
    :conditional()
    :condition({
        type = "always",              -- Always execute then_branch
        -- OR
        type = "never",               -- Always execute else_branch
        -- OR
        type = "shared_data_equals",  -- ✅ NOW WORKING with unified state
        key = "priority",
        value = "urgent"
        -- OR
        type = "shared_data_exists",  -- ✅ NOW WORKING with unified state
        key = "user_id"
    })
    :add_then_step({ ... })  -- Executed when condition is true
    :add_else_step({ ... })  -- Executed when condition is false
    :build()

-- Set shared data for conditions
workflow:set_shared_data("priority", "urgent")
```

### Key Fix
Conditional workflows now use the unified state management system from Task 7.3.8:
- Reads from `ExecutionContext.state` instead of internal state manager
- `SharedDataEquals` and `SharedDataExists` conditions fully functional
- State persists across workflow executions

### Implemented In
- **communication-manager**: Conditional routing for escalation vs standard paths
- **process-orchestrator**: Two conditional workflows for incident routing

## 3. Loop Workflows ✅
Iterate over ranges, collections, or while conditions with proper termination.

### API
```lua
local workflow = Workflow.builder()
    :name("loop_example")
    :loop()  -- or :loop_workflow()
    
    -- Option 1: Range iteration
    :with_range({ 
        start = 1,      -- Starting value
        ["end"] = 10,   -- Ending value (exclusive)
        step = 1        -- Step increment
    })
    
    -- Option 2: Collection iteration
    :with_collection({ "apple", "banana", "cherry" })
    
    -- Option 3: While condition
    :with_while("condition_expression")
    
    :max_iterations(5)  -- Safety limit (applies to all iterator types)
    :add_step({ ... })
    :build()
```

### Features
- **Range Iterator**: Numeric iteration with start, end, step
- **Collection Iterator**: Iterate over array of values
- **While Iterator**: Condition-based iteration with safety limit
- **Max Iterations**: Properly limits all iterator types

### Test Results
```lua
-- Range 2-6 executes 4 iterations (2,3,4,5) ✅
:with_range({ start = 2, ["end"] = 6, step = 1 })

-- Range 1-10 limited to 3 iterations ✅
:with_range({ start = 1, ["end"] = 10, step = 1 })
:max_iterations(3)

-- Collection processes all items ✅
:with_collection({ "item1", "item2", "item3" })
```

### To Be Implemented In
- **file-organizer**: Batch processing of files
- **webapp-creator**: Iterative code generation

## 4. Nested Workflows (Coming Soon)
Compose workflows within workflows for complex orchestration patterns.

### Planned API
```lua
local sub_workflow = Workflow.builder()
    :name("sub_process")
    :sequential()
    :add_step({ ... })
    :build()

local main_workflow = Workflow.builder()
    :name("main_process")
    :sequential()
    :add_step({
        type = "workflow",
        workflow = sub_workflow
    })
    :build()
```

## Migration Guide

### For Conditional Workflows
If you were using placeholder conditions:
```lua
-- Old (placeholder)
:condition({ type = "always" })

-- New (with real conditions)
:condition({ 
    type = "shared_data_equals",
    key = "status",
    value = "active"
})
```

### For Loop Workflows
Replace manual iteration logic with loop workflows:
```lua
-- Old (manual)
for i = 1, 10 do
    -- process item
end

-- New (workflow)
Workflow.builder()
    :loop()
    :with_range({ start = 1, ["end"] = 11, step = 1 })
    :add_step({ ... })
    :build()
```

## Performance Considerations

1. **Parallel Workflows**: 
   - Set `max_concurrency` based on resource availability
   - Ideal for independent operations (e.g., quality checks)

2. **Conditional Workflows**:
   - Shared data operations are O(1) with unified state
   - Conditions evaluated once at branch selection

3. **Loop Workflows**:
   - `max_iterations` prevents infinite loops
   - State persisted per iteration for recovery

## Testing

Run the test scripts to verify functionality:
```bash
# Test conditional workflows
./target/debug/llmspell run /tmp/test_shared_data_equals.lua

# Test loop workflows
./target/debug/llmspell run /tmp/test_loop_improved.lua

# Test parallel workflows (in content-creator)
./target/debug/llmspell run examples/script-users/applications/content-creator/main.lua
```

## Implementation Status

| Workflow Type | Core | Bridge | Lua API | Applications | Tests |
|--------------|------|--------|---------|--------------|-------|
| Sequential | ✅ | ✅ | ✅ | ✅ All | ✅ |
| Parallel | ✅ | ✅ | ✅ | ✅ content-creator | ✅ |
| Conditional | ✅ | ✅ | ✅ | ✅ communication-manager, process-orchestrator | ✅ |
| Loop | ✅ | ✅ | ✅ | 🔄 Pending | ✅ |
| Nested | ✅ | 🔄 | 🔄 | 🔄 Pending | 🔄 |

## Technical Details

### File Locations
- **Core Implementation**: `llmspell-workflows/src/`
  - `conditional.rs` - Conditional workflow with unified state
  - `loop.rs` - Loop workflow with iterators
  - `parallel.rs` - Parallel execution
  
- **Bridge Layer**: `llmspell-bridge/src/`
  - `workflows.rs` - WorkflowBridge with create methods
  - `state_adapter.rs` - NoScopeStateAdapter for state access
  
- **Lua API**: `llmspell-bridge/src/lua/globals/`
  - `workflow.rs` - WorkflowBuilder with all methods

### Key Fixes
1. **Conditional State Integration** (conditional.rs:490-521)
   - Reads from `context.state` instead of internal state_manager
   - Properly accesses workflow-specific and global shared data

2. **Loop Max Iterations** (workflows.rs:1538-1544)
   - Calculates proper end value for ranges
   - Truncates collections to max size
   - Respects limit for while conditions

3. **State Adapter** (state_adapter.rs:385-397)
   - Fixed `NoScopeStateAdapter::list_keys` to return actual keys
   - Properly strips "custom::" prefix from keys