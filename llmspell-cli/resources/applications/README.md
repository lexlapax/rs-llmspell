# Embedded Example Applications

These applications are embedded directly into the `llmspell` binary at compile time for zero-configuration distribution.

## Available Applications (7 Total)

### Universal Layer (2-3 agents)
- **file-organizer** - Organize messy files with AI categorization (3 agents)
- **research-collector** - Research any topic thoroughly (2 agents)

### Power User Layer (4 agents)
- **content-creator** - Create content efficiently with AI assistance (4 agents)

### Business Layer (5 agents)
- **communication-manager** - Manage business communications (5 agents)

### Professional Layer (7-8 agents)
- **process-orchestrator** - Orchestrate complex processes (8 agents)
- **code-review-assistant** - Review code for quality and security (7 agents)

### Expert Layer (20 agents)
- **webapp-creator** - Create complete web applications (20 agents)

## Usage

These applications are accessed through the CLI:

```bash
# List all available applications
llmspell apps list

# Run a specific application
llmspell apps file-organizer
llmspell apps research-collector
llmspell apps content-creator
# ... etc
```

## Architecture

Each application demonstrates:
- **Progressive complexity** - From 2 agents (Universal) to 20 agents (Expert)
- **Real-world problems** - No toy examples, all solve genuine user needs
- **Crate showcase** - Each layer introduces more llmspell crates
- **Learning progression** - Natural skill building from simple to complex

## Implementation

These files are embedded using Rust's `include_str!` macro in `llmspell-cli/src/embedded_resources.rs`:

```rust
lua_script: include_str!("../resources/applications/file-organizer/main.lua"),
config: include_str!("../resources/applications/file-organizer/config.toml"),
```

At runtime, they're extracted to a temporary directory and executed.

## Important Notes

1. **These are copies** - Original development versions are in `examples/script-users/applications/`
2. **Binary embedding** - Compiled into the executable, no external files needed
3. **Zero configuration** - Users don't need to set up paths or find files
4. **Single distribution** - One binary contains everything

## Synchronization

If updating these applications:
1. Edit in `examples/script-users/applications/` (development location)
2. Copy changes here to `llmspell-cli/resources/applications/`
3. Rebuild CLI to embed updated versions

This ensures the embedded versions stay in sync with development.