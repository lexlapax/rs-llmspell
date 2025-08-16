#!/bin/bash
# ABOUTME: Shell script to run all tool examples in the new organized structure
# ABOUTME: Simple test runner that executes each file and tallies pass/fail

# Set the llmspell command path
if [ -x "../target/debug/llmspell" ]; then
    LLMSPELL_CMD="../target/debug/llmspell"
elif [ -x "./target/debug/llmspell" ]; then
    LLMSPELL_CMD="./target/debug/llmspell"
else
    echo "Error: llmspell binary not found in ../target/debug or ./target/debug"
    exit 1
fi

echo "🚀 LLMSpell Tool Examples Test Suite"
echo "====================================="
echo "Date: $(date '+%Y-%m-%d %H:%M:%S')"
echo ""

# Find tool examples in new organized structure
if [[ $(basename "$PWD") == "examples" ]]; then
    base_dir="."
else
    base_dir="examples"
fi

# Tool examples are now organized by learning level
tool_dirs=(
    "$base_dir/script-users/getting-started"
    "$base_dir/script-users/features" 
    "$base_dir/script-users/advanced"
)

# Get all tool Lua files from all directories
tool_files=()
for dir in "${tool_dirs[@]}"; do
    if [ -d "$dir" ]; then
        while IFS= read -r -d '' file; do
            tool_files+=("$file")
        done < <(find "$dir" -name "*tool*.lua" -print0 2>/dev/null | sort -z)
    fi
done

echo "Discovered ${#tool_files[@]} tool examples:"
i=1
for file in "${tool_files[@]}"; do
    # Show relative path from examples/ for clarity
    rel_path=${file#$base_dir/}
    printf "  %2d. %s\n" "$i" "$rel_path"
    ((i++))
done
echo ""

if [ ${#tool_files[@]} -eq 0 ]; then
    echo "❌ No tool examples found!"
    echo "   Expected to find examples in:"
    for dir in "${tool_dirs[@]}"; do
        echo "   - $dir"
    done
    exit 1
fi

# Initialize counters
passed=0
failed=0
skipped=0
start_time=$(date +%s)

echo ""
echo "Running Tool Examples"
echo "===================="

# Run each tool example
i=1
for file in "${tool_files[@]}"; do
    rel_path=${file#$base_dir/}
    echo ""
    echo "[$i/${#tool_files[@]}] Running $rel_path..."
    echo "------------------------------------------------------------"
    
    file_start=$(date +%s.%N)
    
    # Run with timeout
    timeout 30 "$LLMSPELL_CMD" run "$file" 2>&1 | tee /tmp/tools_test_output.log
    exit_code=${PIPESTATUS[0]}
    
    file_end=$(date +%s.%N)
    if command -v bc >/dev/null 2>&1; then
        file_duration=$(echo "$file_end - $file_start" | bc)
    else
        file_duration=$(awk "BEGIN {print $file_end - $file_start}")
    fi
    
    # Check for common patterns that indicate success/failure/skipping
    if grep -q "API key" /tmp/tools_test_output.log; then
        echo "------------------------------------------------------------"
        echo "⏭️  $rel_path skipped - Missing API key"
        ((skipped++))
    elif grep -q "network\|connection\|timeout" /tmp/tools_test_output.log && [ $exit_code -ne 0 ]; then
        echo "------------------------------------------------------------"
        echo "⏭️  $rel_path skipped - Network/connectivity issue"
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
rm -f /tmp/tools_test_output.log

# Calculate totals
end_time=$(date +%s)
total_duration=$((end_time - start_time))
total=$((passed + failed + skipped))

# Print summary
echo ""
echo "============================================================"
echo "Tool Examples Summary Report"
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
echo "✨ Tool test run complete!"
echo ""
echo "📁 Tool examples tested from:"
echo "   • script-users/getting-started/ (basic tool usage)"
echo "   • script-users/features/ (feature-specific tools)"
echo "   • script-users/advanced/ (complex tool patterns)"

# Exit with failure if any tests failed
if [ $failed -gt 0 ]; then
    exit 1
fi