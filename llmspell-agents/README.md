# llmspell-agents

Production-ready agent infrastructure for rs-llmspell with lifecycle management, state persistence, and comprehensive hook integration.

## Overview

This crate provides the complete agent implementation for rs-llmspell, including:

- **Agent Factory & Registry**: Centralized agent creation and discovery
- **8 Agent Templates**: Research, Analysis, Writing, Code, QA, Domain, Tool-Orchestrator, Basic
- **Lifecycle Management**: 7 states with hook integration at each transition
- **State Persistence**: Automatic state saving on pause/stop (Phase 5)
- **Tool Integration**: Agents can discover and invoke any registered tool
- **Agent Composition**: Agents as tools, multi-agent coordination

## Features

### Agent Templates

1. **ResearchAgent** - Web research with source tracking
2. **AnalysisAgent** - Data analysis and insights
3. **WritingAgent** - Content creation and editing
4. **CodeAgent** - Programming and code generation
5. **QAAgent** - Quality assurance and testing
6. **DomainAgent** - Domain-specific expertise
7. **ToolOrchestratorAgent** - Coordinates multiple tools
8. **BasicAgent** - Simple LLM wrapper

### Lifecycle States

```rust
pub enum AgentState {
    Created,      // Initial state
    Initialized,  // Configuration loaded
    Ready,        // Ready to execute
    Running,      // Currently executing
    Paused,       // Temporarily suspended (state saved)
    Stopped,      // Terminated (state saved)
    Failed,       // Error state
}
```

### State Persistence (Phase 5)

Agents automatically persist their state:

```rust
use llmspell_agents::{ResearchAgent, AgentBuilder};
use llmspell_state_persistence::StateScope;

// Create agent with state persistence
let agent = ResearchAgent::builder()
    .with_id("research-bot-1")
    .with_provider(provider)
    .with_state_persistence(true)
    .build()?;

// Execute - state is tracked
agent.execute("Research quantum computing").await?;

// Pause - state automatically saved
agent.pause().await?;

// Resume - state automatically restored
agent.resume().await?;

// Manual state operations
let state = agent.get_state().await?;
agent.save_state().await?;
agent.load_state().await?;
```

### Hook Integration

Every lifecycle transition triggers hooks:

```rust
// Register lifecycle hooks
hook_manager.register("agent:before_creation", creation_hook)?;
hook_manager.register("agent:after_initialization", init_hook)?;
hook_manager.register("agent:before_execution", exec_hook)?;
hook_manager.register("agent:after_execution", completion_hook)?;
hook_manager.register("agent:before_pause", pause_hook)?;
hook_manager.register("agent:after_stop", cleanup_hook)?;
```

### Agent Factory

```rust
use llmspell_agents::{AgentFactory, AgentType};

let factory = AgentFactory::new();

// Create from template
let agent = factory.create(
    AgentType::Research,
    json!({
        "provider": "openai",
        "model": "gpt-4",
        "tools": ["web_search", "calculator"]
    })
).await?;

// Create custom agent
let custom = factory.create_custom(
    "domain-expert",
    json!({
        "base_type": "domain",
        "domain": "quantum-physics",
        "provider": "anthropic",
        "model": "claude-3"
    })
).await?;
```

### Agent Registry

```rust
use llmspell_agents::AgentRegistry;

let registry = AgentRegistry::new();

// Register agent
registry.register("research-1", agent).await?;

// Discover agents
let agents = registry.list_by_capability("web_search").await?;
let research_agents = registry.list_by_type(AgentType::Research).await?;

// Get specific agent
let agent = registry.get("research-1").await?;
```

### Tool Integration

Agents can discover and use tools:

```rust
// Agent with tool discovery
let agent = ToolOrchestratorAgent::builder()
    .with_tool_discovery(true)
    .build()?;

// Execute with automatic tool selection
let result = agent.execute(
    "Search for quantum computing papers and summarize them"
).await?;

// Manual tool invocation
let tool_result = agent.invoke_tool(
    "web_search",
    json!({"query": "quantum computing arxiv"})
).await?;
```

### Multi-Agent Coordination

```rust
use llmspell_agents::{AgentCoordinator, CoordinationPattern};

let coordinator = AgentCoordinator::new();

// Add agents
coordinator.add_agent("researcher", research_agent)?;
coordinator.add_agent("analyst", analysis_agent)?;
coordinator.add_agent("writer", writing_agent)?;

// Execute with coordination pattern
let result = coordinator.execute(
    CoordinationPattern::Sequential,
    vec![
        ("researcher", "Find recent AI breakthroughs"),
        ("analyst", "Analyze the findings"),
        ("writer", "Write a summary report")
    ]
).await?;
```

## Architecture

The crate is organized into:

- **Core** (`agent.rs`, `base_agent.rs`) - Agent trait implementations
- **Factory** (`factory.rs`, `registry.rs`) - Agent creation and discovery
- **Templates** (`templates/`) - Pre-built agent types
- **Lifecycle** (`lifecycle.rs`, `state_machine.rs`) - State management
- **Persistence** (`persistence.rs`) - State saving/loading
- **Coordination** (`coordinator.rs`) - Multi-agent patterns
- **Integration** (`tool_integration.rs`) - Tool discovery and invocation

## Performance

Achieved performance metrics (v0.5.0):

| Operation | Target | Actual |
|-----------|--------|--------|
| Agent Creation | <50ms | <50ms |
| State Save | <5ms | <5ms |
| State Load | <5ms | <5ms |
| Tool Discovery | <10ms | <8ms |
| Hook Overhead | <5% | <2% |

## Usage Examples

### Complete Example with State Persistence

```rust
use llmspell_agents::{ResearchAgent, AgentBuilder};
use llmspell_hooks::HookManager;

// Set up hooks
let hooks = HookManager::new();
hooks.register("agent:after_execution", |ctx| {
    println!("Agent {} completed in {}ms", 
        ctx.agent_id, ctx.duration_ms);
    Ok(HookResult::Continue)
})?;

// Create agent with persistence
let agent = ResearchAgent::builder()
    .with_id("research-bot-001")
    .with_provider("openai")
    .with_model("gpt-4")
    .with_system_prompt("You are an expert researcher")
    .with_tools(vec!["web_search", "calculator", "file_reader"])
    .with_state_persistence(true)
    .with_hook_manager(hooks)
    .build()?;

// Check if we have saved state
if agent.has_saved_state().await? {
    agent.load_state().await?;
    println!("Restored agent state");
}

// Execute task
let result = agent.execute(
    "Research the latest developments in quantum computing"
).await?;

// State is automatically saved on pause
agent.pause().await?;

// Later... resume with state
agent.resume().await?;
let followup = agent.execute(
    "What are the practical applications?"
).await?;

// Clean shutdown with state save
agent.stop().await?;
```

### Agent as Tool Pattern

```rust
use llmspell_agents::AgentAsTool;

// Wrap agent as a tool
let research_tool = AgentAsTool::new(
    research_agent,
    "research_assistant",
    "Performs detailed research on any topic"
);

// Register with tool registry
tool_registry.register(Box::new(research_tool))?;

// Now other agents can use it
let orchestrator = ToolOrchestratorAgent::new();
let result = orchestrator.invoke_tool(
    "research_assistant",
    json!({"query": "Latest AI breakthroughs"})
).await?;
```

## Testing

```bash
# Run all tests
cargo test -p llmspell-agents

# Run specific test category
./scripts/test-by-tag.sh agent

# Test with state persistence
RUST_TEST_THREADS=1 cargo test state_persistence

# Benchmark performance
cargo bench -p llmspell-agents
```

## Dependencies

- `llmspell-core` - Core traits (BaseAgent, Agent)
- `llmspell-tools` - Tool implementations
- `llmspell-providers` - LLM provider backends
- `llmspell-state-persistence` - State management
- `llmspell-hooks` - Hook system integration
- `llmspell-events` - Event emission
- `llmspell-utils` - Shared utilities

## License

This project is dual-licensed under MIT OR Apache-2.0.