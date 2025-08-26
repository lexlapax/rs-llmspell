#!/bin/bash

# Run complete test suite including external dependencies and benchmarks
# This is the most comprehensive test run

set -e

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${BLUE}🎯 Running Complete Test Suite${NC}"
echo "====================================="
echo "Categories: ALL tests including external dependencies and benchmarks"
echo "Note: This may take significant time and requires internet connectivity"
echo ""

echo -e "${YELLOW}⚠️  Complete test suite includes:${NC}"
echo "   - All unit and integration tests"
echo "   - All component-specific tests"
echo "   - Security and performance tests"
echo "   - External dependency tests (may be flaky)"
echo "   - Benchmark tests (performance measurements)"
echo ""

# Run all tests using feature flag and --include-ignored to run everything
if cargo test -p llmspell-testing --features "all-tests" -- --include-ignored; then
    echo ""
    echo -e "${GREEN}✅ Complete test suite finished successfully!${NC}"
    exit 0
else
    echo ""
    echo -e "${RED}❌ Complete test suite failed!${NC}"
    echo "   Check individual test categories to identify failures"
    exit 1
fi