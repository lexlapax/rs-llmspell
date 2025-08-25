# Workflow Module

The Workflow module provides orchestration capabilities for complex multi-step processes.

## Constructor

### Workflow.new(name)
Creates a new workflow instance.

**Parameters:**
- `name` (string): Workflow name

**Returns:** `Workflow` - New workflow instance

**Example:**
```lua
local workflow = Workflow.new("data-pipeline")
```

## Workflow Building

### workflow:add_step(name, config)
Adds a step to the workflow.

**Parameters:**
- `name` (string): Step name
- `config` (table): Step configuration
  - `type` (string): "agent", "tool", "parallel", "conditional"
  - `agent` (Agent, optional): Agent for agent steps
  - `tool` (string, optional): Tool name for tool steps
  - `params` (table, optional): Step parameters
  - `condition` (function, optional): Condition for conditional steps

**Returns:** `Workflow` - Self for chaining

**Example:**
```lua
workflow:add_step("fetch", {
    type = "tool",
    tool = "web-fetch",
    params = {url = "https://api.example.com/data"}
})
:add_step("process", {
    type = "agent",
    agent = processing_agent
})
```

### workflow:add_parallel(name, steps)
Adds parallel execution steps.

**Parameters:**
- `name` (string): Parallel block name
- `steps` (table): Array of step configurations

**Returns:** `Workflow` - Self for chaining

**Example:**
```lua
workflow:add_parallel("fetch_all", {
    {type = "tool", tool = "web-fetch", params = {url = "api1.com"}},
    {type = "tool", tool = "web-fetch", params = {url = "api2.com"}},
    {type = "tool", tool = "web-fetch", params = {url = "api3.com"}}
})
```

### workflow:add_conditional(name, condition, if_true, if_false)
Adds conditional branching.

**Parameters:**
- `name` (string): Condition name
- `condition` (function): Condition function
- `if_true` (table): Steps if condition is true
- `if_false` (table, optional): Steps if condition is false

**Returns:** `Workflow` - Self for chaining

**Example:**
```lua
workflow:add_conditional("check_data", 
    function(context) 
        return context.data_quality > 0.8 
    end,
    {type = "tool", tool = "data-processor"},
    {type = "agent", agent = quality_agent}
)
```

### workflow:add_loop(name, condition, steps)
Adds a loop structure.

**Parameters:**
- `name` (string): Loop name
- `condition` (function): Loop condition
- `steps` (table): Steps to execute in loop

**Returns:** `Workflow` - Self for chaining

**Example:**
```lua
workflow:add_loop("retry_fetch",
    function(context)
        return context.attempts < 3 and not context.success
    end,
    {
        {type = "tool", tool = "http-request"},
        {type = "tool", tool = "delay", params = {seconds = 2}}
    }
)
```

## Workflow Execution

### workflow:run(initial_context)
Executes the workflow.

**Parameters:**
- `initial_context` (table, optional): Initial context data

**Returns:** `table` - Workflow result
- `success` (boolean): Whether workflow succeeded
- `result` (any): Final result data
- `steps_executed` (number): Number of steps executed
- `duration` (number): Total duration in ms
- `context` (table): Final context state

**Example:**
```lua
local result = workflow:run({
    input_data = "initial value",
    config = {timeout = 30000}
})

if result.success then
    print("Result:", result.result)
end
```

### workflow:run_async(initial_context)
Executes the workflow asynchronously.

**Parameters:** Same as `run()`

**Returns:** `Promise` - Promise that resolves to result

**Example:**
```lua
workflow:run_async({data = "input"})
    :then(function(result)
        print("Workflow completed:", result.success)
    end)
    :catch(function(err)
        print("Workflow failed:", err)
    end)
```

### workflow:validate()
Validates the workflow configuration.

**Returns:** `boolean, string` - Valid status and error message if invalid

**Example:**
```lua
local valid, err = workflow:validate()
if not valid then
    print("Invalid workflow:", err)
end
```

## Workflow State

### workflow:get_state()
Gets the current workflow state.

**Returns:** `table` - Current state
- `status` (string): "idle", "running", "completed", "failed"
- `current_step` (string): Current step name
- `context` (table): Current context

### workflow:pause()
Pauses workflow execution.

**Returns:** `boolean` - Success status

### workflow:resume()
Resumes paused workflow.

**Returns:** `boolean` - Success status

### workflow:cancel()
Cancels running workflow.

**Returns:** `boolean` - Success status

## Context Management

### workflow:set_context(key, value)
Sets a context value.

**Parameters:**
- `key` (string): Context key
- `value` (any): Context value

**Returns:** `nil`

### workflow:get_context(key)
Gets a context value.

**Parameters:**
- `key` (string): Context key

**Returns:** `any` - Context value

### workflow:merge_context(data)
Merges data into context.

**Parameters:**
- `data` (table): Data to merge

**Returns:** `nil`

## Step Handlers

### workflow:on_step_start(handler)
Sets handler for step start events.

**Parameters:**
- `handler` (function): Handler function(step_name, context)

**Returns:** `Workflow` - Self for chaining

### workflow:on_step_complete(handler)
Sets handler for step completion.

**Parameters:**
- `handler` (function): Handler function(step_name, result, context)

**Returns:** `Workflow` - Self for chaining

### workflow:on_error(handler)
Sets error handler.

**Parameters:**
- `handler` (function): Handler function(error, step_name, context)

**Returns:** `Workflow` - Self for chaining

**Example:**
```lua
workflow:on_step_start(function(step_name, context)
    print("Starting step:", step_name)
end)
:on_step_complete(function(step_name, result, context)
    print("Completed step:", step_name, "Result:", result)
end)
:on_error(function(error, step_name, context)
    print("Error in step:", step_name, "Error:", error)
    -- Return true to continue, false to stop
    return false
end)
```

## Workflow Templates

### Workflow.register_template(name, template)
Registers a workflow template.

**Parameters:**
- `name` (string): Template name
- `template` (table): Template configuration

**Returns:** `boolean` - Success status

### Workflow.from_template(name)
Creates workflow from template.

**Parameters:**
- `name` (string): Template name

**Returns:** `Workflow` - New workflow instance

**Example:**
```lua
-- Register template
Workflow.register_template("data_etl", {
    steps = {
        {name = "extract", type = "tool", tool = "database-connector"},
        {name = "transform", type = "agent", agent_template = "transformer"},
        {name = "load", type = "tool", tool = "database-connector"}
    }
})

-- Use template
local etl = Workflow.from_template("data_etl")
```

## Workflow Persistence

### workflow:save()
Saves workflow configuration and state.

**Returns:** `string` - Workflow ID

### Workflow.load(id)
Loads a saved workflow.

**Parameters:**
- `id` (string): Workflow ID

**Returns:** `Workflow` - Loaded workflow

### workflow:export()
Exports workflow as JSON.

**Returns:** `string` - JSON representation

### Workflow.import(json)
Imports workflow from JSON.

**Parameters:**
- `json` (string): JSON representation

**Returns:** `Workflow` - Imported workflow

## Advanced Features

### workflow:add_retry(step_name, config)
Adds retry logic to a step.

**Parameters:**
- `step_name` (string): Step to retry
- `config` (table): Retry configuration
  - `max_attempts` (number): Maximum retry attempts
  - `delay` (number): Delay between retries (ms)
  - `backoff` (string): "linear", "exponential"

**Returns:** `Workflow` - Self for chaining

### workflow:add_timeout(step_name, timeout_ms)
Adds timeout to a step.

**Parameters:**
- `step_name` (string): Step name
- `timeout_ms` (number): Timeout in milliseconds

**Returns:** `Workflow` - Self for chaining

### workflow:add_cache(step_name, ttl)
Adds caching to a step.

**Parameters:**
- `step_name` (string): Step name
- `ttl` (number): Cache TTL in seconds

**Returns:** `Workflow` - Self for chaining

## Metrics

### workflow:get_metrics()
Gets workflow execution metrics.

**Returns:** `table` - Metrics data
- `total_runs` (number): Total executions
- `successful_runs` (number): Successful executions
- `failed_runs` (number): Failed executions
- `average_duration` (number): Average duration (ms)
- `step_metrics` (table): Per-step metrics

## See Also
- [Agent Module](./agent.md) - Agent integration
- [Tool Module](./tool.md) - Tool integration
- [State Module](./state.md) - State persistence
- [Event Module](./event.md) - Workflow events