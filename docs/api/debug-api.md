# Debug API Reference

Complete reference for LLMSpell's debug infrastructure API.

## Debug Global Object

The `Debug` global object provides access to all debugging functionality from Lua scripts.

### Logging Methods

#### `Debug.log(level, message, [module])`

Log a message at the specified level.

- **Parameters:**
  - `level` (string): Log level ("error", "warn", "info", "debug", "trace")
  - `message` (string): The message to log
  - `module` (string, optional): Module name for filtering

```lua
Debug.log("info", "Operation completed", "workflow.step1")
```

#### `Debug.error(message, [module])`

Log an error message.

- **Parameters:**
  - `message` (string): Error message
  - `module` (string, optional): Module name

```lua
Debug.error("Connection failed", "network.client")
```

#### `Debug.warn(message, [module])`

Log a warning message.

#### `Debug.info(message, [module])`

Log an info message.

#### `Debug.debug(message, [module])`

Log a debug message.

#### `Debug.trace(message, [module])`

Log a trace message.

#### `Debug.logWithData(level, message, data, [module])`

Log a message with structured metadata.

- **Parameters:**
  - `level` (string): Log level
  - `message` (string): Message text
  - `data` (table): Structured data to include
  - `module` (string, optional): Module name

```lua
Debug.logWithData("info", "User action", {
    user_id = "123",
    action = "login",
    duration_ms = 150
}, "auth.tracking")
```

### Performance Timing

#### `Debug.timer(name)` → `LuaTimer`

Create a new performance timer.

- **Parameters:**
  - `name` (string): Timer name for reporting
- **Returns:** `LuaTimer` object

```lua
local timer = Debug.timer("data_processing")
```

### LuaTimer Object

Timer objects returned by `Debug.timer()`.

#### `timer:stop()` → `number`

Stop the timer and return duration in milliseconds.

- **Returns:** Duration in milliseconds

```lua
local duration = timer:stop()
print("Operation took " .. duration .. "ms")
```

#### `timer:lap(name)`

Record a lap/checkpoint without stopping the timer.

- **Parameters:**
  - `name` (string): Lap name

```lua
timer:lap("data_loaded")
-- Continue processing...
timer:lap("data_processed")
```

#### `timer:elapsed()` → `number`

Get elapsed time without stopping the timer.

- **Returns:** Elapsed time in milliseconds

```lua
local elapsed = timer:elapsed()
if elapsed > 5000 then
    Debug.warn("Operation taking longer than expected", "performance")
end
```

### Configuration Methods

#### `Debug.setLevel(level)`

Set the global debug level.

- **Parameters:**
  - `level` (string): Debug level ("off", "error", "warn", "info", "debug", "trace")

```lua
Debug.setLevel("debug")
```

#### `Debug.getLevel()` → `string`

Get the current debug level.

- **Returns:** Current debug level string

#### `Debug.setEnabled(enabled)`

Enable or disable debugging entirely.

- **Parameters:**
  - `enabled` (boolean): Whether debugging is enabled

#### `Debug.isEnabled()` → `boolean`

Check if debugging is enabled.

- **Returns:** Boolean indicating if debugging is enabled

### Module Filtering

#### `Debug.addModuleFilter(pattern, enabled)`

Add a simple module filter rule.

- **Parameters:**
  - `pattern` (string): Filter pattern (supports wildcards)
  - `enabled` (boolean): Whether to enable or disable matching modules

```lua
Debug.addModuleFilter("workflow.*", true)  -- Enable workflow modules
Debug.addModuleFilter("*.test", false)     -- Disable test modules
```

#### `Debug.addAdvancedFilter(pattern, pattern_type, enabled)`

Add an advanced filter rule with specific pattern type.

- **Parameters:**
  - `pattern` (string): Filter pattern
  - `pattern_type` (string): Pattern type ("exact", "wildcard", "regex", "hierarchical")
  - `enabled` (boolean): Whether to enable or disable

```lua
Debug.addAdvancedFilter("^auth\\..*", "regex", true)
Debug.addAdvancedFilter("core.module", "exact", true)
```

#### `Debug.clearModuleFilters()`

Remove all module filter rules.

#### `Debug.removeModuleFilter(pattern)` → `boolean`

Remove a specific filter pattern.

- **Parameters:**
  - `pattern` (string): Pattern to remove
- **Returns:** Boolean indicating if pattern was found and removed

#### `Debug.setDefaultFilterEnabled(enabled)`

Set the default behavior when no filter rules match.

- **Parameters:**
  - `enabled` (boolean): Default filter behavior

#### `Debug.getFilterSummary()` → `table`

Get a summary of current filter configuration.

- **Returns:** Table with filter information

```lua
local summary = Debug.getFilterSummary()
print("Total rules: " .. summary.total_rules)
print("Default enabled: " .. tostring(summary.default_enabled))

for i = 1, #summary.rules do
    local rule = summary.rules[i]
    print("Rule: " .. rule.pattern .. " (" .. rule.pattern_type .. ") = " .. tostring(rule.enabled))
end
```

### Object Dumping

#### `Debug.dump(value, [label])` → `string`

Dump a Lua value with default formatting.

- **Parameters:**
  - `value` (any): Value to dump
  - `label` (string, optional): Label for the dump
- **Returns:** Formatted string representation

```lua
local data = {name = "test", items = {1, 2, 3}}
print(Debug.dump(data, "test_data"))
```

#### `Debug.dumpCompact(value, [label])` → `string`

Dump a value in compact (one-line) format.

#### `Debug.dumpVerbose(value, [label])` → `string`

Dump a value with verbose details including types and addresses.

#### `Debug.dumpWithOptions(value, options, [label])` → `string`

Dump a value with custom formatting options.

- **Parameters:**
  - `value` (any): Value to dump
  - `options` (table): Formatting options
  - `label` (string, optional): Label for the dump

**Options table:**
- `max_depth` (number): Maximum nesting depth (default: 10)
- `indent_size` (number): Spaces per indent level (default: 2)
- `max_string_length` (number): Max string length before truncation (default: 200)
- `max_array_elements` (number): Max array elements to show (default: 50)
- `max_table_pairs` (number): Max table pairs to show (default: 50)
- `show_types` (boolean): Include type information (default: true)
- `show_addresses` (boolean): Include memory addresses (default: false)
- `compact_mode` (boolean): Use compact formatting (default: false)

```lua
print(Debug.dumpWithOptions(data, {
    max_depth = 3,
    compact_mode = true,
    show_types = false
}, "custom_dump"))
```

### Stack Trace Collection

#### `Debug.stackTrace([options])` → `string`

Capture and format a stack trace.

- **Parameters:**
  - `options` (table, optional): Stack trace options
- **Returns:** Formatted stack trace string

**Options table:**
- `max_depth` (number): Maximum stack depth (default: 50)
- `capture_locals` (boolean): Include local variables (default: false)
- `capture_upvalues` (boolean): Include upvalues (default: false)
- `include_source` (boolean): Include source information (default: true)

```lua
local function error_handler()
    local trace = Debug.stackTrace({
        max_depth = 20,
        capture_locals = true,
        include_source = true
    })
    print("Error stack trace:\n" .. trace)
end
```

#### `Debug.stackTraceJson([options])` → `string`

Capture stack trace and return as JSON.

- **Parameters:**
  - `options` (table, optional): Same as `stackTrace()`
- **Returns:** JSON-formatted stack trace

### Memory Monitoring

#### `Debug.memoryStats()` → `table`

Get current memory statistics.

- **Returns:** Table with memory information

```lua
local stats = Debug.memoryStats()
print("Used bytes: " .. stats.used_bytes)
print("Allocated bytes: " .. stats.allocated_bytes)
print("Resident bytes: " .. stats.resident_bytes)
print("GC collections: " .. stats.collections)
```

#### `Debug.memorySnapshot()` → `table`

Take a memory usage snapshot.

- **Returns:** Table with snapshot data

```lua
local snapshot = Debug.memorySnapshot()
print("Timestamp: " .. snapshot.timestamp_secs)
print("Active trackers: " .. snapshot.active_trackers)

if snapshot.total_memory_delta_bytes then
    print("Memory delta: " .. snapshot.total_memory_delta_bytes .. " bytes")
end
```

### Event Recording

#### `Debug.recordEvent(timer_id, event_name, [metadata])` → `boolean`

Record a custom event within a timer's measurement.

- **Parameters:**
  - `timer_id` (string): Timer ID (from timer.id)
  - `event_name` (string): Event name
  - `metadata` (table, optional): Event metadata
- **Returns:** Boolean indicating success

```lua
local timer = Debug.timer("operation")

Debug.recordEvent(timer.id, "initialization", {
    config_loaded = true,
    plugins = 3
})

-- ... do work ...

Debug.recordEvent(timer.id, "completion", {
    items_processed = 100,
    success = true
})

timer:stop()
```

### Captured Entries Management

#### `Debug.getCapturedEntries([limit])` → `table`

Get captured debug entries.

- **Parameters:**
  - `limit` (number, optional): Maximum number of entries to return
- **Returns:** Array of debug entry tables

```lua
local entries = Debug.getCapturedEntries(10)  -- Get last 10 entries
for i = 1, #entries do
    local entry = entries[i]
    print("[" .. entry.level .. "] " .. entry.message)
    print("  Module: " .. (entry.module or "none"))
    print("  Time: " .. entry.timestamp)
end
```

**Entry table structure:**
- `timestamp` (string): ISO 8601 timestamp
- `level` (string): Log level
- `message` (string): Log message
- `module` (string, optional): Module name
- `metadata` (table, optional): Structured metadata

#### `Debug.clearCaptured()`

Clear all captured debug entries.

### Performance Reports

#### `Debug.performanceReport()` → `string`

Generate a text-based performance report.

- **Returns:** Human-readable performance report

```lua
print("Performance Report:")
print(Debug.performanceReport())
```

#### `Debug.jsonReport()` → `string`

Generate a JSON performance report for external tools.

- **Returns:** JSON-formatted performance data

```lua
local json_report = Debug.jsonReport()
-- Send to external monitoring system
```

#### `Debug.flameGraph()` → `string`

Generate flame graph compatible output.

- **Returns:** Flame graph data in standard format

```lua
local flame_data = Debug.flameGraph()
-- Use with tools like speedscope.app
print(flame_data)
```

## Filter Pattern Types

### Exact Match
Matches module name exactly.
```lua
Debug.addAdvancedFilter("auth.login", "exact", true)
-- Matches: auth.login
-- Doesn't match: auth.login.oauth, auth.logout
```

### Wildcard Match
Supports `*` (any characters) and `?` (single character).
```lua
Debug.addAdvancedFilter("workflow.step*", "wildcard", true)
-- Matches: workflow.step1, workflow.step_init, workflow.stepABC
-- Doesn't match: workflow.other
```

### Hierarchical Match
Matches module and all sub-modules.
```lua
Debug.addAdvancedFilter("system", "hierarchical", true)
-- Matches: system, system.memory, system.cpu.usage
-- Doesn't match: systems, other.system
```

### Regular Expression
Full regex pattern matching.
```lua
Debug.addAdvancedFilter("^(auth|security)\\.", "regex", true)
-- Matches: auth.login, security.audit
-- Doesn't match: user.auth, app.security
```

## Error Handling

All Debug methods are designed to be safe and non-throwing. If an error occurs within the debug system:

1. The error is logged internally
2. The operation fails gracefully  
3. Script execution continues normally
4. No exceptions are thrown to user code

```lua
-- These will never throw errors, even with invalid input
Debug.log("invalid_level", "message")  -- Safely ignored
Debug.timer("")  -- Returns valid timer with empty name
local bad_timer = Debug.timer(nil)
bad_timer:stop()  -- Returns 0, doesn't crash
```

## Performance Considerations

### Overhead

- Log level checks are fast (atomic operations)
- Module filtering uses efficient hash tables and compiled regex
- Timer operations have minimal overhead (<10ms)
- Object dumping can be expensive for large objects - use depth limits

### Memory Usage

- Captured entries are stored in a circular buffer
- Default limit: 10,000 entries
- Clear entries periodically in long-running scripts
- Performance trackers are cleaned up automatically when timers stop

### Best Practices

1. Use appropriate log levels
2. Leverage module filtering in production
3. Limit object dump depth for large structures
4. Clear captured entries periodically
5. Monitor memory usage with `Debug.memoryStats()`

## Thread Safety

The debug infrastructure is fully thread-safe:

- All operations use atomic operations or locks where necessary
- Multiple scripts can use debug features concurrently
- Timer IDs are globally unique across all scripts
- Captured entries are safely shared between threads