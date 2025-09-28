# Quality & CI Scripts

> 🎯 **Purpose**: Automated quality assurance, continuous integration, and code validation tools for maintaining high code standards across the LLMSpell project.

## 📋 Scripts Overview

| Script | Purpose | Usage | Time |
|--------|---------|-------|------|
| [`quality-check-minimal.sh`](#quality-check-minimalsh) | Fast format & lint checks | Pre-commit hook | ~5s |
| [`quality-check-fast.sh`](#quality-check-fastsh) | Adds unit tests & docs | Local development | ~1min |
| [`quality-check.sh`](#quality-checksh) | Full validation suite | Pre-push/CI | ~5min |
| [`ci-test.sh`](#ci-testsh) | CI/CD test runner | GitHub Actions | Variable |
| [`validate_applications.py`](#validate_applicationspy) | Application validation | Integration testing | ~2min |

## 🚀 Quick Start

```bash
# Quick format and lint check
./quality-check-minimal.sh

# Development validation (recommended before commits)
./quality-check-fast.sh

# Full validation (before PR)
./quality-check.sh
```

## 📝 Script Details

### quality-check-minimal.sh
**Fastest quality check (~5 seconds)**

Performs essential checks:
- Code formatting (`cargo fmt --check`)
- Linting (`cargo clippy`)
- Compilation check

```bash
# Usage
./quality-check-minimal.sh

# Exit codes
# 0 - All checks passed
# 1 - Formatting issues found
# 2 - Clippy warnings/errors
# 3 - Compilation failed
```

### quality-check-fast.sh
**Development quality check (~1 minute)**

Includes minimal checks plus:
- Unit tests (`cargo test --lib`)
- Documentation build (`cargo doc --no-deps`)
- Basic integration tests

```bash
# Usage
./quality-check-fast.sh

# With verbose output
VERBOSE=true ./quality-check-fast.sh
```

### quality-check.sh
**Comprehensive validation (~5 minutes)**

Full validation suite:
- All checks from fast version
- Integration tests
- Performance benchmarks
- Application validation
- Coverage analysis

```bash
# Usage
./quality-check.sh

# Skip benchmarks
SKIP_BENCHMARKS=true ./quality-check.sh

# Generate coverage report
WITH_COVERAGE=true ./quality-check.sh
```

### ci-test.sh
**CI/CD Pipeline Runner**

Configurable test levels for different CI stages:

```bash
# Test levels
./ci-test.sh minimal   # Format & lint only
./ci-test.sh fast      # Add unit tests
./ci-test.sh standard  # Add integration tests
./ci-test.sh full      # Everything including benchmarks

# Environment variables
TEST_LEVEL=fast ./ci-test.sh
REPORT_DIR=./test-reports ./ci-test.sh
```

### validate_applications.py
**Application & Example Validation**

Validates all example applications and use cases:

```bash
# Basic validation
python3 validate_applications.py

# Validate specific layer
python3 validate_applications.py --layer 5

# Verbose output with timing
python3 validate_applications.py --verbose --time

# Test expensive operations
RUN_EXPENSIVE_TESTS=1 python3 validate_applications.py
```

## 🔧 Configuration

### Environment Variables

```bash
# Rust toolchain
export RUST_LOG=debug           # Debug output
export RUSTFLAGS="-D warnings"  # Treat warnings as errors

# Test configuration
export SKIP_INTEGRATION=true    # Skip integration tests
export SKIP_BENCHMARKS=true     # Skip benchmarks
export WITH_COVERAGE=true       # Generate coverage

# CI configuration
export CI=true                  # Running in CI environment
export TEST_LEVEL=fast          # CI test level
export REPORT_DIR=./reports     # Test report directory
```

### Performance Targets

All scripts enforce these performance targets:

| Metric | Target | Script |
|--------|--------|--------|
| Tool initialization | <10ms | quality-check.sh |
| State operations | <5ms write, <1ms read | quality-check.sh |
| Zero warnings | `cargo clippy` clean | All scripts |
| >90% test coverage | Unit tests | quality-check.sh |
| >95% API docs | Documentation | quality-check-fast.sh |

## 🏃 Recommended Workflow

### Local Development
```bash
# Before committing
./quality-check-fast.sh

# Before pushing
./quality-check.sh
```

### CI Pipeline
```yaml
# .github/workflows/ci.yml
- name: Minimal checks (PR draft)
  run: scripts/quality/ci-test.sh minimal

- name: Fast checks (PR ready)
  run: scripts/quality/ci-test.sh fast

- name: Full validation (merge)
  run: scripts/quality/ci-test.sh full
```

### Git Hooks
```bash
# .git/hooks/pre-commit
#!/bin/bash
scripts/quality/quality-check-minimal.sh

# .git/hooks/pre-push
#!/bin/bash
scripts/quality/quality-check-fast.sh
```

## 📊 Output & Reports

Scripts generate various reports:

```
target/
├── test-reports/
│   ├── unit-tests.xml
│   ├── integration-tests.xml
│   └── coverage.html
├── doc/
│   └── llmspell/
└── criterion/
    └── report/
```

## 🐛 Troubleshooting

### Common Issues

**Formatting failures:**
```bash
# Auto-fix formatting
cargo fmt --all
```

**Clippy warnings:**
```bash
# See detailed warnings
cargo clippy --workspace --all-targets -- -W clippy::all
```

**Test failures:**
```bash
# Run specific test
cargo test test_name -- --nocapture

# Debug test
RUST_LOG=debug cargo test test_name
```

## 🔗 Related Documentation

- [Testing Guide](../../docs/development/testing.md)
- [CI/CD Pipeline](../../.github/workflows/README.md)
- [Contributing Guidelines](../../CONTRIBUTING.md)