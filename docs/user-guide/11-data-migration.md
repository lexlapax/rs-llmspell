# Data Migration Guide

**Complete guide for PostgreSQL ↔ SQLite data migration**

🔗 **Navigation**: [← User Guide](README.md) | [Storage Setup](07-storage-setup.md) | [CLI Reference](05-cli-reference.md#storage)

---

## Overview

llmspell provides built-in tools for lossless bidirectional migration of all data between PostgreSQL and SQLite storage backends. This enables flexible deployment strategies, environment-specific optimizations, and seamless transitions as your needs evolve.

**What You'll Learn:**
- When and why to migrate between backends
- Step-by-step migration workflows for common scenarios
- Verification procedures to ensure data integrity
- Troubleshooting strategies for migration issues
- Best practices for production migrations

**Migration Capabilities:**
- ✅ Lossless roundtrip migration (zero data loss)
- ✅ All 10 table types included (vectors via vectorlite-rs HNSW, graph, sessions, etc.)
- ✅ Transaction-safe import with automatic rollback
- ✅ Versioned JSON format for compatibility
- ✅ Base64 encoding for binary data preservation (including BLOB vector embeddings)
- ✅ Detailed import statistics for verification

---

## When to Migrate

### Common Migration Scenarios

#### 1. Development → Production
**Scenario**: You've developed and tested locally with SQLite, and now want to deploy to a production PostgreSQL database.

**Why Migrate**:
- Multi-instance deployments require shared database
- Team collaboration needs centralized data
- Scaling beyond single-process limitations

**Migration Path**: SQLite → PostgreSQL

---

#### 2. Production → Development
**Scenario**: You need to debug a production issue locally or test new features with real data.

**Why Migrate**:
- Local testing with production-like data
- Debugging specific customer issues
- Offline development without database dependency

**Migration Path**: PostgreSQL → SQLite

---

#### 3. Environment Synchronization
**Scenario**: Sync data between staging and production environments, or between team members.

**Why Migrate**:
- Consistent test data across environments
- Reproducing production issues in staging
- Sharing dataset for collaborative development

**Migration Path**: PostgreSQL ↔ PostgreSQL or SQLite ↔ SQLite (via JSON)

---

#### 4. Scaling Decisions
**Scenario**: Your deployment needs have changed (scaling up or down).

**Why Migrate**:
- **Scale Up**: SQLite → PostgreSQL when moving from single-server to distributed
- **Scale Down**: PostgreSQL → SQLite when consolidating to embedded deployment

**Migration Path**: Bidirectional (SQLite ↔ PostgreSQL)

---

#### 5. Disaster Recovery
**Scenario**: Primary database is corrupted or lost, need to restore from export.

**Why Migrate**:
- Database server failure
- Data corruption
- Accidental deletion

**Migration Path**: JSON export → Any backend

---

## Prerequisites

### Before You Begin

**1. Install llmspell with appropriate features**

```bash
# For SQLite only
cargo build --release

# For PostgreSQL support
cargo build --release --features postgres
```

**2. Verify source backend is accessible**

```bash
# SQLite: Check database file exists
ls -lh ~/.local/share/llmspell/llmspell.db

# PostgreSQL: Test connection
export DATABASE_URL="postgresql://user:pass@localhost/llmspell"
psql $DATABASE_URL -c "SELECT version();"
```

**3. Ensure target backend is running**

```bash
# PostgreSQL: Start if using Docker
docker compose -f docker/postgres/docker-compose.yml up -d

# Verify target is accessible
llmspell database health --backend postgres
```

**4. Apply migrations to target backend**

```bash
# PostgreSQL
llmspell database migrate

# SQLite (automatic on first connection)
```

**5. Check available disk space**

```bash
# Estimate export size (roughly matches source DB size)
du -sh ~/.local/share/llmspell/llmspell.db
# Ensure target has at least 2x this space available
```

---

## Migration Workflows

### Workflow 1: SQLite → PostgreSQL (Development to Production)

**Most common migration path for deploying to production**

#### Step 1: Export from SQLite

```bash
# Export from local SQLite database
llmspell storage export --backend sqlite --output dev-data.json

# Verify export succeeded
ls -lh dev-data.json
jq '.version, .source_backend, (.data | keys)' dev-data.json
```

**Expected output:**
```json
"1.0"
"sqlite"
[
  "vector_embeddings",
  "knowledge_graph",
  "procedural_memory",
  "agent_state",
  "sessions",
  "artifacts"
]
```

#### Step 2: Transfer export file to production server

```bash
# Using scp
scp dev-data.json prod-server:/tmp/llmspell-import.json

# Or using rsync (with compression)
rsync -avz dev-data.json prod-server:/tmp/llmspell-import.json

# Or using cloud storage (AWS S3 example)
aws s3 cp dev-data.json s3://my-bucket/migrations/
# On production server:
aws s3 cp s3://my-bucket/migrations/dev-data.json /tmp/llmspell-import.json
```

#### Step 3: Import to PostgreSQL

```bash
# On production server, set PostgreSQL connection
export DATABASE_URL="postgresql://llmspell:password@localhost:5432/llmspell_prod"

# Import data
llmspell storage import --backend postgres --input /tmp/llmspell-import.json
```

**Expected output:**
```
Importing data into postgres backend from /tmp/llmspell-import.json...
Reading JSON file...
✅ Imported 1,234 total records:
  - Vectors: 500
  - Entities: 120
  - Relationships: 85
  - Sessions: 15
  - Artifacts: 200
  - Events: 314
```

#### Step 4: Verify migration

```bash
# Export from PostgreSQL
llmspell storage export --backend postgres --output prod-verify.json

# Compare data sections (timestamps will differ)
diff <(jq -S .data dev-data.json) <(jq -S .data prod-verify.json)

# Should output nothing (identical data)
```

#### Step 5: Test application

```bash
# Run integration tests against new backend
cargo test --test integration_tests -- --test-threads=1

# Or manually test critical workflows
llmspell exec "return Memory.search('test query')"
```

---

### Workflow 2: PostgreSQL → SQLite (Production to Development)

**Useful for debugging production issues locally**

#### Step 1: Export from PostgreSQL

```bash
# On production server
export DATABASE_URL="postgresql://llmspell:password@localhost:5432/llmspell_prod"

# Create export with timestamp
EXPORT_FILE="prod-export-$(date +%Y%m%d-%H%M%S).json"
llmspell storage export --backend postgres --output $EXPORT_FILE

# Verify export
ls -lh $EXPORT_FILE
```

#### Step 2: Transfer to development machine

```bash
# From production server to dev machine
scp prod-server:/path/to/prod-export-*.json ~/Downloads/

# Or download from cloud storage
aws s3 cp s3://my-bucket/prod-export-20250122.json ~/Downloads/
```

#### Step 3: Import to local SQLite

```bash
# On dev machine
llmspell storage import --backend sqlite --input ~/Downloads/prod-export-20250122.json
```

**Note**: SQLite database location is determined by config:
```toml
# config.toml
[storage.sqlite]
path = "~/.local/share/llmspell/llmspell.db"
```

#### Step 4: Verify and test

```bash
# Verify import statistics
# (Should match export record counts)

# Test queries locally
llmspell memory search "production data"
llmspell session list
```

---

### Workflow 3: Cross-Environment Sync (Staging ↔ Production)

**Synchronize data between environments for testing**

#### Staging → Production

```bash
# 1. Export from staging
export DATABASE_URL="postgresql://user:pass@staging-db:5432/llmspell_staging"
llmspell storage export --backend postgres --output staging-data.json

# 2. Import to production
export DATABASE_URL="postgresql://user:pass@prod-db:5432/llmspell_prod"
llmspell storage import --backend postgres --input staging-data.json
```

#### Production → Staging (with data sanitization)

```bash
# 1. Export from production
export DATABASE_URL="postgresql://user:pass@prod-db:5432/llmspell_prod"
llmspell storage export --backend postgres --output prod-data.json

# 2. Sanitize sensitive data (optional)
# Example: Remove specific sessions or sanitize metadata
jq 'del(.data.sessions[] | select(.tenant_id == "sensitive-tenant"))' \
   prod-data.json > staging-data.json

# 3. Import to staging
export DATABASE_URL="postgresql://user:pass@staging-db:5432/llmspell_staging"
llmspell storage import --backend postgres --input staging-data.json
```

---

### Workflow 4: Disaster Recovery

**Restore from JSON backup after database failure**

#### Scenario: PostgreSQL database lost/corrupted

```bash
# 1. Verify backup exists
ls -lh /var/backups/llmspell/backup-20250120.json

# 2. Rebuild database (if needed)
docker compose -f docker/postgres/docker-compose.yml down -v
docker compose -f docker/postgres/docker-compose.yml up -d

# 3. Apply migrations
llmspell database migrate

# 4. Restore from backup
export DATABASE_URL="postgresql://llmspell:password@localhost:5432/llmspell_prod"
llmspell storage import --backend postgres --input /var/backups/llmspell/backup-20250120.json

# 5. Verify restoration
llmspell database health
llmspell session list
```

---

### Workflow 5: Backend Testing & Validation

**Verify data consistency across backends**

#### Multiple roundtrip test

```bash
# Round 1: SQLite → JSON → PostgreSQL
llmspell storage export --backend sqlite --output round1-export.json
export DATABASE_URL="postgresql://user:pass@localhost/llmspell_test"
llmspell storage import --backend postgres --input round1-export.json

# Round 2: PostgreSQL → JSON → SQLite
llmspell storage export --backend postgres --output round2-export.json
llmspell storage import --backend sqlite --input round2-export.json

# Round 3: SQLite → JSON (verify consistency)
llmspell storage export --backend sqlite --output round3-export.json

# Compare all rounds (should be identical)
diff <(jq -S .data round1-export.json) <(jq -S .data round2-export.json)
diff <(jq -S .data round2-export.json) <(jq -S .data round3-export.json)
```

---

## Verification Procedures

### Essential Verification Steps

After any migration, perform these verification steps:

#### 1. Check Import Statistics

```bash
llmspell storage import --backend postgres --input data.json
# ✅ Imported 1,234 total records:
#   - Vectors: 500
#   - Entities: 120
#   - Relationships: 85
#   - Patterns: 45
#   - Agent States: 8
#   - KV Entries: 12
#   - Workflow States: 3
#   - Sessions: 15
#   - Artifact Content: 180
#   - Artifacts: 200
#   - Events: 250
#   - Hooks: 16
```

**Verify counts match expectations from source database.**

#### 2. Export-Compare-Verify

```bash
# Export from source
llmspell storage export --backend sqlite --output source.json

# Export from target
llmspell storage export --backend postgres --output target.json

# Compare data (ignore timestamps in metadata)
diff <(jq -S .data source.json) <(jq -S .data target.json)

# Success: No output (files identical)
# Failure: Shows differences
```

#### 3. Functional Testing

```bash
# Test vector search
llmspell exec "return Memory.search('test query', {limit = 5})"

# Test knowledge graph queries
llmspell memory query "entity type"

# Test session access
llmspell session list

# Test artifact retrieval
llmspell exec "return Session.get_artifacts('session-id')"
```

#### 4. Data Integrity Checks

```bash
# PostgreSQL: Run integrity checks
psql $DATABASE_URL <<EOF
-- Check for orphaned relationships (should be 0)
SELECT COUNT(*) FROM graph_relationships r
LEFT JOIN graph_entities e ON r.source_entity_id = e.entity_id
WHERE e.entity_id IS NULL;

-- Check for orphaned artifacts (should be 0)
SELECT COUNT(*) FROM artifacts a
LEFT JOIN artifact_content c ON a.content_hash = c.content_hash
WHERE c.content_hash IS NULL;
EOF
```

#### 5. Performance Validation

```bash
# Run performance tests
cargo test --test performance_tests --release

# Check vector search performance
llmspell exec "
local start = os.clock()
local results = Memory.search('performance test', {limit = 100})
local duration = os.clock() - start
print(string.format('Search took %.3f ms', duration * 1000))
return results
"
```

---

## Troubleshooting

### Common Issues and Solutions

#### Issue 1: Export File Too Large

**Problem:**
```
Error: Ran out of disk space writing export file
```

**Solutions:**

```bash
# 1. Check available space
df -h /tmp

# 2. Use compression
llmspell storage export --backend postgres --output /tmp/export.json
gzip /tmp/export.json  # Creates export.json.gz

# 3. Export to larger partition
llmspell storage export --backend postgres --output /mnt/large-disk/export.json

# 4. Stream export (for very large databases)
# Export directly to compressed format
llmspell storage export --backend postgres --output - | gzip > export.json.gz
```

---

#### Issue 2: Import Fails with JSON Parse Error

**Problem:**
```
Error: Failed to parse export JSON
Caused by: expected value at line 1 column 1
```

**Solutions:**

```bash
# 1. Verify JSON is valid
jq . export.json > /dev/null && echo "Valid JSON" || echo "Invalid JSON"

# 2. Check file not truncated
tail -1 export.json  # Should end with }

# 3. Re-export if corrupted
llmspell storage export --backend sqlite --output fresh-export.json

# 4. Validate export format version
jq '.version' export.json  # Should be "1.0"
```

---

#### Issue 3: Import Fails Partway Through

**Problem:**
```
Error: Failed to import relationships
Caused by: Foreign key constraint violation
```

**Cause**: Import order issues or data integrity problems.

**Solutions:**

```bash
# 1. Check database logs
docker logs llmspell_postgres_dev --tail 100

# 2. Verify migrations applied
llmspell database migrations list

# 3. Try import with transaction rollback (automatic)
# Import is transaction-safe - if it fails, all changes are rolled back

# 4. Check for data integrity issues in export
jq '.data.knowledge_graph.relationships[] | select(.source_entity_id == "INVALID")' export.json

# 5. Re-export with fresh database snapshot
llmspell storage export --backend sqlite --output clean-export.json
```

---

#### Issue 4: PostgreSQL Connection Timeout

**Problem:**
```
Error: Connection timeout after 30 seconds
```

**Solutions:**

```bash
# 1. Verify PostgreSQL is running
docker ps | grep postgres

# 2. Check connection string
echo $DATABASE_URL
psql $DATABASE_URL -c "SELECT 1"

# 3. Increase connection timeout in config
# Edit config.toml:
[storage.postgresql]
connection_timeout_seconds = 60

# 4. Check network connectivity
ping postgres-host
telnet postgres-host 5432
```

---

#### Issue 5: Data Missing After Import

**Problem:** Import reports success but data is missing.

**Solutions:**

```bash
# 1. Check import statistics
# Ensure counts match source database

# 2. Verify tenant_id filtering (if using multi-tenancy)
# Check config.toml for tenant filtering

# 3. Export from target and compare
llmspell storage export --backend postgres --output verify.json
jq '.data | keys' verify.json  # Check which tables have data

# 4. Check PostgreSQL row-level security (RLS)
psql $DATABASE_URL -c "SELECT tablename, rowsecurity FROM pg_tables WHERE schemaname = 'public';"
# Disable RLS temporarily for debugging
psql $DATABASE_URL -c "ALTER TABLE embeddings_1536 DISABLE ROW LEVEL SECURITY;"
```

---

#### Issue 6: Vector Dimension Mismatch

**Problem:**
```
Error: Dimension mismatch: expected 1536, got 768
```

**Solutions:**

```bash
# 1. Check export contains correct dimensions
jq '.data.vector_embeddings | keys' export.json
# Output: ["384", "768", "1536", "3072"]

# 2. Verify target database schema supports dimensions
psql $DATABASE_URL -c "SELECT table_name FROM information_schema.tables WHERE table_name LIKE 'embeddings_%';"

# 3. Apply missing migrations
llmspell database migrate

# 4. Re-export ensuring all dimensions included
llmspell storage export --backend sqlite --output full-export.json
```

---

## Best Practices

### Pre-Migration Checklist

**Before starting any migration:**

- [ ] **Backup source database**
  ```bash
  llmspell storage export --backend sqlite --output pre-migration-backup.json
  ```

- [ ] **Verify target backend health**
  ```bash
  llmspell database health --backend postgres
  ```

- [ ] **Test with small dataset first**
  ```bash
  # Export subset for testing
  jq '.data.sessions = .data.sessions[:5]' full-export.json > test-export.json
  ```

- [ ] **Check disk space**
  ```bash
  df -h /var/lib/postgresql/data
  ```

- [ ] **Schedule during maintenance window** (production migrations)

- [ ] **Notify stakeholders** of migration timing

---

### During Migration

**Best practices while migration is in progress:**

1. **Monitor import progress**
   ```bash
   # Watch import statistics in real-time
   llmspell storage import --backend postgres --input data.json 2>&1 | tee import.log
   ```

2. **Keep source database running**
   - Don't shut down source until migration verified

3. **Use transaction-safe imports**
   - Import automatically uses transactions (rollback on failure)

4. **Log everything**
   ```bash
   # Capture all output for troubleshooting
   llmspell storage import --backend postgres --input data.json > import.log 2>&1
   ```

---

### Post-Migration

**After migration completes:**

1. **Verify data integrity** (see Verification Procedures above)

2. **Update application configuration**
   ```toml
   # config.toml
   [storage]
   backend = "postgresql"  # Changed from "sqlite"
   ```

3. **Update connection strings in deployment**
   ```bash
   # Update environment variables
   export DATABASE_URL="postgresql://user:pass@new-host:5432/llmspell"
   ```

4. **Test critical workflows**
   ```bash
   cargo test --test integration_tests
   ```

5. **Monitor performance**
   ```bash
   # Check query performance
   psql $DATABASE_URL -c "SELECT * FROM pg_stat_statements ORDER BY mean_exec_time DESC LIMIT 10;"
   ```

6. **Archive export files**
   ```bash
   # Keep for disaster recovery
   mv export.json /var/backups/llmspell/export-$(date +%Y%m%d).json
   ```

7. **Document migration**
   - Record migration date, source/target, any issues encountered

---

## Advanced Topics

### Selective Data Migration

**Migrate only specific data types:**

```bash
# 1. Export all data
llmspell storage export --backend sqlite --output full-export.json

# 2. Filter to specific data types
jq '{
  version: .version,
  exported_at: .exported_at,
  source_backend: .source_backend,
  migrations: .migrations,
  data: {
    sessions: .data.sessions,
    artifacts: .data.artifacts
  }
}' full-export.json > sessions-only.json

# 3. Import filtered data
llmspell storage import --backend postgres --input sessions-only.json
```

---

### Automated Migration Pipeline

**Script for regular migrations:**

```bash
#!/bin/bash
# migrate-to-staging.sh

set -e  # Exit on error

# Configuration
SOURCE_DB="postgresql://user:pass@prod-db:5432/llmspell_prod"
TARGET_DB="postgresql://user:pass@staging-db:5432/llmspell_staging"
BACKUP_DIR="/var/backups/llmspell"
DATE=$(date +%Y%m%d-%H%M%S)

# 1. Export from production
echo "Exporting from production..."
export DATABASE_URL="$SOURCE_DB"
llmspell storage export --backend postgres --output "${BACKUP_DIR}/prod-${DATE}.json"

# 2. Sanitize data (optional)
echo "Sanitizing sensitive data..."
jq 'del(.data.sessions[] | select(.tenant_id == "sensitive"))' \
   "${BACKUP_DIR}/prod-${DATE}.json" > "${BACKUP_DIR}/staging-${DATE}.json"

# 3. Import to staging
echo "Importing to staging..."
export DATABASE_URL="$TARGET_DB"
llmspell storage import --backend postgres --input "${BACKUP_DIR}/staging-${DATE}.json"

# 4. Verify
echo "Verifying migration..."
llmspell database health

echo "Migration complete: ${DATE}"
```

**Schedule with cron:**
```bash
# Daily at 2 AM
0 2 * * * /usr/local/bin/migrate-to-staging.sh >> /var/log/llmspell-migration.log 2>&1
```

---

### Cross-Region Migration

**Migrate between cloud regions:**

```bash
# 1. Export from source region
export DATABASE_URL="postgresql://user:pass@us-east-db:5432/llmspell"
llmspell storage export --backend postgres --output export.json

# 2. Upload to cloud storage
aws s3 cp export.json s3://my-bucket/migrations/export-$(date +%Y%m%d).json \
  --region us-east-1

# 3. Download in target region
aws s3 cp s3://my-bucket/migrations/export-20250122.json /tmp/export.json \
  --region eu-west-1

# 4. Import to target region
export DATABASE_URL="postgresql://user:pass@eu-west-db:5432/llmspell"
llmspell storage import --backend postgres --input /tmp/export.json
```

---

### Data Transformation During Migration

**Modify data during migration:**

```bash
# 1. Export
llmspell storage export --backend sqlite --output export.json

# 2. Transform using jq
jq '
  # Update all tenant_ids
  .data.sessions[].tenant_id = "new-tenant" |
  # Remove old sessions
  .data.sessions = [.data.sessions[] | select(.created_at > 1640000000)] |
  # Update metadata
  .data.vector_embeddings."1536"[].metadata.migrated = true
' export.json > transformed.json

# 3. Import transformed data
llmspell storage import --backend postgres --input transformed.json
```

---

## See Also

### User Guides
- [Storage Setup](07-storage-setup.md) - Backend configuration
- [CLI Reference](05-cli-reference.md#storage) - Command documentation
- [Configuration Guide](03-configuration.md) - Storage configuration options

### Technical Documentation
- [PostgreSQL Guide](../technical/postgresql-guide.md) - PostgreSQL-specific details
- [SQLite Architecture](../technical/sqlite-vector-storage-architecture.md) - SQLite internals
- [Migration Internals](../technical/storage-migration-internals.md) - Technical deep dive

### Developer Guides
- [Storage Backends](../developer-guide/reference/storage-backends.md) - Export/Import API
- [Operations Guide](../developer-guide/08-operations.md) - Migration procedures

---

**Version**: 0.14.0 | **Phase**: 13c.3.2.6 | **Last Updated**: 2025-01-22
