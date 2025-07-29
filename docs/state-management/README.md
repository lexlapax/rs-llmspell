# State Management System

**Version**: Phase 5 Implementation  
**Status**: ✅ **PRODUCTION READY** - Full persistent state management  
**Last Updated**: Phase 5.9.3

> **🎯 OVERVIEW**: rs-llmspell's state management system provides comprehensive persistent state with multiple storage backends, hook integration, migration support, backup capabilities, and script integration.

## Table of Contents

- [Overview](#overview)
- [Core Concepts](#core-concepts)
- [Storage Backends](#storage-backends)
- [State Scoping](#state-scoping)
- [Agent State Persistence](#agent-state-persistence)
- [Hook Integration](#hook-integration)
- [Schema & Migrations](#schema--migrations)
- [Backup & Recovery](#backup--recovery)
- [Performance Optimizations](#performance-optimizations)
- [Security Features](#security-features)
- [Script Integration](#script-integration)
- [Configuration](#configuration)
- [Getting Started](#getting-started)
- [Advanced Features](#advanced-features)

## Overview

The rs-llmspell state management system provides a comprehensive solution for persistent state across agents, workflows, tools, and scripts. Built on top of proven storage backends with advanced features like schema versioning, backup/recovery, and performance optimizations.

### Key Features

| Feature | Description | Status |
|---------|-------------|--------|
| **Multiple Backends** | Memory, Sled, RocksDB support | ✅ Production |
| **State Scoping** | Global, Agent, Workflow, Step, Session isolation | ✅ Production |
| **Agent Persistence** | Full agent state save/restore | ✅ Production |
| **Hook Integration** | Automatic state change hooks | ✅ Production |
| **Schema Migration** | Version-aware state evolution | ✅ Production |
| **Backup/Recovery** | Atomic backups with retention | ✅ Production |
| **Performance Opts** | Fast paths, lock-free operations | ✅ Production |
| **Security** | Encryption, access control, data protection | ✅ Production |
| **Script API** | Lua/JavaScript integration | ✅ Production |
| **Monitoring** | Event correlation, performance metrics | ✅ Production |

## Core Concepts

### StateManager

The central component managing all persistent state operations:

```rust
use llmspell_state_persistence::{StateManager, StateScope, PersistenceConfig};
use serde_json::json;

// Create with default in-memory backend
let state_manager = StateManager::new().await?;

// Create with persistent backend
let state_manager = StateManager::with_backend(
    StorageBackendType::Sled(SledConfig {
        path: PathBuf::from("./state"),
        cache_capacity: 10 * 1024 * 1024,
        use_compression: true,
    }),
    PersistenceConfig::default(),
).await?;
```

### Basic Operations

```rust
// Set state
state_manager.set(StateScope::Global, "key", json!("value")).await?;

// Get state
let value = state_manager.get(StateScope::Global, "key").await?;

// Delete state
let deleted = state_manager.delete(StateScope::Global, "key").await?;

// List keys in scope
let keys = state_manager.list_keys(StateScope::Global).await?;
```

## Storage Backends

### Memory Backend
- **Use Case**: Development, testing, temporary state
- **Features**: Fastest access, no persistence
- **Configuration**: Default, no setup required

```rust
let state_manager = StateManager::new().await?; // Uses memory backend
```

### Sled Backend
- **Use Case**: Production applications requiring persistence
- **Features**: ACID transactions, compression, crash recovery
- **Performance**: ~10,000 ops/sec, ~1ms latency

```rust
let state_manager = StateManager::with_backend(
    StorageBackendType::Sled(SledConfig {
        path: PathBuf::from("./state_db"),
        cache_capacity: 50 * 1024 * 1024, // 50MB cache
        use_compression: true,
    }),
    PersistenceConfig::default(),
).await?;
```

### RocksDB Backend
- **Use Case**: High-performance applications, large datasets
- **Features**: Optimized compaction, bloom filters, block cache
- **Performance**: ~50,000 ops/sec, <1ms latency

```rust
let state_manager = StateManager::with_backend(
    StorageBackendType::RocksDB(RocksDBConfig {
        path: PathBuf::from("./rocks_state"),
        cache_capacity: 100 * 1024 * 1024, // 100MB cache
        max_open_files: 1000,
        compression: true,
    }),
    PersistenceConfig::default(),
).await?;
```

## State Scoping

State is organized into hierarchical scopes providing isolation and organization:

```rust
pub enum StateScope {
    Global,                                    // Application-wide state
    Agent(String),                            // Agent-specific state
    Workflow(String),                         // Workflow-specific state
    Step { workflow_id: String, step_name: String }, // Step-specific state
    Session(String),                          // Session-specific state (Phase 6)
    Custom(String),                           // Custom namespaces
}
```

### Scope Examples

```rust
// Global application settings
state_manager.set(StateScope::Global, "app_version", json!("1.0.0")).await?;

// Agent conversation history
let agent_scope = StateScope::Agent("gpt-4-assistant".to_string());
state_manager.set(agent_scope, "conversation", json!(messages)).await?;

// Workflow execution state
let workflow_scope = StateScope::Workflow("data-processing".to_string());
state_manager.set(workflow_scope, "progress", json!({"step": 3, "total": 10})).await?;

// Step-specific data
let step_scope = StateScope::Step {
    workflow_id: "data-processing".to_string(),
    step_name: "validation".to_string(),
};
state_manager.set(step_scope, "validation_results", json!(results)).await?;
```

### Scope Isolation

Each scope provides complete isolation:
- No cross-scope key conflicts
- Independent lifecycle management
- Separate backup/migration policies
- Scope-specific access controls

## Agent State Persistence

Agents can automatically persist their internal state including conversation history, configuration, and execution context.

### PersistentAgent Trait

```rust
use llmspell_state_persistence::{PersistentAgent, PersistentAgentState};

impl PersistentAgent for MyAgent {
    fn agent_id(&self) -> &str {
        &self.id
    }
    
    fn get_persistent_state(&self) -> StateResult<PersistentAgentState> {
        Ok(PersistentAgentState {
            agent_id: self.id.clone(),
            conversation_history: self.conversation.clone(),
            configuration: self.config.clone(),
            tool_usage_stats: self.tool_stats.clone(),
            execution_state: self.execution_state.clone(),
            metadata: self.metadata.clone(),
            schema_version: 2,
            created_at: self.created_at,
            updated_at: SystemTime::now(),
        })
    }
    
    fn apply_persistent_state(&mut self, state: PersistentAgentState) -> StateResult<()> {
        self.conversation = state.conversation_history;
        self.config = state.configuration;
        self.tool_stats = state.tool_usage_stats;
        self.execution_state = state.execution_state;
        self.metadata = state.metadata;
        self.updated_at = state.updated_at;
        Ok(())
    }
}
```

### Automatic Persistence

```rust
// Set state manager for automatic persistence
agent.set_state_manager(Arc::clone(&state_manager));

// State is automatically saved on lifecycle events
agent.pause().await?; // Saves state
agent.stop().await?;  // Saves state
agent.start().await?; // Can restore state
```

## Hook Integration

State changes automatically trigger hooks for monitoring, validation, and side effects.

### Built-in Hooks

- **StateChangeHook**: Logs all state modifications
- **StatePersistenceHook**: Handles automatic agent state persistence
- **StateValidationHook**: Validates state data against schemas
- **StateBackupHook**: Triggers backups on critical state changes

### Custom State Hooks

```rust
use llmspell_hooks::{Hook, HookContext, HookResult};

pub struct CustomStateHook;

impl Hook for CustomStateHook {
    async fn on_event(&self, event: &Event, context: &mut HookContext) -> HookResult {
        match event {
            Event::StateChanged { scope, key, old_value, new_value } => {
                // Custom logic for state changes
                if key == "critical_config" {
                    // Trigger backup
                    self.backup_critical_state().await?;
                }
                Ok(HookAction::Continue)
            }
            _ => Ok(HookAction::Continue),
        }
    }
}
```

### Hook Replay

State hooks support replay for debugging and testing:

```rust
// Replay state change events
let replay_result = hook_replay_manager.replay_hooks_by_type(
    "StateChangeHook",
    TimeRange::last_hour(),
    ReplayOptions::with_modified_params(modified_params),
).await?;
```

## Schema & Migrations

State schemas enable version-aware data evolution and automatic migrations.

### Schema Definition

```rust
use llmspell_state_persistence::{StateSchema, FieldSchema, SemanticVersion};

let schema = StateSchema {
    version: 2,
    semantic_version: SemanticVersion::new(2, 1, 0),
    fields: [
        ("user_id".to_string(), FieldSchema::String { max_length: Some(36) }),
        ("preferences".to_string(), FieldSchema::Object { 
            required_fields: vec!["theme".to_string()],
            optional_fields: vec!["language".to_string()],
        }),
        ("created_at".to_string(), FieldSchema::Timestamp),
    ].into_iter().collect(),
    compatibility: CompatibilityLevel::BackwardCompatible,
    migration_path: vec![
        MigrationStep {
            from_version: 1,
            to_version: 2,
            transformations: vec![
                FieldTransform::Copy {
                    from_field: "lang".to_string(),
                    to_field: "preferences.language".to_string(),
                }
            ],
        }
    ],
};
```

### Automatic Migration

```rust
// Migration engine handles version upgrades automatically
let migration_engine = MigrationEngine::new(
    Arc::clone(&state_manager),
    Arc::clone(&schema_registry),
)?;

// Migrate specific state
let result = migration_engine.migrate_state(
    StateScope::Agent("agent_123".to_string()),
    "user_profile",
    2, // target version
).await?;

// Migrate entire scope
let results = migration_engine.migrate_scope(
    StateScope::Agent("agent_123".to_string()),
    2,
).await?;
```

### Migration Validation

```rust
// Validate migration before applying
let validation = migration_engine.validate_migration(
    &schema_registry.get_schema(1)?,
    &schema_registry.get_schema(2)?,
    ValidationLevel::Strict,
).await?;

if validation.is_valid {
    // Safe to migrate
    migration_engine.execute_migration().await?;
} else {
    // Handle validation errors
    for error in validation.errors {
        eprintln!("Migration error: {}", error);
    }
}
```

## Backup & Recovery

Comprehensive backup system with atomic snapshots, compression, encryption, and retention policies.

### Creating Backups

```rust
use llmspell_state_persistence::backup::{BackupManager, BackupConfig};

let backup_manager = BackupManager::new(
    Arc::clone(&state_manager),
    BackupConfig {
        backup_dir: PathBuf::from("./backups"),
        compression_algorithm: CompressionAlgorithm::Zstd,
        encryption: Some(EncryptionConfig {
            algorithm: EncryptionAlgorithm::AES256GCM,
            key_derivation: KeyDerivationConfig::PBKDF2 { iterations: 100_000 },
        }),
        max_backups: 50,
        max_backup_age: Duration::from_days(30),
    },
)?;

// Create full backup
let backup_id = backup_manager.create_backup(BackupType::Full).await?;

// Create incremental backup
let backup_id = backup_manager.create_backup(BackupType::Incremental {
    base_backup_id: last_full_backup_id,
}).await?;
```

### Backup Recovery

```rust
// List available backups
let backups = backup_manager.list_backups().await?;

// Validate backup integrity
let validation = backup_manager.validate_backup(&backup_id).await?;

if validation.is_valid {
    // Restore from backup
    let recovery_result = backup_manager.restore_backup(
        &backup_id,
        RestoreOptions {
            target_scope: Some(StateScope::Agent("agent_123".to_string())),
            dry_run: false,
            verify_checksums: true,
        },
    ).await?;
    
    println!("Restored {} state entries", recovery_result.restored_count);
} else {
    eprintln!("Backup validation failed: {:?}", validation.errors);
}
```

### Retention Policies

```rust
// Automatic cleanup based on retention policies
backup_manager.cleanup_old_backups().await?;

// Custom retention policy
let policy = RetentionPolicy::TimeBasedPolicy {
    max_age: Duration::from_days(30),
    min_backups: 5, // Always keep at least 5 backups
};

backup_manager.apply_retention_policy(&policy).await?;
```

## Performance Optimizations

### Fast Path Operations

State operations are optimized based on data classification:

```rust
pub enum StateClass {
    Ephemeral,    // Cache-like data, minimal overhead
    Trusted,      // Internal system state, skip validation
    Standard,     // Normal user data, full processing
    Sensitive,    // PII/secrets, extra protection
    External,     // Third-party data, strict validation
}

// Fast path for trusted data
state_manager.set_with_class(
    StateScope::Global,
    "internal_counter",
    json!(42),
    StateClass::Trusted,
).await?;
```

### Lock-Free Agent State

Agent state operations use lock-free patterns for high concurrency:

```rust
// Concurrent agent state updates use atomic operations
let agent_state_ops = FastAgentStateOps::new(Arc::clone(&state_manager));

// Lock-free read-copy-update for agent state
agent_state_ops.update_agent_state(
    "agent_123",
    |current_state| {
        let mut new_state = current_state.clone();
        new_state.message_count += 1;
        new_state
    },
).await?;
```

### Asynchronous Hook Processing

State hooks are processed asynchronously to avoid blocking state operations:

```rust
// Hooks are processed in background
let async_processor = AsyncHookProcessor::new(
    hook_executor,
    ProcessingConfig {
        batch_size: 100,
        flush_interval: Duration::from_millis(50),
        max_queue_size: 10_000,
    },
);

// State operations return immediately
state_manager.set(scope, key, value).await?; // Returns immediately
// Hooks are processed asynchronously in background
```

## Security Features

### Access Control

```rust
use llmspell_state_persistence::{StateAccessControl, StatePermission};

// Set up access control
let access_control = StateAccessControl::new();

// Grant permissions
access_control.grant_permission(
    "agent_123",
    StateScope::Agent("agent_123".to_string()),
    StatePermission::ReadWrite,
).await?;

access_control.grant_permission(
    "agent_123",
    StateScope::Global,
    StatePermission::ReadOnly,
).await?;

// Check permissions before operations
if access_control.check_permission("agent_123", &scope, StatePermission::Write).await? {
    state_manager.set(scope, key, value).await?;
} else {
    return Err(StateError::AccessDenied);
}
```

### Sensitive Data Protection

```rust
use llmspell_state_persistence::{SensitiveDataProtector, SensitiveDataConfig};

let protector = SensitiveDataProtector::new(SensitiveDataConfig {
    redact_patterns: vec![
        r"\b\d{3}-\d{2}-\d{4}\b".to_string(), // SSN
        r"\b\d{16}\b".to_string(),            // Credit card
    ],
    encrypt_fields: vec!["password".to_string(), "api_key".to_string()],
    hash_fields: vec!["user_id".to_string()],
});

// Automatically protect sensitive data
let protected_value = protector.protect_value(sensitive_data).await?;
state_manager.set(scope, key, protected_value).await?;
```

### Encryption

```rust
// Configure encryption for storage backend
let config = PersistenceConfig {
    encryption: Some(EncryptionConfig {
        algorithm: EncryptionAlgorithm::AES256GCM,
        key_derivation: KeyDerivationConfig::PBKDF2 { iterations: 100_000 },
    }),
    ..Default::default()
};

let state_manager = StateManager::with_config(config).await?;
```

## Script Integration

State is accessible from Lua and JavaScript scripts through global APIs.

### Lua API

```lua
-- Basic operations
State.save("global", "app_config", {
    theme = "dark",
    version = "1.0.0"
})

local config = State.load("global", "app_config")
print("Theme: " .. config.theme)

State.delete("global", "old_key")

-- Agent-specific state
State.save("agent:gpt-4", "conversation", messages)
local history = State.load("agent:gpt-4", "conversation")

-- Migration operations (if enabled)
if State.migrate then
    State.migrate("agent:gpt-4", "conversation", 2) -- Migrate to version 2
end

-- Backup operations (if enabled)
if State.create_backup then
    local backup_id = State.create_backup("full")
    State.restore_backup(backup_id)
end
```

### JavaScript API

```javascript
// Same API available in JavaScript
State.save("global", "user_preferences", {
    language: "en",
    notifications: true
});

const prefs = State.load("global", "user_preferences");
console.log("Language:", prefs.language);

// Async operations in JavaScript
async function manageState() {
    await State.save("session", "temp_data", { value: 42 });
    const data = await State.load("session", "temp_data");
    return data;
}
```

## Configuration

### Basic Configuration

```toml
# llmspell.toml
[state]
enabled = true
backend = "sled"  # "memory", "sled", "rocksdb"
path = "./state_db"

[state.performance]
cache_size = 50_000_000  # 50MB
fast_path_enabled = true
async_hooks = true

[state.backup]
enabled = true
backup_dir = "./backups"
retention_days = 30
compression = "zstd"

[state.migration]
enabled = true
auto_migrate = true
validation_level = "strict"

[state.security]
encryption_enabled = true
access_control_enabled = true
```

### Advanced Configuration

```toml
[state.sled]
cache_capacity = 100_000_000
use_compression = true
flush_every_ms = 1000

[state.rocksdb]
max_open_files = 1000
block_cache_size = 100_000_000
bloom_filter_bits = 10

[state.encryption]
algorithm = "AES256GCM"
key_derivation = "PBKDF2"
iterations = 100_000

[state.hooks]
max_concurrent = 10
batch_size = 100
timeout_ms = 5000
```

## Getting Started

### 1. Basic Setup

```rust
use llmspell_state_persistence::{StateManager, StateScope, PersistenceConfig};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create state manager
    let state_manager = StateManager::new().await?;
    
    // Save some data
    state_manager.set(
        StateScope::Global, 
        "welcome_message", 
        json!("Hello, rs-llmspell!")
    ).await?;
    
    // Retrieve data
    let message = state_manager.get(StateScope::Global, "welcome_message").await?;
    println!("Message: {}", message.unwrap());
    
    Ok(())
}
```

### 2. Agent Integration

```rust
use llmspell_agents::{BasicAgent, AgentBuilder};
use llmspell_state_persistence::{StateManager, PersistentAgent};

// Create agent with state persistence
let state_manager = Arc::new(StateManager::new().await?);
let mut agent = BasicAgent::new(
    AgentBuilder::basic("my-agent")
        .description("Agent with state persistence")
        .build()?
)?;

// Enable state persistence
agent.set_state_manager(Arc::clone(&state_manager));

// Agent state is automatically persisted on lifecycle events
agent.pause().await?; // Saves state
agent.resume().await?; // Can restore state
```

### 3. Script Integration

```bash
# Create configuration
cat > llmspell.toml << EOF
[state]
enabled = true
backend = "sled"
path = "./my_state"
EOF

# Run Lua script with state
./llmspell -c llmspell.toml run my_script.lua
```

```lua
-- my_script.lua
State.save("global", "counter", 0)

for i = 1, 10 do
    local count = State.load("global", "counter") or 0
    count = count + 1
    State.save("global", "counter", count)
    print("Count: " .. count)
end
```

## Advanced Features

### Event Correlation

State changes are correlated with other system events:

```rust
// State changes include correlation metadata
let correlation_context = CorrelationContext::new()
    .with_parent_id(parent_event_id)
    .with_component("state-manager")
    .with_operation("set");

state_manager.set_with_correlation(
    scope,
    key,
    value,
    correlation_context,
).await?;

// Query correlated events
let timeline = event_correlation_tracker.get_timeline(
    correlation_id,
    TimeRange::last_hour(),
).await?;
```

### Performance Monitoring

```rust
// Built-in performance metrics
let metrics = state_manager.get_performance_metrics().await?;
println!("Operations per second: {}", metrics.ops_per_second);
println!("Average latency: {:?}", metrics.avg_latency);
println!("Cache hit rate: {:.2}%", metrics.cache_hit_rate * 100.0);

// Hook execution metrics
let hook_metrics = hook_executor.get_metrics().await?;
println!("Hook execution time: {:?}", hook_metrics.avg_execution_time);
```

### Circuit Breaker

Automatic protection against cascading failures:

```rust
// Circuit breaker protects against hook failures
let circuit_breaker = CircuitBreaker::new(CircuitBreakerConfig {
    failure_threshold: 5,
    timeout: Duration::from_secs(30),
    half_open_timeout: Duration::from_secs(10),
});

// Hooks are automatically disabled if they consistently fail
state_manager.set_hook_circuit_breaker(circuit_breaker).await?;
```

### Custom Storage Backends

```rust
use llmspell_storage::StorageBackend;

// Implement custom storage backend
pub struct MyCustomBackend;

impl StorageBackend for MyCustomBackend {
    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>, StorageError> {
        // Custom implementation
    }
    
    async fn set(&self, key: &str, value: Vec<u8>) -> Result<(), StorageError> {
        // Custom implementation
    }
    
    // ... other methods
}

// Use custom backend
let state_manager = StateManager::with_custom_backend(
    Box::new(MyCustomBackend),
    PersistenceConfig::default(),
).await?;
```

## See Also

- [Best Practices Guide](./best-practices.md) - Recommended patterns and practices
- [State Architecture](../technical/state-architecture.md) - Technical architecture details
- [User Guide](../user-guide/state-persistence-guide.md) - End-user documentation
- [API Reference](../user-guide/api-reference.md) - Complete API documentation
- [Examples](../../examples/state_persistence/) - Working code examples
- [Performance Guide](../user-guide/advanced/performance-tips.md) - Optimization techniques

---

**Next**: [Best Practices Guide](./best-practices.md) →