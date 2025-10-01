# CLI Command Architecture

**Version**: v0.9.0 (Phase 10 Complete including 10.22)
**Status**: Production-Ready with Daemon Support, Service Integration, and Tool Commands
**Last Updated**: September 2025
**Phase**: 10 Complete (Integrated Kernel with Daemon Support + Tool CLI Commands)

## Executive Summary

This document describes the CLI command architecture implemented in LLMSpell v0.9.0 with integrated kernel architecture, full daemon support, and direct tool invocation capabilities. Phase 9 achieved a unified kernel architecture eliminating runtime isolation issues, while Phase 10 added Unix daemon mode with signal handling, service integration, and CLI-based tool management.

**Phase 9 Achievements**: Integrated kernel with global IO runtime, protocol/transport abstraction, DAP bridge for debugging, and comprehensive tracing.

**Phase 10 Achievements**:
- Daemon mode with double-fork technique
- Signal-to-message bridge (SIGTERM→shutdown_request)
- systemd/launchd service integration
- Enhanced logging infrastructure with rotation
- Consolidated state/sessions into kernel
- **Phase 10.22: Tool CLI Commands** - Direct tool invocation via kernel message protocol with list/info/invoke/search/test subcommands, enabling CLI access to 40+ tools for testing, debugging, and operations (Tasks 10.22.1-10.22.11)

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
  --config <FILE>    # Configuration file path (env: LLMSPELL_CONFIG)
  --output <FORMAT>  # Output format: text|json|pretty
  -h, --help        # Show contextual help
  -V, --version     # Show version information

Primary Commands:
  run <script>       # Execute script file
  exec <code>        # Execute inline code
  repl              # Interactive REPL
  debug <script>     # Debug with breakpoints

Subcommand Groups:
  kernel            # Kernel management (start/stop/status/connect/install-service)
  tool              # Tool management and direct invocation (Phase 10.22)
  state             # State management
  session           # Session management
  config            # Configuration management
  keys              # API key management
  backup            # Backup operations
  app               # Discover and run applications from filesystem
  version           # Display detailed version information
```

### 1.2 Command Tree

```
llmspell
├── run <script> [--engine] [--connect] [--stream] [--rag-profile] [-- args...]
├── exec <code> [--engine] [--connect] [--stream] [--rag-profile]
├── repl [--engine] [--connect] [--history] [--rag-profile]
├── debug <script> [--engine] [--connect] [--break-at] [--watch] [--step] [--port] [-- args...]
├── kernel
│   ├── start [--port] [--daemon] [--id] [--connection-file] [--log-file] [--pid-file]
│   │         [--idle-timeout] [--max-clients] [--log-rotate-size] [--log-rotate-count]
│   ├── stop [--id] [--pid-file] [--all] [--force] [--timeout] [--no-cleanup]
│   ├── status [--id] [--format] [--quiet] [--watch] [--interval]
│   ├── connect [address]
│   └── install-service [--service-type] [--system] [--name] [--port] [--id]
│                       [--log-file] [--pid-file] [--enable] [--start] [--force]
├── tool                                        # Phase 10.22
│   ├── list [--category] [--format]
│   ├── info <name> [--show-schema]
│   ├── invoke <name> --params <json> [--stream]
│   ├── search <keywords...> [--category]
│   └── test <name> [--verbose]
├── state
│   ├── show [key] [--scope] [--kernel|--connect]
│   ├── clear [key] [--scope] [--kernel|--connect]
│   ├── export <file> [--format] [--kernel|--connect]
│   └── import <file> [--merge] [--kernel|--connect]
├── session
│   ├── list [--detailed] [--kernel|--connect]
│   ├── show <id> [--kernel|--connect]
│   ├── replay <id> [--from-step] [--to-step] [--kernel|--connect]
│   └── delete <id> [--all] [--kernel|--connect]
├── config
│   ├── init [--output] [--force]
│   ├── validate [--file]
│   └── show [section] [--format]
├── keys
│   ├── add <provider> <key>
│   ├── list
│   └── remove <provider>
├── backup
│   ├── create [--output]
│   ├── restore <file>
│   ├── list
│   └── delete <id>
├── app [--search-path PATH]...
│   ├── list
│   ├── info <name>
│   ├── run <name> [-- app-args...]
│   └── search [--tag TAG] [--complexity LEVEL] [--agents N]
└── version [--verbose] [--component cli|kernel|bridge|all] [--short] [--client]
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
| `--kernel` | `--connect` | Connect to kernel | Clearer naming |

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
    --engine <ENGINE>      Script engine [default: lua, options: lua|javascript|python]
    --connect <ADDRESS>    Connect to kernel (e.g., "localhost:9555" or "/path/to/connection.json")
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
    llmspell run script.lua --connect localhost:9555  # Execute on remote kernel
```

### 3.2 exec - Execute Inline Code

```bash
llmspell exec <CODE> [OPTIONS]

OPTIONS:
    --engine <ENGINE>      Script engine [default: lua]
    --connect <ADDRESS>    Connect to kernel
    --stream              Enable streaming
    --rag-profile <NAME>   RAG configuration profile

EXAMPLES:
    llmspell exec "print('hello')"
    llmspell exec "console.log('test')" --engine javascript
    llmspell exec "agent.query('What is 2+2?')"
    llmspell exec "process_data()" --connect localhost:9555
```

### 3.3 repl - Interactive REPL

```bash
llmspell repl [OPTIONS]

OPTIONS:
    --engine <ENGINE>      Script engine [default: lua]
    --connect <ADDRESS>    Connect to kernel
    --history <FILE>       History file path
    --rag-profile <NAME>   RAG configuration profile

EXAMPLES:
    llmspell repl
    llmspell repl --engine javascript
    llmspell repl --connect localhost:9555
    llmspell repl --history ~/.llmspell_history
```

### 3.4 debug - Interactive Debugging

```bash
llmspell debug <script> [OPTIONS] [-- SCRIPT_ARGS...]

OPTIONS:
    --engine <ENGINE>      Script engine [default: lua]
    --connect <ADDRESS>    Connect to kernel [default: auto]
    --break-at <FILE:LINE> Set breakpoints (repeatable)
    --watch <EXPR>         Watch expressions (repeatable)
    --step                Start in step mode
    --port <PORT>          DAP server port for IDE attachment
    --rag-profile <NAME>   RAG configuration profile

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
    llmspell debug script.lua --connect localhost:9555  # Debug on remote kernel
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

START OPTIONS (Phase 10 daemon mode):
    --port <PORT>            Port to bind [default: 9555]
    --daemon                 Run as Unix daemon (detach from terminal)
    --id <ID>                Kernel ID (generated if not provided)
    --connection-file <PATH> Connection file path (for Jupyter discovery)
    --log-file <PATH>        Log file for daemon mode [default: ~/.llmspell/kernel.log]
    --pid-file <PATH>        PID file location [default: ~/.llmspell/kernel.pid]
    --idle-timeout <SEC>     Shutdown after idle time [default: 3600]
    --max-clients <N>        Maximum concurrent clients [default: 10]
    --log-rotate-size <BYTES>  Log rotation size in bytes
    --log-rotate-count <N>   Number of rotated log files to keep [default: 5]

STOP OPTIONS:
    --id <ID>                Kernel ID to stop
    --pid-file <PATH>        PID file path to identify kernel
    --all                    Stop all running kernels
    --force                  Force immediate termination (skip graceful shutdown)
    --timeout <SEC>          Timeout for graceful shutdown [default: 30]
    --no-cleanup             Don't clean up files after stopping

STATUS OPTIONS:
    --id <ID>                Kernel ID for detailed status (lists all if omitted)
    --format <FORMAT>        Output format: table|json|yaml|text [default: table]
    --quiet                  Show only kernel IDs
    --watch                  Watch mode - refresh continuously
    --interval <SEC>         Refresh interval for watch mode [default: 5]

INSTALL-SERVICE OPTIONS:
    --service-type <TYPE>    Service type: systemd|launchd|auto
    --system                 Install as system service (default: user service)
    --name <NAME>            Service name [default: llmspell-kernel]
    --port <PORT>            Port for kernel [default: 9555]
    --id <ID>                Kernel ID
    --log-file <PATH>        Log file path
    --pid-file <PATH>        PID file path
    --enable                 Enable service after installation
    --start                  Start service after installation
    --force                  Override if service already exists

EXAMPLES:
    # Simple start (foreground)
    llmspell kernel start --port 9555

    # Daemon mode with logging (Phase 10)
    llmspell kernel start --daemon --log-file /var/log/llmspell.log

    # Full daemon configuration with log rotation
    llmspell kernel start --daemon \
        --port 9555 \
        --pid-file /var/run/llmspell.pid \
        --log-file /var/log/llmspell/kernel.log \
        --log-rotate-size 104857600 \
        --log-rotate-count 10 \
        --idle-timeout 7200

    # Stop daemon using PID file
    llmspell kernel stop --pid-file /var/run/llmspell.pid

    # Stop all kernels with graceful shutdown
    llmspell kernel stop --all --timeout 60

    # Install as service (Phase 10)
    llmspell kernel install-service --type systemd --enable --start
    sudo systemctl status llmspell-kernel
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
llmspell kernel status --id abc123
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

# Watch mode for monitoring
llmspell kernel status --watch --interval 2
```

### 4.2 Tool Management (Phase 10.22)

**Architecture Note**: Tools execute in the kernel process, not the CLI. The CLI is a thin client that sends `tool_request` messages to the kernel via the message protocol. The kernel accesses the ComponentRegistry and executes tools with proper context, returning results via `tool_reply` messages.

```bash
llmspell tool <SUBCOMMAND>

SUBCOMMANDS:
    list       List available tools with category filtering
    info       Show detailed tool information and schema
    invoke     Execute tool directly with JSON parameters
    search     Search tools by keywords and capabilities
    test       Test tool with built-in example cases

LIST OPTIONS:
    --category <CATEGORY>  Filter by tool category (filesystem, web, data, llm, etc.)
    --format <FORMAT>      Output format (overrides global) [text|json|pretty]

INFO OPTIONS:
    --show-schema          Show detailed input/output schema

INVOKE OPTIONS:
    --params <JSON>        Parameters as JSON object (required)
    --stream              Enable streaming output

SEARCH OPTIONS:
    --category <CATEGORY>  Filter by tool category

TEST OPTIONS:
    --verbose             Show detailed test output

ARCHITECTURE:
    - Tools execute IN KERNEL, not CLI (Phase 10.22 design decision)
    - CLI sends tool_request messages to kernel via transport
    - Kernel accesses ComponentRegistry and executes tools
    - Results returned via tool_reply messages
    - Same protocol works for embedded and remote kernels
    - InProcessTransport for embedded, ZeroMQ for remote

EXAMPLES:
    # List all 40+ tools
    llmspell tool list

    # Filter by category
    llmspell tool list --category filesystem

    # Show tool details with schema
    llmspell tool info calculator --show-schema

    # Execute tool with parameters
    llmspell tool invoke calculator --params '{"expression":"sqrt(16)"}'

    # Complex tool invocation with streaming
    llmspell tool invoke web_scraper --params '{"url":"example.com"}' --stream

    # Search for tools
    llmspell tool search "file" "web"          # Multi-keyword search
    llmspell tool search "json" --category data  # Search with category filter

    # Test tool with example cases
    llmspell tool test calculator --verbose

MESSAGE FLOW (Phase 10.22):
    1. CLI parses tool command and parameters
    2. CLI creates tool_request message with command/params
    3. CLI sends via kernel handle (embedded) or connection (remote)
    4. Kernel receives on shell channel
    5. Kernel.handle_tool_request() processes request
    6. Kernel accesses script_executor.component_registry()
    7. ComponentRegistry looks up tool by name
    8. Tool.execute() runs with AgentInput/AgentOutput
    9. Kernel sends tool_reply with results
    10. CLI receives and formats output

CODE REFERENCES:
    CLI: llmspell-cli/src/cli.rs:440-638 (ToolCommands enum)
    Handler: llmspell-cli/src/commands/tool.rs (handle_tool_command)
    Kernel: llmspell-kernel/src/execution/integrated.rs:1946-2463 (handle_tool_request)
    Registry: llmspell-core/src/traits/script_executor.rs (component_registry method)
```

### 4.3 State Management

```bash
llmspell state <SUBCOMMAND> [OPTIONS]

SUBCOMMANDS:
    show      Display persisted state
    clear     Clear state by scope
    export    Export state to JSON/TOML
    import    Import state from JSON/TOML

OPTIONS (for all subcommands):
    --connect <ADDRESS>     Connect to kernel (e.g., "localhost:9555")
    --kernel <ID>           Use specific kernel by ID
    --scope <SCOPE>         State scope: global|session|workflow|component
    --key <KEY>             Specific state key (for show/clear)

EXAMPLES:
    # Show state from running kernel
    llmspell state show --connect localhost:9555 --scope global

    # Show state from specific kernel by ID
    llmspell state show --kernel abc123 --scope session

    # Clear specific key
    llmspell state clear --key user.preferences

    # Export/import state
    llmspell state export state.json --connect localhost:9555
    llmspell state import state.json --kernel abc123 --merge
```

### 4.4 Session Management

```bash
llmspell session <SUBCOMMAND> [OPTIONS]

SUBCOMMANDS:
    list      List all sessions
    show      Show session details
    replay    Replay a session
    delete    Delete a session

OPTIONS:
    --connect <ADDRESS>     Connect to kernel
    --kernel <ID>           Use specific kernel by ID
    --detailed              Show detailed session information (list only)
    --from-step <N>         Start from specific step (replay only)
    --to-step <N>           Stop at specific step (replay only)
    --all                   Delete all sessions (delete only)

EXAMPLES:
    # List sessions from specific kernel
    llmspell session list --kernel abc123 --detailed

    # Show session details
    llmspell session show sess_xyz --kernel abc123

    # Replay session with step range
    llmspell session replay sess_xyz --from-step 5 --to-step 10

    # Delete old session
    llmspell session delete old_session --kernel abc123
```

### 4.5 Configuration Management

```bash
llmspell config <SUBCOMMAND>

SUBCOMMANDS:
    init      Initialize configuration
    validate  Validate configuration
    show      Show configuration

INIT OPTIONS:
    --output <PATH>         Output path for configuration file [default: llmspell.toml]
    --force                 Force overwrite existing file

VALIDATE OPTIONS:
    --file <PATH>           Configuration file to validate

SHOW OPTIONS:
    --format <FORMAT>       Output format: toml|json [default: toml]

EXAMPLES:
    llmspell config init --output custom.toml --force
    llmspell config validate --file production.toml
    llmspell config show rag --format json
```

### 4.6 Application Management

```bash
llmspell app [--search-path PATH]... <SUBCOMMAND>

GLOBAL OPTIONS:
    --search-path <PATH>    Additional search paths for applications (repeatable)

SUBCOMMANDS:
    list      List all available applications
    info      Show detailed information about an application
    run       Run an application
    search    Search applications by criteria

SEARCH OPTIONS:
    --tag <TAG>             Search by tag
    --complexity <LEVEL>    Search by complexity level
    --agents <COUNT>        Search by number of agents

EXAMPLES:
    # List all apps
    llmspell app list

    # List apps with additional search paths
    llmspell app --search-path /opt/llmspell-apps list

    # Show app details
    llmspell app info file-organizer

    # Run app with arguments
    llmspell app run research-collector -- --verbose

    # Search apps
    llmspell app search --tag productivity
    llmspell app search --complexity Simple
```

### 4.7 Version Information

```bash
llmspell version [OPTIONS]

OPTIONS:
    --verbose               Show verbose version information
    --component <COMPONENT> Show version of specific component
                           [options: cli|kernel|bridge|all]
    --short                Show short commit hash only
    --client               Show client version only (useful for scripts)

EXAMPLES:
    llmspell version                    # Show version information
    llmspell version --verbose          # Show detailed build information
    llmspell version --component kernel # Show kernel version only
    llmspell version --component all    # Show all component versions
    llmspell version --short            # Show just the version number
    llmspell version --client           # Show client version only
    llmspell version --output json      # Output as JSON

VERSION OUTPUT INCLUDES:
    - Package version (from Cargo.toml)
    - Git commit hash (full and short)
    - Git branch name and commit date
    - Working tree dirty state
    - Build timestamp and profile (debug/release)
    - Host and target triple information
    - Rust compiler version
    - Enabled feature flags
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

Daemon mode includes comprehensive logging with rotation:

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

**Log Rotation Flags** (Phase 10):
```bash
llmspell kernel start --daemon \
    --log-file /var/log/llmspell.log \
    --log-rotate-size 104857600 \    # 100MB
    --log-rotate-count 10             # Keep 10 old files
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

**Note**: The CLI does not have a `kernel signal` subcommand. Signal handling is done via standard Unix `kill` command as shown above.

---

## 7. Dual-Mode Design

### 7.1 Online vs Offline Operations

Many commands support both kernel and config contexts:

#### Online Mode (`--connect` or `--kernel`)
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

### 7.2 Resolution Order

```rust
// Command resolution logic
async fn resolve_context(connect: Option<String>, kernel: Option<String>) -> Context {
    match (connect, kernel) {
        (Some(addr), _) => Context::Kernel(connect_to_kernel(addr).await?),
        (_, Some(id)) => Context::Kernel(find_kernel_by_id(id).await?),
        (None, None) => {
            // Auto mode
            if let Some(k) = find_running_kernel().await? {
                Context::Kernel(k)
            } else {
                Context::Embedded(start_embedded_kernel().await?)
            }
        }
    }
}
```

### 7.3 Usage Examples

```bash
# Online - uses running kernel
llmspell state show --connect localhost:9555
llmspell session list --kernel abc123

# Embedded - starts embedded kernel
llmspell state show    # No kernel specified, starts embedded

# Tool commands always use kernel (embedded or connected)
llmspell tool list                    # Uses embedded kernel
llmspell tool list --connect localhost:9555  # Uses remote kernel (future)
```

---

## 8. Help System

### 8.1 Contextual Help Behavior

The `-h/--help` flag provides contextual help based on where it's used:

```bash
# Global help - shows all commands overview
llmspell --help
llmspell -h

# Command help - shows specific command details
llmspell run --help
llmspell exec -h
llmspell tool --help

# Subcommand group help - shows available subcommands
llmspell kernel --help
llmspell state -h
llmspell tool -h

# Specific subcommand help - shows detailed usage
llmspell kernel start --help
llmspell state show -h
llmspell tool invoke --help
```

### 8.2 Help Precedence Rules

1. **Help flag terminates parsing** - When `-h/--help` is encountered, show help and exit
2. **Position matters** - Help shows context for the command level where it appears
3. **Ignores other flags** - `llmspell run --engine js --help` still shows run help
4. **Works with partial commands** - `llmspell kernel` (without subcommand) shows kernel help

### 8.3 Help Output Structure

#### Global Help Format
```
LLMSpell - Scriptable LLM interactions

USAGE:
    llmspell [GLOBAL OPTIONS] <COMMAND> [ARGS]

GLOBAL OPTIONS:
    --trace <LEVEL>     Set trace level [default: warn]
    --config <FILE>     Config file path [env: LLMSPELL_CONFIG]
    --output <FORMAT>   Output format [default: text]
    -h, --help         Print help information
    -V, --version      Print version information

COMMANDS:
    run       Execute a script file
    exec      Execute inline code
    repl      Start interactive REPL
    debug     Debug a script with breakpoints
    kernel    Manage kernel processes
    tool      Tool management and direct invocation
    state     Manage persistent state
    session   Manage sessions
    config    Configuration management
    keys      API key management
    backup    Backup and restore operations
    app       Manage and run applications
    version   Display version information

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
    --connect <ADDRESS>    Connect to kernel
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
| N/A | `llmspell tool <subcommand>` | New in Phase 10.22 |

### 9.2 Flag Removals

All these flags have been removed or renamed:

- ❌ `--debug` → Use `--trace` for logging or `debug` command
- ❌ `--verbose` → Use `--trace info`
- ❌ `--debug-level` → Use `--trace <level>`
- ❌ `--debug-format` → Removed (use `--output`)
- ❌ `--debug-modules` → Move to config file
- ❌ `--debug-perf` → Move to config file
- ❌ `--rag`, `--no-rag`, `--rag-config`, `--rag-dims`, `--rag-backend` → Use `--rag-profile`
- ❌ `--kernel` → Renamed to `--connect` for clarity

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

# NEW in Phase 10.22 (tool commands)
llmspell tool list
llmspell tool invoke calculator --params '{"expression":"2+2"}'
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
    #[arg(long, global = true, value_enum, default_value = "warn")]
    pub trace: TraceLevel,

    /// Configuration file
    #[arg(short = 'c', long, global = true, env = "LLMSPELL_CONFIG")]
    pub config: Option<PathBuf>,

    /// Output format
    #[arg(long, global = true, value_enum, default_value = "text")]
    pub output: OutputFormat,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Execute a script file
    Run {
        script: PathBuf,
        #[arg(long, value_enum, default_value = "lua")]
        engine: ScriptEngine,
        #[arg(long)]
        connect: Option<String>,
        #[arg(long)]
        stream: bool,
        #[arg(long)]
        rag_profile: Option<String>,
        #[arg(last = true)]
        args: Vec<String>,
    },

    /// Tool management and direct invocation (Phase 10.22)
    Tool {
        #[command(subcommand)]
        command: ToolCommands,
        #[arg(long, default_value = "local", hide = true)]
        source: String,
    },

    /// Manage kernel servers
    Kernel {
        #[command(subcommand)]
        command: KernelCommands,
    },

    /// Display version information
    Version(VersionCommand),

    // ... other commands
}

#[derive(Subcommand)]
pub enum ToolCommands {
    List {
        #[arg(long)]
        category: Option<String>,
        #[arg(long)]
        format: Option<OutputFormat>,
    },
    Info {
        name: String,
        #[arg(long)]
        show_schema: bool,
    },
    Invoke {
        name: String,
        #[arg(long, value_parser = parse_json_value)]
        params: serde_json::Value,
        #[arg(long)]
        stream: bool,
    },
    Search {
        query: Vec<String>,
        #[arg(long)]
        category: Option<String>,
    },
    Test {
        name: String,
        #[arg(long)]
        verbose: bool,
    },
}
```

### 10.2 Command Handler

```rust
// llmspell-cli/src/commands/mod.rs
pub async fn execute_command(
    command: Commands,
    runtime_config: LLMSpellConfig,
    output_format: OutputFormat,
) -> Result<()> {
    match command {
        Commands::Run { script, engine, connect, stream, rag_profile, args } => {
            let mut config = runtime_config;
            apply_rag_profile(&mut config, rag_profile).await?;
            let context = ExecutionContext::resolve(connect, None, None, config).await?;
            run::execute_script_file(script, engine, context, stream, args, output_format).await
        }

        Commands::Tool { command, source } => {
            tool::handle_tool_command(command, source, runtime_config, output_format).await
        }

        Commands::Kernel { command } => {
            kernel::handle_kernel_command(command, runtime_config, output_format).await
        }

        // ... other handlers
    }
}
```

### 10.3 Tool Command Implementation (Phase 10.22)

```rust
// llmspell-cli/src/commands/tool.rs
pub async fn handle_tool_command(
    command: ToolCommands,
    source: String,
    config: LLMSpellConfig,
    output_format: OutputFormat,
) -> Result<()> {
    // Create embedded kernel with script executor
    let context = ExecutionContext::resolve(None, None, None, config).await?;

    match command {
        ToolCommands::List { category, format } => {
            // Send tool_request message to kernel
            let request = json!({
                "msg_type": "tool_request",
                "content": {
                    "command": "list",
                    "category": category,
                }
            });

            let response = context.kernel_handle.send_message(request).await?;
            let tools = response["content"]["tools"].as_array()?;

            // Format and display results
            format_tool_list(tools, format.unwrap_or(output_format))
        }

        ToolCommands::Invoke { name, params, stream } => {
            // Send tool invocation request
            let request = json!({
                "msg_type": "tool_request",
                "content": {
                    "command": "invoke",
                    "name": name,
                    "params": params,
                    "stream": stream,
                }
            });

            let response = context.kernel_handle.send_message(request).await?;
            display_tool_result(response, output_format)
        }

        // ... other subcommands
    }
}
```

### 10.4 Kernel Tool Handler (Phase 10.22)

```rust
// llmspell-kernel/src/execution/integrated.rs
async fn handle_tool_request(&mut self, message: HashMap<String, Value>) -> Result<()> {
    let content = message.get("content").ok_or(anyhow!("No content"))?;
    let command = content.get("command").ok_or(anyhow!("No command"))?;

    match command.as_str() {
        Some("list") => {
            // Access ComponentRegistry via script_executor
            let registry = self.script_executor
                .component_registry()
                .ok_or_else(|| anyhow!("No ComponentRegistry available"))?;

            let category = content.get("category").and_then(|c| c.as_str());
            let tools = registry.list_components("tool");

            // Filter by category if specified
            let filtered_tools = if let Some(cat) = category {
                tools.into_iter()
                    .filter(|id| registry.get_metadata(id)
                        .map(|m| m.category == cat)
                        .unwrap_or(false))
                    .collect()
            } else {
                tools
            };

            // Send tool_reply with results
            self.send_tool_reply(json!({
                "tools": filtered_tools,
            })).await
        }

        Some("invoke") => {
            let tool_name = content["name"].as_str()?;
            let params = &content["params"];

            let registry = self.script_executor.component_registry()?;
            let tool = registry.get_component(tool_name)?;

            // Execute tool with AgentInput/AgentOutput
            let result = tool.execute(params).await?;

            self.send_tool_reply(json!({
                "result": result,
            })).await
        }

        // ... other tool commands
    }
}
```

### 10.5 Script Argument Handling

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
                result.insert(key, args[i + 1].clone());
                i += 2;
            } else {
                result.insert(key, "true".to_string());
                i += 1;
            }
        } else {
            result.insert(i.to_string(), arg.clone());
            i += 1;
        }
    }
    result
}
```

---

## Architectural Decisions

### RAG Operations
- **NOT implemented as standalone CLI command** - RAG operations are script-context operations
- **Access via script API**: Use `RAG.*` methods within Lua/JavaScript/Python scripts
- **Configuration via flag**: `--rag-profile` flag available on execution commands (run, exec, repl, debug)
- **Rationale**: RAG operations require script context and state management that doesn't make sense as standalone CLI operations

### Tools Management (Phase 10.22 UPDATED)
- **✅ IMPLEMENTED as CLI command** - Direct tool access for testing, debugging, and operations
- **Architecture**: Tools execute in kernel, CLI is thin client using message protocol
- **Access methods**:
  - **CLI**: `llmspell tool <subcommand>` for direct invocation (Phase 10.22)
  - **Script API**: `Tool.*` methods within scripts for programmatic access
- **Auto-discovery**: Tools are automatically discovered and loaded at runtime
- **Rationale**: CLI tool commands provide essential developer experience for:
  - Rapid testing and debugging of individual tools
  - Production operations and troubleshooting
  - Tool discovery and exploration of 40+ available tools
  - Health checks and manual interventions
  - Foundation for future MCP (Phase 12) and A2A (Phase 18) protocols

### Info Command
- **DELETED** - The info command showing engine availability was vestigial code
- **Engine errors**: Already handled by `--engine` flag with clear error messages
- **Rationale**: Trivial information not worth maintenance burden

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
8. **Tool CLI Commands (10.22)** ✅ - Direct tool invocation via kernel protocol
   - list, info, invoke, search, test subcommands
   - CLI→Kernel message protocol with InProcessTransport
   - ComponentRegistry integration for 40+ tools
   - Proper error handling and output formatting
   - Foundation for future MCP/A2A integration

The architecture maintains backward compatibility for basic usage while providing robust service features for production deployments and comprehensive tool management for developer workflows.

---

*This document reflects the completed CLI command architecture from Phase 10 implementation including integrated kernel, daemon mode, full service integration, and Phase 10.22 tool command capabilities.*
