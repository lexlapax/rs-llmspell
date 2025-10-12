# LLMSpell Examples

**Status**: 🚀 **Phase 8.10.6** - Complete example set with RAG integration, multi-tenancy, and cost optimization

**Learn by example - comprehensive demonstrations of LLMSpell capabilities**

**🔗 Navigation**: [← Project Home](../) | [Docs Hub](../docs/) | [User Guide](../docs/user-guide/) | [Developer Guide](../docs/developer-guide/)

---

## Overview

Welcome to the LLMSpell examples! This directory contains comprehensive examples organized by audience and learning path. All examples are tested and work with the current release, including RAG systems, persistent state management, comprehensive hook and event system integration, and session management.

## 📁 Directory Structure

```
examples/
├── script-users/                    # Lua scripting examples (50+ examples)
│   ├── getting-started/            # 6 progressive learning scripts (00-05)
│   ├── features/                   # 5 core feature demonstrations
│   ├── cookbook/                   # 11 production patterns (8 core + 3 RAG)
│   ├── advanced-patterns/          # 4 complex orchestration examples
│   ├── applications/               # 9 production-ready applications (7 base + 2 RAG)
│   ├── tests/                      # 3 RAG test suites
│   ├── benchmarks/                 # 1 RAG performance benchmark
│   └── configs/                    # 15 configuration templates
└── rust-developers/                # Rust integration examples (6 examples)
    ├── custom-tool-example/        # Tool creation fundamentals
    ├── custom-agent-example/       # Agent implementation patterns
    ├── async-patterns-example/     # Concurrent programming patterns
    ├── extension-pattern-example/  # Plugin architecture
    ├── builder-pattern-example/    # Configuration patterns
    └── integration-test-example/   # Testing strategies
```

## 🚀 Quick Start

All examples work with **builtin profiles** - no configuration files needed:

### Script Users (Lua)

```bash
# Start with basics (no LLM needed)
./target/debug/llmspell -p minimal run examples/script-users/getting-started/00-hello-world.lua
./target/debug/llmspell -p minimal run examples/script-users/getting-started/01-first-tool.lua

# Try agents (requires OpenAI/Anthropic API keys)
./target/debug/llmspell -p providers run examples/script-users/getting-started/02-first-agent.lua

# Build workflows
./target/debug/llmspell -p minimal run examples/script-users/getting-started/03-first-workflow.lua

# Learn error handling with state
./target/debug/llmspell -p state run examples/script-users/getting-started/04-handle-errors.lua

# Try RAG (Phase 8 - requires embedding API)
./target/debug/llmspell -p rag-dev run examples/script-users/getting-started/05-first-rag.lua

# Explore features
./target/debug/llmspell -p minimal run examples/script-users/features/tool-basics.lua
./target/debug/llmspell -p providers run examples/script-users/features/agent-basics.lua
```

### Rust Developers

```bash
cd examples/rust-developers/custom-tool-example && cargo run
cd examples/rust-developers/custom-agent-example && cargo run
cd examples/rust-developers/async-patterns-example && cargo run
```

### Builtin Profiles (Recommended)

LLMSpell includes 10 builtin profiles for common workflows:

```bash
# Tools and workflows only (no LLM providers)
./target/debug/llmspell -p minimal run examples/script-users/getting-started/01-first-tool.lua

# Agents with OpenAI/Anthropic
./target/debug/llmspell -p providers run examples/script-users/features/agent-basics.lua

# State persistence enabled
./target/debug/llmspell -p state run examples/script-users/features/state-persistence.lua

# RAG development (Phase 8)
./target/debug/llmspell -p rag-dev run examples/script-users/getting-started/05-first-rag.lua

# RAG production
./target/debug/llmspell -p rag-prod run examples/script-users/cookbook/rag-multi-tenant.lua

# Local LLM with Ollama
./target/debug/llmspell -p ollama run examples/local_llm_status.lua

# Sessions (state + hooks + events)
./target/debug/llmspell -p sessions run examples/script-users/cookbook/rag-session.lua
```

**Available Profiles**: minimal, development, providers, state, sessions, ollama, candle, rag-dev, rag-prod, rag-perf

### Custom Configuration (Advanced)

For unique patterns not covered by builtin profiles:

```bash
# Use custom configuration file
./target/debug/llmspell -c path/to/custom-config.toml run script.lua

# See examples/script-users/configs/ for 15 configuration templates
```

### Production Patterns (Cookbook)

```bash
# Error handling patterns (no LLM needed)
./target/debug/llmspell run examples/script-users/cookbook/error-handling.lua

# Rate limiting (no LLM needed)
./target/debug/llmspell run examples/script-users/cookbook/rate-limiting.lua

# Multi-agent coordination (requires API keys)
./target/debug/llmspell -p providers run examples/script-users/cookbook/multi-agent-coordination.lua

# RAG patterns (Phase 8 - requires embedding API)
./target/debug/llmspell -p rag-prod run examples/script-users/cookbook/rag-multi-tenant.lua

./target/debug/llmspell -p sessions run examples/script-users/cookbook/rag-session.lua

./target/debug/llmspell -p rag-prod run examples/script-users/cookbook/rag-cost-optimization.lua
```

### Complete Applications

Applications include app-specific config files for demonstration, but can run with builtin profiles:

```bash
# Web app creator - with builtin profile
./target/debug/llmspell -p development run examples/script-users/applications/webapp-creator/main.lua

# Or with app-specific config (demonstrates configuration patterns)
./target/debug/llmspell -c examples/script-users/applications/webapp-creator/config.toml \
  run examples/script-users/applications/webapp-creator/main.lua

# Knowledge base (Phase 8) - with builtin RAG profile
./target/debug/llmspell -p rag-prod run examples/script-users/applications/knowledge-base/main.lua

# Personal assistant (Phase 8) - with sessions profile
./target/debug/llmspell -p sessions run examples/script-users/applications/personal-assistant/main.lua
```

See individual application READMEs for detailed configuration options.

## 📋 Example Categories

### Script Users (`script-users/`)

#### Getting Started (6 examples)
Progressive learning path from basics to RAG:
- **`00-hello-world.lua`** - Simplest possible script
- **`01-first-tool.lua`** - File operations with tools
- **`02-first-agent.lua`** - Creating LLM agents
- **`03-first-workflow.lua`** - Workflow orchestration
- **`04-handle-errors.lua`** - Error handling patterns
- **`05-first-rag.lua`** - RAG system basics (Phase 8)

#### Features (5 demonstrations)
Core capability demonstrations:
- **`agent-basics.lua`** - Agent creation and management
- **`tool-basics.lua`** - Using the 34+ built-in tools
- **`workflow-basics.lua`** - Workflow patterns
- **`state-persistence.lua`** - State management
- **`provider-info.lua`** - Provider configuration

#### Cookbook (11 production patterns)
Battle-tested patterns for production:

**Core Patterns (8):**
- **`error-handling.lua`** - Comprehensive error recovery
- **`rate-limiting.lua`** - API quota management
- **`caching.lua`** - Performance optimization
- **`multi-agent-coordination.lua`** - Agent collaboration
- **`webhook-integration.lua`** - External systems
- **`performance-monitoring.lua`** - Observability
- **`security-patterns.lua`** - Input validation
- **`state-management.lua`** - Persistence patterns

**RAG Patterns (3 - Phase 8):**
- **`rag-multi-tenant.lua`** - Tenant isolation
- **`rag-session.lua`** - Conversational memory
- **`rag-cost-optimization.lua`** - 70% cost reduction

#### Advanced Patterns (4 examples)
Complex orchestration and integration:
- **`multi-agent-orchestration.lua`** - Complex coordination
- **`complex-workflows.lua`** - Advanced workflows
- **`tool-integration-patterns.lua`** - Tool chaining
- **`monitoring-security.lua`** - Production monitoring

#### Applications (9 complete apps)
Production-ready applications:

**Base Applications (7):**
- **`webapp-creator/`** - Multi-agent web app generator
- **`code-review-assistant/`** - Automated code review
- **`content-creator/`** - Multi-format content
- **`communication-manager/`** - Email orchestration
- **`file-organizer/`** - Intelligent file organization
- **`process-orchestrator/`** - Workflow automation
- **`research-collector/`** - v2.0 with RAG integration

**RAG Applications (2 - Phase 8):**
- **`knowledge-base/`** - Personal knowledge management
- **`personal-assistant/`** - AI productivity companion

### Rust Developers (`rust-developers/`)

6 comprehensive Rust integration examples:
- **`custom-tool-example/`** - Creating custom tools with Tool trait
- **`custom-agent-example/`** - Agent implementation patterns
- **`async-patterns-example/`** - Concurrent execution and streaming
- **`extension-pattern-example/`** - Plugin architecture
- **`builder-pattern-example/`** - Fluent API configuration
- **`integration-test-example/`** - Testing strategies

### Tests and Benchmarks (`script-users/`)

#### Test Suites (3 RAG tests)
- **`test-rag-basic.lua`** - Basic RAG operations
- **`test-rag-e2e.lua`** - End-to-end testing
- **`test-rag-errors.lua`** - Error handling

#### Benchmarks (1)
- **`rag-benchmark.lua`** - RAG performance measurement

### Configuration Templates (`script-users/configs/`)

15 ready-to-use configuration files:
- **Core configs**: minimal, basic, llmspell
- **Provider configs**: example-providers, applications, cookbook
- **State configs**: state-enabled, session-enabled, migration-enabled, backup-enabled
- **RAG configs**: rag-basic, rag-development, rag-production, rag-performance, rag-multi-tenant

## 🔧 Configuration

### Builtin Profiles (Recommended)

LLMSpell includes **10 builtin profiles** that cover most use cases:

| Profile | Purpose | When to Use |
|---------|---------|-------------|
| **minimal** | Tools only, no LLM | Testing, learning tools |
| **development** | Dev mode with debug | Development, debugging |
| **providers** | OpenAI + Anthropic | Agent examples, LLM scripts |
| **state** | State persistence | State management examples |
| **sessions** | Sessions + state + events | Conversational apps |
| **ollama** | Local Ollama LLM | Local LLM with Ollama |
| **candle** | Local Candle LLM | Local LLM with Candle |
| **rag-dev** | RAG development | Learning RAG, prototyping |
| **rag-prod** | RAG production | Production RAG deployment |
| **rag-perf** | RAG performance | High-performance RAG |

**Usage**: `llmspell -p <profile-name> run script.lua`

### Custom Configuration Files (Advanced)

For unique patterns not covered by builtin profiles, see custom configuration templates:

**Location**: `script-users/configs/`

- **`basic.toml`** - Basic state operations
- **`example-providers.toml`** - OpenAI + Anthropic setup
- **`state-enabled.toml`** - State persistence enabled
- **`rag-basic.toml`** - Getting started with RAG
- **`rag-multi-tenant.toml`** - Multi-tenant SaaS deployment
- **`applications.toml`** - Full application configuration

See [Configuration Guide](script-users/configs/README.md) for detailed documentation.

### Setting API Keys

```bash
# OpenAI (for GPT models and embeddings)
export OPENAI_API_KEY="your-key-here"

# Anthropic (for Claude models)
export ANTHROPIC_API_KEY="your-key-here"
```

## 📚 Learning Path

### For Beginners

1. Start with `script-users/getting-started/00-hello-world.lua`
2. Progress through examples 01-05 in order
3. Explore `script-users/features/` for specific capabilities
4. Study `script-users/cookbook/` for production patterns

### For Rust Developers

1. Start with `rust-developers/custom-tool-example/`
2. Progress to `custom-agent-example/`
3. Explore `async-patterns-example/` for concurrency
4. Study `builder-pattern-example/` for configuration

### For RAG Users (Phase 8)

1. Start with `script-users/getting-started/05-first-rag.lua`
2. Try RAG cookbook patterns:
   - `rag-multi-tenant.lua` for SaaS applications
   - `rag-session.lua` for conversational AI
   - `rag-cost-optimization.lua` for cost reduction
3. Build complete RAG applications:
   - `applications/knowledge-base/` for knowledge management
   - `applications/personal-assistant/` for AI companion

### For Production Deployment

1. Study all patterns in `script-users/cookbook/`
2. Review `script-users/advanced-patterns/`
3. Examine complete applications in `script-users/applications/`
4. Use appropriate configs from `script-users/configs/`

## 🏃 Running Examples

### Individual Scripts

```bash
# Change to llmspell directory
cd /path/to/rs-llmspell

# Run with builtin profile (recommended)
llmspell -p minimal run examples/hello.lua

# Run with custom config (advanced)
llmspell -c path/to/config.toml run examples/hello.lua

# Or use cargo run
cargo run --bin llmspell -- -p minimal run examples/hello.lua
```

### Batch Execution

```bash
# Make scripts executable
chmod +x examples/*.sh

# Run all examples of a type (legacy scripts)
./examples/run-all-agent-examples.sh
./examples/run-all-tools-examples.sh
./examples/run-workflow-examples.sh

# Run new hook and event examples
llmspell run examples/lua/run-all-examples.lua
llmspell run examples/lua/run-integration-demos.lua
```

## 🐛 Troubleshooting

### Common Issues

1. **"API key not found"**
   - Set environment variables for your providers:
     ```bash
     export OPENAI_API_KEY="your-key-here"
     export ANTHROPIC_API_KEY="your-key-here"
     ```
   - Use the `providers` builtin profile: `llmspell -p providers run script.lua`

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

5. **"State not available"**
   - Use the `state` builtin profile: `llmspell -p state run script.lua`
   - For full sessions: `llmspell -p sessions run script.lua`

6. **"RAG not available"**
   - Use RAG builtin profiles: `llmspell -p rag-dev run script.lua`
   - Ensure OPENAI_API_KEY is set for embeddings

7. **"Hook registration failed"**
   - Check hook point name spelling and validity
   - Ensure hook function returns valid result type
   - Use `Hook.list()` to see registered hooks

8. **"Event not received"**
   - Verify subscription pattern matches event name
   - Check event was published before receive timeout
   - Use `Event.list_subscriptions()` to debug subscriptions

9. **"Integration example failed"**
   - Ensure system has sufficient resources for complex examples
   - Check that all dependencies are properly initialized
   - Review integration logs for specific component failures

### Debug Mode

Enable debug output:

```lua
-- In your script
Logger.set_level("debug")

-- Or via environment
RUST_LOG=debug llmspell run examples/hello.lua
```

### Hook & Event System Debugging

For hook and event system issues:

```lua
-- List all registered hooks
local hooks = Hook.list()
print("Registered hooks:", #hooks)

-- List all event subscriptions  
local subs = Event.list_subscriptions()
print("Active subscriptions:", #subs)

-- Test hook registration
local test_handle = Hook.register("BeforeAgentExecution", function(ctx)
    print("Hook called with:", ctx.component_id.name)
    return "continue"
end, "normal")

-- Test event publish/subscribe
local sub_id = Event.subscribe("test.*")
Event.publish("test.message", {data = "hello"})
local received = Event.receive(sub_id, 1000) -- 1 second timeout
```

## 📖 Documentation

For detailed documentation, see:

- [User Guide](../docs/user-guide/) - Complete usage documentation
- [Tutorial](../docs/user-guide/tutorial-agents-workflows.md) - Step-by-step tutorial
- [API Reference](../docs/user-guide/api-reference-agents-workflows.md) - Full API docs
- [Tool Reference](../docs/user-guide/tool-reference.md) - All 34 tools documented
- [Hook System Guide](../docs/user-guide/hooks-guide.md) - Complete hook system documentation
- [Event System Guide](../docs/user-guide/events-guide.md) - Event system architecture and patterns

## 🤝 Contributing Examples

To add new examples:

1. **Location**: Place in appropriate subdirectory under `script-users/` or `rust-developers/`
2. **Documentation**: Include comprehensive header comments with:
   - Purpose and use case
   - Prerequisites (API keys, configs)
   - Expected output
   - Time to complete
3. **Naming**: Follow existing patterns (e.g., `feature-name.lua`, `pattern-name-example/`)
4. **Testing**: Ensure example works with current release
5. **Configuration**: Provide or reference appropriate config files
6. **Error Handling**: Include proper error handling patterns
7. **Update README**: Add to this file and relevant subdirectory READMEs

## 🆕 Phase 8 RAG Features

Phase 8.10.6 introduces comprehensive RAG (Retrieval-Augmented Generation) support:

### RAG Capabilities

- **Vector Storage**: HNSW algorithm for fast similarity search
- **Multi-Tenancy**: Complete isolation between tenant knowledge bases
- **Session Management**: Conversational memory with automatic cleanup
- **Cost Optimization**: Smart caching reduces embedding costs by 70%
- **Bi-temporal Metadata**: Track both event time and ingestion time
- **TTL Support**: Automatic document expiration for compliance
- **Production Ready**: Battle-tested patterns for enterprise deployment

### RAG Examples

**Getting Started:**
- `script-users/getting-started/05-first-rag.lua` - Learn RAG basics

**Production Patterns:**
- `script-users/cookbook/rag-multi-tenant.lua` - Tenant isolation
- `script-users/cookbook/rag-session.lua` - Conversational memory
- `script-users/cookbook/rag-cost-optimization.lua` - Cost reduction

**Complete Applications:**
- `script-users/applications/knowledge-base/` - Knowledge management system
- `script-users/applications/personal-assistant/` - AI productivity companion
- `script-users/applications/research-collector/` - v2.0 with RAG integration

### Performance Targets

- Document ingestion: >100 docs/second
- Vector search: <50ms for 1M vectors
- Embedding cache hit rate: >80%
- Memory usage: ~1GB per million vectors
- Multi-tenant overhead: <5%

## 📊 Summary

### Total Examples: 60+

**Script Users (Lua)**: 50+ examples
- Getting Started: 6 progressive examples
- Features: 5 capability demonstrations
- Cookbook: 11 production patterns (including 3 RAG)
- Advanced Patterns: 4 complex scenarios
- Applications: 9 complete applications (including 2 RAG)
- Tests: 3 RAG test suites
- Benchmarks: 1 performance benchmark
- Configs: 15 configuration templates

**Rust Developers**: 6 integration examples
- Tool creation, agent implementation, async patterns
- Extension architecture, builder patterns, testing

### What's New in Phase 8

- RAG integration with multi-tenancy and sessions
- Cost optimization patterns (70% reduction)
- 2 new RAG applications (knowledge-base, personal-assistant)
- 3 RAG cookbook patterns
- Comprehensive RAG test suites
- 5 specialized RAG configurations

---

**Happy experimenting with LLMSpell!** 🚀