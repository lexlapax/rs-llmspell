#!/usr/bin/env llmspell

--[[
Code Generator Template Example

Demonstrates code generation from specifications.

Note: This is a Phase 12.4.3 placeholder implementation.
Full code generation capabilities will be implemented in future phases.

Run: llmspell lua examples/templates/codegen/lua-basic.lua
--]]

print("====================================")
print("  Code Generator Template Demo")
print("====================================\n")

-- Example parameters for code generator template
-- Check actual schema with: Template.schema("code-generator")
local params = {
    -- Add template-specific parameters here
    -- The actual schema will vary based on implementation
}

print("Executing code-generator template...")
print("Parameters: (check schema for available options)")
print()

local success, output = pcall(function()
    return Template.execute("code-generator", params)
end)

if not success then
    print(string.format("ERROR: %s", output))
    print("\nTip: Use Template.schema('code-generator') to see required parameters")
    os.exit(1)
end

print(string.rep("=", 50))
print("GENERATED CODE")
print(string.rep("=", 50))
print(output.result)

if output.metrics then
    print("\n" .. string.rep("=", 50))
    print("METRICS")
    print(string.rep("=", 50))
    print(string.format("Duration: %dms", output.metrics.duration_ms or 0))
end

print("\n\n====================================")
print("  Code Generation Complete")
print("====================================")
