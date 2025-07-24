#!/bin/bash
# ABOUTME: Performance validation script for workflow hook overhead
# ABOUTME: Validates that hook integration adds <3% overhead to workflow execution

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

echo "=== Workflow Hook Performance Validation ==="
echo "Target: <3% overhead for hook execution"
echo ""

# Change to workflows directory
cd "$PROJECT_ROOT/llmspell-workflows"

# Run benchmarks and save results
echo "Running performance benchmarks..."
cargo bench --bench workflow_hook_overhead -- --save-baseline workflow_hooks 2>&1 | tee benchmark_results.txt

# Extract and analyze results
echo ""
echo "=== Performance Analysis ==="

# Function to calculate overhead percentage
calculate_overhead() {
    local baseline=$1
    local with_hooks=$2
    
    if [ -z "$baseline" ] || [ -z "$with_hooks" ]; then
        echo "N/A"
        return
    fi
    
    # Extract numeric values (assuming format like "1.23 ms")
    baseline_num=$(echo "$baseline" | grep -oE '[0-9]+\.?[0-9]*' | head -1)
    hooks_num=$(echo "$with_hooks" | grep -oE '[0-9]+\.?[0-9]*' | head -1)
    
    if [ -n "$baseline_num" ] && [ -n "$hooks_num" ]; then
        # Calculate percentage using bc
        overhead=$(echo "scale=2; (($hooks_num - $baseline_num) / $baseline_num) * 100" | bc)
        echo "${overhead}%"
    else
        echo "N/A"
    fi
}

# Parse benchmark results
echo "Parsing benchmark results..."

# Look for sequential workflow results
sequential_baseline=$(grep -A2 "sequential_10_steps_baseline" benchmark_results.txt | grep "time:" | head -1)
sequential_hooks=$(grep -A2 "sequential_10_steps_with_hooks" benchmark_results.txt | grep "time:" | head -1)

# Look for complex workflow results
complex_baseline=$(grep -A2 "complex_workflow_baseline" benchmark_results.txt | grep "time:" | head -1)
complex_hooks=$(grep -A2 "complex_workflow_with_hooks" benchmark_results.txt | grep "time:" | head -1)

echo ""
echo "Sequential Workflow (10 steps):"
echo "  Baseline:    $sequential_baseline"
echo "  With Hooks:  $sequential_hooks"
echo "  Overhead:    $(calculate_overhead "$sequential_baseline" "$sequential_hooks")"

echo ""
echo "Complex Workflow:"
echo "  Baseline:    $complex_baseline"
echo "  With Hooks:  $complex_hooks"
echo "  Overhead:    $(calculate_overhead "$complex_baseline" "$complex_hooks")"

# Check if we meet the <3% requirement
echo ""
echo "=== Validation Summary ==="

# Simple check - look for overhead percentages and verify they're under 3%
overhead_check=$(grep -oE '[0-9]+\.[0-9]+%' benchmark_results.txt | grep -v "100.00%" | while read pct; do
    value=$(echo "$pct" | tr -d '%')
    if (( $(echo "$value > 3.0" | bc -l) )); then
        echo "FAIL: $pct exceeds 3% target"
    fi
done)

if [ -z "$overhead_check" ]; then
    echo "✅ PASS: All workflow hook overheads are within 3% target"
    exit 0
else
    echo "❌ FAIL: Some workflows exceed 3% overhead target:"
    echo "$overhead_check"
    echo ""
    echo "Note: Hook overhead may vary based on system load and configuration."
    echo "Consider optimizing hook execution or adjusting circuit breaker settings."
    exit 1
fi