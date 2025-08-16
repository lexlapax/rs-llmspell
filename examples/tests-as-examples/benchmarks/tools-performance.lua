-- Example: Tools Performance Benchmarks
-- Purpose: Tool performance benchmarking measuring initialization time, execution time, and resource usage
-- Prerequisites: None (tools work locally)
-- Expected Output: Performance metrics for all tool categories
-- Version: 0.7.0
-- Tags: test, benchmark, tools, performance

-- ABOUTME: Tool performance benchmarking
-- ABOUTME: Measures initialization time, execution time, and resource usage

print("⚡ Tool Performance Benchmarks")
print("================================")
print()

-- Helper function to execute tool using synchronous API
local function use_tool(tool_name, params)
    local result = Tool.invoke(tool_name, params)
    
    -- Tool.invoke now returns structured results directly (no JSON parsing needed)
    if result then
        return result
    end
    
    -- Return error result if no result
    return {success = false, error = "Tool returned no result"}
end

-- Performance measurement helpers
local function measure_time(fn)
    local start = os.clock()
    local result = fn()
    local elapsed = os.clock() - start
    return elapsed * 1000, result  -- Convert to milliseconds
end

local function benchmark_tool(tool_name, params, iterations)
    iterations = iterations or 100
    
    print(string.format("Benchmarking %s (%d iterations):", tool_name, iterations))
    
    -- Warm up (5 iterations)
    for i = 1, 5 do
        use_tool(tool_name, params)
    end
    
    -- Actual benchmark
    local times = {}
    local successes = 0
    
    for i = 1, iterations do
        local time, result = measure_time(function()
            return use_tool(tool_name, params)
        end)
        
        table.insert(times, time)
        if result.success ~= false then
            successes = successes + 1
        end
    end
    
    -- Calculate statistics
    table.sort(times)
    local total = 0
    for _, time in ipairs(times) do
        total = total + time
    end
    
    local avg = total / iterations
    local min_time = times[1]
    local max_time = times[#times]
    local p50 = times[math.floor(iterations * 0.5)]
    local p95 = times[math.floor(iterations * 0.95)]
    local success_rate = (successes / iterations) * 100
    
    print(string.format("  Average: %.3fms | Min: %.3fms | Max: %.3fms", avg, min_time, max_time))
    print(string.format("  P50: %.3fms | P95: %.3fms | Success: %.1f%%", p50, p95, success_rate))
    
    return {
        tool = tool_name,
        avg = avg,
        min = min_time,
        max = max_time,
        p50 = p50,
        p95 = p95,
        success_rate = success_rate,
        iterations = iterations
    }
end

local results = {}

-- 1. Utility Tools Performance
print("📊 1. UTILITY TOOLS PERFORMANCE")
print("================================")

-- UUID Generator
table.insert(results, benchmark_tool("uuid_generator", {
    operation = "generate",
    version = "v4",
    format = "hyphenated"
}))

-- Base64 Encoder
table.insert(results, benchmark_tool("base64_encoder", {
    operation = "encode",
    input = "Hello, World! This is a test string for encoding."
}))

-- Hash Calculator
table.insert(results, benchmark_tool("hash_calculator", {
    operation = "hash",
    algorithm = "sha256",
    input = "Performance test string for hashing"
}))

-- Text Manipulator
table.insert(results, benchmark_tool("text_manipulator", {
    operation = "uppercase",
    input = "performance test string manipulation"
}))

-- Calculator
table.insert(results, benchmark_tool("calculator", {
    operation = "evaluate",
    input = "2 + 3 * 4 - 1 + sqrt(25)"
}))

print("\n📁 2. FILE SYSTEM TOOLS PERFORMANCE")
print("=====================================")

-- File Operations (write performance)
table.insert(results, benchmark_tool("file_operations", {
    operation = "write",
    path = "/tmp/perf_test.txt",
    input = "Performance test file content for benchmarking file operations."
}, 50))  -- Fewer iterations for file operations

-- File Operations (read performance)
table.insert(results, benchmark_tool("file_operations", {
    operation = "read",
    path = "/tmp/perf_test.txt"
}, 50))

-- File Search
table.insert(results, benchmark_tool("file_search", {
    operation = "search",
    path = "/tmp",
    pattern = "Performance",
    extensions = {"txt"},
    max_depth = 1
}, 20))  -- Even fewer for search operations

print("\n🌐 3. WEB TOOLS PERFORMANCE")
print("============================")

-- URL Validator
table.insert(results, benchmark_tool("url_validator", {
    operation = "validate",
    input = "https://www.example.com/api/v1/test?param=value"
}))

-- API Tester (with very short timeout)
table.insert(results, benchmark_tool("api_tester", {
    operation = "test",
    input = "https://httpbin.org/get",
    method = "GET",
    timeout_ms = 1000
}, 10))  -- Very few iterations for network operations

print("\n📊 4. DATA PROCESSING TOOLS PERFORMANCE")
print("=========================================")

-- JSON Processor
local test_json = '{"name": "Performance Test", "data": [1,2,3,4,5], "nested": {"key": "value"}}'
table.insert(results, benchmark_tool("json_processor", {
    operation = "parse",
    input = test_json
}))

-- Data Validation
table.insert(results, benchmark_tool("data_validation", {
    input = {
        name = "Test User",
        email = "test@example.com",
        age = 30
    },
    rules = {
        rules = {
            {type = "required"},
            {type = "type", expected = "object"}
        }
    }
}))

-- Template Engine
table.insert(results, benchmark_tool("template_engine", {
    input = "Hello, {{name}}! Your score is {{score}}.",
    context = {
        name = "Performance Tester",
        score = 95
    },
    engine = "handlebars"
}))

print("\n🧮 5. MATHEMATICAL OPERATIONS PERFORMANCE")
print("==========================================")

-- Date Time Handler
table.insert(results, benchmark_tool("date_time_handler", {
    operation = "now"
}))

-- Diff Calculator
table.insert(results, benchmark_tool("diff_calculator", {
    old_text = "The quick brown fox jumps over the lazy dog",
    new_text = "The quick brown fox jumps over the lazy cat",
    format = "unified"
}))

-- Performance Analysis
print("\n📈 PERFORMANCE ANALYSIS")
print("=======================")

-- Sort by average time
table.sort(results, function(a, b) return a.avg < b.avg end)

print("Fastest Tools (by average execution time):")
for i = 1, math.min(5, #results) do
    local r = results[i]
    print(string.format("  %d. %s: %.3fms avg (P95: %.3fms)", i, r.tool, r.avg, r.p95))
end

print("\nSlowest Tools:")
for i = math.max(1, #results - 4), #results do
    local r = results[i]
    print(string.format("  %d. %s: %.3fms avg (P95: %.3fms)", #results - i + 1, r.tool, r.avg, r.p95))
end

-- Calculate overall statistics
local total_avg = 0
local total_p95 = 0
for _, r in ipairs(results) do
    total_avg = total_avg + r.avg
    total_p95 = total_p95 + r.p95
end

local overall_avg = total_avg / #results
local overall_p95 = total_p95 / #results

print(string.format("\nOverall Performance:"))
print(string.format("  Tools benchmarked: %d", #results))
print(string.format("  Average execution time: %.3fms", overall_avg))
print(string.format("  Average P95 time: %.3fms", overall_p95))

-- Performance categories
print("\nPerformance Categories:")
print("  Fast (<1ms avg): " .. tostring(#(function()
    local fast = {}
    for _, r in ipairs(results) do
        if r.avg < 1 then table.insert(fast, r) end
    end
    return fast
end)()) .. " tools")

print("  Medium (1-10ms avg): " .. tostring(#(function()
    local medium = {}
    for _, r in ipairs(results) do
        if r.avg >= 1 and r.avg < 10 then table.insert(medium, r) end
    end
    return medium
end)()) .. " tools")

print("  Slow (>10ms avg): " .. tostring(#(function()
    local slow = {}
    for _, r in ipairs(results) do
        if r.avg >= 10 then table.insert(slow, r) end
    end
    return slow
end)()) .. " tools")

-- Cleanup
print("\n🧹 Cleanup:")
local cleanup_result = use_tool("file_operations", {
    operation = "delete",
    path = "/tmp/perf_test.txt"
})

if cleanup_result.success ~= false then
    print("  ✅ Test files cleaned up")
else
    print("  ⚠️ Cleanup warning: " .. (cleanup_result.error or "Unknown issue"))
end

print("\n✅ Tool performance benchmarking complete!")
print("Use these metrics to identify optimization opportunities")

-- Return summary
return {
    tools_benchmarked = #results,
    overall_avg_time = overall_avg,
    overall_p95_time = overall_p95,
    fastest_tool = results[1].tool,
    slowest_tool = results[#results].tool,
    results = results
}