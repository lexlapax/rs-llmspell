#!/bin/bash

# Fast Quality Gates Check Script
# Runs essential quality checks without slow tests

set -e

echo "🚀 Running Fast Quality Checks..."
echo "================================="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print status
print_status() {
    if [ $1 -eq 0 ]; then
        echo -e "${GREEN}✅ $2${NC}"
    else
        echo -e "${RED}❌ $2${NC}"
        return 1
    fi
}

# Function to print info
print_info() {
    echo -e "${BLUE}ℹ️  $1${NC}"
}

# Track overall success
OVERALL_SUCCESS=0

print_info "Starting fast quality checks for rs-llmspell..."

# 1. Check formatting
echo ""
echo "1. Checking code formatting..."
if cargo fmt --all -- --check > /dev/null 2>&1; then
    print_status 0 "Code formatting check passed"
else
    print_status 1 "Code formatting check failed"
    echo "   Run: cargo fmt --all"
    OVERALL_SUCCESS=1
fi

# 2. Run clippy
echo ""
echo "2. Running clippy lints..."
if cargo clippy --workspace --all-features -- -D warnings > /dev/null 2>&1; then
    print_status 0 "Clippy lints passed"
else
    print_status 1 "Clippy lints failed"
    echo "   Run: cargo clippy --workspace --all-features"
    OVERALL_SUCCESS=1
fi

# 3. Check build
echo ""
echo "3. Building workspace..."
if cargo build --workspace > /dev/null 2>&1; then
    print_status 0 "Workspace build successful"
else
    print_status 1 "Workspace build failed"
    OVERALL_SUCCESS=1
fi

# 4. Run only unit tests (fast)
echo ""
echo "4. Running unit tests..."
print_info "Running core unit tests..."
if cargo test --lib -p llmspell-core > /dev/null 2>&1; then
    print_status 0 "Core unit tests passed"
else
    print_status 1 "Core unit tests failed"
    OVERALL_SUCCESS=1
fi

# 4a. Run tool unit tests separately
echo ""
echo "4a. Running tool unit tests..."
if cargo test --lib -p llmspell-tools > /dev/null 2>&1; then
    print_status 0 "Tool unit tests passed"
else
    print_status 1 "Tool unit tests failed"
    OVERALL_SUCCESS=1
fi

# 4b. Run other package unit tests
echo ""
echo "4b. Running other unit tests..."
if cargo test --lib -p llmspell-bridge -p llmspell-utils -p llmspell-storage > /dev/null 2>&1; then
    print_status 0 "Other unit tests passed"
else
    print_status 1 "Other unit tests failed"
    OVERALL_SUCCESS=1
fi

# 5. Check documentation
echo ""
echo "5. Building documentation..."
if RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --all-features --document-private-items > /dev/null 2>&1; then
    print_status 0 "Documentation build successful"
else
    print_status 1 "Documentation build failed"
    OVERALL_SUCCESS=1
fi

# Summary
echo ""
echo "================================="
if [ $OVERALL_SUCCESS -eq 0 ]; then
    echo -e "${GREEN}🎉 All fast quality checks passed!${NC}"
    echo ""
    echo "Note: This is a fast check. For full validation run:"
    echo "  - Full test suite: cargo test --workspace"
    echo "  - Coverage: cargo tarpaulin --workspace"
    exit 0
else
    echo -e "${RED}💥 Some quality checks failed!${NC}"
    echo "Please fix the issues before pushing."
    exit 1
fi