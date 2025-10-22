-- Basic Research Assistant Template Example (Lua)
-- NOTE: Requires Template global (Phase 12.5 - not yet implemented)
--
-- This demonstrates the simplest usage with just the required topic parameter

print("Executing Research Assistant with minimal parameters...")

-- Execute template with just the required topic parameter
-- All optional parameters will use their defaults:
-- - max_sources: 10
-- - model: "ollama/llama3.2:3b"
-- - output_format: "markdown"
-- - include_citations: true
local result = Template.execute("research-assistant", {
    topic = "Rust async programming patterns"
})

-- Check if execution was successful
if result.success then
    print("\n✓ Research complete!")
    print("Duration: " .. result.metrics.duration_ms .. "ms")
    print("Agents invoked: " .. result.metrics.agents_invoked)
    print("Tools called: " .. result.metrics.tools_invoked)
    print("RAG queries: " .. result.metrics.rag_queries)
    print("\nResult preview:")
    print(result.result:sub(1, 200) .. "...")
else
    print("✗ Research failed: " .. result.error)
end
