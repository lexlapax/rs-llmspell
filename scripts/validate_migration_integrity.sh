#!/bin/bash
# ABOUTME: Validates migration integrity by running comprehensive tests
# ABOUTME: Checks data consistency, rollback functionality, and performance

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

echo -e "${BLUE}=== State Migration Integrity Validation ===${NC}"
echo "Validating migration framework integrity..."
echo

# Function to run tests with specific tag
run_tagged_tests() {
    local tag=$1
    local description=$2
    
    echo -e "${YELLOW}Running ${description}...${NC}"
    if cargo test --package llmspell-state-persistence --lib migration -- --nocapture 2>&1 | grep -E "(test result:|passed|failed)"; then
        echo -e "${GREEN}✓ ${description} passed${NC}"
    else
        echo -e "${RED}✗ ${description} failed${NC}"
        return 1
    fi
    echo
}

# Function to check migration test data
check_test_data() {
    echo -e "${YELLOW}Checking migration test data...${NC}"
    
    local test_data_dir="${PROJECT_ROOT}/tests/data/migration_test_cases"
    if [ ! -d "$test_data_dir" ]; then
        echo -e "${RED}✗ Test data directory not found: $test_data_dir${NC}"
        return 1
    fi
    
    local required_files=(
        "v1_to_v2_user_schema.json"
        "complex_nested_migration.json"
        "error_scenarios.json"
    )
    
    local all_found=true
    for file in "${required_files[@]}"; do
        if [ -f "$test_data_dir/$file" ]; then
            echo -e "${GREEN}✓ Found: $file${NC}"
        else
            echo -e "${RED}✗ Missing: $file${NC}"
            all_found=false
        fi
    done
    
    if [ "$all_found" = true ]; then
        echo -e "${GREEN}✓ All test data files present${NC}"
    else
        echo -e "${RED}✗ Some test data files missing${NC}"
        return 1
    fi
    echo
}

# Function to run integration tests
run_integration_tests() {
    echo -e "${YELLOW}Running migration integration tests...${NC}"
    
    # Run the integration tests
    if cargo test --test state_migration -- --nocapture; then
        echo -e "${GREEN}✓ Integration tests passed${NC}"
    else
        echo -e "${RED}✗ Integration tests failed${NC}"
        return 1
    fi
    echo
}

# Function to run performance benchmarks
run_performance_benchmarks() {
    echo -e "${YELLOW}Running migration performance benchmarks...${NC}"
    
    # Check if benchmark file exists
    if [ ! -f "${PROJECT_ROOT}/tests/performance/migration_performance.rs" ]; then
        echo -e "${YELLOW}! Performance benchmarks not yet implemented${NC}"
        return 0
    fi
    
    # Run benchmarks (quick mode for validation)
    if cargo bench --bench migration_performance -- --quick; then
        echo -e "${GREEN}✓ Performance benchmarks completed${NC}"
    else
        echo -e "${RED}✗ Performance benchmarks failed${NC}"
        return 1
    fi
    echo
}

# Function to validate migration scenarios
validate_migration_scenarios() {
    echo -e "${YELLOW}Validating migration scenarios...${NC}"
    
    # Test basic migration
    echo "1. Testing basic field transformation..."
    cargo test test_complex_schema_migration -- --exact --nocapture || return 1
    
    # Test performance
    echo "2. Testing large dataset migration..."
    cargo test test_large_dataset_migration_performance -- --exact --nocapture || return 1
    
    # Test multi-step
    echo "3. Testing multi-step migration chain..."
    cargo test test_multi_step_migration_chain -- --exact --nocapture || return 1
    
    # Test rollback
    echo "4. Testing migration rollback..."
    cargo test test_migration_rollback_on_error -- --exact --nocapture || return 1
    
    # Test data integrity
    echo "5. Testing data integrity..."
    cargo test test_migration_data_integrity -- --exact --nocapture || return 1
    
    # Test concurrency
    echo "6. Testing concurrent migration safety..."
    cargo test test_concurrent_migration_safety -- --exact --nocapture || return 1
    
    echo -e "${GREEN}✓ All migration scenarios validated${NC}"
    echo
}

# Function to check migration documentation
check_documentation() {
    echo -e "${YELLOW}Checking migration documentation...${NC}"
    
    # Check if migration module has proper docs
    if cargo doc --package llmspell-state-persistence --no-deps 2>&1 | grep -q "warning"; then
        echo -e "${YELLOW}! Documentation warnings found${NC}"
    else
        echo -e "${GREEN}✓ Documentation builds cleanly${NC}"
    fi
    echo
}

# Main validation flow
main() {
    local exit_code=0
    
    # Change to project root
    cd "$PROJECT_ROOT"
    
    # Run all validations
    check_test_data || exit_code=1
    run_tagged_tests "migration" "Migration unit tests" || exit_code=1
    run_integration_tests || exit_code=1
    run_performance_benchmarks || exit_code=1
    validate_migration_scenarios || exit_code=1
    check_documentation || exit_code=1
    
    # Summary
    echo -e "${BLUE}=== Validation Summary ===${NC}"
    if [ $exit_code -eq 0 ]; then
        echo -e "${GREEN}✓ All migration integrity checks passed!${NC}"
        echo -e "${GREEN}✓ Migration framework is ready for use${NC}"
    else
        echo -e "${RED}✗ Some validation checks failed${NC}"
        echo -e "${RED}✗ Please fix the issues before proceeding${NC}"
    fi
    
    exit $exit_code
}

# Run main function
main "$@"