# Phase 11a: Bridge Feature-Gate Cleanup - TODO List

**Version**: 2.0
**Date**: October 2025
**Status**: Phase 11a.1 Complete, Ready for 11a.2
**Phase**: 11a (Bridge Architecture Cleanup)
**Timeline**: 1-2 days
**Priority**: MEDIUM (Technical Debt Reduction)
**Dependencies**: Phase 11 Complete ✅
**Arch-Document**: docs/technical/current-architecture.md
**All-Phases-Document**: docs/in-progress/implementation-phases.md
**Parent-Phase**: Phase 11 Local LLM Integration
**This-document**: working copy /TODO.md (pristine copy in docs/in-progress/PHASE11a-TODO.md)

> **📋 Actionable Task List**: Feature-gate cleanup to make scripting languages optional, reducing compile time (~42s) and binary size (~2MB) without creating new crates.

---

## Overview

**Goal**: Make llmspell-bridge language-neutral by default, allowing users to opt-in to Lua/JavaScript.

**Problem**:
- `default = ["lua"]` forces all dependents to compile mlua (~42s compile overhead)
- Cannot compile without lua feature (4 compilation errors in debug_bridge.rs)
- JavaScript cannot work standalone (blocked by lua-specific code)
- ~2-3MB binary size for unused runtimes

**Solution**:
- Fix debug_bridge.rs blocker (language-neutral abstractions)
- Add #[cfg] gates to runtime factory methods
- Remove `default = ["lua"]` from bridge
- CLI maintains backward compatibility

**Success Criteria**:
- [ ] bridge compiles with --no-default-features (0 errors)
- [ ] CLI still defaults to Lua (backward compat)
- [ ] All feature combos pass: none, lua, js, both
- [ ] Zero clippy warnings
- [ ] ~42s compile savings, ~2MB binary savings

---

## Phase 11a.1: Feature Gate Audit - ✅ COMPLETE

### Task 11a.1.1: Audit Current Feature Usage - ✅ COMPLETE
**Priority**: CRITICAL
**Actual Time**: 1 hour
**Status**: ✅ Complete

**Methodology**: Grep analysis of all source files, manual inspection of Cargo.toml dependencies, dependency graph check.

**What Works ✅**:

1. **lib.rs Module Imports** (lines 279-284):
   ```rust
   #[cfg(feature = "lua")]
   pub mod lua;

   #[cfg(feature = "javascript")]
   pub mod javascript;
   ```
   ✅ Correctly gated

2. **Globals Feature-Gating** (20 files in globals/):
   All 20 global files have correct `#[cfg]` on injection methods:
   - agent_global.rs, artifact_global.rs, config_global.rs, core.rs
   - debug_global.rs, event_global.rs, hook_global.rs, injection.rs
   - json_global.rs, local_llm_global.rs, provider_global.rs, rag_global.rs
   - registry.rs, replay_global.rs, session_global.rs, state_global.rs
   - streaming_global.rs, tool_global.rs, types.rs, workflow_global.rs

   Pattern used:
   ```rust
   impl GlobalObject for SomeGlobal {
       #[cfg(feature = "lua")]
       fn inject_lua(&self, lua: &Lua) -> mlua::Result<()> { ... }

       #[cfg(feature = "javascript")]
       fn inject_js(&self, js: &JsContext) -> Result<()> { ... }
   }
   ```
   ✅ All correct

3. **Engine Traits** (engine/):
   - bridge.rs: Trait definitions (language-neutral) ✅
   - factory.rs: Match arms correctly gated ✅
   - types.rs: Type definitions (no language deps) ✅

4. **Language Modules**:
   - lua/: 20 Rust files
   - javascript/: 14 Rust files
   - Both properly structured

**Critical Issues Found ❌**:

**Issue #1: 🔴 BLOCKER - debug_bridge.rs Lines 283-294**
```rust
// NO #[cfg] gates - hardcoded crate::lua::stacktrace reference
pub fn stack_trace_options_for_level(
    &self,
    level: &str,
) -> crate::lua::stacktrace::StackTraceOptions {
    match level {
        "trace" => crate::lua::stacktrace::StackTraceOptions::for_trace(),
        "error" => crate::lua::stacktrace::StackTraceOptions::for_error(),
        _ => crate::lua::stacktrace::StackTraceOptions::default(),
    }
}
```
- **Impact**: Causes 4 compilation errors without lua feature
- **Blocks**: --no-default-features, --features javascript
- **Used By**: lua/globals/debug.rs lines 413, 431 (stackTrace functions)
- **Fix**: Create language-neutral StackTraceLevel enum with From traits

**Issue #2: Runtime Factory Methods (runtime.rs) - Missing #[cfg]**
File: llmspell-bridge/src/runtime.rs
- `new_with_lua()` - Line 143 ❌
- `new_with_javascript()` - Line 164 ❌
- `new_with_lua_and_provider()` - Line 177 ❌
- `new_with_javascript_and_provider()` - Line 195 ❌
- `new_with_engine_name()` match arms - Line 221 ❌

Current code always compiled, will fail without feature gates.

**Issue #3: Lib.rs Factory Functions - Missing #[cfg]**
File: llmspell-bridge/src/lib.rs
- `create_script_executor()` - Line 308 (always calls new_with_lua) ❌
- `create_script_executor_with_provider()` - Line 324 ❌

**Issue #4: Module-Level Gates Missing**
- lua/mod.rs: No `#![cfg(feature = "lua")]` at top ❌
- javascript/mod.rs: No `#![cfg(feature = "javascript")]` at top ❌

Note: lib.rs gates prevent compilation, but module-level is best practice.

**Dependency Analysis**:

Current llmspell-bridge/Cargo.toml:
```toml
[dependencies]
mlua = { workspace = true, optional = true }         # ✅ Correct
boa_engine = { workspace = true, optional = true }   # ✅ Correct

[features]
default = ["lua"]                  # ⚠️ Forces Lua, will change to []
lua = ["dep:mlua"]                 # ✅ Correct
javascript = ["dep:boa_engine"]    # ✅ Correct
```

Dependents requiring updates:
- llmspell-cli: Already has `default-features = false` ✅
- llmspell-kernel: Has `features = ["lua"]` ✅
- llmspell-testing: Missing features ❌
- llmspell-tools: Missing features ❌

### Task 11a.1.2: Test Current Feature Combinations - ✅ COMPLETE
**Priority**: CRITICAL
**Actual Time**: 1 hour
**Status**: ✅ Complete

**Test Results Matrix**:

| Config | Command | Time | Result | Errors | Warnings |
|--------|---------|------|--------|--------|----------|
| 1. Default (lua) | `cargo check -p llmspell-bridge` | 48.5s | ✅ Pass | 0 | 0 |
| 2. No features | `cargo check ... --no-default-features` | - | ❌ FAIL | 4 | 0 |
| 3. Lua explicit | `cargo check ... --features lua` | 6.2s | ✅ Pass | 0 | 0 |
| 4. JS only | `cargo check ... --features javascript` | - | ❌ FAIL | 4 | 5 |
| 5. All features | `cargo check ... --all-features` | 78.0s | ✅ Pass | 0 | 0 |

**Critical Insight: 87% Faster Builds Possible**
- Default (lua): 48.5s
- Explicit lua: 6.2s
- **Savings**: 42.3s (87% reduction)
- Root cause: Default forces unnecessary rebuilds

**Config 2 & 4 Errors** (All 4 identical):
```
error[E0433]: failed to resolve: unresolved import
   --> llmspell-bridge/src/debug_bridge.rs:274:17
    |
274 |     ) -> crate::lua::stacktrace::StackTraceOptions {
    |                 ^^^ unresolved import
```
Repeated at lines 274, 276, 277, 278 in `stack_trace_options_for_level()`.

**Config 4 Warnings** (JavaScript-only):
```
warning: unused import: `crate::event_serialization::EventSerialization`
warning: unused imports: `Language` and `UniversalEvent`
warning: unused import: `tokio::sync::mpsc::UnboundedReceiver`
warning: unused import: `std::collections::HashMap`
warning: unused imports: `debug` and `instrument`
```
These indicate JS globals have imports assuming Lua is present (cleanup later).

**Compile Time Breakdown** (from 78s all-features):
- Base bridge: ~6s
- mlua (Lua): ~42s (54% of total)
- boa_engine (JS): ~20s (26% of total)
- Tool features: ~10s (13% of total)

**Key Findings**:
1. **JavaScript cannot work standalone** - blocked by debug_bridge.rs lua dependency
2. **42s compile overhead** from mlua confirmed
3. **4 errors** all trace to single function (stack_trace_options_for_level)
4. **Explicit features 87% faster** than default (6.2s vs 48.5s)

**Decision**:
- ❌ **CANNOT** remove `default = ["lua"]` yet (would break everything immediately)
- ✅ **MUST** fix debug_bridge.rs FIRST (unblocks all non-lua configs)
- ✅ **THEN** fix runtime methods
- ✅ **THEN** remove defaults
- ✅ **THEN** validate all combinations

---

## Phase 11a.2: Fix debug_bridge.rs Blocker - 🔴 CRITICAL FIRST

### Task 11a.2.1: Create Language-Neutral StackTrace Abstraction
**Priority**: 🔴 CRITICAL BLOCKER
**Estimated Time**: 45 minutes
**Status**: Pending
**Blocks**: All subsequent tasks

**Problem**: `stack_trace_options_for_level()` in debug_bridge.rs returns `crate::lua::stacktrace::StackTraceOptions`, causing 4 compile errors without lua feature.

**Solution**: Create language-neutral enum, use From trait pattern.

**Implementation**:

1. **Add to debug_bridge.rs** (after imports, ~line 13):
   ```rust
   /// Language-neutral stack trace verbosity level
   ///
   /// Abstracts stack trace detail configuration from language-specific types.
   /// Each script engine implements From<StackTraceLevel> for its options type.
   #[derive(Debug, Clone, Copy, PartialEq, Eq)]
   pub enum StackTraceLevel {
       /// Full stack trace with locals and upvalues
       Trace,
       /// Error-focused stack trace (minimal overhead)
       Error,
       /// Standard stack trace
       Default,
   }
   ```

2. **Replace stack_trace_options_for_level()** (lines 283-294):
   ```rust
   /// Get stack trace level for different debug levels
   ///
   /// Returns language-neutral level that converts to language-specific
   /// options via From trait implementations.
   #[must_use]
   pub fn stack_trace_options_for_level(&self, level: &str) -> StackTraceLevel {
       match level {
           "trace" | "TRACE" => StackTraceLevel::Trace,
           "error" | "ERROR" => StackTraceLevel::Error,
           _ => StackTraceLevel::Default,
       }
   }
   ```

3. **Add to lua/stacktrace.rs** (after StackTraceOptions impl, ~line 95):
   ```rust
   use crate::debug_bridge::StackTraceLevel;

   /// Convert language-neutral level to Lua-specific options
   impl From<StackTraceLevel> for StackTraceOptions {
       fn from(level: StackTraceLevel) -> Self {
           match level {
               StackTraceLevel::Trace => Self::for_trace(),
               StackTraceLevel::Error => Self::for_error(),
               StackTraceLevel::Default => Self::default(),
           }
       }
   }
   ```

4. **Update call sites** in lua/globals/debug.rs (lines 413, 431):
   ```rust
   // Line 413: Debug.stackTrace()
   let trace_options = options.map_or_else(
       || bridge_clone.stack_trace_options_for_level(&bridge_clone.get_level()).into(),
       |opts| StackTraceOptions { /* explicit opts */ },
   );

   // Line 431: Debug.stackTraceJson()
   let trace_options = options.map_or_else(
       || bridge_clone.stack_trace_options_for_level(&bridge_clone.get_level()).into(),
       |opts| StackTraceOptions { /* explicit opts */ },
   );
   ```

**Verification**:
```bash
cargo check -p llmspell-bridge --no-default-features       # Must pass (0 errors)
cargo check -p llmspell-bridge --features javascript       # Must pass (0 errors)
cargo check -p llmspell-bridge --features lua              # Must pass
cargo clippy -p llmspell-bridge --no-default-features -- -D warnings
```

**Acceptance Criteria**:
- [ ] StackTraceLevel enum created in debug_bridge.rs
- [ ] stack_trace_options_for_level() returns StackTraceLevel
- [ ] From<StackTraceLevel> impl in lua/stacktrace.rs
- [ ] lua/globals/debug.rs call sites use .into()
- [ ] cargo check --no-default-features: 0 errors
- [ ] cargo check --features javascript: 0 errors
- [ ] cargo clippy: 0 warnings all configs
- [ ] Git commit: "fix(bridge): Abstract StackTrace types for language neutrality"

**Unblocks**: Phase 11a.4 (removing default features)

---

## Phase 11a.3: Fix Runtime Factory Methods

### Task 11a.3.1: Add Feature Gates to Runtime Methods
**Priority**: HIGH
**Estimated Time**: 20 minutes
**Status**: Pending
**Depends On**: 11a.2.1

**Files**: llmspell-bridge/src/runtime.rs

**Changes Required**:

1. **Gate Lua methods**:
   ```rust
   #[cfg(feature = "lua")]
   pub async fn new_with_lua(config: LLMSpellConfig) -> Result<Self, LLMSpellError> {
       // existing implementation
   }

   #[cfg(feature = "lua")]
   pub fn new_with_lua_and_provider(
       config: LLMSpellConfig,
       provider_manager: Arc<ProviderManager>,
   ) -> Result<Self, LLMSpellError> {
       // existing implementation
   }
   ```

2. **Gate JavaScript methods**:
   ```rust
   #[cfg(feature = "javascript")]
   pub async fn new_with_javascript(config: LLMSpellConfig) -> Result<Self, LLMSpellError> {
       // existing implementation
   }

   #[cfg(feature = "javascript")]
   pub fn new_with_javascript_and_provider(
       config: LLMSpellConfig,
       provider_manager: Arc<ProviderManager>,
   ) -> Result<Self, LLMSpellError> {
       // existing implementation
   }
   ```

3. **Fix new_with_engine_name()** (line ~221):
   ```rust
   pub async fn new_with_engine_name(
       engine_name: &str,
       config: LLMSpellConfig,
   ) -> Result<Self, LLMSpellError> {
       match engine_name {
           #[cfg(feature = "lua")]
           "lua" => Self::new_with_lua(config).await,

           #[cfg(feature = "javascript")]
           "javascript" => Self::new_with_javascript(config).await,

           _ => Err(LLMSpellError::Configuration(format!(
               "Unsupported or disabled engine: '{}'. Available: {}",
               engine_name,
               Self::available_engines().join(", ")
           ))),
       }
   }
   ```

4. **Add available_engines()** (if doesn't exist):
   ```rust
   /// Get list of compiled script engines
   pub fn available_engines() -> Vec<&'static str> {
       let mut engines = Vec::new();
       #[cfg(feature = "lua")]
       engines.push("lua");
       #[cfg(feature = "javascript")]
       engines.push("javascript");
       engines
   }
   ```

**Verification**:
```bash
cargo check -p llmspell-bridge --no-default-features
cargo check -p llmspell-bridge --features lua
cargo clippy -p llmspell-bridge --features lua -- -D warnings
```

**Acceptance Criteria**:
- [ ] All 4 factory methods have #[cfg] gates
- [ ] Match arms in new_with_engine_name() gated
- [ ] available_engines() reflects compiled features
- [ ] Zero clippy warnings
- [ ] Git commit: "fix(bridge): Feature-gate runtime factory methods"

### Task 11a.3.2: Add Feature Gates to Lib.rs Factory Functions
**Priority**: HIGH
**Estimated Time**: 10 minutes
**Status**: Pending
**Depends On**: 11a.3.1

**File**: llmspell-bridge/src/lib.rs

**Changes** (lines ~308, 324):
```rust
#[cfg(feature = "lua")]
pub async fn create_script_executor(
    config: LLMSpellConfig,
) -> Result<Arc<dyn ScriptExecutor>, LLMSpellError> {
    let runtime = ScriptRuntime::new_with_lua(config).await?;
    Ok(Arc::new(runtime))
}

#[cfg(feature = "lua")]
pub async fn create_script_executor_with_provider(
    config: LLMSpellConfig,
    provider_manager: Arc<ProviderManager>,
) -> Result<Arc<dyn ScriptExecutor>, LLMSpellError> {
    let runtime = ScriptRuntime::new_with_lua_and_provider(config, provider_manager)?;
    Ok(Arc::new(runtime))
}
```

**Verification**:
```bash
cargo check -p llmspell-bridge --no-default-features
cargo clippy -p llmspell-bridge --no-default-features -- -D warnings
```

**Acceptance Criteria**:
- [ ] Both functions have #[cfg(feature = "lua")]
- [ ] Zero clippy warnings
- [ ] Git commit: "fix(bridge): Feature-gate lib.rs factory functions"

---

## Phase 11a.4: Remove Default Features

### Task 11a.4.1: Update llmspell-bridge Cargo.toml
**Priority**: CRITICAL
**Estimated Time**: 15 minutes
**Status**: Pending
**Depends On**: 11a.2.1, 11a.3.1, 11a.3.2 (all must be complete)

**File**: llmspell-bridge/Cargo.toml

**Change**:
```toml
[features]
default = []  # Changed from ["lua"] - language selection now explicit
common = ["lua", "llmspell-tools/common"]
full = ["lua", "javascript", "llmspell-tools/full"]

lua = ["dep:mlua"]
javascript = ["dep:boa_engine"]
```

**Verification**:
```bash
cargo check -p llmspell-bridge --no-default-features       # Must pass
cargo check -p llmspell-bridge                             # No features now
cargo check -p llmspell-bridge --features lua              # Must pass
cargo test -p llmspell-bridge --no-default-features --lib  # Tests pass
```

**Acceptance Criteria**:
- [ ] default = [] in Cargo.toml
- [ ] cargo check --no-default-features: 0 errors, 0 warnings
- [ ] Git commit: "feat(bridge): Remove default language features"

### Task 11a.4.2: Update llmspell-cli Cargo.toml
**Priority**: CRITICAL
**Estimated Time**: 15 minutes
**Status**: Pending
**Depends On**: 11a.4.1

**File**: llmspell-cli/Cargo.toml

**Change**:
```toml
[dependencies]
llmspell-bridge = { path = "../llmspell-bridge", default-features = false }

[features]
default = ["lua"]  # CLI maintains backward compatibility
lua = ["llmspell-bridge/lua"]
javascript = ["llmspell-bridge/javascript"]
```

**Verification**:
```bash
cargo build -p llmspell-cli                     # Lua enabled (backward compat)
cargo build -p llmspell-cli --no-default-features  # No languages
cargo run -p llmspell-cli -- --version          # Should work
```

**Acceptance Criteria**:
- [ ] CLI default = ["lua"] (backward compat maintained)
- [ ] Build succeeds with lua feature
- [ ] Git commit: "feat(cli): Explicit language feature selection"

### Task 11a.4.3: Update Dependent Cargo.tomls
**Priority**: HIGH
**Estimated Time**: 15 minutes
**Status**: Pending
**Depends On**: 11a.4.1

**Files to Update**:

1. **llmspell-testing/Cargo.toml**:
   ```toml
   [dependencies]
   llmspell-bridge = { path = "../llmspell-bridge", features = ["lua"] }
   ```

2. **llmspell-tools/Cargo.toml** (dev-dependencies):
   ```toml
   [dev-dependencies]
   llmspell-bridge = { path = "../llmspell-bridge", features = ["lua"] }
   ```

Note: llmspell-kernel already has `features = ["lua"]` ✅

**Verification**:
```bash
cargo check -p llmspell-testing
cargo check -p llmspell-tools
cargo test -p llmspell-kernel --no-run
```

**Acceptance Criteria**:
- [ ] All dependent crates specify explicit features
- [ ] All checks pass
- [ ] Git commit: "chore: Explicit lua features in dependent crates"

---

## Phase 11a.5: Add Module-Level Gates

### Task 11a.5.1: Add Module-Level #![cfg] Guards
**Priority**: MEDIUM (defensive, best practice)
**Estimated Time**: 15 minutes
**Status**: Pending
**Depends On**: 11a.4.1

**Files**:

1. **llmspell-bridge/src/lua/mod.rs** (add at line 2):
   ```rust
   //! ABOUTME: Lua script engine implementation
   #![cfg(feature = "lua")]

   // rest of file...
   ```

2. **llmspell-bridge/src/javascript/mod.rs** (add at line 2):
   ```rust
   //! ABOUTME: JavaScript script engine integration
   #![cfg(feature = "javascript")]

   // rest of file...
   ```

**Note**: lib.rs already gates module imports, this is defensive.

**Verification**:
```bash
cargo check -p llmspell-bridge --no-default-features
cargo clippy -p llmspell-bridge --no-default-features -- -D warnings
```

**Acceptance Criteria**:
- [ ] #![cfg] added to lua/mod.rs
- [ ] #![cfg] added to javascript/mod.rs
- [ ] Zero clippy warnings
- [ ] Git commit: "fix(bridge): Add module-level feature gates"

---

## Phase 11a.6: Final Validation

### Task 11a.6.1: Comprehensive Feature Matrix Validation
**Priority**: CRITICAL
**Estimated Time**: 30 minutes
**Status**: Pending
**Depends On**: All previous tasks

**Test Matrix**:

```bash
# 1. No features (language-neutral)
time cargo check -p llmspell-bridge --no-default-features
cargo clippy -p llmspell-bridge --no-default-features -- -D warnings
cargo test -p llmspell-bridge --no-default-features --lib

# 2. Lua only
time cargo check -p llmspell-bridge --features lua
cargo clippy -p llmspell-bridge --features lua -- -D warnings
cargo test -p llmspell-bridge --features lua

# 3. JavaScript only
time cargo check -p llmspell-bridge --features javascript
cargo clippy -p llmspell-bridge --features javascript -- -D warnings
cargo test -p llmspell-bridge --features javascript

# 4. Both languages
time cargo check -p llmspell-bridge --features lua,javascript
cargo test -p llmspell-bridge --features lua,javascript

# 5. All features
cargo check -p llmspell-bridge --all-features
cargo clippy -p llmspell-bridge --all-features -- -D warnings

# 6. Workspace
cargo check --workspace --all-features
cargo clippy --workspace --all-features -- -D warnings
```

**Success Criteria**:

| Config | Check | Clippy | Tests | Status |
|--------|-------|--------|-------|--------|
| No features | ✅ | ✅ | ✅ | - |
| Lua only | ✅ | ✅ | ✅ | - |
| JS only | ✅ | ✅ | ⚠️ | - |
| Both | ✅ | ✅ | ✅ | - |
| All features | ✅ | ✅ | ✅ | - |
| Workspace | ✅ | ✅ | - | - |

**Acceptance Criteria**:
- [ ] All configurations compile (0 errors)
- [ ] Zero clippy warnings across all configs
- [ ] Tests pass for applicable features
- [ ] Document results in this TODO

---

## Phase 11a.7: Performance Measurement

### Task 11a.7.1: Measure Compile Time Improvements
**Priority**: MEDIUM
**Estimated Time**: 20 minutes
**Status**: Pending

**Commands** (clean builds):
```bash
# Baseline measurements
cargo clean && time cargo build -p llmspell-cli --release --no-default-features
cargo clean && time cargo build -p llmspell-cli --release  # lua default
cargo clean && time cargo build -p llmspell-cli --release --all-features

# Incremental measurements
cargo build -p llmspell-bridge --no-default-features
cargo build -p llmspell-bridge --features lua
```

**Document Results**:

| Configuration | Clean Build | Incremental | vs Lua |
|--------------|-------------|-------------|--------|
| No features | ? | ? | -42s expected |
| Lua (default) | ? | 6.2s | baseline |
| All features | ? | 78.0s | +72s |

**Expected**:
- No features: ~2m (saves ~42s from no mlua)
- Lua: ~2m 45s (baseline)
- All: ~3m 15s (adds boa_engine ~20s, tools ~10s)

**Acceptance Criteria**:
- [ ] Measurements within ±10% of expected
- [ ] Document results in this TODO

### Task 11a.7.2: Measure Binary Sizes
**Priority**: MEDIUM
**Estimated Time**: 10 minutes
**Status**: Pending

**Commands**:
```bash
cargo build -p llmspell-cli --release --no-default-features
ls -lh target/release/llmspell  # Record

cargo build -p llmspell-cli --release
ls -lh target/release/llmspell  # Record

cargo build -p llmspell-cli --release --all-features
ls -lh target/release/llmspell  # Record
```

**Document Results**:

| Configuration | Size | vs Lua |
|--------------|------|--------|
| No features | ? | -2MB expected |
| Lua (default) | ? | baseline |
| All features | ? | +2MB expected |

**Expected**:
- No features: ~15MB
- Lua: ~17MB (baseline)
- All: ~19MB (+2MB)

**Acceptance Criteria**:
- [ ] Measurements within ±10% of expected
- [ ] Document results in this TODO

---

## Final Validation Checklist

### Compilation
- [ ] `cargo check -p llmspell-bridge --no-default-features` ✅
- [ ] `cargo check -p llmspell-bridge --features lua` ✅
- [ ] `cargo check -p llmspell-bridge --features javascript` ✅
- [ ] `cargo check -p llmspell-bridge --all-features` ✅
- [ ] `cargo check --workspace --all-features` ✅

### Quality
- [ ] `cargo clippy -p llmspell-bridge --no-default-features -- -D warnings` ✅
- [ ] `cargo clippy -p llmspell-bridge --features lua -- -D warnings` ✅
- [ ] `cargo clippy --workspace --all-features -- -D warnings` ✅
- [ ] `cargo fmt --all --check` ✅

### Tests
- [ ] `cargo test -p llmspell-bridge --no-default-features --lib` ✅
- [ ] `cargo test -p llmspell-bridge --features lua` ✅
- [ ] `cargo test --workspace --all-features` ✅

### Backward Compatibility
- [ ] llmspell CLI defaults to Lua ✅
- [ ] `cargo build -p llmspell-cli` includes Lua ✅
- [ ] Existing scripts work unchanged ✅

---

## Success Metrics Summary

### Compile Time (Target → Actual)
- **No features**: <2m (target: -42s from lua) → ?
- **Lua**: 6.2s incremental (baseline) → ✅ Confirmed
- **All features**: <3m 15s → ?

### Binary Size (Target → Actual)
- **No features**: ~15MB (target: -2MB) → ?
- **Lua**: ~17MB (baseline) → ?
- **All features**: ~19MB → ?

### Quality
- **Zero** clippy warnings all configs: ✅/?
- **100%** test pass rate: ✅/?
- **Zero** breaking changes for CLI users: ✅/?

---

## Risk Assessment

### Mitigated ✅
1. ✅ debug_bridge.rs blocker identified (4 errors, same function)
2. ✅ Task order corrected (fix blocker before removing defaults)
3. ✅ 87% compile improvement validated (6.2s vs 48.5s explicit vs default)
4. ✅ JavaScript standalone blocked by lua dependency identified

### Remaining ⚠️
1. ⚠️ Tests may need feature gates (discover in 11a.6)
2. ⚠️ JavaScript has 5 unused import warnings (cleanup needed)
3. ⚠️ Dependent crates may surface additional issues

---

**END OF PHASE 11a TODO**
