# llmspell-state-persistence

Enterprise-grade persistent state management for rs-llmspell with multi-backend support, schema migrations, and atomic backups.

## Overview

This crate provides the complete state persistence implementation for rs-llmspell (Phase 5), enabling production-ready state management with:

- **Multi-Backend Support**: Memory (default), Sled, and RocksDB
- **State Scoping**: 6 levels (Global, Agent, Workflow, Step, Session, Custom)
- **Schema Migrations**: Version management with field transformations
- **Atomic Backups**: SHA256-validated backups with compression
- **Hook Integration**: All state changes trigger hooks (<2% overhead)
- **Enterprise Features**: Retention policies, recovery procedures, audit trails

## Features

### Core State Management
- Thread-safe async operations with <5ms latency
- Automatic agent state persistence on pause/stop
- Manual load operations for API safety
- Scope-based isolation and access control

### Migration Framework
- Semantic versioning with compatibility checking
- Field transformations (Copy, Default, Remove)
- Exceptional performance: 2.07μs per item
- Automatic rollback on failure
- Full hook integration for migration lifecycle

### Backup & Recovery
- Atomic backup operations with progress tracking
- Point-in-time recovery with SHA256 validation
- Automated retention policies
- Multiple formats (JSON/Binary)
- Compression support

### Performance Optimizations
- StateClass classification (Critical, Standard, Bulk, Archive)
- Fast path operations with lock-free reads
- Async hook processing pipeline
- Unified serialization
- Lock-free agent state access

### Security
- Circular reference detection
- Sensitive data protection (API key redaction)
- Scope-based isolation
- Path sanitization
- Access control per agent

## Usage

### Basic State Operations

```rust
use llmspell_state_persistence::{StateManager, StateScope};
use llmspell_storage::StorageConfig;

// Create state manager with Sled backend
let config = StorageConfig::sled("/tmp/llmspell-state");
let manager = StateManager::new(config).await?;

// Save state
manager.save(
    StateScope::Agent("gpt-4".to_string()),
    "conversation_history",
    &messages
).await?;

// Load state
let messages: Vec<Message> = manager.load(
    StateScope::Agent("gpt-4".to_string()),
    "conversation_history"
).await?.unwrap_or_default();
```

### Schema Migrations

```rust
use llmspell_state_persistence::{Migration, FieldTransform};

let migration = Migration::new(1, 2)
    .add_transform(FieldTransform::Copy {
        from: "old_field".to_string(),
        to: "new_field".to_string(),
    })
    .add_transform(FieldTransform::Default {
        field: "version".to_string(),
        value: StateValue::Number(2.0),
    });

manager.migrate(migration).await?;
```

### Backup & Recovery

```rust
// Create backup
let backup_id = manager.backup(
    "Pre-deployment backup",
    BackupFormat::Json,
    true // compress
).await?;

// Restore from backup
manager.restore(&backup_id).await?;

// Configure retention
manager.set_retention_policy(
    RetentionPolicy::KeepRecent(7),
    true // enable automatic cleanup
).await?;
```

## Architecture

The crate is organized into 7 major subsystems:

1. **Core** (`manager.rs`, `backend_adapter.rs`) - State manager and backend abstraction
2. **Backup** (`backup/`) - Atomic backup and recovery system
3. **Migration** (`migration/`) - Schema versioning and transformation engine
4. **Performance** (`performance/`) - Optimization modules
5. **Schema** (`schema/`) - Schema registry and compatibility
6. **Security** (`sensitive_data.rs`, `circular_ref.rs`) - Data protection
7. **Integration** (`hooks.rs`, `agent_state.rs`) - System integration

## Performance

Achieved performance metrics (v0.5.0):

| Operation | Target | Actual |
|-----------|--------|--------|
| State Read | <1ms | <1ms |
| State Write | <5ms | <5ms |
| Migration | 100ms/1000 items | 2.07μs/item |
| Hook Overhead | <5% | <2% |
| Memory Increase | <10% | <10% |

## Testing

The crate includes comprehensive tests across 7 categories:

```bash
# Run all tests
cargo test

# Run specific test categories
./scripts/test-by-tag.sh unit
./scripts/test-by-tag.sh integration
./scripts/test-by-tag.sh migration

# Run benchmarks
cargo bench -p llmspell-state-persistence
```

## Examples

See the `examples/` directory for:
- `migration_example.rs` - Complete migration workflow
- `backup_retention_demo.rs` - Backup with retention policies
- `schema_evolution.rs` - Schema versioning patterns

## Dependencies

- `llmspell-state-traits` - Trait definitions
- `llmspell-storage` - Unified storage backend
- `llmspell-hooks` - Hook system integration
- `llmspell-events` - Event emission
- `llmspell-utils` - Shared utilities

## License

This project is dual-licensed under MIT OR Apache-2.0.