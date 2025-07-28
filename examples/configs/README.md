# Example Configuration Files

This directory contains configuration files for running different rs-llmspell examples.

## State Persistence Configurations

### `state-enabled.toml`
Basic state persistence configuration with memory backend.
- Enables State global (save, load, delete)
- Uses in-memory backend (data lost on exit)
- No migration or backup features
- Use for: Basic state examples

### `migration-enabled.toml`
State persistence with migration support.
- Everything from state-enabled.toml
- Enables migration API (State.migrate, State.schema_versions)
- Enables automatic backup on migration
- Use for: Migration examples

### `backup-enabled.toml`
State persistence with backup functionality.
- Everything from state-enabled.toml
- Enables backup API (State.create_backup, State.restore_backup)
- Configures backup directory and retention
- Supports incremental backups with compression
- Use for: Backup and recovery examples

## General Configurations

### `minimal.toml`
Minimal configuration for basic script execution.
- No providers configured
- No state persistence
- Use for: Simple examples that don't need LLM or state

### `llmspell.toml`
Full-featured configuration template.
- Shows all available configuration options
- Includes provider configurations (need API keys)
- Use as: Template for creating custom configurations

## Usage

Run examples with a specific config:

```bash
# Using the binary
./target/debug/llmspell -c examples/configs/state-enabled.toml run examples/lua/state/basic_persistence.lua

# Using cargo
cargo run -- -c examples/configs/state-enabled.toml run examples/lua/state/basic_persistence.lua
```

## Important Notes

1. All state examples use **memory backend** by default - data is not persisted between runs
2. For production use, configure a persistent backend (file, redis, etc.)
3. Provider configurations require valid API keys to work
4. Each example file has comments indicating which config to use