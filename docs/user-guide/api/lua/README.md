# LLMSpell Lua API Reference

**Version**: 0.6.0  
**Status**: Production Ready  
**Purpose**: Complete API reference for LLMSpell Lua scripting

> **📚 COMPREHENSIVE REFERENCE**: This document provides complete API documentation for all Lua globals, methods, and patterns available in LLMSpell. Designed for both human developers and LLM-based coding assistants.

## Table of Contents

1. [Agent API](#agent-api) - LLM agent creation and execution
2. [Tool API](#tool-api) - Tool invocation and management
3. [Workflow API](#workflow-api) - Workflow orchestration patterns
4. [State API](#state-api) - State persistence and management
5. [Session API](#session-api) - Session and artifact management
6. [Hook API](#hook-api) - Event hooks and lifecycle management
7. [Event API](#event-api) - Event emission and subscription
8. [Config API](#config-api) - Configuration access
9. [Provider API](#provider-api) - Provider management
10. [Debug API](#debug-api) - Debugging utilities
11. [JSON API](#json-api) - JSON parsing utilities
12. [Args API](#args-api) - Command-line arguments
13. [Streaming API](#streaming-api) - Streaming responses
14. [Artifact API](#artifact-api) - Artifact storage
15. [Replay API](#replay-api) - Event replay system

---

## Agent API

The Agent global provides LLM agent creation and execution capabilities.

### Agent.builder()

Creates a new agent builder for configuring agents.

**Returns:** `AgentBuilder` - Builder instance for method chaining

**Example:**
```lua
local builder = Agent.builder()
```

### AgentBuilder Methods

#### :name(string) → AgentBuilder
Sets the agent's name (required).

**Parameters:**
- `name: string` - Unique identifier for the agent

**Example:**
```lua
builder:name("assistant")
```

#### :type(string) → AgentBuilder
Sets the agent type (required).

**Parameters:**
- `type: string` - Agent type ("llm", "tool", "workflow")

**Example:**
```lua
builder:type("llm")
```

#### :model(string) → AgentBuilder
Sets the model to use.

**Parameters:**
- `model: string` - Model identifier (e.g., "gpt-4", "claude-3-opus")

**Example:**
```lua
builder:model("gpt-4")
```

#### :provider(string) → AgentBuilder
Sets the provider to use.

**Parameters:**
- `provider: string` - Provider name from configuration

**Example:**
```lua
builder:provider("openai")
```

#### :system_prompt(string) → AgentBuilder
Sets the system prompt for the agent.

**Parameters:**
- `system_prompt: string` - Instructions for agent behavior

**Example:**
```lua
builder:system_prompt("You are a helpful assistant")
```

#### :temperature(number) → AgentBuilder
Sets the temperature for response generation.

**Parameters:**
- `temperature: number` - Value between 0.0 and 2.0 (default: 0.7)

**Example:**
```lua
builder:temperature(0.5)
```

#### :max_tokens(integer) → AgentBuilder
Sets the maximum tokens for responses.

**Parameters:**
- `max_tokens: integer` - Maximum tokens to generate

**Example:**
```lua
builder:max_tokens(1000)
```

#### :timeout(integer) → AgentBuilder
Sets the timeout in seconds.

**Parameters:**
- `timeout: integer` - Timeout in seconds

**Example:**
```lua
builder:timeout(30)
```

#### :tools(table) → AgentBuilder
Sets available tools for the agent.

**Parameters:**
- `tools: table` - Array of tool names

**Example:**
```lua
builder:tools({"file-reader", "web-search"})
```

#### :build() → Agent
Creates the agent with configured settings.

**Returns:** `Agent` - Configured agent instance

**Errors:**
- Throws error if required fields (name, type) are missing
- Throws error if invalid configuration provided

**Example:**
```lua
local agent = Agent.builder()
    :name("assistant")
    :type("llm")
    :model("gpt-4")
    :build()
```

### Agent Instance Methods

#### agent:execute(table) → string
Executes the agent with given input.

**Parameters:**
- `input: table` - Input parameters
  - `prompt: string` - The prompt to process (required)
  - `context: table` - Optional context data
  - `temperature: number` - Override temperature for this execution
  - `max_tokens: integer` - Override max tokens for this execution

**Returns:** `string` - Agent's response

**Errors:**
- Throws error if prompt is missing
- Throws error if agent execution fails

**Example:**
```lua
local response = agent:execute({
    prompt = "What is 2 + 2?",
    temperature = 0.1
})
print(response) -- "4"
```

### Agent.list() → table
Lists all registered agents.

**Returns:** `table` - Array of agent information
- Each entry contains:
  - `id: string` - Agent identifier
  - `type: string` - Agent type
  - `description: string` - Agent description

**Example:**
```lua
local agents = Agent.list()
for i, agent in ipairs(agents) do
    print(agent.id, agent.type)
end
```

### Agent.get(string) → Agent
Retrieves an agent by ID.

**Parameters:**
- `id: string` - Agent identifier

**Returns:** `Agent` - Agent instance or nil if not found

**Example:**
```lua
local agent = Agent.get("assistant")
if agent then
    local response = agent:execute({prompt = "Hello"})
end
```

---

## Tool API

The Tool global provides access to tool functionality.

### Tool.list() → table
Lists all available tools.

**Returns:** `table` - Array of tool names

**Example:**
```lua
local tools = Tool.list()
for i, tool in ipairs(tools) do
    print(i, tool)
end
```

### Tool.invoke(string, table) → table
Invokes a tool with parameters.

**Parameters:**
- `tool_name: string` - Name of the tool to invoke
- `parameters: table` - Tool-specific parameters

**Returns:** `table` - Tool execution result
- `success: boolean` - Whether execution succeeded
- `result: any` - Tool-specific result data
- `error: string` - Error message if failed

**Errors:**
- Throws error if tool not found
- Tool-specific errors may be thrown

**Example:**
```lua
local result = Tool.invoke("file-reader", {
    path = "/tmp/data.txt"
})

if result.success then
    print(result.result)
else
    print("Error:", result.error)
end
```

### Tool.register(table) → boolean
Registers a custom tool (if enabled).

**Parameters:**
- `definition: table` - Tool definition
  - `name: string` - Tool identifier (required)
  - `description: string` - Tool description
  - `parameters: table` - Parameter schema
  - `handler: function` - Execution handler

**Returns:** `boolean` - Registration success

**Example:**
```lua
local success = Tool.register({
    name = "custom-tool",
    description = "My custom tool",
    parameters = {
        input = {type = "string", required = true}
    },
    handler = function(params)
        return {result = "Processed: " .. params.input}
    end
})
```

---

## Workflow API

The Workflow global provides workflow orchestration capabilities.

### Workflow.builder() → WorkflowBuilder
Creates a new workflow builder.

**Returns:** `WorkflowBuilder` - Builder for configuring workflows

**Example:**
```lua
local builder = Workflow.builder()
```

### WorkflowBuilder Methods

#### :name(string) → WorkflowBuilder
Sets the workflow name (required).

**Parameters:**
- `name: string` - Workflow identifier

**Example:**
```lua
builder:name("data-pipeline")
```

#### :type(string) → WorkflowBuilder
Sets the workflow type.

**Parameters:**
- `type: string` - Type ("sequential", "parallel", "conditional", "loop", "nested")

**Example:**
```lua
builder:type("sequential")
```

#### :add_step(table) → WorkflowBuilder
Adds a step to the workflow.

**Parameters:**
- `step: table` - Step configuration
  - `name: string` - Step identifier (required)
  - `type: string` - Step type ("agent", "tool", "function")
  - `agent: string` - Agent ID (if type="agent")
  - `tool: string` - Tool name (if type="tool")
  - `input: table` - Step input parameters
  - `timeout_ms: integer` - Step timeout in milliseconds

**Example:**
```lua
builder:add_step({
    name = "analyze",
    type = "agent",
    agent = "analyzer",
    input = {prompt = "Analyze this data"}
})
```

#### :add_sequential_step(table) → WorkflowBuilder
Adds a sequential step (shorthand).

**Parameters:** Same as `add_step`

#### :add_parallel_step(table) → WorkflowBuilder
Adds a parallel step (shorthand).

**Parameters:** Same as `add_step`

#### :add_condition(table) → WorkflowBuilder
Adds a conditional branch.

**Parameters:**
- `condition: table` - Condition configuration
  - `expression: string` - Condition expression
  - `then_branch: table` - Steps if true
  - `else_branch: table` - Steps if false

**Example:**
```lua
builder:add_condition({
    expression = "result > 0",
    then_branch = {
        {name = "positive", type = "tool", tool = "positive-handler"}
    },
    else_branch = {
        {name = "negative", type = "tool", tool = "negative-handler"}
    }
})
```

#### :build() → Workflow
Creates the workflow.

**Returns:** `Workflow` - Configured workflow instance

**Example:**
```lua
local workflow = builder:build()
```

### Workflow Instance Methods

#### workflow:execute(table) → table
Executes the workflow.

**Parameters:**
- `context: table` - Execution context
  - `state: table` - Initial state data
  - `timeout: integer` - Overall timeout in seconds

**Returns:** `table` - Execution result
- `success: boolean` - Execution success
- `data: table` - Result data
- `error: string` - Error if failed

**Example:**
```lua
local result = workflow:execute({
    state = {input = "data"}
})

if result.success then
    print("Result:", result.data)
end
```

### Workflow.sequential(table) → Workflow
Creates a sequential workflow (shorthand).

**Parameters:**
- `config: table` - Workflow configuration
  - `name: string` - Workflow name
  - `steps: table` - Array of steps

**Returns:** `Workflow` - Sequential workflow

**Example:**
```lua
local workflow = Workflow.sequential({
    name = "pipeline",
    steps = {
        {name = "step1", type = "tool", tool = "processor"},
        {name = "step2", type = "agent", agent = "analyzer"}
    }
})
```

### Workflow.parallel(table) → Workflow
Creates a parallel workflow (shorthand).

**Parameters:** Same as `sequential`

**Returns:** `Workflow` - Parallel workflow

### Workflow.conditional(table) → Workflow
Creates a conditional workflow.

**Parameters:**
- `config: table` - Workflow configuration
  - `name: string` - Workflow name
  - `condition: table` - Condition configuration

**Returns:** `Workflow` - Conditional workflow

### Workflow.loop(table) → Workflow
Creates a loop workflow.

**Parameters:**
- `config: table` - Loop configuration
  - `name: string` - Workflow name
  - `iterator: string` - Iterator type ("range", "collection", "while")
  - `start: integer` - Start value (for range)
  - `end: integer` - End value (for range)
  - `items: table` - Items (for collection)
  - `condition: string` - Loop condition (for while)
  - `body: table` - Loop body steps

**Returns:** `Workflow` - Loop workflow

**Example:**
```lua
local workflow = Workflow.loop({
    name = "processor",
    iterator = "collection",
    items = {"file1.txt", "file2.txt"},
    body = {
        {name = "process", type = "tool", tool = "file-processor"}
    }
})
```

### Workflow.list() → table
Lists all registered workflows.

**Returns:** `table` - Array of workflow information
- Each entry contains:
  - `id: string` - Workflow identifier
  - `type: string` - Workflow type
  - `description: string` - Workflow description

---

## State API

The State global provides persistent state management.

### State.save(string, string) → boolean
Saves a value to state.

**Parameters:**
- `key: string` - State key
- `value: string` - Value to save (must be string)

**Returns:** `boolean` - Save success

**Example:**
```lua
local success = State.save("user_preference", "dark_mode")
```

### State.load(string) → string|nil
Loads a value from state.

**Parameters:**
- `key: string` - State key

**Returns:** `string|nil` - Saved value or nil if not found

**Example:**
```lua
local value = State.load("user_preference")
if value then
    print("Preference:", value)
end
```

### State.exists(string) → boolean
Checks if a key exists in state.

**Parameters:**
- `key: string` - State key

**Returns:** `boolean` - Whether key exists

**Example:**
```lua
if State.exists("user_preference") then
    -- Key exists
end
```

### State.delete(string) → boolean
Deletes a key from state.

**Parameters:**
- `key: string` - State key

**Returns:** `boolean` - Deletion success

**Example:**
```lua
State.delete("old_key")
```

### State.list() → table
Lists all state keys.

**Returns:** `table` - Array of state keys

**Example:**
```lua
local keys = State.list()
for i, key in ipairs(keys) do
    print(key, State.load(key))
end
```

### State.clear() → boolean
Clears all state data.

**Returns:** `boolean` - Clear success

**Example:**
```lua
State.clear()
```

---

## Session API

The Session global manages sessions and artifacts.

### Session.current() → string|nil
Gets the current session ID.

**Returns:** `string|nil` - Session ID or nil if no active session

**Example:**
```lua
local session_id = Session.current()
if session_id then
    print("Active session:", session_id)
end
```

### Session.create(table) → string
Creates a new session.

**Parameters:**
- `config: table` - Session configuration
  - `name: string` - Session name
  - `metadata: table` - Session metadata

**Returns:** `string` - New session ID

**Example:**
```lua
local session_id = Session.create({
    name = "analysis-session",
    metadata = {user = "john", project = "data-analysis"}
})
```

### Session.store_artifact(string, any) → table
Stores an artifact in the session.

**Parameters:**
- `type: string` - Artifact type identifier
- `content: any` - Artifact content

**Returns:** `table` - Artifact information
- `id: string` - Artifact ID
- `session_id: string` - Session ID
- `type: string` - Artifact type

**Example:**
```lua
local artifact = Session.store_artifact("report", "Analysis complete")
print("Artifact stored:", artifact.id)
```

### Session.load_artifact(string) → any|nil
Loads an artifact by ID.

**Parameters:**
- `id: string` - Artifact ID

**Returns:** `any|nil` - Artifact content or nil if not found

**Example:**
```lua
local content = Session.load_artifact("artifact_123")
if content then
    print("Artifact:", content)
end
```

### Session.list_artifacts(string) → table
Lists artifacts for a session.

**Parameters:**
- `session_id: string` - Session ID (optional, defaults to current)

**Returns:** `table` - Array of artifact information

**Example:**
```lua
local artifacts = Session.list_artifacts()
for i, artifact in ipairs(artifacts) do
    print(artifact.id, artifact.type)
end
```

---

## Hook API

The Hook global manages lifecycle hooks and event handlers.

### Hook.register(string, function) → boolean
Registers a hook handler.

**Parameters:**
- `event: string` - Event name to hook
- `handler: function` - Handler function

**Returns:** `boolean` - Registration success

**Handler Function Signature:**
```lua
function(event_data)
    -- event_data contains event-specific information
    return {
        continue = true,  -- Whether to continue processing
        modify = {}      -- Optional data modifications
    }
end
```

**Example:**
```lua
Hook.register("BeforeAgentExecution", function(data)
    print("Agent executing:", data.agent_id)
    return {continue = true}
end)
```

### Hook.emit(string, table) → table
Emits a hook event.

**Parameters:**
- `event: string` - Event name
- `data: table` - Event data

**Returns:** `table` - Hook processing result

**Example:**
```lua
local result = Hook.emit("CustomEvent", {
    message = "Something happened"
})
```

### Hook Events

Standard hook events:
- `BeforeToolExecution` - Before tool executes
- `AfterToolExecution` - After tool completes
- `BeforeAgentExecution` - Before agent executes
- `AfterAgentExecution` - After agent completes
- `BeforeWorkflowStep` - Before workflow step
- `AfterWorkflowStep` - After workflow step
- `SessionStart` - Session begins
- `SessionEnd` - Session ends

---

## Event API

The Event global provides event emission and subscription.

### Event.emit(string, table) → boolean
Emits an event.

**Parameters:**
- `event_type: string` - Event type identifier
- `data: table` - Event data

**Returns:** `boolean` - Emission success

**Example:**
```lua
Event.emit("user_action", {
    action = "button_click",
    timestamp = os.time()
})
```

### Event.subscribe(string, function) → string
Subscribes to an event type.

**Parameters:**
- `event_type: string` - Event type to subscribe to
- `handler: function` - Event handler function

**Returns:** `string` - Subscription ID

**Example:**
```lua
local sub_id = Event.subscribe("user_action", function(data)
    print("User action:", data.action)
end)
```

### Event.unsubscribe(string) → boolean
Unsubscribes from events.

**Parameters:**
- `subscription_id: string` - Subscription ID from subscribe()

**Returns:** `boolean` - Unsubscribe success

**Example:**
```lua
Event.unsubscribe(sub_id)
```

---

## Config API

The Config global provides access to configuration.

### Config.get(string) → any
Gets a configuration value.

**Parameters:**
- `path: string` - Configuration path (dot-separated)

**Returns:** `any` - Configuration value or nil

**Example:**
```lua
local model = Config.get("providers.openai.default_model")
local timeout = Config.get("runtime.timeout_seconds")
```

### Config.exists(string) → boolean
Checks if a configuration path exists.

**Parameters:**
- `path: string` - Configuration path

**Returns:** `boolean` - Whether path exists

**Example:**
```lua
if Config.exists("providers.anthropic") then
    -- Anthropic provider is configured
end
```

---

## Provider API

The Provider global manages LLM providers.

### Provider.list() → table
Lists available providers.

**Returns:** `table` - Array of provider names

**Example:**
```lua
local providers = Provider.list()
for i, provider in ipairs(providers) do
    print(provider)
end
```

### Provider.get(string) → table
Gets provider information.

**Parameters:**
- `name: string` - Provider name

**Returns:** `table` - Provider configuration
- `name: string` - Provider name
- `api_base: string` - API base URL
- `default_model: string` - Default model
- `models: table` - Available models

**Example:**
```lua
local provider = Provider.get("openai")
print("Default model:", provider.default_model)
```

---

## Debug API

The Debug global provides debugging utilities.

### Debug.enabled() → boolean
Checks if debug mode is enabled.

**Returns:** `boolean` - Debug mode status

**Example:**
```lua
if Debug.enabled() then
    print("Debug: Processing started")
end
```

### Debug.log(string, any) → nil
Logs a debug message.

**Parameters:**
- `level: string` - Log level ("trace", "debug", "info", "warn", "error")
- `message: any` - Message to log

**Example:**
```lua
Debug.log("info", "Processing file: " .. filename)
Debug.log("error", "Failed to open file")
```

### Debug.inspect(any) → string
Inspects a value (pretty-prints tables).

**Parameters:**
- `value: any` - Value to inspect

**Returns:** `string` - Formatted representation

**Example:**
```lua
local data = {name = "test", values = {1, 2, 3}}
print(Debug.inspect(data))
```

---

## JSON API

The JSON global provides JSON utilities.

### JSON.parse(string) → table
Parses JSON string to Lua table.

**Parameters:**
- `json: string` - JSON string

**Returns:** `table` - Parsed data

**Errors:**
- Throws error if JSON is invalid

**Example:**
```lua
local data = JSON.parse('{"name": "test", "value": 42}')
print(data.name) -- "test"
```

### JSON.stringify(table) → string
Converts Lua table to JSON string.

**Parameters:**
- `data: table` - Data to serialize

**Returns:** `string` - JSON string

**Example:**
```lua
local json = JSON.stringify({name = "test", value = 42})
print(json) -- '{"name":"test","value":42}'
```

---

## Args API

The Args global provides command-line argument access.

### Args.get(integer) → string|nil
Gets a command-line argument by index.

**Parameters:**
- `index: integer` - Argument index (1-based)

**Returns:** `string|nil` - Argument value or nil

**Example:**
```lua
-- If run with: llmspell run script.lua arg1 arg2
local first_arg = Args.get(1)  -- "arg1"
local second_arg = Args.get(2) -- "arg2"
```

### Args.all() → table
Gets all command-line arguments.

**Returns:** `table` - Array of arguments

**Example:**
```lua
local args = Args.all()
for i, arg in ipairs(args) do
    print(i, arg)
end
```

---

## Streaming API

The Streaming global provides streaming response capabilities.

### Streaming.create(function) → Stream
Creates a streaming handler.

**Parameters:**
- `handler: function` - Stream handler function

**Returns:** `Stream` - Stream instance

**Handler Function Signature:**
```lua
function(chunk)
    -- Process streaming chunk
    print(chunk)
end
```

**Example:**
```lua
local stream = Streaming.create(function(chunk)
    io.write(chunk)
    io.flush()
end)
```

### Stream:process(string) → nil
Processes a stream chunk.

**Parameters:**
- `chunk: string` - Stream chunk

**Example:**
```lua
stream:process("Streaming ")
stream:process("data...")
```

---

## Artifact API

The Artifact global manages artifact storage.

### Artifact.store(table) → string
Stores an artifact.

**Parameters:**
- `artifact: table` - Artifact data
  - `type: string` - Artifact type
  - `content: any` - Artifact content
  - `metadata: table` - Optional metadata

**Returns:** `string` - Artifact ID

**Example:**
```lua
local id = Artifact.store({
    type = "report",
    content = "Analysis results...",
    metadata = {generated_at = os.time()}
})
```

### Artifact.load(string) → table|nil
Loads an artifact.

**Parameters:**
- `id: string` - Artifact ID

**Returns:** `table|nil` - Artifact data or nil

**Example:**
```lua
local artifact = Artifact.load("artifact_123")
if artifact then
    print(artifact.content)
end
```

---

## Replay API

The Replay global provides event replay capabilities.

### Replay.record(boolean) → nil
Enables or disables event recording.

**Parameters:**
- `enabled: boolean` - Whether to record events

**Example:**
```lua
Replay.record(true)  -- Start recording
-- Perform operations
Replay.record(false) -- Stop recording
```

### Replay.play(string) → boolean
Replays recorded events.

**Parameters:**
- `recording_id: string` - Recording identifier

**Returns:** `boolean` - Replay success

**Example:**
```lua
local success = Replay.play("session_123")
```

---

## Error Handling

All API methods may throw errors. Use pcall for safe execution:

```lua
local success, result = pcall(function()
    return agent:execute({prompt = "test"})
end)

if success then
    print("Result:", result)
else
    print("Error:", result)
end
```

## Type Conventions

- **string** - Lua string
- **integer** - Whole number
- **number** - Floating point number
- **boolean** - true or false
- **table** - Lua table (object or array)
- **function** - Lua function
- **any** - Any Lua type
- **nil** - Lua nil value

## Best Practices

1. **Always check return values** - Many methods return success booleans
2. **Use pcall for error handling** - Wrap risky operations
3. **Clean up resources** - Unsubscribe from events, clear state when done
4. **Use builders for complex objects** - Agent and Workflow builders provide validation
5. **Check Debug.enabled()** - Before expensive debug operations

## Examples

See [Example Index](../../../../examples/EXAMPLE-INDEX.md) for complete working examples.

## See Also

- [Rust API Reference](../rust/README.md) - Rust API documentation
- [Getting Started](../../getting-started.md) - Introduction to LLMSpell
- [Agent API Guide](../../agent-api.md) - Detailed agent documentation
- [Workflow API Guide](../../workflow-api.md) - Detailed workflow documentation