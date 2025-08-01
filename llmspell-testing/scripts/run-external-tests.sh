#!/bin/bash

# Run external dependency tests
# These tests require external services (HTTP, APIs, etc.) and are normally ignored

set -e

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${BLUE}🌍 Running External Dependency Tests${NC}"
echo "====================================="
echo "Categories: Tests requiring external services"
echo "Note: These tests require internet connectivity and may be flaky"
echo ""

echo -e "${YELLOW}⚠️  External tests include:${NC}"
echo "   - HTTP requests to real endpoints"
echo "   - Web scraping tests"  
echo "   - API integration tests"
echo "   - Network-dependent functionality"
echo ""

# Run external tests using feature flag and --ignored to include ignored tests
if cargo test -p llmspell-testing --features "external-tests" -- --ignored; then
    echo ""
    echo -e "${GREEN}✅ External test suite completed successfully!${NC}"
    exit 0
else
    echo ""
    echo -e "${RED}❌ External test suite failed!${NC}"
    echo "   This might be due to network issues or external service unavailability"
    exit 1
fi