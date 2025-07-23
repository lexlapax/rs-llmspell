#!/bin/bash
# ABOUTME: Shell script to run all agent examples in examples/lua/agents/
# ABOUTME: Tests agent creation and execution with API calls

# Set the llmspell command path
if [ -x "../target/debug/llmspell" ]; then
    LLMSPELL_CMD="../target/debug/llmspell"
elif [ -x "./target/debug/llmspell" ]; then
    LLMSPELL_CMD="./target/debug/llmspell"
else
    echo "Error: llmspell binary not found in ../target/debug or ./target/debug"
    exit 1
fi

echo "🤖 LLMSpell Agent Examples Test Suite"
echo "====================================="
echo "Date: $(date '+%Y-%m-%d %H:%M:%S')"
echo ""

# Check for API keys
if [ -z "$OPENAI_API_KEY" ] && [ -z "$ANTHROPIC_API_KEY" ]; then
    echo "⚠️  WARNING: No API keys found in environment"
    echo "   Set OPENAI_API_KEY or ANTHROPIC_API_KEY to run agent tests"
    echo ""
    read -p "Continue anyway? (y/N) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 1
    fi
fi

# Find agent examples
if [[ $(basename "$PWD") == "examples" ]]; then
    agents_dir="lua/agents"
else
    agents_dir="examples/lua/agents"
fi

# Get all agent Lua files
agent_files=($(ls $agents_dir/*.lua 2>/dev/null | sort))

echo "Discovered ${#agent_files[@]} agent examples:"
i=1
for file in "${agent_files[@]}"; do
    basename_file=$(basename "$file")
    printf "  %2d. %s\n" "$i" "$basename_file"
    ((i++))
done
echo ""

# Confirm API usage
echo "⚠️  These tests will make real API calls and incur costs!"
read -p "Continue with agent tests? (y/N) " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "Tests cancelled."
    exit 0
fi

# Initialize counters
passed=0
failed=0
skipped=0
start_time=$(date +%s)

echo ""
echo "Running Agent Examples"
echo "======================"

# Run each agent example
i=1
for file in "${agent_files[@]}"; do
    basename_file=$(basename "$file")
    echo ""
    echo "[$i/${#agent_files[@]}] Running $basename_file..."
    echo "------------------------------------------------------------"
    
    file_start=$(date +%s.%N)
    
    # Add delay to avoid rate limiting (except for first test)
    if [ $i -gt 1 ]; then
        echo "⏳ Waiting 2s to avoid rate limits..."
        sleep 2
    fi
    
    # Run with timeout and specific config
    # Store absolute path to llmspell
    LLMSPELL_ABS="$(cd "$(dirname "$LLMSPELL_CMD")" && pwd)/$(basename "$LLMSPELL_CMD")"
    cd $(dirname $file)
    timeout 30 "$LLMSPELL_ABS" run "$(basename $file)" 2>&1 | tee /tmp/agent_test_output.log
    exit_code=${PIPESTATUS[0]}
    cd - > /dev/null
    
    file_end=$(date +%s.%N)
    if command -v bc >/dev/null 2>&1; then
        file_duration=$(echo "$file_end - $file_start" | bc)
    else
        file_duration=$(awk "BEGIN {print $file_end - $file_start}")
    fi
    
    # Check for common error patterns
    if grep -q "API key" /tmp/agent_test_output.log; then
        echo "------------------------------------------------------------"
        echo "⏭️  $basename_file skipped - Missing API key"
        ((skipped++))
    elif [ $exit_code -eq 124 ]; then
        echo "------------------------------------------------------------"
        echo "⏱️  $basename_file timed out after 30s"
        ((failed++))
    elif [ $exit_code -eq 0 ]; then
        echo "------------------------------------------------------------"
        echo "✅ $basename_file completed in ${file_duration}s"
        ((passed++))
    else
        echo "------------------------------------------------------------"
        echo "❌ $basename_file failed after ${file_duration}s (exit code: $exit_code)"
        ((failed++))
    fi
    
    ((i++))
done

# Clean up
rm -f /tmp/agent_test_output.log

# Calculate totals
end_time=$(date +%s)
total_duration=$((end_time - start_time))
total=$((passed + failed + skipped))

# Print summary
echo ""
echo "============================================================"
echo "Agent Examples Summary Report"
echo "============================================================"
echo "Total examples: $total"
echo "✅ Passed: $passed"
echo "❌ Failed: $failed"
echo "⏭️  Skipped: $skipped"
echo "⏱️  Total time: ${total_duration} seconds"

if [ $total -gt 0 ]; then
    if command -v bc >/dev/null 2>&1; then
        success_rate=$(echo "scale=1; $passed * 100 / $total" | bc)
    else
        success_rate=$(awk "BEGIN {printf \"%.1f\", $passed * 100 / $total}")
    fi
    echo "📊 Success rate: ${success_rate}%"
fi

echo ""
echo "✨ Agent test run complete!"

# Exit with failure if any tests failed
if [ $failed -gt 0 ]; then
    exit 1
fi