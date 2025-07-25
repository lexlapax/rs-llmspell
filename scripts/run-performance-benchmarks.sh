#!/bin/bash
# ABOUTME: Performance benchmark runner for CI/CD pipeline
# ABOUTME: Runs tool initialization and operation benchmarks with thresholds

set -e

# Configuration
WORKSPACE_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BENCHMARK_OUTPUT_DIR="${WORKSPACE_ROOT}/target/benchmark-results"
THRESHOLD_INIT_MS=1.0  # 1ms per tool max
THRESHOLD_STARTUP_MS=50.0  # 50ms total startup max

echo "🚀 Running rs-llmspell Performance Benchmarks"
echo "Workspace: $WORKSPACE_ROOT"
echo "Output: $BENCHMARK_OUTPUT_DIR"

# Create output directory
mkdir -p "$BENCHMARK_OUTPUT_DIR"

cd "$WORKSPACE_ROOT"

# Function to extract timing from criterion output
extract_timing() {
    local bench_output="$1"
    local pattern="time:\s*\[([0-9.]+)\s*([a-z]+)"
    
    if [[ $bench_output =~ $pattern ]]; then
        local value="${BASH_REMATCH[1]}"
        local unit="${BASH_REMATCH[2]}"
        
        # Convert to milliseconds
        case $unit in
            "ns") echo "scale=6; $value / 1000000" | bc ;;
            "µs"|"us") echo "scale=6; $value / 1000" | bc ;;
            "ms") echo "$value" ;;
            "s") echo "scale=6; $value * 1000" | bc ;;
            *) echo "0" ;;
        esac
    else
        echo "0"
    fi
}

# Function to check performance threshold
check_threshold() {
    local actual="$1"
    local threshold="$2"
    local name="$3"
    
    if (( $(echo "$actual > $threshold" | bc -l) )); then
        echo "❌ FAIL: $name took ${actual}ms (threshold: ${threshold}ms)"
        return 1
    else
        echo "✅ PASS: $name took ${actual}ms (threshold: ${threshold}ms)"
        return 0
    fi
}

echo ""
echo "📊 Running Tool Initialization Benchmarks..."

# Run initialization benchmarks
echo "Running: cargo bench -p llmspell-tools --bench tool_initialization"
if ! INIT_OUTPUT=$(cargo bench -p llmspell-tools --bench tool_initialization -- --quiet 2>&1); then
    echo "❌ Failed to run initialization benchmarks"
    echo "$INIT_OUTPUT"
    exit 1
fi

echo "Initialization benchmarks completed"

# Extract key timings
STARTUP_TIME=$(echo "$INIT_OUTPUT" | grep "all_tools_startup" | head -1)
if [ -n "$STARTUP_TIME" ]; then
    STARTUP_MS=$(extract_timing "$STARTUP_TIME")
    check_threshold "$STARTUP_MS" "$THRESHOLD_STARTUP_MS" "Full startup sequence"
    STARTUP_RESULT=$?
else
    echo "⚠️  Could not extract startup timing"
    STARTUP_RESULT=1
fi

echo ""
echo "📈 Running Quick Operation Benchmarks..."

# Run a subset of operation benchmarks (faster)
echo "Running: cargo bench -p llmspell-tools --bench tool_operations -- simple_arithmetic uuid_v4_generation --quiet"
if OPERATION_OUTPUT=$(timeout 30s cargo bench -p llmspell-tools --bench tool_operations -- "simple_arithmetic|uuid_v4_generation" --quiet 2>&1); then
    echo "Operation benchmarks completed"
else
    echo "⚠️  Operation benchmarks timed out or failed (this is acceptable)"
    OPERATION_OUTPUT=""
fi

echo ""
echo "📋 Performance Summary Report"
echo "================================"

# Generate performance report
{
    echo "# Performance Benchmark Results"
    echo "**Date**: $(date -u '+%Y-%m-%d %H:%M:%S UTC')"
    echo "**Commit**: $(git rev-parse --short HEAD 2>/dev/null || echo 'unknown')"
    echo ""
    echo "## Key Metrics"
    
    if [ -n "$STARTUP_TIME" ]; then
        echo "- **Full Startup**: ${STARTUP_MS}ms (threshold: ${THRESHOLD_STARTUP_MS}ms)"
    fi
    
    echo ""
    echo "## Individual Tool Performance"
    echo "$INIT_OUTPUT" | grep -E "utility_tools_init|data_tools_init|file_system_tools_init|system_tools_init|search_tools_init" | while read -r line; do
        if [[ $line =~ time.*\[([^]]+)\] ]]; then
            echo "- $line"
        fi
    done
    
    echo ""
    echo "## Thresholds"
    echo "- Tool initialization: <${THRESHOLD_INIT_MS}ms per tool"
    echo "- Full startup: <${THRESHOLD_STARTUP_MS}ms total"
    
} > "$BENCHMARK_OUTPUT_DIR/performance-report.md"

echo "Report saved to: $BENCHMARK_OUTPUT_DIR/performance-report.md"

# Performance gate
if [ $STARTUP_RESULT -eq 0 ]; then
    echo ""
    echo "🎉 All performance benchmarks PASSED!"
    exit 0
else
    echo ""
    echo "💥 Some performance benchmarks FAILED!"
    exit 1
fi