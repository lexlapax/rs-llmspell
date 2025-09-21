# Scripts Directory

**Quality assurance and utility scripts for LLMSpell development**

**🔗 Navigation**: [← Project Root](../) | [Developer Guide](../docs/developer-guide/) | [Contributing](../CONTRIBUTING.md)

---

## Overview

This directory contains utility scripts for development, testing, quality assurance, and user-friendly application launching.

## User Scripts

### llmspell-easy.sh
**Purpose**: User-friendly launcher for LLMSpell applications
**Usage**: `./scripts/llmspell-easy.sh [app-name]`

Features:
- Auto-detects llmspell binary location
- Checks and guides API key setup
- Simplified application launching
- Interactive setup wizard for first-time users

Examples:
```bash
./scripts/llmspell-easy.sh                # List available apps
./scripts/llmspell-easy.sh file-organizer  # Run file organizer
./scripts/llmspell-easy.sh help           # Show help
```

## Quality Check Scripts

### quality-check.sh
Full quality gates check including all tests and optional coverage analysis.
- Runs formatting check, clippy, build, all tests, documentation
- Optional: coverage analysis (requires cargo-tarpaulin)
- Optional: security audit (requires cargo-audit)
- Use `SKIP_SLOW_TESTS=true` to exclude slow/external tests
- Takes 5+ minutes for full run

### quality-check-fast.sh
Fast quality check with essential validations.
- Runs formatting, clippy, build, unit tests only, documentation
- Skips integration tests for speed
- Takes ~1 minute

### quality-check-minimal.sh
Minimal quality check for quick validation.
- Only runs formatting, clippy, and compilation check
- Takes seconds to complete
- Good for pre-commit checks

## Application Validation Suite

### validate_applications.py
**Purpose**: Comprehensive validation of all 9 example Lua applications
**Usage**: `python3 scripts/validate_applications.py [options]`

Features:
- Tests all example applications via llmspell CLI
- Layer-based testing (Universal → Expert)
- Output validation against expected patterns
- File creation verification
- Performance tracking and reporting
- HTML/JSON report generation
- **Script argument testing** - Validates that applications respect CLI arguments

```bash
# Run standard tests (excludes webapp-creator)
python3 scripts/validate_applications.py

# Include expensive tests (webapp-creator takes 8+ minutes)
RUN_EXPENSIVE_TESTS=1 python3 scripts/validate_applications.py

# Generate reports
python3 scripts/validate_applications.py --html report.html --json report.json

# Verbose output for debugging
python3 scripts/validate_applications.py --verbose
```

**Script Arguments Support:**
The validation suite tests that applications properly handle script arguments passed via the CLI.
For example, webapp-creator is tested with `--output /tmp/test-webapp-output` to verify that
the ARGS global is properly injected into the Lua runtime.

**Application Test Matrix:**

| Layer | Agents | Applications | Runtime | Complexity |
|-------|--------|-------------|---------|------------|
| 1 - Universal | 2-3 | file-organizer, research-collector | <30s | Basic agents, simple workflows |
| 2 - Power User | 4 | content-creator | ~30s | Conditional workflows |
| 3 - Business | 5-7 | personal-assistant, communication-manager, code-review-assistant | 30-60s | State persistence |
| 4 - Professional | 8 | process-orchestrator, knowledge-base | 60-90s | Complex orchestration, RAG |
| 5 - Expert | 21 | webapp-creator | 8-10min | Full app generation |

**Current Status:** ✅ 9/9 applications passing (100% success rate)

## Test Runner Scripts

### test-by-tag.sh
Run tests filtered by category tag.
```bash
./scripts/test-by-tag.sh <tag> [cargo test args]
```

Available tags:
- `unit` - Library unit tests only
- `integration` - Integration tests only
- `tool` - Tests in llmspell-tools package
- `agent` - Tests in llmspell-agents package
- `workflow` - Tests with "workflow" in name
- `fast` - Fast unit tests
- `slow` - Ignored tests marked as slow
- `external` - Tests requiring external services
- `all` - All tests including ignored

Examples:
```bash
./scripts/test-by-tag.sh unit
./scripts/test-by-tag.sh tool --release
./scripts/test-by-tag.sh external -- --nocapture
```

### test-multiple-tags.sh
Run tests matching multiple tags.
```bash
./scripts/test-multiple-tags.sh "tag1,tag2" [cargo test args]
```

Examples:
```bash
./scripts/test-multiple-tags.sh "tool,fast"
./scripts/test-multiple-tags.sh "unit,!slow"
```

### list-tests-by-tag.sh
List tests that would run for a given tag without executing them.
```bash
./scripts/list-tests-by-tag.sh <tag>
```

Available tags:
- `unit` - Show unit tests
- `integration` - Show integration tests
- `tool` - Show tool tests
- `ignored` - Show ignored tests with reasons
- `all` - Show test count summary

### tag-integration-tests.sh
Analyze test files and suggest appropriate `#[ignore]` tags based on their characteristics.
```bash
./scripts/tag-integration-tests.sh         # Dry run mode - show suggestions
./scripts/tag-integration-tests.sh --apply  # Apply suggestions (not yet implemented)
```

Detects:
- External network dependencies
- Tool/bridge/LLM tests
- Slow operations
- Database usage

### test-timings.sh
Run tests and show execution times to identify slow tests.
```bash
./scripts/test-timings.sh              # Time all tests
./scripts/test-timings.sh llmspell-tools  # Time tests in specific package
```

Helps identify tests that should be marked with `#[ignore = "slow"]`

## Best Practices

1. **Before commits**: Run `./scripts/quality-check-minimal.sh`
2. **Before pushing**: Run `./scripts/quality-check-fast.sh`
3. **Before PRs**: Run `./scripts/quality-check.sh`
4. **During development**: Use `test-by-tag.sh` for focused testing

## CI Integration

### ci-test.sh
**Purpose**: Unified CI test runner with configurable test levels
**Usage**: `TEST_LEVEL=<level> ./scripts/ci-test.sh`

Provides consistent test execution for both local development and CI environments.

**Test Levels:**
- `minimal` - Format and lint checks only (<1 min)
- `fast` - Unit tests and basic validation (~2 min)
- `full` - All tests including application validation (~10 min)
- `expensive` - Include webapp-creator test (~20 min)
- `coverage` - Generate code coverage report

```bash
# Local CI simulation
TEST_LEVEL=minimal ./scripts/ci-test.sh   # Quick PR checks
TEST_LEVEL=fast ./scripts/ci-test.sh      # Before pushing
TEST_LEVEL=full ./scripts/ci-test.sh      # Before merging

# With custom report directory
REPORT_DIR=./my-reports TEST_LEVEL=full ./scripts/ci-test.sh
```

### GitHub Actions Workflows

**`.github/workflows/test.yml`** - Main test workflow
- Runs on: Pull requests, pushes to main, manual trigger
- Test levels based on event type
- Cross-platform testing (Linux, macOS, Windows)
- Performance benchmarking
- Security audit

**`.github/workflows/scheduled-tests.yml`** - Scheduled comprehensive tests
- Daily: Full test suite at 2 AM UTC
- Weekly: Expensive tests including webapp-creator (Sunday 3 AM UTC)
- Code coverage analysis with threshold checking
- Performance regression detection
- Automatic issue creation on failure

### CI Configuration

**Environment Variables:**
- `TEST_LEVEL` - Test level to run (minimal/fast/full/expensive)
- `RUN_EXPENSIVE_TESTS` - Enable webapp-creator testing
- `REPORT_DIR` - Directory for test reports (default: ./test-reports)

**GitHub Secrets Required:**
- `OPENAI_API_KEY` - For running application tests with real API (optional)

### Test Reports

CI generates multiple report types:
- **HTML Report** - Visual test results dashboard
- **JSON Report** - Machine-readable test data
- **Coverage Report** - Code coverage analysis (weekly)
- **Performance Report** - Benchmark comparisons

All reports are uploaded as GitHub Actions artifacts for review.

## Environment Variables

- `SKIP_SLOW_TESTS=true` - Skip slow/external tests in quality-check.sh
- `RUN_EXPENSIVE_TESTS=1` - Enable webapp-creator test (8+ minutes)
- `TEST_LEVEL=<level>` - CI test level (minimal/fast/full/expensive)
- `REPORT_DIR=<path>` - Custom report output directory
- `LLMSPELL_BIN=<path>` - Override llmspell binary location