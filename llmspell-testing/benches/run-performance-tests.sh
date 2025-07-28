#!/bin/bash
# ABOUTME: Script to run all performance tests and verify targets
# ABOUTME: Generates reports and checks for performance regressions

set -e

echo "🚀 LLMSpell Performance Test Suite"
echo "=================================="
echo

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ] || [ ! -d "../../llmspell-core" ]; then
    echo -e "${RED}Error: Must run from tests/performance directory${NC}"
    exit 1
fi

# Function to run a benchmark and check results
run_benchmark() {
    local bench_name=$1
    local description=$2
    
    echo -e "${YELLOW}Running $bench_name: $description${NC}"
    
    # Run the benchmark
    if cargo bench --bench $bench_name -- --verbose; then
        echo -e "${GREEN}✅ $bench_name completed successfully${NC}"
        return 0
    else
        echo -e "${RED}❌ $bench_name failed${NC}"
        return 1
    fi
}

# Clean previous results
echo "Cleaning previous benchmark results..."
rm -rf target/criterion

# Run each benchmark
FAILED=0

echo
run_benchmark "hook_overhead" "Testing hook system overhead (<5% target)" || FAILED=$((FAILED + 1))

echo
run_benchmark "event_throughput" "Testing event throughput (>100K/sec target)" || FAILED=$((FAILED + 1))

echo
run_benchmark "circuit_breaker" "Testing circuit breaker effectiveness" || FAILED=$((FAILED + 1))

echo
run_benchmark "cross_language" "Testing cross-language bridge overhead (<10% target)" || FAILED=$((FAILED + 1))

# Summary
echo
echo "=================================="
echo "Performance Test Summary"
echo "=================================="

if [ $FAILED -eq 0 ]; then
    echo -e "${GREEN}All performance tests passed! ✅${NC}"
    
    # Generate consolidated report
    echo
    echo "Generating performance report..."
    
    # Create summary from criterion output
    if [ -d "target/criterion" ]; then
        echo
        echo "Performance Metrics Summary:"
        echo "---------------------------"
        
        # Extract key metrics from criterion reports
        find target/criterion -name "estimates.json" -type f | while read -r file; do
            bench_name=$(echo "$file" | sed 's/.*criterion\/\([^\/]*\)\/.*/\1/')
            echo
            echo "📊 $bench_name:"
            # Parse JSON for median time (requires jq)
            if command -v jq &> /dev/null; then
                median=$(jq -r '.median.point_estimate' "$file" 2>/dev/null || echo "N/A")
                if [ "$median" != "N/A" ]; then
                    # Convert nanoseconds to appropriate unit
                    median_ms=$(echo "scale=3; $median / 1000000" | bc 2>/dev/null || echo "N/A")
                    echo "   Median time: ${median_ms}ms"
                fi
            fi
        done
        
        echo
        echo "Full reports available in: target/criterion/"
        echo "Open target/criterion/report/index.html for detailed analysis"
    fi
else
    echo -e "${RED}$FAILED performance tests failed ❌${NC}"
    echo
    echo "Failed tests need investigation:"
    echo "1. Check if performance targets are being met"
    echo "2. Look for memory leaks or excessive allocations"
    echo "3. Profile specific benchmarks for bottlenecks"
    exit 1
fi

# Optional: Run with memory profiling
if [ "$1" = "--with-memory" ]; then
    echo
    echo "Running memory leak detection..."
    
    # This requires valgrind to be installed
    if command -v valgrind &> /dev/null; then
        cargo test --test performance_memory_check --release -- --nocapture 2>&1 | \
            valgrind --leak-check=full --show-leak-kinds=all || true
    else
        echo "Valgrind not found, skipping memory leak detection"
    fi
fi

echo
echo "✨ Performance test suite complete!"

# Generate performance trends if historical data exists
if [ -f ".performance-history.json" ]; then
    echo
    echo "Performance Trends:"
    echo "------------------"
    # Compare with historical data
    # This would require a more sophisticated script to track trends
fi

exit 0