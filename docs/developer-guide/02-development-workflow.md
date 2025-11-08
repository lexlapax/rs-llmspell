# 02: Development Workflow

**Master testing, quality gates, and contribution processes**

**Quick Navigation**: [Testing](#testing-system) | [Quality Gates](#quality-gates) | [Git Workflow](#git-workflow) | [CI/CD](#cicd-integration)

---

## Overview

This guide covers the complete development workflow for rs-llmspell, from writing tests to submitting PRs. Follow these patterns to maintain the project's >90% test coverage and zero-warnings policy.

**Key Principles**:
- Write failing test first (TDD)
- Categorize ALL tests (speed + component)
- Use llmspell-testing helpers (NO duplicates)
- Run quality checks before commits
- Update TODO.md as you work

---

## Testing System

### Test Categories (MANDATORY)

Every test MUST have two categorization attributes:

1. **Speed Category**: `unit`, `integration`, or `external`
2. **Component Category**: `tool`, `agent`, `workflow`, `rag`, `memory`, `storage`, etc.

```rust
#[tokio::test]
#[cfg_attr(test_category = "unit")]        // Speed: fast, no external dependencies
#[cfg_attr(test_category = "tool")]        // Component: what you're testing
async fn test_file_reader_basic() {
    // Test implementation
}

#[tokio::test]
#[cfg_attr(test_category = "integration")]  // Speed: moderate, multiple components
#[cfg_attr(test_category = "rag")]          // Component: RAG pipeline
async fn test_rag_pipeline_e2e() {
    // Test implementation
}

#[tokio::test]
#[cfg_attr(test_category = "external")]     // Speed: slow, real API calls
#[cfg_attr(test_category = "agent")]        // Component: agent execution
#[ignore = "external"]  // Skip in CI
async fn test_real_openai_call() {
    // Test implementation
}
```

### Test Organization

```
llmspell-<crate>/
├── src/
│   └── lib.rs                   # Unit tests in modules
├── tests/
│   ├── integration_test.rs      # Integration tests
│   └── external_test.rs         # External dependency tests
└── benches/
    └── benchmark.rs             # Performance benchmarks
```

### llmspell-testing Helpers (NO DUPLICATES)

**CRITICAL**: Always use centralized helpers from `llmspell-testing`. Never create your own test utilities.

```rust
use llmspell_testing::{
    // Tool testing
    tool_helpers::{create_test_tool_input, create_test_tool, MockTool},

    // Agent testing
    agent_helpers::{AgentTestBuilder, create_mock_provider_agent},

    // Workflow testing
    workflow_helpers::{create_test_workflow_step, create_test_sequential_workflow},

    // State testing
    state_helpers::{create_test_state_manager, create_test_memory_backend},

    // RAG testing
    rag_helpers::{create_test_rag_pipeline, MockEmbeddingFactory},

    // Storage testing
    storage_helpers::{create_test_backend, create_test_vector_storage},
};
```

### Tool Testing Pattern

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use llmspell_testing::tool_helpers::{create_test_tool_input};
    use llmspell_core::traits::BaseAgent;

    #[tokio::test]
    #[cfg_attr(test_category = "unit")]
    #[cfg_attr(test_category = "tool")]
    async fn test_my_tool_success() {
        let tool = MyTool::new(Default::default());

        let input = create_test_tool_input(vec![
            ("operation", "read"),
            ("path", "/tmp/test.txt"),
        ]);

        let result = tool.execute(input, Default::default()).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert!(output.tool_calls[0].result.is_some());
    }

    #[tokio::test]
    #[cfg_attr(test_category = "unit")]
    #[cfg_attr(test_category = "tool")]
    async fn test_my_tool_validation_error() {
        let tool = MyTool::new(Default::default());

        let input = create_test_tool_input(vec![
            ("operation", "read"),
            // Missing required "path" parameter
        ]);

        let result = tool.execute(input, Default::default()).await;
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert!(err.to_string().contains("path"));
    }
}
```

### Agent Testing Pattern

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use llmspell_testing::agent_helpers::AgentTestBuilder;

    #[tokio::test]
    #[cfg_attr(test_category = "integration")]
    #[cfg_attr(test_category = "agent")]
    async fn test_custom_agent_execution() {
        let agent = AgentTestBuilder::new()
            .name("test-agent")
            .with_mock_provider()
            .build()
            .await
            .unwrap();

        let input = AgentInput::text("Test prompt");
        let result = agent.execute(input, Default::default()).await;

        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(!output.text.is_empty());
    }
}
```

### RAG Testing Pattern

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use llmspell_testing::rag_helpers::{create_test_rag_pipeline, MockEmbeddingFactory};

    #[tokio::test]
    #[cfg_attr(test_category = "integration")]
    #[cfg_attr(test_category = "rag")]
    async fn test_rag_pipeline_e2e() {
        let pipeline = create_test_rag_pipeline().await;

        // Ingest document
        let doc = Document {
            content: "Test document content".to_string(),
            metadata: json!({ "source": "test" }),
        };
        let doc_id = pipeline.ingest(doc).await.unwrap();

        // Search
        let results = pipeline.search("test query", 5).await.unwrap();
        assert!(!results.is_empty());
    }
}
```

### Memory Testing Pattern

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use llmspell_testing::memory_helpers::create_test_memory_manager;

    #[tokio::test]
    #[cfg_attr(test_category = "integration")]
    #[cfg_attr(test_category = "memory")]
    async fn test_episodic_memory_store_retrieve() {
        let memory = create_test_memory_manager().await;

        // Store entry
        let entry = EpisodicEntry {
            role: "user".to_string(),
            content: "Test message".to_string(),
            timestamp: Utc::now(),
            metadata: json!({}),
        };
        memory.episodic().store(entry.clone()).await.unwrap();

        // Retrieve
        let results = memory.episodic().search("Test", 5).await.unwrap();
        assert!(!results.is_empty());
        assert_eq!(results[0].content, "Test message");
    }
}
```

### Storage Backend Testing Pattern

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use llmspell_testing::storage_helpers::create_test_backend;

    #[tokio::test]
    #[cfg_attr(test_category = "integration")]
    #[cfg_attr(test_category = "storage")]
    async fn test_backend_basic_operations() {
        let backend = create_test_backend().await;

        // Test SET
        backend.set("test_key", b"test_value".to_vec()).await.unwrap();

        // Test GET
        let value = backend.get("test_key").await.unwrap();
        assert_eq!(value, Some(b"test_value".to_vec()));

        // Test EXISTS
        assert!(backend.exists("test_key").await.unwrap());

        // Test DELETE
        backend.delete("test_key").await.unwrap();
        assert!(!backend.exists("test_key").await.unwrap());
    }
}
```

---

## Quality Gates

### The Three Quality Checks

```bash
# 1. MINIMAL - Run during development (seconds)
./scripts/quality/quality-check-minimal.sh
# - cargo fmt --check
# - cargo clippy --workspace --all-features --all-targets
# - cargo build --release --all-features

# 2. FAST - Run before commits (1 minute)
./scripts/quality/quality-check-fast.sh
# - All minimal checks
# - cargo test --workspace --lib (unit tests only)
# - cargo doc --workspace --all-features --no-deps

# 3. FULL - Run before PR (5+ minutes)
./scripts/quality/quality-check.sh
# - All fast checks
# - cargo test --workspace --all-features (all tests)
# - Integration tests
# - Coverage analysis
```

### Running Specific Test Categories

```bash
# Speed categories
./scripts/testing/test-by-tag.sh unit          # Fast unit tests only
./scripts/testing/test-by-tag.sh integration   # Integration tests
./scripts/testing/test-by-tag.sh external      # External dependency tests (slow)

# Component categories
./scripts/testing/test-by-tag.sh tool          # Tool tests
./scripts/testing/test-by-tag.sh agent         # Agent tests
./scripts/testing/test-by-tag.sh rag           # RAG pipeline tests
./scripts/testing/test-by-tag.sh memory        # Memory system tests
./scripts/testing/test-by-tag.sh storage       # Storage backend tests
./scripts/testing/test-by-tag.sh kernel        # Kernel server tests

# Specific crate
cargo test -p llmspell-tools --lib             # Unit tests only
cargo test -p llmspell-rag --all-features      # All tests
```

### Performance Benchmarks

```bash
# Run all benchmarks
cargo bench --workspace

# Specific crate benchmarks
cargo bench -p llmspell-tools
cargo bench -p llmspell-agents
cargo bench -p llmspell-storage
cargo bench -p llmspell-templates
cargo bench -p llmspell-memory
```

### Coverage Analysis

```bash
# Generate coverage report (requires tarpaulin)
cargo tarpaulin --workspace --all-features --out Html --output-dir target/coverage
open target/coverage/index.html
```

---

## Git Workflow

### Branch Strategy

```bash
# Main branches
main               # Production-ready code (Phase releases)
Phase-<number>     # Active development branch (e.g., Phase-13b)

# Feature branches (short-lived)
feature/<name>     # New features
fix/<issue-id>     # Bug fixes
docs/<topic>       # Documentation updates
```

### Commit Message Format

```
<type>(<scope>): <subject>

<optional body>

<optional footer>
```

**Types**:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `test`: Adding or updating tests
- `refactor`: Code refactoring
- `perf`: Performance improvements
- `chore`: Maintenance tasks

**Examples**:
```bash
git commit -m "feat(tools): Add JSONQueryTool with jq syntax support"
git commit -m "fix(rag): Fix multi-tenant vector search scope filtering"
git commit -m "docs(developer): Create 02-development-workflow.md guide"
git commit -m "test(memory): Add HNSW backend integration tests"
```

### PR Checklist

Before submitting a pull request:

- [ ] All tests pass: `./scripts/quality/quality-check.sh`
- [ ] Zero clippy warnings: `cargo clippy --workspace --all-features --all-targets`
- [ ] Code formatted: `cargo fmt --all`
- [ ] Tests categorized: All tests have speed + component attributes
- [ ] Documentation updated: If adding new features
- [ ] TODO.md updated: Mark tasks as completed
- [ ] Commit messages follow format
- [ ] Performance targets met: Run relevant benchmarks
- [ ] No breaking changes: Or documented in CHANGELOG.md

### Review Process

1. **Self-review**: Review your own PR before requesting review
2. **Automated checks**: CI must pass (GitHub Actions)
3. **Peer review**: At least one approval required
4. **Final check**: Maintainer review for architectural fit

---

## CI/CD Integration

### GitHub Actions Workflows

**Located in**: `.github/workflows/`

#### 1. CI Pipeline (`ci.yml`)

```yaml
name: CI

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        features: [minimal, common, full]
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - name: Build (${{ matrix.features }})
        run: |
          if [ "${{ matrix.features }}" = "minimal" ]; then
            cargo build --release --bin llmspell
          else
            cargo build --release --bin llmspell --features ${{ matrix.features }}
          fi
      - name: Test
        run: cargo test --workspace --features ${{ matrix.features }}
      - name: Clippy
        run: cargo clippy --workspace --features ${{ matrix.features }} --all-targets
```

#### 2. Quality Gate (`quality.yml`)

```yaml
name: Quality Gate

on: pull_request

jobs:
  quality:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Run quality checks
        run: ./scripts/quality/quality-check.sh
      - name: Upload coverage
        uses: codecov/codecov-action@v3
```

### Docker Multi-Stage Builds

```dockerfile
# Minimal production image
FROM rust:1.76 as builder-minimal
WORKDIR /app
COPY . .
RUN cargo build --release --bin llmspell

FROM debian:bookworm-slim as minimal
COPY --from=builder-minimal /app/target/release/llmspell /usr/local/bin/
ENTRYPOINT ["llmspell"]

# Common development image
FROM rust:1.76 as builder-common
WORKDIR /app
COPY . .
RUN cargo build --release --features common --bin llmspell

FROM debian:bookworm-slim as common
RUN apt-get update && apt-get install -y libssl3 ca-certificates
COPY --from=builder-common /app/target/release/llmspell /usr/local/bin/
ENTRYPOINT ["llmspell"]
```

---

## Common Development Tasks

### Bug Fix Workflow

```bash
# 1. Find the bug location
rg "error message" --type rust

# 2. Write failing test first
# In llmspell-<crate>/tests/ or src/

#[tokio::test]
#[cfg_attr(test_category = "unit")]
#[cfg_attr(test_category = "tool")]
async fn test_bug_reproduction() {
    // Should fail before fix
    let result = buggy_function();
    assert!(result.is_ok());  // Currently fails
}

# 3. Run test to verify it fails
cargo test -p llmspell-<crate> test_bug_reproduction

# 4. Fix the bug
# (Edit source files)

# 5. Verify fix
cargo test -p llmspell-<crate> test_bug_reproduction

# 6. Run quality checks
./scripts/quality/quality-check-fast.sh

# 7. Commit
git add .
git commit -m "fix(crate): Fix specific bug description"

# 8. Update TODO.md if tracked
# Mark checkbox with [x]
```

### Adding a New Feature

```bash
# 1. Create feature branch
git checkout -b feature/my-feature

# 2. Write tests first (TDD)
# Create test file in appropriate location

# 3. Implement feature
# Edit source files

# 4. Run tests frequently
cargo test -p llmspell-<crate> --lib

# 5. Document feature
# Add rustdoc comments
# Update user-facing docs if needed

# 6. Run full quality checks
./scripts/quality/quality-check.sh

# 7. Commit with descriptive message
git commit -m "feat(crate): Add new feature description"

# 8. Push and create PR
git push origin feature/my-feature
# Create PR on GitHub
```

### Refactoring Code

```bash
# 1. Ensure tests exist and pass
cargo test -p llmspell-<crate>

# 2. Refactor incrementally
# Make small changes, test frequently

# 3. Run tests after each change
cargo test -p llmspell-<crate>

# 4. Verify performance not regressed
cargo bench -p llmspell-<crate>

# 5. Run quality checks
./scripts/quality/quality-check-fast.sh

# 6. Commit
git commit -m "refactor(crate): Improve code structure"
```

---

## Performance Validation

### Targets and Measurement

| Component | Target | Status | Measure |
|-----------|--------|--------|---------|
| Tool init | <10ms | ✅ | `cargo bench -p llmspell-tools` |
| Agent creation | <50ms | ✅ | `cargo bench -p llmspell-agents` |
| Hook overhead | <2% | ✅ | Performance tests |
| Vector search | <8ms @ 100K | ✅ | `cargo bench -p llmspell-storage` |
| Multi-tenant | 3% overhead | ✅ | Integration tests |
| Template init | <2ms | ✅ | `cargo bench -p llmspell-templates` |
| Memory operations | <2ms overhead | ✅ | `cargo bench -p llmspell-memory` |

### Profiling Tools

```bash
# CPU profiling with cargo-flamegraph
cargo install flamegraph
sudo cargo flamegraph --bin llmspell -- run examples/script-users/getting-started/00-hello-world.lua

# Memory profiling with valgrind
valgrind --tool=massif --massif-out-file=massif.out ./target/release/llmspell run <script>
ms_print massif.out

# Time profiling
time ./target/release/llmspell run <script>

# Detailed timing
cargo build --release --features full
hyperfine './target/release/llmspell run examples/script-users/getting-started/00-hello-world.lua'
```

---

## Troubleshooting

### Common Issues

**Issue**: Tests hang indefinitely
```bash
# Solution: Run with timeout
cargo test --workspace -- --test-threads=1 --nocapture
```

**Issue**: Clippy warnings
```bash
# Solution: Fix warnings (NO #[allow] except in special cases)
cargo clippy --workspace --all-features --all-targets --fix
```

**Issue**: Tests fail in CI but pass locally
```bash
# Solution: Check feature flags
cargo test --workspace --all-features
```

**Issue**: Slow test execution
```bash
# Solution: Run unit tests only during development
cargo test --workspace --lib
# Or specific category
./scripts/testing/test-by-tag.sh unit
```

---

## Best Practices

### DO This

1. **Write tests first** (TDD approach)
2. **Categorize all tests** (speed + component)
3. **Use llmspell-testing helpers** (no duplicates)
4. **Run quality checks frequently** (minimal during dev, fast before commit)
5. **Update TODO.md** as you complete tasks
6. **Commit early and often** with clear messages
7. **Profile performance** for critical paths
8. **Document breaking changes** in commit messages

### DON'T Do This

1. **Skip test categorization** - Always add attributes
2. **Create test helpers** - Use llmspell-testing
3. **Commit without running quality-check-fast.sh**
4. **Use unwrap()** - Proper error handling required
5. **Ignore clippy warnings** - Fix them properly
6. **Write TODO comments** - Complete work holistically
7. **Guess at performance** - Measure with benchmarks
8. **Submit PR with failing tests** - All tests must pass

---

## Summary

**Development Cycle**:
1. ✅ Write failing test (with proper categorization)
2. ✅ Implement minimal solution
3. ✅ Run tests: `cargo test -p <crate> --lib`
4. ✅ Run quality checks: `./scripts/quality/quality-check-fast.sh`
5. ✅ Commit: `git commit -m "<type>(<scope>): <subject>"`
6. ✅ Update TODO.md: Mark tasks complete
7. ✅ Before PR: `./scripts/quality/quality-check.sh`

**Key Tools**:
- `llmspell-testing` - Centralized test helpers
- `quality-check-*.sh` - Quality gate scripts
- `test-by-tag.sh` - Category-based test execution
- `cargo bench` - Performance validation
- GitHub Actions - Automated CI/CD

**Remember**:
- >90% test coverage required
- Zero warnings policy
- Categorize ALL tests
- Use llmspell-testing helpers
- Run quality checks before commits

---

**Next**: [03-extending-components.md](03-extending-components.md) - Learn how to build tools, agents, workflows, and more.
