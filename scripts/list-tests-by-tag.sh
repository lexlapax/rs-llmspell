#!/bin/bash

# List all tests matching a tag pattern
# Usage: ./scripts/list-tests-by-tag.sh <tag>
#
# This script helps discover what tests will run for a given tag
# without actually running them

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

# Function to print test name
print_test() {
    echo -e "  ${GREEN}✓${NC} $1"
}

# Function to print file
print_file() {
    echo -e "${MAGENTA}📄 $1${NC}"
}

# Check if tag is provided
if [ $# -eq 0 ]; then
    print_error "No tag provided!"
    echo "Usage: $0 <tag>"
    echo ""
    echo "Available tags:"
    echo "  unit        - List unit tests"
    echo "  integration - List integration tests"
    echo "  tool        - List tool tests"
    echo "  ignored     - List ignored tests"
    echo "  all         - List all tests"
    exit 1
fi

TAG=$1

echo "🔍 Listing tests for tag: $TAG"
echo "================================"

case $TAG in
    "unit")
        print_info "Unit tests (in src/ directories):"
        echo ""
        # Find all test modules in src directories
        find . -name "*.rs" -path "*/src/*" -type f | while read -r file; do
            if grep -q "#\[test\]" "$file" 2>/dev/null || grep -q "#\[cfg(test)\]" "$file" 2>/dev/null; then
                print_file "${file#./}"
                grep -n "fn test_" "$file" 2>/dev/null | sed 's/^[[:space:]]*//' | while read -r line; do
                    test_name=$(echo "$line" | sed -E 's/.*fn (test_[a-zA-Z0-9_]+).*/\1/')
                    print_test "$test_name"
                done
            fi
        done
        ;;
        
    "integration")
        print_info "Integration tests (in tests/ directories):"
        echo ""
        # Find all test files in tests directories
        find . -name "*.rs" -path "*/tests/*" -type f | while read -r file; do
            print_file "${file#./}"
            grep -n "fn test_" "$file" 2>/dev/null | sed 's/^[[:space:]]*//' | while read -r line; do
                test_name=$(echo "$line" | sed -E 's/.*fn (test_[a-zA-Z0-9_]+).*/\1/')
                print_test "$test_name"
            done
        done
        ;;
        
    "tool")
        print_info "Tool tests (in llmspell-tools):"
        echo ""
        # Find all test files in llmspell-tools
        find ./llmspell-tools -name "*.rs" -type f | while read -r file; do
            if grep -q "#\[test\]" "$file" 2>/dev/null || grep -q "#\[cfg(test)\]" "$file" 2>/dev/null; then
                print_file "${file#./}"
                grep -n "fn test_" "$file" 2>/dev/null | sed 's/^[[:space:]]*//' | while read -r line; do
                    test_name=$(echo "$line" | sed -E 's/.*fn (test_[a-zA-Z0-9_]+).*/\1/')
                    print_test "$test_name"
                done
            fi
        done
        ;;
        
    "ignored")
        print_info "Ignored tests (marked with #[ignore]):"
        echo ""
        # Find all ignored tests
        find . -name "*.rs" -type f | while read -r file; do
            if grep -B1 "#\[ignore" "$file" 2>/dev/null | grep -q "fn test_"; then
                print_file "${file#./}"
                # Print ignored tests with their ignore reason if available
                grep -B1 "#\[ignore" "$file" 2>/dev/null | grep -A1 "fn test_" | grep "fn test_" | while read -r line; do
                    test_name=$(echo "$line" | sed -E 's/.*fn (test_[a-zA-Z0-9_]+).*/\1/')
                    # Try to find ignore reason
                    reason=$(grep -B5 "$test_name" "$file" | grep "#\[ignore" | sed -E 's/.*#\[ignore[[:space:]]*=[[:space:]]*"([^"]+)".*/\1/' | head -1)
                    if [ -n "$reason" ] && [ "$reason" != "#[ignore]" ]; then
                        print_test "$test_name (reason: $reason)"
                    else
                        print_test "$test_name"
                    fi
                done
            fi
        done
        ;;
        
    "all")
        print_info "All tests in the project:"
        echo ""
        # Count tests by type
        UNIT_COUNT=$(find . -name "*.rs" -path "*/src/*" -type f -exec grep -l "#\[test\]" {} \; 2>/dev/null | wc -l)
        INTEGRATION_COUNT=$(find . -name "*.rs" -path "*/tests/*" -type f | wc -l)
        IGNORED_COUNT=$(find . -name "*.rs" -type f -exec grep -l "#\[ignore" {} \; 2>/dev/null | wc -l)
        
        echo "Summary:"
        echo "  Unit test files: $UNIT_COUNT"
        echo "  Integration test files: $INTEGRATION_COUNT"
        echo "  Files with ignored tests: $IGNORED_COUNT"
        echo ""
        echo "Run with specific tag for detailed listing."
        ;;
        
    *)
        print_error "Unknown tag: $TAG"
        echo ""
        echo "Available tags:"
        echo "  unit        - List unit tests"
        echo "  integration - List integration tests"
        echo "  tool        - List tool tests"
        echo "  ignored     - List ignored tests"
        echo "  all         - List all tests"
        exit 1
        ;;
esac

echo ""
echo "================================"
print_info "Use ./scripts/test-by-tag.sh $TAG to run these tests"