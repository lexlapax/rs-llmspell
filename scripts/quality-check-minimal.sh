#!/bin/bash

# Minimal Quality Check - Just formatting and clippy
# For when you need a really quick check

set -e

echo "⚡ Running Minimal Quality Checks..."
echo "===================================="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Track overall success
OVERALL_SUCCESS=0

# 1. Check formatting
echo ""
echo "1. Checking code formatting..."
if cargo fmt --all -- --check > /dev/null 2>&1; then
    echo -e "${GREEN}✅ Code formatting check passed${NC}"
else
    echo -e "${RED}❌ Code formatting check failed${NC}"
    echo "   Run: cargo fmt --all"
    OVERALL_SUCCESS=1
fi

# 2. Run clippy
echo ""
echo "2. Running clippy lints..."
if cargo clippy --workspace --all-features -- -D warnings > /dev/null 2>&1; then
    echo -e "${GREEN}✅ Clippy lints passed${NC}"
else
    echo -e "${RED}❌ Clippy lints failed${NC}"
    echo "   Run: cargo clippy --workspace --all-features"
    OVERALL_SUCCESS=1
fi

# 3. Check if it compiles
echo ""
echo "3. Checking if code compiles..."
if cargo check --workspace > /dev/null 2>&1; then
    echo -e "${GREEN}✅ Code compiles successfully${NC}"
else
    echo -e "${RED}❌ Compilation failed${NC}"
    OVERALL_SUCCESS=1
fi

# Summary
echo ""
echo "===================================="
if [ $OVERALL_SUCCESS -eq 0 ]; then
    echo -e "${GREEN}✅ All minimal checks passed!${NC}"
    echo ""
    echo -e "${BLUE}Note: This is a minimal check that runs quickly.${NC}"
    echo -e "${BLUE}For more thorough validation, run:${NC}"
    echo "  - Fast checks: ./scripts/quality-check-fast.sh"
    echo "  - Full checks: ./scripts/quality-check.sh"
    exit 0
else
    echo -e "${RED}❌ Some checks failed!${NC}"
    echo "Please fix the issues above."
    exit 1
fi