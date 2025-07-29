# State Backup Examples

This directory contains examples demonstrating state backup and recovery functionality in rs-llmspell.

## Current Status

The backup functionality is partially implemented:
- ✅ **BackupManager** - Fully implemented in `llmspell-state-persistence/src/backup/`
- ✅ **RecoveryOrchestrator** - Fully implemented for point-in-time recovery
- ✅ **Compression** - Multiple algorithms supported (gzip, zstd, lz4, brotli)
- ✅ **CLI Commands** - Backup commands implemented in `llmspell-cli`
- ⏳ **StateGlobal Integration** - Backup methods are stubbed but not connected
- ⏳ **Lua API** - Methods defined but return mock results

## Example Files

### Working Examples

1. **`test_state_basic.lua`** - Basic State functionality test
   - Tests set/get/list/delete operations
   - Tests complex data types
   - Tests different scopes
   - Run: `cargo run --bin llmspell -- --config examples/lua/backup/backup-enabled.toml run examples/lua/backup/test_state_basic.lua`

2. **`recovery_scenarios_working.lua`** - Working recovery patterns demo
   - Simulates backup/restore using State scopes
   - Shows time-based recovery patterns
   - Demonstrates selective state management
   - Run: `cargo run --bin llmspell -- --config examples/lua/backup/backup-enabled.toml run examples/lua/backup/recovery_scenarios_working.lua`

### Mock Examples (Demonstrating Future API)

3. **`state_backup_mock.lua`** - Demonstrates planned backup API
   - Shows how backup operations will work once integrated
   - Displays expected result structures
   - Documents integration status

4. **`recovery_scenarios_mock.lua`** - Advanced recovery patterns
   - Time-based recovery scenarios
   - Backup chain management
   - Recovery strategies

### Original Examples (Currently Non-functional)

5. **`state_backup.lua`** - Full backup/restore example
   - Requires backup methods to be implemented in StateGlobal
   - Shows complete backup workflow

6. **`recovery_scenarios.lua`** - Advanced recovery scenarios
   - Demonstrates point-in-time recovery
   - Shows incremental backup chains
   - Includes progress monitoring

### Configuration

7. **`backup-enabled.toml`** - Configuration with state persistence enabled
   - Enables state persistence in runtime settings
   - Uses memory backend for examples

## Running the Examples

```bash
# Test basic State functionality (WORKING)
cargo run --bin llmspell -- --config examples/lua/backup/backup-enabled.toml run examples/lua/backup/test_state_basic.lua

# View mock backup API (WORKING)
cargo run --bin llmspell -- --config examples/lua/backup/backup-enabled.toml run examples/lua/backup/state_backup_mock.lua

# View mock recovery scenarios (WORKING)
cargo run --bin llmspell -- --config examples/lua/backup/backup-enabled.toml run examples/lua/backup/recovery_scenarios_mock.lua

# Run working recovery demo (WORKING - uses State scopes)
cargo run --bin llmspell -- --config examples/lua/backup/backup-enabled.toml run examples/lua/backup/recovery_scenarios_working.lua
```

## API Differences

The current StateGlobal uses different method names than the original examples:
- `State.set()` instead of `State.save()`
- `State.get()` instead of `State.load()`
- `State.list()` instead of `State.list_keys()`

## Integration Requirements

To fully enable backup functionality, the following steps are needed:

1. Initialize `BackupManager` in `state_infrastructure.rs`
2. Pass `BackupManager` to `StateGlobal` constructor
3. Connect the backup methods in `StateGlobal::inject_lua`
4. Wire up the backup CLI commands with the StateGlobal

## Backup API Overview

Once fully integrated, the State global will support:

### `State.create_backup(incremental)`
Creates a backup of the current state.
- `incremental` (boolean, optional): If true, creates an incremental backup. Default is false (full backup).
- Returns: Table with backup metadata including backup_id, success status, size, and entry count.

### `State.list_backups()`
Lists all available backups.
- Returns: Array of backup metadata tables, each containing:
  - `id`: Unique backup identifier
  - `created_at`: Timestamp when backup was created
  - `size_bytes`: Size of the backup in bytes
  - `is_incremental`: Whether this is an incremental backup
  - `parent_id`: For incremental backups, the parent backup ID

### `State.restore_backup(backup_id)`
Restores state from a specific backup.
- `backup_id` (string): The ID of the backup to restore
- Returns: Table with success status and any error messages

### `State.validate_backup(backup_id)`
Validates the integrity of a backup.
- `backup_id` (string): The ID of the backup to validate
- Returns: Table with validation results including:
  - `is_valid`: Overall validation status
  - `checksum_valid`: Whether checksums match
  - `integrity_valid`: Whether data integrity is verified
  - `errors`: Array of any validation errors

## Backup Configuration

Backup functionality requires proper configuration in your llmspell settings:

```toml
[runtime.state_persistence]
enabled = true
backend_type = "memory"  # or "sled", "rocksdb" for persistent storage

[runtime.state_persistence.backup]
backup_dir = "./backups"
compression_enabled = true
compression_type = "zstd"
compression_level = 3
incremental_enabled = true
max_backups = 10
max_backup_age = 2592000  # 30 days in seconds
```

## See Also

- [State Persistence Documentation](../../../docs/user-guide/state-persistence.md)
- [Migration Examples](../migration/) - For schema migration functionality
- [State Management Examples](../state/) - For basic state operations