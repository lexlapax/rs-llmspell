# Rs-LLMSpell Architecture Mapping Analysis

## Current State vs. Target Architecture

### Current rs-llmspell Concepts
```
Current Architecture (Bridge-First):
├── ScriptEngine trait - Script execution
├── Bridge trait - Rust library exposure  
├── Agent trait - High-level agent behavior
├── Tool trait - Agent capability extensions
└── Workflow trait - Multi-agent orchestration
```

### Target Architecture (Based on go-llms/ADK)
```
Target Architecture (Agent-First with Bridge Layer):
├── BaseAgent trait - Fundamental tool-handling core
├── Agent trait - LLM wrapper with specialized prompts
├── Tool trait - LLM-callable functions
├── Workflow types - Deterministic agent orchestration
│   ├── SequentialWorkflow
│   ├── ParallelWorkflow  
│   ├── ConditionalWorkflow
│   └── LoopWorkflow
├── ToolWrappedAgent - Agents usable as tools
├── Hook System - Pre/post execution hooks
├── Event System - Emit/publish/subscribe
└── Built-in Components - Ready-to-use tools and agents
```

## Key Gaps Identified

### 1. Missing BaseAgent Foundation
**Problem**: Current Agent trait is too high-level
**Solution**: Need BaseAgent trait that provides fundamental tool-handling capabilities

### 2. No Tool-Wrapped Agent Pattern
**Problem**: Agents cannot be used as tools by other agents
**Solution**: ToolWrappedAgent that exposes Agent as Tool interface

### 3. Missing Hooks and Events System
**Problem**: No instrumentation, logging, or debugging capabilities
**Solution**: Comprehensive hook system with event publishing

### 4. Generic Workflow vs. Specific Types
**Problem**: Single Workflow trait instead of specific orchestration patterns
**Solution**: Specific workflow types with clear semantics

### 5. No Built-in Component Strategy
**Problem**: No ready-to-use tools or agents
**Solution**: Built-in component library with 30-40 tools and common agents

## Component Relationship Mapping

### go-llms/ADK Pattern → rs-llmspell Implementation

| go-llms/ADK Concept | Current rs-llmspell | Target rs-llmspell |
|---------------------|--------------------|--------------------|
| BaseAgent | Missing | BaseAgent trait |
| LLM Agent | Agent trait | Agent trait (refined) |
| Tool System | Tool trait | Tool trait + ToolWrappedAgent |
| Sequential Workflow | Workflow trait | SequentialWorkflow |
| Parallel Workflow | Missing | ParallelWorkflow |
| Loop Workflow | Missing | LoopWorkflow |
| Conditional Workflow | Missing | ConditionalWorkflow |
| Hook System | Missing | Hook trait + built-ins |
| Event System | Missing | Event emit/subscribe |
| Built-in Tools | Missing | BuiltinTool implementations |
| Multi-agent Systems | Partial | Enhanced with tool-wrapped agents |

## Script Interface Implications

### Current Script API
```lua
-- Current: Direct LLM access
local response = llm.complete({...})

-- Current: Basic agent
local agent = llm.agent({...})
```

### Target Script API  
```lua
-- Target: BaseAgent with tools
local base_agent = agent.base()
base_agent:add_tool("web_search")
base_agent:add_tool("file_write")

-- Target: Specialized agents
local researcher = agent.llm({
    model = "gpt-4",
    system = "You are a researcher",
    tools = {"web_search", "file_write"}
})

-- Target: Tool-wrapped agents
local research_tool = agent.as_tool(researcher, {
    name = "research_assistant",
    description = "Performs research tasks"
})

-- Target: Workflows
local workflow = workflow.sequential()
    :add_step("research", researcher)
    :add_step("summarize", summarizer)

-- Target: Hooks
agent.add_hook("pre_llm", function(context) 
    log.info("Starting LLM call: " .. context.prompt)
end)

-- Target: Events
events.on("agent_started", function(event)
    metrics.increment("agent_executions")
end)
```

## Bridge Layer Impact

### Current Bridge Structure
```rust
pub trait Bridge {
    fn methods(&self) -> Vec<MethodInfo>;
    async fn execute_method(&self, name: &str, args: Vec<ScriptValue>) -> Result<ScriptValue>;
}
```

### Enhanced Bridge Structure
```rust
pub trait Bridge {
    // Existing methods
    fn methods(&self) -> Vec<MethodInfo>;
    async fn execute_method(&self, name: &str, args: Vec<ScriptValue>) -> Result<ScriptValue>;
    
    // New: Hook support
    fn hook_points(&self) -> Vec<HookPoint>;
    fn register_hook(&mut self, point: HookPoint, hook: Box<dyn Hook>);
    
    // New: Event emission
    fn emit_event(&self, event: Event);
}

// New: Specific bridge types
pub trait AgentBridge: Bridge {
    fn create_base_agent(&self) -> Box<dyn BaseAgent>;
    fn create_llm_agent(&self, config: LlmAgentConfig) -> Box<dyn Agent>;
    fn wrap_agent_as_tool(&self, agent: Box<dyn Agent>) -> Box<dyn Tool>;
}

pub trait ToolBridge: Bridge {
    fn builtin_tools(&self) -> Vec<Box<dyn Tool>>;
    fn create_custom_tool(&self, spec: ToolSpec) -> Box<dyn Tool>;
}

pub trait WorkflowBridge: Bridge {
    fn create_sequential_workflow(&self) -> Box<dyn SequentialWorkflow>;
    fn create_parallel_workflow(&self) -> Box<dyn ParallelWorkflow>;
    fn create_conditional_workflow(&self) -> Box<dyn ConditionalWorkflow>;
    fn create_loop_workflow(&self) -> Box<dyn LoopWorkflow>;
}
```

## Testing Strategy Impact

### New Testing Requirements
1. **BaseAgent Contract Tests**: Ensure all BaseAgent implementations handle tools correctly
2. **Tool-Wrapped Agent Tests**: Validate agents work correctly when wrapped as tools
3. **Workflow Type Tests**: Specific tests for each workflow type's semantics
4. **Hook System Tests**: Validate hook execution at all points
5. **Event System Tests**: Test event emission and subscription
6. **Built-in Component Tests**: Comprehensive tests for all built-in tools and agents
7. **Cross-Engine Hook Tests**: Ensure hooks work identically across Lua/JavaScript

### Enhanced Test Fixtures
```rust
pub struct TestFixtures {
    // Existing
    pub scripts: ScriptFixtures,
    pub mock_responses: MockResponseFixtures,
    
    // New
    pub hook_scenarios: HookTestScenarios,
    pub event_scenarios: EventTestScenarios,
    pub workflow_scenarios: WorkflowTestScenarios,
    pub builtin_tool_tests: BuiltinToolTests,
    pub agent_composition_tests: AgentCompositionTests,
}
```

## Implementation Priority

### Phase 1: Core Foundation
1. BaseAgent trait
2. Refined Agent trait
3. ToolWrappedAgent implementation
4. Basic hook system

### Phase 2: Workflow Types  
1. SequentialWorkflow
2. ParallelWorkflow
3. ConditionalWorkflow
4. LoopWorkflow

### Phase 3: Events and Built-ins
1. Event system implementation
2. Built-in tool library (30-40 tools)
3. Built-in agent patterns
4. Advanced hook system

### Phase 4: Integration
1. Enhanced bridge layer
2. Script API updates
3. Testing infrastructure
4. Documentation and examples

## Breaking Changes Required

### Script API Changes
- Agent creation API will change significantly
- New workflow creation patterns
- Hook registration APIs
- Event handling APIs

### Bridge Interface Changes
- Additional methods for hooks and events
- Specialized bridge types
- New tool and agent creation methods

### Configuration Changes
- Built-in component configuration
- Hook configuration
- Event system configuration

## Migration Strategy
1. Maintain current API in v1.x branch
2. Implement new architecture in v2.x
3. Provide migration tools and documentation
4. Gradual deprecation with clear timeline