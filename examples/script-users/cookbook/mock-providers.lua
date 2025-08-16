-- Cookbook: Mock Providers - Testing Without Real LLMs
-- Purpose: Implement mock providers for testing LLM applications
-- Prerequisites: Development/testing environment (no production dependencies)
-- Expected Output: Demonstration of mock provider patterns
-- Version: 0.7.0
-- Tags: cookbook, testing, mocks, providers, development

print("=== Mock Provider Patterns ===\n")

-- ============================================================
-- Pattern 1: Deterministic Mock Provider
-- ============================================================

print("1. Deterministic Mock Provider")
print("-" .. string.rep("-", 40))

local DeterministicMockProvider = {}
DeterministicMockProvider.__index = DeterministicMockProvider

function DeterministicMockProvider:new(name)
    return setmetatable({
        name = name or "mock-deterministic",
        responses = {},
        call_count = {},
        latency = 0  -- Simulated latency in ms
    }, self)
end

function DeterministicMockProvider:set_response(prompt_pattern, response)
    self.responses[prompt_pattern] = response
    print(string.format("   Set response for pattern: %s", prompt_pattern))
end

function DeterministicMockProvider:set_latency(ms)
    self.latency = ms
end

function DeterministicMockProvider:complete(prompt, options)
    options = options or {}
    
    -- Track calls
    self.call_count[prompt] = (self.call_count[prompt] or 0) + 1
    
    -- Simulate latency
    if self.latency > 0 then
        -- In real implementation, would use sleep
        print(string.format("   Simulating %dms latency...", self.latency))
    end
    
    -- Find matching response
    for pattern, response in pairs(self.responses) do
        if string.find(prompt, pattern) then
            return {
                success = true,
                response = response,
                model = self.name,
                tokens_used = #response,  -- Simplified
                latency = self.latency
            }
        end
    end
    
    -- Default response if no pattern matches
    return {
        success = true,
        response = "Default mock response for: " .. string.sub(prompt, 1, 50),
        model = self.name,
        tokens_used = 10,
        latency = self.latency
    }
end

function DeterministicMockProvider:get_call_count(prompt)
    if prompt then
        return self.call_count[prompt] or 0
    end
    
    -- Total calls
    local total = 0
    for _, count in pairs(self.call_count) do
        total = total + count
    end
    return total
end

-- Test deterministic mock
local det_mock = DeterministicMockProvider:new()

-- Set up responses
det_mock:set_response("classify", "Category: Technology")
det_mock:set_response("summarize", "This is a summary of the text.")
det_mock:set_response("translate", "Ceci est une traduction.")
det_mock:set_latency(50)

-- Test prompts
print("\n   Testing deterministic mock:")

local prompts = {
    "Please classify this text",
    "Can you summarize this article",
    "Translate to French: Hello"
}

for _, prompt in ipairs(prompts) do
    local result = det_mock:complete(prompt)
    print(string.format("   Prompt: '%s'", string.sub(prompt, 1, 30)))
    print(string.format("   Response: %s", result.response))
end

print(string.format("\n   Total calls: %d", det_mock:get_call_count()))

print()

-- ============================================================
-- Pattern 2: Stateful Mock Provider
-- ============================================================

print("2. Stateful Mock Provider")
print("-" .. string.rep("-", 40))

local StatefulMockProvider = {}
StatefulMockProvider.__index = StatefulMockProvider

function StatefulMockProvider:new(name)
    return setmetatable({
        name = name or "mock-stateful",
        state = {},
        conversation_history = {},
        behavior = "helpful",  -- helpful, confused, verbose, terse
        error_rate = 0  -- Probability of error (0-1)
    }, self)
end

function StatefulMockProvider:set_behavior(behavior)
    self.behavior = behavior
    print(string.format("   Behavior set to: %s", behavior))
end

function StatefulMockProvider:set_error_rate(rate)
    self.error_rate = math.min(1, math.max(0, rate))
end

function StatefulMockProvider:complete(prompt, options)
    options = options or {}
    
    -- Random error injection
    if math.random() < self.error_rate then
        return {
            success = false,
            error = "Simulated provider error",
            model = self.name
        }
    end
    
    -- Add to conversation history
    table.insert(self.conversation_history, {
        role = "user",
        content = prompt
    })
    
    -- Generate response based on behavior
    local response = self:generate_response(prompt)
    
    -- Add response to history
    table.insert(self.conversation_history, {
        role = "assistant",
        content = response
    })
    
    -- Update state based on prompt
    self:update_state(prompt)
    
    return {
        success = true,
        response = response,
        model = self.name,
        conversation_length = #self.conversation_history
    }
end

function StatefulMockProvider:generate_response(prompt)
    if self.behavior == "helpful" then
        return "I understand your request about '" .. 
            string.sub(prompt, 1, 20) .. "'. Here's my helpful response."
            
    elseif self.behavior == "confused" then
        return "I'm not sure I understand. Could you clarify what you mean by '" ..
            string.sub(prompt, 1, 15) .. "'?"
            
    elseif self.behavior == "verbose" then
        return "That's an excellent question! Let me provide a comprehensive answer. " ..
            "First, we need to consider the context. " ..
            "Second, there are multiple perspectives. " ..
            "Third, the implications are significant. " ..
            "In conclusion, regarding '" .. string.sub(prompt, 1, 10) .. "', the answer is complex."
            
    elseif self.behavior == "terse" then
        return "Yes."
    end
    
    return "Mock response."
end

function StatefulMockProvider:update_state(prompt)
    -- Extract and update state from prompts
    if string.find(prompt, "remember") then
        local item = string.match(prompt, "remember%s+(.+)")
        if item then
            table.insert(self.state, item)
        end
    elseif string.find(prompt, "forget") then
        self.state = {}
    end
end

function StatefulMockProvider:get_state()
    return {
        behavior = self.behavior,
        history_length = #self.conversation_history,
        state_items = #self.state,
        error_rate = self.error_rate
    }
end

-- Test stateful mock
local stateful_mock = StatefulMockProvider:new()

print("\n   Testing stateful mock:")

-- Test different behaviors
local behaviors = {"helpful", "confused", "verbose", "terse"}

for _, behavior in ipairs(behaviors) do
    stateful_mock:set_behavior(behavior)
    local result = stateful_mock:complete("What is the weather?")
    print(string.format("   [%s]: %s", behavior, 
        string.sub(result.response, 1, 50) .. 
        (#result.response > 50 and "..." or "")))
end

-- Test statefulness
stateful_mock:set_behavior("helpful")
stateful_mock:complete("Remember that my name is Alice")
stateful_mock:complete("Remember that I like pizza")

local state = stateful_mock:get_state()
print(string.format("\n   State: %d items, %d messages in history", 
    state.state_items, state.history_length))

print()

-- ============================================================
-- Pattern 3: Scenario-Based Mock Provider
-- ============================================================

print("3. Scenario-Based Mock Provider")
print("-" .. string.rep("-", 40))

local ScenarioMockProvider = {}
ScenarioMockProvider.__index = ScenarioMockProvider

function ScenarioMockProvider:new(name)
    return setmetatable({
        name = name or "mock-scenario",
        scenarios = {},
        current_scenario = nil,
        step = 1
    }, self)
end

function ScenarioMockProvider:add_scenario(name, steps)
    self.scenarios[name] = {
        name = name,
        steps = steps,
        completed = false
    }
    print(string.format("   Added scenario: %s with %d steps", name, #steps))
end

function ScenarioMockProvider:set_scenario(name)
    if self.scenarios[name] then
        self.current_scenario = name
        self.step = 1
        self.scenarios[name].completed = false
        print(string.format("   Active scenario: %s", name))
        return true
    end
    return false
end

function ScenarioMockProvider:complete(prompt, options)
    if not self.current_scenario then
        return {
            success = false,
            error = "No scenario active"
        }
    end
    
    local scenario = self.scenarios[self.current_scenario]
    
    if self.step > #scenario.steps then
        scenario.completed = true
        return {
            success = true,
            response = "Scenario completed",
            scenario_complete = true
        }
    end
    
    local current_step = scenario.steps[self.step]
    
    -- Check if prompt matches expected
    if current_step.expect then
        if not string.find(prompt, current_step.expect) then
            return {
                success = false,
                error = "Unexpected prompt. Expected to contain: " .. current_step.expect,
                expected = current_step.expect,
                received = prompt
            }
        end
    end
    
    -- Get response for this step
    local response = current_step.response
    if type(response) == "function" then
        response = response(prompt)
    end
    
    -- Move to next step
    self.step = self.step + 1
    
    return {
        success = true,
        response = response,
        scenario = self.current_scenario,
        step = self.step - 1,
        total_steps = #scenario.steps
    }
end

-- Test scenario mock
local scenario_mock = ScenarioMockProvider:new()

-- Add a multi-step scenario
scenario_mock:add_scenario("booking_flow", {
    {
        expect = "help.*book",
        response = "I can help you book a flight. Where would you like to go?"
    },
    {
        expect = "Paris",
        response = "Great! When would you like to travel to Paris?"
    },
    {
        expect = "next week",
        response = "I found flights to Paris next week. Economy or Business class?"
    },
    {
        expect = "Economy",
        response = "Perfect! Economy flight to Paris booked for next week."
    }
})

-- Run through scenario
scenario_mock:set_scenario("booking_flow")

print("\n   Testing scenario mock:")

local conversation = {
    "I need help booking a flight",
    "I want to go to Paris",
    "Maybe next week",
    "Economy please"
}

for i, prompt in ipairs(conversation) do
    local result = scenario_mock:complete(prompt)
    
    if result.success then
        print(string.format("   Step %d/%d:", result.step, result.total_steps))
        print(string.format("     User: %s", prompt))
        print(string.format("     Bot: %s", result.response))
    else
        print(string.format("   Error: %s", result.error))
    end
end

print()

-- ============================================================
-- Pattern 4: Benchmark Mock Provider
-- ============================================================

print("4. Benchmark Mock Provider")
print("-" .. string.rep("-", 40))

local BenchmarkMockProvider = {}
BenchmarkMockProvider.__index = BenchmarkMockProvider

function BenchmarkMockProvider:new(name)
    return setmetatable({
        name = name or "mock-benchmark",
        metrics = {
            total_calls = 0,
            total_tokens = 0,
            total_latency = 0,
            errors = 0,
            start_time = os.time()
        },
        latency_distribution = "normal",  -- normal, uniform, exponential
        mean_latency = 100,  -- ms
        response_size = "medium"  -- small, medium, large
    }, self)
end

function BenchmarkMockProvider:set_latency_profile(distribution, mean)
    self.latency_distribution = distribution
    self.mean_latency = mean
end

function BenchmarkMockProvider:set_response_size(size)
    self.response_size = size
end

function BenchmarkMockProvider:generate_latency()
    if self.latency_distribution == "normal" then
        -- Simplified normal distribution
        local variance = self.mean_latency * 0.2
        return self.mean_latency + (math.random() - 0.5) * variance * 2
        
    elseif self.latency_distribution == "uniform" then
        return math.random(self.mean_latency * 0.5, self.mean_latency * 1.5)
        
    elseif self.latency_distribution == "exponential" then
        return -self.mean_latency * math.log(1 - math.random())
    end
    
    return self.mean_latency
end

function BenchmarkMockProvider:generate_response()
    local sizes = {
        small = 10,
        medium = 100,
        large = 1000
    }
    
    local size = sizes[self.response_size] or 100
    local response = "Benchmark response. "
    
    for i = 1, size / 10 do
        response = response .. "Token" .. i .. " "
    end
    
    return response, size
end

function BenchmarkMockProvider:complete(prompt, options)
    self.metrics.total_calls = self.metrics.total_calls + 1
    
    -- Simulate random errors
    if math.random() < 0.01 then  -- 1% error rate
        self.metrics.errors = self.metrics.errors + 1
        return {
            success = false,
            error = "Simulated error for benchmarking"
        }
    end
    
    -- Generate latency
    local latency = self:generate_latency()
    self.metrics.total_latency = self.metrics.total_latency + latency
    
    -- Generate response
    local response, tokens = self:generate_response()
    self.metrics.total_tokens = self.metrics.total_tokens + tokens
    
    return {
        success = true,
        response = response,
        latency = latency,
        tokens = tokens,
        model = self.name
    }
end

function BenchmarkMockProvider:get_metrics()
    local elapsed = os.time() - self.metrics.start_time
    elapsed = math.max(1, elapsed)  -- Avoid division by zero
    
    return {
        total_calls = self.metrics.total_calls,
        successful_calls = self.metrics.total_calls - self.metrics.errors,
        error_rate = self.metrics.errors / math.max(1, self.metrics.total_calls) * 100,
        avg_latency = self.metrics.total_latency / math.max(1, self.metrics.total_calls),
        total_tokens = self.metrics.total_tokens,
        tokens_per_second = self.metrics.total_tokens / elapsed,
        calls_per_second = self.metrics.total_calls / elapsed
    }
end

-- Test benchmark mock
local bench_mock = BenchmarkMockProvider:new()

bench_mock:set_latency_profile("normal", 50)
bench_mock:set_response_size("medium")

print("\n   Running benchmark simulation:")

-- Simulate load
for i = 1, 20 do
    local result = bench_mock:complete("Benchmark prompt " .. i)
    
    if i % 5 == 0 then
        local metrics = bench_mock:get_metrics()
        print(string.format("   After %d calls: avg latency=%.1fms, error rate=%.1f%%",
            i, metrics.avg_latency, metrics.error_rate))
    end
end

-- Final metrics
local final_metrics = bench_mock:get_metrics()
print(string.format("\n   Final metrics:"))
print(string.format("   Total calls: %d", final_metrics.total_calls))
print(string.format("   Success rate: %.1f%%", 100 - final_metrics.error_rate))
print(string.format("   Avg latency: %.1fms", final_metrics.avg_latency))
print(string.format("   Total tokens: %d", final_metrics.total_tokens))

print()

-- ============================================================
-- Pattern 5: Replay Mock Provider
-- ============================================================

print("5. Replay Mock Provider")
print("-" .. string.rep("-", 40))

local ReplayMockProvider = {}
ReplayMockProvider.__index = ReplayMockProvider

function ReplayMockProvider:new(name)
    return setmetatable({
        name = name or "mock-replay",
        recordings = {},
        current_recording = nil,
        replay_index = 1,
        mode = "replay"  -- record, replay
    }, self)
end

function ReplayMockProvider:start_recording(name)
    self.mode = "record"
    self.current_recording = name
    self.recordings[name] = {
        name = name,
        interactions = {},
        recorded_at = os.time()
    }
    print(string.format("   Started recording: %s", name))
end

function ReplayMockProvider:stop_recording()
    if self.mode == "record" and self.current_recording then
        local recording = self.recordings[self.current_recording]
        print(string.format("   Stopped recording: %s (%d interactions)",
            self.current_recording, #recording.interactions))
        self.current_recording = nil
        self.mode = "replay"
    end
end

function ReplayMockProvider:load_recording(name)
    if self.recordings[name] then
        self.current_recording = name
        self.replay_index = 1
        self.mode = "replay"
        print(string.format("   Loaded recording: %s", name))
        return true
    end
    return false
end

function ReplayMockProvider:complete(prompt, options)
    if self.mode == "record" then
        -- In record mode, use a real provider or manual input
        local response = "Recorded response for: " .. prompt
        
        -- Store the interaction
        if self.current_recording then
            local recording = self.recordings[self.current_recording]
            table.insert(recording.interactions, {
                prompt = prompt,
                response = response,
                timestamp = os.time()
            })
        end
        
        return {
            success = true,
            response = response,
            mode = "recording"
        }
        
    elseif self.mode == "replay" then
        -- In replay mode, return recorded responses
        if not self.current_recording then
            return {
                success = false,
                error = "No recording loaded"
            }
        end
        
        local recording = self.recordings[self.current_recording]
        
        if self.replay_index > #recording.interactions then
            return {
                success = false,
                error = "End of recording reached"
            }
        end
        
        local interaction = recording.interactions[self.replay_index]
        
        -- Verify prompt matches (optional)
        if interaction.prompt ~= prompt then
            print(string.format("   Warning: Prompt mismatch"))
            print(string.format("     Expected: %s", interaction.prompt))
            print(string.format("     Received: %s", prompt))
        end
        
        self.replay_index = self.replay_index + 1
        
        return {
            success = true,
            response = interaction.response,
            mode = "replay",
            interaction_number = self.replay_index - 1,
            total_interactions = #recording.interactions
        }
    end
end

-- Test replay mock
local replay_mock = ReplayMockProvider:new()

-- Record interactions
replay_mock:start_recording("test_session")

print("\n   Recording interactions:")
local prompts_to_record = {
    "What is AI?",
    "Explain machine learning",
    "What are neural networks?"
}

for _, prompt in ipairs(prompts_to_record) do
    local result = replay_mock:complete(prompt)
    print(string.format("   Recorded: %s", string.sub(prompt, 1, 30)))
end

replay_mock:stop_recording()

-- Replay interactions
replay_mock:load_recording("test_session")

print("\n   Replaying interactions:")
for _, prompt in ipairs(prompts_to_record) do
    local result = replay_mock:complete(prompt)
    print(string.format("   Interaction %d/%d: %s",
        result.interaction_number or 0,
        result.total_interactions or 0,
        string.sub(result.response, 1, 40)))
end

print()
print("🎯 Key Takeaways:")
print("   • Deterministic mocks ensure predictable testing")
print("   • Stateful mocks test conversation flow")
print("   • Scenario mocks validate multi-step interactions")
print("   • Benchmark mocks measure performance")
print("   • Replay mocks enable regression testing")