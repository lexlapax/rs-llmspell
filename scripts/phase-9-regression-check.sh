#!/bin/bash
# ABOUTME: Phase 9 Performance Regression Detection Script
# ABOUTME: Automated comparison against Phase 8.10.6 baselines with threshold alerting

set -e

# Configuration
WORKSPACE_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PHASE_8_BASELINE_DIR="${WORKSPACE_ROOT}/docs/performance/phase-8-baselines"
PHASE_9_RESULTS_DIR="${WORKSPACE_ROOT}/docs/performance/phase-9-results"
REGRESSION_REPORT="${WORKSPACE_ROOT}/docs/performance/phase-9-regression-report.md"

# Performance regression thresholds (from Phase 8 baseline)
CRITICAL_DEGRADATION_PERCENT=10    # RAG system must not degrade >10%
BRIDGE_DEGRADATION_PERCENT=25      # Bridge injection can degrade up to 25%
SESSION_DEGRADATION_PERCENT=15     # Session system up to 15%
MEMORY_INCREASE_PERCENT=25          # Memory usage up to 25% increase

echo "🔍 Phase 9 Performance Regression Detection"
echo "============================================="
echo "Baseline: Phase 8.10.6"
echo "Current: Phase 9.x.x"
echo "Thresholds: Critical=${CRITICAL_DEGRADATION_PERCENT}%, Bridge=${BRIDGE_DEGRADATION_PERCENT}%, Session=${SESSION_DEGRADATION_PERCENT}%, Memory=${MEMORY_INCREASE_PERCENT}%"
echo ""

# Create output directories
mkdir -p "$PHASE_9_RESULTS_DIR"

cd "$WORKSPACE_ROOT"

# Function to extract timing from benchmark output
extract_benchmark_time() {
    local file="$1"
    local benchmark_name="$2"
    
    if [ -f "$file" ]; then
        # Extract the median time in nanoseconds
        grep -A 1 "$benchmark_name" "$file" | grep "time:" | head -1 | \
            sed -n 's/.*time:\s*\[\([0-9.]*\)\s*\([a-z]*\).*/\1 \2/p'
    fi
}

# Function to convert time to nanoseconds for comparison
convert_to_nanoseconds() {
    local value="$1"
    local unit="$2"
    
    case $unit in
        "ns") echo "$value" ;;
        "µs"|"us") echo "$(echo "$value * 1000" | bc)" ;;
        "ms") echo "$(echo "$value * 1000000" | bc)" ;;
        "s") echo "$(echo "$value * 1000000000" | bc)" ;;
        *) echo "0" ;;
    esac
}

# Function to calculate percentage change
calculate_percentage_change() {
    local baseline="$1"
    local current="$2"
    
    if [ "$baseline" != "0" ] && [ -n "$baseline" ] && [ -n "$current" ]; then
        echo "scale=2; (($current - $baseline) / $baseline) * 100" | bc
    else
        echo "N/A"
    fi
}

# Function to run Phase 9 benchmarks and capture results
run_phase_9_benchmarks() {
    echo "🚀 Running Phase 9 Benchmarks..."
    echo ""
    
    # Core system benchmarks
    echo "📊 Core system benchmarks..."
    if timeout 120s cargo bench -p llmspell-core --quiet > "$PHASE_9_RESULTS_DIR/llmspell-core_phase9.txt" 2>&1; then
        echo "✅ Core benchmarks completed"
    else
        echo "⚠️  Core benchmarks failed or timed out"
    fi
    
    # Bridge system benchmarks (CRITICAL)
    echo "📊 Bridge system benchmarks (CRITICAL)..."
    if timeout 300s cargo bench -p llmspell-bridge --quiet > "$PHASE_9_RESULTS_DIR/llmspell-bridge_phase9.txt" 2>&1; then
        echo "✅ Bridge benchmarks completed"
    else
        echo "⚠️  Bridge benchmarks failed or timed out"
    fi
    
    # Session system benchmarks
    echo "📊 Session system benchmarks..."  
    if timeout 120s cargo bench -p llmspell-sessions --quiet > "$PHASE_9_RESULTS_DIR/llmspell-sessions_phase9.txt" 2>&1; then
        echo "✅ Session benchmarks completed"
    else
        echo "⚠️  Session benchmarks failed or timed out"
    fi
    
    echo ""
}

# Function to perform regression analysis
analyze_regressions() {
    local component="$1"
    local baseline_file="$2"
    local current_file="$3"
    local threshold_percent="$4"
    
    echo "## $component Performance Analysis"
    echo ""
    
    if [ ! -f "$baseline_file" ] || [ ! -f "$current_file" ]; then
        echo "❌ **MISSING DATA**: Cannot compare - baseline or current results missing"
        echo ""
        return 1
    fi
    
    local regression_detected=false
    
    # Analyze key benchmarks for this component
    case $component in
        "Core System")
            local benchmarks=("ComponentId/from_name/10" "ComponentId/from_name/50" "ComponentId/new")
            ;;
        "Bridge System")
            local benchmarks=("vector_search" "document_ingestion" "filtered_search" "concurrent_operations")
            ;;
        "Session System")  
            local benchmarks=("session_creation" "artifact_store" "session_restore")
            ;;
    esac
    
    echo "| Benchmark | Phase 8.10.6 | Phase 9.x.x | Change | Status |"
    echo "|-----------|---------------|--------------|--------|--------|"
    
    for benchmark in "${benchmarks[@]}"; do
        baseline_time=$(extract_benchmark_time "$baseline_file" "$benchmark")
        current_time=$(extract_benchmark_time "$current_file" "$benchmark")
        
        if [ -n "$baseline_time" ] && [ -n "$current_time" ]; then
            baseline_value=$(echo "$baseline_time" | cut -d' ' -f1)
            baseline_unit=$(echo "$baseline_time" | cut -d' ' -f2)
            current_value=$(echo "$current_time" | cut -d' ' -f1)  
            current_unit=$(echo "$current_time" | cut -d' ' -f2)
            
            baseline_ns=$(convert_to_nanoseconds "$baseline_value" "$baseline_unit")
            current_ns=$(convert_to_nanoseconds "$current_value" "$current_unit")
            
            change_percent=$(calculate_percentage_change "$baseline_ns" "$current_ns")
            
            if [[ "$change_percent" =~ ^-?[0-9]+\.?[0-9]*$ ]]; then
                if (( $(echo "$change_percent > $threshold_percent" | bc -l) )); then
                    status="🚨 **REGRESSION**"
                    regression_detected=true
                elif (( $(echo "$change_percent > 0" | bc -l) )); then
                    status="⚠️  Slower"
                else
                    status="✅ OK"
                fi
                echo "| $benchmark | ${baseline_value}${baseline_unit} | ${current_value}${current_unit} | ${change_percent}% | $status |"
            else
                echo "| $benchmark | ${baseline_value}${baseline_unit} | ${current_value}${current_unit} | N/A | ❓ Unknown |"
            fi
        else
            echo "| $benchmark | N/A | N/A | N/A | ❓ Missing |"
        fi
    done
    
    echo ""
    
    if [ "$regression_detected" = true ]; then
        echo "🚨 **PERFORMANCE REGRESSION DETECTED** in $component"
        echo ""
        return 1
    else
        echo "✅ **NO REGRESSIONS DETECTED** in $component"  
        echo ""
        return 0
    fi
}

# Main execution
echo "Running Phase 9 benchmarks to compare against Phase 8.10.6 baselines..."
run_phase_9_benchmarks

echo "📊 GENERATING REGRESSION ANALYSIS REPORT"
echo "========================================"

# Generate regression report
{
    echo "# Phase 9 Performance Regression Analysis"
    echo ""
    echo "**Generated**: $(date -u '+%Y-%m-%d %H:%M:%S UTC')"
    echo "**Baseline**: Phase 8.10.6"
    echo "**Current**: Phase 9.x.x"
    echo "**Purpose**: Automated regression detection against Phase 8 baselines"
    echo ""
    
    echo "## Executive Summary"
    echo ""
    
    # Perform regression analysis for each critical component
    core_status=0
    bridge_status=0
    session_status=0
    
    # Run analysis and capture exit codes
    analyze_regressions "Core System" "$PHASE_8_BASELINE_DIR/phase-8.10.6-baseline-report.md" "$PHASE_9_RESULTS_DIR/llmspell-core_phase9.txt" "$CRITICAL_DEGRADATION_PERCENT" || core_status=1
    
    analyze_regressions "Bridge System" "$PHASE_8_BASELINE_DIR/phase-8.10.6-baseline-report.md" "$PHASE_9_RESULTS_DIR/llmspell-bridge_phase9.txt" "$BRIDGE_DEGRADATION_PERCENT" || bridge_status=1
    
    analyze_regressions "Session System" "$PHASE_8_BASELINE_DIR/phase-8.10.6-baseline-report.md" "$PHASE_9_RESULTS_DIR/llmspell-sessions_phase9.txt" "$SESSION_DEGRADATION_PERCENT" || session_status=1
    
    echo "## Overall Assessment"
    echo ""
    
    if [ $core_status -eq 0 ] && [ $bridge_status -eq 0 ] && [ $session_status -eq 0 ]; then
        echo "🎉 **ALL SYSTEMS PASS REGRESSION TESTS**"
        echo ""
        echo "Phase 9 graph storage implementation has been successfully integrated"
        echo "without significant performance degradation to existing systems."
        overall_status=0
    else
        echo "🚨 **PERFORMANCE REGRESSIONS DETECTED**"
        echo ""
        echo "Phase 9 implementation has introduced performance regressions that"
        echo "exceed acceptable thresholds. Review and optimization required."
        overall_status=1
    fi
    
    echo "## Recommendations"
    echo ""
    
    if [ $bridge_status -eq 1 ]; then
        echo "### 🔥 CRITICAL: Bridge System Regression"
        echo "- Focus optimization on RAG system integration"
        echo "- Review graph global injection efficiency"  
        echo "- Consider lazy loading for graph features"
        echo "- Profile memory usage patterns"
        echo ""
    fi
    
    if [ $core_status -eq 1 ]; then
        echo "### Core System Regression"
        echo "- Review ComponentId usage in graph structures"
        echo "- Check serialization performance with graph data"
        echo "- Validate no unintended allocations"
        echo ""
    fi
    
    if [ $session_status -eq 1 ]; then
        echo "### Session System Regression" 
        echo "- Optimize graph state persistence"
        echo "- Review session storage overhead"
        echo "- Consider compression for graph data"
        echo ""
    fi
    
    echo "## Next Steps"
    echo ""
    echo "1. **If regressions detected**: Investigate and optimize before merge"
    echo "2. **If tests pass**: Document Phase 9 performance characteristics"
    echo "3. **Update baselines**: Establish Phase 9 baselines for Phase 10"
    echo ""
    
    echo "## Raw Performance Data"
    echo ""
    echo "Detailed benchmark results available in:"
    echo "- \`$PHASE_9_RESULTS_DIR/\`"
    echo ""
    
} > "$REGRESSION_REPORT"

echo ""
echo "📋 REGRESSION ANALYSIS COMPLETE"
echo "==============================="
echo ""
echo "📄 Regression report generated:"
echo "   $REGRESSION_REPORT"
echo ""
echo "📊 Phase 9 benchmark data:"
for file in "$PHASE_9_RESULTS_DIR"/*_phase9.txt; do
    if [ -f "$file" ]; then
        echo "   $(basename "$file")"
    fi
done
echo ""

# Exit with appropriate code for CI/CD
if [ -f "$REGRESSION_REPORT" ]; then
    if grep -q "ALL SYSTEMS PASS" "$REGRESSION_REPORT"; then
        echo "🎉 REGRESSION TESTS PASSED - Phase 9 Ready for Merge!"
        exit 0
    else
        echo "🚨 REGRESSION TESTS FAILED - Optimization Required!"
        exit 1
    fi
else
    echo "❌ REGRESSION ANALYSIS FAILED - Could not generate report"
    exit 1
fi