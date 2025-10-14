#!/usr/bin/env llmspell

--[[
Workflow Orchestrator Template Example

Demonstrates custom agent/tool composition patterns.

Note: This is a Phase 12.4.4 placeholder implementation.
Full workflow orchestration capabilities will be implemented in future phases.

Run: llmspell lua examples/templates/orchestration/lua-basic.lua
--]]

print("====================================")
print("  Workflow Orchestrator Template Demo")
print("====================================\n")

-- Example parameters for workflow orchestrator template
-- Check actual schema with: Template.schema("workflow-orchestrator")
local params = {
    -- Add template-specific parameters here
    -- The actual schema will vary based on implementation
}

print("Executing workflow-orchestrator template...")
print("Parameters: (check schema for available options)")
print()

local success, output = pcall(function()
    return Template.execute("workflow-orchestrator", params)
end)

if not success then
    print(string.format("ERROR: %s", output))
    print("\nTip: Use Template.schema('workflow-orchestrator') to see required parameters")
    os.exit(1)
end

print(string.rep("=", 50))
print("WORKFLOW RESULT")
print(string.rep("=", 50))
print(output.result)

if output.metrics then
    print("\n" .. string.rep("=", 50))
    print("METRICS")
    print(string.rep("=", 50))
    print(string.format("Duration: %dms", output.metrics.duration_ms or 0))
    if output.metrics.custom then
        print("\nCustom Metrics:")
        for key, value in pairs(output.metrics.custom) do
            print(string.format("  %s: %s", key, tostring(value)))
        end
    end
end

print("\n\n====================================")
print("  Workflow Complete")
print("====================================")
