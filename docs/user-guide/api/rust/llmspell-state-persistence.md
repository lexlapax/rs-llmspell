# llmspell-state-persistence

**State management with automatic persistence and migration**

**🔗 Navigation**: [← Rust API](README.md) | [Crate Docs](https://docs.rs/llmspell-state-persistence) | [Source](../../../../llmspell-state-persistence)

---

## Overview

`llmspell-state-persistence` provides comprehensive state management with automatic persistence, versioning, migration, and multi-tenant isolation. It extends the basic state traits with production-ready persistence capabilities including backup/restore, schema evolution, and transaction support.

**Key Features:**
- 🔄 Automatic state persistence to disk/database
- 📦 Schema versioning and migration
- 🏢 Multi-tenant state isolation  
- 💾 Backup and restore capabilities
- ⚡ Transaction support with rollback
- 🔍 State querying and indexing
- 🛡️ Encryption at rest
- 📊 State metrics and monitoring

## Core Components

### StateManager Trait

The central trait for persistent state management:

```rust
#[async_trait]
pub trait StateManager: Send + Sync {
    /// Initialize the state manager with configuration
    async fn initialize(&self, config: StateConfig) -> Result<()>;
    
    /// Get a value from state
    async fn get(&self, key: &str) -> Result<Option<Value>>;
    
    /// Set a value in state
    async fn set(&self, key: &str, value: Value) -> Result<()>;
    
    /// Delete a value from state
    async fn delete(&self, key: &str) -> Result<()>;
    
    /// Get all keys matching a pattern
    async fn keys(&self, pattern: &str) -> Result<Vec<String>>;
    
    /// Clear all state
    async communityclear(&self) -> Result<()>;
    
    /// Create a scoped state view
    async fn scope(&self, scope: &str) -> Result<Box<dyn StateManager>>;
    
    /// Begin a transaction
    async fn begin_transaction(&self) -> Result<TransactionId>;
    
    /// Commit a transaction
    async fn commit_transaction(&self, tx_id: TransactionId) -> Result<()>;
    
    /// Rollback a transaction  
    async fn rollback_transaction(&self, tx_id: TransactionId) -> Result<()>;
    
    /// Create a backup
    async fn backup(&self, path: &Path) -> Result<BackupMetadata>;
    
    /// Restore from backup
    async fn restore(&self, path: &Path) -> Result<()>;
    
    /// Get state metrics
    async fn metrics(&self) -> Result<StateMetrics>;
}
```

### PersistentStateManager Implementation

Production-ready state manager with SQLite backend:

```rust
pub struct PersistentStateManager {
    db: Arc<Mutex<Connection>>,
    config: StateConfig,
    tenant_id: Option<TenantId>,
    encryption: Option<Box<dyn Encryptor>>,
    metrics: Arc<StateMetrics>,
}

impl PersistentStateManager {
    /// Create a new persistent state manager
    pub async fn new(config: StateConfig) -> Result<Self> {
        let db_path = config.persistence_path.as_ref()
            .ok_or_else(|| Error::Configuration("No persistence path".into()))?;
        
        let db = Connection::open(db_path)?;
        Self::initialize_schema(&db)?;
        
        Ok(Self {
            db: Arc::new(Mutex::new(db)),
            config,
            tenant_id: None,
            encryption: None,
            metrics: Arc::new(StateMetrics::default()),
        })
    }
    
    /// Enable encryption at rest
    pub fn with_encryption(mut self, encryptor: Box<dyn Encryptor>) -> Self {
        self.encryption = Some(encryptor);
        self
    }
    
    /// Set tenant isolation
    pub fn with_tenant(mut self, tenant_id: TenantId) -> Self {
        self.tenant_id = Some(tenant_id);
        self
    }
}
```

### Schema Migration

Automatic schema versioning and migration:

```rust
pub struct MigrationManager {
    current_version: SchemaVersion,
    migrations: Vec<Migration>,
}

pub struct Migration {
    pub version: SchemaVersion,
    pub description: String,
    pub up: Box<dyn Fn(&Connection) -> Result<()>>,
    pub down: Box<dyn Fn(&Connection) -> Result<()>>,
}

impl MigrationManager {
    /// Apply all pending migrations
    pub async fn migrate(&self, db: &Connection) -> Result<()> {
        let current = self.get_current_version(db)?;
        
        for migration in &self.migrations {
            if migration.version > current {
                println!("Applying migration {}: {}", 
                    migration.version, migration.description);
                (migration.up)(db)?;
                self.record_version(db, migration.version)?;
            }
        }
        
        Ok(())
    }
    
    /// Rollback to a specific version
    pub async fn rollback_to(&self, db: &Connection, target: SchemaVersion) -> Result<()> {
        let current = self.get_current_version(db)?;
        
        for migration in self.migrations.iter().rev() {
            if migration.version <= current && migration.version > target {
                println!("Rolling back migration {}", migration.version);
                (migration.down)(db)?;
                self.record_version(db, migration.version - 1)?;
            }
        }
        
        Ok(())
    }
}
```

## State Configuration

Comprehensive configuration options:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateConfig {
    /// Path for persistent storage
    pub persistence_path: Option<PathBuf>,
    
    /// Enable write-ahead logging
    pub enable_wal: bool,
    
    /// Sync mode (FULL, NORMAL, OFF)
    pub sync_mode: SyncMode,
    
    /// Cache size in MB
    pub cache_size_mb: usize,
    
    /// Enable compression
    pub enable_compression: bool,
    
    /// Backup configuration
    pub backup: BackupConfig,
    
    /// Retention policy
    pub retention: RetentionPolicy,
}

#[derive(Debug, Clone)]
pub struct BackupConfig {
    /// Enable automatic backups
    pub enabled: bool,
    
    /// Backup interval
    pub interval: Duration,
    
    /// Backup retention count
    pub retention_count: usize,
    
    /// Backup path
    pub backup_path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct RetentionPolicy {
    /// Maximum age for state entries
    pub max_age: Option<Duration>,
    
    /// Maximum number of versions to keep
    pub max_versions: usize,
    
    /// Cleanup interval
    pub cleanup_interval: Duration,
}
```

## Transaction Support

ACID transactions for state operations:

```rust
pub struct StateTransaction {
    id: TransactionId,
    operations: Vec<Operation>,
    timestamp: SystemTime,
}

impl StateTransaction {
    /// Add a set operation to the transaction
    pub fn set(&mut self, key: String, value: Value) {
        self.operations.push(Operation::Set { key, value });
    }
    
    /// Add a delete operation to the transaction
    pub fn delete(&mut self, key: String) {
        self.operations.push(Operation::Delete { key });
    }
    
    /// Execute all operations atomically
    pub async fn commit(self, manager: &PersistentStateManager) -> Result<()> {
        let db = manager.db.lock().await;
        let tx = db.transaction()?;
        
        for op in self.operations {
            match op {
                Operation::Set { key, value } => {
                    tx.execute(
                        "INSERT OR REPLACE INTO state (key, value, tenant_id) VALUES (?1, ?2, ?3)",
                        params![key, value, manager.tenant_id],
                    )?;
                }
                Operation::Delete { key } => {
                    tx.execute(
                        "DELETE FROM state WHERE key = ?1 AND tenant_id = ?2",
                        params![key, manager.tenant_id],
                    )?;
                }
            }
        }
        
        tx.commit()?;
        Ok(())
    }
}
```

## Scoped State

Create isolated state scopes:

```rust
pub struct ScopedState {
    parent: Arc<dyn StateManager>,
    scope: String,
}

impl ScopedState {
    /// Create a new scoped state view
    pub fn new(parent: Arc<dyn StateManager>, scope: String) -> Self {
        Self { parent, scope }
    }
    
    /// Convert a key to scoped format
    fn scoped_key(&self, key: &str) -> String {
        format!("{}::{}", self.scope, key)
    }
}

#[async_trait]
impl StateManager for ScopedState {
    async fn get(&self, key: &str) -> Result<Option<Value>> {
        self.parent.get(&self.scoped_key(key)).await
    }
    
    async fn set(&self, key: &str, value: Value) -> Result<()> {
        self.parent.set(&self.scoped_key(key), value).await
    }
    
    async fn keys(&self, pattern: &str) -> Result<Vec<String>> {
        let scoped_pattern = self.scoped_key(pattern);
        let keys = self.parent.keys(&scoped_pattern).await?;
        
        // Remove scope prefix from results
        Ok(keys.into_iter()
            .map(|k| k.strip_prefix(&format!("{}::", self.scope))
                .unwrap_or(&k)
                .to_string())
            .collect())
    }
}
```

## Backup and Restore

Comprehensive backup capabilities:

```rust
pub struct BackupManager {
    config: BackupConfig,
    scheduler: Arc<Mutex<Scheduler>>,
}

impl BackupManager {
    /// Create a manual backup
    pub async fn create_backup(&self, manager: &PersistentStateManager) -> Result<BackupMetadata> {
        let timestamp = SystemTime::now();
        let backup_name = format!("backup_{}.db", timestamp.duration_since(UNIX_EPOCH)?.as_secs());
        let backup_path = self.config.backup_path.join(&backup_name);
        
        // Create backup
        manager.db.lock().await
            .backup(DatabaseName::Main, &backup_path, None)?;
        
        // Create metadata
        let metadata = BackupMetadata {
            id: Uuid::new_v4(),
            timestamp,
            path: backup_path,
            size: std::fs::metadata(&backup_path)?.len(),
            checksum: self.calculate_checksum(&backup_path)?,
        };
        
        // Clean old backups
        self.cleanup_old_backups().await?;
        
        Ok(metadata)
    }
    
    /// Restore from a backup
    pub async fn restore_backup(&self, manager: &PersistentStateManager, backup_id: Uuid) -> Result<()> {
        let metadata = self.get_backup_metadata(backup_id)?;
        
        // Verify checksum
        if self.calculate_checksum(&metadata.path)? != metadata.checksum {
            return Err(Error::BackupCorrupted);
        }
        
        // Restore database
        let db = Connection::open(&metadata.path)?;
        let target_db = manager.db.lock().await;
        db.backup(DatabaseName::Main, &target_db, None)?;
        
        Ok(())
    }
    
    /// Schedule automatic backups
    pub async fn start_automatic_backups(&self, manager: Arc<PersistentStateManager>) {
        let interval = self.config.interval;
        let scheduler = self.scheduler.clone();
        
        tokio::spawn(async move {
            let mut interval_timer = tokio::time::interval(interval);
            
            loop {
                interval_timer.tick().await;
                
                if let Err(e) = self.create_backup(&manager).await {
                    error!("Automatic backup failed: {}", e);
                }
            }
        });
    }
}
```

## State Metrics

Monitor state usage and performance:

```rust
#[derive(Debug, Default)]
pub struct StateMetrics {
    pub total_keys: AtomicUsize,
    pub total_size_bytes: AtomicU64,
    pub read_count: AtomicU64,
    pub write_count: AtomicU64,
    pub delete_count: AtomicU64,
    pub transaction_count: AtomicU64,
    pub backup_count: AtomicU64,
    pub last_backup: RwLock<Option<SystemTime>>,
    pub cache_hits: AtomicU64,
    pub cache_misses: AtomicU64,
}

impl StateMetrics {
    /// Record a read operation
    pub fn record_read(&self, hit: bool) {
        self.read_count.fetch_add(1, Ordering::Relaxed);
        if hit {
            self.cache_hits.fetch_add(1, Ordering::Relaxed);
        } else {
            self.cache_misses.fetch_add(1, Ordering::Relaxed);
        }
    }
    
    /// Get cache hit ratio
    pub fn cache_hit_ratio(&self) -> f64 {
        let hits = self.cache_hits.load(Ordering::Relaxed) as f64;
        let total = (hits + self.cache_misses.load(Ordering::Relaxed) as f64).max(1.0);
        hits / total
    }
    
    /// Export metrics in Prometheus format
    pub fn export_prometheus(&self) -> String {
        format!(
            "# HELP llmspell_state_keys Total number of keys in state\n\
             # TYPE llmspell_state_keys gauge\n\
             llmspell_state_keys {}\n\
             # HELP llmspell_state_size_bytes Total size of state in bytes\n\
             # TYPE llmspell_state_size_bytes gauge\n\
             llmspell_state_size_bytes {}\n\
             # HELP llmspell_state_operations_total Total state operations\n\
             # TYPE llmspell_state_operations_total counter\n\
             llmspell_state_operations_total{{op=\"read\"}} {}\n\
             llmspell_state_operations_total{{op=\"write\"}} {}\n\
             llmspell_state_operations_total{{op=\"delete\"}} {}\n\
             # HELP llmspell_state_cache_hit_ratio Cache hit ratio\n\
             # TYPE llmspell_state_cache_hit_ratio gauge\n\
             llmspell_state_cache_hit_ratio {}",
            self.total_keys.load(Ordering::Relaxed),
            self.total_size_bytes.load(Ordering::Relaxed),
            self.read_count.load(Ordering::Relaxed),
            self.write_count.load(Ordering::Relaxed),
            self.delete_count.load(Ordering::Relaxed),
            self.cache_hit_ratio(),
        )
    }
}
```

## Usage Examples

### Basic State Operations

```rust
use llmspell_state_persistence::{PersistentStateManager, StateConfig};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize state manager
    let config = StateConfig {
        persistence_path: Some("./state.db".into()),
        enable_wal: true,
        sync_mode: SyncMode::Normal,
        cache_size_mb: 64,
        ..Default::default()
    };
    
    let manager = PersistentStateManager::new(config).await?;
    
    // Basic operations
    manager.set("user:123", json!({"name": "Alice", "score": 100})).await?;
    
    if let Some(value) = manager.get("user:123").await? {
        println!("User data: {}", value);
    }
    
    // Pattern matching
    let user_keys = manager.keys("user:*").await?;
    println!("Found {} users", user_keys.len());
    
    Ok(())
}
```

### Transactional Operations

```rust
use llmspell_state_persistence::StateTransaction;

async fn transfer_points(manager: &PersistentStateManager, from: &str, to: &str, amount: i32) -> Result<()> {
    let tx_id = manager.begin_transaction().await?;
    
    let mut tx = StateTransaction::new(tx_id);
    
    // Get current balances
    let from_balance = manager.get(from).await?
        .and_then(|v| v.as_i64())
        .unwrap_or(0) as i32;
    let to_balance = manager.get(to).await?
        .and_then(|v| v.as_i64())
        .unwrap_or(0) as i32;
    
    // Check sufficient balance
    if from_balance < amount {
        manager.rollback_transaction(tx_id).await?;
        return Err(Error::InsufficientBalance);
    }
    
    // Update balances
    tx.set(from.to_string(), json!(from_balance - amount));
    tx.set(to.to_string(), json!(to_balance + amount));
    
    // Commit transaction
    tx.commit(manager).await?;
    
    Ok(())
}
```

### Scoped State for Multi-Tenancy

```rust
async fn tenant_operations(manager: Arc<PersistentStateManager>) -> Result<()> {
    // Create tenant-specific scopes
    let tenant_a = manager.scope("tenant:a").await?;
    let tenant_b = manager.scope("tenant:b").await?;
    
    // Operations are isolated
    tenant_a.set("config", json!({"theme": "dark"})).await?;
    tenant_b.set("config", json!({"theme": "light"})).await?;
    
    // Each tenant sees only their data
    let a_config = tenant_a.get("config").await?;
    let b_config = tenant_b.get("config").await?;
    
    assert_ne!(a_config, b_config);
    
    Ok(())
}
```

### Backup and Restore

```rust
use llmspell_state_persistence::BackupManager;

async fn backup_operations(manager: Arc<PersistentStateManager>) -> Result<()> {
    let backup_manager = BackupManager::new(BackupConfig {
        enabled: true,
        interval: Duration::from_hours(6),
        retention_count: 10,
        backup_path: "./backups".into(),
    });
    
    // Create manual backup
    let metadata = backup_manager.create_backup(&manager).await?;
    println!("Created backup: {} ({} bytes)", metadata.id, metadata.size);
    
    // Start automatic backups
    backup_manager.start_automatic_backups(manager.clone()).await;
    
    // Restore from backup if needed
    if disaster_strikes() {
        backup_manager.restore_backup(&manager, metadata.id).await?;
        println!("Restored from backup {}", metadata.id);
    }
    
    Ok(())
}
```

### Schema Migration

```rust
use llmspell_state_persistence::{MigrationManager, Migration};

fn setup_migrations() -> MigrationManager {
    let mut manager = MigrationManager::new();
    
    // Version 1: Initial schema
    manager.add_migration(Migration {
        version: 1,
        description: "Initial schema".to_string(),
        up: Box::new(|db| {
            db.execute(
                "CREATE TABLE IF NOT EXISTS state (
                    key TEXT PRIMARY KEY,
                    value TEXT NOT NULL,
                    created_at INTEGER NOT NULL
                )",
                [],
            )?;
            Ok(())
        }),
        down: Box::new(|db| {
            db.execute("DROP TABLE state", [])?;
            Ok(())
        }),
    });
    
    // Version 2: Add tenant support
    manager.add_migration(Migration {
        version: 2,
        description: "Add multi-tenant support".to_string(),
        up: Box::new(|db| {
            db.execute(
                "ALTER TABLE state ADD COLUMN tenant_id TEXT",
                [],
            )?;
            db.execute(
                "CREATE INDEX idx_tenant ON state(tenant_id)",
                [],
            )?;
            Ok(())
        }),
        down: Box::new(|db| {
            db.execute("DROP INDEX idx_tenant", [])?;
            // Note: SQLite doesn't support DROP COLUMN
            Ok(())
        }),
    });
    
    // Version 3: Add expiration
    manager.add_migration(Migration {
        version: 3,
        description: "Add TTL support".to_string(),
        up: Box::new(|db| {
            db.execute(
                "ALTER TABLE state ADD COLUMN expires_at INTEGER",
                [],
            )?;
            Ok(())
        }),
        down: Box::new(|db| {
            // Rollback logic
            Ok(())
        }),
    });
    
    manager
}

async fn apply_migrations(db: &Connection) -> Result<()> {
    let migrations = setup_migrations();
    migrations.migrate(db).await?;
    Ok(())
}
```

## Performance Considerations

1. **Write-Ahead Logging (WAL)**: Enable WAL mode for better concurrency
2. **Caching**: In-memory cache reduces database reads
3. **Batch Operations**: Group multiple operations in transactions
4. **Indexing**: Create indexes on frequently queried patterns
5. **Compression**: Enable compression for large values
6. **Async I/O**: All operations are async for non-blocking execution

## Security Features

- **Encryption at Rest**: Optional AES-256 encryption
- **Tenant Isolation**: Complete data separation between tenants
- **Access Control**: Fine-grained permissions per scope
- **Audit Logging**: Track all state modifications
- **Secure Deletion**: Overwrite sensitive data on delete

## Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[tokio::test]
    async fn test_persistence_across_restart() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        
        // First session
        {
            let config = StateConfig {
                persistence_path: Some(db_path.clone()),
                ..Default::default()
            };
            let manager = PersistentStateManager::new(config).await.unwrap();
            manager.set("persistent_key", json!("persistent_value")).await.unwrap();
        }
        
        // Second session - data should persist
        {
            let config = StateConfig {
                persistence_path: Some(db_path),
                ..Default::default()
            };
            let manager = PersistentStateManager::new(config).await.unwrap();
            let value = manager.get("persistent_key").await.unwrap();
            assert_eq!(value, Some(json!("persistent_value")));
        }
    }
    
    #[tokio::test]
    async fn test_transaction_rollback() {
        let manager = PersistentStateManager::new(StateConfig::default()).await.unwrap();
        
        manager.set("balance", json!(100)).await.unwrap();
        
        let tx_id = manager.begin_transaction().await.unwrap();
        let mut tx = StateTransaction::new(tx_id);
        tx.set("balance".to_string(), json!(50));
        
        // Rollback instead of commit
        manager.rollback_transaction(tx_id).await.unwrap();
        
        // Balance should remain unchanged
        let balance = manager.get("balance").await.unwrap();
        assert_eq!(balance, Some(json!(100)));
    }
}
```

## Related Documentation

- [llmspell-state-traits](llmspell-state-traits.md) - Core state trait definitions
- [llmspell-sessions](llmspell-sessions.md) - Session management with state
- [llmspell-security](llmspell-security.md) - Security and access control
- [llmspell-tenancy](llmspell-tenancy.md) - Multi-tenant isolation