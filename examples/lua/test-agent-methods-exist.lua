-- ABOUTME: Simple test to check if new Agent methods exist
-- ABOUTME: Part of Task 3.3.28 testing

print("=== Checking Agent Methods ===")
print()

-- Check Agent exists
if Agent == nil then
    print("❌ Agent global not found!")
    os.exit(1)
end
print("✓ Agent global exists")

-- List of methods to check
local methods = {
    "create", 
    "list", 
    "discover", 
    "register",      -- New in 3.3.28
    "get",          -- New in 3.3.28
    "wrapAsTool",   -- New in 3.3.28
    "getInfo",      -- New in 3.3.28
    "listCapabilities", -- New in 3.3.28
    "createComposite",  -- New in 3.3.28
    "discoverByCapability" -- New in 3.3.28
}

local all_found = true

-- Check each method
for _, method in ipairs(methods) do
    if type(Agent[method]) == "function" then
        print(string.format("✓ Agent.%s() found", method))
    else
        print(string.format("❌ Agent.%s() NOT FOUND (type: %s)", method, type(Agent[method])))
        all_found = false
    end
end

print()
if all_found then
    print("✅ All Agent methods found!")
else
    print("❌ Some Agent methods are missing!")
    os.exit(1)
end