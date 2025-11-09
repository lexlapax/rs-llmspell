# Crate Index - Complete API Reference

**Quick reference to all 21 llmspell crates**

🔗 **Thematic Guides**: [Core Traits](core-traits.md) | [Storage](storage-backends.md) | [RAG](rag-pipeline.md) | [Memory](memory-backends.md) | [Security](security-integration.md)

---

## Overview

This index provides quick access to all llmspell crates with links to detailed documentation and generated cargo docs.

**Total Crates**: 21
**Documentation Coverage**: >95%
**Version**: 0.13.0

---

## Core Infrastructure (3 crates)

### llmspell-core
**Foundation traits and types**

- `BaseAgent` trait - Foundation for all components
- `ExecutionContext` - Runtime context management
- Error handling with `LLMSpellError`
- Component metadata and lifecycle

📚 **Thematic Guide**: [Core Traits & Foundation](core-traits.md)
📦 **Cargo Doc**: `cargo doc --open -p llmspell-core`

---

### llmspell-utils
**Shared utilities and helpers**

- Async operations and timeouts
- Security utilities (SSRF, path validation)
- File operations and encoding
- Rate limiting and circuit breakers
- API key management

📚 **Thematic Guide**: [Core Traits & Foundation](core-traits.md)
📦 **Cargo Doc**: `cargo doc --open -p llmspell-utils`

---

### llmspell-testing
**Testing framework and utilities**

- Test categorization and macros
- Mock implementations
- Property-based test generators
- Fixtures and benchmarking

📚 **Thematic Guide**: [Core Traits & Foundation](core-traits.md)
📦 **Cargo Doc**: `cargo doc --open -p llmspell-testing`

---

## Storage and Persistence (1 crate)

### llmspell-storage ⭐ Phase 8
**Vector and key-value storage**

- HNSW vector storage for RAG
- Multiple backend implementations (InMemory, Sled, HNSW)
- Multi-tenant data isolation
- Collection management
- Bi-temporal support
- TTL mechanism

📚 **Thematic Guide**: [Storage Backends](storage-backends.md)
📦 **Cargo Doc**: `cargo doc --open -p llmspell-storage`

---

## Security and Multi-Tenancy (2 crates)

### llmspell-security
**Security framework**

- Access control policies (RBAC)
- Authentication/authorization
- Input validation (SSRF, path traversal)
- Sandboxing
- Audit logging

📚 **Thematic Guide**: [Security & Multi-Tenancy](security-integration.md)
📦 **Cargo Doc**: `cargo doc --open -p llmspell-security`

---

### llmspell-tenancy ⭐ Phase 8
**Multi-tenant isolation**

- Tenant management
- Resource quotas
- Data isolation
- Cross-tenant prevention

📚 **Thematic Guide**: [Security & Multi-Tenancy](security-integration.md)
📦 **Cargo Doc**: `cargo doc --open -p llmspell-tenancy`

---

## AI and RAG Components (7 crates)

### llmspell-memory ⭐ Phase 13
**Adaptive memory system**

- Episodic memory (conversation history with vector search)
- Semantic memory (knowledge graph integration)
- Procedural memory (learned patterns)
- Hot-swappable backends (HNSW, InMemory, ChromaDB, Qdrant)
- Consolidation engine (LLM-driven knowledge extraction)
- Multi-session isolation with <2ms add performance

📚 **Thematic Guide**: [Memory Backends](memory-backends.md)
📦 **Cargo Doc**: `cargo doc --open -p llmspell-memory`

---

### llmspell-graph ⭐ Phase 13
**Bi-temporal knowledge graph**

- Entity and relationship storage
- Event time and ingestion time tracking
- Temporal queries and time-travel
- SurrealDB embedded backend
- Corrections without losing history
- Semantic memory backend for llmspell-memory

📚 **Thematic Guide**: [RAG Pipeline & Context Engineering](rag-pipeline.md)
📦 **Cargo Doc**: `cargo doc --open -p llmspell-graph`

---

### llmspell-context ⭐ Phase 13
**Context engineering pipeline**

- Query understanding and intent classification
- Multi-strategy retrieval (episodic, semantic, hybrid, RAG)
- Reranking (DeBERTa cross-encoder, BM25 fallback)
- Token-budget-aware context assembly
- Parallel retrieval (~2x speedup)
- Confidence scoring and temporal ordering

📚 **Thematic Guide**: [RAG Pipeline & Context Engineering](rag-pipeline.md)
📦 **Cargo Doc**: `cargo doc --open -p llmspell-context`

---

### llmspell-rag ⭐ Phase 8
**Retrieval-Augmented Generation**

- Document ingestion pipeline
- Chunking strategies (semantic, fixed-size, recursive)
- Embedding providers (OpenAI, Candle)
- Vector search integration
- Multi-tenant RAG

📚 **Thematic Guide**: [RAG Pipeline & Context Engineering](rag-pipeline.md)
📦 **Cargo Doc**: `cargo doc --open -p llmspell-rag`

---

### llmspell-agents
**Agent framework**

- Agent trait and builders
- Context management
- Tool integration
- Agent composition
- Templates and discovery

📦 **Cargo Doc**: `cargo doc --open -p llmspell-agents`

---

### llmspell-providers ⭐ Phase 11
**LLM provider integrations**

- Provider trait
- OpenAI, Anthropic, Groq
- Local models: Ollama, Candle (CPU/GPU)
- Model management (pull, list, info)
- Health checks and status
- Streaming support
- Rate limiting

📦 **Cargo Doc**: `cargo doc --open -p llmspell-providers`

---

### llmspell-templates ⭐ Phase 12
**Production-Ready AI Workflow Templates**

- Template trait and registry
- 10 built-in templates (research, chat, analysis, code generation, document processing, workflow orchestration)
- ExecutionContext for infrastructure access
- Parameter validation and cost estimation
- Template discovery and search
- CLI and Lua API integration

📦 **Cargo Doc**: `cargo doc --open -p llmspell-templates`

---

## Execution and Orchestration (4 crates)

### llmspell-workflows
**Workflow orchestration**

- Sequential, parallel, conditional flows
- Step definitions
- Error handling
- State management

📦 **Cargo Doc**: `cargo doc --open -p llmspell-workflows`

---

### llmspell-tools
**Tool system**

- Tool trait and registry
- Built-in tools (100+)
- Security levels
- Tool composition

📦 **Cargo Doc**: `cargo doc --open -p llmspell-tools`

---

### llmspell-hooks
**Hook system**

- Lifecycle hooks
- Event interception
- Hook priorities
- Replay support

📦 **Cargo Doc**: `cargo doc --open -p llmspell-hooks`

---

### llmspell-events
**Event system**

- Event bus
- Pub/sub patterns
- Event correlation
- Persistence

📦 **Cargo Doc**: `cargo doc --open -p llmspell-events`

---

## Integration and Runtime (4 crates)

### llmspell-bridge ⭐ Phase 11a.8
**Script language bridges**

- Lua integration
- Typed struct pattern for configurations
- Parser functions for type-safe conversions
- Global injection (18 globals)
- Performance optimization

📦 **Cargo Doc**: `cargo doc --open -p llmspell-bridge`

---

### llmspell-kernel ⭐ Phase 10
**Kernel and daemon infrastructure**

- Jupyter protocol v5.3 (5-channel ZeroMQ)
- Debug Adapter Protocol (DAP)
- Unix daemon (double-fork)
- Signal handling (SIGTERM/SIGINT)
- Session management
- Fleet orchestration

📦 **Cargo Doc**: `cargo doc --open -p llmspell-kernel`

---

### llmspell-config
**Configuration system**

- Config schema
- Environment variables
- Provider configs
- Validation

📦 **Cargo Doc**: `cargo doc --open -p llmspell-config`

---

### llmspell-cli ⭐ Phase 10
**CLI application**

- Command parsing (run, kernel, tool, model)
- Runtime initialization
- Script execution
- Tool CLI commands (list, info, invoke, search, test)
- Output formatting

📦 **Cargo Doc**: `cargo doc --open -p llmspell-cli`

---

## Quick Reference Tables

### By Phase

| Phase | Crates | Features |
|-------|--------|----------|
| **Phase 13** | memory, graph, context | Adaptive memory, knowledge graph, context engineering |
| **Phase 12** | templates | AI workflow templates |
| **Phase 11** | providers, bridge | Local LLM, typed bridge pattern |
| **Phase 10** | kernel, cli | Jupyter protocol, daemon, tool CLI |
| **Phase 8** | storage, rag, tenancy | Vector storage, RAG, multi-tenancy |
| **Phase 7** | hooks, events | Lifecycle hooks, event system |
| **Phase 6** | workflows, tools | Workflow orchestration, tool system |
| **Core** | core, utils, testing, agents, config, security | Foundation |

### By Layer

| Layer | Crates | Purpose |
|-------|--------|---------|
| **Core** | core, utils, testing | Foundation traits, utilities, testing |
| **Storage** | storage | Key-value, vector storage |
| **Security** | security, tenancy | Access control, multi-tenancy |
| **AI/RAG** | rag, agents, providers, memory, graph, context, templates | LLM integration, RAG, memory |
| **Execution** | workflows, tools, hooks, events | Orchestration, tools, events |
| **Integration** | bridge, config, cli, kernel | Script integration, configuration, runtime |

### By Dependency Graph

**Foundation (no dependencies)**:
- llmspell-core
- llmspell-utils

**Layer 1 (depends on foundation)**:
- llmspell-testing
- llmspell-config
- llmspell-security

**Layer 2 (depends on Layer 1)**:
- llmspell-storage
- llmspell-tenancy
- llmspell-events
- llmspell-hooks

**Layer 3 (depends on Layer 2)**:
- llmspell-rag
- llmspell-graph
- llmspell-memory
- llmspell-tools
- llmspell-workflows

**Layer 4 (depends on Layer 3)**:
- llmspell-context
- llmspell-agents
- llmspell-providers
- llmspell-templates

**Layer 5 (top level)**:
- llmspell-bridge
- llmspell-kernel
- llmspell-cli

---

## Documentation Navigation

### For Beginners

Start here to understand the system:
1. [core-traits.md](core-traits.md) - Foundation concepts with BaseAgent trait
2. Generate API docs: `cargo doc --open -p llmspell-core`
3. [storage-backends.md](storage-backends.md) - Understanding storage systems

### For RAG/Memory Features

1. [rag-pipeline.md](rag-pipeline.md) - RAG overview
2. [memory-backends.md](memory-backends.md) - Memory system
3. [storage-backends.md](storage-backends.md) - Vector storage details

### For Security/Multi-Tenancy

1. [security-integration.md](security-integration.md) - Complete security guide
2. Generate API docs: `cargo doc --open -p llmspell-security`
3. Generate API docs: `cargo doc --open -p llmspell-tenancy`

### For Extension Development

1. [../03-extending-components.md](../03-extending-components.md) - Extension guide
2. [core-traits.md](core-traits.md) - Trait implementation patterns
3. Generate testing API docs: `cargo doc --open -p llmspell-testing`

---

## Generating Documentation

### Full Workspace Docs

```bash
# Generate all crate documentation
cargo doc --workspace --all-features --no-deps

# Open in browser
cargo doc --workspace --all-features --no-deps --open
```

### Single Crate Docs

```bash
# Generate docs for specific crate
cargo doc --package llmspell-core --all-features --open

# With dependencies
cargo doc --package llmspell-rag --all-features --open
```

### Documentation Server

```bash
# Serve documentation locally
cargo install cargo-server
cargo doc --workspace --all-features --no-deps
cargo server target/doc
# Browse to http://localhost:3000
```

---

## Contributing to Documentation

When adding or updating crate documentation:

1. **API Documentation**: Use `///` doc comments in code
2. **Crate README**: Update individual crate .md files
3. **Thematic Guides**: Update relevant thematic guide if architecture changes
4. **This Index**: Update crate description if major features added

**Documentation Standards**:
- >95% API coverage
- Examples for all public APIs
- Link to related crates
- Include performance characteristics

---

## Related Documentation

- **Thematic Guides**: Start with these for overview
  - [core-traits.md](core-traits.md)
  - [storage-backends.md](storage-backends.md)
  - [rag-pipeline.md](rag-pipeline.md)
  - [memory-backends.md](memory-backends.md)
  - [security-integration.md](security-integration.md)

- **Developer Guides**:
  - [../README.md](../README.md) - Developer overview
  - [../03-extending-components.md](../03-extending-components.md) - Extension guide
  - [../04-bridge-patterns.md](../04-bridge-patterns.md) - Bridge patterns
  - [../05-production-deployment.md](../05-production-deployment.md) - Production deployment

- **Architecture**:
  - [../../technical/master-architecture-vision.md](../../technical/master-architecture-vision.md) - System architecture
  - [../../technical/platform-support.md](../../technical/platform-support.md) - Platform support

---

**Version**: 0.13.0 | **Phase**: 13b.18.1 | **Last Updated**: 2025-11-08
