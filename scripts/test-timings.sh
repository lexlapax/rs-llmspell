#!/bin/bash

# Script to run tests and show execution times
# Helps identify which tests should be marked as "slow"

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

# Check if package is provided
PACKAGE=""
if [ $# -gt 0 ]; then
    PACKAGE="-p $1"
    print_info "Running tests for package: $1"
else
    print_info "Running all tests"
fi

echo "⏱️  Running Tests with Timing Information"
echo "========================================"
echo ""

# Run tests with timing information
# The --nocapture flag shows test output including timing
# The -- --test-threads=1 ensures consistent timing
print_info "Running tests single-threaded for accurate timing..."
echo ""

# Create a temporary file to store results
TEMP_FILE=$(mktemp)

# Run tests and capture output
if [ -n "$PACKAGE" ]; then
    cargo test $PACKAGE -- --nocapture --test-threads=1 2>&1 | tee "$TEMP_FILE"
else
    cargo test -- --nocapture --test-threads=1 2>&1 | tee "$TEMP_FILE"
fi

echo ""
echo "========================================"
echo "Test Timing Summary"
echo "========================================"
echo ""

# Extract test results and sort by time
print_info "Tests taking more than 1 second (candidates for #[ignore = \"slow\"]):"
echo ""

# Parse the output for test timings
grep -E "test .* \.\.\. ok" "$TEMP_FILE" | while read -r line; do
    # Extract test name and time
    test_name=$(echo "$line" | sed -E 's/test (.*) \.\.\. ok.*/\1/')
    
    # Try to extract time if it's in the output
    if [[ $line =~ ([0-9]+\.[0-9]+)s ]]; then
        time="${BASH_REMATCH[1]}"
        # Check if time is greater than 1 second
        if (( $(echo "$time > 1.0" | bc -l) )); then
            echo -e "${YELLOW}⚠️  $test_name - ${time}s${NC}"
        fi
    fi
done

# Also check for tests that might have timing in their output
echo ""
print_info "Tests with explicit sleep/delay operations:"
echo ""

# Search for sleep patterns in test files
if [ -n "$PACKAGE" ]; then
    find . -name "*.rs" -path "*/$1/*" -type f | while read -r file; do
        if grep -q "tokio::time::sleep\|thread::sleep\|Duration::from_secs" "$file" 2>/dev/null; then
            echo "📄 ${file#./}"
            grep -n "sleep\|Duration::from_secs" "$file" | head -3
            echo ""
        fi
    done
else
    find . -name "*.rs" -path "*/tests/*" -type f | while read -r file; do
        if grep -q "tokio::time::sleep\|thread::sleep\|Duration::from_secs" "$file" 2>/dev/null; then
            echo "📄 ${file#./}"
            grep -n "sleep\|Duration::from_secs" "$file" | head -3
            echo ""
        fi
    done
fi

# Clean up
rm -f "$TEMP_FILE"

echo ""
echo "========================================"
print_info "Recommendation: Tests taking >1s should be marked with #[ignore = \"slow\"]"
print_info "This allows developers to run fast tests during development with:"
echo "  cargo test              # Skip slow tests"
echo "  cargo test -- --ignored # Run only slow tests"