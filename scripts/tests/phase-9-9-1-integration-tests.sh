#!/bin/bash

# Task 9.9.1: Core Systems Integration Testing
# This script runs all comprehensive tests for Phase 9 components

set -e

echo "========================================="
echo "Task 9.9.1: Core Systems Integration Testing"
echo "========================================="

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Test results tracking
PASSED=0
FAILED=0
TESTS=()

# Function to run a test and track results
run_test() {
    local test_name="$1"
    local test_cmd="$2"
    
    echo -e "\n${YELLOW}Running: $test_name${NC}"
    if eval "$test_cmd"; then
        echo -e "${GREEN}✅ PASS: $test_name${NC}"
        PASSED=$((PASSED + 1))
        TESTS+=("✅ $test_name")
    else
        echo -e "${RED}❌ FAIL: $test_name${NC}"
        FAILED=$((FAILED + 1))
        TESTS+=("❌ $test_name")
    fi
}

# Clean up any existing kernels
echo "Cleaning up existing kernels..."
pkill -f llmspell-kernel 2>/dev/null || true
sleep 1

# Test 1: Kernel Architecture
echo -e "\n${YELLOW}=== Test 1: Kernel Architecture ===${NC}"

# Start kernel
./target/debug/llmspell kernel start --port 9577 --id test-kernel &
KERNEL_PID=$!
sleep 2

# Test kernel status
run_test "Kernel Status Check" "./target/debug/llmspell kernel status test-kernel | grep -q 'Status: Running'"

# Test kernel exec
run_test "Kernel Exec Command" "./target/debug/llmspell exec --connect test-kernel \"print('External kernel works')\" 2>&1 | grep -q 'External kernel works'"

# Stop kernel
run_test "Kernel Stop Command" "./target/debug/llmspell kernel stop test-kernel"

sleep 2

# Test 2: Debug Infrastructure (using expect script)
echo -e "\n${YELLOW}=== Test 2: Debug Infrastructure ===${NC}"
if command -v expect &> /dev/null; then
    chmod +x scripts/tests/test-debug-infrastructure.exp
    run_test "Debug Infrastructure with Breakpoints" "scripts/tests/test-debug-infrastructure.exp"
else
    echo -e "${YELLOW}SKIP: expect not installed, skipping interactive debug test${NC}"
fi

# Test 3: RAG System
echo -e "\n${YELLOW}=== Test 3: RAG System ===${NC}"

# Create test document
echo "Test document about Lua programming and scripting" > /tmp/test_rag.txt

# Test RAG ingestion
run_test "RAG Ingest" "./target/debug/llmspell rag ingest /tmp/test_rag.txt --metadata '{\"source\": \"test\"}' 2>&1 | grep -qE 'Ingest|Success|Complete'"

# Test RAG search
run_test "RAG Search" "./target/debug/llmspell rag search 'Lua programming' --k 5 2>&1 | grep -qE 'Lua|programming|Results'"

# Test RAG clear
run_test "RAG Clear" "./target/debug/llmspell rag clear --confirm 2>&1 | grep -qE 'Clear|Success|Removed'"

# Test 4: State Management
echo -e "\n${YELLOW}=== Test 4: State Management ===${NC}"

# Test state set
run_test "State Set" "./target/debug/llmspell state set testkey 'testvalue' 2>&1"

# Test state get
run_test "State Get" "./target/debug/llmspell state get testkey 2>&1 | grep -q 'testvalue'"

# Test state list
run_test "State List" "./target/debug/llmspell state list 2>&1 | grep -q 'testkey'"

# Test state delete
run_test "State Delete" "./target/debug/llmspell state delete testkey 2>&1"

# Test 5: Session Management
echo -e "\n${YELLOW}=== Test 5: Session Management ===${NC}"

# Test session create
run_test "Session Create" "./target/debug/llmspell session create test-session 2>&1"

# Test session list
run_test "Session List" "./target/debug/llmspell session list 2>&1 | grep -q 'test-session'"

# Test session info
run_test "Session Info" "./target/debug/llmspell session info test-session 2>&1"

# Test session delete
run_test "Session Delete" "./target/debug/llmspell session delete test-session 2>&1"

# Test 6: REPL Commands (using expect script)
echo -e "\n${YELLOW}=== Test 6: REPL Commands ===${NC}"
if command -v expect &> /dev/null; then
    chmod +x scripts/tests/test-repl-commands.exp
    run_test "REPL Interactive Commands" "scripts/tests/test-repl-commands.exp"
else
    echo -e "${YELLOW}SKIP: expect not installed, skipping interactive REPL test${NC}"
fi

# Test 7: Configuration Management
echo -e "\n${YELLOW}=== Test 7: Configuration Management ===${NC}"

# Test config get
run_test "Config Get" "./target/debug/llmspell config get rag.enabled 2>&1"

# Test config set
run_test "Config Set" "./target/debug/llmspell config set debug.breakpoint_limit 100 2>&1"

# Test config list
run_test "Config List" "./target/debug/llmspell config list 2>&1"

# Summary
echo -e "\n========================================="
echo -e "${YELLOW}Test Summary:${NC}"
echo -e "${GREEN}Passed: $PASSED${NC}"
echo -e "${RED}Failed: $FAILED${NC}"
echo "========================================="
echo "Individual Results:"
for test in "${TESTS[@]}"; do
    echo "  $test"
done
echo "========================================="

if [ $FAILED -eq 0 ]; then
    echo -e "${GREEN}🎉 All tests passed!${NC}"
    exit 0
else
    echo -e "${RED}⚠️  Some tests failed${NC}"
    exit 1
fi