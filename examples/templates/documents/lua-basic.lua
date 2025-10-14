#!/usr/bin/env llmspell

--[[
Document Processor Template Example

Demonstrates PDF/OCR extraction and document transformation.

Note: This is a Phase 12.4.4 placeholder implementation.
Full document processing capabilities will be implemented in future phases.

Run: llmspell lua examples/templates/documents/lua-basic.lua
--]]

print("====================================")
print("  Document Processor Template Demo")
print("====================================\n")

-- Example parameters for document processor template
-- Check actual schema with: Template.schema("document-processor")
local params = {
    -- Add template-specific parameters here
    -- The actual schema will vary based on implementation
}

print("Executing document-processor template...")
print("Parameters: (check schema for available options)")
print()

local success, output = pcall(function()
    return Template.execute("document-processor", params)
end)

if not success then
    print(string.format("ERROR: %s", output))
    print("\nTip: Use Template.schema('document-processor') to see required parameters")
    os.exit(1)
end

print(string.rep("=", 50))
print("PROCESSED DOCUMENT")
print(string.rep("=", 50))
print(output.result)

if output.metrics then
    print("\n" .. string.rep("=", 50))
    print("METRICS")
    print(string.rep("=", 50))
    print(string.format("Duration: %dms", output.metrics.duration_ms or 0))
end

print("\n\n====================================")
print("  Document Processing Complete")
print("====================================")
