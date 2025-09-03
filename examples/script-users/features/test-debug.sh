#!/bin/bash
# examples/script-users/features/test-debug.sh

echo "=== Testing LLMSpell Debug Functionality ==="

# Test 1: Run with tracing (should work already)
echo "Test 1: Tracing mode"
llmspell run --debug examples/script-users/features/debug-showcase.lua

# Test 2: Interactive debug mode with breakpoints
echo "Test 2: Interactive debugging"
cat << 'EOF' | llmspell debug examples/script-users/features/debug-showcase.lua
.break 7
.break 11
.break 34 counter == 500
.continue
.locals
.stack
.step
.stepin
.stepout
.continue
.quit
EOF

# Test 3: Verify all commands work
echo "Test 3: Command verification"
llmspell debug --help | grep -E "break|step|continue|locals|stack"