-- Basic Debug Infrastructure Example
-- Simple introduction to debug features

print("🔧 LLMSpell Debug - Basic Example")
print("=" .. string.rep("=", 40))

-- 1. Simple logging
print("\n📋 Basic Logging")
Debug.info("Application started", "app")
Debug.warn("Low memory warning", "system")
Debug.error("Failed to connect to service", "network")

-- 2. Simple timing
print("\n⏱️  Basic Timing")
local timer = Debug.timer("operation")

-- Simulate some work
local result = 0
for i = 1, 1000000 do
    result = result + i
end

local duration = timer:stop()
Debug.info("Operation completed in " .. string.format("%.2f", duration) .. "ms", "performance")
print("Result: " .. result)

-- 3. Simple object dumping
print("\n📦 Object Dumping")
local data = {
    name = "Example",
    count = 42,
    items = {"apple", "banana", "cherry"}
}

print("Data structure:")
print(Debug.dump(data))

-- 4. Check debug state
print("\n🔧 Debug State")
print("Debug level: " .. Debug.getLevel())
print("Debug enabled: " .. tostring(Debug.isEnabled()))

print("\n✅ Basic example complete!")