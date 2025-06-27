# Getting Started with LLMSpell

Welcome to LLMSpell! This guide will help you get up and running with scriptable LLM interactions using Lua.

## Table of Contents

1. [Installation](#installation)
2. [Basic Usage](#basic-usage)
3. [Configuration](#configuration)
4. [Your First Script](#your-first-script)
5. [Working with Providers](#working-with-providers)
6. [Next Steps](#next-steps)

## Installation

### From Source (Phase 1)

Currently, LLMSpell must be built from source:

```bash
# Clone the repository
git clone https://github.com/yourusername/rs-llmspell.git
cd rs-llmspell

# Build the project
cargo build --release

# The binary will be at target/release/llmspell
```

### System Requirements

- Rust 1.70 or later
- Cargo (comes with Rust)
- Git

## Basic Usage

LLMSpell provides a command-line interface for running scripts:

```bash
# Run a Lua script
llmspell run script.lua

# Execute inline code
llmspell exec "return 42"

# Show available engines
llmspell info

# List providers
llmspell providers
```

### Command-Line Options

- `--engine <ENGINE>`: Select script engine (currently only `lua` is available)
- `--output <FORMAT>`: Output format: `text`, `json`, or `pretty` (default: text)
- `--config <FILE>`: Specify configuration file
- `--verbose`: Enable verbose output
- `--help`: Show help information

## Configuration

LLMSpell can be configured through a TOML file. By default, it looks for `llmspell.toml` in:
1. Current directory
2. `~/.config/llmspell/`
3. `/etc/llmspell/`

### Example Configuration

```toml
# llmspell.toml

# Default script engine
default_engine = "lua"

# Runtime settings
[runtime]
log_level = "info"
default_timeout_ms = 30000

# Security settings
[runtime.security]
allow_file_access = false
allow_network_access = true
allow_process_spawn = false
max_memory_bytes = 52428800  # 50MB
max_execution_time_ms = 300000  # 5 minutes

# Lua engine configuration
[engines.lua]
stdlib = "safe"
memory_limit_mb = 50
debug_enabled = false

# Provider configuration
[providers]
default_provider = "openai"

[providers.openai]
provider_type = "openai"
model = "gpt-4"
api_key_env = "OPENAI_API_KEY"

[providers.anthropic]
provider_type = "anthropic"
model = "claude-3-opus"
api_key_env = "ANTHROPIC_API_KEY"
```

### Environment Variables

LLMSpell respects the following environment variables:

- `LLMSPELL_CONFIG`: Path to configuration file
- `LLMSPELL_LOG_LEVEL`: Logging level (trace, debug, info, warn, error)
- Provider API keys (as specified in configuration)

## Your First Script

Let's create a simple Lua script that demonstrates LLMSpell's capabilities:

### Hello World

Create a file called `hello.lua`:

```lua
-- hello.lua
print("Hello from LLMSpell!")

-- Return a value to the runtime
return {
    message = "Script completed successfully",
    timestamp = os.date(),
    lua_version = _VERSION
}
```

Run it:

```bash
llmspell run hello.lua
```

### Working with Tables

Lua tables are the primary data structure:

```lua
-- data.lua
local data = {
    users = {
        { name = "Alice", age = 30 },
        { name = "Bob", age = 25 }
    },
    total = 2
}

-- Process the data
for i, user in ipairs(data.users) do
    print(string.format("User %d: %s (age %d)", i, user.name, user.age))
end

return data
```

### Using Provider API (Phase 1 - Placeholder)

The Provider API allows listing configured providers:

```lua
-- providers.lua
print("Checking available providers...")

-- List all providers
local providers = Provider.list()

-- Display provider information
for _, provider in ipairs(providers) do
    print("Provider: " .. provider.name)
    if provider.capabilities then
        print("  - Supports streaming: " .. tostring(provider.capabilities.supports_streaming))
        print("  - Supports multimodal: " .. tostring(provider.capabilities.supports_multimodal))
    end
end

return {
    provider_count = #providers,
    providers = providers
}
```

## Working with Providers

### Provider Configuration

Providers must be configured in `llmspell.toml` with their API keys:

```toml
[providers.openai]
provider_type = "openai"
model = "gpt-4"
api_key_env = "OPENAI_API_KEY"  # Read from environment variable
```

### Setting API Keys

```bash
# Set API key for OpenAI
export OPENAI_API_KEY="your-api-key-here"

# Set API key for Anthropic
export ANTHROPIC_API_KEY="your-api-key-here"
```

### Provider Capabilities

Different providers support different features:

- **Streaming**: Receive responses as they're generated
- **Multimodal**: Support for images, audio, and video
- **Context Windows**: Maximum token limits vary by provider

## Next Steps

### Phase 1 Capabilities

In the current phase, you can:

1. Execute Lua scripts with the runtime
2. List available providers
3. Use basic Lua functionality
4. Configure security settings

### Coming Soon

Future phases will add:

- **Phase 2**: Agent creation and execution
- **Phase 3**: Tool integration
- **Phase 4**: Workflow orchestration
- **Phase 5**: JavaScript support
- **Phase 6**: Advanced streaming features

### Learning Resources

1. **Lua Programming**: [Lua 5.4 Reference Manual](https://www.lua.org/manual/5.4/)
2. **Examples**: Check the `/examples` directory for more scripts
3. **API Documentation**: Run `cargo doc --open` to view API docs
4. **Architecture**: See `/docs/technical/rs-llmspell-final-architecture.md`

### Getting Help

- **Issues**: Report bugs on GitHub
- **Documentation**: Check `/docs` for detailed guides
- **Community**: Join our Discord (coming soon)

## Security Considerations

LLMSpell runs scripts in a controlled environment with configurable security:

1. **File Access**: Disabled by default
2. **Network Access**: Enabled for LLM providers only
3. **Process Spawning**: Disabled by default
4. **Memory Limits**: Enforced to prevent runaway scripts
5. **Execution Time**: Limited to prevent infinite loops

Always review scripts before running them, especially from untrusted sources.

---

Ready to build your first LLM-powered application? Continue to the [Examples Guide](./examples.md) for more advanced scenarios!