-- Test tool directly without helpers
print("Testing tools directly...")

-- Test 1: UUID Generator
print("\n1. UUID Generator:")
local uuid_tool = Tool.get("uuid_generator")
if uuid_tool then
    local result = uuid_tool:execute({
        operation = "generate",
        version = "v4"
    })
    print("  Success: " .. tostring(result.success))
    print("  Output: " .. (result.output or "nil"))
    if result.error then
        print("  Error: " .. result.error)
    end
else
    print("  ERROR: Could not get uuid_generator tool")
end

-- Test 2: Base64 Encoder
print("\n2. Base64 Encoder:")
local base64_tool = Tool.get("base64_encoder")
if base64_tool then
    local result = base64_tool:execute({
        operation = "encode",
        input = "Hello, World!"
    })
    print("  Success: " .. tostring(result.success))
    print("  Output: " .. (result.output or "nil"))
    if result.error then
        print("  Error: " .. result.error)
    end
else
    print("  ERROR: Could not get base64_encoder tool")
end

-- Test 3: List all tools
print("\n3. Available tools:")
local tools = Tool.list()
print("  Found " .. #tools .. " tools")
for i, tool_name in ipairs(tools) do
    if i <= 5 then  -- Show first 5
        print("  - " .. tool_name)
    end
end
if #tools > 5 then
    print("  ... and " .. (#tools - 5) .. " more")
end