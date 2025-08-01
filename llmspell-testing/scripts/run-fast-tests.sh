#!/bin/bash

# Run fast test suite (unit + integration tests)
# This excludes external dependencies and benchmarks

set -e

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}🚀 Running Fast Test Suite${NC}"
echo "====================================="
echo "Categories: Unit Tests + Integration Tests"
echo "Excludes: External dependencies, benchmarks, slow tests"
echo ""

# Run fast tests using the feature flag
if cargo test -p llmspell-testing --features "fast-tests"; then
    echo ""
    echo -e "${GREEN}✅ Fast test suite completed successfully!${NC}"
    exit 0
else
    echo ""
    echo -e "${RED}❌ Fast test suite failed!${NC}"
    exit 1
fi