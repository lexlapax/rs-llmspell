#!/bin/bash
# ABOUTME: Shell script to run all tools-*.lua examples and report results
# ABOUTME: Simple test runner that executes each file and tallies pass/fail

# Set the llmspell command path
# Look for llmspell in either ../target/debug or ./target/debug
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

# Find all tools-*.lua files (excluding tools-run-all.lua)
# If we're already in the examples directory, look here, otherwise look in examples/
if [[ $(basename "$PWD") == "examples" ]]; then
    examples_dir="."
else
    examples_dir="examples"
fi
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
    # Use llmspell-test.toml if it exists to avoid API key requirements
    if [ -f "llmspell-test.toml" ]; then
        LLMSPELL_CONFIG=llmspell-test.toml timeout 30 $LLMSPELL_CMD run "$file" 2>&1 | cat
    else
        timeout 30 $LLMSPELL_CMD run "$file" 2>&1 | cat
    fi
    exit_code=${PIPESTATUS[0]}
    
    file_end=$(date +%s.%N)
    # Use awk if bc is not available
    if command -v bc >/dev/null 2>&1; then
        file_duration=$(echo "$file_end - $file_start" | bc)
    else
        file_duration=$(awk "BEGIN {print $file_end - $file_start}")
    fi
    
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
    # Use awk if bc is not available
    if command -v bc >/dev/null 2>&1; then
        success_rate=$(echo "scale=1; $passed * 100 / $total" | bc)
    else
        success_rate=$(awk "BEGIN {printf \"%.1f\", $passed * 100 / $total}")
    fi
    echo "📊 Success rate: ${success_rate}%"
fi

echo ""
echo "✨ Test run complete!"

# Exit with failure if any tests failed
if [ $failed -gt 0 ]; then
    exit 1
fi