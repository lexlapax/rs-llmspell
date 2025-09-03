-- examples/script-users/features/debug-showcase.lua
-- Comprehensive debugging feature showcase

-- Test 1: Basic breakpoints
function calculate_fibonacci(n)
    if n <= 1 then
        return n  -- Breakpoint here (line 7)
    end
    local a, b = 0, 1
    for i = 2, n do
        local temp = a + b  -- Breakpoint here (line 11)
        a = b
        b = temp
    end
    return b
end

-- Test 2: Conditional breakpoints
function process_items(items)
    local total = 0
    for i, item in ipairs(items) do
        if item.value > 100 then  -- Conditional breakpoint: item.value > 100
            total = total + item.value * 1.1
        else
            total = total + item.value
        end
    end
    return total
end

-- Test 3: Hit count breakpoints
function stress_test()
    local counter = 0
    for i = 1, 1000 do
        counter = counter + 1  -- Hit count breakpoint: break on 500th hit
        if counter % 100 == 0 then
            print("Processed", counter)
        end
    end
    return counter
end

-- Test 4: Step debugging
function nested_calls()
    local result = step_one()
    return result
end

function step_one()
    local value = 10
    return step_two(value)  -- Step into this
end

function step_two(val)
    local doubled = val * 2
    return step_three(doubled)  -- Step over this
end

function step_three(val)
    return val + 5  -- Step out from here
end

-- Test 5: Variable inspection
function test_variables()
    local simple = "hello"
    local number = 42
    local table_var = {
        name = "test",
        values = {1, 2, 3},
        nested = {
            deep = "value"
        }
    }
    local function_var = calculate_fibonacci
    
    -- Breakpoint here to inspect all variable types
    return { simple, number, table_var, function_var }
end

-- Test 6: Stack navigation
function deep_recursion(n, accumulator)
    if n <= 0 then
        return accumulator  -- Breakpoint here to see full stack
    end
    return deep_recursion(n - 1, accumulator + n)
end

-- Test 7: Exception handling
function test_error_handling()
    local success, result = pcall(function()
        error("Intentional error for debugging")  -- Should pause here
    end)
    return success, result
end

-- Main test runner
function main()
    print("=== Debug Showcase Starting ===")
    
    -- Run all tests
    print("Fibonacci(10):", calculate_fibonacci(10))
    
    local items = {
        {name = "A", value = 50},
        {name = "B", value = 150},  -- Should trigger conditional breakpoint
        {name = "C", value = 75},
    }
    print("Process items:", process_items(items))
    
    print("Stress test:", stress_test())
    print("Nested calls:", nested_calls())
    print("Variables:", test_variables())
    print("Deep recursion:", deep_recursion(5, 0))
    
    local ok, err = test_error_handling()
    print("Error test:", ok, err)
    
    print("=== Debug Showcase Complete ===")
end

-- Entry point
main()