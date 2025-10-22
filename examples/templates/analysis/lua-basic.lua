#!/usr/bin/env llmspell

--[[
Data Analysis Template Example

Demonstrates statistical analysis and visualization template.

Note: This is a Phase 12.4.2 placeholder implementation.
Full data analysis capabilities will be implemented in future phases.

Run: llmspell lua examples/templates/analysis/lua-basic.lua
--]]

print("====================================")
print("  Data Analysis Template Demo")
print("====================================\n")

-- Example parameters for data analysis template
-- Check actual schema with: Template.schema("data-analysis")
local params = {
    -- Add template-specific parameters here
    -- The actual schema will vary based on implementation
}

print("Executing data-analysis template...")
print("Parameters: (check schema for available options)")
print()

local success, output = pcall(function()
    return Template.execute("data-analysis", params)
end)

if not success then
    print(string.format("ERROR: %s", output))
    print("\nTip: Use Template.schema('data-analysis') to see required parameters")
    os.exit(1)
end

print(string.rep("=", 50))
print("ANALYSIS RESULT")
print(string.rep("=", 50))
print(output.result)

if output.metrics then
    print("\n" .. string.rep("=", 50))
    print("METRICS")
    print(string.rep("=", 50))
    print(string.format("Duration: %dms", output.metrics.duration_ms or 0))
end

print("\n\n====================================")
print("  Analysis Complete")
print("====================================")
