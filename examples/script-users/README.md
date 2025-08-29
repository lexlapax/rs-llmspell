# Script Users Examples

**Status**: 🚀 **Phase 8.10.6** - RAG integration complete with multi-tenancy, sessions, and cost optimization

**Lua scripting examples for LLMSpell - from basics to production applications**

**🔗 Navigation**: [← Examples](../) | [Project Home](../../) | [Getting Started](getting-started/) | [Applications](applications/)

## 📊 Quick Stats

- **6 Getting Started Examples** - Learn the basics step-by-step
- **5 Feature Demonstrations** - Explore specific capabilities
- **11 Cookbook Patterns** - Production-ready patterns (including 3 RAG patterns)
- **4 Advanced Patterns** - Complex orchestration and integration
- **9 Complete Applications** - Full production examples (2 new RAG apps)
- **3 RAG Test Suites** - Comprehensive RAG testing
- **1 RAG Benchmark** - Performance measurement
- **15 Configuration Files** - Ready-to-use configurations

## 📂 Directory Structure

```
script-users/
├── getting-started/     # 6 beginner examples (00-05)
├── features/           # 5 feature demonstrations
├── cookbook/           # 11 production patterns (8 core + 3 RAG)
├── advanced-patterns/  # 4 complex orchestration examples
├── applications/       # 9 complete applications (7 base + 2 RAG)
├── tests/             # 3 RAG test suites
├── benchmarks/        # 1 performance benchmark
└── configs/           # 15 configuration files (.toml)
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
6. `05-first-rag.lua` - **NEW** - Your first RAG system (Phase 8)

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

**RAG Patterns (v0.8.0 - Phase 8):**
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
- `knowledge-base` - **NEW** - Personal knowledge management (Phase 8)
- `personal-assistant` - **NEW** - AI productivity companion (Phase 8)

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

### [Configs](configs/) - 15 Configuration Files
Ready-to-use configuration files for various scenarios.

**Key Configurations:**
- Provider configurations (OpenAI, Anthropic, etc.)
- State persistence configurations
- RAG configurations (basic, production, multi-tenant)
- Session and migration configurations

## 🚀 Running Examples

```bash
# Run any Lua example
./target/debug/llmspell run examples/script-users/getting-started/00-hello-world.lua

# With provider configuration
./target/debug/llmspell -c examples/script-users/configs/example-providers.toml \
  run examples/script-users/features/agent-basics.lua

# With RAG configuration (Phase 8)
./target/debug/llmspell -c examples/script-users/configs/rag-basic.toml \
  run examples/script-users/getting-started/05-first-rag.lua

# Run a complete application
./target/debug/llmspell -c examples/script-users/configs/applications.toml \
  run examples/script-users/applications/knowledge-base/main.lua
```

## 📖 Prerequisites

### For Getting Started Examples
- LLMSpell installed (`cargo install llmspell` or build from source)
- No API keys required for basic examples

### For Agent Examples
- API key for at least one provider:
  - OpenAI: Set `OPENAI_API_KEY` environment variable
  - Anthropic: Set `ANTHROPIC_API_KEY` environment variable
  - Local models: Configure in `llmspell.toml`

### For Advanced Examples
- Understanding of Lua basics
- Familiarity with async patterns
- Knowledge of error handling

## 🆕 Phase 8 RAG Features

This release introduces comprehensive RAG (Retrieval-Augmented Generation) support:

- **Vector Storage**: HNSW algorithm for fast similarity search
- **Multi-Tenancy**: Complete isolation between tenant knowledge bases
- **Session Management**: Conversational memory with automatic cleanup
- **Cost Optimization**: Smart caching reduces embedding costs by 70%
- **Bi-temporal Metadata**: Track event time and ingestion time
- **TTL Support**: Automatic document expiration
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