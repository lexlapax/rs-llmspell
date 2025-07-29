#!/bin/bash

# Run all tests in llmspell-testing with proper organization
# This script is called by CI/CD and development workflows

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo "🧪 Running llmspell-testing Test Suite"
echo "====================================="

# Function to run test category
run_category() {
    local category=$1
    local feature=$2
    
    echo ""
    echo "📋 Running $category tests..."
    if cargo test -p llmspell-testing --features $feature --quiet; then
        echo -e "${GREEN}✅ $category tests passed${NC}"
        return 0
    else
        echo -e "${RED}❌ $category tests failed${NC}"
        return 1
    fi
}

# Track overall success
OVERALL_SUCCESS=0

# Run each test category
run_category "Unit" "unit-tests" || OVERALL_SUCCESS=1
run_category "Integration" "integration-tests" || OVERALL_SUCCESS=1
run_category "Agent" "agent-tests" || OVERALL_SUCCESS=1
run_category "Scenario" "scenario-tests" || OVERALL_SUCCESS=1
run_category "Lua" "lua-tests" || OVERALL_SUCCESS=1

# Summary
echo ""
echo "====================================="
if [ $OVERALL_SUCCESS -eq 0 ]; then
    echo -e "${GREEN}🎉 All test categories passed!${NC}"
    exit 0
else
    echo -e "${RED}💥 Some test categories failed!${NC}"
    exit 1
fi