# State Management Best Practices

**Version**: Phase 5 Implementation  
**Audience**: Developers using rs-llmspell state management  
**Last Updated**: Phase 5.9.3

> **🎯 PURPOSE**: This guide provides recommended patterns, performance tips, security considerations, and common pitfalls to help you use the state management system effectively.

## Table of Contents

- [Overview](#overview)
- [General Principles](#general-principles)
- [Scope Selection](#scope-selection)
- [Key Design](#key-design)
- [Data Organization](#data-organization)
- [Performance Best Practices](#performance-best-practices)
- [Security Best Practices](#security-best-practices)
- [Agent State Management](#agent-state-management)
- [Schema Design & Migration](#schema-design--migration)
- [Backup & Recovery](#backup--recovery)
- [Hook Integration](#hook-integration)
- [Script Integration](#script-integration)
- [Error Handling](#error-handling)
- [Testing Strategies](#testing-strategies)
- [Monitoring & Debugging](#monitoring--debugging)
- [Common Pitfalls](#common-pitfalls)

## Overview

The rs-llmspell state management system is designed for high performance, reliability, and security. Following these best practices will help you build maintainable, efficient applications while avoiding common pitfalls.

### Key Principles

1. **Scope Appropriately**: Choose the right scope for your data
2. **Design for Evolution**: Plan for schema changes and migrations
3. **Optimize for Access Patterns**: Structure data for how it's used
4. **Secure by Default**: Apply appropriate security measures
5. **Monitor Performance**: Track metrics and optimize bottlenecks
6. **Handle Errors Gracefully**: Plan for failure scenarios
7. **Test Thoroughly**: Validate state behavior under various conditions

## General Principles

### Favor Immutability

When possible, treat state as immutable and create new versions rather than modifying in place:

```rust
// ❌ Avoid: Modifying state in place
let mut user_data = state_manager.get(scope, "user").await?.unwrap();
user_data["last_login"] = json!(SystemTime::now());
state_manager.set(scope, "user", user_data).await?;

// ✅ Prefer: Create new state version
let user_data = state_manager.get(scope, "user").await?
    .unwrap_or_else(|| json!({}));
let updated_user = json!({
    "user_id": user_data["user_id"],
    "name": user_data["name"],
    "last_login": SystemTime::now(),
    "version": user_data["version"].as_u64().unwrap_or(0) + 1
});
state_manager.set(scope, "user", updated_user).await?;
```

### Use Atomic Operations

Group related changes into single operations to maintain consistency:

```rust
// ❌ Avoid: Multiple separate operations
state_manager.set(scope, "user_name", json!("John")).await?;
state_manager.set(scope, "user_email", json!("john@example.com")).await?;
state_manager.set(scope, "user_role", json!("admin")).await?;

// ✅ Prefer: Single atomic operation
state_manager.set(scope, "user", json!({
    "name": "John",
    "email": "john@example.com",
    "role": "admin",
    "updated_at": SystemTime::now()
})).await?;
```

### Version Your Data

Always include version information for future migrations:

```rust
let user_data = json!({
    "schema_version": 2,
    "user_id": "user123",
    "profile": {
        "name": "John Doe",
        "email": "john@example.com"
    },
    "preferences": {
        "theme": "dark",
        "language": "en"
    },
    "metadata": {
        "created_at": SystemTime::now(),
        "updated_at": SystemTime::now()
    }
});
```

## Scope Selection

Choose the appropriate scope based on data lifecycle and access patterns:

### Global Scope
**Use for**: Application-wide configuration, system settings, shared caches
```rust
// ✅ Good uses of Global scope
state_manager.set(StateScope::Global, "app_version", json!("1.0.0")).await?;
state_manager.set(StateScope::Global, "feature_flags", json!(flags)).await?;
state_manager.set(StateScope::Global, "rate_limits", json!(limits)).await?;

// ❌ Avoid: User-specific data in Global scope
state_manager.set(StateScope::Global, "user123_preferences", json!(prefs)).await?;
```

### Agent Scope
**Use for**: Agent-specific state, conversation history, model configuration
```rust
let agent_scope = StateScope::Agent("gpt-4-assistant".to_string());

// ✅ Good uses of Agent scope
state_manager.set(agent_scope.clone(), "conversation_history", json!(messages)).await?;
state_manager.set(agent_scope.clone(), "model_config", json!(config)).await?;
state_manager.set(agent_scope.clone(), "tool_usage_stats", json!(stats)).await?;

// ❌ Avoid: Global settings in Agent scope
state_manager.set(agent_scope, "app_version", json!("1.0.0")).await?;
```

### Workflow Scope
**Use for**: Workflow execution state, step results, workflow-specific configuration
```rust
let workflow_scope = StateScope::Workflow("data-processing".to_string());

// ✅ Good uses of Workflow scope
state_manager.set(workflow_scope.clone(), "execution_state", json!({
    "current_step": "validation",
    "completed_steps": ["input", "preprocessing"],
    "progress": 0.6
})).await?;

// ❌ Avoid: Persistent configuration in Workflow scope
state_manager.set(workflow_scope, "database_config", json!(config)).await?;
```

### Step Scope
**Use for**: Step-specific temporary data, intermediate results
```rust
let step_scope = StateScope::Step {
    workflow_id: "data-processing".to_string(),
    step_name: "validation".to_string(),
};

// ✅ Good uses of Step scope
state_manager.set(step_scope.clone(), "validation_results", json!(results)).await?;
state_manager.set(step_scope.clone(), "temp_data", json!(intermediate)).await?;

// ❌ Avoid: Long-lived data in Step scope
state_manager.set(step_scope, "user_profile", json!(profile)).await?;
```

## Key Design

### Use Hierarchical Keys

Design keys to reflect data relationships and enable efficient queries:

```rust
// ✅ Good key design - hierarchical and descriptive
"user:profile:basic"
"user:profile:preferences" 
"user:sessions:active"
"user:sessions:history"
"cache:api:users:list"
"cache:api:users:details:user123"
"system:config:database"
"system:config:features"

// ❌ Poor key design - flat and ambiguous
"user_data"
"config"
"cache1"
"temp"
```

### Include Metadata in Keys

Add context information to keys when helpful:

```rust
// ✅ Include timestamps for time-based data
let key = format!("events:{}:{}", date.format("%Y-%m-%d"), event_type);
state_manager.set(scope, &key, event_data).await?;

// ✅ Include entity IDs for relational data
let key = format!("user:{}:preferences", user_id);
state_manager.set(scope, &key, preferences).await?;

// ✅ Include version for versioned data
let key = format!("schema:{}:v{}", entity_type, version);
state_manager.set(scope, &key, schema).await?;
```

### Keep Keys Readable

Use descriptive names that are self-documenting:

```rust
// ✅ Clear and descriptive
"conversation_history"
"user_preferences_v2"
"last_backup_timestamp"
"feature_flags_production"

// ❌ Cryptic and unclear
"ch"
"up2"
"lbt"
"ffp"
```

## Data Organization

### Structure for Access Patterns

Organize data based on how it will be accessed:

```rust
// ✅ Grouped by access pattern
let user_profile = json!({
    "basic": {  // Frequently accessed together
        "user_id": "user123",
        "name": "John Doe",
        "email": "john@example.com"
    },
    "preferences": {  // Modified independently
        "theme": "dark",
        "language": "en",
        "notifications": true
    },
    "metadata": {  // Rarely accessed
        "created_at": "2024-01-01T00:00:00Z",
        "last_login": "2024-07-29T10:30:00Z"
    }
});

// ❌ Flat structure requiring full object updates
let user_data = json!({
    "user_id": "user123",
    "name": "John Doe",
    "email": "john@example.com",
    "theme": "dark",
    "language": "en",
    "notifications": true,
    "created_at": "2024-01-01T00:00:00Z",
    "last_login": "2024-07-29T10:30:00Z"
});
```

### Use Collections Wisely

For collections, consider access patterns and size limits:

```rust
// ✅ For small, frequently accessed collections
let recent_files = json!([
    {"name": "doc1.txt", "opened": "2024-07-29T10:00:00Z"},
    {"name": "doc2.txt", "opened": "2024-07-29T09:30:00Z"}
]);
state_manager.set(scope, "recent_files", recent_files).await?;

// ✅ For large collections, use separate keys with pagination
for page in 0..total_pages {
    let key = format!("user_events_page_{}", page);
    let events = get_events_page(page, PAGE_SIZE);
    state_manager.set(scope, &key, json!(events)).await?;
}

// Store pagination metadata
state_manager.set(scope, "user_events_meta", json!({
    "total_pages": total_pages,
    "page_size": PAGE_SIZE,
    "total_count": total_count
})).await?;
```

### Normalize Related Data

Avoid duplication by normalizing related data:

```rust
// ✅ Normalized approach
// Store user basic info once
state_manager.set(user_scope, "profile", json!({
    "user_id": "user123",
    "name": "John Doe",
    "email": "john@example.com"
})).await?;

// Reference by ID in related data
state_manager.set(session_scope, "current_user", json!({
    "user_id": "user123",  // Reference, not duplication
    "session_start": SystemTime::now(),
    "permissions": ["read", "write"]
})).await?;

// ❌ Denormalized approach with duplication
state_manager.set(session_scope, "current_user", json!({
    "user_id": "user123",
    "name": "John Doe",      // Duplicated
    "email": "john@example.com", // Duplicated
    "session_start": SystemTime::now(),
    "permissions": ["read", "write"]
})).await?;
```

## Performance Best Practices

### Use State Classes Appropriately

Classify your data to enable performance optimizations:

```rust
// ✅ Use StateClass::Trusted for internal system data
state_manager.set_with_class(
    StateScope::Global,
    "internal_counter",
    json!(42),
    StateClass::Trusted,  // Skip validation, enable fast path
).await?;

// ✅ Use StateClass::Ephemeral for cache-like data
state_manager.set_with_class(
    StateScope::Global,
    "temp_cache",
    json!(cached_data),
    StateClass::Ephemeral,  // Minimal persistence overhead
).await?;

// ✅ Use StateClass::Sensitive for PII
state_manager.set_with_class(
    StateScope::Agent(agent_id),
    "user_credentials",
    json!(credentials),
    StateClass::Sensitive,  // Extra protection
).await?;
```

### Batch Related Operations

Group related state changes to reduce overhead:

```rust
// ❌ Avoid: Multiple individual operations
for user in users {
    state_manager.set(
        StateScope::Global,
        &format!("user:{}", user.id),
        json!(user),
    ).await?;
}

// ✅ Prefer: Batch operations
let batch_data = users.into_iter()
    .map(|user| (format!("user:{}", user.id), json!(user)))
    .collect::<Vec<_>>();

for (key, value) in batch_data {
    state_manager.set(StateScope::Global, &key, value).await?;
}

// ✅ Even better: Use transactions if available
let transaction = state_manager.begin_transaction().await?;
for user in users {
    transaction.set(
        StateScope::Global,
        &format!("user:{}", user.id),
        json!(user),
    ).await?;
}
transaction.commit().await?;
```

### Cache Frequently Accessed Data

Use local caching for frequently accessed state:

```rust
// ✅ Implement local cache for hot data
struct StateCache {
    cache: Arc<RwLock<HashMap<String, Value>>>,
    state_manager: Arc<StateManager>,
}

impl StateCache {
    async fn get_cached(&self, scope: StateScope, key: &str) -> StateResult<Option<Value>> {
        let cache_key = format!("{}:{}", scope.to_string(), key);
        
        // Check cache first
        if let Some(value) = self.cache.read().get(&cache_key) {
            return Ok(Some(value.clone()));
        }
        
        // Fallback to state manager
        if let Some(value) = self.state_manager.get(scope, key).await? {
            // Update cache
            self.cache.write().insert(cache_key, value.clone());
            Ok(Some(value))
        } else {
            Ok(None)
        }
    }
}
```

### Avoid Large Objects

Keep individual state objects reasonably sized:

```rust
// ✅ Good: Reasonable object size
let user_config = json!({
    "theme": "dark",
    "language": "en",
    "timezone": "UTC",
    "preferences": {
        "auto_save": true,
        "notifications": false
    }
});

// ❌ Avoid: Very large objects (>1MB)
let massive_log = json!({
    "entries": vec![...; 100_000],  // Too large
    "metadata": {...}
});

// ✅ Better: Split large data
let log_metadata = json!({
    "total_entries": 100_000,
    "pages": 100,
    "page_size": 1000
});
state_manager.set(scope, "log_metadata", log_metadata).await?;

for page in 0..100 {
    let page_data = json!(get_log_page(page));
    let page_key = format!("log_page_{}", page);
    state_manager.set(scope, &page_key, page_data).await?;
}
```

## Security Best Practices

### Classify Sensitive Data

Identify and properly handle sensitive information:

```rust
// ✅ Classify data by sensitivity
let user_public = json!({
    "user_id": "user123",
    "display_name": "John D.",
    "public_profile": true
});

let user_private = json!({
    "email": "john@example.com",
    "phone": "+1-555-0123",
    "address": {...}
});

let user_secret = json!({
    "password_hash": "...",
    "api_keys": {...},
    "payment_info": {...}
});

// Store with appropriate protection
state_manager.set_with_class(
    scope, "user_public", user_public, StateClass::Standard
).await?;

state_manager.set_with_class(
    scope, "user_private", user_private, StateClass::Sensitive
).await?;

state_manager.set_with_class(
    scope, "user_secret", user_secret, StateClass::Sensitive
).await?;
```

### Use Access Controls

Implement proper access controls for state data:

```rust
// ✅ Set up access controls
let access_control = StateAccessControl::new();

// Grant specific permissions
access_control.grant_permission(
    "agent_123",
    StateScope::Agent("agent_123".to_string()),
    StatePermission::ReadWrite,
).await?;

access_control.grant_permission(
    "agent_123",
    StateScope::Global,
    StatePermission::ReadOnly,  // Read-only for global state
).await?;

// Always check permissions
async fn secure_state_access(
    state_manager: &StateManager,
    access_control: &StateAccessControl,
    actor_id: &str,
    scope: StateScope,
    key: &str,
    value: Value,
) -> StateResult<()> {
    if access_control.check_permission(actor_id, &scope, StatePermission::Write).await? {
        state_manager.set(scope, key, value).await?;
        Ok(())
    } else {
        Err(StateError::AccessDenied)
    }
}
```

### Sanitize Input Data

Always validate and sanitize data before storing:

```rust
// ✅ Validate input data
fn validate_user_input(data: &Value) -> StateResult<()> {
    if let Some(email) = data.get("email").and_then(|v| v.as_str()) {
        if !email.contains('@') {
            return Err(StateError::InvalidInput("Invalid email format".to_string()));
        }
    }
    
    if let Some(name) = data.get("name").and_then(|v| v.as_str()) {
        if name.len() > 100 {
            return Err(StateError::InvalidInput("Name too long".to_string()));
        }
    }
    
    Ok(())
}

// ✅ Sanitize sensitive data
let protector = SensitiveDataProtector::new(SensitiveDataConfig {
    redact_patterns: vec![
        r"\b\d{3}-\d{2}-\d{4}\b".to_string(), // SSN
        r"\b\d{16}\b".to_string(),            // Credit card
    ],
    encrypt_fields: vec!["password".to_string(), "api_key".to_string()],
    hash_fields: vec!["user_id".to_string()],
});

let protected_data = protector.protect_value(user_data).await?;
state_manager.set(scope, key, protected_data).await?;
```

### Encrypt Storage

Use encryption for persistent storage backends:

```rust
// ✅ Configure encryption
let config = PersistenceConfig {
    encryption: Some(EncryptionConfig {
        algorithm: EncryptionAlgorithm::AES256GCM,
        key_derivation: KeyDerivationConfig::PBKDF2 { 
            iterations: 100_000 
        },
    }),
    ..Default::default()
};

let state_manager = StateManager::with_config(config).await?;
```

## Agent State Management

### Implement PersistentAgent Properly

Follow best practices for agent state persistence:

```rust
impl PersistentAgent for MyAgent {
    fn agent_id(&self) -> &str {
        &self.id
    }
    
    fn get_persistent_state(&self) -> StateResult<PersistentAgentState> {
        // ✅ Only include essential state
        Ok(PersistentAgentState {
            agent_id: self.id.clone(),
            conversation_history: self.conversation.clone(),
            configuration: self.config.serialize()?,
            tool_usage_stats: self.tool_stats.clone(),
            execution_state: self.execution_state.clone(),
            metadata: json!({
                "last_activity": SystemTime::now(),
                "state_version": 2,
                "capabilities": self.capabilities.clone()
            }),
            schema_version: 2,
            created_at: self.created_at,
            updated_at: SystemTime::now(),
        })
    }
    
    fn apply_persistent_state(&mut self, state: PersistentAgentState) -> StateResult<()> {
        // ✅ Validate state before applying
        if state.schema_version > 2 {
            return Err(StateError::UnsupportedVersion(state.schema_version));
        }
        
        // ✅ Handle version migrations
        match state.schema_version {
            1 => self.migrate_from_v1(state)?,
            2 => self.apply_v2_state(state)?,
            _ => return Err(StateError::UnsupportedVersion(state.schema_version)),
        }
        
        self.updated_at = state.updated_at;
        Ok(())
    }
}
```

### Manage Agent Lifecycle

Properly handle agent state during lifecycle events:

```rust
// ✅ Proper agent lifecycle management
pub struct ManagedAgent {
    agent: Box<dyn PersistentAgent>,
    state_manager: Arc<StateManager>,
    auto_save_interval: Duration,
    last_save: Arc<RwLock<SystemTime>>,
}

impl ManagedAgent {
    pub async fn pause(&mut self) -> StateResult<()> {
        // Save state before pausing
        self.save_state().await?;
        self.agent.pause().await?;
        Ok(())
    }
    
    pub async fn resume(&mut self) -> StateResult<()> {
        // Load state when resuming
        self.load_state().await?;
        self.agent.resume().await?;
        Ok(())
    }
    
    pub async fn auto_save_if_needed(&self) -> StateResult<()> {
        let last_save = *self.last_save.read();
        if last_save.elapsed().unwrap_or(Duration::MAX) > self.auto_save_interval {
            self.save_state().await?;
        }
        Ok(())
    }
}
```

## Schema Design & Migration

### Design Schemas for Evolution

Plan for future changes when designing schemas:

```rust
// ✅ Good schema design - extensible
let schema = StateSchema {
    version: 1,
    semantic_version: SemanticVersion::new(1, 0, 0),
    fields: [
        ("user_id".to_string(), FieldSchema::String { max_length: Some(36) }),
        ("profile".to_string(), FieldSchema::Object { 
            required_fields: vec!["name".to_string()],
            optional_fields: vec!["email".to_string(), "phone".to_string()],
        }),
        ("preferences".to_string(), FieldSchema::Object { 
            required_fields: vec![],
            optional_fields: vec!["theme".to_string(), "language".to_string()],
        }),
        ("metadata".to_string(), FieldSchema::Object { 
            required_fields: vec!["created_at".to_string()],
            optional_fields: vec!["updated_at".to_string(), "last_login".to_string()],
        }),
    ].into_iter().collect(),
    compatibility: CompatibilityLevel::BackwardCompatible,
    migration_path: vec![], // Will be populated for future versions
};
```

### Plan Migration Paths

Design clear migration paths between schema versions:

```rust
// ✅ Well-planned migration
let migration_v1_to_v2 = MigrationStep {
    from_version: 1,
    to_version: 2,
    transformations: vec![
        // Move email from profile to contact_info
        FieldTransform::Move {
            from_field: "profile.email".to_string(),
            to_field: "contact_info.email".to_string(),
        },
        // Add new required field with default
        FieldTransform::Default {
            field: "contact_info.primary".to_string(),
            value: json!("email"),
        },
        // Transform old boolean to new enum
        FieldTransform::Custom {
            from_fields: vec!["preferences.notifications".to_string()],
            to_fields: vec!["preferences.notification_level".to_string()],
            transformer: "boolean_to_enum".to_string(),
            config: [
                ("true_value".to_string(), json!("all")),
                ("false_value".to_string(), json!("none")),
            ].into_iter().collect(),
        },
    ],
};
```

### Test Migrations Thoroughly

Always test migrations with real data:

```rust
// ✅ Migration testing
#[cfg(test)]
mod migration_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_v1_to_v2_migration() {
        let state_manager = StateManager::new().await.unwrap();
        let migration_engine = MigrationEngine::new(
            Arc::new(state_manager),
            Arc::new(schema_registry),
        ).unwrap();
        
        // Set up v1 test data
        let v1_data = json!({
            "user_id": "user123",
            "profile": {
                "name": "John Doe",
                "email": "john@example.com"
            },
            "preferences": {
                "notifications": true,
                "theme": "dark"
            },
            "metadata": {
                "created_at": "2024-01-01T00:00:00Z"
            }
        });
        
        state_manager.set(StateScope::Global, "test_user", v1_data).await.unwrap();
        
        // Perform migration
        let result = migration_engine.migrate_state(
            StateScope::Global, 
            "test_user", 
            2
        ).await.unwrap();
        
        assert!(result.success);
        
        // Verify migrated data
        let v2_data = state_manager.get(StateScope::Global, "test_user").await.unwrap().unwrap();
        assert_eq!(v2_data["contact_info"]["email"], "john@example.com");
        assert_eq!(v2_data["contact_info"]["primary"], "email");
        assert_eq!(v2_data["preferences"]["notification_level"], "all");
        assert!(v2_data["profile"]["email"].is_null());
    }
}
```

## Backup & Recovery

### Regular Backup Schedule

Implement a comprehensive backup strategy:

```rust
// ✅ Backup management
pub struct BackupScheduler {
    backup_manager: Arc<BackupManager>,
    schedule: BackupSchedule,
}

impl BackupScheduler {
    pub async fn run_scheduled_backups(&self) -> Result<(), BackupError> {
        // Full backup weekly
        if self.should_create_full_backup().await? {
            let backup_id = self.backup_manager.create_backup(BackupType::Full).await?;
            tracing::info!("Created full backup: {}", backup_id);
        }
        
        // Incremental backup daily
        if self.should_create_incremental_backup().await? {
            let base_backup = self.get_latest_full_backup().await?;
            let backup_id = self.backup_manager.create_backup(
                BackupType::Incremental { base_backup_id: base_backup }
            ).await?;
            tracing::info!("Created incremental backup: {}", backup_id);
        }
        
        // Cleanup old backups
        self.backup_manager.cleanup_old_backups().await?;
        
        Ok(())
    }
}
```

### Test Recovery Procedures

Regularly test backup recovery:

```rust
// ✅ Recovery testing
#[cfg(test)]
mod recovery_tests {
    #[tokio::test]
    async fn test_full_recovery_procedure() {
        let state_manager = create_test_state_manager().await;
        let backup_manager = BackupManager::new(
            Arc::clone(&state_manager),
            BackupConfig::default(),
        ).unwrap();
        
        // Create test data
        populate_test_data(&state_manager).await;
        
        // Create backup
        let backup_id = backup_manager.create_backup(BackupType::Full).await.unwrap();
        
        // Simulate data loss
        clear_all_state(&state_manager).await;
        
        // Perform recovery
        let recovery_result = backup_manager.restore_backup(
            &backup_id,
            RestoreOptions::default(),
        ).await.unwrap();
        
        // Verify recovery
        assert_eq!(recovery_result.restored_count, EXPECTED_STATE_COUNT);
        verify_test_data(&state_manager).await;
    }
}
```

### Monitor Backup Health

Track backup system health and performance:

```rust
// ✅ Backup monitoring
pub struct BackupMonitor {
    backup_manager: Arc<BackupManager>,
    metrics: Arc<BackupMetrics>,
}

impl BackupMonitor {
    pub async fn check_backup_health(&self) -> BackupHealthReport {
        let mut report = BackupHealthReport::new();
        
        // Check recent backups
        let recent_backups = self.backup_manager.list_recent_backups(24 * 7).await?;
        report.recent_backup_count = recent_backups.len();
        
        // Validate backup integrity
        for backup in recent_backups.iter().take(5) {
            let validation = self.backup_manager.validate_backup(&backup.id).await?;
            if !validation.is_valid {
                report.corrupted_backups.push(backup.id.clone());
            }
        }
        
        // Check storage usage
        let storage_usage = self.backup_manager.get_storage_usage().await?;
        report.storage_used = storage_usage.total_size;
        report.storage_available = storage_usage.available_space;
        
        // Performance metrics
        report.avg_backup_time = self.metrics.get_average_backup_time();
        report.avg_restore_time = self.metrics.get_average_restore_time();
        
        report
    }
}
```

## Hook Integration

### Design Efficient Hooks

Create hooks that don't impact performance:

```rust
// ✅ Efficient hook implementation
pub struct MetricsHook {
    metrics: Arc<StateMetrics>,
}

impl Hook for MetricsHook {
    async fn on_event(&self, event: &Event, _context: &mut HookContext) -> HookResult {
        match event {
            Event::StateChanged { scope, key, .. } => {
                // Fast, non-blocking metrics update
                self.metrics.increment_state_changes();
                self.metrics.record_scope_activity(scope.clone());
                
                // Don't block on expensive operations
                if self.should_detailed_logging(key) {
                    tokio::spawn({
                        let metrics = Arc::clone(&self.metrics);
                        let key = key.clone();
                        async move {
                            metrics.record_detailed_change(&key).await;
                        }
                    });
                }
                
                Ok(HookAction::Continue)
            }
            _ => Ok(HookAction::Continue),
        }
    }
}
```

### Handle Hook Failures Gracefully

Implement proper error handling for hooks:

```rust
// ✅ Robust hook error handling
pub struct RobustHook {
    inner: Box<dyn Hook>,
    circuit_breaker: CircuitBreaker,
    retry_config: RetryConfig,
}

impl Hook for RobustHook {
    async fn on_event(&self, event: &Event, context: &mut HookContext) -> HookResult {
        let mut attempts = 0;
        loop {
            match self.circuit_breaker.call(|| {
                self.inner.on_event(event, context)
            }).await {
                Ok(result) => return Ok(result),
                Err(e) if attempts < self.retry_config.max_attempts => {
                    attempts += 1;
                    tracing::warn!("Hook attempt {} failed: {}", attempts, e);
                    tokio::time::sleep(self.retry_config.delay).await;
                }
                Err(e) => {
                    tracing::error!("Hook failed after {} attempts: {}", attempts, e);
                    // Don't fail the entire operation due to hook failure
                    return Ok(HookAction::Continue);
                }
            }
        }
    }
}
```

## Script Integration

### Provide Clear APIs

Design script APIs that are intuitive and consistent:

```lua
-- ✅ Clear, consistent API design
local function save_user_preferences(user_id, preferences)
    local scope = "user:" .. user_id
    local success, error = pcall(function()
        State.save(scope, "preferences", preferences)
    end)
    
    if not success then
        print("Error saving preferences: " .. tostring(error))
        return false
    end
    
    return true
end

local function load_user_preferences(user_id)
    local scope = "user:" .. user_id
    local preferences = State.load(scope, "preferences")
    
    -- Always provide defaults
    if not preferences then
        preferences = {
            theme = "light",
            language = "en",
            notifications = true
        }
        
        -- Save defaults for next time
        save_user_preferences(user_id, preferences)
    end
    
    return preferences
end
```

### Handle Script Errors Properly

Implement proper error handling in scripts:

```lua
-- ✅ Robust error handling
local function safe_state_operation(operation_name, operation_func)
    local success, result = pcall(operation_func)
    
    if not success then
        print("State operation '" .. operation_name .. "' failed: " .. tostring(result))
        
        -- Log error for debugging
        if State.save then
            local error_log = State.load("system", "error_log") or {}
            table.insert(error_log, {
                operation = operation_name,
                error = tostring(result),
                timestamp = os.time()
            })
            State.save("system", "error_log", error_log)
        end
        
        return nil, result
    end
    
    return result
end

-- Usage
local preferences, error = safe_state_operation("load_preferences", function()
    return State.load("user", "preferences")
end)

if not preferences then
    print("Using default preferences due to error: " .. tostring(error))
    preferences = get_default_preferences()
end
```

### Validate Script Input

Always validate data coming from scripts:

```rust
// ✅ Script input validation
pub fn validate_script_state_input(
    scope: &str,
    key: &str,
    value: &Value,
) -> StateResult<()> {
    // Validate scope format
    if !is_valid_scope_format(scope) {
        return Err(StateError::InvalidInput(
            format!("Invalid scope format: {}", scope)
        ));
    }
    
    // Validate key format
    if key.len() > MAX_KEY_LENGTH {
        return Err(StateError::InvalidInput(
            format!("Key too long: {} > {}", key.len(), MAX_KEY_LENGTH)
        ));
    }
    
    // Validate value size
    let serialized_size = serde_json::to_vec(value)?.len();
    if serialized_size > MAX_VALUE_SIZE {
        return Err(StateError::InvalidInput(
            format!("Value too large: {} > {}", serialized_size, MAX_VALUE_SIZE)
        ));
    }
    
    // Validate value structure
    validate_json_structure(value)?;
    
    Ok(())
}
```

## Error Handling

### Use Specific Error Types

Create meaningful error messages:

```rust
// ✅ Specific error handling
#[derive(Debug, thiserror::Error)]
pub enum StateError {
    #[error("State key not found: {scope}:{key}")]
    KeyNotFound { scope: String, key: String },
    
    #[error("Invalid state value: {reason}")]
    InvalidValue { reason: String },
    
    #[error("Schema migration failed: {from_version} -> {to_version}: {reason}")]
    MigrationFailed { 
        from_version: u32, 
        to_version: u32, 
        reason: String 
    },
    
    #[error("Access denied for scope {scope} by actor {actor}")]
    AccessDenied { scope: String, actor: String },
    
    #[error("Storage backend error: {source}")]
    StorageError { 
        #[from] 
        source: llmspell_storage::StorageError 
    },
}
```

### Implement Graceful Degradation

Handle failures without breaking the entire system:

```rust
// ✅ Graceful degradation
pub struct FallbackStateManager {
    primary: Arc<StateManager>,
    fallback: Arc<StateManager>, // In-memory fallback
}

impl StateManagerTrait for FallbackStateManager {
    async fn get(&self, scope: StateScope, key: &str) -> StateResult<Option<Value>> {
        match self.primary.get(scope.clone(), key).await {
            Ok(value) => Ok(value),
            Err(e) => {
                tracing::warn!("Primary state manager failed, using fallback: {}", e);
                self.fallback.get(scope, key).await
            }
        }
    }
    
    async fn set(&self, scope: StateScope, key: &str, value: Value) -> StateResult<()> {
        // Try primary first
        match self.primary.set(scope.clone(), key, value.clone()).await {
            Ok(()) => Ok(()),
            Err(e) => {
                tracing::warn!("Primary state manager failed, using fallback: {}", e);
                // Still save to fallback for consistency
                self.fallback.set(scope, key, value).await?;
                // Return the original error but operation succeeded in fallback
                Err(e)
            }
        }
    }
}
```

### Log Errors Appropriately

Provide helpful error information without exposing sensitive data:

```rust
// ✅ Appropriate error logging
pub fn log_state_error(error: &StateError, context: &str) {
    match error {
        StateError::KeyNotFound { scope, key } => {
            tracing::debug!("State key not found in {}: {}:{}", context, scope, key);
        }
        StateError::AccessDenied { scope, actor } => {
            tracing::warn!(
                "Access denied in {}: actor '{}' attempted to access scope '{}'",
                context, actor, scope
            );
        }
        StateError::StorageError { source } => {
            tracing::error!("Storage error in {}: {}", context, source);
        }
        StateError::MigrationFailed { from_version, to_version, reason } => {
            tracing::error!(
                "Migration failed in {}: {} -> {}: {}",
                context, from_version, to_version, reason
            );
        }
        _ => {
            tracing::error!("State error in {}: {}", context, error);
        }
    }
}
```

## Testing Strategies

### Test State Operations

Create comprehensive tests for state functionality:

```rust
// ✅ Comprehensive state testing
#[cfg(test)]
mod state_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_basic_state_operations() {
        let state_manager = StateManager::new().await.unwrap();
        let scope = StateScope::Global;
        
        // Test set operation
        let value = json!({"test": "value"});
        state_manager.set(scope.clone(), "test_key", value.clone()).await.unwrap();
        
        // Test get operation
        let retrieved = state_manager.get(scope.clone(), "test_key").await.unwrap();
        assert_eq!(retrieved, Some(value));
        
        // Test delete operation
        let deleted = state_manager.delete(scope.clone(), "test_key").await.unwrap();
        assert!(deleted);
        
        // Test get after delete
        let after_delete = state_manager.get(scope, "test_key").await.unwrap();
        assert_eq!(after_delete, None);
    }
    
    #[tokio::test]
    async fn test_concurrent_access() {
        let state_manager = Arc::new(StateManager::new().await.unwrap());
        let scope = StateScope::Global;
        
        // Spawn multiple concurrent operations
        let mut handles = vec![];
        for i in 0..100 {
            let sm = Arc::clone(&state_manager);
            let scope = scope.clone();
            let handle = tokio::spawn(async move {
                let key = format!("key_{}", i);
                let value = json!({"id": i});
                sm.set(scope, &key, value).await.unwrap();
            });
            handles.push(handle);
        }
        
        // Wait for all operations
        for handle in handles {
            handle.await.unwrap();
        }
        
        // Verify all data was stored
        for i in 0..100 {
            let key = format!("key_{}", i);
            let value = state_manager.get(scope.clone(), &key).await.unwrap();
            assert_eq!(value.unwrap()["id"], i);
        }
    }
}
```

### Test Error Conditions

Test failure scenarios:

```rust
#[tokio::test]
async fn test_error_conditions() {
    let state_manager = StateManager::new().await.unwrap();
    
    // Test invalid scope
    let invalid_scope = StateScope::Custom("".to_string());
    let result = state_manager.set(invalid_scope, "key", json!("value")).await;
    assert!(result.is_err());
    
    // Test oversized value
    let large_value = json!("x".repeat(10_000_000)); // 10MB
    let result = state_manager.set(StateScope::Global, "large", large_value).await;
    assert!(result.is_err());
    
    // Test concurrent modification
    // ... test implementation
}
```

### Load Testing

Test performance under load:

```rust
#[tokio::test]
async fn load_test_state_operations() {
    let state_manager = Arc::new(StateManager::new().await.unwrap());
    let start_time = Instant::now();
    let operation_count = 10_000;
    
    // Concurrent load test
    let mut handles = vec![];
    for batch in 0..100 {
        let sm = Arc::clone(&state_manager);
        let handle = tokio::spawn(async move {
            for i in 0..100 {
                let key = format!("load_test_{}_{}", batch, i);
                let value = json!({"batch": batch, "item": i});
                sm.set(StateScope::Global, &key, value).await.unwrap();
            }
        });
        handles.push(handle);
    }
    
    for handle in handles {
        handle.await.unwrap();
    }
    
    let duration = start_time.elapsed();
    let ops_per_second = operation_count as f64 / duration.as_secs_f64();
    
    println!("Load test: {} ops in {:?} ({:.0} ops/sec)", 
             operation_count, duration, ops_per_second);
    
    // Assert minimum performance
    assert!(ops_per_second > 1000.0, "Performance below threshold");
}
```

## Monitoring & Debugging

### Implement Comprehensive Metrics

Track state system performance and health:

```rust
// ✅ State metrics collection
pub struct StateMetrics {
    operation_counter: Counter,
    operation_duration: Histogram,
    error_counter: Counter,
    cache_hit_rate: Gauge,
    storage_size: Gauge,
}

impl StateMetrics {
    pub fn record_operation(&self, operation: &str, duration: Duration, success: bool) {
        self.operation_counter
            .with_label_values(&[operation, if success { "success" } else { "error" }])
            .inc();
            
        self.operation_duration
            .with_label_values(&[operation])
            .observe(duration.as_secs_f64());
            
        if !success {
            self.error_counter
                .with_label_values(&[operation])
                .inc();
        }
    }
    
    pub fn update_cache_hit_rate(&self, hits: u64, total: u64) {
        let rate = if total > 0 { hits as f64 / total as f64 } else { 0.0 };
        self.cache_hit_rate.set(rate);
    }
}
```

### Enable Debug Logging

Provide detailed logging for troubleshooting:

```rust
// ✅ Structured debug logging
#[tracing::instrument(skip(self, value))]
async fn set_with_logging(
    &self,
    scope: StateScope,
    key: &str,
    value: Value,
) -> StateResult<()> {
    tracing::debug!(
        scope = ?scope,
        key = key,
        value_size = serde_json::to_vec(&value)?.len(),
        "Setting state value"
    );
    
    let start = Instant::now();
    let result = self.inner_set(scope.clone(), key, value).await;
    let duration = start.elapsed();
    
    match &result {
        Ok(()) => {
            tracing::debug!(
                scope = ?scope,
                key = key,
                duration_ms = duration.as_millis(),
                "State value set successfully"
            );
        }
        Err(e) => {
            tracing::error!(
                scope = ?scope,
                key = key,
                duration_ms = duration.as_millis(),
                error = %e,
                "Failed to set state value"
            );
        }
    }
    
    result
}
```

### Health Checks

Implement health checks for monitoring:

```rust
// ✅ State system health checks
pub struct StateHealthChecker {
    state_manager: Arc<StateManager>,
}

impl StateHealthChecker {
    pub async fn check_health(&self) -> HealthStatus {
        let mut status = HealthStatus::new();
        
        // Test basic operations
        match self.test_basic_operations().await {
            Ok(latency) => {
                status.add_check("basic_operations", true, Some(format!("{:?}", latency)));
            }
            Err(e) => {
                status.add_check("basic_operations", false, Some(e.to_string()));
            }
        }
        
        // Check storage backend health
        match self.state_manager.check_backend_health().await {
            Ok(()) => {
                status.add_check("storage_backend", true, None);
            }
            Err(e) => {
                status.add_check("storage_backend", false, Some(e.to_string()));
            }
        }
        
        // Check memory usage
        let memory_usage = self.state_manager.get_memory_usage().await.unwrap_or(0);
        let memory_limit = 1024 * 1024 * 1024; // 1GB
        status.add_check(
            "memory_usage",
            memory_usage < memory_limit,
            Some(format!("{} / {} bytes", memory_usage, memory_limit))
        );
        
        status
    }
    
    async fn test_basic_operations(&self) -> StateResult<Duration> {
        let test_scope = StateScope::Custom("health_check".to_string());
        let test_key = "test";
        let test_value = json!({"timestamp": SystemTime::now()});
        
        let start = Instant::now();
        
        // Test set
        self.state_manager.set(test_scope.clone(), test_key, test_value.clone()).await?;
        
        // Test get
        let retrieved = self.state_manager.get(test_scope.clone(), test_key).await?;
        if retrieved != Some(test_value) {
            return Err(StateError::InvalidValue { 
                reason: "Retrieved value doesn't match".to_string() 
            });
        }
        
        // Test delete
        self.state_manager.delete(test_scope, test_key).await?;
        
        Ok(start.elapsed())
    }
}
```

## Common Pitfalls

### Avoid These Anti-Patterns

#### 1. Storing Functions or Complex Objects
```rust
// ❌ Don't store functions or non-serializable data
let bad_data = json!({
    "callback": "function() { return 42; }", // Will break
    "date": SystemTime::now(), // May not serialize correctly
});

// ✅ Store serializable data only
let good_data = json!({
    "callback_name": "calculate_result",
    "timestamp": SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
});
```

#### 2. Ignoring Errors
```rust
// ❌ Ignoring potential errors
state_manager.set(scope, key, value).await.unwrap(); // Will panic on error

// ✅ Handle errors appropriately
match state_manager.set(scope, key, value).await {
    Ok(()) => {
        tracing::debug!("State saved successfully");
    }
    Err(e) => {
        tracing::error!("Failed to save state: {}", e);
        // Handle error appropriately - maybe use fallback or retry
    }
}
```

#### 3. Not Planning for Growth
```rust
// ❌ Hard-coded limits that will break
let user_list = vec![]; // Will grow indefinitely
state_manager.set(scope, "all_users", json!(user_list)).await?;

// ✅ Plan for pagination and growth
let user_batch = json!({
    "users": current_batch,
    "page": page_number,
    "total_pages": total_pages,
    "last_updated": SystemTime::now()
});
let key = format!("users_page_{}", page_number);
state_manager.set(scope, &key, user_batch).await?;
```

#### 4. Mixing Scopes Inappropriately
```rust
// ❌ Wrong scope usage
let global_scope = StateScope::Global;
state_manager.set(global_scope, "user123_temp_data", json!(temp)).await?; // Wrong!

// ✅ Use appropriate scopes
let user_scope = StateScope::Agent("user123".to_string());
state_manager.set(user_scope, "temp_data", json!(temp)).await?; // Correct!
```

#### 5. Not Validating Input
```rust
// ❌ Storing untrusted data directly
async fn save_user_input(input: Value) -> StateResult<()> {
    state_manager.set(StateScope::Global, "user_input", input).await // Dangerous!
}

// ✅ Validate and sanitize input
async fn save_user_input(input: Value) -> StateResult<()> {
    let sanitized = sanitize_user_input(&input)?;
    validate_input_size(&sanitized)?;
    validate_input_structure(&sanitized)?;
    
    state_manager.set(
        StateScope::Custom("user_input".to_string()),
        &generate_safe_key(),
        sanitized
    ).await
}
```

### Performance Pitfalls

#### 1. Frequent Small Updates
```rust
// ❌ Many small updates
for item in large_list {
    state_manager.set(scope.clone(), &item.id, json!(item)).await?;
}

// ✅ Batch updates
let batch_data = json!({
    "items": large_list,
    "updated_at": SystemTime::now(),
    "count": large_list.len()
});
state_manager.set(scope, "batch_data", batch_data).await?;
```

#### 2. Storing Large Objects
```rust
// ❌ Storing everything together
let massive_object = json!({
    "metadata": small_metadata,
    "huge_dataset": enormous_data, // This will slow everything down
});

// ✅ Split large data
state_manager.set(scope.clone(), "metadata", json!(small_metadata)).await?;

// Store large data in chunks
for (i, chunk) in enormous_data.chunks(1000).enumerate() {
    let chunk_key = format!("dataset_chunk_{}", i);
    state_manager.set(scope.clone(), &chunk_key, json!(chunk)).await?;
}
```

#### 3. Not Using State Classes
```rust
// ❌ Treating all data the same
state_manager.set(scope, "internal_counter", json!(42)).await?; // Slow path

// ✅ Use appropriate state classes
state_manager.set_with_class(
    scope,
    "internal_counter",
    json!(42),
    StateClass::Trusted // Fast path
).await?;
```

---

**Next**: [Examples](../../examples/state_persistence/) →

## See Also

- [State Management Overview](./README.md) - System overview and features
- [State Architecture](../technical/state-architecture.md) - Technical architecture
- [API Reference](../user-guide/api-reference.md) - Complete API documentation
- [Performance Guide](../user-guide/advanced/performance-tips.md) - Optimization techniques
- [Security Guide](../developer-guide/security-guide.md) - Security considerations