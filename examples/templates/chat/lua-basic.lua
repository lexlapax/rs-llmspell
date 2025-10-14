#!/usr/bin/env llmspell

--[[
Interactive Chat Template Example

Demonstrates programmatic chat execution (single message mode).
For interactive stdin mode, omit the 'message' parameter.

Parameters:
  - model (optional): LLM model (default: "ollama/llama3.2:3b")
  - system_prompt (optional): System context (default: helpful assistant)
  - max_turns (optional): Max conversation turns 1-100 (default: 10)
  - tools (optional): Array of tool names (default: [])
  - enable_memory (optional): Enable Phase 13 memory (default: false)
  - message (optional): Single message for programmatic mode

Run: llmspell lua examples/templates/chat/lua-basic.lua
--]]

print("=================================")
print("  Interactive Chat Template Demo")
print("=================================\n")

local params = {
    model = "ollama/llama3.2:3b",
    system_prompt = "You are a helpful coding assistant specializing in Rust.",
    message = "Explain the difference between String and &str in Rust",
    max_turns = 1,
    tools = {},
    enable_memory = false
}

print("Parameters:")
for key, value in pairs(params) do
    if type(value) == "table" then
        print(string.format("  %s: [%s]", key, table.concat(value, ", ")))
    else
        print(string.format("  %s: %s", key, tostring(value)))
    end
end
print()

local success, output = pcall(function()
    return Template.execute("interactive-chat", params)
end)

if not success then
    print(string.format("ERROR: %s", output))
    os.exit(1)
end

print(string.rep("=", 50))
print("CHAT RESPONSE")
print(string.rep("=", 50))
print(output.result)

if output.metrics then
    print("\n" .. string.rep("=", 50))
    print("METRICS")
    print(string.rep("=", 50))
    print(string.format("Duration: %dms", output.metrics.duration_ms or 0))
    print(string.format("Turns: %d", output.metrics.custom and output.metrics.custom.turn_count or 0))
    print(string.format("Total Tokens: %d", output.metrics.custom and output.metrics.custom.total_tokens or 0))
end

print("\n\n=================================")
print("  Chat Complete")
print("=================================")
