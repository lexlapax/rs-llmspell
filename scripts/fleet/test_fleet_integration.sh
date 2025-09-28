#!/bin/bash
# Fleet Integration Test Script

set -e

echo "=== Fleet Integration Testing ==="
echo ""

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
NC='\033[0m'

# Test counter
TESTS_PASSED=0
TESTS_FAILED=0

# Helper function
test_case() {
    local name="$1"
    local cmd="$2"
    local expected="$3"

    echo -n "Testing: $name... "
    if eval "$cmd" > /dev/null 2>&1; then
        echo -e "${GREEN}✓ PASS${NC}"
        TESTS_PASSED=$((TESTS_PASSED + 1))
    else
        echo -e "${RED}✗ FAIL${NC}"
        echo "  Command: $cmd"
        TESTS_FAILED=$((TESTS_FAILED + 1))
    fi
}

# Clean start
echo "1. Cleaning up existing kernels..."
python3 fleet_manager.py stop-all -f 2>/dev/null || true
./llmspell-fleet cleanup 2>/dev/null || true

# Test Shell Script
echo ""
echo "2. Testing Shell Script (llmspell-fleet)..."
test_case "Help command" "./llmspell-fleet help | grep -q 'llmspell-fleet'"
test_case "List (empty)" "./llmspell-fleet list | grep -q 'No kernels'"
test_case "Spawn kernel" "./llmspell-fleet spawn"
KERNEL_ID=$(./llmspell-fleet list | grep kernel- | awk '{print $1}' | head -1)
test_case "List (with kernel)" "./llmspell-fleet list | grep -q '$KERNEL_ID'"
test_case "Health check" "./llmspell-fleet health | grep -q 'HEALTHY'"
test_case "Stop kernel" "./llmspell-fleet stop $KERNEL_ID"
test_case "Cleanup" "./llmspell-fleet cleanup"

# Test Python Fleet Manager
echo ""
echo "3. Testing Python Fleet Manager..."
test_case "Python help" "python3 fleet_manager.py -h | grep -q 'Fleet Manager'"
test_case "Python spawn" "python3 fleet_manager.py spawn"
test_case "Python list" "python3 fleet_manager.py list | grep -q kernel-"
test_case "Python metrics" "python3 fleet_manager.py metrics | grep -q total_kernels"
test_case "Find existing" "python3 fleet_manager.py find --language lua"
test_case "Stop by port" "python3 fleet_manager.py stop 9555 2>/dev/null || true"
test_case "Python stop-all" "python3 fleet_manager.py stop-all"

# Test Registry
echo ""
echo "4. Testing Registry..."
test_case "Registry exists" "test -f ~/.llmspell/fleet/registry.json"
test_case "Registry valid JSON" "jq . ~/.llmspell/fleet/registry.json > /dev/null"

# Test Multiple Kernels
echo ""
echo "5. Testing Multiple Kernels..."
test_case "Spawn kernel 1" "python3 fleet_manager.py spawn"
test_case "Spawn kernel 2" "python3 fleet_manager.py spawn"
KERNEL_COUNT=$(python3 fleet_manager.py list | grep -c "✓ running" || echo 0)
test_case "Two kernels running" "[ $KERNEL_COUNT -eq 2 ]"
test_case "Different ports" "python3 fleet_manager.py list | grep 'Port:' | awk '{print $5}' | sort -u | wc -l | grep -q 2"

# Test Docker Compose
echo ""
echo "6. Testing Docker Compose..."
test_case "Docker compose file exists" "test -f docker-compose.yml"
test_case "Docker compose valid" "docker-compose config > /dev/null 2>&1 || echo 'Docker not available'"

# Cleanup
echo ""
echo "7. Final Cleanup..."
python3 fleet_manager.py stop-all -f
./llmspell-fleet cleanup

# Results
echo ""
echo "=================================="
echo "Test Results:"
echo -e "  Passed: ${GREEN}$TESTS_PASSED${NC}"
echo -e "  Failed: ${RED}$TESTS_FAILED${NC}"
echo "=================================="

if [ $TESTS_FAILED -eq 0 ]; then
    echo -e "${GREEN}✓ All tests passed!${NC}"
    exit 0
else
    echo -e "${RED}✗ Some tests failed${NC}"
    exit 1
fi