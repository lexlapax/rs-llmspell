# CLI Command Architecture

**Version**: v0.9.0 (Phase 10 Complete)
**Status**: Production-Ready with Daemon Support and Service Integration
**Last Updated**: December 2024
**Phase**: 10 Complete (Integrated Kernel with Daemon Support)

## Executive Summary

This document describes the CLI command architecture implemented in LLMSpell v0.9.0 with integrated kernel architecture and full daemon support. Phase 9 achieved a unified kernel architecture eliminating runtime isolation issues, while Phase 10 added Unix daemon mode with signal handling and service integration.

**Phase 9 Achievements**: Integrated kernel with global IO runtime, protocol/transport abstraction, DAP bridge for debugging, and comprehensive tracing.

**Phase 10 Achievements**: Daemon mode with double-fork technique, signal-to-message bridge (SIGTERM→shutdown_request), systemd/launchd service integration, enhanced logging infrastructure, and consolidated state/sessions into kernel.

---

## Table of Contents

1. [Command Hierarchy](#1-command-hierarchy)
2. [Flag Consolidation](#2-flag-consolidation)
3. [Primary Execution Commands](#3-primary-execution-commands)
4. [Subcommand Groups](#4-subcommand-groups)
5. [Daemon Mode and Service Integration](#5-daemon-mode-and-service-integration)
6. [Signal Handling Architecture](#6-signal-handling-architecture)
7. [Dual-Mode Design](#7-dual-mode-design)
8. [Help System](#8-help-system)
9. [Breaking Changes](#9-breaking-changes)
10. [Implementation Examples](#10-implementation-examples)

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
  app               # Discover and run applications from filesystem
```

### 1.2 Command Tree

```
llmspell
├── run <script> [--rag-profile] [-- args...]
├── exec <code> [--rag-profile]
├── repl [--history]
├── debug <script> [--break-at] [--port] [-- args...]
├── kernel
│   ├── start [--port] [--daemon] [--log-file] [--pid-file]
│   ├── stop <id|--pid-file>
│   ├── status [id]
│   ├── connect <address>
│   ├── install-service [--type systemd|launchd]
│   └── signal <id> <signal>
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
└── app
    ├── list
    ├── info <name>
    ├── run <name> [-- app-args...]
    └── search [--tag TAG] [--complexity LEVEL] [--agents N]
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

DEBUGGING INFRASTRUCTURE (Phase 9):
    Uses integrated DAP bridge in kernel for full debugging support
    Supports VS Code, IntelliJ, and other DAP-compatible IDEs
    Real-time variable inspection and call stack navigation
    Integrated with kernel's execution manager for seamless debugging

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
    start            Start kernel server (integrated kernel from Phase 9)
    stop             Stop kernel by ID or PID file
    status           Show running kernels or specific kernel details
    connect          Connect to external kernel
    install-service  Install as system service (Phase 10)
    signal           Send signal to kernel (Phase 10)

START OPTIONS (Phase 10 daemon mode):
    --port <PORT>        Port to bind [default: auto]
    --daemon             Run as Unix daemon (detach from terminal)
    --log-file <PATH>    Log file for daemon mode [default: ~/.llmspell/kernel.log]
    --pid-file <PATH>    PID file location [default: ~/.llmspell/kernel.pid]
    --connection-file    Write connection info to file
    --idle-timeout <SEC> Shutdown after idle time [default: 3600]
    --max-clients <N>    Maximum concurrent clients [default: 10]

EXAMPLES:
    # Simple start (foreground)
    llmspell kernel start --port 9555

    # Daemon mode with logging (Phase 10)
    llmspell kernel start --daemon --log-file /var/log/llmspell.log

    # Full daemon configuration
    llmspell kernel start --daemon \
        --port 9555 \
        --pid-file /var/run/llmspell.pid \
        --log-file /var/log/llmspell/kernel.log \
        --idle-timeout 7200

    # Stop daemon using PID file
    llmspell kernel stop --pid-file /var/run/llmspell.pid

    # Send signal to kernel (Phase 10)
    llmspell kernel signal abc123 TERM  # Graceful shutdown
    llmspell kernel signal abc123 INT   # Interrupt execution

    # Install as service (Phase 10)
    llmspell kernel install-service --type systemd
    sudo systemctl start llmspell-kernel
    sudo systemctl enable llmspell-kernel
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

## 5. Daemon Mode and Service Integration

### 5.1 Daemon Mode Architecture (Phase 10)

The kernel supports Unix daemon mode with proper process management:

```bash
# Start as daemon
llmspell kernel start --daemon

# What happens internally:
1. First fork() - parent exits, child continues
2. setsid() - create new session, become session leader
3. Second fork() - prevent reacquiring controlling terminal
4. chdir("/") - change to root directory
5. Redirect stdin/stdout/stderr to /dev/null or log file
6. Write PID file for process management
7. Set up signal handlers for graceful shutdown
```

### 5.2 Service File Generation

#### systemd Service (Linux)
```bash
# Generate and install systemd service
llmspell kernel install-service --type systemd

# Creates: /etc/systemd/system/llmspell-kernel.service
```

```ini
[Unit]
Description=LLMSpell Kernel Service
After=network.target

[Service]
Type=forking
PIDFile=/var/run/llmspell-kernel.pid
ExecStart=/usr/local/bin/llmspell kernel start --daemon \
    --pid-file /var/run/llmspell-kernel.pid \
    --log-file /var/log/llmspell/kernel.log
ExecStop=/usr/local/bin/llmspell kernel stop --pid-file /var/run/llmspell-kernel.pid
Restart=on-failure
RestartSec=5s
User=llmspell
Group=llmspell

[Install]
WantedBy=multi-user.target
```

#### launchd Service (macOS)
```bash
# Generate and install launchd plist
llmspell kernel install-service --type launchd

# Creates: ~/Library/LaunchAgents/com.llmspell.kernel.plist
```

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN"
    "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.llmspell.kernel</string>
    <key>ProgramArguments</key>
    <array>
        <string>/usr/local/bin/llmspell</string>
        <string>kernel</string>
        <string>start</string>
        <string>--daemon</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <dict>
        <key>SuccessfulExit</key>
        <false/>
    </dict>
    <key>StandardOutPath</key>
    <string>/var/log/llmspell/kernel.log</string>
    <key>StandardErrorPath</key>
    <string>/var/log/llmspell/kernel.error.log</string>
</dict>
</plist>
```

### 5.3 Logging Infrastructure

Daemon mode includes comprehensive logging:

```rust
// Log levels and outputs
pub struct DaemonLogger {
    log_file: Option<File>,
    syslog: Option<Syslog>,
    rotation: LogRotation,
}

// Log rotation configuration
pub struct LogRotation {
    max_size: u64,        // Rotate at size (default: 100MB)
    max_files: usize,     // Keep N old files (default: 5)
    compress: bool,       // Compress rotated logs (default: true)
}
```

Log format example:
```
2024-12-20T10:15:30.123Z [INFO] kernel::daemon - Kernel started as daemon (PID: 12345)
2024-12-20T10:15:30.456Z [DEBUG] kernel::transport - Binding to port 9555
2024-12-20T10:15:31.789Z [INFO] kernel::signal - Signal handler installed for SIGTERM
2024-12-20T10:16:45.012Z [INFO] kernel::client - Client connected from 127.0.0.1:54321
```

---

## 6. Signal Handling Architecture

### 6.1 Signal-to-Message Bridge (Phase 10)

Unix signals are converted to Jupyter protocol messages for graceful handling:

```rust
pub struct SignalBridge {
    kernel: Arc<IntegratedKernel>,
    signal_handlers: HashMap<Signal, SignalAction>,
}

pub enum SignalAction {
    Shutdown,      // SIGTERM → shutdown_request
    Interrupt,     // SIGINT → interrupt_request
    Reload,        // SIGUSR1 → reload_config
    DumpState,     // SIGUSR2 → dump_state
    Ignore,        // SIGHUP in daemon mode
}
```

### 6.2 Signal Handling Examples

```bash
# Graceful shutdown (SIGTERM)
kill -TERM $(cat ~/.llmspell/kernel.pid)
# Kernel receives shutdown_request, saves state, closes connections

# Interrupt execution (SIGINT)
kill -INT $(cat ~/.llmspell/kernel.pid)
# Kernel receives interrupt_request, stops current execution

# Reload configuration (SIGUSR1)
kill -USR1 $(cat ~/.llmspell/kernel.pid)
# Kernel reloads config without restart

# Dump state for debugging (SIGUSR2)
kill -USR2 $(cat ~/.llmspell/kernel.pid)
# Kernel writes state to log file
```

### 6.3 Signal Safety

All signal handlers are async-signal-safe:

```rust
// Signal handler just sets atomic flag
extern "C" fn handle_sigterm(_: c_int) {
    SHUTDOWN_REQUESTED.store(true, Ordering::SeqCst);
}

// Main loop checks flag periodically
loop {
    if SHUTDOWN_REQUESTED.load(Ordering::SeqCst) {
        kernel.graceful_shutdown().await?;
        break;
    }
    // Normal message processing
}
```

---

## 7. Dual-Mode Design

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

## 8. Help System

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

## 9. Breaking Changes

### 9.1 Command Structure Changes

| Old Command | New Command | Notes |
|-------------|-------------|-------|
| `llmspell kernel --port 9555` | `llmspell kernel start --port 9555` | Subcommand pattern |
| `llmspell serve` (Phase 10 early) | `llmspell kernel start --daemon` | No separate serve command |
| `llmspell apps file-organizer run` | `llmspell app run file-organizer` | Subcommand pattern |
| `llmspell init` | `llmspell config init` | Grouped under config |
| `llmspell validate` | `llmspell config validate` | Grouped under config |
| `llmspell providers` | `llmspell providers list` | Explicit subcommand |

### 9.2 Flag Removals

All these flags have been removed or renamed:

- ❌ `--debug` → Use `--trace` for logging or `debug` command
- ❌ `--verbose` → Use `--trace info`
- ❌ `--debug-level` → Use `--trace <level>`
- ❌ `--debug-format` → Removed (use `--output`)
- ❌ `--debug-modules` → Move to config file
- ❌ `--debug-perf` → Move to config file
- ❌ `--rag`, `--no-rag`, `--rag-config`, `--rag-dims`, `--rag-backend` → Use `--rag-profile`

### 9.3 Migration Examples

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

## 10. Implementation Examples

### 10.1 CLI Structure (Clap)

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

### 10.2 Command Handler

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

### 10.3 Daemon Mode Implementation (Phase 10)

```rust
// llmspell-cli/src/kernel/daemon.rs
use nix::sys::signal::{self, Signal};
use nix::unistd::{self, ForkResult, Pid};
use std::os::unix::io::RawFd;

pub struct DaemonManager {
    kernel: Arc<IntegratedKernel<JupyterProtocol>>,
    config: DaemonConfig,
    signal_bridge: SignalBridge,
}

impl DaemonManager {
    pub fn daemonize(&mut self) -> Result<()> {
        // First fork
        match unsafe { unistd::fork()? } {
            ForkResult::Parent { .. } => {
                // Parent exits
                std::process::exit(0);
            }
            ForkResult::Child => {}
        }

        // Create new session
        unistd::setsid()?;

        // Second fork (prevent controlling terminal)
        match unsafe { unistd::fork()? } {
            ForkResult::Parent { .. } => {
                std::process::exit(0);
            }
            ForkResult::Child => {}
        }

        // Change to root directory
        std::env::set_current_dir("/")?;

        // Clear umask
        unsafe { libc::umask(0) };

        // Redirect standard file descriptors
        self.redirect_stdio()?;

        // Write PID file
        self.write_pid_file()?;

        // Set up signal handlers
        self.setup_signal_handlers()?;

        // Start kernel with logging
        info!("Kernel daemonized successfully (PID: {})", std::process::id());
        Ok(())
    }

    fn redirect_stdio(&self) -> Result<()> {
        let dev_null = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open("/dev/null")?;

        let log_file = if let Some(path) = &self.config.log_file {
            std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(path)?
        } else {
            dev_null.try_clone()?
        };

        // Redirect stdin to /dev/null
        unistd::dup2(dev_null.as_raw_fd(), 0)?;

        // Redirect stdout/stderr to log file
        unistd::dup2(log_file.as_raw_fd(), 1)?;
        unistd::dup2(log_file.as_raw_fd(), 2)?;

        Ok(())
    }
}
```

### 10.4 Script Argument Handling

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

## Architectural Decisions

### Why RAG and Tools Are Not CLI Commands

After comprehensive analysis (Task 10.17.2), the following architectural decisions were made:

#### RAG Operations
- **NOT implemented as CLI command** - RAG operations are script-context operations
- **Access via script API**: Use `RAG.*` methods within Lua/JavaScript/Python scripts
- **Configuration via flag**: `--rag-profile` flag available on execution commands (run, exec, repl, debug)
- **Rationale**: RAG operations require script context and state management that doesn't make sense as standalone CLI operations

#### Tools Management
- **NOT implemented as CLI command** - Tools are runtime discoveries, not CLI operations
- **Access via script API**: Use `Tool.*` methods within scripts
- **Auto-discovery**: Tools are automatically discovered and loaded at runtime
- **Rationale**: No user value in CLI-level tool management; tools are implementation details

#### Info Command
- **DELETED** - The info command showing engine availability was vestigial code
- **Engine errors**: Already handled by `--engine` flag with clear error messages
- **Rationale**: Trivial information not worth maintenance burden

These decisions maintain CLI simplicity while keeping full functionality accessible through the script execution context where it belongs.

---

## Summary

The CLI command architecture provides a production-ready interface with:

### Phase 9 Achievements (Implemented)
1. **Integrated kernel architecture** - No separate kernel binary
2. **Global IO runtime** - Eliminates "dispatch task is gone" errors
3. **DAP bridge integration** - Full debugging support in IDEs
4. **Clear command hierarchy** - Logical subcommand groups
5. **Unambiguous flags** - Removed `--debug` confusion
6. **RAG simplification** - Single profile flag replaces 5

### Phase 10 Enhancements (Completed)
1. **Unix daemon mode** ✅ - Proper double-fork with setsid
2. **Signal handling** ✅ - SIGTERM/SIGINT to Jupyter messages
3. **Service integration** ✅ - systemd/launchd support
4. **Comprehensive logging** ✅ - With rotation and syslog
5. **Multi-tenant kernels** ✅ - Isolation and resource limits
6. **Production deployment** ✅ - PID files, idle timeout, connection limits
7. **Consolidated kernel** ✅ - State/sessions merged into llmspell-kernel

The architecture maintains backward compatibility for basic usage while providing robust service features for production deployments.

---

*This document reflects the completed CLI command architecture from Phase 10 implementation with integrated kernel, daemon mode, and full service integration.*