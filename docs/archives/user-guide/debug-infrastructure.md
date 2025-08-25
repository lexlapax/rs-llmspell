# Debug Infrastructure

LLMSpell provides a comprehensive debug infrastructure that enables developers to efficiently debug scripts, monitor performance, and diagnose issues in complex workflows.

## Quick Start

```lua
-- Basic logging
Debug.info("Application started", "app")
Debug.warn("Memory usage high", "system")
Debug.error("Connection failed", "network")

-- Performance timing
local timer = Debug.timer("operation")
-- ... do work ...
local duration = timer:stop()
Debug.info("Operation took " .. duration .. "ms", "performance")

-- Object inspection
local data = {name = "test", items = {1, 2, 3}}
print(Debug.dump(data))
```

## Core Features

### 1. Hierarchical Logging

LLMSpell supports structured logging with hierarchical levels:

- **ERROR**: Critical errors that require immediate attention
- **WARN**: Warnings about potential issues
- **INFO**: General information messages (default level)
- **DEBUG**: Detailed debugging information
- **TRACE**: Very detailed tracing information

```lua
-- Set debug level
Debug.setLevel("debug")
print("Current level: " .. Debug.getLevel())

-- Enable/disable debugging entirely
Debug.setEnabled(false)
print("Debug enabled: " .. tostring(Debug.isEnabled()))
```

### 2. Performance Profiling

#### Basic Timing

```lua
local timer = Debug.timer("data_processing")

-- Your code here
local sum = 0
for i = 1, 1000000 do
    sum = sum + i
end

local duration = timer:stop()
print("Processing took " .. duration .. "ms")
```

#### Advanced Timing with Laps

```lua
local timer = Debug.timer("multi_stage_operation")

-- Stage 1
timer:lap("loading")
-- ... loading code ...

-- Stage 2  
timer:lap("processing")
-- ... processing code ...

-- Stage 3
timer:lap("saving")
-- ... saving code ...

local total_time = timer:stop()
```

#### Performance Reports

```lua
-- Generate comprehensive performance report
print(Debug.performanceReport())

-- Generate JSON report for external tools
local json_report = Debug.jsonReport()

-- Generate flame graph data
local flame_data = Debug.flameGraph()
print(flame_data)  -- Use with external flame graph tools
```

### 3. Module-Based Filtering

Filter debug output to focus on specific components:

#### Simple Patterns

```lua
-- Only show workflow-related messages
Debug.addModuleFilter("workflow.*", true)

-- Hide all test-related messages
Debug.addModuleFilter("*.test", false)
```

#### Advanced Patterns

```lua
-- Use regex patterns
Debug.addAdvancedFilter("^(auth|security)\\..*", "regex", true)

-- Exact matches
Debug.addAdvancedFilter("critical.component", "exact", true)

-- Hierarchical patterns
Debug.addAdvancedFilter("system.memory", "hierarchical", true)
```

#### Filter Management

```lua
-- Get current filter state
local summary = Debug.getFilterSummary()
print("Total rules: " .. summary.total_rules)
print("Default enabled: " .. tostring(summary.default_enabled))

-- Remove specific filters
Debug.removeModuleFilter("workflow.*")

-- Clear all filters
Debug.clearModuleFilters()
```

### 4. Object Dumping and Inspection

#### Basic Dumping

```lua
local data = {
    name = "User",
    age = 30,
    preferences = {"theme_dark", "notifications_on"},
    metadata = {
        created = "2024-01-01",
        last_login = "2024-01-15"
    }
}

-- Different dump styles
print(Debug.dump(data, "user_data"))           -- Default
print(Debug.dumpCompact(data, "user_compact")) -- One-liner
print(Debug.dumpVerbose(data, "user_verbose")) -- Detailed
```

#### Custom Dump Options

```lua
print(Debug.dumpWithOptions(data, {
    max_depth = 2,
    compact_mode = true,
    show_types = true,
    max_string_length = 50,
    show_addresses = false
}, "custom_dump"))
```

### 5. Stack Trace Collection

#### Basic Stack Traces

```lua
local function problematic_function()
    local trace = Debug.stackTrace()
    print("Error occurred at:")
    print(trace)
end

local function call_chain()
    problematic_function()
end

call_chain()
```

#### Detailed Stack Traces

```lua
local function detailed_trace()
    local trace = Debug.stackTrace({
        max_depth = 20,
        capture_locals = true,
        capture_upvalues = true,
        include_source = true
    })
    print(trace)
end

-- JSON format for external tools
local function json_trace()
    local trace_json = Debug.stackTraceJson({
        max_depth = 10,
        include_source = true
    })
    return trace_json
end
```

### 6. Memory Monitoring

```lua
-- Get current memory statistics
local stats = Debug.memoryStats()
print("Memory used: " .. stats.used_bytes .. " bytes")
print("Memory allocated: " .. stats.allocated_bytes .. " bytes")
print("GC collections: " .. stats.collections)

-- Take memory snapshot
local snapshot = Debug.memorySnapshot()
print("Active trackers: " .. snapshot.active_trackers)
print("Snapshot timestamp: " .. snapshot.timestamp_secs)
```

### 7. Event Recording

Track custom events within performance measurements:

```lua
local timer = Debug.timer("complex_operation")

-- Record initialization
Debug.recordEvent(timer.id, "initialization", {
    config_loaded = true,
    plugins_count = 5
})

-- Record processing events
for i = 1, 10 do
    Debug.recordEvent(timer.id, "item_processed", {
        item_id = i,
        size_bytes = math.random(100, 1000)
    })
end

-- Record completion
Debug.recordEvent(timer.id, "completion", {
    items_processed = 10,
    errors_encountered = 0
})

local total_time = timer:stop()
```

### 8. Metadata Logging

Include structured data with log messages:

```lua
Debug.logWithData("info", "User action completed", {
    user_id = "user123",
    action = "file_upload",
    file_size = 1024000,
    duration_ms = 250,
    success = true
}, "user.actions")
```

## Configuration

### Environment Variables

```bash
# Set debug level
export LLMSPELL_DEBUG_LEVEL=debug

# Enable/disable debug output
export LLMSPELL_DEBUG_ENABLED=true

# Set debug output format
export LLMSPELL_DEBUG_FORMAT=json
```

### Configuration File

```toml
[debug]
level = "info"
enabled = true
output_format = "text"

[debug.filtering]
default_enabled = true

[[debug.filtering.rules]]
pattern = "workflow.*"
enabled = true
type = "hierarchical"

[[debug.filtering.rules]]
pattern = "*.test"
enabled = false
type = "wildcard"
```

## Best Practices

### 1. Structured Module Naming

Use hierarchical module names for better filtering:

```lua
-- Good
Debug.info("Step completed", "workflow.execution.step1")
Debug.info("Cache hit", "system.cache.user_data")
Debug.info("Query executed", "database.query.users")

-- Less ideal
Debug.info("Step completed", "step1")
Debug.info("Cache hit", "cache")
Debug.info("Query executed", "db")
```

### 2. Appropriate Log Levels

```lua
-- ERROR: Critical failures
Debug.error("Database connection lost", "database.connection")

-- WARN: Potential issues
Debug.warn("Memory usage at 90%", "system.memory")

-- INFO: General information
Debug.info("User logged in", "auth.login")

-- DEBUG: Development debugging
Debug.debug("Cache state updated", "cache.internal")

-- TRACE: Very detailed tracing
Debug.trace("Function entered with params", "module.function")
```

### 3. Performance Monitoring

```lua
-- Monitor critical paths
local critical_timer = Debug.timer("critical_operation")
-- ... critical code ...
local duration = critical_timer:stop()

if duration > 1000 then  -- More than 1 second
    Debug.warn("Critical operation took " .. duration .. "ms", "performance")
end
```

### 4. Error Context

```lua
local function handle_error(err)
    Debug.error("Operation failed: " .. tostring(err), "error.handler")
    
    -- Include stack trace for debugging
    local trace = Debug.stackTrace()
    Debug.debug("Error stack trace:\n" .. trace, "error.stack")
    
    -- Log error metadata
    Debug.logWithData("error", "Error details", {
        error_type = type(err),
        error_message = tostring(err),
        timestamp = os.time(),
        context = "user_operation"
    }, "error.metadata")
end
```

## Integration with External Tools

### Flame Graphs

```lua
-- Generate flame graph data
local flame_data = Debug.flameGraph()

-- Save to file for external processing
-- Use with tools like speedscope.app or FlameGraph
```

### JSON Export

```lua
-- Export performance data as JSON
local json_report = Debug.jsonReport()

-- This can be processed by external monitoring tools
-- like Grafana, Prometheus, or custom dashboards
```

### Captured Entries

```lua
-- Get recent debug entries for analysis
local recent_entries = Debug.getCapturedEntries(100)

-- Process entries for external logging systems
for i = 1, #recent_entries do
    local entry = recent_entries[i]
    print("Level: " .. entry.level)
    print("Message: " .. entry.message)
    print("Module: " .. (entry.module or "none"))
    print("Timestamp: " .. entry.timestamp)
end
```

## Troubleshooting

### Common Issues

1. **No debug output appearing**
   - Check if debugging is enabled: `Debug.isEnabled()`
   - Verify debug level: `Debug.getLevel()`
   - Check module filters: `Debug.getFilterSummary()`

2. **Performance impact**
   - Use appropriate debug levels in production
   - Leverage module filtering to reduce overhead
   - Monitor performance with the profiling tools

3. **Memory usage**
   - Monitor with `Debug.memoryStats()`
   - Clear captured entries periodically: `Debug.clearCaptured()`
   - Use appropriate capture limits

### Debug the Debug System

```lua
-- Check debug system state
print("Debug enabled: " .. tostring(Debug.isEnabled()))
print("Debug level: " .. Debug.getLevel())

-- Review current filters
local summary = Debug.getFilterSummary()
print("Filter rules: " .. summary.total_rules)
print("Default enabled: " .. tostring(summary.default_enabled))

-- Monitor debug system performance
local stats = Debug.memoryStats()
print("Debug memory usage: " .. stats.used_bytes .. " bytes")
```

## Examples

See the following example files for comprehensive demonstrations:

- `examples/lua/debug/debug-basic.lua` - Basic usage patterns
- `examples/lua/debug/debug-performance.lua` - Advanced performance profiling
- `examples/lua/debug/debug-filtering.lua` - Module filtering strategies
- `examples/lua/debug/debug-comprehensive.lua` - Complete feature demonstration

## API Reference

For detailed API documentation, see the [Debug API Reference](../api/debug-api.md).