-- Performance Profiling Example
-- Demonstrates advanced timing and profiling capabilities

print("🚀 LLMSpell Debug - Performance Profiling")
print("=" .. string.rep("=", 50))

-- 1. Nested Performance Tracking
print("\n⏱️  Nested Performance Tracking")
print("-" .. string.rep("-", 35))

local main_timer = Debug.timer("data_pipeline")

-- Stage 1: Data Loading
local load_timer = Debug.timer("data_loading")
for batch = 1, 3 do
    load_timer:lap("batch_" .. batch)
    
    -- Simulate loading work
    local data = {}
    for i = 1, 50000 do
        data[i] = math.random() * 100
    end
    
    Debug.info("Loaded batch " .. batch .. " (" .. #data .. " items)", "loading")
end
local load_time = load_timer:stop()

-- Stage 2: Data Processing
local process_timer = Debug.timer("data_processing")

-- Sub-stage: Filtering
process_timer:lap("filtering")
local filtered_count = 0
for i = 1, 100000 do
    if math.random() > 0.5 then
        filtered_count = filtered_count + 1
    end
end

-- Sub-stage: Sorting simulation
process_timer:lap("sorting")
local sort_operations = 0
for i = 1, 25000 do
    sort_operations = sort_operations + 1
end

-- Sub-stage: Aggregation
process_timer:lap("aggregation")
local sum = 0
for i = 1, 75000 do
    sum = sum + i
end

local process_time = process_timer:stop()

-- Stage 3: Output Generation
local output_timer = Debug.timer("output_generation")
local output_size = 0
for i = 1, 10000 do
    output_size = output_size + string.len("data_" .. i)
end
local output_time = output_timer:stop()

local total_time = main_timer:stop()

print("\n📊 Performance Summary")
print("-" .. string.rep("-", 25))
print("Data Loading: " .. string.format("%.2f", load_time) .. "ms")
print("Data Processing: " .. string.format("%.2f", process_time) .. "ms")
print("Output Generation: " .. string.format("%.2f", output_time) .. "ms")
print("Total Pipeline: " .. string.format("%.2f", total_time) .. "ms")

-- 2. Memory Tracking Simulation
print("\n💾 Memory Tracking")
print("-" .. string.rep("-", 20))

local memory_timer = Debug.timer("memory_intensive_operation")

-- Record start event
Debug.recordEvent(memory_timer.id, "operation_start", {
    initial_memory = "baseline"
})

-- Simulate memory allocation
local large_data = {}
for i = 1, 10000 do
    large_data[i] = {
        id = i,
        data = string.rep("x", 100),
        metadata = {
            created = os.time(),
            size = 100
        }
    }
end

Debug.recordEvent(memory_timer.id, "allocation_complete", {
    items_allocated = #large_data,
    estimated_size = #large_data * 150
})

-- Process the data
local processed = 0
for i = 1, #large_data do
    if large_data[i].id % 2 == 0 then
        processed = processed + 1
    end
end

Debug.recordEvent(memory_timer.id, "processing_complete", {
    items_processed = processed,
    processing_rate = processed / #large_data
})

-- Cleanup
large_data = nil
collectgarbage("collect")

Debug.recordEvent(memory_timer.id, "cleanup_complete", {
    garbage_collected = true
})

local memory_time = memory_timer:stop()

-- 3. Advanced Profiling Reports
print("\n📈 Advanced Profiling Reports")
print("-" .. string.rep("-", 35))

-- Generate detailed performance report
print("Detailed Performance Report:")
print(Debug.performanceReport())

-- Generate memory snapshot
print("\nMemory Snapshot:")
local snapshot = Debug.memorySnapshot()
print("Active trackers: " .. snapshot.active_trackers)
print("Timestamp: " .. snapshot.timestamp_secs)

-- Generate flame graph data for external analysis
print("\nFlame Graph Data (for external tools):")
local flame_data = Debug.flameGraph()
print(flame_data)

-- 4. Performance Analysis Helper
print("\n🔍 Performance Analysis")
print("-" .. string.rep("-", 30))

-- Function to analyze a piece of code
local function benchmark_operation(name, operation, iterations)
    iterations = iterations or 1
    local timer = Debug.timer(name)
    
    for i = 1, iterations do
        timer:lap("iteration_" .. i)
        operation()
    end
    
    local duration = timer:stop()
    local avg_time = duration / iterations
    
    Debug.info(string.format("%s: %.2fms total, %.2fms avg (%d iterations)", 
               name, duration, avg_time, iterations), "benchmark")
    
    return duration, avg_time
end

-- Benchmark different operations
benchmark_operation("string_concat", function()
    local result = ""
    for i = 1, 1000 do
        result = result .. "test"
    end
end, 5)

benchmark_operation("table_insert", function()
    local result = {}
    for i = 1, 1000 do
        table.insert(result, "item_" .. i)
    end
end, 5)

benchmark_operation("math_operations", function()
    local result = 0
    for i = 1, 1000 do
        result = result + math.sqrt(i) * math.sin(i)
    end
end, 5)

-- 5. Memory Stats Analysis
print("\n📊 Memory Statistics")
print("-" .. string.rep("-", 25))

local memory_stats = Debug.memoryStats()
print("Memory Usage:")
print("  Used: " .. memory_stats.used_bytes .. " bytes")
print("  Allocated: " .. memory_stats.allocated_bytes .. " bytes")
print("  Resident: " .. memory_stats.resident_bytes .. " bytes")
print("  GC Collections: " .. memory_stats.collections)

-- 6. Export Performance Data
print("\n📤 Export Performance Data")
print("-" .. string.rep("-", 30))

-- Generate JSON report for external analysis
local json_report = Debug.jsonReport()
print("JSON Report generated (" .. string.len(json_report) .. " characters)")

-- Sample of the JSON data
print("Sample JSON data (first 200 chars):")
print(string.sub(json_report, 1, 200) .. "...")

print("\n✅ Performance profiling example complete!")
print("🔧 Tip: Use the flame graph data with external tools like speedscope.app")