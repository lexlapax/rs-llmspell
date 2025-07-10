#!/bin/bash
# ABOUTME: Shell script to run all tools-*.lua examples and report results
# ABOUTME: Simple test runner that executes each file and tallies pass/fail

# Set the llmspell command path
LLMSPELL_CMD="./target/debug/llmspell"

echo "🚀 LLMSpell Tool Examples Test Suite"
echo "====================================="
echo "Date: $(date '+%Y-%m-%d %H:%M:%S')"
echo ""

# Find all tools-*.lua files (excluding tools-run-all.lua)
examples_dir="examples"
example_files=($(ls $examples_dir/tools-*.lua 2>/dev/null | grep -v tools-run-all.lua | sort))

echo "Discovered ${#example_files[@]} example files:"
i=1
for file in "${example_files[@]}"; do
    basename_file=$(basename "$file")
    printf "  %2d. %s\n" "$i" "$basename_file"
    ((i++))
done
echo ""

# Initialize counters
passed=0
failed=0
start_time=$(date +%s)

echo "Running Examples"
echo "================"

# Run each example
i=1
for file in "${example_files[@]}"; do
    basename_file=$(basename "$file")
    echo ""
    echo "[$i/${#example_files[@]}] Running $basename_file..."
    echo "------------------------------------------------------------"
    
    file_start=$(date +%s.%N)
    
    # Run the file with timeout and capture exit code
    # Redirect output to avoid potential buffering issues
    timeout 30 $LLMSPELL_CMD run "$file" 2>&1 | cat
    exit_code=${PIPESTATUS[0]}
    
    file_end=$(date +%s.%N)
    file_duration=$(echo "$file_end - $file_start" | bc)
    
    echo "------------------------------------------------------------"
    if [ $exit_code -eq 0 ]; then
        echo "✅ $basename_file completed in ${file_duration}s"
        ((passed++))
    else
        echo "❌ $basename_file failed after ${file_duration}s (exit code: $exit_code)"
        ((failed++))
    fi
    
    ((i++))
done

# Calculate totals
end_time=$(date +%s)
total_duration=$((end_time - start_time))
total=$((passed + failed))

# Print summary
echo ""
echo "============================================================"
echo "Summary Report"
echo "============================================================"
echo "Total examples: $total"
echo "✅ Passed: $passed"
echo "❌ Failed: $failed"
echo "⏱️  Total time: ${total_duration} seconds"

if [ $total -gt 0 ]; then
    success_rate=$(echo "scale=1; $passed * 100 / $total" | bc)
    echo "📊 Success rate: ${success_rate}%"
fi

echo ""
echo "✨ Test run complete!"

# Exit with failure if any tests failed
if [ $failed -gt 0 ]; then
    exit 1
fi