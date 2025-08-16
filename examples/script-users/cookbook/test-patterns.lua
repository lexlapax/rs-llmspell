-- Cookbook: Testing Patterns - Comprehensive Testing Strategies
-- Purpose: Implement patterns for testing LLM applications and agents
-- Prerequisites: Testing framework patterns (implementation-agnostic)
-- Expected Output: Demonstration of testing patterns
-- Version: 0.7.0
-- Tags: cookbook, testing, quality, validation, production

print("=== Testing Patterns for LLM Applications ===\n")

-- ============================================================
-- Pattern 1: Property-Based Testing
-- ============================================================

print("1. Property-Based Testing")
print("-" .. string.rep("-", 40))

local PropertyTest = {}
PropertyTest.__index = PropertyTest

function PropertyTest:new()
    return setmetatable({
        generators = {},
        properties = {},
        results = {},
        config = {
            runs = 100,
            seed = os.time(),
            shrink = true
        }
    }, self)
end

function PropertyTest:generator(name, gen_fn)
    self.generators[name] = gen_fn
    return self
end

function PropertyTest:property(name, property_fn)
    self.properties[name] = property_fn
    return self
end

function PropertyTest:generate_input(generator_name)
    local gen = self.generators[generator_name]
    if not gen then
        error("Unknown generator: " .. generator_name)
    end
    return gen()
end

function PropertyTest:check(property_name, input_generator, runs)
    runs = runs or self.config.runs
    
    local property_fn = self.properties[property_name]
    if not property_fn then
        error("Unknown property: " .. property_name)
    end
    
    local failures = {}
    local successes = 0
    
    print(string.format("   Testing property '%s' with %d runs", property_name, runs))
    
    for i = 1, runs do
        local input = self:generate_input(input_generator)
        local success, result = pcall(property_fn, input)
        
        if success and result then
            successes = successes + 1
        else
            table.insert(failures, {
                input = input,
                error = result
            })
            
            -- Try to shrink on failure
            if self.config.shrink and #failures == 1 then
                self:shrink_failure(property_fn, input)
            end
        end
    end
    
    self.results[property_name] = {
        runs = runs,
        successes = successes,
        failures = failures,
        success_rate = (successes / runs) * 100
    }
    
    print(string.format("   Results: %d/%d passed (%.1f%%)", 
        successes, runs, (successes / runs) * 100))
    
    if #failures > 0 then
        print(string.format("   First failure: %s", 
            failures[1].error or "Property returned false"))
    end
    
    return #failures == 0
end

function PropertyTest:shrink_failure(property_fn, failing_input)
    -- Simple shrinking: try smaller inputs
    print("   Attempting to shrink failing input...")
    
    local current = failing_input
    local smaller_found = false
    
    -- Try to make the input smaller/simpler
    if type(current) == "number" then
        for i = 1, 10 do
            local smaller = math.floor(current / 2)
            local success, result = pcall(property_fn, smaller)
            
            if not success or not result then
                current = smaller
                smaller_found = true
            end
        end
    elseif type(current) == "string" then
        for i = #current, 1, -1 do
            local smaller = string.sub(current, 1, i-1)
            local success, result = pcall(property_fn, smaller)
            
            if not success or not result then
                current = smaller
                smaller_found = true
            end
        end
    end
    
    if smaller_found then
        print(string.format("   Shrunk to simpler failing case: %s", 
            tostring(current)))
    end
    
    return current
end

-- Set up property tests
local prop_test = PropertyTest:new()

-- Define generators
prop_test:generator("positive_int", function()
    return math.random(1, 1000)
end)

prop_test:generator("any_string", function()
    local chars = "abcdefghijklmnopqrstuvwxyz0123456789"
    local length = math.random(0, 20)
    local str = ""
    for i = 1, length do
        local idx = math.random(1, #chars)
        str = str .. string.sub(chars, idx, idx)
    end
    return str
end)

prop_test:generator("email", function()
    local domains = {"example.com", "test.org", "mail.net"}
    local user = ""
    for i = 1, math.random(3, 10) do
        user = user .. string.char(math.random(97, 122))
    end
    return user .. "@" .. domains[math.random(1, #domains)]
end)

-- Define properties
prop_test:property("reversible", function(str)
    -- Property: reversing twice gives original
    local reversed = string.reverse(str)
    local double_reversed = string.reverse(reversed)
    return str == double_reversed
end)

prop_test:property("monotonic_length", function(str)
    -- Property: appending always increases length
    local original_len = #str
    local appended = str .. "x"
    return #appended > original_len
end)

prop_test:property("email_format", function(email)
    -- Property: emails contain @ and .
    return string.find(email, "@") and string.find(email, "%.")
end)

-- Run property tests
prop_test:check("reversible", "any_string", 50)
prop_test:check("monotonic_length", "any_string", 50)
prop_test:check("email_format", "email", 50)

print()

-- ============================================================
-- Pattern 2: Golden Testing / Snapshot Testing
-- ============================================================

print("2. Golden Testing / Snapshot Testing")
print("-" .. string.rep("-", 40))

local GoldenTest = {}
GoldenTest.__index = GoldenTest

function GoldenTest:new()
    return setmetatable({
        snapshots = {},
        mode = "test",  -- "test" or "update"
        failures = {}
    }, self)
end

function GoldenTest:set_mode(mode)
    self.mode = mode
    print(string.format("   Golden test mode: %s", mode))
end

function GoldenTest:snapshot(name, actual)
    if self.mode == "update" then
        -- Update golden file
        self.snapshots[name] = {
            value = actual,
            updated_at = os.time(),
            version = (self.snapshots[name] and self.snapshots[name].version or 0) + 1
        }
        print(string.format("   Updated snapshot: %s", name))
        return true
    else
        -- Test mode - compare with golden
        local golden = self.snapshots[name]
        
        if not golden then
            print(string.format("   ⚠️  No snapshot found for: %s", name))
            self.snapshots[name] = {
                value = actual,
                updated_at = os.time(),
                version = 1
            }
            return true  -- First run, create snapshot
        end
        
        local matches = self:deep_equal(golden.value, actual)
        
        if matches then
            print(string.format("   ✅ Snapshot match: %s", name))
        else
            print(string.format("   ❌ Snapshot mismatch: %s", name))
            table.insert(self.failures, {
                name = name,
                expected = golden.value,
                actual = actual
            })
        end
        
        return matches
    end
end

function GoldenTest:deep_equal(a, b)
    if type(a) ~= type(b) then
        return false
    end
    
    if type(a) == "table" then
        for k, v in pairs(a) do
            if not self:deep_equal(v, b[k]) then
                return false
            end
        end
        for k, v in pairs(b) do
            if not self:deep_equal(v, a[k]) then
                return false
            end
        end
        return true
    else
        return a == b
    end
end

function GoldenTest:test_llm_response(name, prompt, response_processor)
    -- Process response to extract testable parts
    local processed = response_processor(response)
    
    return self:snapshot(name, processed)
end

-- Test golden testing
local golden = GoldenTest:new()

-- Simulate testing LLM responses
local function process_classification_response(response)
    -- Extract structured data from response
    return {
        classification = "positive",
        confidence = 0.85,
        keywords = {"good", "excellent", "amazing"}
    }
end

local function process_extraction_response(response)
    return {
        entities = {
            {type = "person", value = "John Doe"},
            {type = "location", value = "New York"}
        }
    }
end

-- First run - creates snapshots
golden:snapshot("classification_test", process_classification_response("test response"))
golden:snapshot("extraction_test", process_extraction_response("test response"))

-- Subsequent run - compares with snapshots
golden:snapshot("classification_test", process_classification_response("test response"))

-- Test with different output (will fail)
golden:snapshot("extraction_test", {
    entities = {
        {type = "person", value = "Jane Doe"},  -- Different!
        {type = "location", value = "New York"}
    }
})

print()

-- ============================================================
-- Pattern 3: Behavior Testing for Agents
-- ============================================================

print("3. Behavior Testing for Agents")
print("-" .. string.rep("-", 40))

local BehaviorTest = {}
BehaviorTest.__index = BehaviorTest

function BehaviorTest:new()
    return setmetatable({
        scenarios = {},
        expectations = {},
        results = {}
    }, self)
end

function BehaviorTest:scenario(name, setup)
    self.scenarios[name] = {
        name = name,
        setup = setup,
        steps = {},
        assertions = {}
    }
    return self
end

function BehaviorTest:given(scenario_name, context)
    local scenario = self.scenarios[scenario_name]
    if scenario then
        scenario.context = context
    end
    return self
end

function BehaviorTest:when(scenario_name, action)
    local scenario = self.scenarios[scenario_name]
    if scenario then
        table.insert(scenario.steps, {
            type = "action",
            fn = action
        })
    end
    return self
end

function BehaviorTest:expect(scenario_name, assertion)
    local scenario = self.scenarios[scenario_name]
    if scenario then
        table.insert(scenario.assertions, assertion)
    end
    return self
end

function BehaviorTest:run_scenario(name)
    local scenario = self.scenarios[name]
    if not scenario then
        return false, "Scenario not found"
    end
    
    print(string.format("   Running scenario: %s", name))
    
    -- Setup
    local context = scenario.context or {}
    if scenario.setup then
        context = scenario.setup(context)
    end
    
    -- Execute steps
    for _, step in ipairs(scenario.steps) do
        local success, result = pcall(step.fn, context)
        if not success then
            return false, "Step failed: " .. result
        end
        
        -- Update context with results
        if type(result) == "table" then
            for k, v in pairs(result) do
                context[k] = v
            end
        end
    end
    
    -- Check assertions
    local all_passed = true
    local failures = {}
    
    for i, assertion in ipairs(scenario.assertions) do
        local passed, message = assertion(context)
        
        if passed then
            print(string.format("     ✅ Assertion %d passed", i))
        else
            print(string.format("     ❌ Assertion %d failed: %s", i, message or ""))
            all_passed = false
            table.insert(failures, message)
        end
    end
    
    self.results[name] = {
        passed = all_passed,
        failures = failures
    }
    
    return all_passed, failures
end

-- Test behavior testing
local behavior = BehaviorTest:new()

-- Define agent behavior scenario
behavior:scenario("agent_retry_behavior", function(ctx)
    -- Setup mock agent
    return {
        agent = {
            retry_count = 0,
            max_retries = 3,
            last_error = nil
        }
    }
end)

behavior:given("agent_retry_behavior", {
    input = "test query",
    should_fail_times = 2
})

behavior:when("agent_retry_behavior", function(ctx)
    -- Simulate agent processing with retries
    local agent = ctx.agent
    local failures = 0
    
    for i = 1, agent.max_retries do
        agent.retry_count = i
        
        if failures < ctx.should_fail_times then
            agent.last_error = "Simulated failure " .. i
            failures = failures + 1
        else
            -- Success
            return {
                success = true,
                attempts = i,
                result = "Success after " .. i .. " attempts"
            }
        end
    end
    
    return {
        success = false,
        attempts = agent.retry_count
    }
end)

behavior:expect("agent_retry_behavior", function(ctx)
    -- Should eventually succeed
    if not ctx.success then
        return false, "Agent should have succeeded after retries"
    end
    return true
end)

behavior:expect("agent_retry_behavior", function(ctx)
    -- Should take correct number of attempts
    if ctx.attempts ~= ctx.should_fail_times + 1 then
        return false, string.format("Expected %d attempts, got %d", 
            ctx.should_fail_times + 1, ctx.attempts)
    end
    return true
end)

-- Run the scenario
behavior:run_scenario("agent_retry_behavior")

print()

-- ============================================================
-- Pattern 4: Fuzzing for Robustness
-- ============================================================

print("4. Fuzzing for Robustness Testing")
print("-" .. string.rep("-", 40))

local Fuzzer = {}
Fuzzer.__index = Fuzzer

function Fuzzer:new()
    return setmetatable({
        mutations = {},
        corpus = {},
        crashes = {},
        interesting = {}
    }, self)
end

function Fuzzer:add_mutation(name, mutate_fn)
    self.mutations[name] = mutate_fn
end

function Fuzzer:add_seed(seed)
    table.insert(self.corpus, {
        value = seed,
        interesting = false,
        generation = 0
    })
end

function Fuzzer:mutate(input)
    -- Apply random mutation
    local mutation_names = {}
    for name in pairs(self.mutations) do
        table.insert(mutation_names, name)
    end
    
    if #mutation_names == 0 then
        return input
    end
    
    local mutation_name = mutation_names[math.random(1, #mutation_names)]
    local mutate_fn = self.mutations[mutation_name]
    
    return mutate_fn(input)
end

function Fuzzer:fuzz(target_fn, iterations)
    print(string.format("   Fuzzing with %d iterations", iterations))
    
    local crashes_found = 0
    local interesting_found = 0
    
    for i = 1, iterations do
        -- Pick input from corpus
        local corpus_entry = self.corpus[math.random(1, #self.corpus)]
        local input = corpus_entry.value
        
        -- Mutate input
        local mutated = self:mutate(input)
        
        -- Test target
        local success, result = pcall(target_fn, mutated)
        
        if not success then
            -- Found a crash
            crashes_found = crashes_found + 1
            table.insert(self.crashes, {
                input = mutated,
                error = result,
                iteration = i
            })
            
            print(string.format("     💥 Crash found at iteration %d", i))
            
            -- Add to corpus for further mutation
            table.insert(self.corpus, {
                value = mutated,
                interesting = true,
                generation = corpus_entry.generation + 1
            })
        elseif self:is_interesting(result) then
            -- Found interesting behavior
            interesting_found = interesting_found + 1
            table.insert(self.interesting, {
                input = mutated,
                output = result,
                iteration = i
            })
            
            -- Add to corpus
            table.insert(self.corpus, {
                value = mutated,
                interesting = true,
                generation = corpus_entry.generation + 1
            })
        end
    end
    
    print(string.format("   Found %d crashes, %d interesting inputs", 
        crashes_found, interesting_found))
    
    return {
        crashes = crashes_found,
        interesting = interesting_found
    }
end

function Fuzzer:is_interesting(output)
    -- Define what makes output interesting
    if type(output) == "string" then
        -- Long outputs might be interesting
        return #output > 1000
    elseif type(output) == "number" then
        -- Extreme values might be interesting
        return output > 1000000 or output < -1000000
    end
    return false
end

-- Set up fuzzer
local fuzzer = Fuzzer:new()

-- Add mutations
fuzzer:add_mutation("duplicate", function(input)
    if type(input) == "string" then
        return input .. input
    end
    return input
end)

fuzzer:add_mutation("truncate", function(input)
    if type(input) == "string" and #input > 0 then
        return string.sub(input, 1, math.random(1, #input))
    end
    return input
end)

fuzzer:add_mutation("insert_special", function(input)
    if type(input) == "string" then
        local specials = {"'", '"', "\\", "\n", "\0", "{{", "}}"}
        local special = specials[math.random(1, #specials)]
        local pos = math.random(0, #input)
        return string.sub(input, 1, pos) .. special .. string.sub(input, pos + 1)
    end
    return input
end)

-- Add seed inputs
fuzzer:add_seed("normal input")
fuzzer:add_seed("{{ template }}")
fuzzer:add_seed("")

-- Target function to fuzz
local function prompt_processor(input)
    -- Simulate prompt processing that might crash
    if string.find(input, "\0") then
        error("Null byte in input")
    end
    
    if #input > 100 then
        error("Input too long")
    end
    
    if string.find(input, "{{") and not string.find(input, "}}") then
        error("Unclosed template")
    end
    
    return "Processed: " .. input
end

-- Run fuzzing
fuzzer:fuzz(prompt_processor, 100)

-- Report crashes
if #fuzzer.crashes > 0 then
    print("\n   Sample crashes found:")
    for i = 1, math.min(3, #fuzzer.crashes) do
        local crash = fuzzer.crashes[i]
        print(string.format("     Input: '%s'", 
            string.sub(tostring(crash.input), 1, 30)))
    end
end

print()

-- ============================================================
-- Pattern 5: Regression Testing
-- ============================================================

print("5. Regression Testing")
print("-" .. string.rep("-", 40))

local RegressionTest = {}
RegressionTest.__index = RegressionTest

function RegressionTest:new()
    return setmetatable({
        baseline = {},
        tests = {},
        thresholds = {
            performance = 0.1,  -- 10% degradation allowed
            accuracy = 0.05     -- 5% accuracy drop allowed
        }
    }, self)
end

function RegressionTest:set_baseline(name, metrics)
    self.baseline[name] = {
        metrics = metrics,
        timestamp = os.time()
    }
    print(string.format("   Set baseline for '%s'", name))
end

function RegressionTest:add_test(name, test_fn)
    self.tests[name] = test_fn
end

function RegressionTest:run(name)
    local test_fn = self.tests[name]
    if not test_fn then
        return false, "Test not found"
    end
    
    -- Run test and get current metrics
    local success, current_metrics = pcall(test_fn)
    
    if not success then
        return false, "Test failed: " .. current_metrics
    end
    
    -- Compare with baseline
    local baseline = self.baseline[name]
    if not baseline then
        -- No baseline, set it
        self:set_baseline(name, current_metrics)
        return true, "Baseline set"
    end
    
    -- Check for regressions
    local regressions = {}
    
    for metric, current_value in pairs(current_metrics) do
        local baseline_value = baseline.metrics[metric]
        
        if baseline_value then
            local change = (current_value - baseline_value) / baseline_value
            
            -- Check if regression based on metric type
            if string.find(metric, "time") or string.find(metric, "latency") then
                -- For time metrics, increase is bad
                if change > self.thresholds.performance then
                    table.insert(regressions, {
                        metric = metric,
                        baseline = baseline_value,
                        current = current_value,
                        change = change * 100
                    })
                end
            elseif string.find(metric, "accuracy") or string.find(metric, "score") then
                -- For accuracy metrics, decrease is bad
                if change < -self.thresholds.accuracy then
                    table.insert(regressions, {
                        metric = metric,
                        baseline = baseline_value,
                        current = current_value,
                        change = change * 100
                    })
                end
            end
        end
    end
    
    if #regressions > 0 then
        print(string.format("   ⚠️  %d regressions detected:", #regressions))
        for _, reg in ipairs(regressions) do
            print(string.format("     %s: %.2f -> %.2f (%.1f%% change)", 
                reg.metric, reg.baseline, reg.current, reg.change))
        end
        return false, regressions
    else
        print("   ✅ No regressions detected")
        return true, current_metrics
    end
end

-- Set up regression tests
local regression = RegressionTest:new()

-- Add test for response time
regression:add_test("response_time", function()
    -- Simulate measuring response time
    return {
        avg_latency = 120 + math.random(-10, 10),  -- ms
        p95_latency = 200 + math.random(-20, 20),
        max_latency = 500 + math.random(-50, 50)
    }
end)

-- Add test for accuracy
regression:add_test("classification_accuracy", function()
    -- Simulate measuring accuracy
    return {
        accuracy = 0.92 + (math.random(-5, 5) / 100),
        precision = 0.89 + (math.random(-3, 3) / 100),
        recall = 0.88 + (math.random(-4, 4) / 100)
    }
end)

-- Set baselines
regression:set_baseline("response_time", {
    avg_latency = 120,
    p95_latency = 200,
    max_latency = 500
})

regression:set_baseline("classification_accuracy", {
    accuracy = 0.92,
    precision = 0.89,
    recall = 0.88
})

-- Run regression tests
print("\n   Running regression tests:")
regression:run("response_time")
regression:run("classification_accuracy")

print()
print("🎯 Key Takeaways:")
print("   • Property testing finds edge cases automatically")
print("   • Golden tests detect unexpected changes")
print("   • Behavior tests verify agent decisions")
print("   • Fuzzing discovers robustness issues")
print("   • Regression tests prevent performance degradation")