#!/bin/bash

# Run tests matching multiple tags
# Usage: ./scripts/test-multiple-tags.sh "tool,integration" [additional cargo test args]
#
# This script allows running tests that match multiple criteria
# For example: "tool,fast" runs fast tests in the tools package

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

# Check if tags are provided
if [ $# -eq 0 ]; then
    print_error "No tags provided!"
    echo "Usage: $0 \"tag1,tag2\" [additional cargo test args]"
    echo ""
    echo "Examples:"
    echo "  $0 \"tool,fast\"         - Run fast tests in tools package"
    echo "  $0 \"unit,!slow\"        - Run unit tests excluding slow ones"
    echo "  $0 \"integration,tool\"  - Run integration tests in tools"
    exit 1
fi

TAGS=$1
shift

echo "🏷️  Running tests matching tags: $TAGS"
echo "======================================"

# Parse comma-separated tags
IFS=',' read -ra TAG_ARRAY <<< "$TAGS"

# Build cargo test command based on tags
CARGO_CMD="cargo test"
TEST_ARGS=""
PACKAGE=""
TEST_TYPE=""

for tag in "${TAG_ARRAY[@]}"; do
    # Remove leading/trailing whitespace
    tag=$(echo "$tag" | xargs)
    
    # Handle negation
    if [[ $tag == !* ]]; then
        tag="${tag:1}"
        print_info "Excluding tag: $tag"
        # Add exclusion logic here if needed
        continue
    fi
    
    case $tag in
        "unit")
            TEST_TYPE="--lib"
            ;;
        "integration")
            TEST_TYPE="--test '*'"
            ;;
        "tool")
            PACKAGE="-p llmspell-tools"
            ;;
        "agent")
            PACKAGE="-p llmspell-agents"
            ;;
        "workflow")
            TEST_ARGS="$TEST_ARGS workflow"
            ;;
        "fast")
            TEST_TYPE="--lib"
            ;;
        "slow")
            TEST_ARGS="$TEST_ARGS -- --ignored --test-threads=1"
            ;;
        "external")
            TEST_ARGS="$TEST_ARGS -- --ignored"
            ;;
        *)
            print_error "Unknown tag: $tag"
            exit 1
            ;;
    esac
done

# Construct final command
FINAL_CMD="$CARGO_CMD $PACKAGE $TEST_TYPE $TEST_ARGS $@"

print_info "Executing: $FINAL_CMD"
echo ""

# Run the command
eval $FINAL_CMD

# Check test result
if [ $? -eq 0 ]; then
    print_success "Tests passed!"
else
    print_error "Tests failed!"
    exit 1
fi