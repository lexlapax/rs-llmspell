# rs-llmspell

Scriptable LLM interactions via Lua and JavaScript

## Overview

Rs-LLMSpell provides script-driven workflows for LLM interactions with 34 built-in tools, agent coordination, and state management. Currently Phase 3.3 complete, working toward 1.0 release.

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
- **State Management**: Thread-safe state sharing between agents and workflows
- **Security**: Comprehensive sandboxing and resource limits
- **Multi-Provider**: Support for OpenAI, Anthropic, and local models

## Current Status

- **v0.3.0 Released**: 34 tools, agent infrastructure, and workflows (2025-01-30)
- **Phase 3.3 Complete**: Agent infrastructure and workflow integration done
- **Pre-1.0 Software**: Breaking changes expected before stable release
- **Not Production Ready**: Use for experimentation and development only
- See [CHANGELOG.md](CHANGELOG.md) for detailed version history
- See [RELEASE_NOTES_v0.3.0.md](RELEASE_NOTES_v0.3.0.md) for latest release

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

**Current Focus**: Working toward 1.0 release with stable APIs and production readiness.