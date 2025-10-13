#!/bin/bash
# Basic Research Assistant Template Example
# This demonstrates the simplest usage with just the required topic parameter

# Run the research assistant with default settings
# - Uses default max_sources (10)
# - Uses default model (ollama/llama3.2:3b)
# - Uses default output format (markdown)
# - Includes citations by default

llmspell template exec research-assistant \
  --param topic="Rust async programming patterns"

echo ""
echo "✓ Research assistant executed with minimal parameters"
echo "  - Topic: Rust async programming patterns"
echo "  - Max sources: 10 (default)"
echo "  - Model: ollama/llama3.2:3b (default)"
echo "  - Output format: markdown (default)"
echo "  - Citations: included (default)"
