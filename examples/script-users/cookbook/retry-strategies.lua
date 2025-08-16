-- Cookbook: Smart Retry Strategies with Exponential Backoff
-- Purpose: Production-ready retry patterns for handling transient failures
-- Prerequisites: None
-- Expected Output: Demonstration of various retry strategies
-- Version: 0.7.0
-- Tags: cookbook, retry, resilience, exponential-backoff, production

print("=== Retry Strategies Cookbook ===\n")

-- ============================================================
-- Pattern 1: Simple Retry with Fixed Delay
-- ============================================================

print("1. Simple Retry with Fixed Delay")
print("-" .. string.rep("-", 40))

local function retry_simple(fn, max_attempts, delay_seconds)
    max_attempts = max_attempts or 3
    delay_seconds = delay_seconds or 1
    
    local last_error
    
    for attempt = 1, max_attempts do
        print("   Attempt " .. attempt .. "/" .. max_attempts)
        
        local success, result = pcall(fn, attempt)
        
        if success then
            print("   ✅ Success on attempt " .. attempt)
            return result, nil
        end
        
        last_error = result
        print("   ❌ Failed: " .. tostring(result))
        
        if attempt < max_attempts then
            print("   Waiting " .. delay_seconds .. " seconds...")
            -- In production, use proper sleep
            -- os.execute("sleep " .. delay_seconds)
        end
    end
    
    return nil, "All " .. max_attempts .. " attempts failed. Last error: " .. tostring(last_error)
end

-- Test simple retry
local counter = 0
local result, err = retry_simple(function(attempt)
    counter = counter + 1
    if counter < 2 then
        error("Simulated failure")
    end
    return "Success after " .. counter .. " attempts"
end, 3, 1)

print("   Result: " .. tostring(result or err))
print()

-- ============================================================
-- Pattern 2: Exponential Backoff with Jitter
-- ============================================================

print("2. Exponential Backoff with Jitter")
print("-" .. string.rep("-", 40))

local function calculate_backoff(attempt, base_delay, max_delay, jitter)
    -- Calculate exponential delay: base * 2^(attempt-1)
    local delay = base_delay * math.pow(2, attempt - 1)
    
    -- Cap at max_delay
    delay = math.min(delay, max_delay)
    
    -- Add jitter to prevent thundering herd
    if jitter then
        local jitter_amount = delay * 0.1 * (math.random() - 0.5) * 2
        delay = delay + jitter_amount
    end
    
    return delay
end

local function retry_exponential(fn, options)
    options = options or {}
    local max_attempts = options.max_attempts or 5
    local base_delay = options.base_delay or 1
    local max_delay = options.max_delay or 60
    local jitter = options.jitter ~= false
    
    local total_wait_time = 0
    local attempts_log = {}
    
    for attempt = 1, max_attempts do
        local start_time = os.time()
        
        local success, result = pcall(fn, attempt)
        
        if success then
            table.insert(attempts_log, {
                attempt = attempt,
                success = true,
                duration = os.time() - start_time
            })
            
            return {
                result = result,
                attempts = attempt,
                total_wait_time = total_wait_time,
                log = attempts_log
            }, nil
        end
        
        table.insert(attempts_log, {
            attempt = attempt,
            success = false,
            error = tostring(result),
            duration = os.time() - start_time
        })
        
        if attempt < max_attempts then
            local delay = calculate_backoff(attempt, base_delay, max_delay, jitter)
            print(string.format("   Attempt %d failed. Waiting %.2f seconds...", attempt, delay))
            total_wait_time = total_wait_time + delay
            -- os.execute("sleep " .. delay)
        end
    end
    
    return nil, {
        error = "Max attempts reached",
        attempts = max_attempts,
        total_wait_time = total_wait_time,
        log = attempts_log
    }
end

-- Test exponential backoff
print("   Testing exponential backoff (base=1s, max=10s):")
for i = 1, 5 do
    local delay = calculate_backoff(i, 1, 10, true)
    print(string.format("   Attempt %d: %.2f seconds", i, delay))
end

print()

-- ============================================================
-- Pattern 3: Conditional Retry Based on Error Type
-- ============================================================

print("3. Conditional Retry Based on Error Type")
print("-" .. string.rep("-", 40))

local RetryableErrors = {
    NETWORK = true,
    TIMEOUT = true,
    RATE_LIMIT = true,
    TEMPORARY = true
}

local NonRetryableErrors = {
    VALIDATION = true,
    PERMISSION = true,
    NOT_FOUND = true,
    INVALID_CREDENTIALS = true
}

local function classify_error(error_msg)
    -- Simple classification based on error message
    if string.find(error_msg, "timeout") or string.find(error_msg, "timed out") then
        return "TIMEOUT"
    elseif string.find(error_msg, "network") or string.find(error_msg, "connection") then
        return "NETWORK"
    elseif string.find(error_msg, "rate limit") or string.find(error_msg, "429") then
        return "RATE_LIMIT"
    elseif string.find(error_msg, "validation") or string.find(error_msg, "invalid") then
        return "VALIDATION"
    elseif string.find(error_msg, "permission") or string.find(error_msg, "403") then
        return "PERMISSION"
    elseif string.find(error_msg, "not found") or string.find(error_msg, "404") then
        return "NOT_FOUND"
    else
        return "UNKNOWN"
    end
end

local function retry_conditional(fn, options)
    options = options or {}
    local max_attempts = options.max_attempts or 3
    local should_retry = options.should_retry or function(error_type, attempt)
        return RetryableErrors[error_type] == true
    end
    
    for attempt = 1, max_attempts do
        local success, result = pcall(fn, attempt)
        
        if success then
            print("   ✅ Success on attempt " .. attempt)
            return result, nil
        end
        
        local error_type = classify_error(tostring(result))
        print("   Error type: " .. error_type .. " - " .. tostring(result))
        
        if not should_retry(error_type, attempt) then
            print("   ⛔ Error type '" .. error_type .. "' is not retryable")
            return nil, {
                error = result,
                error_type = error_type,
                retryable = false,
                attempts = attempt
            }
        end
        
        if attempt < max_attempts then
            print("   🔄 Retrying (error is retryable)...")
        end
    end
    
    return nil, {
        error = "Max attempts reached",
        attempts = max_attempts
    }
end

-- Test conditional retry
local test_errors = {
    "network connection failed",
    "validation error: invalid input",
    "rate limit exceeded",
    "permission denied"
}

for _, error_msg in ipairs(test_errors) do
    print("\n   Testing: " .. error_msg)
    retry_conditional(function()
        error(error_msg)
    end, {max_attempts = 2})
end

print()

-- ============================================================
-- Pattern 4: Circuit Breaker Pattern
-- ============================================================

print("4. Circuit Breaker Pattern")
print("-" .. string.rep("-", 40))

local CircuitBreaker = {}
CircuitBreaker.__index = CircuitBreaker

function CircuitBreaker:new(options)
    options = options or {}
    return setmetatable({
        failure_threshold = options.failure_threshold or 5,
        recovery_timeout = options.recovery_timeout or 60,
        success_threshold = options.success_threshold or 2,
        state = "CLOSED", -- CLOSED, OPEN, HALF_OPEN
        failure_count = 0,
        success_count = 0,
        last_failure_time = nil,
        stats = {
            total_calls = 0,
            total_failures = 0,
            total_successes = 0,
            circuit_opens = 0
        }
    }, self)
end

function CircuitBreaker:call(fn)
    self.stats.total_calls = self.stats.total_calls + 1
    
    -- Check circuit state
    if self.state == "OPEN" then
        -- Check if we should try half-open
        if self.last_failure_time and 
           (os.time() - self.last_failure_time) > self.recovery_timeout then
            print("   Circuit moving to HALF_OPEN state")
            self.state = "HALF_OPEN"
            self.success_count = 0
        else
            return nil, "Circuit breaker is OPEN"
        end
    end
    
    -- Try the function
    local success, result = pcall(fn)
    
    if success then
        self.stats.total_successes = self.stats.total_successes + 1
        
        if self.state == "HALF_OPEN" then
            self.success_count = self.success_count + 1
            if self.success_count >= self.success_threshold then
                print("   Circuit moving to CLOSED state")
                self.state = "CLOSED"
                self.failure_count = 0
            end
        elseif self.state == "CLOSED" then
            self.failure_count = 0
        end
        
        return result, nil
    else
        self.stats.total_failures = self.stats.total_failures + 1
        self.last_failure_time = os.time()
        
        if self.state == "CLOSED" then
            self.failure_count = self.failure_count + 1
            if self.failure_count >= self.failure_threshold then
                print("   Circuit moving to OPEN state")
                self.state = "OPEN"
                self.stats.circuit_opens = self.stats.circuit_opens + 1
            end
        elseif self.state == "HALF_OPEN" then
            print("   Circuit moving back to OPEN state")
            self.state = "OPEN"
            self.failure_count = 0
        end
        
        return nil, result
    end
end

function CircuitBreaker:get_state()
    return {
        state = self.state,
        failure_count = self.failure_count,
        stats = self.stats
    }
end

-- Test circuit breaker
local breaker = CircuitBreaker:new({
    failure_threshold = 3,
    recovery_timeout = 5,
    success_threshold = 2
})

print("   Testing circuit breaker:")

-- Simulate failures to open circuit
for i = 1, 4 do
    local result, err = breaker:call(function()
        error("Service unavailable")
    end)
    local state = breaker:get_state()
    print("   Call " .. i .. ": " .. state.state .. " (failures: " .. state.failure_count .. ")")
end

-- Circuit should be open now
local result, err = breaker:call(function()
    return "This won't execute"
end)
print("   Call with open circuit: " .. tostring(err))

print()

-- ============================================================
-- Pattern 5: Retry with Multiple Strategies
-- ============================================================

print("5. Combined Retry Strategies")
print("-" .. string.rep("-", 40))

local function retry_advanced(fn, strategies)
    local execution_log = {
        strategies_tried = {},
        total_attempts = 0,
        total_time = 0
    }
    
    local start_time = os.time()
    
    for i, strategy in ipairs(strategies) do
        print("   Trying strategy " .. i .. ": " .. strategy.name)
        
        local strategy_result = {
            name = strategy.name,
            attempts = 0
        }
        
        for attempt = 1, strategy.max_attempts do
            execution_log.total_attempts = execution_log.total_attempts + 1
            strategy_result.attempts = strategy_result.attempts + 1
            
            local success, result = pcall(fn, execution_log.total_attempts)
            
            if success then
                strategy_result.success = true
                table.insert(execution_log.strategies_tried, strategy_result)
                execution_log.total_time = os.time() - start_time
                execution_log.result = result
                
                print("   ✅ Success with strategy: " .. strategy.name)
                return execution_log
            end
            
            strategy_result.last_error = tostring(result)
            
            if attempt < strategy.max_attempts then
                local delay = strategy.calculate_delay(attempt)
                print(string.format("     Retry %d/%d in %.2fs", 
                    attempt, strategy.max_attempts, delay))
                -- os.execute("sleep " .. delay)
            end
        end
        
        strategy_result.success = false
        table.insert(execution_log.strategies_tried, strategy_result)
        
        if strategy.on_failure then
            strategy.on_failure()
        end
    end
    
    execution_log.total_time = os.time() - start_time
    execution_log.success = false
    
    return execution_log
end

-- Define retry strategies
local strategies = {
    {
        name = "Immediate retry",
        max_attempts = 2,
        calculate_delay = function(attempt) return 0 end
    },
    {
        name = "Linear backoff",
        max_attempts = 3,
        calculate_delay = function(attempt) return attempt * 0.5 end
    },
    {
        name = "Exponential backoff",
        max_attempts = 3,
        calculate_delay = function(attempt) return math.pow(2, attempt - 1) end,
        on_failure = function()
            print("     Exponential backoff exhausted, escalating...")
        end
    }
}

-- Test combined strategies
local fail_count = 0
local log = retry_advanced(function(total_attempt)
    fail_count = fail_count + 1
    if fail_count < 6 then
        error("Still failing (attempt " .. total_attempt .. ")")
    end
    return "Finally succeeded!"
end, strategies)

print("\n   Execution Summary:")
print("   Total attempts: " .. log.total_attempts)
print("   Strategies tried: " .. #log.strategies_tried)
print("   Success: " .. tostring(log.success or false))

print()
print("🎯 Key Takeaways:")
print("   • Use exponential backoff for API calls")
print("   • Add jitter to prevent thundering herd")
print("   • Don't retry non-retryable errors")
print("   • Circuit breakers prevent cascade failures")
print("   • Combine strategies for complex scenarios")