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
    echo "  all           - Run all test categories (default)"
    echo "  fast          - Run fast test suite (unit + integration)"
    echo "  comprehensive - Run comprehensive test suite (excludes external/benchmark)"
    echo ""
    echo "Primary Types:"
    echo "  unit          - Run unit tests"
    echo "  integration   - Run integration tests"
    echo "  external      - Run external dependency tests"
    echo "  benchmark     - Run benchmark tests"
    echo ""
    echo "Component Categories:"
    echo "  tool          - Run tool tests"
    echo "  agent         - Run agent tests"
    echo "  workflow      - Run workflow tests"
    echo "  bridge        - Run bridge tests"
    echo "  hook          - Run hook tests"
    echo "  event         - Run event tests"
    echo "  session       - Run session tests"
    echo "  state         - Run state tests"
    echo "  util          - Run utility tests"
    echo "  core          - Run core tests"
    echo "  testing       - Run testing utility tests"
    echo ""
    echo "Specialty Categories:"
    echo "  security      - Run security tests"
    echo "  performance   - Run performance tests"
    echo ""
    echo "Legacy (deprecated):"
    echo "  scenario      - Run scenario tests (use 'comprehensive' instead)"
    echo "  lua           - Run Lua bridge tests (use 'bridge' instead)"
    echo ""
    echo "Options:"
    echo "  --quiet     - Suppress detailed output"
    echo "  --verbose   - Show detailed test output"
    echo "  --release   - Run tests in release mode"
    echo "  --nocapture - Don't capture test output"
    echo "  --help      - Show this help message"
    echo ""
    echo "Examples:"
    echo "  $0                         # Run all tests"
    echo "  $0 fast                    # Run fast test suite (unit + integration)"
    echo "  $0 comprehensive           # Run comprehensive test suite"
    echo "  $0 unit --verbose          # Run unit tests with verbose output"
    echo "  $0 external --release      # Run external tests in release mode"
    echo "  $0 tool,agent,workflow     # Run multiple component categories"
}

# Parse command line arguments
CATEGORY=""
CARGO_OPTS=""
QUIET=false
VERBOSE=false

# Check for help first
if [[ "$1" == "--help" ]]; then
    show_help
    exit 0
fi

CATEGORY="${1:-all}"
shift || true

# Parse options
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

# Handle comma-separated categories
if [[ "$CATEGORY" == *","* ]]; then
    print_info "Running multiple categories: $CATEGORY"
    IFS=',' read -ra CATEGORIES <<< "$CATEGORY"
    for cat in "${CATEGORIES[@]}"; do
        cat=$(echo "$cat" | xargs)  # trim whitespace
        case $cat in
            "unit") run_tests "unit" "unit-tests" "Unit tests" || OVERALL_SUCCESS=1 ;;
            "integration") run_tests "integration" "integration-tests" "Integration tests" || OVERALL_SUCCESS=1 ;;
            "external") run_tests "external" "external-tests" "External dependency tests" || OVERALL_SUCCESS=1 ;;
            "benchmark") run_tests "benchmark" "benchmark-tests" "Benchmark tests" || OVERALL_SUCCESS=1 ;;
            "tool") run_tests "tool" "tool-tests" "Tool tests" || OVERALL_SUCCESS=1 ;;
            "agent") run_tests "agent" "agent-tests" "Agent tests" || OVERALL_SUCCESS=1 ;;
            "workflow") run_tests "workflow" "workflow-tests" "Workflow tests" || OVERALL_SUCCESS=1 ;;
            "bridge") run_tests "bridge" "bridge-tests" "Bridge tests" || OVERALL_SUCCESS=1 ;;
            "hook") run_tests "hook" "hook-tests" "Hook tests" || OVERALL_SUCCESS=1 ;;
            "event") run_tests "event" "event-tests" "Event tests" || OVERALL_SUCCESS=1 ;;
            "session") run_tests "session" "session-tests" "Session tests" || OVERALL_SUCCESS=1 ;;
            "state") run_tests "state" "state-tests" "State tests" || OVERALL_SUCCESS=1 ;;
            "util") run_tests "util" "util-tests" "Utility tests" || OVERALL_SUCCESS=1 ;;
            "core") run_tests "core" "core-tests" "Core tests" || OVERALL_SUCCESS=1 ;;
            "testing") run_tests "testing" "testing-tests" "Testing utility tests" || OVERALL_SUCCESS=1 ;;
            "security") run_tests "security" "security-tests" "Security tests" || OVERALL_SUCCESS=1 ;;
            "performance") run_tests "performance" "performance-tests" "Performance tests" || OVERALL_SUCCESS=1 ;;
            *) print_error "Unknown category: $cat"; OVERALL_SUCCESS=1 ;;
        esac
    done
else
    case $CATEGORY in
        "all")
            print_info "Running all test categories..."
            run_tests "all" "all-tests" "All tests" || OVERALL_SUCCESS=1
            ;;
            
        "fast")
            print_info "Running fast test suite..."
            run_tests "fast" "fast-tests" "Fast test suite (unit + integration)" || OVERALL_SUCCESS=1
            ;;
            
        "comprehensive")
            print_info "Running comprehensive test suite..."
            run_tests "comprehensive" "comprehensive-tests" "Comprehensive test suite" || OVERALL_SUCCESS=1
            ;;
            
        # Primary categories
        "unit")
            run_tests "unit" "unit-tests" "Unit tests" || OVERALL_SUCCESS=1
            ;;
            
        "integration")
            run_tests "integration" "integration-tests" "Integration tests" || OVERALL_SUCCESS=1
            ;;
            
        "external")
            run_tests "external" "external-tests" "External dependency tests" || OVERALL_SUCCESS=1
            ;;
            
        "benchmark")
            run_tests "benchmark" "benchmark-tests" "Benchmark tests" || OVERALL_SUCCESS=1
            ;;
            
        # Component categories
        "tool")
            run_tests "tool" "tool-tests" "Tool tests" || OVERALL_SUCCESS=1
            ;;
            
        "agent")
            run_tests "agent" "agent-tests" "Agent tests" || OVERALL_SUCCESS=1
            ;;
            
        "workflow")
            run_tests "workflow" "workflow-tests" "Workflow tests" || OVERALL_SUCCESS=1
            ;;
            
        "bridge")
            run_tests "bridge" "bridge-tests" "Bridge tests" || OVERALL_SUCCESS=1
            ;;
            
        "hook")
            run_tests "hook" "hook-tests" "Hook tests" || OVERALL_SUCCESS=1
            ;;
            
        "event")
            run_tests "event" "event-tests" "Event tests" || OVERALL_SUCCESS=1
            ;;
            
        "session")
            run_tests "session" "session-tests" "Session tests" || OVERALL_SUCCESS=1
            ;;
            
        "state")
            run_tests "state" "state-tests" "State tests" || OVERALL_SUCCESS=1
            ;;
            
        "util")
            run_tests "util" "util-tests" "Utility tests" || OVERALL_SUCCESS=1
            ;;
            
        "core")
            run_tests "core" "core-tests" "Core tests" || OVERALL_SUCCESS=1
            ;;
            
        "testing")
            run_tests "testing" "testing-tests" "Testing utility tests" || OVERALL_SUCCESS=1
            ;;
            
        # Specialty categories
        "security")
            run_tests "security" "security-tests" "Security tests" || OVERALL_SUCCESS=1
            ;;
            
        "performance")
            run_tests "performance" "performance-tests" "Performance tests" || OVERALL_SUCCESS=1
            ;;
            
        # Legacy categories (deprecated)
        "scenario")
            print_warning "'scenario' category is deprecated, use 'comprehensive' instead"
            run_tests "scenario" "scenario-tests" "Scenario tests (deprecated)" || OVERALL_SUCCESS=1
            ;;
            
        "lua")
            print_warning "'lua' category is deprecated, use 'bridge' instead"
            run_tests "lua" "lua-tests" "Lua bridge tests (deprecated)" || OVERALL_SUCCESS=1
            ;;
            
        *)
            print_error "Unknown category: $CATEGORY"
            echo ""
            show_help
            exit 1
            ;;
    esac
fi

# Summary
echo ""
echo "======================"
if [ $OVERALL_SUCCESS -eq 0 ]; then
    print_success "All tests passed! 🎉"
    
    if [ "$QUIET" != "true" ] && [ "$CATEGORY" = "all" ]; then
        echo ""
        print_info "For detailed coverage report, run:"
        echo "  ./scripts/test-coverage.sh"
        echo ""
        print_info "Available test categories:"
        echo "  Primary: unit, integration, external, benchmark"
        echo "  Components: tool, agent, workflow, bridge, hook, event, session, state, util, core, testing"
        echo "  Specialty: security, performance"
        echo "  Suites: fast, comprehensive, all"
    fi
else
    print_error "Some tests failed! 💥"
    exit 1
fi