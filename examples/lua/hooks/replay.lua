-- ABOUTME: Lua example demonstrating hook replay functionality with parameter modification
-- ABOUTME: Shows how to use the Replay API for debugging and what-if analysis

-- Example 1: Create replay configurations
print("=== Replay Configuration Example ===")

-- Create different replay modes
local exactMode = Replay.modes.exact
local modifiedMode = Replay.modes.modified
local simulateMode = Replay.modes.simulate
local debugMode = Replay.modes.debug

print("Exact mode:", exactMode:name())
print("Modified mode:", modifiedMode:name())

-- Create a replay configuration
local config = Replay.create_config(modifiedMode)
print("Config mode:", config:get_mode():name())
print("Should compare results:", config:should_compare_results())
print("Timeout seconds:", config:get_timeout_seconds())

-- Example 2: Parameter modifications
print("\n=== Parameter Modifications Example ===")

-- Create parameter modifications
local mod1 = Replay.create_modification("context.data.input_value", 42.5, true)
local mod2 = Replay.create_modification("context.data.multiplier", 2.0, true)

print("Modification 1 path:", mod1:get_path())
print("Modification 1 value:", mod1:get_value())
print("Modification 1 enabled:", mod1:is_enabled())

-- Add modifications to config
config:add_modification("context.data.test_value", "modified", true)
config:add_modification("context.data.count", 100, true)

local mods = config:get_modifications()
print("Total modifications:", #mods)

-- Example 3: Replay schedules
print("\n=== Replay Schedules Example ===")

-- Create different schedule types
local onceSchedule = Replay.schedules.once(5.0)  -- 5 seconds delay
local intervalSchedule = Replay.schedules.interval(10.0, 60.0, 5)  -- 10s initial, 60s interval, max 5
local cronSchedule = Replay.schedules.cron("hourly")

print("Once schedule type:", onceSchedule:type_name())
print("Interval schedule type:", intervalSchedule:type_name())
print("Cron schedule type:", cronSchedule:type_name())

-- Example 4: Result comparison
print("\n=== Result Comparison Example ===")

-- Create a comparator
local comparator = Replay.create_comparator()

-- Create two JSON structures to compare
local original = {
    status = "success",
    value = 42,
    items = {1, 2, 3},
    metadata = {
        timestamp = 1234567890,
        user = "test"
    }
}

local replayed = {
    status = "success", 
    value = 43,  -- Different
    items = {1, 2, 3, 4},  -- Different length
    metadata = {
        timestamp = 1234567891,  -- Different
        user = "test"
    }
}

-- Compare the results
local comparison = comparator:compare_json(original, replayed)

print("Results identical:", comparison.identical)
print("Similarity score:", comparison.similarity_score)
print("Summary:", comparison.summary)

if comparison.differences then
    print("Differences found:")
    for i, diff in ipairs(comparison.differences) do
        print(string.format("  [%d] %s: %s", i, diff.path, diff.description))
    end
end

-- Example 5: Complex nested comparison
print("\n=== Complex Comparison Example ===")

local complex1 = {
    agents = {
        {name = "agent1", config = {model = "gpt-4", temperature = 0.7}},
        {name = "agent2", config = {model = "claude", temperature = 0.5}}
    },
    workflow = {
        steps = {"init", "process", "complete"},
        timeout = 300
    }
}

local complex2 = {
    agents = {
        {name = "agent1", config = {model = "gpt-4", temperature = 0.8}},  -- Different temp
        {name = "agent2", config = {model = "claude", temperature = 0.5}}
    },
    workflow = {
        steps = {"init", "process", "validate", "complete"},  -- Extra step
        timeout = 300
    }
}

local complexComparison = comparator:compare_json(complex1, complex2)
print("Complex comparison - Identical:", complexComparison.identical)
print("Complex comparison - Similarity:", complexComparison.similarity_score)

-- Example 6: Working with modifications programmatically
print("\n=== Programmatic Modifications Example ===")

-- Create a config for what-if analysis
local whatIfConfig = Replay.create_config(Replay.modes.modified)

-- Add multiple modifications based on conditions
local testValues = {10, 20, 50, 100}
for i, value in ipairs(testValues) do
    whatIfConfig:add_modification(
        string.format("context.data.test_%d", i),
        value * 2,
        true
    )
end

print("Created", #whatIfConfig:get_modifications(), "what-if modifications")

-- Disable some modifications
local allMods = whatIfConfig:get_modifications()
if #allMods > 2 then
    allMods[2]:set_enabled(false)
    print("Disabled modification at index 2")
end

print("\nReplay Lua example completed!")