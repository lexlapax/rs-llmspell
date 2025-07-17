#!/bin/bash

# Run tests by tag/category
# Usage: ./scripts/test-by-tag.sh <tag> [additional cargo test args]
#
# Available tags:
#   unit        - Run only unit tests (--lib)
#   integration - Run only integration tests (--test)
#   tool        - Run tests in llmspell-tools package
#   agent       - Run tests in llmspell-agents package
#   workflow    - Run tests containing 'workflow' in name
#   fast        - Run only fast unit tests
#   slow        - Run slow/ignored tests
#   external    - Run tests requiring external services
#   all         - Run all tests including ignored

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

# Check if tag is provided
if [ $# -eq 0 ]; then
    print_error "No tag provided!"
    echo "Usage: $0 <tag> [additional cargo test args]"
    echo ""
    echo "Available tags:"
    echo "  unit        - Run only unit tests (--lib)"
    echo "  integration - Run only integration tests (--test)"
    echo "  tool        - Run tests in llmspell-tools package"
    echo "  agent       - Run tests in llmspell-agents package"
    echo "  workflow    - Run tests containing 'workflow' in name"
    echo "  fast        - Run only fast unit tests"
    echo "  slow        - Run slow/ignored tests"
    echo "  external    - Run tests requiring external services"
    echo "  all         - Run all tests including ignored"
    exit 1
fi

TAG=$1
shift

echo "🏷️  Running tests tagged as: $TAG"
echo "================================="

case $TAG in
    "unit")
        print_info "Running unit tests (library tests only)..."
        cargo test --lib --all $@
        ;;
    "integration")
        print_info "Running integration tests..."
        cargo test --test '*' $@
        ;;
    "tool")
        print_info "Running tool tests..."
        cargo test -p llmspell-tools $@
        ;;
    "agent")
        print_info "Running agent tests..."
        if cargo metadata --no-deps | grep -q '"name": "llmspell-agents"'; then
            cargo test -p llmspell-agents $@
        else
            print_error "llmspell-agents package not found (not yet implemented)"
            exit 1
        fi
        ;;
    "workflow")
        print_info "Running workflow tests..."
        cargo test workflow $@
        ;;
    "fast")
        print_info "Running fast tests (unit tests only)..."
        cargo test --lib --all $@
        ;;
    "slow")
        print_info "Running slow tests (ignored tests with single thread)..."
        cargo test -- --ignored --test-threads=1 $@
        ;;
    "external")
        print_info "Running external tests (tests requiring external services)..."
        cargo test -- --ignored external $@
        ;;
    "all")
        print_info "Running all tests including ignored..."
        cargo test --all --include-ignored $@
        ;;
    "bridge")
        print_info "Running bridge tests..."
        cargo test -p llmspell-bridge $@
        ;;
    "llm")
        print_info "Running LLM provider tests..."
        cargo test llm $@ -- --ignored
        ;;
    "database")
        print_info "Running database tests..."
        cargo test database $@ -- --ignored
        ;;
    *)
        print_error "Unknown tag: $TAG"
        echo ""
        echo "Available tags:"
        echo "  unit        - Run only unit tests (--lib)"
        echo "  integration - Run only integration tests (--test)"
        echo "  tool        - Run tests in llmspell-tools package"
        echo "  agent       - Run tests in llmspell-agents package"
        echo "  bridge      - Run tests in llmspell-bridge package"
        echo "  workflow    - Run tests containing 'workflow' in name"
        echo "  llm         - Run LLM provider tests"
        echo "  database    - Run database tests"
        echo "  fast        - Run only fast unit tests"
        echo "  slow        - Run slow/ignored tests"
        echo "  external    - Run tests requiring external services"
        echo "  all         - Run all tests including ignored"
        exit 1
        ;;
esac

# Check test result
if [ $? -eq 0 ]; then
    print_success "Tests passed!"
else
    print_error "Tests failed!"
    exit 1
fi