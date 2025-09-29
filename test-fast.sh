#!/bin/bash
# Fast test runner for development
# Avoids full compilation by testing specific packages with minimal features

set -e

echo "Running fast tests for llmspell-bridge and llmspell-tools..."
echo "This avoids --all-features and --all-targets for faster compilation"

# Test with default features only
cargo test -p llmspell-bridge --test simple_tool_integration_test

# Add more specific tests as needed
echo "Tests completed successfully!"