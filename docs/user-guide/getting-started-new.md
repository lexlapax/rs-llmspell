# Getting Started with LLMSpell

**Version**: 0.6.0  
**Status**: Production Ready  
**Last Updated**: August 2025

> **🚀 Quick Start**: Get up and running with LLMSpell in under 5 minutes.

**🔗 Navigation**: [Documentation Hub](../README.md) | [Installation](installation.md) | [Examples →](../../examples/EXAMPLE-INDEX.md)

---

## Table of Contents

1. [Overview](#overview)
2. [Prerequisites](#prerequisites)
3. [Quick Start](#quick-start)
4. [Core Concepts](#core-concepts)
5. [Your First Script](#your-first-script)
6. [Next Steps](#next-steps)
7. [Troubleshooting](#troubleshooting)

---

## Overview

LLMSpell is a scriptable LLM interaction platform that lets you orchestrate AI agents, tools, and workflows using simple Lua scripts. Think of it as "casting spells to animate LLM golems."

### Key Features
- **Script-First**: Write Lua scripts to control LLMs
- **30+ Built-in Tools**: File operations, web search, data processing
- **Agent Orchestration**: Create and compose AI agents
- **Workflow Automation**: Build complex multi-step processes
- **State Management**: Persist data across sessions

### When to Use LLMSpell
- Automating repetitive AI tasks
- Building AI-powered applications
- Orchestrating multiple LLMs
- Creating intelligent workflows
- Research and experimentation

---

## Prerequisites

### Required Knowledge
- Basic command line usage
- Familiarity with any scripting language (Lua knowledge helps but not required)

### Required Setup
```bash
# Install Rust (if building from source)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone the repository
git clone https://github.com/yourusername/rs-llmspell.git
cd rs-llmspell

# Build the project
cargo build --release
```

### Environment Variables
```bash
# At least one API key is required
export OPENAI_API_KEY="sk-..."
# Optional: Additional providers
export ANTHROPIC_API_KEY="sk-ant-..."
```

---

## Quick Start

The fastest way to see LLMSpell in action:

```bash
# Run a simple command
./target/release/llmspell exec 'print("Hello from LLMSpell!")'

# Create your first agent
./target/release/llmspell exec '
local agent = Agent.builder()
    :name("assistant")
    :model("openai/gpt-4o-mini")
    :build()
    
local response = agent:execute({
    prompt = "What is 2 + 2?"
})
print(response.response)
'
```

Expected output:
```
Hello from LLMSpell!
4
```

---

## Core Concepts

### Agents
AI entities that can process prompts and generate responses:
```lua
local agent = Agent.builder()
    :name("my_agent")
    :model("openai/gpt-4")
    :system_prompt("You are a helpful assistant")
    :build()
```

### Tools
Pre-built functions for common operations:
```lua
local result = Tool.invoke("web-search", {
    query = "LLMSpell documentation"
})
```

### Workflows
Orchestrate complex multi-step processes:
```lua
local workflow = Workflow.new("my_pipeline")
    :add_step("fetch", {type = "tool", tool = "web-fetch"})
    :add_step("process", {type = "agent", agent = agent})
    :run()
```

---

## Your First Script

Create a file called `hello.lua`:

```lua
-- hello.lua - Your first LLMSpell script

-- Create an AI assistant
local assistant = Agent.builder()
    :name("helpful_assistant")
    :model("openai/gpt-4o-mini")
    :system_prompt("You are a friendly and helpful assistant")
    :temperature(0.7)
    :build()

-- Get user input (or use a default)
local user_input = arg[1] or "Tell me an interesting fact"

-- Execute the agent
local response = assistant:execute({
    prompt = user_input
})

-- Display the result
print("Assistant:", response.response)
print("\nTokens used:", response.usage.total_tokens)
```

Run it:
```bash
./target/release/llmspell run hello.lua "What's the weather like?"
```

---

## Next Steps

### 1. Explore Examples
Start with the Universal layer examples:
```bash
# File organizer (simplest)
./target/release/llmspell -c examples/script-users/applications/file-organizer/config.toml \
    run examples/script-users/applications/file-organizer/main.lua
```

### 2. Learn the APIs
- [Agent API](api/lua/agent.md) - Create and manage agents
- [Tool Reference](tool-reference.md) - All 37 built-in tools
- [Workflow Guide](workflow-api.md) - Build complex processes

### 3. Build Something
Ideas to get started:
- Content generator
- Code reviewer
- Research assistant
- Data pipeline
- Customer support bot

---

## Troubleshooting

### Common Issues

#### Issue 1: "No API key found"
**Cause**: Missing environment variable
**Solution**: 
```bash
export OPENAI_API_KEY="your-api-key-here"
```

#### Issue 2: "Model not available"
**Cause**: Trying to use a model you don't have access to
**Solution**: Use a different model:
```lua
-- Instead of gpt-4, use:
:model("openai/gpt-3.5-turbo")
```

#### Issue 3: Script doesn't run
**Cause**: Path or permission issues
**Solution**: 
```bash
# Make sure llmspell is executable
chmod +x ./target/release/llmspell
# Use full paths
./target/release/llmspell run ./path/to/script.lua
```

### Debug Tips
```bash
# Enable debug output
RUST_LOG=debug ./target/release/llmspell run script.lua

# Check available tools
./target/release/llmspell exec 'for i, tool in ipairs(Tool.list()) do print(i, tool) end'
```

---

## See Also

- [Example Index](../../examples/EXAMPLE-INDEX.md) - All examples with descriptions
- [Configuration Guide](configuration/configuration.md) - Advanced configuration
- [API Documentation](api/README.md) - Complete API reference
- [Best Practices](state-management-best-practices.md) - Production tips

---

**Need Help?** Report issues on [GitHub](https://github.com/yourusername/rs-llmspell/issues)