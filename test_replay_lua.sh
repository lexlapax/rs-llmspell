#!/bin/bash
# Test script to verify Lua replay functionality

echo "Building llmspell CLI..."
cargo build --bin llmspell --features lua

echo -e "\nRunning basic replay test..."
./target/debug/llmspell run examples/lua/hooks/test_replay_basic.lua

echo -e "\nRunning full replay example..."
./target/debug/llmspell run examples/lua/hooks/replay.lua