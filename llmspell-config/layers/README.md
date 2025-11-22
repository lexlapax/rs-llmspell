# Layer Architecture

**Version**: 0.14.0
**Phase**: 13c.4 (Storage Consolidation)
**Module**: `llmspell-config/layers`

> **Purpose**: Composable configuration layers for flexible profile management.

---

## Overview

This directory contains the **4-layer profile architecture** for LLMSpell configuration. Layers are reusable TOML configuration fragments that can be composed together to create complete application profiles.

### Why Layers?

Instead of maintaining separate monolithic configuration files for different scenarios (dev, prod, rag, minimal, etc.), we use composable layers that can be mixed and matched:

- **Reusability**: Share common configuration across profiles
- **Maintainability**: Update one layer, affects all profiles using it
- **Clarity**: Each layer has a single, well-defined purpose
- **Flexibility**: Compose layers in any combination to create custom profiles

### The 4-Layer System

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

---

## Directory Structure

```
layers/
├── bases/           # Deployment modes (4 files)
│   ├── cli.toml
│   ├── daemon.toml
│   ├── embedded.toml
│   └── testing.toml
├── features/        # Capability sets (7 files)
│   ├── minimal.toml
│   ├── llm.toml
│   ├── rag.toml
│   ├── memory.toml
│   ├── state.toml
│   ├── full.toml
│   └── local.toml
├── envs/            # Environment tuning (4 files)
│   ├── dev.toml
│   ├── staging.toml
│   ├── prod.toml
│   └── perf.toml
└── backends/        # Storage backends (3 files)
    ├── memory.toml
    ├── sqlite.toml
    └── postgres.toml
```

**Total**: 18 reusable layer files.

---

## Layer Types

### 1. Bases (Deployment Modes)

**Purpose**: Define runtime characteristics for different deployment modes.

| Layer | Purpose | Concurrency | State | Sessions |
|-------|---------|-------------|-------|----------|
| `bases/cli` | Interactive CLI execution | 1 | In-memory | 1 |
| `bases/daemon` | Long-running service | 100 | Persistent | 1000 |
| `bases/embedded` | Library integration | 10 | Configurable | 100 |
| `bases/testing` | Automated testing | 20 | Ephemeral | 50 |

**Example**:
```bash
llmspell -p bases/cli,features/minimal run script.lua
```

### 2. Features (Capabilities)

**Purpose**: Enable specific functionality domains.

| Layer | LLM | RAG | Memory | Graph | State |
|-------|-----|-----|--------|-------|-------|
| `features/minimal` | ❌ | ❌ | ❌ | ❌ | ❌ |
| `features/llm` | ✅ | ❌ | ❌ | ❌ | ❌ |
| `features/rag` | ❌ | ✅ | ❌ | ❌ | ❌ |
| `features/memory` | ❌ | ❌ | ✅ | ❌ | ❌ |
| `features/state` | ❌ | ❌ | ❌ | ❌ | ✅ |
| `features/full` | ✅ | ✅ | ✅ | ✅ | ✅ |
| `features/local` | ✅ (Ollama/Candle) | ❌ | ❌ | ❌ | ❌ |

**Example**:
```bash
llmspell -p bases/cli,features/full,envs/dev run app.lua
```

### 3. Environments (Tuning)

**Purpose**: Tune settings for different deployment stages.

| Layer | Log Level | Timeout | Concurrency | Use Case |
|-------|-----------|---------|-------------|----------|
| `envs/dev` | debug | 600s | 10 | Development, debugging |
| `envs/staging` | info | 300s | 50 | Pre-production testing |
| `envs/prod` | warn | 300s | N/A (from base) | Production deployments |
| `envs/perf` | error | 60s | 200 | Performance benchmarking |

**Example**:
```bash
llmspell -p bases/cli,features/rag,envs/prod run app.lua
```

### 4. Backends (Storage)

**Purpose**: Configure persistence and storage layer.

| Layer | Type | Persistence | Scalability | Use Case |
|-------|------|-------------|-------------|----------|
| `backends/memory` | In-memory | ❌ | Single instance | Testing, development |
| `backends/sqlite` | Embedded DB | ✅ | Single instance | Local deployments |
| `backends/postgres` | Shared DB | ✅ | Multi-instance | Production, clustering |

**Example**:
```bash
llmspell -p bases/daemon,features/full,envs/prod,backends/postgres kernel start
```

---

## Usage

### Multi-Layer Composition Syntax

Compose layers using comma-separated paths:

```bash
llmspell -p base,feature,env,backend run script.lua
```

**Standard 4-layer composition**:
```bash
llmspell -p bases/cli,features/full,envs/dev,backends/sqlite run app.lua
```

**Partial composition** (omit layers you don't need):
```bash
# Just base + feature
llmspell -p bases/cli,features/minimal run simple.lua

# Base + feature + environment
llmspell -p bases/cli,features/rag,envs/dev run doc-search.lua
```

### Layer Merge Order

Layers are merged left-to-right, with later layers overriding earlier ones:

```
llmspell -p layer1,layer2,layer3,layer4
           ^^^^^^  ^^^^^^  ^^^^^^  ^^^^^^
           lowest                 highest precedence
```

**Example**:
```bash
# prod overrides dev settings
llmspell -p bases/cli,envs/dev,envs/prod  # Result: prod tuning
```

### Using in Presets

Presets use the `extends` field to specify layers:

```toml
# presets/rag-dev.toml
[profile]
name = "RAG Development"
description = "RAG features with debug logging"
extends = ["bases/cli", "features/rag", "envs/dev", "backends/sqlite"]
```

Then use the preset:
```bash
llmspell -p rag-dev run script.lua
# Equivalent to: llmspell -p bases/cli,features/rag,envs/dev,backends/sqlite ...
```

---

## Layer File Format

Each layer is a TOML file with the following structure:

```toml
# Comment describing the layer's purpose

[profile]
name = "Layer Name"
description = "Brief description of what this layer provides"

# Configuration sections (as needed)
[runtime]
max_concurrent_scripts = 1

[debug]
enabled = true
level = "info"

# ... additional configuration sections
```

### Profile Metadata

The `[profile]` section is **required** and provides metadata:

```toml
[profile]
name = "CLI Base"
description = "Interactive command-line execution with minimal overhead"
```

This metadata is used for:
- Documentation generation
- Layer discovery
- Error messages

### Configuration Sections

Layers can include any valid configuration sections:

- `[runtime]` - Script execution settings
- `[debug]` - Logging and debugging
- `[providers]` - LLM provider configuration
- `[rag]` - RAG system settings
- `[runtime.memory]` - Memory system configuration
- `[runtime.state_persistence]` - State persistence settings
- `[events]` - Event system configuration
- And many more...

See [`llmspell-config/src/lib.rs`](../src/lib.rs) for the complete configuration schema.

---

## Creating New Layers

### Guidelines

**Each layer should**:
1. Have a **single, clear purpose**
2. Focus on **one dimension of variation** (deployment mode, features, tuning, or storage)
3. Include **descriptive comments** explaining settings
4. Provide **sensible defaults** that work in composition

**Each layer should NOT**:
1. Duplicate settings from other layers (unless intentionally overriding)
2. Make assumptions about other layers
3. Include environment-specific secrets (use environment variables)

### Example: Creating a New Environment Layer

Let's say you want to create a `envs/qa.toml` layer for QA testing:

```toml
# QA Environment Layer
# Balanced settings for quality assurance testing

[profile]
name = "QA Environment"
description = "Settings optimized for QA testing and validation"

# Moderate logging for test observation
[debug]
enabled = true
level = "info"

# Reasonable timeouts for test scenarios
[runtime]
script_timeout_seconds = 300
max_concurrent_scripts = 20

# Enable events for test verification
[events]
enabled = true
```

Then use it:
```bash
llmspell -p bases/cli,features/full,envs/qa,backends/sqlite run qa-tests.lua
```

### Naming Conventions

**Layer files**: Use descriptive lowercase names with hyphens for multi-word names
- Good: `cli.toml`, `full.toml`, `rag.toml`
- Avoid: `CLI.toml`, `full_features.toml`, `rag_system.toml`

**Layer names** (in `[profile].name`): Use title case with clear descriptions
- Good: `"CLI Base"`, `"Full Features"`, `"RAG Features"`
- Avoid: `"cli"`, `"FULL"`, `"rag_feat"`

---

## Common Patterns

### Development → Production Progression

**Step 1: Local development**
```bash
llmspell -p bases/cli,features/full,envs/dev,backends/sqlite run app.lua
```

**Step 2: Staging deployment**
```bash
llmspell -p bases/daemon,features/full,envs/staging,backends/postgres kernel start
```

**Step 3: Production deployment**
```bash
llmspell -p bases/daemon,features/full,envs/prod,backends/postgres kernel start
```

### Feature Progression

**Start minimal**:
```bash
llmspell -p bases/cli,features/minimal run test.lua
```

**Add LLM capabilities**:
```bash
llmspell -p bases/cli,features/llm,envs/dev run agent.lua
```

**Add RAG for document search**:
```bash
llmspell -p bases/cli,features/rag,envs/dev,backends/sqlite run doc-app.lua
```

**Full stack**:
```bash
llmspell -p bases/cli,features/full,envs/prod,backends/postgres run production-app.lua
```

### Testing Patterns

**Unit tests** (fast, ephemeral):
```bash
llmspell -p bases/testing,features/minimal,backends/memory run unit-tests.lua
```

**Integration tests** (realistic):
```bash
llmspell -p bases/testing,features/full,backends/sqlite run integration-tests.lua
```

**Performance tests** (stress configuration):
```bash
llmspell -p bases/daemon,features/full,envs/perf,backends/postgres run load-test.lua
```

---

## Troubleshooting

### Layer Not Found

**Error**: `Error loading layer: File not found: layers/features/xyz.toml`

**Solution**: Check layer name and directory:
- Verify file exists: `ls llmspell-config/layers/features/`
- Check spelling: `xyz.toml` vs `xyx.toml`
- Verify correct category: `features/` vs `envs/`

### Unexpected Configuration Values

**Issue**: Configuration value not what you expected

**Debug steps**:
1. **Check layer order**: Later layers override earlier ones
2. **Verify layer contents**: `cat llmspell-config/layers/envs/dev.toml`
3. **Review merge precedence**: See "Layer Merge Order" above

**Example problem**:
```bash
# Problem: debug level is "warn" but you expected "debug"
llmspell -p bases/cli,envs/dev,envs/prod run app.lua
#                     ^^^^^^^^ ^^^^^^^^
#                     debug    PROD WINS (last one)
```

**Fix**: Remove unwanted layer or reorder:
```bash
llmspell -p bases/cli,envs/dev run app.lua  # Now uses dev settings
```

### Merge Conflicts

**Issue**: Two layers define the same setting with different values

**Resolution**: This is **expected behavior**. The merge strategy is:
- **Primitive values**: Last layer wins
- **Nested tables**: Deep merge (recursive)
- **Arrays**: Last layer replaces entire array

If you need different behavior, adjust layer order or create a custom preset that explicitly sets the desired value.

---

## See Also

- **[Preset Catalog](../presets/README.md)** - 20 builtin presets using these layers
- **[Profile Layers Guide](../../docs/user-guide/profile-layers-guide.md)** - Comprehensive user guide
- **[Configuration Guide](../../docs/user-guide/03-configuration.md)** - Full configuration reference
- **[Development Workflow](../../docs/developer-guide/02-development-workflow.md)** - Using layers in development

---

## Architecture Notes

### Implementation

Layers are loaded and merged by the `ProfileComposer` in `llmspell-config/src/profile_composer.rs`:

```rust
pub fn load_multi(&mut self, layer_names: &[&str]) -> Result<LLMSpellConfig> {
    let mut config = LLMSpellConfig::default();
    for layer_name in layer_names {
        let layer = self.load_layer(layer_name)?;
        config = merge(config, layer);
    }
    Ok(config)
}
```

### Extends Resolution

Layers can use the `extends` field to reference other layers:

```toml
[profile]
extends = ["bases/cli", "features/minimal"]
```

The `ProfileComposer` recursively resolves extends with:
- **Circular detection**: Prevents infinite loops
- **Max depth limit**: 10 levels (prevents deep recursion)
- **Visited set tracking**: Ensures each layer loaded only once per chain

### Merge Strategy

Deep merge is implemented in `llmspell-config/src/merge.rs`:

```rust
pub fn merge(base: LLMSpellConfig, overlay: LLMSpellConfig) -> LLMSpellConfig {
    // Recursively merge nested structures
    // Primitive values: overlay wins
    // Tables: deep merge
    // Arrays: overlay replaces
}
```

---

**Phase**: 13c.4 (Storage Consolidation)
**Date**: January 2025
**Total Layers**: 18 files (4 bases + 7 features + 4 envs + 3 backends)
