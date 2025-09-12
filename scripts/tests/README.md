# Phase 9.9.1 Integration Tests

This directory contains integration test scripts for validating Phase 9 features.

## Main Test Suite

### phase-9-9-1-integration-tests.sh
Comprehensive test suite that validates all Phase 9 components:
- Kernel Architecture (start/stop/status)
- Debug Infrastructure (breakpoints, locals)
- RAG System (ingest/search/clear)
- State Management (set/get/list/delete)
- Session Management (create/list/info/delete)
- REPL Commands (.state, .session, .locals, .help)
- Configuration Management (get/set/list)

Run with:
```bash
./scripts/tests/phase-9-9-1-integration-tests.sh
```

## Interactive Test Scripts (Expect)

### test-debug-infrastructure.exp
Tests the debug command with interactive breakpoint handling:
- Sets breakpoint at line 7
- Continues execution to breakpoint
- Checks .locals command
- Steps through code
- Continues to completion

### test-repl-commands.exp
Tests REPL interactive commands:
- Basic print statements
- .state command
- .session command
- .locals command
- .help command
- .exit command

## Test Results (2025-09-12)

Current status:
- ✅ Working: Kernel start/stop/status, basic REPL
- ⚠️ Partial: Kernel exec (output issues), Session delete only
- ❌ Not Implemented: RAG commands, State commands, Config commands, REPL .state/.session commands

## Requirements

- Expect (for interactive tests): `brew install expect` on macOS
- Built llmspell binary at `./target/debug/llmspell`

## Troubleshooting

If tests fail:
1. Ensure no other kernels are running: `pkill -f llmspell-kernel`
2. Check port availability (9577-9581)
3. Verify expect is installed for interactive tests
4. Check debug build is current: `cargo build`