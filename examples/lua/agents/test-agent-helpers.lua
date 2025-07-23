-- Test agent helpers loading
print("Testing agent helpers...")

local helpers = dofile("agent-helpers.lua")
print("Helpers loaded:", helpers ~= nil)

if helpers then
    print("Available functions:")
    for k, v in pairs(helpers) do
        print("  -", k, type(v))
    end
end

print("Test complete.")