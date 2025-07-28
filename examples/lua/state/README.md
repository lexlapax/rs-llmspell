# State Persistence Examples

This directory contains examples demonstrating state persistence functionality in rs-llmspell.

## Configuration Files

State persistence requires specific configuration. Use these config files from `examples/configs/`:

- **`state-enabled.toml`** - Basic state persistence with memory backend
- **`migration-enabled.toml`** - State persistence with migration support
- **`backup-enabled.toml`** - State persistence with backup functionality

## Running Examples

All examples include instructions at the top of each file. The general pattern is:

```bash
# From the project root:
./target/debug/llmspell -c examples/configs/state-enabled.toml run examples/lua/state/basic_persistence.lua

# Or using cargo:
cargo run -- -c examples/configs/state-enabled.toml run examples/lua/state/basic_persistence.lua
```

## Examples Overview

### Basic Examples
- `basic_persistence.lua` - Core State API: save, load, delete
- `scope_isolation.lua` - How scopes provide isolation (global, system, agent:*, workflow:*)
- `agent_state.lua` - Agent-specific state management patterns

### Advanced Examples
- `list_operations.lua` - Working without State.list_keys() by maintaining key registries
- `state_migration.lua` - Schema evolution and migration patterns

### Test Utilities
- `test_state_api.lua` - Check which State functions are available at runtime
- `basic_persistence_working.lua` - Can be deleted (duplicate of fixed basic_persistence.lua)

## Important Notes

1. **State.list_keys() is not available at runtime** - Examples show how to work around this by maintaining explicit key registries

2. **API Methods**:
   - ✅ Use: `State.save()`, `State.load()`, `State.delete()`
   - ❌ Don't use: `State.set()`, `State.get()`, `State.list()`

3. **Scopes** provide complete isolation:
   - `"global"` - Application-wide state
   - `"system"` - System configuration
   - `"agent:name"` - Agent-specific state
   - `"workflow:name"` - Workflow-specific state

4. **Migration and Backup APIs** require specific configuration:
   - Migration: Set `migration_enabled = true` in config
   - Backup: Set `backup_enabled = true` in config