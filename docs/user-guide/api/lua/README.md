# LLMSpell Lua API Documentation

This document provides comprehensive documentation of all Lua globals available in LLMSpell scripts. Each global object provides specific functionality for building LLM-powered applications.

## Table of Contents

1. [Agent](#agent) - LLM agent creation and management
2. [Tool](#tool) - Tool invocation and discovery
3. [Workflow](#workflow) - Workflow orchestration
4. [Session](#session) - Session management and persistence
5. [State](#state) - Global state management
6. [Memory](#memory) - Adaptive memory system (episodic, semantic, consolidation)
7. [Context](#context) - Context assembly and retrieval strategies
8. [Event](#event) - Event publishing and subscription
9. [Hook](#hook) - Hook registration and management
10. [RAG](#rag) - Retrieval-Augmented Generation with vector storage
11. [LocalLLM](#localllm) - Local model management (Ollama, Candle)
12. [Config](#config) - Configuration access and management
13. [Provider](#provider) - LLM provider information
14. [Artifact](#artifact) - Artifact storage and retrieval
15. [Replay](#replay) - Hook replay and testing
16. [Debug](#debug) - Debugging and profiling utilities
17. [JSON](#json) - JSON parsing and serialization
18. [Template](#template) - Production-ready AI workflow templates
19. [ARGS](#args) - Command-line argument access
20. [Streaming](#streaming) - Streaming and coroutine utilities

---

## Agent

The `Agent` global provides functionality for creating and managing LLM agents.

### Core Methods

#### Agent.builder()
Creates a new agent builder for configuring agents.

```lua
local agent = Agent.builder()
    :name("assistant")
    :type("llm")  -- or "tool", "composite"
    :model("openai/gpt-4")
    :temperature(0.7)
    :max_tokens(2000)
    :system_prompt("You are a helpful assistant")
    :tool("calculator")
    :tool("file-reader")
    :capability("reasoning")
    :capability("code-generation")
    :memory_enabled(true)
    :context_window(8000)
    :build()
```

#### Agent.create(config)
⚠️ **DEPRECATED** - Use `Agent.builder()` instead. This method returns an error directing users to use the builder pattern.

```lua
-- DEPRECATED - Do not use
-- Use Agent.builder() instead
```

#### Agent.list()
Lists all available agents.

```lua
local agents = Agent.list()
for i, agent_info in ipairs(agents) do
    print(agent_info.name, agent_info.type)
end
```

#### Agent.get(name)
Gets a specific agent by name.

```lua
local agent = Agent.get("assistant")
if agent then
    local response = agent:execute("Hello!")
end
```

### Agent Discovery

#### Agent.discover_by_capability(capability)
Finds agents with a specific capability.

```lua
local coders = Agent.discover_by_capability("code-generation")
```

#### Agent.count()
Returns the count of registered agents.

```lua
local total = Agent.count()
print("Total agents:", total)
```

### Agent Templates

#### Agent.list_templates()
Lists available agent templates.

```lua
local templates = Agent.list_templates()
```

#### Agent.create_from_template(template_name, overrides)
Creates an agent from a template.

```lua
local agent = Agent.create_from_template("code-assistant", {
    model = "openai/gpt-4-turbo"
})
```

#### Agent.register(name, config)
Registers a new agent configuration.

```lua
Agent.register("custom-agent", {
    type = "llm",
    model = "openai/gpt-4",
    system_prompt = "Custom prompt"
})
```

### Agent Context Management

#### Agent.create_context(name, data)
Creates a new agent context.

```lua
local ctx = Agent.create_context("session-123", {
    user = "alice",
    preferences = {theme = "dark"}
})
```

#### Agent.get_context_data(name)
Gets context data.

```lua
local data = Agent.get_context_data("session-123")
```

#### Agent.update_context(name, data)
Updates existing context.

```lua
Agent.update_context("session-123", {
    last_query = "What is the weather?"
})
```

#### Agent.create_child_context(parent, name, data)
Creates a child context.

```lua
Agent.create_child_context("session-123", "subsession-1", {
    task = "weather-query"
})
```

#### Agent.remove_context(name)
Removes a context.

```lua
Agent.remove_context("session-123")
```

### Agent Memory

#### Agent.set_shared_memory(key, value)
Sets shared memory accessible to all agents.

```lua
Agent.set_shared_memory("api_results", {data = results})
```

#### Agent.get_shared_memory(key)
Gets shared memory value.

```lua
local data = Agent.get_shared_memory("api_results")
```

### Agent Composition

#### Agent.create_composite(config)
Creates a composite agent that coordinates multiple agents.

```lua
local composite = Agent.create_composite({
    name = "research-team",
    agents = {"researcher", "writer", "reviewer"},
    strategy = "sequential"  -- or "parallel", "vote"
})
```

#### Agent.wrap_as_tool(agent_name)
Wraps an agent as a tool for use by other agents.

```lua
local tool = Agent.wrap_as_tool("calculator-agent")
```

### Agent Information

#### Agent.get_info(name)
Gets detailed information about an agent.

```lua
local info = Agent.get_info("assistant")
print(info.model, info.capabilities)
```

#### Agent.list_capabilities(name)
Lists capabilities of an agent.

```lua
local caps = Agent.list_capabilities("assistant")
```

#### Agent.list_instances()
Lists all running agent instances.

```lua
local instances = Agent.list_instances()
```

#### Agent.get_hierarchy()
Gets the agent hierarchy tree.

```lua
local tree = Agent.get_hierarchy()
```

#### Agent.get_details(name)
Gets comprehensive agent details.

```lua
local details = Agent.get_details("assistant")
```

### Agent Instance Methods

When you have an agent instance:

#### agent:execute(input_table)
Executes the agent with an input table.

```lua
-- Correct usage - input must be a table with 'text' field
local response = agent:execute({
    text = "Write a poem about autumn"
})

-- With parameters
local response = agent:execute({
    text = "Translate to French: Hello world",
    temperature = 0.9,
    max_tokens = 500
})
```

**⚠️ IMPORTANT**: The `agent:execute()` method requires a **table** as input, not a string. The table must contain a `text` field with the prompt. Additional parameters (temperature, max_tokens, etc.) can be included in the same table.

**Common mistake**:
```lua
-- ❌ WRONG - will fail with "error converting Lua string to table"
local result = agent:execute("Hello world")

-- ✅ CORRECT
local result = agent:execute({text = "Hello world"})
```

#### agent:invokeStream(prompt, options, callback)
Executes with streaming response.

```lua
agent:invokeStream("Tell a story", {}, function(chunk)
    print(chunk)
end)
```

#### agent:get_state()
Gets the current state of the agent.

```lua
local state = agent:get_state()
```

#### agent:set_state(new_state)
Sets the agent's state.

```lua
agent:set_state({context = "updated"})
```

#### agent:get_model()
Gets the model name used by this agent.

```lua
local model = agent:get_model()
print("Using model:", model)
```

#### agent:get_name()
Gets the agent's name.

```lua
local name = agent:get_name()
```

#### agent:get_type()
Gets the agent's type (e.g., "llm", "tool", "composite").

```lua
local agent_type = agent:get_type()
```

#### agent:get_capabilities()
Gets the agent's capabilities.

```lua
local capabilities = agent:get_capabilities()
for i, cap in ipairs(capabilities) do
    print("Capability:", cap)
end
```

---

## Tool

The `Tool` global provides functionality for tool execution and management.

### Core Methods

#### Tool.list()
Lists all available tools.

```lua
local tools = Tool.list()
for i, tool in ipairs(tools) do
    print(tool.name, tool.category)
end
```

#### Tool.execute(name, params)
Invokes a tool by name with parameters.

```lua
local result = Tool.execute("calculator", {
    operation = "add",
    a = 5,
    b = 3
})
```

#### Tool.get(name)
Gets a tool instance by name.

```lua
local tool = Tool.get("calculator")
if tool then
    local result = tool.execute({operation = "add", a = 5, b = 3})
end
```

### Tool Discovery

#### Tool.discover(filter)
Discovers tools matching criteria.

```lua
local tools = Tool.discover({
    category = "data",
    capabilities = {"json"}
})
```

#### Tool.get_info(name)
Gets detailed tool information.

```lua
local info = Tool.get_info("calculator")
print(info.description)
print(info.parameters)
```

#### Tool.get_schema(name)
Gets the parameter schema for a tool.

```lua
local schema = Tool.get_schema("file-reader")
```

### Tool Availability

#### Tool.is_available(name)
Checks if a tool is available.

```lua
if Tool.is_available("calculator") then
    -- Use calculator
end
```

### Batch Operations

#### Tool.batch(operations)
Executes multiple tool operations.

```lua
local results = Tool.batch({
    {tool = "calculator", params = {operation = "add", a = 1, b = 2}},
    {tool = "calculator", params = {operation = "multiply", a = 3, b = 4}}
})
```

---

## Workflow

The `Workflow` global provides workflow orchestration capabilities.

### Workflow Builders

#### Workflow.sequential()
Creates a sequential workflow builder.

```lua
local workflow = Workflow.sequential()
    :name("data-pipeline")
    :step("load", {tool = "file-reader", params = {file = "data.json"}})
    :step("process", {agent = "processor", prompt = "Clean this data"})
    :step("save", {tool = "file-writer", params = {file = "output.json"}})
    :on_error("retry")  -- or "skip", "fail"
    :max_retries(3)
    :timeout(30000)
    :build()
```

#### Workflow.parallel()
Creates a parallel workflow builder.

```lua
local workflow = Workflow.parallel()
    :name("multi-search")
    :branch("web", {tool = "web-search", params = {query = "topic"}})
    :branch("docs", {tool = "doc-search", params = {query = "topic"}})
    :branch("db", {tool = "database", params = {query = "topic"}})
    :merge_strategy("combine")  -- or "first", "vote"
    :build()
```

#### Workflow.conditional()
Creates a conditional workflow builder.

```lua
local workflow = Workflow.conditional()
    :name("smart-router")
    :condition(function(context)
        return context.input_type == "code"
    end)
    :when_true({agent = "code-assistant"})
    :when_false({agent = "general-assistant"})
    :build()
```

#### Workflow.loop()
Creates a loop workflow builder.

```lua
local workflow = Workflow.loop()
    :name("data-processor")
    :condition(function(context)
        return context.items_remaining > 0
    end)
    :body({
        tool = "process-item",
        update = function(context, result)
            context.items_remaining = context.items_remaining - 1
            table.insert(context.results, result)
        end
    })
    :max_iterations(100)
    :build()
```

### Workflow Creation

#### Workflow.create(config)
Creates a workflow from configuration.

```lua
local workflow = Workflow.create({
    name = "my-workflow",
    type = "sequential",
    steps = {
        {id = "step1", action = {tool = "calculator"}},
        {id = "step2", action = {agent = "assistant"}}
    }
})
```

#### Workflow.from_yaml(yaml_string)
Creates a workflow from YAML.

```lua
local yaml = [[
name: my-workflow
type: sequential
steps:
  - id: fetch
    tool: web-fetch
  - id: process
    agent: processor
]]
local workflow = Workflow.from_yaml(yaml)
```

#### Workflow.from_file(filepath)
Loads a workflow from a file.

```lua
local workflow = Workflow.from_file("workflows/pipeline.yaml")
```

### Workflow Management

#### Workflow.list()
Lists all workflows.

```lua
local workflows = Workflow.list()
```

#### Workflow.get(name)
Gets a workflow by name.

```lua
local workflow = Workflow.get("data-pipeline")
```

#### Workflow.save(name, workflow)
Saves a workflow.

```lua
Workflow.save("my-pipeline", workflow)
```

#### Workflow.delete(name)
Deletes a saved workflow.

```lua
Workflow.delete("old-pipeline")
```

### Workflow Execution

#### workflow:execute(input)
Executes a workflow.

```lua
local result = workflow:execute({
    data = "input data",
    options = {verbose = true}
})
```

#### workflow:execute_async(input, callback)
Executes workflow asynchronously.

```lua
workflow:execute_async(input, function(result, error)
    if error then
        print("Error:", error)
    else
        print("Result:", result)
    end
end)
```

#### workflow:validate()
Validates workflow configuration.

```lua
local is_valid, errors = workflow:validate()
```

#### workflow:get_status()
Gets workflow execution status.

```lua
local status = workflow:get_status()
print(status.state)  -- "running", "completed", "failed"
```

### Workflow Result Structure

All workflows return a result with the following structure:

```lua
{
    success = true,              -- Overall success status
    execution_id = "uuid...",    -- Unique execution ID
    workflow_type = "sequential",-- Type of workflow
    status = "completed",        -- Workflow status
    metadata = {
        extra = {
            execution_id = "uuid...",  -- Execution ID (redundant, for convenience)
            agent_outputs = {          -- Collected agent outputs (if agents present)
                ["agent_id_timestamp"] = { ... },  -- Agent output JSON
                ...
            },
            ...
        }
    },
    ...
}
```

**Accessing Agent Outputs**:

```lua
local result = workflow:execute(input)

-- Option 1: Direct access
local outputs = result.metadata.extra.agent_outputs

-- Option 2: Safe access with fallback
local outputs = result.metadata and result.metadata.extra
    and result.metadata.extra.agent_outputs or {}

-- Use outputs
for agent_id, output in pairs(outputs) do
    -- Process output
end
```

---

## Custom Workflow Logic - Tool & Agent Patterns

**Note**: Custom step type was removed in v0.11. Use these superior patterns instead:

### Pattern 1: Custom Logic via Tools

For simple transformations, create a custom tool:

```lua
-- Instead of custom step:
-- workflow:add_step({ type = "custom", function = "transform", ... })

-- Use Tool pattern:
Tool.register("my-transformer", function(params)
    -- Your custom logic here
    local result = params.input:upper()
    return { text = result }
end)

workflow:add_step({
    type = "tool",
    tool = "my-transformer",
    input = { input = "hello" }
})
```

**Benefits**:
- ✅ Reusable across workflows
- ✅ Unit testable
- ✅ Discoverable via Tool.list()
- ✅ Supports full error handling

### Pattern 2: Custom Logic via Agents

For complex reasoning, create a custom agent:

```lua
-- Instead of custom step with complex logic:
-- workflow:add_step({ type = "custom", function = "analyze", ... })

-- Use Agent pattern:
local analyzer = Agent.create({
    name = "custom-analyzer",
    provider = "openai",
    model = "gpt-4o-mini",
    system_prompt = "Analyze the input and extract key insights."
})

workflow:add_step({
    type = "agent",
    agent = "custom-analyzer",
    input = "Analyze this text..."
})
```

**Benefits**:
- ✅ LLM-powered reasoning
- ✅ Natural language input
- ✅ Stateful across steps
- ✅ Supports streaming

### Pattern 3: Conditional Workflows for Branching Logic

For if/else logic:

```lua
-- Instead of custom step with branching:
-- workflow:add_step({ type = "custom", function = "route", ... })

-- Use Conditional workflow:
local router = Workflow.conditional()
    :name("smart-router")
    :condition("step:validation:output", "success")
    :when_true({ type = "tool", tool = "process-data" })
    :when_false({ type = "tool", tool = "handle-error" })
    :build()
```

### Pattern 4: Loop Workflows for Iteration

For custom iteration logic:

```lua
-- Instead of custom step with loop:
-- workflow:add_step({ type = "custom", function = "iterate", ... })

-- Use Loop workflow:
local processor = Workflow.loop()
    :name("batch-processor")
    :max_iterations(100)
    :body_step({ type = "tool", tool = "process-item" })
    :build()
```

### Pattern 5: Nested Workflows for Composition

For complex orchestration:

```lua
-- Instead of multiple custom steps:
-- workflow:add_step({ type = "custom", function = "step1", ... })
-- workflow:add_step({ type = "custom", function = "step2", ... })

-- Use nested workflows:
local preprocessing = Workflow.sequential()
    :name("preprocessing")
    :add_step({ type = "tool", tool = "validate" })
    :add_step({ type = "tool", tool = "transform" })
    :build()

local main = Workflow.sequential()
    :name("main-pipeline")
    :add_step({ type = "workflow", workflow = preprocessing })
    :add_step({ type = "agent", agent = "processor" })
    :build()
```

### Pattern 6: State Management for Custom Variables

For custom state tracking:

```lua
-- Use State API for custom variables
workflow:add_step({
    type = "tool",
    tool = "calculator",
    input = { operation = "add", values = {1, 2} }
})

-- Access results via state
local result = State.load("custom", ":workflow:my_flow:tool:calculator:output")

-- Or use agent_outputs for agents
local outputs = workflow_result.metadata.extra.agent_outputs
```

### Migration Examples

#### Example 1: Data Transformation

**Before (Custom Step - Didn't Work)**:
```lua
workflow:add_step({
    type = "custom",
    function = "data_transform",
    parameters = { format = "json" }
})
```

**After (Tool Pattern)**:
```lua
-- Create reusable tool
Tool.register("json-transformer", function(params)
    local data = JSON.parse(params.input)
    return { text = JSON.stringify(data) }
end)

workflow:add_step({
    type = "tool",
    tool = "json-transformer",
    input = { input = raw_data }
})
```

#### Example 2: Validation Logic

**Before (Custom Step - Didn't Work)**:
```lua
workflow:add_step({
    type = "custom",
    function = "validation",
    parameters = { rules = {...} }
})
```

**After (Agent Pattern)**:
```lua
local validator = Agent.create({
    name = "data-validator",
    provider = "anthropic",
    model = "claude-3-5-sonnet-20241022",
    system_prompt = "Validate data against these rules: ..."
})

workflow:add_step({
    type = "agent",
    agent = "data-validator",
    input = data_to_validate
})
```

#### Example 3: Conditional Processing

**Before (Custom Step - Didn't Work)**:
```lua
workflow:add_step({
    type = "custom",
    function = "check_and_route",
    parameters = { threshold = 0.8 }
})
```

**After (Conditional Workflow)**:
```lua
local router = Workflow.conditional()
    :condition("step:scorer:output", "> 0.8")
    :when_true({ type = "agent", agent = "high-quality-processor" })
    :when_false({ type = "agent", agent = "standard-processor" })
    :build()

main_workflow:add_step({
    type = "workflow",
    workflow = router
})
```

### Why These Patterns Are Better

| Feature | Custom Steps (Old) | Tools/Agents/Workflows (New) |
|---------|-------------------|------------------------------|
| **Functionality** | ❌ Mock only | ✅ Real execution |
| **Reusability** | ❌ None | ✅ Full reuse |
| **Testing** | ❌ Can't test | ✅ Unit testable |
| **Discovery** | ❌ Invisible | ✅ Tool.list(), Agent.discover() |
| **Documentation** | ❌ No docs | ✅ Tool.get("name").schema |
| **Error Handling** | ❌ Basic | ✅ Retry, fallback, hooks |
| **State Management** | ❌ Manual | ✅ Automatic |
| **Composition** | ❌ Limited | ✅ Nested workflows |
| **LLM Integration** | ❌ None | ✅ Agent pattern |

### Summary

Custom steps never provided real functionality - they were mocks. The tool/agent/workflow primitives are:
- ✅ **More powerful** - Full Turing-complete via tools + agents
- ✅ **Better architecture** - Single responsibility, composable
- ✅ **Easier to test** - Isolated, mockable components
- ✅ **Better UX** - Discoverable, documented, reusable

**Recommendation**: Always use tools for logic, agents for reasoning, workflows for orchestration.

---

## Session

The `Session` global manages user sessions and their persistence.

### Session Creation

#### Session.create(config)
Creates a new session.

```lua
local session = Session.create({
    id = "user-123",
    metadata = {
        username = "alice",
        created_at = os.time()
    }
})
```

#### Session.builder()
Creates a session builder.

```lua
local session = Session.builder()
    :name("my-session")
    :description("User session")
    :tag("production")
    :tag("user-session")
    :build()
```

### Session Management

#### Session.get(id)
Gets a session by ID.

```lua
local session = Session.get("user-123")
```

#### Session.list(filter)
Lists sessions with optional filter.

```lua
local sessions = Session.list({
    user = "alice",
    active = true
})
```

#### Session.get_current()
Gets the current active session.

```lua
local session = Session.get_current()
```

#### Session.set_current(id)
Sets the current active session.

```lua
Session.set_current("user-123")
```

### Session Lifecycle

#### Session.save(id)
Saves session to persistent storage.

```lua
Session.save("user-123")
```

#### Session.load(id)
Loads session from storage.

```lua
local session = Session.load("user-123")
```

#### Session.delete(id)
Deletes a session.

```lua
Session.delete("old-session")
```

#### Session.complete(id, summary)
Marks session as complete.

```lua
Session.complete("user-123", {
    total_queries = 10,
    duration = 3600
})
```

#### Session.suspend(id)
Suspends a session.

```lua
Session.suspend("user-123")
```

#### Session.resume(id)
Resumes a suspended session.

```lua
Session.resume("user-123")
```

### Session Replay

#### Session.can_replay(id)
Checks if session can be replayed.

```lua
if Session.can_replay("user-123") then
    -- Session has replay data
end
```

#### Session.replay(id, options)
Replays a session with specified configuration.

```lua
local results = Session.replay("user-123", {
    mode = "exact",  -- "exact", "modified", "simulate", or "debug"
    compare_results = true,
    timeout_seconds = 300,
    stop_on_error = false,
    metadata = {
        replay_reason = "testing",
        replayed_by = "admin"
    }
})

-- Result structure:
print(results.session_id)
print(results.correlation_id)
print(results.hooks_replayed)
print(results.successful_replays)
print(results.failed_replays)
print(results.total_duration)
```

#### Session.get_replay_metadata(id)
Gets replay metadata.

```lua
local meta = Session.get_replay_metadata("user-123")
```

#### Session.list_replayable()
Lists all replayable sessions.

```lua
local replayable = Session.list_replayable()
```

### Session Instance Methods

#### session:get_id()
Gets the session ID.

```lua
local id = session:get_id()
```

#### session:get_name()
Gets the session name.

```lua
local name = session:get_name()
```

#### session:get_status()
Gets the session status (e.g., "active", "completed", "suspended").

```lua
local status = session:get_status()
```

#### session:get_created_at()
Gets the session creation timestamp.

```lua
local created = session:get_created_at()
```

#### session:get_tags()
Gets all session tags.

```lua
local tags = session:get_tags()
for i, tag in ipairs(tags) do
    print("Tag:", tag)
end
```

#### session:add_tag(tag)
Adds a tag to the session.

```lua
session:add_tag("important")
```

#### session:has_tag(tag)
Checks if session has a specific tag.

```lua
if session:has_tag("production") then
    -- Handle production session
end
```

#### session:store_value(key, value)
Stores a value in the session.

```lua
session:store_value("last_query", "weather")
```

#### session:get_value(key)
Gets a value from the session.

```lua
local value = session:get_value("last_query")
```

---

## State

The `State` global provides persistent state management.

### Scoped State Operations

All state operations in LLMSpell use scopes for organization and isolation.

#### State.save(scope, key, value)
Saves a value to a scoped key.

```lua
State.save("user:123", "preferences", {
    theme = "dark",
    language = "en"
})
```

#### State.load(scope, key)
Loads a value from a scoped key.

```lua
local value = State.load("user:123", "preferences")
```

#### State.delete(scope, key)
Deletes a scoped key.

```lua
State.delete("user:123", "temp_state")
```

#### State.list_keys(scope)
Lists all keys in a scope.

```lua
local keys = State.list_keys("user:123")
for i, key in ipairs(keys) do
    print("Key:", key)
end
```

### Component-Specific State

#### State.workflow_get(workflow_id, step_name)
Gets state for a specific workflow step.

```lua
local step_state = State.workflow_get("workflow-123", "validate")
```

#### State.workflow_list(workflow_id)
Lists all state keys for a workflow.

```lua
local keys = State.workflow_list("workflow-123")
```

#### State.agent_get(agent_id, key)
Gets agent-specific state.

```lua
local agent_state = State.agent_get("agent-456", "memory")
```

#### State.agent_set(agent_id, key, value)
Sets agent-specific state.

```lua
State.agent_set("agent-456", "memory", {last_interaction = os.time()})
```

#### State.tool_get(tool_id, key)
Gets tool-specific state.

```lua
local tool_state = State.tool_get("calculator", "history")
```

#### State.tool_set(tool_id, key, value)
Sets tool-specific state.

```lua
State.tool_set("calculator", "history", {last_calc = "2+2=4"})
```

### State Migrations

#### State.migrate(version)
Migrates state to a new version.

```lua
State.migrate("2.0.0")
```

#### State.get_migration_status()
Gets migration status.

```lua
local status = State.get_migration_status()
```

#### State.get_schema_versions()
Gets schema version history.

```lua
local versions = State.get_schema_versions()
```

### State Backups

#### State.create_backup(name)
Creates a state backup.

```lua
local backup_id = State.create_backup("before_upgrade")
```

#### State.list_backups()
Lists available backups.

```lua
local backups = State.list_backups()
```

#### State.restore_backup(id)
Restores from a backup.

```lua
State.restore_backup("backup_123")
```

#### State.validate_backup(id)
Validates a backup.

```lua
local is_valid = State.validate_backup("backup_123")
```

#### State.cleanup_backups(keep_count)
Cleans up old backups.

```lua
State.cleanup_backups(5)  -- Keep only 5 most recent
```

### State Utilities

#### State.get_storage_usage()
Gets storage usage statistics.

```lua
local usage = State.get_storage_usage()
print(usage.bytes_used, usage.entries_count)
```

---

## Memory

The `Memory` global provides access to LLMSpell's adaptive memory system with three subsystems:
- **Episodic**: Conversation history and interactions (temporal memory)
- **Semantic**: Knowledge graph with entities and relationships (conceptual memory)
- **Consolidation**: LLM-driven extraction of knowledge from conversations

**Examples**: See `examples/script-users/getting-started/06-episodic-memory-basic.lua`

### Episodic Memory

Stores conversation exchanges with automatic timestamping and embedding generation.

#### Memory.episodic.add(session_id, role, content, metadata)

Adds a conversation exchange to episodic memory.

**Parameters**:
- `session_id` (string): Session identifier for isolation
- `role` (string): Speaker role (`"user"`, `"assistant"`, `"system"`)
- `content` (string): Message content
- `metadata` (table, optional): Additional metadata as key-value pairs

**Returns**: String (entry ID)

**Example**:
```lua
local id = Memory.episodic.add(
    "session-123",
    "user",
    "What is Rust?",
    {topic = "programming", priority = "high"}
)
print("Added entry: " .. id)
```

**Notes**:
- Session IDs enable conversation isolation
- Metadata is stored and searchable
- Entries are automatically timestamped
- Embeddings are generated asynchronously

#### Memory.episodic.search(session_id, query, limit)

Searches episodic memory using semantic similarity.

**Parameters**:
- `session_id` (string): Session ID to filter by (empty string = all sessions)
- `query` (string): Search query text
- `limit` (number, optional): Maximum results (default: 10)

**Returns**: Array of entry tables

**Entry Structure**:
```lua
{
    id = "entry-uuid",
    session_id = "session-123",
    role = "user",
    content = "What is Rust?",
    metadata = {topic = "programming"},
    created_at = "2025-01-27T10:30:00Z"
}
```

**Example**:
```lua
local entries = Memory.episodic.search("session-123", "ownership", 5)
for i, entry in ipairs(entries) do
    print(string.format("[%s] %s", entry.role, entry.content))
end
```

**Notes**:
- Returns entries sorted by relevance
- Empty session_id searches all sessions
- Combines vector similarity + metadata filtering

### Semantic Memory

Stores structured knowledge as entities in a knowledge graph.

#### Memory.semantic.query(query, limit)

Queries the knowledge graph for relevant entities.

**Parameters**:
- `query` (string): Query text
- `limit` (number, optional): Maximum results (default: 10)

**Returns**: Array of entity tables

**Example**:
```lua
local entities = Memory.semantic.query("Rust programming", 5)
for _, entity in ipairs(entities) do
    print(entity.name .. ": " .. entity.description)
end
```

**Notes**:
- Entities are populated via consolidation
- Semantic memory is global (not session-specific)

### Consolidation

Converts episodic memories into semantic knowledge using LLM analysis.

#### Memory.consolidate(session_id, force)

Runs consolidation to extract knowledge from conversations.

**Parameters**:
- `session_id` (string, optional): Session to consolidate (nil = all sessions)
- `force` (boolean, optional): Force immediate consolidation (default: false)

**Returns**: Table with consolidation statistics

**Example**:
```lua
local stats = Memory.consolidate("session-123", true)
print(string.format("Processed %d entries", stats.entries_processed))
print(string.format("Extracted %d entities", stats.entities_added))
```

**Notes**:
- Background mode processes periodically
- Immediate mode blocks until complete
- Requires LLM provider for entity extraction

### Statistics

#### Memory.stats()

Returns memory system statistics.

**Returns**: Table with counts and status

**Example**:
```lua
local stats = Memory.stats()
print(string.format("Episodic: %d entries", stats.episodic_count))
print(string.format("Semantic: %d entities", stats.semantic_count))
print(string.format("Pending: %d sessions", stats.sessions_with_unprocessed))
```

**Returned Fields**:
- `episodic_count`: Number of conversation entries
- `semantic_count`: Number of knowledge entities
- `sessions_with_unprocessed`: Sessions awaiting consolidation
- `has_episodic`: Boolean indicating episodic availability
- `has_semantic`: Boolean indicating semantic availability
- `has_consolidation`: Boolean indicating consolidation availability

---

## Context

The `Context` global provides intelligent context assembly from memory using retrieval strategies.

**Examples**: See `examples/script-users/getting-started/07-context-assembly-basic.lua`

### Context Assembly

#### Context.assemble(query, strategy, max_tokens, session_id)

Assembles relevant context from memory for a query.

**Parameters**:
- `query` (string): Query text to find relevant context for
- `strategy` (string): Retrieval strategy (`"episodic"`, `"semantic"`, or `"hybrid"`)
- `max_tokens` (number, optional): Token budget for context (default: 8192)
- `session_id` (string, optional): Session to filter by (episodic strategy only)

**Returns**: Table with assembled context

**Result Structure**:
```lua
{
    chunks = {           -- Array of ranked chunks
        {
            chunk = {
                role = "user",
                content = "What is Rust?",
                source = "session-123",
                timestamp = "2025-01-27T10:30:00Z",
                metadata = {}
            },
            score = 0.95,     -- Relevance score 0-1
            ranker = "bm25"   -- Reranking method used
        }
    },
    total_confidence = 0.87,  -- Average relevance
    temporal_span = {         -- Time range of chunks
        "2025-01-27T10:00:00Z",
        "2025-01-27T11:00:00Z"
    },
    token_count = 1234,       -- Total tokens in context
    formatted = "..."         -- LLM-ready formatted context
}
```

**Example**:
```lua
local context = Context.assemble(
    "How does ownership work in Rust?",
    "episodic",
    2000,
    "session-123"
)

print(string.format("Retrieved %d chunks using %d tokens",
    #context.chunks, context.token_count))

-- Use formatted context in LLM prompt
local prompt = context.formatted .. "\\n\\nUser: " .. user_query
```

**Strategies**:
- `"episodic"`: Recent conversation history (fast, temporal)
- `"semantic"`: Knowledge graph entities (conceptual, global)
- `"hybrid"`: Combined episodic + semantic (comprehensive)

**Notes**:
- Token budget enforced via truncation
- Chunks are reranked by relevance (BM25)
- Formatted context ready for LLM prompting
- Min token budget: 100, warn if >8192

#### Context.test(query, session_id)

Quick test of context assembly with hybrid strategy and 2000 token budget.

**Parameters**:
- `query` (string): Test query
- `session_id` (string, optional): Session filter

**Returns**: Same as `Context.assemble()`

**Example**:
```lua
local result = Context.test("test query", "session-123")
print(string.format("Test retrieved %d chunks", #result.chunks))
```

### Strategy Statistics

#### Context.strategy_stats()

Returns statistics about available retrieval strategies.

**Returns**: Table with strategy information

**Example**:
```lua
local stats = Context.strategy_stats()
print(string.format("Episodic: %d entries", stats.episodic_count))
print(string.format("Semantic: %d entities", stats.semantic_count))
print("Strategies: " .. table.concat(stats.strategies, ", "))
```

**Returned Fields**:
- `episodic_count`: Number of episodic entries available
- `semantic_count`: Number of semantic entities available
- `strategies`: Array of available strategy names

### Integration Pattern

Typical workflow combining Memory and Context:

```lua
-- 1. Store conversation
Memory.episodic.add(session_id, "user", user_message, {})

-- 2. Assemble relevant context
local context = Context.assemble(
    user_message,
    "episodic",
    4000,
    session_id
)

-- 3. Build LLM prompt with context
local prompt = context.formatted .. "\\n\\nUser: " .. user_message

-- 4. Call LLM (Agent, Tool, or direct API)
local response = Agent.create("gpt-4"):prompt(prompt)

-- 5. Store response
Memory.episodic.add(session_id, "assistant", response, {})
```

---

## Event

The `Event` global provides event publishing and subscription.

### Publishing Events

#### Event.publish(topic, data)
Publishes an event.

```lua
local success = Event.publish("user.login", {
    user_id = "123",
    timestamp = os.time()
})
```

### Subscribing to Events

#### Event.subscribe(pattern)
Subscribes to events matching a pattern.

```lua
local subscription_id = Event.subscribe("user.*")
```

#### Event.receive(subscription_id, timeout_ms)
Receives events for a subscription.

```lua
local event = Event.receive(subscription_id, 5000)
if event then
    print(event.topic, event.data)
end
```

#### Event.unsubscribe(subscription_id)
Unsubscribes from events.

```lua
Event.unsubscribe(subscription_id)
```

### Event Management

#### Event.list_subscriptions()
Lists active subscriptions.

```lua
local subs = Event.list_subscriptions()
for i, sub in ipairs(subs) do
    print(sub.id, sub.pattern)
end
```

#### Event.get_stats()
Gets event system statistics.

```lua
local stats = Event.get_stats()
print(stats.event_bus_stats.total_published)
print(stats.bridge_stats.active_subscriptions)
```

---

## Hook

The `Hook` global provides hook registration for intercepting operations.

### Hook Registration

#### Hook.register(hook_point, callback, priority)
Registers a hook.

```lua
local handle = Hook.register("BeforeToolExecution", function(context)
    print("Tool executing:", context.data.tool_name)
    return "continue"  -- or "skip", "cancel", {type = "modified", data = {...}}
end, "normal")  -- priority: "highest", "high", "normal", "low", "lowest"
```

### Hook Points

Available hook points:
- `SystemStartup`, `SystemShutdown`
- `BeforeAgentInit`, `AfterAgentInit`
- `BeforeAgentExecution`, `AfterAgentExecution`
- `BeforeAgentShutdown`, `AfterAgentShutdown`
- `AgentError`
- `BeforeToolDiscovery`, `AfterToolDiscovery`
- `BeforeToolExecution`, `AfterToolExecution`
- `ToolValidation`, `ToolError`
- `BeforeWorkflowStart`, `AfterWorkflowComplete`
- `WorkflowStageTransition`, `BeforeWorkflowStage`, `AfterWorkflowStage`
- `WorkflowCheckpoint`, `WorkflowRollback`, `WorkflowError`

### Hook Results

Hooks can return:
- `"continue"` - Continue normal execution
- `"skip"` or `"skipped"` - Skip this operation
- `"cancel"` - Cancel the operation
- `{type = "modified", data = {...}}` - Modify the data
- `{type = "redirect", target = "..."}` - Redirect to another target
- `{type = "replace", data = {...}}` - Replace the data
- `{type = "retry", delay_ms = 1000, max_attempts = 3}` - Retry with delay

### Hook Management

#### Hook.list(filter)
Lists registered hooks.

```lua
-- List all hooks
local hooks = Hook.list()

-- List hooks for specific point
local hooks = Hook.list("BeforeToolExecution")

-- List with complex filter
local hooks = Hook.list({
    hook_point = "BeforeToolExecution",
    language = "lua",
    priority = "high",
    tag = "security"
})
```

#### Hook.unregister(handle)
Unregisters a hook.

```lua
Hook.unregister(handle)
-- or
handle:unregister()
```

### Hook Handle Methods

#### handle:id()
Gets the hook ID.

```lua
local id = handle:id()
```

#### handle:hook_point()
Gets the hook point.

```lua
local point = handle:hook_point()
```

---

## RAG

The `RAG` global provides Retrieval-Augmented Generation with vector storage, including temporal metadata support for Phase 9's Adaptive Memory System.

### Vector Search

#### RAG.search(query, options)
Searches for similar vectors.

```lua
local results = RAG.search("How do I create an agent?", {
    limit = 5,            -- Number of results (also accepts 'k' or 'top_k')
    threshold = 0.7,      -- Similarity threshold (0-1)
    collection = "documentation",
    scope = "tenant",     -- Scope type: "global", "tenant", "session", etc.
    scope_id = "org-123", -- Scope identifier
    tenant_id = "org-123" -- Alternative to scope/scope_id for tenant isolation
})
```

**Note**: Temporal query filters (`event_time_range`, `exclude_expired`) are implemented in the storage layer but not yet exposed through the Lua API. This functionality will be available in a future update.

### Data Ingestion

#### RAG.ingest(data, options)
Ingests data into vector storage with support for temporal metadata.

```lua
-- Single document with temporal metadata
local success = RAG.ingest({
    content = "Agent creation guide...",
    metadata = {
        source = "docs/agents.md",
        type = "documentation",
        -- Temporal metadata (Phase 8.11.2)
        timestamp = 1699564800,        -- Unix timestamp for event_time
        created_at = "2024-11-09T12:00:00Z", -- ISO 8601 timestamp (alternative)
        event_time = 1699564800,       -- When the event actually occurred
        ttl = 86400,                   -- Time-to-live in seconds (24 hours)
        ttl_seconds = 86400,           -- Alternative TTL field
        expires_in = 3600              -- Expire in 1 hour (alternative)
    }
}, {
    collection = "documentation",
    chunk_size = 500,
    chunk_overlap = 50,
    tenant_id = "org-123"  -- For multi-tenant isolation
})

-- Multiple documents
local success = RAG.ingest({
    {
        content = "First document",
        metadata = { 
            timestamp = os.time() - 3600,  -- Event from 1 hour ago
            ttl = 7200                     -- Expires in 2 hours
        }
    },
    {
        content = "Second document",
        metadata = {
            event_time = os.time() - 86400, -- Event from yesterday
            ttl_seconds = 604800            -- Expires in 1 week
        }
    }
})
```

**Temporal Metadata Fields**:
- **`timestamp`**, **`created_at`**, or **`event_time`**: When the actual event occurred (Unix timestamp or ISO 8601 string)
- **`ttl`**, **`ttl_seconds`**, or **`expires_in`**: Time-to-live in seconds before the vector expires
- The system automatically tracks:
  - `created_at`: When the vector was ingested (set automatically)
  - `updated_at`: When the vector was last modified (set automatically)
  - `expires_at`: Calculated from TTL if provided

### Configuration

#### RAG.configure(options)
Configures RAG settings.

```lua
RAG.configure({
    provider = "openai",
    embedding_model = "text-embedding-ada-002",
    vector_dimensions = 1536
})
```

#### RAG.list_providers()
Lists available RAG providers.

```lua
local providers = RAG.list_providers()
```

### Session Collections

#### RAG.create_session_collection(session_id, options)
Creates a session-specific collection.

```lua
local collection = RAG.create_session_collection("session-123", {
    ttl = 3600,
    max_vectors = 1000
})
```

#### RAG.configure_session(session_id, options)
Configures session-specific settings.

```lua
RAG.configure_session("session-123", {
    auto_ingest = true,
    persist = false
})
```

### Temporal Queries (Future)

The storage layer supports bi-temporal queries for Phase 9's Adaptive Memory System. These will be exposed through the Lua API in a future update:

```lua
-- Future API - not yet available in Lua
local results = RAG.search("query", {
    -- Filter by when events actually occurred
    event_time_range = {
        from = os.time() - 86400,  -- Yesterday
        to = os.time()              -- Now
    },
    
    -- Filter by when we learned about the events
    ingestion_time_range = {
        from = os.time() - 3600,    -- Last hour
        to = os.time()              -- Now
    },
    
    -- Exclude expired vectors
    exclude_expired = true
})
```

**Bi-temporal Model Benefits**:
- Query "What did we know last week about events from last month?"
- Find recent events that were just discovered
- Implement memory consolidation based on age and relevance
- Automatic cleanup of expired memories

### Management

#### RAG.cleanup_scope(scope)
Cleans up vectors in a scope.

```lua
RAG.cleanup_scope("session:123")
```

#### RAG.get_stats(scope, scope_id)
Gets RAG statistics for a specific scope.

```lua
-- Global stats
local stats = RAG.get_stats("global", nil)

-- Tenant-specific stats
local tenant_stats = RAG.get_stats("tenant", "org-123")

-- Session-specific stats
local session_stats = RAG.get_stats("session", "session-uuid")

if stats then
    print(stats.total_vectors)
    print(stats.storage_bytes)
    print(stats.namespace_count)
end
```

#### RAG.save()
Saves RAG state to persistent storage.

```lua
RAG.save()
```

---

## LocalLLM

The `LocalLLM` global provides local model management for Ollama and Candle backends (Phase 11).

### Backend Status

#### LocalLLM.status()
Checks health status of local backends.

```lua
local status = LocalLLM.status()

-- Ollama backend
print("Ollama running:", status.ollama.running)
print("Ollama models:", status.ollama.models)
if status.ollama.version then
    print("Ollama version:", status.ollama.version)
end
if status.ollama.error then
    print("Ollama error:", status.ollama.error)
end

-- Candle backend
print("Candle ready:", status.candle.ready)
print("Candle models:", status.candle.models)
if status.candle.version then
    print("Candle version:", status.candle.version)
end
if status.candle.error then
    print("Candle error:", status.candle.error)
end
```

### Model Management

#### LocalLLM.list(backend)
Lists local models from specified backend(s).

```lua
-- List all models from all backends
local models = LocalLLM.list()

-- List from specific backend
local ollama_models = LocalLLM.list("ollama")
local candle_models = LocalLLM.list("candle")

-- Iterate results
for _, model in ipairs(models) do
    print("ID:", model.id)
    print("Backend:", model.backend)
    print("Size:", model.size_bytes, "bytes")
    if model.quantization then
        print("Quantization:", model.quantization)
    end
    if model.modified_at then
        print("Modified:", os.date("%Y-%m-%d", model.modified_at))
    end
end
```

#### LocalLLM.pull(spec)
Downloads a model from backend library.

```lua
-- Pull from Ollama
local progress = LocalLLM.pull("llama3.1:8b@ollama")

-- Pull from Candle
local progress = LocalLLM.pull("mistral:7b@candle")

-- Check progress
print("Model:", progress.model_id)
print("Status:", progress.status)  -- "starting", "downloading", "verifying", "complete", "failed"
print("Progress:", progress.percent_complete .. "%")
print("Downloaded:", progress.bytes_downloaded, "bytes")
if progress.bytes_total then
    print("Total:", progress.bytes_total, "bytes")
end
if progress.error then
    print("Error:", progress.error)
end
```

#### LocalLLM.info(model_id)
Gets detailed information about a specific model.

```lua
local info = LocalLLM.info("llama3.1:8b")

print("ID:", info.id)
print("Backend:", info.backend)
print("Size:", info.size_bytes, "bytes")
print("Format:", info.format)
print("Loaded:", info.loaded)

if info.parameter_count then
    print("Parameters:", info.parameter_count)
end
if info.quantization then
    print("Quantization:", info.quantization)
end
```

### Model Specification Format

Models are specified using the format: `model_name[:tag][@backend]`

Examples:
- `llama3.1:8b@ollama` - Llama 3.1 8B from Ollama
- `mistral:7b@candle` - Mistral 7B from Candle
- `tinyllama@candle` - TinyLlama from Candle (default tag)

---

## Config

The `Config` global provides access to configuration.

### Configuration Access

#### Config.get()
Gets full configuration.

```lua
local config = Config.get()
print(config.default_engine)
```

#### Config.getSection(name)
Gets a specific configuration section.

```lua
local tools_config = Config.getSection("tools")
```

#### Config.getDefaultEngine()
Gets the default engine name.

```lua
local engine = Config.getDefaultEngine()
```

### Provider Configuration

#### Config.getProvider(name)
Gets provider configuration.

```lua
local openai_config = Config.getProvider("openai")
```

#### Config.listProviders()
Lists all configured providers.

```lua
local providers = Config.listProviders()
```

#### Config.setProvider(name, config)
Sets provider configuration (if permitted).

```lua
Config.setProvider("custom", {
    provider_type = "openai",
    api_key_env = "CUSTOM_API_KEY",
    model = "gpt-4"
})
```

### Security Configuration

#### Config.getSecurity()
Gets security settings.

```lua
local security = Config.getSecurity()
print(security.allow_file_access)
```

#### Config.setSecurity(config)
Sets security configuration (dangerous!).

```lua
Config.setSecurity({
    allow_file_access = true,
    allow_network_access = true
})
```

#### Config.isFileAccessAllowed()
Checks if file access is allowed.

```lua
if Config.isFileAccessAllowed() then
    -- Can access files
end
```

#### Config.isNetworkAccessAllowed()
Checks if network access is allowed.

```lua
if Config.isNetworkAccessAllowed() then
    -- Can make network requests
end
```

---

### Security & Permissions

> **📚 Complete Guide**: See [Security & Permissions Guide](../../security-and-permissions.md) for comprehensive configuration, troubleshooting, and scenarios.

#### Understanding Security Constraints

LLMSpell scripts run in a sandboxed environment with three security levels:

- **Safe**: Pure computation, no file/network/process access (e.g., calculator, hash-calculator)
- **Restricted** (default): Explicit permissions required via config.toml
- **Privileged**: Full access (rare, requires admin approval)

Most tools operate at **Restricted** level, requiring explicit configuration.

#### Checking Permissions Before Use

**Pattern**: Check before executing to provide helpful error messages

```lua
-- Network access check
if Config.isNetworkAccessAllowed() then
    local result = Tool.execute("http-request", {
        method = "GET",
        url = "https://api.example.com/data"
    })
else
    print("❌ Network access denied")
    print("Add to config.toml:")
    print("[tools.http_request]")
    print('allowed_hosts = ["api.example.com"]')
end

-- File access check
if Config.isFileAccessAllowed() then
    local data = Tool.execute("file-operations", {
        operation = "read",
        path = "/workspace/data.txt"
    })
else
    print("❌ File access denied")
    print("Add to config.toml:")
    print("[tools.file_operations]")
    print('allowed_paths = ["/workspace"]')
end

-- Process execution check (via config)
local can_execute = Config.get("tools.system.allow_process_execution")
if can_execute then
    Tool.execute("process-executor", {
        executable = "echo",
        arguments = {"Hello"}
    })
else
    print("❌ Process execution disabled")
    print("Set in config.toml:")
    print("[tools.system]")
    print("allow_process_execution = true")
end
```

#### Handling Permission Errors

**Pattern**: Use `pcall()` to catch and handle permission errors gracefully

```lua
-- Wrap tool calls to catch errors
local success, result = pcall(function()
    return Tool.execute("http-request", {
        method = "GET",
        url = "https://blocked-domain.com/api"
    })
end)

if not success then
    local error_msg = tostring(result)

    if error_msg:match("Domain not in allowed list") or
       error_msg:match("Host blocked") then
        print("❌ ERROR: Domain not allowed")
        print("Solution: Add domain to config.toml:")
        print("[tools.http_request]")
        print('allowed_hosts = ["blocked-domain.com"]')

    elseif error_msg:match("Path not in allowlist") or
           error_msg:match("Permission denied") then
        print("❌ ERROR: File access denied")
        print("Solution: Add path to config.toml:")
        print("[tools.file_operations]")
        print('allowed_paths = ["/your/path"]')

    elseif error_msg:match("Command blocked") or
           error_msg:match("Executable not allowed") then
        print("❌ ERROR: Process execution denied")
        print("Solution: Enable in config.toml:")
        print("[tools.system]")
        print("allow_process_execution = true")
        print('allowed_commands = "echo,cat,ls"')

    else
        print("❌ ERROR: " .. error_msg)
        print("See docs/user-guide/security-and-permissions.md")
    end
end
```

#### Permission Configuration (Admin Only)

**Important**: Lua scripts **CANNOT modify security settings**. Permissions must be configured in `config.toml`:

```toml
# config.toml - Network access example
[tools.web_search]
allowed_domains = ["api.example.com", "*.github.com"]
rate_limit_per_minute = 100

[tools.http_request]
allowed_hosts = ["api.example.com", "*.trusted.com"]
blocked_hosts = ["localhost", "127.0.0.1"]  # SSRF prevention

# Process execution example
[tools.system]
allow_process_execution = false  # Set true to enable
allowed_commands = "echo,cat,ls,pwd"  # Comma-separated allowlist
command_timeout_seconds = 30

# File access example
[tools.file_operations]
allowed_paths = ["/workspace", "/tmp", "/data"]
max_file_size = 50000000  # 50MB
blocked_extensions = ["exe", "dll", "so"]
```

> **⚠️ Security Note**: `Config.setSecurity()` is available only for development/testing. Production scripts cannot modify security settings.

#### Best Practices

1. **Check permissions before use**: Use `Config.is*Allowed()` to detect missing permissions early
   ```lua
   if not Config.isNetworkAccessAllowed() then
       error("Script requires network access. Configure [tools.network] in config.toml")
   end
   ```

2. **Handle permission errors gracefully**: Always use `pcall()` and provide helpful error messages
   ```lua
   local success, result = pcall(function()
       return Tool.execute("http-request", {...})
   end)
   if not success then
       print("Error with helpful config fix suggestion")
   end
   ```

3. **Request minimal permissions**: Follow principle of least privilege
   - Only request paths you actually need
   - Only request domains you actually access
   - Only enable commands you actually use

4. **Document required permissions**: Add comments to your scripts
   ```lua
   -- REQUIRED CONFIG:
   -- [tools.http_request]
   -- allowed_hosts = ["api.example.com"]
   --
   -- [tools.file_operations]
   -- allowed_paths = ["/workspace/data"]

   local data = fetch_and_save()
   ```

5. **Test permission boundaries**: Verify your script handles missing permissions
   ```lua
   -- Test without permissions first
   -- Then add minimal permissions
   -- Verify error messages are helpful
   ```

#### Common Permission Errors

| Error Message | Solution |
|--------------|----------|
| "Network access denied" | Add `[tools.http_request]` with `allowed_hosts` |
| "Domain not in allowed list" | Add domain to `allowed_domains` in `[tools.web_search]` |
| "Path not in allowlist" | Add path to `allowed_paths` in `[tools.file_operations]` |
| "Command blocked" | Set `allow_process_execution = true` and add to `allowed_commands` |
| "Executable not allowed" | Add executable to `allowed_commands` in `[tools.system]` |
| "File extension blocked" | Remove from `blocked_extensions` or add to `allowed_extensions` |

---

### Tools Configuration

#### Config.getTools()
Gets tools configuration.

```lua
local tools = Config.getTools()
```

#### Config.addAllowedPath(path)
Adds an allowed path for file operations.

```lua
Config.addAllowedPath("/tmp/myapp")
```

### Permissions

#### Config.getPermissions()
Gets current permissions.

```lua
local perms = Config.getPermissions()
print(perms.modify_providers)
print(perms.modify_security)
```

### Configuration Management

#### Config.snapshot()
Creates a configuration snapshot.

```lua
Config.snapshot()
```

#### Config.restoreSnapshot(timestamp)
Restores from a snapshot.

```lua
Config.restoreSnapshot(1234567890)
```

#### Config.toJson()
Exports configuration as JSON.

```lua
local json = Config.toJson()
```

---

## Provider

The `Provider` global provides LLM provider information.

### Provider Information

#### Provider.list()
Lists all available providers.

```lua
local providers = Provider.list()
for i, provider in ipairs(providers) do
    print(provider.name, provider.enabled)
    if provider.capabilities then
        print("  Streaming:", provider.capabilities.supports_streaming)
        print("  Multimodal:", provider.capabilities.supports_multimodal)
        print("  Max tokens:", provider.capabilities.max_context_tokens)
    end
end
```

#### Provider.get(name)
Gets specific provider information.

```lua
local provider = Provider.get("openai")
if provider then
    print(provider.name, provider.enabled)
end
```

#### Provider.getCapabilities(name)
Gets provider capabilities.

```lua
local caps = Provider.getCapabilities("anthropic")
if caps then
    print("Models:", table.concat(caps.available_models, ", "))
end
```

#### Provider.isAvailable(name)
Checks if a provider is available.

```lua
if Provider.isAvailable("openai") then
    -- Provider is configured and enabled
end
```

---

## Artifact

The `Artifact` global manages session artifacts.

### Storing Artifacts

#### Artifact.store(session_id, type, name, content, metadata)
Stores an artifact.

```lua
local artifact_id = Artifact.store("session-123", "data", "results.json", 
    '{"data": "results"}', {
        created_by = "processor",
        version = "1.0"
    })
```

#### Artifact.store_file(session_id, file_path, type, metadata)
Stores a file as an artifact.

```lua
local artifact_id = Artifact.store_file("session-123", 
    "/tmp/report.pdf", "document", {
        title = "Analysis Report"
    })
```

### Retrieving Artifacts

#### Artifact.get(session_id, artifact_id)
Gets an artifact.

```lua
local artifact = Artifact.get("session-123", artifact_id)
print(artifact.content)
print(artifact.metadata)
```

#### Artifact.list(session_id)
Lists artifacts for a session.

```lua
local artifacts = Artifact.list("session-123")
```

#### Artifact.query(query)
Queries artifacts with filters.

```lua
local artifacts = Artifact.query({
    session_id = "session-123",
    type = "data",
    name_pattern = "results*",
    tags = {"processed"},
    created_after = "2024-01-01T00:00:00Z",
    limit = 10
})
```

### Managing Artifacts

#### Artifact.delete(session_id, artifact_id)
Deletes an artifact.

```lua
Artifact.delete("session-123", artifact_id)
```

---

## Replay

The `Replay` global provides hook replay functionality for testing.

### Replay Modes

#### Replay.modes
Available replay modes.

```lua
local mode = Replay.modes.exact     -- Exact replay
local mode = Replay.modes.modified  -- With modifications
local mode = Replay.modes.simulate  -- Simulation mode
local mode = Replay.modes.debug     -- Debug mode
```

### Replay Configuration

#### Replay.create_config(mode)
Creates a replay configuration.

```lua
local config = Replay.create_config(Replay.modes.modified)
config:add_modification("params.temperature", 0.5, true)
```

#### Replay.create_modification(path, value, enabled)
Creates a parameter modification.

```lua
local mod = Replay.create_modification("params.max_tokens", 1000, true)
```

### Replay Scheduling

#### Replay.schedules.once(delay_seconds)
Creates a one-time schedule.

```lua
local schedule = Replay.schedules.once(5.0)
```

#### Replay.schedules.interval(initial_delay, interval, max_executions)
Creates an interval schedule.

```lua
local schedule = Replay.schedules.interval(0, 60, 10)
```

#### Replay.schedules.cron(expression)
Creates a cron schedule.

```lua
local schedule = Replay.schedules.cron("0 */5 * * * *")
```

### Result Comparison

#### Replay.create_comparator()
Creates a result comparator.

```lua
local comparator = Replay.create_comparator()
local comparison = comparator:compare_json(original, replayed)
print(comparison.identical)
print(comparison.similarity_score)
print(comparison.summary)
```

---

## Debug

The `Debug` global provides comprehensive debugging utilities.

### Logging

#### Debug.log(level, message, module)
Logs a message at specified level.

```lua
Debug.log("info", "Processing started", "processor")
```

#### Debug.trace(message, module)
Logs at trace level.

```lua
Debug.trace("Detailed trace info", "module")
```

#### Debug.debug(message, module)
Logs at debug level.

```lua
Debug.debug("Debug information", "module")
```

#### Debug.info(message, module)
Logs at info level.

```lua
Debug.info("Processing complete", "module")
```

#### Debug.warn(message, module)
Logs at warning level.

```lua
Debug.warn("Deprecated function used", "module")
```

#### Debug.error(message, module)
Logs at error level.

```lua
Debug.error("Failed to process", "module")
```

#### Debug.logWithData(level, message, data, module)
Logs with structured data.

```lua
Debug.logWithData("info", "Request processed", {
    duration = 1500,
    status = 200
}, "api")
```

### Timing

#### Debug.timer(name)
Creates a timer.

```lua
local timer = Debug.timer("operation")
-- Do work...
local duration = timer:stop()
```

Timer methods:
- `timer:stop()` - Stops timer and returns duration
- `timer:lap(name)` - Records a lap time
- `timer:elapsed()` - Gets elapsed time without stopping

### Debug Configuration

#### Debug.setLevel(level)
Sets the debug level.

```lua
Debug.setLevel("debug")
```

#### Debug.getLevel()
Gets current debug level.

```lua
local level = Debug.getLevel()
```

#### Debug.setEnabled(enabled)
Enables/disables debugging.

```lua
Debug.setEnabled(true)
```

#### Debug.isEnabled()
Checks if debugging is enabled.

```lua
if Debug.isEnabled() then
    -- Debug is on
end
```

### Filtering

#### Debug.addModuleFilter(pattern, enabled)
Adds a module filter.

```lua
Debug.addModuleFilter("api.*", true)
Debug.addModuleFilter("verbose.*", false)
```

#### Debug.clearModuleFilters()
Clears all module filters.

```lua
Debug.clearModuleFilters()
```

#### Debug.removeModuleFilter(pattern)
Removes a specific filter.

```lua
Debug.removeModuleFilter("api.*")
```

#### Debug.setDefaultFilterEnabled(enabled)
Sets default filter behavior.

```lua
Debug.setDefaultFilterEnabled(false)
```

#### Debug.addAdvancedFilter(pattern, pattern_type, enabled)
Adds an advanced filter.

```lua
Debug.addAdvancedFilter("api.*", "wildcard", true)
Debug.addAdvancedFilter("^core\\..*", "regex", false)
```

#### Debug.getFilterSummary()
Gets filter configuration summary.

```lua
local summary = Debug.getFilterSummary()
print(summary.total_rules)
```

### Capture

#### Debug.getCapturedEntries(limit)
Gets captured debug entries.

```lua
local entries = Debug.getCapturedEntries(100)
for i, entry in ipairs(entries) do
    print(entry.timestamp, entry.level, entry.message)
end
```

#### Debug.clearCaptured()
Clears captured entries.

```lua
Debug.clearCaptured()
```

### Value Dumping

#### Debug.dump(value, label)
Dumps a value with formatting.

```lua
Debug.dump(complex_table, "Configuration")
```

#### Debug.dumpCompact(value, label)
Compact one-line dump.

```lua
Debug.dumpCompact(data, "Data")
```

#### Debug.dumpVerbose(value, label)
Detailed verbose dump.

```lua
Debug.dumpVerbose(object, "Object")
```

#### Debug.dumpWithOptions(value, options, label)
Dump with custom options.

```lua
Debug.dumpWithOptions(data, {
    max_depth = 5,
    indent_size = 4,
    max_string_length = 100,
    show_types = true,
    show_addresses = false
}, "Custom")
```

### Performance

#### Debug.performanceReport()
Generates performance report.

```lua
local report = Debug.performanceReport()
```

#### Debug.memoryStats()
Gets memory statistics.

```lua
local stats = Debug.memoryStats()
print(stats.used_bytes)
print(stats.allocated_bytes)
```

#### Debug.jsonReport()
Generates JSON debug report.

```lua
local json = Debug.jsonReport()
```

#### Debug.flameGraph()
Generates flame graph data.

```lua
local flame_data = Debug.flameGraph()
```

#### Debug.memorySnapshot()
Takes a memory snapshot.

```lua
local snapshot = Debug.memorySnapshot()
print(snapshot.timestamp_secs)
print(snapshot.active_trackers)
```

#### Debug.recordEvent(timer_id, event_name, metadata)
Records a timing event.

```lua
Debug.recordEvent("timer-123", "checkpoint", {step = 5})
```

### Stack Traces

#### Debug.stackTrace(options)
Captures stack trace.

```lua
local trace = Debug.stackTrace({
    max_depth = 50,
    capture_locals = true,
    capture_upvalues = false,
    include_source = true
})
```

#### Debug.stackTraceJson(options)
Gets stack trace as JSON.

```lua
local json = Debug.stackTraceJson()
```

---

## JSON

The `JSON` global provides JSON utilities.

### JSON Operations

#### JSON.parse(string)
Parses JSON string to Lua value.

```lua
local data = JSON.parse('{"name": "Alice", "age": 30}')
print(data.name)  -- "Alice"
```

#### JSON.stringify(value)
Converts Lua value to JSON string.

```lua
local json = JSON.stringify({
    name = "Bob",
    items = {1, 2, 3},
    active = true
})
```

---

## Template

The `Template` global provides production-ready AI workflow templates combining agents, tools, RAG, and LocalLLM into turn-key solutions (Phase 12).

### Core Methods

#### Template.list([category])
Lists available templates, optionally filtered by category.

```lua
-- List all templates
local all_templates = Template.list()
for _, template in ipairs(all_templates) do
    print(template.name .. " (" .. template.category .. "): " .. template.description)
end

-- List by category
local research_templates = Template.list("research")
local chat_templates = Template.list("chat")
local analysis_templates = Template.list("analysis")
local codegen_templates = Template.list("codegen")
local document_templates = Template.list("document")
local workflow_templates = Template.list("workflow")
```

**Categories**:
- `"research"` - Research workflows (research-assistant, knowledge-management)
- `"chat"` - Conversational AI (interactive-chat)
- `"analysis"` - Data analysis (data-analysis)
- `"codegen"` - Code generation and review (code-generator, code-review)
- `"document"` - Document processing (document-processor, content-generation)
- `"workflow"` - Multi-step orchestration (workflow-orchestrator, file-classification)

#### Template.info(name, [show_schema])
Gets detailed information about a template.

```lua
-- Basic info
local info = Template.info("research-assistant")
print("Name:", info.metadata.name)
print("Description:", info.metadata.description)
print("Version:", info.metadata.version)
print("Category:", info.metadata.category)

-- With schema
local info_with_schema = Template.info("research-assistant", true)
if info_with_schema.schema then
    print("\nParameters:")
    for _, param in ipairs(info_with_schema.schema.parameters) do
        local required = param.required and "required" or "optional"
        print(string.format("  - %s (%s, %s)", param.name, param.param_type, required))
        if param.description then
            print("    " .. param.description)
        end
    end
end
```

#### Template.execute(name, params)
Executes a template with parameters (async operation).

```lua
-- Basic execution
local result = Template.execute("research-assistant", {
    topic = "Rust async programming",
    max_sources = 10
})

-- Check result
if result.success then
    print("Result:", result.result)

    -- Access artifacts
    for _, artifact in ipairs(result.artifacts) do
        print("Generated:", artifact.filename)
        print("Type:", artifact.artifact_type)
        print("Size:", #artifact.content, "bytes")
    end

    -- Access metrics
    if result.metrics then
        print("Duration:", result.metrics.duration_ms, "ms")
        print("Tokens:", result.metrics.tokens_used)
        print("Cost:", result.metrics.cost_usd, "USD")
    end
else
    print("Error:", result.error)
end
```

#### Template.search(query, [category])
Searches templates by keyword query.

```lua
-- Search all templates
local results = Template.search("research")
for _, template in ipairs(results) do
    print(template.name)
end

-- Search within category
local code_results = Template.search("review", "codegen")
local doc_results = Template.search("content", "document")
```

#### Template.schema(name)
Gets the parameter schema for a template.

```lua
local schema = Template.schema("code-generator")

-- Inspect parameters
for _, param in ipairs(schema.parameters) do
    print("\nParameter:", param.name)
    print("  Type:", param.param_type)
    print("  Required:", param.required)
    print("  Description:", param.description or "N/A")

    -- Constraints
    if param.constraints then
        if param.constraints.min_value then
            print("  Min value:", param.constraints.min_value)
        end
        if param.constraints.max_value then
            print("  Max value:", param.constraints.max_value)
        end
        if param.constraints.min_length then
            print("  Min length:", param.constraints.min_length)
        end
        if param.constraints.max_length then
            print("  Max length:", param.constraints.max_length)
        end
        if param.constraints.pattern then
            print("  Pattern:", param.constraints.pattern)
        end
        if param.constraints.allowed_values then
            print("  Allowed values:", table.concat(param.constraints.allowed_values, ", "))
        end
    end
end
```

#### Template.estimate_cost(name, params)
Estimates execution cost (tokens, USD, duration) before running (async, bonus method).

```lua
local estimate = Template.estimate_cost("research-assistant", {
    topic = "Climate change impacts",
    max_sources = 20
})

if estimate then
    if estimate.estimated_tokens then
        print("Estimated tokens:", estimate.estimated_tokens)
    end
    if estimate.estimated_cost_usd then
        print("Estimated cost:", string.format("$%.4f", estimate.estimated_cost_usd))
    end
    if estimate.estimated_duration_ms then
        print("Estimated duration:", estimate.estimated_duration_ms / 1000, "seconds")
    end
    print("Confidence:", estimate.confidence)  -- "low", "medium", "high"
else
    print("Cost estimation not available for this template")
end
```

### Built-in Templates (10 Total)

#### 1. research-assistant (Research)
**Status**: ✅ Production Ready

Multi-phase research workflow with web search, analysis, synthesis, and validation.

```lua
local result = Template.execute("research-assistant", {
    topic = "Rust async/await patterns",       -- required: Research topic
    max_sources = 10,                           -- optional: Max web sources (default: 10)
    min_quality_score = 0.7,                    -- optional: Quality threshold (default: 0.7)
    research_depth = "comprehensive",           -- optional: "basic", "standard", "comprehensive"
    enable_validation = true,                   -- optional: Enable fact-checking (default: true)
    output_format = "markdown",                 -- optional: "markdown", "json", "html"
    model = "openai/gpt-4o",                    -- optional: LLM model (default: from config)
    save_sources = true                         -- optional: Save source list (default: true)
})

-- Output: Research report (Markdown) + source list (JSON)
print(result.result)  -- Full research report
```

**4-Phase Pipeline**:
1. **Discovery** - Web search across multiple sources
2. **Analysis** - Content extraction and quality scoring
3. **Synthesis** - Intelligent synthesis with citations
4. **Validation** - Fact-checking and verification

**Performance**: ~45s for 10 sources, ~2,500 tokens, ~$0.05 cost

#### 2. interactive-chat (Chat)
**Status**: ✅ Production Ready

Session-based conversational AI with context management and REPL interface.

```lua
local result = Template.execute("interactive-chat", {
    message = "Explain dependency injection",  -- required: User message or "repl" for REPL mode
    session_name = "my-chat",                  -- optional: Session name for persistence
    system_prompt = "You are a helpful coding assistant",  -- optional: System prompt
    model = "openai/gpt-4o",                   -- optional: LLM model
    enable_memory = true,                      -- optional: Use conversation memory (default: true)
    max_history = 10,                          -- optional: Max conversation turns (default: 10)
    temperature = 0.7                          -- optional: Response randomness (default: 0.7)
})

-- REPL mode (Phase 12.9)
local chat = Template.execute("interactive-chat", {
    message = "repl",  -- Special value for REPL mode
    session_name = "coding-help",
    model = "ollama/llama3.2:3b"
})

-- REPL commands:
-- .system <text>    - Change system prompt
-- .model <name>     - Switch LLM model
-- .tools <list>     - Enable tool access (comma-separated)
-- .context <data>   - Add context data
-- .clearchat        - Clear conversation history
-- .info             - Show session info
-- Ctrl-C            - Exit REPL
```

**Features**: Multi-turn context, session persistence, streaming responses, REPL mode

#### 3. data-analysis (Analysis)
**Status**: ✅ Production Ready

Automated data analysis with statistics, visualization, and LLM insights.

```lua
local result = Template.execute("data-analysis", {
    data_file = "/path/to/data.csv",           -- required: Data file path
    analysis_type = "descriptive",             -- required: "descriptive", "correlation", "regression", "timeseries"
    chart_type = "bar",                        -- optional: "bar", "line", "scatter", "heatmap", "none"
    model = "openai/gpt-4o",                   -- optional: LLM model for insights
    output_format = "markdown",                -- optional: "markdown", "json", "html"
    include_visualizations = true              -- optional: Generate charts (default: true)
})

-- Output: Analysis report + generated charts
```

**Supported Formats**: CSV, Excel (.xlsx), JSON, Parquet
**Analysis Types**:
- `descriptive` - Summary statistics, distributions
- `correlation` - Correlation matrices, relationships
- `regression` - Linear/polynomial regression
- `timeseries` - Trend analysis, forecasting

#### 4. code-generator (CodeGen)
**Status**: ✅ Production Ready

Multi-language code generation with tests, documentation, and quality validation.

```lua
local result = Template.execute("code-generator", {
    description = "A function to calculate Fibonacci numbers",  -- required: Code description
    language = "rust",                         -- required: "rust", "python", "javascript", "typescript", "go", etc.
    model = "openai/gpt-4o",                   -- optional: LLM model
    generate_tests = true,                     -- optional: Generate unit tests (default: true)
    generate_docs = true,                      -- optional: Generate documentation (default: true)
    code_style = "idiomatic",                  -- optional: "idiomatic", "simple", "production"
    include_examples = true                    -- optional: Include usage examples (default: true)
})

-- Output: Generated code + tests + documentation
print(result.artifacts[1].content)  -- Main code file
print(result.artifacts[2].content)  -- Test file
print(result.artifacts[3].content)  -- Documentation
```

**3-Agent Pipeline**:
1. **Code Generator** - Generates implementation
2. **Test Generator** - Creates comprehensive tests
3. **Doc Generator** - Writes API documentation

**Supported Languages**: 10+ (Rust, Python, JavaScript, TypeScript, Go, Java, C++, C#, Ruby, PHP)

#### 5. document-processor (Document)
**Status**: ✅ Production Ready

Document transformation with extraction, translation, and format conversion.

```lua
local result = Template.execute("document-processor", {
    document_paths = {"/path/to/doc1.pdf", "/path/to/doc2.txt"},  -- required: Document paths (array)
    transformation_type = "summarize",         -- required: "extract", "summarize", "translate", "transform"
    target_language = "es",                    -- optional: For translation (ISO 639-1)
    model = "openai/gpt-4o",                   -- optional: LLM model
    output_format = "markdown",                -- optional: "markdown", "json", "text", "html"
    ocr_enabled = true                         -- optional: Enable OCR for images (default: true)
})

-- Output: Transformed documents
```

**Transformation Types**:
- `extract` - Extract key information, entities
- `summarize` - Generate concise summaries
- `translate` - Translate to target language
- `transform` - Custom transformation with instructions

**Supported Formats**: PDF, TXT, DOCX, HTML, Markdown, Images (with OCR)

#### 6. workflow-orchestrator (Workflow)
**Status**: ✅ Production Ready

Custom multi-step workflows with parallel/sequential/conditional/loop execution.

```lua
local result = Template.execute("workflow-orchestrator", {
    workflow_config = {                        -- required: Workflow configuration
        name = "data-pipeline",
        steps = {
            {
                name = "fetch",
                step_type = "tool",
                description = "Fetch data from API",
                tool_name = "http-request",
                params = {method = "GET", url = "https://api.example.com/data"}
            },
            {
                name = "analyze",
                step_type = "agent",
                description = "Analyze fetched data",
                agent_config = {model = "openai/gpt-4o", system_prompt = "Analyze this data"}
            },
            {
                name = "save",
                step_type = "tool",
                description = "Save results",
                tool_name = "file-operations",
                params = {operation = "write", path = "./output.json"}
            }
        }
    },
    execution_mode = "sequential",             -- required: "sequential", "parallel", "conditional", "loop"
    model = "openai/gpt-4o",                   -- optional: Default LLM model for agents
    collect_intermediate_results = true        -- optional: Save intermediate outputs (default: false)
})

-- Access step outputs
if result.metadata and result.metadata.extra and result.metadata.extra.agent_outputs then
    for step_name, output in pairs(result.metadata.extra.agent_outputs) do
        print("Step:", step_name)
        print("Output:", output)
    end
end
```

**Execution Modes**:
- `sequential` - Steps run one after another
- `parallel` - Steps run concurrently
- `conditional` - If/else branching based on conditions
- `loop` - Iterative execution with termination condition

**Step Types**: `tool`, `agent`, `workflow` (nested)

#### 7. code-review (CodeGen)
**Status**: ✅ Production Ready (Phase 12.10)

Multi-aspect code analysis with configurable review aspects and quality scoring.

```lua
local result = Template.execute("code-review", {
    code_path = "src/main.rs",                 -- required: File or directory path
    aspects = {"security", "quality", "performance"},  -- required: Review aspects (array)
    language = "rust",                         -- optional: Programming language (auto-detect if omitted)
    model = "openai/gpt-4o",                   -- optional: LLM model
    generate_fixes = true,                     -- optional: Generate fix suggestions (default: false)
    quality_threshold = 7.0                    -- optional: Minimum quality score (0-10, default: 7.0)
})

-- Output: Multi-aspect analysis + quality scores + fix suggestions
```

**Available Aspects** (7 total):
- `security` - Security vulnerabilities, injection risks, auth issues
- `quality` - Code quality, maintainability, readability
- `performance` - Performance bottlenecks, algorithmic complexity
- `practices` - Best practices, language idioms, patterns
- `dependencies` - Dependency management, versioning, licenses
- `architecture` - Design patterns, modularity, coupling
- `documentation` - Code comments, API docs, examples

**Output**: Per-aspect analysis with findings, severity, scores, and optional fix suggestions

#### 8. content-generation (Document)
**Status**: ✅ Production Ready (Phase 12.11)

Quality-driven content creation with iterative refinement and multi-format output.

```lua
local result = Template.execute("content-generation", {
    topic = "Product launch announcement",     -- required: Content topic
    content_type = "marketing",                -- required: "blog", "documentation", "marketing", "technical", "creative"
    target_audience = "developers",            -- optional: Target audience description
    tone = "professional",                     -- optional: "professional", "casual", "formal", "friendly"
    word_count = 800,                          -- optional: Target word count (default: 500)
    model = "openai/gpt-4o",                   -- optional: LLM model
    quality_threshold = 8.0,                   -- optional: Quality score threshold (0-10, default: 7.5)
    max_iterations = 3,                        -- optional: Max refinement iterations (default: 3)
    style_guide = "Follow AP style"            -- optional: Custom style guidelines
})

-- Output: Final content + quality scores + revision history
```

**Content Types** (5 presets):
- `blog` - Blog posts, articles
- `documentation` - Technical documentation, guides
- `marketing` - Marketing copy, announcements
- `technical` - Technical writing, specifications
- `creative` - Creative writing, stories

**4-Stage Pipeline**:
1. **Generate** - Initial content creation
2. **Evaluate** - Quality scoring (clarity, relevance, grammar, style)
3. **Edit** - Iterative refinement if below threshold
4. **Finalize** - Final content with metadata

#### 9. file-classification (Workflow)
**Status**: ✅ Production Ready (Phase 12.12)

Bulk file organization with customizable categories and dry-run mode.

```lua
local result = Template.execute("file-classification", {
    source_path = "/Users/me/Documents",       -- required: Directory to scan
    classification_strategy = "hybrid",        -- required: "rule-based", "llm", "hybrid"
    categories = {"work", "personal", "archive"},  -- optional: Custom categories (auto-detect if omitted)
    dry_run = true,                            -- optional: Preview without moving (default: false)
    model = "openai/gpt-4o",                   -- optional: LLM model for classification
    create_destination_dirs = true,            -- optional: Auto-create category dirs (default: true)
    file_extensions = {".pdf", ".docx", ".txt"}  -- optional: Filter by extensions
})

-- Output: Classification plan + moved files summary
```

**Classification Strategies**:
- `rule-based` - File extension + name pattern rules (fast, no LLM)
- `llm` - LLM content analysis (accurate, slower)
- `hybrid` - Rules first, LLM for ambiguous files (balanced)

**4 Category Presets**:
- `documents` - Work documents, personal files, finance, media
- `code` - Source code, configs, docs, tests
- `media` - Photos, videos, audio, designs
- `general` - General-purpose organization

**Scan-Classify-Act Pattern**: Scan files → Classify into categories → Execute actions (move/copy/report)

#### 10. knowledge-management (Research)
**Status**: ✅ Production Ready (Phase 12.13)

RAG-based knowledge base with CRUD operations and semantic search.

```lua
-- Ingest documents
local ingest_result = Template.execute("knowledge-management", {
    operation = "ingest",                      -- required: "ingest", "query", "update", "delete"
    collection = "ai-research",                -- required: Collection name
    document_paths = {"/path/to/paper1.pdf", "/path/to/paper2.pdf"},  -- for ingest
    model = "openai/gpt-4o",                   -- optional: LLM model
    chunk_size = 512,                          -- optional: Chunking size (default: 512)
    chunk_overlap = 50                         -- optional: Chunk overlap (default: 50)
})

-- Query with citations
local query_result = Template.execute("knowledge-management", {
    operation = "query",
    collection = "ai-research",
    query = "What are the benefits of async Rust?",
    top_k = 5,                                 -- optional: Number of results (default: 5)
    include_citations = true,                  -- optional: Include source citations (default: true)
    model = "openai/gpt-4o"
})

-- Update documents
local update_result = Template.execute("knowledge-management", {
    operation = "update",
    collection = "ai-research",
    document_id = "doc-123",                   -- required for update
    updated_content = "Updated content here"
})

-- Delete documents
local delete_result = Template.execute("knowledge-management", {
    operation = "delete",
    collection = "ai-research",
    document_ids = {"doc-123", "doc-456"}      -- required for delete (array)
})
```

**CRUD Operations**:
- `ingest` - Add new documents to collection
- `query` - Semantic search with LLM-generated answers
- `update` - Update existing documents
- `delete` - Remove documents from collection

**Features**: Multi-collection support, semantic chunking, citation tracking, metadata search

### Template Result Structure

All templates return a standardized `TemplateOutput` structure:

```lua
{
    success = true,                            -- Execution success status
    result = "Main result text/data",          -- Primary output
    artifacts = {                              -- Generated files/data
        {
            filename = "research_report.md",
            artifact_type = "document",
            content = "...",
            metadata = {source = "research", format = "markdown"}
        }
    },
    metadata = {                               -- Execution metadata
        template_id = "research-assistant",
        template_version = "1.0.0",
        execution_id = "uuid...",
        started_at = 1234567890,
        completed_at = 1234567900,
        extra = {                              -- Template-specific data
            agent_outputs = {},                -- Workflow agent outputs
            phase_results = {}                 -- Multi-phase results
        }
    },
    metrics = {                                -- Performance metrics
        duration_ms = 45123,
        tokens_used = 2500,
        cost_usd = 0.05,
        api_calls = 3
    },
    error = nil                                -- Error message if failed
}
```

### Error Handling

Templates use the standard error pattern:

```lua
local success, result = pcall(function()
    return Template.execute("research-assistant", {
        topic = "Invalid topic"
    })
end)

if not success then
    print("Execution failed:", result)
elseif result.error then
    print("Template error:", result.error)
elseif not result.success then
    print("Template execution was not successful")
else
    print("Success:", result.result)
end
```

### Common Template Patterns

#### Pattern 1: Template Discovery

```lua
-- Find templates by capability
local search_templates = Template.search("search")
local code_templates = Template.search("code", "codegen")

-- List by category
for _, category in ipairs({"research", "chat", "analysis", "codegen", "document", "workflow"}) do
    local templates = Template.list(category)
    print(category .. ": " .. #templates .. " templates")
end
```

#### Pattern 2: Schema Introspection

```lua
-- Check required parameters before execution
local schema = Template.schema("research-assistant")
local required_params = {}

for _, param in ipairs(schema.parameters) do
    if param.required then
        table.insert(required_params, param.name)
    end
end

print("Required parameters:", table.concat(required_params, ", "))
```

#### Pattern 3: Cost-Aware Execution

```lua
-- Estimate before executing
local estimate = Template.estimate_cost("research-assistant", {
    topic = "Large topic with many sources",
    max_sources = 50
})

if estimate and estimate.estimated_cost_usd and estimate.estimated_cost_usd > 1.0 then
    print("Warning: High cost estimated ($" .. estimate.estimated_cost_usd .. ")")
    print("Continue? (y/n)")
    local answer = io.read()
    if answer ~= "y" then
        return
    end
end

local result = Template.execute("research-assistant", {...})
```

#### Pattern 4: Artifact Processing

```lua
local result = Template.execute("code-generator", {
    description = "REST API handler",
    language = "rust"
})

-- Save artifacts to files
for _, artifact in ipairs(result.artifacts) do
    local file = io.open("output/" .. artifact.filename, "w")
    if file then
        file:write(artifact.content)
        file:close()
        print("Saved:", artifact.filename)
    end
end
```

#### Pattern 5: Multi-Template Workflows

```lua
-- Research → Analyze → Generate
local research = Template.execute("research-assistant", {
    topic = "Modern web frameworks"
})

local analysis = Template.execute("data-analysis", {
    data_file = research.artifacts[2].filename,  -- Use research output
    analysis_type = "descriptive"
})

local content = Template.execute("content-generation", {
    topic = "Blog post about " .. research.result,
    content_type = "blog",
    tone = "professional"
})
```

---

## ARGS

The `ARGS` global provides command-line argument access.

### Accessing Arguments

#### Positional Arguments
Access by numeric index.

```lua
local script_name = ARGS[0]  -- Script name
local first_arg = ARGS[1]    -- First positional argument
local second_arg = ARGS[2]   -- Second positional argument
```

#### Named Arguments
Access by name.

```lua
local input_file = ARGS.input or "default.txt"
local output_file = ARGS.output or "result.txt"
local debug_mode = ARGS.debug == "true"
```

#### Traditional arg Table
For Lua compatibility.

```lua
local script = arg[0]
local first = arg[1]
```

---

## Streaming

The `Streaming` global provides streaming and coroutine utilities.

### Stream Creation

#### Streaming.create(generator_function)
Creates a stream from a generator.

```lua
local stream = Streaming.create(function()
    for i = 1, 10 do
        coroutine.yield(i * 2)
    end
end)
```

### Stream Methods

#### stream:next()
Gets the next value from the stream.

```lua
local value = stream:next()
```

#### stream:isDone()
Checks if stream is exhausted.

```lua
if stream:isDone() then
    -- No more values
end
```

#### stream:collect()
Collects all remaining values.

```lua
local all_values = stream:collect()
```

### Streaming Utilities

#### Streaming.yield(value)
Yields a value in a coroutine (placeholder).

```lua
Streaming.yield(computed_value)
```

---

## Common Patterns

### Error Handling

Most operations return nil or false on failure:

```lua
local result = Tool.execute("calculator", {operation = "divide", a = 10, b = 0})
if not result then
    print("Operation failed")
elseif result.error then
    print("Error:", result.error)
else
    print("Result:", result.value)
end
```

### Async Operations

Many operations support both sync and async variants:

```lua
-- Synchronous
local result = agent:execute("Hello")

-- Asynchronous with callback
agent:execute_async("Hello", function(result, error)
    if error then
        print("Error:", error)
    else
        print("Result:", result)
    end
end)
```

### Builder Patterns

Many objects support fluent builder interfaces:

```lua
local agent = Agent.builder()
    :name("assistant")
    :model("gpt-4")
    :temperature(0.7)
    :build()

local workflow = Workflow.sequential()
    :step("fetch", {tool = "web-fetch"})
    :step("process", {agent = "processor"})
    :build()
```

### Configuration Tables

Most methods accept configuration tables:

```lua
local result = Tool.execute("web-search", {
    query = "LLMSpell"
}, {
    timeout = 5000,
    retry = 3,
    cache = true
})
```

### Template Patterns

Templates provide turn-key workflows:

```lua
-- Discovery and introspection
local templates = Template.list("research")
local schema = Template.schema("research-assistant")

-- Cost-aware execution
local estimate = Template.estimate_cost("research-assistant", {topic = "AI", max_sources = 20})
if estimate and estimate.estimated_cost_usd < 0.10 then
    local result = Template.execute("research-assistant", {topic = "AI", max_sources = 20})
end

-- Multi-template workflows
local research = Template.execute("research-assistant", {topic = "Rust"})
local content = Template.execute("content-generation", {
    topic = research.result,
    content_type = "blog"
})
```

## Best Practices

1. **Always check for nil/false returns** - Operations may fail
2. **Use scoped state for isolation** - Prefix keys with scope
3. **Clean up resources** - Unsubscribe from events, unregister hooks
4. **Handle streaming data incrementally** - Don't collect all at once
5. **Use appropriate debug levels** - trace < debug < info < warn < error
6. **Validate inputs** - Check types and ranges before operations
7. **Use builders for complex objects** - Cleaner than configuration tables
8. **Batch operations when possible** - Tool.batch() for multiple operations
9. **Set timeouts for external calls** - Prevent hanging on network/LLM calls
10. **Use session artifacts for persistence** - Better than global state for user data
11. **Inspect template schemas before execution** - Template.schema() shows required parameters and constraints
12. **Estimate costs for expensive templates** - Template.estimate_cost() helps budget LLM usage
13. **Use templates for common workflows** - 10 built-in templates cover research, chat, analysis, code generation, document processing, and orchestration
14. **Handle template artifacts properly** - Save artifacts to files, process results incrementally
15. **Chain templates for complex tasks** - research-assistant → data-analysis → content-generation for end-to-end workflows