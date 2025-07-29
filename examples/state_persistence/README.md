# State Persistence Examples

**Purpose**: Comprehensive examples demonstrating rs-llmspell's state persistence system  
**Phase**: 5 Implementation  
**Last Updated**: Phase 5.9.3

> **🎯 OVERVIEW**: This directory contains examples showcasing all aspects of the state persistence system, from basic operations to advanced features like migrations, backups, and performance optimization.

## Table of Contents

- [Overview](#overview)
- [Quick Start](#quick-start) 
- [Example Categories](#example-categories)
- [Running Examples](#running-examples)
- [Configuration Files](#configuration-files)
- [Language Examples](#language-examples)
- [Advanced Features](#advanced-features)
- [Performance Examples](#performance-examples)
- [Testing Examples](#testing-examples)

## Overview

The state persistence system provides comprehensive persistent state management with:

- **Multiple Storage Backends**: Memory, Sled, RocksDB
- **State Scoping**: Global, Agent, Workflow, Step, Session isolation
- **Agent Integration**: Automatic agent state persistence
- **Schema Evolution**: Version-aware migrations
- **Backup & Recovery**: Atomic backups with retention policies
- **Performance Optimization**: Fast paths, lock-free operations
- **Security Features**: Encryption, access control, data protection
- **Script Integration**: Lua and JavaScript APIs
- **Hook System**: Automatic state change notifications

## Quick Start

### 1. Basic State Operations (Rust)

```rust
// examples/state_persistence/basic_operations.rs
use llmspell_state_persistence::{StateManager, StateScope};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create state manager
    let state_manager = StateManager::new().await?;
    
    // Save data
    state_manager.set(
        StateScope::Global, 
        "app_config", 
        json!({"theme": "dark", "version": "1.0.0"})
    ).await?;
    
    // Load data
    let config = state_manager.get(StateScope::Global, "app_config").await?;
    println!("Config: {}", config.unwrap());
    
    Ok(())
}
```

### 2. Basic State Operations (Lua)

```lua
-- examples/state_persistence/basic_operations.lua
-- Run with: ./llmspell -c examples/configs/state-enabled.toml run examples/state_persistence/basic_operations.lua

-- Save application configuration
State.save("global", "app_config", {
    theme = "dark",
    version = "1.0.0",
    features = {
        auto_save = true,
        notifications = false
    }
})

-- Load configuration
local config = State.load("global", "app_config")
print("Loaded config:")
print("  Theme: " .. config.theme)
print("  Version: " .. config.version)
print("  Auto-save: " .. tostring(config.features.auto_save))
```

## Example Categories

### Basic Examples
| File | Language | Description |
|------|----------|-------------|
| `basic_operations.rs` | Rust | Basic get/set/delete operations |
| `basic_operations.lua` | Lua | Script API basics |
| `scope_examples.rs` | Rust | Using different state scopes |
| `data_types.lua` | Lua | Working with different data types |

### Agent Integration
| File | Language | Description |
|------|----------|-------------|
| `agent_persistence.rs` | Rust | PersistentAgent trait implementation |
| `agent_lifecycle.rs` | Rust | Agent state during lifecycle events |
| `agent_state.lua` | Lua | Agent state from scripts |

### Storage Backends
| File | Language | Description |
|------|----------|-------------|
| `memory_backend.rs` | Rust | In-memory storage setup |
| `sled_backend.rs` | Rust | Sled persistent storage |
| `rocksdb_backend.rs` | Rust | RocksDB high-performance storage |
| `backend_comparison.rs` | Rust | Performance comparison |

### Schema & Migration
| File | Language | Description |
|------|----------|-------------|
| `schema_definition.rs` | Rust | Defining state schemas |
| `migration_example.rs` | Rust | Schema migrations |
| `migration_validation.rs` | Rust | Migration testing |
| `schema_migration.lua` | Lua | Script-triggered migrations |

### Backup & Recovery
| File | Language | Description |
|------|----------|-------------|
| `backup_creation.rs` | Rust | Creating and managing backups |
| `backup_recovery.rs` | Rust | Recovery procedures |
| `retention_policies.rs` | Rust | Backup retention management |
| `backup_example.lua` | Lua | Script backup operations |

### Security
| File | Language | Description |
|------|----------|-------------|
| `access_control.rs` | Rust | State access permissions |
| `encryption_example.rs` | Rust | Encrypted storage |
| `sensitive_data.rs` | Rust | PII protection |

### Performance
| File | Language | Description |
|------|----------|-------------|
| `fast_path.rs` | Rust | State class optimizations |
| `concurrent_access.rs` | Rust | Lock-free operations |
| `batch_operations.rs` | Rust | Efficient bulk operations |
| `performance_monitoring.rs` | Rust | Metrics and monitoring |

### Hooks & Events
| File | Language | Description |
|------|----------|-------------|
| `state_hooks.rs` | Rust | Custom state change hooks |
| `hook_integration.rs` | Rust | Hook system integration |
| `event_correlation.rs` | Rust | Event correlation tracking |

## Running Examples

### Prerequisites

1. **Rust Examples**: Compile and run directly
```bash
cd examples/state_persistence
cargo run --bin basic_operations
cargo run --bin agent_persistence
```

2. **Lua Examples**: Use llmspell CLI with appropriate config
```bash
# Basic state operations
./llmspell -c examples/configs/state-enabled.toml run examples/state_persistence/basic_operations.lua

# Migration examples
./llmspell -c examples/configs/migration-enabled.toml run examples/state_persistence/schema_migration.lua

# Backup examples
./llmspell -c examples/configs/backup-enabled.toml run examples/state_persistence/backup_example.lua
```

### Run All Examples

```bash
# Run all Rust examples
./run_rust_examples.sh

# Run all Lua examples
./run_lua_examples.sh

# Run comprehensive test suite
./run_all_examples.sh
```

## Configuration Files

### Basic State Configuration
```toml
# examples/state_persistence/configs/basic.toml
[state]
enabled = true
backend = "memory"

[state.performance]
cache_size = 10_000_000
fast_path_enabled = true
```

### Persistent Storage Configuration
```toml
# examples/state_persistence/configs/persistent.toml
[state]
enabled = true
backend = "sled"
path = "./example_state"

[state.sled]
cache_capacity = 50_000_000
use_compression = true
```

### Full Features Configuration
```toml
# examples/state_persistence/configs/full_features.toml
[state]
enabled = true
backend = "rocksdb"
path = "./example_rocks_state"

[state.rocksdb]
max_open_files = 1000
cache_capacity = 100_000_000

[state.migration]
enabled = true
auto_migrate = true
validation_level = "strict"

[state.backup]
enabled = true
backup_dir = "./example_backups"
retention_days = 7

[state.security]
encryption_enabled = true
access_control_enabled = true

[state.performance]
fast_path_enabled = true
async_hooks = true
```

## Language Examples

### Rust Examples

Most comprehensive examples showing full API usage:

- **Core Operations**: Basic CRUD operations with all backends
- **Agent Integration**: Complete PersistentAgent implementations
- **Advanced Features**: Migrations, backups, security, performance
- **Error Handling**: Robust error handling patterns
- **Testing**: Unit and integration test examples

### Lua Script Examples

Script-friendly examples for runtime usage:

- **Simple Operations**: Basic state save/load/delete
- **Agent State**: Working with agent-scoped state
- **Configuration Management**: App settings and preferences
- **Session Management**: User session handling
- **Data Validation**: Input validation patterns

### Configuration Examples

Ready-to-use configuration files:

- **Development**: Memory backend for testing
- **Production**: Persistent backends with optimization
- **High-Performance**: RocksDB with tuned settings
- **Secure**: Encryption and access control enabled
- **Full-Featured**: All capabilities enabled

## Advanced Features

### Schema Migrations

```rust
// examples/state_persistence/schema_migration.rs
use llmspell_state_persistence::schema::{StateSchema, MigrationStep, FieldTransform};

let migration_v1_to_v2 = MigrationStep {
    from_version: 1,
    to_version: 2,
    transformations: vec![
        FieldTransform::Move {
            from_field: "user.email".to_string(),
            to_field: "contact.email".to_string(),
        },
        FieldTransform::Default {
            field: "contact.verified".to_string(),
            value: json!(false),
        },
    ],
};
```

### Backup Strategies

```rust
// examples/state_persistence/backup_strategies.rs
use llmspell_state_persistence::backup::{BackupManager, BackupType, RetentionPolicy};

// Create full backup
let backup_id = backup_manager.create_backup(BackupType::Full).await?;

// Create incremental backup
let incremental_id = backup_manager.create_backup(
    BackupType::Incremental { base_backup_id: backup_id }
).await?;

// Apply retention policy
let policy = RetentionPolicy::TimeBasedPolicy {
    max_age: Duration::from_days(30),
    min_backups: 5,
};
backup_manager.apply_retention_policy(&policy).await?;
```

### Performance Optimization

```rust
// examples/state_persistence/performance_optimization.rs
use llmspell_state_persistence::performance::{StateClass, FastPathConfig};

// Use fast path for trusted data
state_manager.set_with_class(
    StateScope::Global,
    "internal_counter",
    json!(42),
    StateClass::Trusted, // Skips validation, uses MessagePack
).await?;

// Configure performance settings
let config = FastPathConfig {
    enabled: true,
    bypass_validation: true,
    use_messagepack: true,
    skip_hooks: false, // Still trigger hooks for monitoring
};
```

## Performance Examples

### Benchmarking State Operations

```rust
// examples/state_persistence/benchmarks.rs
use criterion::{criterion_group, criterion_main, Criterion};

fn benchmark_state_operations(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let state_manager = rt.block_on(StateManager::new()).unwrap();
    
    c.bench_function("set_operation", |b| {
        b.iter(|| {
            rt.block_on(state_manager.set(
                StateScope::Global,
                "bench_key",
                json!({"test": "data"}),
            )).unwrap();
        });
    });
    
    c.bench_function("get_operation", |b| {
        b.iter(|| {
            rt.block_on(state_manager.get(
                StateScope::Global,
                "bench_key",
            )).unwrap();
        });
    });
}

criterion_group!(benches, benchmark_state_operations);
criterion_main!(benches);
```

### Concurrent Access Patterns

```rust
// examples/state_persistence/concurrent_patterns.rs
async fn concurrent_access_example() -> Result<(), StateError> {
    let state_manager = Arc::new(StateManager::new().await?);
    
    // Spawn multiple concurrent writers
    let mut handles = vec![];
    for i in 0..100 {
        let sm = Arc::clone(&state_manager);
        let handle = tokio::spawn(async move {
            let key = format!("concurrent_key_{}", i);
            let value = json!({"id": i, "data": format!("test_data_{}", i)});
            sm.set(StateScope::Global, &key, value).await.unwrap();
        });
        handles.push(handle);
    }
    
    // Wait for all operations to complete
    for handle in handles {
        handle.await.unwrap();
    }
    
    println!("All concurrent operations completed successfully");
    Ok(())
}
```

## Testing Examples

### Unit Test Patterns

```rust
// examples/state_persistence/testing_patterns.rs
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_state_persistence() {
        let state_manager = StateManager::new().await.unwrap();
        
        // Test data
        let test_data = json!({
            "user_id": "test123",
            "preferences": {
                "theme": "dark",
                "language": "en"
            }
        });
        
        // Set state
        state_manager.set(
            StateScope::Global,
            "test_user",
            test_data.clone()
        ).await.unwrap();
        
        // Verify state
        let retrieved = state_manager.get(
            StateScope::Global,
            "test_user"
        ).await.unwrap();
        
        assert_eq!(retrieved, Some(test_data));
    }
    
    #[tokio::test]
    async fn test_agent_state_integration() {
        // Test agent state persistence
        let state_manager = Arc::new(StateManager::new().await.unwrap());
        let mut agent = TestAgent::new("test_agent");
        agent.set_state_manager(Arc::clone(&state_manager));
        
        // Modify agent state
        agent.add_message("Hello, world!".to_string());
        agent.update_config("model", "gpt-4");
        
        // Save state
        agent.save_state().await.unwrap();
        
        // Create new agent instance
        let mut new_agent = TestAgent::new("test_agent");
        new_agent.set_state_manager(Arc::clone(&state_manager));
        
        // Load state
        new_agent.load_state().await.unwrap();
        
        // Verify state was restored
        assert_eq!(agent.get_messages(), new_agent.get_messages());
        assert_eq!(agent.get_config(), new_agent.get_config());
    }
}
```

### Integration Test Setup

```rust
// examples/state_persistence/integration_tests.rs
use llmspell_state_persistence::*;
use tempfile::TempDir;

struct TestEnvironment {
    temp_dir: TempDir,
    state_manager: Arc<StateManager>,
}

impl TestEnvironment {
    async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        let state_manager = StateManager::with_backend(
            StorageBackendType::Sled(SledConfig {
                path: temp_dir.path().to_path_buf(),
                cache_capacity: 10 * 1024 * 1024,
                use_compression: true,
            }),
            PersistenceConfig::default(),
        ).await?;
        
        Ok(Self {
            temp_dir,
            state_manager: Arc::new(state_manager),
        })
    }
}

#[tokio::test]
async fn test_full_integration() {
    let env = TestEnvironment::new().await.unwrap();
    
    // Test various integration scenarios
    test_basic_operations(&env.state_manager).await;
    test_concurrent_access(&env.state_manager).await;
    test_error_handling(&env.state_manager).await;
    
    // Cleanup happens automatically when TempDir is dropped
}
```

## See Also

- [State Management Overview](../../docs/state-management/README.md) - System overview
- [Best Practices Guide](../../docs/state-management/best-practices.md) - Recommended patterns
- [State Architecture](../../docs/technical/state-architecture.md) - Technical details  
- [User Guide](../../docs/user-guide/state-persistence-guide.md) - End-user documentation
- [Existing Lua Examples](../lua/state/) - More Lua script examples

---

**Getting Started**: Run `./run_quick_start.sh` to try the basic examples!