-- Working Example: Tools and Agents in LLMSpell (Fixed)
-- This example demonstrates the core functionality without requiring LLM providers

print("=== LLMSpell Working Example ===\n")

-- Helper function to extract tool results
function extractToolResult(result)
    if result.output then
        -- Parse the JSON output
        local success, parsed = pcall(JSON.parse, result.output)
        if success and parsed.result then
            return parsed.result.result
        end
    end
    return nil
end

-- Part 1: Working with Tools
print("1. TOOLS DEMONSTRATION")
print("----------------------")

-- List available tools
local tools = Tool.list()
print("Available tools: " .. #tools)
print("Sample tools:")
local sample_tools = {"calculator", "uuid_generator", "hash_calculator", "base64_encoder", "regex_matcher"}
for _, name in ipairs(sample_tools) do
    print("  - " .. name)
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
        "pi * 2"
    }
    
    for _, expr in ipairs(expressions) do
        local result = calc:execute({ input = expr })
        local value = extractToolResult(result)
        if value then
            print(string.format("  %s = %s", expr, tostring(value)))
        else
            print(string.format("  %s = error", expr))
        end
    end
end

-- Use the UUID generator
print("\n3. UUID GENERATOR")
print("-----------------")
local uuid_gen = Tool.get("uuid_generator")
if uuid_gen then
    print("✓ UUID generator loaded")
    local result = uuid_gen:execute({ input = "v4" })
    
    -- Parse the output
    if result.output then
        local success, parsed = pcall(JSON.parse, result.output)
        if success and parsed.uuid then
            print("  Generated UUID: " .. parsed.uuid)
            print("  Version: " .. (parsed.version or "unknown"))
        end
    end
end

-- Use the base64 encoder
print("\n4. BASE64 ENCODER")
print("-----------------")
local b64 = Tool.get("base64_encoder")
if b64 then
    print("✓ Base64 encoder loaded")
    
    -- Encode
    local encode_result = b64:execute({ 
        input = "Hello, LLMSpell!",
        operation = "encode"
    })
    if encode_result.output then
        local success, parsed = pcall(JSON.parse, encode_result.output)
        if success and parsed.encoded then
            print("  Encoded: " .. parsed.encoded)
            
            -- Decode it back
            local decode_result = b64:execute({
                input = parsed.encoded,
                operation = "decode"
            })
            if decode_result.output then
                local success2, parsed2 = pcall(JSON.parse, decode_result.output)
                if success2 and parsed2.decoded then
                    print("  Decoded: " .. parsed2.decoded)
                end
            end
        end
    end
end

-- Part 2: Agent Templates
print("\n5. AGENT TEMPLATES")
print("------------------")
local templates = Agent.listTemplates()
print("Available agent templates:")
for _, template in ipairs(templates) do
    print("  - " .. template)
end

-- List active agent instances
local instances = Agent.list()
print("\nActive agent instances: " .. #instances)

-- Part 3: Working with State
print("\n6. STATE MANAGEMENT")
print("-------------------")
if State then
    print("✓ State global available")
    
    -- Store some values
    State.set("example_counter", 42)
    State.set("example_message", "Hello from LLMSpell!")
    State.set("example_data", {
        timestamp = os.time(),
        user = "demo",
        flags = {debug = false, verbose = true}
    })
    
    -- Retrieve and display
    print("  Stored values:")
    print("    counter: " .. tostring(State.get("example_counter")))
    print("    message: " .. State.get("example_message"))
    
    local data = State.get("example_data")
    if data then
        print("    data.user: " .. data.user)
        print("    data.flags.verbose: " .. tostring(data.flags.verbose))
    end
    
    -- Clean up
    State.delete("example_counter")
    State.delete("example_message")
    State.delete("example_data")
    print("  (State cleaned up)")
else
    print("✗ State global not available")
end

-- Part 4: JSON Operations
print("\n7. JSON OPERATIONS")
print("------------------")
if JSON then
    print("✓ JSON global available")
    
    local test_data = {
        title = "LLMSpell Demo",
        version = "0.3.0",
        features = {"tools", "agents", "workflows"},
        metadata = {
            created = os.time(),
            author = "system"
        }
    }
    
    -- Stringify with pretty print
    local json_str = JSON.stringify(test_data)
    print("  JSON length: " .. #json_str .. " characters")
    
    -- Parse and verify
    local parsed = JSON.parse(json_str)
    print("  Parsed title: " .. parsed.title)
    print("  Feature count: " .. #parsed.features)
end

-- Part 5: Tool Categories
print("\n8. TOOL CATEGORIES")
print("------------------")
local categories = Tool.categories()
print("Available tool categories: " .. #categories)
for _, cat in ipairs(categories) do
    print("  - " .. cat)
end

print("\n=== Example Complete ===")
print("This example demonstrated LLMSpell's core functionality")
print("without requiring LLM provider configuration.")
print("\nNext steps:")
print("- Configure providers in llmspell.toml to enable LLM agents")
print("- Explore workflow creation with Workflow global")
print("- Check examples/ directory for more advanced usage")