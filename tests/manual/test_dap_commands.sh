#!/bin/bash
# Test DAP Bridge functionality through REPL
# Since DAP server isn't implemented, we test the bridge through REPL commands

set -e

echo "===================================="
echo "DAP Bridge Manual Testing"
echo "===================================="
echo ""

cd /Users/spuri/projects/lexlapax/rs-llmspell

# Build first
echo "Building llmspell..."
cargo build

# Create test Lua file
cat > /tmp/test_dap.lua << 'EOF'
-- Test file for DAP bridge testing
local x = 10
local y = 20

function test_function()
    local local_var = 42
    local nested = {a = 1, b = 2}
    print("In test_function")  -- Line 7: breakpoint here
    return local_var + x
end

local result = test_function()
print("Result:", result)

-- Test closure
function make_closure()
    local captured = 100
    return function()
        print("Captured:", captured)  -- Line 18: breakpoint here
        return captured + 1
    end
end

local closure = make_closure()
closure()
EOF

echo "✅ Created test file: /tmp/test_dap.lua"
echo ""

# Test 1: REPL with debug mode
echo "=== Test 1: Debug Commands in REPL ==="
echo "Starting REPL in debug mode..."
echo ""
echo "Commands to test manually:"
echo "1. .break /tmp/test_dap.lua 7"
echo "2. .break /tmp/test_dap.lua 18"
echo "3. .list  (should show 2 breakpoints)"
echo "4. dofile('/tmp/test_dap.lua')"
echo "5. When paused at line 7:"
echo "   - .locals  (should show local_var, nested)"
echo "   - .globals (should show test_function, make_closure)"
echo "   - .stack   (should show call stack)"
echo "   - .step    (step to next line)"
echo "   - .continue (continue to next breakpoint)"
echo "6. When paused at line 18:"
echo "   - .locals   (should be empty or minimal)"
echo "   - .upvalues (should show captured = 100)"
echo "   - .continue"
echo ""
echo "Run: ./target/debug/llmspell repl --debug"
echo ""
echo "Press Enter when ready to continue..."
read

# Test 2: Test DAP Bridge directly
echo "=== Test 2: DAP Bridge Unit Test ==="
cargo test -p llmspell-kernel dap_bridge -- --nocapture

echo ""
echo "=== Test 3: Test Variable Inspection ==="
cat > /tmp/test_vars.lua << 'EOF'
-- Complex variable types for testing
local number_var = 42
local string_var = "hello"
local table_var = {a = 1, b = 2, nested = {x = 10}}
local bool_var = true
local nil_var = nil
local func_var = function() return 1 end

-- Unicode and special characters
_G["你好"] = "Chinese"
_G["var-with-dash"] = "dashed"
_G["var.with.dots"] = "dotted"

print("Variables created")
EOF

echo "Run in REPL:"
echo "1. dofile('/tmp/test_vars.lua')"
echo "2. .locals"
echo "3. .globals (should show special character variables)"
echo ""

echo "=== Test 4: Breakpoint Persistence ==="
echo "1. Set breakpoints in REPL"
echo "2. Run script multiple times"
echo "3. Verify breakpoints persist across runs"
echo ""

echo "=== Manual Verification Checklist ==="
echo "[ ] .locals command shows local variables"
echo "[ ] .globals command shows global variables"  
echo "[ ] .upvalues command shows closure variables"
echo "[ ] .stack command shows call stack"
echo "[ ] .break command sets breakpoints"
echo "[ ] .delete command removes breakpoints"
echo "[ ] .list command shows all breakpoints"
echo "[ ] .step command steps to next line"
echo "[ ] .stepin command steps into function"
echo "[ ] .stepout command steps out of function"
echo "[ ] .continue command resumes execution"
echo "[ ] Variables with special characters display correctly"
echo "[ ] Nested tables show as table type with reference"
echo "[ ] Functions show as function type"
echo "[ ] Userdata (file handles) show as userdata type"
echo ""

echo "=== Performance Check ==="
echo "Create script with 100+ variables and test performance:"
cat > /tmp/test_perf.lua << 'EOF'
-- Performance test with many variables
for i = 1, 100 do
    _G["var" .. i] = i * 2
end
print("Created 100 variables")
EOF

echo "1. dofile('/tmp/test_perf.lua')"
echo "2. Time the .globals command"
echo "3. Should complete in < 5ms"
echo ""

echo "Testing complete! Check all boxes above."