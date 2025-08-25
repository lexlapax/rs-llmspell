# API Refactoring Priority List

**Date**: August 1, 2025
**Version**: 1.0
**Target Release**: 0.6.0

## Priority Levels

- **P0**: Critical - Core API improvements (clean break approach)
- **P1**: High - Naming inconsistencies that affect user experience
- **P2**: Medium - Missing features that improve ergonomics
- **P3**: Low - Documentation and polish improvements

---

## P0: Critical - Core API Improvements

### P0.1: Remove Service Suffix
**Impact**: Clean break - Direct improvement
**Effort**: Low - Simple renaming
**Files**: 
- `llmspell-hooks/src/executor.rs`
- `llmspell-events/src/bus.rs`

**Changes**:
```rust
// Before
pub struct HookExecutorService { }
pub struct EventBusService { }

// After (direct replacement)
pub struct HookExecutor { }
pub struct EventBus { }
```

**Implementation**: Direct rename, update all references

---

## P1: High Priority - Naming Inconsistencies

### P1.1: Standardize Getter Methods
**Impact**: Medium - Common API pattern
**Effort**: Low - Simple renaming
**Files**: `llmspell-sessions/src/manager.rs`

**Changes**:
```rust
// Before
pub async fn retrieve_session(&self, id: &SessionId) -> Result<Session>
pub async fn retrieve_artifact(&self, id: &ArtifactId) -> Result<Artifact>

// After
pub async fn get_session(&self, id: &SessionId) -> Result<Session>
pub async fn get_artifact(&self, id: &ArtifactId) -> Result<Artifact>
```

**Implementation**: Direct rename, no compatibility needed

### P1.2: Constructor Consistency
**Impact**: Low - Mostly internal
**Effort**: Low
**Files**: Various

**Changes**:
```rust
// Standardize on these patterns:
new() -> Self                    // Simple construction
with_config(config) -> Self      // With configuration  
from_parts(...) -> Self          // From components
builder() -> Builder             // Builder pattern
```

---

## P2: Medium Priority - Missing Features

### P2.1: Add Builder for SessionManagerConfig
**Impact**: High - Improves usability
**Effort**: Medium - New code
**Files**: `llmspell-sessions/src/config.rs`

**Implementation**:
```rust
pub struct SessionManagerConfigBuilder {
    max_sessions: Option<usize>,
    retention_policy: Option<RetentionPolicy>,
    storage_backend: Option<Arc<dyn StorageBackend>>,
    auto_save_interval: Option<Duration>,
}

impl SessionManagerConfig {
    pub fn builder() -> SessionManagerConfigBuilder {
        SessionManagerConfigBuilder::default()
    }
}
```

### P2.2: Add Builder for WorkflowConfig
**Impact**: Medium
**Effort**: Medium
**Files**: `llmspell-workflows/src/config.rs`

### P2.3: Add Builder for AgentConfig
**Impact**: High - Complex configuration
**Effort**: Medium
**Files**: `llmspell-agents/src/config.rs`

### P2.4: Add Builder for Complex Tool Configs
**Impact**: Low
**Effort**: Low per tool
**Files**: Various tool implementations

---

## P3: Low Priority - Documentation

### P3.1: Complete Rustdoc Coverage
**Impact**: High for new users
**Effort**: High - Time consuming
**Target**: 100% public API coverage

**Priority Order**:
1. Core traits and types
2. Manager/Factory types
3. Tool implementations
4. Utility functions

### P3.2: Add Examples to All Major APIs
**Impact**: High for adoption
**Effort**: Medium
**Focus Areas**:
- Session management
- Agent creation
- Workflow building
- Tool usage

### P3.3: Create Migration Guide
**Impact**: Critical for P0 changes
**Effort**: Low
**Content**:
- Breaking change list
- Migration examples
- Deprecation timeline

---

## Implementation Schedule

### Week 1 (Days 1-2)
- [ ] P1.1: Standardize getter methods (2 hours)
- [ ] P1.2: Constructor consistency (2 hours)
- [ ] P2.1: SessionManagerConfig builder (3 hours)
- [ ] P2.2: WorkflowConfig builder (3 hours)
- [ ] P2.3: AgentConfig builder (3 hours)

### Week 1 (Day 3)
- [ ] P0.1: Remove Service suffix with deprecation (4 hours)
- [ ] P3.3: Create migration guide (2 hours)

### Week 1 (Days 4-5)
- [ ] P3.1: Core rustdoc coverage (8 hours)
- [ ] P3.2: Add examples to core APIs (6 hours)

### Week 2 (Days 6-7)
- [ ] P3.1: Complete rustdoc coverage (8 hours)
- [ ] P2.4: Tool config builders (4 hours)
- [ ] Final testing and validation (4 hours)

---

## Validation Checklist

Before implementing each change:
- [ ] Review against style guide
- [ ] Check for breaking changes
- [ ] Plan deprecation strategy
- [ ] Write migration examples
- [ ] Update tests

After implementing:
- [ ] All tests pass
- [ ] Documentation updated
- [ ] Examples compile
- [ ] Deprecation warnings clear
- [ ] No clippy warnings

---

## Risk Matrix

| Change | Risk Level | Mitigation |
|--------|------------|------------|
| Service → Manager | High | Type aliases, clear migration guide |
| retrieve_ → get_ | Medium | Deprecated wrappers |
| Missing builders | Low | Additive change only |
| Documentation | Low | No code changes |

---

## Success Metrics

1. **Zero breaking changes without migration path**
2. **All deprecated APIs have replacements**
3. **100% of complex configs have builders**
4. **100% public API documentation**
5. **All examples compile and run**

---

## Notes

- Prioritize user-facing APIs over internal ones
- Keep deprecated APIs for at least one minor version
- Document all decisions in commit messages
- Create before/after examples for major changes