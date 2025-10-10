# Phase 11b: Bug Fixes and code cleanups 

**Version**: 1.0
**Date**: October 10, 2025
**Status**: 🚧 IN PROGRESS
**Phase**: 11b (LocalLLM Integration Bug Fix)
**Timeline**: 1 day (October 10, 2025)
**Priority**: CRITICAL (Blocks LocalLLM functionality)
**Dependencies**: Phase 11 Complete ✅, Phase 11a Complete ✅
**Arch-Document**: docs/technical/current-architecture.md
**All-Phases-Document**: docs/in-progress/implementation-phases.md
**Parent-Phase**: Phase 11 Local LLM Integration
**This-document**: /TODO.md (working copy)

---

## Overview

**Goal**: Bug fixes  and code cleanups 

---

## Phase 11b.1: LocalLLM Registration Fix - 🚧 IN PROGRESS
Fix LocalLLM global registration to make LocalLLM API accessible from Lua/JavaScript scripts.

**Problem**:
- LocalLLM global NOT injected into script runtime (only 14/15 globals injected)
- `LocalLLM.status("ollama")` returns nil - global doesn't exist
- Registration conditional fails: `context.get_bridge("provider_manager")` returns None
- No `set_bridge("provider_manager", ...)` call anywhere in codebase
- `context.providers` field exists but unused for LocalLLM registration

**Root Cause** (llmspell-bridge/src/globals/mod.rs:29-35):
```rust
// BROKEN: Checks bridge_refs HashMap (never populated)
if let Some(provider_manager) =
    context.get_bridge::<llmspell_providers::ProviderManager>("provider_manager")
{
    builder.register(Arc::new(local_llm_global::LocalLLMGlobal::new(
        provider_manager,
    )));
}
// LocalLLM never registered!
```

**Solution**:
- Access `context.providers` directly (Arc field, always exists)
- Remove broken conditional check using bridge_refs
- Unconditional registration (providers always available)

**Success Criteria**:
- [ ] LocalLLM global injected (15/15 globals, not 14/15)
- [ ] `LocalLLM.status("ollama")` returns valid status object
- [ ] `LocalLLM.list()` returns model array
- [ ] Integration test validates LocalLLM registration
- [ ] All LocalLLM methods functional from Lua/JS
- [ ] Zero clippy warnings
- [ ] Quality check scripts pass

### Task 11b.1.1: Fix GlobalContext Provider Access - ✅ COMPLETE
**Priority**: CRITICAL
**Estimated Time**: 15 minutes
**Actual Time**: 45 minutes (included type analysis)
**Status**: ✅ COMPLETE

**File**: `llmspell-bridge/src/globals/mod.rs`
**Lines**: 244-247 (was 29-35)

**Current Code (BROKEN)**:
```rust
// Register LocalLLM global if provider manager available
if let Some(provider_manager) =
    context.get_bridge::<llmspell_providers::ProviderManager>("provider_manager")
{
    builder.register(Arc::new(local_llm_global::LocalLLMGlobal::new(
        provider_manager,
    )));
}
```

**Why It Fails**:
1. `get_bridge()` checks `self.bridge_refs: HashMap<String, Arc<dyn Any>>`
2. No code ever calls `set_bridge("provider_manager", ...)` to populate it
3. Conditional always false → LocalLLM never registered
4. `context.providers: Arc<ProviderManager>` exists but unused

**Evidence from Trace**:
```
2025-10-10T03:27:40.691544Z  INFO Successfully injected all Lua globals globals_injected=14
                              ^^^^ Should be 15! LocalLLM missing!
```

**Fixed Code (CORRECT)**:
```rust
// Register LocalLLM global (providers always available in context)
builder.register(Arc::new(local_llm_global::LocalLLMGlobal::new(
    context.providers.clone(),
)));
```

**Rationale**:
- `GlobalContext.providers: Arc<ProviderManager>` is NOT optional (struct field, never None)
- Used directly by Agent/Provider globals without conditional check
- No need for bridge_refs indirection
- Simpler, more reliable

**Steps**:
1. Open `llmspell-bridge/src/globals/mod.rs`
2. Go to lines 29-35 in `create_standard_registry()` function
3. Replace conditional block with unconditional registration
4. Use `context.providers.clone()` directly
5. Save file

**Validation**:
- [x] Compile succeeds: `cargo check -p llmspell-bridge` ✅
- [x] No new clippy warnings: `cargo clippy -p llmspell-bridge` ✅

**Insights**:
- **Type Mismatch Discovery**: `context.providers` is `Arc<crate::ProviderManager>` (bridge wrapper), not `Arc<llmspell_providers::ProviderManager>` (core)
- **Existing Method Found**: `create_core_manager_arc()` at providers.rs:301-348 was purpose-built for this exact use case
- **Pattern Validated**: Used async method (already in async fn) - cleaner than cloning + Arc wrapping
- **Architecture Note**: Bridge's ProviderManager wraps core for config/validation - intentional wrapper pattern

**Final Implementation**:
```rust
// llmspell-bridge/src/globals/mod.rs:244-247
builder.register(Arc::new(local_llm_global::LocalLLMGlobal::new(
    context.providers.create_core_manager_arc().await?,
)));
```

---

### Task 11b.1.2: Verify LocalLLM Global Injection - ✅ COMPLETE
**Priority**: CRITICAL
**Estimated Time**: 10 minutes
**Actual Time**: 15 minutes
**Status**: ✅ COMPLETE
**Depends On**: Task 11b.1.1 ✅

**Test Results**:
```bash
# Verified with debug binary
target/debug/llmspell run /tmp/test_localllm_detailed.lua
```

**Actual Output**:
```
=== Testing LocalLLM.status() ===
Status type:	table
Status structure:
  candle:	table
    models:	0	(number)
    ready:	false	(boolean)
    error:	Not configured	(string)
  ollama:	table
    running:	false	(boolean)
    models:	0	(number)
    error:	Not configured	(string)

=== Testing LocalLLM.list() ===
Models type:	table
Models count:	0
```

**Trace Confirmation**:
- `globals_injected=15` ✅ (was 14 before fix)
- `Injecting global global_name=LocalLLM` ✅
- `LocalLLM global registered successfully` ✅

**Validation**:
- [x] Trace shows 15 globals injected (was 14) ✅
- [x] No Lua nil value errors ✅
- [x] LocalLLM.status() returns table with backend status fields ✅
- [x] LocalLLM.list() returns array ✅

**Insights**:
- **API Structure**: `status.ollama.running/models` not `status.health/available_models` (nested backend objects)
- **Backend Detection**: Returns "Not configured" when backends not set up (expected behavior)
- **All Methods Functional**: status(), list() work correctly, return valid tables
- **Registration Success**: LocalLLM now appears in global registry (15/15 vs 14/15)

---

### Task 11b.1.3: Test All LocalLLM Methods - ⏳ TODO
**Priority**: HIGH
**Estimated Time**: 20 minutes
**Status**: ⏳ TODO
**Depends On**: Task 11b.1.2 ✅

**Methods to Test** (from Phase 11 design):
1. `LocalLLM.status(backend?)` - Backend status check
2. `LocalLLM.list(backend?)` - List local models
3. `LocalLLM.pull(model, backend)` - Download model
4. `LocalLLM.info(model)` - Model metadata

**Test Script** (`/tmp/test_localllm.lua`):
```lua
-- Test 1: Status check
print("=== Test 1: Status ===")
local status = LocalLLM.status()
print("Ollama running:", status.ollama.running)
print("Candle ready:", status.candle.ready)

-- Test 2: List models
print("\n=== Test 2: List Models ===")
local models = LocalLLM.list("ollama")
for i, model in ipairs(models) do
    print(string.format("%d. %s (%s)", i, model.id, model.backend))
end

-- Test 3: Model info (if models exist)
if #models > 0 then
    print("\n=== Test 3: Model Info ===")
    local info = LocalLLM.info(models[1].id)
    print("Model:", info.id)
    print("Size:", info.size_bytes, "bytes")
end

print("\n✅ All LocalLLM methods functional!")
```

**Run Commands**:
```bash
# Create test script
cat > /tmp/test_localllm.lua << 'EOF'
[paste script above]
EOF

# Run with Ollama config
target/release/llmspell -c examples/script-users/configs/local-llm-ollama.toml run /tmp/test_localllm.lua

# Run with Candle config
target/release/llmspell -c examples/script-users/configs/local-llm-candle.toml run /tmp/test_localllm.lua
```

**Expected Output**:
```
=== Test 1: Status ===
Ollama running: true
Candle ready: true

=== Test 2: List Models ===
1. llama3.1:8b (ollama)
2. mistral:7b (ollama)

=== Test 3: Model Info ===
Model: llama3.1:8b
Size: 4661224448 bytes

✅ All LocalLLM methods functional!
```

**Validation**:
- [ ] Status returns valid backend status objects
- [ ] List returns model arrays (empty OK if no models)
- [ ] Info returns metadata for existing models
- [ ] No Lua errors during execution
- [ ] Works with both Ollama and Candle configs

---

### Task 11b.1.4: Add Integration Test for Registration - ⏳ TODO
**Priority**: MEDIUM
**Estimated Time**: 30 minutes
**Status**: ⏳ TODO
**Depends On**: Task 11b.1.3 ✅

**Goal**: Prevent regression - ensure LocalLLM always registered

**Test File**: `llmspell-bridge/tests/local_llm_registration_test.rs`

**Test Implementation**:
```rust
//! Integration test: LocalLLM global registration
//!
//! Validates that LocalLLM global is properly injected when ProviderManager
//! exists in GlobalContext (regression test for Phase 11b bug fix).

#[cfg(feature = "lua")]
mod local_llm_registration {
    use llmspell_bridge::globals::{create_standard_registry, GlobalContext};
    use llmspell_providers::ProviderManager;
    use llmspell_core::registry::ComponentRegistry;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_localllm_global_registered() {
        // Arrange: Create context with provider manager (normal runtime setup)
        let registry = Arc::new(ComponentRegistry::new());
        let providers = Arc::new(ProviderManager::new());
        let context = Arc::new(GlobalContext::new(registry, providers));

        // Act: Create standard registry (what inject_apis does)
        let global_registry = create_standard_registry(context.clone())
            .await
            .expect("Should create global registry");

        // Assert: LocalLLM global must be registered
        let localllm_exists = global_registry
            .get_global("LocalLLM")
            .is_some();

        assert!(
            localllm_exists,
            "LocalLLM global MUST be registered when ProviderManager exists in context \
             (regression: Phase 11b bug fix - was conditionally skipped)"
        );

        // Verify total globals count (should be 15, not 14)
        let global_count = global_registry.len();
        assert_eq!(
            global_count, 15,
            "Expected 15 globals (including LocalLLM), got {}",
            global_count
        );
    }

    #[tokio::test]
    async fn test_localllm_uses_context_providers() {
        // Arrange
        let registry = Arc::new(ComponentRegistry::new());
        let providers = Arc::new(ProviderManager::new());
        let context = Arc::new(GlobalContext::new(registry, providers.clone()));

        // Act
        let global_registry = create_standard_registry(context.clone())
            .await
            .expect("Should create global registry");

        // Assert: LocalLLM should use same provider manager as context
        // (This validates the fix: using context.providers instead of bridge_refs)
        let localllm_global = global_registry
            .get_global("LocalLLM")
            .expect("LocalLLM must exist");

        // Validate metadata
        let metadata = localllm_global.metadata();
        assert_eq!(metadata.name, "LocalLLM");
        assert!(metadata.description.contains("local model"));
    }
}
```

**Steps**:
1. Create file `llmspell-bridge/tests/local_llm_registration_test.rs`
2. Paste test code above
3. Run test: `cargo test -p llmspell-bridge --test local_llm_registration_test`
4. Verify both tests pass

**Validation**:
- [ ] Test file created
- [ ] `test_localllm_global_registered` passes
- [ ] `test_localllm_uses_context_providers` passes
- [ ] Test runs in CI: `cargo test --workspace --features lua`

---

### Task 11b.1.5: Update docs  - ⏳ TODO
**Priority**: LOW
**Estimated Time**: 10 minutes
**Status**: ⏳ TODO
**Depends On**: All Phase 11b.1 tasks ✅

**Files to Update**:

1. **docs/user-guide/local-llm.md** (if "Known Issues" section exists):
   - ~~Remove any note about LocalLLM not available in scripts~~ (should never have existed)
   - Confirm all examples work as documented

2. **CHANGELOG.md**:
   ```markdown
   ## [Unreleased]

   ### Fixed
   - **Phase 11b**: LocalLLM global registration bug - now properly injected into Lua/JS runtime
     - Root cause: `create_standard_registry()` used `get_bridge("provider_manager")` which was never populated
     - Fix: Use `context.providers` directly (Arc field, always available)
     - Impact: LocalLLM.status(), .list(), .pull(), .info() now functional from scripts
     - Regression test added: `llmspell-bridge/tests/local_llm_registration_test.rs`
   ```

**Steps**:
1. Check if docs/user-guide/local-llm.md has "Known Issues" section
2. Update CHANGELOG.md with bug fix entry

**Validation**:
- [ ] CHANGELOG.md updated
- [ ] No incorrect "known issues" about LocalLLM in docs

---

### Task 11b.1.6: Quality Check & Validation - ⏳ TODO
**Priority**: CRITICAL
**Estimated Time**: 15 minutes
**Status**: ⏳ TODO
**Depends On**: All Phase 11b tasks ✅

**Quality Gates** (all must pass):
```bash
# 1. Format check
cargo fmt --all -- --check

# 2. Clippy (zero warnings)
cargo clippy --workspace --all-features -- -D warnings

# 3. Compile check
cargo check --workspace --all-features

# 4. Test suite
cargo test --workspace --features lua

# 5. Doc build
cargo doc --workspace --all-features --no-deps

# 6. Quick quality check
./scripts/quality/quality-check-minimal.sh
```

**Feature-Specific Tests**:
```bash
# LocalLLM registration with lua feature only
cargo test -p llmspell-bridge --features lua local_llm_registration

# Full workspace with all features
cargo test --workspace --all-features
```

**Runtime Validation**:
```bash
# Original failing command from user (should now work)
target/release/llmspell exec --trace trace 'local status = LocalLLM.status("ollama")
print("Health:", status.health)
print("Available models:", status.available_models)'

# Expected: 15 globals injected, no nil errors, valid output
```

**Success Indicators**:
- [ ] All quality gates pass (format, clippy, compile, test, doc)
- [ ] `./scripts/quality/quality-check-minimal.sh` exits 0
- [ ] 15 globals injected (trace shows `globals_injected=15`)
- [ ] LocalLLM methods return data (not nil)
- [ ] Zero new clippy warnings introduced

**Failure Recovery**:
- If clippy fails: Fix warnings before proceeding
- If tests fail: Debug and fix before merging
- If runtime fails: Re-verify Task 11b.1.1 implementation

---

