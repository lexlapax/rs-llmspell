# Utility Scripts

> 🛠️ **Purpose**: Helper utilities for development, deployment, and maintenance of LLMSpell installations and examples.

## 📋 Scripts Overview

| Script | Purpose | Target Users | Complexity |
|--------|---------|--------------|------------|
| [`llmspell-easy.sh`](#llmspell-easysh) | Simplified launcher | Non-technical users | Simple |
| [`find-examples.sh`](#find-examplessh) | Example discovery | Developers | Simple |
| [`backup_maintenance.sh`](#backup_maintenancesh) | Backup automation | System admins | Moderate |
| [`test_backup_integrity.sh`](#test_backup_integritysh) | Backup validation | DevOps | Advanced |

## 🚀 Quick Start

```bash
# Launch LLMSpell easily
./llmspell-easy.sh

# Find relevant examples
./find-examples.sh agent

# Create backup
./backup_maintenance.sh --backup-only

# Validate backup integrity
./test_backup_integrity.sh
```

## 📝 Script Details

### llmspell-easy.sh
**User-Friendly Launcher**

Zero-configuration wrapper for non-technical users:

```bash
# Basic usage - auto-detects everything
./llmspell-easy.sh

# Run specific example
./llmspell-easy.sh examples/hello.lua

# Interactive mode with API key setup
./llmspell-easy.sh --setup

# Force specific binary
LLMSPELL_BIN=/usr/local/bin/llmspell ./llmspell-easy.sh
```

**Features:**
- Auto-detects llmspell binary location
- Checks and configures API keys
- Provides friendly error messages
- Suggests fixes for common issues
- Interactive setup mode

**Binary search order:**
1. `./target/debug/llmspell`
2. `./target/release/llmspell`
3. `/usr/local/bin/llmspell`
4. `/usr/bin/llmspell`
5. `~/.cargo/bin/llmspell`
6. System PATH

### find-examples.sh
**Example Discovery Tool**

Find relevant examples by keyword, tag, or feature:

```bash
# Search by keyword
./find-examples.sh agent           # Find agent examples
./find-examples.sh "state.*persist" # Regex search

# Search by tag
./find-examples.sh --tag beginner
./find-examples.sh --tag production

# Search by feature
./find-examples.sh --feature workflow
./find-examples.sh --feature rag

# List available options
./find-examples.sh --list-tags
./find-examples.sh --list-features

# Advanced search
./find-examples.sh --complexity intermediate --feature agent
```

**Available Tags:**
- `beginner` - Getting started examples
- `intermediate` - Core feature demonstrations
- `advanced` - Complex patterns
- `production` - Production-ready applications

**Available Features:**
- `agents` - Agent orchestration
- `tools` - Tool usage
- `workflows` - Workflow patterns
- `state` - State management
- `rag` - RAG implementations

### backup_maintenance.sh
**Automated Backup Management**

Comprehensive backup creation and maintenance:

```bash
# Create backup
./backup_maintenance.sh

# Backup only (skip cleanup)
./backup_maintenance.sh --backup-only

# Cleanup only (skip backup)
./backup_maintenance.sh --cleanup-only

# Dry run (show what would be done)
./backup_maintenance.sh --dry-run

# Incremental backup
./backup_maintenance.sh --incremental

# Custom backup directory
./backup_maintenance.sh --backup-dir /backups

# Verbose output
./backup_maintenance.sh --verbose
```

**Environment Variables:**
```bash
export BACKUP_DIR=/path/to/backups     # Backup location
export DRY_RUN=true                    # Dry run mode
export VERBOSE=true                    # Verbose output
export CREATE_BACKUP=false             # Skip backup creation
export CLEANUP_BACKUPS=false           # Skip cleanup
export INCREMENTAL=true                # Incremental backup
```

**Features:**
- Full and incremental backups
- Automatic cleanup of old backups
- Dry-run mode for safety
- Detailed logging
- Configurable retention policies

### test_backup_integrity.sh
**Backup Integrity Validation**

Comprehensive backup testing and validation:

```bash
# Run all integrity tests
./test_backup_integrity.sh

# Test specific backup
./test_backup_integrity.sh --backup backup_20240101.tar.gz

# Quick validation only
./test_backup_integrity.sh --quick

# Full restore test
./test_backup_integrity.sh --full-restore

# Custom test directory
TEST_DIR=/tmp/backup_test ./test_backup_integrity.sh
```

**Test Suite:**
1. **Archive Integrity**
   - Checksum verification
   - Archive structure validation
   - Compression integrity

2. **Content Validation**
   - File count verification
   - Directory structure check
   - Permission preservation

3. **Restore Testing**
   - Test restore to temporary location
   - State file validation
   - Configuration integrity

4. **Performance Metrics**
   - Backup size analysis
   - Compression ratio
   - Restore time measurement

## 🔧 Configuration

### Global Settings

```bash
# ~/.llmspell/scripts.conf
LLMSPELL_BIN=/usr/local/bin/llmspell
BACKUP_DIR=/var/backups/llmspell
LOG_DIR=/var/log/llmspell
EXAMPLES_DIR=/usr/share/llmspell/examples
```

### Backup Configuration

```bash
# Backup settings
BACKUP_RETENTION_DAYS=30        # Keep backups for 30 days
BACKUP_MAX_COUNT=10             # Maximum number of backups
BACKUP_COMPRESSION=gzip         # Compression method
BACKUP_VERIFY=true              # Verify after creation

# Cleanup settings
CLEANUP_AGE_DAYS=30            # Remove backups older than
CLEANUP_MIN_KEEP=3             # Always keep at least 3
CLEANUP_MAX_SIZE_GB=10         # Max total backup size
```

## 🏃 Common Workflows

### New User Setup
```bash
# 1. Easy launcher with setup
./llmspell-easy.sh --setup

# 2. Find beginner examples
./find-examples.sh --tag beginner

# 3. Run first example
./llmspell-easy.sh examples/hello.lua
```

### Backup Management
```bash
# Daily backup cron job
0 2 * * * /path/to/backup_maintenance.sh

# Weekly integrity check
0 3 * * 0 /path/to/test_backup_integrity.sh

# Manual backup before upgrade
./backup_maintenance.sh --backup-only
```

### Example Exploration
```bash
# Find all RAG examples
./find-examples.sh --feature rag

# Find intermediate agent examples
./find-examples.sh --complexity intermediate --feature agent

# List all available examples
./find-examples.sh --list-all
```

## 📊 Logging & Output

### Log Locations
```
~/.llmspell/logs/
├── backup_maintenance.log
├── backup_integrity.log
└── llmspell-easy.log
```

### Output Formats
```bash
# JSON output
./find-examples.sh --format json

# Quiet mode
./backup_maintenance.sh --quiet

# Verbose debugging
VERBOSE=true ./test_backup_integrity.sh
```

## 🐛 Troubleshooting

### llmspell-easy.sh Issues

**Binary not found:**
```bash
# Manually specify binary
export LLMSPELL_BIN=/path/to/llmspell
./llmspell-easy.sh
```

**API key issues:**
```bash
# Check current keys
./llmspell-easy.sh --check-keys

# Re-run setup
./llmspell-easy.sh --setup
```

### Backup Issues

**Backup fails:**
```bash
# Check disk space
df -h /backup/location

# Verify permissions
ls -la /backup/location

# Run with verbose output
./backup_maintenance.sh --verbose --dry-run
```

**Integrity check fails:**
```bash
# Detailed validation
./test_backup_integrity.sh --verbose

# Check specific backup
./test_backup_integrity.sh --backup file.tar.gz --detailed
```

## 🔗 Related Documentation

- [Installation Guide](../../docs/installation/README.md)
- [Backup & Recovery](../../docs/operations/backup.md)
- [Examples Guide](../../examples/README.md)
- [CLI Reference](../../docs/reference/cli.md)