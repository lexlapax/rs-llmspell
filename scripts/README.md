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

## Environment Variables

- `SKIP_SLOW_TESTS=true` - Skip slow/external tests in quality-check.sh
- More test filtering options coming in future updates