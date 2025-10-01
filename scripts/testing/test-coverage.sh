#!/bin/bash

# Test coverage report generator for llmspell-testing
# Uses tarpaulin to generate coverage reports

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
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

# Check if tarpaulin is installed
if ! command -v cargo-tarpaulin &> /dev/null; then
    print_error "cargo-tarpaulin is not installed!"
    echo "Install it with: cargo install cargo-tarpaulin"
    exit 1
fi

echo "📊 Generating Test Coverage Report"
echo "=================================="

# Parse arguments
COVERAGE_TYPE="${1:-all}"
OUTPUT_FORMAT="${2:-html}"

case $COVERAGE_TYPE in
    "unit")
        print_info "Running coverage for unit tests..."
        FEATURES="--features unit-tests"
        ;;
    "integration")
        print_info "Running coverage for integration tests..."
        FEATURES="--features integration-tests"
        ;;
    "all")
        print_info "Running coverage for all tests..."
        FEATURES="--features all-tests"
        ;;
    *)
        print_error "Unknown coverage type: $COVERAGE_TYPE"
        echo ""
        echo "Usage: $0 [coverage_type] [output_format]"
        echo ""
        echo "Coverage types:"
        echo "  unit        - Coverage for unit tests only"
        echo "  integration - Coverage for integration tests only"
        echo "  all         - Coverage for all tests (default)"
        echo ""
        echo "Output formats:"
        echo "  html        - HTML report (default)"
        echo "  lcov        - LCOV format for CI"
        echo "  json        - JSON format"
        exit 1
        ;;
esac

# Set output options based on format
case $OUTPUT_FORMAT in
    "html")
        OUTPUT_OPTS="--out Html"
        ;;
    "lcov")
        OUTPUT_OPTS="--out Lcov"
        ;;
    "json")
        OUTPUT_OPTS="--out Json"
        ;;
    *)
        print_error "Unknown output format: $OUTPUT_FORMAT"
        exit 1
        ;;
esac

# Run tarpaulin
print_info "Running tarpaulin with options: $FEATURES $OUTPUT_OPTS"
echo ""

if cargo tarpaulin \
    --workspace \
    --exclude llmspell-cli \
    --exclude llmspell-testing \
    $FEATURES \
    $OUTPUT_OPTS \
    --timeout 300 \
    --avoid-cfg-tarpaulin; then
    
    print_success "Coverage report generated successfully!"
    
    if [ "$OUTPUT_FORMAT" = "html" ]; then
        echo ""
        print_info "HTML report available at: tarpaulin-report.html"
        print_info "Open with: open tarpaulin-report.html"
    fi
else
    print_error "Coverage generation failed!"
    exit 1
fi

# Summary
echo ""
echo "=================================="
print_info "Coverage complete for: $COVERAGE_TYPE tests"
print_info "Output format: $OUTPUT_FORMAT"