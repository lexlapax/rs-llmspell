# Phase 10 Troubleshooting Guide

**Version**: 0.9.0 (Phase 10)
**Last Updated**: 2025-09-30
**Components**: Kernel Service, Tool Commands, Jupyter Protocol, DAP Debugging

> **🔧 Phase 10 Features**: This guide covers troubleshooting for Phase 10 components including kernel service mode, tool commands, Jupyter Wire Protocol v5.3, and Debug Adapter Protocol (DAP) integration.

**📚 See Also**:
- [Main Troubleshooting Guide](troubleshooting.md) - General llmspell issues
- [Configuration Guide](configuration.md) - Kernel and service configuration
- [Protocol Compliance Report](../technical/protocol-compliance-report.md) - Protocol specifications

---

## Table of Contents

1. [Kernel Service Issues](#1-kernel-service-issues)
2. [Tool Command Issues](#2-tool-command-issues)
3. [Jupyter Protocol Issues](#3-jupyter-protocol-issues)
4. [DAP Debugging Issues](#4-dap-debugging-issues)
5. [Performance Issues](#5-performance-issues)
6. [Configuration Issues](#6-configuration-issues)
7. [Diagnostic Procedures](#7-diagnostic-procedures)
8. [FAQ](#8-faq)

---

## 1. Kernel Service Issues

### 1.1 Kernel Won't Start

#### Symptom: "Failed to start kernel"

**Error Messages**:
```
Error: Failed to start kernel service
Error: Could not start embedded kernel
Error: Failed to initialize kernel
```

**Diagnostic Steps**:
```bash
# Check if kernel binary exists
ls -la ./target/debug/llmspell

# Try with verbose logging
RUST_LOG=debug ./target/debug/llmspell kernel start --trace debug

# Check system resources
ulimit -n  # Open file descriptors
free -h    # Available memory
```

**Common Causes**:

**1. Binary Not Built**
```bash
# Solution: Build the project
cargo build

# Or build in release mode for production
cargo build --release
```

**2. Lua Bridge Initialization Failure**
```bash
# Error: Failed to create Lua runtime
# Solution: Check Lua dependencies
cargo build --features lua

# Verify feature is enabled
cargo build -p llmspell-kernel --features lua
```

**3. ComponentRegistry Registration Failure**
```bash
# Error: Failed to register tools
# Solution: Check tool dependencies
RUST_LOG=llmspell_tools=trace cargo build

# Verify tool count
./target/debug/llmspell tool list
# Expected: 30 tools
```

---

### 1.2 Port Already in Use

#### Symptom: "Address already in use"

**Error Messages**:
```
Error: Failed to bind to port 59000: Address already in use
Error: Port 59000 is already bound
```

**Diagnostic Steps**:
```bash
# Find process using the port
lsof -i :59000
netstat -an | grep 59000

# Check for existing kernels
./target/debug/llmspell kernel status

# List all llmspell processes
ps aux | grep llmspell
```

**Solutions**:

**Option 1: Stop Existing Kernel**
```bash
# Stop by kernel ID
./target/debug/llmspell kernel stop abc123

# Stop all kernels
pkill -f "llmspell kernel"

# Verify stopped
./target/debug/llmspell kernel status
```

**Option 2: Use Different Port**
```bash
# Start on alternative port
./target/debug/llmspell kernel start --port 59001

# Or let system assign port (port 0)
./target/debug/llmspell kernel start --port 0
```

**Option 3: Force Kill Stuck Process**
```bash
# Find PID
lsof -i :59000 | grep LISTEN | awk '{print $2}'

# Kill process
kill -9 <PID>

# Clean up stale PID files
rm -f ~/.llmspell/kernels/*/kernel.pid
```

---

### 1.3 Daemon Mode Issues

#### Symptom: Daemon starts but no logs appear

**Error Messages**:
```
# No output after running:
./target/debug/llmspell kernel start --daemon
```

**Diagnostic Steps**:
```bash
# Check if daemon is actually running
./target/debug/llmspell kernel status

# Check process list
ps aux | grep llmspell | grep -v grep

# Find log files
find ~/.llmspell -name "*.log" -type f

# Check for errors in logs
tail -f ~/.llmspell/kernels/*/kernel.log
```

**Common Causes**:

**1. Log File Permission Issues**
```bash
# Error: Permission denied writing to log file
# Solution: Create log directory with correct permissions
mkdir -p ~/.llmspell/kernels
chmod 755 ~/.llmspell/kernels

# Or specify writable log location
./target/debug/llmspell kernel start --daemon \
  --log-file /tmp/llmspell.log
```

**2. Daemon Double-Fork Failure**
```bash
# Error: Failed to daemonize
# Cause: Insufficient permissions or resource limits

# Check ulimits
ulimit -a

# Run in foreground mode for debugging
./target/debug/llmspell kernel start --port 59000
# (without --daemon flag)
```

**3. Silent Crash After Daemonization**
```bash
# Daemon starts then immediately exits
# Check dmesg for segfaults
dmesg | tail -20

# Check core dumps
coredumpctl list llmspell

# Run with backtrace
RUST_BACKTRACE=1 ./target/debug/llmspell kernel start
```

---

### 1.4 PID File Issues

#### Symptom: "PID file already exists"

**Error Messages**:
```
Error: PID file already exists: ~/.llmspell/kernels/abc123/kernel.pid
Error: Kernel is already running with PID 12345
```

**Diagnostic Steps**:
```bash
# Check if PID file exists
ls -la ~/.llmspell/kernels/*/kernel.pid

# Read PID from file
cat ~/.llmspell/kernels/abc123/kernel.pid

# Check if process is actually running
ps -p <PID>

# Check if it's a stale PID file (process doesn't exist)
kill -0 <PID> 2>/dev/null && echo "Running" || echo "Stale"
```

**Solutions**:

**Stale PID File (Process Not Running)**
```bash
# Safe cleanup: kernel status command auto-cleans stale PIDs
./target/debug/llmspell kernel status

# Manual cleanup if needed
rm ~/.llmspell/kernels/abc123/kernel.pid

# Start fresh
./target/debug/llmspell kernel start
```

**Multiple Kernel Instances**
```bash
# List all running kernels
./target/debug/llmspell kernel status

# Stop specific kernel
./target/debug/llmspell kernel stop abc123

# Or stop all and start fresh
pkill -f "llmspell kernel"
rm ~/.llmspell/kernels/*/kernel.pid
./target/debug/llmspell kernel start
```

---

### 1.5 Signal Handling Issues

#### Symptom: Kernel doesn't respond to signals

**Error Messages**:
```
# Kernel doesn't stop on Ctrl+C
# SIGTERM doesn't trigger graceful shutdown
```

**Diagnostic Steps**:
```bash
# Test signal handling
./target/debug/llmspell kernel start --port 59000

# In another terminal:
# Send SIGTERM (should trigger graceful shutdown)
kill -TERM <PID>

# Check if signal was handled
# Should see in logs: "Received SIGTERM, initiating shutdown"
tail -f ~/.llmspell/kernels/*/kernel.log
```

**Common Causes**:

**1. Signal Bridge Not Initialized**
```bash
# Check if signal handlers are registered
# Look for this in logs at startup:
# "Signal bridge initialized"

# If missing, check kernel configuration
RUST_LOG=llmspell_kernel::daemon=debug ./target/debug/llmspell kernel start
```

**2. Kernel Stuck in Blocking Operation**
```bash
# Kernel may be blocked on I/O or deadlocked
# Try force kill as last resort
kill -9 <PID>

# Check for deadlocks with trace logging
RUST_LOG=trace ./target/debug/llmspell kernel start
```

**3. Zombie Process**
```bash
# Process shows as <defunct>
ps aux | grep llmspell | grep defunct

# Parent process needs to reap child
# Usually resolves on next kernel start
./target/debug/llmspell kernel start
```

---

## 2. Tool Command Issues

### 2.1 Tool List Command Fails

#### Symptom: "Failed to list tools"

**Error Messages**:
```
Error: Failed to send tool request
Error: Timeout waiting for tool_reply
Error: No tools array in response
```

**Diagnostic Steps**:
```bash
# Test tool list with verbose output
./target/debug/llmspell tool list --trace debug

# Check kernel is running (for remote kernel)
./target/debug/llmspell kernel status

# Test with embedded kernel (faster)
./target/debug/llmspell tool list
# (uses embedded kernel by default)
```

**Common Causes**:

**1. InProcess Transport Not Connected**
```bash
# Error: Failed to send tool request
# Cause: Message channel not established

# Check if using embedded mode (should work)
./target/debug/llmspell tool list

# If using remote kernel, verify connection
./target/debug/llmspell kernel connect localhost:59000
./target/debug/llmspell tool list
```

**2. ComponentRegistry Empty**
```bash
# Error: No tools registered
# Cause: Tool registration failed during startup

# Check tool count in logs
RUST_LOG=llmspell_tools=debug ./target/debug/llmspell tool list

# Expected output: ~30 tools
# If 0 tools, rebuild with proper features
cargo build --features lua
```

**3. Response Timeout**
```bash
# Error: Timeout waiting for tool_reply
# Cause: Kernel overloaded or deadlocked

# Check kernel health
./target/debug/llmspell kernel status abc123

# Restart kernel if needed
./target/debug/llmspell kernel stop abc123
./target/debug/llmspell kernel start
```

---

### 2.2 Tool Invoke Command Fails

#### Symptom: Tool execution returns error

**Error Messages**:
```
Error: Tool 'calculator' not found
Error: Failed to execute tool
Error: Invalid parameters for tool
```

**Diagnostic Steps**:
```bash
# Verify tool exists
./target/debug/llmspell tool list | grep calculator

# Check tool info for parameter requirements
./target/debug/llmspell tool info calculator

# Test with simple parameters
./target/debug/llmspell tool invoke calculator --params '{"input": "2+2"}'

# Use verbose logging
./target/debug/llmspell tool invoke calculator \
  --params '{"input": "2+2"}' \
  --trace debug
```

**Common Causes**:

**1. Tool Not Found**
```bash
# Error: Tool 'my_tool' not found
# Solution: Check exact tool name
./target/debug/llmspell tool list

# Use search to find similar tools
./target/debug/llmspell tool search calc
```

**2. Invalid Parameters**
```bash
# Error: Missing required parameter 'input'
# Solution: Check tool schema
./target/debug/llmspell tool info calculator

# Provide all required parameters
./target/debug/llmspell tool invoke calculator \
  --params '{"input": "10 + 5"}'
```

**3. Tool Execution Error**
```bash
# Error: Division by zero
# Error: Invalid expression syntax

# Check tool-specific error messages
./target/debug/llmspell tool invoke calculator \
  --params '{"input": "10 / 0"}' \
  --trace debug

# Validate input before invoking
./target/debug/llmspell tool test calculator
```

---

### 2.3 Tool Search Returns No Results

#### Symptom: Search query finds no tools

**Error Messages**:
```
# No results found for query 'xyz'
```

**Diagnostic Steps**:
```bash
# List all tools to see what's available
./target/debug/llmspell tool list

# Try broader search terms
./target/debug/llmspell tool search file
./target/debug/llmspell tool search calc
./target/debug/llmspell tool search text

# Check tool categories
./target/debug/llmspell tool list --category utility
```

**Tips**:
- Search is case-insensitive partial matching
- Searches tool names, descriptions, and categories
- Use shorter, more general terms (e.g., "file" not "file_operations")
- Common tool categories: filesystem, utility, web, data, analysis

---

## 3. Jupyter Protocol Issues

### 3.1 HMAC Authentication Failures

#### Symptom: "Invalid signature"

**Error Messages**:
```
Error: HMAC signature verification failed
Error: Invalid message signature
Warning: Message rejected due to authentication failure
```

**Diagnostic Steps**:
```bash
# Check connection file has auth key
cat ~/.llmspell/kernels/abc123/kernel.json | grep key

# Verify client is using correct key
# (Check your Jupyter client configuration)

# Test with raw ZeroMQ (bypasses Jupyter client)
cd tests/python && python test_raw_zmq.py
```

**Common Causes**:

**1. Wrong Auth Key**
```bash
# Client using incorrect key from old connection file
# Solution: Use fresh connection file
./target/debug/llmspell kernel stop abc123
./target/debug/llmspell kernel start
# New connection file generated at ~/.llmspell/kernels/<new-id>/kernel.json
```

**2. Signature Not Included**
```bash
# Client not sending signature in multipart message
# Solution: Ensure client sends 7-part message:
# [<IDS|MSG>, signature, header, parent_header, metadata, content, extra]

# Test with compliant client
pip install jupyter_client
python -c "
import jupyter_client
client = jupyter_client.BlockingKernelClient()
client.load_connection_file('/path/to/kernel.json')
client.start_channels()
print(client.kernel_info())
"
```

**3. Corrupted Connection File**
```bash
# Connection file missing or malformed
# Solution: Regenerate connection file
./target/debug/llmspell kernel stop abc123
./target/debug/llmspell kernel start --connection-file /tmp/kernel.json

# Verify file is valid JSON
cat /tmp/kernel.json | jq .
```

---

### 3.2 Channel Connection Issues

#### Symptom: "Failed to connect to channel"

**Error Messages**:
```
Error: Failed to connect to shell channel
Error: ZeroMQ socket connection failed
Timeout waiting for reply on shell channel
```

**Diagnostic Steps**:
```bash
# Check kernel is running
./target/debug/llmspell kernel status

# Verify ports are open
lsof -i :59000
netstat -an | grep 59000

# Test with netcat
nc -zv 127.0.0.1 59000

# Check firewall rules
sudo iptables -L | grep 59000  # Linux
sudo pfctl -s rules | grep 59000  # macOS
```

**Common Causes**:

**1. Wrong Port Numbers**
```bash
# Client connecting to wrong port
# Solution: Use exact ports from connection file
cat ~/.llmspell/kernels/abc123/kernel.json

{
  "shell_port": 59000,
  "control_port": 59001,
  "iopub_port": 59002,
  "stdin_port": 59003,
  "hb_port": 59004,
  ...
}

# Ensure client loads connection file correctly
jupyter_client.load_connection_file('/path/to/kernel.json')
```

**2. Firewall Blocking Ports**
```bash
# Solution: Allow ports in firewall
# Linux (iptables)
sudo iptables -A INPUT -p tcp --dport 59000:59004 -j ACCEPT

# macOS (pf)
echo "pass in proto tcp from any to any port 59000:59004" | sudo pfctl -f -

# Or disable firewall temporarily for testing
sudo ufw disable  # Linux
sudo pfctl -d     # macOS
```

**3. IPv4 vs IPv6 Mismatch**
```bash
# Kernel listening on IPv4, client connecting on IPv6 (or vice versa)
# Solution: Force IPv4
./target/debug/llmspell kernel start --host 127.0.0.1

# Or force IPv6
./target/debug/llmspell kernel start --host ::1
```

---

### 3.3 Message Format Errors

#### Symptom: "Invalid message format"

**Error Messages**:
```
Error: Failed to parse message
Error: Expected 7-part message, got 5
Error: Invalid JSON in message part
```

**Diagnostic Steps**:
```bash
# Test with compliant client (test_raw_zmq.py)
cd tests/python && python test_raw_zmq.py

# Check message format in logs
RUST_LOG=llmspell_kernel::transport=trace \
  ./target/debug/llmspell kernel start

# Verify multipart message structure
# Should see: [delimiter, signature, header, parent, metadata, content]
```

**Common Causes**:

**1. Wrong Message Structure**
```python
# WRONG: Missing delimiter
message = [signature, header, parent_header, metadata, content]

# CORRECT: 7-part with delimiter
message = [
    b'<IDS|MSG>',     # Delimiter
    signature,         # HMAC-SHA256
    header,           # JSON
    parent_header,    # JSON
    metadata,         # JSON
    content,          # JSON
]
```

**2. JSON Encoding Issues**
```python
# Ensure proper JSON encoding
import json

header = json.dumps({
    "msg_id": "abc123",
    "session": "session_id",
    "username": "test",
    "msg_type": "execute_request",
    "version": "5.3",
    "date": datetime.utcnow().isoformat() + 'Z'
}).encode('utf-8')

# NOT: header = str(header_dict).encode()  # Wrong!
```

**3. Binary vs String Encoding**
```python
# All parts must be bytes, not strings
# WRONG:
header = '{"msg_id": "abc123"}'

# CORRECT:
header = b'{"msg_id": "abc123"}'
# or
header = json.dumps({"msg_id": "abc123"}).encode('utf-8')
```

---

## 4. DAP Debugging Issues

### 4.1 Breakpoints Not Hit

#### Symptom: Execution doesn't stop at breakpoint

**Diagnostic Steps**:
```bash
# Verify DAP is enabled
./target/debug/llmspell kernel start --trace debug
# Should see: "DAP bridge initialized"

# Check if breakpoints were set
# Look for: "Set breakpoint at file:line"

# Verify file path matches exactly
# Breakpoint path must match executed file path
```

**Common Causes**:

**1. Wrong File Path**
```json
// Breakpoint set with absolute path:
{"source": {"path": "/Users/user/script.lua"}, "line": 10}

// But script run with relative path:
llmspell run script.lua

// Solution: Use consistent paths
llmspell run /Users/user/script.lua
```

**2. Line Number Mismatch**
```lua
-- File changed since setting breakpoint
-- Breakpoint set at line 10, but code moved to line 12

-- Solution: Clear and reset breakpoints after file changes
```

**3. Debug Mode Not Enabled**
```json
// Launch request must have noDebug: false
{
  "request": "launch",
  "program": "script.lua",
  "noDebug": false  // Enable debugging
}
```

---

### 4.2 Variable Inspection Fails

#### Symptom: Cannot inspect variables at breakpoint

**Error Messages**:
```
Error: Variable not found
Error: Scope not available
```

**Diagnostic Steps**:
```bash
# Check if execution is actually paused
# Should see "stopped" event in DAP

# Verify stack frame is valid
# Request stackTrace before requesting variables

# Check scope IDs match
# Scopes request returns variablesReference IDs
```

**Solutions**:

**Request Stack Trace First**
```json
// 1. Request stack trace
{
  "command": "stackTrace",
  "arguments": {"threadId": 1}
}

// 2. Get frame ID from response
// response.body.stackFrames[0].id

// 3. Request scopes for that frame
{
  "command": "scopes",
  "arguments": {"frameId": <frameId>}
}

// 4. Get variables for scope
{
  "command": "variables",
  "arguments": {"variablesReference": <scopeId>}
}
```

---

### 4.3 Stepping Commands Don't Work

#### Symptom: "next", "stepIn", "stepOut" have no effect

**Diagnostic Steps**:
```bash
# Verify execution is paused
# Can only step when stopped at breakpoint

# Check if continue was called instead
# Continuing resumes full execution

# Look for "continued" event
# Should see "stopped" event after each step
```

**Common Causes**:

**1. Not Stopped at Breakpoint**
```
# Can only step when execution is paused
# Must hit breakpoint or stopOnEntry first

# Solution: Set breakpoint and wait for "stopped" event
```

**2. Step Command After Continue**
```json
// WRONG order:
{"command": "continue"}
{"command": "next"}  // No effect - already running

// CORRECT: Wait for stopped event
{"command": "setBreakpoints"}
{"command": "continue"}
// ... wait for "stopped" event ...
{"command": "next"}  // Now will work
```

---

## 5. Performance Issues

### 5.1 Slow Tool Invocation

#### Symptom: Tool commands take >1 second

**Diagnostic Steps**:
```bash
# Measure operation time
time ./target/debug/llmspell tool invoke calculator \
  --params '{"input": "2+2"}'

# Expected: <100ms
# If >1s, check for issues

# Run with tracing to identify bottleneck
RUST_LOG=trace ./target/debug/llmspell tool invoke calculator \
  --params '{"input": "2+2"}'
```

**Common Causes**:

**1. Using Remote Kernel (Network Latency)**
```bash
# Remote kernel adds network roundtrip
# Solution: Use embedded kernel for CLI tools
./target/debug/llmspell tool invoke calculator --params '...'
# (automatically uses embedded kernel)

# Or connect to local kernel
./target/debug/llmspell kernel connect localhost:59000
```

**2. Debug Build (Not Optimized)**
```bash
# Debug builds are 10-100x slower
# Solution: Use release build for performance testing
cargo build --release
time ./target/release/llmspell tool invoke calculator \
  --params '{"input": "2+2"}'
```

**3. Complex Tool Operations**
```bash
# Some tools are inherently slow (API calls, file I/O)
# Expected times:
# - calculator: <10ms
# - file_operations: 10-100ms (depends on file size)
# - web_scraper: 100-1000ms (network latency)

# Use --trace to see breakdown
./target/debug/llmspell tool invoke web_scraper \
  --params '{"url": "..."}' \
  --trace info
```

---

### 5.2 High Message Latency

#### Symptom: Jupyter messages take >100ms

**Diagnostic Steps**:
```bash
# Run performance benchmarks
cargo bench -p llmspell-kernel

# Check message handling time
# Expected: <12ms per operation (from stress tests)

# Test with Python client timing
python -c "
import time
import jupyter_client
client = jupyter_client.BlockingKernelClient()
client.load_connection_file('kernel.json')
client.start_channels()

start = time.time()
msg_id = client.execute('print(2+2)')
client.get_shell_msg(msg_id, timeout=5)
elapsed = time.time() - start
print(f'Roundtrip time: {elapsed*1000:.2f}ms')
"
```

**Common Causes**:

**1. Kernel Overloaded**
```bash
# Too many concurrent clients
# Solution: Check client count
./target/debug/llmspell kernel status abc123

# Limit concurrent connections (future feature)
# For now, stop unnecessary clients
```

**2. Debug Logging Overhead**
```bash
# RUST_LOG=trace adds significant overhead
# Solution: Use info or warn level for production
RUST_LOG=info ./target/debug/llmspell kernel start

# Or disable logging entirely
RUST_LOG=off ./target/debug/llmspell kernel start
```

**3. Slow Tool Execution**
```bash
# Tool takes long time, blocks message handling
# Check which tools are running
RUST_LOG=llmspell_tools=info ./target/debug/llmspell kernel start

# Future: async tool execution will prevent blocking
```

---

## 6. Configuration Issues

### 6.1 Connection File Not Found

#### Symptom: "Failed to load connection file"

**Error Messages**:
```
Error: Connection file not found: /path/to/kernel.json
Error: Failed to read connection file
```

**Diagnostic Steps**:
```bash
# List all kernel connection files
find ~/.llmspell -name "kernel.json"

# Check specific kernel directory
ls -la ~/.llmspell/kernels/abc123/

# Verify kernel ID is correct
./target/debug/llmspell kernel status
```

**Solutions**:

**Option 1: Use Kernel Status to Find Connection File**
```bash
# Get connection file path from status
./target/debug/llmspell kernel status abc123
# Output includes: connection_file: ~/.llmspell/kernels/abc123/kernel.json

# Copy to accessible location
cp ~/.llmspell/kernels/abc123/kernel.json /tmp/kernel.json
```

**Option 2: Specify Connection File at Startup**
```bash
# Create kernel with known connection file path
./target/debug/llmspell kernel start \
  --connection-file /tmp/kernel.json \
  --port 59000

# File will be created at specified path
ls -la /tmp/kernel.json
```

**Option 3: Regenerate Connection File**
```bash
# Stop and restart kernel
./target/debug/llmspell kernel stop abc123
./target/debug/llmspell kernel start

# New connection file created
./target/debug/llmspell kernel status
```

---

### 6.2 Port Configuration Issues

#### Symptom: Ports don't match expected values

**Diagnostic Steps**:
```bash
# Check what ports kernel is using
./target/debug/llmspell kernel status abc123

# Check connection file
cat ~/.llmspell/kernels/abc123/kernel.json | jq .

# Verify ports are accessible
for port in $(seq 59000 59004); do
  nc -zv 127.0.0.1 $port
done
```

**Solutions**:

**Specify Exact Ports**
```bash
# Start with specific base port
./target/debug/llmspell kernel start --port 59000

# Ports will be:
# shell: 59000
# control: 59001
# iopub: 59002
# stdin: 59003
# heartbeat: 59004
```

**Let System Assign Ports**
```bash
# Use port 0 for automatic assignment
./target/debug/llmspell kernel start --port 0

# Check assigned ports
./target/debug/llmspell kernel status
```

---

### 6.3 Log File Issues

#### Symptom: No logs or logs not rotating

**Diagnostic Steps**:
```bash
# Find log files
find ~/.llmspell -name "*.log" -type f

# Check log file size
du -h ~/.llmspell/kernels/*/kernel.log

# Check for rotated logs
ls -la ~/.llmspell/kernels/*/kernel.log*

# Test log rotation
# (should rotate at 10MB by default)
```

**Solutions**:

**Logs Not Created**
```bash
# Specify explicit log location
./target/debug/llmspell kernel start --daemon \
  --log-file /tmp/llmspell.log

# Check permissions
ls -la /tmp/llmspell.log

# Verify directory exists
mkdir -p $(dirname /tmp/llmspell.log)
```

**Logs Not Rotating**
```bash
# Check current log size
du -h ~/.llmspell/kernels/abc123/kernel.log

# Rotation happens at 10MB by default
# If >10MB and not rotating, check for issues

# Force new kernel to test rotation
./target/debug/llmspell kernel stop abc123
./target/debug/llmspell kernel start --daemon
```

**Rotated Logs Not Compressed**
```bash
# Check for .gz files
ls -la ~/.llmspell/kernels/abc123/kernel.log*.gz

# If missing, compression may have failed
# Check for flate2 dependency
cargo tree | grep flate2
```

---

## 7. Diagnostic Procedures

### 7.1 Enable Trace Logging

**Global Trace Flag**:
```bash
# Set trace level when running commands
./target/debug/llmspell tool list --trace debug

# Available levels: off, error, warn, info, debug, trace
./target/debug/llmspell kernel start --trace trace
```

**Environment Variable**:
```bash
# Set RUST_LOG for fine-grained control
RUST_LOG=llmspell_kernel=debug,llmspell_tools=trace \
  ./target/debug/llmspell kernel start

# Component-specific logging
RUST_LOG=llmspell_kernel::transport=trace  # Transport layer only
RUST_LOG=llmspell_kernel::daemon=debug     # Daemon operations
RUST_LOG=llmspell_tools=info               # Tool execution
```

**Log to File**:
```bash
# Redirect output to file
./target/debug/llmspell kernel start 2>&1 | tee kernel.log

# Or use --log-file for daemon mode
./target/debug/llmspell kernel start --daemon \
  --log-file /tmp/kernel.log \
  --trace debug
```

---

### 7.2 Check Kernel Health

**Status Command**:
```bash
# List all kernels
./target/debug/llmspell kernel status

# Check specific kernel
./target/debug/llmspell kernel status abc123

# Output includes:
# - PID
# - Port
# - Uptime
# - Connection file path
# - Log file path
# - Status (healthy/busy/unknown)
```

**Process Information**:
```bash
# Check if kernel process exists
ps aux | grep llmspell | grep kernel

# Check resource usage
top -p <PID>

# Check open files
lsof -p <PID>

# Check network connections
netstat -anp | grep <PID>
```

**Log Analysis**:
```bash
# Check for errors in logs
grep -i error ~/.llmspell/kernels/*/kernel.log

# Check recent activity
tail -f ~/.llmspell/kernels/abc123/kernel.log

# Search for specific messages
grep "tool_request" ~/.llmspell/kernels/*/kernel.log

# Count message types
grep "msg_type" ~/.llmspell/kernels/*/kernel.log | \
  cut -d'"' -f4 | sort | uniq -c
```

---

### 7.3 Test Protocol Compliance

**DAP Tests**:
```bash
# Run all DAP unit tests
cargo test -p llmspell-kernel --test dap_tests

# Run specific DAP test
cargo test -p llmspell-kernel --test dap_tests \
  test_conditional_breakpoints

# Expected: 16 passed; 0 failed
```

**Jupyter Tests**:
```bash
# Setup Python environment
cd tests/python
python3 -m venv venv
source venv/bin/activate
pip install jupyter_client pyzmq pytest

# Run Jupyter protocol tests
./tests/scripts/run_python_tests.sh

# Or run specific test
python tests/python/test_raw_zmq.py
```

**Stress Tests**:
```bash
# Run all stress tests (takes ~3 minutes)
cargo test -p llmspell-kernel --test stress_test -- --ignored --nocapture

# Run specific stress test
cargo test -p llmspell-kernel --test stress_test \
  test_rapid_tool_list_operations -- --ignored --nocapture

# Expected: 100% success rate across 15,000+ operations
```

---

### 7.4 Profile Performance

**Benchmark Tests**:
```bash
# Run performance benchmarks
./scripts/testing/kernel-benchmark.sh

# Run specific package benchmarks
./scripts/testing/kernel-benchmark.sh -p llmspell-kernel

# Save baseline for comparison
./scripts/testing/kernel-benchmark.sh -b my-baseline

# Compare against baseline
./scripts/testing/kernel-benchmark.sh -c my-baseline

# View HTML reports
open target/criterion/report/index.html
```

**Manual Timing**:
```bash
# Time tool commands
time ./target/debug/llmspell tool list
time ./target/debug/llmspell tool invoke calculator --params '{"input": "2+2"}'

# Expected times (debug build):
# tool list: <50ms
# tool invoke: <50ms
```

---

## 8. FAQ

### 8.1 General Questions

**Q: How do I start a kernel in daemon mode?**

A: Use the `--daemon` flag:
```bash
./target/debug/llmspell kernel start --daemon --port 59000

# Check it's running
./target/debug/llmspell kernel status
```

**Q: Where are kernel logs stored?**

A: Logs are in `~/.llmspell/kernels/<kernel-id>/kernel.log`:
```bash
# Find all kernel logs
find ~/.llmspell -name "kernel.log"

# View latest logs
tail -f ~/.llmspell/kernels/*/kernel.log
```

**Q: How do I stop a kernel?**

A: Use the stop command or kill signal:
```bash
# Stop by kernel ID
./target/debug/llmspell kernel stop abc123

# Or send signal to process
kill -TERM <PID>

# Force kill if stuck
kill -9 <PID>
```

**Q: Can I run multiple kernels simultaneously?**

A: Yes, use different ports:
```bash
./target/debug/llmspell kernel start --port 59000 --daemon
./target/debug/llmspell kernel start --port 60000 --daemon
./target/debug/llmspell kernel start --port 61000 --daemon

# List all running kernels
./target/debug/llmspell kernel status
```

---

### 8.2 Tool Commands

**Q: How many tools are available?**

A: 30 tools (as of Phase 10):
```bash
./target/debug/llmspell tool list | wc -l
# Should show ~30 tools
```

**Q: How do I find a specific tool?**

A: Use the search command:
```bash
./target/debug/llmspell tool search calc
./target/debug/llmspell tool search file
```

**Q: What parameters does a tool accept?**

A: Use the info command:
```bash
./target/debug/llmspell tool info calculator

# Shows:
# - Description
# - Category
# - Required parameters
# - Optional parameters
# - Example usage
```

**Q: Can I invoke tools without starting a kernel?**

A: Yes, tool commands use embedded kernel by default:
```bash
# No kernel start needed
./target/debug/llmspell tool list
./target/debug/llmspell tool invoke calculator --params '{"input": "2+2"}'
```

---

### 8.3 Jupyter Integration

**Q: How do I connect Jupyter Lab to llmspell?**

A: Start kernel and use connection file:
```bash
# Start kernel
./target/debug/llmspell kernel start --daemon

# Get connection file path
./target/debug/llmspell kernel status
# Output shows: connection_file: ~/.llmspell/kernels/abc123/kernel.json

# In Jupyter Lab:
# File → New → Console
# Connect to existing kernel → Browse to connection file
```

**Q: What Jupyter protocol version is supported?**

A: Jupyter Wire Protocol v5.3 (latest stable):
```json
// Message header includes:
{
  "version": "5.3",
  "msg_type": "kernel_info_request"
}
```

**Q: Do I need HMAC authentication?**

A: Yes, it's required by Jupyter protocol:
```bash
# Auth key is in connection file
cat ~/.llmspell/kernels/abc123/kernel.json | grep key

# Jupyter clients handle HMAC automatically
# Only manual ZeroMQ connections need to implement signing
```

---

### 8.4 DAP Debugging

**Q: How do I debug Lua scripts with VS Code?**

A: Use DAP via Jupyter control channel (future: native DAP adapter):
```bash
# 1. Start kernel with DAP enabled
./target/debug/llmspell kernel start --daemon

# 2. Create VS Code launch configuration
# (see docs/user-guide/ide-integration.md)

# 3. Set breakpoints in VS Code
# 4. Run script with debugging enabled
```

**Q: What DAP commands are supported?**

A: Core DAP features implemented:
- initialize, launch, attach
- setBreakpoints (including conditional)
- continue, next, stepIn, stepOut
- stackTrace, scopes, variables
- evaluate, disconnect

See: `cargo test -p llmspell-kernel --test dap_tests` for full list.

**Q: Can I set conditional breakpoints?**

A: Yes:
```json
{
  "command": "setBreakpoints",
  "arguments": {
    "source": {"path": "script.lua"},
    "breakpoints": [
      {
        "line": 10,
        "condition": "x > 5"  // Only break if condition true
      }
    ]
  }
}
```

---

### 8.5 Performance

**Q: What are the expected performance metrics?**

A: From stress test results:
- Tool operations: 88 ops/sec sustained
- Message handling: ~12ms average latency
- Large payloads: 1MB JSON in 12ms
- Sustained load: 10,000 operations without degradation

**Q: How can I improve performance?**

A: Tips:
1. Use release build: `cargo build --release`
2. Reduce logging: `RUST_LOG=warn` or `RUST_LOG=off`
3. Use embedded kernel for CLI tools (automatic)
4. Close unnecessary kernel connections
5. Use local kernel (not remote over network)

**Q: Why is debug build so slow?**

A: Debug builds are not optimized:
```bash
# Debug build: ~100ms per operation
time ./target/debug/llmspell tool list

# Release build: ~10ms per operation
time ./target/release/llmspell tool list
```

---

### 8.6 Troubleshooting

**Q: How do I enable verbose logging?**

A: Use `--trace` flag or `RUST_LOG` environment variable:
```bash
# Command-line flag
./target/debug/llmspell kernel start --trace debug

# Environment variable
RUST_LOG=debug ./target/debug/llmspell kernel start

# Component-specific
RUST_LOG=llmspell_kernel=trace ./target/debug/llmspell kernel start
```

**Q: Where can I find more help?**

A: Resources:
- Documentation: `docs/user-guide/README.md`
- Technical docs: `docs/technical/README.md`
- Protocol compliance: `docs/technical/protocol-compliance-report.md`
- GitHub issues: https://github.com/anthropics/llmspell/issues
- Main troubleshooting: `docs/user-guide/troubleshooting.md`

**Q: How do I report a bug?**

A: Include diagnostics:
```bash
# 1. Gather information
./target/debug/llmspell --version
rustc --version
uname -a

# 2. Reproduce with verbose logging
RUST_LOG=debug ./target/debug/llmspell [command] 2>&1 | tee bug-report.log

# 3. Check kernel status
./target/debug/llmspell kernel status

# 4. Include logs
find ~/.llmspell -name "*.log" -type f

# 5. Report at: https://github.com/anthropics/llmspell/issues
```

---

## Appendix A: Quick Reference Commands

### Kernel Management
```bash
# Start kernel
./target/debug/llmspell kernel start --port 59000

# Start in daemon mode
./target/debug/llmspell kernel start --daemon

# Stop kernel
./target/debug/llmspell kernel stop abc123

# Check status
./target/debug/llmspell kernel status

# Connect to remote kernel
./target/debug/llmspell kernel connect localhost:59000
```

### Tool Commands
```bash
# List all tools
./target/debug/llmspell tool list

# Search for tools
./target/debug/llmspell tool search calculator

# Get tool info
./target/debug/llmspell tool info calculator

# Invoke tool
./target/debug/llmspell tool invoke calculator \
  --params '{"input": "2+2"}'
```

### Diagnostics
```bash
# Enable verbose logging
RUST_LOG=debug ./target/debug/llmspell [command]

# View logs
tail -f ~/.llmspell/kernels/*/kernel.log

# Run tests
cargo test -p llmspell-kernel --test dap_tests
cargo test -p llmspell-kernel --test stress_test -- --ignored

# Run benchmarks
./scripts/testing/kernel-benchmark.sh
```

---

## Appendix B: Error Code Reference

### Kernel Errors (1000-1999)
- 1001: Failed to start kernel
- 1002: Port already in use
- 1003: PID file exists
- 1004: Failed to daemonize
- 1005: Signal handler registration failed

### Transport Errors (2000-2999)
- 2001: Failed to bind socket
- 2002: Connection failed
- 2003: HMAC authentication failed
- 2004: Invalid message format
- 2005: Timeout waiting for response

### Tool Errors (3000-3999)
- 3001: Tool not found
- 3002: Invalid parameters
- 3003: Tool execution failed
- 3004: Timeout executing tool

### Protocol Errors (4000-4999)
- 4001: Protocol version mismatch
- 4002: Unsupported message type
- 4003: Invalid DAP command
- 4004: Breakpoint error

---

## Appendix C: Monitoring and Health Checks

### Health Check Script
```bash
#!/bin/bash
# Check kernel health

KERNEL_ID="abc123"

# Check if kernel is running
if ./target/debug/llmspell kernel status $KERNEL_ID > /dev/null 2>&1; then
    echo "✓ Kernel is running"
else
    echo "✗ Kernel is not running"
    exit 1
fi

# Check if ports are accessible
for port in 59000 59001 59002 59003 59004; do
    if nc -zv 127.0.0.1 $port 2>&1 | grep -q succeeded; then
        echo "✓ Port $port is accessible"
    else
        echo "✗ Port $port is not accessible"
    fi
done

# Check log file size
LOG_FILE=~/.llmspell/kernels/$KERNEL_ID/kernel.log
if [ -f "$LOG_FILE" ]; then
    SIZE=$(du -h "$LOG_FILE" | cut -f1)
    echo "✓ Log file size: $SIZE"
else
    echo "✗ Log file not found"
fi
```

### Monitoring Dashboard (Future)
```bash
# Future: Web-based monitoring dashboard
./target/debug/llmspell kernel monitor --port 8080
# Open http://localhost:8080 for dashboard
```
