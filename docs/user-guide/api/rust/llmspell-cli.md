# llmspell-cli

**Command-line interface application** ⚠️ **BREAKING CHANGES in v0.9.0**

**🔗 Navigation**: [← Rust API](README.md) | [Crate Docs](https://docs.rs/llmspell-cli) | [Source](../../../../llmspell-cli)

---

## Overview

`llmspell-cli` provides the command-line interface for LLMSpell, including script execution, REPL mode, kernel management, debug commands, and configuration management. **Phase 9 introduced major breaking changes to the command structure.**

**Key Features:**
- 🖥️ Interactive REPL with debug commands
- 📜 Script execution through kernel
- 🔧 Kernel management (start/stop/status)
- 🐛 Interactive debugging with breakpoints
- ⚙️ Configuration and state management
- 🎨 Rich output formatting
- 📊 Session management and replay
- 🚀 Performance profiling

## CLI Commands (v0.9.0)

### Core Commands

```bash
# Execute inline code
llmspell exec 'print("Hello, LLMSpell!")'

# Run script file with arguments (use -- separator)
llmspell run script.lua -- arg1 arg2

# Interactive REPL
llmspell repl

# Debug a script with breakpoints
llmspell debug script.lua --break-at script.lua:10 --break-at utils.lua:25
```

### Kernel Management (NEW)

```bash
# Start kernel server
llmspell kernel start --port 9555 --daemon

# Check kernel status
llmspell kernel status

# Stop kernel
llmspell kernel stop

# Connect to existing kernel
llmspell kernel connect localhost:9555
```

### State Management

```bash
# Show state
llmspell state show
llmspell state show mykey

# Clear state
llmspell state clear
llmspell state clear mykey

# Export/import state
llmspell state export backup.json
llmspell state import backup.json --merge
```

### Session Management

```bash
# List sessions
llmspell session list --detailed

# Replay session
llmspell session replay session-123

# Show session info
llmspell session show session-123

# Clean old sessions
llmspell session clean --days 30
```

### Configuration

```bash
# Show configuration
llmspell config show
llmspell config show --section providers

# Set configuration values
llmspell config set providers.openai.model gpt-4

# Validate configuration
llmspell config validate

# Reset to defaults
llmspell config reset --confirm
```

## CLI Structure (v0.9.0)

```rust
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about)]
pub struct Cli {
    /// Script engine to use
    #[arg(long, value_enum, default_value = "lua")]
    pub engine: ScriptEngine,
    
    /// Configuration file path
    #[arg(short, long)]
    pub config: Option<PathBuf>,
    
    /// Set trace level (off|error|warn|info|debug|trace)
    #[arg(long, global = true)]  // Replaces --debug flag
    pub trace: Option<TraceLevel>,
    
    /// Output format
    #[arg(long, value_enum, default_value = "text")]
    pub output: OutputFormat,
    
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Execute inline code
    Exec { 
        code: String,
        #[arg(long)]
        connect: Option<String>,  // Connect to kernel
        #[arg(long)]
        rag_profile: Option<String>,  // Replaces 5 RAG flags
    },
    
    /// Run script file
    Run { 
        script: PathBuf,
        #[arg(long)]
        rag_profile: Option<String>,
        #[arg(last = true)]  // Use -- separator
        args: Vec<String>,
    },
    
    /// Interactive REPL
    Repl {
        #[arg(long)]
        connect: Option<String>,
        #[arg(long)]
        history: Option<PathBuf>,
    },
    
    /// Debug a script (NEW)
    Debug {
        script: PathBuf,
        #[arg(long)]
        break_at: Vec<String>,  // file:line format
        #[arg(long)]
        port: Option<u16>,      // DAP server port
    },
    
    /// Kernel management (NEW)
    Kernel {
        #[command(subcommand)]
        command: KernelCommands,
    },
    
    /// State management
    State {
        #[command(subcommand)]
        command: StateCommands,
    },
    
    /// Session management
    Session {
        #[command(subcommand)]
        command: SessionCommands,
    },
    
    /// Configuration
    Config {
        #[command(subcommand)]
        command: ConfigCommands,
    },
}
```

## Breaking Changes in v0.9.0

### Flag Changes

| Old Flag | New Flag | Notes |
|----------|----------|-------||
| `--debug` | `--trace` | Debug now means interactive debugging |
| `--rag` | `--rag-profile` | Single profile replaces 5 flags |
| `--rag-config` | `--rag-profile` | Merged into profile |
| `--rag-dims` | `--rag-profile` | Configured in profile |
| `--rag-backend` | `--rag-profile` | Configured in profile |
| `--no-rag` | (removed) | Disable in config |

### Command Structure Changes

```bash
# Old (flat commands)
llmspell exec-code "print('hi')"
llmspell run-script file.lua
llmspell show-config

# New (subcommands)
llmspell exec "print('hi')"
llmspell run file.lua
llmspell config show
```

### Script Arguments

```bash
# Old (direct arguments)
llmspell run script.lua arg1 arg2

# New (use -- separator)
llmspell run script.lua -- arg1 arg2
```

## REPL Mode with Debug Commands

### REPL Debug Commands (NEW in v0.9.0)

```bash
# Start REPL
llmspell repl

# Debug commands in REPL
.break script.lua 10        # Set breakpoint
.break script.lua 15 x > 5  # Conditional breakpoint
.step                       # Step to next line
.continue                   # Continue execution
.locals                     # Show local variables
.globals                    # Show global variables
.upvalues                   # Show upvalues
.stack                      # Show call stack
.watch x * 2                # Add watch expression
.help                       # Show all commands
.exit                       # Exit REPL
```

### REPL Configuration

```rust
use llmspell_repl::{ReplSession, ReplConfig};
use llmspell_kernel::JupyterKernel;

// Create kernel connection
let kernel = JupyterKernel::spawn_embedded().await?;

// Configure REPL
let config = ReplConfig {
    enable_performance_monitoring: true,
    enable_debug_commands: true,
};

// Create and run REPL session
let mut session = ReplSession::new(
    Box::new(kernel),
    config
).await?;

session.run().await?;
```

## Output Formatting

```rust
use llmspell_cli::output::{OutputFormatter, OutputStyle};

let formatter = OutputFormatter::new(OutputStyle::Rich);

// Format agent response
formatter.format_agent_response(&response)?;

// Format table data
formatter.format_table(headers, rows)?;

// Progress indicator
let progress = formatter.progress_bar(100);
for i in 0..100 {
    progress.inc(1);
    // work...
}
progress.finish();
```

## Script Arguments (Updated in v0.9.0)

```lua
-- script.lua
print("Script arguments:")
for i, arg in ipairs(ARGS) do
    print(i, arg)
end

-- Named arguments
print("Input file:", ARGS.input)
print("Output file:", ARGS.output)
```

```bash
# NEW: Use -- separator for script arguments
llmspell run script.lua -- file1.txt file2.txt --input data.json --output result.json

# With RAG profile
llmspell run script.lua --rag-profile production -- --input data.json
```

## Debug and Trace Modes (v0.9.0)

### Interactive Debugging (NEW)

```bash
# Debug a script with breakpoints
llmspell debug script.lua --break-at script.lua:10 --break-at utils.lua:25

# Debug with DAP server for IDE integration
llmspell debug script.lua --port 5678

# Debug with script arguments
llmspell debug script.lua --break-at main.lua:15 -- arg1 arg2
```

### Trace Levels (Replaces --debug)

```bash
# Set trace level
llmspell --trace debug run script.lua
llmspell --trace trace exec 'Agent.list()'  # Most verbose

# Available levels: off, error, warn, info, debug, trace
llmspell --trace info run script.lua

# Performance profiling
LLMSPELL_PROFILE=1 llmspell run heavy_script.lua
```

## Kernel Integration (NEW in v0.9.0)

All script execution now goes through the kernel architecture:

```rust
use llmspell_cli::kernel_client::{KernelConnection, EmbeddedKernel};
use llmspell_config::LLMSpellConfig;
use std::sync::Arc;

// Scripts automatically spawn embedded kernel if needed
let config = Arc::new(LLMSpellConfig::default());
let kernel = EmbeddedKernel::new(config).await?;

// Execute through kernel (automatic in CLI)
let result = kernel.execute("print('Hello')").await?;

// Kernel provides:
// - State persistence across executions
// - Debug support with ExecutionManager
// - ~1ms overhead after first execution
// - Automatic cleanup on exit
```

## Integration Example

```rust
use llmspell_cli::{Cli, run_cli};
use clap::Parser;

#[tokio::main]
async fn main() -> Result<()> {
    // Parse CLI arguments
    let cli = Cli::parse();
    
    // Initialize tracing based on --trace flag
    if let Some(trace_level) = cli.trace {
        tracing_subscriber::fmt()
            .with_max_level(trace_level.into())
            .init();
    }
    
    // Run CLI (kernel auto-spawns if needed)
    run_cli(cli).await
}
```

## Migration Guide from v0.8.x

### Update Commands
```bash
# Old
llmspell exec-code "print('test')"
llmspell run-script file.lua arg1
llmspell show-config

# New
llmspell exec "print('test')"
llmspell run file.lua -- arg1
llmspell config show
```

### Update Flags
```bash
# Old
llmspell --debug run script.lua
llmspell --rag --rag-dims 1536 run script.lua

# New
llmspell --trace debug run script.lua
llmspell --rag-profile production run script.lua
```

### Update Debug Usage
```bash
# Old (no interactive debugging)
llmspell --debug run script.lua

# New (interactive debugging)
llmspell debug script.lua --break-at script.lua:10
```

## Related Documentation

- [Getting Started](../../getting-started.md) - CLI usage guide
- [llmspell-kernel](llmspell-kernel.md) - Kernel architecture (NEW)
- [llmspell-repl](llmspell-repl.md) - REPL implementation (NEW)
- [llmspell-debug](llmspell-debug.md) - Debug infrastructure (NEW)
- [llmspell-bridge](llmspell-bridge.md) - Script execution engine
- [llmspell-config](llmspell-config.md) - Configuration management
- [Migration Guide](../../migration-0.9.0.md) - Full migration details