#!/bin/bash
# ABOUTME: Simple script to run tool examples from the tools directory
# ABOUTME: Usage: ./run-tools-here.sh [tool-name.lua]

# Set the llmspell command path (go up to examples level, then find target)
if [ -x "../../../target/debug/llmspell" ]; then
    LLMSPELL_CMD="../../../target/debug/llmspell"
elif [ -x "../../../../target/debug/llmspell" ]; then  
    LLMSPELL_CMD="../../../../target/debug/llmspell"
else
    echo "Error: llmspell binary not found"
    exit 1
fi

if [ $# -eq 0 ]; then
    echo "Usage: $0 <tool-example.lua>"
    echo "Available tools:"
    ls -1 tools-*.lua 2>/dev/null | sed 's/^/  /'
    exit 1
fi

echo "Running $1..."
$LLMSPELL_CMD run "$1"