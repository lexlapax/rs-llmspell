# llmspell-agents

Agent implementations for Rs-LLMSpell framework.

## Features
- Built-in agent templates (Research, Analysis, Writing, Code, QA, Domain)
- Agent composition through tool wrapping pattern
- State management and hook integration

## Usage
```rust
use llmspell_agents::{ResearchAgent, AgentBuilder};

let agent = ResearchAgent::builder()
    .with_provider(provider)
    .with_tools(vec![web_search, calculator])
    .build()?;
```

## Dependencies
- `llmspell-core` - Core traits (BaseAgent, Agent)
- `llmspell-tools` - Tool implementations
- `llmspell-providers` - LLM provider backends