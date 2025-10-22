#!/usr/bin/env llmspell

--[[
Research Assistant Template Example

Demonstrates executing the Research Assistant template which:
- Gathers sources from web search (placeholder)
- Ingests them into RAG (placeholder)
- Synthesizes findings with an AI agent (placeholder)
- Validates citations (placeholder)

Parameters:
  - topic (required): Research topic or question
  - max_sources (optional): Max sources to gather (1-50, default: 10)
  - model (optional): LLM model (default: "ollama/llama3.2:3b")
  - output_format (optional): markdown|json|html (default: "markdown")
  - include_citations (optional): Include citations (default: true)

Run: llmspell lua examples/templates/research/lua-basic.lua
--]]

print("====================================")
print("  Research Assistant Template Demo")
print("====================================\n")

-- Execute template with basic parameters
print("Executing research-assistant template...\n")

local params = {
    topic = "Rust async programming best practices",
    max_sources = 5,
    model = "ollama/llama3.2:3b",
    output_format = "markdown",
    include_citations = true
}

print("Parameters:")
for key, value in pairs(params) do
    print(string.format("  %s: %s", key, tostring(value)))
end
print()

-- Execute the template (async operation)
local success, output = pcall(function()
    return Template.execute("research-assistant", params)
end)

if not success then
    print(string.format("ERROR: Template execution failed: %s", output))
    os.exit(1)
end

-- Inspect the result
print(string.rep("=", 50))
print("EXECUTION RESULT")
print(string.rep("=", 50))

print(string.format("\nResult Type: %s", output.result_type))

if output.result then
    print("\n--- Research Report ---")
    print(output.result)
end

-- Show execution metrics
if output.metrics then
    print("\n" .. string.rep("=", 50))
    print("EXECUTION METRICS")
    print(string.rep("=", 50))
    print(string.format("Duration: %dms", output.metrics.duration_ms or 0))
    print(string.format("Agents Invoked: %d", output.metrics.agents_invoked or 0))
    print(string.format("Tools Invoked: %d", output.metrics.tools_invoked or 0))
    print(string.format("RAG Queries: %d", output.metrics.rag_queries or 0))

    -- Show custom metrics if any
    if output.metrics.custom then
        print("\nCustom Metrics:")
        for key, value in pairs(output.metrics.custom) do
            print(string.format("  %s: %s", key, tostring(value)))
        end
    end
end

-- Show artifacts if any
if output.artifacts and #output.artifacts > 0 then
    print("\n" .. string.rep("=", 50))
    print(string.format("ARTIFACTS (%d files)", #output.artifacts))
    print(string.rep("=", 50))
    for i, artifact in ipairs(output.artifacts) do
        print(string.format("\n%d. %s", i, artifact.path or "unknown"))
        print(string.format("   MIME Type: %s", artifact.mime_type or "unknown"))
        print(string.format("   Size: %d bytes", #(artifact.content or "")))
    end
end

print("\n\n====================================")
print("  Research Complete")
print("====================================")
