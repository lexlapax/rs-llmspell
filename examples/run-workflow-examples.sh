#!/bin/bash
# ABOUTME: Shell script to run all workflow examples in the new organized structure
# ABOUTME: Tests workflow execution patterns and tool chaining examples

# Set the llmspell command path
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
PROJECT_ROOT="$( cd "$SCRIPT_DIR/.." && pwd )"

if [ -n "$LLMSPELL_CMD" ]; then
    # Use the provided command
    echo "Using provided LLMSPELL_CMD: $LLMSPELL_CMD"
elif [ -x "$PROJECT_ROOT/target/debug/llmspell" ]; then
    LLMSPELL_CMD="$PROJECT_ROOT/target/debug/llmspell"
    echo "Using llmspell binary: $LLMSPELL_CMD"
elif [ -x "$PROJECT_ROOT/target/release/llmspell" ]; then
    LLMSPELL_CMD="$PROJECT_ROOT/target/release/llmspell"
    echo "Using llmspell binary: $LLMSPELL_CMD"
elif command -v cargo &> /dev/null; then
    LLMSPELL_CMD="cargo run --bin llmspell --"
    echo "Using cargo run as llmspell command"
else
    echo "Error: llmspell binary not found and cargo not available"
    exit 1
fi

echo "🔄 LLMSpell Workflow Examples Test Suite"
echo "========================================"
echo "Date: $(date '+%Y-%m-%d %H:%M:%S')"
echo ""

# Find workflow examples in new organized structure
if [[ $(basename "$PWD") == "examples" ]]; then
    base_dir="."
else
    base_dir="examples"
fi

# Workflow examples are in multiple locations
workflow_dirs=(
    "$base_dir/script-users/workflows"
    "$base_dir/script-users/features"
)

# Get all workflow-related Lua files
workflow_files=()
for dir in "${workflow_dirs[@]}"; do
    if [ -d "$dir" ]; then
        while IFS= read -r -d '' file; do
            workflow_files+=("$file")
        done < <(find "$dir" -name "*workflow*.lua" -print0 2>/dev/null | sort -z)
    fi
done

echo "Discovered ${#workflow_files[@]} workflow examples:"
i=1
for file in "${workflow_files[@]}"; do
    # Show relative path from examples/ for clarity
    rel_path=${file#$base_dir/}
    printf "  %2d. %s\n" "$i" "$rel_path"
    ((i++))
done
echo ""

if [ ${#workflow_files[@]} -eq 0 ]; then
    echo "❌ No workflow examples found!"
    echo "   Expected to find examples in:"
    for dir in "${workflow_dirs[@]}"; do
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
echo "Running Workflow Examples"
echo "========================"

# Run each workflow example
i=1
for file in "${workflow_files[@]}"; do
    rel_path=${file#$base_dir/}
    echo ""
    echo "[$i/${#workflow_files[@]}] Running $rel_path..."
    echo "------------------------------------------------------------"
    
    file_start=$(date +%s.%N)
    
    # Run with timeout
    timeout 60 "$LLMSPELL_CMD" run "$file" 2>&1 | tee /tmp/workflow_test_output.log
    exit_code=${PIPESTATUS[0]}
    
    file_end=$(date +%s.%N)
    if command -v bc >/dev/null 2>&1; then
        file_duration=$(echo "$file_end - $file_start" | bc)
    else
        file_duration=$(awk "BEGIN {print $file_end - $file_start}")
    fi
    
    # Check for common patterns that indicate success/failure/skipping
    if grep -q "API key" /tmp/workflow_test_output.log; then
        echo "------------------------------------------------------------"
        echo "⏭️  $rel_path skipped - Missing API key"
        ((skipped++))
    elif grep -q "network\|connection\|timeout" /tmp/workflow_test_output.log && [ $exit_code -ne 0 ]; then
        echo "------------------------------------------------------------"
        echo "⏭️  $rel_path skipped - Network/connectivity issue"
        ((skipped++))
    elif [ $exit_code -eq 124 ]; then
        echo "------------------------------------------------------------"
        echo "⏱️  $rel_path timed out after 60s"
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
rm -f /tmp/workflow_test_output.log

# Calculate totals
end_time=$(date +%s)
total_duration=$((end_time - start_time))
total=$((passed + failed + skipped))

# Print summary
echo ""
echo "============================================================"
echo "Workflow Examples Summary Report"
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
echo "✨ Workflow test run complete!"
echo ""
echo "📁 Workflow examples tested from:"
echo "   • script-users/workflows/ (core workflow patterns)"
echo "   • script-users/features/ (workflow features like tool chaining)"

# Exit with failure if any tests failed
if [ $failed -gt 0 ]; then
    exit 1
fi