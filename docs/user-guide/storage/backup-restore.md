# PostgreSQL Backup and Restore Guide

**Phase 13b Storage Backend**: Disaster recovery procedures for production rs-llmspell deployments

## Table of Contents

- [Overview](#overview)
- [Backup Strategies](#backup-strategies)
- [Logical Backups (pg_dump)](#logical-backups-pg_dump)
- [Physical Backups (pg_basebackup)](#physical-backups-pg_basebackup)
- [Point-in-Time Recovery (PITR)](#point-in-time-recovery-pitr)
- [Continuous Archiving](#continuous-archiving)
- [Restore Procedures](#restore-procedures)
- [Disaster Recovery](#disaster-recovery)
- [Automation](#automation)
- [Testing & Validation](#testing--validation)

---

## Overview

rs-llmspell PostgreSQL databases require comprehensive backup strategies to protect against:
- **Hardware failures**: Disk corruption, server crashes
- **Human errors**: Accidental DELETE, DROP TABLE
- **Data corruption**: Application bugs, schema migrations gone wrong
- **Disasters**: Fire, flood, ransomware

**Recovery Objectives:**
- **RTO (Recovery Time Objective)**: <15 minutes for production
- **RPO (Recovery Point Objective)**: <5 minutes data loss maximum
- **RLO (Recovery Level Objective)**: Full database or single table

**Backup Types:**
| Type | RTO | RPO | Storage | Use Case |
|------|-----|-----|---------|----------|
| **Logical (pg_dump)** | 15-60 min | 24 hours | Compressed SQL | Daily backups, migrations |
| **Physical (pg_basebackup)** | 5-15 min | 1 hour | Raw data files | Weekly full backups |
| **PITR (WAL archiving)** | 10-30 min | <5 minutes | WAL segments | Continuous protection |

---

## Backup Strategies

### Strategy 1: Daily Logical Backups (Development/Staging)

**Use case:** Development and staging environments

```bash
#!/bin/bash
# /usr/local/bin/llmspell-backup-daily.sh

BACKUP_DIR="/var/backups/llmspell"
DATABASE="llmspell_dev"
USER="llmspell"
RETENTION_DAYS=7

DATE=$(date +%Y%m%d_%H%M%S)
BACKUP_FILE="${BACKUP_DIR}/llmspell_${DATE}.sql.gz"

# Create backup with compression
pg_dump -U $USER -d $DATABASE | gzip > $BACKUP_FILE

# Verify backup
if [ $? -eq 0 ]; then
    echo "Backup successful: $BACKUP_FILE"
    SIZE=$(du -h $BACKUP_FILE | cut -f1)
    echo "Size: $SIZE"
else
    echo "Backup failed!" >&2
    exit 1
fi

# Delete old backups
find $BACKUP_DIR -name "llmspell_*.sql.gz" -mtime +$RETENTION_DAYS -delete

# Upload to S3 (optional)
# aws s3 cp $BACKUP_FILE s3://llmspell-backups/daily/
```

**Cron schedule (2 AM daily):**
```cron
0 2 * * * /usr/local/bin/llmspell-backup-daily.sh
```

### Strategy 2: Physical + PITR (Production)

**Use case:** Production environments with <5 min RPO

**Components:**
1. **Weekly full backup** (pg_basebackup): Sunday 2 AM
2. **Continuous WAL archiving**: Real-time WAL segments
3. **Daily verification**: Test restore on staging

```bash
#!/bin/bash
# /usr/local/bin/llmspell-backup-full.sh

BACKUP_DIR="/var/backups/llmspell/basebackup"
DATE=$(date +%Y%m%d_%H%M%S)
BACKUP_PATH="${BACKUP_DIR}/base_${DATE}"

# Create base backup
pg_basebackup -U llmspell -D $BACKUP_PATH -Ft -z -Xs -P

# Verify backup
if [ $? -eq 0 ]; then
    echo "Base backup successful: $BACKUP_PATH"
    SIZE=$(du -sh $BACKUP_PATH | cut -f1)
    echo "Size: $SIZE"
else
    echo "Base backup failed!" >&2
    exit 1
fi

# Delete old backups (keep 4 weeks)
find $BACKUP_DIR -name "base_*" -mtime +28 -exec rm -rf {} \;
```

**Cron schedule:**
```cron
# Weekly full backup (Sunday 2 AM)
0 2 * * 0 /usr/local/bin/llmspell-backup-full.sh

# Daily WAL archive cleanup (keep 30 days)
0 3 * * * find /var/backups/llmspell/wal_archive -mtime +30 -delete
```

### Strategy 3: Hybrid (Small Production)

**Use case:** Small production deployments (<100 GB)

- **Daily pg_dump**: Full logical backup
- **Hourly pg_dump (schema-only)**: Fast schema snapshots
- **S3/GCS replication**: Off-site storage

---

## Logical Backups (pg_dump)

### Full Database Backup

```bash
# Compressed SQL format (human-readable)
pg_dump -U llmspell -d llmspell_prod | gzip > llmspell_prod.sql.gz

# Custom format (faster restore, parallel)
pg_dump -U llmspell -d llmspell_prod -F c -f llmspell_prod.dump

# Directory format (parallel dump, per-table files)
pg_dump -U llmspell -d llmspell_prod -F d -j 4 -f llmspell_prod_dir/
```

**Format comparison:**
| Format | Extension | Parallel Dump | Parallel Restore | Compression | Size (100 GB DB) |
|--------|-----------|---------------|------------------|-------------|------------------|
| **SQL** | `.sql` | No | No | gzip | 15 GB |
| **Custom** | `.dump` | No | Yes | Built-in | 20 GB |
| **Directory** | `/dir` | Yes | Yes | Built-in | 20 GB |

**Recommendation:** Use **Custom** format for single-file backups, **Directory** for large databases (>50 GB)

### Schema-Only Backup

```bash
# Schema without data (fast, <1 MB)
pg_dump -U llmspell -d llmspell_prod --schema-only -f schema.sql

# Schema + ownership + privileges
pg_dump -U llmspell -d llmspell_prod --schema-only --no-owner --no-privileges -f schema_portable.sql
```

**Use cases:**
- Schema version control (git)
- Pre-migration snapshots
- Development environment setup

### Table-Specific Backup

```bash
# Single table
pg_dump -U llmspell -d llmspell_prod -t llmspell.sessions -F c -f sessions.dump

# Multiple tables (pattern matching)
pg_dump -U llmspell -d llmspell_prod -t 'llmspell.vector_embeddings_*' -F c -f vectors.dump

# Exclude tables (all except event_log)
pg_dump -U llmspell -d llmspell_prod --exclude-table=llmspell.event_log -F c -f without_events.dump
```

### VectorChord Index Handling

**Problem:** HNSW indexes are not dumped by pg_dump (extension-specific)

**Solution:** Rebuild indexes after restore

```bash
# Backup excludes HNSW indexes
pg_dump -U llmspell -d llmspell_prod -F c -f llmspell_prod.dump

# After restore, rebuild HNSW indexes
psql -U llmspell -d llmspell_restored <<EOF
REINDEX INDEX CONCURRENTLY llmspell.idx_vector_384_hnsw;
REINDEX INDEX CONCURRENTLY llmspell.idx_vector_768_hnsw;
REINDEX INDEX CONCURRENTLY llmspell.idx_vector_1536_hnsw;
EOF
```

**Rebuild time:** ~15 seconds per 10K vectors (m=16, ef_construction=128)

### Parallel Dump (Large Databases)

```bash
# Directory format with 8 parallel jobs
pg_dump -U llmspell -d llmspell_prod -F d -j 8 -f llmspell_prod_dir/

# Custom format (no parallel, but can parallel restore)
pg_dump -U llmspell -d llmspell_prod -F c -Z 6 -f llmspell_prod.dump
```

**Performance:**
- **Single-threaded**: 100 GB in 60 minutes
- **8 parallel jobs**: 100 GB in 15 minutes (4x speedup)

---

## Physical Backups (pg_basebackup)

### Full Physical Backup

```bash
# Tar format (compressed, single file)
pg_basebackup -U llmspell -D /var/backups/llmspell/base -Ft -z -Xs -P

# Options:
# -Ft: tar format
# -z: gzip compression
# -Xs: stream WAL (includes WAL in backup)
# -P: show progress
```

**Output:**
```
/var/backups/llmspell/base/
├── base.tar.gz      # Data files
└── pg_wal.tar.gz    # WAL segments
```

### Plain Directory Format

```bash
# Plain format (uncompressed, directory)
pg_basebackup -U llmspell -D /var/backups/llmspell/base_plain -Fp -Xs -P

# Compress manually
tar -czf base_plain.tar.gz /var/backups/llmspell/base_plain/
```

**Use cases:**
- **Tar format**: Off-site archival, S3 uploads
- **Plain format**: Fast local restores, rsync incrementals

### Backup Verification

```bash
# Verify tar archive integrity
tar -tzf /var/backups/llmspell/base/base.tar.gz | head

# Extract and inspect
mkdir -p /tmp/backup_verify
tar -xzf /var/backups/llmspell/base/base.tar.gz -C /tmp/backup_verify
ls -lh /tmp/backup_verify/
```

---

## Point-in-Time Recovery (PITR)

### Enable WAL Archiving

**Edit `postgresql.conf`:**
```conf
# WAL archiving
wal_level = replica                # Required for PITR
archive_mode = on                  # Enable archiving
archive_command = 'test ! -f /var/backups/llmspell/wal_archive/%f && cp %p /var/backups/llmspell/wal_archive/%f'
archive_timeout = 300              # Force archive every 5 minutes

# WAL retention
wal_keep_size = 1GB                # Keep 1 GB of WAL on server
max_wal_senders = 3                # For streaming replication
```

**Restart PostgreSQL:**
```bash
sudo systemctl restart postgresql@18-main
```

### Verify Archiving

```bash
# Check archive status
psql -U postgres -c "SELECT archived_count, last_archived_wal, last_archived_time FROM pg_stat_archiver;"

# Monitor WAL archive directory
ls -lh /var/backups/llmspell/wal_archive/ | tail

# Force WAL switch (creates new segment)
psql -U postgres -c "SELECT pg_switch_wal();"
```

### WAL Archive Structure

```
/var/backups/llmspell/wal_archive/
├── 000000010000000000000001
├── 000000010000000000000002
├── 000000010000000000000003
...
└── 00000001000000000000005A

# Each file: 16 MB (default WAL segment size)
# Naming: timeline + log file + segment number
```

### S3 WAL Archiving (Production)

**Install wal-g or wal-e:**
```bash
# wal-g (recommended, faster)
wget https://github.com/wal-g/wal-g/releases/download/v2.0.1/wal-g-pg-ubuntu-20.04-amd64.tar.gz
tar -xzf wal-g-pg-ubuntu-20.04-amd64.tar.gz -C /usr/local/bin/
chmod +x /usr/local/bin/wal-g
```

**Configure `postgresql.conf`:**
```conf
archive_command = 'wal-g wal-push %p'
restore_command = 'wal-g wal-fetch %f %p'
```

**Environment variables (in systemd service or .bashrc):**
```bash
export WALG_S3_PREFIX="s3://llmspell-backups/wal"
export AWS_REGION="us-east-1"
export AWS_ACCESS_KEY_ID="..."
export AWS_SECRET_ACCESS_KEY="..."
```

---

## Continuous Archiving

### Full Backup + WAL Workflow

**Step 1: Create base backup**
```bash
pg_basebackup -U llmspell -D /var/backups/llmspell/base -Ft -z -Xs -P
```

**Step 2: Continuous WAL archiving (automatic)**
PostgreSQL archives WAL segments as they fill (16 MB) or timeout (5 minutes)

**Step 3: Restore to any point in time**
```bash
# Restore base backup
tar -xzf base.tar.gz -C /var/lib/postgresql/18/main/

# Restore WAL segments up to specific time
# (see PITR Restore section below)
```

### WAL Archive Cleanup

```bash
#!/bin/bash
# /usr/local/bin/llmspell-wal-cleanup.sh

WAL_ARCHIVE="/var/backups/llmspell/wal_archive"
RETENTION_DAYS=30

# Delete WAL files older than 30 days
find $WAL_ARCHIVE -type f -mtime +$RETENTION_DAYS -delete

# Keep minimum for PITR (last base backup)
# Use pg_archivecleanup for safe cleanup
pg_archivecleanup $WAL_ARCHIVE $(cat /var/backups/llmspell/base/base.tar.gz.wal)
```

**Cron schedule (daily 3 AM):**
```cron
0 3 * * * /usr/local/bin/llmspell-wal-cleanup.sh
```

---

## Restore Procedures

### Logical Restore (pg_restore)

**From SQL dump:**
```bash
# Decompress and restore
gunzip -c llmspell_prod.sql.gz | psql -U llmspell -d llmspell_restored

# Or single command
psql -U llmspell -d llmspell_restored < llmspell_prod.sql
```

**From custom format:**
```bash
# Single-threaded restore
pg_restore -U llmspell -d llmspell_restored -v llmspell_prod.dump

# Parallel restore (4 jobs)
pg_restore -U llmspell -d llmspell_restored -j 4 -v llmspell_prod.dump

# Restore specific table only
pg_restore -U llmspell -d llmspell_restored -t sessions llmspell_prod.dump
```

**From directory format:**
```bash
# Parallel restore (8 jobs)
pg_restore -U llmspell -d llmspell_restored -F d -j 8 llmspell_prod_dir/
```

### Physical Restore (pg_basebackup)

**Step 1: Stop PostgreSQL**
```bash
sudo systemctl stop postgresql@18-main
```

**Step 2: Backup current data (safety)**
```bash
sudo mv /var/lib/postgresql/18/main /var/lib/postgresql/18/main.old
```

**Step 3: Extract base backup**
```bash
sudo mkdir -p /var/lib/postgresql/18/main
sudo tar -xzf /var/backups/llmspell/base/base.tar.gz -C /var/lib/postgresql/18/main
sudo tar -xzf /var/backups/llmspell/base/pg_wal.tar.gz -C /var/lib/postgresql/18/main/pg_wal
```

**Step 4: Set ownership**
```bash
sudo chown -R postgres:postgres /var/lib/postgresql/18/main
```

**Step 5: Start PostgreSQL**
```bash
sudo systemctl start postgresql@18-main
```

**Step 6: Verify**
```bash
psql -U llmspell -d llmspell_prod -c "SELECT count(*) FROM llmspell.sessions;"
```

### PITR Restore (Point-in-Time Recovery)

**Scenario:** Restore database to 2025-01-15 14:30:00 (before accidental DELETE)

**Step 1: Stop PostgreSQL**
```bash
sudo systemctl stop postgresql@18-main
```

**Step 2: Restore base backup**
```bash
sudo rm -rf /var/lib/postgresql/18/main
sudo mkdir -p /var/lib/postgresql/18/main
sudo tar -xzf /var/backups/llmspell/base/base.tar.gz -C /var/lib/postgresql/18/main
```

**Step 3: Create recovery configuration**
```bash
# /var/lib/postgresql/18/main/postgresql.auto.conf (append)
cat <<EOF | sudo tee -a /var/lib/postgresql/18/main/postgresql.auto.conf
restore_command = 'cp /var/backups/llmspell/wal_archive/%f %p'
recovery_target_time = '2025-01-15 14:30:00'
recovery_target_action = 'promote'
EOF
```

**Step 4: Create recovery signal file**
```bash
sudo touch /var/lib/postgresql/18/main/recovery.signal
```

**Step 5: Set ownership**
```bash
sudo chown -R postgres:postgres /var/lib/postgresql/18/main
```

**Step 6: Start PostgreSQL (recovery begins)**
```bash
sudo systemctl start postgresql@18-main

# Monitor recovery
tail -f /var/log/postgresql/postgresql-18-main.log
```

**Expected log output:**
```
LOG:  starting point-in-time recovery to 2025-01-15 14:30:00+00
LOG:  restored log file "000000010000000000000042" from archive
LOG:  redo starts at 0/42000028
...
LOG:  recovery stopping before commit of transaction 1234, time 2025-01-15 14:30:00.123456+00
LOG:  recovery has paused
LOG:  selected new timeline ID: 2
LOG:  database system is ready to accept connections
```

**Step 7: Verify data**
```bash
psql -U llmspell -d llmspell_prod -c "SELECT max(created_at) FROM llmspell.sessions;"
# Should show timestamps <= 2025-01-15 14:30:00
```

### Table-Level Restore

**Scenario:** Restore single table from backup

```bash
# Extract table from backup
pg_restore -U llmspell -d llmspell_prod -t llmspell.sessions --data-only llmspell_prod.dump

# Or restore to temp table first (safer)
pg_restore -U llmspell -d llmspell_prod -t llmspell.sessions --schema-only llmspell_prod.dump
psql -U llmspell -d llmspell_prod <<EOF
ALTER TABLE llmspell.sessions RENAME TO sessions_backup;
EOF

pg_restore -U llmspell -d llmspell_prod -t llmspell.sessions llmspell_prod.dump

# Verify data
psql -U llmspell -d llmspell_prod -c "SELECT count(*) FROM llmspell.sessions;"
psql -U llmspell -d llmspell_prod -c "SELECT count(*) FROM llmspell.sessions_backup;"
```

---

## Disaster Recovery

### DR Runbook (Complete Database Loss)

**Scenario:** Primary server failed, restore from backups

**RTO Target:** 15 minutes
**RPO Target:** <5 minutes

**Checklist:**

1. **[0-2 min] Provision new server**
   - Launch EC2 instance or bare metal
   - Install PostgreSQL 18 + VectorChord

2. **[2-5 min] Restore base backup**
   ```bash
   pg_basebackup restore (see Physical Restore section)
   ```

3. **[5-10 min] Apply WAL archives (PITR)**
   ```bash
   Configure recovery.signal + restore_command
   ```

4. **[10-12 min] Rebuild VectorChord indexes**
   ```bash
   REINDEX INDEX CONCURRENTLY idx_vector_*_hnsw;
   ```

5. **[12-15 min] Verify and promote**
   ```bash
   # Verify data integrity
   psql -c "SELECT count(*) FROM sessions;"

   # Update DNS or load balancer
   # Point application to new server
   ```

### Split-Brain Prevention

**Problem:** Old primary comes back online after DR

**Solution:** Use timeline IDs to prevent conflicts

```bash
# Check current timeline
psql -U postgres -c "SELECT timeline_id, pg_current_wal_lsn();"

# After PITR, new timeline = old_timeline + 1
# Old primary cannot replay WAL from new timeline (safe)
```

### Off-Site Replication

**S3 backup sync (daily):**
```bash
#!/bin/bash
# /usr/local/bin/llmspell-s3-sync.sh

BACKUP_DIR="/var/backups/llmspell"
S3_BUCKET="s3://llmspell-backups"

# Sync base backups
aws s3 sync $BACKUP_DIR/basebackup/ $S3_BUCKET/basebackup/ \
    --storage-class GLACIER_IR \
    --exclude "*" --include "base_*"

# Sync WAL archives (faster storage class)
aws s3 sync $BACKUP_DIR/wal_archive/ $S3_BUCKET/wal_archive/ \
    --storage-class STANDARD_IA
```

**Cron schedule (4 AM daily):**
```cron
0 4 * * * /usr/local/bin/llmspell-s3-sync.sh
```

---

## Automation

### Systemd Timer (Modern Cron Alternative)

**Create timer unit:**
```ini
# /etc/systemd/system/llmspell-backup.timer
[Unit]
Description=llmspell daily backup timer

[Timer]
OnCalendar=daily
Persistent=true
Unit=llmspell-backup.service

[Install]
WantedBy=timers.target
```

**Create service unit:**
```ini
# /etc/systemd/system/llmspell-backup.service
[Unit]
Description=llmspell daily backup
After=postgresql.service

[Service]
Type=oneshot
User=postgres
ExecStart=/usr/local/bin/llmspell-backup-daily.sh
StandardOutput=journal
StandardError=journal
```

**Enable and start:**
```bash
sudo systemctl daemon-reload
sudo systemctl enable llmspell-backup.timer
sudo systemctl start llmspell-backup.timer

# Check status
sudo systemctl list-timers --all | grep llmspell
```

### Monitoring and Alerts

**Backup success monitoring:**
```bash
#!/bin/bash
# /usr/local/bin/llmspell-backup-monitor.sh

BACKUP_DIR="/var/backups/llmspell"
MAX_AGE_HOURS=26  # Alert if no backup in 26 hours (>1 day)

LATEST_BACKUP=$(find $BACKUP_DIR -name "llmspell_*.sql.gz" -type f -printf '%T@ %p\n' | sort -rn | head -1 | cut -d' ' -f2)

if [ -z "$LATEST_BACKUP" ]; then
    echo "ERROR: No backups found in $BACKUP_DIR"
    # Send alert (email, Slack, PagerDuty)
    exit 1
fi

AGE_SECONDS=$(( $(date +%s) - $(stat -c %Y "$LATEST_BACKUP") ))
AGE_HOURS=$(( AGE_SECONDS / 3600 ))

if [ $AGE_HOURS -gt $MAX_AGE_HOURS ]; then
    echo "ERROR: Latest backup is $AGE_HOURS hours old (max: $MAX_AGE_HOURS)"
    echo "File: $LATEST_BACKUP"
    # Send alert
    exit 1
fi

echo "OK: Latest backup is $AGE_HOURS hours old"
echo "File: $LATEST_BACKUP"
```

**Prometheus exporter (optional):**
```bash
# Export metrics for Prometheus scraping
cat <<EOF > /var/lib/node_exporter/textfile_collector/llmspell_backup.prom
# HELP llmspell_backup_age_seconds Age of latest backup in seconds
# TYPE llmspell_backup_age_seconds gauge
llmspell_backup_age_seconds $(( $(date +%s) - $(stat -c %Y "$LATEST_BACKUP") ))

# HELP llmspell_backup_size_bytes Size of latest backup in bytes
# TYPE llmspell_backup_size_bytes gauge
llmspell_backup_size_bytes $(stat -c %s "$LATEST_BACKUP")
EOF
```

---

## Testing & Validation

### Restore Test (Monthly)

**Purpose:** Validate backups are restorable

```bash
#!/bin/bash
# /usr/local/bin/llmspell-restore-test.sh

BACKUP_FILE="/var/backups/llmspell/llmspell_latest.sql.gz"
TEST_DB="llmspell_restore_test"

echo "=== Restore Test Started: $(date) ==="

# Create test database
psql -U postgres -c "DROP DATABASE IF EXISTS $TEST_DB;"
psql -U postgres -c "CREATE DATABASE $TEST_DB OWNER llmspell;"

# Restore backup
START_TIME=$(date +%s)
gunzip -c $BACKUP_FILE | psql -U llmspell -d $TEST_DB
END_TIME=$(date +%s)
DURATION=$(( END_TIME - START_TIME ))

# Verify data
SESSIONS_COUNT=$(psql -U llmspell -d $TEST_DB -t -c "SELECT count(*) FROM llmspell.sessions;")
EVENTS_COUNT=$(psql -U llmspell -d $TEST_DB -t -c "SELECT count(*) FROM llmspell.event_log;")

echo "Restore completed in $DURATION seconds"
echo "Sessions: $SESSIONS_COUNT"
echo "Events: $EVENTS_COUNT"

# Cleanup
psql -U postgres -c "DROP DATABASE $TEST_DB;"

echo "=== Restore Test Completed Successfully ==="
```

**Cron schedule (1st of month, 3 AM):**
```cron
0 3 1 * * /usr/local/bin/llmspell-restore-test.sh | mail -s "llmspell Restore Test" admin@example.com
```

### Backup Integrity Check

```bash
# Verify pg_dump output is valid SQL
pg_dump -U llmspell -d llmspell_prod | psql -U llmspell -d llmspell_test --dry-run

# Verify custom format integrity
pg_restore -l llmspell_prod.dump | head -20

# Verify tar archive
tar -tzf base.tar.gz | wc -l
```

### Performance Benchmarks

**Backup performance:**
```bash
# Time backup
time pg_dump -U llmspell -d llmspell_prod -F c -f llmspell_prod.dump

# Expected: 10 GB in 5 minutes (single-threaded)
```

**Restore performance:**
```bash
# Time restore
time pg_restore -U llmspell -d llmspell_restored -j 4 llmspell_prod.dump

# Expected: 10 GB in 3 minutes (4 parallel jobs)
```

---

## Summary

### Quick Reference

| Operation | Command | RTO | Use Case |
|-----------|---------|-----|----------|
| **Daily logical backup** | `pg_dump -F c` | 15-60 min | Development, migrations |
| **Weekly physical backup** | `pg_basebackup -Ft -z` | 5-15 min | Production full backups |
| **Continuous archiving** | WAL archiving | 10-30 min | PITR, <5 min RPO |
| **Single table restore** | `pg_restore -t` | 1-5 min | Accidental DELETE recovery |
| **PITR** | recovery.signal + WAL | 10-30 min | Time-travel to before incident |

### Backup Checklist (Production)

- [ ] Daily pg_dump backups (2 AM)
- [ ] Weekly pg_basebackup (Sunday 2 AM)
- [ ] WAL archiving enabled (`archive_mode = on`)
- [ ] Off-site replication (S3/GCS)
- [ ] Backup monitoring (age <26 hours)
- [ ] Monthly restore test (1st of month)
- [ ] Backup retention policy (30 days WAL, 90 days dumps)
- [ ] Disaster recovery runbook documented
- [ ] RTO/RPO validated (<15 min / <5 min)

### Next Steps

- **Setup Guide**: See [postgresql-setup.md](./postgresql-setup.md) for installation
- **Schema Reference**: See [schema-reference.md](./schema-reference.md) for table details
- **Performance Tuning**: See [performance-tuning.md](./performance-tuning.md) for optimization
- **Migration Guide**: See [migration-guide.md](./migration-guide.md) for version upgrades
