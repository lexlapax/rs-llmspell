-- Cookbook: Timeout Patterns - Handle Slow Operations
-- Purpose: Implement timeout patterns for handling slow operations and preventing hangs
-- Prerequisites: Tools optional for real operations
-- Expected Output: Demonstration of timeout handling patterns
-- Version: 0.7.0
-- Tags: cookbook, timeout, async, slow-operations, resilience

print("=== Timeout Patterns ===\n")

-- ============================================================
-- Pattern 1: Simple Timeout with pcall
-- ============================================================

print("1. Simple Timeout Implementation")
print("-" .. string.rep("-", 40))

local function with_timeout(operation, timeout_seconds, operation_name)
    operation_name = operation_name or "operation"
    local start_time = os.time()
    
    -- Simple timeout simulation (in production, use proper async/threading)
    local function check_timeout()
        return (os.time() - start_time) >= timeout_seconds
    end
    
    local success, result = pcall(function()
        -- Simulate checking timeout during operation
        local iterations = 0
        while iterations < 10 do
            if check_timeout() then
                error("Operation timed out after " .. timeout_seconds .. " seconds")
            end
            
            -- Simulate work with progress callback
            iterations = iterations + 1
            if iterations % 3 == 0 then
                print(string.format("   %s progress: %d/10", operation_name, iterations))
            end
            
            -- Simulate actual operation
            local result = operation(iterations)
            if result then
                return result
            end
            
            -- Small delay to simulate work
            local delay_end = os.time() + 0.1
            while os.time() < delay_end do end
        end
        
        return "Operation completed successfully"
    end)
    
    if success then
        local elapsed = os.time() - start_time
        return {
            success = true,
            result = result,
            elapsed = elapsed
        }
    else
        return {
            success = false,
            error = result,
            elapsed = os.time() - start_time
        }
    end
end

-- Test simple timeout
print("   Testing fast operation (should succeed):")
local fast_op = function(iteration)
    if iteration >= 5 then return "Fast operation done" end
    return nil
end

local result1 = with_timeout(fast_op, 3, "fast_operation")
print(string.format("   Result: %s (elapsed: %ds)", 
    result1.success and result1.result or result1.error, result1.elapsed))

print("\n   Testing slow operation (should timeout):")
local slow_op = function(iteration)
    -- Never completes within timeout
    return nil
end

local result2 = with_timeout(slow_op, 2, "slow_operation")
print(string.format("   Result: %s (elapsed: %ds)", 
    result2.success and result2.result or result2.error, result2.elapsed))

print()

-- ============================================================
-- Pattern 2: Hierarchical Timeouts
-- ============================================================

print("2. Hierarchical Timeout Management")
print("-" .. string.rep("-", 40))

local TimeoutManager = {}
TimeoutManager.__index = TimeoutManager

function TimeoutManager:new(default_timeout)
    return setmetatable({
        default_timeout = default_timeout or 30,
        timeout_stack = {},
        operation_timeouts = {}
    }, self)
end

function TimeoutManager:push_timeout(name, timeout)
    table.insert(self.timeout_stack, {
        name = name,
        timeout = timeout,
        start_time = os.time()
    })
    print(string.format("   ⏰ Started timeout context: %s (%ds)", name, timeout))
end

function TimeoutManager:pop_timeout()
    local context = table.remove(self.timeout_stack)
    if context then
        local elapsed = os.time() - context.start_time
        print(string.format("   ✅ Completed timeout context: %s (%ds elapsed)", 
            context.name, elapsed))
    end
    return context
end

function TimeoutManager:check_timeout()
    local current_time = os.time()
    
    for i, context in ipairs(self.timeout_stack) do
        local elapsed = current_time - context.start_time
        if elapsed >= context.timeout then
            error(string.format("Timeout in context '%s' after %d seconds", 
                context.name, context.timeout))
        end
    end
    
    return true
end

function TimeoutManager:execute_with_timeout(name, timeout, operation)
    self:push_timeout(name, timeout)
    
    local success, result = pcall(function()
        local iterations = 0
        while iterations < 8 do
            self:check_timeout()
            iterations = iterations + 1
            
            local op_result = operation(iterations)
            if op_result then
                return op_result
            end
            
            -- Simulate work delay
            local delay_end = os.time() + 0.2
            while os.time() < delay_end do end
        end
        return "Operation completed"
    end)
    
    self:pop_timeout()
    
    return {
        success = success,
        result = success and result or nil,
        error = not success and result or nil
    }
end

-- Test hierarchical timeouts
local timeout_mgr = TimeoutManager:new()

print("   Testing nested timeout operations:")

local result3 = timeout_mgr:execute_with_timeout("outer_operation", 3, function(iter)
    if iter >= 3 then
        -- Nested operation with shorter timeout
        local nested_result = timeout_mgr:execute_with_timeout("inner_operation", 1, function(inner_iter)
            if inner_iter >= 2 then return "Inner completed" end
            return nil
        end)
        
        if nested_result.success then
            return "Outer completed with: " .. nested_result.result
        else
            return "Outer completed despite inner timeout"
        end
    end
    return nil
end)

print(string.format("   Final result: %s", 
    result3.success and result3.result or result3.error))

print()

-- ============================================================
-- Pattern 3: Adaptive Timeouts
-- ============================================================

print("3. Adaptive Timeout Management")
print("-" .. string.rep("-", 40))

local AdaptiveTimeout = {}
AdaptiveTimeout.__index = AdaptiveTimeout

function AdaptiveTimeout:new()
    return setmetatable({
        operation_history = {},
        default_timeout = 5,
        timeout_multiplier = 1.5,
        min_timeout = 1,
        max_timeout = 30
    }, self)
end

function AdaptiveTimeout:record_operation(operation_name, duration, success)
    if not self.operation_history[operation_name] then
        self.operation_history[operation_name] = {
            attempts = 0,
            total_duration = 0,
            successes = 0,
            failures = 0
        }
    end
    
    local history = self.operation_history[operation_name]
    history.attempts = history.attempts + 1
    history.total_duration = history.total_duration + duration
    
    if success then
        history.successes = history.successes + 1
    else
        history.failures = history.failures + 1
    end
end

function AdaptiveTimeout:calculate_timeout(operation_name)
    local history = self.operation_history[operation_name]
    
    if not history or history.attempts < 3 then
        return self.default_timeout
    end
    
    local avg_duration = history.total_duration / history.attempts
    local success_rate = history.successes / history.attempts
    
    -- Base timeout on average duration
    local calculated_timeout = avg_duration * self.timeout_multiplier
    
    -- Adjust based on success rate
    if success_rate < 0.5 then
        calculated_timeout = calculated_timeout * 2  -- Double timeout for failing operations
    elseif success_rate > 0.9 then
        calculated_timeout = calculated_timeout * 0.8  -- Reduce timeout for reliable operations
    end
    
    -- Clamp to min/max bounds
    calculated_timeout = math.max(self.min_timeout, 
        math.min(self.max_timeout, calculated_timeout))
    
    return math.floor(calculated_timeout)
end

function AdaptiveTimeout:execute_with_adaptive_timeout(operation_name, operation)
    local timeout = self:calculate_timeout(operation_name)
    local start_time = os.time()
    
    print(string.format("   %s: Using %ds timeout (adaptive)", operation_name, timeout))
    
    local success, result = pcall(function()
        local iterations = 0
        while iterations < 6 do
            if (os.time() - start_time) >= timeout then
                error("Adaptive timeout exceeded")
            end
            
            iterations = iterations + 1
            local op_result = operation(iterations)
            if op_result then
                return op_result
            end
            
            -- Variable delay to simulate different operation speeds
            local delay = (operation_name == "slow_db_query") and 0.4 or 0.1
            local delay_end = os.time() + delay
            while os.time() < delay_end do end
        end
        return "Adaptive operation completed"
    end)
    
    local duration = os.time() - start_time
    self:record_operation(operation_name, duration, success)
    
    return {
        success = success,
        result = success and result or nil,
        error = not success and result or nil,
        duration = duration,
        timeout_used = timeout
    }
end

-- Test adaptive timeouts
local adaptive = AdaptiveTimeout:new()

local operations = {
    {"fast_api_call", function(iter) return iter >= 2 and "API response" or nil end},
    {"slow_db_query", function(iter) return iter >= 4 and "DB result" or nil end},
    {"unreliable_service", function(iter) return math.random() > 0.7 and "Service response" or nil end}
}

print("   Testing adaptive timeouts (multiple runs):")

for run = 1, 3 do
    print(string.format("\n   Run %d:", run))
    for _, op_config in ipairs(operations) do
        local op_name, op_func = op_config[1], op_config[2]
        local result = adaptive:execute_with_adaptive_timeout(op_name, op_func)
        
        local status = result.success and "✅" or "❌"
        print(string.format("     %s %s: %ds (timeout: %ds)", 
            status, op_name, result.duration, result.timeout_used))
    end
end

print()

-- ============================================================
-- Pattern 4: Timeout with Cleanup
-- ============================================================

print("4. Timeout with Resource Cleanup")
print("-" .. string.rep("-", 40))

local ResourceManager = {}
ResourceManager.__index = ResourceManager

function ResourceManager:new()
    return setmetatable({
        allocated_resources = {},
        cleanup_handlers = {}
    }, self)
end

function ResourceManager:allocate_resource(name, cleanup_fn)
    self.allocated_resources[name] = {
        name = name,
        allocated_at = os.time(),
        cleanup = cleanup_fn
    }
    print(string.format("   📦 Allocated resource: %s", name))
end

function ResourceManager:cleanup_resource(name)
    local resource = self.allocated_resources[name]
    if resource and resource.cleanup then
        resource.cleanup()
        self.allocated_resources[name] = nil
        print(string.format("   🧹 Cleaned up resource: %s", name))
    end
end

function ResourceManager:cleanup_all()
    for name, resource in pairs(self.allocated_resources) do
        self:cleanup_resource(name)
    end
end

function ResourceManager:execute_with_cleanup(operation_name, timeout, operation)
    local start_time = os.time()
    local success = false
    local result = nil
    
    local cleanup_success, cleanup_error = pcall(function()
        success, result = pcall(function()
            local iterations = 0
            while iterations < 5 do
                if (os.time() - start_time) >= timeout then
                    error("Operation timed out")
                end
                
                iterations = iterations + 1
                
                -- Simulate resource allocation during operation
                if iterations == 2 then
                    self:allocate_resource("temp_file", function()
                        print("     Removing temporary file")
                    end)
                end
                
                if iterations == 3 then
                    self:allocate_resource("network_connection", function()
                        print("     Closing network connection")
                    end)
                end
                
                local op_result = operation(iterations)
                if op_result then
                    return op_result
                end
                
                local delay_end = os.time() + 0.3
                while os.time() < delay_end do end
            end
            return "Operation with cleanup completed"
        end)
        
        -- Always cleanup, regardless of success/failure
        self:cleanup_all()
    end)
    
    if not cleanup_success then
        print(string.format("   ⚠️  Cleanup failed: %s", cleanup_error))
    end
    
    return {
        success = success,
        result = success and result or nil,
        error = not success and result or nil,
        duration = os.time() - start_time
    }
end

-- Test timeout with cleanup
local resource_mgr = ResourceManager:new()

print("   Testing timeout with resource cleanup:")

local result4 = resource_mgr:execute_with_cleanup("cleanup_operation", 2, function(iter)
    if iter >= 4 then return "Success with resources" end
    return nil
end)

print(string.format("   Result: %s (duration: %ds)", 
    result4.success and result4.result or result4.error, result4.duration))

print()
print("🎯 Key Takeaways:")
print("   • Always implement timeout handling for external calls")
print("   • Use hierarchical timeouts for nested operations")
print("   • Adapt timeouts based on historical performance")
print("   • Always cleanup resources on timeout")
print("   • Provide meaningful timeout error messages")
print("   • Consider user experience when setting timeouts")