# Configuration Cleanup Analysis - Phase 11b.3

**Date**: 2025-10-10
**Purpose**: Comprehensive audit of all config files across the project to identify consolidation opportunities using the new builtin profile system.

## Executive Summary

**Current State**: 38 total config files (7 builtins + 17 examples + 10 applications + 4 fleet)
**Recommendation**: Add 3 new builtins, remove 7 duplicate configs, update 46+ lua files and READMEs
**Impact**: Cleaner examples, better demonstration of `-p` flag, reduced maintenance burden

---

## Current State

### Builtin Profiles (7) - llmspell-config/builtins/
1. **minimal.toml** (11 lines) - Bare minimum, no providers, no RAG
2. **development.toml** (30 lines) - OpenAI/Anthropic providers, debug logging, full stdlib
3. **ollama.toml** (21 lines) - Local Ollama LLM backend
4. **candle.toml** (25 lines) - Local Candle LLM backend (Rust-native)
5. **rag-development.toml** (75 lines) - RAG development with debug features
6. **rag-production.toml** (91 lines) - RAG production settings
7. **rag-performance.toml** (69 lines) - RAG performance tuning

### Example Configs (17) - examples/script-users/configs/

#### DUPLICATES (7 configs - candidates for removal)
These are duplicates or near-duplicates of builtin profiles:

1. **minimal.toml** (13 lines) - Slightly different structure from builtin
2. **rag-development.toml** (87 lines) - More comments, uses deprecated structure `[providers.providers.name]`
3. **rag-production.toml** (83 lines) - More comments, uses deprecated structure
4. **rag-performance.toml** (66 lines) - More comments, uses deprecated structure
5. **local-llm-ollama.toml** (65 lines) - Verbose commented version of builtin ollama
6. **local-llm-candle.toml** (60 lines) - Verbose commented version of builtin candle
7. **cookbook.toml** (33 lines) - Nearly identical to development.toml

#### UNIQUE CONFIGS (10 configs - need analysis)
These serve specific purposes not covered by builtins:

1. **example-providers.toml** (29 lines) - Simple OpenAI/Anthropic provider demo
2. **llmspell.toml** (13 lines) - Tool testing only, no providers
3. **basic.toml** (32 lines) - State persistence basics with memory backend
4. **state-enabled.toml** (50 lines) - Full state configuration with detailed settings
5. **session-enabled.toml** (68 lines) - Session + state + hooks + events enabled
6. **backup-enabled.toml** - State with backup features enabled
7. **migration-enabled.toml** - State with migration features enabled
8. **applications.toml** (136 lines) - Complex app-specific configs with cost management
9. **rag-basic.toml** - Simplified RAG intro (simpler than rag-development)
10. **rag-multi-tenant.toml** - Multi-tenant RAG isolation patterns

### Application Configs (10) - examples/script-users/applications/*/config.toml
Each application has its own config.toml with app-specific settings:
- code-review-assistant, communication-manager, content-creator, file-organizer
- instrumented-agent, knowledge-base, personal-assistant, process-orchestrator
- research-collector, webapp-creator

**Note**: Application configs should remain - they demonstrate different configuration patterns for specific use cases.

### Fleet Configs (4) - scripts/fleet/configs/
Fleet-specific configs for distributed kernel management:
- anthropic.toml, default.toml, local.toml, openai.toml

**Note**: Fleet configs are operational and should remain unchanged.

---

## Lua File Config Requirements

Analyzed 46 lua files across examples to map config dependencies:

### Getting Started (6 files)
| File | Config Requirement | Current Config | Builtin Alternative |
|------|-------------------|----------------|---------------------|
| 00-hello-world.lua | NONE | - | -p minimal |
| 01-first-tool.lua | NONE | - | -p minimal |
| 02-first-agent.lua | Providers | example-providers.toml | **NEEDS: -p providers** |
| 03-first-workflow.lua | NONE | - | -p minimal |
| 04-handle-errors.lua | OPTIONAL State | state-enabled.toml | **NEEDS: -p state** |
| 05-first-rag.lua | RAG | rag-basic.toml | -p rag-dev (or new -p rag) |

### Features (5 files)
| File | Config Requirement | Current Config | Builtin Alternative |
|------|-------------------|----------------|---------------------|
| agent-basics.lua | Providers | example-providers.toml | **NEEDS: -p providers** |
| provider-info.lua | OPTIONAL Providers | any provider config | -p development |
| state-persistence.lua | State | state-enabled.toml | **NEEDS: -p state** |
| tool-basics.lua | NONE | - | -p minimal |
| workflow-basics.lua | NONE | - | -p minimal |

### Cookbook (12 files)
| File | Config Requirement | Current Config | Builtin Alternative |
|------|-------------------|----------------|---------------------|
| error-handling.lua | NONE | - | -p minimal |
| rate-limiting.lua | NONE | - | -p minimal |
| caching.lua | NONE | - | -p minimal |
| multi-agent-coordination.lua | Providers | example-providers.toml | **NEEDS: -p providers** |
| webhook-integration.lua | Providers | cookbook.toml | -p development |
| performance-monitoring.lua | NONE | - | -p minimal |
| security-patterns.lua | NONE | - | -p minimal |
| state-management.lua | State | state-enabled.toml | **NEEDS: -p state** |
| rag-multi-tenant.lua | RAG Multi-tenant | rag-multi-tenant.toml | KEEP (unique pattern) |
| rag-session.lua | Sessions + RAG | session-enabled.toml | **NEEDS: -p sessions** |
| rag-cost-optimization.lua | RAG Production | rag-production.toml | -p rag-prod |
| sandbox-permissions.lua | NONE | - | -p minimal |

### Applications (10 apps)
All applications have their own configs and should keep them - they demonstrate configuration patterns.

### Tests (3 files)
- test-rag-basic.lua, test-rag-e2e.lua, test-rag-errors.lua - use RAG configs

---

## Gap Analysis

### Missing Builtins (3 recommended additions)

#### 1. **providers.toml** - Simple Provider Setup
**Purpose**: Basic OpenAI/Anthropic providers without RAG, state, or advanced features
**Replaces**: example-providers.toml, cookbook.toml
**Used by**: 5+ lua files (agent-basics, multi-agent-coordination, etc.)

```toml
# Simple Providers Profile
# Basic OpenAI/Anthropic setup for agent examples
default_engine = "lua"

[engines.lua]
stdlib = "All"

[providers.openai]
provider_type = "openai"
api_key_env = "OPENAI_API_KEY"
default_model = "gpt-3.5-turbo"
temperature = 0.7
max_tokens = 2000

[providers.anthropic]
provider_type = "anthropic"
api_key_env = "ANTHROPIC_API_KEY"
default_model = "claude-3-haiku-20240307"
temperature = 0.7
max_tokens = 2000

[runtime]
log_level = "info"
```

#### 2. **state.toml** - State Persistence Enabled
**Purpose**: State persistence with memory backend, no providers
**Replaces**: basic.toml, state-enabled.toml
**Used by**: 3+ lua files (state-persistence, state-management, error-handling)

```toml
# State Persistence Profile
# Enables State global with memory backend
default_engine = "lua"

[engines.lua]
stdlib = "All"

[runtime]
log_level = "info"

[runtime.state_persistence]
enabled = true
backend_type = "memory"
max_state_size_bytes = 10_000_000  # 10MB
migration_enabled = false
backup_enabled = false
```

#### 3. **sessions.toml** - Sessions + State + Hooks
**Purpose**: Full session management with state, hooks, and events
**Replaces**: session-enabled.toml
**Used by**: rag-session.lua and other session examples

```toml
# Sessions Profile
# Sessions + state + hooks + events enabled
default_engine = "lua"

[engines.lua]
stdlib = "All"

[runtime]
log_level = "info"

[runtime.state_persistence]
enabled = true
backend_type = "memory"
max_state_size_bytes = 10_000_000

[runtime.sessions]
enabled = true
max_sessions = 100
max_artifacts_per_session = 1000
session_timeout_seconds = 3600
storage_backend = "memory"

[runtime.hooks]
enabled = true
max_hooks = 100

[runtime.events]
enabled = true
max_subscribers = 100
event_buffer_size = 1000
```

---

## Proposed Changes

### Phase 1: Add 3 New Builtins
Create 3 new builtin profiles in `llmspell-config/builtins/`:
1. `providers.toml` - Simple provider setup
2. `state.toml` - State persistence
3. `sessions.toml` - Full session support

**Total builtins**: 7 → 10

### Phase 2: Remove 7 Duplicate Configs
Delete from `examples/script-users/configs/`:
1. minimal.toml (use `-p minimal`)
2. rag-development.toml (use `-p rag-dev`)
3. rag-production.toml (use `-p rag-prod`)
4. rag-performance.toml (use `-p rag-perf`)
5. local-llm-ollama.toml (use `-p ollama`)
6. local-llm-candle.toml (use `-p candle`)
7. cookbook.toml (use `-p development` or new `-p providers`)

**Consider removing** (after new builtins):
- example-providers.toml (replaced by new `-p providers`)
- basic.toml (replaced by new `-p state`)
- state-enabled.toml (replaced by new `-p state`)
- session-enabled.toml (replaced by new `-p sessions`)
- llmspell.toml (use `-p minimal`)

**Potential removals**: 7-12 configs
**Example configs remaining**: 5-10 configs (keeping unique patterns)

### Phase 3: Update 46+ Lua Files
Update header comments in all lua files to use `-p` flags:

**Before**:
```lua
-- Usage: ./llmspell -c examples/script-users/configs/example-providers.toml run agent-basics.lua
```

**After**:
```lua
-- Usage: ./llmspell -p providers run agent-basics.lua
-- Or: ./llmspell -p development run agent-basics.lua  # with debug logging
```

**Files to update**:
- getting-started/ (6 files)
- features/ (5 files)
- cookbook/ (12 files)
- Top-level examples/ (4 files)
- Application main.lua files (10 files)
- Test files (3 files)

**Total**: ~40 lua files

### Phase 4: Update READMEs and Documentation
Update documentation to demonstrate builtin profile usage:

**Files to update**:
1. `examples/script-users/README.md` - Update all examples to use `-p`
2. `examples/script-users/getting-started/README.md` - Update run commands
3. `examples/script-users/features/README.md` - Update config references
4. `examples/script-users/cookbook/README.md` - Update patterns
5. `examples/script-users/configs/README.md` - Document remaining configs
6. `examples/README.md` - Top-level examples guide
7. `docs/user-guide/configuration.md` - Update config guide

**Total**: 7+ README files

---

## Benefits

### User Experience
- **Simpler commands**: `llmspell -p providers run script.lua` vs full path
- **Clearer examples**: Fewer config files to understand
- **Better discovery**: Built-in profiles are documented and predictable
- **Faster onboarding**: New users see builtin profiles first

### Maintenance
- **Less duplication**: Single source of truth for common patterns
- **Easier updates**: Update builtin once vs multiple example configs
- **Better testing**: Builtin profiles are tested as part of llmspell-config
- **Clear separation**: Examples focus on unique patterns, not basic setup

### Documentation
- **Demonstrates `-p` flag**: Shows proper usage of unified profile system (Phase 11b.3)
- **Clearer purpose**: Remaining example configs show specific patterns
- **Better examples**: Lua files are easier to read with shorter command lines

---

## Risks and Mitigation

### Risk 1: Breaking Existing User Scripts
**Mitigation**: Keep backward compatibility by leaving a few key configs as "deprecated but functional" with comments pointing to builtins

### Risk 2: Losing Useful Comments
**Mitigation**: Preserve valuable comments from example configs in:
- Builtin profile comments
- Dedicated documentation (docs/user-guide/configuration.md)
- Example-specific README files

### Risk 3: Application Configs Too Specific
**Mitigation**: Keep all application configs - they demonstrate real-world patterns

---

## Implementation Strategy

### Recommended Approach: Phased Migration

**Phase 1: Add New Builtins** (Task 11b.4.1)
- Create 3 new builtin profiles with comprehensive comments
- Test with existing examples
- Update llmspell-config tests

**Phase 2: Update Examples** (Task 11b.4.2)
- Update lua file headers to use `-p` flags
- Keep example configs with deprecation warnings
- Test all examples with new builtins

**Phase 3: Update Documentation** (Task 11b.4.3)
- Update all README files
- Update docs/user-guide/
- Create migration guide

**Phase 4: Remove Duplicates** (Task 11b.4.4)
- Remove duplicate configs after verification
- Update .gitignore if needed
- Final testing pass

---

## Alternative Strategies

### Strategy B: Keep All Configs, Update Instructions Only
- **Pro**: No breaking changes, preserves comments
- **Con**: Doesn't demonstrate builtin system, maintains complexity
- **Verdict**: Not recommended - defeats purpose of Phase 11b.3

### Strategy C: Aggressive Removal Without New Builtins
- **Pro**: Immediate simplification
- **Con**: Gaps in coverage, forces users to write configs
- **Verdict**: Not recommended - creates friction

---

## Conclusion

**Recommendation**: Implement Strategy A (Phased Migration)

**Effort Estimate**:
- Add 3 builtins: 2-3 hours
- Update 46 lua files: 3-4 hours
- Update 7 READMEs: 2-3 hours
- Testing: 2 hours
- **Total**: 9-12 hours

**Impact**: High - significantly improves user experience and demonstrates Phase 11b.3 unified profile system

**Next Step**: Create TODO.md task 11b.4 with 4 sub-tasks for phased implementation
