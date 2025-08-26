# Troubleshooting Guide

**Version**: 0.6.0  
**Last Updated**: August 2025

> **🔧 Quick Reference**: Common issues, debugging techniques, and performance optimization for LLMSpell.

**🔗 Navigation**: [← User Guide](README.md) | [Configuration](configuration.md) | [Getting Started](getting-started.md)

---

## Table of Contents

1. [Common Issues](#common-issues)
2. [Debugging Techniques](#debugging-techniques)
3. [Performance Issues](#performance-issues)
4. [API and Provider Issues](#api-and-provider-issues)
5. [Memory and Resource Issues](#memory-and-resource-issues)
6. [Script Errors](#script-errors)
7. [Configuration Problems](#configuration-problems)
8. [Tool-Specific Issues](#tool-specific-issues)
9. [Advanced Debugging](#advanced-debugging)
10. [Getting Help](#getting-help)

---

## Common Issues

### "No API key found"

**Symptoms:**
```
Error: No API key configured for provider 'openai'
```

**Solutions:**
```bash
# Solution 1: Set environment variable
export OPENAI_API_KEY="sk-..."

# Solution 2: Use configuration file
echo '[providers.openai]' >> config.toml
echo 'api_key = "sk-..."' >> config.toml
./llmspell -c config.toml run script.lua

# Solution 3: Check if variable is set
echo $OPENAI_API_KEY
```

### "Model not available"

**Symptoms:**
```
Error: Model 'gpt-4' not available for your account
```

**Solutions:**
```lua
-- Use a different model you have access to
local agent = Agent.builder()
    :model("openai/gpt-3.5-turbo")  -- Instead of gpt-4
    :build()

-- Check available models for your account
-- Visit: https://platform.openai.com/account/limits
```

### "Script doesn't run"

**Symptoms:**
- Permission denied
- Command not found
- No output

**Solutions:**
```bash
# Make executable
chmod +x ./target/release/llmspell

# Use full path
./target/release/llmspell run script.lua

# Check if built
ls -la ./target/release/llmspell

# Build if missing
cargo build --release
```

### "Rate limit exceeded"

**Symptoms:**
```
Error: Rate limit exceeded. Please retry after X seconds
```

**Solutions:**
```lua
-- Solution 1: Add delays between requests
local function delayed_execute(agent, prompts)
    for i, prompt in ipairs(prompts) do
        local result = agent:execute({prompt = prompt})
        if i < #prompts then
            os.execute("sleep 1")  -- 1 second delay
        end
    end
end

-- Solution 2: Use exponential backoff
local function execute_with_retry(agent, input, max_retries)
    max_retries = max_retries or 3
    local delay = 1
    
    for attempt = 1, max_retries do
        local success, result = pcall(function()
            return agent:execute(input)
        end)
        
        if success then
            return result
        end
        
        if attempt < max_retries then
            print("Retry " .. attempt .. " after " .. delay .. "s")
            os.execute("sleep " .. delay)
            delay = delay * 2  -- Exponential backoff
        end
    end
    
    error("Failed after " .. max_retries .. " retries")
end
```

---

## Debugging Techniques

### Enable Debug Logging

**Via Environment:**
```bash
RUST_LOG=debug ./llmspell run script.lua
RUST_LOG=trace ./llmspell run script.lua  # Very verbose
```

**Via Configuration:**
```toml
[global]
debug = true
log_level = "debug"  # or "trace" for maximum detail
```

**Via Script:**
```lua
-- Enable debug output
Debug.setEnabled(true)
Debug.setLevel("debug")

-- Log messages with modules
Debug.info("Starting process", "main")
Debug.debug("Input data: " .. Debug.dump(data), "processor")
Debug.error("Connection failed: " .. error_msg, "network")
```

### Performance Profiling

```lua
-- Time operations
local timer = Debug.timer("heavy_operation")

-- Do work
for i = 1, 1000000 do
    -- processing
end

local duration = timer:stop()
print("Operation took: " .. duration .. "ms")

-- Get performance report
print(Debug.performanceReport())
```

### Object Inspection

```lua
-- Dump complex objects
local data = {
    nested = {
        values = {1, 2, 3},
        config = {enabled = true}
    }
}

print(Debug.dump(data))  -- Pretty-printed output
print(Debug.dump(data, {max_depth = 2}))  -- Limit depth
```

---

## Performance Issues

### Script Running Slowly

**Diagnosis:**
```lua
-- Profile your script
local timer = Debug.timer("main")

timer:lap("initialization")
-- init code

timer:lap("processing")
-- main logic

timer:lap("cleanup")
-- cleanup

local total = timer:stop()
print(Debug.performanceReport())
```

**Common Fixes:**

1. **Cache table lookups:**
```lua
-- Slow
for i = 1, 10000 do
    process(data.users[i].profile.name)
end

-- Fast
for i = 1, 10000 do
    local profile = data.users[i].profile
    process(profile.name)
end
```

2. **Use local variables:**
```lua
-- Slow (global)
count = 0
for i = 1, 1000000 do
    count = count + 1
end

-- Fast (local)
local count = 0
for i = 1, 1000000 do
    count = count + 1
end
```

3. **Efficient string concatenation:**
```lua
-- Slow
local result = ""
for i = 1, 1000 do
    result = result .. "Line " .. i .. "\n"
end

-- Fast
local lines = {}
for i = 1, 1000 do
    lines[i] = "Line " .. i
end
local result = table.concat(lines, "\n")
```

### Memory Usage High

**Monitor memory:**
```lua
-- Check memory usage
local mem_before = collectgarbage("count")
-- ... your code ...
local mem_after = collectgarbage("count")
print("Memory used: " .. (mem_after - mem_before) .. " KB")

-- Force garbage collection
collectgarbage("collect")
```

**Reduce memory:**
```lua
-- Clear large tables when done
local function process_data()
    local huge_table = load_data()
    local result = analyze(huge_table)
    
    huge_table = nil  -- Release reference
    collectgarbage()  -- Clean up
    
    return result
end
```

---

## API and Provider Issues

### Connection Timeouts

**Symptoms:**
```
Error: Request timeout after 30 seconds
```

**Solutions:**
```toml
# Increase timeout in config
[providers.openai]
timeout = 60  # Seconds

[tools.web]
timeout = 45
```

### SSL/TLS Errors

**Solutions:**
```bash
# Behind corporate proxy
export HTTP_PROXY="http://proxy.company.com:8080"
export HTTPS_PROXY="http://proxy.company.com:8080"
export NO_PROXY="localhost,127.0.0.1"

# Custom certificates
export SSL_CERT_FILE="/path/to/cert.pem"
```

### Authentication Failures

**Debug authentication:**
```lua
-- Test with minimal request
local agent = Agent.builder()
    :model("openai/gpt-3.5-turbo")
    :max_tokens(10)  -- Minimal tokens
    :build()

local success, result = pcall(function()
    return agent:execute({prompt = "test"})
end)

if not success then
    print("Auth error: " .. tostring(result))
end
```

---

## Memory and Resource Issues

### "Memory limit exceeded"

**Solutions:**
```toml
# Increase limits in config
[resources.profiles.default]
memory_limit = "1GB"  # Increase from 512MB
```

### "Too many open files"

**Solutions:**
```bash
# Check current limit
ulimit -n

# Increase limit (Unix/Linux)
ulimit -n 4096

# Or in config
[security.sandboxing.filesystem]
max_open_files = 1000
```

### Stack Overflow

**Common cause:** Infinite recursion

**Fix:**
```lua
-- Add recursion depth check
local function recursive_function(data, depth)
    depth = depth or 0
    if depth > 100 then
        error("Maximum recursion depth exceeded")
    end
    
    -- ... logic ...
    
    if condition then
        recursive_function(new_data, depth + 1)
    end
end
```

---

## Script Errors

### "Attempt to index nil value"

**Common causes and fixes:**
```lua
-- Check if value exists
if data and data.field then
    process(data.field)
end

-- Use safe navigation
local value = data and data.nested and data.nested.field

-- Provide defaults
local config = data.config or {}
local timeout = config.timeout or 30
```

### "Bad argument type"

**Debug type issues:**
```lua
-- Check types before operations
local function safe_concat(a, b)
    if type(a) ~= "string" then
        a = tostring(a)
    end
    if type(b) ~= "string" then
        b = tostring(b)
    end
    return a .. b
end

-- Validate function arguments
local function process(data)
    assert(type(data) == "table", "Expected table, got " .. type(data))
    -- ... process ...
end
```

### Async/Await Issues

**Common pattern for handling async:**
```lua
-- Proper error handling for async operations
local function safe_execute(agent, input)
    local success, result = pcall(function()
        return agent:execute(input)
    end)
    
    if not success then
        Debug.error("Execution failed: " .. tostring(result), "agent")
        return nil, result
    end
    
    return result
end
```

---

## Configuration Problems

### Config File Not Loading

**Debug config loading:**
```bash
# Validate config
./llmspell validate -c config.toml

# Check which config is loaded
RUST_LOG=debug ./llmspell -c config.toml run script.lua 2>&1 | grep -i config

# Use absolute path
./llmspell -c /absolute/path/to/config.toml run script.lua
```

### Environment Variables Not Working

**Common issues:**
```bash
# Variable not exported
OPENAI_API_KEY="sk-..."  # Wrong - not exported
export OPENAI_API_KEY="sk-..."  # Correct

# Check if set
echo $OPENAI_API_KEY

# Debug with env command
env | grep OPENAI
```

---

## Tool-Specific Issues

### File Operations Fail

**Common causes:**
```lua
-- Check permissions
local file_tool = Tool.get("file-operations")

-- Test with simple operation
local success, result = pcall(function()
    return file_tool:execute({
        operation = "read",
        path = "/tmp/test.txt"
    })
end)

if not success then
    print("File error: " .. tostring(result))
    -- Check: Does file exist? Permissions? Path correct?
end
```

### Web Tools Timeout

**Solutions:**
```lua
-- Increase timeout for specific operation
local web_tool = Tool.get("web-fetch")
local result = web_tool:execute({
    url = "https://slow-api.com",
    timeout = 60  -- Increase timeout
})

-- Use retry logic
local function fetch_with_retry(url, retries)
    for i = 1, retries do
        local success, result = pcall(function()
            return web_tool:execute({url = url})
        end)
        if success then return result end
        if i < retries then
            os.execute("sleep " .. (2^i))  -- Exponential backoff
        end
    end
    error("Failed after " .. retries .. " attempts")
end
```

---

## Advanced Debugging

### Enable Trace Logging

```bash
# Maximum verbosity
RUST_LOG=trace ./llmspell run script.lua 2> debug.log

# Filter by module
RUST_LOG=llmspell_agents=debug,llmspell_tools=trace ./llmspell run script.lua

# Specific components
RUST_LOG=llmspell_bridge::lua=trace ./llmspell run script.lua
```

### Use Debug Hooks

```lua
-- Add debug hook to track execution
Hook.register("BeforeAgentExecution", function(context)
    Debug.info("Agent executing: " .. context.component_id.name, "hooks")
    Debug.debug("Input: " .. Debug.dump(context.data.input), "hooks")
    return "continue"
end, "high")

-- Monitor tool invocations
Hook.register("AfterToolInvocation", function(context)
    local duration = context.data.duration_ms or 0
    if duration > 1000 then
        Debug.warn("Slow tool: " .. context.component_id.name .. 
                   " took " .. duration .. "ms", "performance")
    end
    return "continue"
end)
```

### Memory Profiling

```lua
-- Track memory allocations
local function profile_memory(name, func)
    collectgarbage("collect")
    local before = collectgarbage("count")
    
    local result = func()
    
    collectgarbage("collect")
    local after = collectgarbage("count")
    
    Debug.info(name .. " used " .. (after - before) .. " KB", "memory")
    return result
end

-- Use it
local data = profile_memory("data_loading", function()
    return load_large_dataset()
end)
```

### Stack Traces

```lua
-- Capture stack traces on error
local function safe_call(func, ...)
    local args = {...}
    local success, result = xpcall(
        function() return func(table.unpack(args)) end,
        function(err)
            Debug.error("Error: " .. err, "runtime")
            Debug.error("Stack: " .. debug.traceback(), "runtime")
            return err
        end
    )
    return success, result
end
```

---

## Getting Help

### Diagnostic Information

When reporting issues, include:

```bash
# Version info
./llmspell --version

# System info
uname -a  # Unix/Linux
sw_vers  # macOS

# Config validation
./llmspell validate -c your-config.toml

# Debug output
RUST_LOG=debug ./llmspell run script.lua 2> debug.log
# Attach debug.log to issue
```

### Minimal Reproduction

Create minimal script that shows the issue:

```lua
-- minimal_repro.lua
print("LLMSpell version test")

local agent = Agent.builder()
    :model("openai/gpt-3.5-turbo")
    :build()

-- This line causes the error:
local result = agent:execute({
    prompt = "test"  -- Minimal prompt
})

print("Success!")
```

### Community Resources

- **GitHub Issues**: [Report bugs](https://github.com/yourusername/rs-llmspell/issues)
- **Documentation**: Check [API docs](api/README.md) for detailed reference
- **Examples**: Review [working examples](../../examples/EXAMPLE-INDEX.md)

### Error Reporting Template

```markdown
## Environment
- LLMSpell version: X.Y.Z
- OS: Linux/macOS/Windows
- Rust version: X.Y.Z

## Description
Brief description of the issue

## Steps to Reproduce
1. Create file `test.lua` with: ...
2. Run: `./llmspell run test.lua`
3. See error: ...

## Expected Behavior
What should happen

## Actual Behavior
What actually happens

## Debug Output
```
RUST_LOG=debug output here
```

## Additional Context
Any other relevant information
```

---

## See Also

- [Configuration Guide](configuration.md) - Detailed configuration options
- [Core Concepts](concepts.md) - Understanding LLMSpell architecture
- [Getting Started](getting-started.md) - Basic setup and usage
- [API Documentation](api/README.md) - Complete API reference