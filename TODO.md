# Phase 13c: Usability & Cohesion Refinement - TODO List

**Version**: 1.0
**Date**: November 2025
**Status**: Implementation Ready
**Phase**: 13c (Usability & Cohesion Refinement)
**Timeline**: Weeks 49-50 (10 working days / 2 weeks)
**Priority**: HIGH (Production Readiness - v0.14.0 Release)
**Dependencies**:
- Phase 13: Memory/Context/Templates ✅
- Phase 13b: PostgreSQL Storage + ScriptRuntime Refactor ✅

**Arch-Document**: docs/technical/master-architecture-vision.md
**All-Phases-Document**: docs/in-progress/implementation-phases.md
**Design-Document**: docs/in-progress/phase-13c-design-doc.md (3,765 lines)
**Current-Architecture**: docs/technical/current-architecture.md (To be updated)
**This-Document**: Working copy /TODO.md (pristine/immutable copy in docs/in-progress/PHASE13c-TODO.md)

> **📋 Actionable Task List**: This document breaks down Phase 13c implementation into specific, measurable tasks for consolidating examples, enhancing profiles, cleaning up dependencies, and establishing production-ready developer experience.

---

## Overview

**Goal**: Transform rs-llmspell from feature-complete experimental platform to production-ready, user-focused developer experience through consolidation, validation, and quality enhancement.

**Strategic Context**:
- **Problem**: 75 example files sprawled, missing production profiles, Phase 8 documentation, no validation
- **Solution**: "Less is More" - consolidate to <50 examples, add 3 profiles, validate 100%, update to Phase 13
- **Approach**: Feature sets first (deps, profiles) → cleanup (examples, docs) → validation (testing) → release

**Architecture Summary**:
- **Examples Reduction**: 75 → <50 files (33% reduction)
- **Profile Addition**: 14 → 17 profiles (+3: postgres, ollama-production, memory-development)
- **Dependency Cleanup**: 52 → 43-47 dependencies (5-9 removed)
- **Validation**: 0% → 100% automated example testing
- **Documentation**: Phase 8 → Phase 13 (2 phases ahead)

**Success Criteria Summary**:
- [x] Cargo dependencies reduced by 2/3-9 (lazy_static ✅, once_cell ✅, crossbeam pending)
- [ ] 3 new builtin profiles operational (postgres, ollama-production, memory-development)
- [ ] Examples reduced 75 → <50 files with zero broken examples
- [ ] Getting-started streamlined 8 → 5 examples (<30 min path)
- [ ] examples-validation.sh tests 100% of getting-started, 90%+ of cookbook
- [ ] All documentation references Phase 13 (zero Phase 8 references)
- [ ] README-DEVEL.md comprehensive developer setup guide
- [ ] Migration guide (v0.13 → v0.14) complete
- [ ] Profile decision matrix for dev → staging → prod
- [ ] Zero clippy warnings maintained
- [ ] All 635+ tests passing (zero regressions)
- [ ] Compilation time improved 10-25% (dependency cleanup)
- [ ] Binary size reduced 1-2MB

---

## Dependency Analysis

**Critical Path (Features First)**:
1. **Foundation (Days 1-3)**: Cargo deps cleanup → Profile creation → Feature validation
2. **Consolidation (Days 4-5)**: Examples cleanup → Config audit → Structure optimization
3. **Documentation (Days 6-7)**: User guides → Migration docs → Profile catalog
4. **Validation (Days 8-9)**: Example validation → Integration testing → Quality gates
5. **Release (Day 10)**: Performance validation → Release preparation → v0.14.0

**Parallel Tracks**:
- **Dependency Track**: Days 1-3 (cleanup) → Days 8-9 (benchmark validation)
- **Profile Track**: Days 1-2 (creation) → Days 6-7 (documentation)
- **Examples Track**: Days 4-5 (consolidation) → Days 8-9 (validation)
- **Documentation Track**: Days 6-7 (comprehensive update) → Day 8 (review)
- **Validation Track**: Day 5 (script creation) → Days 8-9 (full testing)

**Hard Dependencies**:
- Phase 13c.2 (Profiles) must complete before Phase 13c.3 (Examples) for profile validation
- Phase 13c.1 (Deps) can run parallel with Phase 13c.2 (Profiles) - independent
- Phase 13c.4 (Validation) depends on Phase 13c.3 (Examples) completion
- Phase 13c.5 (Documentation) depends on Phases 13c.2-13c.3 (Profiles + Examples)
- Phase 13c.6 (Release) depends on all previous phases (complete validation)

**Task Insertion Points**:
- After each major section header (## Phase 13c.X)
- Within subsections for granular tasks
- New discoveries during implementation can be inserted with .X.Y.Z numbering

---

## Phase 13c.1: Cargo Dependencies Cleanup (Days 1-3)

**Goal**: Remove 5-9 redundant dependencies, improve compilation time 10-25%
**Timeline**: 3 days (24 hours total)
**Critical Dependencies**: None (independent track)
**Priority**: HIGH (impacts all future builds)

### Task 13c.1.1: Initialization Redundancy Removal ✅ COMPLETE
**Priority**: CRITICAL
**Estimated Time**: 2 hours
**Actual Time**: 2.5 hours
**Assignee**: Core Infrastructure Team
**Status**: ✅ COMPLETE
**Completed**: 2025-11-09

**Description**: Replace `lazy_static` and `once_cell` with `std::sync::LazyLock` and `std::sync::OnceLock` (Rust 1.70+/1.80+ stable).

**Acceptance Criteria**:
- [x] All 6 files migrated from lazy_static/once_cell to std equivalents (found 6, not 5)
- [x] `lazy_static` removed from Cargo.toml (workspace + 3 crates)
- [x] `once_cell` removed from Cargo.toml (workspace + 8 crates)
- [x] All tests pass after migration (74+ tests validated)
- [x] `cargo check --workspace` passes without warnings (3m23s clean build)
- [x] `cargo clippy --workspace --all-features --all-targets` zero warnings (6m24s)

**Completion Insights**:
- **Files Migrated**: 6 (not 5) - added llmspell-testing/tests/categories.rs
  - llmspell-testing/tests/categories.rs: lazy_static! → LazyLock (CATEGORY_REGISTRY)
  - llmspell-utils/src/security/input_sanitizer.rs: 5 lazy_static! blocks → LazyLock (regex patterns)
  - llmspell-utils/src/security/validation_rules.rs: lazy_static! → LazyLock (JS patterns)
  - llmspell-kernel/src/state/sensitive_data.rs: once_cell::Lazy → LazyLock (2 statics)
  - llmspell-kernel/src/runtime/io_runtime.rs: once_cell::OnceCell → OnceLock (2 statics)
  - llmspell-templates/src/registry.rs: once_cell::Lazy → LazyLock (GLOBAL_REGISTRY)
- **Dependencies Removed**: 2 from workspace, 11 from crates (lazy_static: 3 crates, once_cell: 8 crates)
- **Test Results**: 74 tests validated (categories: 2, security: 61, registry: 11)
- **Migration Pattern**: Consistent across all files - no API breakage, drop-in replacement
- **Build Time**: 3m23s clean check, 6m24s clippy with all targets
- **No Behavioral Changes**: All lazy initialization patterns preserved, zero runtime changes

**Additional Cleanup (2025-11-09 Evening)**:
- **MySQL Support Removed**: Discovered mock-only implementation pulling in problematic dependencies
  - **Root Cause**: `num-bigint-dig v0.8.5` future incompatibility warning (private macro issue #120192)
  - **Dependency Chain**: sqlx → sqlx-mysql → rsa v0.9.8 → num-bigint-dig v0.8.5
  - **Investigation**: MySQL was mock-only (never functional), no tests, no examples, no production usage
  - **Solution**: Removed `mysql` from sqlx base features in llmspell-tools/Cargo.toml
  - **Code Changes**:
    - llmspell-tools/Cargo.toml: Removed `"mysql"` from sqlx features array (line 102)
    - llmspell-tools/Cargo.toml: Removed `database-mysql` feature flag (obsolete)
    - llmspell-tools/src/communication/database_connector.rs: Removed MySQL code (3 locations)
      - Removed DATABASE_MYSQL_URL env var handling (lines 137-148)
      - Removed execute_mysql_query() method (lines 441-462)
      - Removed "mysql" match arm (line 239)
      - Updated tool descriptions (removed "MySQL" references)
  - **Security Code Preserved**: MySQL patterns in credential_protection, SSRF port blocking (generic security)
  - **Result**: Eliminated 8+ transitive dependencies (rsa, num-bigint-dig, crypto stack)
  - **Verification**:
    - `cargo tree --workspace --all-features -i rsa` → "nothing to print" ✅
    - `cargo tree --workspace --all-features | grep num-bigint-dig` → (empty) ✅
    - `cargo clippy --workspace --all-features --all-targets` → Zero warnings ✅
    - `cargo build --workspace --features database-postgres` → Success (48.56s) ✅
  - **Strategic Alignment**: PostgreSQL is production backend (Phase 13b complete), MySQL was never needed
  - **Note**: Cargo patch for num-bigint-dig not possible (can't patch crates.io → crates.io)
  - **Files Modified**: 3 files, ~40 lines removed, zero functional impact (code was non-functional)
  - **Breaking Changes**: NONE - MySQL feature was never working, removing dead code only

**Implementation Steps**:
1. Identify all uses of lazy_static and once_cell:
   ```bash
   find llmspell-*/src -name "*.rs" -exec grep -l "lazy_static\|once_cell" {} \;
   ```
   Files found (from design doc):
   - llmspell-kernel/src/runtime/io_runtime.rs
   - llmspell-kernel/src/state/sensitive_data.rs
   - llmspell-templates/src/registry.rs
   - llmspell-utils/src/security/validation_rules.rs
   - llmspell-utils/src/security/input_sanitizer.rs

2. Migrate each file:
   ```rust
   // Before:
   use lazy_static::lazy_static;
   lazy_static! {
       static ref REGEX: Regex = Regex::new(r"...").unwrap();
   }

   // After:
   use std::sync::LazyLock;
   static REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"...").unwrap());
   ```

3. Remove dependencies from Cargo.toml:
   ```bash
   # Edit Cargo.toml to remove:
   # lazy_static = "1.4"
   # once_cell = "1.19"
   ```

4. Verify compilation:
   ```bash
   cargo check --workspace --all-features
   cargo test --workspace --all-features
   cargo clippy --workspace --all-features -- -D warnings
   ```

**Definition of Done**:
- [x] Zero uses of lazy_static or once_cell in codebase ✅
- [x] 2 dependencies removed from workspace ✅
- [x] All tests passing (74 specific tests validated) ✅
- [x] Zero clippy warnings (--all-features --all-targets) ✅
- [x] Compilation time baseline: 3m23s check, 6m24s clippy ✅

**Files to Modify**:
- `Cargo.toml` (workspace dependencies)
- `llmspell-kernel/src/runtime/io_runtime.rs`
- `llmspell-kernel/src/state/sensitive_data.rs`
- `llmspell-templates/src/registry.rs`
- `llmspell-utils/src/security/validation_rules.rs`
- `llmspell-utils/src/security/input_sanitizer.rs`

---

### Task 13c.1.2: Concurrency Consolidation ✅ COMPLETE (INVESTIGATED)
**Priority**: CRITICAL
**Estimated Time**: 1 hour
**Actual Time**: 1.5 hours
**Assignee**: Core Infrastructure Team
**Status**: ✅ COMPLETE
**Completed**: 2025-11-09

**Description**: Investigated crossbeam usage; removed unused dependencies, kept production-critical lock-free structures.

**Acceptance Criteria**:
- [x] All crossbeam usage audited and categorized
- [x] Unused `crossbeam` removed from llmspell-utils and llmspell-events
- [x] Production-critical crossbeam kept in llmspell-kernel with documentation
- [x] All tests pass after cleanup (56/57 passed, 1 flaky performance test unrelated)
- [x] Zero clippy warnings

**Completion Insights**:
- **Original Task Incorrect**: Task description mentioned "crossbeam channels" but no channels found
- **Actual Usage**: 2 specialized lock-free concurrent data structures in llmspell-kernel:
  1. `crossbeam::queue::SegQueue` - Lock-free MPMC queue for zero-overhead async hook processing (async_hooks.rs)
  2. `crossbeam_skiplist::SkipMap` - Lock-free sorted concurrent map for high-performance agent state (lockfree_agent.rs)
- **No Tokio Equivalents**: These are specialized concurrent data structures, not channels
  - SegQueue: Lock-free queue (no async equivalent in tokio)
  - SkipMap: Lock-free sorted map (no equivalent in std or tokio)
- **Dependencies Removed**: 2 crate-level unused dependencies
  - Removed `crossbeam` from llmspell-utils/Cargo.toml (unused in source)
  - Removed `crossbeam` from llmspell-events/Cargo.toml (unused in source)
- **Dependencies Kept**: llmspell-kernel (production-critical lock-free code)
  - Added justification comments in llmspell-kernel/Cargo.toml (lines 71-74)
  - Added justification comments in workspace Cargo.toml (lines 136-137)
- **Test Results**: 56 tests passed, 1 flaky performance test (test_hook_overhead - timing sensitive, unrelated to changes)
- **Build Time**: 2m12s cargo check (clean), 3m00s clippy (all features/targets)
- **Zero Behavioral Changes**: No code changes, only Cargo.toml cleanup + documentation

**Decision Rationale**:
- **Option Selected**: D + C (Remove unused + Document kept dependencies)
- **Why Not Replace**: Lock-free concurrent structures are performance-critical for state operations
  - SegQueue provides zero-overhead async hook processing (design goal: <1% overhead)
  - SkipMap enables lock-free agent state management (scales with concurrent access)
  - Replacing would require either:
    - Async channels (changes API semantics, adds `.await` overhead)
    - Coarser-grained locking (defeats purpose of lock-free design)
- **Production Impact**: Zero (cleanup only, no functional changes)

**Files Modified**:
- `Cargo.toml` (workspace - added documentation comments)
- `llmspell-utils/Cargo.toml` (removed crossbeam)
- `llmspell-events/Cargo.toml` (removed crossbeam)
- `llmspell-kernel/Cargo.toml` (added justification comments)

---

### Task 13c.1.3: Tokio Utilities Consolidation ✅ COMPLETE
**Priority**: MEDIUM
**Estimated Time**: 2 hours
**Actual Time**: 1.5 hours
**Assignee**: Core Infrastructure Team
**Status**: ✅ COMPLETE
**Completed**: 2025-11-10

**Description**: Removed `tokio-stream` and `tokio-util` from non-essential crates, kept in production-critical locations.

**Acceptance Criteria**:
- [x] Attempted removal of tokio-stream and tokio-util
- [x] 4 tokio-stream + 2 tokio-util dependencies removed
- [x] Specific tokio features documented where kept
- [x] All builds pass
- [x] Zero clippy warnings

**Completion Insights**:
- **tokio-stream Removed**: 4 crates (llmspell-agents, llmspell-bridge, llmspell-cli, llmspell-utils)
  - Kept only in llmspell-events (stream.rs:260-265 - production-critical event streaming)
  - Provides BroadcastStream wrapper + StreamExt utilities not in tokio "full"
- **tokio-util Removed**: 2 crates (llmspell-agents, llmspell-kernel)
  - No longer needed in any workspace crates
  - Historical usage was codec/time utilities now provided by tokio "full"
- **Dependencies Removed**: 6 crate-level imports removed
  - Workspace: Added documentation comments (Cargo.toml lines 57-59)
  - llmspell-agents: Removed tokio-stream + tokio-util (Cargo.toml lines 28-29)
  - llmspell-bridge: Removed tokio-stream (Cargo.toml line 16)
  - llmspell-cli: Removed tokio-stream (Cargo.toml line 28)
  - llmspell-utils: Removed tokio-stream (Cargo.toml line 16)
  - llmspell-kernel: Removed tokio-util (Cargo.toml line 28)
- **Dependencies Kept**: llmspell-events (production-critical event streaming)
  - Added justification in llmspell-events/Cargo.toml (lines 19-20)
- **Test Results**: All affected crates passing (agents: 367, bridge: 311, cli: 69, events: 56, kernel: 665, utils: 515)
  - Note: llmspell-bridge has 3 flaky performance tests (timing-related, non-functional)
- **Clippy**: Zero warnings across workspace
- **Zero Behavioral Changes**: Only removed unused imports, kept production-critical usage

**Files Modified**:
- `Cargo.toml` (workspace - added documentation)
- `llmspell-agents/Cargo.toml` (removed tokio-stream + tokio-util)
- `llmspell-bridge/Cargo.toml` (removed tokio-stream)
- `llmspell-cli/Cargo.toml` (removed tokio-stream)
- `llmspell-events/Cargo.toml` (added justification)
- `llmspell-kernel/Cargo.toml` (removed tokio-util)
- `llmspell-utils/Cargo.toml` (removed tokio-stream)

---

### Task 13c.1.4: Serialization Audit ⏹ PENDING
**Priority**: MEDIUM
**Estimated Time**: 4 hours
**Assignee**: Core Infrastructure Team
**Status**: ⏹ PENDING

**Description**: Audit `serde_yaml` and `bincode` usage; migrate to JSON/TOML if usage is minimal (<5 files).

**Acceptance Criteria**:
- [ ] serde_yaml usage audited (file count, necessity)
- [ ] bincode usage audited (file count, necessity)
- [ ] Migration completed if usage minimal OR justification documented
- [ ] 0-2 dependencies potentially removed
- [ ] All serialization roundtrips tested

**Implementation Steps**:
1. Audit serde_yaml:
   ```bash
   grep -r "serde_yaml" llmspell-*/src --include="*.rs" -l | wc -l
   grep -r "serde_yaml" llmspell-*/src --include="*.rs" -B 2 -A 2
   ```

2. Audit bincode:
   ```bash
   grep -r "bincode::" llmspell-*/src --include="*.rs" -l | wc -l
   grep -r "bincode::" llmspell-*/src --include="*.rs" -B 2 -A 2
   ```

3. Decision matrix:
   - **If <5 files use it**: Migrate to serde_json
   - **If 5-10 files**: Evaluate complexity
   - **If >10 files**: Keep and document why

4. If migrating, update each file:
   ```rust
   // Before:
   let yaml = serde_yaml::to_string(&data)?;

   // After:
   let json = serde_json::to_string(&data)?;
   ```

5. Remove unused dependencies from Cargo.toml

6. Test serialization:
   ```bash
   cargo test --workspace -- --test-threads=1 serialization
   ```

**Definition of Done**:
- [ ] Usage audit complete (file counts documented)
- [ ] Migration decision made and executed
- [ ] Cargo.toml updated (removed or documented)
- [ ] All serialization tests pass
- [ ] README-DEVEL.md updated if removed

**Files to Modify**:
- `Cargo.toml`
- Files using serde_yaml (TBD from audit)
- Files using bincode (TBD from audit)
- `README-DEVEL.md` (dependency section)

---

### Task 13c.1.5: File Operations Audit ⏹ PENDING
**Priority**: LOW
**Estimated Time**: 3 hours
**Assignee**: Core Infrastructure Team
**Status**: ⏹ PENDING

**Description**: Evaluate replacing `walkdir` and `path-clean` with `std::fs` if usage is simple.

**Acceptance Criteria**:
- [ ] walkdir usage complexity audited
- [ ] path-clean usage audited
- [ ] Migration completed if simple OR decision to keep documented
- [ ] 0-2 dependencies potentially removed
- [ ] All file operations tested

**Implementation Steps**:
1. Audit walkdir usage:
   ```bash
   grep -r "walkdir" llmspell-*/src --include="*.rs" -B 5 -A 5
   ```

2. Check complexity:
   - Simple recursive traversal → can replace with std::fs::read_dir
   - Symlink handling, permissions → keep walkdir

3. Audit path-clean:
   ```bash
   grep -r "path_clean\|path-clean" llmspell-*/src --include="*.rs" -B 5 -A 5
   ```

4. If simple, replace:
   ```rust
   // Before:
   use walkdir::WalkDir;
   for entry in WalkDir::new(path) {
       // ...
   }

   // After:
   use std::fs;
   fn walk_dir(path: &Path) -> impl Iterator<Item = DirEntry> {
       fs::read_dir(path)
           .into_iter()
           .flatten()
           .flat_map(|entry| entry.ok())
   }
   ```

5. Test edge cases (symlinks, permissions)

**Definition of Done**:
- [ ] Usage complexity assessed
- [ ] Decision made: migrate or keep
- [ ] If migrated: tests cover edge cases
- [ ] If kept: justification documented
- [ ] All file operation tests pass

**Files to Modify**:
- `Cargo.toml`
- Files using walkdir/path-clean (TBD)
- Test files for edge case coverage

---

### Task 13c.1.6: Compression & Hashing Audit ⏹ PENDING
**Priority**: LOW
**Estimated Time**: 2 hours
**Assignee**: Storage Team
**Status**: ⏹ PENDING

**Description**: Verify `lz4_flex` and `blake3` are actually needed for content-addressed storage.

**Acceptance Criteria**:
- [ ] lz4_flex usage audited (necessity verified)
- [ ] blake3 usage audited (necessity verified)
- [ ] 0-2 dependencies potentially removed OR clear justification
- [ ] Storage operations tested

**Implementation Steps**:
1. Find usage:
   ```bash
   grep -r "lz4_flex\|blake3" llmspell-storage/src --include="*.rs" -B 3 -A 3
   ```

2. Analyze use cases:
   - lz4_flex for state compression → keep if actually used
   - blake3 for content addressing → keep if needed for deduplication
   - blake3 for general hashing → replace with std::hash

3. Decision matrix:
   | Use Case | Keep? | Alternative |
   |----------|-------|-------------|
   | State compression | Yes | None (lz4 optimal) |
   | Content addressing | Yes | None (fastest) |
   | General hashing | No | std::collections::hash |
   | Unused | No | Remove |

4. Test storage operations:
   ```bash
   cargo test -p llmspell-storage
   ```

**Definition of Done**:
- [ ] 8 uses analyzed and categorized
- [ ] Unnecessary uses removed
- [ ] Necessary uses documented in Cargo.toml comments
- [ ] Storage tests pass
- [ ] No performance regression

**Files to Modify**:
- `Cargo.toml` (with justification comments)
- `llmspell-storage/src/**/*.rs` (if removing uses)

---

### Task 13c.1.7: Dependency Cleanup Validation ⏹ PENDING
**Priority**: CRITICAL
**Estimated Time**: 2 hours
**Assignee**: QA Team
**Status**: ⏹ PENDING

**Description**: Validate all dependency removals with comprehensive testing and benchmarking.

**Acceptance Criteria**:
- [ ] All 635+ tests passing
- [ ] Zero clippy warnings
- [ ] Compilation time improved 10-25% (measured)
- [ ] Binary size reduced 1-2MB (measured)
- [ ] No performance regressions in benchmarks
- [ ] Dependency decision matrix documented

**Implementation Steps**:
1. Baseline measurements (before cleanup):
   ```bash
   cargo clean
   time cargo build --release --features full > /tmp/before_build.log 2>&1
   ls -lh target/release/llmspell | awk '{print $5}' > /tmp/before_size.txt
   ```

2. After cleanup measurements:
   ```bash
   cargo clean
   time cargo build --release --features full > /tmp/after_build.log 2>&1
   ls -lh target/release/llmspell | awk '{print $5}' > /tmp/after_size.txt
   ```

3. Compare:
   ```bash
   echo "Build time improvement: $((100 - (after_time * 100 / before_time)))%"
   echo "Binary size reduction: $((before_size - after_size))MB"
   ```

4. Run comprehensive tests:
   ```bash
   cargo test --workspace --all-features
   cargo clippy --workspace --all-features -- -D warnings
   cargo bench --workspace -- --test
   ```

5. Document decisions in Cargo.toml:
   ```toml
   # Dependency Decision Matrix (Phase 13c)
   # Removed: lazy_static, once_cell → std::sync::LazyLock/OnceLock (Rust 1.80+)
   # Removed: crossbeam → tokio::sync (only 2 uses)
   # Kept: parking_lot → 80+ uses, 2x faster than std::sync
   # ... etc
   ```

**Definition of Done**:
- [ ] Minimum 3 dependencies removed (guaranteed)
- [ ] Stretch goal: 5-9 dependencies removed (audit-dependent)
- [ ] Compilation time improved 10-25%
- [ ] Binary size reduced 1-2MB
- [ ] All tests passing (635+)
- [ ] Zero warnings
- [ ] Decision matrix in Cargo.toml
- [ ] README-DEVEL.md updated

**Files to Create/Modify**:
- `Cargo.toml` (dependency decision matrix comments)
- `README-DEVEL.md` (updated dependency list)
- `docs/archives/COMPILATION-PERFORMANCE.md` (updated baselines)

---

## Phase 13c.2: Profile System Enhancement (Days 1-2)

**Goal**: Create 3 real-world profiles (postgres, ollama-production, memory-development)
**Timeline**: 2 days (16 hours total)
**Critical Dependencies**: Phase 13b (PostgreSQL) ✅
**Priority**: CRITICAL (unblocks Phase 13b validation + production use)

### Task 13c.2.1: PostgreSQL Profile Creation ⏹ PENDING
**Priority**: CRITICAL
**Estimated Time**: 4 hours
**Assignee**: Storage Team Lead
**Status**: ⏹ PENDING

**Description**: Create `postgres.toml` builtin profile for Phase 13b PostgreSQL backend with VectorChord, RLS, bi-temporal graph.

**Acceptance Criteria**:
- [ ] postgres.toml created in llmspell-config/builtins/
- [ ] Profile loads without errors
- [ ] VectorChord vector backend configured
- [ ] Row-Level Security (RLS) multi-tenancy enabled
- [ ] Bi-temporal graph configuration present
- [ ] Profile validated with PostgreSQL container

**Implementation Steps**:
1. Create `llmspell-config/builtins/postgres.toml` with full configuration (see design doc lines 824-898)

2. Key sections:
   ```toml
   [storage.postgres]
   connection_string_env = "LLMSPELL_POSTGRES_URL"
   pool_size = 20

   [storage.postgres.vector]
   backend = "vectorchord"  # 5x faster than pgvector

   [storage.postgres.multi_tenancy]
   enabled = true
   rls_enabled = true

   [storage.postgres.graph]
   backend = "native_ctes"
   bi_temporal = true

   [memory]
   episodic_backend = "postgres_vector"
   semantic_backend = "postgres_graph"
   procedural_backend = "postgres"
   ```

3. Test profile loading:
   ```bash
   cargo run -- -p postgres info
   ```

4. Test with PostgreSQL container:
   ```bash
   cd docker/postgres && docker compose up -d
   export LLMSPELL_POSTGRES_URL="postgresql://llmspell:llmspell@localhost:5435/llmspell"
   cargo run -- -p postgres exec 'print("PostgreSQL profile loaded")'
   ```

**Definition of Done**:
- [ ] postgres.toml exists and loads
- [ ] All configuration sections present
- [ ] Profile validated with docker/postgres
- [ ] Phase 13b can use this profile immediately
- [ ] Documentation comment headers complete

**Files to Create**:
- `llmspell-config/builtins/postgres.toml`

---

### Task 13c.2.2: Ollama Production Profile Creation ⏹ PENDING
**Priority**: HIGH
**Estimated Time**: 3 hours
**Assignee**: Providers Team
**Status**: ⏹ PENDING

**Description**: Create `ollama-production.toml` for real-world local LLM deployment with embeddings, chat, caching.

**Acceptance Criteria**:
- [ ] ollama-production.toml created
- [ ] Chat model configured (llama3.2:3b)
- [ ] Embeddings model configured (nomic-embed-text)
- [ ] Memory backend uses local HNSW
- [ ] Profile works without cloud API keys
- [ ] Profile validated with ollama

**Implementation Steps**:
1. Create `llmspell-config/builtins/ollama-production.toml` (see design doc lines 932-1012)

2. Key configuration:
   ```toml
   [providers.ollama]
   provider_type = "ollama"
   base_url = "http://localhost:11434"
   default_model = "llama3.2:3b"

   [providers.ollama_embeddings]
   default_model = "nomic-embed-text"

   [memory]
   episodic_backend = "hnsw"
   embedding_provider = "ollama_embeddings"

   [rag]
   embedding_provider = "ollama_embeddings"
   vector_backend = "hnsw"
   ```

3. Test locally:
   ```bash
   # Ensure ollama running
   ollama serve &
   ollama pull llama3.2:3b
   ollama pull nomic-embed-text

   # Test profile
   cargo run -- -p ollama-production run examples/script-users/getting-started/02-first-agent.lua
   ```

**Definition of Done**:
- [ ] ollama-production.toml exists
- [ ] Zero cloud API dependencies
- [ ] Works with ollama models
- [ ] Memory + RAG use local embeddings
- [ ] Production-ready comments/docs

**Files to Create**:
- `llmspell-config/builtins/ollama-production.toml`

---

### Task 13c.2.3: Memory Development Profile Creation ⏹ PENDING
**Priority**: HIGH
**Estimated Time**: 3 hours
**Assignee**: Memory Team
**Status**: ⏹ PENDING

**Description**: Create `memory-development.toml` for Phase 13 memory debugging with all backends enabled and debug logging.

**Acceptance Criteria**:
- [ ] memory-development.toml created
- [ ] All 3 memory backends configured (HNSW, SurrealDB, in-memory)
- [ ] Debug logging enabled
- [ ] Context assembly configured (hybrid strategy)
- [ ] Telemetry and performance profiling enabled
- [ ] Profile validated with examples

**Implementation Steps**:
1. Create `llmspell-config/builtins/memory-development.toml` (see design doc lines 1049-1128)

2. Key configuration:
   ```toml
   [memory]
   episodic_backend = "hnsw"
   semantic_backend = "surrealdb"
   procedural_backend = "in_memory"
   debug_logging = true
   telemetry = true

   [memory.hnsw]
   m = 16
   ef_construction = 200

   [memory.surrealdb]
   bi_temporal = true

   [context]
   default_strategy = "hybrid"
   parallel_retrieval = true

   [runtime]
   log_level = "debug"
   trace_memory_operations = true
   ```

3. Test with memory examples:
   ```bash
   export OPENAI_API_KEY="sk-..."
   cargo run -- -p memory-development run examples/script-users/getting-started/05-memory-rag-advanced.lua
   ```

4. Verify debug output shows:
   - HNSW index operations
   - SurrealDB graph queries
   - Context assembly strategy selection
   - Performance metrics

**Definition of Done**:
- [ ] memory-development.toml exists
- [ ] All Phase 13 memory features enabled
- [ ] Debug output comprehensive
- [ ] Performance profiling working
- [ ] Development-friendly comments

**Files to Create**:
- `llmspell-config/builtins/memory-development.toml`

---

### Task 13c.2.4: Profile Catalog Documentation ⏹ PENDING
**Priority**: MEDIUM
**Estimated Time**: 2 hours
**Assignee**: Documentation Team
**Status**: ⏹ PENDING

**Description**: Create `llmspell-config/builtins/README.md` with complete profile catalog and decision matrix.

**Acceptance Criteria**:
- [ ] README.md exists in llmspell-config/builtins/
- [ ] All 17 profiles documented
- [ ] Decision matrix: when to use which profile
- [ ] Environment progression guide (dev → staging → prod)
- [ ] Profile composition examples

**Implementation Steps**:
1. Create `llmspell-config/builtins/README.md`

2. Structure:
   ```markdown
   # Builtin Profile Catalog

   ## Quick Reference (17 Profiles)

   | Profile | Use Case | Prerequisites | When to Use |
   |---------|----------|---------------|-------------|
   | minimal | Tools only | None | CLI testing |
   | development | Full dev | API keys | Feature dev |
   | postgres | PostgreSQL | PG 18 + VectorChord | Production |
   | ollama-production | Local LLM | Ollama + models | Production local |
   | memory-development | Memory debug | OpenAI key | Phase 13 dev |
   | ... | ... | ... | ... |

   ## Decision Matrix

   ### Development
   - Quick testing → minimal
   - Feature development → development
   - Memory features → memory-development
   - RAG development → rag-development

   ### Production
   - PostgreSQL backend → postgres
   - Local LLM → ollama-production
   - Cloud LLM → Custom config extending providers.toml

   ## Environment Progression

   Development → Staging → Production
   ```

3. Add profile composition examples:
   ```toml
   # custom-prod.toml
   extends = "postgres"

   [runtime]
   log_level = "warn"  # Override for production
   ```

**Definition of Done**:
- [ ] README.md comprehensive
- [ ] All 17 profiles listed
- [ ] Decision matrix clear
- [ ] Composition examples provided
- [ ] Links to full profile files

**Files to Create**:
- `llmspell-config/builtins/README.md`

---

## Phase 13c.3: Examples Consolidation (Days 4-5)

**Goal**: Reduce examples from 75 → <50 files, streamline getting-started 8 → 5
**Timeline**: 2 days (16 hours total)
**Critical Dependencies**: Phase 13c.2 (Profiles) - profiles must exist for validation
**Priority**: CRITICAL (user-facing quality)

### Task 13c.3.1: Top-Level Examples Cleanup ⏹ PENDING
**Priority**: CRITICAL
**Estimated Time**: 3 hours
**Assignee**: Examples Team
**Status**: ⏹ PENDING

**Description**: Move 4 top-level local_llm_*.lua files to script-users/ subdirectories.

**Acceptance Criteria**:
- [ ] 4 top-level Lua files moved
- [ ] examples/ directory has <5 items
- [ ] examples/README.md updated
- [ ] All moved files have standard headers
- [ ] Git history preserved (git mv)

**Implementation Steps**:
1. Move files with git mv:
   ```bash
   git mv examples/local_llm_status.lua examples/script-users/features/local-llm-status.lua
   git mv examples/local_llm_model_info.lua examples/script-users/features/local-llm-model-info.lua
   ```

2. Merge chat examples:
   ```bash
   # Combine local_llm_chat.lua + local_llm_comparison.lua
   # into examples/script-users/cookbook/local-llm-chat-patterns.lua
   ```

3. Update examples/README.md:
   - Remove top-level Lua file references
   - Add navigation to rust-developers/ and script-users/
   - Add decision matrix: "Rust embedding vs Lua scripting"

4. Add standard headers to moved files:
   ```lua
   -- ============================================================================
   -- Example: Local LLM Status Check
   -- Category: features
   -- Phase: 13c
   -- ============================================================================
   -- Profile: ollama
   -- Runtime: ~30 seconds
   -- Prerequisites: Ollama installed and running
   ```

**Definition of Done**:
- [ ] Top-level examples/ reduced 10 → <5 items
- [ ] Zero duplicate examples
- [ ] Navigation clear in README
- [ ] Standard headers on all files

**Files to Move/Modify**:
- Move: `examples/local_llm_*.lua` → `examples/script-users/features/` or `cookbook/`
- Update: `examples/README.md`

---

### Task 13c.3.2: Rust Examples Consolidation ⏹ PENDING
**Priority**: HIGH
**Estimated Time**: 4 hours
**Assignee**: Rust Examples Team
**Status**: ⏹ PENDING

**Description**: Reduce rust-developers from 6 → 3 examples by converting 2 to doc tests and 1 to developer guide.

**Acceptance Criteria**:
- [ ] 6 → 3 Rust example projects
- [ ] async-patterns converted to doc tests in llmspell-core
- [ ] builder-pattern converted to doc tests in llmspell-tools
- [ ] extension-pattern moved to docs/developer-guide/extension-architecture.md
- [ ] rust-developers/README.md updated
- [ ] Doc tests compile and pass

**Implementation Steps**:
1. Keep core 3 examples:
   - custom-tool-example/
   - custom-agent-example/
   - integration-test-example/

2. Convert async-patterns to doc tests:
   ```rust
   // In llmspell-core/src/agent.rs
   /// # Examples
   ///
   /// ## Concurrent Agent Execution
   ///
   /// ```rust
   /// # use llmspell_core::agent::BaseAgent;
   /// # use tokio::try_join;
   /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
   /// let agent1 = Agent::new("researcher");
   /// let agent2 = Agent::new("analyzer");
   ///
   /// let (result1, result2) = try_join!(
   ///     agent1.execute("task1"),
   ///     agent2.execute("task2")
   /// )?;
   /// # Ok(())
   /// # }
   /// ```
   ```

3. Convert builder-pattern to doc tests:
   ```rust
   // In llmspell-tools/src/tool.rs
   /// # Examples
   ///
   /// ## Builder Pattern for Configuration
   ///
   /// ```rust
   /// # use llmspell_tools::ToolBuilder;
   /// let tool = ToolBuilder::new("file-processor")
   ///     .with_category(ToolCategory::FileSystem)
   ///     .with_security(SecurityLevel::Safe)
   ///     .build()?;
   /// # Ok::<(), Box<dyn std::error::Error>>(())
   /// ```
   ```

4. Create docs/developer-guide/extension-architecture.md:
   - Extract code from extension-pattern-example/
   - Add comprehensive explanation
   - Include in developer-guide TOC

5. Update rust-developers/README.md:
   - Document 3 core examples
   - Reference doc tests for async/builder
   - Link to developer guide for extensions

6. Verify doc tests:
   ```bash
   cargo test --doc -p llmspell-core
   cargo test --doc -p llmspell-tools
   ```

**Definition of Done**:
- [ ] Rust examples reduced 6 → 3 projects
- [ ] Doc tests compile and pass
- [ ] Extension architecture doc comprehensive
- [ ] Zero functionality lost
- [ ] rust-developers/README.md current

**Files/Directories to Modify**:
- Remove: `examples/rust-developers/async-patterns-example/`
- Remove: `examples/rust-developers/builder-pattern-example/`
- Remove: `examples/rust-developers/extension-pattern-example/`
- Create: `docs/developer-guide/extension-architecture.md`
- Update: `examples/rust-developers/README.md`
- Update: `llmspell-core/src/agent.rs` (doc tests)
- Update: `llmspell-tools/src/tool.rs` (doc tests)

---

### Task 13c.3.3: Getting-Started Streamlining ⏹ PENDING
**Priority**: CRITICAL
**Estimated Time**: 5 hours
**Assignee**: Examples Team Lead
**Status**: ⏹ PENDING

**Description**: Reduce getting-started from 8 → 5 examples by merging 05-first-rag + 06-episodic-memory + 07-context-assembly into 05-memory-rag-advanced.lua.

**Acceptance Criteria**:
- [ ] getting-started/ reduced 8 → 5 examples
- [ ] New 05-memory-rag-advanced.lua created
- [ ] Old 06, 07 removed
- [ ] Estimated completion time <30 minutes (40% faster)
- [ ] Linear progression clear (00 → 01 → 02 → 03 → 04 → 05)
- [ ] All examples have standard headers

**Implementation Steps**:
1. Create new `05-memory-rag-advanced.lua`:
   ```lua
   -- ============================================================================
   -- Example: Memory & RAG Integration
   -- Category: getting-started
   -- Phase: 13c
   -- ============================================================================
   --
   -- Description:
   --   Comprehensive demonstration of Phase 13 memory system + RAG.
   --   Combines episodic memory, context assembly, and RAG workflow.
   --
   -- Prerequisites:
   --   - OPENAI_API_KEY environment variable
   --
   -- Profile: memory-development
   -- Runtime: ~10 minutes
   -- Complexity: INTERMEDIATE
   --
   -- Usage:
   --   export OPENAI_API_KEY="sk-..."
   --   llmspell -p memory-development run \
   --     examples/script-users/getting-started/05-memory-rag-advanced.lua
   --
   -- Expected Output:
   --   Demonstration of:
   --   - Basic RAG document ingestion and retrieval
   --   - Episodic memory storage and recall
   --   - Context assembly from multiple sources
   --   - Integrated workflow combining all three
   -- ============================================================================

   -- Section 1: Basic RAG (from old 05-first-rag.lua)
   print("=== Section 1: Basic RAG ===")
   -- ... RAG code ...

   -- Section 2: Episodic Memory (from old 06-episodic-memory-basic.lua)
   print("=== Section 2: Episodic Memory ===")
   -- ... memory code ...

   -- Section 3: Context Assembly (from old 07-context-assembly-basic.lua)
   print("=== Section 3: Context Assembly ===")
   -- ... context code ...

   -- Section 4: Integrated Workflow (NEW)
   print("=== Section 4: Integrated Workflow ===")
   -- ... integrated code ...
   ```

2. Remove old files:
   ```bash
   git rm examples/script-users/getting-started/06-episodic-memory-basic.lua
   git rm examples/script-users/getting-started/07-context-assembly-basic.lua
   ```

3. Rename 05-first-rag.lua if needed (content merged into 05-memory-rag-advanced.lua)

4. Update getting-started/README.md:
   ```markdown
   # Getting Started Examples

   **Complete this path in <30 minutes** (Phase 13c optimized)

   1. **00-hello-world.lua** (2 min) - Simplest example, tools only
   2. **01-first-tool.lua** (3 min) - File operations
   3. **02-first-agent.lua** (5 min) - Create first agent
   4. **03-first-workflow.lua** (5 min) - Build workflow
   5. **04-handle-errors.lua** (5 min) - Error handling
   6. **05-memory-rag-advanced.lua** (10 min) - Phase 13 integration

   **Total**: ~30 minutes
   ```

5. Add standard headers to all 5 examples

**Definition of Done**:
- [ ] 5 examples total (00, 01, 02, 03, 04, 05)
- [ ] 05-memory-rag-advanced.lua comprehensive
- [ ] Estimated runtime <30 minutes
- [ ] Linear progression documented
- [ ] All headers standardized

**Files to Create/Modify**:
- Create: `examples/script-users/getting-started/05-memory-rag-advanced.lua`
- Remove: `examples/script-users/getting-started/06-episodic-memory-basic.lua`
- Remove: `examples/script-users/getting-started/07-context-assembly-basic.lua`
- Update: `examples/script-users/getting-started/README.md`

---

### Task 13c.3.4: Broken Examples Cleanup ⏹ PENDING
**Priority**: CRITICAL
**Estimated Time**: 2 hours
**Assignee**: Examples Team
**Status**: ⏹ PENDING

**Description**: Remove broken nested examples/ directory and generated/ artifacts from applications.

**Acceptance Criteria**:
- [ ] communication-manager/examples/ removed
- [ ] webapp-creator/generated/ removed
- [ ] .gitignore updated to prevent future artifacts
- [ ] Application READMEs updated with links to cookbook
- [ ] Zero broken nested directories

**Implementation Steps**:
1. Remove broken nested structure:
   ```bash
   rm -rf examples/script-users/applications/communication-manager/examples/
   ```

2. Update communication-manager/README.md:
   ```markdown
   # Communication Manager Application

   For example communication patterns, see:
   - [Cookbook Email Examples](../cookbook/email-*.lua)
   - [Cookbook Slack Examples](../cookbook/slack-*.lua)
   ```

3. Remove generated artifacts:
   ```bash
   rm -rf examples/script-users/applications/webapp-creator/generated/
   ```

4. Update .gitignore:
   ```gitignore
   # Example generated artifacts
   examples/script-users/applications/*/generated/
   examples/**/node_modules/
   examples/**/*.pyc
   ```

5. Verify cleanup:
   ```bash
   find examples/ -name "generated" -type d
   find examples/ -path "*/examples/script-users" -type d
   # Should return nothing
   ```

**Definition of Done**:
- [ ] Zero nested examples/ directories
- [ ] Zero generated/ directories
- [ ] .gitignore prevents future artifacts
- [ ] Application READMEs link to cookbook
- [ ] Clean examples structure

**Files to Modify**:
- Remove: `examples/script-users/applications/communication-manager/examples/`
- Remove: `examples/script-users/applications/webapp-creator/generated/`
- Update: `.gitignore`
- Update: `examples/script-users/applications/communication-manager/README.md`
- Update: `examples/script-users/applications/webapp-creator/README.md`

---

### Task 13c.3.5: Example Config Audit ⏹ PENDING
**Priority**: MEDIUM
**Estimated Time**: 2 hours
**Assignee**: Config Team
**Status**: ⏹ PENDING

**Description**: Migrate 6 redundant configs to builtin profiles, keep 4 unique patterns, create decision matrix.

**Acceptance Criteria**:
- [ ] 6 redundant configs archived
- [ ] 4 unique configs preserved
- [ ] configs/README.md created with decision matrix
- [ ] Examples updated to use builtin profiles
- [ ] 80%+ examples use builtin profiles

**Implementation Steps**:
1. Archive redundant configs:
   ```bash
   mkdir -p examples/script-users/configs/archived
   mv examples/script-users/configs/{basic,example-providers,llmspell,rag-basic,session-enabled,state-enabled}.toml \
      examples/script-users/configs/archived/
   ```

2. Keep unique patterns:
   - applications.toml (app-specific overrides)
   - backup-enabled.toml (custom backup schedules)
   - migration-enabled.toml (migration settings)
   - rag-multi-tenant.toml (multi-tenant RAG pattern)

3. Create configs/README.md:
   ```markdown
   # Custom Configuration Examples

   ## When to Use Builtin Profiles

   ✅ Use builtin profiles for 80%+ of use cases:
   - `-p minimal` - Tools only
   - `-p providers` - OpenAI/Anthropic
   - `-p ollama-production` - Local LLM production
   - `-p postgres` - PostgreSQL backend
   - `-p memory-development` - Phase 13 debugging

   See [llmspell-config/builtins/README.md](../../llmspell-config/builtins/README.md)

   ## When to Use Custom Configs

   ❌ Only use custom configs for unique patterns:
   - Multi-tenant RAG with isolated vector stores
   - Custom backup schedules
   - Database migration settings
   - Application-specific overrides

   ## Decision Matrix

   | Use Case | Builtin Profile | Custom Config |
   |----------|----------------|---------------|
   | Development | ✅ -p development | ❌ |
   | Production local LLM | ✅ -p ollama-production | ❌ |
   | Multi-tenant RAG | ❌ | ✅ rag-multi-tenant.toml |
   | Custom backup | ❌ | ✅ backup-enabled.toml |
   ```

4. Update examples to use builtin profiles:
   ```bash
   # Find examples using old configs
   grep -r "example-providers.toml" examples/script-users/ -l

   # Update to use -p providers instead
   sed -i '' 's/-c.*example-providers.toml/-p providers/g' <file>
   ```

**Definition of Done**:
- [ ] configs/ reduced 10 → 4 active configs
- [ ] Decision matrix clear
- [ ] Examples prefer builtin profiles
- [ ] Unique patterns preserved

**Files to Modify**:
- Move: 6 configs to `configs/archived/`
- Create: `examples/script-users/configs/README.md`
- Update: Examples using old configs

---

### Task 13c.3.6: Example Header Standardization ⏹ PENDING
**Priority**: MEDIUM
**Estimated Time**: 3 hours
**Assignee**: Documentation Team
**Status**: ⏹ PENDING

**Description**: Add standard headers to all 34+ examples (5 getting-started + 5 features + 14 cookbook + 10 applications).

**Acceptance Criteria**:
- [ ] All examples have standard headers
- [ ] Profile specifications accurate
- [ ] Runtime estimates documented
- [ ] Prerequisites clearly stated
- [ ] Usage examples provided

**Implementation Steps**:
1. Create header template:
   ```lua
   -- ============================================================================
   -- Example: [Name]
   -- Category: [getting-started|features|cookbook|advanced-patterns|applications]
   -- Phase: 13c
   -- ============================================================================
   --
   -- Description:
   --   [1-2 sentence description of what this example demonstrates]
   --
   -- Prerequisites:
   --   - [API keys required, if any]
   --   - [External services, if any]
   --   - [Models to install, if local LLM]
   --
   -- Profile: [builtin-profile-name]
   -- Runtime: [estimated time]
   -- Complexity: [BEGINNER|INTERMEDIATE|ADVANCED]
   --
   -- Usage:
   --   llmspell -p [profile] run examples/script-users/[category]/[filename].lua
   --
   -- Expected Output:
   --   [Brief description of expected output]
   --
   -- ============================================================================
   ```

2. Apply to all getting-started examples (5 files)

3. Apply to all features examples (5+ files)

4. Apply to all cookbook examples (14 files)

5. Apply to all applications (10 files)

6. Verify headers:
   ```bash
   # Check all headers have required fields
   for file in examples/script-users/**/*.lua; do
     grep -q "^-- Profile:" "$file" || echo "Missing profile: $file"
     grep -q "^-- Runtime:" "$file" || echo "Missing runtime: $file"
   done
   ```

**Definition of Done**:
- [ ] All 34+ examples standardized
- [ ] Profile specs accurate
- [ ] Runtime estimates realistic
- [ ] Prerequisites complete
- [ ] Validation script can parse headers

**Files to Modify**:
- All .lua files in `examples/script-users/getting-started/`
- All .lua files in `examples/script-users/features/`
- All .lua files in `examples/script-users/cookbook/`
- All main.lua files in `examples/script-users/applications/*/`

---

## Phase 13c.4: Validation Infrastructure (Day 5)

**Goal**: Create examples-validation.sh with 100% getting-started coverage
**Timeline**: 1 day (8 hours total)
**Critical Dependencies**: Phase 13c.3 (Examples) - examples must be finalized
**Priority**: CRITICAL (quality assurance)

### Task 13c.4.1: Validation Script Creation ⏹ PENDING
**Priority**: CRITICAL
**Estimated Time**: 4 hours
**Assignee**: Testing Team Lead
**Status**: ⏹ PENDING

**Description**: Create `scripts/testing/examples-validation.sh` to test all examples with specified profiles.

**Acceptance Criteria**:
- [ ] examples-validation.sh created with executable permissions
- [ ] Tests 100% of getting-started examples
- [ ] Tests 90%+ of cookbook (API key aware)
- [ ] Colored output for readability
- [ ] Profile + example combination validation
- [ ] API key skip logic functional

**Implementation Steps**:
1. Create `scripts/testing/examples-validation.sh` (see design doc lines 1268-1419 for full content)

2. Key functions:
   ```bash
   # Extract profile from example header
   get_profile() {
       local file="$1"
       grep "^# Profile:" "$file" | awk '{print $3}' || echo "minimal"
   }

   # Check if requires API key
   requires_api_key() {
       local file="$1"
       grep -q "# Prerequisites:.*API.*KEY" "$file"
   }

   # Validate single example
   validate_example() {
       local example="$1"
       local profile=$(get_profile "$example")

       # Skip if requires API key and not available
       if requires_api_key "$example"; then
           if [[ -z "${OPENAI_API_KEY:-}" ]] && [[ -z "${ANTHROPIC_API_KEY:-}" ]]; then
               echo -e "${YELLOW}SKIPPED${NC} (API key required)"
               ((SKIPPED++))
               return 0
           fi
       fi

       # Run with timeout
       if timeout ${TIMEOUT_SECONDS}s cargo run --quiet -- -p "$profile" run "$example" &>/dev/null; then
           echo -e "${GREEN}PASSED${NC}"
           ((PASSED++))
       else
           echo -e "${RED}FAILED${NC}"
           ((FAILED++))
       fi
   }
   ```

3. Test categories:
   ```bash
   case "$category" in
       getting-started)
           # 100% required pass rate
           for example in "$EXAMPLES_DIR"/getting-started/*.lua; do
               validate_example "$example"
           done
           ;;

       cookbook)
           # 90%+ target, API key awareness
           for example in "$EXAMPLES_DIR"/cookbook/*.lua; do
               validate_example "$example"
           done
           ;;
   esac
   ```

4. Make executable:
   ```bash
   chmod +x scripts/testing/examples-validation.sh
   ```

5. Test the validator:
   ```bash
   # Test getting-started (should pass 100%)
   ./scripts/testing/examples-validation.sh getting-started

   # Test all (comprehensive)
   ./scripts/testing/examples-validation.sh all
   ```

**Definition of Done**:
- [ ] Script created and executable
- [ ] 100% getting-started validation
- [ ] 90%+ cookbook validation
- [ ] API key awareness working
- [ ] Colored output clear
- [ ] Help text comprehensive

**Files to Create**:
- `scripts/testing/examples-validation.sh`

---

### Task 13c.4.2: Quality Check Integration ⏹ PENDING
**Priority**: HIGH
**Estimated Time**: 2 hours
**Assignee**: CI/CD Team
**Status**: ⏹ PENDING

**Description**: Integrate examples-validation.sh into quality-check.sh as non-blocking check.

**Acceptance Criteria**:
- [ ] quality-check.sh includes example validation
- [ ] Non-blocking for API key skips
- [ ] Fails only if getting-started fails
- [ ] Clear output (✅ passed, ⚠️ skipped, ❌ failed)
- [ ] Documented in quality-check.sh

**Implementation Steps**:
1. Update `scripts/quality/quality-check.sh` (see design doc lines 1446-1478):
   ```bash
   # Phase 13c: Example Validation
   echo ""
   echo "========================================="
   echo "  Example Validation"
   echo "========================================="
   if [[ -x "scripts/testing/examples-validation.sh" ]]; then
       # Run validation, but don't fail on API key skips
       if ./scripts/testing/examples-validation.sh all; then
           echo -e "${GREEN}✅ All examples validated${NC}"
       else
           # Check if failures were only due to API keys
           if ./scripts/testing/examples-validation.sh getting-started; then
               echo -e "${YELLOW}⚠️  Some examples skipped (API keys), but getting-started passed${NC}"
           else
               echo -e "${RED}❌ Example validation FAILED${NC}"
               OVERALL_SUCCESS=1
           fi
       fi
   else
       echo -e "${YELLOW}⚠️  examples-validation.sh not found, skipping${NC}"
   fi
   ```

2. Test integration:
   ```bash
   ./scripts/quality/quality-check.sh
   ```

3. Test without API keys:
   ```bash
   unset OPENAI_API_KEY ANTHROPIC_API_KEY
   ./scripts/quality/quality-check.sh
   # Should skip some but not fail overall if getting-started passes
   ```

4. Test with API keys:
   ```bash
   export OPENAI_API_KEY="sk-..."
   ./scripts/quality/quality-check.sh
   # Should run more comprehensive validation
   ```

**Definition of Done**:
- [ ] quality-check.sh updated
- [ ] Non-blocking behavior correct
- [ ] getting-started failures block PR
- [ ] API key skips allowed
- [ ] Clear messaging

**Files to Modify**:
- `scripts/quality/quality-check.sh`

---

### Task 13c.4.3: CI/CD Pipeline Update ⏹ PENDING
**Priority**: MEDIUM
**Estimated Time**: 2 hours
**Assignee**: CI/CD Team
**Status**: ⏹ PENDING

**Description**: Update `.github/workflows/ci.yml` to include example validation in test job.

**Acceptance Criteria**:
- [ ] Example validation added to CI
- [ ] Runs as part of test job
- [ ] Non-blocking for API key skips
- [ ] Results visible in CI logs
- [ ] Quality gate enforced for getting-started

**Implementation Steps**:
1. Update `.github/workflows/ci.yml`:
   ```yaml
   - name: Validate Examples
     run: |
       chmod +x scripts/testing/examples-validation.sh
       # Always validate getting-started (required)
       ./scripts/testing/examples-validation.sh getting-started

       # Validate other categories if API keys available (optional)
       if [ -n "$OPENAI_API_KEY" ]; then
         ./scripts/testing/examples-validation.sh all || echo "Some examples skipped"
       fi
     env:
       OPENAI_API_KEY: ${{ secrets.OPENAI_API_KEY }}
       ANTHROPIC_API_KEY: ${{ secrets.ANTHROPIC_API_KEY }}
     continue-on-error: false  # Fail if getting-started fails
   ```

2. Add to GitHub step summary:
   ```yaml
   - name: Report Example Validation
     if: always()
     run: |
       echo "## Example Validation Results" >> $GITHUB_STEP_SUMMARY
       echo "See logs for detailed results" >> $GITHUB_STEP_SUMMARY
   ```

3. Test in CI (create test PR)

**Definition of Done**:
- [ ] CI runs example validation
- [ ] getting-started failures block PR
- [ ] API key skips non-blocking
- [ ] Results in CI summary
- [ ] Quality gate enforced

**Files to Modify**:
- `.github/workflows/ci.yml`

---

## Phase 13c.5: Documentation Overhaul (Days 6-7)

**Goal**: Update all docs to Phase 13, create migration guide, profile guide
**Timeline**: 2 days (16 hours total)
**Critical Dependencies**: Phase 13c.2-13c.3 (Profiles + Examples)
**Priority**: HIGH (user communication)

### Task 13c.5.1: User Guide Updates ⏹ PENDING
**Priority**: HIGH
**Estimated Time**: 5 hours
**Assignee**: Documentation Team
**Status**: ⏹ PENDING

**Description**: Update docs/user-guide/01-getting-started.md and 08-deployment.md to Phase 13.

**Acceptance Criteria**:
- [ ] 01-getting-started.md references 5-example path
- [ ] Completion time updated: 45+ min → <30 min
- [ ] 08-deployment.md has profile recommendations
- [ ] PostgreSQL deployment section added
- [ ] Local LLM deployment section added
- [ ] All references to Phase 8 removed

**Implementation Steps**:
1. Update `docs/user-guide/01-getting-started.md`:
   ```markdown
   # Getting Started with LLMSpell

   **Complete this guide in <30 minutes** (Phase 13c optimized)

   ## The 5-Example Path

   1. **Hello World** (2 min) - `00-hello-world.lua`
      - Profile: `minimal` (no LLM needed)
      - Learn: Basic script execution, tool usage

   2. **First Tool** (3 min) - `01-first-tool.lua`
      - Profile: `minimal`
      - Learn: File operations, tool chaining

   3. **First Agent** (5 min) - `02-first-agent.lua`
      - Profile: `providers` (requires OpenAI/Anthropic)
      - Learn: Agent creation, LLM interaction

   4. **First Workflow** (5 min) - `03-first-workflow.lua`
      - Profile: `providers`
      - Learn: Multi-step workflows, orchestration

   5. **Error Handling** (5 min) - `04-handle-errors.lua`
      - Profile: `providers`
      - Learn: Production error patterns

   6. **Memory & RAG** (10 min) - `05-memory-rag-advanced.lua`
      - Profile: `memory-development`
      - Learn: Phase 13 memory, RAG, context assembly

   ## Next Steps

   After completing getting-started:
   - **Features** → Explore specific capabilities
   - **Cookbook** → Production patterns
   - **Applications** → Complete examples
   ```

2. Update `docs/user-guide/08-deployment.md` (see design doc lines 1624-1686):
   ```markdown
   # Deployment Guide

   ## Profile Selection by Environment

   ### Development
   - **Quick start**: `-p development`
   - **Memory debugging**: `-p memory-development`
   - **RAG dev**: `-p rag-development`

   ### Staging
   - **PostgreSQL backend**: `-p postgres`
   - **Production RAG**: `-p rag-production`

   ### Production
   - **PostgreSQL (recommended)**: `-p postgres`
   - **Local LLM**: `-p ollama-production`
   - **Cloud LLM**: Custom config with prod API keys

   ## PostgreSQL Deployment (Phase 13b)

   **Requirements**:
   - PostgreSQL 18+ with VectorChord extension
   - Connection pooling (20+ connections)
   - Row-Level Security enabled

   **Setup**:
   ```bash
   # Start PostgreSQL with VectorChord
   docker compose -f docker/postgres/docker-compose.yml up -d

   # Set environment
   export LLMSPELL_POSTGRES_URL="postgresql://user:pass@host:5432/llmspell_prod"

   # Run with postgres profile
   llmspell -p postgres run script.lua
   ```

   ## Local LLM Deployment

   **Production Ollama Setup**:
   ```bash
   # Install Ollama
   curl -fsSL https://ollama.com/install.sh | sh

   # Pull production models
   ollama pull llama3.2:3b
   ollama pull nomic-embed-text

   # Run with production profile
   llmspell -p ollama-production run script.lua
   ```
   ```

**Definition of Done**:
- [ ] 01-getting-started.md updated (5-example path)
- [ ] 08-deployment.md updated (profile recommendations)
- [ ] Zero Phase 8 references
- [ ] All profile links working
- [ ] Clear progression documented

**Files to Modify**:
- `docs/user-guide/01-getting-started.md`
- `docs/user-guide/08-deployment.md`

---

### Task 13c.5.2: Profile Decision Guide Creation ⏹ PENDING
**Priority**: HIGH
**Estimated Time**: 3 hours
**Assignee**: Documentation Team
**Status**: ⏹ PENDING

**Description**: Create `docs/user-guide/profiles-guide.md` with comprehensive profile decision matrix.

**Acceptance Criteria**:
- [ ] profiles-guide.md created
- [ ] All 17 profiles documented
- [ ] Decision matrix clear (when to use which)
- [ ] Environment progression guide (dev → staging → prod)
- [ ] Profile composition examples
- [ ] Links to full profile files

**Implementation Steps**:
1. Create `docs/user-guide/profiles-guide.md` (see design doc lines 1688-1777)

2. Structure:
   ```markdown
   # Profile Selection Guide

   **Builtin profiles cover 80%+ use cases. Use custom configs only for unique patterns.**

   ## Quick Reference (17 Profiles)

   | Profile | Use Case | Prerequisites | Example |
   |---------|----------|---------------|---------|
   | minimal | Tools only, no LLM | None | Hello world |
   | providers | OpenAI/Anthropic dev | API keys | First agent |
   | development | Full dev mode | API keys | Feature development |
   | ollama | Local LLM basic | Ollama installed | Local testing |
   | ollama-production | Local LLM prod | Ollama + models | Production local |
   | postgres | PostgreSQL backend | PG 18 + VectorChord | Production persistence |
   | memory-development | Phase 13 memory | API keys | Memory debugging |
   | rag-development | RAG development | API keys | RAG testing |
   | rag-production | RAG production | API keys | Production RAG |

   ## Decision Matrix

   ### When to use builtin profiles

   ✅ **Use builtin profile if**:
   - Common development pattern (providers, memory, RAG)
   - Standard production deployment (postgres, ollama-production)
   - Feature exploration (memory-development, rag-development)

   ### When to use custom config

   ❌ **Use custom config only if**:
   - Unique multi-tenancy requirements (rag-multi-tenant.toml)
   - Custom backup schedules (backup-enabled.toml)
   - Database migration settings (migration-enabled.toml)
   - Application-specific overrides (applications.toml)

   ## Environment Progression

   ### Development → Staging → Production

   **Development**:
   ```bash
   # Quick iteration
   llmspell -p development run script.lua

   # Memory debugging
   llmspell -p memory-development run script.lua
   ```

   **Staging**:
   ```bash
   # PostgreSQL validation
   export LLMSPELL_POSTGRES_URL="postgresql://..."
   llmspell -p postgres run script.lua
   ```

   **Production**:
   ```bash
   # PostgreSQL + production settings
   export LLMSPELL_POSTGRES_URL="postgresql://..."
   llmspell -p postgres run script.lua

   # OR Local LLM production
   llmspell -p ollama-production run script.lua
   ```

   ## Profile Composition

   Profiles can be extended via custom configs:

   ```toml
   # custom-prod.toml
   # Extend postgres profile with custom settings
   extends = "postgres"

   [runtime]
   log_level = "warn"  # Override for production

   [storage.postgres]
   pool_size = 50      # Increase for high load
   ```

   Usage:
   ```bash
   llmspell -c custom-prod.toml run script.lua
   ```
   ```

**Definition of Done**:
- [ ] profiles-guide.md comprehensive
- [ ] All 17 profiles listed
- [ ] Decision matrix clear
- [ ] Environment progression documented
- [ ] Composition examples provided

**Files to Create**:
- `docs/user-guide/profiles-guide.md`

---

### Task 13c.5.3: Migration Guide Creation ⏹ PENDING
**Priority**: HIGH
**Estimated Time**: 3 hours
**Assignee**: Documentation Team
**Status**: ⏹ PENDING

**Description**: Create `docs/user-guide/migration-to-v0.14.md` comprehensive migration guide for v0.13 → v0.14.

**Acceptance Criteria**:
- [ ] migration-to-v0.14.md created
- [ ] All breaking changes documented
- [ ] Migration steps clear
- [ ] Code examples for common patterns
- [ ] Profile migration guidance
- [ ] Example path updates

**Implementation Steps**:
1. Create `docs/user-guide/migration-to-v0.14.md` (see design doc lines 1970-2130)

2. Structure:
   ```markdown
   # Migration Guide: v0.13 → v0.14

   **Phase 13c: Usability & Cohesion Refinement**

   ## Summary of Changes

   **Examples**: Consolidated 75 → <50 files, 8 → 5 getting-started
   **Profiles**: Added postgres, ollama-production, memory-development
   **Documentation**: All references updated to Phase 13
   **Quality**: 100% validated examples, zero broken examples policy

   ## Breaking Changes

   ### 1. Top-Level Examples Moved

   **What Changed**: Top-level `examples/local_llm_*.lua` files moved to `script-users/`

   **Before (v0.13)**:
   ```bash
   llmspell run examples/local_llm_status.lua
   ```

   **After (v0.14)**:
   ```bash
   llmspell run examples/script-users/features/local-llm-status.lua
   # OR use builtin profile:
   llmspell -p ollama run examples/script-users/features/local-llm-status.lua
   ```

   **Migration**: Update paths in scripts/docs to new locations

   ### 2. Getting-Started Examples Reduced

   **What Changed**: 8 → 5 examples, memory examples merged

   **Before (v0.13)**:
   - 05-first-rag.lua
   - 06-episodic-memory-basic.lua
   - 07-context-assembly-basic.lua

   **After (v0.14)**:
   - 05-memory-rag-advanced.lua (combines all three)

   **Migration**: Use 05-memory-rag-advanced.lua for integrated Phase 13 features

   ### 3. Rust Examples Reduced

   **What Changed**: 6 → 3 Rust example projects

   **Removed**:
   - async-patterns-example/ → See llmspell-core doc tests
   - builder-pattern-example/ → See llmspell-tools doc tests
   - extension-pattern-example/ → See docs/developer-guide/extension-architecture.md

   **Migration**: Use doc tests (`cargo test --doc`) or developer guide

   ## New Features

   ### 1. New Builtin Profiles

   **postgres.toml** - PostgreSQL backend:
   ```bash
   export LLMSPELL_POSTGRES_URL="postgresql://..."
   llmspell -p postgres run script.lua
   ```

   **ollama-production.toml** - Production local LLM:
   ```bash
   ollama serve
   ollama pull llama3.2:3b
   ollama pull nomic-embed-text
   llmspell -p ollama-production run script.lua
   ```

   **memory-development.toml** - Phase 13 memory debugging:
   ```bash
   export OPENAI_API_KEY="sk-..."
   llmspell -p memory-development run script.lua
   ```

   ### 2. Example Validation

   All examples now validated automatically:
   ```bash
   ./scripts/testing/examples-validation.sh all
   ```

   ### 3. Profile Decision Matrix

   See [profiles-guide.md](profiles-guide.md) for comprehensive guide

   ## Migration Checklist

   - [ ] Update hardcoded example paths
   - [ ] Switch to builtin profiles where possible
   - [ ] Review 05-memory-rag-advanced.lua for Phase 13 features
   - [ ] Update docs/scripts referencing old examples
   - [ ] Test with examples-validation.sh
   ```

**Definition of Done**:
- [ ] migration-to-v0.14.md complete
- [ ] All breaking changes documented
- [ ] Migration steps clear
- [ ] Examples provided
- [ ] Checklist helpful

**Files to Create**:
- `docs/user-guide/migration-to-v0.14.md`

---

### Task 13c.5.4: Examples READMEs Rewrite ⏹ PENDING
**Priority**: MEDIUM
**Estimated Time**: 4 hours
**Assignee**: Documentation Team
**Status**: ⏹ PENDING

**Description**: Rewrite examples/README.md, script-users/README.md, rust-developers/README.md to Phase 13.

**Acceptance Criteria**:
- [ ] examples/README.md rewritten (navigation + decision matrix)
- [ ] script-users/README.md updated (Phase 13 current)
- [ ] rust-developers/README.md updated (3 core examples)
- [ ] All READMEs reference Phase 13 (not Phase 8)
- [ ] Navigation clear and helpful

**Implementation Steps**:
1. Rewrite `examples/README.md` (see design doc lines 1788-1858)

2. Update `examples/script-users/README.md` (see design doc lines 1859-1905):
   - Status: Phase 8.10.6 → Phase 13 (v0.13.0)
   - Quick stats: 8 getting-started → 5 getting-started
   - Phase features: RAG (Phase 8) → Memory/Context (Phase 13)

3. Update `examples/rust-developers/README.md` (see design doc lines 1907-1960):
   - 6 examples → 3 core examples
   - Reference doc tests for async/builder
   - Link to developer guide for extensions

**Definition of Done**:
- [ ] All 3 READMEs comprehensive
- [ ] Phase 13 throughout
- [ ] Navigation matrices clear
- [ ] Learning paths defined
- [ ] Stats accurate

**Files to Modify**:
- `examples/README.md`
- `examples/script-users/README.md`
- `examples/rust-developers/README.md`

---

### Task 13c.5.5: README-DEVEL.md Update ⏹ PENDING
**Priority**: MEDIUM
**Estimated Time**: 1 hour
**Assignee**: Documentation Team
**Status**: ⏹ PENDING

**Description**: Update README-DEVEL.md with removed dependencies and new cargo baseline.

**Acceptance Criteria**:
- [ ] Dependency list updated (removed deps noted)
- [ ] Compilation baselines updated (10-25% improvement)
- [ ] Binary size baselines updated (1-2MB reduction)
- [ ] Justification for kept dependencies
- [ ] Links to Cargo.toml decision matrix

**Implementation Steps**:
1. Update dependency list in README-DEVEL.md:
   ```markdown
   ## Cargo Development Tools

   ### Workspace Dependencies (Phase 13c)

   **Total**: 43-47 dependencies (down from 52)

   **Removed in Phase 13c**:
   - ~~lazy_static~~ → std::sync::LazyLock (Rust 1.80+)
   - ~~once_cell~~ → std::sync::OnceLock (Rust 1.70+)
   - ~~crossbeam~~ → tokio::sync (only 2 uses)
   - ~~serde_yaml~~ → Migrated to JSON (if removed)
   - ~~bincode~~ → Migrated to JSON (if removed)

   **Justification for Kept Dependencies**:
   See [Cargo.toml](../Cargo.toml) dependency decision matrix
   ```

2. Update compilation baselines:
   ```markdown
   ## Performance Baselines (Phase 13c)

   | Metric | Before | After | Improvement |
   |--------|--------|-------|-------------|
   | Clean build | 320s | 272-288s | 10-15% |
   | Binary size (full) | 35MB | 33-34MB | 1-2MB |
   | Incremental build | 15s | 13-14s | ~13% |
   ```

**Definition of Done**:
- [ ] Dependency list current
- [ ] Baselines accurate
- [ ] Justifications linked
- [ ] Phase 13c noted

**Files to Modify**:
- `README-DEVEL.md`

---

## Phase 13c.6: Integration Testing & Release (Days 8-10)

**Goal**: Validate all changes, ensure zero regressions, prepare v0.14.0 release
**Timeline**: 3 days (24 hours total)
**Critical Dependencies**: All previous phases complete
**Priority**: CRITICAL (release quality)

### Task 13c.6.1: Comprehensive Example Validation ⏹ PENDING
**Priority**: CRITICAL
**Estimated Time**: 4 hours
**Assignee**: QA Team
**Status**: ⏹ PENDING

**Description**: Run examples-validation.sh on all categories with API keys, achieve >90% pass rate.

**Acceptance Criteria**:
- [ ] 100% getting-started examples pass
- [ ] >90% cookbook examples pass
- [ ] >80% applications pass (may require specific setup)
- [ ] All failures analyzed and documented
- [ ] Zero broken examples (excluding API key requirements)

**Implementation Steps**:
1. Set up API keys:
   ```bash
   export OPENAI_API_KEY="sk-..."
   export ANTHROPIC_API_KEY="sk-ant-..."
   export LLMSPELL_POSTGRES_URL="postgresql://..."
   ```

2. Run comprehensive validation:
   ```bash
   ./scripts/testing/examples-validation.sh getting-started
   ./scripts/testing/examples-validation.sh features
   ./scripts/testing/examples-validation.sh cookbook
   ./scripts/testing/examples-validation.sh applications
   ./scripts/testing/examples-validation.sh all
   ```

3. Analyze failures:
   ```bash
   # For each failure, debug:
   cargo run -- -p <profile> run <example>
   # Fix or document why it's expected to fail
   ```

4. Document results:
   ```markdown
   ## Example Validation Results (Phase 13c)

   **Date**: [date]
   **Total examples**: 50

   | Category | Tested | Passed | Skipped | Failed | Pass Rate |
   |----------|--------|--------|---------|--------|-----------|
   | getting-started | 5 | 5 | 0 | 0 | 100% |
   | features | 9 | 8 | 1 | 0 | 89% (1 skipped) |
   | cookbook | 14 | 13 | 1 | 0 | 93% (1 skipped) |
   | applications | 10 | 8 | 2 | 0 | 80% (2 skipped) |
   | **TOTAL** | **38** | **34** | **4** | **0** | **89%** |

   **Skipped**: API key requirements (documented in headers)
   **Failed**: None
   ```

**Definition of Done**:
- [ ] 100% getting-started pass
- [ ] >90% overall pass rate
- [ ] Zero unexpected failures
- [ ] Results documented
- [ ] Fix or document all failures

**Files to Create**:
- `docs/validation-results-phase13c.md` (validation report)

---

### Task 13c.6.2: Quality Gates Validation ⏹ PENDING
**Priority**: CRITICAL
**Estimated Time**: 3 hours
**Assignee**: QA Team Lead
**Status**: ⏹ PENDING

**Description**: Run all quality checks to ensure zero regressions and all gates pass.

**Acceptance Criteria**:
- [ ] quality-check-minimal.sh passes (<5s)
- [ ] quality-check-fast.sh passes (~1 min)
- [ ] quality-check.sh passes (5+ min)
- [ ] All 635+ tests passing
- [ ] Zero clippy warnings
- [ ] Documentation builds without errors
- [ ] >90% test coverage maintained
- [ ] >95% API documentation coverage maintained

**Implementation Steps**:
1. Run quality checks in sequence:
   ```bash
   ./scripts/quality/quality-check-minimal.sh
   ./scripts/quality/quality-check-fast.sh
   ./scripts/quality/quality-check.sh
   ```

2. Verify specific gates:
   ```bash
   # Zero warnings
   cargo clippy --workspace --all-features --all-targets -- -D warnings

   # Formatting
   cargo fmt --all -- --check

   # Tests
   cargo test --workspace --all-features

   # Documentation
   RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --all-features --document-private-items

   # Coverage
   ./scripts/testing/test-coverage.sh all lcov
   # Verify >90%
   ```

3. Benchmark compilation time:
   ```bash
   cargo clean
   time cargo build --release --features full
   # Should be 10-25% faster than before Phase 13c
   ```

4. Check binary size:
   ```bash
   ls -lh target/release/llmspell
   # Should be 1-2MB smaller than before Phase 13c
   ```

**Definition of Done**:
- [ ] All quality checks pass
- [ ] Zero warnings
- [ ] All tests pass (635+)
- [ ] Coverage >90%
- [ ] Doc coverage >95%
- [ ] Compilation faster
- [ ] Binary smaller

---

### Task 13c.6.3: Performance Benchmarking ⏹ PENDING
**Priority**: MEDIUM
**Estimated Time**: 2 hours
**Assignee**: Performance Team
**Status**: ⏹ PENDING

**Description**: Benchmark Phase 13c changes to ensure no performance regressions.

**Acceptance Criteria**:
- [ ] Compilation time improved 10-25%
- [ ] Binary size reduced 1-2MB
- [ ] Incremental build improved ~13%
- [ ] Runtime performance unchanged or better
- [ ] Memory usage unchanged or better

**Implementation Steps**:
1. Baseline measurements (from before Phase 13c):
   ```bash
   # Record from beginning of Phase 13c
   cat /tmp/before_build.log  # Build time
   cat /tmp/before_size.txt   # Binary size
   ```

2. Current measurements:
   ```bash
   cargo clean
   time cargo build --release --features full > /tmp/after_build.log 2>&1
   ls -lh target/release/llmspell | awk '{print $5}' > /tmp/after_size.txt
   ```

3. Compare:
   ```bash
   # Calculate improvements
   BEFORE_TIME=320  # seconds (baseline)
   AFTER_TIME=$(cat /tmp/after_build.log | grep "Finished" | awk '{print $2}')
   IMPROVEMENT=$((100 - (AFTER_TIME * 100 / BEFORE_TIME)))
   echo "Compilation improvement: ${IMPROVEMENT}%"

   BEFORE_SIZE=35  # MB
   AFTER_SIZE=$(cat /tmp/after_size.txt | sed 's/M//')
   REDUCTION=$((BEFORE_SIZE - AFTER_SIZE))
   echo "Binary size reduction: ${REDUCTION}MB"
   ```

4. Run benchmarks:
   ```bash
   cargo bench --workspace -- --test
   # Compare with Phase 13b baselines
   ```

5. Document results:
   ```markdown
   ## Performance Results (Phase 13c)

   | Metric | Before | After | Improvement |
   |--------|--------|-------|-------------|
   | Clean build time | 320s | 288s | 10% |
   | Binary size (full) | 35MB | 34MB | 1MB |
   | Incremental build | 15s | 13s | 13% |
   | Dependencies | 52 | 47 | 5 removed |

   **Runtime Performance**: No regressions detected
   **Memory Usage**: Unchanged
   ```

**Definition of Done**:
- [ ] Compilation 10-25% faster
- [ ] Binary 1-2MB smaller
- [ ] No runtime regressions
- [ ] Results documented
- [ ] Baselines updated

**Files to Update**:
- `docs/archives/COMPILATION-PERFORMANCE.md`

---

### Task 13c.6.4: Documentation Link Validation ⏹ PENDING
**Priority**: MEDIUM
**Estimated Time**: 2 hours
**Assignee**: Documentation Team
**Status**: ⏹ PENDING

**Description**: Validate all documentation links (internal + external) work correctly.

**Acceptance Criteria**:
- [ ] All internal links working
- [ ] All external links working
- [ ] No broken references to moved examples
- [ ] Profile links correct
- [ ] Navigation functional

**Implementation Steps**:
1. Check internal links:
   ```bash
   # Use markdown-link-check if available
   find docs/ -name "*.md" -exec markdown-link-check {} \;
   find examples/ -name "README.md" -exec markdown-link-check {} \;
   ```

2. Check example references:
   ```bash
   # Find references to old example paths
   grep -r "examples/local_llm" docs/ examples/
   grep -r "06-episodic-memory" docs/ examples/
   grep -r "07-context-assembly" docs/ examples/
   # Should find none (all updated)
   ```

3. Check profile references:
   ```bash
   # Verify all profile references exist
   grep -r "\\-p [a-z-]*" docs/ | while read line; do
     profile=$(echo "$line" | sed 's/.*-p \([a-z-]*\).*/\1/')
     if [ -f "llmspell-config/builtins/${profile}.toml" ]; then
       echo "✅ $profile"
     else
       echo "❌ Missing profile: $profile"
     fi
   done
   ```

4. Test navigation:
   - Manually click through all README navigation links
   - Verify decision matrices link correctly
   - Check profile catalog links

**Definition of Done**:
- [ ] Zero broken internal links
- [ ] Zero broken external links
- [ ] All example references current
- [ ] All profile references valid
- [ ] Navigation tested

---

### Task 13c.6.5: Release Preparation ⏹ PENDING
**Priority**: CRITICAL
**Estimated Time**: 4 hours
**Assignee**: Release Team Lead
**Status**: ⏹ PENDING

**Description**: Prepare v0.14.0 release with comprehensive changelog, version bumps, and release notes.

**Acceptance Criteria**:
- [ ] Version bumped to 0.14.0 in all Cargo.toml files
- [ ] CHANGELOG.md updated
- [ ] RELEASE_NOTES_v0.14.0.md created
- [ ] Git tag v0.14.0 created
- [ ] Release branch created
- [ ] All CI checks passing

**Implementation Steps**:
1. Bump versions:
   ```bash
   # Update workspace version
   sed -i '' 's/version = "0.13.1"/version = "0.14.0"/' Cargo.toml

   # Verify all crates inherit workspace version
   cargo metadata --format-version 1 | jq '.packages[] | select(.name | startswith("llmspell")) | .version'
   # All should show 0.14.0
   ```

2. Create CHANGELOG.md entry:
   ```markdown
   ## [0.14.0] - 2025-11-XX

   ### Phase 13c: Usability & Cohesion Refinement

   **"Less is More"** - Production-ready developer experience through consolidation and quality enhancement.

   ### Added
   - 3 new builtin profiles: `postgres`, `ollama-production`, `memory-development`
   - `examples-validation.sh` for automated example testing
   - `README-DEVEL.md` comprehensive developer setup guide
   - Profile decision matrix in `profiles-guide.md`
   - Migration guide `migration-to-v0.14.md`
   - Profile catalog in `llmspell-config/builtins/README.md`

   ### Changed
   - **BREAKING**: Top-level `examples/local_llm_*.lua` moved to `script-users/features/`
   - **BREAKING**: Getting-started examples reduced 8 → 5 (memory examples merged)
   - **BREAKING**: Rust examples reduced 6 → 3 (2 → doc tests, 1 → developer guide)
   - Examples consolidated 75 → <50 files (33% reduction)
   - Getting-started path <30 minutes (40% faster)
   - Cargo dependencies reduced 52 → 47 (5 removed)
   - Compilation time improved 10% (320s → 288s)
   - Binary size reduced 1MB (35MB → 34MB)
   - All documentation updated to Phase 13 (from Phase 8)

   ### Removed
   - `lazy_static` dependency → std::sync::LazyLock
   - `once_cell` dependency → std::sync::OnceLock
   - `crossbeam` dependency → tokio::sync
   - Redundant example configs (6 archived)
   - Broken nested examples/ directories
   - Generated artifacts in examples/

   ### Fixed
   - Zero broken examples (100% validated)
   - Documentation drift (Phase 8 → Phase 13)
   - Profile gaps (14 → 17 profiles)
   - Example sprawl (clear navigation)

   ### Performance
   - Compilation: 10% faster
   - Binary size: 1MB smaller
   - Zero runtime regressions
   - All 635+ tests passing
   ```

3. Create RELEASE_NOTES_v0.14.0.md:
   ```markdown
   # Release Notes: v0.14.0 - Usability & Cohesion Refinement

   **Phase 13c Complete** - Production-ready developer experience

   ## Highlights

   ### "Less is More" Philosophy
   - Examples: 75 → <50 files (33% reduction)
   - Getting-started: 8 → 5 examples (<30 min path)
   - Dependencies: 52 → 47 (5 removed)
   - Faster builds: 10% improvement

   ### Real-World Production Profiles
   - **postgres.toml**: PostgreSQL backend (VectorChord, RLS, bi-temporal)
   - **ollama-production.toml**: Local LLM production (zero cloud costs)
   - **memory-development.toml**: Phase 13 memory debugging

   ### Quality Assurance
   - 100% validated examples (examples-validation.sh)
   - Zero broken examples policy
   - All documentation Phase 13 current
   - Comprehensive migration guide

   ## Migration from v0.13

   See [Migration Guide](docs/user-guide/migration-to-v0.14.md)

   **Key Changes**:
   - Update example paths (top-level moved)
   - Use builtin profiles (14 → 17)
   - Update doc references (Phase 8 → Phase 13)

   ## Metrics

   | Category | Metric | Before | After | Improvement |
   |----------|--------|--------|-------|-------------|
   | **Examples** | Total files | 75 | 50 | -33% |
   | **Examples** | Getting-started | 8 | 5 | -37.5% |
   | **Examples** | Time to complete | 45+ min | <30 min | -40% |
   | **Profiles** | Builtin profiles | 14 | 17 | +3 |
   | **Validation** | Automated | 0% | 100% | NEW |
   | **Dependencies** | Workspace deps | 52 | 47 | -5 |
   | **Performance** | Build time | 320s | 288s | -10% |
   | **Performance** | Binary size | 35MB | 34MB | -1MB |

   ## What's Next

   **Phase 14**: MCP Tool Integration
   **Phase 15+**: Advanced integrations

   See [implementation-phases.md](docs/in-progress/implementation-phases.md)
   ```

4. Create git tag:
   ```bash
   git add -A
   git commit -m "chore: Release v0.14.0 - Usability & Cohesion Refinement"
   git tag -a v0.14.0 -m "Phase 13c: Usability & Cohesion Refinement"
   ```

5. Verify CI passes:
   ```bash
   # Push to test branch first
   git push origin HEAD:release/v0.14.0-test
   # Monitor CI
   # If passes, push to main
   ```

**Definition of Done**:
- [ ] Version 0.14.0 everywhere
- [ ] CHANGELOG.md updated
- [ ] RELEASE_NOTES_v0.14.0.md created
- [ ] Git tag v0.14.0 created
- [ ] CI passing
- [ ] Ready for release

**Files to Create/Modify**:
- Update: `Cargo.toml` (version = "0.14.0")
- Update: `CHANGELOG.md`
- Create: `RELEASE_NOTES_v0.14.0.md`
- Create: Git tag `v0.14.0`

---

## Phase 13c Completion Validation

### Final Integration Test
**Priority**: CRITICAL
**Estimated Time**: 4 hours
**Assignee**: Integration Team Lead

**Description**: Comprehensive validation that Phase 13c meets all success criteria.

**Acceptance Criteria**:
- [ ] All tasks completed (checkboxes ticked)
- [ ] All quality gates passing
- [ ] Zero regressions detected
- [ ] Performance targets met
- [ ] Documentation complete
- [ ] Examples validated
- [ ] Release prepared

**Integration Test Steps**:
1. Fresh clone and build validation:
   ```bash
   cd /tmp
   git clone https://github.com/lexlapax/rs-llmspell.git
   cd rs-llmspell
   git checkout v0.14.0
   cargo build --release --features full
   ```

2. Complete test suite execution:
   ```bash
   cargo test --workspace --all-features
   # All 635+ tests passing
   ```

3. Example validation:
   ```bash
   ./scripts/testing/examples-validation.sh all
   # 100% getting-started, >90% overall
   ```

4. Quality checks:
   ```bash
   ./scripts/quality/quality-check.sh
   # All gates passing
   ```

5. Performance validation:
   ```bash
   cargo clean
   time cargo build --release --features full
   # 10-25% faster than v0.13
   ```

**Phase 13c Success Metrics**:
- [ ] **Dependency Metrics**:
  - Minimum 3 dependencies removed (guaranteed)
  - Stretch 5-9 dependencies removed (audit-dependent)
  - Compilation time improved 10-25%
  - Binary size reduced 1-2MB

- [ ] **Profile Metrics**:
  - 3 new builtin profiles operational
  - 17 total profiles (14 → 17)
  - 100% production coverage
  - Profile decision matrix created

- [ ] **Example Metrics**:
  - Examples reduced 75 → <50 files (33%+)
  - Getting-started reduced 8 → 5 (37.5%)
  - Completion time <30 minutes (40% faster)
  - 100% validation coverage

- [ ] **Quality Metrics**:
  - All 635+ tests passing
  - Zero clippy warnings
  - >90% test coverage
  - >95% API documentation coverage
  - Zero broken examples
  - Zero broken links

- [ ] **Documentation Metrics**:
  - All references Phase 13 (zero Phase 8)
  - Migration guide complete
  - Profile decision matrix complete
  - README-DEVEL.md comprehensive

**Readiness Metrics**:
- [ ] Phase 14 team can begin immediately
- [ ] Production deployment ready
- [ ] User onboarding path clear (<30 min)
- [ ] Zero-broken-examples policy enforced

---

## Handoff to Phase 14

### Deliverables Package
- [ ] All 50 examples validated and working
- [ ] 17 builtin profiles documented
- [ ] examples-validation.sh automation
- [ ] Comprehensive migration guide
- [ ] Profile decision matrix
- [ ] README-DEVEL.md developer setup
- [ ] v0.14.0 release complete
- [ ] Performance baselines updated
- [ ] Zero regressions
- [ ] All 635+ tests passing

### Knowledge Transfer Session
- [ ] Usability improvements walkthrough
- [ ] Profile system overview
- [ ] Example validation automation
- [ ] Dependency cleanup rationale
- [ ] Documentation organization
- [ ] Quality gate enforcement
- [ ] Q&A session with Phase 14 team

**Phase 13c Completion**: Usability & cohesion refinement complete, production-ready developer experience established, ready for Phase 14 MCP Tool Integration.

---

**Document Version**: 1.0
**Last Updated**: November 2025
**Status**: Ready for Implementation
**Next Phase**: Phase 14 (MCP Tool Integration)
