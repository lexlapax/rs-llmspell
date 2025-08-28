# llmspell-agents

**Agent framework for LLM interactions**

**🔗 Navigation**: [← Rust API](README.md) | [Crate Docs](https://docs.rs/llmspell-agents) | [Source](../../../../llmspell-agents)

---

## Overview

`llmspell-agents` provides the core agent infrastructure for LLM interactions, including agent creation, context management, tool integration, composition patterns, and template-based agent discovery.

**Key Features:**
- 🤖 Flexible agent creation with builders
- 🔧 Tool integration and function calling
- 🎭 Agent composition and delegation
- 📝 Template-based agent creation
- 🧠 Context and memory management
- 🔄 Streaming responses
- 📊 Token usage tracking
- 🎯 Agent discovery and registry

## Core Trait

```rust
#[async_trait]
pub trait Agent: BaseAgent {
    /// Execute with prompt
    async fn execute(&self, input: AgentInput) -> Result<AgentOutput>;
    
    /// Stream execution results
    async fn execute_stream(&self, input: AgentInput) -> Result<AgentStream>;
    
    /// Get agent capabilities
    fn capabilities(&self) -> AgentCapabilities;
    
    /// Clone as trait object
    fn clone_box(&self) -> Box<dyn Agent>;
}
```

## AgentBuilder

```rust
pub struct AgentBuilder {
    name: Option<String>,
    model: Option<String>,
    temperature: Option<f32>,
    max_tokens: Option<u32>,
    tools: Vec<Box<dyn Tool>>,
    system_prompt: Option<String>,
    memory: Option<Box<dyn Memory>>,
}

impl AgentBuilder {
    pub fn new() -> Self { /* ... */ }
    pub fn name(mut self, name: impl Into<String>) -> Self { /* ... */ }
    pub fn model(mut self, model: impl Into<String>) -> Self { /* ... */ }
    pub fn temperature(mut self, temp: f32) -> Self { /* ... */ }
    pub fn with_tool(mut self, tool: Box<dyn Tool>) -> Self { /* ... */ }
    pub fn system_prompt(mut self, prompt: impl Into<String>) -> Self { /* ... */ }
    pub fn build(self) -> Result<Box<dyn Agent>> { /* ... */ }
}
```

## Usage Examples

### Basic Agent Creation

```rust
use llmspell_agents::{AgentBuilder, AgentInput};

let agent = AgentBuilder::new()
    .name("assistant")
    .model("openai/gpt-4")
    .temperature(0.7)
    .system_prompt("You are a helpful assistant")
    .build()?;

let response = agent.execute(AgentInput::from("Hello!")).await?;
println!("{}", response.content);
```

### Agent with Tools

```rust
let calculator = Box::new(CalculatorTool::new());
let web_search = Box::new(WebSearchTool::new());

let agent = AgentBuilder::new()
    .name("research-assistant")
    .model("openai/gpt-4")
    .with_tool(calculator)
    .with_tool(web_search)
    .build()?;

let response = agent.execute(AgentInput::from(
    "What is the square root of 144 and find recent news about it"
)).await?;
```

### Agent Composition

```rust
use llmspell_agents::composition::{DelegatingAgent, AgentRouter};

let analyst = AgentBuilder::new()
    .name("analyst")
    .model("claude-3-opus")
    .build()?;

let writer = AgentBuilder::new()
    .name("writer")
    .model("gpt-4")
    .build()?;

let coordinator = DelegatingAgent::builder()
    .add_agent("analysis", analyst)
    .add_agent("writing", writer)
    .routing_strategy(RoutingStrategy::CapabilityBased)
    .build()?;
```

## Related Documentation

- [llmspell-providers](llmspell-providers.md) - LLM provider implementations
- [llmspell-tools](llmspell-tools.md) - Tool integration
- [llmspell-workflows](llmspell-workflows.md) - Agent orchestration