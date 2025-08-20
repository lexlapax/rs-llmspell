-- Comprehensive Debug Infrastructure Example
-- Demonstrates all debug features available in LLMSpell

print("🔧 LLMSpell Debug Infrastructure - Comprehensive Example")
print("=" .. string.rep("=", 60))

-- 1. Basic Debug Logging
print("\n📋 1. Basic Debug Logging")
print("-" .. string.rep("-", 30))

Debug.info("Starting debug example", "debug.example")
Debug.warn("This is a warning message", "debug.example")
Debug.error("This is an error message", "debug.example")
Debug.debug("This debug message might not show depending on level", "debug.example")
Debug.trace("This trace message might not show depending on level", "debug.example")

-- Check current debug level
print("Current debug level: " .. Debug.getLevel())
print("Debug enabled: " .. tostring(Debug.isEnabled()))

-- 2. Performance Timing
print("\n⏱️  2. Performance Timing")
print("-" .. string.rep("-", 30))

-- Create a timer for the entire operation
local main_timer = Debug.timer("comprehensive_example")
local start_time = main_timer:elapsed()

-- Simulate some work with nested timers
local data_timer = Debug.timer("data_processing")

-- Simulate data loading
for i = 1, 3 do
    data_timer:lap("loading_batch_" .. i)
    -- Simulate work
    local sum = 0
    for j = 1, 100000 do
        sum = sum + j
    end
    Debug.info("Processed batch " .. i .. " (sum: " .. sum .. ")", "data.processing")
end

local data_time = data_timer:stop()
Debug.info("Data processing completed in " .. string.format("%.2f", data_time) .. "ms", "performance")

-- 3. Module Filtering
print("\n🎯 3. Module Filtering")
print("-" .. string.rep("-", 30))

-- Show current filter state
print("Current filter summary:")
local filter_summary = Debug.getFilterSummary()
print("  Default enabled: " .. tostring(filter_summary.default_enabled))
print("  Total rules: " .. filter_summary.total_rules)

-- Add some filters
Debug.addModuleFilter("workflow.*", true)
Debug.addModuleFilter("*.test", false)
Debug.addAdvancedFilter("agent\\.internal\\..*", "regex", false)

-- Test the filters
Debug.info("This should be logged", "workflow.step1")
Debug.info("This should NOT be logged", "unit.test")
Debug.info("This should NOT be logged", "agent.internal.cache")
Debug.info("This should be logged", "agent.executor")

-- 4. Advanced Debug Features
print("\n🚀 4. Advanced Debug Features")
print("-" .. string.rep("-", 30))

-- Log with metadata
Debug.logWithData("info", "Operation completed with metadata", {
    duration_ms = 150,
    items_processed = 42,
    success_rate = 0.95,
    errors = {"timeout", "network"}
}, "advanced.logging")

-- 5. Object Dumping
print("\n📦 5. Object Dumping")
print("-" .. string.rep("-", 30))

-- Create complex test data
local test_data = {
    string_field = "Hello, World!",
    number_field = 42,
    boolean_field = true,
    nil_field = nil,
    array = {1, 2, 3, "four", true},
    nested = {
        level1 = {
            level2 = {
                deep_value = "Found me!"
            }
        },
        metadata = {
            created = "2024-01-01",
            version = "1.0.0"
        }
    },
    mixed_array = {
        {name = "Alice", age = 30},
        {name = "Bob", age = 25},
        {name = "Charlie", age = 35}
    }
}

-- Different dump styles
print("Default dump:")
print(Debug.dump(test_data, "test_data"))

print("\nCompact dump:")
print(Debug.dumpCompact(test_data, "test_data_compact"))

print("\nVerbose dump:")
print(Debug.dumpVerbose(test_data, "test_data_verbose"))

-- Custom dump options
print("\nCustom dump (max depth 2, compact):")
print(Debug.dumpWithOptions(test_data, {
    max_depth = 2,
    compact_mode = true,
    show_types = true,
    max_string_length = 50
}, "test_data_custom"))

-- 6. Stack Trace Collection
print("\n🔍 6. Stack Trace Collection")
print("-" .. string.rep("-", 30))

local function level3()
    local trace = Debug.stackTrace()
    print("Stack trace from level3:")
    print(trace)
end

local function level2()
    level3()
end

local function level1()
    level2()
end

level1()

-- Stack trace with options
local function detailed_trace()
    local trace = Debug.stackTrace({
        max_depth = 10,
        capture_locals = true,
        include_source = true
    })
    print("Detailed stack trace:")
    print(trace)
end

detailed_trace()

-- 7. Memory and Performance Monitoring
print("\n💾 7. Memory and Performance Monitoring")
print("-" .. string.rep("-", 30))

-- Get memory stats
local memory_stats = Debug.memoryStats()
print("Memory stats:")
print("  Used bytes: " .. memory_stats.used_bytes)
print("  Allocated bytes: " .. memory_stats.allocated_bytes)
print("  Collections: " .. memory_stats.collections)

-- Memory snapshot
local snapshot = Debug.memorySnapshot()
print("Memory snapshot:")
print("  Timestamp: " .. snapshot.timestamp_secs)
print("  Active trackers: " .. snapshot.active_trackers)

-- 8. Event Recording
print("\n📈 8. Event Recording")
print("-" .. string.rep("-", 30))

local event_timer = Debug.timer("event_example")

-- Record custom events
Debug.recordEvent(event_timer.id, "initialization", {
    config_loaded = true,
    plugins = 3
})

-- Simulate some processing
for i = 1, 5 do
    Debug.recordEvent(event_timer.id, "processing_item", {
        item_id = i,
        processing_time = math.random(10, 100)
    })
end

Debug.recordEvent(event_timer.id, "cleanup", {
    temp_files_removed = 7,
    cache_cleared = true
})

local event_time = event_timer:stop()
Debug.info("Event recording example completed in " .. string.format("%.2f", event_time) .. "ms", "events")

-- 9. Performance Reports
print("\n📊 9. Performance Reports")
print("-" .. string.rep("-", 30))

-- Generate text report
print("Performance Report:")
print(Debug.performanceReport())

-- Generate JSON report
print("\nJSON Report:")
local json_report = Debug.jsonReport()
print(json_report)

-- Generate flame graph data
print("\nFlame Graph Data:")
local flame_data = Debug.flameGraph()
print(flame_data)

-- 10. Debug State Management
print("\n🔧 10. Debug State Management")
print("-" .. string.rep("-", 30))

-- Get captured entries
local entries = Debug.getCapturedEntries(5) -- Last 5 entries
print("Last 5 debug entries:")
for i = 1, #entries do
    local entry = entries[i]
    print("  [" .. entry.level .. "] " .. entry.message .. " (" .. (entry.module or "no-module") .. ")")
end

-- Cleanup and final timing
local total_time = main_timer:stop()

print("\n✅ Example completed successfully!")
print("Total execution time: " .. string.format("%.2f", total_time) .. "ms")

-- Clear captured entries for cleanup
Debug.clearCaptured()

print("\n🎉 Debug infrastructure demonstration complete!")