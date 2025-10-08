# Advanced Patterns

Production-ready patterns for complex LLMSpell applications.

## Overview

This directory contains advanced patterns that bridge between basic features and full applications. Each pattern demonstrates production-ready code with proper error handling, performance considerations, and security controls.

## Files (4)

### 1. multi-agent-orchestration.lua
**Purpose**: Complex multi-agent coordination patterns  
**Key Patterns**:
- Agent specialization and role definition
- Task delegation between agents
- Consensus building across agents
- Error recovery with fallback agents
- Pipeline pattern with multiple agents
- Parallel agent processing
- Performance monitoring

**Prerequisites**: API keys (OPENAI_API_KEY or ANTHROPIC_API_KEY)  
**Runtime**: 15-30 seconds with API calls

### 2. complex-workflows.lua
**Purpose**: Advanced workflow orchestration patterns  
**Key Patterns**:
- Multi-stage sequential pipelines
- Parallel processing with aggregation
- Conditional routing with shared state
- Multi-branch priority systems
- Nested workflow composition
- Error recovery workflows
- Performance optimization

**Prerequisites**: None (uses local tools only)  
**Runtime**: 2-5 seconds

### 3. tool-integration-patterns.lua
**Purpose**: Advanced tool usage and integration  
**Key Patterns**:
- Tool chaining for complex operations
- Parallel tool execution
- System integration (environment, processes, services)
- Database operations (with configuration)
- Email integration (with credentials)
- Error recovery in tool chains
- Rate limiting and circuit breakers (simulated)

**Prerequisites**: Optional external service credentials  
**Runtime**: 1-3 seconds

### 4. monitoring-security.lua
**Purpose**: Production monitoring and security patterns  
**Key Patterns**:
- System health monitoring with agents
- File system security controls
- Process execution sandboxing
- Anomaly detection
- Security audit logging
- Data encryption patterns
- Rate limiting for security
- Environment security checks

**Prerequisites**: Optional API keys for agent monitoring  
**Runtime**: 2-5 seconds

## Progression Path

```
features/ (basics) → advanced-patterns/ (this) → cookbook/ (complete apps)
```

## Usage Examples

### Run all patterns
```bash
for file in *.lua; do
    echo "Running $file..."
    ./target/debug/llmspell run examples/script-users/advanced-patterns/$file
done
```

### With API keys (for agent patterns)
```bash
OPENAI_API_KEY=$OPENAI_API_KEY ./target/debug/llmspell run \
    examples/script-users/advanced-patterns/multi-agent-orchestration.lua
```

### Quick validation
```bash
# Test without API calls
./target/debug/llmspell run \
    examples/script-users/advanced-patterns/complex-workflows.lua
```

## Key Concepts Demonstrated

### 1. Production Error Handling
- Proper use of `pcall` for error catching
- Graceful degradation patterns
- Fallback strategies
- Error recovery workflows

### 2. Performance Optimization
- Parallel execution where appropriate
- Efficient tool chaining
- Performance monitoring
- Resource limit management

### 3. Security Best Practices
- Path traversal prevention
- Command whitelisting
- Rate limiting
- Audit logging
- Secure data handling

### 4. State Management
- Shared data for conditional workflows
- State persistence between steps
- Cross-workflow communication
- Workflow metadata tracking

## Common Issues & Solutions

### Issue: Agent creation fails
**Solution**: Ensure API keys are set:
```bash
export OPENAI_API_KEY="your-key"
export ANTHROPIC_API_KEY="your-key"
```

### Issue: Conditional workflows not routing correctly
**Solution**: Use table-based conditions with shared_data:
```lua
workflow:set_shared_data("key", "value")
builder:condition({
    type = "shared_data_equals",
    key = "key",
    value = "value"
})
```

### Issue: Tool not found errors
**Solution**: Check available tools:
```lua
local tools = Tool.list()
for i, tool in ipairs(tools) do
    print(tool)
end
```

### Issue: Text manipulator operation invalid
**Solution**: Use template_engine for complex text operations:
```lua
-- Instead of text_manipulator with "prepend"/"append"
Tool.execute("template_engine", {
    input = "prefix {{content}} suffix",
    context = {content = "your text"}
})
```

## Best Practices

1. **Always validate agent creation**
   ```lua
   local success, agent = pcall(function()
       return Agent.builder():name("test"):build()
   end)
   if success then
       -- Use agent
   end
   ```

2. **Use proper workflow result handling**
   ```lua
   local result = workflow:execute({})
   if result and result.text and result.text:match("completed successfully") then
       -- Success
   end
   ```

3. **Implement security checks**
   ```lua
   -- Always validate paths and commands
   -- Use whitelisted operations only
   -- Log security events
   ```

4. **Monitor performance**
   ```lua
   local start_time = os.clock()
   -- Operation
   local duration = (os.clock() - start_time) * 1000
   ```

## Next Steps

After mastering these patterns:
1. Explore `cookbook/` for complete applications
2. Review `applications/` for production examples
3. Build your own patterns combining these techniques

## Architecture Notes

These patterns demonstrate the full power of LLMSpell's architecture:
- **Agents**: Autonomous AI components with specialized roles
- **Workflows**: Orchestration of complex multi-step processes
- **Tools**: Integration with external systems and services
- **Security**: Production-ready security controls

Each pattern is self-contained but can be combined for more complex scenarios.