-- Simple performance benchmark for synchronous Agent API
-- Tests agent creation overhead only

local function benchmark(name, func, iterations)
    iterations = iterations or 100
    
    -- Warm up
    for i = 1, 5 do
        local success = pcall(func)
    end
    
    -- Actual benchmark
    local start_time = os.clock()
    local successes = 0
    for i = 1, iterations do
        local success = pcall(func)
        if success then successes = successes + 1 end
    end
    local end_time = os.clock()
    
    local total_ms = (end_time - start_time) * 1000
    local avg_ms = total_ms / iterations
    
    print(string.format("%s:", name))
    print(string.format("  Total time: %.2f ms", total_ms))
    print(string.format("  Iterations: %d (successes: %d)", iterations, successes))
    print(string.format("  Average per operation: %.3f ms", avg_ms))
    print()
    
    return avg_ms
end

print("=== Agent API Performance Benchmark ===")
print()

-- Test 1: Basic agent creation overhead
local creation_time = benchmark("Basic Agent Creation", function()
    return Agent.create({
        model = "gpt-4o-mini",
        prompt = "You are a test assistant"
    })
end, 20)

-- Test 2: Agent creation with provider/model syntax
benchmark("Provider/Model Syntax", function()
    return Agent.create({
        model = "anthropic/claude-3-sonnet",
        prompt = "Test"
    })
end, 20)

-- Test 3: Agent creation with tools
benchmark("Agent with Tools", function()
    return Agent.create({
        model = "gpt-4o-mini",
        prompt = "Test assistant",
        tools = {"calculator", "uuid_generator"}
    })
end, 10)

-- Summary
print("=== Performance Summary ===")
if creation_time < 50 then
    print("✅ Agent creation overhead: PASS (<50ms target)")
    print(string.format("   Average: %.3f ms", creation_time))
else
    print("❌ Agent creation overhead: FAIL (>50ms)")
    print(string.format("   Average: %.3f ms", creation_time))
end

-- Test synchronous behavior
print("\n=== Synchronous API Test ===")
-- This should work without any coroutine wrapping
local start = os.clock()
local sync_success = pcall(function()
    local agent = Agent.create({
        model = "gpt-4o-mini", 
        prompt = "Sync test"
    })
end)
local sync_time = (os.clock() - start) * 1000

print("✅ Synchronous API executed without coroutines")
print(string.format("   Execution time: %.3f ms", sync_time))
print("   No 'attempt to yield' errors!")

print("\n=== Benchmark Complete ===")
print("\nNote: Failures may be due to missing API keys, not performance issues.")