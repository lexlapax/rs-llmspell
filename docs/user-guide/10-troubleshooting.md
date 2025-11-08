# Troubleshooting Guide

**Comprehensive guide to debugging and resolving issues**

🔗 **Navigation**: [← User Guide](README.md) | [Configuration](03-configuration.md) | [Deployment](08-deployment.md)

---

## Quick Navigation

- [Common Issues](#common-issues) - Quick fixes for frequent problems
- [Kernel & Service Issues](#kernel--service-issues) - Service deployment and daemon mode
- [Debugging Techniques](#debugging-techniques) - Tools and strategies for debugging
- [Performance Issues](#performance-issues) - Optimization and profiling
- [API and Provider Issues](#api-and-provider-issues) - LLM provider troubleshooting
- [Memory and Resource Issues](#memory-and-resource-issues) - Resource limits and memory
- [Script Errors](#script-errors) - Lua/JS script debugging
- [Tool-Specific Issues](#tool-specific-issues) - Tool invocation problems
- [IDE & DAP Integration](#ide--dap-integration) - VS Code, Jupyter, debugging
- [Advanced Debugging](#advanced-debugging) - Profiling and diagnostics
- [Getting Help](#getting-help) - How to report issues

> **💡 Tip**: Use Ctrl+F to search for specific error messages in this guide.

---

## Table of Contents

1. [Common Issues](#common-issues)
2. [Kernel & Service Issues](#kernel--service-issues)
3. [Debugging Techniques](#debugging-techniques)
4. [Performance Issues](#performance-issues)
5. [API and Provider Issues](#api-and-provider-issues)
6. [Memory and Resource Issues](#memory-and-resource-issues)
7. [Script Errors](#script-errors)
8. [Configuration Problems](#configuration-problems)
9. [Tool-Specific Issues](#tool-specific-issues)
10. [IDE & DAP Integration](#ide--dap-integration)
11. [Advanced Debugging](#advanced-debugging)
12. [Getting Help](#getting-help)

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

## Kernel & Service Issues

### Kernel Won't Start

**Symptoms:**
```
Error: Failed to start kernel service
Error: Address already in use
```

**Solutions:**
```bash
# Check if port is in use
lsof -i :9555
netstat -an | grep 9555

# Kill existing kernel
./target/release/llmspell kernel stop --all

# Start on different port
./target/release/llmspell kernel start --port 9600

# Check PID file conflicts
rm /var/run/llmspell/*.pid
```

### Daemon Mode Issues

**Symptoms:**
- Daemon doesn't start
- No log output
- Can't find PID file

**Solutions:**
```bash
# Check log directory permissions
ls -la /var/log/llmspell/

# Create necessary directories
sudo mkdir -p /var/log/llmspell
sudo mkdir -p /var/run/llmspell
sudo chown $USER:$USER /var/log/llmspell /var/run/llmspell

# Start with explicit paths
./target/release/llmspell kernel start --daemon \
  --log-file /tmp/llmspell.log \
  --pid-file /tmp/llmspell.pid

# Debug daemon startup
RUST_LOG=debug ./target/release/llmspell kernel start --daemon
```

### Service Deployment Failures

**systemd Issues (Linux):**
```bash
# Check service status
systemctl --user status llmspell-kernel

# View detailed logs
journalctl --user -u llmspell-kernel -n 50

# Reload service files after changes
systemctl --user daemon-reload

# Fix permissions
chmod 644 ~/.config/systemd/user/llmspell-kernel.service
```

**launchd Issues (macOS):**
```bash
# Check if loaded
launchctl list | grep llmspell

# View logs
tail -f /usr/local/var/log/llmspell/kernel.log

# Unload and reload
launchctl unload ~/Library/LaunchAgents/com.llmspell.kernel.plist
launchctl load ~/Library/LaunchAgents/com.llmspell.kernel.plist
```

### Signal Handling Problems

**Kernel not responding to signals:**
```bash
# Check if kernel is running
ps aux | grep llmspell

# Send signals properly
kill -TERM $(cat /var/run/llmspell/kernel.pid)  # Graceful shutdown
kill -HUP $(cat /var/run/llmspell/kernel.pid)   # Reload config
kill -USR1 $(cat /var/run/llmspell/kernel.pid)  # Dump stats
kill -USR2 $(cat /var/run/llmspell/kernel.pid)  # Toggle debug

# Force stop if needed
kill -9 $(cat /var/run/llmspell/kernel.pid)
```

### Connection File Issues

**Can't connect to kernel:**
```bash
# Check connection file
cat /var/lib/llmspell/kernel-*.json

# Connect with explicit file
./target/release/llmspell kernel connect --connection-file /path/to/kernel.json

# Verify ZeroMQ ports are open
netstat -an | grep -E '955[5-9]'
```

### Kernel Discovery (Phase 9-10)

**Find running kernels:**
```bash
# List all kernels
./target/release/llmspell kernel list

# Show kernel status
./target/release/llmspell kernel status

# Find kernel by ID
./target/release/llmspell kernel info <kernel-id>

# Discover kernels via connection files
ls /var/lib/llmspell/kernel-*.json
ls ~/.llmspell/kernels/*.json
```

**Auto-discovery not working:**
```bash
# Check runtime directory
ls -la /var/run/llmspell/

# Check PID files
for pid in /var/run/llmspell/*.pid; do
    echo "Kernel PID: $(cat $pid)"
    ps -p $(cat $pid) || echo "Not running"
done
```

### Service Installation Issues (Phase 10)

**kernel install-service command fails:**
```bash
# Check if service already exists
systemctl --user status llmspell-kernel  # User service
sudo systemctl status llmspell-kernel    # System service

# Manual installation if auto-install fails
./target/release/llmspell kernel install-service --dry-run > llmspell.service

# For systemd (Linux)
cp llmspell.service ~/.config/systemd/user/
systemctl --user daemon-reload
systemctl --user enable llmspell-kernel

# For launchd (macOS)
cp com.llmspell.kernel.plist ~/Library/LaunchAgents/
launchctl load ~/Library/LaunchAgents/com.llmspell.kernel.plist
```

**Service template generation:**
```bash
# Generate systemd service
./target/release/llmspell kernel install-service \
  --service-type systemd \
  --template-only > llmspell-kernel.service

# Generate launchd plist
./target/release/llmspell kernel install-service \
  --service-type launchd \
  --template-only > com.llmspell.kernel.plist
```

### Fleet Management (Multiple Kernels)

**Running multiple kernel instances:**
```bash
# Start multiple kernels on different ports
./target/release/llmspell kernel start --daemon --port 9555 --id kernel1
./target/release/llmspell kernel start --daemon --port 9565 --id kernel2
./target/release/llmspell kernel start --daemon --port 9575 --id kernel3

# List all running kernels
./target/release/llmspell kernel list

# Stop specific kernel
./target/release/llmspell kernel stop --id kernel2

# Stop all kernels
./target/release/llmspell kernel stop --all
```

**Fleet configuration:**
```toml
[fleet]
max_kernels = 10
port_range_start = 9555
port_range_end = 9655
auto_spawn = true
load_balance = true

[fleet.kernels.primary]
id = "primary"
port = 9555
priority = 1

[fleet.kernels.secondary]
id = "secondary"
port = 9565
priority = 2
```

### REPL Service Issues (Phase 9)

**REPL not responding:**
```bash
# Check if REPL service is enabled
./target/release/llmspell kernel start --repl

# Connect to REPL
./target/release/llmspell kernel connect --address tcp://localhost:9555

# Test REPL directly
echo 'print("test")' | nc localhost 9555
```

**REPL history issues:**
```bash
# Check history file
ls -la ~/.llmspell/repl_history

# Clear corrupted history
rm ~/.llmspell/repl_history

# Set custom history location
export LLMSPELL_REPL_HISTORY="/tmp/repl_history"
```

---

## Debugging Techniques

### The --trace Flag (Phase 9)

**IMPORTANT:** Phase 9 replaced `--debug` and `--verbose` with the unified `--trace` flag:

```bash
# Set trace level (replaces old --debug/--verbose)
./target/release/llmspell --trace debug run script.lua
./target/release/llmspell --trace info exec "print('test')"
./target/release/llmspell --trace trace kernel start  # Maximum verbosity

# Available levels:
# - off: No logging
# - error: Errors only
# - warn: Warnings and errors (default)
# - info: Informational messages (replaces --verbose)
# - debug: Debug messages (replaces --debug)
# - trace: Everything including trace spans
```

**Priority Order:**
1. `RUST_LOG` environment variable (highest priority)
2. `--trace` flag
3. Default level (warn)

```bash
# RUST_LOG overrides --trace
RUST_LOG=debug ./target/release/llmspell --trace error run script.lua
# Will use debug level, not error

# Without RUST_LOG, --trace is used
./target/release/llmspell --trace debug run script.lua
# Will use debug level
```

### Enable Debug Logging

**Via --trace Flag (Recommended):**
```bash
# Use the new --trace flag
./target/release/llmspell --trace debug run script.lua
./target/release/llmspell --trace info kernel start
./target/release/llmspell --trace trace exec "code"  # Maximum verbosity
```

**Via Environment (Override):**
```bash
RUST_LOG=debug ./target/release/llmspell run script.lua
RUST_LOG=llmspell_kernel=trace ./target/release/llmspell kernel start
RUST_LOG=trace ./target/release/llmspell run script.lua  # Very verbose
```

**Via Configuration:**
```toml
[global]
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

### Kernel Debugging

**Monitor kernel operations:**
```bash
# Start kernel with tracing
RUST_LOG=llmspell_kernel=trace ./target/release/llmspell kernel start

# View kernel metrics
curl http://localhost:9555/metrics

# Health check
curl http://localhost:9555/health
```

### DAP (Debug Adapter Protocol) Debugging

**Connect debugger to running scripts:**
```lua
-- Enable DAP in script
Debug.enableDAP()

-- Set breakpoints programmatically
Debug.breakpoint("myfunction", 10)

-- Step through code
Debug.step()
Debug.stepOver()
Debug.continue()
```

**VS Code Integration:**
```json
// .vscode/launch.json
{
  "version": "0.2.0",
  "configurations": [
    {
      "type": "llmspell",
      "request": "attach",
      "name": "Attach to LLMSpell",
      "port": 9556,
      "host": "localhost"
    }
  ]
}
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

### Kernel Running Slowly

**Diagnosis:**
```bash
# Check kernel performance
curl http://localhost:9555/metrics | jq '.performance'

# Monitor resource usage
top -p $(cat /var/run/llmspell/kernel.pid)
```

**Common Fixes:**

1. **Increase kernel resources:**
```bash
./target/release/llmspell kernel start \
  --max-clients 100 \
  --idle-timeout 0
```

2. **Optimize HNSW parameters:**
```toml
[rag.vector_storage.hnsw]
ef_search = 50  # Reduce for faster search
m = 16          # Reduce for less memory
```

3. **Enable caching:**
```toml
[rag.cache]
search_cache_enabled = true
embedding_cache_enabled = true
```

### Script Running Slowly

**Profile your script:**
```lua
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

**Kernel memory limits:**
```toml
[kernel.limits]
max_memory_mb = 2048
max_message_size_mb = 10
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

### Provider HTTP Client Errors

**Note:** Phase 9's global IO runtime fixes the "dispatch task is gone" error that occurred in long-running operations.

```bash
# If you still see HTTP client errors, check:
RUST_LOG=llmspell_kernel::runtime=debug ./target/release/llmspell run script.lua
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

[kernel.limits]
max_memory_mb = 2048
```

### "Too many open files"

**Solutions:**
```bash
# Check current limit
ulimit -n

# Increase limit (Unix/Linux)
ulimit -n 4096

# For systemd service
# Add to service file:
LimitNOFILE=65536
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
./target/release/llmspell validate -c config.toml

# Check which config is loaded
RUST_LOG=debug ./target/release/llmspell -c config.toml run script.lua 2>&1 | grep -i config

# Use absolute path
./target/release/llmspell -c /absolute/path/to/config.toml run script.lua
```

### Kernel Configuration Issues

**Kernel-specific config:**
```toml
[kernel]
port = 9555
connection_file = "/var/lib/llmspell/kernel.json"
idle_timeout = 0

[daemon]
daemonize = true
pid_file = "/var/run/llmspell/kernel.pid"

[logging]
log_file = "/var/log/llmspell/kernel.log"
log_level = "info"
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

## IDE & DAP Integration

### VS Code Connection Issues

**Can't connect to DAP:**
```bash
# Check if DAP is enabled
grep -i dap /var/log/llmspell/kernel.log

# Start kernel with DAP enabled
./target/release/llmspell kernel start --dap

# Verify DAP port
netstat -an | grep 9556
```

### Breakpoints Not Working

**Common issues:**
- Source maps not generated
- Incorrect file paths
- DAP not initialized

**Solutions:**
```lua
-- Initialize DAP properly
Debug.enableDAP({
    port = 9556,
    wait_for_debugger = true
})

-- Use absolute paths for breakpoints
Debug.breakpoint("/full/path/to/script.lua", 10)
```

### Jupyter Lab Integration

**Connection issues:**
```bash
# Check Jupyter connection file
cat /var/lib/llmspell/kernel-*.json

# Verify all 5 channels are listening
netstat -an | grep -E '955[5-9]'

# Test with Jupyter console
jupyter console --existing kernel-*.json
```

**Install Jupyter kernel spec:**
```bash
# Install kernel spec for Jupyter
./target/release/llmspell kernel install-jupyter

# Manual installation
jupyter kernelspec install --user llmspell-kernel/

# List installed kernels
jupyter kernelspec list
```

### Multi-Protocol Issues (Phase 9-10)

**Protocol conflicts:**
```bash
# Check which protocols are active
curl http://localhost:9555/protocols

# Start with specific protocols only
./target/release/llmspell kernel start \
  --jupyter --no-dap --no-lsp

# Debug protocol parsing
RUST_LOG=llmspell_kernel::protocols=trace ./target/release/llmspell kernel start
```

**Transport layer issues:**
```bash
# Test ZeroMQ transport
RUST_LOG=llmspell_kernel::transport::zeromq=trace ./target/release/llmspell kernel start

# Test WebSocket transport (if configured)
RUST_LOG=llmspell_kernel::transport::websocket=trace ./target/release/llmspell kernel start

# Test in-process transport
./target/release/llmspell run script.lua  # Uses in-process by default
```

### Message Routing Issues (Multi-Client)

**Clients not receiving messages:**
```bash
# Check message router status
curl http://localhost:9555/router/status

# Monitor message flow
RUST_LOG=llmspell_kernel::router=trace ./target/release/llmspell kernel start

# Test IOPub broadcast
curl -X POST http://localhost:9555/test/broadcast \
  -d '{"msg": "test broadcast"}'
```

**Client isolation problems:**
```bash
# Verify client registration
curl http://localhost:9555/clients

# Check client-specific state
curl http://localhost:9555/client/<client-id>/state
```

---

## Advanced Debugging

### Enable Trace Logging

```bash
# Maximum verbosity
RUST_LOG=trace ./target/release/llmspell run script.lua 2> debug.log

# Filter by module
RUST_LOG=llmspell_agents=debug,llmspell_tools=trace ./target/release/llmspell run script.lua

# Kernel-specific tracing
RUST_LOG=llmspell_kernel::execution=trace,llmspell_kernel::transport=debug ./target/release/llmspell kernel start
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

### Kernel Event Correlation (Phase 9)

**Use correlation IDs for debugging:**
```bash
# Enable event correlation tracing
RUST_LOG=llmspell_kernel::events::correlation=trace ./target/release/llmspell kernel start

# View all correlated events
curl http://localhost:9555/events | jq '.[].correlation_id'

# Filter by correlation ID
curl "http://localhost:9555/events?correlation_id=abc-123" | jq '.'

# Track causation chain
curl "http://localhost:9555/events?causation_id=root-456" | jq '.'
```

**Trace request flow:**
```bash
# Enable request tracing
./target/release/llmspell kernel start --trace trace

# Get trace for specific execution
curl "http://localhost:9555/trace/<msg-id>" | jq '.'

# Export traces
curl "http://localhost:9555/traces/export" > traces.json
```

### Global IO Runtime Verification (Phase 9)

**Verify global runtime is active:**
```bash
# Check runtime status
curl http://localhost:9555/runtime/status | jq '.'

# Should show:
# {
#   "global_runtime": true,
#   "worker_threads": 4,
#   "active_tasks": 12
# }

# Monitor runtime issues
RUST_LOG=llmspell_kernel::runtime::io_runtime=debug ./target/release/llmspell kernel start
```

**"dispatch task is gone" error (should be fixed):**
```bash
# If you still see this error after Phase 9:
# 1. Verify you're using the latest build
./target/release/llmspell --version

# 2. Check global runtime initialization
RUST_LOG=llmspell_kernel::runtime=trace ./target/release/llmspell kernel start 2>&1 | grep "global_io_runtime"

# 3. Force rebuild with global runtime
cargo clean
cargo build --release --features global-io-runtime
```

---

## Getting Help

### Diagnostic Information

When reporting issues, include:

```bash
# Version info
./target/release/llmspell --version

# Kernel version
./target/release/llmspell kernel version

# System info
uname -a  # Unix/Linux
sw_vers  # macOS

# Config validation
./target/release/llmspell validate -c your-config.toml

# Debug output (use --trace flag)
./target/release/llmspell --trace debug run script.lua 2> debug.log
# Or with RUST_LOG
RUST_LOG=debug ./target/release/llmspell run script.lua 2> debug.log
# Attach debug.log to issue

# Kernel diagnostics
./target/release/llmspell kernel diagnose

# Full kernel status
./target/release/llmspell kernel status --verbose

# Export kernel state
./target/release/llmspell kernel export-state > kernel-state.json
```

### Comprehensive Diagnostics Script

```bash
#!/bin/bash
# Save as diagnose.sh

echo "=== LLMSpell Diagnostics ==="
echo "Date: $(date)"
echo

echo "=== Version Information ==="
./target/release/llmspell --version
./target/release/llmspell kernel version
echo

echo "=== System Information ==="
uname -a
echo "CPU Cores: $(nproc 2>/dev/null || sysctl -n hw.ncpu)"
echo "Memory: $(free -h 2>/dev/null || vm_stat | head -5)"
echo

echo "=== Kernel Status ==="
./target/release/llmspell kernel status
echo

echo "=== Running Kernels ==="
./target/release/llmspell kernel list
echo

echo "=== Port Status ==="
netstat -an | grep -E '955[0-9]' || echo "No kernel ports found"
echo

echo "=== Configuration ==="
if [ -f ~/.config/llmspell/config.toml ]; then
    echo "User config found"
    grep -E '^\[|^[^#]*=' ~/.config/llmspell/config.toml | head -20
else
    echo "No user config"
fi
echo

echo "=== Recent Logs ==="
if [ -f /var/log/llmspell/kernel.log ]; then
    tail -50 /var/log/llmspell/kernel.log
else
    echo "No kernel log found"
fi
echo

echo "=== Service Status ==="
systemctl --user status llmspell-kernel 2>/dev/null || \
    launchctl list | grep llmspell 2>/dev/null || \
    echo "No service found"
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
- LLMSpell version: 0.9.0
- OS: Linux/macOS/Windows
- Rust version: X.Y.Z
- Kernel mode: daemon/embedded
- Service type: systemd/launchd/manual

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
- Using kernel mode? Yes/No
- DAP enabled? Yes/No
- Service deployment? Yes/No
```

---

## Phase 9 & 10 Specific Issues

### Global IO Runtime

The Phase 9 kernel architecture includes a global IO runtime that fixes the "dispatch task is gone" error. If you're still seeing runtime errors:

```bash
# Check runtime initialization
RUST_LOG=llmspell_kernel::runtime=debug ./target/release/llmspell kernel start

# Verify global runtime is active
curl http://localhost:9555/metrics | jq '.runtime'
```

### Protocol/Transport Issues

```bash
# Check active transports
curl http://localhost:9555/transports

# Debug ZeroMQ issues
RUST_LOG=llmspell_kernel::transport::zeromq=trace ./target/release/llmspell kernel start

# Test Jupyter protocol
jupyter console --existing /var/lib/llmspell/kernel.json
```

### State Consolidation

Phase 9 consolidated state and sessions into the kernel. If you see state-related errors:

```lua
-- Check state scope availability
local scopes = State.list_scopes()
print("Available scopes: " .. table.concat(scopes, ", "))

-- Use proper scope
State.set("global", "key", "value")  -- Not just State.set("key", "value")
```

---

---

## See Also

- [Configuration Guide](03-configuration.md) - Detailed configuration options
- [Core Concepts](02-core-concepts.md) - Understanding llmspell architecture
- [Deployment Guide](08-deployment.md) - Production deployment strategies
- [Getting Started](01-getting-started.md) - Basic setup and usage
- [Lua API Reference](appendix/lua-api-reference.md) - Complete Lua API documentation

---

**Version**: 0.13.0 | **Phase**: 13b.18.3 | **Last Updated**: 2025-11-08