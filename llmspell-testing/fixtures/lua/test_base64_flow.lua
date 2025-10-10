-- Test the exact base64 flow from the example

local TestHelpers = dofile("examples/test-helpers.lua")

local function use_tool(tool_name, params)
    return TestHelpers.execute_tool(tool_name, params)
end

print("Testing Base64 flow from example:")

local original = "Hello, World!"
print("Original:", original)

local encoded = use_tool("base64-encoder", {
    operation = "encode",
    input = original
})

print("\nEncoded result structure:")
print("  Type:", type(encoded))
print("  Success:", encoded.success)
print("  Has result?", encoded.result ~= nil)
print("  Has output?", encoded.output ~= nil)

if encoded.result then
    print("\nencoded.result structure:")
    for k, v in pairs(encoded.result) do
        print("    " .. k .. ":", type(v), tostring(v))
    end
end

-- This is the exact check from the example
if encoded.result and encoded.result.output then
    print("\n✅ Check passed: encoded.result.output exists")
    print("  Value:", encoded.result.output)
else
    print("\n❌ Check failed: encoded.result.output does NOT exist")
    
    -- Let's see what we actually have
    if encoded.output then
        print("\nWe have encoded.output (JSON string):")
        print(encoded.output)
    end
end