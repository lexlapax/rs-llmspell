-- ABOUTME: Debug script to understand Agent table structure
-- ABOUTME: Lists all keys and values in the Agent global table

print("=== Agent Table Debug ===")
print()

if Agent == nil then
    print("❌ Agent global is nil!")
    os.exit(1)
end

print("Agent global type:", type(Agent))
print()

-- List all keys in Agent table
print("Keys in Agent table:")
local keys = {}
for k, v in pairs(Agent) do
    table.insert(keys, {key = k, type = type(v)})
end

-- Sort keys alphabetically
table.sort(keys, function(a, b) return a.key < b.key end)

-- Print each key and its type
for _, item in ipairs(keys) do
    print(string.format("  %s: %s", item.key, item.type))
end

print()
print("Total methods found:", #keys)