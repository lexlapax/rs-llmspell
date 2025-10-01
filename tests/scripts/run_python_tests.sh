#!/bin/bash
#
# Run Python integration tests for llmspell Jupyter DAP functionality
#
# This script:
# 1. Sets up a Python virtual environment
# 2. Installs required dependencies
# 3. Runs pytest with appropriate configuration
# 4. Reports results and cleans up
#
# Exit codes:
#   0 - All tests passed
#   1 - Test failures
#   2 - Setup/environment errors

set -e  # Exit on error

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Get script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
TESTS_DIR="${PROJECT_ROOT}/tests/python"

# Configuration
VENV_DIR="${TESTS_DIR}/venv"
PYTHON_VERSION="python3"
PYTEST_ARGS="${PYTEST_ARGS:-}"  # Allow override via environment

# Logging functions
log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check Python availability
check_python() {
    if ! command -v $PYTHON_VERSION &> /dev/null; then
        log_error "Python 3 is not installed or not in PATH"
        exit 2
    fi

    PY_VERSION=$($PYTHON_VERSION --version 2>&1)
    log_info "Using $PY_VERSION"
}

# Setup virtual environment
setup_venv() {
    log_info "Setting up Python virtual environment..."

    cd "$TESTS_DIR"

    # Create venv if it doesn't exist
    if [ ! -d "$VENV_DIR" ]; then
        log_info "Creating new virtual environment..."
        $PYTHON_VERSION -m venv "$VENV_DIR"
    else
        log_info "Using existing virtual environment"
    fi

    # Activate venv
    source "${VENV_DIR}/bin/activate"

    # Upgrade pip
    log_info "Upgrading pip..."
    pip install --quiet --upgrade pip

    # Install or update dependencies
    log_info "Installing/updating dependencies..."
    pip install --quiet --requirement requirements.txt

    # Show installed packages for debugging
    if [ "${VERBOSE:-0}" = "1" ]; then
        log_info "Installed packages:"
        pip list
    fi
}

# Build llmspell if needed
build_llmspell() {
    log_info "Building llmspell..."

    cd "$PROJECT_ROOT"

    # Check if we need to rebuild
    BINARY="${PROJECT_ROOT}/target/debug/llmspell"
    if [ ! -f "$BINARY" ] || [ "${FORCE_BUILD:-0}" = "1" ]; then
        cargo build -p llmspell-cli
        if [ $? -ne 0 ]; then
            log_error "Failed to build llmspell"
            exit 2
        fi
    else
        log_info "llmspell binary already exists (use FORCE_BUILD=1 to rebuild)"
    fi
}

# Run the tests
run_tests() {
    log_info "Running Python integration tests..."

    cd "$TESTS_DIR"

    # Prepare pytest arguments
    ARGS="-v"  # Verbose by default
    ARGS="$ARGS --tb=short"  # Short traceback format
    ARGS="$ARGS --timeout=60"  # 60 second timeout per test
    ARGS="$ARGS --color=yes"  # Color output

    # Add coverage if requested
    if [ "${WITH_COVERAGE:-0}" = "1" ]; then
        ARGS="$ARGS --cov=. --cov-report=term-missing"
    fi

    # Add specific test file or all DAP tests
    if [ -n "${TEST_FILE:-}" ]; then
        ARGS="$ARGS $TEST_FILE"
    else
        # Run all DAP-related tests
        ARGS="$ARGS test_dap*.py"
    fi

    # Add any additional arguments from environment
    if [ -n "$PYTEST_ARGS" ]; then
        ARGS="$ARGS $PYTEST_ARGS"
    fi

    # Run pytest
    log_info "Executing: pytest $ARGS"
    pytest $ARGS

    TEST_RESULT=$?

    return $TEST_RESULT
}

# Cleanup function
cleanup() {
    log_info "Cleaning up..."

    # Kill any orphaned llmspell processes
    if command -v pkill &> /dev/null; then
        pkill -f "llmspell kernel start --daemon" || true
    fi

    # Deactivate virtual environment if active
    if [ -n "${VIRTUAL_ENV:-}" ]; then
        deactivate 2>/dev/null || true
    fi
}

# Main execution
main() {
    log_info "Starting llmspell Python integration tests"
    log_info "Project root: $PROJECT_ROOT"

    # Set up trap for cleanup
    trap cleanup EXIT

    # Check prerequisites
    check_python

    # Build llmspell
    build_llmspell

    # Setup Python environment
    setup_venv

    # Run tests
    run_tests
    TEST_RESULT=$?

    # Report results
    if [ $TEST_RESULT -eq 0 ]; then
        log_info "All tests passed successfully!"
    else
        log_error "Some tests failed. See output above for details."
    fi

    exit $TEST_RESULT
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --help|-h)
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --verbose, -v      Show verbose output"
            echo "  --coverage         Run with coverage reporting"
            echo "  --force-build      Force rebuild of llmspell"
            echo "  --test FILE        Run specific test file"
            echo "  --help, -h         Show this help message"
            echo ""
            echo "Environment variables:"
            echo "  VERBOSE=1          Enable verbose output"
            echo "  WITH_COVERAGE=1    Enable coverage reporting"
            echo "  FORCE_BUILD=1      Force rebuild of llmspell"
            echo "  TEST_FILE=<file>   Run specific test file"
            echo "  PYTEST_ARGS=<args> Additional pytest arguments"
            exit 0
            ;;
        --verbose|-v)
            export VERBOSE=1
            shift
            ;;
        --coverage)
            export WITH_COVERAGE=1
            shift
            ;;
        --force-build)
            export FORCE_BUILD=1
            shift
            ;;
        --test)
            export TEST_FILE="$2"
            shift 2
            ;;
        *)
            log_error "Unknown option: $1"
            exit 1
            ;;
    esac
done

# Run main function
main