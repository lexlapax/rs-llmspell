# CLAUDE.md

rs-llmspell: **Scriptable LLM interactions** via Lua, JavaScript - Cast scripting spells to animate LLM golems

## Primary Documentation

- **Architecture**: `/docs/technical/rs-llmspell-final-architecture.md` - Complete system architecture
- **Implementation Phases**: `/docs/in-progress/implementation-phases.md` - 16-phase roadmap
- **Current Status**: Phase 6 COMPLETE (39/39 tasks) ✅ - Session and Artifact Management ready for production
- **User Guide**: `/docs/user-guide/README.md` - For end users
- **Developer Guide**: `/docs/developer-guide/README.md` - For contributors

## Development Norms

### Code Quality Standards

- **Zero Warnings Policy**: All code must compile without warnings (`cargo clippy -- -D warnings`)
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

### Testing Guidelines (CRITICAL - MAINTAIN 7.1.6 ARCHITECTURE)

**⚠️ IMPORTANT**: Always follow the test categorization system established in Task 7.1.6. Improper categorization breaks CI and causes flaky tests.

#### Test Placement Rules

**Unit Tests**: Place in `src/` files with `#[cfg(test)]` module
- Fast, isolated component tests 
- No external dependencies, no network calls, no file I/O
- Should complete in <5 seconds total per crate
- Test individual functions, methods, and small components

**Integration Tests**: Place in `tests/` directories as separate files
- Cross-component, cross-crate tests
- External dependencies MUST be mocked (use llmspell-testing helpers)
- Should complete in <30 seconds total per crate
- Test interaction between multiple components

**External Tests**: Place in `tests/` directories with `#[ignore = "external"]`
- Real API calls, network requests, LLM providers
- Require credentials/environment setup (env vars, API keys)
- Can be slow, skipped in CI by default
- Test real integrations (OpenAI, Anthropic, web requests)

#### Test Categorization (MANDATORY)

**Basic Categories** (always required - choose one):
```rust
#[test]
#[cfg_attr(test_category = "unit")]        // Fast, isolated
#[cfg_attr(test_category = "integration")] // Cross-component  
#[cfg_attr(test_category = "external")]    // External dependencies
```

**Component Categories** (add one that matches your functionality):
```rust
#[cfg_attr(test_category = "tool")]        // Tool-related functionality
#[cfg_attr(test_category = "agent")]       // Agent-related functionality  
#[cfg_attr(test_category = "workflow")]    // Workflow-related functionality
#[cfg_attr(test_category = "bridge")]      // Script bridge functionality
#[cfg_attr(test_category = "hook")]        // Hook system functionality
#[cfg_attr(test_category = "event")]       // Event system functionality
#[cfg_attr(test_category = "session")]     // Session management functionality
#[cfg_attr(test_category = "state")]       // State management functionality
#[cfg_attr(test_category = "core")]        // Core trait/type functionality
#[cfg_attr(test_category = "util")]        // Utility functionality
```

**Specialized Categories** (when applicable):
```rust
#[cfg_attr(test_category = "security")]    // Security-related tests
#[cfg_attr(test_category = "performance")] // Performance/benchmark tests
```

**Example Test Structure**:
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    #[cfg_attr(test_category = "unit")]
    #[cfg_attr(test_category = "tool")]
    fn test_file_reader_basic_functionality() {
        // Unit test for file reader tool
    }
    
    #[test]
    #[cfg_attr(test_category = "integration")]
    #[cfg_attr(test_category = "tool")]
    fn test_file_reader_with_agent() {
        // Integration test: file reader + agent
    }
    
    #[test]
    #[cfg_attr(test_category = "external")]
    #[cfg_attr(test_category = "tool")]
    #[ignore = "external"]
    fn test_file_reader_with_real_filesystem() {
        // External test: real file operations
    }
}
```

#### Test Helper Usage (MANDATORY - NO DUPLICATES)

**ALWAYS use llmspell-testing helpers instead of creating your own:**

```rust
use llmspell_testing::{
    // Tool testing
    tool_helpers::{create_test_tool, create_test_tool_input, MockTool},
    
    // Agent testing  
    agent_helpers::{AgentTestBuilder, create_mock_provider_agent, TestProviderAgent},
    
    // Event testing
    event_helpers::{create_test_event, create_test_event_bus},
    
    // Workflow testing
    workflow_helpers::{create_test_workflow_step, create_test_sequential_workflow},
    
    // Common test infrastructure
    mocks::{MockBaseAgent, MockProvider},
};

// Example: Tool testing
#[test]
#[cfg_attr(test_category = "unit")]
#[cfg_attr(test_category = "tool")]
fn test_my_tool() {
    let tool = create_test_tool("my-tool", "Test tool", vec![("param1", "string")]);
    let input = create_test_tool_input(vec![("param1", "value")]);
    // ... test logic
}
```

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
   - **Write failing test** with proper categorization (see Test Guidelines above)
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
   - Ensure all tests pass with proper categorization
   - Update CHANGELOG.md if adding features
   - Keep commits focused and atomic

4. **Test Execution Commands** (use during development):
   ```bash
   # Test specific components
   ./scripts/test-by-tag.sh unit         # Run only unit tests
   ./scripts/test-by-tag.sh tool         # Run tool tests
   ./scripts/test-by-tag.sh external     # Run external/network tests
   SKIP_SLOW_TESTS=true ./scripts/quality-check.sh  # Skip slow tests

   # Feature-based test execution
   cargo test --features unit-tests              # Fast unit tests only
   cargo test --features integration-tests       # Integration tests only  
   cargo test --features external-tests --ignored # External tests only
   cargo test --features all-tests               # All tests (slow)
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

Remember: When in doubt, refer to `/docs/technical/rs-llmspell-final-architecture.md` for architectural decisions and `/docs/in-progress/implementation-phases.md` for the roadmap.