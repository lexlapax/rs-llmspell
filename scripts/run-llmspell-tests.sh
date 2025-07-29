#!/bin/bash

# Comprehensive test runner for llmspell-testing
# This script provides a unified interface to run all test categories

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
MAGENTA='\033[0;35m'
NC='\033[0m' # No Color

# Function to print info
print_info() {
    echo -e "${BLUE}ℹ️  $1${NC}"
}

# Function to print error
print_error() {
    echo -e "${RED}❌ $1${NC}"
}

# Function to print success
print_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

# Function to print warning
print_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

# Display help
show_help() {
    echo "🧪 llmspell Test Runner"
    echo "====================="
    echo ""
    echo "Usage: $0 [category] [options]"
    echo ""
    echo "Categories:"
    echo "  all         - Run all test categories (default)"
    echo "  unit        - Run unit tests"
    echo "  integration - Run integration tests"
    echo "  agent       - Run agent tests"
    echo "  scenario    - Run scenario tests"
    echo "  lua         - Run Lua bridge tests"
    echo "  performance - Run performance benchmarks"
    echo ""
    echo "Options:"
    echo "  --quiet     - Suppress detailed output"
    echo "  --verbose   - Show detailed test output"
    echo "  --release   - Run tests in release mode"
    echo "  --nocapture - Don't capture test output"
    echo "  --help      - Show this help message"
    echo ""
    echo "Examples:"
    echo "  $0                    # Run all tests"
    echo "  $0 unit               # Run only unit tests"
    echo "  $0 integration --verbose  # Run integration tests with verbose output"
    echo "  $0 performance --release  # Run performance tests in release mode"
}

# Parse command line arguments
CATEGORY="${1:-all}"
shift || true

# Parse options
CARGO_OPTS=""
QUIET=false
VERBOSE=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --quiet)
            QUIET=true
            CARGO_OPTS="$CARGO_OPTS --quiet"
            shift
            ;;
        --verbose)
            VERBOSE=true
            shift
            ;;
        --release)
            CARGO_OPTS="$CARGO_OPTS --release"
            shift
            ;;
        --nocapture)
            CARGO_OPTS="$CARGO_OPTS -- --nocapture"
            shift
            ;;
        --help)
            show_help
            exit 0
            ;;
        *)
            print_error "Unknown option: $1"
            echo ""
            show_help
            exit 1
            ;;
    esac
done

# Function to run test category
run_tests() {
    local category=$1
    local feature=$2
    local description=$3
    
    if [ "$QUIET" != "true" ]; then
        echo ""
        print_info "Running $description..."
    fi
    
    if [ "$VERBOSE" = "true" ]; then
        cargo test -p llmspell-testing --features $feature $CARGO_OPTS
    else
        if cargo test -p llmspell-testing --features $feature $CARGO_OPTS > /tmp/test_output_$$.log 2>&1; then
            print_success "$description passed"
            return 0
        else
            print_error "$description failed"
            if [ "$QUIET" != "true" ]; then
                echo ""
                echo "Last 20 lines of output:"
                tail -20 /tmp/test_output_$$.log
                echo ""
                echo "Full output saved to: /tmp/test_output_$$.log"
            fi
            return 1
        fi
    fi
}

# Function to run performance benchmarks
run_benchmarks() {
    if [ "$QUIET" != "true" ]; then
        echo ""
        print_info "Running performance benchmarks..."
    fi
    
    cd llmspell-testing
    
    if [ "$VERBOSE" = "true" ]; then
        cargo bench --features performance-tests $CARGO_OPTS
    else
        if cargo bench --features performance-tests $CARGO_OPTS > /tmp/bench_output_$$.log 2>&1; then
            print_success "Performance benchmarks completed"
            if [ "$QUIET" != "true" ]; then
                # Show summary of benchmark results
                echo ""
                echo "Benchmark Summary:"
                grep -E "(time:|thrpt:|overhead:)" /tmp/bench_output_$$.log | tail -10
            fi
            return 0
        else
            print_error "Performance benchmarks failed"
            if [ "$QUIET" != "true" ]; then
                echo ""
                echo "Last 20 lines of output:"
                tail -20 /tmp/bench_output_$$.log
            fi
            return 1
        fi
    fi
    
    cd ..
}

# Main execution
echo "🧪 llmspell Test Runner"
echo "======================"
print_info "Category: $CATEGORY"

# Track overall success
OVERALL_SUCCESS=0

case $CATEGORY in
    "all")
        print_info "Running all test categories..."
        
        run_tests "unit" "unit-tests" "Unit tests" || OVERALL_SUCCESS=1
        run_tests "integration" "integration-tests" "Integration tests" || OVERALL_SUCCESS=1
        run_tests "agent" "agent-tests" "Agent tests" || OVERALL_SUCCESS=1
        run_tests "scenario" "scenario-tests" "Scenario tests" || OVERALL_SUCCESS=1
        run_tests "lua" "lua-tests" "Lua bridge tests" || OVERALL_SUCCESS=1
        ;;
        
    "unit")
        run_tests "unit" "unit-tests" "Unit tests" || OVERALL_SUCCESS=1
        ;;
        
    "integration")
        run_tests "integration" "integration-tests" "Integration tests" || OVERALL_SUCCESS=1
        ;;
        
    "agent")
        run_tests "agent" "agent-tests" "Agent tests" || OVERALL_SUCCESS=1
        ;;
        
    "scenario")
        run_tests "scenario" "scenario-tests" "Scenario tests" || OVERALL_SUCCESS=1
        ;;
        
    "lua")
        run_tests "lua" "lua-tests" "Lua bridge tests" || OVERALL_SUCCESS=1
        ;;
        
    "performance")
        run_benchmarks || OVERALL_SUCCESS=1
        ;;
        
    *)
        print_error "Unknown category: $CATEGORY"
        echo ""
        show_help
        exit 1
        ;;
esac

# Summary
echo ""
echo "======================"
if [ $OVERALL_SUCCESS -eq 0 ]; then
    print_success "All tests passed! 🎉"
    
    if [ "$QUIET" != "true" ] && [ "$CATEGORY" = "all" ]; then
        echo ""
        print_info "For detailed coverage report, run:"
        echo "  ./scripts/test-coverage.sh"
    fi
else
    print_error "Some tests failed! 💥"
    exit 1
fi