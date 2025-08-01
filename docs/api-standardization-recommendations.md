# API Standardization Recommendations

**Date**: August 1, 2025
**Author**: API Review Team
**Status**: Final Recommendations

## Executive Summary

After comprehensive analysis of the rs-llmspell codebase and research into Rust API design standards, we recommend a series of targeted improvements that will enhance API consistency while maintaining backward compatibility. The codebase already demonstrates excellent design patterns; these recommendations focus on minor refinements for a polished public release.

## Key Findings

### Strengths ✅
1. **Excellent trait hierarchy**: Clean inheritance from `BaseAgent`
2. **Consistent type suffixes**: `*Tool`, `*Manager`, `*Config`
3. **Strong async patterns**: Proper `Send + Sync` bounds
4. **Good error handling**: Centralized error types with context

### Areas for Improvement ⚠️
1. **Service vs Manager naming**: 2 inconsistencies found
2. **Getter method naming**: Mix of `retrieve_*` and `get_*`
3. **Missing builders**: Complex configs lack builder patterns
4. **Documentation gaps**: Some public APIs lack examples

## Prioritized Recommendations

### 🔴 Priority 1: Breaking Changes (Handle with Care)

#### 1.1 Remove Service Suffix
**Current**: `HookExecutorService`, `EventBusService`
**Recommended**: `HookExecutor`, `EventBus`
**Rationale**: Aligns with Rust ecosystem patterns (tokio, actix)
**Migration Strategy**: 
```rust
// Deprecation approach
#[deprecated(since = "0.6.0", note = "Use `HookExecutor` instead")]
pub type HookExecutorService = HookExecutor;
```

#### 1.2 Standardize Getter Methods
**Current**: `retrieve_session()`, `get_session()` coexist
**Recommended**: Use `get_*` consistently for lookups
**Rationale**: Follows Rust API guidelines (C-GETTER)
**Note**: Keep `get_` prefix as these are lookup operations, not simple field access

### 🟡 Priority 2: API Enhancements (Non-Breaking)

#### 2.1 Add Builder Patterns
Implement builders for complex configurations:

```rust
// Priority targets (3+ fields)
SessionManagerConfig::builder()
    .max_sessions(100)
    .retention_policy(RetentionPolicy::Days(30))
    .storage_backend(backend)
    .build()

WorkflowConfig::builder()
    .timeout(Duration::from_secs(300))
    .retry_policy(RetryPolicy::Exponential)
    .build()

AgentConfig::builder()
    .name("assistant")
    .provider(Provider::OpenAI)
    .capabilities(Capabilities::all())
    .build()
```

#### 2.2 Constructor Consistency
Ensure all types follow patterns:
- `new()` - Simple/default construction
- `with_*()` - Parameterized construction
- `from_*()` - Type conversions
- `builder()` - Complex construction

### 🟢 Priority 3: Documentation & Polish

#### 3.1 Rustdoc Standards
Every public API should have:
```rust
/// Brief one-line summary.
///
/// More detailed explanation of purpose and behavior.
/// 
/// # Arguments
/// 
/// * `param` - Description with constraints
///
/// # Returns
/// 
/// Description of return value
///
/// # Errors
///
/// When this function returns errors
///
/// # Examples
///
/// ```rust
/// # use llmspell_core::*;
/// let result = function(param)?;
/// assert_eq!(result, expected);
/// ```
```

#### 3.2 API Stability Markers
```rust
// For experimental features
#[cfg(feature = "experimental")]
pub struct ExperimentalFeature;

// For enums that may grow
#[non_exhaustive]
pub enum HookPoint { ... }
```

## Implementation Plan

### Phase 1: Non-Breaking Improvements (Days 1-2)
1. Add builder patterns to complex configs
2. Add comprehensive rustdoc to all public APIs
3. Ensure consistent constructor patterns

### Phase 2: Deprecations (Day 3)
1. Deprecate `HookExecutorService` → `HookExecutor`
2. Deprecate `EventBusService` → `EventBus`  
3. Deprecate `retrieve_*` methods → `get_*` methods
4. Maintain type aliases for compatibility

### Phase 3: Documentation (Days 4-5)
1. Complete rustdoc coverage
2. Add examples to all major APIs
3. Create migration guide

### Phase 4: Validation (Days 6-7)
1. Run all tests with deprecations
2. Update all examples
3. Verify backward compatibility

## Style Guide Summary

### Naming Conventions
```rust
// Types
struct SessionManager     // UpperCamelCase
trait StorageBackend     // UpperCamelCase
enum SessionStatus       // UpperCamelCase

// Functions/Methods
fn get_session()         // snake_case
fn create_artifact()     // snake_case

// Constants
const MAX_SESSIONS: u32  // SCREAMING_SNAKE_CASE
```

### Method Patterns
```rust
// Getters (no prefix for field access)
impl Session {
    pub fn id(&self) -> &SessionId { &self.id }
    pub fn name(&self) -> &str { &self.name }
}

// Getters (with prefix for lookups)
impl SessionManager {
    pub async fn get_session(&self, id: &SessionId) -> Result<Session>
    pub async fn get_artifact(&self, id: &ArtifactId) -> Result<Artifact>
}

// Setters
impl Session {
    pub fn set_name(&mut self, name: String)
    pub fn set_status(&mut self, status: SessionStatus)
}

// Lifecycle
impl Service {
    pub async fn start(&mut self) -> Result<()>
    pub async fn stop(&mut self) -> Result<()>
}
```

### Builder Pattern Template
```rust
pub struct ConfigBuilder {
    field1: Option<Type1>,
    field2: Option<Type2>,
}

impl ConfigBuilder {
    pub fn new() -> Self {
        Self {
            field1: None,
            field2: None,
        }
    }
    
    pub fn field1(mut self, value: Type1) -> Self {
        self.field1 = Some(value);
        self
    }
    
    pub fn build(self) -> Result<Config> {
        Ok(Config {
            field1: self.field1.unwrap_or_default(),
            field2: self.field2.ok_or(Error::MissingField)?,
        })
    }
}
```

## Success Metrics

1. **API Consistency**: 100% consistent naming patterns
2. **Documentation**: 100% rustdoc coverage
3. **Examples**: All major APIs have working examples
4. **Compatibility**: Zero breaking changes for users
5. **Builder Coverage**: All 3+ field configs have builders

## Risks and Mitigations

### Risk 1: Breaking Changes Impact
**Mitigation**: Use deprecation warnings for full version cycle

### Risk 2: Documentation Drift  
**Mitigation**: Add doc tests that compile with examples

### Risk 3: Naming Conflicts
**Mitigation**: Careful review of each rename

## Conclusion

The rs-llmspell codebase demonstrates strong API design. These recommendations represent refinements that will polish the APIs for public release. By following established Rust patterns and maintaining backward compatibility through deprecation, we can deliver a professional, consistent API surface that developers will find intuitive and pleasant to use.

## Appendix: Quick Reference

### Do's ✅
- Use `get_*` for lookup operations
- Use builders for 3+ parameters
- Document all public APIs
- Maintain backward compatibility
- Follow Rust naming conventions

### Don'ts ❌
- Don't use `get_` prefix for simple field access
- Don't break existing APIs without deprecation
- Don't use `Service` suffix (use `Manager` or nothing)
- Don't skip examples in documentation
- Don't forget error context

### Decision Record
- **Why remove Service suffix?** Aligns with tokio, actix patterns
- **Why standardize on get_?** Rust API guidelines for lookup operations  
- **Why builders?** Improves ergonomics for complex configurations
- **Why now?** Pre-1.0 is the right time for API polish