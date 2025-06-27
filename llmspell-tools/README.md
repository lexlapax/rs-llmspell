# llmspell-tools

Tool implementations for agent capabilities in Rs-LLMSpell.

## Features
- 40+ built-in tools across 8 categories (web, data, code, system, communication, analysis, utility, domain)
- Tool composition and agent-as-tool wrapper pattern
- Async execution with resource limits and error handling

## Usage
```rust
use llmspell_tools::{WebSearchTool, CalculatorTool};

let search = WebSearchTool::new(api_key);
let result = search.execute(json!({"query": "rust async"})).await?;
```

## Dependencies
- `llmspell-core` - Core Tool trait
- `llmspell-security` - Sandboxing for system tools
- `llmspell-providers` - External API integrations