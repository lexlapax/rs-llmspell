-- test-api.lua
-- Test what APIs are available in the current Lua environment

print("🔍 Testing Available APIs")
print("========================")

-- Test global variables
print("\nGlobal variables:")
for k, v in pairs(_G) do
    if not string.match(k, "^[_%a][_%w]*$") or k == "_G" then
        -- Skip standard Lua globals
    else
        print(string.format("  %s: %s", k, type(v)))
    end
end

-- Test if Agent is available
if Agent then
    print("\n✅ Agent API is available")
    print("Agent type:", type(Agent))
    
    -- Test Agent methods
    if Agent.create then
        print("  Agent.create: available")
    end
    
    if Agent.list then
        print("  Agent.list: available")
    end
else
    print("\n❌ Agent API is not available")
end

-- Test if Provider is available  
if Provider then
    print("\n✅ Provider API is available")
    print("Provider type:", type(Provider))
    
    if Provider.list then
        print("  Provider.list: available")
    end
else
    print("\n❌ Provider API is not available")
end

-- Test if Tool is available
if Tool then
    print("\n✅ Tool API is available")
    print("Tool type:", type(Tool))
else
    print("\n❌ Tool API is not available")
end

-- Test standard Lua functionality
print("\n📋 Standard Lua test:")
local test_table = {a = 1, b = 2, c = 3}
print("Table test:", test_table.a + test_table.b + test_table.c)

local function test_function(x)
    return x * 2
end
print("Function test:", test_function(21))

return {
    status = "api_test_complete",
    agent_available = Agent ~= nil,
    provider_available = Provider ~= nil,
    tool_available = Tool ~= nil,
    lua_version = _VERSION
}