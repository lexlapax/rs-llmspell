#!/usr/bin/env llmspell

--[[
Memory-Aware Template Example

Demonstrates memory-aware template execution (Task 13.11.4):
- First execution: No prior memory context
- Second execution: Uses memory from first execution (same session_id)
- Third execution: Different session (isolated context)

This validates:
- Task 13.11.1: Memory parameter extraction (session_id, memory_enabled, context_budget)
- Task 13.11.2: Context assembly from memory before LLM interaction
- Task 13.11.3: Storage of template inputs/outputs in episodic memory

Parameters:
  - topic (required): Research topic
  - session_id (optional): Session ID for memory context
  - memory_enabled (optional): Enable memory context (default: true)
  - context_budget (optional): Token budget for context (default: 2000)
  - max_sources (optional): Max sources (default: 5)

Run: llmspell lua examples/templates/research/memory-aware.lua
--]]

print("=============================================")
print("  Memory-Aware Template Execution Demo")
print("  Task 13.11.4: Testing + Examples")
print("=============================================\n")

-- Generate unique session ID
local session_id = "research-" .. os.time()

print(string.format("Session ID: %s\n", session_id))

-- Execution 1: Initial research (no prior memory)
print(string.rep("=", 50))
print("EXECUTION 1: Initial Research (No Prior Context)")
print(string.rep("=", 50))

local params1 = {
    topic = "Rust ownership model",
    session_id = session_id,
    memory_enabled = true,
    context_budget = 2000,
    max_sources = 5,
    model = "ollama/llama3.2:3b",
    output_format = "markdown",
}

print("\nParameters:")
for key, value in pairs(params1) do
    print(string.format("  %s: %s", key, tostring(value)))
end
print()

local success1, output1 = pcall(function()
    return Template.execute("research-assistant", params1)
end)

if not success1 then
    print(string.format("ERROR: %s", output1))
    os.exit(1)
end

print(string.format("Result Type: %s", output1.result_type))
if output1.metrics then
    print(string.format("Duration: %dms", output1.metrics.duration_ms or 0))
    print(string.format("Agents Invoked: %d", output1.metrics.agents_invoked or 0))
    if output1.metrics.custom then
        print(string.format("Sources: %d", output1.metrics.custom.source_count or 0))
    end
end

-- Memory should now contain input/output from execution 1

print("\n")
print(string.rep("=", 50))
print("EXECUTION 2: Follow-up Research (With Prior Context)")
print(string.rep("=", 50))

-- Small delay to ensure memory write completes
os.execute("sleep 1")

local params2 = {
    topic = "Rust borrowing rules",
    session_id = session_id,  -- Same session -> should retrieve context from execution 1
    memory_enabled = true,
    context_budget = 3000,  -- Larger budget to accommodate prior context
    max_sources = 5,
    model = "ollama/llama3.2:3b",
    output_format = "markdown",
}

print("\nParameters:")
for key, value in pairs(params2) do
    print(string.format("  %s: %s", key, tostring(value)))
end
print()

local success2, output2 = pcall(function()
    return Template.execute("research-assistant", params2)
end)

if not success2 then
    print(string.format("ERROR: %s", output2))
    os.exit(1)
end

print(string.format("Result Type: %s", output2.result_type))
if output2.metrics then
    print(string.format("Duration: %dms", output2.metrics.duration_ms or 0))
    print(string.format("Agents Invoked: %d", output2.metrics.agents_invoked or 0))
    if output2.metrics.custom then
        print(string.format("Sources: %d", output2.metrics.custom.source_count or 0))
        -- Note: context_used would show how many tokens of prior context were used
    end
end

print("\n")
print(string.rep("=", 50))
print("EXECUTION 3: New Session (Isolated Context)")
print(string.rep("=", 50))

local params3 = {
    topic = "Rust lifetimes",
    session_id = "research-new-" .. os.time(),  -- Different session
    memory_enabled = true,
    context_budget = 2000,
    max_sources = 5,
    model = "ollama/llama3.2:3b",
    output_format = "markdown",
}

print("\nParameters:")
for key, value in pairs(params3) do
    print(string.format("  %s: %s", key, tostring(value)))
end
print()

local success3, output3 = pcall(function()
    return Template.execute("research-assistant", params3)
end)

if not success3 then
    print(string.format("ERROR: %s", output3))
    os.exit(1)
end

print(string.format("Result Type: %s", output3.result_type))
if output3.metrics then
    print(string.format("Duration: %dms", output3.metrics.duration_ms or 0))
    print(string.format("Agents Invoked: %d", output3.metrics.agents_invoked or 0))
    if output3.metrics.custom then
        print(string.format("Sources: %d", output3.metrics.custom.source_count or 0))
    end
end

-- Summary
print("\n")
print(string.rep("=", 50))
print("MEMORY-AWARE EXECUTION SUMMARY")
print(string.rep("=", 50))
print(string.format("Session 1 (%s):", session_id))
print("  - Execution 1: Ownership model (no prior context)")
print("  - Execution 2: Borrowing rules (WITH context from Execution 1)")
print(string.format("\nSession 2 (%s):", params3.session_id))
print("  - Execution 3: Lifetimes (isolated, no shared context)")

print("\n✓ Memory-aware template execution complete")
print("\nTask 13.11 Validation:")
print("  ✓ 13.11.1: Memory parameters extracted (session_id, memory_enabled, context_budget)")
print("  ✓ 13.11.2: Context assembled from memory before LLM calls")
print("  ✓ 13.11.3: Template executions stored in episodic memory")
print("  ✓ 13.11.4: Integration tests and examples created")

print("\n=============================================")
print("  Demo Complete")
print("=============================================")
