# CLAUDE.md

rs-llmspell: **Scriptable LLM interactions** via Lua, JavaScript - Cast scripting spells to animate LLM golems

## Primary Documentation

- **Architecture**: `/docs/technical/master-architecture-vision.md` - Complete system architecture vision (not actual architecture)
- **Implementation Phases**: `/docs/in-progress/implementation-phases.md` - 16-phase roadmap
- **Current Status**: Phase 6 COMPLETE (39/39 tasks) ✅ - Session and Artifact Management ready for production
- **User Guide**: `/docs/user-guide/README.md` - For end users
- **Developer Guide**: `/docs/developer-guide/README.md` - For contributors
- **Current TODOs**: always read `/TODO.md` for current work. we follow check boxes with tasks and subtasks in a numbered hierarchical way.

## Planning, architecture, design and implementation Norms
- **Always Megathink**: look at and analyze existing code instead of guessing.
- **Future based**: think about how things/components fit together for future work in implementation-phases.md
- **No Backward Compatibility**: Prioritize correctness and less code instead of backward compatibility until 1.0 release
- **Modularity and Traits**: Use the power of Traits in rust for modularity, separation of concerns, clean code and building blocks instead of direct dependencies
- **Update TODO.md**: update as you accomplish sub-tasks not after everything in the task is complete. new insights, architecture decisions should be updated in the relevant sections as we make them.

## Development Norms

### Code Quality Standards
- **Zero Warnings Policy**: All code must compile without warnings (`cargo clippy --workspace --all-target --all-features`)
- **Test Coverage**: >90% coverage required (enforced in CI)
- **Documentation**: >95% API documentation coverage required
- **Formatting**: Run `cargo fmt --all` before every commit
- **Performance**: Maintain established benchmarks (e.g., <10ms tool initialization, <1% hook overhead)

### Quality Check Commands

```bash
# MANDATORY before commits
./scripts/quality-check-minimal.sh     # Quick check (seconds) - formatting, clippy, compilation
./scripts/quality-check-fast.sh        # Fast check (~1 min) - adds unit tests & docs
./scripts/quality-check.sh             # Full check (5+ min) - all tests & coverage

# Test specific components
./scripts/test-by-tag.sh unit         # Run only unit tests
./scripts/test-by-tag.sh tool         # Run tool tests
./scripts/test-by-tag.sh external     # Run external/network tests
./scripts/list-tests-by-tag.sh all    # List test categories
SKIP_SLOW_TESTS=true ./scripts/quality-check.sh  # Skip slow tests
```

### Implementation Principles

1. **State-First Design**: Components communicate through shared state, not direct messaging
2. **DRY Principle**: Use `llmspell-utils` crate for all shared functionality
3. **Security First**: All inputs validated, all paths sanitized, resource limits enforced
4. **Composition Over Inheritance**: Prefer trait composition patterns
5. **Bridge-First Architecture**: Leverage existing Rust crates rather than reimplementing
6. **Script API Consistency**: Same API surface across Lua, JavaScript, and Python

### Coding Standards

1. **Parameter Naming**:
   - Primary data: `input` (not: text, content, data, expression, query, etc.)
   - File paths: `path` for single files, `source_path`/`target_path` for operations
   - Operations: Always require explicit `operation` parameter for multi-function tools

2. **Response Format**:
   ```json
   {
     "operation": "operation_name",
     "success": true,
     "result": {...},
     "error": null,
     "metadata": {...}
   }
   ```

3. **Error Handling**:
   - Use `Result<T, E>` for all fallible operations
   - Create specific error types, not generic strings
   - Include context in errors (use `anyhow` with `.context()`)
   - Sanitize error messages (no sensitive paths or data)

4. **File Structure**:
   - Each tool in its own module under `llmspell-tools/src/`
   - Shared utilities in `llmspell-utils/src/`
   - Script bindings in `llmspell-bridge/src/lua/globals/`
   - Examples in `examples/` with working code

### Testing Guidelines (PHASE 7 FEATURE-BASED ARCHITECTURE)

**⚠️ IMPORTANT**: Use feature-based testing system established in Task 7.1.6. The `cfg_attr` attribute system was deprecated due to syntax issues and replaced with Cargo feature flags.

#### Test Organization

Tests are organized through the centralized `llmspell-testing` crate using Cargo feature flags:

**Test Categories (via features):**
- `unit-tests` - Fast, isolated component tests in `src/` files
- `integration-tests` - Cross-component tests in `tests/` directories  
- `external-tests` - Tests requiring external services (APIs, network)
- `benchmark-tests` - Performance benchmarks

**Component Categories:**
- `tool-tests`, `agent-tests`, `workflow-tests` - Component-specific tests
- `bridge-tests`, `hook-tests`, `event-tests` - System-specific tests  
- `session-tests`, `state-tests`, `core-tests` - Infrastructure tests
- `security-tests`, `performance-tests` - Specialty categories

**Test Suites:**
- `fast-tests` = `unit-tests` + `integration-tests` 
- `comprehensive-tests` = All non-external tests
- `all-tests` = Everything including external tests

#### Test Execution

**Primary Method - Script-based:**
```bash
# Run specific test categories
./scripts/test-by-tag.sh unit         # Unit tests only
./scripts/test-by-tag.sh integration  # Integration tests only  
./scripts/test-by-tag.sh tool         # Tool tests via llmspell-tools
./scripts/test-by-tag.sh external     # External service tests
./scripts/test-by-tag.sh all          # All tests including ignored

# Quality gates (use in development)
./scripts/quality-check-minimal.sh    # Format + clippy + compile (seconds)
./scripts/quality-check-fast.sh       # Above + unit tests (1 min)
./scripts/quality-check.sh            # Full validation (5+ min)
```

**Alternative - Direct feature usage:**
```bash
# Feature-based execution (if scripts unavailable)
cargo test -p llmspell-testing --features unit-tests
cargo test -p llmspell-testing --features integration-tests
cargo test -p llmspell-testing --features all-tests
```

#### Test Structure & Helpers

**MANDATORY - Use centralized helpers:**
```rust
// Import from llmspell-testing (NOT individual crates)
use llmspell_testing::{
    // Component helpers
    tool_helpers::{create_test_tool, MockTool},
    agent_helpers::{AgentTestBuilder, create_mock_provider_agent},
    workflow_helpers::{create_test_workflow_step},
    
    // Infrastructure helpers  
    mocks::{MockBaseAgent, MockProvider},
    fixtures::{TempFixture, create_test_state},
    generators::{component_id_strategy, random_workflow_config},
};
```

**Test Placement Rules:**
- **Unit tests**: Place in `src/` files with `#[cfg(test)]` modules
- **Integration tests**: Place in `tests/` directories as separate files
- **External tests**: Add `#[ignore = "external"]` attribute for credential-required tests

**NO cfg_attr attributes** - These were removed in Phase 7 refactoring

#### Development Workflow

1. **Write Tests First:**
   - Create failing test in appropriate location (`src/` or `tests/`)
   - Use `llmspell-testing` helpers, never create custom mocks
   - Run: `./scripts/test-by-tag.sh unit` (for unit tests)

2. **Implementation:**
   - Write minimal code to pass tests
   - Run: `./scripts/quality-check-fast.sh` (format + clippy + unit tests)

3. **Before Commit (MANDATORY):**
   ```bash
   ./scripts/quality-check-fast.sh     # Essential checks (1 min)
   # OR for thorough validation:
   ./scripts/quality-check.sh          # Full validation (5+ min)
   ```

#### External Test Requirements

For tests requiring credentials/network access:
- Add `#[ignore = "external"]` attribute
- Document required environment variables
- Provide mock alternatives for CI
- Run via `./scripts/test-by-tag.sh external`

#### Performance Targets

- Unit tests: <5 seconds total per crate
- Integration tests: <30 seconds total per crate  
- Quality check scripts: <1 min (fast), <5 min (full)
- Test initialization: <10ms per test helper

#### Test Coverage Requirements

- Every public API must have tests
- Security edge cases must be tested
- Performance benchmarks for critical paths
- Script integration tests for all exposed APIs
- >90% coverage required (enforced in CI)
- >95% API documentation coverage required

### Development Workflow

1. **Before Starting Work**:
   - Read relevant phase design doc in `/docs/in-progress/`
   - Check TODO.md for current task status
   - Run full quality check to ensure clean baseline: `./scripts/quality-check.sh`

2. **During Development (TDD Process)**:
   - **Write failing test** in appropriate location (`src/` or `tests/`)
   - **Run fast test suite**: `./scripts/quality-check-fast.sh` 
   - **Write minimal implementation** to make test pass
   - **Run fast tests again** to confirm: `./scripts/test-by-tag.sh unit`
   - **Refactor** while keeping tests green
   - Update documentation as you code
   - No lazy implementations or TODOs without explicit approval

3. **Before Committing (MANDATORY)**:
   - Run **MANDATORY** quality checks:
     ```bash
     # MANDATORY before commits  
     ./scripts/quality-check-minimal.sh     # Quick check (seconds) - formatting, clippy, compilation
     ./scripts/quality-check-fast.sh        # Fast check (~1 min) - adds unit tests & docs
     ./scripts/quality-check.sh             # Full check (5+ min) - all tests & coverage
     ```
   - Ensure all tests pass
   - Update CHANGELOG.md if adding features
   - Keep commits focused and atomic

4. **Test Execution Commands** (use during development):
   ```bash
   # Test specific components
   ./scripts/test-by-tag.sh unit         # Run only unit tests
   ./scripts/test-by-tag.sh tool         # Run tool tests
   ./scripts/test-by-tag.sh external     # Run external/network tests
   SKIP_SLOW_TESTS=true ./scripts/quality-check.sh  # Skip slow tests

   # Feature-based test execution (alternative)
   cargo test -p llmspell-testing --features unit-tests       # Fast unit tests only
   cargo test -p llmspell-testing --features integration-tests # Integration tests only  
   cargo test -p llmspell-testing --features external-tests   # External tests only
   cargo test -p llmspell-testing --features all-tests        # All tests (slow)
   ```

4. **Definition of Done**:
   - Feature fully implemented (no stubs)
   - All tests passing
   - Documentation complete
   - Quality checks passing
   - No regression in performance
   - Security considerations addressed

### Architecture Patterns

1. **Trait Hierarchy**:
   ```
   BaseAgent ← Agent ← SpecializedAgent
       ↑
     Tool ← ToolWrappedAgent
       ↑
   Workflow ← Sequential, Parallel, Conditional, Loop
   ```

2. **Crate Organization**:
   - `llmspell-core`: Core traits and types
   - `llmspell-tools`: Tool implementations
   - `llmspell-agents`: Agent infrastructure
   - `llmspell-workflows`: Workflow patterns
   - `llmspell-bridge`: Script language integration
   - `llmspell-utils`: Shared utilities
   - `llmspell-state-persistence`: State management with persistence
   - `llmspell-hooks`: Hook system with replay capabilities
   - `llmspell-events`: Event system with correlation
   - `llmspell-sessions`: Session management with artifacts and replay

3. **Script Integration**:
   - All functionality exposed through global objects
   - Synchronous wrappers for async operations
   - Consistent error propagation to scripts
   - Zero-configuration access (globals injected automatically)

### Breaking Changes Policy

- Pre-1.0: Clean breaks allowed with documentation, no backward compatibility
- Post-1.0: Deprecation cycle required
- Always document in CHANGELOG.md
- Provide migration examples
- Update all examples and tests

### Performance Targets

- Tool initialization: <10ms
- Agent creation: <50ms including provider setup
- Tool invocation overhead: <10ms
- Workflow step overhead: <20ms
- Script bridge overhead: <5ms
- Hook execution overhead: <1% (enforced by CircuitBreaker)
- Event throughput: >90K events/sec
- Memory usage: Linear with workload
- State operations: <5ms write, <1ms read ✅ (achieved)
- State migration: 2.07μs per item ✅ (achieved)
- Backup/recovery: Atomic with SHA256 validation ✅
- Session operations: <50ms create, save, load ✅ (24.5μs, 15.3μs, 3.4μs achieved)
- Artifact storage: <5ms for text/JSON ✅ (<1ms achieved)

### Security Requirements

1. **Input Validation**:
   - Length limits on all string inputs
   - Path traversal prevention
   - Command injection prevention
   - URL validation for web operations

2. **Resource Limits**:
   - Memory limits enforced
   - CPU time limits
   - Operation count limits
   - Concurrent operation limits

3. **Sandboxing**:
   - File system access restrictions
   - Network access controls
   - Process spawning limits
   - Credential protection

### Common Pitfalls to Avoid

1. **Don't** create new files unless absolutely necessary
2. **Don't** implement features not in the current phase
3. **Don't** skip tests to make deadlines
4. **Don't** ignore security implications
5. **Don't** use unwrap() in production code
6. **Don't** hardcode configuration values
7. **Don't** expose internal implementation details

### Useful Commands

```bash
# Find TODO items
rg "TODO|FIXME|HACK" --type rust

# Check for unwrap usage
rg "\.unwrap\(\)" --type rust

# Run specific test
cargo test -p llmspell-tools test_name_here

# Benchmark performance
cargo bench -p llmspell-tools

# Generate documentation
cargo doc --workspace --no-deps --open

# Check dependencies
cargo tree -d  # Find duplicate dependencies
cargo audit    # Security audit
```

Remember: When in doubt, refer to `/docs/technical/master-architecture-vision.md` for architectural decisions and `/docs/in-progress/implementation-phases.md` for the roadmap.