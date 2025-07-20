-- ABOUTME: Demonstrates all global objects available in rs-llmspell scripts
-- ABOUTME: Shows Agent, Tool, Workflow, JSON, State, Hook, and Event usage

-- Example 1: Using the JSON global
print("=== JSON Global Demo ===")
local data = {
    name = "rs-llmspell",
    version = "0.3.0",
    features = {"agents", "tools", "workflows"},
    metadata = {
        phase = 3,
        author = "LLMSpell Team"
    }
}

-- Stringify the data
local json_str = JSON.stringify(data)
print("JSON stringified:", json_str)

-- Parse it back
local parsed = JSON.parse(json_str)
print("Parsed name:", parsed.name)
print("First feature:", parsed.features[1])

-- Example 2: Using the State global (in-memory storage)
print("\n=== State Global Demo ===")
State.set("user_preference", "dark_mode")
State.set("session_data", {
    user_id = 12345,
    login_time = os.date(),
    permissions = {"read", "write"}
})

-- Retrieve stored values
print("User preference:", State.get("user_preference"))
local session = State.get("session_data")
print("User ID:", session.user_id)
print("Permissions:", JSON.stringify(session.permissions))

-- List all stored keys
local keys = State.list()
print("All state keys:", JSON.stringify(keys))

-- Example 3: Using the Tool global
print("\n=== Tool Global Demo ===")
-- List available tool categories
local categories = Tool.categories()
print("Tool categories available:", JSON.stringify(categories))

-- Get a specific tool
local calculator = Tool.get("calculator")
if calculator then
    print("Calculator tool found!")
    
    -- Execute the calculator tool
    local result = calculator:execute({
        operation = "add",
        a = 10,
        b = 20
    })
    print("10 + 20 =", result.result)
end

-- List all tools
local all_tools = Tool.list()
print("Total tools available:", #all_tools)

-- Example 4: Using the Agent global
print("\n=== Agent Global Demo ===")
-- Create a simple agent
local my_agent = Agent.create({
    name = "demo_agent",
    provider = "mock",  -- Using mock provider for demo
    model = "mock-model",
    system_prompt = "You are a helpful assistant."
})

print("Agent created:", my_agent.id)

-- Execute the agent (would normally make LLM call)
-- Note: With mock provider, this returns a mock response
local response = my_agent:execute({
    prompt = "What is 2+2?"
})
print("Agent response:", response.content)

-- List all agent instances
local agents = Agent.list()
print("Active agents:", #agents)

-- Example 5: Using the Workflow global
print("\n=== Workflow Global Demo ===")
-- Create a sequential workflow
local seq_workflow = Workflow.sequential({
    name = "demo_sequential",
    steps = {
        {
            name = "step1",
            tool = "calculator",
            parameters = {
                operation = "multiply",
                a = 5,
                b = 6
            }
        },
        {
            name = "step2",
            tool = "calculator",
            parameters = {
                operation = "add",
                a = "$step1.result",  -- Reference previous step's output
                b = 10
            }
        }
    }
})

-- Execute the workflow
local workflow_result = Workflow.execute(seq_workflow)
print("Workflow final result:", workflow_result.output)

-- Example 6: Using Hook and Event globals (placeholders)
print("\n=== Hook & Event Global Demo (Placeholders) ===")
-- Register a hook (placeholder - will be functional in Phase 4)
local hook_result = Hook.register("workflow_complete", function(data)
    print("Workflow completed:", data.workflow_id)
end)
print("Hook registration:", hook_result)

-- Emit an event (placeholder - will be functional in Phase 4)
local event_result = Event.emit("custom_event", {
    timestamp = os.date(),
    message = "Demo event"
})
print("Event emission:", event_result)

-- Example 7: Combining globals for complex scenarios
print("\n=== Combined Example: Agent + Tool + State ===")
-- Store configuration in state
State.set("agent_config", {
    temperature = 0.7,
    max_tokens = 100
})

-- Create an agent with config from state
local config = State.get("agent_config")
local smart_agent = Agent.create({
    name = "smart_demo",
    provider = "mock",
    model = "mock-model",
    temperature = config.temperature
})

-- Use the agent to decide which tool to use
-- (In real scenario, agent would analyze and choose)
local tool_choice = "calculator"  -- Simulated decision

-- Execute the chosen tool
local chosen_tool = Tool.get(tool_choice)
if chosen_tool then
    local tool_result = chosen_tool:execute({
        operation = "subtract",
        a = 100,
        b = 42
    })
    
    -- Store result in state for later use
    State.set("last_calculation", tool_result.result)
    print("Calculation stored in state:", State.get("last_calculation"))
end

print("\n=== Demo Complete ===")
print("This demo showed all available globals:")
print("- JSON: Parse and stringify data")
print("- State: In-memory key-value storage")
print("- Tool: Access and execute tools")
print("- Agent: Create and execute LLM agents")
print("- Workflow: Orchestrate multi-step processes")
print("- Hook: Lifecycle event registration (Phase 4)")
print("- Event: Event emission system (Phase 4)")