# Migration Examples

This directory contains examples demonstrating the state migration functionality in rs-llmspell.

## Files

- `schema_migration.lua` - Basic Lua example showing migration API usage
- `test_migration_example.rs` - Integration test with proper migration initialization

## Current Limitation

The migration functionality requires the StateGlobal to be initialized with:
1. StateManager instance
2. MigrationEngine instance  
3. SchemaRegistry instance

Currently, the default global initialization in `create_standard_registry()` creates StateGlobal without migration support (`StateGlobal::new()` instead of `StateGlobal::with_migration_support()`).

To use migration features:
1. Run the integration test: `cargo test -p llmspell-bridge test_migration_from_lua -- --nocapture`
2. Or modify the global initialization to include migration support

## Migration API

When properly initialized, the State global provides these migration methods:

```lua
-- Get current migration status
local status = State.migration_status()
-- Returns: { 
--   migration_available = true/false,
--   current_version = "1.0.0",
--   latest_version = "2.0.0",
--   is_latest = false,
--   migration_targets = {"1.1.0", "2.0.0"}
-- }

-- List all schema versions
local versions = State.schema_versions()
-- Returns: {"1.0.0", "1.1.0", "2.0.0"}

-- Migrate to a specific version
local result = State.migrate("2.0.0")
-- Returns: {
--   success = true/false,
--   status = "completed",
--   from_version = "1.0.0",
--   to_version = "2.0.0",
--   items_migrated = 100,
--   duration_ms = 1234
-- }
```

## Future Work

The migration initialization should be made available through:
1. Configuration option to enable migration support
2. Automatic initialization when StateManager is configured
3. Dedicated migration CLI commands