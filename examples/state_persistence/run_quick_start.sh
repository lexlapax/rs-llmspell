#!/bin/bash
# Quick start script for state persistence examples
# Purpose: Run basic examples to demonstrate functionality

set -e

echo "🚀 State Persistence Examples - Quick Start"
echo "=========================================="

# Check if llmspell binary exists
if [ ! -f "./target/debug/llmspell" ]; then
    echo "Building llmspell binary..."
    cargo build
fi

echo
echo "Running basic state operations example..."
echo "----------------------------------------"

# Run the basic Lua example
./target/debug/llmspell -c examples/state_persistence/configs/basic.toml run examples/state_persistence/basic_operations.lua

echo
echo "✅ Quick start completed!"
echo
echo "Next steps:"
echo "- Try other examples in examples/lua/state/"
echo "- Run with persistent storage: examples/configs/state-enabled.toml"
echo "- Explore migration examples: examples/lua/migration/"
echo "- Check backup examples: examples/lua/backup/"