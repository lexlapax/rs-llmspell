# Preset Catalog

**Version**: 0.14.0
**Phase**: 13c.4 (Storage Consolidation)
**Module**: `llmspell-config/presets`

> **Purpose**: Ready-to-use configuration presets for common LLMSpell use cases.

---

## Overview

This directory contains **20 builtin configuration presets** that combine layers from `../layers/` to create complete, ready-to-use profiles for common scenarios.

### Why Presets?

**Convenience**: Single command to get a fully configured environment
```bash
llmspell -p rag-dev run script.lua
# Instead of: llmspell -p bases/cli,features/rag,envs/dev,backends/sqlite run script.lua
```

**Best practices**: Presets embody recommended configurations for specific use cases

**Backward compatibility**: Maintains compatibility with v0.13.0 profile names

---

## Quick Reference

### By Use Case

| Use Case | Preset | Command |
|----------|--------|---------|
| **Learning/Testing** | `minimal` | `llmspell -p minimal run script.lua` |
| **Agent Development** | `development` | `llmspell -p development run agent.lua` |
| **Document Search** | `rag-dev` | `llmspell -p rag-dev run doc-search.lua` |
| **Production RAG** | `rag-prod` | `llmspell -p rag-prod run knowledge-base.lua` |
| **Memory System** | `memory` | `llmspell -p memory run chatbot.lua` |
| **Full Local Stack** | `full-local-ollama` | `llmspell -p full-local-ollama run offline-app.lua` |
| **Production Service** | `daemon-prod` | `llmspell -p daemon-prod kernel start` |

### By Category

**Backward Compatible (12 presets)**:
- Core: `minimal`, `development`, `default`
- Providers: `providers`, `ollama`, `candle`
- State: `state`, `sessions`
- Memory: `memory`
- RAG: `rag-dev`, `rag-prod`, `rag-perf`

**New Combinations (8 presets)**:
- Production: `postgres-prod`, `daemon-dev`, `daemon-prod`
- Provider-specific: `gemini-prod`, `openai-prod`, `claude-prod`
- Local: `full-local-ollama`
- Research: `research`

---

## Preset Catalog

### Backward-Compatible Presets

These presets maintain backward compatibility with v0.13.0 and earlier.

#### `minimal`

**Extends**: `bases/cli`, `features/minimal`, `backends/memory`

**Purpose**: Tools and workflows only, no LLM features

**Use Cases**:
- Testing tool functionality
- Learning workflow patterns
- Scripts that don't need LLM access

**Example**:
```bash
llmspell -p minimal run examples/tools/file-operations.lua
```

**What's Enabled**: Tool execution, basic workflows
**What's Disabled**: LLM providers, RAG, memory, graph, state persistence

---

#### `development`

**Extends**: `bases/cli`, `features/llm`, `envs/dev`, `backends/memory`

**Purpose**: Full development environment with debug logging

**Use Cases**:
- Development
- Debugging
- Comprehensive logging and tracing

**Example**:
```bash
export OPENAI_API_KEY="sk-..."
llmspell -p development run examples/agents/simple-agent.lua
```

**What's Enabled**: All LLM providers, debug logging, in-memory state
**What's Disabled**: Persistence, RAG, memory system

---

#### `providers`

**Extends**: `bases/cli`, `features/llm`, `backends/memory`

**Purpose**: Simple OpenAI + Anthropic + Gemini agent setup

**Use Cases**:
- Agent examples
- Basic LLM scripts
- Getting started with cloud providers

**Example**:
```bash
export OPENAI_API_KEY="sk-..."
export ANTHROPIC_API_KEY="sk-ant-..."
llmspell -p providers run examples/agents/multi-agent.lua
```

**What's Enabled**: All cloud LLM providers (OpenAI, Anthropic, Gemini)
**What's Disabled**: Persistence, RAG, memory

---

#### `state`

**Extends**: `bases/cli`, `features/state`, `backends/memory`

**Purpose**: State persistence with memory backend

**Use Cases**:
- State management examples
- Data persistence across runs (in-memory)
- Learning state patterns

**Example**:
```bash
llmspell -p state run examples/state/stateful-workflow.lua
```

**What's Enabled**: State persistence, sessions, hooks, events
**What's Disabled**: LLM providers, RAG, disk persistence

---

#### `sessions`

**Extends**: `bases/cli`, `features/state`, `backends/sqlite`

**Purpose**: Full session management (state + hooks + events)

**Use Cases**:
- Multi-session workflows
- Session artifacts
- Comprehensive state with SQLite persistence

**Example**:
```bash
llmspell -p sessions run examples/sessions/session-artifacts.lua
```

**What's Enabled**: State persistence (SQLite), sessions, hooks, events, artifacts
**What's Disabled**: LLM providers, RAG, memory

---

#### `ollama`

**Extends**: `bases/cli`, `features/local`, `backends/sqlite`

**Purpose**: Local Ollama models

**Use Cases**:
- Offline development
- Privacy-focused applications
- Local LLM experimentation

**Requirements**: Ollama running locally (`ollama serve`)

**Example**:
```bash
ollama serve &
llmspell -p ollama run examples/ollama/local-chat.lua
```

**What's Enabled**: Ollama provider, Candle provider, SQLite persistence
**What's Disabled**: Cloud providers, RAG, memory

---

#### `candle`

**Extends**: `bases/cli`, `features/local`, `backends/sqlite`

**Purpose**: Local Candle ML models

**Use Cases**:
- Embedded ML
- Local inference
- Air-gapped environments

**Example**:
```bash
llmspell -p candle run examples/candle/ml-inference.lua
```

**What's Enabled**: Candle provider, Ollama provider, SQLite persistence
**What's Disabled**: Cloud providers, RAG, memory

---

#### `memory`

**Extends**: `bases/cli`, `features/memory`, `envs/dev`, `backends/sqlite`

**Purpose**: Adaptive memory system (Phase 13)

**Use Cases**:
- Conversational agents
- Context retention across interactions
- Memory experiments

**Example**:
```bash
export OPENAI_API_KEY="sk-..."
llmspell -p memory run examples/memory/chatbot-with-memory.lua
```

**What's Enabled**: 3-tier memory system, debug logging, SQLite persistence
**What's Disabled**: RAG, graph (use `full` for these)

---

#### `rag-dev`

**Extends**: `bases/cli`, `features/rag`, `envs/dev`, `backends/sqlite`

**Purpose**: RAG development with trace logging

**Use Cases**:
- Developing document search
- Debugging vector storage
- Experimenting with embeddings

**Example**:
```bash
export OPENAI_API_KEY="sk-..."  # For embeddings
llmspell -p rag-dev run examples/rag/document-indexing.lua
```

**What's Enabled**: Vector storage (HNSW), embeddings, debug logging, SQLite
**What's Disabled**: LLM providers (except for embeddings), memory, graph

---

#### `rag-prod`

**Extends**: `bases/cli`, `features/rag`, `envs/prod`, `backends/sqlite`

**Purpose**: RAG production with SQLite

**Use Cases**:
- Production document search
- Knowledge bases
- Semantic retrieval systems

**Example**:
```bash
export OPENAI_API_KEY="sk-..."
llmspell -p rag-prod run knowledge-base.lua
```

**What's Enabled**: Vector storage, production logging (warn), SQLite
**What's Disabled**: Debug logging, LLM providers (except embeddings)

---

#### `rag-perf`

**Extends**: `bases/cli`, `features/rag`, `envs/perf`, `backends/sqlite`

**Purpose**: RAG performance tuned

**Use Cases**:
- Benchmarking vector search
- Load testing retrieval
- Performance optimization

**Example**:
```bash
llmspell -p rag-perf run benchmarks/rag-performance.lua
```

**What's Enabled**: Vector storage, minimal logging (error), SQLite
**What's Disabled**: Debug output, unnecessary features

---

#### `default`

**Extends**: `bases/cli`, `features/minimal`, `backends/memory`

**Purpose**: Minimal CLI setup (same as `minimal`)

**Use Case**: Default when no profile specified

**Example**:
```bash
llmspell run script.lua  # Uses 'default' profile automatically
```

---

### New Combination Presets

These presets showcase advanced layer combinations new in v0.14.0.

#### `postgres-prod`

**Extends**: `bases/daemon`, `features/full`, `envs/prod`, `backends/postgres`

**Purpose**: Production daemon with PostgreSQL backend

**Use Cases**:
- Multi-instance production deployments
- Enterprise systems
- Scalable production services

**Requirements**: PostgreSQL with `pgvector` extension

**Example**:
```bash
export DATABASE_URL="postgresql://user:pass@localhost/llmspell"
llmspell -p postgres-prod kernel start --port 9555
```

**What's Enabled**: Everything (graph, RAG, memory, context), high concurrency (100), PostgreSQL

---

#### `daemon-dev`

**Extends**: `bases/daemon`, `features/full`, `envs/dev`, `backends/sqlite`

**Purpose**: Daemon mode development

**Use Cases**:
- Developing long-running services
- Testing concurrency
- Service development with debug logging

**Example**:
```bash
llmspell -p daemon-dev kernel start --port 9555
```

**What's Enabled**: Everything, high concurrency, debug logging, SQLite

---

#### `daemon-prod`

**Extends**: `bases/daemon`, `features/full`, `envs/prod`, `backends/postgres`

**Purpose**: Production daemon (alias for `postgres-prod`)

**Use Case**: Production service deployments

**Example**:
```bash
export DATABASE_URL="postgresql://user:pass@localhost/llmspell"
llmspell -p daemon-prod kernel start --port 9555
```

**What's Enabled**: Everything, high concurrency (100), production logging, PostgreSQL

---

#### `gemini-prod`

**Extends**: `bases/cli`, `features/full`, `envs/prod`, `backends/sqlite`

**Overrides**: `providers.default_provider = "gemini"`

**Purpose**: Full Phase 13 stack with Gemini as default provider

**Use Case**: Google Gemini production deployments

**Requirements**: `GEMINI_API_KEY` environment variable

**Example**:
```bash
export GEMINI_API_KEY="..."
llmspell -p gemini-prod run gemini-app.lua
```

**What's Enabled**: Graph, RAG, memory, context, Gemini default provider

---

#### `openai-prod`

**Extends**: `bases/cli`, `features/full`, `envs/prod`, `backends/sqlite`

**Overrides**: `providers.default_provider = "openai"`

**Purpose**: Full Phase 13 stack with OpenAI as default provider

**Use Case**: OpenAI production deployments

**Requirements**: `OPENAI_API_KEY` environment variable

**Example**:
```bash
export OPENAI_API_KEY="sk-..."
llmspell -p openai-prod run openai-app.lua
```

**What's Enabled**: Graph, RAG, memory, context, OpenAI default provider

---

#### `claude-prod`

**Extends**: `bases/cli`, `features/full`, `envs/prod`, `backends/sqlite`

**Overrides**: `providers.default_provider = "anthropic"`

**Purpose**: Full Phase 13 stack with Claude/Anthropic as default provider

**Use Case**: Anthropic Claude production deployments

**Requirements**: `ANTHROPIC_API_KEY` environment variable

**Example**:
```bash
export ANTHROPIC_API_KEY="sk-ant-..."
llmspell -p claude-prod run claude-app.lua
```

**What's Enabled**: Graph, RAG, memory, context, Anthropic default provider

---

#### `full-local-ollama`

**Extends**: `bases/cli`, `features/full`, `envs/dev`, `backends/sqlite`

**Overrides**: `providers.default_provider = "ollama"`

**Purpose**: Complete local stack (Ollama + SQLite)

**Use Case**: Fully offline AI applications

**Requirements**: Ollama running locally

**Example**:
```bash
ollama serve &
llmspell -p full-local-ollama run offline-full-stack.lua
```

**What's Enabled**: Graph, RAG, memory, context, Ollama default provider, SQLite

---

#### `research`

**Extends**: `bases/cli`, `features/full`, `envs/dev`, `backends/sqlite`

**Overrides**: `debug.level = "trace"`

**Purpose**: Full features with trace logging for research and debugging

**Use Case**: Research experiments, deep debugging, learning internals

**Example**:
```bash
llmspell -p research run experiment.lua 2>&1 | tee research.log
```

**What's Enabled**: Graph, RAG, memory, context, trace-level logging, SQLite

---

## Usage Patterns

### Development Workflow

**1. Start with minimal**:
```bash
llmspell -p minimal run prototype.lua
```

**2. Add LLM capabilities**:
```bash
llmspell -p providers run agent-prototype.lua
```

**3. Develop with full features**:
```bash
llmspell -p development run full-app.lua
```

**4. Test in daemon mode**:
```bash
llmspell -p daemon-dev kernel start
```

**5. Deploy to production**:
```bash
llmspell -p daemon-prod kernel start
```

### Choosing the Right Preset

**Ask yourself**:

1. **Do I need LLM access?**
   - No → `minimal`
   - Yes → Continue to #2

2. **Which provider?**
   - Cloud (OpenAI/Anthropic/Gemini) → `providers` or `development`
   - Local (Ollama) → `ollama` or `full-local-ollama`
   - Specific provider → `openai-prod`, `gemini-prod`, `claude-prod`

3. **Do I need document search (RAG)?**
   - No → Use current preset
   - Yes → `rag-dev` or `rag-prod`

4. **Do I need persistent memory?**
   - No → Use current preset
   - Yes → `memory` or preset with `features/full`

5. **What's my deployment mode?**
   - Interactive CLI → Current preset works
   - Long-running service → `daemon-dev` or `daemon-prod`

6. **What environment?**
   - Development → Preset with `envs/dev` or `development`
   - Production → Preset with `envs/prod` or create custom

### Environment Variables

Many presets require environment variables:

**Cloud providers**:
```bash
export OPENAI_API_KEY="sk-..."
export ANTHROPIC_API_KEY="sk-ant-..."
export GEMINI_API_KEY="..."
```

**Database backends**:
```bash
# PostgreSQL presets
export DATABASE_URL="postgresql://user:pass@localhost:5432/llmspell"

# SQLite presets (optional override)
export LLMSPELL_DB_PATH="/custom/path/llmspell.db"
```

**Ollama**:
```bash
# Default: http://localhost:11434
export OLLAMA_HOST="http://localhost:11434"
```

---

## Creating Custom Presets

### Preset File Format

```toml
# Preset description comment

[profile]
name = "Preset Name"
description = "Brief description of purpose"
extends = ["layer1", "layer2", "layer3", "layer4"]

# Optional: Override specific settings
[section]
setting = "value"
```

### Example: QA Testing Preset

Create `presets/qa.toml`:

```toml
# QA Testing Preset
# Optimized for quality assurance testing environments

[profile]
name = "QA Testing"
description = "Full features with QA-optimized logging and SQLite persistence"
extends = ["bases/cli", "features/full", "envs/staging", "backends/sqlite"]

# Override for QA-specific settings
[debug]
level = "info"

[runtime]
max_concurrent_scripts = 20
```

Then use it:
```bash
llmspell -p qa run qa-test-suite.lua
```

---

## Troubleshooting

### Preset Not Found

**Error**: `Unknown builtin profile: xyz`

**Solution**: Check available presets with:
```bash
llmspell --list-profiles
```

Or see the [Quick Reference](#quick-reference) above.

### Missing API Key

**Error**: `Provider error: Missing API key for OpenAI`

**Solution**: Set required environment variable:
```bash
export OPENAI_API_KEY="sk-..."
```

See [Environment Variables](#environment-variables) section.

### Database Connection Error

**PostgreSQL presets error**: `connection refused`

**Solution**:
1. Ensure PostgreSQL is running: `pg_isready`
2. Verify connection string: `echo $DATABASE_URL`
3. Check database exists: `psql $DATABASE_URL -c '\l'`

**SQLite presets error**: `unable to open database`

**Solution**: Check write permissions in directory or set custom path:
```bash
export LLMSPELL_DB_PATH="/tmp/llmspell.db"
```

---

## See Also

- **[Layer Catalog](../layers/README.md)** - 18 reusable layers used by these presets
- **[Profile Layers Guide](../../docs/user-guide/profile-layers-guide.md)** - Comprehensive user guide
- **[Configuration Guide](../../docs/user-guide/03-configuration.md)** - Full configuration reference
- **[CLI Reference](../../docs/user-guide/05-cli-reference.md)** - Command-line interface documentation

---

**Phase**: 13c.4 (Storage Consolidation)
**Date**: January 2025
**Total Presets**: 20 files (12 backward-compatible + 8 new combinations)
