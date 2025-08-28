# llmspell-cli

**Command-line interface application**

**🔗 Navigation**: [← Rust API](README.md) | [Crate Docs](https://docs.rs/llmspell-cli) | [Source](../../../../llmspell-cli)

---

## Overview

`llmspell-cli` provides the command-line interface for LLMSpell, including script execution, REPL mode, configuration management, and output formatting.

**Key Features:**
- 🖥️ Interactive REPL
- 📜 Script execution
- ⚙️ Configuration management
- 🎨 Rich output formatting
- 📊 Progress indicators
- 🔍 Debug modes
- 📝 Command history
- 🚀 Performance profiling

## CLI Commands

```bash
# Execute inline code
llmspell exec 'print("Hello, LLMSpell!")'

# Run script file
llmspell run script.lua

# Interactive REPL
llmspell repl

# Configuration
llmspell config show
llmspell config set providers.openai.model gpt-4
llmspell config validate

# List available tools
llmspell tools list
llmspell tools search json

# Agent operations
llmspell agent list
llmspell agent create --name assistant --model gpt-4
```

## CLI Structure

```rust
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
    
    #[arg(short, long, global = true)]
    pub config: Option<PathBuf>,
    
    #[arg(short, long, global = true)]
    pub verbose: bool,
    
    #[arg(long, global = true)]
    pub debug: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Execute inline code
    Exec { code: String },
    
    /// Run script file
    Run { 
        script: PathBuf,
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },
    
    /// Start interactive REPL
    Repl,
    
    /// Configuration management
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },
    
    /// Tool management
    Tools {
        #[command(subcommand)]
        action: ToolAction,
    },
}
```

## REPL Mode

```rust
use llmspell_cli::repl::Repl;

let repl = Repl::new(ReplConfig {
    prompt: "llmspell> ",
    history_file: Some(".llmspell_history"),
    multiline: true,
    syntax_highlighting: true,
})?;

repl.run().await?;
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

## Script Arguments

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
# Pass arguments
llmspell run script.lua file1.txt file2.txt --input data.json --output result.json
```

## Debug Mode

```bash
# Enable debug output
llmspell --debug run script.lua

# Verbose logging
llmspell --verbose exec 'Agent.list()'

# Performance profiling
LLMSPELL_PROFILE=1 llmspell run heavy_script.lua
```

## Integration Example

```rust
use llmspell_cli::{Cli, run_cli};

#[tokio::main]
async fn main() -> Result<()> {
    // Parse CLI arguments
    let cli = Cli::parse();
    
    // Initialize logging
    if cli.debug {
        env_logger::init_from_env(
            env_logger::Env::default()
                .default_filter_or("debug")
        );
    }
    
    // Run CLI
    run_cli(cli).await
}
```

## Related Documentation

- [Getting Started](../../getting-started.md) - CLI usage guide
- [llmspell-bridge](llmspell-bridge.md) - Script execution engine
- [llmspell-config](llmspell-config.md) - Configuration management