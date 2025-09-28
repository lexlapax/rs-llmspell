#!/bin/bash

# Run tests by tag/category
# Usage: ./scripts/test-by-tag.sh <tag> [additional cargo test args]
#
# Available tags:
#   unit        - Run unit tests from llmspell-testing
#   integration - Run integration tests from llmspell-testing
#   agent       - Run agent-specific tests from llmspell-testing
#   scenarios   - Run end-to-end scenario tests from llmspell-testing
#   lua         - Run Lua scripting tests from llmspell-testing
#   tool        - Run tests in llmspell-tools package
#   bridge      - Run tests in llmspell-bridge package
#   workflow    - Run tests in llmspell-workflows package
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

# Check if test runner is available
if command -v llmspell-test >/dev/null 2>&1; then
    TEST_RUNNER="llmspell-test"
else
    TEST_RUNNER="cargo run -p llmspell-testing --features test-runner --bin llmspell-test --"
fi

case $TAG in
    "unit"|"integration"|"agent"|"scenario"|"scenarios"|"lua")
        # Normalize scenarios -> scenario
        if [ "$TAG" = "scenarios" ]; then
            TAG="scenario"
        fi
        print_info "Delegating to llmspell-test runner..."
        $TEST_RUNNER run $TAG $@
        ;;
    "tool")
        print_info "Running tool tests..."
        cargo test -p llmspell-tools $@
        ;;
    "workflow")
        print_info "Running workflow tests..."
        cargo test -p llmspell-workflows $@
        ;;
    "fast")
        print_info "Running fast tests (unit tests only)..."
        $TEST_RUNNER run unit $@
        ;;
    "slow")
        print_info "Running slow tests (ignored tests with single thread)..."
        cargo test -p llmspell-testing --features all-tests -- --ignored --test-threads=1 $@
        ;;
    "external")
        print_info "Running external tests (tests requiring external services)..."
        cargo test -p llmspell-testing --features all-tests -- --ignored external $@
        ;;
    "all")
        print_info "Running all tests..."
        $TEST_RUNNER run all $@
        ;;
    "bridge")
        print_info "Running bridge tests..."
        cargo test -p llmspell-bridge $@
        ;;
    "llm")
        print_info "Running LLM provider tests..."
        cargo test -p llmspell-testing --features integration-tests llm $@ -- --ignored
        ;;
    "database")
        print_info "Running database tests..."
        cargo test -p llmspell-testing --features integration-tests database $@ -- --ignored
        ;;
    *)
        print_error "Unknown tag: $TAG"
        echo ""
        echo "Available tags:"
        echo "  unit        - Run unit tests from llmspell-testing"
        echo "  integration - Run integration tests from llmspell-testing"
        echo "  agent       - Run agent-specific tests from llmspell-testing"
        echo "  scenarios   - Run end-to-end scenario tests from llmspell-testing"
        echo "  lua         - Run Lua scripting tests from llmspell-testing"
        echo "  tool        - Run tests in llmspell-tools package"
        echo "  bridge      - Run tests in llmspell-bridge package"
        echo "  workflow    - Run tests in llmspell-workflows package"
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