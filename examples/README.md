# Rs-LLMSpell Examples

**Current Version**: Phase 3.3 Complete  
**Available Tools**: 34 production tools  
**Features**: Agents, Workflows, Tools, State Management

**🔗 Navigation**: [← Project Home](../README.md) | [Documentation Hub](../docs/README.md) | [User Guide](../docs/user-guide/README.md)

---

## Overview

This directory contains working examples demonstrating rs-llmspell capabilities. All examples are tested and work with the current Phase 3.3 implementation.

## 📁 Directory Structure

```
examples/
├── hello.lua                    # Basic hello world script
├── llmspell-demo.lua           # Comprehensive demo
├── provider-info.lua           # Provider configuration info
├── streaming-demo.lua          # Streaming responses
├── multimodal-stub.lua         # Multimodal capabilities
├── llmspell.toml              # Example configuration
├── minimal.toml               # Minimal configuration
├── lua/
│   ├── agents/                 # Agent examples (10 files)
│   ├── tools/                  # Tool examples (13 files)
│   └── workflows/              # Workflow examples (9 files)
└── *.sh                        # Runner scripts
```

## 🚀 Quick Start

### Basic Examples

```bash
# Run hello world
llmspell run examples/hello.lua

# Run comprehensive demo
llmspell run examples/llmspell-demo.lua

# Check provider configuration
llmspell run examples/provider-info.lua
```

### Agent Examples (Requires API Keys)

```bash
# Run all agent examples
./examples/run-all-agent-examples.sh

# Run specific agent example
llmspell run examples/lua/agents/agent-simple.lua
llmspell run examples/lua/agents/agent-orchestrator.lua
```

### Tool Examples

```bash
# Run all tool examples
./examples/run-all-tools-examples.sh

# Run specific tool category
llmspell run examples/lua/tools/tools-filesystem.lua
llmspell run examples/lua/tools/tools-utility.lua
```

### Workflow Examples

```bash
# Run workflow examples
./examples/run-workflow-examples.sh

# Run specific workflow pattern
llmspell run examples/lua/workflows/workflow-sequential.lua
llmspell run examples/lua/workflows/workflow-parallel.lua
```

## 📋 Example Categories

### Root Examples

- **`hello.lua`** - Simplest possible script
- **`llmspell-demo.lua`** - Shows core functionality
- **`provider-info.lua`** - Display configured providers
- **`streaming-demo.lua`** - Streaming LLM responses
- **`multimodal-stub.lua`** - Multimodal input examples

### Agent Examples (`lua/agents/`)

Demonstrates agent creation, configuration, and usage patterns:

- **`agent-simple.lua`** - Basic agent creation and execution
- **`agent-orchestrator.lua`** - Agent coordinating multiple tools
- **`agent-coordinator.lua`** - Multi-agent coordination
- **`agent-processor.lua`** - Data processing with agents
- **`agent-monitor.lua`** - System monitoring agent
- **`agent-composition.lua`** - Composing agents together
- **`agent-async-example.lua`** - Async patterns (internal use)
- **`agent-simple-demo.lua`** - Interactive demo
- **`agent-simple-benchmark.lua`** - Performance testing
- **`agent-api-comprehensive.lua`** - Full API demonstration

### Tool Examples (`lua/tools/`)

Shows all 34 available tools organized by category:

- **`tools-showcase.lua`** - Demonstrates all tools
- **`tools-filesystem.lua`** - File operations (5 tools)
- **`tools-data.lua`** - Data processing (4 tools)
- **`tools-web.lua`** - Web & network tools (7 tools)
- **`tools-system.lua`** - System integration (4 tools)
- **`tools-utility.lua`** - Utility tools (10 tools)
- **`tools-media.lua`** - Media processing (3 tools)
- **`tools-security.lua`** - Security features demo
- **`tools-integration.lua`** - External integrations
- **`tools-workflow.lua`** - Tools in workflows
- **`tools-performance.lua`** - Performance benchmarks
- **`tools-utility-reference.lua`** - API reference example

### Workflow Examples (`lua/workflows/`)

Demonstrates all 4 workflow patterns:

- **`workflow-sequential.lua`** - Step-by-step execution
- **`workflow-conditional.lua`** - Branching logic
- **`workflow-loop.lua`** - Iteration patterns
- **`workflow-parallel.lua`** - Concurrent execution
- **`workflow-basics-*.lua`** - Basic pattern examples
- **`workflow-agent-integration.lua`** - Agents in workflows

## 🔧 Configuration

### Example Configuration Files

- **`llmspell.toml`** - Full configuration example
- **`minimal.toml`** - Minimal required configuration

### Setting API Keys

```bash
# OpenAI
export OPENAI_API_KEY="your-key-here"

# Anthropic
export ANTHROPIC_API_KEY="your-key-here"

# Other providers as needed
```

## 📚 Learning Path

### For Beginners

1. Start with `hello.lua`
2. Explore `tools-utility.lua` for basic tool usage
3. Try `agent-simple.lua` for agent basics
4. Move to `workflow-sequential.lua` for workflows

### For Tool Users

1. Browse `tools-showcase.lua` for overview
2. Deep dive into category-specific examples
3. Check `tools-security.lua` for security patterns
4. Study `tools-workflow.lua` for integration

### For Agent Developers

1. Start with `agent-simple.lua`
2. Progress to `agent-orchestrator.lua`
3. Explore `agent-coordinator.lua` for multi-agent
4. Review `agent-api-comprehensive.lua` for full API

### For Workflow Builders

1. Learn patterns in `workflow-basics-*.lua`
2. Study complete examples in main workflow files
3. Check `workflow-agent-integration.lua` for AI workflows

## 🏃 Running Examples

### Individual Scripts

```bash
# Change to llmspell directory
cd /path/to/rs-llmspell

# Run with llmspell
llmspell run examples/hello.lua

# Or use cargo run
cargo run --bin llmspell -- run examples/hello.lua
```

### Batch Execution

```bash
# Make scripts executable
chmod +x examples/*.sh

# Run all examples of a type
./examples/run-all-agent-examples.sh
./examples/run-all-tools-examples.sh
./examples/run-workflow-examples.sh
```

### With Custom Config

```bash
# Use specific configuration
llmspell run --config examples/minimal.toml examples/hello.lua
```

## 🐛 Troubleshooting

### Common Issues

1. **"API key not found"**
   - Set environment variables for your providers
   - Check `llmspell.toml` configuration

2. **"Tool not found"**
   - Ensure you're using correct tool names
   - Run `Tool.list()` to see available tools

3. **"Agent timeout"**
   - Check network connectivity
   - Verify API keys are valid
   - Increase timeout in agent configuration

4. **"Permission denied"**
   - Some system tools require elevated permissions
   - File operations are sandboxed by default

### Debug Mode

Enable debug output:

```lua
-- In your script
Logger.set_level("debug")

-- Or via environment
RUST_LOG=debug llmspell run examples/hello.lua
```

## 📖 Documentation

For detailed documentation, see:

- [User Guide](../docs/user-guide/) - Complete usage documentation
- [Tutorial](../docs/user-guide/tutorial-agents-workflows.md) - Step-by-step tutorial
- [API Reference](../docs/user-guide/api-reference-agents-workflows.md) - Full API docs
- [Tool Reference](../docs/user-guide/tool-reference.md) - All 34 tools documented

## 🤝 Contributing Examples

To add new examples:

1. Follow existing naming patterns
2. Include clear comments explaining the example
3. Test thoroughly before submitting
4. Update this README if adding new categories
5. Ensure examples work with current implementation

---

**Happy experimenting with rs-llmspell!** 🚀