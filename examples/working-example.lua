-- Working Example: Tools and Agents in LLMSpell
-- This example demonstrates the core functionality without requiring LLM providers

print("=== LLMSpell Working Example ===\n")

-- Part 1: Working with Tools
print("1. TOOLS DEMONSTRATION")
print("----------------------")

-- List available tools
local tools = Tool.list()
print("Available tools: " .. #tools)
print("First 10 tools:")
for i = 1, math.min(10, #tools) do
    print("  - " .. tools[i])
end

-- Use the calculator tool
print("\n2. CALCULATOR TOOL")
print("------------------")
local calc = Tool.get("calculator")
if calc then
    print("✓ Calculator tool loaded")
    
    -- Perform calculations
    local expressions = {
        "2 + 2",
        "10 * 5",
        "100 / 4",
        "2 ^ 8",
        "sqrt(16)",
        "sin(0)"
    }
    
    for _, expr in ipairs(expressions) do
        local result = calc:execute({ input = expr })
        local value = result.result and result.result.result or "error"
        print(string.format("  %s = %s", expr, tostring(value)))
    end
end

-- Use the UUID generator
print("\n3. UUID GENERATOR")
print("-----------------")
local uuid_gen = Tool.get("uuid_generator")
if uuid_gen then
    print("✓ UUID generator loaded")
    local result = uuid_gen:execute({ input = "" })
    print("  Generated UUID: " .. (result.uuid or "error"))
end

-- Use the hash calculator
print("\n4. HASH CALCULATOR")
print("------------------")
local hasher = Tool.get("hash_calculator")
if hasher then
    print("✓ Hash calculator loaded")
    local result = hasher:execute({ 
        input = "Hello, LLMSpell!",
        algorithm = "sha256"
    })
    print("  SHA256 of 'Hello, LLMSpell!': " .. (result.hash or "error"))
end

-- Part 2: Agent Templates
print("\n5. AGENT TEMPLATES")
print("------------------")
local templates = Agent.listTemplates()
print("Available agent templates:")
for _, template in ipairs(templates) do
    print("  - " .. template)
end

-- Part 3: Working with State
print("\n6. STATE MANAGEMENT")
print("-------------------")
if State then
    print("✓ State global available")
    
    -- Store some values
    State.set("counter", 0)
    State.set("user_name", "LLMSpell User")
    State.set("config", {
        theme = "dark",
        language = "en",
        debug = false
    })
    
    -- Retrieve and display
    print("  Stored values:")
    print("    counter: " .. tostring(State.get("counter")))
    print("    user_name: " .. State.get("user_name"))
    
    local config = State.get("config")
    if config then
        print("    config.theme: " .. config.theme)
    end
    
    -- List all keys
    local keys = State.list()
    print("  All state keys: " .. table.concat(keys, ", "))
end

-- Part 4: JSON Operations
print("\n7. JSON OPERATIONS")
print("------------------")
if JSON then
    print("✓ JSON global available")
    
    local data = {
        name = "Test Object",
        value = 42,
        nested = {
            array = {1, 2, 3},
            flag = true
        }
    }
    
    -- Stringify
    local json_str = JSON.stringify(data)
    print("  Stringified: " .. string.sub(json_str, 1, 50) .. "...")
    
    -- Parse
    local parsed = JSON.parse(json_str)
    print("  Parsed back: name = " .. parsed.name .. ", value = " .. parsed.value)
end

-- Part 5: Utils
print("\n8. UTILITIES")
print("------------")
if Utils then
    print("✓ Utils global available")
    
    -- Generate random ID
    local id = Utils.generateId()
    print("  Generated ID: " .. id)
    
    -- Format timestamp
    local timestamp = Utils.timestamp()
    print("  Current timestamp: " .. timestamp)
end

print("\n=== Example Complete ===")
print("This example demonstrated LLMSpell's core functionality")
print("without requiring LLM provider configuration.")