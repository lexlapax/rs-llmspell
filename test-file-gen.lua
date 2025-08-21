-- Test file generation from state
print("=== Testing File Generation from State ===")

-- Create an agent that generates content
local content_agent = Agent.builder()
    :name("content_generator")
    :type("llm")
    :model("gpt-4o-mini")
    :system_prompt("Generate a simple HTML file with a heading saying 'Hello from State!'")
    :build()

print("✓ Content agent created")

-- Create a workflow
local workflow = Workflow.builder()
    :name("file_gen_workflow")
    :description("Generate file from state")
    :sequential()

workflow:add_step({
    name = "generate_content",
    type = "agent",
    agent = "content_generator",
    input = "Create a simple HTML page"
})

workflow = workflow:build()
print("✓ Workflow created")

-- Execute workflow
local result = workflow:execute({text = "Generate HTML"})
print("✓ Workflow executed")

-- Read the generated content from state
if result.metadata and result.metadata.extra then
    local exec_id = result.metadata.extra.execution_id
    print("\nExecution ID: " .. tostring(exec_id))
    
    -- Read agent output from state
    local key = "workflow:" .. exec_id .. ":agent:content_generator:output"
    print("Reading from state key: " .. key)
    
    -- Use the correct scope for reading
    local content = State.load("custom", ":" .. key)
    
    if content then
        print("✅ SUCCESS! Found content in state")
        print("Content length: " .. string.len(content))
        
        -- Write to file
        local file_path = "/tmp/test-generated.html"
        local file = io.open(file_path, "w")
        if file then
            file:write(content)
            file:close()
            print("✅ File written to: " .. file_path)
        else
            print("❌ Failed to write file")
        end
    else
        print("❌ No content found in state")
    end
else
    print("❌ No execution ID in result")
end

print("\n=== Test Complete ===")