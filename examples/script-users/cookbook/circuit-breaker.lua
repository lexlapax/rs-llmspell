-- Cookbook: Circuit Breaker Pattern - Prevent Cascade Failures
-- Purpose: Implement circuit breaker to protect against cascading failures
-- Prerequisites: None
-- Expected Output: Circuit breaker demonstration with state transitions
-- Version: 0.7.0
-- Tags: cookbook, circuit-breaker, resilience, fault-tolerance, production

print("=== Circuit Breaker Pattern ===\n")

-- ============================================================
-- Complete Circuit Breaker Implementation
-- ============================================================

local CircuitBreaker = {}
CircuitBreaker.__index = CircuitBreaker

-- Circuit states
local States = {
    CLOSED = "CLOSED",       -- Normal operation
    OPEN = "OPEN",          -- Failing, reject calls
    HALF_OPEN = "HALF_OPEN" -- Testing if service recovered
}

function CircuitBreaker:new(name, options)
    options = options or {}
    
    local breaker = {
        -- Configuration
        name = name,
        failure_threshold = options.failure_threshold or 5,
        success_threshold = options.success_threshold or 2,
        timeout = options.timeout or 60,  -- seconds
        half_open_max_calls = options.half_open_max_calls or 3,
        
        -- State
        state = States.CLOSED,
        failure_count = 0,
        success_count = 0,
        half_open_calls = 0,
        last_failure_time = nil,
        last_state_change = os.time(),
        
        -- Statistics
        stats = {
            total_calls = 0,
            successful_calls = 0,
            failed_calls = 0,
            rejected_calls = 0,
            state_changes = {},
            error_types = {},
            response_times = {}
        },
        
        -- Callbacks
        on_state_change = options.on_state_change,
        on_reject = options.on_reject,
        fallback = options.fallback
    }
    
    return setmetatable(breaker, self)
end

function CircuitBreaker:record_response_time(duration)
    table.insert(self.stats.response_times, duration)
    -- Keep only last 100 response times
    if #self.stats.response_times > 100 then
        table.remove(self.stats.response_times, 1)
    end
end

function CircuitBreaker:get_average_response_time()
    if #self.stats.response_times == 0 then
        return 0
    end
    
    local sum = 0
    for _, time in ipairs(self.stats.response_times) do
        sum = sum + time
    end
    return sum / #self.stats.response_times
end

function CircuitBreaker:change_state(new_state)
    local old_state = self.state
    self.state = new_state
    self.last_state_change = os.time()
    
    -- Record state change
    table.insert(self.stats.state_changes, {
        from = old_state,
        to = new_state,
        timestamp = os.time(),
        reason = self:get_state_change_reason(old_state, new_state)
    })
    
    -- Reset counters based on new state
    if new_state == States.CLOSED then
        self.failure_count = 0
        self.half_open_calls = 0
    elseif new_state == States.HALF_OPEN then
        self.success_count = 0
        self.half_open_calls = 0
    end
    
    -- Callback
    if self.on_state_change then
        self.on_state_change(self.name, old_state, new_state)
    end
    
    print(string.format("   [%s] State: %s → %s", self.name, old_state, new_state))
end

function CircuitBreaker:get_state_change_reason(old_state, new_state)
    if old_state == States.CLOSED and new_state == States.OPEN then
        return "Failure threshold exceeded"
    elseif old_state == States.OPEN and new_state == States.HALF_OPEN then
        return "Timeout expired, testing recovery"
    elseif old_state == States.HALF_OPEN and new_state == States.CLOSED then
        return "Success threshold met, service recovered"
    elseif old_state == States.HALF_OPEN and new_state == States.OPEN then
        return "Test failed, service still unhealthy"
    else
        return "Unknown transition"
    end
end

function CircuitBreaker:should_attempt_reset()
    return self.state == States.OPEN and 
           self.last_failure_time and
           (os.time() - self.last_failure_time) >= self.timeout
end

function CircuitBreaker:call(fn, ...)
    self.stats.total_calls = self.stats.total_calls + 1
    
    -- Check if we should move from OPEN to HALF_OPEN
    if self:should_attempt_reset() then
        self:change_state(States.HALF_OPEN)
    end
    
    -- Handle based on current state
    if self.state == States.OPEN then
        self.stats.rejected_calls = self.stats.rejected_calls + 1
        
        if self.on_reject then
            self.on_reject(self.name)
        end
        
        -- Use fallback if available
        if self.fallback then
            return self.fallback(...)
        end
        
        return nil, {
            error = "Circuit breaker is OPEN",
            circuit = self.name,
            state = self.state,
            will_retry_at = self.last_failure_time + self.timeout
        }
    end
    
    -- For HALF_OPEN state, limit concurrent calls
    if self.state == States.HALF_OPEN then
        self.half_open_calls = self.half_open_calls + 1
        if self.half_open_calls > self.half_open_max_calls then
            self.stats.rejected_calls = self.stats.rejected_calls + 1
            return nil, {
                error = "Circuit breaker is testing (HALF_OPEN), too many concurrent calls",
                circuit = self.name,
                state = self.state
            }
        end
    end
    
    -- Execute the function
    local start_time = os.time()
    local success, result = pcall(fn, ...)
    local duration = os.time() - start_time
    
    self:record_response_time(duration)
    
    if success then
        self:on_success()
        return result, nil
    else
        self:on_failure(result)
        return nil, result
    end
end

function CircuitBreaker:on_success()
    self.stats.successful_calls = self.stats.successful_calls + 1
    
    if self.state == States.HALF_OPEN then
        self.success_count = self.success_count + 1
        if self.success_count >= self.success_threshold then
            self:change_state(States.CLOSED)
        end
    elseif self.state == States.CLOSED then
        self.failure_count = 0  -- Reset failure count on success
    end
end

function CircuitBreaker:on_failure(error)
    self.stats.failed_calls = self.stats.failed_calls + 1
    self.last_failure_time = os.time()
    
    -- Track error types
    local error_str = tostring(error)
    self.stats.error_types[error_str] = (self.stats.error_types[error_str] or 0) + 1
    
    if self.state == States.CLOSED then
        self.failure_count = self.failure_count + 1
        if self.failure_count >= self.failure_threshold then
            self:change_state(States.OPEN)
        end
    elseif self.state == States.HALF_OPEN then
        self:change_state(States.OPEN)
    end
end

function CircuitBreaker:get_status()
    return {
        name = self.name,
        state = self.state,
        failure_count = self.failure_count,
        success_count = self.success_count,
        stats = self.stats,
        avg_response_time = self:get_average_response_time(),
        time_until_retry = self:should_attempt_reset() and 0 or 
            (self.last_failure_time and (self.last_failure_time + self.timeout - os.time()) or nil)
    }
end

function CircuitBreaker:reset()
    self.state = States.CLOSED
    self.failure_count = 0
    self.success_count = 0
    self.half_open_calls = 0
    self.last_failure_time = nil
    print(string.format("   [%s] Circuit breaker manually reset", self.name))
end

-- ============================================================
-- Usage Example: API Call Protection
-- ============================================================

print("1. Basic Circuit Breaker Usage")
print("-" .. string.rep("-", 40))

-- Create circuit breaker for API calls
local api_breaker = CircuitBreaker:new("API_SERVICE", {
    failure_threshold = 3,
    success_threshold = 2,
    timeout = 10,
    on_state_change = function(name, old_state, new_state)
        print(string.format("   📊 Circuit '%s' changed: %s → %s", 
            name, old_state, new_state))
    end,
    fallback = function()
        return {cached = true, data = "Fallback data"}
    end
})

-- Simulate API calls
local function simulate_api_call(should_fail)
    if should_fail then
        error("API connection timeout")
    end
    return {status = 200, data = "Fresh data"}
end

-- Test sequence
print("   Testing API circuit breaker:")

-- Successful calls
for i = 1, 2 do
    local result, err = api_breaker:call(simulate_api_call, false)
    print("   Call " .. i .. ": Success")
end

-- Failures to trigger circuit open
for i = 1, 4 do
    local result, err = api_breaker:call(simulate_api_call, true)
    if err then
        print("   Call " .. (i + 2) .. ": " .. (err.error or err))
    end
end

-- Try call with open circuit (should use fallback)
local result, err = api_breaker:call(simulate_api_call, false)
if result and result.cached then
    print("   Call 7: Used fallback data")
elseif err then
    print("   Call 7: " .. err.error)
end

print()

-- ============================================================
-- Pattern: Cascading Circuit Breakers
-- ============================================================

print("2. Cascading Circuit Breakers")
print("-" .. string.rep("-", 40))

local CircuitBreakerChain = {}
CircuitBreakerChain.__index = CircuitBreakerChain

function CircuitBreakerChain:new()
    return setmetatable({
        breakers = {}
    }, self)
end

function CircuitBreakerChain:add(name, breaker)
    self.breakers[name] = breaker
    return self
end

function CircuitBreakerChain:call(service_calls)
    local results = {}
    
    for _, service in ipairs(service_calls) do
        local breaker = self.breakers[service.name]
        if not breaker then
            results[service.name] = {error = "No circuit breaker for service"}
        else
            local result, err = breaker:call(service.fn)
            results[service.name] = result or {error = err}
        end
    end
    
    return results
end

-- Create chain of circuit breakers
local chain = CircuitBreakerChain:new()

chain:add("database", CircuitBreaker:new("DATABASE", {
    failure_threshold = 2,
    timeout = 5
}))

chain:add("cache", CircuitBreaker:new("CACHE", {
    failure_threshold = 5,
    timeout = 3
}))

chain:add("external_api", CircuitBreaker:new("EXTERNAL_API", {
    failure_threshold = 3,
    timeout = 10
}))

print("   Created circuit breaker chain with 3 services")
print()

-- ============================================================
-- Pattern: Health Check Integration
-- ============================================================

print("3. Circuit Breaker with Health Checks")
print("-" .. string.rep("-", 40))

local HealthAwareCircuitBreaker = {}
setmetatable(HealthAwareCircuitBreaker, {__index = CircuitBreaker})
HealthAwareCircuitBreaker.__index = HealthAwareCircuitBreaker

function HealthAwareCircuitBreaker:new(name, options)
    local breaker = CircuitBreaker.new(self, name, options)
    breaker.health_check = options.health_check
    breaker.health_check_interval = options.health_check_interval or 30
    breaker.last_health_check = 0
    return breaker
end

function HealthAwareCircuitBreaker:should_run_health_check()
    return self.state == States.OPEN and
           self.health_check and
           (os.time() - self.last_health_check) >= self.health_check_interval
end

function HealthAwareCircuitBreaker:run_health_check()
    if not self:should_run_health_check() then
        return false
    end
    
    self.last_health_check = os.time()
    print(string.format("   [%s] Running health check...", self.name))
    
    local success, result = pcall(self.health_check)
    
    if success and result then
        print(string.format("   [%s] Health check passed, resetting circuit", self.name))
        self:reset()
        return true
    else
        print(string.format("   [%s] Health check failed", self.name))
        return false
    end
end

-- Example with health check
local service_breaker = HealthAwareCircuitBreaker:new("MONITORED_SERVICE", {
    failure_threshold = 2,
    timeout = 60,
    health_check_interval = 5,
    health_check = function()
        -- Simulate health check
        print("     Pinging service...")
        return math.random() > 0.3  -- 70% chance of success
    end
})

print("   Created health-aware circuit breaker")
service_breaker:run_health_check()

print()

-- ============================================================
-- Pattern: Bulkhead Pattern Integration
-- ============================================================

print("4. Bulkhead Pattern with Circuit Breaker")
print("-" .. string.rep("-", 40))

local Bulkhead = {}
Bulkhead.__index = Bulkhead

function Bulkhead:new(name, max_concurrent)
    return setmetatable({
        name = name,
        max_concurrent = max_concurrent or 10,
        current_calls = 0,
        queue = {},
        breaker = CircuitBreaker:new(name .. "_BREAKER", {
            failure_threshold = 5,
            timeout = 30
        })
    }, self)
end

function Bulkhead:call(fn, ...)
    if self.current_calls >= self.max_concurrent then
        return nil, {
            error = "Bulkhead full",
            current = self.current_calls,
            max = self.max_concurrent
        }
    end
    
    self.current_calls = self.current_calls + 1
    
    -- Use circuit breaker for the actual call
    local result, err = self.breaker:call(fn, ...)
    
    self.current_calls = self.current_calls - 1
    
    return result, err
end

function Bulkhead:get_status()
    return {
        name = self.name,
        current_calls = self.current_calls,
        max_concurrent = self.max_concurrent,
        utilization = (self.current_calls / self.max_concurrent) * 100,
        breaker_state = self.breaker.state
    }
end

-- Example bulkhead
local api_bulkhead = Bulkhead:new("API", 3)

print("   Created bulkhead with max 3 concurrent calls")
print("   Current status:")
local status = api_bulkhead:get_status()
print(string.format("   Utilization: %.1f%% (%d/%d)", 
    status.utilization, status.current_calls, status.max_concurrent))
print(string.format("   Circuit state: %s", status.breaker_state))

print()

-- ============================================================
-- Monitoring and Metrics
-- ============================================================

print("5. Circuit Breaker Monitoring")
print("-" .. string.rep("-", 40))

local function print_breaker_metrics(breaker)
    local status = breaker:get_status()
    
    print(string.format("\n   Circuit: %s", status.name))
    print(string.format("   State: %s", status.state))
    print(string.format("   Total calls: %d", status.stats.total_calls))
    print(string.format("   Success rate: %.1f%%", 
        status.stats.total_calls > 0 and 
        (status.stats.successful_calls / status.stats.total_calls * 100) or 0))
    print(string.format("   Rejected calls: %d", status.stats.rejected_calls))
    print(string.format("   Avg response time: %.2fs", status.avg_response_time))
    
    if status.time_until_retry and status.time_until_retry > 0 then
        print(string.format("   Retry in: %ds", status.time_until_retry))
    end
    
    -- Top errors
    if next(status.stats.error_types) then
        print("   Top errors:")
        for error, count in pairs(status.stats.error_types) do
            print(string.format("     - %s: %d times", error:sub(1, 30), count))
        end
    end
end

-- Print metrics for our test breaker
print_breaker_metrics(api_breaker)

print()
print("🎯 Key Takeaways:")
print("   • Circuit breakers prevent cascade failures")
print("   • Three states: CLOSED, OPEN, HALF_OPEN")
print("   • Combine with bulkhead for isolation")
print("   • Add health checks for faster recovery")
print("   • Monitor metrics for operational insight")