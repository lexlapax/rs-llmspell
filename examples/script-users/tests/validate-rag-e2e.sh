#!/bin/bash
# Complete RAG End-to-End Validation Script
# Tests all aspects of RAG integration from CLI to storage

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"
LLMSPELL_BIN="${PROJECT_ROOT}/target/debug/llmspell"
CONFIGS_DIR="${SCRIPT_DIR}/../configs"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}=== RAG End-to-End Validation ===${NC}"
echo ""

# Build if needed
if [ ! -f "$LLMSPELL_BIN" ]; then
    echo -e "${YELLOW}Building llmspell...${NC}"
    cd "$PROJECT_ROOT"
    cargo build --bin llmspell
fi

# Track results
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0

# Function to run a test
run_test() {
    local name=$1
    local config=$2
    local script=$3
    local timeout_sec=${4:-30}
    
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
    echo -n "Running: $name... "
    
    if LLMSPELL_CONFIG="$config" timeout ${timeout_sec}s "$LLMSPELL_BIN" run "$script" &>/tmp/rag_test.log; then
        echo -e "${GREEN}PASS${NC}"
        PASSED_TESTS=$((PASSED_TESTS + 1))
        return 0
    else
        echo -e "${RED}FAIL${NC}"
        echo "  Output:"
        tail -10 /tmp/rag_test.log | sed 's/^/    /'
        FAILED_TESTS=$((FAILED_TESTS + 1))
        return 1
    fi
}

# Test 1: Basic functionality with different configs
echo -e "${BLUE}Test Suite 1: Configuration Loading${NC}"
for config_file in rag-basic.toml rag-development.toml rag-performance.toml rag-multi-tenant.toml; do
    run_test "Config: $config_file" \
        "$CONFIGS_DIR/$config_file" \
        "$SCRIPT_DIR/test-rag-e2e.lua" \
        60
done

# Test 2: Error handling
echo ""
echo -e "${BLUE}Test Suite 2: Error Handling${NC}"
run_test "Error handling validation" \
    "$CONFIGS_DIR/rag-development.toml" \
    "$SCRIPT_DIR/test-rag-errors.lua" \
    30

# Test 3: Performance benchmarks
echo ""
echo -e "${BLUE}Test Suite 3: Performance Benchmarks${NC}"
run_test "Performance benchmark" \
    "$CONFIGS_DIR/rag-performance.toml" \
    "$SCRIPT_DIR/../benchmarks/rag-benchmark.lua" \
    120

# Test 4: CLI flags override
echo ""
echo -e "${BLUE}Test Suite 4: CLI Flag Overrides${NC}"

# Test with --rag flag
echo -n "Testing --rag flag... "
if timeout 10s "$LLMSPELL_BIN" run --rag --no-config -c 'if RAG then print("OK") else error("FAIL") end' &>/tmp/rag_cli.log; then
    echo -e "${GREEN}PASS${NC}"
    PASSED_TESTS=$((PASSED_TESTS + 1))
else
    echo -e "${RED}FAIL${NC}"
    FAILED_TESTS=$((FAILED_TESTS + 1))
fi
TOTAL_TESTS=$((TOTAL_TESTS + 1))

# Test with --no-rag flag
echo -n "Testing --no-rag flag... "
if LLMSPELL_CONFIG="$CONFIGS_DIR/rag-basic.toml" timeout 10s "$LLMSPELL_BIN" run --no-rag -c 'if RAG then error("FAIL") else print("OK") end' &>/tmp/rag_cli2.log; then
    echo -e "${GREEN}PASS${NC}"
    PASSED_TESTS=$((PASSED_TESTS + 1))
else
    echo -e "${RED}FAIL${NC}"
    FAILED_TESTS=$((FAILED_TESTS + 1))
fi
TOTAL_TESTS=$((TOTAL_TESTS + 1))

# Test 5: Memory leak check
echo ""
echo -e "${BLUE}Test Suite 5: Memory Stability${NC}"

# Create a memory test script
cat > /tmp/rag_memory_test.lua << 'EOF'
-- Memory leak test
local initial_mem = collectgarbage("count")

for i = 1, 100 do
    RAG.ingest({
        content = "Memory test document " .. i,
        metadata = { index = i }
    })
    
    if i % 10 == 0 then
        collectgarbage("collect")
    end
end

collectgarbage("collect")
local final_mem = collectgarbage("count")
local mem_growth = final_mem - initial_mem

print(string.format("Memory growth: %.2f KB", mem_growth))

if mem_growth > 10000 then  -- More than 10MB growth
    error("Excessive memory growth detected")
end

print("Memory test passed")
EOF

run_test "Memory stability" \
    "$CONFIGS_DIR/rag-development.toml" \
    "/tmp/rag_memory_test.lua" \
    60

# Test 6: Persistence check
echo ""
echo -e "${BLUE}Test Suite 6: Persistence${NC}"

TEMP_DIR=$(mktemp -d)
PERSIST_CONFIG="$TEMP_DIR/persist.toml"

# Create config with persistence
cat > "$PERSIST_CONFIG" << EOF
[rag]
enabled = true

[rag.vector_storage]
dimensions = 384
backend = "hnsw"
persistence_path = "$TEMP_DIR/vectors"

[rag.embedding]
default_provider = "mock"
EOF

# Write data
cat > "$TEMP_DIR/write.lua" << 'EOF'
RAG.ingest({
    content = "Persistent test data that should survive",
    metadata = { persistent = true }
})
print("Data written")
EOF

# Read data
cat > "$TEMP_DIR/read.lua" << 'EOF'
local results = RAG.search({
    query = "persistent test data",
    top_k = 1
})

if #results == 0 then
    error("Persistence failed - data not found")
end

print("Data persisted successfully")
EOF

run_test "Write persistent data" "$PERSIST_CONFIG" "$TEMP_DIR/write.lua" 10
run_test "Read persistent data" "$PERSIST_CONFIG" "$TEMP_DIR/read.lua" 10

# Cleanup
rm -rf "$TEMP_DIR"

# Test 7: Integration test compilation
echo ""
echo -e "${BLUE}Test Suite 7: Rust Integration Tests${NC}"

echo -n "Compiling integration tests... "
if cd "$PROJECT_ROOT" && cargo test -p llmspell-bridge --test rag_e2e_integration_test --no-run &>/tmp/compile.log 2>&1; then
    echo -e "${GREEN}PASS${NC}"
    PASSED_TESTS=$((PASSED_TESTS + 1))
else
    echo -e "${RED}FAIL${NC}"
    echo "  Compilation errors:"
    grep -E "error\[" /tmp/compile.log | head -5 | sed 's/^/    /'
    FAILED_TESTS=$((FAILED_TESTS + 1))
fi
TOTAL_TESTS=$((TOTAL_TESTS + 1))

# Summary
echo ""
echo -e "${BLUE}${NC}"
echo "=================================="
echo "       VALIDATION SUMMARY         "
echo "=================================="
echo ""
echo -e "Total Tests:  ${TOTAL_TESTS}"
echo -e "Passed:      ${GREEN}${PASSED_TESTS}${NC}"
echo -e "Failed:      ${RED}${FAILED_TESTS}${NC}"
echo ""

if [ $FAILED_TESTS -eq 0 ]; then
    echo -e "${GREEN}✅ All RAG end-to-end validation tests passed!${NC}"
    echo ""
    echo "RAG integration is fully functional:"
    echo "  • Configuration loading works"
    echo "  • Error handling is robust"
    echo "  • Performance meets targets"
    echo "  • CLI flags work correctly"
    echo "  • Memory usage is stable"
    echo "  • Persistence functions properly"
    echo "  • Integration tests compile"
    exit 0
else
    echo -e "${RED}❌ Some tests failed${NC}"
    echo ""
    echo "Please review the failures above."
    exit 1
fi