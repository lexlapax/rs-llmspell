-- Example: 03-first-workflow.lua
-- Author: LLMSpell Examples
-- Purpose: First introduction to workflows - chaining steps together
-- Learning: Sequential workflow creation and execution

print("=== LLMSpell: Your First Workflow ===")
print("This example shows how to create a workflow that chains multiple steps!")
print()

-- Check providers first
local providers = Provider.list()
if #providers == 0 then
    print("❌ No providers configured. Please check your configuration.")
    return
end

print("1. Creating a multi-step workflow...")
print("   This workflow will:")
print("   - Create a text file")
print("   - Ask an agent to analyze it")
print("   - Save the analysis to another file")

-- Create a sequential workflow
local workflow_result = Workflow.sequential({
    name = "first_workflow",
    description = "My first workflow example"
})

if not workflow_result.success then
    print("❌ Error creating workflow: " .. (workflow_result.error or "Unknown error"))
    return
end

local workflow = workflow_result.result
print("✅ Workflow created successfully!")

print()
print("2. Adding workflow steps...")

-- Step 1: Create a test file
local step1_result = workflow:add_step({
    name = "create_file",
    type = "tool",
    tool_name = "file_operations",
    parameters = {
        operation = "write",
        path = "/tmp/story.txt",
        content = "Once upon a time, in a digital realm called LLMSpell, brave developers learned to command AI agents with the power of scripts."
    }
})

-- Step 2: Create an agent to analyze the file
local step2_result = workflow:add_step({
    name = "create_agent",
    type = "agent_creation",
    provider = providers[1],
    system_prompt = "You are a literary analyst. Analyze text files and provide brief insights."
})

-- Step 3: Use the agent to analyze the story
local step3_result = workflow:add_step({
    name = "analyze_story",
    type = "agent_invoke",
    agent_step = "create_agent",
    prompt = "Please analyze the story in /tmp/story.txt. Read it first using file operations, then provide a brief analysis."
})

-- Step 4: Save the analysis
local step4_result = workflow:add_step({
    name = "save_analysis",
    type = "tool",
    tool_name = "file_operations",
    parameters = {
        operation = "write",
        path = "/tmp/analysis.txt",
        content = "{{analyze_story.result.content}}"  -- Reference previous step output
    }
})

if step1_result.success and step2_result.success and step3_result.success and step4_result.success then
    print("✅ All 4 workflow steps added successfully!")
else
    print("❌ Error adding workflow steps")
    return
end

print()
print("3. Executing the workflow...")
local execution_result = workflow:execute()

if execution_result.success then
    print("✅ Workflow executed successfully!")
    print("📊 Execution summary:")
    
    -- Check if files were created
    local story_check = Tool.invoke("file_operations", {
        operation = "exists",
        path = "/tmp/story.txt"
    })
    
    local analysis_check = Tool.invoke("file_operations", {
        operation = "exists", 
        path = "/tmp/analysis.txt"
    })
    
    if story_check.success and story_check.result.exists then
        print("   ✅ Story file created: /tmp/story.txt")
    end
    
    if analysis_check.success and analysis_check.result.exists then
        print("   ✅ Analysis file created: /tmp/analysis.txt")
        
        -- Show a snippet of the analysis
        local analysis_content = Tool.invoke("file_operations", {
            operation = "read",
            path = "/tmp/analysis.txt"
        })
        
        if analysis_content.success then
            local content = analysis_content.result.content
            local snippet = string.sub(content, 1, 100) .. (string.len(content) > 100 and "..." or "")
            print("   📝 Analysis preview: " .. snippet)
        end
    end
else
    print("❌ Workflow execution failed: " .. (execution_result.error or "Unknown error"))
end

print()
print("🎉 Congratulations! You've successfully:")
print("   - Created a sequential workflow")
print("   - Added multiple steps (tool + agent)")
print("   - Executed a complete workflow")
print("   - Chained operations together")
print()
print("Next: Try 04-save-state.lua to learn about state persistence!")