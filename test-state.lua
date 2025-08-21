-- Minimal test for state capture from workflows
print("=== Testing State Capture from Workflow ===")

-- Create a simple agent
local agent = Agent.builder()
    :name("test_agent")
    :type("llm")
    :model("gpt-4o-mini")
    :system_prompt("Respond with 'Hello from test agent'")
    :build()

print("✓ Agent created")

-- Create a simple workflow
local workflow = Workflow.builder()
    :name("test_workflow")
    :description("Test state capture")
    :sequential()

workflow:add_step({
    name = "step1",
    type = "agent",
    agent = "test_agent",
    input = "test"
})

workflow = workflow:build()
print("✓ Workflow created")

-- Execute workflow
local result = workflow:execute({text = "test input"})
print("✓ Workflow executed")

-- Check execution ID
if result.metadata and result.metadata.extra then
    local exec_id = result.metadata.extra.execution_id
    print("\nExecution ID: " .. tostring(exec_id))
    
    -- Try to read agent output from state
    -- Both StateGlobal and WorkflowGlobal now use NoScopeStateAdapter 
    -- which uses StateScope::Custom("") adding "custom::" prefix
    -- So the full key stored is: custom::workflow:{id}:agent:{name}:output
    -- State.load creates {scope}:{key}, so we need to match exactly
    local workflow_key = "workflow:" .. exec_id .. ":agent:test_agent:output"
    print("\nStored with key: custom::" .. workflow_key)
    
    -- To read "custom::workflow:...", we use State.load("custom", ":workflow:...")
    -- This creates "custom:" + ":workflow:..." = "custom::workflow:..."
    print("About to call State.load with scope='custom' and key=':" .. workflow_key .. "'")
    local value = State.load("custom", ":" .. workflow_key)
    print("State.load returned: " .. tostring(value))
    if value then
        print("✅ SUCCESS! Found agent output in state: " .. tostring(value))
    else
        print("❌ FAILED! Agent output not found in state")
    end
else
    print("❌ No execution ID in result metadata")
end

print("\n=== Test Complete ===")