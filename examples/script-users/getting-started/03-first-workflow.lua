-- ============================================================
-- LLMSPELL GETTING STARTED SHOWCASE
-- ============================================================
-- Example ID: 03 - First Workflow v0.7.0
-- Complexity Level: BEGINNER
-- Real-World Use Case: Multi-step automation and process orchestration
--
-- Purpose: Learn workflow creation and execution. Demonstrates how to chain
--          multiple tools together in a sequential workflow, pass data between
--          steps, and handle workflow results. Foundation for complex automation.
-- Architecture: Sequential workflow pattern with step chaining
-- Crates Showcased: llmspell-workflows, llmspell-tools, llmspell-bridge
-- Key Features:
--   • Workflow builder pattern
--   • Sequential step execution
--   • Multiple tool orchestration
--   • Data flow between steps
--   • Result aggregation
--
-- Prerequisites:
--   • LLMSpell installed and built
--   • Write access to /tmp directory
--   • No API keys required
--
-- HOW TO RUN:
-- ./target/debug/llmspell run examples/script-users/getting-started/03-first-workflow.lua
--
-- EXPECTED OUTPUT:
-- Workflow created with 4 steps
-- Step 1: Generated UUID v4
-- Step 2: Got current timestamp
-- Step 3: Calculated hash
-- Step 4: Created summary file at /tmp/workflow_[uuid].txt
--
-- Time to Complete: <20 milliseconds
-- ============================================================

print("=== LLMSpell: Your First Workflow ===")
print("Example 03: BEGINNER - Creating multi-step workflows")
print("Showcasing: Sequential workflow orchestration\n")

print("1. Creating a simple multi-step workflow...")
print("   This workflow will:")
print("   - Generate a unique ID")
print("   - Get current timestamp")
print("   - Calculate a hash")
print("   - Create a summary file")

-- Create a sequential workflow using the builder pattern
-- The builder API allows method chaining for configuration
local workflow = Workflow.builder()
    :name("first_workflow")
    :description("My first workflow example")
    :sequential()  -- Sets the workflow type to sequential
    :add_step({
        name = "generate_id",
        type = "tool",  -- Step type must be specified
        tool = "uuid-generator",  -- Tool to invoke
        input = {  -- Parameters for the tool
            operation = "generate",
            version = "v4"
        }
    })
    :add_step({
        name = "get_timestamp",
        type = "tool",
        tool = "datetime-handler",
        input = {
            operation = "now"
        }
    })
    :add_step({
        name = "calculate_hash",
        type = "tool",
        tool = "hash-calculator",
        input = {
            operation = "hash",
            algorithm = "sha256",
            input = "workflow_started"
        }
    })
    :add_step({
        name = "create_summary",
        type = "tool",
        tool = "file-operations",
        input = {
            operation = "write",
            path = "/tmp/workflow_summary.txt",
            input = "Workflow execution summary created by first_workflow"
        }
    })
    :build()

if not workflow then
    print("❌ Error creating workflow")
    return
end

print("✅ Workflow created with 4 steps!")
print("   Workflow ID: " .. (workflow.id or "unknown"))
print("   Workflow name: " .. (workflow.name or "unknown"))
print("   Workflow type: " .. (workflow.type or "unknown"))

print()
print("2. Executing the workflow...")

-- Execute the workflow (workflows use execute() method, not run())
local success, execution_result = pcall(function()
    return workflow:execute({})  -- Pass empty table as input
end)

if success and execution_result then
    print("✅ Workflow executed successfully!")
    print("📊 Execution summary:")
    
    -- Display execution results
    if type(execution_result) == "table" then
        for k, v in pairs(execution_result) do
            local value_str = type(v) == "table" and "[table]" or tostring(v)
            print("   " .. k .. ": " .. value_str)
        end
    end
    
    -- Check if the summary file was created
    local file_check = Tool.execute("file-operations", {
        operation = "exists",
        path = "/tmp/workflow_summary.txt"
    })

    if file_check and file_check.text then
        print("   ✅ Summary file created: /tmp/workflow_summary.txt")

        -- Read and display the summary
        local file_content = Tool.execute("file-operations", {
            operation = "read",
            path = "/tmp/workflow_summary.txt"
        })
        
        if file_content and file_content.text then
            print("   📝 File content: " .. file_content.text)
        end
    end
else
    print("❌ Workflow execution failed: " .. tostring(execution_result))
end

print()
print("🎉 Congratulations! You've successfully:")
print("   - Created a sequential workflow using builder pattern")
print("   - Added multiple tool steps")
print("   - Executed a complete workflow")
print("   - Chained tool operations together")
print()
print("💡 Key concepts learned:")
print("   - Workflows automate multi-step processes")
print("   - Sequential workflows execute steps in order")
print("   - The builder pattern provides a fluent API")
print("   - Each step can be a tool, agent, or nested workflow")
print()
print("Next: Try 04-handle-errors.lua to learn about error handling!")