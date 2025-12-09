# Profile Layers System Guide

**Version**: 0.14.0
**Last Updated**: January 2025
**Phase**: 13c.4 (Storage Consolidation)

> **📋 Quick Reference**: Complete guide to LLMSpell's 4-layer profile architecture for composable configuration management.

**🔗 Navigation**: [← User Guide](README.md) | [Configuration](03-configuration.md) | [CLI Reference](05-cli-reference.md)

---

## Table of Contents

1. [Introduction](#introduction)
2. [Quick Start](#quick-start)
3. [The 4-Layer Architecture](#the-4-layer-architecture)
4. [Multi-Layer Composition](#multi-layer-composition)
5. [Layer Type Reference](#layer-type-reference)
6. [Preset Catalog](#preset-catalog)
7. [Common Composition Patterns](#common-composition-patterns)
8. [Merge Strategy & Precedence](#merge-strategy--precedence)
9. [Best Practices](#best-practices)
10. [Troubleshooting](#troubleshooting)

---

## Introduction

LLMSpell's **4-layer profile system** provides composable, modular configuration management through a layered architecture. Instead of maintaining separate monolithic configuration files, you compose profiles from reusable layers.

### Why Layers?

**Problem with monolithic configs**:
- Duplication across similar environments (dev/staging/prod)
- Difficult to maintain consistency
- Hard to share common patterns
- No reusable building blocks

**Solution with layers**:
- **Composable**: Mix and match layers as needed
- **Reusable**: Share layers across profiles
- **Maintainable**: Update one layer, affects all compositions
- **Clear**: Explicit separation of concerns

### Design Philosophy

The 4-layer system follows a clear hierarchy:

```
Base Layer (Deployment Mode)
  ↓ overrides ↓
Feature Layer (Capabilities)
  ↓ overrides ↓
Environment Layer (Tuning)
  ↓ overrides ↓
Backend Layer (Storage)
  = Final Configuration
```

Each layer focuses on a single concern, making configurations predictable and maintainable.

---

## Quick Start

### Using Builtin Presets

The simplest way to use the layer system is through builtin presets:

```bash
# Single preset name (backward compatible)
llmspell -p minimal run script.lua

# Explicit preset path (equivalent)
llmspell -p presets/minimal run script.lua
```

Available presets combine layers for common use cases. See [Preset Catalog](#preset-catalog) for the full list of 21 presets.

> **🚀 Quick Tip: Production-Ready Presets**
>
> For production use with full features (Graph + RAG + Memory + Context + SQLite), use one of these three presets based on your preferred LLM:
> - `gemini-prod` - Google Gemini (requires `GEMINI_API_KEY`)
> - `openai-prod` - OpenAI GPT (requires `OPENAI_API_KEY`)
> - `claude-prod` - Anthropic Claude (requires `ANTHROPIC_API_KEY`)
>
> All three are identical except for the default LLM provider!
>
> ```bash
> export GEMINI_API_KEY="your-key"
> llmspell -p gemini-prod run your-app.lua
> ```

### Using Multi-Layer Composition

For custom configurations, compose layers explicitly:

```bash
# Basic 2-layer composition: base + feature
llmspell -p bases/cli,features/minimal run script.lua

# Standard 3-layer composition: base + feature + environment
llmspell -p bases/cli,features/rag,envs/dev run script.lua

# Full 4-layer composition: base + feature + env + backend
llmspell -p bases/cli,features/full,envs/prod,backends/sqlite run script.lua
```

Layers are applied left-to-right, with later layers overriding earlier ones.

### Syntax Forms

The profile system supports three syntax forms:

| Syntax | Example | Description |
|--------|---------|-------------|
| **Single preset** | `minimal` | Backward-compatible preset name |
| **Explicit preset** | `presets/rag-dev` | Explicit preset path |
| **Multi-layer** | `bases/cli,features/rag,envs/dev` | Comma-separated layer composition |

All three forms can be used interchangeably with the `-p` flag.

---

## The 4-Layer Architecture

### Layer Types

The layer system uses four orthogonal layer types:

| Layer Type | Purpose | Count | Location |
|------------|---------|-------|----------|
| **bases/** | Deployment mode (CLI, daemon, embedded) | 4 | `llmspell-config/layers/bases/` |
| **features/** | Capabilities (minimal, LLM, RAG, memory, full) | 7 | `llmspell-config/layers/features/` |
| **envs/** | Environment tuning (dev, staging, prod, perf) | 4 | `llmspell-config/layers/envs/` |
| **backends/** | Storage backend (memory, SQLite, PostgreSQL) | 3 | `llmspell-config/layers/backends/` |

**Total**: 18 reusable layer files.

### Layer Composition Rules

1. **One layer per type** (typically): Choose one base, one feature, one env, one backend
2. **Order matters**: Later layers override earlier layers
3. **Partial composition allowed**: Can use 1-4 layers (e.g., just `bases/cli,features/minimal`)
4. **No duplicates needed**: The system handles extends and merging automatically

### Example Composition

```bash
llmspell -p bases/daemon,features/full,envs/prod,backends/postgres run script.lua
```

This composition:
1. Starts with **daemon** base (high concurrency, service mode)
2. Adds **full** features (graph + RAG + memory + context)
3. Tunes for **production** environment (warn-level logging, optimized)
4. Configures **PostgreSQL** backend (persistent storage)

Result: Production-ready daemon with all Phase 13 features and PostgreSQL persistence.

---

## Multi-Layer Composition

### Syntax

Multi-layer composition uses comma-separated layer paths:

```
llmspell -p layer1,layer2,layer3,layer4
```

Whitespace is automatically trimmed:

```bash
# These are equivalent:
llmspell -p bases/cli,features/rag,envs/dev
llmspell -p "bases/cli, features/rag, envs/dev"
llmspell -p " bases/cli , features/rag , envs/dev "
```

### Composition Examples

**Minimal CLI** (tools only, no LLM):
```bash
llmspell -p bases/cli,features/minimal
```

**RAG Development** (RAG + debug logging):
```bash
llmspell -p bases/cli,features/rag,envs/dev
```

**Full Local Stack** (all features, local models, SQLite):
```bash
llmspell -p bases/cli,features/full,envs/dev,backends/sqlite
```

**Production Daemon** (high concurrency, PostgreSQL):
```bash
llmspell -p bases/daemon,features/full,envs/prod,backends/postgres
```

### Preset vs. Multi-Layer

You can achieve the same result using either approach:

**Using preset**:
```bash
llmspell -p rag-dev
```

**Using multi-layer composition**:
```bash
llmspell -p bases/cli,features/rag,envs/dev,backends/sqlite
```

Presets are convenient shortcuts for common layer combinations. Multi-layer composition gives you full control for custom configurations.

---

## Layer Type Reference

### Bases (Deployment Modes)

Base layers define the deployment mode and runtime characteristics.

#### `bases/cli`

**Purpose**: Interactive command-line execution
**Use Case**: One-shot script execution, development, experimentation

**Key Settings**:
- `max_concurrent_scripts = 1` (single-user)
- `script_timeout = 300s` (5 minutes)
- `state_persistence.enabled = false` (in-memory only)
- `sessions.max_sessions = 1` (single session)
- `security.*_access = false` (safe defaults)

**Example**:
```bash
llmspell -p bases/cli,features/minimal run hello.lua
```

#### `bases/daemon`

**Purpose**: Long-running service mode
**Use Case**: Production deployments, multi-user systems, background processing

**Key Settings**:
- `max_concurrent_scripts = 100` (high concurrency)
- `script_timeout = 3600s` (1 hour)
- `state_persistence.enabled = true` (persistent state)
- `sessions.max_sessions = 1000` (multi-user)
- `events.enabled = true` (full event tracking)

**Example**:
```bash
llmspell -p bases/daemon,features/full,envs/prod,backends/postgres kernel start
```

#### `bases/embedded`

**Purpose**: Embedded library usage
**Use Case**: Integration into larger applications, API-first usage

**Key Settings**:
- `max_concurrent_scripts = 10` (moderate concurrency)
- `script_timeout = 600s` (10 minutes)
- `debug.output.stdout = false` (API logging only)
- `events.enabled = true` (programmatic event access)

**Example**:
```bash
llmspell -p bases/embedded,features/llm,backends/memory run api-server.lua
```

#### `bases/testing`

**Purpose**: Automated testing environments
**Use Case**: CI/CD, integration tests, benchmarks

**Key Settings**:
- `max_concurrent_scripts = 20` (parallel test execution)
- `script_timeout = 120s` (2 minutes - fast fail)
- `debug.level = "trace"` (comprehensive logging)
- `state_persistence.backend_type = "memory"` (ephemeral)

**Example**:
```bash
llmspell -p bases/testing,features/full run test-suite.lua
```

### Features (Capabilities)

Feature layers enable specific functionality domains.

#### `features/minimal`

**Purpose**: Bare minimum (tools and workflows only)
**Use Case**: Testing, learning, scripts without LLM needs

**Enables**:
- Tool execution
- Basic workflow patterns
- No LLM providers
- No RAG/memory/graph

**Example**:
```bash
llmspell -p bases/cli,features/minimal run tool-test.lua
```

#### `features/llm`

**Purpose**: LLM provider access
**Use Case**: Basic agent scripting, cloud LLM usage

**Enables**:
- OpenAI provider
- Anthropic provider
- Gemini provider
- Simple agent creation

**Excludes**: RAG, memory, graph (for that, use `features/full`)

**Example**:
```bash
export OPENAI_API_KEY="sk-..."
llmspell -p bases/cli,features/llm,envs/dev run agent-chat.lua
```

#### `features/rag`

**Purpose**: Retrieval-Augmented Generation
**Use Case**: Document search, semantic retrieval, knowledge bases

**Enables**:
- Vector storage (HNSW)
- Embedding generation (OpenAI/local)
- Document chunking
- Semantic search

**Configuration**:
```toml
[rag]
enabled = true
[rag.vector_storage]
dimensions = 384
backend = "hnsw"
[rag.vector_storage.hnsw]
m = 16
ef_construction = 200
metric = "cosine"
```

**Example**:
```bash
llmspell -p bases/cli,features/rag,envs/dev run doc-search.lua
```

#### `features/memory`

**Purpose**: Adaptive memory system (Phase 13)
**Use Case**: Conversational agents, context retention, long-running sessions

**Enables**:
- 3-tier memory architecture (working/episodic/semantic)
- Hot-swap backends (memory/SQLite/PostgreSQL)
- Memory consolidation strategies
- Context window management

**Configuration**:
```toml
[runtime.memory]
enabled = true
default_backend = "memory"
consolidation_interval_seconds = 300
max_working_memory_items = 100
```

**Example**:
```bash
llmspell -p bases/cli,features/memory,envs/dev,backends/sqlite run chatbot.lua
```

#### `features/state`

**Purpose**: State persistence and session management
**Use Case**: Stateful workflows, persistent data, multi-step processes

**Enables**:
- State persistence
- Session management
- Hook system
- Event correlation

**Configuration**:
```toml
[runtime.state_persistence]
enabled = true
[runtime.sessions]
enabled = true
max_sessions = 100
```

**Example**:
```bash
llmspell -p bases/cli,features/state,backends/sqlite run workflow.lua
```

#### `features/full`

**Purpose**: Complete Phase 13 feature set
**Use Case**: Production systems, research, comprehensive AI applications

**Enables**:
- **Graph**: Temporal knowledge graph with bi-temporal versioning
- **RAG**: Vector storage + HNSW search
- **Memory**: 3-tier adaptive memory system
- **Context**: Context engineering pipeline
- **All LLM providers**: OpenAI, Anthropic, Gemini, Ollama, Candle

**This is the superset** - includes everything from minimal/llm/rag/memory/state.

**Example**:
```bash
llmspell -p bases/cli,features/full,envs/prod,backends/postgres run full-app.lua
```

#### `features/local`

**Purpose**: Local-only models (Ollama + Candle)
**Use Case**: Air-gapped environments, privacy-focused deployments, offline usage

**Enables**:
- Ollama provider (local LLMs)
- Candle provider (local ML models)
- No cloud API dependencies

**Example**:
```bash
llmspell -p bases/cli,features/local,backends/sqlite run offline-agent.lua
```

### Environments (Tuning)

Environment layers tune settings for different deployment stages.

#### `envs/dev`

**Purpose**: Development and debugging
**Use Case**: Local development, experimentation, troubleshooting

**Key Settings**:
- `debug.level = "debug"` (verbose logging)
- `runtime.max_concurrent_scripts = 10` (moderate concurrency)
- `script_timeout = 600s` (long timeouts for debugging)
- `debug.output.colored = true` (colored terminal output)

**Example**:
```bash
llmspell -p bases/cli,features/full,envs/dev,backends/sqlite run dev-script.lua
```

#### `envs/staging`

**Purpose**: Pre-production testing
**Use Case**: Integration testing, performance validation, UAT

**Key Settings**:
- `debug.level = "info"` (informational logging)
- `runtime.max_concurrent_scripts = 50` (production-like load)
- `script_timeout = 300s` (reasonable timeouts)
- Monitoring enabled

**Example**:
```bash
llmspell -p bases/daemon,features/full,envs/staging,backends/postgres kernel start
```

#### `envs/prod`

**Purpose**: Production deployments
**Use Case**: Live systems, customer-facing services

**Key Settings**:
- `debug.level = "warn"` (warnings and errors only)
- `script_timeout = 300s` (strict timeouts)
- `debug.output.colored = false` (machine-readable logs)
- Security hardened

**Example**:
```bash
llmspell -p bases/daemon,features/full,envs/prod,backends/postgres kernel start
```

#### `envs/perf`

**Purpose**: Performance benchmarking
**Use Case**: Load testing, profiling, optimization

**Key Settings**:
- `debug.level = "error"` (minimal logging overhead)
- `runtime.max_concurrent_scripts = 200` (stress testing)
- `script_timeout = 60s` (fast fail)
- Caching optimized

**Example**:
```bash
llmspell -p bases/daemon,features/full,envs/perf,backends/postgres run benchmark.lua
```

### Backends (Storage)

Backend layers configure persistence and storage.

#### `backends/memory`

**Purpose**: In-memory ephemeral storage
**Use Case**: Testing, development, stateless scripts

**Key Settings**:
- `state_persistence.backend_type = "memory"`
- No disk I/O
- Fast performance
- Data lost on exit

**Example**:
```bash
llmspell -p bases/cli,features/minimal,backends/memory run test.lua
```

#### `backends/sqlite`

**Purpose**: SQLite embedded database
**Use Case**: Single-instance deployments, local persistence, development

**Key Settings**:
- `state_persistence.backend_type = "sqlite"`
- `state_persistence.sqlite.path = "./llmspell.db"`
- Local file storage
- No separate database server needed

**Vector Storage**: Uses `vectorlite-rs` extension for HNSW vector search.

**Example**:
```bash
llmspell -p bases/cli,features/full,envs/prod,backends/sqlite run app.lua
```

#### `backends/postgres`

**Purpose**: PostgreSQL database
**Use Case**: Multi-instance deployments, production systems, scalability

**Key Settings**:
- `state_persistence.backend_type = "postgres"`
- `state_persistence.postgres.connection_string = "postgresql://..."`
- Centralized storage
- Supports clustering

**Vector Storage**: Uses `pgvector` extension for vector search.

**Example**:
```bash
export DATABASE_URL="postgresql://user:pass@localhost/llmspell"
llmspell -p bases/daemon,features/full,envs/prod,backends/postgres kernel start
```

---

## Preset Catalog

LLMSpell includes **21 builtin presets** that combine layers for common use cases.

### Backward-Compatible Presets (13)

These presets maintain backward compatibility with v0.13.0 and earlier.

#### `minimal`

**Extends**: `bases/cli`, `features/minimal`, `backends/memory`

**Description**: Tools and workflows only, no LLM features

**Use Case**: Testing tools, learning workflow patterns

```bash
llmspell -p minimal run tool-demo.lua
```

#### `development`

**Extends**: `bases/cli`, `features/llm`, `envs/dev`, `backends/memory`

**Description**: Full development environment with debug logging

**Use Case**: Development, debugging, comprehensive logging

```bash
llmspell -p development run dev-script.lua
```

#### `providers`

**Extends**: `bases/cli`, `features/llm`, `backends/memory`

**Description**: All LLM providers (OpenAI, Anthropic, Gemini, Ollama, Candle)

**Use Case**: Agent examples, basic LLM scripts, getting started

```bash
export OPENAI_API_KEY="sk-..."
llmspell -p providers run agent-chat.lua
```

#### `state`

**Extends**: `bases/cli`, `features/state`, `backends/memory`

**Description**: State persistence with memory backend

**Use Case**: State management examples, data persistence across runs

```bash
llmspell -p state run stateful-workflow.lua
```

#### `sessions`

**Extends**: `bases/cli`, `features/state`, `backends/sqlite`

**Description**: Full session management (state + hooks + events)

**Use Case**: Multi-session workflows, session artifacts, comprehensive state

```bash
llmspell -p sessions run session-demo.lua
```

#### `ollama`

**Extends**: `bases/cli`, `features/local`, `backends/sqlite`

**Description**: Local Ollama models

**Use Case**: Offline development, privacy-focused applications

**Requires**: Ollama running locally (`ollama serve`)

```bash
llmspell -p ollama run local-chat.lua
```

#### `candle`

**Extends**: `bases/cli`, `features/local`, `backends/sqlite`

**Description**: Local Candle ML models

**Use Case**: Embedded ML, local inference, air-gapped environments

```bash
llmspell -p candle run ml-inference.lua
```

#### `memory`

**Extends**: `bases/cli`, `features/memory`, `envs/dev`, `backends/sqlite`

**Description**: Adaptive memory system (Phase 13)

**Use Case**: Conversational agents, context retention, memory experiments

```bash
llmspell -p memory run chatbot-with-memory.lua
```

#### `memory-development`

**Extends**: `bases/cli`, `features/llm`, `features/memory`, `features/rag`, `envs/dev`, `backends/sqlite`

**Description**: Full memory + RAG stack for development

**Use Case**: Memory system debugging, RAG development with LLM integration

```bash
llmspell -p memory-development run memory-debug.lua
```

#### `rag-dev`

**Extends**: `bases/cli`, `features/rag`, `envs/dev`, `backends/sqlite`

**Description**: RAG development with trace logging

**Use Case**: Developing document search, debugging vector storage

```bash
llmspell -p rag-dev run doc-indexing.lua
```

#### `rag-prod`

**Extends**: `bases/cli`, `features/rag`, `envs/prod`, `backends/sqlite`

**Description**: RAG production with SQLite

**Use Case**: Production document search with local storage

```bash
llmspell -p rag-prod run knowledge-base.lua
```

#### `rag-perf`

**Extends**: `bases/cli`, `features/rag`, `envs/perf`, `backends/sqlite`

**Description**: RAG performance tuned

**Use Case**: Benchmarking vector search, load testing retrieval

```bash
llmspell -p rag-perf run rag-benchmark.lua
```

#### `default`

**Extends**: `bases/cli`, `features/minimal`, `backends/memory`

**Description**: Minimal CLI setup (same as `minimal`)

**Use Case**: Default when no profile specified

```bash
llmspell run script.lua  # Uses 'default' profile
```

### New Combination Presets (8)

These presets showcase advanced layer combinations new in v0.14.0.

#### `postgres-prod`

**Extends**: `bases/daemon`, `features/full`, `envs/prod`, `backends/postgres`

**Description**: Production daemon with PostgreSQL backend

**Use Case**: Multi-instance production deployments, enterprise systems

**Requires**: PostgreSQL with pgvector extension

```bash
export DATABASE_URL="postgresql://user:pass@localhost/llmspell"
llmspell -p postgres-prod kernel start --port 9555
```

#### `daemon-dev`

**Extends**: `bases/daemon`, `features/full`, `envs/dev`, `backends/sqlite`

**Description**: Daemon mode development

**Use Case**: Developing long-running services, testing concurrency

```bash
llmspell -p daemon-dev kernel start --port 9555
```

#### `daemon-prod`

**Extends**: `bases/daemon`, `features/full`, `envs/prod`, `backends/postgres`

**Description**: Production daemon (alias for `postgres-prod`)

**Use Case**: Production service deployments

```bash
llmspell -p daemon-prod kernel start
```

#### `gemini-prod`

**Extends**: `bases/cli`, `features/full`, `envs/prod`, `backends/sqlite`

**Overrides**: `providers.default_provider = "gemini"`

**Description**: Full Phase 13 stack with Gemini as default provider

**Use Case**: Google Gemini production deployments

**Requires**: `GEMINI_API_KEY` environment variable

```bash
export GEMINI_API_KEY="..."
llmspell -p gemini-prod run gemini-app.lua
```

#### `openai-prod`

**Extends**: `bases/cli`, `features/full`, `envs/prod`, `backends/sqlite`

**Overrides**: `providers.default_provider = "openai"`

**Description**: Full Phase 13 stack with OpenAI as default provider

**Use Case**: OpenAI production deployments

**Requires**: `OPENAI_API_KEY` environment variable

```bash
export OPENAI_API_KEY="sk-..."
llmspell -p openai-prod run openai-app.lua
```

#### `claude-prod`

**Extends**: `bases/cli`, `features/full`, `envs/prod`, `backends/sqlite`

**Overrides**: `providers.default_provider = "anthropic"`

**Description**: Full Phase 13 stack with Claude/Anthropic as default provider

**Use Case**: Anthropic Claude production deployments

**Requires**: `ANTHROPIC_API_KEY` environment variable

```bash
export ANTHROPIC_API_KEY="sk-ant-..."
llmspell -p claude-prod run claude-app.lua
```

> **💡 Production Preset Comparison**
>
> The three production presets (`gemini-prod`, `openai-prod`, `claude-prod`) are **identical in structure** and differ only in the default LLM provider:
>
> | Preset | Default Provider | API Key Required |
> |--------|-----------------|------------------|
> | `gemini-prod` | Google Gemini | `GEMINI_API_KEY` |
> | `openai-prod` | OpenAI GPT | `OPENAI_API_KEY` |
> | `claude-prod` | Anthropic Claude | `ANTHROPIC_API_KEY` |
>
> **All three include**:
> - ✅ **Full Phase 13 features**: Graph + RAG + Memory + Context
> - ✅ **SQLite persistence**: Local database storage
> - ✅ **Production tuning**: Optimized logging and performance
> - ✅ **CLI deployment**: Interactive command-line mode
>
> **Choose based on your preferred LLM provider** - the capabilities are identical!
>
> **Layer composition** (all three):
> ```
> bases/cli + features/full + envs/prod + backends/sqlite
> ```
>
> **What's included in `features/full`**:
> - **Temporal Knowledge Graph**: Bi-temporal versioning, relationship tracking
> - **RAG System**: Vector storage with HNSW indexing, semantic search
> - **Adaptive Memory**: 3-tier memory (working/episodic/semantic)
> - **Context Engineering**: Context window management, prompt optimization
> - **All LLM Providers**: OpenAI, Anthropic, Gemini, Ollama, Candle
> - **State Management**: Persistent state, sessions, hooks, events
>
> **Example usage**:
> ```bash
> # Use Gemini
> export GEMINI_API_KEY="your-key"
> llmspell -p gemini-prod run app.lua
>
> # Use OpenAI
> export OPENAI_API_KEY="sk-..."
> llmspell -p openai-prod run app.lua
>
> # Use Claude
> export ANTHROPIC_API_KEY="sk-ant-..."
> llmspell -p claude-prod run app.lua
> ```

#### `full-local-ollama`

**Extends**: `bases/cli`, `features/full`, `envs/dev`, `backends/sqlite`

**Overrides**: `providers.default_provider = "ollama"`

**Description**: Complete local stack (Ollama + SQLite)

**Use Case**: Fully offline AI applications

**Requires**: Ollama running locally

```bash
ollama serve &
llmspell -p full-local-ollama run offline-full-stack.lua
```

#### `research`

**Extends**: `bases/cli`, `features/full`, `envs/dev`, `backends/sqlite`

**Overrides**: `debug.level = "trace"`

**Description**: Full features with trace logging for research and debugging

**Use Case**: Research experiments, deep debugging, learning internals

```bash
llmspell -p research run experiment.lua
```

---

## Common Composition Patterns

### Development Patterns

**Quick prototyping** (no persistence):
```bash
llmspell -p bases/cli,features/minimal
```

**LLM development** (cloud providers + debug logging):
```bash
llmspell -p bases/cli,features/llm,envs/dev
```

**Full local development** (all features + SQLite):
```bash
llmspell -p bases/cli,features/full,envs/dev,backends/sqlite
```

### Production Patterns

**Simple production CLI** (minimal + production tuning):
```bash
llmspell -p bases/cli,features/minimal,envs/prod
```

**RAG production** (document search + PostgreSQL):
```bash
llmspell -p bases/cli,features/rag,envs/prod,backends/postgres
```

**Full production daemon** (all features + PostgreSQL + multi-user):
```bash
llmspell -p bases/daemon,features/full,envs/prod,backends/postgres
```

### Testing Patterns

**Unit testing** (fast, ephemeral):
```bash
llmspell -p bases/testing,features/minimal,backends/memory
```

**Integration testing** (realistic, SQLite):
```bash
llmspell -p bases/testing,features/full,backends/sqlite
```

**Performance testing** (stress test configuration):
```bash
llmspell -p bases/daemon,features/full,envs/perf,backends/postgres
```

### Offline/Local Patterns

**Fully offline** (Ollama + SQLite):
```bash
llmspell -p bases/cli,features/local,backends/sqlite
```

**Local with full features** (Ollama + graph/RAG/memory):
```bash
llmspell -p bases/cli,features/full,envs/dev,backends/sqlite
# Override provider: --provider ollama (if needed)
```

---

## Merge Strategy & Precedence

### How Layers Merge

Layers are merged using a **deep merge strategy** with the following rules:

1. **Later layers override earlier layers**
2. **Nested tables are merged recursively**
3. **Arrays are replaced entirely** (not concatenated)
4. **Primitive values are replaced**

### Example Merge

Given layers applied in order: `bases/cli`, `envs/dev`

**bases/cli**:
```toml
[debug]
enabled = true
level = "info"

[runtime]
max_concurrent_scripts = 1
```

**envs/dev**:
```toml
[debug]
level = "debug"  # Overrides "info"

[runtime]
script_timeout_seconds = 600  # Added
```

**Merged result**:
```toml
[debug]
enabled = true         # From bases/cli
level = "debug"        # From envs/dev (override)

[runtime]
max_concurrent_scripts = 1    # From bases/cli
script_timeout_seconds = 600  # From envs/dev (added)
```

### Precedence Rules

When using multi-layer composition, precedence follows left-to-right order:

```bash
llmspell -p layer1,layer2,layer3,layer4
#          ^^^^^^  ^^^^^^  ^^^^^^  ^^^^^^
#          lowest                 highest precedence
```

**Standard 4-layer stack** follows this precedence:

```
bases/cli (lowest)
  ↓
features/full
  ↓
envs/prod
  ↓
backends/postgres (highest)
```

### Preset Extends

Presets use the `extends` field to specify layers:

```toml
# presets/gemini-prod.toml
[profile]
extends = ["bases/cli", "features/full", "envs/prod", "backends/sqlite"]

[providers]
default_provider = "gemini"  # Additional override
```

The `extends` array is processed left-to-right, then any additional settings in the preset override.

---

## Best Practices

### Choosing Layers

**Start with a base** matching your deployment mode:
- CLI scripts → `bases/cli`
- Long-running service → `bases/daemon`
- Library integration → `bases/embedded`
- Testing → `bases/testing`

**Add features** you need:
- No LLM → `features/minimal`
- Cloud LLMs only → `features/llm`
- Document search → `features/rag`
- Conversational memory → `features/memory`
- Everything → `features/full`

**Pick environment** tuning:
- Development → `envs/dev`
- Pre-production → `envs/staging`
- Live system → `envs/prod`
- Benchmarking → `envs/perf`

**Select backend** storage:
- No persistence → `backends/memory`
- Local file → `backends/sqlite`
- Shared database → `backends/postgres`

### Preset vs. Composition

**Use presets** when:
- Common use case covered by existing preset
- Backward compatibility desired
- Simplicity preferred

**Use multi-layer composition** when:
- Custom configuration needed
- Experimenting with combinations
- Fine-grained control required

### Development Workflow

**Typical progression**:

1. **Prototype** with minimal preset:
   ```bash
   llmspell -p minimal run prototype.lua
   ```

2. **Develop** with dev environment:
   ```bash
   llmspell -p bases/cli,features/full,envs/dev,backends/sqlite run app.lua
   ```

3. **Test** with staging environment:
   ```bash
   llmspell -p bases/cli,features/full,envs/staging,backends/sqlite run app.lua
   ```

4. **Deploy** with production configuration:
   ```bash
   llmspell -p bases/daemon,features/full,envs/prod,backends/postgres kernel start
   ```

### Performance Considerations

**Faster profiles** (less overhead):
- Use `features/minimal` instead of `features/full`
- Use `envs/prod` (minimal logging) instead of `envs/dev`
- Use `backends/memory` instead of `backends/sqlite` (if persistence not needed)

**Resource usage**:
- `bases/cli`: Low concurrency (1 script)
- `bases/daemon`: High concurrency (100 scripts)
- `features/full`: Higher memory (graph + RAG + memory enabled)
- `features/minimal`: Lower memory (no LLM/RAG/graph)

---

## Troubleshooting

### Layer Not Found

**Error**: `Layer not found: features/xyz`

**Solution**: Check layer name spelling. Available layers:
- **bases**: cli, daemon, embedded, testing
- **features**: minimal, llm, rag, memory, state, full, local
- **envs**: dev, staging, prod, perf
- **backends**: memory, sqlite, postgres

### Unexpected Configuration

**Issue**: Configuration value not what you expected

**Debug steps**:
1. Check layer order (later layers override earlier)
2. Verify preset `extends` field (use `cat llmspell-config/presets/<name>.toml`)
3. Check for explicit overrides in preset
4. Review merge precedence rules above

**Example**:
```bash
# If debug level is "warn" but you expected "debug":
llmspell -p bases/cli,envs/dev,envs/prod  # prod overrides dev!
```

**Fix**: Correct layer order:
```bash
llmspell -p bases/cli,envs/prod  # Use only the environment you want
```

### Missing Dependencies

**Error**: `RAG features require vector storage`

**Solution**: Ensure required features are enabled:
- RAG requires `features/rag` or `features/full`
- Memory requires `features/memory` or `features/full`
- State persistence requires `features/state` or `features/full`

### Backend Connection Issues

**SQLite error**: `unable to open database file`

**Solution**: Check write permissions in current directory or specify path:
```bash
export LLMSPELL_DB_PATH="/tmp/llmspell.db"
llmspell -p backends/sqlite ...
```

**PostgreSQL error**: `connection refused`

**Solution**: Verify PostgreSQL is running and connection string is correct:
```bash
export DATABASE_URL="postgresql://user:pass@localhost:5432/llmspell"
llmspell -p backends/postgres ...
```

### Performance Issues

**Slow startup**:
- Check if `features/full` is necessary (try `features/llm` or `features/minimal`)
- Verify database connection pooling settings
- Review `envs/dev` verbose logging (try `envs/prod` for less overhead)

**High memory usage**:
- Reduce `max_concurrent_scripts` (base layer setting)
- Disable unused features (`features/full` → `features/llm`)
- Tune RAG cache sizes (if using `features/rag`)

### Layer Conflicts

**Issue**: Two layers define conflicting values

**Resolution**: The merge strategy resolves conflicts automatically (later layers win). If you need different behavior, use explicit ordering:

```bash
# This:
llmspell -p layer1,layer2  # layer2 wins

# Is different from:
llmspell -p layer2,layer1  # layer1 wins
```

---

## Summary

The **4-layer profile system** provides:

- **18 reusable layers** organized into 4 types (bases, features, envs, backends)
- **20 builtin presets** for common use cases
- **Composable architecture** with clear separation of concerns
- **Backward compatibility** with v0.13.0 presets
- **Flexible composition** via multi-layer syntax

**Key Takeaways**:

1. **Use presets** for common cases: `llmspell -p minimal`, `llmspell -p rag-dev`
2. **Compose layers** for custom configs: `llmspell -p bases/cli,features/rag,envs/dev`
3. **Layer order matters**: Later layers override earlier layers
4. **Start simple**: Use `minimal` or `development`, add layers as needed

**Next Steps**:

- Explore [Preset Catalog](#preset-catalog) for ready-to-use profiles
- Review [Layer Type Reference](#layer-type-reference) for detailed layer documentation
- Check [Common Composition Patterns](#common-composition-patterns) for examples
- Read [Configuration Guide](03-configuration.md) for detailed config options

---

**See Also**:
- [Configuration Guide](03-configuration.md) - Detailed configuration reference
- [CLI Reference](05-cli-reference.md) - Command-line interface documentation
- [Development Workflow](../developer-guide/02-development-workflow.md) - Using profiles in development

**Layer Files**: `/llmspell-config/layers/{bases,features,envs,backends}/*.toml`
**Preset Files**: `/llmspell-config/presets/*.toml`
