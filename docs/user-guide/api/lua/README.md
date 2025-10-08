# LLMSpell Lua API Documentation

This document provides comprehensive documentation of all Lua globals available in LLMSpell scripts. Each global object provides specific functionality for building LLM-powered applications.

## Table of Contents

1. [Agent](#agent) - LLM agent creation and management
2. [Tool](#tool) - Tool invocation and discovery
3. [Workflow](#workflow) - Workflow orchestration
4. [Session](#session) - Session management and persistence
5. [State](#state) - Global state management
6. [Event](#event) - Event publishing and subscription
7. [Hook](#hook) - Hook registration and management
8. [RAG](#rag) - Retrieval-Augmented Generation with vector storage
9. [LocalLLM](#localllm) - Local model management (Ollama, Candle)
10. [Config](#config) - Configuration access and management
11. [Provider](#provider) - LLM provider information
12. [Artifact](#artifact) - Artifact storage and retrieval
13. [Replay](#replay) - Hook replay and testing
14. [Debug](#debug) - Debugging and profiling utilities
15. [JSON](#json) - JSON parsing and serialization
16. [ARGS](#args) - Command-line argument access
17. [Streaming](#streaming) - Streaming and coroutine utilities

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
Creates an agent directly from configuration.

```lua
local agent = Agent.create({
    name = "my-agent",
    type = "llm",
    model = "anthropic/claude-3",
    temperature = 0.5
})
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

#### Agent.discover(options)
Discovers agents based on criteria.

```lua
local agents = Agent.discover({
    type = "llm",
    capabilities = {"code-generation"},
    min_context = 4000
})
```

#### Agent.discover_by_capability(capability)
Finds agents with a specific capability.

```lua
local coders = Agent.discover_by_capability("code-generation")
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

#### agent:execute(prompt, options)
Executes the agent with a prompt.

```lua
local response = agent:execute("Write a poem", {
    temperature = 0.9,
    max_tokens = 500
})
```

#### agent:execute_streaming(prompt, options, callback)
Executes with streaming response.

```lua
agent:execute_streaming("Tell a story", {}, function(chunk)
    print(chunk)
end)
```

#### agent:reset()
Resets the agent's state.

```lua
agent:reset()
```

#### agent:get_history()
Gets conversation history.

```lua
local history = agent:get_history()
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

#### Tool.execute(name, params, options)
Executes a tool with additional options.

```lua
local result = Tool.execute("web-search", {
    query = "LLMSpell documentation"
}, {
    timeout = 5000,
    retry = 3
})
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

### Tool Registration

#### Tool.register(name, handler)
Registers a new tool.

```lua
Tool.register("custom-tool", function(params)
    return {result = params.input * 2}
end)
```

#### Tool.unregister(name)
Unregisters a tool.

```lua
Tool.unregister("custom-tool")
```

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
    :id("session-456")
    :user("bob")
    :ttl(3600)
    :metadata({source = "web"})
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

#### session:get(key)
Gets a session value.

```lua
local value = session:get("user_preference")
```

#### session:set(key, value)
Sets a session value.

```lua
session:set("last_query", "weather")
```

#### session:delete(key)
Deletes a session value.

```lua
session:delete("temp_data")
```

#### session:store_artifact(name, data)
Stores an artifact in the session.

```lua
session:store_artifact("query_results", results)
```

---

## State

The `State` global provides persistent state management.

### Basic Operations

#### State.get(key)
Gets a state value.

```lua
local value = State.get("app_config")
```

#### State.set(key, value)
Sets a state value.

```lua
State.set("app_config", {
    theme = "dark",
    language = "en"
})
```

#### State.delete(key)
Deletes a state entry.

```lua
State.delete("temp_state")
```

#### State.exists(key)
Checks if a key exists.

```lua
if State.exists("user_settings") then
    -- Key exists
end
```

#### State.clear()
Clears all state.

```lua
State.clear()
```

#### State.list()
Lists all state keys.

```lua
local keys = State.list()
```

### Scoped State

#### State.get_scoped(scope, key)
Gets value from a scope.

```lua
local value = State.get_scoped("user:123", "preferences")
```

#### State.set_scoped(scope, key, value)
Sets value in a scope.

```lua
State.set_scoped("tenant:abc", "settings", config)
```

#### State.delete_scoped(scope, key)
Deletes from a scope.

```lua
State.delete_scoped("session:456", "temp")
```

#### State.clear_scope(scope)
Clears an entire scope.

```lua
State.clear_scope("user:123")
```

#### State.list_scoped(scope)
Lists keys in a scope.

```lua
local keys = State.list_scoped("tenant:abc")
```

### Atomic Operations

#### State.increment(key, amount)
Atomically increments a numeric value.

```lua
local new_value = State.increment("counter", 1)
```

#### State.append(key, value)
Appends to a list value.

```lua
State.append("event_log", {timestamp = os.time(), event = "login"})
```

#### State.compare_and_swap(key, old_value, new_value)
Atomic compare and swap.

```lua
local success = State.compare_and_swap("status", "pending", "active")
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