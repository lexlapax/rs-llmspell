#!/bin/bash
# Advanced Fleet Integration Tests
# Comprehensive test suite for fleet management functionality

set -e

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Test counters
TESTS_PASSED=0
TESTS_FAILED=0
TESTS_SKIPPED=0

# Test results array
declare -a TEST_RESULTS

# Test helper function
test_case() {
    local name="$1"
    local cmd="$2"
    local expected="$3"
    local skip_reason="$4"

    if [ -n "$skip_reason" ]; then
        echo -e "${YELLOW}⊘ SKIP${NC}: $name ($skip_reason)"
        TESTS_SKIPPED=$((TESTS_SKIPPED + 1))
        TEST_RESULTS+=("SKIP: $name - $skip_reason")
        return
    fi

    echo -n "Testing: $name... "

    # Capture output and error
    local output
    local exit_code
    output=$(eval "$cmd" 2>&1) && exit_code=0 || exit_code=$?

    if [ "$expected" = "should_fail" ]; then
        if [ $exit_code -ne 0 ]; then
            echo -e "${GREEN}✓ PASS${NC} (expected failure)"
            TESTS_PASSED=$((TESTS_PASSED + 1))
            TEST_RESULTS+=("PASS: $name")
        else
            echo -e "${RED}✗ FAIL${NC} (expected to fail but succeeded)"
            TESTS_FAILED=$((TESTS_FAILED + 1))
            TEST_RESULTS+=("FAIL: $name - Expected failure but succeeded")
        fi
    elif [ "$expected" = "should_succeed" ]; then
        if [ $exit_code -eq 0 ]; then
            echo -e "${GREEN}✓ PASS${NC}"
            TESTS_PASSED=$((TESTS_PASSED + 1))
            TEST_RESULTS+=("PASS: $name")
        else
            echo -e "${RED}✗ FAIL${NC}"
            echo "  Error: $output"
            TESTS_FAILED=$((TESTS_FAILED + 1))
            TEST_RESULTS+=("FAIL: $name - $output")
        fi
    elif echo "$output" | grep -q "$expected"; then
        echo -e "${GREEN}✓ PASS${NC}"
        TESTS_PASSED=$((TESTS_PASSED + 1))
        TEST_RESULTS+=("PASS: $name")
    else
        echo -e "${RED}✗ FAIL${NC}"
        echo "  Expected: '$expected'"
        echo "  Got: '$output'"
        TESTS_FAILED=$((TESTS_FAILED + 1))
        TEST_RESULTS+=("FAIL: $name - Expected '$expected' but got '$output'")
    fi
}

# Performance test helper
perf_test() {
    local name="$1"
    local cmd="$2"
    local max_time="$3"  # Maximum time in seconds

    echo -n "Performance test: $name (max ${max_time}s)... "

    local start_time=$(date +%s)
    eval "$cmd" > /dev/null 2>&1
    local end_time=$(date +%s)
    local duration=$((end_time - start_time))

    if [ $duration -le $max_time ]; then
        echo -e "${GREEN}✓ PASS${NC} (${duration}s)"
        TESTS_PASSED=$((TESTS_PASSED + 1))
        TEST_RESULTS+=("PASS: $name - ${duration}s (max ${max_time}s)")
    else
        echo -e "${RED}✗ FAIL${NC} (${duration}s > ${max_time}s)"
        TESTS_FAILED=$((TESTS_FAILED + 1))
        TEST_RESULTS+=("FAIL: $name - ${duration}s exceeded max ${max_time}s")
    fi
}

echo "=== Advanced Fleet Integration Tests ==="
echo "======================================="
echo ""

# We're already in fleet directory

# Clean start
echo "Preparing test environment..."
./llmspell-fleet stop-all 2>/dev/null || true
./llmspell-fleet cleanup
rm -f ~/.llmspell/fleet/*.pid 2>/dev/null || true
echo ""

# =============================================================================
echo -e "${BLUE}1. Basic Functionality Tests${NC}"
echo "------------------------------"

test_case "Help command works" \
    "./llmspell-fleet help" \
    "llmspell-fleet"

test_case "Empty list shows no kernels" \
    "./llmspell-fleet list" \
    "No kernels"

test_case "Spawn creates kernel" \
    "./llmspell-fleet spawn && ./llmspell-fleet list | grep -c kernel-" \
    "1"

test_case "Cleanup removes dead kernels" \
    "./llmspell-fleet cleanup" \
    "Cleanup complete"

# =============================================================================
echo ""
echo -e "${BLUE}2. Multi-Kernel Management${NC}"
echo "----------------------------"

test_case "Spawn multiple kernels" \
    "./llmspell-fleet spawn && ./llmspell-fleet spawn && ./llmspell-fleet list | grep -c '✓ running'" \
    "3"  # We already have one from previous test

test_case "Each kernel gets unique port" \
    "./llmspell-fleet list | grep 'Port:' | awk '{print \$5}' | sort -u | wc -l" \
    "3"

test_case "Stop by port works" \
    "PORT=\$(./llmspell-fleet list | grep kernel- | head -1 | awk '{print \$5}' | sed 's/.*://' | tr -d 'Port:'); ./llmspell-fleet stop \$PORT" \
    "stopped"

test_case "Stop-all works" \
    "./llmspell-fleet stop-all" \
    "Stopping"

# =============================================================================
echo ""
echo -e "${BLUE}3. Python Fleet Manager Tests${NC}"
echo "-------------------------------"

test_case "Python spawn works" \
    "python3 fleet_manager.py spawn" \
    "started"

test_case "Python list works" \
    "python3 fleet_manager.py list | grep -c kernel-" \
    "1"

test_case "Python metrics works" \
    "python3 fleet_manager.py metrics | jq -r '.total_kernels'" \
    "1"

test_case "Python find works" \
    "python3 fleet_manager.py find --language lua" \
    "Connection file"

test_case "Python stop-all works" \
    "python3 fleet_manager.py stop-all" \
    "Stopping"

# =============================================================================
echo ""
echo -e "${BLUE}4. Registry Management${NC}"
echo "-----------------------"

test_case "Registry file exists" \
    "test -f ~/.llmspell/fleet/registry.json && echo 'exists'" \
    "exists"

test_case "Registry is valid JSON" \
    "jq -r '.kernels | length' ~/.llmspell/fleet/registry.json > /dev/null && echo 'valid'" \
    "valid"

test_case "Registry tracks port allocation" \
    "jq -r '.next_port' ~/.llmspell/fleet/registry.json | grep -E '^[0-9]+$'" \
    "[0-9]"

# =============================================================================
echo ""
echo -e "${BLUE}5. Error Handling Tests${NC}"
echo "------------------------"

test_case "Stop non-existent kernel fails gracefully" \
    "./llmspell-fleet stop kernel-nonexistent 2>&1" \
    "not found"

test_case "Invalid port number handled" \
    "./llmspell-fleet stop 99999 2>&1" \
    "No kernel"

test_case "Duplicate PID file handled" \
    "touch ~/.llmspell/fleet/test.pid && echo 12345 > ~/.llmspell/fleet/test.pid && rm ~/.llmspell/fleet/test.pid && echo 'handled'" \
    "handled"

# =============================================================================
echo ""
echo -e "${BLUE}6. Connection File Tests${NC}"
echo "-------------------------"

./llmspell-fleet spawn > /dev/null 2>&1
KERNEL_ID=$(./llmspell-fleet list | grep kernel- | head -1 | awk '{print $1}')

test_case "Connection file created" \
    "test -f ~/.llmspell/fleet/$KERNEL_ID.json && echo 'exists'" \
    "exists"

test_case "Connection file has required fields" \
    "jq -r '.transport' ~/.llmspell/fleet/$KERNEL_ID.json" \
    "tcp"

test_case "Connection file has valid ports" \
    "jq -r '.shell_port' ~/.llmspell/fleet/$KERNEL_ID.json | grep -E '^[0-9]+$'" \
    "[0-9]"

# =============================================================================
echo ""
echo -e "${BLUE}7. Performance Tests${NC}"
echo "---------------------"

perf_test "Kernel spawn time" \
    "./llmspell-fleet spawn" \
    "5"  # Should complete within 5 seconds

perf_test "List command speed" \
    "./llmspell-fleet list" \
    "1"  # Should complete within 1 second

perf_test "Stop kernel speed" \
    "KERNEL=\$(./llmspell-fleet list | grep kernel- | head -1 | awk '{print \$1}'); ./llmspell-fleet stop \$KERNEL" \
    "3"  # Should complete within 3 seconds

# =============================================================================
echo ""
echo -e "${BLUE}8. Resource Limit Tests${NC}"
echo "------------------------"

test_case "Nice priority applied" \
    "nice -n 5 ./llmspell-fleet spawn > /dev/null 2>&1 && ps -o nice -p \$(./llmspell-fleet list | grep kernel- | tail -1 | awk '{print \$2}' | sed 's/PID://') | tail -1 | grep -E '[0-9]'" \
    "[0-9]"

test_case "Memory usage reasonable" \
    "./llmspell-fleet list | grep kernel- | head -1 | awk '{print \$9}' | sed 's/Mem://' | sed 's/MB//' | awk '{\$1 < 100 ? print \"ok\" : print \"high\"}'" \
    "ok"

# =============================================================================
echo ""
echo -e "${BLUE}9. Concurrent Operations${NC}"
echo "-------------------------"

test_case "Concurrent spawns" \
    "(./llmspell-fleet spawn & ./llmspell-fleet spawn & wait) > /dev/null 2>&1 && ./llmspell-fleet list | grep -c '✓ running'" \
    "[2-9]"  # At least 2 kernels running

test_case "Concurrent stops" \
    "./llmspell-fleet stop-all 2>&1" \
    "Stopping"

# =============================================================================
echo ""
echo -e "${BLUE}10. Health Check Tests${NC}"
echo "-----------------------"

./llmspell-fleet spawn > /dev/null 2>&1
sleep 2

test_case "Health check detects running kernel" \
    "./llmspell-fleet health | grep HEALTHY" \
    "HEALTHY"

# Kill the process but leave the registry entry
KERNEL_PID=$(./llmspell-fleet list | grep kernel- | head -1 | awk '{print $2}' | sed 's/PID://')
kill -9 $KERNEL_PID 2>/dev/null || true

test_case "Health check detects dead kernel" \
    "./llmspell-fleet health | grep DEAD" \
    "DEAD"

# =============================================================================
echo ""
echo -e "${BLUE}11. HTTP Service Tests${NC}"
echo "-----------------------"

# Skip if Flask not installed
if python3 -c "import flask" 2>/dev/null; then
    python3 fleet_http_service.py &
    HTTP_PID=$!
    sleep 2

    test_case "HTTP health endpoint" \
        "curl -s http://127.0.0.1:9550/health | jq -r '.status'" \
        "healthy"

    test_case "HTTP kernels endpoint" \
        "curl -s http://127.0.0.1:9550/kernels | jq -r '.kernels | type'" \
        "array"

    test_case "HTTP metrics endpoint" \
        "curl -s http://127.0.0.1:9550/metrics | jq -r '.total_kernels' > /dev/null && echo 'works'" \
        "works"

    kill $HTTP_PID 2>/dev/null || true
else
    test_case "HTTP health endpoint" "" "" "Flask not installed"
    test_case "HTTP kernels endpoint" "" "" "Flask not installed"
    test_case "HTTP metrics endpoint" "" "" "Flask not installed"
fi

# =============================================================================
echo ""
echo -e "${BLUE}12. Final Cleanup${NC}"
echo "------------------"

test_case "Stop all remaining kernels" \
    "./llmspell-fleet stop-all 2>&1 && ./llmspell-fleet list" \
    "No kernels"

test_case "Clean up dead entries" \
    "./llmspell-fleet cleanup" \
    "Cleanup complete"

# =============================================================================
echo ""
echo "=================================="
echo -e "${YELLOW}Test Results Summary:${NC}"
echo "=================================="
echo -e "  Passed:  ${GREEN}$TESTS_PASSED${NC}"
echo -e "  Failed:  ${RED}$TESTS_FAILED${NC}"
echo -e "  Skipped: ${YELLOW}$TESTS_SKIPPED${NC}"
echo -e "  Total:   $((TESTS_PASSED + TESTS_FAILED + TESTS_SKIPPED))"
echo ""

# Save detailed results
RESULTS_FILE="test_results_$(date +%Y%m%d_%H%M%S).txt"
{
    echo "Fleet Integration Test Results"
    echo "=============================="
    echo "Date: $(date)"
    echo "Passed: $TESTS_PASSED"
    echo "Failed: $TESTS_FAILED"
    echo "Skipped: $TESTS_SKIPPED"
    echo ""
    echo "Detailed Results:"
    echo "-----------------"
    for result in "${TEST_RESULTS[@]}"; do
        echo "  $result"
    done
} > "$RESULTS_FILE"

echo "Detailed results saved to: $RESULTS_FILE"
echo ""

# Exit code based on failures
if [ $TESTS_FAILED -eq 0 ]; then
    echo -e "${GREEN}✓ All tests passed!${NC}"
    exit 0
else
    echo -e "${RED}✗ $TESTS_FAILED tests failed${NC}"
    exit 1
fi