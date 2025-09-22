# CLI Command Architecture

**Version**: v0.9.0  
**Status**: Production Implementation - Breaking Changes  
**Last Updated**: December 2025  
**Phase**: 9 (REPL, Debugging, and Kernel Architecture)  

## Executive Summary

This document describes the CLI command architecture implemented in LLMSpell v0.9.0. The restructure addresses critical usability issues discovered during Phase 9.8 testing, implementing a clean subcommand organization with proper separation of concerns. **All changes are breaking** - no backward compatibility maintained as we're pre-1.0.

**Key Changes**: Removed ambiguous `--debug` flag (replaced with `--trace` for logging and `debug` command for debugging), simplified RAG configuration from 5 flags to 1, and reorganized all commands into logical subcommand groups.

---

## Table of Contents

1. [Command Hierarchy](#1-command-hierarchy)
2. [Flag Consolidation](#2-flag-consolidation)
3. [Primary Execution Commands](#3-primary-execution-commands)
4. [Subcommand Groups](#4-subcommand-groups)
5. [Dual-Mode Design](#5-dual-mode-design)
6. [Help System](#6-help-system)
7. [Breaking Changes](#7-breaking-changes)
8. [Implementation Examples](#8-implementation-examples)

---

## 1. Command Hierarchy

### 1.1 Overall Structure

```
llmspell [global-flags] <command> [command-flags] [-- script-args]

Global Flags (available everywhere):
  --trace <LEVEL>    # Logging verbosity: off|error|warn|info|debug|trace
  --config <FILE>    # Configuration file path
  --output <FORMAT>  # Output format: text|json|yaml|pretty
  -h, --help        # Show contextual help

Primary Commands:
  run <script>       # Execute script file
  exec <code>        # Execute inline code
  repl              # Interactive REPL
  debug <script>     # Debug with breakpoints

Subcommand Groups:
  kernel            # Kernel management
  state             # State management
  session           # Session management
  config            # Configuration management
  keys              # API key management
  backup            # Backup operations
  app               # Example applications
```

### 1.2 Command Tree

```
llmspell
├── run <script> [--rag-profile] [-- args...]
├── exec <code> [--rag-profile]
├── repl [--history]
├── debug <script> [--break-at] [--port] [-- args...]
├── kernel
│   ├── start [--port] [--daemon]
│   ├── stop <id>
│   ├── status [id]
│   └── connect <address>
├── state
│   ├── show [--key] [--kernel|--config]
│   ├── clear [--key] [--kernel|--config]
│   ├── export [--kernel|--config]
│   └── import [--kernel|--config]
├── session
│   ├── list [--kernel|--config]
│   ├── show <id> [--kernel|--config]
│   ├── replay <id> [--kernel|--config]
│   └── delete <id> [--kernel|--config]
├── config
│   ├── init [--force]
│   ├── validate [--file]
│   └── show [--section]
├── keys
│   ├── add <provider> <key>
│   ├── list
│   └── remove <provider>
├── backup
│   ├── create [--output]
│   ├── restore <file>
│   ├── list
│   └── delete <id>
└── app [name] [-- app-args...]
```

---

## 2. Flag Consolidation

### 2.1 Global Flag Changes

| Old Flag | New Flag/Command | Purpose | Rationale |
|----------|------------------|---------|-----------|
| `--debug` (global) | `--trace` | Logging verbosity | Removes ambiguity |
| `--debug` (action) | `debug` command | Interactive debugging | Clear separation |
| `--verbose` | `--trace info` | Info-level logging | Unified logging |
| `--debug-level` | `--trace <level>` | Set log level | Consolidated |
| `--debug-format` | Removed | Use `--output` | Redundant |
| `--debug-modules` | Config file | Module filtering | Better in config |
| `--debug-perf` | Config file | Performance logging | Better in config |
| `--engine` (global) | Per-command flag | Script engine | Command-specific |

### 2.2 RAG Configuration Simplification

**Before** (20 flag instances across 4 commands):
```bash
llmspell run script.lua \
  --rag \
  --rag-config custom.toml \
  --rag-dims 384 \
  --rag-backend hnsw \
  --no-rag  # Confusing!
```

**After** (4 flag instances total):
```bash
llmspell run script.lua --rag-profile production
```

Profile defined in config:
```toml
[rag.profiles.production]
enabled = true
backend = "hnsw"
dimensions = 384
config_file = "custom.toml"
description = "Production RAG configuration"
```

---

## 3. Primary Execution Commands

### 3.1 run - Execute Script File

```bash
llmspell run <script> [OPTIONS] [-- SCRIPT_ARGS...]

OPTIONS:
    --engine <ENGINE>      Script engine [default: lua]
    --format <FORMAT>      Output format [overrides global --output]
    --kernel <ADDRESS>     Kernel connection [default: auto]
    --stream              Enable streaming output
    --rag-profile <NAME>   RAG configuration profile

SCRIPT ARGUMENTS:
    Arguments after -- are passed to the script as ARGS global variable
    Format: -- --key value --flag
    Access in Lua: ARGS["key"], ARGS["flag"]

EXAMPLES:
    llmspell run script.lua
    llmspell run script.lua -- arg1 arg2
    llmspell run script.js --engine javascript
    llmspell run webapp-creator.lua -- --output /tmp/my-app --input spec.lua
    llmspell run ml_task.lua --rag-profile production -- --model gpt-4
```

### 3.2 exec - Execute Inline Code

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
    llmspell exec "agent.query('What is 2+2?')"
```

### 3.3 repl - Interactive REPL

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

### 3.4 debug - Interactive Debugging

```bash
llmspell debug <script> [OPTIONS] [-- SCRIPT_ARGS...]

OPTIONS:
    --break-at <FILE:LINE> Set breakpoints (repeatable)
    --watch <EXPR>         Watch expressions (repeatable)
    --step                Start in step mode
    --engine <ENGINE>      Script engine [default: lua]
    --kernel <ADDRESS>     Kernel connection [default: auto]
    --port <PORT>          DAP server port for IDE attachment

SCRIPT ARGUMENTS:
    Arguments after -- are passed to the script as ARGS global variable
    Available during debugging for testing different parameters

EXAMPLES:
    llmspell debug script.lua --break-at main.lua:10
    llmspell debug app.lua --watch "state.counter" --step -- --verbose
    llmspell debug test.lua --break-at test.lua:5 -- --test-mode
    llmspell debug remote.lua --port 9555  # For VS Code attachment
```

---

## 4. Subcommand Groups

### 4.1 Kernel Management

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
    llmspell kernel status abc123             # Show detailed status
    llmspell kernel stop abc123
    llmspell kernel connect localhost:9555
```

#### Kernel Status Output

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

### 4.2 State Management

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
    --key <KEY>             Specific state key (for show/clear)

EXAMPLES:
    # Show state from running kernel
    llmspell state show --kernel abc123 --scope global
    
    # Show state from config file (no kernel needed)
    llmspell state show --config production.toml --scope session
    
    # Clear specific key
    llmspell state clear --key user.preferences
    
    # Export/import state
    llmspell state export --kernel abc123 > kernel_state.json
    llmspell state import --kernel abc123 < state_backup.json
```

### 4.3 Session Management

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
    
    # Show session details
    llmspell session show sess_xyz --kernel abc123
    
    # Replay session
    llmspell session replay sess_xyz --kernel abc123
    
    # Delete old session
    llmspell session delete old_session --kernel abc123
```

### 4.4 Configuration Management

```bash
llmspell config <SUBCOMMAND>

SUBCOMMANDS:
    init      Initialize configuration
    validate  Validate configuration
    show      Show configuration

EXAMPLES:
    llmspell config init --force
    llmspell config validate --file custom.toml
    llmspell config show --section rag
```

---

## 5. Dual-Mode Design

### 5.1 Online vs Offline Operations

Many commands support both kernel and config contexts:

#### Online Mode (`--kernel`)
- Operates on running kernel's live state
- Real-time state modifications
- Active session management
- Multi-client coordination
- Memory-resident operations

#### Offline Mode (`--config`)
- Operates on persisted state via config file
- No kernel required to be running
- Direct file-based state access
- Useful for backup/restore operations
- Debugging without kernel overhead

#### Auto Mode (default)
- Smart detection of best option
- Finds running kernel if available
- Falls back to config file if no kernel
- Uses connection discovery mechanism

### 5.2 Resolution Order

```rust
// Command resolution logic
async fn resolve_context(kernel: Option<String>, config: Option<String>) -> Context {
    match (kernel, config) {
        (Some(k), _) => Context::Kernel(connect_to_kernel(k).await?),
        (_, Some(c)) => Context::Config(load_config(c)?),
        (None, None) => {
            // Auto mode
            if let Some(k) = find_running_kernel().await? {
                Context::Kernel(k)
            } else {
                Context::Config(load_default_config()?)
            }
        }
    }
}
```

### 5.3 Usage Examples

```bash
# Online - uses running kernel
llmspell state show --kernel localhost:9555
llmspell session list --kernel abc123

# Offline - uses config file
llmspell state show --config production.toml
llmspell session list --config ~/.llmspell/config.toml

# Auto - detects best option
llmspell state show    # Finds kernel or uses config
llmspell session list  # Smart detection
```

---

## 6. Help System

### 6.1 Contextual Help Behavior

The `-h/--help` flag provides contextual help based on where it's used:

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

### 6.2 Help Precedence Rules

1. **Help flag terminates parsing** - When `-h/--help` is encountered, show help and exit
2. **Position matters** - Help shows context for the command level where it appears
3. **Ignores other flags** - `llmspell run --engine js --help` still shows run help
4. **Works with partial commands** - `llmspell kernel` (without subcommand) shows kernel help

### 6.3 Help Output Structure

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

---

## 7. Breaking Changes

### 7.1 Command Structure Changes

| Old Command | New Command | Notes |
|-------------|-------------|-------|
| `llmspell kernel --port 9555` | `llmspell kernel start --port 9555` | Subcommand pattern |
| `llmspell apps file-organizer run` | `llmspell app file-organizer` | Simplified |
| `llmspell init` | `llmspell config init` | Grouped under config |
| `llmspell validate` | `llmspell config validate` | Grouped under config |
| `llmspell providers` | `llmspell providers list` | Explicit subcommand |

### 7.2 Flag Removals

All these flags have been removed or renamed:

- ❌ `--debug` → Use `--trace` for logging or `debug` command
- ❌ `--verbose` → Use `--trace info`
- ❌ `--debug-level` → Use `--trace <level>`
- ❌ `--debug-format` → Removed (use `--output`)
- ❌ `--debug-modules` → Move to config file
- ❌ `--debug-perf` → Move to config file
- ❌ `--rag`, `--no-rag`, `--rag-config`, `--rag-dims`, `--rag-backend` → Use `--rag-profile`

### 7.3 Migration Examples

```bash
# OLD
llmspell run script.lua --debug --verbose
llmspell run script.lua --rag --rag-backend hnsw --rag-dims 384

# NEW
llmspell run script.lua --trace debug
llmspell run script.lua --rag-profile production

# OLD
llmspell --debug script.lua  # Debug mode? Logging?

# NEW (clear separation)
llmspell debug script.lua    # Interactive debugging
llmspell run script.lua --trace debug  # With debug logging
```

---

## 8. Implementation Examples

### 8.1 CLI Structure (Clap)

```rust
// llmspell-cli/src/cli.rs
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "llmspell")]
#[command(about = "Scriptable LLM interactions")]
pub struct Cli {
    /// Set trace level
    #[arg(long, global = true, value_enum)]
    pub trace: Option<TraceLevel>,
    
    /// Configuration file
    #[arg(long, global = true)]
    pub config: Option<PathBuf>,
    
    /// Output format
    #[arg(long, global = true, value_enum)]
    pub output: Option<OutputFormat>,
    
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Execute a script file
    Run {
        /// Script to execute
        script: PathBuf,
        
        /// RAG profile to use
        #[arg(long)]
        rag_profile: Option<String>,
        
        /// Script arguments
        #[arg(last = true)]
        args: Vec<String>,
    },
    
    /// Debug a script with interactive debugging
    Debug {
        script: PathBuf,
        
        /// Set breakpoints
        #[arg(long)]
        break_at: Vec<String>,
        
        /// DAP server port
        #[arg(long)]
        port: Option<u16>,
        
        #[arg(last = true)]
        args: Vec<String>,
    },
    
    /// Manage kernel servers
    Kernel {
        #[command(subcommand)]
        command: KernelCommands,
    },
    
    /// Manage persistent state
    State {
        #[command(subcommand)]
        command: StateCommands,
    },
    
    // ... other commands
}
```

### 8.2 Command Handler

```rust
// llmspell-cli/src/main.rs
#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
    // Set up tracing based on --trace flag
    if let Some(level) = cli.trace {
        setup_tracing(level);
    }
    
    // Load configuration
    let config = load_config(cli.config)?;
    
    // Set output format
    let output_format = cli.output.unwrap_or(config.output.format);
    
    // Handle command
    match cli.command {
        Commands::Run { script, rag_profile, args } => {
            handle_run_command(script, rag_profile, args, config, output_format).await
        }
        Commands::Debug { script, break_at, port, args } => {
            handle_debug_command(script, break_at, port, args, config, output_format).await
        }
        Commands::Kernel { command } => {
            handle_kernel_command(command, config, output_format).await
        }
        Commands::State { command } => {
            handle_state_command(command, config, output_format).await
        }
        // ... other handlers
    }
}
```

### 8.3 Script Argument Handling

#### Parsing and Passing Arguments

```rust
// Properly separate CLI args from script args
fn parse_args(raw_args: Vec<String>) -> (Vec<String>, Vec<String>) {
    if let Some(separator_pos) = raw_args.iter().position(|arg| arg == "--") {
        let cli_args = raw_args[..separator_pos].to_vec();
        let script_args = raw_args[separator_pos + 1..].to_vec();
        (cli_args, script_args)
    } else {
        (raw_args, vec![])
    }
}

// Convert script args to HashMap for script injection
fn parse_script_args(args: &[String]) -> HashMap<String, String> {
    let mut result = HashMap::new();
    let mut i = 0;

    while i < args.len() {
        let arg = &args[i];
        if arg.starts_with("--") {
            let key = arg.trim_start_matches("--").to_string();
            if i + 1 < args.len() && !args[i + 1].starts_with("--") {
                // Key-value pair
                result.insert(key, args[i + 1].clone());
                i += 2;
            } else {
                // Flag
                result.insert(key, "true".to_string());
                i += 1;
            }
        } else {
            // Positional argument
            result.insert(i.to_string(), arg.clone());
            i += 1;
        }
    }
    result
}
```

#### Script Executor Implementation

```rust
// ScriptExecutor trait extension
async fn execute_script_with_args(
    &self,
    script: &str,
    args: HashMap<String, String>,
) -> Result<ScriptExecutionOutput, LLMSpellError> {
    // Inject ARGS global into script preamble
    let script_with_args = if !args.is_empty() {
        let mut preamble = String::from("-- Injected script arguments\nARGS = {}\n");
        for (key, value) in &args {
            let escaped_value = value.replace('\\', "\\\\").replace('"', "\\\"");
            preamble.push_str(&format!("ARGS[\"{}\"] = \"{}\"\n", key, escaped_value));
        }
        preamble.push_str("\n-- Original script\n");
        preamble.push_str(script);
        preamble
    } else {
        script.to_string()
    };

    self.execute_script(&script_with_args).await
}
```

#### Usage in Scripts

```lua
-- Example: webapp-creator using script arguments
-- Run: llmspell run webapp-creator.lua -- --output /tmp/my-app --template vue

local output_dir = ARGS and ARGS["output"] or "./output"
local template = ARGS and ARGS["template"] or "react"

print("Creating app in: " .. output_dir)
print("Using template: " .. template)

-- Use the arguments
local file_writer = get_tool("file_writer")
file_writer:write({
    path = output_dir .. "/package.json",
    content = generate_package_json(template)
})
```

---

## Summary

The CLI command architecture provides a clean, intuitive interface through:

1. **Clear command hierarchy** with logical subcommand groups
2. **Unambiguous flags** - no more `--debug` confusion
3. **RAG simplification** - single profile flag replaces 5
4. **Dual-mode design** - online (kernel) and offline (config) operations
5. **Contextual help** - intelligent help based on command level
6. **Script argument separation** - proper `--` handling

All changes are breaking but necessary for a cleaner, more maintainable CLI. The restructure eliminates confusion, reduces flag duplication (20 → 4 for RAG), and provides a solid foundation for future features.

---

*This document consolidates the CLI command architecture from Phase 9.8.13.10 implementation, providing a comprehensive reference for the restructured command-line interface.*