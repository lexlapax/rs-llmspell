-- Example: 03-first-workflow.lua
-- Author: LLMSpell Examples
-- Purpose: First introduction to workflows - chaining steps together
-- Learning: Sequential workflow creation and execution

print("=== LLMSpell: Your First Workflow ===")
print("This example shows how to create a workflow that chains multiple steps!")
print()

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
        tool = "uuid_generator",  -- Tool to invoke
        input = {  -- Parameters for the tool
            operation = "generate",
            version = "v4"
        }
    })
    :add_step({
        name = "get_timestamp",
        type = "tool",
        tool = "date_time_handler",
        input = {
            operation = "now"
        }
    })
    :add_step({
        name = "calculate_hash",
        type = "tool",
        tool = "hash_calculator",
        input = {
            operation = "hash",
            algorithm = "sha256",
            input = "workflow_started"
        }
    })
    :add_step({
        name = "create_summary",
        type = "tool",
        tool = "file_operations",
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
    local file_check = Tool.invoke("file_operations", {
        operation = "exists",
        path = "/tmp/workflow_summary.txt"
    })
    
    if file_check and file_check.text then
        print("   ✅ Summary file created: /tmp/workflow_summary.txt")
        
        -- Read and display the summary
        local file_content = Tool.invoke("file_operations", {
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
print("   - Each step can be a tool, agent, or custom function")
print()
print("Next: Try 04-save-state.lua to learn about state persistence!")