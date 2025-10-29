#!/bin/bash
# ABOUTME: Validates all Lua API examples run successfully
# ABOUTME: Tests Memory and Context global examples

set -e

echo "=== Validating Lua API Examples ==="
echo ""

EXAMPLES_DIR="examples/script-users"
LLMSPELL="${LLMSPELL:-./target/release/llmspell}"

# Check if llmspell binary exists
if [ ! -f "$LLMSPELL" ]; then
    echo "❌ llmspell binary not found at $LLMSPELL"
    echo "   Run: cargo build"
    exit 1
fi

echo "Using llmspell binary: $LLMSPELL"
echo ""

# Memory examples
echo "📚 Testing Memory examples..."
echo "  • 06-episodic-memory-basic.lua"
$LLMSPELL run $EXAMPLES_DIR/getting-started/06-episodic-memory-basic.lua > /dev/null
echo "  • memory-session-isolation.lua"
$LLMSPELL run $EXAMPLES_DIR/cookbook/memory-session-isolation.lua > /dev/null
echo "  • memory-stats.lua"
$LLMSPELL run $EXAMPLES_DIR/features/memory-stats.lua > /dev/null
echo "  • memory-semantic-basic.lua"
$LLMSPELL run $EXAMPLES_DIR/features/memory-semantic-basic.lua > /dev/null
echo "  ✓ All Memory examples passed"
echo ""

# Context examples
echo "🔍 Testing Context examples..."
echo "  • 07-context-assembly-basic.lua"
$LLMSPELL run $EXAMPLES_DIR/getting-started/07-context-assembly-basic.lua > /dev/null
echo "  • context-strategy-comparison.lua"
$LLMSPELL run $EXAMPLES_DIR/cookbook/context-strategy-comparison.lua > /dev/null
echo "  • memory-context-workflow.lua"
$LLMSPELL run $EXAMPLES_DIR/cookbook/memory-context-workflow.lua > /dev/null
echo "  • rag-memory-hybrid.lua"
$LLMSPELL run $EXAMPLES_DIR/cookbook/rag-memory-hybrid.lua > /dev/null
echo "  ✓ All Context examples passed"
echo ""

echo "✅ All Lua API examples executed successfully"
echo "   Total examples tested: 8"
