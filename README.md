# rs-llmspell

Scriptable LLM interactions via Lua and JavaScript - Cast scripting spells to animate LLM golems

## Overview

Rs-LLMSpell provides script-driven workflows for LLM interactions with 34 built-in tools, agent coordination, persistent state management, session lifecycle management, and a powerful hook/event system for extensibility. Currently v0.6.0 (Phase 6 complete), working toward 1.0 release.

## Quick Example

```lua
-- Create an agent and use tools
local agent = Agent.create({
    model = "openai/gpt-4",
    system_prompt = "You are a helpful assistant"
})

local tool = Tool.get("file_operations")
local content = tool:execute({
    operation = "read",
    path = "data.txt"
})

local response = agent:execute({
    prompt = "Summarize this content: " .. content.output
})

Logger.info("Summary", {result = response.output})
```

### Hook and Event Example (v0.4.0)

```lua
-- Register a hook to monitor agent execution
Hook.register("agent:before_execution", function(context)
    Logger.info("Agent starting", {agent_id = context.agent_id})
    return {continue_execution = true}
end)

-- Subscribe to error events
Event.subscribe("*.error", function(event)
    Alert.send("Error occurred", event.payload)
end)

-- Emit custom events
Event.emit({
    event_type = "custom:analysis_complete",
    payload = {duration_ms = 250, records = 1000}
})
```

### Persistent State Example (v0.5.0)

```lua
-- Save state with automatic persistence
State.save("agent:gpt-4", "conversation_history", messages)

-- Load state with fallback
local config = State.load("global", "app_config") or {theme = "dark"}

-- Backup state before critical operations
local backup_id = State.backup({description = "Before update"})

-- Perform migration
State.migrate({
    from_version = 1,
    to_version = 2,
    transformations = {
        {field = "old_field", transform = "copy", to = "new_field"}
    }
})
```

### Session Management Example (v0.6.0)

```lua
-- Create a session for long-running interactions
local session = Session.create({
    name = "research_session",
    max_duration = 3600  -- 1 hour
})

-- Store artifacts within the session
local artifact_id = Artifact.store(session.id, "analysis_result", {
    summary = "Market analysis complete",
    data = {revenue = 1000000, growth = 0.15}
})

-- Retrieve artifacts later
local artifact = Artifact.get(session.id, artifact_id)

-- Suspend and resume sessions
Session.suspend(session.id)
-- ... later ...
Session.resume(session.id)

-- List all artifacts in a session
local artifacts = Artifact.list(session.id)
```

## Installation

```bash
# Clone and build
git clone https://github.com/lexlapax/rs-llmspell
cd rs-llmspell
cargo build --release

# Run examples
./target/release/llmspell run examples/hello.lua
```

## Features

- **34 Production Tools**: File operations, web scraping, data processing, system utilities
- **Agent Coordination**: Create and orchestrate LLM agents with different models
- **Workflow Patterns**: Sequential, parallel, conditional, and loop execution
- **Session Management**: Long-running sessions with suspend/resume, artifacts, and replay
- **Persistent State Management**: Multi-backend persistence with migration and backup support
- **Hook System**: 40+ extensibility points with <1% performance overhead
- **Event Bus**: Cross-language event propagation at >90K events/sec
- **Built-in Hooks**: Logging, metrics, caching, rate limiting, retry, cost tracking, and security
- **State Persistence**: Sled/RocksDB backends, schema migrations, atomic backups
- **Artifact Storage**: Content-addressed storage with versioning and compression
- **Session Replay**: Full session replay capability with hook execution history
- **Security**: Comprehensive sandboxing, resource limits, and sensitive data protection
- **Multi-Provider**: Support for OpenAI, Anthropic, and local models

## Current Status

- **v0.6.0 Released**: Session management with artifacts and replay capabilities (2025-08-01)
- **v0.5.0 Released**: Persistent state management with enterprise features (2025-07-29)
- **v0.4.0 Released**: Hook and event system with cross-language support (2025-07-25)
- **v0.3.0 Released**: 34 tools, agent infrastructure, and workflows (2025-07-23)
- **Phase 6 Complete**: Production-ready session management with 39/39 tasks done
- **Pre-1.0 Software**: Breaking changes expected in Phase 7 (API standardization)
- **Production-Ready Components**: Sessions, state persistence, tools, hooks, and events
- See [CHANGELOG.md](CHANGELOG.md) for detailed version history
- See [RELEASE_NOTES_v0.6.0.md](RELEASE_NOTES_v0.6.0.md) for latest release

## Documentation

- **[Quick Start Guide](docs/user-guide/getting-started.md)** - Get started in 5 minutes
- **[Documentation Hub](docs/README.md)** - Complete documentation index
- **[Tool Reference](docs/user-guide/tool-reference.md)** - All 34 tools documented
- **[Examples](examples/)** - Working code examples for all features

## Development

```bash
# Run quality checks before committing
./scripts/quality-check-minimal.sh

# Run full test suite
cargo test --workspace

# See development guide for more
cat docs/developer-guide/README.md
```

- **[Contributing Guide](CONTRIBUTING.md)** - How to contribute
- **[Developer Documentation](docs/developer-guide/)** - Development setup and workflows
- **[Architecture](docs/technical/rs-llmspell-final-architecture.md)** - System architecture

## Project Links

- **Issues**: [GitHub Issues](https://github.com/lexlapax/rs-llmspell/issues)
- **Discussions**: [GitHub Discussions](https://github.com/lexlapax/rs-llmspell/discussions)
- **Progress Tracking**: [Phase Status](docs/in-progress/)

## License

This project is dual-licensed under:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT License ([LICENSE-MIT](LICENSE-MIT))

You may choose either license for your use.

---

**Current Focus**: Phase 7 - API consistency and standardization across all crates for 1.0 release.