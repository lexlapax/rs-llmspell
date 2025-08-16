-- Cookbook: Performance Testing - Measuring and Optimizing Performance
-- Purpose: Implement patterns for performance testing LLM applications
-- Prerequisites: Performance monitoring tools (optional for enhanced metrics)
-- Expected Output: Demonstration of performance testing patterns
-- Version: 0.7.0
-- Tags: cookbook, performance, testing, benchmarking, optimization

print("=== Performance Testing Patterns ===\n")

-- ============================================================
-- Pattern 1: Latency Measurement
-- ============================================================

print("1. Latency Measurement")
print("-" .. string.rep("-", 40))

local LatencyProfiler = {}
LatencyProfiler.__index = LatencyProfiler

function LatencyProfiler:new()
    return setmetatable({
        measurements = {},
        percentiles = {50, 90, 95, 99},
        buckets = {}  -- For histogram
    }, self)
end

function LatencyProfiler:start_timer()
    return os.clock()
end

function LatencyProfiler:end_timer(start_time, operation)
    local elapsed = (os.clock() - start_time) * 1000  -- Convert to ms
    
    operation = operation or "default"
    
    if not self.measurements[operation] then
        self.measurements[operation] = {}
    end
    
    table.insert(self.measurements[operation], elapsed)
    
    -- Update histogram buckets
    local bucket = self:get_bucket(elapsed)
    self.buckets[bucket] = (self.buckets[bucket] or 0) + 1
    
    return elapsed
end

function LatencyProfiler:get_bucket(latency)
    -- Define bucket boundaries (ms)
    local boundaries = {10, 25, 50, 100, 250, 500, 1000, 2500, 5000}
    
    for _, boundary in ipairs(boundaries) do
        if latency <= boundary then
            return "≤" .. boundary .. "ms"
        end
    end
    
    return ">5000ms"
end

function LatencyProfiler:calculate_percentile(data, percentile)
    if #data == 0 then
        return 0
    end
    
    table.sort(data)
    local index = math.ceil(#data * percentile / 100)
    return data[index]
end

function LatencyProfiler:get_stats(operation)
    local data = self.measurements[operation]
    
    if not data or #data == 0 then
        return nil
    end
    
    -- Calculate statistics
    local sum = 0
    local min = math.huge
    local max = 0
    
    for _, value in ipairs(data) do
        sum = sum + value
        min = math.min(min, value)
        max = math.max(max, value)
    end
    
    local stats = {
        count = #data,
        mean = sum / #data,
        min = min,
        max = max,
        percentiles = {}
    }
    
    -- Calculate percentiles
    for _, p in ipairs(self.percentiles) do
        stats.percentiles["p" .. p] = self:calculate_percentile(data, p)
    end
    
    return stats
end

function LatencyProfiler:print_histogram()
    print("\n   Latency Distribution:")
    
    local total = 0
    for _, count in pairs(self.buckets) do
        total = total + count
    end
    
    local sorted_buckets = {}
    for bucket in pairs(self.buckets) do
        table.insert(sorted_buckets, bucket)
    end
    table.sort(sorted_buckets)
    
    for _, bucket in ipairs(sorted_buckets) do
        local count = self.buckets[bucket]
        local percentage = (count / total) * 100
        local bar = string.rep("█", math.floor(percentage / 2))
        
        print(string.format("   %-10s: %3d (%.1f%%) %s",
            bucket, count, percentage, bar))
    end
end

-- Test latency profiler
local profiler = LatencyProfiler:new()

print("   Simulating operations with varying latency:")

-- Simulate operations with different latencies
for i = 1, 100 do
    local start = profiler:start_timer()
    
    -- Simulate work with random latency
    local work_time = math.random(5, 200) / 1000  -- 5-200ms
    local work_start = os.clock()
    while (os.clock() - work_start) < work_time do
        -- Busy wait
    end
    
    local latency = profiler:end_timer(start, "api_call")
    
    if i % 20 == 0 then
        print(string.format("   Operation %d: %.2fms", i, latency))
    end
end

-- Show statistics
local stats = profiler:get_stats("api_call")
print(string.format("\n   Latency Statistics:"))
print(string.format("   Count: %d", stats.count))
print(string.format("   Mean: %.2fms", stats.mean))
print(string.format("   Min: %.2fms", stats.min))
print(string.format("   Max: %.2fms", stats.max))
print(string.format("   P50: %.2fms", stats.percentiles.p50))
print(string.format("   P90: %.2fms", stats.percentiles.p90))
print(string.format("   P95: %.2fms", stats.percentiles.p95))
print(string.format("   P99: %.2fms", stats.percentiles.p99))

profiler:print_histogram()

print()

-- ============================================================
-- Pattern 2: Throughput Testing
-- ============================================================

print("2. Throughput Testing")
print("-" .. string.rep("-", 40))

local ThroughputTester = {}
ThroughputTester.__index = ThroughputTester

function ThroughputTester:new()
    return setmetatable({
        start_time = nil,
        end_time = nil,
        operations = 0,
        bytes_processed = 0,
        errors = 0,
        concurrent_operations = 0,
        max_concurrent = 0
    }, self)
end

function ThroughputTester:start()
    self.start_time = os.clock()
    self.operations = 0
    self.bytes_processed = 0
    self.errors = 0
    print("   Started throughput test")
end

function ThroughputTester:record_operation(success, bytes)
    if not self.start_time then
        self:start()
    end
    
    self.operations = self.operations + 1
    
    if success then
        self.bytes_processed = self.bytes_processed + (bytes or 0)
    else
        self.errors = self.errors + 1
    end
end

function ThroughputTester:start_concurrent_operation()
    self.concurrent_operations = self.concurrent_operations + 1
    self.max_concurrent = math.max(self.max_concurrent, self.concurrent_operations)
end

function ThroughputTester:end_concurrent_operation()
    self.concurrent_operations = math.max(0, self.concurrent_operations - 1)
end

function ThroughputTester:stop()
    self.end_time = os.clock()
end

function ThroughputTester:get_metrics()
    local elapsed = (self.end_time or os.clock()) - self.start_time
    
    return {
        duration = elapsed,
        total_operations = self.operations,
        successful_operations = self.operations - self.errors,
        error_rate = (self.errors / math.max(1, self.operations)) * 100,
        ops_per_second = self.operations / math.max(0.001, elapsed),
        bytes_per_second = self.bytes_processed / math.max(0.001, elapsed),
        max_concurrent = self.max_concurrent
    }
end

-- Test throughput
local throughput = ThroughputTester:new()
throughput:start()

print("   Simulating high-throughput operations:")

-- Simulate burst of operations
for i = 1, 50 do
    throughput:start_concurrent_operation()
    
    -- Simulate operation
    local success = math.random() > 0.05  -- 95% success rate
    local bytes = math.random(100, 1000)
    
    throughput:record_operation(success, bytes)
    
    -- Simulate some concurrency
    if i % 3 == 0 then
        throughput:end_concurrent_operation()
    end
end

-- Clear remaining concurrent operations
while throughput.concurrent_operations > 0 do
    throughput:end_concurrent_operation()
end

throughput:stop()

-- Show metrics
local metrics = throughput:get_metrics()
print(string.format("\n   Throughput Metrics:"))
print(string.format("   Duration: %.2fs", metrics.duration))
print(string.format("   Total operations: %d", metrics.total_operations))
print(string.format("   Success rate: %.1f%%", 100 - metrics.error_rate))
print(string.format("   Throughput: %.1f ops/sec", metrics.ops_per_second))
print(string.format("   Data rate: %.1f bytes/sec", metrics.bytes_per_second))
print(string.format("   Max concurrent: %d", metrics.max_concurrent))

print()

-- ============================================================
-- Pattern 3: Memory Profiling
-- ============================================================

print("3. Memory Profiling")
print("-" .. string.rep("-", 40))

local MemoryProfiler = {}
MemoryProfiler.__index = MemoryProfiler

function MemoryProfiler:new()
    return setmetatable({
        baseline = 0,
        samples = {},
        allocations = {},
        peak_memory = 0
    }, self)
end

function MemoryProfiler:get_memory_usage()
    -- Simulate memory usage (in real implementation, use collectgarbage("count"))
    return collectgarbage("count") * 1024  -- Convert KB to bytes
end

function MemoryProfiler:set_baseline()
    collectgarbage("collect")
    self.baseline = self:get_memory_usage()
    print(string.format("   Baseline memory: %.2f KB", self.baseline / 1024))
end

function MemoryProfiler:sample(label)
    local current = self:get_memory_usage()
    local delta = current - self.baseline
    
    table.insert(self.samples, {
        label = label,
        memory = current,
        delta = delta,
        timestamp = os.clock()
    })
    
    self.peak_memory = math.max(self.peak_memory, current)
    
    return current, delta
end

function MemoryProfiler:track_allocation(name, size)
    self.allocations[name] = (self.allocations[name] or 0) + size
end

function MemoryProfiler:force_gc()
    collectgarbage("collect")
    return self:get_memory_usage()
end

function MemoryProfiler:get_report()
    local report = {
        baseline = self.baseline,
        current = self:get_memory_usage(),
        peak = self.peak_memory,
        samples = #self.samples,
        allocations = {}
    }
    
    -- Sort allocations by size
    for name, size in pairs(self.allocations) do
        table.insert(report.allocations, {name = name, size = size})
    end
    
    table.sort(report.allocations, function(a, b)
        return a.size > b.size
    end)
    
    return report
end

-- Test memory profiler
local mem_profiler = MemoryProfiler:new()
mem_profiler:set_baseline()

print("\n   Simulating memory allocations:")

-- Simulate memory usage patterns
local data_structures = {}

-- Allocation phase
for i = 1, 5 do
    -- Create some data structures
    local large_table = {}
    for j = 1, 1000 do
        large_table[j] = "Data item " .. j
    end
    
    table.insert(data_structures, large_table)
    mem_profiler:track_allocation("large_table_" .. i, 1000 * 20)  -- Approximate
    
    local current, delta = mem_profiler:sample("After allocation " .. i)
    print(string.format("   Allocation %d: %.2f KB (Δ %.2f KB)",
        i, current / 1024, delta / 1024))
end

-- Cleanup phase
print("\n   Running garbage collection:")
local before_gc = mem_profiler:get_memory_usage()
local after_gc = mem_profiler:force_gc()
print(string.format("   Before GC: %.2f KB", before_gc / 1024))
print(string.format("   After GC: %.2f KB", after_gc / 1024))
print(string.format("   Freed: %.2f KB", (before_gc - after_gc) / 1024))

-- Generate report
local report = mem_profiler:get_report()
print(string.format("\n   Memory Report:"))
print(string.format("   Peak memory: %.2f KB", report.peak / 1024))
print(string.format("   Current memory: %.2f KB", report.current / 1024))

print()

-- ============================================================
-- Pattern 4: Load Testing
-- ============================================================

print("4. Load Testing")
print("-" .. string.rep("-", 40))

local LoadTester = {}
LoadTester.__index = LoadTester

function LoadTester:new()
    return setmetatable({
        scenarios = {},
        results = {},
        limits = {
            max_concurrent = 10,
            timeout = 5000,  -- ms
            error_threshold = 0.1  -- 10%
        }
    }, self)
end

function LoadTester:add_scenario(name, config)
    self.scenarios[name] = {
        name = name,
        users = config.users or 1,
        duration = config.duration or 10,
        ramp_up = config.ramp_up or 0,
        think_time = config.think_time or 1,
        workload = config.workload
    }
    
    print(string.format("   Added scenario: %s (%d users, %ds)",
        name, config.users or 1, config.duration or 10))
end

function LoadTester:run_scenario(name)
    local scenario = self.scenarios[name]
    if not scenario then
        return nil, "Scenario not found"
    end
    
    print(string.format("\n   Running load test: %s", name))
    
    local result = {
        scenario = name,
        start_time = os.clock(),
        requests = 0,
        errors = 0,
        response_times = {},
        active_users = 0
    }
    
    -- Simulate ramp-up
    if scenario.ramp_up > 0 then
        print(string.format("   Ramping up over %ds...", scenario.ramp_up))
    end
    
    -- Simulate load
    local start = os.clock()
    local duration = scenario.duration
    
    while (os.clock() - start) < duration do
        -- Simulate users
        for user = 1, scenario.users do
            -- Check if user should be active (ramp-up)
            local elapsed = os.clock() - start
            local user_start_time = (user - 1) * (scenario.ramp_up / scenario.users)
            
            if elapsed >= user_start_time then
                -- Execute workload
                local request_start = os.clock()
                local success = true
                
                if scenario.workload then
                    success = scenario.workload()
                else
                    -- Default workload simulation
                    success = math.random() > 0.05  -- 95% success
                end
                
                local response_time = (os.clock() - request_start) * 1000
                
                result.requests = result.requests + 1
                if not success then
                    result.errors = result.errors + 1
                end
                
                table.insert(result.response_times, response_time)
                
                -- Think time
                local think_start = os.clock()
                while (os.clock() - think_start) < (scenario.think_time / 1000) do
                    -- Wait
                end
            end
        end
        
        -- Progress update
        if result.requests % 10 == 0 then
            local error_rate = result.errors / math.max(1, result.requests)
            print(string.format("   Progress: %d requests, %.1f%% errors",
                result.requests, error_rate * 100))
            
            -- Circuit breaker
            if error_rate > self.limits.error_threshold then
                print("   ⚠️  Error threshold exceeded, stopping test")
                break
            end
        end
    end
    
    result.end_time = os.clock()
    result.duration = result.end_time - result.start_time
    
    self.results[name] = result
    return result
end

function LoadTester:analyze_results(scenario_name)
    local result = self.results[scenario_name]
    if not result then
        return nil
    end
    
    -- Calculate statistics
    local response_times = result.response_times
    table.sort(response_times)
    
    local sum = 0
    for _, rt in ipairs(response_times) do
        sum = sum + rt
    end
    
    return {
        total_requests = result.requests,
        successful_requests = result.requests - result.errors,
        error_rate = (result.errors / math.max(1, result.requests)) * 100,
        requests_per_second = result.requests / result.duration,
        avg_response_time = #response_times > 0 and (sum / #response_times) or 0,
        min_response_time = response_times[1] or 0,
        max_response_time = response_times[#response_times] or 0,
        p50_response_time = response_times[math.ceil(#response_times * 0.5)] or 0,
        p95_response_time = response_times[math.ceil(#response_times * 0.95)] or 0
    }
end

-- Test load tester
local load_tester = LoadTester:new()

-- Define load scenarios
load_tester:add_scenario("light_load", {
    users = 2,
    duration = 2,
    ramp_up = 1,
    think_time = 0.5,
    workload = function()
        -- Simulate API call
        local latency = math.random(10, 100) / 1000
        local start = os.clock()
        while (os.clock() - start) < latency do
            -- Simulate work
        end
        return math.random() > 0.05  -- 95% success
    end
})

-- Run load test
local result = load_tester:run_scenario("light_load")

-- Analyze results
local analysis = load_tester:analyze_results("light_load")

print(string.format("\n   Load Test Results:"))
print(string.format("   Total requests: %d", analysis.total_requests))
print(string.format("   Success rate: %.1f%%", 100 - analysis.error_rate))
print(string.format("   Throughput: %.1f req/s", analysis.requests_per_second))
print(string.format("   Avg response time: %.1fms", analysis.avg_response_time))
print(string.format("   P95 response time: %.1fms", analysis.p95_response_time))

print()

-- ============================================================
-- Pattern 5: Comparative Benchmarking
-- ============================================================

print("5. Comparative Benchmarking")
print("-" .. string.rep("-", 40))

local Benchmark = {}
Benchmark.__index = Benchmark

function Benchmark:new()
    return setmetatable({
        implementations = {},
        test_cases = {},
        results = {}
    }, self)
end

function Benchmark:add_implementation(name, impl)
    self.implementations[name] = impl
    print(string.format("   Added implementation: %s", name))
end

function Benchmark:add_test_case(name, input, expected)
    table.insert(self.test_cases, {
        name = name,
        input = input,
        expected = expected
    })
end

function Benchmark:run(iterations)
    iterations = iterations or 100
    
    print(string.format("\n   Running benchmark with %d iterations:", iterations))
    
    for impl_name, impl in pairs(self.implementations) do
        print(string.format("\n   Testing: %s", impl_name))
        
        local impl_results = {
            name = impl_name,
            correct = 0,
            total_time = 0,
            min_time = math.huge,
            max_time = 0,
            times = {}
        }
        
        for i = 1, iterations do
            for _, test_case in ipairs(self.test_cases) do
                local start = os.clock()
                local success, result = pcall(impl, test_case.input)
                local elapsed = os.clock() - start
                
                impl_results.total_time = impl_results.total_time + elapsed
                impl_results.min_time = math.min(impl_results.min_time, elapsed)
                impl_results.max_time = math.max(impl_results.max_time, elapsed)
                table.insert(impl_results.times, elapsed)
                
                -- Check correctness
                if success and result == test_case.expected then
                    impl_results.correct = impl_results.correct + 1
                end
            end
        end
        
        self.results[impl_name] = impl_results
    end
    
    self:print_comparison()
end

function Benchmark:print_comparison()
    print("\n   Benchmark Comparison:")
    print("   " .. string.rep("-", 60))
    print(string.format("   %-20s %10s %10s %10s %10s",
        "Implementation", "Avg (ms)", "Min (ms)", "Max (ms)", "Correct"))
    print("   " .. string.rep("-", 60))
    
    local baseline = nil
    
    for name, results in pairs(self.results) do
        local avg_time = (results.total_time / #results.times) * 1000
        
        if not baseline then
            baseline = avg_time
        end
        
        local speedup = baseline / avg_time
        local correct_pct = (results.correct / (#self.test_cases * #results.times)) * 100
        
        print(string.format("   %-20s %10.3f %10.3f %10.3f %9.1f%%",
            name,
            avg_time,
            results.min_time * 1000,
            results.max_time * 1000,
            correct_pct))
        
        if speedup ~= 1.0 then
            print(string.format("   %20s (%.2fx %s)",
                "", 
                speedup,
                speedup > 1 and "faster" or "slower"))
        end
    end
    
    print("   " .. string.rep("-", 60))
end

-- Test comparative benchmark
local benchmark = Benchmark:new()

-- Add different implementations to compare
benchmark:add_implementation("naive", function(n)
    -- Naive fibonacci
    if n <= 1 then return n end
    local a, b = 0, 1
    for i = 2, n do
        a, b = b, a + b
    end
    return b
end)

benchmark:add_implementation("memoized", function(n)
    -- Memoized fibonacci
    local cache = {[0] = 0, [1] = 1}
    local function fib(x)
        if cache[x] then
            return cache[x]
        end
        -- Iterative for memoization
        for i = 2, x do
            if not cache[i] then
                cache[i] = cache[i-1] + cache[i-2]
            end
        end
        return cache[x]
    end
    return fib(n)
end)

-- Add test cases
benchmark:add_test_case("small", 10, 55)
benchmark:add_test_case("medium", 20, 6765)
benchmark:add_test_case("large", 30, 832040)

-- Run benchmark
benchmark:run(50)

print()
print("🎯 Key Takeaways:")
print("   • Measure latency percentiles, not just averages")
print("   • Test throughput under realistic load")
print("   • Monitor memory usage and leaks")
print("   • Simulate production load patterns")
print("   • Compare implementations objectively")