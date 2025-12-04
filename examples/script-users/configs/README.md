# LLMSpell Configuration Examples

**Phase**: 13c.5.5 - Config Audit Complete
**Status**: 4 unique custom configuration patterns preserved

## 🎯 Primary Approach: Use Builtin Profiles

**For 80%+ of use cases, use builtin profiles** (`-p profile-name`) instead of custom config files:

```bash
# Recommended: Use builtin profiles
llmspell -p minimal run script.lua                    # Tools only, no providers
llmspell -p providers run script.lua                  # OpenAI/Anthropic agents
llmspell -p ollama-production run script.lua          # Local LLM production
llmspell -p postgres run script.lua                   # PostgreSQL backend
llmspell -p memory-development run script.lua         # Phase 13 memory debugging
llmspell -p rag-dev run script.lua                    # RAG development
llmspell -p rag-prod run script.lua                   # RAG production
llmspell -p state run script.lua                      # State persistence
```

**See all builtin profiles**: `llmspell-config/builtins/README.md`

## 🔧 When to Use Custom Configs

This directory contains **4 unique configuration templates** for advanced scenarios that builtin profiles don't cover:

| File | Purpose | When to Use |
|------|---------|-------------|
| **applications.toml** | Production app configuration | Multi-provider rate limiting, app-specific security policies |
| **backup-enabled.toml** | Custom backup schedules | Non-standard backup intervals, custom backup paths |
| **migration-enabled.toml** | State migration settings | Database schema evolution, custom migration workflows |
| **rag-multi-tenant.toml** | Multi-tenant RAG | Isolated vector stores per tenant, tenant-specific indexing |

## 📊 Decision Matrix

| Use Case | Solution | Command |
|----------|----------|---------|
| **Development** | ✅ Builtin Profile | `llmspell -p development run script.lua` |
| **Production Local LLM** | ✅ Builtin Profile | `llmspell -p ollama-production run script.lua` |
| **RAG Development** | ✅ Builtin Profile | `llmspell -p rag-dev run script.lua` |
| **State Persistence** | ✅ Builtin Profile | `llmspell -p state run script.lua` |
| **PostgreSQL Backend** | ✅ Builtin Profile | `llmspell -p postgres run script.lua` |
| **Multi-tenant RAG** | ❌ Custom Config | `LLMSPELL_CONFIG=configs/rag-multi-tenant.toml llmspell run script.lua` |
| **Custom Backup Schedule** | ❌ Custom Config | `LLMSPELL_CONFIG=configs/backup-enabled.toml llmspell run script.lua` |
| **Production Apps** | ❌ Custom Config | `LLMSPELL_CONFIG=configs/applications.toml llmspell run script.lua` |
| **State Migration** | ❌ Custom Config | `LLMSPELL_CONFIG=configs/migration-enabled.toml llmspell run script.lua` |

## 📁 Configuration Details

### applications.toml

**Production Application Configuration**

```toml
# Key Features:
- Multiple provider support (OpenAI, Anthropic, Gemini)
- Rate limiting and retry logic
- Production security levels
- Application-specific resource limits
```

**Use Case**: Running production application examples (webapp-creator, content-creator, etc.)

**Example**:
```bash
LLMSPELL_CONFIG=examples/script-users/configs/applications.toml \
  llmspell run examples/script-users/applications/webapp-creator/main.lua
```

### backup-enabled.toml

**Custom Backup Schedule Configuration**

```toml
# Key Features:
- Configurable backup intervals
- Custom backup directory paths
- Backup retention policies
- Automatic cleanup schedules
```

**Use Case**: Applications requiring non-standard backup intervals or custom backup paths

**Example**:
```bash
LLMSPELL_CONFIG=examples/script-users/configs/backup-enabled.toml \
  llmspell run your-stateful-app.lua
```

### migration-enabled.toml

**State Migration Configuration**

```toml
# Key Features:
- Schema evolution tracking
- Migration version management
- Rollback capabilities
- Data transformation rules
```

**Use Case**: Applications with evolving state schemas requiring migration support

**Example**:
```bash
LLMSPELL_CONFIG=examples/script-users/configs/migration-enabled.toml \
  llmspell run migration-example.lua
```

### rag-multi-tenant.toml

**Multi-Tenant RAG Configuration**

```toml
# Key Features:
- Tenant-specific vector stores
- Isolated indexing per tenant
- Per-tenant embedding models
- Tenant data isolation
```

**Use Case**: SaaS applications with tenant-specific knowledge bases

**Example**:
```bash
LLMSPELL_CONFIG=examples/script-users/configs/rag-multi-tenant.toml \
  llmspell run multi-tenant-rag.lua
```

## 📚 Additional Resources

### Documentation
- **[Builtin Profiles](../../../llmspell-config/builtins/README.md)** - Complete list of 17+ builtin profiles
- **[MIGRATION.md](MIGRATION.md)** - Guide for migrating between configurations
- **[Configuration Architecture](../../../docs/user-guide/01-getting-started.md)** - Profile system overview

### Scripts
- **validate-rag-configs.sh** - Validates RAG configuration files

## 🗂️ Archived Configurations

Redundant configurations have been moved to `archived/`:
- **llmspell.toml** → Use `-p minimal` instead (tools-only configuration)

These were replaced by builtin profiles in Phase 13c.5.5 to reduce configuration sprawl.

## ⚠️ Migration Notes

If you have existing scripts using old config files:

```bash
# Old approach (deprecated)
LLMSPELL_CONFIG=configs/llmspell.toml llmspell run script.lua

# New approach (recommended)
llmspell -p minimal run script.lua
```

**Builtin profiles are:**
- Faster to load (compiled into binary)
- Better tested (CI/CD validation)
- Always up-to-date (no manual sync)
- Easier to use (no config paths)

## 📋 Summary

**Current State** (Phase 13c.5.5):
- ✅ 4 unique custom configs preserved
- ✅ 1 redundant config archived
- ✅ 17+ builtin profiles available
- ✅ 80%+ examples use builtin profiles
- ✅ Clear decision matrix provided

**Recommendation**: Start with builtin profiles, only use custom configs for the 4 unique patterns documented above.
