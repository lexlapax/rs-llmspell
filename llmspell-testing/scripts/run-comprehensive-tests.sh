#!/bin/bash

# Run comprehensive test suite (everything except external dependencies)
# Includes unit, integration, component, security, and performance tests

set -e

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${BLUE}🔬 Running Comprehensive Test Suite${NC}"
echo "====================================="
echo "Categories: Unit + Integration + Tool + Agent + Workflow + Bridge + Security + Performance"
echo "Excludes: External dependencies, benchmarks"
echo ""

# Run comprehensive tests using the feature flag
if cargo test -p llmspell-testing --features "comprehensive-tests"; then
    echo ""
    echo -e "${GREEN}✅ Comprehensive test suite completed successfully!${NC}"
    exit 0
else
    echo ""
    echo -e "${RED}❌ Comprehensive test suite failed!${NC}"
    exit 1
fi