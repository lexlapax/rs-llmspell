# API Migration Strategy - Clean Break Approach

**Date**: August 1, 2025
**Version**: 1.0
**Target Release**: 0.6.0

## Overview

Since rs-llmspell is pre-1.0 and has **no backward compatibility requirements**, we will implement a clean break approach. This allows us to make all necessary API improvements without the complexity of deprecation cycles.

## Strategy: Direct Replacement

### Advantages of Clean Break
1. **Simpler codebase** - No deprecated wrapper functions
2. **Cleaner API** - No legacy naming patterns
3. **Faster implementation** - Direct refactoring
4. **Better performance** - No indirection layers
5. **Clear documentation** - Single way to do things

### Implementation Approach

#### 1. Direct Renaming
```rust
// Simply rename without deprecation
// Before
pub struct HookExecutorService { }

// After (direct replacement)
pub struct HookExecutor { }
```

#### 2. Update All References
- Use IDE refactoring tools for safety
- Update all imports and usage sites
- No need for compatibility aliases

#### 3. Method Updates
```rust
// Direct replacement
// Before
pub async fn retrieve_session(&self, id: &SessionId) -> Result<Session>

// After
pub async fn get_session(&self, id: &SessionId) -> Result<Session>
```

---

## Migration Tasks

### Phase 1: Bulk Renaming (Day 1)
1. **Service → Manager/Core Type**
   - `HookExecutorService` → `HookExecutor`
   - `EventBusService` → `EventBus`
   - Update all imports across codebase

2. **Method Standardization**
   - All `retrieve_*` → `get_*`
   - Ensure consistent naming patterns

3. **Constructor Updates**
   - Standardize on `new()`, `with_*()`, `from_*()` patterns
   - Remove non-standard constructor names

### Phase 2: API Enhancements (Day 2)
1. **Add Builder Patterns**
   - Direct implementation, no compatibility concerns
   - Start fresh with best practices

2. **Restructure Complex APIs**
   - Can redesign APIs for better ergonomics
   - No need to maintain old signatures

### Phase 3: Documentation (Day 3)
1. **Update All Documentation**
   - Remove references to old APIs
   - Update all examples
   - Ensure consistency

2. **Create "What's New" Guide**
   - List all API changes
   - Show before/after examples
   - Highlight improvements

---

## Implementation Checklist

### For Each API Change:
- [ ] Rename in source file
- [ ] Update all imports
- [ ] Update all call sites
- [ ] Update tests
- [ ] Update documentation
- [ ] Update examples

### Global Tasks:
- [ ] Run full test suite after each major change
- [ ] Update README with new patterns
- [ ] Update all tutorial code
- [ ] Verify all examples compile

---

## Testing Strategy

### 1. Continuous Testing
```bash
# After each refactoring step
cargo test --workspace
cargo clippy -- -D warnings
cargo doc --no-deps
```

### 2. Example Validation
```bash
# Ensure all examples still work
for example in examples/**/*.rs; do
    cargo run --example $(basename $example .rs)
done
```

### 3. Documentation Build
```bash
# Verify documentation builds
cargo doc --workspace --no-deps
```

---

## Communication Plan

### 1. Changelog Entry
```markdown
## [0.6.0] - 2025-08-XX

### Breaking Changes
- Renamed `HookExecutorService` to `HookExecutor`
- Renamed `EventBusService` to `EventBus`
- Standardized all `retrieve_*` methods to `get_*`
- Added builder patterns for complex configurations

### Improvements
- Consistent API naming throughout
- Better ergonomics with builder patterns
- Cleaner, more intuitive API surface
```

### 2. Announcement Template
```markdown
# rs-llmspell 0.6.0: Cleaner, More Consistent APIs

We've standardized all APIs for consistency and better ergonomics:

**Key Changes:**
- Removed unnecessary `Service` suffixes
- Standardized getter methods to `get_*`
- Added builder patterns for complex types
- Improved overall API consistency

Since we're pre-1.0, we took this opportunity to clean up the APIs
without maintaining backward compatibility. The result is a much
cleaner, more intuitive API surface.
```

---

## Benefits of This Approach

1. **Clean Codebase**
   - No technical debt from compatibility layers
   - Single, clear way to use each API
   - Easier to maintain

2. **Better Developer Experience**
   - Consistent patterns everywhere
   - No confusion from multiple ways to do same thing
   - Clear, modern API design

3. **Faster Development**
   - No time spent on deprecation
   - Direct refactoring is faster
   - Can make optimal design choices

4. **Documentation Clarity**
   - Only document current APIs
   - No legacy examples
   - Clear best practices

---

## Risk Mitigation

### 1. Thorough Testing
- Run full test suite after each change
- Test all examples
- Verify documentation builds

### 2. Clear Communication
- Prominent changelog
- Updated README
- Clear upgrade instructions

### 3. Atomic Changes
- Each refactoring in separate commit
- Easy to track changes
- Can revert if needed

---

## Timeline

### Day 1: Core Refactoring
- Morning: Service → Core type renaming
- Afternoon: Method standardization

### Day 2: Enhancements
- Morning: Add builder patterns
- Afternoon: API polish and consistency

### Day 3: Documentation
- Morning: Update all docs
- Afternoon: Update examples and guides

---

## Conclusion

By embracing a clean break approach, we can deliver a polished, consistent API surface without the complexity of maintaining backward compatibility. This is the perfect time (pre-1.0) to make these improvements, resulting in a better foundation for future growth.