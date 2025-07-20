# Workflow Bridge Integration

This document describes the workflow bridge integration completed in Task 3.3.16, including multi-agent coordination capabilities.

## Overview

The workflow bridge provides comprehensive script-to-workflow integration, enabling Lua and JavaScript scripts to:
- Discover and create workflows
- Execute workflows with parameters
- Coordinate multiple agents through workflow patterns
- Monitor performance and execution history

## Components Implemented

### 1. Core Workflow Bridge (`workflow_bridge.rs`)
- WorkflowBridge struct with discovery, factory, and execution
- Performance optimization with <10ms overhead
- Execution history tracking
- Comprehensive metrics collection

### 2. Workflow Discovery (`workflows.rs`)
- WorkflowDiscovery service for type enumeration
- WorkflowInfo metadata for each workflow type
- Dynamic workflow type registration

### 3. Parameter Conversion (`workflow_conversion.rs`, `workflow_conversion_core.rs`)
- Language-agnostic conversion utilities
- Lua-specific table to JSON conversion
- Type-safe parameter validation

### 4. Multi-Agent Coordination (`multi_agent_workflow.rs`)
- Pipeline pattern for sequential agent collaboration
- Fork-Join pattern for parallel agent execution
- Consensus pattern for multi-agent decision making
- Additional patterns: Delegation, Collaboration, Hierarchical

### 5. Performance Optimization (`workflow_performance.rs`)
- Parameter validation cache with pre-compiled validators
- Execution result cache (LRU, 100 entries, 60s TTL)
- Workflow type information cache
- Real-time performance metrics (average, P99)
- <10ms overhead verification

### 6. Lua Integration (`lua/api/workflow.rs`)
- Data-oriented API design (no complex closures)
- Workflow constructors return configuration tables
- Single execute function with bridge stored in Lua registry
- Full support for all workflow types

## Multi-Agent Coordination Patterns

### Pipeline Pattern
```rust
pub fn create_pipeline_workflow(
    name: String,
    agents: Vec<String>,
    initial_input: Value,
) -> Result<Box<dyn Workflow>>
```

Sequential processing where each agent's output feeds the next agent's input.

### Fork-Join Pattern
```rust
pub fn create_fork_join_workflow(
    name: String,
    agent_tasks: Vec<(String, String, Value)>,
    coordinator: Option<String>,
) -> Result<Box<dyn Workflow>>
```

Parallel execution of multiple agents with optional result coordination.

### Consensus Pattern
```rust
pub fn create_consensus_workflow(
    name: String,
    evaluators: Vec<String>,
    consensus_threshold: f64,
    options: Value,
) -> Result<Box<dyn Workflow>>
```

Multiple agents evaluate options and reach consensus based on threshold.

## Lua API

### Basic Usage
```lua
-- List workflow types
local types = Workflow.types()

-- Get workflow info
local info = Workflow.info("sequential")

-- Create workflow
local workflow = Workflow.sequential({
    name = "test",
    steps = [...]
})

-- Execute workflow
local result = Workflow.execute(workflow, {input = "data"})
```

### Multi-Agent Examples
See the following example files:
- `examples/multi_agent_pipeline.lua` - Research pipeline with 3 agents
- `examples/multi_agent_parallel.lua` - Document analysis with 4 parallel agents
- `examples/multi_agent_consensus.lua` - Investment decision with expert consensus

## Performance Characteristics

### Measured Performance
- Workflow creation: <5ms average
- Workflow execution: <10ms overhead (excluding actual work)
- Parameter validation: <1ms with cache
- Type discovery: <1ms with cache

### Optimization Techniques
1. Pre-compiled parameter validators
2. LRU execution cache for repeated workflows
3. Static type information cache
4. Lazy initialization of heavy components

## Testing

### Unit Tests
- `workflow_bridge_basic_tests.rs` - Core bridge functionality
- `lua_workflow_api_tests.rs` - Lua integration tests
- `multi_agent_workflow_tests.rs` - Multi-agent coordination tests

### Performance Benchmarks
- `benches/workflow_bridge_bench.rs` - Performance verification
  - Workflow creation benchmarks
  - Discovery operation benchmarks
  - Parameter conversion benchmarks
  - End-to-end execution benchmarks

## Integration Points

### With Agent Infrastructure
- Agents can be invoked as workflow steps
- Agent outputs feed into subsequent steps
- Agent coordination through workflow patterns

### With Tool System
- Tools can be workflow steps
- Tool composition through workflows
- Tool results as workflow data

### With Script Engines
- Lua integration complete
- JavaScript integration pending
- Consistent API across languages

## Future Work

### Remaining Items
1. JavaScript workflow API implementation
2. Advanced performance optimizations
3. Distributed workflow execution
4. Workflow persistence and recovery

### Completed Items
- ✅ WorkflowBridge implementation
- ✅ Workflow discovery and factory
- ✅ Parameter conversion system
- ✅ Multi-agent coordination patterns
- ✅ Performance optimization (<10ms)
- ✅ Lua API implementation
- ✅ Comprehensive testing
- ✅ Documentation

## Migration Guide

For users upgrading from direct workflow usage:

### Before (Direct Workflow Creation)
```rust
let workflow = SequentialWorkflow::new("test", vec![...]);
workflow.execute(input).await?;
```

### After (Through Bridge)
```lua
local workflow = Workflow.sequential({
    name = "test",
    steps = [...]
})
local result = Workflow.execute(workflow, input)
```

## Troubleshooting

### Common Issues

1. **"Invalid parameters for workflow type"**
   - Use `Workflow.info(type)` to see required parameters
   - Ensure all required fields are present

2. **Performance degradation**
   - Check `Workflow.performance()` metrics
   - Verify cache is working (hit rate > 80%)
   - Ensure workflows aren't too complex

3. **Multi-agent coordination failures**
   - Verify all agents are registered
   - Check agent availability with `Agent.list()`
   - Ensure parameter compatibility between agents