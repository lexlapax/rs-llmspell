#!/bin/bash
# ABOUTME: Shell script to run all agent examples in the new organized structure
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

# Find agent examples in new organized structure
if [[ $(basename "$PWD") == "examples" ]]; then
    base_dir="."
else
    base_dir="examples"
fi

# Agent examples are now organized by learning level
agent_dirs=(
    "$base_dir/script-users/getting-started"
    "$base_dir/script-users/features" 
    "$base_dir/script-users/advanced"
)

# Get all agent Lua files from all directories
agent_files=()
for dir in "${agent_dirs[@]}"; do
    if [ -d "$dir" ]; then
        while IFS= read -r -d '' file; do
            agent_files+=("$file")
        done < <(find "$dir" -name "*agent*.lua" -print0 2>/dev/null | sort -z)
    fi
done

echo "Discovered ${#agent_files[@]} agent examples:"
i=1
for file in "${agent_files[@]}"; do
    # Show relative path from examples/ for clarity
    rel_path=${file#$base_dir/}
    printf "  %2d. %s\n" "$i" "$rel_path"
    ((i++))
done
echo ""

if [ ${#agent_files[@]} -eq 0 ]; then
    echo "❌ No agent examples found!"
    echo "   Expected to find examples in:"
    for dir in "${agent_dirs[@]}"; do
        echo "   - $dir"
    done
    exit 1
fi

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
    rel_path=${file#$base_dir/}
    echo ""
    echo "[$i/${#agent_files[@]}] Running $rel_path..."
    echo "------------------------------------------------------------"
    
    file_start=$(date +%s.%N)
    
    # Add delay to avoid rate limiting (except for first test)
    if [ $i -gt 1 ]; then
        echo "⏳ Waiting 2s to avoid rate limits..."
        sleep 2
    fi
    
    # Run with timeout - use the file path relative to working directory
    timeout 30 "$LLMSPELL_CMD" run "$file" 2>&1 | tee /tmp/agent_test_output.log
    exit_code=${PIPESTATUS[0]}
    
    file_end=$(date +%s.%N)
    if command -v bc >/dev/null 2>&1; then
        file_duration=$(echo "$file_end - $file_start" | bc)
    else
        file_duration=$(awk "BEGIN {print $file_end - $file_start}")
    fi
    
    # Check for common error patterns
    if grep -q "API key" /tmp/agent_test_output.log; then
        echo "------------------------------------------------------------"
        echo "⏭️  $rel_path skipped - Missing API key"
        ((skipped++))
    elif [ $exit_code -eq 124 ]; then
        echo "------------------------------------------------------------"
        echo "⏱️  $rel_path timed out after 30s"
        ((failed++))
    elif [ $exit_code -eq 0 ]; then
        echo "------------------------------------------------------------"
        echo "✅ $rel_path completed in ${file_duration}s"
        ((passed++))
    else
        echo "------------------------------------------------------------"
        echo "❌ $rel_path failed after ${file_duration}s (exit code: $exit_code)"
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
echo ""
echo "📁 Agent examples tested from:"
echo "   • script-users/getting-started/ (basic examples)"
echo "   • script-users/features/ (feature demonstrations)"
echo "   • script-users/advanced/ (complex patterns)"

# Exit with failure if any tests failed
if [ $failed -gt 0 ]; then
    exit 1
fi