# Script Users Examples

**Status**: 🚀 **Phase 13** (v0.13.x) - Adaptive memory, context engineering, and 21 preset profiles

**Lua scripting examples for LLMSpell - from basics to production applications**

**🔗 Navigation**: [← Examples](../) | [Project Home](../../) | [Getting Started](getting-started/) | [Applications](applications/)

## 📊 Quick Stats

- **6 Getting Started Examples** - Learn the basics step-by-step
- **5 Feature Demonstrations** - Explore specific capabilities
- **16 Cookbook Patterns** - Production-ready patterns (including 3 RAG patterns)
- **4 Advanced Patterns** - Complex orchestration and integration
- **11 Complete Applications** - Full production examples (2 RAG apps)
- **3 RAG Test Suites** - Comprehensive RAG testing
- **1 RAG Benchmark** - Performance measurement
- **21 Builtin Profiles** - Zero-config quick start
- **Custom Config Examples** - For unique patterns

## 🚀 Quick Start

All examples work with builtin profiles - no configuration files needed:

```bash
# Tools and workflows (no LLM needed)
llmspell -p minimal run getting-started/00-hello-world.lua

# Agent examples (requires OpenAI/Anthropic API keys)
llmspell -p providers run getting-started/02-first-agent.lua

# State persistence examples
llmspell -p state run features/state-persistence.lua

# Memory & RAG examples (requires API key)
llmspell -p memory run getting-started/05-memory-rag-advanced.lua

# Local LLM examples (requires Ollama installed)
llmspell -p ollama run features/local-llm-status.lua
```

### Available Builtin Profiles (21 total)

- **minimal** - Tools only, no LLM providers
- **development** - Dev settings with OpenAI/Anthropic + debug logging
- **providers** - Simple OpenAI/Anthropic setup for agents
- **state** - State persistence enabled
- **sessions** - Sessions + state + hooks + events
- **memory** - Adaptive memory system (Phase 13)
- **memory-development** - Memory + RAG debugging
- **ollama** - Local Ollama LLM backend
- **candle** - Local Candle LLM backend (CPU/GPU)
- **rag-dev** - RAG development with debug features
- **rag-prod** - RAG production settings

See [Profile Guide](../../docs/user-guide/profile-layers-guide.md) for all 21 profiles.

### Custom Configuration (Optional)

For advanced use cases, create a custom config file:
```bash
llmspell -c path/to/custom-config.toml run script.lua
```

## 📂 Directory Structure

```
script-users/
├── getting-started/     # 6 beginner examples (00-05)
├── features/           # 5 feature demonstrations
├── cookbook/           # 16 production patterns (13 core + 3 RAG)
├── advanced-patterns/  # 4 complex orchestration examples
├── applications/       # 11 complete applications (9 base + 2 RAG)
├── tests/             # 3 RAG test suites
├── benchmarks/        # 1 performance benchmark
└── configs/           # Custom configuration examples (unique patterns)
```

## 📚 Categories

### [Getting Started](getting-started/) - 6 Examples
Progressive examples from hello world to RAG systems. Start here if you're new to LLMSpell.

**Learning Path:**
1. `00-hello-world.lua` - Simplest possible example
2. `01-first-tool.lua` - Using your first tool (file operations)
3. `02-first-agent.lua` - Creating your first agent
4. `03-first-workflow.lua` - Building your first workflow
5. `04-handle-errors.lua` - Proper error handling
6. `05-memory-rag-advanced.lua` - Memory & RAG system (Phase 13)

### [Features](features/) - 5 Demonstrations
Comprehensive demonstrations of LLMSpell capabilities.

**Available Demonstrations:**
- `agent-basics.lua` - Agent creation and management
- `tool-basics.lua` - Using the 34+ built-in tools
- `workflow-basics.lua` - Workflow patterns and orchestration
- `state-persistence.lua` - State management with scopes
- `provider-info.lua` - Provider configuration and detection

### [Cookbook](cookbook/) - 11 Production Patterns
Battle-tested patterns and best practices for production use.

**Core Patterns (v0.7.0):**
- `01-error-handling.lua` - Comprehensive error recovery strategies
- `02-rate-limiting.lua` - API quota and throttling management
- `03-caching.lua` - High-performance caching strategies
- `04-multi-agent-coordination.lua` - Agent collaboration patterns
- `05-webhook-integration.lua` - External system integration
- `06-performance-monitoring.lua` - Observability and metrics
- `07-security-patterns.lua` - Input validation and secure handling
- `08-state-management.lua` - Versioning and persistence

**RAG Patterns:**
- `RAG-01-rag-multi-tenant.lua` - Isolated vector stores per tenant
- `RAG-02-rag-session.lua` - Conversational memory with context
- `RAG-03-rag-cost-optimization.lua` - Reduce embedding costs by 70%

### [Applications](applications/) - 9 Complete Applications
Complete, production-ready example applications.

**Available Applications:**
- `webapp-creator` - Multi-agent web application generator
- `code-review-assistant` - Automated code review system
- `content-creator` - Multi-format content generation
- `communication-manager` - Email and notification orchestration
- `file-organizer` - Intelligent file organization system
- `process-orchestrator` - Complex workflow automation
- `research-collector` - v2.0 with RAG integration
- `knowledge-base` - Personal knowledge management
- `personal-assistant` - AI productivity companion

### [Advanced Patterns](advanced-patterns/) - 4 Complex Examples
Production-ready patterns bridging features and applications.

**Advanced Patterns:**
- `multi-agent-orchestration.lua` - Complex agent coordination
- `complex-workflows.lua` - Advanced workflow orchestration
- `tool-integration-patterns.lua` - Tool chaining and integration
- `monitoring-security.lua` - Production monitoring and security

### [Tests](tests/) - RAG Testing Suite
Comprehensive test suites for RAG functionality.

**Test Files:**
- `test-rag-basic.lua` - Basic RAG operations
- `test-rag-e2e.lua` - End-to-end RAG testing
- `test-rag-errors.lua` - Error handling and edge cases

### [Benchmarks](benchmarks/) - Performance Measurement
- `rag-benchmark.lua` - RAG performance benchmarking

### [Configs](configs/) - Custom Configuration Examples
Custom configuration files for unique patterns and advanced scenarios.

**Demonstration Configs:**
- RAG configurations (basic, production, multi-tenant)
- Multi-provider setups
- Advanced session management
- Application-specific settings

**Note:** Most examples work with builtin profiles (`-p profile-name`). These configs demonstrate custom patterns for advanced use cases.

## 🚀 Running Examples

### With Builtin Profiles (Recommended)

```bash
# Run with appropriate builtin profile
llmspell -p minimal run examples/script-users/getting-started/00-hello-world.lua
llmspell -p providers run examples/script-users/features/agent-basics.lua
llmspell -p rag-dev run examples/script-users/getting-started/05-first-rag.lua
llmspell -p development run examples/script-users/applications/knowledge-base/main.lua
```

### With Custom Configuration (Advanced)

For unique patterns not covered by builtin profiles:

```bash
# With custom RAG configuration
llmspell -c examples/script-users/configs/rag-multi-tenant.toml \
  run examples/script-users/cookbook/rag-multi-tenant.lua

# With application-specific configuration
llmspell -c examples/script-users/applications/webapp-creator/config.toml \
  run examples/script-users/applications/webapp-creator/main.lua
```

## 📖 Prerequisites

### For Getting Started Examples
- LLMSpell installed (`cargo install llmspell` or build from source)
- No API keys required for basic examples

### For Agent Examples
- API key for at least one provider:
  - OpenAI: Set `OPENAI_API_KEY` environment variable
  - Anthropic: Set `ANTHROPIC_API_KEY` environment variable
  - Local models: Use `-p ollama-production` (or see [configs/](configs/) for custom patterns)

### For Advanced Examples
- Understanding of Lua basics
- Familiarity with async patterns
- Knowledge of error handling

## 🆕 Phase 13 Memory & RAG Features

This release introduces comprehensive adaptive memory and context engineering:

- **Adaptive Memory**: 3-tier memory system (episodic, semantic, procedural)
- **Context Engineering**: Strategy-based context assembly
- **Vector Storage**: HNSW algorithm for fast similarity search
- **Multi-Tenancy**: Complete isolation between tenant knowledge bases
- **Session Management**: Conversational memory with automatic cleanup
- **21 Preset Profiles**: Zero-config deployment for any use case
- **Production Ready**: Battle-tested patterns for enterprise deployment

## 🎯 Learning Recommendations

### Complete Beginner
1. Start with [getting-started](getting-started/) examples in order
2. Try modifying examples to understand concepts
3. Move to [features](features/) for specific capabilities
4. Study [cookbook](cookbook/) for production patterns

### Experienced Developer
1. Skim [getting-started](getting-started/) for LLMSpell specifics
2. Jump to [features](features/) for your use case
3. Review [cookbook](cookbook/) for best practices
4. Study [applications](applications/) for architecture patterns

### Production Deployment
1. Start with [cookbook](cookbook/) patterns
2. Review [applications](applications/) for complete examples
3. Focus on error handling, monitoring, and performance
4. Test thoroughly with your specific use case

## 📝 Example Standards

All examples follow these standards:
- **Clear Documentation**: Comprehensive header with purpose, prerequisites, and usage
- **Expected Output**: Documented output for verification
- **Error Handling**: Production-ready error recovery patterns
- **Security**: No hardcoded secrets or credentials
- **Self-Contained**: Runnable without external dependencies (unless documented)
- **Version Tagged**: Examples marked with version (v0.7.0, v0.8.0)
- **Complexity Levels**: BEGINNER, INTERMEDIATE, PRODUCTION
- **Time Estimates**: Expected runtime documented

## 🔗 Related Resources

- [Rust Developer Examples](../rust-developers/) - For embedding LLMSpell
- [User Guide](../../docs/user-guide/) - Comprehensive documentation
- [Lua API Reference](../../docs/user-guide/api/lua/README.md) - Lua API documentation
- [Tool Catalog](../../docs/user-guide/tools-catalog.md) - All 34+ tools
- [Architecture Guide](../../docs/technical/master-architecture-vision.md) - System architecture