# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

rs-llmspell: **Scriptable LLM interactions** via Lua, JavaScript - Cast scripting spells to animate LLM golems

## Primary Documentation

- **Architecture**: `/docs/technical/rs-llmspell-final-architecture.md` - Complete system architecture
- **Implementation Phases**: `/docs/in-progress/implementation-phases.md` - 16-phase roadmap
- **Current Phase**: See `/docs/in-progress/PHASE*-TODO.md` for active phase tracking
- **User Guide**: `/docs/user-guide/README.md` - For end users
- **Developer Guide**: `/docs/developer-guide/README.md` - For contributors

## Development Norms

### Code Quality Standards

- **Zero Warnings Policy**: All code must compile without warnings (`cargo clippy -- -D warnings`)
- **Test Coverage**: >90% coverage required (enforced in CI)
- **Documentation**: >95% API documentation coverage required
- **Formatting**: Run `cargo fmt --all` before every commit
- **Performance**: Maintain established benchmarks (e.g., <10ms tool initialization)

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

### Testing Requirements

1. **Test Categories** (use `#[cfg_attr(test_category = "...")]`):
   - `unit`: Fast, isolated component tests
   - `integration`: Cross-component tests
   - `tool`: Individual tool functionality tests
   - `agent`: Agent-specific tests
   - `workflow`: Workflow pattern tests
   - `external`: Tests requiring network/external resources
   - `security`: Security-specific tests

2. **Test Coverage**:
   - Every public API must have tests
   - Security edge cases must be tested
   - Performance benchmarks for critical paths
   - Script integration tests for all exposed APIs

### Development Workflow

1. **Before Starting Work**:
   - Read relevant phase design doc in `/docs/in-progress/`
   - Check TODO.md for current task status
   - Run full quality check to ensure clean baseline

2. **During Development**:
   - Follow TDD: Write tests first
   - Run minimal quality check frequently
   - Update documentation as you code
   - No lazy implementations or TODOs without explicit approval

3. **Before Committing**:
   - Run `./scripts/quality-check-fast.sh`
   - Ensure all tests pass
   - Update CHANGELOG.md if adding features
   - Keep commits focused and atomic

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
- Megathink and widen scope of research in code 

### Performance Targets

- Tool initialization: <10ms
- Agent creation: <50ms including provider setup
- Tool invocation overhead: <10ms
- Workflow step overhead: <20ms
- Script bridge overhead: <5ms
- Memory usage: Linear with workload

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
6. **Don't** use unwrap() in production code
7. **Don't** hardcode configuration values
8. **Don't** expose internal implementation details

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