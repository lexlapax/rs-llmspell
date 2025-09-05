# CLI Architecture Restructure Design

**Date**: 2025-09-05  
**Status**: Breaking Changes - No Backward Compatibility  
**Target**: v0.9.0  

## Executive Summary

This document outlines the complete restructuring of the LLMSpell CLI to address critical usability issues discovered during Phase 9.8 testing. **All changes are breaking** - we are not maintaining backward compatibility as we're pre-1.0.

## Problems Identified

1. **Flag Confusion**: `--debug` means both trace logging AND interactive debugging
2. **Inconsistent Structure**: Some commands use flags where subcommands are appropriate
3. **Architectural Issues**: Script arguments parsed but not passed, `--engine` flag ignored
4. **Excessive Duplication**: RAG flags repeated 20 times (5 flags × 4 commands)
5. **Deep Nesting**: Apps command has unnecessary 3-level structure

## Current vs Proposed Structure

### Command Hierarchy Comparison

```
CURRENT STRUCTURE                    PROPOSED STRUCTURE
─────────────────────────────────────────────────────────────
llmspell [global] command           llmspell [global] command [flags]

Global Flags:                        Global Flags:
  --engine (ignored!)                  --trace (logging level)
  --debug (ambiguous!)                 --config (config file)
  --output                             --output (format)
  --config                             --help
  --verbose
  --debug-level
  --debug-format
  --debug-modules
  --debug-perf

Primary Commands:                    Primary Commands:
  run <script> [flags]                 run <script> [flags] -- [args]
  exec <code> [flags]                  exec <code> [flags]
  repl [flags]                         repl [flags]
  debug <script> [args]                debug <script> [flags] -- [args]
  
Flat Commands:                       Subcommand Groups:
  kernel --port --id                   kernel start|stop|status|connect
  providers [--detailed]               providers list [--detailed]
  validate [--config]                  config init|validate|show
  info [--all]                         info [--all]
  init [--output --force]              
  keys <subcommand>                    keys add|list|remove
  backup <subcommand>                  backup create|restore|list|delete
  apps <app> <subcommand>              app <name>|list [-- args]
  setup [--force]                      setup [--force]
                                       state show|clear|export|import
                                       session list|replay|delete
```

### Flag Consolidation Strategy

#### Global Flags (Available Everywhere)
```bash
--trace <LEVEL>    # Logging verbosity: off|error|warn|info|debug|trace
--config <FILE>    # Configuration file path
--output <FORMAT>  # Output format: text|json|yaml|pretty
-h, --help        # Show contextual help (see Help System section)
```

#### Command-Specific Flags

**run/exec/repl Commands:**
```bash
--engine <ENGINE>     # Script engine: lua|javascript|python
--format <FORMAT>     # Override output format (command-specific)
--kernel <ADDRESS>    # Connect to kernel: auto|localhost:9555|/path/to/connection.json
--stream             # Enable streaming output
```

**debug Command (NEW):**
```bash
--break <FILE:LINE>   # Set breakpoints
--watch <EXPR>        # Watch expressions
--step               # Start in step mode
```

**RAG Configuration (Simplified):**
```bash
--rag-profile <NAME>  # Use named RAG profile from config
# Replaces: --rag, --no-rag, --rag-config, --rag-dims, --rag-backend
```

## Breaking Changes

### 1. Flag Removals/Renames
- ❌ `--debug` → ✅ `--trace` (global flag for logging)
- ❌ `--verbose` → ✅ `--trace info` (use trace levels)
- ❌ `--debug-level` → ✅ `--trace <level>` (consolidated)
- ❌ `--debug-format` → Removed (use --output)
- ❌ `--debug-modules` → Moved to config file
- ❌ `--debug-perf` → Moved to config file
- ❌ `--engine` (global) → ✅ `--engine` (command-specific)

### 2. Command Structure Changes
- ❌ `llmspell kernel --port 9555` → ✅ `llmspell kernel start --port 9555`
- ❌ `llmspell apps file-organizer run` → ✅ `llmspell app file-organizer`
- ❌ `llmspell init` → ✅ `llmspell config init`
- ❌ `llmspell validate` → ✅ `llmspell config validate`
- ❌ `llmspell providers` → ✅ `llmspell providers list`

### 3. New Commands
- ✅ `llmspell debug <script>` - Interactive debugging with breakpoints
- ✅ `llmspell state` - State management commands
- ✅ `llmspell session` - Session management commands
- ✅ `llmspell kernel status` - Show running kernels
- ✅ `llmspell kernel connect` - Connect to external kernel

### 4. RAG Configuration
```bash
# OLD (20 flag instances across commands)
llmspell run script.lua --rag --rag-config custom.toml --rag-dims 384 --rag-backend hnsw

# NEW (single profile reference)
llmspell run script.lua --rag-profile production
```

## Command Details

### Primary Execution Commands

#### run - Execute Script File
```bash
llmspell run <script> [OPTIONS] [-- SCRIPT_ARGS...]

OPTIONS:
    --engine <ENGINE>      Script engine [default: lua]
    --format <FORMAT>      Output format [overrides global --output]
    --kernel <ADDRESS>     Kernel connection [default: auto]
    --stream              Enable streaming output
    --rag-profile <NAME>   RAG configuration profile

EXAMPLES:
    llmspell run script.lua
    llmspell run script.lua -- arg1 arg2
    llmspell run script.js --engine javascript
    llmspell run ml_task.lua --rag-profile production
```

#### exec - Execute Inline Code
```bash
llmspell exec <CODE> [OPTIONS]

OPTIONS:
    --engine <ENGINE>      Script engine [default: lua]
    --format <FORMAT>      Output format
    --kernel <ADDRESS>     Kernel connection
    --stream              Enable streaming
    --rag-profile <NAME>   RAG configuration profile

EXAMPLES:
    llmspell exec "print('hello')"
    llmspell exec "console.log('test')" --engine javascript
```

#### repl - Interactive REPL
```bash
llmspell repl [OPTIONS]

OPTIONS:
    --engine <ENGINE>      Script engine [default: lua]
    --kernel <ADDRESS>     Kernel connection
    --history <FILE>       History file path

EXAMPLES:
    llmspell repl
    llmspell repl --engine javascript
    llmspell repl --kernel localhost:9555
```

#### debug - Interactive Debugging
```bash
llmspell debug <script> [OPTIONS] [-- SCRIPT_ARGS...]

OPTIONS:
    --break <FILE:LINE>    Set breakpoints (repeatable)
    --watch <EXPR>         Watch expressions (repeatable)
    --step                Start in step mode
    --engine <ENGINE>      Script engine [default: lua]
    --kernel <ADDRESS>     Kernel connection [default: auto]

EXAMPLES:
    llmspell debug script.lua --break main.lua:10
    llmspell debug app.lua --watch "state.counter" --step
    llmspell debug test.lua --break test.lua:5 --break lib.lua:20
    llmspell debug remote.lua --kernel localhost:9555 --break remote.lua:8
```

### Kernel Management

```bash
llmspell kernel <SUBCOMMAND>

SUBCOMMANDS:
    start     Start kernel server
    stop      Stop kernel by ID
    status    Show running kernels or specific kernel details
    connect   Connect to external kernel

EXAMPLES:
    llmspell kernel start --port 9555 --daemon
    llmspell kernel status                    # List all running kernels
    llmspell kernel status abc123             # Show detailed status of specific kernel
    llmspell kernel stop abc123
    llmspell kernel connect localhost:9555
```

#### kernel status - Details

```bash
# List all kernels
llmspell kernel status
ID        PORT   ENGINE   STATUS   CLIENTS   UPTIME
abc123    9555   lua      idle     0         2h 15m
def456    9556   js       busy     2         45m
ghi789    9557   lua      idle     1         3d 4h

# Show specific kernel details
llmspell kernel status abc123
Kernel ID:        abc123
Port:             9555
Engine:           lua
Status:           idle
Clients:          0
Uptime:           2h 15m
Memory:           45 MB
Last Activity:    5m ago
State Persisted:  yes
Session Count:    3
Connection File:  ~/.llmspell/kernels/abc123.json
```

### State Management

```bash
llmspell state <SUBCOMMAND> [OPTIONS]

SUBCOMMANDS:
    show      Display persisted state
    clear     Clear state by scope
    export    Export state to JSON
    import    Import state from JSON

OPTIONS (for all subcommands):
    --kernel <ID|ADDRESS>   Kernel to operate on [default: auto]
    --config <FILE>         Use state from config file (offline mode)
    --scope <SCOPE>         State scope: global|session|workflow|component

EXAMPLES:
    # Show state from running kernel
    llmspell state show --kernel abc123 --scope global
    llmspell state show --kernel localhost:9555
    
    # Show state from config file (no kernel needed)
    llmspell state show --config production.toml --scope session
    
    # Clear state in specific kernel
    llmspell state clear --kernel abc123 --scope session
    
    # Export state from kernel or config
    llmspell state export --kernel abc123 > kernel_state.json
    llmspell state export --config dev.toml > config_state.json
    
    # Import state into kernel or config
    llmspell state import --kernel abc123 < state_backup.json
    llmspell state import --config prod.toml < state_backup.json
```

#### State Resolution Order
1. If `--kernel` specified: Use that kernel's state
2. If `--config` specified: Use state backend from config (offline)
3. If neither: Auto-detect running kernel or use default config

### Session Management

```bash
llmspell session <SUBCOMMAND> [OPTIONS]

SUBCOMMANDS:
    list      List all sessions
    show      Show session details
    replay    Replay a session
    delete    Delete a session

OPTIONS:
    --kernel <ID|ADDRESS>   Kernel to operate on [default: auto]
    --config <FILE>         Use sessions from config file (offline mode)

EXAMPLES:
    # List sessions from specific kernel
    llmspell session list --kernel abc123
    llmspell session list --kernel localhost:9555
    
    # List sessions from config file
    llmspell session list --config production.toml
    
    # Show session details
    llmspell session show sess_xyz --kernel abc123
    
    # Replay session on specific kernel
    llmspell session replay sess_xyz --kernel abc123
    llmspell session replay sess_xyz --config dev.toml
    
    # Delete session
    llmspell session delete old_session --kernel abc123
```

#### Session Context
- Sessions are tied to the kernel/config that created them
- Replaying a session requires compatible state backend
- Session IDs are unique within a kernel/config context

### Application Launcher

```bash
llmspell app [NAME] [-- APP_ARGS...]

ARGUMENTS:
    NAME    Application name (omit to list available apps)

EXAMPLES:
    llmspell app                          # List available apps
    llmspell app file-organizer -- /path  # Run app with args
    llmspell app research-collector
```

## Design Rationale: Kernel vs Config Context

Many commands support both `--kernel` and `--config` options because:

1. **Online Mode** (`--kernel`): Operates on a running kernel's live state
   - Real-time state modifications
   - Active session management
   - Multi-client coordination
   - Memory-resident operations

2. **Offline Mode** (`--config`): Operates on persisted state via config file
   - No kernel required to be running
   - Direct file-based state access
   - Useful for backup/restore operations
   - Debugging without kernel overhead

3. **Auto Mode** (default): Smart detection
   - Finds running kernel if available
   - Falls back to config file if no kernel
   - Uses connection discovery mechanism

This dual-mode design enables both production operations (kernel-based) and maintenance tasks (config-based) with the same command interface.

## Help System Design

### Contextual Help Behavior

The `-h/--help` flag provides **contextual help** based on where it's used:

```bash
# Global help - shows all commands overview
llmspell --help
llmspell -h

# Command help - shows specific command details
llmspell run --help
llmspell exec -h

# Subcommand group help - shows available subcommands
llmspell kernel --help
llmspell state -h

# Specific subcommand help - shows detailed usage
llmspell kernel start --help
llmspell state show -h
```

### Help Precedence Rules

1. **Help flag terminates parsing** - When `-h/--help` is encountered, show help and exit
2. **Position matters** - Help shows context for the command level where it appears
3. **Ignores other flags** - `llmspell run --engine js --help` still shows run help
4. **Works with partial commands** - `llmspell kernel` (without subcommand) shows kernel help

### Help Output Structure

#### Global Help Format
```
LLMSpell - Scriptable LLM interactions

USAGE:
    llmspell [GLOBAL OPTIONS] <COMMAND> [ARGS]

GLOBAL OPTIONS:
    --trace <LEVEL>     Set trace level [default: warn]
    --config <FILE>     Config file path [default: ~/.llmspell/config.toml]
    --output <FORMAT>   Output format [default: text]
    -h, --help         Print help information

COMMANDS:
    run       Execute a script file
    exec      Execute inline code
    repl      Start interactive REPL
    debug     Debug a script with breakpoints
    kernel    Manage kernel processes
    state     Manage persistent state
    session   Manage sessions
    config    Configuration management
    backup    Backup and restore operations
    app       Run example applications

Run 'llmspell <COMMAND> --help' for more information on a command.
```

#### Command Help Format
```
llmspell-run - Execute a script file

USAGE:
    llmspell run [OPTIONS] <SCRIPT> [-- SCRIPT_ARGS...]

ARGUMENTS:
    <SCRIPT>           Path to script file
    [SCRIPT_ARGS...]   Arguments passed to script

OPTIONS:
    --engine <ENGINE>      Script engine [default: lua]
    --format <FORMAT>      Output format [overrides global]
    --kernel <ADDRESS>     Kernel connection [default: auto]
    --stream              Enable streaming output
    --rag-profile <NAME>   RAG configuration profile
    -h, --help            Print help information

EXAMPLES:
    llmspell run script.lua
    llmspell run script.lua -- arg1 arg2
    llmspell run ml.lua --rag-profile production

GLOBAL OPTIONS:
    --trace <LEVEL>     Set trace level
    --config <FILE>     Config file path
    --output <FORMAT>   Output format
```

### Help Generation Strategy

1. **Auto-generated from clap** - Primary help text from argument definitions
2. **Custom examples** - Hand-written examples in help text
3. **Global options repeated** - Show applicable global flags in command help
4. **Subcommand discovery** - List available subcommands prominently
5. **Exit codes documented** - Help should mention exit codes

### Special Help Cases

```bash
# Help as a command (alternative syntax - both work)
llmspell help              # Same as llmspell --help
llmspell help run           # Same as llmspell run --help
llmspell help kernel start  # Same as llmspell kernel start --help

# Invalid command shows help
llmspell invalid-cmd        # Shows error + suggests --help

# No arguments shows help
llmspell                    # Shows global help

# Incomplete subcommand shows subcommand help
llmspell kernel             # Shows kernel subcommands
llmspell state              # Shows state subcommands
```

## Flag Inheritance Rules

1. **Global flags** are available to all commands and subcommands
2. **Command flags** override global flags where there's overlap
3. **Subcommands** inherit their parent command's flags
4. **Script arguments** must come after `--` separator

## Implementation Priority

1. **Critical** (Blocks other work):
   - Separate --trace from --debug (9.8.13.2)
   - Fix script argument passing (9.8.13.4)
   - Wire up --engine flag (9.8.13.6)

2. **High** (Major usability improvements):
   - Implement kernel subcommands (9.8.13.3)
   - Implement --format flag (9.8.13.5)
   - Simplify RAG configuration (9.8.13.7)

3. **Medium** (Nice to have):
   - Flatten Apps structure (9.8.13.8)
   - Add State/Session commands (9.8.13.9)
   - Update documentation (9.8.13.10)

## Migration Impact

Since we're **not maintaining backward compatibility**:

1. **All existing scripts using CLI will break**
2. **CI/CD pipelines must be updated**
3. **Documentation must be completely rewritten**
4. **No deprecation period - immediate replacement**

## Benefits

1. **Clarity**: Clear separation between logging (--trace) and debugging (debug command)
2. **Consistency**: All management commands use subcommands, not flags
3. **Discoverability**: `llmspell kernel --help` shows all kernel operations
4. **Simplicity**: 20 RAG flag instances reduced to profile references
5. **Correctness**: Script arguments actually work, --engine flag respected
6. **Future-proof**: Structure supports Phase 11 DAP, Phase 12 daemon mode

## Testing Strategy

1. **Remove all existing CLI tests** - they test the old structure
2. **Write new tests from scratch** for the new structure
3. **No compatibility tests** - we're not supporting old commands
4. **Focus on correctness** over migration smoothness

## Success Metrics

- ✅ Zero ambiguous flags (no --debug confusion)
- ✅ All commands follow Unix conventions (noun-verb pattern)
- ✅ Script arguments passed correctly to scripts
- ✅ Engine selection works for all relevant commands
- ✅ RAG configuration simplified from 20 to 4 flag instances
- ✅ Help text clear and comprehensive
- ✅ No dead code paths remaining

---

**Decision**: Proceed with complete restructure, accepting all breaking changes.