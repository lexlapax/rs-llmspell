-- Cookbook: Error Handling - Production LLM Error Patterns
-- Purpose: Demonstrate comprehensive error handling for real LLM interactions
-- Prerequisites: OPENAI_API_KEY or ANTHROPIC_API_KEY environment variable
-- Expected Output: Error handling demonstrations with real LLM calls
-- Version: 0.7.0
-- Tags: cookbook, error-handling, production, robustness, llm

print("=== Real LLM Error Handling Patterns ===\n")

-- ============================================================
-- Pattern 1: Safe Agent Creation and Invocation
-- ============================================================

print("1. Safe Agent Creation and Invocation")
print("-" .. string.rep("-", 40))

local ErrorHandler = {}
ErrorHandler.__index = ErrorHandler

function ErrorHandler:new()
    return setmetatable({
        errors = {},
        error_counts = {},
        fallback_providers = {"openai", "anthropic"}
    }, self)
end

function ErrorHandler:safe_create_agent(config)
    config = config or {}
    
    local success, agent = pcall(function()
        return Agent.builder()
            :provider(config.provider or "openai")
            :model(config.model or "gpt-3.5-turbo")
            :system_prompt(config.system_prompt or "You are a helpful assistant.")
            :max_tokens(config.max_tokens or 150)
            :temperature(config.temperature or 0.7)
            :build()
    end)
    
    if success then
        return agent, nil
    else
        self:log_error("agent_creation", agent)
        return nil, "Failed to create agent: " .. tostring(agent)
    end
end

function ErrorHandler:safe_agent_call(agent, prompt, max_retries)
    max_retries = max_retries or 3
    local last_error = nil
    
    for attempt = 1, max_retries do
        local success, result = pcall(function()
            return agent:call(prompt)
        end)
        
        if success then
            return result, nil, attempt
        else
            last_error = result
            self:log_error("agent_call", result)
            
            print(string.format("   Attempt %d failed: %s", attempt, 
                self:classify_error(result)))
            
            if attempt < max_retries then
                local delay = self:calculate_backoff(attempt)
                print(string.format("   Retrying in %.1fs...", delay))
                -- In real implementation: sleep(delay)
            end
        end
    end
    
    return nil, "All attempts failed: " .. tostring(last_error), max_retries
end

function ErrorHandler:classify_error(error_msg)
    local error_str = tostring(error_msg):lower()
    
    if string.find(error_str, "rate limit") or string.find(error_str, "429") then
        return "RATE_LIMIT"
    elseif string.find(error_str, "timeout") or string.find(error_str, "connection") then
        return "NETWORK"
    elseif string.find(error_str, "invalid") or string.find(error_str, "400") then
        return "INVALID_REQUEST"
    elseif string.find(error_str, "unauthorized") or string.find(error_str, "401") then
        return "AUTH_ERROR"
    elseif string.find(error_str, "service") or string.find(error_str, "500") then
        return "SERVICE_ERROR"
    else
        return "UNKNOWN"
    end
end

function ErrorHandler:calculate_backoff(attempt)
    -- Exponential backoff with jitter
    local base_delay = 0.5  -- 500ms
    local max_delay = 8.0   -- 8 seconds
    
    local delay = math.min(base_delay * (2 ^ (attempt - 1)), max_delay)
    local jitter = delay * 0.1 * math.random()  -- 10% jitter
    
    return delay + jitter
end

function ErrorHandler:log_error(category, error)
    self.error_counts[category] = (self.error_counts[category] or 0) + 1
    table.insert(self.errors, {
        category = category,
        error = tostring(error),
        timestamp = os.time(),
        classification = self:classify_error(error)
    })
end

function ErrorHandler:get_error_summary()
    local summary = {}
    for category, count in pairs(self.error_counts) do
        summary[category] = count
    end
    return summary
end

-- Test error handling
local handler = ErrorHandler:new()

print("   Testing agent creation and calls...")

-- Try to create agent
local agent, create_err = handler:safe_create_agent({
    provider = "openai",
    model = "gpt-3.5-turbo",
    system_prompt = "You are a helpful assistant. Give very brief answers."
})

if agent then
    print("   ✅ Agent created successfully")
    
    -- Test successful call
    local result, err, attempts = handler:safe_agent_call(agent, 
        "What is the capital of France? (one word answer)")
    
    if result then
        print(string.format("   ✅ Call succeeded after %d attempts", attempts))
        print(string.format("   Response: %s", 
            string.sub(result.response or "", 1, 50)))
    else
        print(string.format("   ❌ All calls failed: %s", err))
    end
    
    -- Test with problematic prompt to trigger errors
    local bad_result, bad_err = handler:safe_agent_call(agent, 
        string.rep("x", 50000), 1)  -- Very long prompt might fail
    
    if not bad_result then
        print("   ⚠️  Expected failure with oversized prompt")
    end
    
else
    print(string.format("   ❌ Agent creation failed: %s", create_err))
    print("   (This is expected if no API key is configured)")
end

print()

-- ============================================================
-- Pattern 2: Provider Fallback Strategy
-- ============================================================

print("2. Provider Fallback Strategy")
print("-" .. string.rep("-", 40))

local FallbackHandler = {}
FallbackHandler.__index = FallbackHandler

function FallbackHandler:new(providers)
    return setmetatable({
        providers = providers or {"openai", "anthropic"},
        agent_cache = {},
        provider_health = {}
    }, self)
end

function FallbackHandler:get_or_create_agent(provider)
    if self.agent_cache[provider] then
        return self.agent_cache[provider], nil
    end
    
    local success, agent = pcall(function()
        return Agent.builder()
            :provider(provider)
            :system_prompt("You are a helpful assistant. Be concise.")
            :max_tokens(100)
            :build()
    end)
    
    if success then
        self.agent_cache[provider] = agent
        return agent, nil
    else
        return nil, "Failed to create " .. provider .. " agent: " .. tostring(agent)
    end
end

function FallbackHandler:call_with_fallback(prompt)
    local errors = {}
    
    for i, provider in ipairs(self.providers) do
        print(string.format("   Trying provider: %s", provider))
        
        local agent, create_err = self:get_or_create_agent(provider)
        
        if agent then
            local success, result = pcall(function()
                return agent:call(prompt)
            end)
            
            if success then
                print(string.format("   ✅ Success with %s", provider))
                return result, provider, i
            else
                errors[provider] = result
                print(string.format("   ❌ %s failed: %s", provider, 
                    string.sub(tostring(result), 1, 50)))
            end
        else
            errors[provider] = create_err
            print(string.format("   ❌ %s creation failed", provider))
        end
    end
    
    return nil, "All providers failed", errors
end

-- Test fallback strategy
local fallback = FallbackHandler:new({"openai", "anthropic"})

print("   Testing provider fallback...")

local fb_result, fb_provider, fb_attempt = fallback:call_with_fallback(
    "What is AI? (brief answer)")

if fb_result then
    print(string.format("   ✅ Succeeded with %s on attempt %d", 
        fb_provider, fb_attempt))
else
    print("   ❌ All providers failed (expected if no API keys)")
end

print()

-- ============================================================
-- Pattern 3: Graceful Degradation
-- ============================================================

print("3. Graceful Degradation")
print("-" .. string.rep("-", 40))

local GracefulService = {}
GracefulService.__index = GracefulService

function GracefulService:new()
    return setmetatable({
        cache = {},
        fallback_responses = {
            classification = "Unable to classify at this time",
            summarization = "Summary not available",
            translation = "Translation service unavailable",
            default = "Service temporarily unavailable"
        }
    }, self)
end

function GracefulService:classify_text(text, agent)
    local cache_key = "classify:" .. string.sub(text, 1, 20)
    
    -- Check cache first
    if self.cache[cache_key] then
        print("   📦 Using cached classification")
        return self.cache[cache_key]
    end
    
    if agent then
        local success, result = pcall(function()
            return agent:call("Classify this text as positive, negative, or neutral: " .. text)
        end)
        
        if success and result.response then
            self.cache[cache_key] = result.response
            return result.response
        end
    end
    
    -- Graceful fallback
    print("   🔄 Using fallback classification")
    return self.fallback_responses.classification
end

function GracefulService:get_fallback_summary(text)
    -- Simple extractive fallback
    local sentences = {}
    for sentence in string.gmatch(text, "[^%.!%?]+") do
        if #sentence > 10 then
            table.insert(sentences, sentence)
        end
    end
    
    if #sentences > 0 then
        return sentences[1] .. "."
    else
        return self.fallback_responses.summarization
    end
end

-- Test graceful degradation
local service = GracefulService:new()

print("   Testing graceful degradation...")

-- Try with nil agent (simulates failure)
local classification = service:classify_text("This is a great product!", nil)
print(string.format("   Classification result: %s", classification))

-- Test fallback summary
local summary = service:get_fallback_summary(
    "This is the first sentence. This is the second sentence. This is the third.")
print(string.format("   Fallback summary: %s", summary))

print()

-- ============================================================
-- Pattern 4: Error Recovery and Circuit Breaking
-- ============================================================

print("4. Error Recovery and Circuit Breaking")
print("-" .. string.rep("-", 40))

local CircuitBreaker = {}
CircuitBreaker.__index = CircuitBreaker

function CircuitBreaker:new(failure_threshold, recovery_timeout)
    return setmetatable({
        failure_threshold = failure_threshold or 5,
        recovery_timeout = recovery_timeout or 60,  -- seconds
        failure_count = 0,
        last_failure = 0,
        state = "CLOSED"  -- CLOSED, OPEN, HALF_OPEN
    }, self)
end

function CircuitBreaker:can_execute()
    if self.state == "CLOSED" then
        return true
    elseif self.state == "OPEN" then
        if (os.time() - self.last_failure) > self.recovery_timeout then
            self.state = "HALF_OPEN"
            print("   🔄 Circuit breaker moved to HALF_OPEN")
            return true
        end
        return false
    elseif self.state == "HALF_OPEN" then
        return true
    end
    
    return false
end

function CircuitBreaker:record_success()
    self.failure_count = 0
    if self.state == "HALF_OPEN" then
        self.state = "CLOSED"
        print("   ✅ Circuit breaker recovered to CLOSED")
    end
end

function CircuitBreaker:record_failure()
    self.failure_count = self.failure_count + 1
    self.last_failure = os.time()
    
    if self.failure_count >= self.failure_threshold then
        self.state = "OPEN"
        print(string.format("   ⚠️  Circuit breaker OPENED after %d failures", 
            self.failure_count))
    end
end

function CircuitBreaker:execute(fn)
    if not self:can_execute() then
        return nil, "Circuit breaker is OPEN"
    end
    
    local success, result = pcall(fn)
    
    if success then
        self:record_success()
        return result, nil
    else
        self:record_failure()
        return nil, result
    end
end

-- Test circuit breaker
local breaker = CircuitBreaker:new(3, 30)  -- 3 failures, 30 sec recovery

print("   Testing circuit breaker...")

-- Simulate failures
for i = 1, 5 do
    local result, err = breaker:execute(function()
        if i <= 3 then
            error("Simulated failure " .. i)
        else
            return "Success!"
        end
    end)
    
    if result then
        print(string.format("   Call %d: ✅ %s", i, result))
    else
        print(string.format("   Call %d: ❌ %s", i, err))
    end
end

print()

-- Show error summary
local error_summary = handler:get_error_summary()
print("🎯 Error Summary:")
for category, count in pairs(error_summary) do
    print(string.format("   %s: %d errors", category, count))
end

print("\n🎯 Key Takeaways:")
print("   • Always wrap LLM calls in error handling")
print("   • Implement provider fallback strategies")
print("   • Use graceful degradation when possible")
print("   • Circuit breakers prevent cascade failures")
print("   • Cache successful results for resilience")
print("   • Classify errors for appropriate responses")