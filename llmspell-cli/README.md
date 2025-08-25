# llmspell-cli

Command-line interface for Rs-LLMSpell - scriptable LLM interactions via Lua and JavaScript.

## Features
- Interactive REPL for Lua/JavaScript scripting with LLMs
- Execute script files with agent and workflow support
- Built-in library access with 40+ tools and templates
- **Embedded Applications**: 7 production-ready applications compiled into the binary

## Usage

### Basic Commands
```bash
llmspell repl --language lua        # Start Lua REPL
llmspell run script.lua             # Execute script file
llmspell list-tools                 # Show available tools
```

### Embedded Applications
All example applications are compiled directly into the `llmspell` binary for single-file distribution:

```bash
llmspell apps list                  # List all embedded applications
llmspell apps file-organizer        # Run file organization automation
llmspell apps research-collector    # Run research automation
llmspell apps content-creator       # Run content generation
llmspell apps communication-manager # Run business communication automation
llmspell apps process-orchestrator  # Run process orchestration
llmspell apps code-review-assistant # Run code review automation
llmspell apps webapp-creator        # Run webapp generation
```

These applications are:
- Compiled into the binary using `include_str!` from `resources/applications/`
- Extract to temp directory at runtime
- Require zero path configuration
- Demonstrate progression from Universal (2-3 agents) to Expert (20+ agents) complexity

For development, the source applications remain in `examples/script-users/applications/` where they can be edited and tested using traditional path-based execution:

```bash
llmspell run examples/script-users/applications/file-organizer/main.lua
```

## Dependencies
- `llmspell-core` - Core traits and types
- `llmspell-bridge` - Script engine integration
- `llmspell-agents` - Agent implementations
- `llmspell-workflows` - Workflow orchestration