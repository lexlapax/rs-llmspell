-- ABOUTME: Advanced hook patterns including retry logic, conditional execution, and complex workflows
-- ABOUTME: Demonstrates sophisticated hook coordination, state management, and adaptive behavior patterns

print("=== Advanced Hook Patterns Example ===")
print("Demonstrates: Complex hook patterns, retry logic, conditional execution, and adaptive behavior")
print()

local handles = {}
local pattern_state = {
    retry_attempts = {},
    conditional_state = {},
    circuit_breaker_state = {},
    rate_limiter_state = {},
    adaptive_thresholds = {},
    execution_history = {}
}

-- Helper function to log pattern execution
local function log_pattern_execution(pattern_name, details)
    local entry = {
        timestamp = os.time(),
        pattern = pattern_name,
        details = details or {}
    }
    table.insert(pattern_state.execution_history, entry)
    print(string.format("   📋 [%s] Pattern: %s", 
          os.date("%H:%M:%S", entry.timestamp), pattern_name))
end

print("1. Retry Logic Pattern with Exponential Backoff:")

-- Advanced retry pattern with exponential backoff
handles.retry_pattern = Hook.register("BeforeAgentExecution", function(context)
    local agent_name = context.component_id.name
    local correlation_id = context.correlation_id
    
    -- Initialize retry state if not exists
    if not pattern_state.retry_attempts[correlation_id] then
        pattern_state.retry_attempts[correlation_id] = {
            attempts = 0,
            first_attempt = os.time(),
            last_attempt = 0,
            backoff_delay = 1000 -- Start with 1 second
        }
    end
    
    local retry_info = pattern_state.retry_attempts[correlation_id]
    retry_info.attempts = retry_info.attempts + 1
    retry_info.last_attempt = os.time()
    
    log_pattern_execution("RETRY_LOGIC", {
        agent = agent_name,
        attempt = retry_info.attempts,
        correlation_id = correlation_id
    })
    
    print(string.format("   🔄 Retry attempt #%d for agent: %s", retry_info.attempts, agent_name))
    
    -- Simulate failure conditions for demonstration
    local should_fail = (retry_info.attempts <= 2) and (math.random() < 0.7) -- 70% failure rate for first 2 attempts
    
    if should_fail and retry_info.attempts < 5 then
        print("   ❌ Simulated failure, requesting retry with exponential backoff")
        
        -- Calculate exponential backoff delay
        local delay_ms = retry_info.backoff_delay * math.pow(2, retry_info.attempts - 1)
        delay_ms = math.min(delay_ms, 30000) -- Cap at 30 seconds
        
        retry_info.backoff_delay = delay_ms
        
        print(string.format("   ⏱️  Retry delay: %.1fs (attempt %d/%d)", 
              delay_ms / 1000, retry_info.attempts, 5))
        
        return {
            type = "retry",
            delay_ms = delay_ms,
            max_attempts = 5
        }
    elseif retry_info.attempts >= 5 then
        print("   🚫 Max retry attempts reached, failing permanently")
        
        return {
            type = "cancel",
            reason = string.format("Max retry attempts (%d) exceeded for %s", 5, agent_name)
        }
    else
        print("   ✅ Retry successful after", retry_info.attempts, "attempts")
        
        -- Clean up retry state on success
        pattern_state.retry_attempts[correlation_id] = nil
        
        return "continue"
    end
end, "high")
print("   ✅ Registered exponential backoff retry pattern")

print()
print("2. Circuit Breaker Pattern for Fault Tolerance:")

-- Circuit breaker pattern to prevent cascade failures
handles.circuit_breaker = Hook.register("BeforeToolExecution", function(context)
    local tool_name = context.component_id.name
    
    -- Initialize circuit breaker state
    if not pattern_state.circuit_breaker_state[tool_name] then
        pattern_state.circuit_breaker_state[tool_name] = {
            state = "closed", -- closed, open, half_open
            failure_count = 0,
            last_failure_time = 0,
            success_count = 0,
            threshold = 5, -- Open after 5 failures
            timeout = 30, -- Try again after 30 seconds
            half_open_max_calls = 3
        }
    end
    
    local breaker = pattern_state.circuit_breaker_state[tool_name]
    local current_time = os.time()
    
    log_pattern_execution("CIRCUIT_BREAKER", {
        tool = tool_name,
        state = breaker.state,
        failures = breaker.failure_count
    })
    
    print(string.format("   ⚡ Circuit breaker for %s: %s state", tool_name, breaker.state:upper()))
    
    -- Check circuit breaker state
    if breaker.state == "open" then
        -- Check if timeout period has passed
        if current_time - breaker.last_failure_time >= breaker.timeout then
            breaker.state = "half_open"
            breaker.success_count = 0
            print("   🔄 Circuit breaker transitioning to HALF_OPEN")
        else
            print("   🚫 Circuit breaker OPEN - blocking execution")
            return {
                type = "cancel",
                reason = string.format("Circuit breaker open for %s (failures: %d)", 
                       tool_name, breaker.failure_count)
            }
        end
    end
    
    if breaker.state == "half_open" then
        if breaker.success_count >= breaker.half_open_max_calls then
            print("   🚫 Circuit breaker HALF_OPEN - max test calls reached")
            return {
                type = "cancel",
                reason = "Circuit breaker in half-open state - testing limit reached"
            }
        end
    end
    
    -- Simulate tool execution result
    local execution_success = math.random() > 0.3 -- 70% success rate
    
    if execution_success then
        if breaker.state == "half_open" then
            breaker.success_count = breaker.success_count + 1
            if breaker.success_count >= breaker.half_open_max_calls then
                breaker.state = "closed"
                breaker.failure_count = 0
                print("   ✅ Circuit breaker reset to CLOSED state")
            end
        else
            breaker.failure_count = math.max(0, breaker.failure_count - 1) -- Slowly recover
        end
        
        return "continue"
    else
        -- Handle failure
        breaker.failure_count = breaker.failure_count + 1
        breaker.last_failure_time = current_time
        
        if breaker.failure_count >= breaker.threshold then
            breaker.state = "open"
            print("   ⚠️  Circuit breaker OPENED due to excessive failures")
            
            return {
                type = "cancel",
                reason = string.format("Circuit breaker opened for %s after %d failures", 
                       tool_name, breaker.failure_count)
            }
        else
            print(string.format("   ⚠️  Tool failure recorded (%d/%d)", 
                  breaker.failure_count, breaker.threshold))
            
            return {
                type = "modified",
                data = {
                    circuit_breaker_warning = true,
                    failure_count = breaker.failure_count,
                    threshold = breaker.threshold
                }
            }
        end
    end
end, "high")
print("   ✅ Registered circuit breaker fault tolerance pattern")

print()
print("3. Rate Limiting Pattern with Token Bucket:")

-- Rate limiting using token bucket algorithm
handles.rate_limiter = Hook.register("BeforeAgentExecution", function(context)
    local agent_name = context.component_id.name
    local current_time = os.time()
    
    -- Initialize rate limiter state
    if not pattern_state.rate_limiter_state[agent_name] then
        pattern_state.rate_limiter_state[agent_name] = {
            tokens = 10, -- Start with full bucket
            max_tokens = 10, -- Bucket capacity
            refill_rate = 2, -- Tokens per second
            last_refill = current_time
        }
    end
    
    local limiter = pattern_state.rate_limiter_state[agent_name]
    
    -- Refill tokens based on elapsed time
    local elapsed = current_time - limiter.last_refill
    local tokens_to_add = elapsed * limiter.refill_rate
    limiter.tokens = math.min(limiter.max_tokens, limiter.tokens + tokens_to_add)
    limiter.last_refill = current_time
    
    log_pattern_execution("RATE_LIMITER", {
        agent = agent_name,
        tokens = limiter.tokens,
        elapsed = elapsed
    })
    
    print(string.format("   🪣 Rate limiter for %s: %.1f tokens available", 
          agent_name, limiter.tokens))
    
    if limiter.tokens >= 1 then
        -- Consume a token
        limiter.tokens = limiter.tokens - 1
        print(string.format("   ✅ Token consumed, %.1f tokens remaining", limiter.tokens))
        
        return "continue"
    else
        print("   🚫 Rate limit exceeded - no tokens available")
        
        -- Calculate wait time for next token
        local wait_time = (1 - limiter.tokens) / limiter.refill_rate
        
        return {
            type = "retry",
            delay_ms = math.ceil(wait_time * 1000),
            max_attempts = 3
        }
    end
end, "high")
print("   ✅ Registered token bucket rate limiting pattern")

print()
print("4. Conditional Execution Pattern with State Machine:")

-- Conditional execution based on system state
handles.conditional_execution = Hook.register("BeforeWorkflowStart", function(context)
    local workflow_name = context.component_id.name
    
    -- Initialize conditional state
    if not pattern_state.conditional_state[workflow_name] then
        pattern_state.conditional_state[workflow_name] = {
            state = "ready",
            conditions_met = {},
            prerequisites = {"auth", "resources", "dependencies"},
            last_check = 0
        }
    end
    
    local conditional = pattern_state.conditional_state[workflow_name]
    local current_time = os.time()
    
    log_pattern_execution("CONDITIONAL_EXECUTION", {
        workflow = workflow_name,
        state = conditional.state,
        conditions_met = #conditional.conditions_met
    })
    
    print(string.format("   🎯 Conditional execution for %s: %s state", 
          workflow_name, conditional.state:upper()))
    
    -- Check prerequisites (simulated)
    local prerequisites_met = {}
    for _, prereq in ipairs(conditional.prerequisites) do
        local is_met = math.random() > 0.2 -- 80% chance each prerequisite is met
        prerequisites_met[prereq] = is_met
        
        if is_met then
            print(string.format("   ✅ Prerequisite '%s' satisfied", prereq))
        else
            print(string.format("   ❌ Prerequisite '%s' not satisfied", prereq))
        end
    end
    
    -- Check if all prerequisites are met
    local all_met = true
    for _, is_met in pairs(prerequisites_met) do
        if not is_met then
            all_met = false
            break
        end
    end
    
    if all_met then
        conditional.state = "executing"
        conditional.conditions_met = conditional.prerequisites
        print("   🚀 All conditions met - proceeding with execution")
        
        return {
            type = "modified",
            data = {
                conditional_execution = true,
                all_prerequisites_met = true,
                execution_authorized = true,
                authorization_time = current_time
            }
        }
    else
        print("   ⏳ Prerequisites not met - deferring execution")
        
        return {
            type = "retry",
            delay_ms = 5000, -- Wait 5 seconds and check again
            max_attempts = 10
        }
    end
end, "normal")
print("   ✅ Registered conditional execution state machine pattern")

print()
print("5. Adaptive Threshold Pattern with Machine Learning-like Behavior:")

-- Adaptive thresholds that adjust based on historical performance
handles.adaptive_threshold = Hook.register("AfterAgentExecution", function(context)
    local agent_name = context.component_id.name
    local execution_time = (context.data and context.data.execution_time) or math.random(100, 2000)
    
    -- Initialize adaptive threshold state
    if not pattern_state.adaptive_thresholds[agent_name] then
        pattern_state.adaptive_thresholds[agent_name] = {
            current_threshold = 1000, -- Start with 1 second
            execution_times = {},
            adjustment_factor = 0.1,
            min_threshold = 100,
            max_threshold = 5000,
            sample_size = 10
        }
    end
    
    local adaptive = pattern_state.adaptive_thresholds[agent_name]
    
    -- Record execution time
    table.insert(adaptive.execution_times, execution_time)
    if #adaptive.execution_times > adaptive.sample_size then
        table.remove(adaptive.execution_times, 1) -- Remove oldest sample
    end
    
    log_pattern_execution("ADAPTIVE_THRESHOLD", {
        agent = agent_name,
        execution_time = execution_time,
        current_threshold = adaptive.current_threshold
    })
    
    print(string.format("   📊 Adaptive threshold for %s: execution %.0fms vs threshold %.0fms", 
          agent_name, execution_time, adaptive.current_threshold))
    
    -- Calculate average execution time
    local total_time = 0
    for _, time in ipairs(adaptive.execution_times) do
        total_time = total_time + time
    end
    local average_time = total_time / #adaptive.execution_times
    
    -- Adjust threshold based on recent performance
    local target_threshold = average_time * 1.2 -- 20% buffer above average
    local adjustment = (target_threshold - adaptive.current_threshold) * adaptive.adjustment_factor
    
    adaptive.current_threshold = adaptive.current_threshold + adjustment
    adaptive.current_threshold = math.max(adaptive.min_threshold, 
                                        math.min(adaptive.max_threshold, adaptive.current_threshold))
    
    print(string.format("   🎯 Threshold adjusted: %.0fms → %.0fms (avg: %.0fms)", 
          adaptive.current_threshold - adjustment, adaptive.current_threshold, average_time))
    
    -- Determine if execution was within acceptable bounds
    if execution_time > adaptive.current_threshold then
        print("   ⚠️  Execution exceeded adaptive threshold")
        
        return {
            type = "modified",
            data = {
                performance_warning = true,
                execution_time = execution_time,
                threshold = adaptive.current_threshold,
                threshold_exceeded_by = execution_time - adaptive.current_threshold
            }
        }
    else
        print("   ✅ Execution within adaptive threshold")
        return "continue"
    end
end, "low")
print("   ✅ Registered adaptive threshold machine learning pattern")

print()
print("6. Composite Pattern - Chained Hook Execution:")

-- Composite pattern that chains multiple hook behaviors
handles.composite_pattern = Hook.register("BeforeToolExecution", function(context)
    local tool_name = context.component_id.name
    
    log_pattern_execution("COMPOSITE_PATTERN", {
        tool = tool_name,
        chain_length = 4
    })
    
    print("   🔗 Composite pattern: Chaining multiple behaviors")
    
    -- Chain 1: Authentication check
    print("   1️⃣  Authentication check...")
    local auth_passed = math.random() > 0.1 -- 90% pass rate
    if not auth_passed then
        print("   ❌ Authentication failed")
        return {
            type = "cancel",
            reason = "Authentication check failed in composite pattern"
        }
    end
    print("   ✅ Authentication passed")
    
    -- Chain 2: Resource availability check
    print("   2️⃣  Resource availability check...")
    local resources_available = math.random() > 0.2 -- 80% pass rate
    if not resources_available then
        print("   ⏳ Resources not available, retrying...")
        return {
            type = "retry",
            delay_ms = 2000,
            max_attempts = 3
        }
    end
    print("   ✅ Resources available")
    
    -- Chain 3: Security validation
    print("   3️⃣  Security validation...")
    local security_check = math.random() > 0.15 -- 85% pass rate
    if not security_check then
        print("   🚫 Security validation failed")
        return {
            type = "redirect",
            target = "safe_mode_tool"
        }
    end
    print("   ✅ Security validation passed")
    
    -- Chain 4: Data enhancement
    print("   4️⃣  Data enhancement...")
    local enhanced_data = {
        original_tool = tool_name,
        composite_chain_applied = true,
        security_validated = true,
        resources_checked = true,
        authentication_verified = true,
        enhancement_timestamp = os.time()
    }
    
    print("   ✨ All composite pattern checks passed")
    
    return {
        type = "modified",
        data = enhanced_data
    }
end, "highest")
print("   ✅ Registered composite pattern with chained behaviors")

print()
print("7. Advanced Pattern Statistics Dashboard:")

print("   📊 Advanced Pattern Statistics:")

-- Retry pattern statistics
local retry_stats = 0
for _, retry_info in pairs(pattern_state.retry_attempts) do
    retry_stats = retry_stats + retry_info.attempts
end
print("   • Total retry attempts across all operations:", retry_stats)

-- Circuit breaker statistics
local breaker_stats = {open = 0, closed = 0, half_open = 0}
for tool_name, breaker in pairs(pattern_state.circuit_breaker_state) do
    breaker_stats[breaker.state] = breaker_stats[breaker.state] + 1
    print(string.format("   • Circuit breaker for %s: %s (%d failures)", 
          tool_name, breaker.state:upper(), breaker.failure_count))
end

-- Rate limiter statistics
print("   • Rate limiters active:", (function()
    local count = 0
    for _ in pairs(pattern_state.rate_limiter_state) do count = count + 1 end
    return count
end)())

-- Adaptive threshold statistics
print("   • Adaptive thresholds managed:", (function()
    local count = 0
    for _ in pairs(pattern_state.adaptive_thresholds) do count = count + 1 end
    return count
end)())

print("   • Total pattern executions logged:", #pattern_state.execution_history)

print()
print("8. Pattern execution timeline:")

if #pattern_state.execution_history > 0 then
    print("   📅 Recent Pattern Executions:")
    local recent_start = math.max(1, #pattern_state.execution_history - 7)
    for i = recent_start, #pattern_state.execution_history do
        local entry = pattern_state.execution_history[i]
        print(string.format("   %d. %s at %s", 
              i, entry.pattern, os.date("%H:%M:%S", entry.timestamp)))
    end
end

print()
print("9. Advanced pattern best practices:")

print("   💡 Advanced Pattern Best Practices Demonstrated:")
print("   • Exponential Backoff: Prevent system overload during retries")
print("   • Circuit Breaker: Fail fast to prevent cascade failures")
print("   • Rate Limiting: Control resource consumption with token bucket")
print("   • Conditional Execution: State-driven workflow control")
print("   • Adaptive Thresholds: Machine learning-like performance tuning")
print("   • Composite Patterns: Chain multiple behaviors for complex logic")
print("   • State Management: Persistent state across hook executions")
print("   • Performance Monitoring: Track and optimize pattern performance")

print()
print("10. Cleaning up advanced pattern hooks:")

for name, handle in pairs(handles) do
    Hook.unregister(handle)
    print("   🧹 Unregistered", name, "advanced pattern")
end

-- Clean up pattern state
pattern_state = {
    retry_attempts = {},
    conditional_state = {},
    circuit_breaker_state = {},
    rate_limiter_state = {},
    adaptive_thresholds = {},
    execution_history = {}
}

local final_count = #Hook.list()
print("   ✅ Final hook count:", final_count)
print("   🧹 Pattern state cleaned up")

print()
print("✨ Advanced hook patterns example complete!")
print("   Key concepts demonstrated:")
print("   • Exponential backoff retry logic with failure recovery")
print("   • Circuit breaker pattern for fault tolerance")
print("   • Token bucket rate limiting algorithm")
print("   • Conditional execution with state machine logic")
print("   • Adaptive thresholds with machine learning behavior")
print("   • Composite patterns with chained hook behaviors")
print("   • Advanced state management across hook executions")
print("   • Performance monitoring and optimization patterns")