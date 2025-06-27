# llmspell-bridge

Script engine integration bridge for Rs-LLMSpell.

## Features
- Lua 5.4 integration via mlua with coroutine-based async
- JavaScript support through boa/quickjs engines
- Unified async patterns across scripting languages

## Usage
```rust
use llmspell_bridge::{LuaBridge, ScriptContext};

let bridge = LuaBridge::new()?;
let context = ScriptContext::new()
    .with_agent("research", research_agent)
    .with_tool("web_search", web_search);
bridge.execute(script, context).await?;
```

## Dependencies
- `llmspell-core` - Core types for script bindings
- `llmspell-agents` - Agent exposure to scripts
- `llmspell-tools` - Tool exposure to scripts
- `llmspell-security` - Script sandboxing