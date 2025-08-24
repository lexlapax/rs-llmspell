# WebApp Creator Configuration Guide

## Overview

WebApp Creator is a comprehensive multi-agent workflow that orchestrates 20 specialized AI agents to generate complete web applications. This guide covers configuration, timeout management, and troubleshooting.

## Architecture Validation

WebApp Creator serves as the ultimate framework validator for llmspell, exercising:
- **Agent Orchestration**: 20 agents working in sequence
- **State Management**: Persistent state across workflow execution
- **Tool Integration**: File operations for code generation
- **Event System**: Workflow lifecycle events
- **Component Registry**: Dynamic component lookup and execution
- **Timeout Handling**: Long-running LLM operations

## Required Configuration

### API Keys

WebApp Creator requires access to LLM providers:

```bash
# Set environment variables
export OPENAI_API_KEY="your-openai-key"
export ANTHROPIC_API_KEY="your-anthropic-key"
```

### config.toml Structure

```toml
[providers.openai]
api_key = "${OPENAI_API_KEY}"
models = ["gpt-4", "gpt-3.5-turbo"]
default_model = "gpt-4"
timeout = 120000  # 2 minutes for OpenAI

[providers.anthropic]
api_key = "${ANTHROPIC_API_KEY}"
models = ["claude-3-5-sonnet-20241022", "claude-3-opus-20240229"]
default_model = "claude-3-5-sonnet-20241022"
timeout = 180000  # 3 minutes for Claude

[state]
enabled = true
path = ".llmspell/state"
persistence = true

[tools.file-operations]
allowed_paths = ["./generated", "/tmp", "./examples"]
max_file_size = "10MB"
create_dirs = true

[workflows]
default_timeout_ms = 120000  # 2 minutes default
max_parallel_steps = 1  # Sequential execution for WebApp Creator
```

## Critical: Timeout Configuration

### The 30-Second Problem

By default, workflow steps timeout after 30 seconds (defined in `llmspell-workflows/src/types.rs:298`). This is insufficient for LLM agents generating code.

### Solution: Configure Timeouts in Lua

```lua
-- In main.lua, configure timeout per step type
local step_config = {
    name = agent_name,
    type = "agent",
    agent_id = agent.id,
    timeout_ms = nil  -- Will be set based on agent type
}

-- Set appropriate timeouts based on agent role
if name:match("developer") or name:match("engineer") then
    step_config.timeout_ms = 120000  -- 2 minutes for code generation
elseif name:match("architect") or name:match("designer") then
    step_config.timeout_ms = 90000   -- 1.5 minutes for design work
elseif name:match("analyst") or name:match("researcher") then
    step_config.timeout_ms = 60000   -- 1 minute for analysis
else
    step_config.timeout_ms = 45000   -- 45 seconds for simple tasks
end
```

### Timeout Fix Implementation

The timeout configuration was fixed in `llmspell-bridge/src/lua/globals/workflow.rs`:

```rust
// Lines 844-850: Parse and apply timeout from Lua
if let Ok(timeout_ms) = step_table.get::<_, u64>("timeout_ms") {
    debug!("Step timeout requested: {}ms", timeout_ms);
    final_step = final_step.with_timeout(
        std::time::Duration::from_millis(timeout_ms)
    );
}
```

## Model Selection

### Per-Agent Model Configuration

Different agents can use different models based on their requirements:

```lua
-- Use GPT-4 for complex reasoning
agents.requirements_analyst = Agent.builder()
    :name("requirements_analyst")
    :type("llm")
    :provider("openai")
    :model("gpt-4")
    :temperature(0.7)
    :build()

-- Use Claude Sonnet for code generation (better at structured output)
agents.frontend_developer = Agent.builder()
    :name("frontend_developer")
    :type("llm")
    :provider("anthropic")
    :model("claude-3-5-sonnet-20241022")
    :temperature(0.3)  -- Lower temperature for code
    :max_tokens(4000)  -- More tokens for complete code
    :build()
```

## Running WebApp Creator

### Basic Usage

```bash
# With e-commerce example
./target/release/llmspell run \
  examples/script-users/applications/webapp-creator/main.lua \
  -- --input user-input-ecommerce.lua --output ./generated

# With default TaskFlow example
./target/release/llmspell run \
  examples/script-users/applications/webapp-creator/main.lua \
  -- --output ./generated

# With custom input
./target/release/llmspell run \
  examples/script-users/applications/webapp-creator/main.lua \
  -- --input my-project.lua --output ./my-app
```

### Debug Mode

```bash
# Enable debug logging to see timeout and execution details
RUST_LOG=debug ./target/debug/llmspell run \
  examples/script-users/applications/webapp-creator/main.lua \
  -- --input user-input-ecommerce.lua --output /tmp/debug-test
```

## Common Issues and Solutions

### Issue 1: "Step timed out after 30 seconds"

**Symptom**: Workflow fails at `frontend_developer` or other code generation steps.

**Cause**: Default timeout too short for LLM operations.

**Solution**: Ensure `timeout_ms` is configured in workflow steps (see Timeout Configuration above).

### Issue 2: "No registry available in context"

**Symptom**: Workflow executes but returns mock data instead of real agent outputs.

**Cause**: ComponentRegistry not properly threaded through execution context.

**Solution**: This was fixed in Task 10.1-10.6. Ensure you're using the latest build from Phase-7 branch.

### Issue 3: "Agent not found in registry"

**Symptom**: Workflow fails to find registered agents.

**Cause**: Agents not properly registered or registry not accessible.

**Solution**: 
```lua
-- Ensure agents are registered before workflow creation
local agent = Agent.builder():name("test"):type("llm"):build()
-- Agent is automatically registered upon creation

-- Verify in workflow step
local step = {
    name = "test_step",
    type = "agent",
    agent_id = agent.id  -- Use the agent's ID, not name
}
```

### Issue 4: "API rate limit exceeded"

**Symptom**: Agents fail with rate limit errors.

**Cause**: Too many rapid API calls to LLM providers.

**Solution**:
1. Add delays between agent executions
2. Use different API keys for different agents
3. Configure retry logic with exponential backoff

### Issue 5: "State not persisting between steps"

**Symptom**: Later agents can't access outputs from earlier agents.

**Cause**: State not properly configured or not being passed through execution context.

**Solution**:
```toml
# Ensure state is enabled in config.toml
[state]
enabled = true
persistence = true
```

## Performance Considerations

### Execution Times

Based on successful runs:
- **E-commerce App (ShopEasy)**: ~168 seconds for 20 agents
- **Task Management App (TaskFlow)**: ~174 seconds for 20 agents
- **Average per agent**: 8-9 seconds (including LLM latency)

### Optimization Tips

1. **Parallel Execution**: For independent agents, consider parallel workflow
2. **Caching**: Enable state caching to avoid regenerating unchanged components
3. **Model Selection**: Use faster models (GPT-3.5, Claude Instant) for simpler tasks
4. **Token Limits**: Configure appropriate `max_tokens` to avoid unnecessary generation

## Architectural Insights

### Single Execution Path

WebApp Creator validates the unified execution architecture:
- All components (agents, tools, workflows) implement `BaseAgent`
- Single path: `execute()` → `execute_impl()`
- State and events handled uniformly

### Component Registry

The framework properly manages component lifecycle:
- Agents auto-register on creation
- Registry accessible throughout execution
- Dynamic lookup enables flexible workflows

### State Management

Robust state persistence across workflow:
- Each agent's output saved with unique key
- State accessible to subsequent agents
- Survives process restarts with persistence enabled

## Monitoring and Debugging

### Event Tracking

Monitor workflow execution through events:

```lua
-- Workflow events are automatically emitted
-- workflow.started
-- workflow.step.started
-- workflow.step.completed
-- workflow.completed
```

### Logging

Enable detailed logging for troubleshooting:

```bash
# Component-specific logging
RUST_LOG=llmspell_workflows=debug    # Workflow execution
RUST_LOG=llmspell_agents=debug       # Agent operations
RUST_LOG=llmspell_bridge=debug       # Lua bridge operations

# Full debug output
RUST_LOG=debug ./target/debug/llmspell run main.lua
```

## Production Deployment

### Recommendations

1. **Use Release Build**: 10-20x performance improvement
   ```bash
   cargo build --release
   ```

2. **Configure Appropriate Timeouts**: Based on your LLM provider and complexity

3. **Enable State Persistence**: For recovery from failures

4. **Monitor API Usage**: Track costs and rate limits

5. **Implement Retry Logic**: Handle transient failures gracefully

## Validated Capabilities

Through extensive testing, WebApp Creator has validated:

✅ **20 Sequential Agents**: Successfully orchestrated without failures
✅ **State Persistence**: All outputs correctly saved and retrieved
✅ **Event Emission**: Lifecycle events properly emitted
✅ **Tool Integration**: File operations create complete project structure
✅ **Timeout Handling**: Long-running operations complete successfully
✅ **Model Flexibility**: Different models for different agent types
✅ **Error Recovery**: Graceful handling of failures with state preservation

## Conclusion

WebApp Creator serves as both a powerful application generator and a comprehensive validation suite for the llmspell framework. Its successful execution demonstrates the production readiness of the entire system for complex, multi-agent workflow orchestration.