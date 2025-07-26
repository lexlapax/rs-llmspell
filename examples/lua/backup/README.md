# State Backup Examples

This directory contains examples demonstrating the state backup and restore functionality in rs-llmspell.

## Examples

### state_backup.lua
Demonstrates basic backup operations including:
- Creating full backups of state data
- Creating incremental backups for efficient storage
- Listing available backups with metadata
- Validating backup integrity
- Restoring state from backups
- Recovering from data loss scenarios

## Running the Examples

```bash
# Run the backup example
llmspell run examples/lua/backup/state_backup.lua
```

## Backup API Overview

The State global provides the following backup methods:

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
backend_type = "sled"  # or "rocksdb" for persistent storage

[runtime.state_persistence.backup]
backup_dir = "./backups"
compression_enabled = true
compression_type = "zstd"
compression_level = 3
incremental_enabled = true
max_backups = 10
max_backup_age = 2592000  # 30 days in seconds
```

## Best Practices

1. **Regular Backups**: Schedule regular full backups (e.g., daily) with incremental backups between them.

2. **Validation**: Always validate backups after creation to ensure they can be restored.

3. **Retention Policy**: Configure appropriate retention policies to balance storage usage with recovery needs.

4. **Testing**: Regularly test restore procedures to ensure backups are working correctly.

5. **Compression**: Enable compression to reduce storage requirements, especially for large state data.

## Implementation Status

⚠️ **Note**: The backup functionality is currently in development. The API is available but returns stub responses. Full implementation will be available when the BackupManager is fully integrated with the StateManager.

## See Also

- [State Persistence Documentation](../../../docs/user-guide/state-persistence.md)
- [Migration Examples](../migration/) - For schema migration functionality
- [State Management Examples](../state/) - For basic state operations