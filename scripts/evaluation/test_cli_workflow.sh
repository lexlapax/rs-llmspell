#!/bin/bash
# ABOUTME: End-to-end CLI workflow test

set -e

echo "=== Phase 13 CLI Workflow Test ==="

SESSION_ID="cli-test-$(date +%s)"

# Add memory entries
echo "Adding memory entries..."
llmspell memory add "$SESSION_ID" user "What is Rust?" --metadata '{"topic":"rust"}'
llmspell memory add "$SESSION_ID" assistant "Rust is a systems programming language"

# Search memory
echo "Searching memory..."
llmspell memory search "Rust" --session-id "$SESSION_ID" --limit 5

# Get stats
echo "Getting memory stats..."
llmspell memory stats

# Consolidate
echo "Running consolidation..."
llmspell memory consolidate --session-id "$SESSION_ID" --force

# Assemble context
echo "Assembling context..."
llmspell context assemble "Rust programming" --strategy hybrid --budget 2000 --session-id "$SESSION_ID"

# List strategies
echo "Listing context strategies..."
llmspell context strategies

echo "✓ CLI workflow test complete"
