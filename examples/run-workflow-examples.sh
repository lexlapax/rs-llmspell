#!/bin/bash
# ABOUTME: Shell script to run all workflow examples in examples/lua/workflows/
# ABOUTME: Tests workflow execution patterns (sequential, conditional, loop, parallel)

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

# Find workflow examples
if [[ $(basename "$PWD") == "examples" ]]; then
    workflows_dir="lua/workflows"
else
    workflows_dir="examples/lua/workflows"
fi

# Get all workflow Lua files
workflow_files=($(ls $workflows_dir/*.lua 2>/dev/null | sort))

echo "Discovered ${#workflow_files[@]} workflow examples:"
i=1
for file in "${workflow_files[@]}"; do
    basename_file=$(basename "$file")
    printf "  %2d. %s\n" "$i" "$basename_file"
    ((i++))
done
echo ""

# Check if any workflows use agents
echo "Checking for agent-based workflows..."
agent_workflows=()
for file in "${workflow_files[@]}"; do
    if grep -q "agent" "$file" || grep -q "Agent" "$file"; then
        agent_workflows+=("$file")
    fi
done

if [ ${#agent_workflows[@]} -gt 0 ]; then
    echo "⚠️  Found ${#agent_workflows[@]} workflows that may use agents:"
    for file in "${agent_workflows[@]}"; do
        echo "   - $(basename "$file")"
    done
    echo ""
    if [ -z "$OPENAI_API_KEY" ] && [ -z "$ANTHROPIC_API_KEY" ]; then
        echo "⚠️  WARNING: No API keys found - agent workflows may fail"
    fi
fi

# Initialize counters
passed=0
failed=0
start_time=$(date +%s)

echo ""
echo "Running Workflow Examples"
echo "========================="

# Run each workflow example
i=1
for file in "${workflow_files[@]}"; do
    basename_file=$(basename "$file")
    echo ""
    echo "[$i/${#workflow_files[@]}] Running $basename_file..."
    echo "------------------------------------------------------------"
    
    file_start=$(date +%s.%N)
    
    # Run with timeout
    cd $(dirname $file)
    if [[ "$LLMSPELL_CMD" == *"cargo run"* ]]; then
        # For cargo run, we need to handle it differently
        timeout 60 bash -c "$LLMSPELL_CMD run \"$(basename $file)\"" 2>&1 | cat
    else
        # For direct binary execution
        timeout 60 $LLMSPELL_CMD run "$(basename $file)" 2>&1 | cat
    fi
    exit_code=${PIPESTATUS[0]}
    cd - > /dev/null
    
    file_end=$(date +%s.%N)
    if command -v bc >/dev/null 2>&1; then
        file_duration=$(echo "$file_end - $file_start" | bc)
    else
        file_duration=$(awk "BEGIN {print $file_end - $file_start}")
    fi
    
    echo "------------------------------------------------------------"
    if [ $exit_code -eq 124 ]; then
        echo "⏱️  $basename_file timed out after 60s"
        ((failed++))
    elif [ $exit_code -eq 0 ]; then
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
echo "Workflow Examples Summary Report"
echo "============================================================"
echo "Total examples: $total"
echo "✅ Passed: $passed"
echo "❌ Failed: $failed"
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

# Exit with failure if any tests failed
if [ $failed -gt 0 ]; then
    exit 1
fi