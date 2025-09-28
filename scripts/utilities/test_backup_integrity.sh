#!/bin/bash

# ABOUTME: Backup integrity testing script for operational validation
# ABOUTME: Automated testing of backup creation, validation, and recovery procedures

set -euo pipefail

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
TEST_DIR="$PROJECT_ROOT/target/backup_integrity_tests"
LOG_FILE="$TEST_DIR/backup_integrity_$(date +%Y%m%d_%H%M%S).log"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging functions
log() {
    echo -e "${BLUE}[$(date '+%Y-%m-%d %H:%M:%S')]${NC} $*" | tee -a "$LOG_FILE"
}

log_success() {
    echo -e "${GREEN}[$(date '+%Y-%m-%d %H:%M:%S')] ✅ $*${NC}" | tee -a "$LOG_FILE"
}

log_warning() {
    echo -e "${YELLOW}[$(date '+%Y-%m-%d %H:%M:%S')] ⚠️  $*${NC}" | tee -a "$LOG_FILE"
}

log_error() {
    echo -e "${RED}[$(date '+%Y-%m-%d %H:%M:%S')] ❌ $*${NC}" | tee -a "$LOG_FILE"
}

# Cleanup function
cleanup() {
    if [[ -d "$TEST_DIR" ]]; then
        log "Cleaning up test directory: $TEST_DIR"
        rm -rf "$TEST_DIR"
    fi
}

# Set up cleanup trap
trap cleanup EXIT

# Create test directory
setup_test_environment() {
    log "Setting up test environment..."
    mkdir -p "$TEST_DIR"
    mkdir -p "$TEST_DIR/backups"
    mkdir -p "$TEST_DIR/restore_tests"
    
    log "Test directory: $TEST_DIR"
    log "Log file: $LOG_FILE"
}

# Run backup integration tests
run_backup_integration_tests() {
    log "Running backup integration tests..."
    
    cd "$PROJECT_ROOT"
    
    # Run backup-specific tests
    if cargo test --test "*" -- --test-threads=1 backup_recovery::backup_recovery_tests 2>&1 | tee -a "$LOG_FILE"; then
        log_success "Backup integration tests passed"
        return 0
    else
        log_error "Backup integration tests failed"
        return 1
    fi
}

# Run disaster recovery scenario tests
run_disaster_recovery_tests() {
    log "Running disaster recovery scenario tests..."
    
    cd "$PROJECT_ROOT"
    
    # Run disaster recovery tests
    if cargo test --test "*" -- --test-threads=1 disaster_recovery::disaster_recovery_scenarios 2>&1 | tee -a "$LOG_FILE"; then
        log_success "Disaster recovery tests passed"
        return 0
    else
        log_error "Disaster recovery tests failed"
        return 1
    fi
}

# Test backup performance under load
test_backup_performance() {
    log "Testing backup performance under load..."
    
    cd "$PROJECT_ROOT"
    
    # Run performance-specific backup tests
    if cargo test --test "*" -- --test-threads=1 test_backup_performance_impact 2>&1 | tee -a "$LOG_FILE"; then
        log_success "Backup performance tests passed"
        return 0
    else
        log_error "Backup performance tests failed"
        return 1
    fi
}

# Test concurrent backup operations
test_concurrent_operations() {
    log "Testing concurrent backup operations..."
    
    cd "$PROJECT_ROOT"
    
    # Run concurrent operation tests
    if cargo test --test "*" -- --test-threads=1 test_concurrent_backup_operations 2>&1 | tee -a "$LOG_FILE"; then
        log_success "Concurrent backup tests passed"
        return 0
    else
        log_error "Concurrent backup tests failed"
        return 1
    fi
}

# Test backup retention and cleanup
test_retention_policies() {
    log "Testing backup retention and cleanup policies..."
    
    cd "$PROJECT_ROOT"
    
    # Run retention policy tests
    if cargo test --test "*" -- --test-threads=1 test_backup_retention_and_cleanup 2>&1 | tee -a "$LOG_FILE"; then
        log_success "Backup retention tests passed"
        return 0
    else
        log_error "Backup retention tests failed"
        return 1
    fi
}

# Test data integrity validation
test_data_integrity() {
    log "Testing backup data integrity validation..."
    
    cd "$PROJECT_ROOT"
    
    # Run integrity validation tests
    if cargo test --test "*" -- --test-threads=1 test_backup_integrity_validation 2>&1 | tee -a "$LOG_FILE"; then
        log_success "Data integrity tests passed"
        return 0
    else
        log_error "Data integrity tests failed"
        return 1
    fi
}

# Test incremental backup chains
test_incremental_backups() {
    log "Testing incremental backup chains..."
    
    cd "$PROJECT_ROOT"
    
    # Run incremental backup tests
    if cargo test --test "*" -- --test-threads=1 test_incremental_backup_chain 2>&1 | tee -a "$LOG_FILE"; then
        log_success "Incremental backup tests passed"
        return 0
    else
        log_error "Incremental backup tests failed"
        return 1
    fi
}

# Generate test report
generate_test_report() {
    local exit_code=$1
    
    log "Generating test report..."
    
    local report_file="$TEST_DIR/backup_integrity_report_$(date +%Y%m%d_%H%M%S).md"
    
    cat > "$report_file" << EOF
# Backup Integrity Test Report

**Date**: $(date)
**Test Environment**: $TEST_DIR
**Overall Result**: $([ $exit_code -eq 0 ] && echo "✅ PASSED" || echo "❌ FAILED")

## Test Categories

### 1. Backup Integration Tests
- Complete backup/recovery cycles
- Backup validation
- State persistence verification

### 2. Disaster Recovery Tests
- Complete system failure scenarios
- Partial system failure recovery
- Point-in-time recovery
- Cascading failure recovery
- Recovery under load

### 3. Performance Tests
- Backup creation performance
- Validation performance
- Restore performance
- Large dataset handling

### 4. Concurrent Operations
- Multiple backup operations
- Concurrent restore operations
- Thread safety validation

### 5. Retention Policies
- Automatic cleanup
- Policy enforcement
- Storage optimization

### 6. Data Integrity
- Checksum validation
- Corruption detection
- Round-trip verification

### 7. Incremental Backups
- Backup chain integrity
- Parent-child relationships
- Differential recovery

## Test Results Summary

$(if [ $exit_code -eq 0 ]; then
    echo "All backup integrity tests passed successfully."
    echo ""
    echo "**Key Validations Completed:**"
    echo "- ✅ Full backup and recovery cycles"
    echo "- ✅ Disaster recovery procedures"
    echo "- ✅ Performance under load"
    echo "- ✅ Concurrent operation safety"
    echo "- ✅ Data integrity validation"
    echo "- ✅ Incremental backup chains"
    echo "- ✅ Retention policy enforcement"
else
    echo "Some backup integrity tests failed. Please review the logs for details."
    echo ""
    echo "**Check the following:**"
    echo "- Log file: $LOG_FILE"
    echo "- Test output for specific failures"
    echo "- System resources and permissions"
fi)

## Recommendations

### Operational Readiness
- [ ] Verify backup schedules are configured
- [ ] Test disaster recovery procedures regularly
- [ ] Monitor backup storage usage
- [ ] Validate retention policies match requirements

### Performance Optimization
- [ ] Review backup performance metrics
- [ ] Consider compression settings for large datasets
- [ ] Optimize backup scheduling to minimize system impact

### Security Considerations
- [ ] Verify backup encryption if enabled
- [ ] Ensure backup storage is secure
- [ ] Test access controls for backup operations

---

**Log File**: $LOG_FILE
**Generated**: $(date)
EOF

    log "Test report generated: $report_file"
    
    # Display summary
    echo ""
    log "=== BACKUP INTEGRITY TEST SUMMARY ==="
    if [ $exit_code -eq 0 ]; then
        log_success "All backup integrity tests PASSED"
        log "Backup and recovery systems are operationally ready"
    else
        log_error "Some backup integrity tests FAILED"
        log "Review the logs and address issues before production deployment"
    fi
    log "Full report: $report_file"
    echo ""
}

# Main execution
main() {
    local exit_code=0
    
    log "Starting backup integrity testing..."
    log "Project root: $PROJECT_ROOT"
    
    setup_test_environment
    
    # Run all test categories
    run_backup_integration_tests || exit_code=1
    run_disaster_recovery_tests || exit_code=1
    test_backup_performance || exit_code=1
    test_concurrent_operations || exit_code=1
    test_retention_policies || exit_code=1
    test_data_integrity || exit_code=1
    test_incremental_backups || exit_code=1
    
    generate_test_report $exit_code
    
    exit $exit_code
}

# Script usage
show_usage() {
    cat << EOF
Usage: $0 [OPTIONS]

Backup Integrity Testing Script

OPTIONS:
    -h, --help          Show this help message
    -v, --verbose       Enable verbose output
    --quick             Run only essential tests
    --performance       Run only performance tests
    --disaster          Run only disaster recovery tests

EXAMPLES:
    $0                  # Run all backup integrity tests
    $0 --quick         # Run essential tests only
    $0 --performance   # Run performance tests only
    $0 --disaster      # Run disaster recovery tests only

EOF
}

# Handle command line arguments
case "${1:-}" in
    -h|--help)
        show_usage
        exit 0
        ;;
    --quick)
        log "Running essential backup tests only..."
        setup_test_environment
        run_backup_integration_tests
        exit $?
        ;;
    --performance)
        log "Running performance tests only..."
        setup_test_environment
        test_backup_performance
        exit $?
        ;;
    --disaster)
        log "Running disaster recovery tests only..."
        setup_test_environment
        run_disaster_recovery_tests
        exit $?
        ;;
    "")
        main
        ;;
    *)
        log_error "Unknown option: $1"
        show_usage
        exit 1
        ;;
esac