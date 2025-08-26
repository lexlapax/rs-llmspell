# Backup Retention and Cleanup - Architectural Design

## Overview

Task 5.5.3 implements intelligent backup retention and cleanup within the existing `llmspell-state-persistence` crate, leveraging established patterns from the codebase.

## Key Discoveries from Codebase Analysis

### 1. **Module Location**
- Backup functionality lives in `llmspell-state-persistence/src/backup/` NOT `llmspell-core`
- Existing modules: `manager.rs`, `atomic.rs`, `compression.rs`, `events.rs`, `recovery.rs`
- New modules needed: `retention.rs`, `cleanup.rs`

### 2. **Existing Infrastructure to Leverage**

#### BackupConfig (already has retention fields!)
```rust
pub struct BackupConfig {
    pub max_backups: Option<usize>,        // Already exists!
    pub max_backup_age: Option<Duration>,  // Already exists!
    pub incremental_enabled: bool,
    pub full_backup_interval: Duration,
    // ... other fields
}
```

#### BackupManager Fields
```rust
pub struct BackupManager {
    config: BackupConfig,                  // Has retention config
    backup_index: Arc<RwLock<HashMap<BackupId, BackupMetadata>>>,  // Tracks all backups
    // ... other fields
}
```

#### TODO Comments Found
- `manager.rs:488`: `// TODO: Implement backup retention policy`
- `manager.rs:579`: `// TODO: Load existing backup metadata from disk`

### 3. **Event System Integration**

The codebase uses events extensively:
- `BackupEvent` already exists in `events.rs`
- EventBus pattern used throughout
- Need to emit events: `BackupDeleted`, `RetentionPolicyApplied`

### 4. **CLI Integration**

Existing backup commands in CLI:
- `backup create`
- `backup list`
- `backup restore`

Need to add:
- `backup cleanup` - Manual retention policy application
- `backup retention` - View/modify retention settings

### 5. **Architectural Patterns to Follow**

#### Policy-Based Design
```rust
pub trait RetentionPolicy: Send + Sync {
    fn should_retain(&self, backup: &BackupMetadata, all_backups: &[BackupMetadata]) -> bool;
    fn priority(&self) -> RetentionPriority;
}
```

#### Event-Driven Cleanup
- Emit events for all retention decisions
- Enable hooks to react to backup deletions
- Maintain audit trail

## Implementation Strategy

### Sub-tasks for 5.5.3

1. **5.5.3.1: Create Retention Policy Framework**
   - Define `RetentionPolicy` trait
   - Implement standard policies: `TimeBasedPolicy`, `CountBasedPolicy`, `ImportanceBasedPolicy`
   - Create `CompositePolicy` for combining multiple policies

2. **5.5.3.2: Implement Backup Importance Scoring**
   - Score backups based on: age, type (full vs incremental), restoration frequency
   - Preserve "checkpoint" backups (first of day/week/month)
   - Never delete the most recent full backup

3. **5.5.3.3: Build Cleanup Engine**
   - Integrate with existing `BackupManager`
   - Safe deletion with verification
   - Atomic operations to prevent partial cleanup

4. **5.5.3.4: Add Monitoring and Alerts**
   - Storage usage metrics
   - Emit events through EventBus
   - Hook integration for custom alerts

5. **5.5.3.5: Create CLI Commands**
   - `backup cleanup` - Apply retention policies manually
   - `backup retention show` - Display current retention config
   - `backup retention set` - Modify retention settings

## Design Details

### Retention Policy System

```rust
// retention.rs
#[derive(Debug, Clone)]
pub enum RetentionPriority {
    Critical,    // Never delete (most recent full backup)
    Important,   // Keep if possible (checkpoint backups)
    Standard,    // Normal retention rules apply
    Low,         // Delete first when space needed
}

pub trait RetentionPolicy: Send + Sync {
    fn evaluate(&self, backup: &BackupMetadata, context: &RetentionContext) -> RetentionDecision;
    fn name(&self) -> &str;
}

pub struct RetentionContext {
    pub all_backups: Vec<BackupMetadata>,
    pub total_size: u64,
    pub storage_limit: Option<u64>,
    pub current_time: SystemTime,
}

pub struct RetentionDecision {
    pub should_retain: bool,
    pub priority: RetentionPriority,
    pub reason: String,
}
```

### Integration with BackupManager

```rust
impl BackupManager {
    /// Apply retention policies and cleanup old backups
    pub async fn apply_retention_policies(&self) -> Result<RetentionReport> {
        let policies = self.load_retention_policies()?;
        let backups = self.list_all_backups().await?;
        let decisions = self.evaluate_retention(&backups, &policies)?;
        
        // Emit pre-cleanup event
        self.emit_event(BackupEvent::RetentionStarted { 
            backup_count: backups.len() 
        }).await?;
        
        let deleted = self.execute_cleanup(decisions).await?;
        
        // Emit post-cleanup event
        self.emit_event(BackupEvent::RetentionCompleted { 
            deleted_count: deleted.len() 
        }).await?;
        
        Ok(RetentionReport { deleted, retained: ... })
    }
}
```

### Safety Features

1. **Never Delete Critical Backups**
   - Most recent full backup
   - Only backup for a time period
   - Backups marked as "preserved"

2. **Atomic Operations**
   - Create deletion transaction log
   - Verify backup not in use
   - Update index after successful deletion

3. **Rollback Capability**
   - Keep deletion log for undo
   - Soft-delete with grace period
   - Emergency recovery option

## Testing Strategy

1. **Unit Tests**
   - Each retention policy in isolation
   - Importance scoring algorithm
   - Safety validation logic

2. **Integration Tests**  
   - Full retention cycle with BackupManager
   - Event emission verification
   - CLI command testing

3. **Edge Cases**
   - No backups to delete
   - All backups are critical
   - Storage limit exceeded significantly
   - Concurrent backup creation during cleanup

## Future Considerations

1. **Cloud Storage Integration** (Future Phase)
   - Archive old backups to cloud
   - Tiered storage policies
   - Cost-based retention

2. **Machine Learning** (Future Phase)
   - Predict which backups likely needed
   - Adaptive retention based on usage
   - Anomaly detection for backup patterns

## Summary

This design leverages existing infrastructure in `llmspell-state-persistence`, follows established patterns (event-driven, policy-based), and integrates seamlessly with the current backup system. The implementation will be safe, efficient, and extensible for future enhancements.