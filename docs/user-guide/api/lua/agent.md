# Agent Module

The Agent module provides functionality for creating and managing LLM agents.

## Constructor

### Agent.builder()
Creates a new agent builder for configuring agents.

**Returns:** `AgentBuilder` - Builder object for agent configuration

**Example:**
```lua
local builder = Agent.builder()
```

## AgentBuilder Methods

### :name(name)
Sets the agent's name.

**Parameters:**
- `name` (string): Agent name

**Returns:** `AgentBuilder` - Self for chaining

### :type(agent_type)
Sets the agent type.

**Parameters:**
- `agent_type` (string): One of "llm", "tool", "workflow", "hybrid"

**Returns:** `AgentBuilder` - Self for chaining

### :model(model_name)
Sets the LLM model for the agent.

**Parameters:**
- `model_name` (string): Model identifier (e.g., "openai/gpt-4", "anthropic/claude-3")

**Returns:** `AgentBuilder` - Self for chaining

### :system_prompt(prompt)
Sets the system prompt for the agent.

**Parameters:**
- `prompt` (string): System prompt text

**Returns:** `AgentBuilder` - Self for chaining

### :temperature(temp)
Sets the temperature for response generation.

**Parameters:**
- `temp` (number): Temperature value (0.0 to 2.0)

**Returns:** `AgentBuilder` - Self for chaining

### :max_tokens(tokens)
Sets the maximum tokens for responses.

**Parameters:**
- `tokens` (number): Maximum token count

**Returns:** `AgentBuilder` - Self for chaining

### :tools(tool_list)
Assigns tools to the agent.

**Parameters:**
- `tool_list` (table): Array of tool names

**Returns:** `AgentBuilder` - Self for chaining

### :memory(enabled)
Enables or disables conversation memory.

**Parameters:**
- `enabled` (boolean): Whether to enable memory

**Returns:** `AgentBuilder` - Self for chaining

### :build()
Builds and returns the configured agent.

**Returns:** `Agent` - Configured agent instance

**Example:**
```lua
local agent = Agent.builder()
    :name("assistant")
    :type("llm")
    :model("openai/gpt-4o-mini")
    :system_prompt("You are a helpful assistant")
    :temperature(0.7)
    :tools({"calculator", "web-search"})
    :memory(true)
    :build()
```

## Agent Methods

### agent:execute(params)
Executes the agent with given parameters.

**Parameters:**
- `params` (table): Execution parameters
  - `prompt` (string): User prompt
  - `context` (table, optional): Additional context
  - `stream` (boolean, optional): Enable streaming

**Returns:** 
- `result` (table): Execution result
  - `response` (string): Agent response
  - `usage` (table): Token usage statistics
  - `tools_used` (table): List of tools invoked

**Example:**
```lua
local result = agent:execute({
    prompt = "What's the weather like?",
    context = {location = "San Francisco"}
})
print(result.response)
```

### agent:execute_async(params)
Executes the agent asynchronously.

**Parameters:** Same as `execute()`

**Returns:** `Promise` - Promise that resolves to result

**Example:**
```lua
agent:execute_async({prompt = "Complex task"})
    :then(function(result)
        print("Done:", result.response)
    end)
    :catch(function(err)
        print("Error:", err)
    end)
```

### agent:add_tool(tool_name)
Adds a tool to the agent dynamically.

**Parameters:**
- `tool_name` (string): Name of tool to add

**Returns:** `boolean` - Success status

### agent:remove_tool(tool_name)
Removes a tool from the agent.

**Parameters:**
- `tool_name` (string): Name of tool to remove

**Returns:** `boolean` - Success status

### agent:clear_memory()
Clears the agent's conversation memory.

**Returns:** `nil`

### agent:get_memory()
Gets the current conversation memory.

**Returns:** `table` - Array of message objects

### agent:set_memory(messages)
Sets the conversation memory.

**Parameters:**
- `messages` (table): Array of message objects

**Returns:** `nil`

### agent:save()
Saves the agent's state to persistent storage.

**Returns:** `string` - Agent ID for loading

**Example:**
```lua
local id = agent:save()
print("Saved agent:", id)
```

### agent:clone()
Creates a copy of the agent.

**Returns:** `Agent` - New agent instance

## Static Methods

### Agent.load(id)
Loads an agent from persistent storage.

**Parameters:**
- `id` (string): Agent ID

**Returns:** `Agent` - Loaded agent instance

**Example:**
```lua
local agent = Agent.load("agent_123")
```

### Agent.list()
Lists all available agents.

**Returns:** `table` - Array of agent descriptors

### Agent.register(name, template)
Registers an agent template.

**Parameters:**
- `name` (string): Template name
- `template` (table): Template configuration

**Returns:** `boolean` - Success status

**Example:**
```lua
Agent.register("customer_support", {
    type = "llm",
    model = "openai/gpt-4",
    system_prompt = "You are a customer support agent",
    tools = {"ticket_system", "knowledge_base"}
})
```

### Agent.from_template(template_name)
Creates an agent from a registered template.

**Parameters:**
- `template_name` (string): Name of template

**Returns:** `Agent` - Agent instance

## Properties

### agent.id
Unique identifier for the agent (read-only).

### agent.name
Agent name.

### agent.type
Agent type (read-only).

### agent.model
Current model being used.

### agent.metrics
Performance metrics (read-only).
- `total_calls` (number): Total executions
- `total_tokens` (number): Total tokens used
- `average_latency` (number): Average response time

## Events

Agents emit the following events:

### "execution_start"
Emitted when execution begins.

### "execution_complete"
Emitted when execution completes successfully.

### "execution_error"
Emitted when execution fails.

### "tool_invoked"
Emitted when agent invokes a tool.

**Example:**
```lua
agent:on("execution_complete", function(result)
    print("Completed in", result.duration, "ms")
end)
```

## Error Handling

All agent methods that can fail return `nil` and an error message:

```lua
local agent, err = Agent.builder()
    :name("test")
    :type("invalid_type")  -- This will fail
    :build()

if not agent then
    print("Failed to create agent:", err)
end
```

## See Also
- [Tool Module](./tool.md) - Tool integration
- [Workflow Module](./workflow.md) - Workflow orchestration
- [State Module](./state.md) - State persistence