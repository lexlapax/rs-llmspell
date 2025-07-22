-- ABOUTME: Test how Lua tables are converted to JSON
-- ABOUTME: Debug test for Task 3.3.28 array conversion issue

print("=== Testing Lua Table to JSON Conversion ===")
print()

-- Test different ways to create arrays
local empty_table = {}
local empty_array1 = {n = 0}  -- Lua idiom for empty array
local empty_array2 = setmetatable({}, {__len = function() return 0 end})

print("Testing empty table: ", type(empty_table), #empty_table)
print("Testing empty_array1:", type(empty_array1), #empty_array1)
print("Testing empty_array2:", type(empty_array2), #empty_array2)

-- Try with Tool.list() to see what format it returns
if Tool then
    local tools = Tool.list()
    print("\nTool.list() returns:")
    print("  Type:", type(tools))
    print("  Length:", #tools)
    print("  First element:", tools[1])
    
    -- Create a subset
    local subset = {}
    for i = 1, 0 do  -- Create empty array by not adding elements
        subset[i] = tools[i]
    end
    print("\nEmpty subset:")
    print("  Type:", type(subset))
    print("  Length:", #subset)
end

-- Test with actual array syntax
local real_array = {}
real_array[1] = nil  -- This might help?
print("\nArray with nil[1]:")
print("  Type:", type(real_array))
print("  Length:", #real_array)