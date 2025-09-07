#!/bin/bash
# Manual Testing Script for DAP Bridge (Task 9.8.13.7)
# This script helps perform manual verification of DAP Bridge functionality

set -e

echo "==================================="
echo "DAP Bridge Manual Testing Suite"
echo "==================================="
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Build the project first
echo -e "${YELLOW}Step 1: Building llmspell with debug features...${NC}"
cd "$(dirname "$0")/../.."
cargo build --release
echo -e "${GREEN}✓ Build complete${NC}"
echo ""

# Test 1: Start kernel with DAP server
echo -e "${YELLOW}Test 1: Starting kernel with DAP server on port 5678...${NC}"
echo "Run this in a separate terminal:"
echo -e "${GREEN}./target/release/llmspell-kernel --port 9572 --dap-port 5678 --engine lua${NC}"
echo ""
echo "Press Enter when kernel is running..."
read

# Test 2: Test REPL .locals command
echo -e "${YELLOW}Test 2: Testing REPL .locals command...${NC}"
cat > /tmp/test_locals.lua << 'EOF'
local x = 42
local y = "hello"
local z = {a = 1, b = 2}

function test_func()
    local inner_var = 100
    print("Breakpoint here")  -- Set breakpoint on this line
    return inner_var + x
end

test_func()
EOF

echo "Starting REPL test..."
echo "Commands to run in REPL:"
echo "1. .break /tmp/test_locals.lua 7"
echo "2. Run: dofile('/tmp/test_locals.lua')"
echo "3. When paused: .locals"
echo "4. .continue"
echo ""
echo -e "${GREEN}./target/release/llmspell repl --debug${NC}"
echo ""
echo "Press Enter after testing .locals command..."
read

# Test 3: Test all DAP commands
echo -e "${YELLOW}Test 3: Testing DAP Commands${NC}"
echo ""
echo "Testing commands:"
echo "  .locals  - Show local variables"
echo "  .globals - Show global variables"
echo "  .upvalues - Show closure variables"
echo "  .stack   - Show call stack"
echo "  .step    - Step to next line"
echo "  .continue - Continue execution"
echo ""

# Test 4: Breakpoint functionality
echo -e "${YELLOW}Test 4: Breakpoint Testing${NC}"
cat > /tmp/breakpoint_test.lua << 'EOF'
print("Line 1")
print("Line 2")  -- Set breakpoint here
print("Line 3")
for i = 1, 3 do
    print("Loop " .. i)  -- Set breakpoint here too
end
print("Done")
EOF

echo "Test script created at /tmp/breakpoint_test.lua"
echo ""
echo "1. Set breakpoints at lines 2 and 5"
echo "2. Run: ./target/release/llmspell run --debug /tmp/breakpoint_test.lua"
echo "3. Verify execution pauses at each breakpoint"
echo "4. Use step/continue commands"
echo ""
echo "Press Enter after breakpoint testing..."
read

# Final summary
echo ""
echo -e "${GREEN}==================================="
echo "Manual Testing Checklist"
echo "===================================${NC}"
echo ""
echo "[ ] VS Code can connect to DAP server"
echo "[ ] Breakpoints set in VS Code pause execution"
echo "[ ] Variables view shows local variables"
echo "[ ] Call stack shows proper frames"
echo "[ ] Step commands work correctly"
echo "[ ] Multiple concurrent debug sessions work"
echo "[ ] .locals command shows variables in REPL"
echo "[ ] .globals command shows global variables"
echo "[ ] .upvalues command shows closure variables"
echo "[ ] .break command sets breakpoints"
echo "[ ] .step/.continue commands work"
echo "[ ] .stack shows call frames"
echo ""
echo -e "${YELLOW}Performance Checks:${NC}"
echo "[ ] DAP request handling < 10ms"
echo "[ ] Variable retrieval < 5ms for 100 variables"
echo "[ ] No blocking on async operations"
echo ""
echo "Run final check:"
echo -e "${GREEN}cargo clippy --workspace --all-features --all-targets -- -D warnings${NC}"