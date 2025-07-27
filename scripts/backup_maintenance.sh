#!/bin/bash

# Backup Maintenance Script for llmspell
# Automates backup creation and cleanup operations

set -euo pipefail

# Script configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
LOG_FILE="${LOG_FILE:-$PROJECT_ROOT/logs/backup_maintenance.log}"
LLMSPELL_CMD="${LLMSPELL_CMD:-llmspell}"

# Default settings (can be overridden by environment variables)
BACKUP_DIR="${BACKUP_DIR:-$PROJECT_ROOT/backups}"
DRY_RUN="${DRY_RUN:-false}"
VERBOSE="${VERBOSE:-false}"
CREATE_BACKUP="${CREATE_BACKUP:-true}"
CLEANUP_BACKUPS="${CLEANUP_BACKUPS:-true}"
INCREMENTAL="${INCREMENTAL:-false}"

# Ensure log directory exists
mkdir -p "$(dirname "$LOG_FILE")"

# Logging function
log() {
    local level="$1"
    shift
    local message="$*"
    local timestamp=$(date '+%Y-%m-%d %H:%M:%S')
    echo "[$timestamp] [$level] $message" | tee -a "$LOG_FILE"
}

# Show usage
usage() {
    cat << EOF
Usage: $0 [OPTIONS]

Backup maintenance script for llmspell state persistence.

OPTIONS:
    -h, --help              Show this help message
    -d, --dry-run           Run cleanup in dry-run mode (don't delete anything)
    -v, --verbose           Enable verbose output
    -b, --backup-only       Only create backup, skip cleanup
    -c, --cleanup-only      Only run cleanup, skip backup creation
    -i, --incremental       Create incremental backup (default: full)
    --backup-dir DIR        Set backup directory (default: ./backups)
    --log-file FILE         Set log file location

ENVIRONMENT VARIABLES:
    BACKUP_DIR              Backup directory path
    DRY_RUN                 Set to 'true' for dry-run mode
    VERBOSE                 Set to 'true' for verbose output
    CREATE_BACKUP           Set to 'false' to skip backup creation
    CLEANUP_BACKUPS         Set to 'false' to skip cleanup
    INCREMENTAL             Set to 'true' for incremental backups
    LLMSPELL_CMD            Path to llmspell command

EXAMPLES:
    # Regular maintenance (create backup + cleanup)
    $0

    # Dry run to see what would be cleaned up
    $0 --dry-run --cleanup-only

    # Create incremental backup only
    $0 --backup-only --incremental

    # Verbose cleanup
    $0 --cleanup-only --verbose

CRON EXAMPLES:
    # Daily full backup and cleanup at 2 AM
    0 2 * * * /path/to/backup_maintenance.sh >> /var/log/llmspell-backup.log 2>&1

    # Hourly incremental backup (no cleanup)
    0 * * * * /path/to/backup_maintenance.sh --backup-only --incremental

    # Weekly cleanup on Sunday at 3 AM
    0 3 * * 0 /path/to/backup_maintenance.sh --cleanup-only

EOF
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -h|--help)
            usage
            exit 0
            ;;
        -d|--dry-run)
            DRY_RUN="true"
            shift
            ;;
        -v|--verbose)
            VERBOSE="true"
            shift
            ;;
        -b|--backup-only)
            CREATE_BACKUP="true"
            CLEANUP_BACKUPS="false"
            shift
            ;;
        -c|--cleanup-only)
            CREATE_BACKUP="false"
            CLEANUP_BACKUPS="true"
            shift
            ;;
        -i|--incremental)
            INCREMENTAL="true"
            shift
            ;;
        --backup-dir)
            BACKUP_DIR="$2"
            shift 2
            ;;
        --log-file)
            LOG_FILE="$2"
            shift 2
            ;;
        *)
            echo "Unknown option: $1"
            usage
            exit 1
            ;;
    esac
done

# Start maintenance
log "INFO" "Starting backup maintenance"
log "INFO" "Configuration: BACKUP_DIR=$BACKUP_DIR, DRY_RUN=$DRY_RUN, VERBOSE=$VERBOSE"

# Check if llmspell command exists
if ! command -v "$LLMSPELL_CMD" &> /dev/null; then
    log "ERROR" "llmspell command not found. Please ensure it's in your PATH or set LLMSPELL_CMD"
    exit 1
fi

# Ensure backup directory exists
if [[ ! -d "$BACKUP_DIR" ]]; then
    log "INFO" "Creating backup directory: $BACKUP_DIR"
    mkdir -p "$BACKUP_DIR"
fi

# Create backup if requested
if [[ "$CREATE_BACKUP" == "true" ]]; then
    log "INFO" "Creating ${INCREMENTAL:+incremental }backup..."
    
    BACKUP_ARGS=""
    if [[ "$INCREMENTAL" == "true" ]]; then
        BACKUP_ARGS="--incremental"
    fi
    
    if $LLMSPELL_CMD backup create $BACKUP_ARGS; then
        log "INFO" "Backup created successfully"
    else
        log "ERROR" "Failed to create backup"
        exit 1
    fi
fi

# Run cleanup if requested
if [[ "$CLEANUP_BACKUPS" == "true" ]]; then
    log "INFO" "Running backup cleanup${DRY_RUN:+ (dry run)}..."
    
    CLEANUP_ARGS=""
    if [[ "$DRY_RUN" == "true" ]]; then
        CLEANUP_ARGS="--dry-run"
    fi
    if [[ "$VERBOSE" == "true" ]]; then
        CLEANUP_ARGS="$CLEANUP_ARGS --verbose"
    fi
    
    if $LLMSPELL_CMD backup cleanup $CLEANUP_ARGS; then
        log "INFO" "Cleanup completed successfully"
    else
        log "ERROR" "Failed to run cleanup"
        exit 1
    fi
fi

# List current backups if verbose
if [[ "$VERBOSE" == "true" ]]; then
    log "INFO" "Current backup status:"
    $LLMSPELL_CMD backup list --limit 10 || true
fi

log "INFO" "Backup maintenance completed successfully"

# Optional: Send notification (uncomment and configure as needed)
# send_notification() {
#     local subject="$1"
#     local message="$2"
#     
#     # Example: Send email
#     # echo "$message" | mail -s "$subject" admin@example.com
#     
#     # Example: Send to Slack
#     # curl -X POST -H 'Content-type: application/json' \
#     #     --data "{\"text\":\"$subject: $message\"}" \
#     #     "$SLACK_WEBHOOK_URL"
# }
# 
# send_notification "Backup Maintenance Complete" "Successfully completed backup maintenance at $(date)"