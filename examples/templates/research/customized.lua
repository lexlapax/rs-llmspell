-- Advanced Research Assistant Template Example (Lua)
-- NOTE: Requires Template global (Phase 12.5 - not yet implemented)
--
-- This demonstrates all available parameters with custom configuration

print("Executing Research Assistant with custom parameters...")

-- First, let's inspect the template schema to see available parameters
local info = Template.info("research-assistant")
print("\nTemplate: " .. info.name)
print("Version: " .. info.version)
print("Category: " .. info.category)
print("Description: " .. info.description)

if info.show_schema then
    print("\nAvailable parameters:")
    for _, param in ipairs(info.schema.parameters) do
        print("  - " .. param.name .. " (" .. param.type .. ")")
        if param.required then
            print("    Required")
        else
            print("    Optional, default: " .. tostring(param.default))
        end
    end
end

-- Execute template with all custom parameters
-- This shows the full power of the template with fine-grained control
local result = Template.execute("research-assistant", {
    -- Required parameter
    topic = "Machine learning model interpretability techniques",

    -- Optional: Limit number of sources for faster execution
    max_sources = 5,

    -- Optional: Use smaller/faster model
    model = "ollama/llama3.2:1b",

    -- Optional: Get structured JSON output for programmatic processing
    output_format = "json",

    -- Optional: Disable citations for cleaner output
    include_citations = false
})

-- Process results
if result.success then
    print("\n✓ Research complete with custom configuration!")
    print("\nExecution metrics:")
    print("  Duration: " .. result.metrics.duration_ms .. "ms")
    print("  Agents invoked: " .. result.metrics.agents_invoked)
    print("  Tools called: " .. result.metrics.tools_invoked)
    print("  RAG queries: " .. result.metrics.rag_queries)

    -- Parse JSON result if output_format was "json"
    if result.result_type == "Structured" then
        print("\nStructured output received (JSON)")
        print("  Topic: " .. result.result.topic)
        print("  Sources: " .. #result.result.sources)

        -- Save to file for further processing
        local file = io.open("research_output.json", "w")
        if file then
            file:write(JSON.encode(result.result))
            file:close()
            print("\n✓ Saved structured output to: research_output.json")
        end
    else
        -- Text output (markdown or html)
        print("\nText output received")
        print(result.result:sub(1, 300) .. "...")
    end

    -- List generated artifacts
    if result.artifacts and #result.artifacts > 0 then
        print("\nGenerated artifacts:")
        for _, artifact in ipairs(result.artifacts) do
            print("  - " .. artifact.filename .. " (" .. artifact.size .. " bytes)")
        end
    end
else
    print("✗ Research failed: " .. result.error)
end

print("\n" .. string.rep("-", 60))
print("Example complete! See output above for results.")
