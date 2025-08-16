-- LLMSpell Demo - Core Functionality
-- This demo shows working features of LLMSpell without requiring LLM configuration

print("╔═══════════════════════════════════════╗")
print("║       LLMSpell Demo - Phase 3         ║")
print("╚═══════════════════════════════════════╝\n")

-- Helper to parse tool output
function parseToolOutput(result)
    if result.output then
        local ok, parsed = pcall(JSON.parse, result.output)
        if ok then return parsed end
    end
    return nil
end

-- 1. CALCULATOR DEMO
print("1️⃣  CALCULATOR TOOL")
print("─────────────────")
local calc = Tool.get("calculator")
if calc then
    local tests = {
        "42 + 13",
        "100 - 25", 
        "7 * 8",
        "144 / 12",
        "2 ^ 10",
        "sqrt(64)"
    }
    
    for _, expr in ipairs(tests) do
        local result = calc:execute({ input = expr })
        local parsed = parseToolOutput(result)
        if parsed and parsed.result then
            print(string.format("   %s = %.1f", expr, parsed.result.result))
        end
    end
end

-- 2. UUID GENERATOR
print("\n2️⃣  UUID GENERATOR")
print("─────────────────")
local uuid = Tool.get("uuid_generator")
if uuid then
    for i = 1, 3 do
        local result = uuid:execute({ input = "v4" })
        local parsed = parseToolOutput(result)
        if parsed and parsed.uuid then
            print("   UUID #" .. i .. ": " .. parsed.uuid)
        end
    end
end

-- 3. BASE64 ENCODING
print("\n3️⃣  BASE64 ENCODER")
print("─────────────────")
local b64 = Tool.get("base64_encoder")
if b64 then
    local text = "LLMSpell rocks! 🚀"
    
    -- Encode
    local enc_result = b64:execute({ input = text, operation = "encode" })
    local enc_parsed = parseToolOutput(enc_result)
    
    if enc_parsed and enc_parsed.encoded then
        print("   Original: " .. text)
        print("   Encoded:  " .. enc_parsed.encoded)
        
        -- Decode back
        local dec_result = b64:execute({ 
            input = enc_parsed.encoded, 
            operation = "decode" 
        })
        local dec_parsed = parseToolOutput(dec_result)
        if dec_parsed and dec_parsed.decoded then
            print("   Decoded:  " .. dec_parsed.decoded)
        end
    end
end

-- 4. HASH CALCULATOR
print("\n4️⃣  HASH CALCULATOR")
print("─────────────────")
local hasher = Tool.get("hash_calculator")
if hasher then
    local text = "Hello, World!"
    local algorithms = {"md5", "sha256"}
    
    print("   Text: \"" .. text .. "\"")
    for _, algo in ipairs(algorithms) do
        local result = hasher:execute({ 
            operation = "hash",
            input = text,
            algorithm = algo 
        })
        local parsed = parseToolOutput(result)
        if parsed and parsed.hash then
            print("   " .. string.upper(algo) .. ": " .. 
                  string.sub(parsed.hash, 1, 16) .. "...")
        end
    end
end

-- 5. REGEX MATCHER
print("\n5️⃣  REGEX MATCHER")
print("───────────────")
local regex = Tool.get("regex_matcher")
if regex then
    local result = regex:execute({
        input = "Contact: john@example.com or call 555-1234",
        pattern = "\\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\\.[A-Z|a-z]{2,}\\b"
    })
    local parsed = parseToolOutput(result)
    if parsed and parsed.matches then
        print("   Found email: " .. parsed.matches[1])
    end
end

-- 6. AGENT SYSTEM
print("\n6️⃣  AGENT SYSTEM")
print("──────────────")
-- Check if Agent API is available
if Agent and Agent.discover then
    local agent_types = Agent.discover()
    if agent_types and #agent_types > 0 then
        local type_names = {}
        for _, agent_type in ipairs(agent_types) do
            if type(agent_type) == "table" and agent_type.name then
                table.insert(type_names, agent_type.name)
            elseif type(agent_type) == "string" then
                table.insert(type_names, agent_type)
            end
        end
        if #type_names > 0 then
            print("   Available types: " .. table.concat(type_names, ", "))
        else
            print("   Available types: (none discovered)")
        end
    else
        print("   Available types: (none discovered)")
    end
    print("   Active instances: " .. #Agent.list())
else
    print("   Agent system not available")
end

-- 7. JSON OPERATIONS
print("\n7️⃣  JSON OPERATIONS")
print("─────────────────")
local data = {
    name = "LLMSpell",
    version = "0.3.0",
    phase = 3,
    features = {
        tools = 34,
        agents = true,
        workflows = "coming soon"
    }
}

local json = JSON.stringify(data)
local parsed = JSON.parse(json)
print("   Serialization: ✓")
print("   Parsing: ✓")
print("   Tools available: " .. parsed.features.tools)

-- 8. AVAILABLE TOOLS
print("\n8️⃣  TOOL INVENTORY")
print("────────────────")
local all_tools = Tool.list()
print("   Total tools: " .. #all_tools)
print("   Categories: System, Utility, Data, Media, Web, Security")

-- Show a few interesting tools
local interesting = {
    "process_executor", "file_watcher", "image_processor",
    "archive_handler", "template_engine", "webhook-caller"
}
print("\n   Interesting tools:")
for _, name in ipairs(interesting) do
    if Tool.exists(name) then
        print("   ✓ " .. name)
    end
end

print("\n╔═══════════════════════════════════════╗")
print("║          Demo Complete! 🎉            ║")
print("╚═══════════════════════════════════════╝")
print("\nThis demo showed LLMSpell's capabilities without")
print("requiring LLM provider configuration.")
print("\nTo enable LLM agents, configure providers in")
print("your llmspell.toml file.")