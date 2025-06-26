# Scripting Interface Implications Analysis

## Overview

Analysis of how the new BaseAgent/Agent/Tool/Workflow hierarchy affects the scripting interface, focusing on bridge layer implications, script API design, and cross-engine compatibility.

## Table of Contents

1. [Current vs New Architecture Impact](#current-vs-new-architecture-impact)
2. [Bridge Layer Redesign](#bridge-layer-redesign)
3. [Script API Changes](#script-api-changes)
4. [Type System Implications](#type-system-implications)
5. [Cross-Engine Compatibility](#cross-engine-compatibility)
6. [Performance Considerations](#performance-considerations)
7. [Error Handling Updates](#error-handling-updates)
8. [Migration Strategy](#migration-strategy)

## Current vs New Architecture Impact

### Current Script Interface

**Existing API Structure:**
```lua
-- Current: Simple agent creation
local agent = llm.agent({
    model = "claude-3-opus",
    system = "You are helpful",
    tools = {"web_search", "file_write"}
})

-- Current: Direct execution
local response = agent:run("Do something")
```

**Limitations:**
- No separation between base capabilities and LLM-specific features
- Limited composition patterns
- No built-in hook support
- Minimal state management
- Tool execution not observable

### New Architecture Requirements

**New API Structure:**
```lua
-- New: BaseAgent with explicit capabilities
local base_agent = agent.base({
    id = "research_assistant",
    name = "Research Assistant"
})

-- Add tools to base agent
base_agent:add_tool("web_search")
base_agent:add_tool("file_write")

-- Add hooks for observability
base_agent:add_hook("logging", {
    level = "info",
    points = {"pre_tool", "post_tool"}
})

-- Create LLM agent from base
local llm_agent = agent.llm(base_agent, {
    model = "claude-3-opus",
    system = "You are a research assistant",
    temperature = 0.7
})

-- Execute with state management
local state = state.create({
    variables = { topic = "quantum computing" }
})

local response = llm_agent:run_with_state("Research the topic", state)
```

**Key Differences:**
1. **Explicit Base Layer**: BaseAgent creation is exposed to scripts
2. **Composition Pattern**: Tools and hooks are added incrementally  
3. **State Integration**: State objects are first-class citizens
4. **Observability**: Hooks are scriptable and configurable
5. **Tool-Agent Composition**: Agents can be wrapped as tools

## Bridge Layer Redesign

### Current Bridge Structure

```rust
// Current: Single bridge per functionality area
pub trait Bridge {
    fn methods(&self) -> Vec<MethodInfo>;
    async fn execute_method(&self, name: &str, args: []ScriptValue) -> Result<ScriptValue>;
}

// Simple bridge implementations
struct AgentBridge { /* basic agent operations */ }
struct ToolBridge { /* tool execution */ }
```

### New Hierarchical Bridge Structure

```rust
// New: Hierarchical bridge system reflecting architecture layers
pub trait BaseBridge: Bridge {
    // BaseAgent functionality
    fn create_base_agent(&self, config: BaseAgentConfig) -> Result<ScriptValue>;
    fn add_tool_to_agent(&self, agent_id: &str, tool_name: &str) -> Result<()>;
    fn add_hook_to_agent(&self, agent_id: &str, hook_config: HookConfig) -> Result<()>;
    fn get_agent_state(&self, agent_id: &str) -> Result<ScriptValue>;
    fn set_agent_state(&self, agent_id: &str, state: ScriptValue) -> Result<()>;
}

pub trait AgentBridge: BaseBridge {
    // LLM Agent functionality
    fn create_llm_agent(&self, base_agent_id: &str, config: LlmConfig) -> Result<ScriptValue>;
    fn run_agent(&self, agent_id: &str, input: &str) -> Result<ScriptValue>;
    fn run_agent_with_state(&self, agent_id: &str, input: &str, state_id: &str) -> Result<ScriptValue>;
    fn stream_agent(&self, agent_id: &str, input: &str) -> Result<StreamHandle>;
}

pub trait ToolBridge: Bridge {
    // Tool management
    fn list_builtin_tools(&self) -> Result<ScriptValue>;
    fn get_tool_schema(&self, tool_name: &str) -> Result<ScriptValue>;
    fn execute_tool(&self, tool_name: &str, input: ScriptValue, context: ScriptValue) -> Result<ScriptValue>;
    fn wrap_agent_as_tool(&self, agent_id: &str, tool_config: ToolConfig) -> Result<ScriptValue>;
}

pub trait WorkflowBridge: Bridge {
    // Workflow orchestration
    fn create_sequential_workflow(&self, config: WorkflowConfig) -> Result<ScriptValue>;
    fn create_parallel_workflow(&self, config: WorkflowConfig) -> Result<ScriptValue>;
    fn create_conditional_workflow(&self, config: WorkflowConfig) -> Result<ScriptValue>;
    fn add_step_to_workflow(&self, workflow_id: &str, step: ScriptValue) -> Result<()>;
    fn execute_workflow(&self, workflow_id: &str, context: ScriptValue) -> Result<ScriptValue>;
}

pub trait StateBridge: Bridge {
    // State management
    fn create_state(&self, config: StateConfig) -> Result<ScriptValue>;
    fn get_state_value(&self, state_id: &str, key: &str) -> Result<ScriptValue>;
    fn set_state_value(&self, state_id: &str, key: &str, value: ScriptValue) -> Result<()>;
    fn transform_state(&self, state_id: &str, transform: StateTransform) -> Result<ScriptValue>;
    fn merge_states(&self, state_ids: Vec<&str>, strategy: MergeStrategy) -> Result<ScriptValue>;
}

pub trait HookBridge: Bridge {
    // Hook management
    fn create_hook(&self, hook_config: HookConfig) -> Result<ScriptValue>;
    fn register_script_hook(&self, hook_point: HookPoint, script_function: ScriptValue) -> Result<()>;
    fn list_hook_points(&self) -> Result<ScriptValue>;
}

pub trait EventBridge: Bridge {
    // Event system
    fn subscribe_to_events(&self, filter: EventFilter) -> Result<StreamHandle>;
    fn emit_custom_event(&self, event_type: &str, data: ScriptValue) -> Result<()>;
    fn get_recent_events(&self, count: usize) -> Result<ScriptValue>;
}
```

### Bridge Registration Pattern

```rust
pub struct BridgeRegistry {
    bridges: HashMap<String, Box<dyn Bridge>>,
    dependencies: HashMap<String, Vec<String>>,
}

impl BridgeRegistry {
    pub fn register_hierarchical_bridges(&mut self) {
        // Core foundation bridges
        self.register("state", Box::new(StateBridge::new()));
        self.register("hooks", Box::new(HookBridge::new()));
        self.register("events", Box::new(EventBridge::new()));
        
        // Base layer bridges (depend on foundation)
        self.register_with_deps("base_agent", 
            Box::new(BaseAgentBridge::new()), 
            vec!["state", "hooks", "events"]);
        
        // Tool bridges (depend on base layer)
        self.register_with_deps("tools", 
            Box::new(ToolBridge::new()), 
            vec!["base_agent"]);
        
        // Agent bridges (depend on base and tools)
        self.register_with_deps("agent", 
            Box::new(AgentBridge::new()), 
            vec!["base_agent", "tools"]);
        
        // Workflow bridges (depend on agents and tools)
        self.register_with_deps("workflow", 
            Box::new(WorkflowBridge::new()), 
            vec!["agent", "tools"]);
    }
}
```

## Script API Changes

### Lua API Design

**Core Module Structure:**
```lua
-- Core modules exposed to scripts
local agent = require("rs_llmspell.agent")
local tools = require("rs_llmspell.tools")
local workflows = require("rs_llmspell.workflows")
local state = require("rs_llmspell.state")
local hooks = require("rs_llmspell.hooks")
local events = require("rs_llmspell.events")

-- BaseAgent API
local base = agent.base({
    id = "my_agent",
    name = "My Agent",
    description = "Does useful things"
})

-- Tool management
tools.list_builtin() -- Returns table of available tools
tools.get_schema("web_search") -- Returns JSON schema

base:add_tool("web_search")
base:add_tool("file_write")

-- Hook registration
local logging_hook = hooks.create("logging", {
    level = "info",
    points = {"pre_tool", "post_tool", "pre_llm", "post_llm"}
})

base:add_hook(logging_hook)

-- Custom script hook
base:add_script_hook("pre_tool", function(context)
    print("About to execute tool: " .. context.tool_name)
    return { continue = true }
end)

-- State management
local agent_state = state.create({
    persistence = "memory",
    variables = {
        session_id = "abc123",
        user_preferences = { theme = "dark" }
    }
})

-- LLM Agent creation
local llm_agent = agent.llm(base, {
    provider = "anthropic",
    model = "claude-3-opus",
    temperature = 0.7,
    system = "You are a helpful assistant"
})

-- Execution patterns
local response = llm_agent:run("Hello")
local response_with_state = llm_agent:run_with_state("Analyze data", agent_state)

-- Streaming
local stream = llm_agent:stream("Write a story")
for chunk in stream do
    if chunk.type == "content" then
        io.write(chunk.text)
    elseif chunk.type == "tool_call" then
        print("Calling tool: " .. chunk.tool_name)
    end
end
```

**Workflow API:**
```lua
-- Sequential workflow
local workflow = workflows.sequential({
    id = "research_pipeline",
    name = "Research Pipeline"
})

workflow:add_step({
    type = "agent",
    agent = researcher_agent,
    input = "Research {{topic}}"
})

workflow:add_step({
    type = "agent", 
    agent = writer_agent,
    input = "Write article based on: {{step_1.output}}"
})

-- Conditional workflow
local conditional = workflows.conditional({
    id = "smart_routing"
})

conditional:add_branch({
    condition = "input.type == 'code'",
    workflow = code_analysis_workflow
})

conditional:add_branch({
    condition = "input.type == 'text'", 
    workflow = text_analysis_workflow
})

-- Execute workflow
local context = workflows.context({
    variables = { topic = "AI safety" }
})

local result = workflow:execute(context)
```

**Tool-Wrapped Agent API:**
```lua
-- Create specialized agent
local code_reviewer = agent.llm(
    agent.base({ id = "code_reviewer" }):add_tool("file_read"),
    { model = "claude-3-opus", system = "You review code for quality" }
)

-- Wrap agent as tool
local review_tool = tools.wrap_agent(code_reviewer, {
    name = "code_review",
    description = "Reviews code files for quality and security",
    parameters = {
        file_path = { type = "string", description = "Path to code file" }
    }
})

-- Use wrapped agent in another agent
local senior_agent = agent.llm(
    agent.base({ id = "senior_dev" }):add_tool(review_tool),
    { model = "gpt-4", system = "You coordinate development tasks" }
)

local result = senior_agent:run("Review the main.rs file and suggest improvements")
```

### JavaScript API Design

**ES6 Module Structure:**
```javascript
import { agent, tools, workflows, state, hooks, events } from 'rs-llmspell';

// BaseAgent with fluent API
const base = agent.base({
    id: 'my_agent',
    name: 'My Agent'
})
.addTool('web_search')
.addTool('file_write')
.addHook(hooks.logging({ level: 'info' }));

// Async/await pattern
const llmAgent = agent.llm(base, {
    provider: 'anthropic',
    model: 'claude-3-opus',
    temperature: 0.7
});

// Promise-based execution
const response = await llmAgent.run('Hello world');
console.log(response.content);

// State management with async
const agentState = await state.create({
    persistence: 'file',
    path: './agent_state.json'
});

await agentState.set('user_id', 'user123');
const result = await llmAgent.runWithState('Process user data', agentState);

// Event subscription
const eventStream = events.subscribe({
    types: ['agent_completed', 'tool_executed']
});

eventStream.on('data', (event) => {
    console.log(`Event: ${event.type}`, event.data);
});

// Workflow with async/await
const workflow = workflows.sequential({ id: 'async_pipeline' });
workflow.addStep({ type: 'agent', agent: dataProcessor });
workflow.addStep({ type: 'agent', agent: reportGenerator });

const context = workflows.context({ 
    variables: { dataSource: 'database' } 
});

const workflowResult = await workflow.execute(context);
```

## Type System Implications

### Enhanced ScriptValue Types

```rust
// Extended ScriptValue for new architecture
#[derive(Debug, Clone)]
pub enum ScriptValue {
    // Basic types
    Nil,
    Boolean(bool),
    Number(f64),
    String(String),
    Array(Vec<ScriptValue>),
    Object(HashMap<String, ScriptValue>),
    
    // New: Complex types for architecture
    AgentHandle(AgentHandle),
    ToolHandle(ToolHandle),
    WorkflowHandle(WorkflowHandle),
    StateHandle(StateHandle),
    HookHandle(HookHandle),
    StreamHandle(StreamHandle),
    
    // Function types
    Function(ScriptFunction),
    AsyncFunction(AsyncScriptFunction),
    
    // Error handling
    Error(ScriptError),
    Result(Box<Result<ScriptValue, ScriptValue>>),
}

// Handle types for resource management
#[derive(Debug, Clone)]
pub struct AgentHandle {
    pub id: String,
    pub agent_type: AgentType,
    pub capabilities: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ToolHandle {
    pub name: String,
    pub schema: ToolSchema,
    pub categories: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct WorkflowHandle {
    pub id: String,
    pub workflow_type: WorkflowType,
    pub step_count: usize,
}

#[derive(Debug, Clone)]
pub struct StateHandle {
    pub id: String,
    pub persistence_type: PersistenceType,
    pub size_hint: Option<usize>,
}
```

### Type Conversion Patterns

```rust
// Enhanced type converter for new types
impl TypeConverter {
    pub fn agent_to_script_value(&self, agent: &dyn Agent) -> Result<ScriptValue> {
        Ok(ScriptValue::AgentHandle(AgentHandle {
            id: agent.id().to_string(),
            agent_type: self.classify_agent_type(agent),
            capabilities: agent.capabilities().supported_input_types,
        }))
    }
    
    pub fn script_value_to_agent_ref(&self, value: &ScriptValue) -> Result<&dyn Agent> {
        match value {
            ScriptValue::AgentHandle(handle) => {
                self.agent_registry.get(&handle.id)
                    .ok_or_else(|| ConversionError::AgentNotFound(handle.id.clone()))
            }
            _ => Err(ConversionError::TypeMismatch {
                expected: "AgentHandle".to_string(),
                actual: format!("{:?}", value),
            })
        }
    }
    
    // Workflow step conversion
    pub fn script_workflow_step_to_rust(&self, step: ScriptValue) -> Result<Box<dyn WorkflowStep>> {
        match step {
            ScriptValue::Object(map) => {
                let step_type = map.get("type")
                    .and_then(|v| v.as_string())
                    .ok_or(ConversionError::MissingField("type"))?;
                
                match step_type.as_str() {
                    "agent" => {
                        let agent_handle = map.get("agent")
                            .ok_or(ConversionError::MissingField("agent"))?;
                        let agent = self.script_value_to_agent_ref(agent_handle)?;
                        let input = map.get("input")
                            .and_then(|v| v.as_string())
                            .unwrap_or_default();
                        
                        Ok(Box::new(AgentStep::new(
                            &format!("agent_{}", agent.id()),
                            agent,
                            input
                        )))
                    }
                    "tool" => {
                        let tool_name = map.get("tool")
                            .and_then(|v| v.as_string())
                            .ok_or(ConversionError::MissingField("tool"))?;
                        let input = map.get("input")
                            .cloned()
                            .unwrap_or(ScriptValue::Object(HashMap::new()));
                        
                        Ok(Box::new(ToolStep::new(
                            &format!("tool_{}", tool_name),
                            self.tool_registry.get(tool_name)?,
                            self.script_value_to_tool_input(input)?
                        )))
                    }
                    _ => Err(ConversionError::UnsupportedStepType(step_type))
                }
            }
            _ => Err(ConversionError::TypeMismatch {
                expected: "Object".to_string(),
                actual: format!("{:?}", step),
            })
        }
    }
}
```

## Cross-Engine Compatibility

### Unified API Pattern

```rust
// Engine-agnostic bridge interface
pub trait ScriptEngine {
    // New: Handle-based resource management
    fn register_agent_handle(&mut self, handle: AgentHandle) -> Result<()>;
    fn register_tool_handle(&mut self, handle: ToolHandle) -> Result<()>;
    fn register_workflow_handle(&mut self, handle: WorkflowHandle) -> Result<()>;
    
    // Enhanced execution with typed returns
    async fn execute_with_typed_result(&self, script: &str) -> Result<TypedScriptResult>;
    
    // Stream support for async operations
    fn create_stream_handle(&self, stream_id: &str) -> Result<StreamHandle>;
    fn read_from_stream(&self, handle: StreamHandle) -> Result<Option<ScriptValue>>;
    
    // Hook support
    fn register_script_hook(&mut self, point: HookPoint, function: ScriptFunction) -> Result<()>;
}

// Typed script result for better error handling
#[derive(Debug)]
pub enum TypedScriptResult {
    Value(ScriptValue),
    Agent(AgentHandle),
    Tool(ToolHandle),
    Workflow(WorkflowHandle),
    State(StateHandle),
    Stream(StreamHandle),
    Void,
}
```

### Engine-Specific Adaptations

**Lua Engine Enhancements:**
```rust
impl ScriptEngine for LuaEngine {
    fn register_agent_handle(&mut self, handle: AgentHandle) -> Result<()> {
        // Create Lua userdata for agent handle
        let userdata = self.lua.create_userdata(handle)?;
        
        // Add methods to userdata
        userdata.add_method("run", |_, agent_handle, input: String| {
            // Bridge to Rust agent execution
            let agent = AGENT_REGISTRY.get(&agent_handle.id)?;
            agent.run(&input)
        });
        
        userdata.add_method("run_with_state", |_, agent_handle, (input, state_id): (String, String)| {
            let agent = AGENT_REGISTRY.get(&agent_handle.id)?;
            let state = STATE_REGISTRY.get(&state_id)?;
            agent.run_with_state(&input, &mut state)
        });
        
        // Register in global scope
        self.lua.globals().set(&handle.id, userdata)?;
        Ok(())
    }
}
```

**JavaScript Engine Enhancements:**
```rust
impl ScriptEngine for JavaScriptEngine {
    fn register_agent_handle(&mut self, handle: AgentHandle) -> Result<()> {
        // Create JavaScript object with methods
        let agent_obj = self.context.new_object();
        
        // Add async methods
        agent_obj.set_property("run", self.context.new_function(|input| async move {
            let agent = AGENT_REGISTRY.get(&handle.id)?;
            let result = agent.run(&input).await?;
            Ok(self.convert_agent_response_to_js(result))
        }))?;
        
        agent_obj.set_property("runWithState", self.context.new_function(|(input, state_id)| async move {
            let agent = AGENT_REGISTRY.get(&handle.id)?;
            let state = STATE_REGISTRY.get(&state_id)?;
            let result = agent.run_with_state(&input, &mut state).await?;
            Ok(self.convert_agent_response_to_js(result))
        }))?;
        
        // Register in global scope
        self.context.global().set_property(&handle.id, agent_obj)?;
        Ok(())
    }
}
```

## Performance Considerations

### Handle-Based Resource Management

```rust
// Resource pool for efficient handle management
pub struct ResourcePool {
    agents: Arc<RwLock<HandlePool<AgentHandle, Arc<dyn Agent>>>>,
    tools: Arc<RwLock<HandlePool<ToolHandle, Arc<dyn Tool>>>>,
    workflows: Arc<RwLock<HandlePool<WorkflowHandle, Arc<dyn Workflow>>>>,
    states: Arc<RwLock<HandlePool<StateHandle, Arc<RwLock<AgentState>>>>>,
}

struct HandlePool<H, R> {
    handles: HashMap<String, H>,
    resources: HashMap<String, R>,
    reference_counts: HashMap<String, usize>,
}

impl<H, R> HandlePool<H, R> {
    pub fn register(&mut self, handle: H, resource: R) -> String 
    where 
        H: HasId 
    {
        let id = handle.id().to_string();
        self.handles.insert(id.clone(), handle);
        self.resources.insert(id.clone(), resource);
        self.reference_counts.insert(id.clone(), 1);
        id
    }
    
    pub fn acquire(&mut self, id: &str) -> Option<&R> {
        if let Some(count) = self.reference_counts.get_mut(id) {
            *count += 1;
            self.resources.get(id)
        } else {
            None
        }
    }
    
    pub fn release(&mut self, id: &str) -> Option<R> {
        if let Some(count) = self.reference_counts.get_mut(id) {
            *count -= 1;
            if *count == 0 {
                self.reference_counts.remove(id);
                self.handles.remove(id);
                self.resources.remove(id)
            } else {
                None
            }
        } else {
            None
        }
    }
}
```

### Async Stream Optimization

```rust
// Efficient streaming for long-running operations
pub struct AsyncStreamBridge {
    active_streams: Arc<RwLock<HashMap<String, StreamContext>>>,
    stream_buffer_size: usize,
}

struct StreamContext {
    sender: mpsc::Sender<ScriptValue>,
    receiver: mpsc::Receiver<ScriptValue>,
    metadata: StreamMetadata,
}

impl AsyncStreamBridge {
    pub async fn create_agent_stream(&self, agent_id: &str, input: &str) -> Result<StreamHandle> {
        let stream_id = format!("agent_{}_{}", agent_id, Utc::now().timestamp_nanos());
        let (sender, receiver) = mpsc::channel(self.stream_buffer_size);
        
        let agent = AGENT_REGISTRY.get(agent_id)?;
        
        // Spawn async task for agent execution
        let stream_sender = sender.clone();
        tokio::spawn(async move {
            let mut stream = agent.run_stream(input);
            
            while let Some(chunk) = stream.next().await {
                match chunk {
                    Ok(agent_chunk) => {
                        let script_value = convert_agent_chunk_to_script_value(agent_chunk);
                        if stream_sender.send(script_value).await.is_err() {
                            break; // Stream was closed
                        }
                    }
                    Err(e) => {
                        let error_value = ScriptValue::Error(ScriptError::from(e));
                        let _ = stream_sender.send(error_value).await;
                        break;
                    }
                }
            }
        });
        
        let context = StreamContext {
            sender,
            receiver,
            metadata: StreamMetadata {
                created_at: Utc::now(),
                stream_type: StreamType::Agent,
                source_id: agent_id.to_string(),
            },
        };
        
        let mut streams = self.active_streams.write().await;
        streams.insert(stream_id.clone(), context);
        
        Ok(StreamHandle {
            id: stream_id,
            stream_type: StreamType::Agent,
        })
    }
}
```

## Error Handling Updates

### Hierarchical Error Types

```rust
// Enhanced error types for new architecture
#[derive(Debug, thiserror::Error)]
pub enum ScriptBridgeError {
    #[error("Agent error: {0}")]
    Agent(#[from] AgentError),
    
    #[error("Tool error: {0}")]
    Tool(#[from] ToolError),
    
    #[error("Workflow error: {0}")]
    Workflow(#[from] WorkflowError),
    
    #[error("State error: {0}")]
    State(#[from] StateError),
    
    #[error("Hook error: {0}")]
    Hook(#[from] HookError),
    
    #[error("Handle not found: {handle_type} {id}")]
    HandleNotFound { handle_type: String, id: String },
    
    #[error("Type conversion error: {0}")]
    Conversion(#[from] ConversionError),
    
    #[error("Script execution error: {0}")]
    ScriptExecution(String),
    
    #[error("Async operation error: {0}")]
    AsyncOperation(String),
}

// Script-friendly error representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptError {
    pub error_type: String,
    pub message: String,
    pub context: HashMap<String, String>,
    pub stack_trace: Option<Vec<String>>,
    pub retry_after: Option<Duration>,
}

impl From<ScriptBridgeError> for ScriptError {
    fn from(err: ScriptBridgeError) -> Self {
        match err {
            ScriptBridgeError::Agent(agent_err) => ScriptError {
                error_type: "agent_error".to_string(),
                message: agent_err.to_string(),
                context: extract_agent_error_context(&agent_err),
                stack_trace: None,
                retry_after: None,
            },
            ScriptBridgeError::HandleNotFound { handle_type, id } => ScriptError {
                error_type: "handle_not_found".to_string(),
                message: format!("{} '{}' not found", handle_type, id),
                context: {
                    let mut ctx = HashMap::new();
                    ctx.insert("handle_type".to_string(), handle_type);
                    ctx.insert("handle_id".to_string(), id);
                    ctx
                },
                stack_trace: None,
                retry_after: None,
            },
            // ... other error conversions
        }
    }
}
```

### Error Recovery Patterns

```lua
-- Lua error handling with retry logic
local function safe_agent_call(agent, input, max_retries)
    local retries = 0
    
    while retries < max_retries do
        local success, result = pcall(function()
            return agent:run(input)
        end)
        
        if success then
            return result
        else
            -- Check if error is retryable
            if result.error_type == "rate_limit" and result.retry_after then
                print("Rate limited, waiting " .. result.retry_after .. " seconds")
                sleep(result.retry_after)
                retries = retries + 1
            elseif result.error_type == "temporary_failure" then
                print("Temporary failure, retrying...")
                retries = retries + 1
            else
                -- Non-retryable error
                error("Agent call failed: " .. result.message)
            end
        end
    end
    
    error("Agent call failed after " .. max_retries .. " retries")
end
```

```javascript
// JavaScript error handling with async/await
async function safeAgentCall(agent, input, maxRetries = 3) {
    for (let attempt = 0; attempt < maxRetries; attempt++) {
        try {
            return await agent.run(input);
        } catch (error) {
            if (error.errorType === 'rate_limit' && error.retryAfter) {
                console.log(`Rate limited, waiting ${error.retryAfter} seconds`);
                await new Promise(resolve => 
                    setTimeout(resolve, error.retryAfter * 1000)
                );
            } else if (error.errorType === 'temporary_failure') {
                console.log('Temporary failure, retrying...');
                await new Promise(resolve => setTimeout(resolve, 1000 * attempt));
            } else {
                throw error; // Non-retryable error
            }
        }
    }
    
    throw new Error(`Agent call failed after ${maxRetries} retries`);
}
```

## Migration Strategy

### Backward Compatibility Layer

```rust
// Compatibility shim for existing scripts
pub struct LegacyAgentBridge {
    new_bridge: Arc<AgentBridge>,
    legacy_agents: HashMap<String, LegacyAgentInfo>,
}

struct LegacyAgentInfo {
    base_agent_id: String,
    llm_agent_id: String,
    tools: Vec<String>,
}

impl Bridge for LegacyAgentBridge {
    async fn execute_method(&self, name: &str, args: Vec<ScriptValue>) -> Result<ScriptValue> {
        match name {
            "create_agent" => {
                // Legacy: llm.agent({ model, tools, system })
                // New: Create BaseAgent, add tools, create LLM agent
                let config = args[0].as_object()?;
                
                let base_agent_id = format!("legacy_{}", uuid::Uuid::new_v4());
                let base_agent = self.new_bridge.create_base_agent(BaseAgentConfig {
                    id: base_agent_id.clone(),
                    name: "Legacy Agent".to_string(),
                    description: "Migrated from legacy API".to_string(),
                }).await?;
                
                // Add tools if specified
                if let Some(tools) = config.get("tools").and_then(|v| v.as_array()) {
                    for tool in tools {
                        let tool_name = tool.as_string()?;
                        self.new_bridge.add_tool_to_agent(&base_agent_id, tool_name).await?;
                    }
                }
                
                // Create LLM agent
                let llm_config = LlmConfig {
                    model: config.get("model").and_then(|v| v.as_string()).unwrap_or("gpt-3.5-turbo"),
                    system: config.get("system").and_then(|v| v.as_string()),
                    temperature: config.get("temperature").and_then(|v| v.as_number()),
                    max_tokens: config.get("max_tokens").and_then(|v| v.as_number()).map(|n| n as u32),
                };
                
                let llm_agent_id = self.new_bridge.create_llm_agent(&base_agent_id, llm_config).await?;
                
                // Store legacy mapping
                self.legacy_agents.insert(llm_agent_id.clone(), LegacyAgentInfo {
                    base_agent_id,
                    llm_agent_id: llm_agent_id.clone(),
                    tools: tools.unwrap_or_default(),
                });
                
                Ok(ScriptValue::String(llm_agent_id))
            }
            "run_agent" => {
                // Legacy: agent:run(input)
                // New: agent:run(input)
                let agent_id = args[0].as_string()?;
                let input = args[1].as_string()?;
                
                self.new_bridge.run_agent(agent_id, input).await
            }
            _ => Err(ScriptBridgeError::UnknownMethod(name.to_string()))
        }
    }
}
```

### Migration Script Generator

```rust
// Tool to help migrate existing scripts
pub struct ScriptMigrator {
    patterns: Vec<MigrationPattern>,
}

struct MigrationPattern {
    from: Regex,
    to: String,
    explanation: String,
}

impl ScriptMigrator {
    pub fn new() -> Self {
        Self {
            patterns: vec![
                MigrationPattern {
                    from: Regex::new(r"llm\.agent\((\{[^}]+\})\)").unwrap(),
                    to: r"
-- New pattern:
local base = agent.base({ id = 'migrated_agent' })
$tools
local llm_agent = agent.llm(base, $config)".to_string(),
                    explanation: "Split agent creation into base + LLM layers".to_string(),
                },
                MigrationPattern {
                    from: Regex::new(r"agent:run\(([^)]+)\)").unwrap(),
                    to: r"agent:run($1)".to_string(),
                    explanation: "Basic run method unchanged".to_string(),
                },
                // More patterns...
            ],
        }
    }
    
    pub fn migrate_script(&self, script: &str) -> MigrationResult {
        let mut migrated = script.to_string();
        let mut applied_patterns = Vec::new();
        
        for pattern in &self.patterns {
            if pattern.from.is_match(&migrated) {
                migrated = pattern.from.replace_all(&migrated, &pattern.to).to_string();
                applied_patterns.push(pattern.explanation.clone());
            }
        }
        
        MigrationResult {
            original: script.to_string(),
            migrated,
            changes: applied_patterns,
            breaking_changes: self.detect_breaking_changes(script),
        }
    }
}
```

## Conclusion

The new BaseAgent/Agent/Tool/Workflow hierarchy requires significant changes to the scripting interface but provides much more powerful and flexible capabilities:

### Key Benefits

1. **Composability**: Scripts can build complex agent behaviors incrementally
2. **Observability**: Full hook and event system exposed to scripts
3. **State Management**: First-class state handling for agent handoff
4. **Type Safety**: Better error handling and type checking
5. **Performance**: Resource pooling and efficient streaming

### Breaking Changes

1. **Agent Creation**: Multi-step process instead of single function call
2. **Tool Management**: Explicit tool addition to base agents
3. **State Handling**: New state management patterns required
4. **Error Types**: Enhanced error information requires script updates

### Migration Path

1. **Phase 1**: Deploy compatibility layer alongside new API
2. **Phase 2**: Provide migration tools and documentation
3. **Phase 3**: Deprecate legacy API with clear timeline
4. **Phase 4**: Remove compatibility layer in next major version

The enhanced scripting interface unlocks the full power of the new architecture while maintaining the ease of use that makes rs-llmspell valuable for rapid LLM application development.