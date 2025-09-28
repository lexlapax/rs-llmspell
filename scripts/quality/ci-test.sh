#!/bin/bash

# CI Test Runner with Configurable Test Levels
# Provides consistent test execution for both local and CI environments

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$PROJECT_ROOT"

# Test level from environment or argument
TEST_LEVEL="${TEST_LEVEL:-${1:-fast}}"
REPORT_DIR="${REPORT_DIR:-./test-reports}"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)

# Create report directory
mkdir -p "$REPORT_DIR"

# Log function
log() {
    echo -e "${BLUE}[CI-TEST]${NC} $1"
}

error() {
    echo -e "${RED}[ERROR]${NC} $1" >&2
}

success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

# Timer functions
start_timer() {
    START_TIME=$(date +%s)
}

end_timer() {
    END_TIME=$(date +%s)
    DURATION=$((END_TIME - START_TIME))
    echo "$DURATION"
}

# Test execution functions
run_minimal_tests() {
    log "Running MINIMAL test suite (formatting and linting)"

    log "Checking code formatting..."
    cargo fmt --all -- --check || {
        error "Code formatting check failed"
        return 1
    }

    log "Running clippy..."
    cargo clippy --workspace --all-targets --all-features -- \
        -D warnings \
        -W clippy::pedantic \
        -A clippy::module_name_repetitions \
        -A clippy::must_use_candidate \
        -A clippy::missing_errors_doc \
        -A clippy::missing_panics_doc \
        -A clippy::similar_names || {
        error "Clippy check failed"
        return 1
    }

    log "Checking compilation..."
    cargo check --workspace --all-features || {
        error "Compilation check failed"
        return 1
    }

    success "Minimal tests passed"
}

run_fast_tests() {
    log "Running FAST test suite (unit tests + minimal checks)"

    # Run minimal tests first
    run_minimal_tests || return 1

    log "Building debug binary..."
    cargo build --workspace --all-features || {
        error "Build failed"
        return 1
    }

    log "Running unit tests..."
    cargo test --workspace --lib --bins -- --test-threads=4 || {
        error "Unit tests failed"
        return 1
    }

    log "Building documentation..."
    cargo doc --workspace --no-deps --all-features || {
        error "Documentation build failed"
        return 1
    }

    success "Fast tests passed"
}

run_full_tests() {
    log "Running FULL test suite (all tests + application validation)"

    # Run fast tests first
    run_fast_tests || return 1

    log "Running integration tests..."
    cargo test --workspace --all-features -- --test-threads=2 || {
        error "Integration tests failed"
        return 1
    }

    log "Running doc tests..."
    cargo test --doc --workspace --all-features || {
        error "Doc tests failed"
        return 1
    }

    log "Running application validation suite..."
    python3 scripts/validate_applications.py \
        --json "$REPORT_DIR/validation-${TIMESTAMP}.json" \
        --html "$REPORT_DIR/validation-${TIMESTAMP}.html" || {
        error "Application validation failed"
        warning "Check report at $REPORT_DIR/validation-${TIMESTAMP}.html"
        return 1
    }

    success "Full tests passed"
}

run_expensive_tests() {
    log "Running EXPENSIVE test suite (includes webapp-creator)"

    # Set environment variable for expensive tests
    export RUN_EXPENSIVE_TESTS=1

    # Run full tests
    run_full_tests || return 1

    log "Note: webapp-creator test adds 8-10 minutes to test runtime"

    success "Expensive tests passed (including webapp-creator)"
}

run_coverage_tests() {
    log "Running COVERAGE test suite (requires cargo-tarpaulin)"

    # Check if tarpaulin is installed
    if ! command -v cargo-tarpaulin &> /dev/null; then
        warning "cargo-tarpaulin not installed, installing..."
        cargo install cargo-tarpaulin || {
            error "Failed to install cargo-tarpaulin"
            return 1
        }
    }

    log "Running tests with coverage..."
    cargo tarpaulin \
        --workspace \
        --all-features \
        --out Html \
        --output-dir "$REPORT_DIR" \
        --timeout 300 \
        --skip-clean || {
        error "Coverage tests failed"
        return 1
    }

    mv "$REPORT_DIR/tarpaulin-report.html" "$REPORT_DIR/coverage-${TIMESTAMP}.html"

    success "Coverage report generated at $REPORT_DIR/coverage-${TIMESTAMP}.html"
}

# Performance tracking
track_performance() {
    local test_name="$1"
    local duration="$2"

    # Create performance log
    echo "$(date +%Y-%m-%d_%H:%M:%S),${TEST_LEVEL},${test_name},${duration}" >> "$REPORT_DIR/performance.csv"

    # Check against expected times
    case "$TEST_LEVEL" in
        minimal)
            if [ "$duration" -gt 60 ]; then
                warning "Minimal tests took ${duration}s (expected <60s)"
            fi
            ;;
        fast)
            if [ "$duration" -gt 120 ]; then
                warning "Fast tests took ${duration}s (expected <120s)"
            fi
            ;;
        full)
            if [ "$duration" -gt 600 ]; then
                warning "Full tests took ${duration}s (expected <600s)"
            fi
            ;;
    esac
}

# Main execution
main() {
    log "Starting CI test run"
    log "Test Level: $TEST_LEVEL"
    log "Report Directory: $REPORT_DIR"

    # Create summary file
    SUMMARY_FILE="$REPORT_DIR/summary-${TIMESTAMP}.txt"
    {
        echo "CI Test Summary"
        echo "==============="
        echo "Date: $(date)"
        echo "Test Level: $TEST_LEVEL"
        echo "Rust Version: $(rustc --version)"
        echo "Cargo Version: $(cargo --version)"
        echo ""
    } > "$SUMMARY_FILE"

    # Start overall timer
    start_timer

    # Execute tests based on level
    case "$TEST_LEVEL" in
        minimal|min)
            run_minimal_tests
            TEST_RESULT=$?
            ;;
        fast)
            run_fast_tests
            TEST_RESULT=$?
            ;;
        full)
            run_full_tests
            TEST_RESULT=$?
            ;;
        expensive|all)
            run_expensive_tests
            TEST_RESULT=$?
            ;;
        coverage|cov)
            run_coverage_tests
            TEST_RESULT=$?
            ;;
        *)
            error "Unknown test level: $TEST_LEVEL"
            error "Valid levels: minimal, fast, full, expensive, coverage"
            exit 1
            ;;
    esac

    # End timer and track performance
    DURATION=$(end_timer)
    track_performance "$TEST_LEVEL" "$DURATION"

    # Write summary
    {
        echo "Test Result: $([ $TEST_RESULT -eq 0 ] && echo "PASSED" || echo "FAILED")"
        echo "Duration: ${DURATION} seconds"
        echo ""
        echo "Reports generated in: $REPORT_DIR"
    } >> "$SUMMARY_FILE"

    # Print summary
    log "Test execution completed in ${DURATION} seconds"

    if [ $TEST_RESULT -eq 0 ]; then
        success "All tests passed for level: $TEST_LEVEL"

        # Print next steps
        case "$TEST_LEVEL" in
            minimal)
                log "Consider running: TEST_LEVEL=fast $0"
                ;;
            fast)
                log "Consider running: TEST_LEVEL=full $0"
                ;;
            full)
                log "Consider running: TEST_LEVEL=expensive $0"
                ;;
        esac
    else
        error "Tests failed for level: $TEST_LEVEL"
        error "Check reports in: $REPORT_DIR"
    fi

    exit $TEST_RESULT
}

# Handle interrupts
trap 'error "Test interrupted"; exit 1' INT TERM

# Run main
main