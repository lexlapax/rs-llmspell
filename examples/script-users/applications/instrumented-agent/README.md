# Instrumented Agent Debugger

**Layer**: Development/Debug Tools
**Status**: ✅ Working
**Purpose**: Demonstrate debugging and tracing capabilities for agent applications
**Agents**: 2 (analyzer + reviewer)

## What This Example Does

This application demonstrates how to instrument your agent applications with debugging, tracing, and state inspection capabilities. It's not another "app" but rather a template showing debugging techniques you can apply to your own agent applications.

### Key Features Demonstrated

1. **Debug Logging** - Multiple log levels with module tagging
2. **Performance Timing** - Measure execution time with Debug.timer()
3. **State Persistence** - Checkpoint and recovery patterns
4. **REPL Inspection** - Inspect saved state after execution
5. **Workflow Debugging** - Trace multi-agent workflows

## Prerequisites

- **API Keys Required**: Set `OPENAI_API_KEY` environment variable
- Without API keys, runs in demo mode showing debugging features

## Quick Start

```bash
# Load your API keys
source .env  # or export OPENAI_API_KEY="your-key"

# Run with configuration
./target/debug/llmspell -c examples/script-users/applications/instrumented-agent/config.toml \
  run examples/script-users/applications/instrumented-agent/main.lua

# For extra debug output
./target/debug/llmspell --trace debug -c examples/script-users/applications/instrumented-agent/config.toml \
  run examples/script-users/applications/instrumented-agent/main.lua
```

## Using the REPL for Inspection

After running the application, use the REPL to inspect the saved state:

### Start the REPL
```bash
./target/debug/llmspell repl
```

### Inspection Commands

```lua
-- Load the analysis result
State.load('custom', ':last_analysis')

-- Check the checkpoint
State.load('custom', ':checkpoint:pre_analysis')

-- View recent debug logs
Debug.getCapturedEntries(10)

-- Check debug configuration
Debug.getLevel()
Debug.isEnabled()

-- List all custom state keys
State.list_keys('custom:')

-- Check current session
Session.get_current()
```

## Debugging Techniques Shown

### 1. Performance Timing
```lua
local timer = Debug.timer("operation_name")
-- ... do work ...
local duration = timer:stop()
Debug.info("Operation took " .. duration .. "ms", "module")
```

### 2. State Checkpointing
```lua
-- Save checkpoint before risky operation
State.save("custom", ":checkpoint:before_operation", {
    timestamp = os.time(),
    input_size = #input
})

-- Later, recover if needed
local checkpoint = State.load("custom", ":checkpoint:before_operation")
```

### 3. Debug Logging Levels
```lua
Debug.error("Critical error occurred", "module")
Debug.warn("Warning condition", "module")
Debug.info("Information message", "module")
Debug.debug("Debug details", "module")
```

### 4. Agent Nil Checking
```lua
local agent = Agent.builder():name("test"):build()
if not agent then
    Debug.warn("No API key - using demo mode", "module")
    return
end
```

## Files Generated

- `/tmp/instrumented-analysis-{timestamp}.md` - Analysis results with debug info

## Configuration

The `config.toml` demonstrates:
- Debug level configuration
- Provider setup for OpenAI/Anthropic
- State persistence settings
- Performance tracking enablement

## Extending This Example

To add debugging to your own applications:

1. **Add Debug Logging**
   - Use appropriate log levels
   - Tag with module names
   - Include timing information

2. **Implement Checkpointing**
   - Save state before critical operations
   - Enable recovery on failure
   - Use State.save() and State.load()

3. **Measure Performance**
   - Use Debug.timer() for operations
   - Log durations for analysis
   - Identify bottlenecks

4. **Enable REPL Inspection**
   - Save important results to State
   - Use meaningful key names
   - Document inspection commands

## Value for Developers

This example teaches you how to:
- Debug agent execution problems
- Measure and optimize performance
- Implement error recovery
- Inspect application state post-execution
- Trace multi-agent workflows

## Common Issues and Solutions

### No API Keys
```
❌ No agents created - check API keys
```
**Solution**: Set OPENAI_API_KEY environment variable

### State Not Found in REPL
```lua
> State.load('custom', ':last_analysis')
nil
```
**Solution**: Ensure the application ran successfully first

### Debug Logs Not Captured
**Solution**: Check debug is enabled in config.toml:
```toml
[runtime.debug]
enabled = true
level = "debug"
```

## Next Steps

1. Run the application with your API keys
2. Use REPL to inspect the state
3. Apply these patterns to your own agent applications
4. Experiment with different debug levels
5. Add performance profiling to your workflows

---

**Note**: This is a debugging template, not a production application. Use these techniques to instrument your real agent applications for better observability and debugging.