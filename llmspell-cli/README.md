# llmspell-cli

Command-line interface for Rs-LLMSpell - scriptable LLM interactions via Lua and JavaScript.

## Features
- Interactive REPL for Lua/JavaScript scripting with LLMs
- Execute script files with agent and workflow support
- Built-in library access with 40+ tools and templates

## Usage
```bash
llmspell repl --language lua        # Start Lua REPL
llmspell run script.lua             # Execute script file
llmspell list-tools                 # Show available tools
```

## Dependencies
- `llmspell-core` - Core traits and types
- `llmspell-bridge` - Script engine integration
- `llmspell-agents` - Agent implementations
- `llmspell-workflows` - Workflow orchestration