# Phase 11a: Bridge Feature-Gate Cleanup - TODO List

**Version**: 1.0
**Date**: October 2025
**Status**: Implementation Ready
**Phase**: 11a (Bridge Architecture Cleanup)
**Timeline**: 1-2 days (focused cleanup)
**Priority**: MEDIUM (Technical Debt Reduction)
**Dependencies**: Phase 11 Complete ✅
**Arch-Document**: docs/technical/current-architecture.md
**All-Phases-Document**: docs/in-progress/implementation-phases.md
**Parent-Phase**: Phase 11 Local LLM Integration
**This-document**: working copy /TODO.md (pristine/immutable copy in docs/in-progress/PHASE11a-TODO.md)

> **📋 Actionable Task List**: This document breaks down bridge feature-gate cleanup into specific, measurable tasks for improving compilation performance and binary size without creating new crates.

---

## Overview

**Goal**: Refactor llmspell-bridge feature gates to make scripting languages truly optional, reducing compile time and binary size for users who don't need all language runtimes.

**Problem Statement**:
- Currently `llmspell-bridge` has `default = ["lua"]`, forcing all dependents to compile mlua
- JavaScript stubs are 90% incomplete (~300 LOC) but always compiled when feature enabled
- Users building Python-only (future) or tool-only deployments pay unnecessary compile cost
- ~45s mlua compile time on cold builds
- ~2-3MB binary size for unused language runtimes

**Solution**:
- Remove default features from llmspell-bridge (make it language-neutral by default)
- Push language selection to final binary (llmspell-cli)
- Ensure all feature gates are correct and comprehensive
- Zero new crates, zero breaking API changes (internal only)

**Success Criteria Summary:**
- [ ] llmspell-bridge compiles with `--no-default-features` (no language deps)
- [ ] llmspell-cli maintains backward compatibility (still defaults to Lua)
- [ ] All tests pass with feature combinations: none, lua, javascript, lua+javascript
- [ ] Zero clippy warnings after each task
- [ ] Documentation updated for feature selection
- [ ] Binary size reduced by ~2MB for no-language builds
- [ ] Compile time baseline established for each configuration

---

## Phase 11a.1: Feature Gate Audit (2 hours)

### Task 11a.1.1: Audit Current Feature Usage
**Priority**: CRITICAL
**Estimated Time**: 1 hour
**Assignee**: Architecture Lead

**Description**: Document all current feature gates and identify missing or incorrect gates.

**Acceptance Criteria:**
- [ ] Complete inventory of `#[cfg(feature = "lua")]` usage
- [ ] Complete inventory of `#[cfg(feature = "javascript")]` usage
- [ ] Identify modules missing feature gates
- [ ] Identify incorrect or redundant gates
- [ ] Document current dependency tree

**Implementation Steps:**
1. Grep all `#[cfg(feature` in llmspell-bridge/src
   ```bash
   grep -r "#\[cfg(feature" llmspell-bridge/src/ > /tmp/feature-audit.txt
   ```
2. List all files in `src/lua/` and verify feature gates
3. List all files in `src/javascript/` and verify feature gates
4. Check `src/globals/` for language-specific code without gates
5. Verify `src/engine/` trait definitions are language-neutral
6. Document findings in `/tmp/bridge-feature-audit.md`

**Definition of Done:**
- [ ] Audit document created with findings
- [ ] All feature-gated modules identified
- [ ] Missing gates documented with remediation plan
- [ ] Baseline established for comparison

### Task 11a.1.2: Test Current Feature Combinations
**Priority**: CRITICAL
**Estimated Time**: 1 hour
**Assignee**: QA Lead

**Description**: Establish baseline test results for all feature combinations.

**Acceptance Criteria:**
- [ ] Tests pass with `default` features
- [ ] Tests pass with `--no-default-features --features lua`
- [ ] Tests pass with `--no-default-features --features javascript`
- [ ] Tests pass with `--all-features`
- [ ] Document which tests require which features

**Implementation Steps:**
1. Run baseline tests:
   ```bash
   cargo test -p llmspell-bridge  # default (lua)
   cargo test -p llmspell-bridge --no-default-features
   cargo test -p llmspell-bridge --no-default-features --features lua
   cargo test -p llmspell-bridge --no-default-features --features javascript
   cargo test -p llmspell-bridge --all-features
   ```
2. Document test failures for `--no-default-features`
3. Identify tests that require specific language features
4. Create test matrix in `/tmp/bridge-test-matrix.md`
5. Establish baseline compile times for each configuration

**Definition of Done:**
- [ ] Test matrix documented
- [ ] Baseline failures identified (expected for --no-default-features)
- [ ] Compile time baselines captured
- [ ] Binary size baselines captured

---

## Phase 11a.2: Remove Default Features (3 hours)

### Task 11a.2.1: Update llmspell-bridge Cargo.toml
**Priority**: CRITICAL
**Estimated Time**: 30 minutes
**Assignee**: Build Engineer

**Description**: Remove default language features from llmspell-bridge.

**Acceptance Criteria:**
- [ ] `default = []` in llmspell-bridge/Cargo.toml
- [ ] All language deps remain optional
- [ ] Feature forwarding preserved for tools
- [ ] Cargo.toml compiles without errors

**Implementation Steps:**
1. Edit `llmspell-bridge/Cargo.toml`:
   ```toml
   [features]
   default = []  # CHANGED: No default language, users choose at CLI level
   common = ["lua", "llmspell-tools/common"]  # Add lua explicitly
   full = ["lua", "javascript", "llmspell-tools/full"]  # Explicit all languages

   # Script engine features (unchanged)
   lua = ["dep:mlua"]
   javascript = ["dep:boa_engine"]
   ```
2. Verify optional deps are still marked `optional = true`:
   ```toml
   mlua = { workspace = true, optional = true }
   boa_engine = { workspace = true, optional = true }
   ```
3. Run `cargo check -p llmspell-bridge --no-default-features`
4. Run `cargo check -p llmspell-bridge --features lua`
5. Verify all downstream crates still compile

**Definition of Done:**
- [ ] Cargo.toml updated
- [ ] `cargo check` passes with `--no-default-features`
- [ ] `cargo check` passes with `--features lua`
- [ ] Zero clippy warnings
- [ ] Git commit: "feat(bridge): Remove default language features"

### Task 11a.2.2: Update llmspell-cli Cargo.toml
**Priority**: CRITICAL
**Estimated Time**: 30 minutes
**Assignee**: Build Engineer

**Description**: Make llmspell-cli specify lua as default (backward compatibility).

**Acceptance Criteria:**
- [ ] CLI maintains `default = ["lua"]` behavior
- [ ] Feature forwarding to bridge works
- [ ] Users can opt-out with `--no-default-features`
- [ ] Binary builds successfully

**Implementation Steps:**
1. Edit `llmspell-cli/Cargo.toml`:
   ```toml
   [features]
   default = ["lua"]  # KEEP: CLI maintains backward compatibility
   common = ["llmspell-bridge/common", "lua"]
   full = ["llmspell-bridge/full", "lua", "javascript"]

   # Language features (forward to bridge)
   lua = ["llmspell-bridge/lua"]
   javascript = ["llmspell-bridge/javascript"]
   python = []  # future
   ```
2. Verify dependency declaration:
   ```toml
   [dependencies]
   llmspell-bridge = { path = "../llmspell-bridge", default-features = false }
   ```
3. Build and test:
   ```bash
   cargo build -p llmspell-cli  # should enable lua
   cargo build -p llmspell-cli --no-default-features  # no languages
   cargo build -p llmspell-cli --features javascript --no-default-features
   ```
4. Verify binary size differences

**Definition of Done:**
- [ ] CLI Cargo.toml updated
- [ ] Default build includes lua (backward compat verified)
- [ ] No-default build excludes all languages
- [ ] Binary size measured for all configs
- [ ] Zero clippy warnings
- [ ] Git commit: "feat(cli): Explicit language feature forwarding"

### Task 11a.2.3: Update llmspell-kernel Dev Dependencies
**Priority**: HIGH
**Estimated Time**: 30 minutes
**Assignee**: Build Engineer

**Description**: Update kernel's dev-dependency on bridge to specify lua feature.

**Acceptance Criteria:**
- [ ] Kernel tests still pass
- [ ] Feature specification explicit
- [ ] No runtime dependency on bridge
- [ ] Dev builds faster with explicit features

**Implementation Steps:**
1. Edit `llmspell-kernel/Cargo.toml` dev-dependencies:
   ```toml
   [dev-dependencies]
   llmspell-bridge = { path = "../llmspell-bridge", features = ["lua"] }
   ```
2. Verify tests compile:
   ```bash
   cargo test -p llmspell-kernel --no-run
   ```
3. Run kernel test suite:
   ```bash
   cargo test -p llmspell-kernel
   ```
4. Check for feature-gated test issues

**Definition of Done:**
- [ ] Kernel Cargo.toml updated
- [ ] All kernel tests pass
- [ ] Zero clippy warnings
- [ ] Git commit: "chore(kernel): Explicit lua feature in dev-deps"

### Task 11a.2.4: Update llmspell-testing Dependencies
**Priority**: HIGH
**Estimated Time**: 30 minutes
**Assignee**: QA Engineer

**Description**: Update testing crate to specify lua feature explicitly.

**Acceptance Criteria:**
- [ ] Testing helpers compile
- [ ] Test utilities work with lua
- [ ] Feature specification explicit
- [ ] No unexpected test failures

**Implementation Steps:**
1. Edit `llmspell-testing/Cargo.toml`:
   ```toml
   [dependencies]
   llmspell-bridge = { path = "../llmspell-bridge", features = ["lua"] }
   ```
2. Verify testing crate compiles:
   ```bash
   cargo check -p llmspell-testing
   ```
3. Run any tests in testing crate itself:
   ```bash
   cargo test -p llmspell-testing
   ```
4. Spot-check one dependent test suite

**Definition of Done:**
- [ ] Testing Cargo.toml updated
- [ ] Testing crate compiles
- [ ] Helper functions work correctly
- [ ] Zero clippy warnings
- [ ] Git commit: "chore(testing): Explicit lua feature dependency"

### Task 11a.2.5: Update llmspell-tools Dev Dependencies
**Priority**: MEDIUM
**Estimated Time**: 30 minutes
**Assignee**: Build Engineer

**Description**: Update tools' dev-dependency on bridge to specify features.

**Acceptance Criteria:**
- [ ] Tools tests compile
- [ ] No circular dependency issues
- [ ] Feature specification correct
- [ ] Tests pass

**Implementation Steps:**
1. Edit `llmspell-tools/Cargo.toml` dev-dependencies:
   ```toml
   [dev-dependencies]
   llmspell-bridge = { path = "../llmspell-bridge", features = ["lua"] }
   ```
2. Verify tools compile:
   ```bash
   cargo check -p llmspell-tools
   ```
3. Run tools test suite:
   ```bash
   cargo test -p llmspell-tools
   ```
4. Check for bridge-related test issues

**Definition of Done:**
- [ ] Tools Cargo.toml updated
- [ ] All tools tests pass
- [ ] Zero clippy warnings
- [ ] Git commit: "chore(tools): Explicit lua feature in dev-deps"

---

## Phase 11a.3: Fix Feature Gates in Code (4 hours)

### Task 11a.3.1: Add Feature Gates to src/lua/ Module
**Priority**: CRITICAL
**Estimated Time**: 1 hour
**Assignee**: Core Developer

**Description**: Ensure all Lua-specific code is properly feature-gated.

**Acceptance Criteria:**
- [ ] `src/lua/mod.rs` has `#[cfg(feature = "lua")]`
- [ ] All `pub mod lua;` in lib.rs feature-gated
- [ ] No Lua code compiled without feature
- [ ] Compilation succeeds with `--no-default-features`

**Implementation Steps:**
1. Edit `llmspell-bridge/src/lib.rs`:
   ```rust
   // Language-specific implementations (feature-gated)
   #[cfg(feature = "lua")]
   pub mod lua;

   #[cfg(feature = "javascript")]
   pub mod javascript;
   ```
2. Verify `src/lua/mod.rs` has module-level gate:
   ```rust
   //! Lua script engine integration
   #![cfg(feature = "lua")]
   ```
3. Check all public re-exports from lua are gated
4. Test compilation:
   ```bash
   cargo check -p llmspell-bridge --no-default-features
   cargo check -p llmspell-bridge --features lua
   ```
5. Run clippy on both configs

**Definition of Done:**
- [ ] All lua/ code gated correctly
- [ ] Compiles without lua feature
- [ ] Compiles with lua feature
- [ ] Zero clippy warnings both ways
- [ ] Git commit: "fix(bridge): Add feature gates to lua module"

### Task 11a.3.2: Add Feature Gates to src/javascript/ Module
**Priority**: HIGH
**Estimated Time**: 30 minutes
**Assignee**: Core Developer

**Description**: Ensure all JavaScript-specific code is properly feature-gated.

**Acceptance Criteria:**
- [ ] `src/javascript/mod.rs` has `#[cfg(feature = "javascript")]`
- [ ] All `pub mod javascript;` in lib.rs feature-gated
- [ ] No JS code compiled without feature
- [ ] Compilation succeeds without javascript feature

**Implementation Steps:**
1. Verify `llmspell-bridge/src/lib.rs` (should be done in 11a.3.1):
   ```rust
   #[cfg(feature = "javascript")]
   pub mod javascript;
   ```
2. Verify `src/javascript/mod.rs` has module-level gate:
   ```rust
   //! JavaScript script engine integration
   #![cfg(feature = "javascript")]
   ```
3. Test compilation:
   ```bash
   cargo check -p llmspell-bridge --no-default-features
   cargo check -p llmspell-bridge --features javascript
   ```
4. Run clippy

**Definition of Done:**
- [ ] All javascript/ code gated correctly
- [ ] Compiles without javascript feature
- [ ] Compiles with javascript feature
- [ ] Zero clippy warnings
- [ ] Git commit: "fix(bridge): Add feature gates to javascript module"

### Task 11a.3.3: Fix Runtime Factory Methods
**Priority**: CRITICAL
**Estimated Time**: 1.5 hours
**Assignee**: Core Developer

**Description**: Add feature gates to language-specific factory methods in ScriptRuntime.

**Acceptance Criteria:**
- [ ] `new_with_lua()` gated on lua feature
- [ ] `new_with_lua_and_provider()` gated on lua feature
- [ ] `new_with_engine_name()` handles missing features gracefully
- [ ] Compilation errors clear when feature missing
- [ ] Backward compatibility maintained

**Implementation Steps:**
1. Edit `llmspell-bridge/src/runtime.rs`:
   ```rust
   impl ScriptRuntime {
       /// Create runtime with Lua engine (requires "lua" feature)
       #[cfg(feature = "lua")]
       pub async fn new_with_lua(config: LLMSpellConfig) -> Result<Self> {
           let engine = crate::lua::engine::LuaEngine::new(config.clone())?;
           // ... existing impl
       }

       /// Create runtime with Lua engine and existing provider (requires "lua" feature)
       #[cfg(feature = "lua")]
       pub fn new_with_lua_and_provider(
           config: LLMSpellConfig,
           provider_manager: Arc<ProviderManager>,
       ) -> Result<Self> {
           let engine = crate::lua::engine::LuaEngine::new_with_provider(
               config.clone(),
               provider_manager.clone()
           )?;
           // ... existing impl
       }

       /// Create runtime with specified engine by name
       pub async fn new_with_engine_name(
           engine_name: &str,
           config: LLMSpellConfig,
       ) -> Result<Self> {
           match engine_name {
               #[cfg(feature = "lua")]
               "lua" => Self::new_with_lua(config).await,

               #[cfg(feature = "javascript")]
               "javascript" => {
                   let engine = crate::javascript::engine::JsEngine::new(config.clone())?;
                   Self::new_with_engine(Arc::new(engine), config).await
               }

               _ => Err(LLMSpellError::Configuration(format!(
                   "Unsupported or disabled engine: '{}'. Available engines: {}",
                   engine_name,
                   Self::available_engines().join(", ")
               ))),
           }
       }

       /// List available engines based on compiled features
       pub fn available_engines() -> Vec<&'static str> {
           let mut engines = Vec::new();
           #[cfg(feature = "lua")]
           engines.push("lua");
           #[cfg(feature = "javascript")]
           engines.push("javascript");
           engines
       }
   }
   ```
2. Update lib.rs factory functions:
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
3. Test compilation all ways:
   ```bash
   cargo check -p llmspell-bridge --no-default-features
   cargo check -p llmspell-bridge --features lua
   cargo check -p llmspell-bridge --features javascript
   cargo check -p llmspell-bridge --all-features
   ```
4. Run clippy on all configs

**Definition of Done:**
- [ ] Factory methods properly gated
- [ ] Helpful error messages for missing features
- [ ] `available_engines()` reflects compiled features
- [ ] All feature combinations compile
- [ ] Zero clippy warnings
- [ ] Git commit: "fix(bridge): Feature-gate runtime factory methods"

### Task 11a.3.4: Audit and Fix globals/ Module
**Priority**: HIGH
**Estimated Time**: 1 hour
**Assignee**: Core Developer

**Description**: Ensure globals/ contains only language-neutral code or is properly gated.

**Acceptance Criteria:**
- [ ] Language-neutral trait definitions not gated
- [ ] Language-specific implementations properly gated
- [ ] No mlua types in ungated code
- [ ] No boa_engine types in ungated code
- [ ] Compilation succeeds without any language features

**Implementation Steps:**
1. Audit `src/globals/*.rs` files:
   ```bash
   grep -l "mlua" src/globals/*.rs > /tmp/globals-lua-deps.txt
   grep -l "boa_engine" src/globals/*.rs > /tmp/globals-js-deps.txt
   ```
2. For each file with language deps, add appropriate gates:
   ```rust
   // Example in a global that has Lua-specific code
   #[cfg(feature = "lua")]
   use mlua::{Lua, Table, UserData};

   // Keep trait definitions ungated
   pub trait AgentGlobalProvider {
       fn metadata(&self) -> GlobalMetadata;
   }

   // Gate concrete implementations
   #[cfg(feature = "lua")]
   impl LuaAgentGlobal {
       // Lua-specific code
   }
   ```
3. Test each configuration:
   ```bash
   cargo check -p llmspell-bridge --no-default-features
   cargo check -p llmspell-bridge --features lua
   ```
4. Run clippy on affected files

**Definition of Done:**
- [ ] All language deps properly gated in globals/
- [ ] Trait definitions remain accessible without features
- [ ] Compiles cleanly with --no-default-features
- [ ] Zero clippy warnings
- [ ] Git commit: "fix(bridge): Feature-gate language deps in globals"

---

## Phase 11a.4: Test Suite Updates (3 hours)

### Task 11a.4.1: Add Feature Gates to Test Files
**Priority**: HIGH
**Estimated Time**: 1 hour
**Assignee**: QA Engineer

**Description**: Feature-gate test files that require specific languages.

**Acceptance Criteria:**
- [ ] Lua-specific tests gated with `#[cfg(feature = "lua")]`
- [ ] JavaScript tests gated with `#[cfg(feature = "javascript")]`
- [ ] Language-neutral tests run without features
- [ ] Test matrix documented

**Implementation Steps:**
1. Audit test files:
   ```bash
   find llmspell-bridge/tests -name "*.rs" | while read f; do
     if grep -q "mlua" "$f"; then
       echo "LUA: $f"
     fi
     if grep -q "boa_engine" "$f"; then
       echo "JS: $f"
     fi
   done > /tmp/test-feature-audit.txt
   ```
2. Add module-level gates to Lua tests:
   ```rust
   #![cfg(feature = "lua")]

   use llmspell_bridge::ScriptRuntime;
   // ... test code
   ```
3. Add module-level gates to JS tests:
   ```rust
   #![cfg(feature = "javascript")]

   // ... test code
   ```
4. Identify language-neutral tests (should run without features)
5. Test all configurations:
   ```bash
   cargo test -p llmspell-bridge --no-default-features
   cargo test -p llmspell-bridge --features lua
   cargo test -p llmspell-bridge --features javascript
   cargo test -p llmspell-bridge --all-features
   ```

**Definition of Done:**
- [ ] All tests properly feature-gated
- [ ] --no-default-features runs language-neutral tests only
- [ ] Each feature flag runs appropriate tests
- [ ] Test matrix documented in tests/README.md
- [ ] Zero clippy warnings
- [ ] Git commit: "test(bridge): Feature-gate language-specific tests"

### Task 11a.4.2: Fix Integration Tests
**Priority**: HIGH
**Estimated Time**: 1 hour
**Assignee**: QA Engineer

**Description**: Update integration tests in tests/ directory for feature combinations.

**Acceptance Criteria:**
- [ ] Integration tests compile with all feature combos
- [ ] Cross-language tests properly gated
- [ ] Session/workflow tests work without specific languages
- [ ] RAG tests work without specific languages

**Implementation Steps:**
1. Review `tests/integration/*.rs` files
2. Identify tests that can run language-neutral:
   - State persistence tests
   - Session management tests (if not using scripts)
   - RAG tests (if not using scripts)
3. Feature-gate tests requiring scripts:
   ```rust
   #[test]
   #[cfg(feature = "lua")]
   fn test_lua_workflow() {
       // ...
   }
   ```
4. Create feature-specific test modules:
   ```rust
   #[cfg(feature = "lua")]
   mod lua_integration {
       // Lua-specific integration tests
   }

   #[cfg(feature = "javascript")]
   mod js_integration {
       // JS-specific integration tests
   }
   ```
5. Run comprehensive test suite:
   ```bash
   cargo test -p llmspell-bridge --no-default-features
   cargo test -p llmspell-bridge --features lua
   cargo test -p llmspell-bridge --all-features
   ```

**Definition of Done:**
- [ ] All integration tests properly gated
- [ ] Tests pass for all feature combinations
- [ ] Zero test failures
- [ ] Zero clippy warnings
- [ ] Git commit: "test(bridge): Feature-gate integration tests"

### Task 11a.4.3: Update Benchmark Gates
**Priority**: MEDIUM
**Estimated Time**: 1 hour
**Assignee**: Performance Engineer

**Description**: Feature-gate benchmark files appropriately.

**Acceptance Criteria:**
- [ ] Benchmarks compile with correct features
- [ ] Feature requirements documented in bench comments
- [ ] Benchmarks excluded when features missing
- [ ] Performance baselines established

**Implementation Steps:**
1. Review `llmspell-bridge/benches/*.rs` files
2. Add feature requirements to Cargo.toml:
   ```toml
   [[bench]]
   name = "workflow_bridge_bench"
   harness = false
   required-features = ["lua"]

   [[bench]]
   name = "session_bench"
   harness = false
   required-features = ["lua"]
   ```
3. Add feature gates in bench source if needed:
   ```rust
   #![cfg(feature = "lua")]
   ```
4. Test benchmark compilation:
   ```bash
   cargo bench -p llmspell-bridge --no-run --features lua
   cargo bench -p llmspell-bridge --no-run --no-default-features || echo "Expected: no benches"
   ```
5. Run benchmarks and capture baselines:
   ```bash
   cargo bench -p llmspell-bridge --features lua
   ```

**Definition of Done:**
- [ ] Benchmarks feature-gated correctly
- [ ] Compilation succeeds/skips appropriately
- [ ] Performance baselines captured
- [ ] Zero clippy warnings
- [ ] Git commit: "bench(bridge): Feature-gate benchmarks"

---

## Phase 11a.5: Documentation Updates (2 hours)

### Task 11a.5.1: Update Cargo.toml Documentation
**Priority**: HIGH
**Estimated Time**: 30 minutes
**Assignee**: Documentation Lead

**Description**: Add comprehensive feature documentation to Cargo.toml files.

**Acceptance Criteria:**
- [ ] Feature flags documented in llmspell-bridge/Cargo.toml
- [ ] Feature flags documented in llmspell-cli/Cargo.toml
- [ ] Usage examples provided
- [ ] Migration notes for existing users

**Implementation Steps:**
1. Update `llmspell-bridge/Cargo.toml` with comments:
   ```toml
   [features]
   # Language Support Features
   # By default, NO scripting languages are enabled to minimize binary size and compile time.
   # Users should enable language features at the CLI level or in their application.
   #
   # Available languages:
   #   - lua: Lua 5.4 scripting via mlua (~45s compile, +2MB binary)
   #   - javascript: JavaScript via boa_engine (experimental, ~30s compile, +1.5MB binary)
   #
   # Convenience presets:
   #   - common: Lua + common tools (recommended for most users)
   #   - full: All languages + all tools
   #
   # Example usage:
   #   cargo build --features lua                    # Just Lua
   #   cargo build --features javascript             # Just JavaScript
   #   cargo build --features lua,javascript         # Both languages
   #   cargo build --no-default-features             # No languages (core only)

   default = []  # No default language - choose explicitly
   common = ["lua", "llmspell-tools/common"]
   full = ["lua", "javascript", "llmspell-tools/full"]

   # Script engine features
   lua = ["dep:mlua"]
   javascript = ["dep:boa_engine"]
   ```
2. Update `llmspell-cli/Cargo.toml` similarly
3. Add migration note to llmspell-bridge/README.md
4. Verify comments render correctly

**Definition of Done:**
- [ ] Cargo.toml comments comprehensive
- [ ] Examples clear and accurate
- [ ] Migration guidance provided
- [ ] Git commit: "docs(bridge): Document feature flags in Cargo.toml"

### Task 11a.5.2: Update README Files
**Priority**: HIGH
**Estimated Time**: 30 minutes
**Assignee**: Documentation Lead

**Description**: Update README files with feature flag usage.

**Acceptance Criteria:**
- [ ] llmspell-bridge/README.md updated
- [ ] Root README.md updated with build instructions
- [ ] Feature selection examples provided
- [ ] Compile time comparisons documented

**Implementation Steps:**
1. Update `llmspell-bridge/README.md`:
   ```markdown
   ## Feature Flags

   By default, `llmspell-bridge` does NOT include any scripting language runtimes.
   This keeps compile times fast and binary sizes small for users who don't need
   scripting or are using only specific languages.

   ### Available Features

   - `lua` - Lua 5.4 scripting support (recommended, most mature)
   - `javascript` - JavaScript support (experimental)
   - `common` - Lua + common tools preset
   - `full` - All languages and tools

   ### Usage Examples

   ```toml
   # In your Cargo.toml:
   llmspell-bridge = { version = "0.11", features = ["lua"] }
   ```

   ```bash
   # Building llmspell CLI with Lua (default):
   cargo build -p llmspell-cli

   # Building without any languages:
   cargo build -p llmspell-cli --no-default-features

   # Building with JavaScript only:
   cargo build -p llmspell-cli --no-default-features --features javascript
   ```

   ### Compile Time Comparison

   | Configuration | Clean Build Time | Binary Size |
   |--------------|------------------|-------------|
   | No features  | ~2m              | ~15MB       |
   | Lua only     | ~2m 45s          | ~17MB       |
   | JavaScript   | ~2m 30s          | ~16.5MB     |
   | All features | ~3m 15s          | ~19MB       |

   ### Migration from v0.11.0

   If you were using `llmspell-bridge` directly (most users go through `llmspell-cli`),
   you now need to explicitly enable language features:

   ```toml
   # Before (v0.11.0):
   llmspell-bridge = "0.11"  # Lua included by default

   # After (v0.11.1+):
   llmspell-bridge = { version = "0.11.1", features = ["lua"] }
   ```

   The `llmspell` CLI maintains backward compatibility - Lua is still enabled by default.
   ```
   ```
2. Update root README.md build section
3. Verify markdown renders correctly
4. Add to CHANGELOG.md

**Definition of Done:**
- [ ] READMEs updated with feature info
- [ ] Examples tested and accurate
- [ ] Markdown renders correctly
- [ ] Git commit: "docs: Update README with feature flag usage"

### Task 11a.5.3: Update API Documentation
**Priority**: MEDIUM
**Estimated Time**: 30 minutes
**Assignee**: Documentation Lead

**Description**: Update doc comments for feature-gated items.

**Acceptance Criteria:**
- [ ] Feature requirements in doc comments
- [ ] Examples show feature flags
- [ ] Compile errors mention features
- [ ] cargo doc builds successfully

**Implementation Steps:**
1. Add feature requirements to doc comments:
   ```rust
   /// Create a new script runtime with Lua engine.
   ///
   /// # Features
   ///
   /// Requires the `lua` feature to be enabled.
   ///
   /// # Examples
   ///
   /// ```toml
   /// # Cargo.toml
   /// llmspell-bridge = { version = "0.11", features = ["lua"] }
   /// ```
   ///
   /// ```rust,no_run
   /// # #[cfg(feature = "lua")]
   /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
   /// use llmspell_bridge::ScriptRuntime;
   /// use llmspell_config::LLMSpellConfig;
   ///
   /// let runtime = ScriptRuntime::new_with_lua(LLMSpellConfig::default()).await?;
   /// # Ok(())
   /// # }
   /// ```
   #[cfg(feature = "lua")]
   pub async fn new_with_lua(config: LLMSpellConfig) -> Result<Self> {
       // ...
   }
   ```
2. Update lib.rs module docs
3. Build docs with different features:
   ```bash
   cargo doc -p llmspell-bridge --no-deps --no-default-features
   cargo doc -p llmspell-bridge --no-deps --features lua
   cargo doc -p llmspell-bridge --no-deps --all-features
   ```
4. Verify feature flags shown in docs

**Definition of Done:**
- [ ] Doc comments include feature requirements
- [ ] Examples compile with correct features
- [ ] cargo doc succeeds for all configs
- [ ] Git commit: "docs(bridge): Add feature requirements to API docs"

### Task 11a.5.4: Create Migration Guide
**Priority**: MEDIUM
**Estimated Time**: 30 minutes
**Assignee**: Documentation Lead

**Description**: Create comprehensive migration guide for v0.11.0 -> v0.11.1.

**Acceptance Criteria:**
- [ ] Migration guide document created
- [ ] Breaking changes documented
- [ ] Code examples provided
- [ ] Troubleshooting section included

**Implementation Steps:**
1. Create `docs/migration/v0.11.0-to-v0.11.1.md`:
   ```markdown
   # Migration Guide: v0.11.0 to v0.11.1

   ## Summary of Changes

   Version 0.11.1 removes default language features from `llmspell-bridge` to improve
   compile times and reduce binary sizes. This is a **non-breaking change** for
   `llmspell` CLI users, but may require updates if you use `llmspell-bridge` directly.

   ## Who is Affected?

   - ✅ **llmspell CLI users**: No changes required (backward compatible)
   - ⚠️ **Rust developers using llmspell-bridge directly**: Feature flags required
   - ✅ **Script users**: No changes to Lua/JS scripts

   ## Migration Steps

   ### For llmspell-bridge Direct Users

   #### Before (v0.11.0)
   ```toml
   [dependencies]
   llmspell-bridge = "0.11.0"  # Lua enabled by default
   ```

   #### After (v0.11.1+)
   ```toml
   [dependencies]
   llmspell-bridge = { version = "0.11.1", features = ["lua"] }
   ```

   ### Common Scenarios

   #### Scenario 1: "I just want Lua support (most common)"
   ```toml
   llmspell-bridge = { version = "0.11.1", features = ["lua"] }
   ```

   #### Scenario 2: "I want both Lua and JavaScript"
   ```toml
   llmspell-bridge = { version = "0.11.1", features = ["lua", "javascript"] }
   ```

   #### Scenario 3: "I don't need any scripting languages"
   ```toml
   llmspell-bridge = { version = "0.11.1", default-features = false }
   ```

   #### Scenario 4: "I want the common preset"
   ```toml
   llmspell-bridge = { version = "0.11.1", features = ["common"] }
   ```

   ## Troubleshooting

   ### Error: "cannot find function `new_with_lua`"

   **Cause**: Lua feature not enabled.

   **Solution**: Add `features = ["lua"]` to your dependency.

   ### Error: "`mlua` is not found"

   **Cause**: Trying to use Lua without the feature flag.

   **Solution**: Enable the `lua` feature.

   ### Compile Time Not Improved

   **Cause**: Dependencies might still enable features.

   **Solution**: Check `cargo tree -f "{p} {f}"` for feature activation.

   ## Benefits

   After migration, you'll see:

   - 🚀 ~45s faster compile time (if not using Lua)
   - 📦 ~2-3MB smaller binaries (if not using all languages)
   - 🎯 Explicit feature selection (better control)

   ## Questions?

   See [README](../../llmspell-bridge/README.md) for more examples.
   ```
2. Link from CHANGELOG.md
3. Link from README.md
4. Review with team

**Definition of Done:**
- [ ] Migration guide complete
- [ ] Examples tested
- [ ] Linked from relevant docs
- [ ] Git commit: "docs: Add v0.11.0 to v0.11.1 migration guide"

---

## Phase 11a.6: CI/CD Updates (2 hours)

### Task 11a.6.1: Update CI Test Matrix
**Priority**: HIGH
**Estimated Time**: 1 hour
**Assignee**: DevOps Engineer

**Description**: Update CI to test all feature combinations.

**Acceptance Criteria:**
- [ ] CI tests --no-default-features
- [ ] CI tests --features lua
- [ ] CI tests --features javascript
- [ ] CI tests --all-features
- [ ] Test matrix documented

**Implementation Steps:**
1. Edit `.github/workflows/rust-ci.yml` (or equivalent):
   ```yaml
   strategy:
     matrix:
       features:
         - ""  # no features
         - "lua"
         - "javascript"
         - "lua,javascript"
       include:
         - features: ""
           name: "No features"
         - features: "lua"
           name: "Lua only"
         - features: "javascript"
           name: "JavaScript only"
         - features: "lua,javascript"
           name: "All languages"

   steps:
     - name: Check (${{ matrix.name }})
       run: |
         if [ -z "${{ matrix.features }}" ]; then
           cargo check -p llmspell-bridge --no-default-features
         else
           cargo check -p llmspell-bridge --no-default-features --features ${{ matrix.features }}
         fi

     - name: Test (${{ matrix.name }})
       run: |
         if [ -z "${{ matrix.features }}" ]; then
           cargo test -p llmspell-bridge --no-default-features
         else
           cargo test -p llmspell-bridge --no-default-features --features ${{ matrix.features }}
         fi

     - name: Clippy (${{ matrix.name }})
       run: |
         if [ -z "${{ matrix.features }}" ]; then
           cargo clippy -p llmspell-bridge --no-default-features -- -D warnings
         else
           cargo clippy -p llmspell-bridge --no-default-features --features ${{ matrix.features }} -- -D warnings
         fi
   ```
2. Add binary size check job:
   ```yaml
   - name: Binary Size Check
     run: |
       cargo build -p llmspell-cli --release --no-default-features
       NO_LANG_SIZE=$(stat -f%z target/release/llmspell || stat -c%s target/release/llmspell)

       cargo build -p llmspell-cli --release
       WITH_LUA_SIZE=$(stat -f%z target/release/llmspell || stat -c%s target/release/llmspell)

       echo "Binary size without languages: $NO_LANG_SIZE bytes"
       echo "Binary size with Lua: $WITH_LUA_SIZE bytes"
       echo "Difference: $(($WITH_LUA_SIZE - $NO_LANG_SIZE)) bytes"
   ```
3. Test CI locally if possible
4. Push and verify CI passes

**Definition of Done:**
- [ ] CI tests all feature combinations
- [ ] All CI jobs pass
- [ ] Binary size differences logged
- [ ] Test matrix documented
- [ ] Git commit: "ci: Add feature flag test matrix"

### Task 11a.6.2: Update Quality Check Scripts
**Priority**: MEDIUM
**Estimated Time**: 1 hour
**Assignee**: DevOps Engineer

**Description**: Update local quality check scripts for feature testing.

**Acceptance Criteria:**
- [ ] quality-check-minimal.sh tests multiple features
- [ ] quality-check-fast.sh includes feature matrix
- [ ] Scripts document feature coverage
- [ ] Scripts remain fast

**Implementation Steps:**
1. Edit `scripts/quality/quality-check-minimal.sh`:
   ```bash
   #!/bin/bash
   set -e

   echo "=== Quality Check: Minimal (with feature matrix) ==="

   echo "--- Format check ---"
   cargo fmt --all --check

   echo "--- Clippy (no features) ---"
   cargo clippy -p llmspell-bridge --no-default-features --all-targets -- -D warnings

   echo "--- Clippy (lua) ---"
   cargo clippy -p llmspell-bridge --no-default-features --features lua --all-targets -- -D warnings

   echo "--- Clippy (all features) ---"
   cargo clippy --workspace --all-features --all-targets -- -D warnings

   echo "--- Compile check (no features) ---"
   cargo check -p llmspell-bridge --no-default-features

   echo "--- Compile check (lua) ---"
   cargo check -p llmspell-bridge --features lua

   echo "✅ Minimal quality check passed with feature matrix!"
   ```
2. Edit `scripts/quality/quality-check-fast.sh`:
   ```bash
   #!/bin/bash
   set -e

   # Run minimal checks first
   ./scripts/quality/quality-check-minimal.sh

   echo "--- Unit tests (no features) ---"
   cargo test -p llmspell-bridge --no-default-features --lib

   echo "--- Unit tests (lua) ---"
   cargo test -p llmspell-bridge --features lua --lib

   echo "--- Doc tests (lua) ---"
   cargo test -p llmspell-bridge --features lua --doc

   echo "✅ Fast quality check passed with feature matrix!"
   ```
3. Test scripts locally:
   ```bash
   ./scripts/quality/quality-check-minimal.sh
   ./scripts/quality/quality-check-fast.sh
   ```
4. Update script documentation

**Definition of Done:**
- [ ] Scripts test feature combinations
- [ ] Scripts run successfully
- [ ] Performance acceptable (<5min for fast)
- [ ] Documentation updated
- [ ] Git commit: "chore: Update quality scripts for feature matrix"

---

## Phase 11a.7: Performance Validation (2 hours)

### Task 11a.7.1: Measure Compile Time Improvements
**Priority**: HIGH
**Estimated Time**: 1 hour
**Assignee**: Performance Engineer

**Description**: Measure and document compile time improvements.

**Acceptance Criteria:**
- [ ] Baseline compile times captured (pre-cleanup)
- [ ] Post-cleanup compile times measured
- [ ] Improvements documented with percentages
- [ ] Results reproducible

**Implementation Steps:**
1. Create benchmark script `scripts/benchmark-compile-time.sh`:
   ```bash
   #!/bin/bash
   set -e

   echo "=== Compile Time Benchmark ==="

   # Clean first
   cargo clean

   echo "--- Measuring: No features ---"
   time cargo build -p llmspell-cli --no-default-features --release 2>&1 | \
     grep "Finished" | tee /tmp/compile-no-features.txt
   NO_FEAT_TIME=$(grep "Finished" /tmp/compile-no-features.txt | awk '{print $(NF-1), $NF}')

   cargo clean

   echo "--- Measuring: Lua only ---"
   time cargo build -p llmspell-cli --release 2>&1 | \
     grep "Finished" | tee /tmp/compile-lua.txt
   LUA_TIME=$(grep "Finished" /tmp/compile-lua.txt | awk '{print $(NF-1), $NF}')

   cargo clean

   echo "--- Measuring: All features ---"
   time cargo build -p llmspell-cli --release --all-features 2>&1 | \
     grep "Finished" | tee /tmp/compile-all.txt
   ALL_TIME=$(grep "Finished" /tmp/compile-all.txt | awk '{print $(NF-1), $NF}')

   echo ""
   echo "=== Results ==="
   echo "No features:  $NO_FEAT_TIME"
   echo "Lua only:     $LUA_TIME"
   echo "All features: $ALL_TIME"
   ```
2. Run benchmark 3 times, average results
3. Compare with Phase 11 baseline (if available)
4. Document in `/tmp/phase11a-compile-perf.md`
5. Create graphs if possible

**Definition of Done:**
- [ ] Compile times measured for all configs
- [ ] Results documented with comparison
- [ ] Improvements quantified (if any)
- [ ] Report saved to docs/performance/
- [ ] Git commit: "perf: Document compile time improvements"

### Task 11a.7.2: Measure Binary Size Improvements
**Priority**: HIGH
**Estimated Time**: 30 minutes
**Assignee**: Performance Engineer

**Description**: Measure and document binary size reductions.

**Acceptance Criteria:**
- [ ] Binary sizes measured for each config
- [ ] Size differences documented
- [ ] Results reproducible
- [ ] Comparison with Phase 11 baseline

**Implementation Steps:**
1. Build all configurations:
   ```bash
   cargo build -p llmspell-cli --release --no-default-features
   cp target/release/llmspell target/llmspell-no-features

   cargo build -p llmspell-cli --release
   cp target/release/llmspell target/llmspell-lua

   cargo build -p llmspell-cli --release --all-features
   cp target/release/llmspell target/llmspell-all
   ```
2. Measure sizes:
   ```bash
   ls -lh target/llmspell-* | awk '{print $9, $5}' | tee /tmp/binary-sizes.txt
   ```
3. Calculate differences:
   ```bash
   NO_FEAT=$(stat -f%z target/llmspell-no-features || stat -c%s target/llmspell-no-features)
   LUA=$(stat -f%z target/llmspell-lua || stat -c%s target/llmspell-lua)
   ALL=$(stat -f%z target/llmspell-all || stat -c%s target/llmspell-all)

   echo "Binary Size Analysis:"
   echo "No features:  $(($NO_FEAT / 1024 / 1024))MB"
   echo "Lua only:     $(($LUA / 1024 / 1024))MB (+$(( ($LUA - $NO_FEAT) / 1024 / 1024))MB)"
   echo "All features: $(($ALL / 1024 / 1024))MB (+$(( ($ALL - $NO_FEAT) / 1024 / 1024))MB)"
   ```
4. Document in performance report
5. Compare with Phase 11 baseline

**Definition of Done:**
- [ ] Binary sizes measured
- [ ] Differences calculated and documented
- [ ] Report saved to docs/performance/
- [ ] Git commit: "perf: Document binary size reductions"

### Task 11a.7.3: Validate Runtime Performance
**Priority**: MEDIUM
**Estimated Time**: 30 minutes
**Assignee**: Performance Engineer

**Description**: Ensure no runtime performance regression from feature-gating.

**Acceptance Criteria:**
- [ ] Benchmark suite runs on all configs
- [ ] No performance regression detected
- [ ] Results documented
- [ ] Baselines established for future

**Implementation Steps:**
1. Run benchmarks for Lua config:
   ```bash
   cargo bench -p llmspell-bridge --features lua
   ```
2. Compare with Phase 11 benchmarks (if available)
3. Verify no significant regression (>5%)
4. Document any anomalies
5. Save results to `docs/performance/phase11a-benchmarks.md`

**Definition of Done:**
- [ ] Benchmarks run successfully
- [ ] No performance regression
- [ ] Results documented
- [ ] Git commit: "perf: Validate runtime performance post-feature-gate"

---

## Phase 11a.8: Final Validation (1 hour)

### Task 11a.8.1: Comprehensive Quality Gate
**Priority**: CRITICAL
**Estimated Time**: 30 minutes
**Assignee**: QA Lead

**Description**: Run full quality gate on all feature combinations.

**Acceptance Criteria:**
- [ ] All quality scripts pass
- [ ] Zero clippy warnings on all configs
- [ ] All tests pass on all configs
- [ ] Documentation builds on all configs
- [ ] Examples work on relevant configs

**Implementation Steps:**
1. Run comprehensive checks:
   ```bash
   # No features
   cargo clippy -p llmspell-bridge --no-default-features --all-targets -- -D warnings
   cargo test -p llmspell-bridge --no-default-features
   cargo doc -p llmspell-bridge --no-default-features --no-deps

   # Lua
   cargo clippy -p llmspell-bridge --features lua --all-targets -- -D warnings
   cargo test -p llmspell-bridge --features lua
   cargo doc -p llmspell-bridge --features lua --no-deps

   # JavaScript
   cargo clippy -p llmspell-bridge --features javascript --all-targets -- -D warnings
   cargo test -p llmspell-bridge --features javascript
   cargo doc -p llmspell-bridge --features javascript --no-deps

   # All features
   cargo clippy --workspace --all-features --all-targets -- -D warnings
   cargo test --workspace --all-features
   cargo doc --workspace --all-features --no-deps
   ```
2. Run quality scripts:
   ```bash
   ./scripts/quality/quality-check-minimal.sh
   ./scripts/quality/quality-check-fast.sh
   ```
3. Check for any warnings/errors
4. Document any issues found

**Definition of Done:**
- [ ] All checks pass
- [ ] Zero warnings
- [ ] Zero errors
- [ ] All configs validated
- [ ] Ready for merge

### Task 11a.8.2: Update CHANGELOG and Release Notes
**Priority**: HIGH
**Estimated Time**: 30 minutes
**Assignee**: Release Manager

**Description**: Document changes in CHANGELOG and prepare release notes.

**Acceptance Criteria:**
- [ ] CHANGELOG.md updated with Phase 11a changes
- [ ] Migration notes included
- [ ] Breaking changes documented (if any)
- [ ] Benefits highlighted

**Implementation Steps:**
1. Add to CHANGELOG.md:
   ```markdown
   ## [0.11.1] - 2025-10-XX

   ### Changed
   - **BREAKING (minor)**: `llmspell-bridge` no longer enables Lua by default
     - Users depending directly on `llmspell-bridge` must add `features = ["lua"]`
     - `llmspell` CLI users are **not affected** (backward compatible)
     - See [Migration Guide](docs/migration/v0.11.0-to-v0.11.1.md)

   ### Added
   - Feature flag documentation in Cargo.toml files
   - Compile time and binary size comparisons in README
   - Migration guide for direct llmspell-bridge users
   - CI testing for all feature combinations

   ### Performance
   - 🚀 ~45s faster compile time when Lua not needed
   - 📦 ~2-3MB smaller binaries without unused language runtimes
   - ✨ Zero performance regression for enabled features

   ### Fixed
   - Feature gates now consistent across all bridge modules
   - Test suite properly gated for optional language features
   - Documentation examples include required features
   ```
2. Update release notes template
3. Prepare announcement draft
4. Review with team

**Definition of Done:**
- [ ] CHANGELOG.md updated
- [ ] Release notes drafted
- [ ] Migration guide linked
- [ ] Git commit: "docs: Update CHANGELOG for v0.11.1"

---

## Final Validation Checklist

### Compilation Validation
- [ ] `cargo check -p llmspell-bridge --no-default-features` ✅
- [ ] `cargo check -p llmspell-bridge --features lua` ✅
- [ ] `cargo check -p llmspell-bridge --features javascript` ✅
- [ ] `cargo check -p llmspell-bridge --all-features` ✅
- [ ] `cargo check --workspace --all-features` ✅

### Test Validation
- [ ] `cargo test -p llmspell-bridge --no-default-features` ✅
- [ ] `cargo test -p llmspell-bridge --features lua` ✅
- [ ] `cargo test -p llmspell-bridge --features javascript` ✅
- [ ] `cargo test -p llmspell-bridge --all-features` ✅
- [ ] `cargo test --workspace --all-features` ✅

### Quality Validation
- [ ] `cargo clippy -p llmspell-bridge --no-default-features -- -D warnings` ✅
- [ ] `cargo clippy -p llmspell-bridge --features lua -- -D warnings` ✅
- [ ] `cargo clippy -p llmspell-bridge --all-features -- -D warnings` ✅
- [ ] `cargo clippy --workspace --all-features -- -D warnings` ✅
- [ ] `cargo fmt --all --check` ✅

### Documentation Validation
- [ ] `cargo doc -p llmspell-bridge --no-default-features --no-deps` ✅
- [ ] `cargo doc -p llmspell-bridge --features lua --no-deps` ✅
- [ ] `cargo doc --workspace --all-features --no-deps` ✅
- [ ] README examples accurate ✅
- [ ] Migration guide complete ✅

### Performance Validation
- [ ] Compile time measured for all configs ✅
- [ ] Binary size measured for all configs ✅
- [ ] Runtime benchmarks show no regression ✅
- [ ] Improvements documented ✅

### CI/CD Validation
- [ ] CI pipeline tests all feature combos ✅
- [ ] Quality scripts updated ✅
- [ ] All CI jobs pass ✅

### Backward Compatibility
- [ ] llmspell CLI still defaults to Lua ✅
- [ ] Existing user scripts work unchanged ✅
- [ ] Migration path clear for direct bridge users ✅

---

## Success Metrics

### Compile Time Targets
- **No features**: <2m clean build (target: -45s from Lua default)
- **Lua only**: <2m 45s clean build (baseline)
- **All features**: <3m 15s clean build

### Binary Size Targets
- **No features**: ~15MB release binary (target: -2MB from Lua default)
- **Lua only**: ~17MB release binary (baseline)
- **All features**: ~19MB release binary

### Quality Targets
- **Zero** clippy warnings on all feature combinations
- **100%** test pass rate on all feature combinations
- **>95%** API documentation coverage
- **Zero** breaking changes for CLI users

---

## Risk Mitigation

### Technical Risks
1. **Feature gate missing in code**: Mitigated by comprehensive audit (11a.1.1)
2. **Test failures on new configs**: Mitigated by baseline testing (11a.1.2)
3. **Documentation incomplete**: Mitigated by dedicated doc tasks (11a.5.*)
4. **CI complexity increase**: Mitigated by matrix strategy (11a.6.1)

### User Impact Risks
1. **Breaking change for bridge users**: Mitigated by migration guide (11a.5.4)
2. **Confusing error messages**: Mitigated by helpful compile errors (11a.3.3)
3. **Lost backward compat**: Mitigated by CLI maintaining defaults (11a.2.2)

---

## Notes and Decisions Log

### Architectural Decisions
- **Decision**: Remove default features from bridge, not CLI
  - **Rationale**: Maintains backward compatibility for 99% of users
  - **Impact**: Only direct bridge users need to update (rare)

- **Decision**: Use feature flags, not separate crates
  - **Rationale**: Simpler, zero new crates, same benefits
  - **Impact**: Slightly more complex Cargo.toml, but manageable

- **Decision**: Test matrix in CI for all combinations
  - **Rationale**: Ensure quality across all feature sets
  - **Impact**: ~4x CI time, but runs in parallel

### Implementation Notes
- JavaScript remains experimental (~300 LOC stubs)
- Python support deferred to Phase 12+
- Focus on clean feature gates, not refactoring
- Maintain <5% performance overhead for gating checks

---

## Team Assignments

**Architecture Lead**: Overall coordination, feature gate audit
**Build Engineer**: Cargo.toml updates, dependency management
**Core Developer**: Code-level feature gates, runtime factory
**QA Engineer**: Test suite updates, validation matrix
**Performance Engineer**: Compile time, binary size, benchmarks
**Documentation Lead**: README, migration guide, API docs
**DevOps Engineer**: CI/CD updates, quality scripts
**Release Manager**: CHANGELOG, release notes, announcements

---

**END OF PHASE 11a TODO DOCUMENT**
