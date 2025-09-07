-- Test script for DAP debugging with llmspell
-- Set breakpoints on lines marked with "BREAKPOINT"

local function calculate(x, y)
    local result = x + y  -- BREAKPOINT: Set breakpoint here (line 4)
    print("Result:", result)
    return result
end

local a = 10
local b = 20
local c = calculate(a, b)  -- BREAKPOINT: Set breakpoint here (line 11)

-- Test nested functions and closures
local function outer()
    local outer_var = 42
    local function inner()
        local inner_var = 100
        print("Inner function")  -- BREAKPOINT: Set breakpoint here (line 18)
        return inner_var + outer_var
    end
    return inner()
end

local result = outer()
print("Final result:", result)

-- Test tables and complex types
local complex_table = {
    name = "test",
    value = 123,
    nested = {
        a = 1,
        b = 2,
        c = {
            deep = "value"
        }
    }
}

print("Table test")  -- BREAKPOINT: Set breakpoint here (line 40)

-- Test loops
for i = 1, 3 do
    local loop_var = i * 2
    print("Loop iteration:", i, "value:", loop_var)  -- BREAKPOINT: Set breakpoint here (line 45)
end

-- Test error handling
local function might_error(should_error)
    if should_error then
        error("Test error")
    end
    return "success"
end

local status, result = pcall(might_error, false)
print("Error test:", status, result)  -- BREAKPOINT: Set breakpoint here (line 57)

-- Test global variables
_G["global_test"] = "I am global"
local test_global = _G["global_test"]

-- Test special characters in variable names
_G["var-with-dash"] = "dashed"
_G["var.with.dots"] = "dotted"
_G["123numeric"] = "numeric start"

print("Tests completed!")  -- BREAKPOINT: Final breakpoint (line 68)