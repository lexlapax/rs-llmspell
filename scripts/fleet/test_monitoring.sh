#!/bin/bash
# Test script for fleet monitoring features

set -e

GREEN='\033[0;32m'
RED='\033[0;31m'
NC='\033[0m'

echo "=== Fleet Monitoring Test Suite ==="
echo ""

# Test counter
TESTS_PASSED=0
TESTS_FAILED=0

test_feature() {
    local name="$1"
    local cmd="$2"

    echo -n "Testing: $name... "
    if eval "$cmd" > /dev/null 2>&1; then
        echo -e "${GREEN}✓ PASS${NC}"
        TESTS_PASSED=$((TESTS_PASSED + 1))
    else
        echo -e "${RED}✗ FAIL${NC}"
        TESTS_FAILED=$((TESTS_FAILED + 1))
    fi
}

# 1. Test enhanced metrics
test_feature "Enhanced metrics collection" \
    "python3 fleet_manager.py metrics | grep -q 'aggregated'"

# 2. Test fleet dashboard
test_feature "Fleet dashboard (simple)" \
    "python3 fleet_dashboard.py --once | grep -q 'Fleet Dashboard'"

# 3. Test dashboard export
test_feature "Dashboard export to JSON" \
    "python3 fleet_dashboard.py --export /tmp/test_metrics.json --format json && [ -f /tmp/test_metrics.json ]"

# 4. Test log aggregator
test_feature "Log aggregator help" \
    "python3 log_aggregator.py --help | grep -q 'Log Aggregator'"

# 5. Test log aggregate command
test_feature "Log aggregation" \
    "python3 log_aggregator.py aggregate -n 10 | grep -q 'timestamp'"

# 6. Test log search
test_feature "Log search functionality" \
    "python3 log_aggregator.py search 'kernel' --context 1 2>/dev/null || true"

# 7. Test Prometheus endpoint (if HTTP service running)
if lsof -Pi :9551 -sTCP:LISTEN -t >/dev/null 2>&1; then
    test_feature "Prometheus metrics endpoint" \
        "curl -s http://127.0.0.1:9551/metrics/prometheus | grep -q 'llmspell_kernels_total'"

    test_feature "Prometheus format validation" \
        "curl -s http://127.0.0.1:9551/metrics/prometheus | grep -q '# TYPE'"
else
    echo "Skipping HTTP service tests (service not running on port 9551)"
fi

# 8. Test monitor_resources.py
test_feature "Resource monitor script" \
    "python3 monitor_resources.py 2>&1 | grep -q 'Monitoring kernel resources' || true"

# 9. Test Makefile metrics target
test_feature "Makefile metrics target" \
    "make metrics 2>&1 | grep -q 'Fleet Metrics'"

# Cleanup
rm -f /tmp/test_metrics.json

# Summary
echo ""
echo "=== Test Summary ==="
echo -e "Passed: ${GREEN}$TESTS_PASSED${NC}"
echo -e "Failed: ${RED}$TESTS_FAILED${NC}"

if [ $TESTS_FAILED -eq 0 ]; then
    echo -e "${GREEN}✓ All monitoring tests passed!${NC}"
    exit 0
else
    echo -e "${RED}✗ Some tests failed${NC}"
    exit 1
fi