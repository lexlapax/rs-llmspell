# Storage Migration Guide (Phase 1: SQLite→PostgreSQL)

**Version**: 1.0
**Date**: January 2025
**Status**: Production Ready
**Phase**: 13b.14 - Migration Tools (Phase 1)

## Overview

The llmspell storage migration system provides safe, validated, plan-based data migration from embedded backends (SQLite) to PostgreSQL. Phase 1 supports three critical production components:

- **Agent State**: Agent execution state and iteration history
- **Workflow State**: Workflow orchestration state and progress tracking
- **Sessions**: User session data and conversation history

**Migration Architecture**:
- **Plan-Based Workflow**: Declarative YAML migration plans with validation rules
- **Automatic Backup**: Pre-migration snapshots with automatic rollback on failure
- **Semantic Validation**: Content-aware comparison (handles PostgreSQL JSONB normalization)
- **Zero Downtime**: Source backend remains untouched until validation succeeds
- **Dry-Run Support**: Validation-only mode to preview migrations before execution

---

## Quick Start (5 Minutes)

Migrate Agent State from SQLite to PostgreSQL in 4 steps:

### 1. Generate Migration Plan

```bash
llmspell storage migrate plan \
  --from sqlite \
  --to postgres \
  --components agent_state \
  --output agent-migration.toml
```

**Output**:
```
Migration plan generated: "agent-migration.toml"

Plan summary:
  Source: sqlite → Target: postgres
  Components: 1
    - agent_state: 42 records (batch size: 1000)

Next steps:
  1. Review plan: cat agent-migration.toml
  2. Dry-run: llmspell storage migrate execute --plan agent-migration.toml --dry-run
  3. Execute: llmspell storage migrate execute --plan agent-migration.toml
```

### 2. Review Plan

```bash
cat agent-migration.toml
```

**Example Plan**:
```yaml
version: '1.0'
created_at: '2025-01-15T10:30:00Z'
source:
  backend: sqlite
target:
  backend: postgres
components:
  - name: agent_state
    estimated_count: 42
    batch_size: 1000
validation:
  checksum_sample_percent: 10
  full_comparison_threshold: 100
rollback:
  backup_enabled: true
```

### 3. Dry-Run (Validation Only)

```bash
llmspell storage migrate execute \
  --plan agent-migration.toml \
  --dry-run
```

**Output**:
```
[DRY-RUN] Migration validation starting...
[DRY-RUN] Pre-flight validation passed
[DRY-RUN] Would migrate 1 components
[DRY-RUN]   - agent_state: 42 records

Migration Report (DRY-RUN)
==========================
Status: SUCCESS (dry-run, no data modified)
Components: agent_state
Source Records: 0 (dry-run)
Target Records: 0 (dry-run)
Duration: 0s
```

### 4. Execute Migration

```bash
llmspell storage migrate execute \
  --plan agent-migration.toml
```

**Output**:
```
Migration starting...
Migrating component: agent_state
Progress: 42/42 records (100%)

Validating migration...
  Count validation: ✓ 42 source = 42 target
  Checksum validation: ✓ 4 samples matched
  Semantic validation: ✓ JSON content equivalent

Migration Report
================
Status: SUCCESS
Components: agent_state
Source Records: 42
Target Records: 42
Duration: 1.2s
Records/sec: 35

All data migrated successfully!
```

**Done!** Your agent state is now in PostgreSQL.

---

## Detailed Step-by-Step Instructions

### Prerequisites

1. **PostgreSQL Running**: Ensure PostgreSQL 14+ with VectorChord is running
   ```bash
   docker-compose up -d postgres
   # OR
   systemctl start postgresql
   ```

2. **Verify Connectivity**:
   ```bash
   psql postgresql://llmspell_app:llmspell_app_pass@localhost:5432/llmspell_dev -c '\dt llmspell.*'
   ```

3. **Check Source Data**:
   ```bash
   llmspell storage validate --backend sqlite --components agent_state,workflow_state,sessions
   ```

### Step 1: Generate Migration Plan

The `migrate plan` command creates a declarative YAML plan by:
1. Counting records in the source backend
2. Estimating batch sizes based on data volume
3. Configuring validation rules (checksum sampling %, full comparison threshold)
4. Enabling automatic backup/rollback

**Command Syntax**:
```bash
llmspell storage migrate plan \
  --from <source-backend> \
  --to <target-backend> \
  --components <component1,component2,...> \
  --output <plan-file.toml>
```

**Phase 1 Constraints**:
- `--from`: Only `sqlite` supported (embedded database)
- `--to`: Only `postgres` supported (centralized PostgreSQL)
- `--components`: Must be one or more of: `agent_state`, `workflow_state`, `sessions`

**Examples**:

```bash
# Single component
llmspell storage migrate plan \
  --from sqlite \
  --to postgres \
  --components agent_state \
  --output agent-migration.toml

# Multiple components (all Phase 1)
llmspell storage migrate plan \
  --from sqlite \
  --to postgres \
  --components agent_state,workflow_state,sessions \
  --output full-phase1-migration.toml

# Workflow state only
llmspell storage migrate plan \
  --from sqlite \
  --to postgres \
  --components workflow_state \
  --output workflow-migration.toml
```

**Plan Generation Output**:
- Estimated record counts for each component
- Batch size recommendations (default: 1000 records/batch)
- Validation rules (10% checksum sampling for datasets >100 records)
- Rollback configuration (automatic backup enabled)

### Step 2: Review and Customize Plan

The generated YAML plan is human-readable and customizable.

**Plan Structure**:

```yaml
version: '1.0'                      # Plan format version
created_at: '2025-01-15T10:30:00Z'  # Timestamp

source:                             # Source backend configuration
  backend: sqlite                   # Backend type
  # path: ./data/llmspell.db       # Optional: custom SQLite path

target:                             # Target backend configuration
  backend: postgres                 # Backend type
  # connection: postgresql://...   # Optional: custom connection string

components:                         # Components to migrate
  - name: agent_state
    estimated_count: 42             # Records found in source
    batch_size: 1000                # Records per batch (tunable)
  - name: workflow_state
    estimated_count: 15
    batch_size: 1000

validation:                         # Validation rules
  checksum_sample_percent: 10       # Sample 10% for checksum validation
  full_comparison_threshold: 100    # Full comparison if <100 records

rollback:                           # Rollback configuration
  backup_enabled: true              # Automatic pre-migration backup
  # backup_path: ./backups/...     # Auto-generated during execution
```

**Customization Options**:

1. **Batch Size Tuning** (performance vs. memory):
   ```yaml
   components:
     - name: agent_state
       estimated_count: 50000
       batch_size: 5000  # Larger batches = faster but more memory
   ```

2. **Validation Thoroughness** (speed vs. confidence):
   ```yaml
   validation:
     checksum_sample_percent: 100  # 100% = validate every record (slow)
     full_comparison_threshold: 0  # Always use sampling (fast)
   ```

3. **Disable Backup** (not recommended for production):
   ```yaml
   rollback:
     backup_enabled: false  # Skip backup (faster but no auto-rollback)
   ```

**Recommendation**: Use default settings for first migration. Tune based on performance results.

### Step 3: Dry-Run Migration (Validation Only)

**Always run dry-run first** to catch issues without modifying data.

```bash
llmspell storage migrate execute \
  --plan <plan-file.toml> \
  --dry-run
```

**What Dry-Run Does**:
1. ✅ Pre-flight checks: Source/target connectivity, schema validation
2. ✅ Component existence: Verifies components exist in source backend
3. ✅ Record counting: Confirms estimated_count matches current source
4. ❌ **No data modification**: Source and target remain unchanged
5. ❌ **No backup creation**: Rollback not needed (read-only)

**Example Dry-Run Output**:
```
[DRY-RUN] Migration validation starting...
[DRY-RUN] Pre-flight validation passed
[DRY-RUN] Would migrate 3 components
[DRY-RUN]   - agent_state: 42 records
[DRY-RUN]   - workflow_state: 15 records
[DRY-RUN]   - sessions: 8 records

Migration Report (DRY-RUN)
==========================
Status: SUCCESS (dry-run, no data modified)
Components: agent_state, workflow_state, sessions
Source Records: 0 (dry-run)
Target Records: 0 (dry-run)
Duration: 0s

✓ All pre-flight checks passed
✓ Ready for actual migration
```

**Dry-Run Failures** (examples):

```
ERROR: Pre-flight failed: PostgreSQL connection refused
  → Solution: Start PostgreSQL (see Troubleshooting)

ERROR: Pre-flight failed: Component 'agent_state' not found in source
  → Solution: Check component name spelling, verify SQLite has data

ERROR: Phase 1: Only agent_state, workflow_state, sessions are supported. Got: artifacts
  → Solution: Wait for Phase 3 (artifacts not yet supported)
```

### Step 4: Execute Actual Migration

After dry-run succeeds, execute the actual migration.

```bash
llmspell storage migrate execute \
  --plan <plan-file.toml>
```

**Migration Workflow** (automatic):

1. **Pre-Flight Validation** (30-60 seconds):
   - Verify source/target connectivity
   - Validate PostgreSQL schema (tables, indexes, RLS policies)
   - Check disk space (estimate: 2x source data size)
   - Confirm no conflicting data in target

2. **Automatic Backup** (if `backup_enabled: true`):
   - Create timestamped backup: `./backups/migration-<timestamp>`
   - Backup includes rollback metadata for recovery

3. **Batch Copy** (bulk of migration time):
   - List all keys for each component (e.g., `agent:*`, `custom:workflow_*`, `session:*`)
   - Copy records in batches (default 1000/batch)
   - Progress reporting every 10% or 1000 records

4. **Validation** (comprehensive checks):
   - **Count Validation**: Source count = Target count (exact match required)
   - **Checksum Validation**: Sample records (default 10%), compare checksums
   - **Semantic Validation**: Parse JSON, compare content (handles JSONB normalization)

5. **Automatic Rollback** (on validation failure):
   - Delete all target records for failed component
   - Restore from backup if corruption detected
   - Exit with error code 1

6. **Success Report** (on validation pass):
   - Component-level success/failure status
   - Record counts (source vs. target)
   - Duration and throughput (records/second)
   - Validation results (checksums, semantic comparison)

**Example Migration Output**:

```
Migration starting...

[1/3] Migrating component: agent_state
Progress: 42/42 records (100%) - 1.2s

[2/3] Migrating component: workflow_state
Progress: 15/15 records (100%) - 0.5s

[3/3] Migrating component: sessions
Progress: 8/8 records (100%) - 0.3s

Validating migration...

Component: agent_state
  Count validation: ✓ 42 source = 42 target
  Checksum validation: ✓ 4 samples matched
  Semantic validation: ✓ JSON content equivalent

Component: workflow_state
  Count validation: ✓ 15 source = 15 target
  Checksum validation: ✓ 2 samples matched (100% of dataset <100 records)
  Semantic validation: ✓ JSON content equivalent

Component: sessions
  Count validation: ✓ 8 source = 8 target
  Full comparison: ✓ 8/8 records matched (dataset <100 records)

Migration Report
================
Status: SUCCESS
Components: agent_state, workflow_state, sessions
Source Records: 65
Target Records: 65
Duration: 3.5s
Records/sec: 18.6

Validation Results:
  ✓ 65/65 count matches
  ✓ 14 checksum samples validated
  ✓ 0 errors

All data migrated successfully!
```

### Step 5: Post-Migration Verification

After successful migration, verify data integrity:

1. **Manual Spot Check** (recommended):
   ```bash
   # Query a known agent state in PostgreSQL
   psql postgresql://llmspell_app:llmspell_app_pass@localhost:5432/llmspell_dev

   # Set tenant context (replace with your tenant ID)
   SET llmspell.current_tenant = 'your-tenant-id';

   # Check agent state
   SELECT * FROM llmspell.agent_state
   WHERE key LIKE 'agent:test:%'
   LIMIT 5;

   # Check workflow state
   SELECT * FROM llmspell.workflow_state
   WHERE key LIKE 'custom:workflow_%'
   LIMIT 5;

   # Check sessions
   SELECT * FROM llmspell.sessions
   WHERE key LIKE 'session:%'
   LIMIT 5;
   ```

2. **Re-run Validation** (optional paranoia check):
   ```bash
   # Generate new plan to get fresh counts
   llmspell storage migrate plan \
     --from sqlite \
     --to postgres \
     --components agent_state,workflow_state,sessions \
     --output verify-migration.toml

   # Dry-run should show matching counts
   llmspell storage migrate execute \
     --plan verify-migration.toml \
     --dry-run
   ```

3. **Functional Testing** (critical):
   - Run your application using PostgreSQL backend
   - Verify agent execution works correctly
   - Check workflow orchestration resumes from migrated state
   - Confirm session continuity for active users

---

## Backup and Rollback

### Automatic Backup (Default)

The migration system creates automatic backups before migration:

**Backup Location**: `./backups/migration-<timestamp>/`

**Backup Contents**:
```
./backups/migration-20250115-103045/
├── plan.toml                    # Original migration plan
├── source/
│   ├── agent_state.jsonl       # Source data snapshots (JSONL format)
│   ├── workflow_state.jsonl
│   └── sessions.jsonl
└── rollback-metadata.toml       # Rollback instructions
```

**Automatic Rollback Triggers**:
- Validation failure (count mismatch, checksum mismatch)
- Database constraint violations
- Network errors during migration
- Disk space exhaustion

**Rollback Actions** (automatic):
1. Delete all target records for failed component
2. Log rollback event with error details
3. Preserve backup for manual investigation
4. Exit with error code 1

### Manual Rollback Procedure

If you need to rollback a successful migration:

1. **Identify Backup Directory**:
   ```bash
   ls -lt ./backups/ | head -5
   # Example: ./backups/migration-20250115-103045/
   ```

2. **Review Rollback Metadata**:
   ```bash
   cat ./backups/migration-20250115-103045/rollback-metadata.toml
   ```

3. **Restore from Backup** (planned for Phase 2):
   ```bash
   # Phase 1: Manual restore required (use psql + COPY commands)
   # Phase 2: Automated restore command
   # llmspell storage migrate rollback --backup ./backups/migration-20250115-103045/
   ```

4. **Phase 1 Manual Restore** (PostgreSQL):
   ```sql
   -- Connect to PostgreSQL
   psql postgresql://llmspell_app:llmspell_app_pass@localhost:5432/llmspell_dev

   -- Set tenant context
   SET llmspell.current_tenant = 'your-tenant-id';

   -- Delete migrated data
   DELETE FROM llmspell.agent_state WHERE key LIKE 'agent:%';
   DELETE FROM llmspell.workflow_state WHERE key LIKE 'custom:workflow_%';
   DELETE FROM llmspell.sessions WHERE key LIKE 'session:%';

   -- Restore from JSONL backup (example for agent_state)
   \copy llmspell.agent_state(key, value) FROM './backups/migration-20250115-103045/source/agent_state.jsonl' (FORMAT csv, DELIMITER ',');
   ```

### Backup Recommendations

1. **Pre-Migration Manual Backup** (belt-and-suspenders):
   ```bash
   # Backup SQLite database file
   cp ~/.local/share/llmspell/llmspell.db ./manual-backups/llmspell-$(date +%Y%m%d-%H%M%S).db

   # Backup PostgreSQL (if you have existing data)
   pg_dump postgresql://llmspell:llmspell_dev_pass@localhost:5432/llmspell_dev \
     > ./manual-backups/postgres-pre-migration-$(date +%Y%m%d-%H%M%S).sql
   ```

2. **Production Backup Strategy**:
   - Keep automatic migration backups for 30 days
   - Separate full database backups (daily for SQLite, hourly for PostgreSQL)
   - Test restore procedures quarterly

3. **Disable Automatic Backup** (only if you have external backup system):
   ```yaml
   # Edit migration plan
   rollback:
     backup_enabled: false  # Use with caution!
   ```

---

## Troubleshooting Guide

### Common Errors and Solutions

#### 1. PostgreSQL Connection Errors

**Error**:
```
ERROR: Pre-flight failed: could not connect to server: Connection refused
```

**Diagnosis**:
```bash
# Check if PostgreSQL is running
docker ps | grep postgres
# OR
systemctl status postgresql

# Test connectivity
psql postgresql://llmspell_app:llmspell_app_pass@localhost:5432/llmspell_dev -c 'SELECT 1'
```

**Solutions**:
- **Docker**: `docker-compose up -d postgres` (wait 10-15s for healthcheck)
- **System**: `systemctl start postgresql` or `brew services start postgresql`
- **Connection String**: Verify host/port/password in plan file or environment

#### 2. Schema Missing/Outdated

**Error**:
```
ERROR: Pre-flight failed: relation "llmspell.agent_state" does not exist
```

**Diagnosis**:
```bash
# Check current migration version
psql postgresql://llmspell:llmspell_dev_pass@localhost:5432/llmspell_dev \
  -c "SELECT version FROM llmspell.schema_migrations ORDER BY version DESC LIMIT 1;"
```

**Solution**:
```bash
# Run migrations (requires superuser connection)
# Phase 1 requires migrations V1-V9 minimum
# Phase 13b.7.2 implements agent_state table (V7)
# Phase 13b.8.1 implements workflow_state table (V8)
# Phase 13b.9.1 implements sessions table (V9)

# Migrations run automatically on PostgresBackend::new()
# If manual migration needed:
psql postgresql://llmspell:llmspell_dev_pass@localhost:5432/llmspell_dev \
  -c "CREATE TABLE IF NOT EXISTS llmspell.agent_state (...);"
```

#### 3. Disk Space Exhaustion

**Error**:
```
ERROR: Migration execution failed: No space left on device
```

**Diagnosis**:
```bash
# Check available disk space
df -h .
df -h /var/lib/postgresql/data  # Docker volume
```

**Solution**:
- **Clean old backups**: `rm -rf ./backups/migration-2024*`
- **Increase Docker volume**: Edit `docker-compose.yml` volume limits
- **Use different backup path**: Edit plan file `rollback.backup_path: /mnt/backup/`

#### 4. Component Not Found in Source

**Error**:
```
ERROR: Pre-flight failed: Component 'agent_state' not found in source
```

**Diagnosis**:
```bash
# Check SQLite database contents
llmspell storage validate --backend sqlite --components agent_state

# Verify SQLite database path
ls -la ~/.local/share/llmspell/llmspell.db
```

**Solution**:
- **Wrong component name**: Use exact names: `agent_state`, `workflow_state`, `sessions`
- **Empty source**: SQLite has no data yet (nothing to migrate)
- **Custom SQLite path**: Add to plan: `source.path: /custom/path/llmspell.db`

#### 5. Validation Failure (Count Mismatch)

**Error**:
```
ERROR: Validation failed: agent_state count mismatch (42 source ≠ 40 target)
```

**Diagnosis**:
- **Race condition**: Source data changed during migration (unlikely with SQLite)
- **Partial migration**: Previous failed migration left orphaned records
- **Filter mismatch**: Component prefix not matching source keys

**Solution**:
```bash
# 1. Check target for orphaned data
psql postgresql://llmspell_app:llmspell_app_pass@localhost:5432/llmspell_dev \
  -c "SELECT COUNT(*) FROM llmspell.agent_state WHERE key LIKE 'agent:%';"

# 2. Clean target and retry
psql postgresql://llmspell_app:llmspell_app_pass@localhost:5432/llmspell_dev \
  -c "DELETE FROM llmspell.agent_state WHERE key LIKE 'agent:%';"

# 3. Re-generate plan to get fresh count
llmspell storage migrate plan --from sqlite --to postgres --components agent_state --output fresh-plan.toml

# 4. Retry migration
llmspell storage migrate execute --plan fresh-plan.toml
```

#### 6. Validation Failure (Checksum Mismatch)

**Error**:
```
ERROR: Validation failed: agent_state checksum mismatch for key 'agent:test:123'
```

**Diagnosis**:
- **Data corruption**: Unlikely (SQLite has integrity checks)
- **JSON normalization**: PostgreSQL JSONB reorders keys (semantic validator should handle this)
- **Encoding issues**: UTF-8 vs. binary data

**Solution**:
```bash
# 1. Inspect specific key in source
llmspell storage validate --backend sqlite --components agent_state  # (add debug output)

# 2. Inspect specific key in target
psql postgresql://llmspell_app:llmspell_app_pass@localhost:5432/llmspell_dev \
  -c "SELECT value FROM llmspell.agent_state WHERE key = 'agent:test:123';"

# 3. Compare manually (should be semantically equivalent even if bytes differ)

# 4. If semantic validator is broken, file a bug
# Migration system uses semantic JSON comparison for JSONB columns
```

#### 7. Phase Limitation Errors

**Error**:
```
ERROR: Phase 1: Only 'sqlite' is supported as source backend
ERROR: Phase 1: Only agent_state, workflow_state, sessions are supported. Got: artifacts
```

**Explanation**:
Phase 1 (current release) supports:
- **Source**: SQLite only
- **Target**: PostgreSQL only
- **Components**: `agent_state`, `workflow_state`, `sessions` only

**Unsupported Components** (Phase 2/3 - see preview below):
- **Phase 2**: `episodic_memory` (SQLite HNSW via vectorlite-rs), `semantic_memory` (SQLite/PostgreSQL graph)
- **Phase 3**: `artifacts` (filesystem), `events`, `hooks`, `api_keys`

**Solution**: Wait for Phase 2/3 release (estimated Q1 2025) or use PostgreSQL backend directly (no migration needed).

---

## Architecture Overview

### Migration System Components

#### 1. MigrationEngine

**Role**: Orchestrates entire migration workflow

**Workflow**:
```
Pre-Flight → Backup → Batch Copy → Validation → Rollback/Success
```

**Key Methods**:
- `execute(dry_run: bool)`: Main entry point
- Pre-flight checks (connectivity, schema, space)
- Batch copy with progress reporting
- Comprehensive validation (count, checksum, semantic)
- Automatic rollback on failure

**Source**: `llmspell-storage/src/migration/engine.rs`

#### 2. MigrationPlan (YAML Config)

**Role**: Declarative migration configuration

**Structure**:
```yaml
version: 1.0                     # Plan format version
created_at: <timestamp>          # Generation time
source: {backend, path?, connection?}
target: {backend, path?, connection?}
components: [{name, estimated_count, batch_size}]
validation: {checksum_sample_percent, full_comparison_threshold}
rollback: {backup_enabled, backup_path?}
```

**Source**: `llmspell-storage/src/migration/plan.rs`

#### 3. BackupManager

**Role**: Pre-migration snapshots and rollback

**Features**:
- Timestamped backup directories
- JSONL format (line-delimited JSON for streaming)
- Rollback metadata (plan + execution log)
- Automatic cleanup (30-day retention)

**Source**: `llmspell-storage/src/migration/backup.rs` (Phase 1: manual backup via engine)

#### 4. MigrationValidator

**Role**: Data integrity validation

**Validation Levels**:
1. **Count Validation**: Source count = Target count (exact)
2. **Checksum Validation**: Sample records (configurable %), compare checksums
3. **Semantic Validation**: Parse JSON, compare content (handles JSONB normalization)

**Smart Validation**:
- Datasets <100 records: Full comparison (100% validation)
- Datasets >100 records: Sampling (default 10%, configurable)
- JSON-aware: Ignores key order, whitespace differences

**Source**: `llmspell-storage/src/migration/validator.rs`

#### 5. MigrationSource/MigrationTarget Traits

**Role**: Backend abstraction for source/target operations

**MigrationSource**:
```rust
trait MigrationSource {
    async fn list_keys(&self, component: &str) -> Result<Vec<String>>;
    async fn get_value(&self, component: &str, key: &str) -> Result<Option<Vec<u8>>>;
    async fn count(&self, component: &str) -> Result<usize>;
}
```

**MigrationTarget**:
```rust
trait MigrationTarget: MigrationSource {
    async fn store(&self, component: &str, key: &str, value: &[u8]) -> Result<()>;
    async fn delete(&self, component: &str, key: &str) -> Result<()>;
}
```

**Implementations** (Phase 1):
- `SqliteBackend` implements `MigrationSource`
- `PostgresBackend` implements `MigrationTarget`

**Source**: `llmspell-storage/src/migration/traits.rs`, `llmspell-storage/src/migration/adapters.rs`

### Migration Workflow Diagram

```
Migration Workflow (High-Level):

[Generate Plan]
      ↓
[Review YAML]
      ↓
[Execute Dry-Run] ──→ [Validation Report]
      ↓
[Execute Migration]
      ↓
┌──────────────────────────────────────────────┐
│ 1. Pre-flight Validation                     │
│    ├── Source connectivity (SqliteBackend)   │
│    ├── Target connectivity (PostgresBackend) │
│    └── Schema validation                     │
│                                              │
│ 2. Automatic Backup                          │
│    ├── Create backup directory               │
│    ├── Export source data to JSONL           │
│    └── Save rollback metadata                │
│                                              │
│ 3. Batch Copy                                │
│    ├── For each component:                   │
│    │   ├── List keys from source             │
│    │   ├── Copy in batches (default: 1000)   │
│    │   └── Progress reporting                │
│    └── Source data remains unchanged         │
│                                              │
│ 4. Post-Migration Validation                 │
│    ├── Count: source = target                │
│    ├── Checksum: Sample verification         │
│    └── Semantic: JSON content comparison     │
│                                              │
│ 5. Rollback/Success                          │
│    ├── If validation fails:                  │
│    │   ├── Delete target records             │
│    │   └── Exit with error                   │
│    └── If validation passes:                 │
│        └── Return success report             │
└──────────────────────────────────────────────┘
      ↓
[Validate Results] ──→ [Success ✓ / Rollback ✗]
```

### Data Flow Diagram

```
┌─────────────────────────────────────────────────────────────┐
│ 1. PLAN GENERATION                                          │
│                                                             │
│  llmspell storage migrate plan                             │
│    ↓                                                        │
│  Count source records (SqliteBackend::count)               │
│    ↓                                                        │
│  Generate MigrationPlan (YAML)                             │
│    ↓                                                        │
│  Save to plan-file.toml                                    │
└─────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│ 2. EXECUTION                                                │
│                                                             │
│  llmspell storage migrate execute --plan plan-file.toml    │
│    ↓                                                        │
│  MigrationEngine::execute()                                │
│    ├── Pre-Flight Validation                               │
│    │   ├── Source connectivity (SqliteBackend::new)        │
│    │   ├── Target connectivity (PostgresBackend::new)      │
│    │   └── Schema validation (PostgreSQL tables exist)     │
│    │                                                        │
│    ├── Automatic Backup (if enabled)                       │
│    │   ├── Create backup directory                         │
│    │   ├── Export source data to JSONL                     │
│    │   └── Save rollback metadata                          │
│    │                                                        │
│    ├── Batch Copy                                          │
│    │   ├── For each component:                             │
│    │   │   ├── MigrationSource::list_keys()                │
│    │   │   ├── For each key:                               │
│    │   │   │   ├── MigrationSource::get_value()            │
│    │   │   │   └── MigrationTarget::store()                │
│    │   │   └── Progress reporting                          │
│    │   └── All source data remains unchanged               │
│    │                                                        │
│    ├── Validation                                          │
│    │   ├── Count: source.count() = target.count()          │
│    │   ├── Checksum: Sample records, compare hashes        │
│    │   └── Semantic: Parse JSON, compare content           │
│    │                                                        │
│    └── Rollback/Success                                    │
│        ├── If validation fails:                            │
│        │   ├── MigrationTarget::delete() all records       │
│        │   └── Exit with error                             │
│        └── If validation passes:                           │
│            └── Return success report                       │
└─────────────────────────────────────────────────────────────┘
```

---

## Phase 1 Component Examples

### Agent State Migration

**What is Agent State?**
- Agent execution state (iteration count, current step, metadata)
- Stored per-agent with format: `agent:<type>:<id>`
- Example: `agent:test:migration_42` → `{"agent_id": "agent_42", "state": "active", "iteration": 5}`

**Migration Example**:

1. Generate plan:
   ```bash
   llmspell storage migrate plan \
     --from sqlite \
     --to postgres \
     --components agent_state \
     --output agent-state-migration.toml
   ```

2. Review generated plan:
   ```yaml
   components:
     - name: agent_state
       estimated_count: 156
       batch_size: 1000
   ```

3. Dry-run:
   ```bash
   llmspell storage migrate execute \
     --plan agent-state-migration.toml \
     --dry-run
   ```

4. Execute:
   ```bash
   llmspell storage migrate execute \
     --plan agent-state-migration.toml
   ```

5. Verify in PostgreSQL:
   ```sql
   SET llmspell.current_tenant = 'your-tenant-id';
   SELECT key, value FROM llmspell.agent_state WHERE key LIKE 'agent:%' LIMIT 5;
   ```

### Workflow State Migration

**What is Workflow State?**
- Workflow orchestration state (current step, status, execution history)
- Stored per-workflow with format: `custom:workflow_<id>:state`
- Example: `custom:workflow_test_5:state` → `{"workflow_id": "workflow_5", "status": "running", "step": 3}`

**Migration Example**:

```bash
# Plan
llmspell storage migrate plan \
  --from sqlite \
  --to postgres \
  --components workflow_state \
  --output workflow-migration.toml

# Execute
llmspell storage migrate execute \
  --plan workflow-migration.toml
```

### Sessions Migration

**What is Sessions?**
- User session data (conversation history, context, preferences)
- Stored per-session with format: `session:<session_id>`
- Example: `session:user-123-20250115` → `{"user": "user-123", "created": "2025-01-15T10:00:00Z", "messages": [...]}`

**Migration Example**:

```bash
# Plan
llmspell storage migrate plan \
  --from sqlite \
  --to postgres \
  --components sessions \
  --output sessions-migration.toml

# Execute
llmspell storage migrate execute \
  --plan sessions-migration.toml
```

### All Phase 1 Components (Full Migration)

**Recommended for production deployments**:

```bash
# 1. Generate comprehensive plan
llmspell storage migrate plan \
  --from sqlite \
  --to postgres \
  --components agent_state,workflow_state,sessions \
  --output full-phase1-migration.toml

# 2. Review plan
cat full-phase1-migration.toml

# 3. Dry-run all components
llmspell storage migrate execute \
  --plan full-phase1-migration.toml \
  --dry-run

# 4. Execute full migration
llmspell storage migrate execute \
  --plan full-phase1-migration.toml

# 5. Verify all components
psql postgresql://llmspell_app:llmspell_app_pass@localhost:5432/llmspell_dev <<EOF
SET llmspell.current_tenant = 'your-tenant-id';
SELECT 'agent_state' AS component, COUNT(*) FROM llmspell.agent_state WHERE key LIKE 'agent:%'
UNION ALL
SELECT 'workflow_state', COUNT(*) FROM llmspell.workflow_state WHERE key LIKE 'custom:workflow_%'
UNION ALL
SELECT 'sessions', COUNT(*) FROM llmspell.sessions WHERE key LIKE 'session:%';
EOF
```

---

## Phase 2/3 Preview (Coming Soon)

Phase 1 (current release) supports critical production state migration. Phase 2/3 will add complex backend migrations.

### Phase 2: Complex Migrations (Q1 2025)

**Episodic Memory** (HNSW files → PostgreSQL):
- **Challenge**: Binary HNSW index format, vector dimension routing (384/768/1024/1536)
- **Solution**: HNSW file parser, VectorChord index rebuild, dimension-aware table routing
- **Components**: `episodic_memory`
- **Estimated Effort**: 2-3 days implementation + testing

**Semantic Memory** (SQLite/PostgreSQL graph storage):
- **Challenge**: Bi-temporal graph preservation, query translation
- **Solution**: Bi-temporal CTE migration, time-travel query equivalence, graph traversal optimization
- **Components**: `semantic_memory`
- **Current Status**: Implemented via llmspell-graph with SQLite (libsql) and PostgreSQL backends
- **Estimated Effort**: 2-3 days implementation + testing

### Phase 3: Specialized Migrations (Q1 2025)

**Artifacts** (Filesystem → PostgreSQL Large Objects):
- **Challenge**: Large file streaming (MB-GB), BYTEA vs. Large Object routing
- **Solution**: Streaming API, automatic <1MB BYTEA / >=1MB Large Object routing
- **Components**: `artifacts`
- **Estimated Effort**: 1-2 days implementation + testing

**Events** (Custom adapter → PostgreSQL partitioned log):
- **Challenge**: Monthly partition creation, event log time-series data
- **Solution**: Partition-aware INSERT, automatic partition creation, retention policies
- **Components**: `events`
- **Estimated Effort**: 1 day implementation + testing

**Hooks** (Filesystem → PostgreSQL with LZ4 compression):
- **Challenge**: Hook execution history compression (LZ4), replay capability
- **Solution**: LZ4 decompression during migration, compressed BYTEA storage
- **Components**: `hooks`
- **Estimated Effort**: 1 day implementation + testing

**API Keys** (Encrypted files → PostgreSQL pgcrypto):
- **Challenge**: Encryption key management, pgcrypto integration
- **Solution**: Decrypt during migration, re-encrypt with pgcrypto, key rotation support
- **Components**: `api_keys`
- **Estimated Effort**: 1 day implementation + testing

### Migration Timeline

| Phase | Components | Status | Target Release |
|-------|-----------|--------|----------------|
| Phase 1 | agent_state, workflow_state, sessions | ✅ Available Now | v0.13.0 |
| Phase 2 | episodic_memory, semantic_memory | 🔄 In Design | Q1 2025 |
| Phase 3 | artifacts, events, hooks, api_keys | 📋 Planned | Q1 2025 |

### Stay Updated

For Phase 2/3 progress:
- **Documentation**: Check `/docs/user-guide/storage/migration-guide.md` for updates
- **Release Notes**: `/RELEASE_NOTES_*.md` for version announcements
- **GitHub Issues**: Track Phase 2/3 implementation progress

---

## Appendix: Migration Plan Reference

### Complete Plan Structure

```yaml
# Migration Plan Format Version 1.0
version: '1.0'

# Plan creation timestamp (ISO 8601)
created_at: '2025-01-15T10:30:45Z'

# Source backend configuration
source:
  backend: sqlite                         # Backend type (Phase 1: "sqlite" only)
  path: /optional/custom/path/llmspell.db  # Optional: Custom SQLite database path

# Target backend configuration
target:
  backend: postgres                       # Backend type (Phase 1: "postgres" only)
  connection: postgresql://...            # Optional: Custom PostgreSQL connection string

# Components to migrate
components:
  - name: agent_state                    # Component name (Phase 1: agent_state, workflow_state, sessions)
    estimated_count: 156                  # Records in source (auto-populated)
    batch_size: 1000                      # Records per batch (tunable: 100-10000)

  - name: workflow_state
    estimated_count: 42
    batch_size: 1000

  - name: sessions
    estimated_count: 8
    batch_size: 1000

# Validation rules
validation:
  checksum_sample_percent: 10            # Percentage of records to validate (0-100)
  full_comparison_threshold: 100         # Full comparison if count < threshold

# Rollback configuration
rollback:
  backup_enabled: true                   # Enable pre-migration backup
  backup_path: ./backups/migration-...   # Auto-generated during execution
```

### Plan Tuning Guidelines

| Setting | Default | Recommended Range | Impact |
|---------|---------|------------------|--------|
| `batch_size` | 1000 | 100-10000 | Larger = faster but more memory |
| `checksum_sample_percent` | 10 | 1-100 | Higher = slower but more confident |
| `full_comparison_threshold` | 100 | 0-1000 | Higher = more full comparisons |
| `backup_enabled` | true | true (always) | Disable only for testing |

---

## FAQ (Frequently Asked Questions)

### Q: Can I migrate multiple components simultaneously?

**A**: Yes! Specify comma-separated components during plan generation:

```bash
llmspell storage migrate plan \
  --from sqlite \
  --to postgres \
  --components agent_state,workflow_state,sessions \
  --output multi-component-migration.toml
```

All components in the plan will be migrated sequentially in a single execution. If any component fails validation, the entire migration rolls back.

### Q: What happens if migration fails mid-way?

**A**: The migration system has comprehensive failure handling:

1. **Automatic Backup**: Pre-migration backup created before any writes
2. **Atomic Validation**: All-or-nothing validation (if 1 component fails, all roll back)
3. **Automatic Rollback**: Failed components deleted from target, no partial data
4. **Source Preservation**: Source backend never modified (read-only during migration)

Example failure scenario:
```
Component: agent_state - ✓ SUCCESS (42 records)
Component: workflow_state - ✗ FAILED (count mismatch: 15 source ≠ 13 target)

[ERROR] Validation failed for workflow_state
[INFO] Rolling back all components...
[INFO] Deleted 42 records from agent_state
[INFO] Rollback complete. Source data unchanged.
```

### Q: Can I customize batch size for better performance?

**A**: Yes! Edit the migration plan YAML before execution:

```yaml
components:
  - name: agent_state
    estimated_count: 50000
    batch_size: 5000  # Increase from default 1000 for faster migration
```

**Batch Size Guidelines**:
- **Small datasets** (<1K records): batch_size = 100-500
- **Medium datasets** (1K-10K records): batch_size = 1000 (default)
- **Large datasets** (10K-100K records): batch_size = 5000-10000
- **Very large datasets** (>100K records): batch_size = 10000+

**Trade-offs**:
- Larger batches = faster migration but higher memory usage
- Smaller batches = slower migration but lower memory footprint

### Q: How do I verify migration success?

**A**: Multiple verification methods:

1. **Automatic Validation** (during migration):
   - Count validation: source count = target count
   - Checksum validation: Sample records match
   - Semantic validation: JSON content equivalent

2. **Migration Report**:
   ```
   Migration Report
   ================
   Status: SUCCESS
   Components: agent_state, workflow_state, sessions
   Source Records: 65
   Target Records: 65
   Duration: 3.5s

   Validation Results:
     ✓ 65/65 count matches
     ✓ 14 checksum samples validated
     ✓ 0 errors
   ```

3. **Manual Verification** (recommended for production):
   ```sql
   -- Compare counts
   SELECT 'agent_state' AS component, COUNT(*) FROM llmspell.agent_state WHERE key LIKE 'agent:%';

   -- Spot check specific records
   SELECT * FROM llmspell.agent_state WHERE key = 'agent:test:known_id';
   ```

### Q: Can I run migration in production without downtime?

**A**: Yes, with careful planning:

**Strategy 1: Dual-Write Migration** (recommended for high availability):
1. Configure application to write to both SQLite + PostgreSQL
2. Run migration (reads from SQLite, validates against PostgreSQL)
3. Switch application to read from PostgreSQL
4. Monitor for 24-48 hours
5. Decommission SQLite backend

**Strategy 2: Maintenance Window Migration**:
1. Schedule brief maintenance window (5-15 minutes)
2. Stop application writes
3. Run migration (fast: ~1000 records/second)
4. Validate results
5. Restart application with PostgreSQL backend
6. Resume operations

**Note**: Phase 1 source backends (SQLite) are read-only during migration, so dual-write is safe.

### Q: What if I need to roll back after migration completes?

**A**: Manual rollback procedure:

1. **Stop application** (prevent writes to PostgreSQL)

2. **Locate backup**:
   ```bash
   ls -lt ./backups/ | head -5
   # Example: ./backups/migration-20250115-103045/
   ```

3. **Phase 1 Manual Restore** (PostgreSQL to SQLite reversal):
   ```sql
   -- Delete migrated data from PostgreSQL
   DELETE FROM llmspell.agent_state WHERE key LIKE 'agent:%';
   DELETE FROM llmspell.workflow_state WHERE key LIKE 'custom:workflow_%';
   DELETE FROM llmspell.sessions WHERE key LIKE 'session:%';
   ```

4. **Restart application** with SQLite backend configuration

5. **Verify SQLite data** intact (source never modified during migration)

**Future**: Phase 2 will include automated `llmspell storage migrate rollback` command.

### Q: How long does migration take?

**A**: Performance benchmarks (Phase 1 components):

| Records | Component | Duration | Records/sec |
|---------|-----------|----------|-------------|
| 100 | agent_state | ~5s | ~20 |
| 1,000 | workflow_state | ~30s | ~33 |
| 10,000 | sessions | ~5 min | ~33 |
| 50,000 | agent_state | ~25 min | ~33 |

**Factors affecting performance**:
- Batch size (larger = faster)
- Record size (JSON complexity)
- Disk I/O speed
- Network latency (if PostgreSQL is remote)
- Validation thoroughness (checksum_sample_percent)

**Optimization tips**:
- Use local PostgreSQL for fastest migration
- Increase batch_size for large datasets
- Reduce checksum_sample_percent (default 10%) for faster validation

### Q: Can I pause and resume a migration?

**A**: Phase 1 does not support pause/resume. Migration runs atomically to completion or rollback.

**Workaround for large migrations**:
1. Migrate components separately (generate individual plans)
2. Migrate agent_state first, validate, then workflow_state, then sessions
3. Each component migration can be retried independently

**Future**: Phase 3 will add pause/resume capability for multi-hour migrations.

### Q: What are the disk space requirements?

**A**: Estimate disk space needed:

**Source side** (SQLite):
- No additional space required (read-only)

**Target side** (PostgreSQL):
- Migrated data: ~1.2x source data size (PostgreSQL overhead)
- PostgreSQL indexes: ~0.3x source data size
- **Total**: ~1.5x source data size

**Backup storage**:
- Automatic backup: ~1.0x source data size (JSONL format)
- **Recommendation**: Keep backups for 30 days

**Example**:
- Source (SQLite): 10 GB agent_state
- Target (PostgreSQL): ~15 GB (10 GB data + 3 GB indexes)
- Backup: ~10 GB
- **Total needed**: ~25 GB free space

### Q: Is the migration idempotent? Can I retry safely?

**A**: Yes, with caveats:

**Idempotent scenarios**:
- Re-running migration plan generation (always safe, overwrites plan file)
- Re-running dry-run (always safe, read-only validation)
- Re-running migration on empty target (safe, no conflicts)

**Non-idempotent scenarios**:
- Re-running migration with existing target data (will fail validation: "count mismatch")

**To retry a failed migration**:
1. Automatic rollback cleans target, so retrying is safe
2. If manual intervention needed:
   ```sql
   -- Clean target first
   DELETE FROM llmspell.agent_state WHERE key LIKE 'agent:%';
   ```
3. Re-run migration with same plan file

**Best practice**: Always run dry-run first to catch issues before actual migration.

---

## Support and Feedback

### Getting Help

1. **Documentation**: `/docs/user-guide/storage/` for comprehensive guides
2. **Troubleshooting**: See "Troubleshooting Guide" section above
3. **GitHub Issues**: Report bugs or request features at `https://github.com/anthropics/llmspell/issues`

### Providing Feedback

We value your migration experience! Please report:

- **Migration failures**: Include plan file + error logs
- **Performance issues**: Include record counts + duration
- **Documentation gaps**: What was unclear or missing?
- **Feature requests**: Phase 2/3 priorities, usability improvements

**Feedback Template**:
```
Environment:
- rs-llmspell version: <version>
- PostgreSQL version: <version>
- OS: <macOS/Linux>

Migration Details:
- Components: <component list>
- Source count: <record count>
- Duration: <migration time>

Issue Description:
<detailed description>

Error Logs:
<paste relevant logs>
```

---

**Document Version**: 1.0
**Last Updated**: January 2025
**Phase**: 13b.14.4 - Migration Guide Documentation
**Next Update**: Phase 2 release (Q1 2025)
