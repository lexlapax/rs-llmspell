# Bridge Integration Analysis for Rs-LLMSpell

## Overview

Analysis of how the new BaseAgent/Agent/Tool/Workflow hierarchy integrates with rs-llmspell's bridge layer, examining script engine implications, testing strategy updates, and implementation requirements.

## Table of Contents

1. [Current Bridge Architecture Review](#current-bridge-architecture-review)
2. [New Hierarchy Bridge Requirements](#new-hierarchy-bridge-requirements)
3. [Bridge Layer Redesign](#bridge-layer-redesign)
4. [Script Engine Integration](#script-engine-integration)
5. [Type System Integration](#type-system-integration)
6. [Performance Implications](#performance-implications)
7. [Testing Strategy Updates](#testing-strategy-updates)
8. [Migration Path](#migration-path)
9. [Implementation Roadmap](#implementation-roadmap)

## Current Bridge Architecture Review

### Existing Bridge Pattern

Based on the current rs-llmspell architecture, the bridge layer follows a simple delegation pattern:

```rust
// Current simplified bridge interface
pub trait Bridge: Send + Sync {
    fn name(&self) -> &str;
    fn methods(&self) -> Vec<MethodInfo>;
    async fn execute_method(&self, name: &str, args: Vec<ScriptValue>) -> Result<ScriptValue, BridgeError>;
}

// Current implementation focuses on basic agent operations
pub struct AgentBridge {
    agents: Arc<RwLock<HashMap<String, Agent>>>,
}

impl Bridge for AgentBridge {
    async fn execute_method(&self, name: &str, args: Vec<ScriptValue>) -> Result<ScriptValue, BridgeError> {
        match name {
            "create_agent" => { /* Simple agent creation */ },
            "run_agent" => { /* Basic agent execution */ },
            _ => Err(BridgeError::UnknownMethod(name.to_string()))
        }
    }
}
```

**Limitations of Current Approach:**
1. **Flat Method Space**: All operations are in a single namespace
2. **Limited Composition**: No support for complex object relationships
3. **No State Management**: State is internal to agents, not bridged
4. **Basic Error Handling**: Limited error context and recovery
5. **No Hook Support**: No way to inject custom behavior
6. **Tool Isolation**: Tools and agents are separate systems

## New Hierarchy Bridge Requirements

### Required Bridge Capabilities

The new BaseAgent/Agent/Tool/Workflow hierarchy requires the bridge layer to support:

1. **Hierarchical Object Management**: BaseAgents, Agents, Tools, Workflows as first-class objects
2. **Resource Lifecycle**: Creation, modification, destruction of complex objects
3. **Composition Patterns**: Adding tools to agents, wrapping agents as tools
4. **Hook Integration**: Script-level hook registration and execution
5. **Event System**: Event subscription, emission, and handling
6. **State Management**: Shared state creation, access, and modification
7. **Streaming Operations**: Long-running operations with incremental results

### Object Relationship Mapping

```rust
// New hierarchical bridge system
pub trait BaseBridge: Send + Sync {
    // Resource management
    async fn create_resource(&self, resource_type: ResourceType, config: ScriptValue) -> Result<ResourceHandle, BridgeError>;
    async fn get_resource(&self, handle: &ResourceHandle) -> Result<ScriptValue, BridgeError>;
    async fn destroy_resource(&self, handle: &ResourceHandle) -> Result<(), BridgeError>;
    
    // Composition operations
    async fn compose_resources(&self, parent: &ResourceHandle, child: &ResourceHandle, relation: CompositionType) -> Result<(), BridgeError>;
    async fn decompose_resources(&self, parent: &ResourceHandle, child: &ResourceHandle) -> Result<(), BridgeError>;
}

#[derive(Debug, Clone)]
pub enum ResourceType {
    BaseAgent,
    Agent,
    Tool,
    Workflow,
    State,
    Hook,
    EventSubscription,
}

#[derive(Debug, Clone)]
pub enum CompositionType {
    ToolToAgent,    // Add tool to agent
    AgentToTool,    // Wrap agent as tool
    StepToWorkflow, // Add step to workflow
    HookToAgent,    // Add hook to agent
    StateToAgent,   // Associate state with agent
}

#[derive(Debug, Clone)]
pub struct ResourceHandle {
    pub id: String,
    pub resource_type: ResourceType,
    pub created_at: DateTime<Utc>,
    pub metadata: HashMap<String, String>,
}
```

## Bridge Layer Redesign

### Modular Bridge Architecture

Instead of a monolithic bridge, implement specialized bridges for each concept:

```rust
// Bridge registry with specialized bridges
pub struct BridgeRegistry {
    bridges: HashMap<String, Box<dyn Bridge>>,
    resource_manager: Arc<ResourceManager>,
    type_converter: Arc<TypeConverter>,
}

impl BridgeRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            bridges: HashMap::new(),
            resource_manager: Arc::new(ResourceManager::new()),
            type_converter: Arc::new(TypeConverter::new()),
        };
        
        // Register specialized bridges
        registry.register_bridge("base_agent", Box::new(BaseAgentBridge::new()));
        registry.register_bridge("agent", Box::new(AgentBridge::new()));
        registry.register_bridge("tool", Box::new(ToolBridge::new()));
        registry.register_bridge("workflow", Box::new(WorkflowBridge::new()));
        registry.register_bridge("state", Box::new(StateBridge::new()));
        registry.register_bridge("hooks", Box::new(HookBridge::new()));
        registry.register_bridge("events", Box::new(EventBridge::new()));
        
        registry
    }
}

// Specialized bridge for BaseAgent operations
pub struct BaseAgentBridge {
    agents: Arc<RwLock<HashMap<String, Arc<dyn BaseAgent>>>>,
    resource_manager: Arc<ResourceManager>,
}

impl Bridge for BaseAgentBridge {
    async fn execute_method(&self, name: &str, args: Vec<ScriptValue>) -> Result<ScriptValue, BridgeError> {
        match name {
            "create" => self.create_base_agent(args).await,
            "add_tool" => self.add_tool_to_agent(args).await,
            "remove_tool" => self.remove_tool_from_agent(args).await,
            "add_hook" => self.add_hook_to_agent(args).await,
            "get_state" => self.get_agent_state(args).await,
            "set_state" => self.set_agent_state(args).await,
            "list_tools" => self.list_agent_tools(args).await,
            "list_hooks" => self.list_agent_hooks(args).await,
            _ => Err(BridgeError::UnknownMethod(format!("base_agent.{}", name)))
        }
    }
}

impl BaseAgentBridge {
    async fn create_base_agent(&self, args: Vec<ScriptValue>) -> Result<ScriptValue, BridgeError> {
        let config = args.get(0).ok_or(BridgeError::MissingArgument("config"))?;
        
        // Extract configuration from script value
        let agent_config = self.resource_manager.script_value_to_base_agent_config(config)?;
        
        // Create base agent
        let base_agent = BaseAgentImpl::new(agent_config)?;
        let agent_id = base_agent.id().to_string();
        
        // Store in resource manager
        let handle = ResourceHandle {
            id: agent_id.clone(),
            resource_type: ResourceType::BaseAgent,
            created_at: Utc::now(),
            metadata: HashMap::new(),
        };
        
        self.agents.write().await.insert(agent_id.clone(), Arc::new(base_agent));
        self.resource_manager.register_resource(handle.clone()).await?;
        
        // Return handle as script value
        Ok(ScriptValue::ResourceHandle(handle))
    }
    
    async fn add_tool_to_agent(&self, args: Vec<ScriptValue>) -> Result<ScriptValue, BridgeError> {
        let agent_handle = args.get(0).ok_or(BridgeError::MissingArgument("agent_handle"))?;
        let tool_name = args.get(1).ok_or(BridgeError::MissingArgument("tool_name"))?
            .as_string().ok_or(BridgeError::InvalidArgumentType("tool_name must be string"))?;
        
        let agent_id = match agent_handle {
            ScriptValue::ResourceHandle(handle) => &handle.id,
            _ => return Err(BridgeError::InvalidArgumentType("Expected ResourceHandle"))
        };
        
        // Get base agent and tool
        let agents = self.agents.read().await;
        let agent = agents.get(agent_id).ok_or(BridgeError::ResourceNotFound(agent_id.clone()))?;
        
        let tool = TOOL_REGISTRY.get(tool_name)
            .ok_or(BridgeError::ToolNotFound(tool_name.clone()))?;
        
        // Add tool to agent through resource manager composition
        self.resource_manager.compose_resources(
            agent_handle.clone(),
            ScriptValue::String(tool_name.clone()),
            CompositionType::ToolToAgent
        ).await?;
        
        Ok(ScriptValue::Nil)
    }
}
```

### Agent Bridge Implementation

```rust
// Specialized bridge for LLM Agent operations
pub struct AgentBridge {
    base_bridge: Arc<BaseAgentBridge>,
    llm_agents: Arc<RwLock<HashMap<String, Arc<dyn Agent>>>>,
}

impl Bridge for AgentBridge {
    async fn execute_method(&self, name: &str, args: Vec<ScriptValue>) -> Result<ScriptValue, BridgeError> {
        match name {
            "create_from_base" => self.create_agent_from_base(args).await,
            "run" => self.run_agent(args).await,
            "run_with_state" => self.run_agent_with_state(args).await,
            "stream" => self.stream_agent(args).await,
            "set_system_prompt" => self.set_system_prompt(args).await,
            "set_model_config" => self.set_model_config(args).await,
            "get_conversation_history" => self.get_conversation_history(args).await,
            _ => Err(BridgeError::UnknownMethod(format!("agent.{}", name)))
        }
    }
}

impl AgentBridge {
    async fn create_agent_from_base(&self, args: Vec<ScriptValue>) -> Result<ScriptValue, BridgeError> {
        let base_agent_handle = args.get(0).ok_or(BridgeError::MissingArgument("base_agent_handle"))?;
        let llm_config = args.get(1).ok_or(BridgeError::MissingArgument("llm_config"))?;
        
        // Get base agent from handle
        let base_agent = self.base_bridge.get_agent_from_handle(base_agent_handle).await?;
        
        // Convert script config to LLM config
        let config = self.resource_manager.script_value_to_llm_config(llm_config)?;
        
        // Create agent wrapper
        let agent = AgentImpl::new(base_agent, config)?;
        let agent_id = format!("agent_{}", uuid::Uuid::new_v4());
        
        // Register resource
        let handle = ResourceHandle {
            id: agent_id.clone(),
            resource_type: ResourceType::Agent,
            created_at: Utc::now(),
            metadata: {
                let mut meta = HashMap::new();
                meta.insert("base_agent_id".to_string(), base_agent.id().to_string());
                meta.insert("model".to_string(), config.model.clone());
                meta
            },
        };
        
        self.llm_agents.write().await.insert(agent_id, Arc::new(agent));
        self.resource_manager.register_resource(handle.clone()).await?;
        
        Ok(ScriptValue::ResourceHandle(handle))
    }
    
    async fn run_agent(&self, args: Vec<ScriptValue>) -> Result<ScriptValue, BridgeError> {
        let agent_handle = args.get(0).ok_or(BridgeError::MissingArgument("agent_handle"))?;
        let input = args.get(1).ok_or(BridgeError::MissingArgument("input"))?
            .as_string().ok_or(BridgeError::InvalidArgumentType("input must be string"))?;
        
        let agent_id = match agent_handle {
            ScriptValue::ResourceHandle(handle) => &handle.id,
            _ => return Err(BridgeError::InvalidArgumentType("Expected ResourceHandle"))
        };
        
        let agents = self.llm_agents.read().await;
        let agent = agents.get(agent_id).ok_or(BridgeError::ResourceNotFound(agent_id.clone()))?;
        
        // Execute agent with hook integration
        let result = agent.run(input).await.map_err(BridgeError::AgentExecution)?;
        
        // Convert response to script value
        Ok(self.resource_manager.agent_response_to_script_value(result)?)
    }
    
    async fn stream_agent(&self, args: Vec<ScriptValue>) -> Result<ScriptValue, BridgeError> {
        let agent_handle = args.get(0).ok_or(BridgeError::MissingArgument("agent_handle"))?;
        let input = args.get(1).ok_or(BridgeError::MissingArgument("input"))?
            .as_string().ok_or(BridgeError::InvalidArgumentType("input must be string"))?;
        
        let agent_id = match agent_handle {
            ScriptValue::ResourceHandle(handle) => &handle.id,
            _ => return Err(BridgeError::InvalidArgumentType("Expected ResourceHandle"))
        };
        
        let agents = self.llm_agents.read().await;
        let agent = agents.get(agent_id).ok_or(BridgeError::ResourceNotFound(agent_id.clone()))?;
        
        // Create stream handle
        let stream_id = format!("stream_{}", uuid::Uuid::new_v4());
        let stream_handle = StreamHandle {
            id: stream_id.clone(),
            stream_type: StreamType::Agent,
            created_at: Utc::now(),
        };
        
        // Start streaming operation
        let agent_clone = Arc::clone(agent);
        let input_clone = input.clone();
        tokio::spawn(async move {
            // Implementation of streaming logic
            let _ = agent_clone.run_stream(&input_clone).await;
        });
        
        Ok(ScriptValue::StreamHandle(stream_handle))
    }
}
```

### Tool Bridge Implementation

```rust
// Specialized bridge for Tool operations
pub struct ToolBridge {
    tools: Arc<RwLock<HashMap<String, Arc<dyn Tool>>>>,
    wrapped_agents: Arc<RwLock<HashMap<String, Arc<dyn Agent>>>>,
}

impl Bridge for ToolBridge {
    async fn execute_method(&self, name: &str, args: Vec<ScriptValue>) -> Result<ScriptValue, BridgeError> {
        match name {
            "list_builtin" => self.list_builtin_tools(args).await,
            "get_schema" => self.get_tool_schema(args).await,
            "execute" => self.execute_tool(args).await,
            "wrap_agent" => self.wrap_agent_as_tool(args).await,
            "register_custom" => self.register_custom_tool(args).await,
            _ => Err(BridgeError::UnknownMethod(format!("tool.{}", name)))
        }
    }
}

impl ToolBridge {
    async fn wrap_agent_as_tool(&self, args: Vec<ScriptValue>) -> Result<ScriptValue, BridgeError> {
        let agent_handle = args.get(0).ok_or(BridgeError::MissingArgument("agent_handle"))?;
        let tool_config = args.get(1).ok_or(BridgeError::MissingArgument("tool_config"))?;
        
        let agent_id = match agent_handle {
            ScriptValue::ResourceHandle(handle) => &handle.id,
            _ => return Err(BridgeError::InvalidArgumentType("Expected ResourceHandle"))
        };
        
        // Get agent from resource manager
        let agent = self.resource_manager.get_agent(agent_id).await?;
        
        // Create tool wrapper configuration
        let config = self.resource_manager.script_value_to_tool_wrapper_config(tool_config)?;
        
        // Create wrapped tool
        let wrapped_tool = ToolWrappedAgent::new(agent, config)?;
        let tool_id = wrapped_tool.name().to_string();
        
        // Register as tool
        self.tools.write().await.insert(tool_id.clone(), Arc::new(wrapped_tool));
        
        let handle = ResourceHandle {
            id: tool_id.clone(),
            resource_type: ResourceType::Tool,
            created_at: Utc::now(),
            metadata: {
                let mut meta = HashMap::new();
                meta.insert("wrapped_agent_id".to_string(), agent_id.clone());
                meta.insert("tool_type".to_string(), "agent_wrapper".to_string());
                meta
            },
        };
        
        self.resource_manager.register_resource(handle.clone()).await?;
        
        Ok(ScriptValue::ResourceHandle(handle))
    }
    
    async fn execute_tool(&self, args: Vec<ScriptValue>) -> Result<ScriptValue, BridgeError> {
        let tool_name = args.get(0).ok_or(BridgeError::MissingArgument("tool_name"))?
            .as_string().ok_or(BridgeError::InvalidArgumentType("tool_name must be string"))?;
        let input = args.get(1).ok_or(BridgeError::MissingArgument("input"))?;
        let context = args.get(2).unwrap_or(&ScriptValue::Object(HashMap::new()));
        
        let tools = self.tools.read().await;
        let tool = tools.get(tool_name).ok_or(BridgeError::ToolNotFound(tool_name.clone()))?;
        
        // Convert script input to tool input
        let tool_input = self.resource_manager.script_value_to_tool_input(input)?;
        let tool_context = self.resource_manager.script_value_to_tool_context(context)?;
        
        // Execute tool with hook integration
        let result = tool.execute(tool_input, tool_context).await.map_err(BridgeError::ToolExecution)?;
        
        // Convert result back to script value
        Ok(self.resource_manager.tool_result_to_script_value(result)?)
    }
}
```

## Script Engine Integration

### Engine-Specific Adaptations

**Lua Engine Enhancement:**

```rust
impl ScriptEngine for LuaEngine {
    async fn register_bridges(&mut self, registry: &BridgeRegistry) -> Result<(), ScriptEngineError> {
        // Create Lua modules for each bridge
        let base_agent_module = self.lua.create_table()?;
        let agent_module = self.lua.create_table()?;
        let tool_module = self.lua.create_table()?;
        let workflow_module = self.lua.create_table()?;
        let state_module = self.lua.create_table()?;
        let hooks_module = self.lua.create_table()?;
        let events_module = self.lua.create_table()?;
        
        // Register methods for base_agent module
        self.register_bridge_methods(&base_agent_module, "base_agent", registry).await?;
        self.register_bridge_methods(&agent_module, "agent", registry).await?;
        self.register_bridge_methods(&tool_module, "tool", registry).await?;
        
        // Create metatable for resource handles
        self.create_resource_handle_metatable()?;
        
        // Register modules in global space
        self.lua.globals().set("base_agent", base_agent_module)?;
        self.lua.globals().set("agent", agent_module)?;
        self.lua.globals().set("tool", tool_module)?;
        self.lua.globals().set("workflow", workflow_module)?;
        self.lua.globals().set("state", state_module)?;
        self.lua.globals().set("hooks", hooks_module)?;
        self.lua.globals().set("events", events_module)?;
        
        Ok(())
    }
    
    fn create_resource_handle_metatable(&self) -> Result<(), ScriptEngineError> {
        // Create metatable for resource handles with methods
        let metatable = self.lua.create_table()?;
        let methods = self.lua.create_table()?;
        
        // Add common methods for all resource handles
        methods.set("get_id", self.lua.create_function(|_, handle: ResourceHandleUserData| {
            Ok(handle.handle.id.clone())
        })?)?;
        
        methods.set("get_type", self.lua.create_function(|_, handle: ResourceHandleUserData| {
            Ok(format!("{:?}", handle.handle.resource_type))
        })?)?;
        
        methods.set("get_metadata", self.lua.create_function(|_, handle: ResourceHandleUserData| {
            Ok(handle.handle.metadata.clone())
        })?)?;
        
        metatable.set("__index", methods)?;
        
        // Register metatable
        self.lua.set_named_registry_value("ResourceHandleMetatable", metatable)?;
        
        Ok(())
    }
}

// Lua userdata for resource handles
struct ResourceHandleUserData {
    handle: ResourceHandle,
}

impl mlua::UserData for ResourceHandleUserData {
    fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
        // Type-specific methods based on resource type
        methods.add_method("run", |_, this, input: String| async move {
            match this.handle.resource_type {
                ResourceType::Agent => {
                    // Call agent bridge
                    BRIDGE_REGISTRY.get("agent")
                        .execute_method("run", vec![
                            ScriptValue::ResourceHandle(this.handle.clone()),
                            ScriptValue::String(input)
                        ]).await
                },
                _ => Err(mlua::Error::RuntimeError("run not supported for this resource type".to_string()))
            }
        });
        
        methods.add_method("add_tool", |_, this, tool_name: String| async move {
            match this.handle.resource_type {
                ResourceType::BaseAgent | ResourceType::Agent => {
                    BRIDGE_REGISTRY.get("base_agent")
                        .execute_method("add_tool", vec![
                            ScriptValue::ResourceHandle(this.handle.clone()),
                            ScriptValue::String(tool_name)
                        ]).await
                },
                _ => Err(mlua::Error::RuntimeError("add_tool not supported for this resource type".to_string()))
            }
        });
    }
}
```

**JavaScript Engine Enhancement:**

```rust
impl ScriptEngine for JavaScriptEngine {
    async fn register_bridges(&mut self, registry: &BridgeRegistry) -> Result<(), ScriptEngineError> {
        // Create ES6 modules for each bridge
        let global = self.context.global_object()?;
        
        // Create rs_llmspell namespace
        let rs_llmspell = self.context.new_object()?;
        
        // Register bridge modules
        rs_llmspell.set_property("baseAgent", self.create_base_agent_module(registry).await?)?;
        rs_llmspell.set_property("agent", self.create_agent_module(registry).await?)?;
        rs_llmspell.set_property("tool", self.create_tool_module(registry).await?)?;
        rs_llmspell.set_property("workflow", self.create_workflow_module(registry).await?)?;
        rs_llmspell.set_property("state", self.create_state_module(registry).await?)?;
        rs_llmspell.set_property("hooks", self.create_hooks_module(registry).await?)?;
        rs_llmspell.set_property("events", self.create_events_module(registry).await?)?;
        
        global.set_property("rsLlmspell", rs_llmspell)?;
        
        Ok(())
    }
    
    async fn create_agent_module(&self, registry: &BridgeRegistry) -> Result<JSObject, ScriptEngineError> {
        let module = self.context.new_object()?;
        
        // createFromBase function
        module.set_property("createFromBase", self.context.new_function(|base_handle, config| async move {
            let bridge = BRIDGE_REGISTRY.get("agent");
            bridge.execute_method("create_from_base", vec![
                script_value_from_js(base_handle)?,
                script_value_from_js(config)?
            ]).await
        })?)?;
        
        // Add prototype methods to ResourceHandle for agents
        let resource_handle_prototype = self.get_resource_handle_prototype()?;
        
        resource_handle_prototype.set_property("run", self.context.new_function(|this, input| async move {
            let handle = get_resource_handle_from_this(this)?;
            if handle.resource_type != ResourceType::Agent {
                return Err(JSError::new("run method only available for Agent resources"));
            }
            
            let bridge = BRIDGE_REGISTRY.get("agent");
            bridge.execute_method("run", vec![
                ScriptValue::ResourceHandle(handle),
                ScriptValue::String(input)
            ]).await
        })?)?;
        
        resource_handle_prototype.set_property("stream", self.context.new_function(|this, input| async move {
            let handle = get_resource_handle_from_this(this)?;
            if handle.resource_type != ResourceType::Agent {
                return Err(JSError::new("stream method only available for Agent resources"));
            }
            
            let bridge = BRIDGE_REGISTRY.get("agent");
            bridge.execute_method("stream", vec![
                ScriptValue::ResourceHandle(handle),
                ScriptValue::String(input)
            ]).await
        })?)?;
        
        Ok(module)
    }
}
```

## Type System Integration

### Enhanced ScriptValue Types

```rust
// Extended ScriptValue enum for new architecture
#[derive(Debug, Clone)]
pub enum ScriptValue {
    // Basic types
    Nil,
    Boolean(bool),
    Number(f64),
    String(String),
    Array(Vec<ScriptValue>),
    Object(HashMap<String, ScriptValue>),
    
    // Resource management types
    ResourceHandle(ResourceHandle),
    StreamHandle(StreamHandle),
    
    // Function types for hooks
    Function(ScriptFunction),
    AsyncFunction(AsyncScriptFunction),
    
    // Result types
    AgentResponse(AgentResponse),
    ToolResult(ToolResult),
    WorkflowResult(WorkflowResult),
    
    // Error types
    Error(ScriptError),
}

// Type conversion utilities
pub struct TypeConverter {
    base_agent_configs: HashMap<String, BaseAgentConfig>,
    llm_configs: HashMap<String, LlmConfig>,
    tool_schemas: HashMap<String, ToolSchema>,
}

impl TypeConverter {
    pub fn script_value_to_base_agent_config(&self, value: &ScriptValue) -> Result<BaseAgentConfig, ConversionError> {
        match value {
            ScriptValue::Object(map) => {
                let id = map.get("id")
                    .and_then(|v| v.as_string())
                    .ok_or(ConversionError::MissingField("id"))?;
                let name = map.get("name")
                    .and_then(|v| v.as_string())
                    .ok_or(ConversionError::MissingField("name"))?;
                let description = map.get("description")
                    .and_then(|v| v.as_string())
                    .unwrap_or_default();
                
                Ok(BaseAgentConfig {
                    id: id.clone(),
                    name: name.clone(),
                    description: description.clone(),
                    initial_tools: map.get("tools")
                        .and_then(|v| v.as_array())
                        .map(|arr| arr.iter()
                            .filter_map(|v| v.as_string())
                            .cloned()
                            .collect())
                        .unwrap_or_default(),
                    initial_hooks: map.get("hooks")
                        .and_then(|v| v.as_array())
                        .map(|arr| self.parse_hook_configs(arr))
                        .transpose()?
                        .unwrap_or_default(),
                })
            }
            _ => Err(ConversionError::TypeMismatch {
                expected: "Object".to_string(),
                actual: format!("{:?}", value),
            })
        }
    }
    
    pub fn agent_response_to_script_value(&self, response: AgentResponse) -> Result<ScriptValue, ConversionError> {
        let mut obj = HashMap::new();
        
        obj.insert("content".to_string(), ScriptValue::String(response.content));
        obj.insert("model".to_string(), ScriptValue::String(response.model));
        obj.insert("tokens_used".to_string(), ScriptValue::Number(response.tokens_used as f64));
        obj.insert("finish_reason".to_string(), ScriptValue::String(format!("{:?}", response.finish_reason)));
        
        if let Some(tool_calls) = response.tool_calls {
            let calls: Vec<ScriptValue> = tool_calls.into_iter()
                .map(|call| self.tool_call_to_script_value(call))
                .collect::<Result<Vec<_>, _>>()?;
            obj.insert("tool_calls".to_string(), ScriptValue::Array(calls));
        }
        
        if !response.metadata.is_empty() {
            let metadata: HashMap<String, ScriptValue> = response.metadata.into_iter()
                .map(|(k, v)| (k, ScriptValue::String(v)))
                .collect();
            obj.insert("metadata".to_string(), ScriptValue::Object(metadata));
        }
        
        Ok(ScriptValue::Object(obj))
    }
}
```

## Performance Implications

### Resource Management Optimization

```rust
// Efficient resource management with pooling
pub struct ResourceManager {
    resources: Arc<RwLock<HashMap<String, Arc<dyn Resource>>>>,
    resource_pools: HashMap<ResourceType, Box<dyn ResourcePool>>,
    cleanup_scheduler: tokio::time::Interval,
}

impl ResourceManager {
    pub async fn register_resource(&self, handle: ResourceHandle) -> Result<(), ResourceError> {
        // Use resource pools for common types
        match handle.resource_type {
            ResourceType::BaseAgent => {
                self.resource_pools.get(&ResourceType::BaseAgent)
                    .ok_or(ResourceError::PoolNotFound)?
                    .acquire(&handle.id).await?;
            },
            _ => {
                // Direct storage for uncommon types
                self.resources.write().await.insert(handle.id.clone(), 
                    self.create_resource_from_handle(handle).await?);
            }
        }
        Ok(())
    }
    
    // Periodic cleanup of unused resources
    pub async fn cleanup_unused_resources(&self) -> Result<(), ResourceError> {
        let now = Utc::now();
        let retention_period = Duration::hours(1);
        
        let mut resources = self.resources.write().await;
        resources.retain(|_, resource| {
            now.signed_duration_since(resource.created_at()) < retention_period
        });
        
        Ok(())
    }
}

// Resource pooling for frequently created/destroyed objects
trait ResourcePool: Send + Sync {
    async fn acquire(&self, id: &str) -> Result<Arc<dyn Resource>, PoolError>;
    async fn release(&self, id: &str) -> Result<(), PoolError>;
    fn pool_size(&self) -> usize;
    fn active_count(&self) -> usize;
}

struct BaseAgentPool {
    pool: Arc<RwLock<Vec<Arc<dyn BaseAgent>>>>,
    active: Arc<RwLock<HashMap<String, Arc<dyn BaseAgent>>>>,
    max_size: usize,
}

impl ResourcePool for BaseAgentPool {
    async fn acquire(&self, id: &str) -> Result<Arc<dyn Resource>, PoolError> {
        let mut pool = self.pool.write().await;
        let mut active = self.active.write().await;
        
        if let Some(agent) = pool.pop() {
            // Reset agent state
            agent.reset().await?;
            active.insert(id.to_string(), Arc::clone(&agent));
            Ok(agent as Arc<dyn Resource>)
        } else if active.len() < self.max_size {
            // Create new agent if pool is empty but under limit
            let agent = Arc::new(BaseAgentImpl::new_pooled()?);
            active.insert(id.to_string(), Arc::clone(&agent));
            Ok(agent as Arc<dyn Resource>)
        } else {
            Err(PoolError::PoolExhausted)
        }
    }
    
    async fn release(&self, id: &str) -> Result<(), PoolError> {
        let mut pool = self.pool.write().await;
        let mut active = self.active.write().await;
        
        if let Some(agent) = active.remove(id) {
            if pool.len() < self.max_size {
                pool.push(agent);
            }
            // Agent is dropped if pool is full
        }
        
        Ok(())
    }
}
```

### Async Operation Optimization

```rust
// Efficient streaming with backpressure handling
pub struct StreamManager {
    active_streams: Arc<RwLock<HashMap<String, StreamContext>>>,
    buffer_size: usize,
    max_concurrent_streams: usize,
}

impl StreamManager {
    pub async fn create_agent_stream(&self, agent_id: &str, input: &str) -> Result<StreamHandle, StreamError> {
        // Check concurrent stream limit
        let active_count = self.active_streams.read().await.len();
        if active_count >= self.max_concurrent_streams {
            return Err(StreamError::TooManyConcurrentStreams);
        }
        
        let stream_id = format!("stream_{}_{}", agent_id, Utc::now().timestamp_nanos());
        let (sender, receiver) = mpsc::channel(self.buffer_size);
        
        let context = StreamContext {
            sender: sender.clone(),
            receiver: Some(receiver),
            metadata: StreamMetadata {
                created_at: Utc::now(),
                stream_type: StreamType::Agent,
                source_id: agent_id.to_string(),
            },
            cancellation_token: CancellationToken::new(),
        };
        
        // Store context
        self.active_streams.write().await.insert(stream_id.clone(), context);
        
        // Start streaming task with proper error handling
        let agent = AGENT_REGISTRY.get(agent_id)?.clone();
        let input_clone = input.to_string();
        let stream_id_clone = stream_id.clone();
        let streams_ref = Arc::clone(&self.active_streams);
        
        tokio::spawn(async move {
            let result = Self::run_stream_task(agent, input_clone, sender).await;
            
            // Clean up stream on completion/error
            streams_ref.write().await.remove(&stream_id_clone);
            
            if let Err(e) = result {
                eprintln!("Stream {} failed: {}", stream_id_clone, e);
            }
        });
        
        Ok(StreamHandle {
            id: stream_id,
            stream_type: StreamType::Agent,
            created_at: Utc::now(),
        })
    }
    
    async fn run_stream_task(
        agent: Arc<dyn Agent>,
        input: String,
        sender: mpsc::Sender<ScriptValue>
    ) -> Result<(), StreamError> {
        let mut stream = agent.run_stream(&input).await?;
        
        while let Some(chunk) = stream.next().await {
            match chunk {
                Ok(agent_chunk) => {
                    let script_value = convert_agent_chunk_to_script_value(agent_chunk)?;
                    
                    // Send with timeout to prevent hanging
                    match timeout(Duration::from_secs(5), sender.send(script_value)).await {
                        Ok(Ok(())) => continue,
                        Ok(Err(_)) => break, // Channel closed
                        Err(_) => return Err(StreamError::SendTimeout),
                    }
                }
                Err(e) => {
                    let error_value = ScriptValue::Error(ScriptError::from(e));
                    let _ = sender.send(error_value).await;
                    return Err(StreamError::AgentError(e));
                }
            }
        }
        
        Ok(())
    }
}
```

## Testing Strategy Updates

### Integration Testing for Bridge Layer

```rust
// Comprehensive integration tests for bridge layer
#[cfg(test)]
mod bridge_integration_tests {
    use super::*;
    use tokio_test;
    
    #[tokio::test]
    async fn test_base_agent_bridge_lifecycle() {
        let registry = BridgeRegistry::new();
        let base_agent_bridge = registry.get_bridge("base_agent").unwrap();
        
        // Test 1: Create base agent
        let create_args = vec![
            ScriptValue::Object({
                let mut config = HashMap::new();
                config.insert("id".to_string(), ScriptValue::String("test_agent".to_string()));
                config.insert("name".to_string(), ScriptValue::String("Test Agent".to_string()));
                config.insert("description".to_string(), ScriptValue::String("A test agent".to_string()));
                config
            })
        ];
        
        let result = base_agent_bridge.execute_method("create", create_args).await;
        assert!(result.is_ok());
        
        let agent_handle = match result.unwrap() {
            ScriptValue::ResourceHandle(handle) => handle,
            _ => panic!("Expected ResourceHandle")
        };
        
        // Test 2: Add tool to agent
        let add_tool_args = vec![
            ScriptValue::ResourceHandle(agent_handle.clone()),
            ScriptValue::String("web_search".to_string())
        ];
        
        let result = base_agent_bridge.execute_method("add_tool", add_tool_args).await;
        assert!(result.is_ok());
        
        // Test 3: List agent tools
        let list_tools_args = vec![ScriptValue::ResourceHandle(agent_handle.clone())];
        let result = base_agent_bridge.execute_method("list_tools", list_tools_args).await;
        assert!(result.is_ok());
        
        if let ScriptValue::Array(tools) = result.unwrap() {
            assert_eq!(tools.len(), 1);
            assert_eq!(tools[0], ScriptValue::String("web_search".to_string()));
        } else {
            panic!("Expected array of tools");
        }
        
        // Test 4: Agent state management
        let state_config = ScriptValue::Object({
            let mut state = HashMap::new();
            state.insert("test_key".to_string(), ScriptValue::String("test_value".to_string()));
            state
        });
        
        let set_state_args = vec![
            ScriptValue::ResourceHandle(agent_handle.clone()),
            state_config
        ];
        
        let result = base_agent_bridge.execute_method("set_state", set_state_args).await;
        assert!(result.is_ok());
        
        // Test 5: Get agent state
        let get_state_args = vec![ScriptValue::ResourceHandle(agent_handle)];
        let result = base_agent_bridge.execute_method("get_state", get_state_args).await;
        assert!(result.is_ok());
        
        if let ScriptValue::Object(state) = result.unwrap() {
            assert_eq!(state.get("test_key"), Some(&ScriptValue::String("test_value".to_string())));
        } else {
            panic!("Expected state object");
        }
    }
    
    #[tokio::test]
    async fn test_agent_bridge_creation_and_execution() {
        let registry = BridgeRegistry::new();
        let base_agent_bridge = registry.get_bridge("base_agent").unwrap();
        let agent_bridge = registry.get_bridge("agent").unwrap();
        
        // Create base agent first
        let base_agent_handle = create_test_base_agent(&base_agent_bridge).await;
        
        // Create LLM agent from base
        let llm_config = ScriptValue::Object({
            let mut config = HashMap::new();
            config.insert("model".to_string(), ScriptValue::String("gpt-3.5-turbo".to_string()));
            config.insert("temperature".to_string(), ScriptValue::Number(0.7));
            config.insert("system".to_string(), ScriptValue::String("You are helpful".to_string()));
            config
        });
        
        let create_agent_args = vec![
            ScriptValue::ResourceHandle(base_agent_handle),
            llm_config
        ];
        
        let result = agent_bridge.execute_method("create_from_base", create_agent_args).await;
        assert!(result.is_ok());
        
        let agent_handle = match result.unwrap() {
            ScriptValue::ResourceHandle(handle) => handle,
            _ => panic!("Expected ResourceHandle")
        };
        
        // Test agent execution
        let run_args = vec![
            ScriptValue::ResourceHandle(agent_handle),
            ScriptValue::String("Hello, world!".to_string())
        ];
        
        let result = agent_bridge.execute_method("run", run_args).await;
        assert!(result.is_ok());
        
        // Verify response structure
        if let ScriptValue::Object(response) = result.unwrap() {
            assert!(response.contains_key("content"));
            assert!(response.contains_key("model"));
            assert!(response.contains_key("tokens_used"));
        } else {
            panic!("Expected response object");
        }
    }
    
    #[tokio::test]
    async fn test_tool_wrapped_agent_bridge() {
        let registry = BridgeRegistry::new();
        let base_agent_bridge = registry.get_bridge("base_agent").unwrap();
        let agent_bridge = registry.get_bridge("agent").unwrap();
        let tool_bridge = registry.get_bridge("tool").unwrap();
        
        // Create specialized agent
        let agent_handle = create_test_llm_agent(&base_agent_bridge, &agent_bridge).await;
        
        // Wrap agent as tool
        let tool_config = ScriptValue::Object({
            let mut config = HashMap::new();
            config.insert("name".to_string(), ScriptValue::String("code_reviewer".to_string()));
            config.insert("description".to_string(), ScriptValue::String("Reviews code".to_string()));
            config.insert("parameters".to_string(), ScriptValue::Object({
                let mut params = HashMap::new();
                params.insert("code".to_string(), ScriptValue::Object({
                    let mut param = HashMap::new();
                    param.insert("type".to_string(), ScriptValue::String("string".to_string()));
                    param.insert("description".to_string(), ScriptValue::String("Code to review".to_string()));
                    param
                }));
                params
            }));
            config
        });
        
        let wrap_args = vec![
            ScriptValue::ResourceHandle(agent_handle),
            tool_config
        ];
        
        let result = tool_bridge.execute_method("wrap_agent", wrap_args).await;
        assert!(result.is_ok());
        
        let tool_handle = match result.unwrap() {
            ScriptValue::ResourceHandle(handle) => handle,
            _ => panic!("Expected ResourceHandle")
        };
        
        // Test wrapped tool execution
        let execute_args = vec![
            ScriptValue::String("code_reviewer".to_string()),
            ScriptValue::Object({
                let mut input = HashMap::new();
                input.insert("code".to_string(), ScriptValue::String("fn main() { println!(\"Hello\"); }".to_string()));
                input
            }),
            ScriptValue::Object(HashMap::new()) // empty context
        ];
        
        let result = tool_bridge.execute_method("execute", execute_args).await;
        assert!(result.is_ok());
        
        // Verify tool execution result
        if let ScriptValue::Object(result) = result.unwrap() {
            assert!(result.contains_key("output"));
        } else {
            panic!("Expected tool result object");
        }
    }
    
    // Helper functions
    async fn create_test_base_agent(bridge: &dyn Bridge) -> ResourceHandle {
        let args = vec![
            ScriptValue::Object({
                let mut config = HashMap::new();
                config.insert("id".to_string(), ScriptValue::String("test_base".to_string()));
                config.insert("name".to_string(), ScriptValue::String("Test Base".to_string()));
                config
            })
        ];
        
        match bridge.execute_method("create", args).await.unwrap() {
            ScriptValue::ResourceHandle(handle) => handle,
            _ => panic!("Expected ResourceHandle")
        }
    }
    
    async fn create_test_llm_agent(base_bridge: &dyn Bridge, agent_bridge: &dyn Bridge) -> ResourceHandle {
        let base_handle = create_test_base_agent(base_bridge).await;
        
        let llm_config = ScriptValue::Object({
            let mut config = HashMap::new();
            config.insert("model".to_string(), ScriptValue::String("gpt-3.5-turbo".to_string()));
            config.insert("system".to_string(), ScriptValue::String("You are helpful".to_string()));
            config
        });
        
        let args = vec![
            ScriptValue::ResourceHandle(base_handle),
            llm_config
        ];
        
        match agent_bridge.execute_method("create_from_base", args).await.unwrap() {
            ScriptValue::ResourceHandle(handle) => handle,
            _ => panic!("Expected ResourceHandle")
        }
    }
}

// Cross-engine compatibility tests
#[cfg(test)]
mod cross_engine_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_lua_javascript_bridge_compatibility() {
        let registry = BridgeRegistry::new();
        
        // Test same operations in both engines
        let lua_engine = LuaEngine::new();
        let js_engine = JavaScriptEngine::new();
        
        // Register bridges in both engines
        lua_engine.register_bridges(&registry).await.unwrap();
        js_engine.register_bridges(&registry).await.unwrap();
        
        // Lua script test
        let lua_script = r#"
            local base = base_agent.create({
                id = "test_agent",
                name = "Test Agent"
            })
            
            base:add_tool("web_search")
            
            local agent = agent.create_from_base(base, {
                model = "gpt-3.5-turbo",
                system = "You are helpful"
            })
            
            local response = agent:run("Hello!")
            return response.content
        "#;
        
        // JavaScript equivalent
        let js_script = r#"
            const base = rsLlmspell.baseAgent.create({
                id: "test_agent",
                name: "Test Agent"
            });
            
            await base.addTool("web_search");
            
            const agent = await rsLlmspell.agent.createFromBase(base, {
                model: "gpt-3.5-turbo",
                system: "You are helpful"
            });
            
            const response = await agent.run("Hello!");
            return response.content;
        "#;
        
        // Execute and compare results
        let lua_result = lua_engine.execute(lua_script).await.unwrap();
        let js_result = js_engine.execute(js_script).await.unwrap();
        
        // Both should produce similar responses (content will differ but structure should be same)
        match (lua_result, js_result) {
            (ScriptValue::String(lua_content), ScriptValue::String(js_content)) => {
                assert!(!lua_content.is_empty());
                assert!(!js_content.is_empty());
                // Both engines successfully executed the same logical operations
            }
            _ => panic!("Both engines should return string content")
        }
    }
}
```

## Migration Path

### Backward Compatibility Strategy

```rust
// Legacy bridge wrapper for existing scripts
pub struct LegacyBridgeAdapter {
    new_bridges: Arc<BridgeRegistry>,
    legacy_mappings: HashMap<String, LegacyMapping>,
}

struct LegacyMapping {
    old_method: String,
    new_bridge: String,
    new_method: String,
    arg_transformer: Box<dyn Fn(Vec<ScriptValue>) -> Vec<ScriptValue> + Send + Sync>,
}

impl Bridge for LegacyBridgeAdapter {
    async fn execute_method(&self, name: &str, args: Vec<ScriptValue>) -> Result<ScriptValue, BridgeError> {
        if let Some(mapping) = self.legacy_mappings.get(name) {
            // Transform arguments for new bridge
            let new_args = (mapping.arg_transformer)(args);
            
            // Execute on new bridge
            let new_bridge = self.new_bridges.get_bridge(&mapping.new_bridge)
                .ok_or(BridgeError::BridgeNotFound(mapping.new_bridge.clone()))?;
            
            new_bridge.execute_method(&mapping.new_method, new_args).await
        } else {
            Err(BridgeError::UnknownMethod(format!("Legacy method '{}' not supported", name)))
        }
    }
}

impl LegacyBridgeAdapter {
    pub fn new(new_bridges: Arc<BridgeRegistry>) -> Self {
        let mut mappings = HashMap::new();
        
        // Map legacy agent creation to new pattern
        mappings.insert("create_agent".to_string(), LegacyMapping {
            old_method: "create_agent".to_string(),
            new_bridge: "base_agent".to_string(),
            new_method: "create".to_string(),
            arg_transformer: Box::new(|args| {
                // Transform legacy config to new base agent + llm agent pattern
                if let Some(ScriptValue::Object(config)) = args.get(0) {
                    // Extract tools and create base agent config
                    let mut base_config = HashMap::new();
                    base_config.insert("id".to_string(), 
                        config.get("id")
                            .cloned()
                            .unwrap_or(ScriptValue::String(format!("legacy_{}", uuid::Uuid::new_v4()))));
                    base_config.insert("name".to_string(),
                        config.get("name")
                            .cloned()
                            .unwrap_or(ScriptValue::String("Legacy Agent".to_string())));
                    
                    if let Some(tools) = config.get("tools") {
                        base_config.insert("tools".to_string(), tools.clone());
                    }
                    
                    vec![ScriptValue::Object(base_config)]
                } else {
                    args
                }
            }),
        });
        
        // Map legacy agent run to new pattern
        mappings.insert("run_agent".to_string(), LegacyMapping {
            old_method: "run_agent".to_string(),
            new_bridge: "agent".to_string(),
            new_method: "run".to_string(),
            arg_transformer: Box::new(|args| args), // Direct mapping
        });
        
        Self {
            new_bridges,
            legacy_mappings: mappings,
        }
    }
}
```

### Migration Script Generator

```rust
// Automated migration tool for existing scripts
pub struct BridgeMigrationTool {
    patterns: Vec<MigrationPattern>,
    dependency_graph: DependencyGraph,
}

struct MigrationPattern {
    from_pattern: Regex,
    to_template: String,
    transformation_type: TransformationType,
    requires_manual_review: bool,
}

enum TransformationType {
    DirectReplacement,
    StructuralChange,
    PatternSplit, // One old pattern becomes multiple new patterns
    PatternMerge, // Multiple old patterns become one new pattern
}

impl BridgeMigrationTool {
    pub fn analyze_script(&self, script: &str) -> MigrationAnalysis {
        let mut analysis = MigrationAnalysis::new();
        
        // Detect legacy patterns
        for pattern in &self.patterns {
            for capture in pattern.from_pattern.captures_iter(script) {
                analysis.add_legacy_usage(LegacyUsage {
                    pattern: pattern.from_pattern.as_str().to_string(),
                    location: capture.get(0).unwrap().range(),
                    complexity: self.assess_transformation_complexity(pattern),
                    requires_manual_review: pattern.requires_manual_review,
                });
            }
        }
        
        // Analyze dependencies
        analysis.dependencies = self.dependency_graph.analyze_script_dependencies(script);
        
        analysis
    }
    
    pub fn generate_migration(&self, script: &str) -> MigrationResult {
        let analysis = self.analyze_script(script);
        let mut migrated_script = script.to_string();
        let mut applied_transformations = Vec::new();
        
        // Apply transformations in dependency order
        for usage in analysis.legacy_usages.iter() {
            if let Some(pattern) = self.patterns.iter().find(|p| p.from_pattern.as_str() == usage.pattern) {
                match pattern.transformation_type {
                    TransformationType::DirectReplacement => {
                        migrated_script = pattern.from_pattern.replace_all(&migrated_script, &pattern.to_template).to_string();
                        applied_transformations.push(AppliedTransformation {
                            transformation_type: pattern.transformation_type.clone(),
                            from: pattern.from_pattern.as_str().to_string(),
                            to: pattern.to_template.clone(),
                        });
                    },
                    TransformationType::StructuralChange => {
                        // More complex transformation requiring context analysis
                        let transformed = self.apply_structural_transformation(&migrated_script, pattern)?;
                        migrated_script = transformed;
                        applied_transformations.push(AppliedTransformation {
                            transformation_type: pattern.transformation_type.clone(),
                            from: pattern.from_pattern.as_str().to_string(),
                            to: pattern.to_template.clone(),
                        });
                    },
                    _ => {
                        // Mark for manual review
                        applied_transformations.push(AppliedTransformation {
                            transformation_type: pattern.transformation_type.clone(),
                            from: pattern.from_pattern.as_str().to_string(),
                            to: "MANUAL_REVIEW_REQUIRED".to_string(),
                        });
                    }
                }
            }
        }
        
        MigrationResult {
            original_script: script.to_string(),
            migrated_script,
            analysis,
            applied_transformations,
            manual_review_items: analysis.legacy_usages.iter()
                .filter(|u| u.requires_manual_review)
                .cloned()
                .collect(),
        }
    }
}

// Example migration patterns
fn create_migration_patterns() -> Vec<MigrationPattern> {
    vec![
        // Simple agent creation migration
        MigrationPattern {
            from_pattern: Regex::new(r"llm\.agent\((\{[^}]+\})\)").unwrap(),
            to_template: r"
-- Migrated: Split into base agent + LLM agent
local base = base_agent.create({
    id = 'migrated_' .. os.time(),
    name = 'Migrated Agent'
})

-- Add tools if specified in original config
$TOOLS_ADDITION$

local agent = agent.create_from_base(base, $1)
".to_string(),
            transformation_type: TransformationType::StructuralChange,
            requires_manual_review: true,
        },
        
        // Direct method call migration
        MigrationPattern {
            from_pattern: Regex::new(r"agent:run\(([^)]+)\)").unwrap(),
            to_template: r"agent:run($1)".to_string(),
            transformation_type: TransformationType::DirectReplacement,
            requires_manual_review: false,
        },
        
        // Tool execution migration
        MigrationPattern {
            from_pattern: Regex::new(r"tool\.execute\(([^,]+),\s*([^)]+)\)").unwrap(),
            to_template: r"tool.execute($1, $2, {})".to_string(), // Add empty context
            transformation_type: TransformationType::DirectReplacement,
            requires_manual_review: false,
        },
    ]
}
```

## Implementation Roadmap

### Phase 1: Core Bridge Infrastructure (Weeks 1-2)

1. **Resource Management System**
   - Implement `ResourceManager` with handle-based resource tracking
   - Create resource pools for common types (BaseAgent, Tool)
   - Add cleanup and lifecycle management

2. **Bridge Registry**
   - Implement modular bridge system
   - Create bridge discovery and registration
   - Add dependency management between bridges

3. **Type System Updates**
   - Extend `ScriptValue` enum for new types
   - Implement comprehensive type conversion utilities
   - Add validation and error handling

### Phase 2: Specialized Bridges (Weeks 3-4)

1. **BaseAgent Bridge**
   - Implement creation, tool management, hook integration
   - Add state management operations
   - Create comprehensive error handling

2. **Agent Bridge**
   - Implement LLM agent creation from base agents
   - Add execution methods (run, stream)
   - Integrate with conversation history management

3. **Tool Bridge**
   - Implement tool discovery and execution
   - Add agent wrapping functionality
   - Create tool composition patterns

### Phase 3: Advanced Features (Weeks 5-6)

1. **Workflow Bridge**
   - Implement workflow creation and execution
   - Add step management and composition
   - Create workflow templates

2. **State and Hook Bridges**
   - Implement state management operations
   - Add hook registration and execution
   - Create event system integration

3. **Stream Management**
   - Implement streaming operations
   - Add backpressure handling
   - Create stream lifecycle management

### Phase 4: Script Engine Integration (Weeks 7-8)

1. **Lua Engine Updates**
   - Register new bridge modules
   - Create userdata types for resource handles
   - Add method injection for resource types

2. **JavaScript Engine Updates**
   - Create ES6 modules for bridge access
   - Implement Promise-based async operations
   - Add prototype methods for resource types

3. **Cross-Engine Testing**
   - Implement compatibility test suite
   - Verify consistent behavior across engines
   - Test error handling and edge cases

### Phase 5: Migration and Optimization (Weeks 9-10)

1. **Legacy Compatibility**
   - Implement legacy bridge adapter
   - Create migration tool and patterns
   - Add automated script analysis

2. **Performance Optimization**
   - Optimize resource pooling and lifecycle
   - Implement efficient streaming
   - Add performance monitoring and metrics

3. **Documentation and Examples**
   - Update architecture documentation
   - Create comprehensive examples
   - Write migration guide

## Conclusion

The bridge integration analysis reveals significant changes required to support the new BaseAgent/Agent/Tool/Workflow hierarchy, but these changes unlock powerful new capabilities:

### Key Benefits

1. **Composable Architecture**: Script-level composition of agents, tools, and workflows
2. **Resource Management**: Efficient lifecycle management with pooling and cleanup
3. **Type Safety**: Enhanced type system with comprehensive error handling
4. **Performance**: Optimized streaming and async operations
5. **Extensibility**: Modular bridge system for easy extension

### Breaking Changes

1. **API Structure**: Move from flat method calls to hierarchical object management
2. **Resource Lifecycle**: Explicit resource creation and management
3. **Type System**: New handle-based resource references
4. **Error Handling**: Enhanced error types and context

### Migration Strategy

1. **Compatibility Layer**: Legacy bridge adapter for existing scripts
2. **Migration Tools**: Automated analysis and transformation
3. **Incremental Adoption**: Support both old and new APIs during transition
4. **Documentation**: Comprehensive guides and examples

The new bridge architecture provides a solid foundation for the enhanced rs-llmspell capabilities while maintaining the core principle of scriptable LLM orchestration.