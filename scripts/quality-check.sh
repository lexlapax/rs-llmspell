#!/bin/bash

# Quality Gates Check Script
# Runs the same quality checks as CI pipeline locally

set -e

echo "🔍 Running Quality Gates Checks..."
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

# Function to print warning
print_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

# Track overall success
OVERALL_SUCCESS=0

print_info "Starting quality checks for rs-llmspell..."

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
    echo "   Run: cargo clippy --workspace --all-features -- -D warnings"
    OVERALL_SUCCESS=1
fi

# 3. Build workspace
echo ""
echo "3. Building workspace..."
if cargo build --workspace --all-features > /dev/null 2>&1; then
    print_status 0 "Workspace build successful"
else
    print_status 1 "Workspace build failed"
    OVERALL_SUCCESS=1
fi

# 4. Run tests
echo ""
echo "4. Running test suite..."

# Check for environment variable to skip slow tests
if [ "$SKIP_SLOW_TESTS" = "true" ]; then
    print_info "Running tests (skipping slow/external)..."
    print_warning "SKIP_SLOW_TESTS is set - ignoring slow and external tests"
    
    # Run all tests except ignored ones
    if timeout 300s cargo test --workspace > /dev/null 2>&1; then
        print_status 0 "Test suite passed (slow tests skipped)"
    else
        if [ $? -eq 124 ]; then
            print_status 1 "Test suite timed out (>5 minutes)"
            print_warning "Consider using ./scripts/test-by-tag.sh to run specific test categories"
        else
            print_status 1 "Test suite failed"
        fi
        OVERALL_SUCCESS=1
    fi
else
    print_info "Running all tests including slow/external..."
    print_info "Set SKIP_SLOW_TESTS=true to skip slow tests"
    
    # Run all tests including ignored ones
    if timeout 300s cargo test --workspace --include-ignored > /dev/null 2>&1; then
        print_status 0 "Full test suite passed"
    else
        if [ $? -eq 124 ]; then
            print_status 1 "Test suite timed out (>5 minutes)"
            print_warning "Consider using SKIP_SLOW_TESTS=true or ./scripts/test-by-tag.sh"
        else
            print_status 1 "Test suite failed"
            print_info "Run tests by category with ./scripts/test-by-tag.sh <tag>"
        fi
        OVERALL_SUCCESS=1
    fi
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

# 6. Test coverage (optional - requires tarpaulin)
echo ""
echo "6. Checking test coverage (optional)..."
if command -v cargo-tarpaulin >/dev/null 2>&1; then
    echo "   Running coverage analysis..."
    COVERAGE_OUTPUT=$(timeout 180s cargo tarpaulin --workspace --out Json --timeout 120 2>/dev/null || echo "failed")
    
    if [ "$COVERAGE_OUTPUT" != "failed" ]; then
        # Try to extract coverage percentage (simplified)
        if command -v jq >/dev/null 2>&1; then
            COVERAGE=$(echo "$COVERAGE_OUTPUT" | jq -r '.files | to_entries | map(.value.summary.lines.percent) | add / length' 2>/dev/null || echo "unknown")
            if [ "$COVERAGE" != "unknown" ] && [ "$COVERAGE" != "null" ]; then
                COVERAGE_INT=$(echo "$COVERAGE" | cut -d. -f1)
                if [ "$COVERAGE_INT" -ge 90 ]; then
                    print_status 0 "Test coverage: ${COVERAGE}% (≥90% threshold)"
                else
                    print_status 1 "Test coverage: ${COVERAGE}% (<90% threshold)"
                    OVERALL_SUCCESS=1
                fi
            else
                print_warning "Could not parse coverage percentage"
            fi
        else
            print_warning "jq not available for coverage parsing"
        fi
    else
        print_warning "Coverage analysis failed"
    fi
else
    print_warning "cargo-tarpaulin not installed (install with: cargo install cargo-tarpaulin)"
fi

# 7. Security audit (optional - requires cargo-audit)
echo ""
echo "7. Running security audit (optional)..."
if command -v cargo-audit >/dev/null 2>&1; then
    if cargo audit > /dev/null 2>&1; then
        print_status 0 "Security audit passed"
    else
        print_status 1 "Security audit found vulnerabilities"
        echo "   Run: cargo audit for details"
        OVERALL_SUCCESS=1
    fi
else
    print_warning "cargo-audit not installed (install with: cargo install cargo-audit)"
fi

# Summary
echo ""
echo "================================="
if [ $OVERALL_SUCCESS -eq 0 ]; then
    echo -e "${GREEN}🎉 All quality checks passed!${NC}"
    echo "Ready to push to repository."
    exit 0
else
    echo -e "${RED}💥 Some quality checks failed!${NC}"
    echo "Please fix the issues before pushing."
    echo ""
    echo "Quick fixes:"
    echo "  - Format code: cargo fmt --all"
    echo "  - Fix lints: cargo clippy --workspace --all-features"
    echo "  - Add tests for coverage"
    echo "  - Fix documentation warnings"
    exit 1
fi