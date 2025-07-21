-- LLMSpell Final Demo - Working Features Only
print("=== LLMSpell Demo (Phase 3.3) ===\n")

-- Helper for tool output
function getToolResult(result)
    if result and result.output then
        local ok, parsed = pcall(JSON.parse, result.output)
        if ok then return parsed end
    end
    return nil
end

-- 1. Calculator Tool
print("📐 CALCULATOR")
local calc = Tool.get("calculator")
if calc then
    print("  5 + 3 = " .. getToolResult(calc:execute({input="5+3"})).result.result)
    print("  10 * 4 = " .. getToolResult(calc:execute({input="10*4"})).result.result)
    print("  sqrt(25) = " .. getToolResult(calc:execute({input="sqrt(25)"})).result.result)
end

-- 2. Tool Discovery
print("\n🔧 AVAILABLE TOOLS (" .. #Tool.list() .. " total)")
local tools = Tool.list()
for i = 1, math.min(5, #tools) do
    print("  - " .. tools[i])
end
print("  ...")

-- 3. Agent System
print("\n🤖 AGENT TEMPLATES")
for _, t in ipairs(Agent.listTemplates()) do
    print("  - " .. t)
end

-- 4. JSON Operations
print("\n📋 JSON TEST")
local obj = {test = true, value = 42}
local json = JSON.stringify(obj)
local back = JSON.parse(json)
print("  Roundtrip: " .. (back.test and "✓ success" or "✗ failed"))

print("\n✅ Demo Complete!")
print("Configure providers in llmspell.toml to enable LLM agents.")