# Testing Scripts

> 🧪 **Purpose**: Comprehensive test execution, coverage analysis, and test management tools for the LLMSpell project's extensive test suite.

## 📋 Scripts Overview

| Script | Purpose | Usage | Time |
|--------|---------|-------|------|
| [`test-by-tag.sh`](#test-by-tagsh) | Run tests by category/tag | Focused testing | Variable |
| [`test-multiple-tags.sh`](#test-multiple-tagssh) | Run multiple test categories | Batch testing | Variable |
| [`list-tests-by-tag.sh`](#list-tests-by-tagsh) | List available test tags | Discovery | Instant |
| [`test-coverage.sh`](#test-coveragesh) | Generate coverage reports | Quality metrics | ~3min |
| [`run-llmspell-tests.sh`](#run-llmspell-testssh) | Comprehensive test runner | Full validation | ~5min |

## 🚀 Quick Start

```bash
# Run unit tests only
./test-by-tag.sh unit

# Run multiple test categories
./test-multiple-tags.sh unit integration

# Generate coverage report
./test-coverage.sh all html

# List available test tags
./list-tests-by-tag.sh
```

## 📝 Script Details

### test-by-tag.sh
**Category-Based Test Runner**

Run specific test categories using the llmspell-testing framework:

```bash
# Available tags
./test-by-tag.sh unit        # Unit tests from llmspell-testing
./test-by-tag.sh integration # Integration tests
./test-by-tag.sh agent       # Agent-specific tests
./test-by-tag.sh scenarios   # End-to-end scenarios
./test-by-tag.sh lua         # Lua scripting tests
./test-by-tag.sh tool        # Tool package tests
./test-by-tag.sh bridge      # Bridge package tests
./test-by-tag.sh workflow    # Workflow tests
./test-by-tag.sh fast        # Fast unit tests only
./test-by-tag.sh slow        # Slow/ignored tests
./test-by-tag.sh external    # Tests requiring external services
./test-by-tag.sh all         # All tests including ignored

# With additional cargo arguments
./test-by-tag.sh unit -- --nocapture
./test-by-tag.sh integration -- --test-threads=1
```

### test-multiple-tags.sh
**Multi-Category Test Runner**

Run multiple test categories in sequence:

```bash
# Run unit and integration tests
./test-multiple-tags.sh unit integration

# Run all component tests
./test-multiple-tags.sh tool agent workflow bridge

# With timing information
TIME_TESTS=true ./test-multiple-tags.sh unit integration agent
```

### list-tests-by-tag.sh
**Test Discovery Tool**

List and explore available test categories:

```bash
# List all tags
./list-tests-by-tag.sh

# List with descriptions
./list-tests-by-tag.sh --detailed

# Show test counts per category
./list-tests-by-tag.sh --count

# Filter by pattern
./list-tests-by-tag.sh --filter "agent"
```

### test-coverage.sh
**Coverage Report Generator**

Generate detailed test coverage reports:

```bash
# Coverage types
./test-coverage.sh unit        # Unit test coverage only
./test-coverage.sh integration # Integration test coverage
./test-coverage.sh all         # Complete coverage (default)

# Output formats
./test-coverage.sh all html    # HTML report (default)
./test-coverage.sh all lcov    # LCOV for CI tools
./test-coverage.sh all json    # JSON format

# Advanced usage
COVERAGE_DIR=./coverage ./test-coverage.sh all html
MIN_COVERAGE=90 ./test-coverage.sh all    # Fail if <90%
```

Coverage reports location:
```
target/llvm-cov/
├── html/           # HTML reports
├── lcov.info       # LCOV data
└── coverage.json   # JSON report
```

### run-llmspell-tests.sh
**Comprehensive Test Suite Runner**

Main test orchestrator for all test types:

```bash
# Test suites
./run-llmspell-tests.sh all           # All test categories (default)
./run-llmspell-tests.sh fast          # Fast suite (unit + integration)
./run-llmspell-tests.sh comprehensive # Excludes external/benchmark

# Primary test types
./run-llmspell-tests.sh unit          # Unit tests only
./run-llmspell-tests.sh integration   # Integration tests only
./run-llmspell-tests.sh external      # External dependency tests
./run-llmspell-tests.sh benchmark     # Benchmark tests

# Component categories
./run-llmspell-tests.sh tool          # Tool tests
./run-llmspell-tests.sh agent         # Agent tests
./run-llmspell-tests.sh workflow      # Workflow tests
./run-llmspell-tests.sh bridge        # Bridge tests
./run-llmspell-tests.sh hook          # Hook tests
./run-llmspell-tests.sh event         # Event tests

# Options
--verbose              # Detailed output
--fail-fast           # Stop on first failure
--parallel            # Run tests in parallel
--report              # Generate test report
```

## 🎯 Test Categories

### Primary Types
- **unit**: Fast, isolated unit tests
- **integration**: Component integration tests
- **external**: Tests requiring external services
- **benchmark**: Performance benchmarks

### Component Categories
- **tool**: Tool implementation tests
- **agent**: Agent orchestration tests
- **workflow**: Workflow pattern tests
- **bridge**: Language bridge tests
- **hook**: Hook system tests
- **event**: Event system tests

### Execution Categories
- **fast**: Quick tests (<100ms each)
- **slow**: Long-running tests
- **scenarios**: End-to-end scenarios
- **lua**: Lua-specific tests

## 🔧 Configuration

### Environment Variables

```bash
# Test execution
export RUST_TEST_THREADS=1      # Sequential execution
export RUST_TEST_NOCAPTURE=1    # Show println! output
export RUST_LOG=debug            # Debug logging

# Coverage configuration
export COVERAGE_DIR=./coverage  # Coverage output directory
export MIN_COVERAGE=90           # Minimum coverage percentage
export EXCLUDE_PATTERNS="*/tests/*,*/benches/*"

# Test selection
export SKIP_SLOW=true           # Skip slow tests
export RUN_EXPENSIVE_TESTS=1    # Run expensive tests
export TEST_EXTERNAL=true       # Include external tests
```

### Test Organization

Tests are organized in the `llmspell-testing` crate:

```
llmspell-testing/
├── src/
│   ├── unit/         # Unit test helpers
│   ├── integration/  # Integration test utilities
│   ├── scenarios/    # End-to-end scenarios
│   └── fixtures/     # Test fixtures
├── tests/
│   ├── unit/         # Unit tests
│   ├── integration/  # Integration tests
│   └── scenarios/    # Scenario tests
└── Cargo.toml
```

## 🏃 Testing Workflows

### Local Development
```bash
# Quick validation
./test-by-tag.sh unit

# Before commit
./test-multiple-tags.sh unit integration

# Full validation
./run-llmspell-tests.sh comprehensive
```

### Coverage Analysis
```bash
# Check coverage
./test-coverage.sh all html
open target/llvm-cov/html/index.html

# CI coverage check
MIN_COVERAGE=85 ./test-coverage.sh all lcov
```

### Debugging Tests
```bash
# Run specific test with output
cargo test test_name -- --nocapture

# Debug with logging
RUST_LOG=debug ./test-by-tag.sh unit

# Run single test file
cargo test --test specific_test_file
```

## 📊 Test Reports

### Report Generation
```bash
# Generate JSON report
./run-llmspell-tests.sh all --report

# Custom report location
REPORT_DIR=./test-reports ./run-llmspell-tests.sh all --report
```

### Report Structure
```
test-reports/
├── summary.json       # Overall summary
├── unit/             # Unit test results
├── integration/      # Integration results
├── coverage/         # Coverage data
└── timings.json      # Test execution times
```

## 🐛 Troubleshooting

### Common Issues

**Test failures:**
```bash
# Re-run failed tests only
cargo test --failed

# Debug specific test
RUST_LOG=trace cargo test test_name -- --nocapture
```

**Coverage issues:**
```bash
# Clean and rebuild
cargo clean
./test-coverage.sh all html

# Check ignored code
grep -r "#\[cfg(not(tarpaulin" .
```

**Slow tests:**
```bash
# Identify slow tests
./test-by-tag.sh all -- -Z unstable-options --report-time

# Skip slow tests
SKIP_SLOW=true ./run-llmspell-tests.sh fast
```

## 🔗 Related Documentation

- [Testing Guide](../../docs/development/testing.md)
- [llmspell-testing Crate](../../llmspell-testing/README.md)
- [Quality Scripts](../quality/README.md)