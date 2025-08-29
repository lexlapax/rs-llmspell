#!/bin/bash
# ABOUTME: Phase 8.10.6 Performance Baseline Capture for Phase 9 Comparison
# ABOUTME: Comprehensive benchmark suite to establish pre-Phase 9 baselines

set -e

# Configuration
WORKSPACE_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BASELINE_OUTPUT_DIR="${WORKSPACE_ROOT}/docs/performance/phase-8-baselines"
PHASE_VERSION="8.10.6"

echo "🎯 Phase $PHASE_VERSION Performance Baseline Capture"
echo "================================================================"
echo "Workspace: $WORKSPACE_ROOT"
echo "Output: $BASELINE_OUTPUT_DIR"
echo ""

# Create output directory
mkdir -p "$BASELINE_OUTPUT_DIR"

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

# Function to run benchmark with timeout and capture
run_benchmark() {
    local crate="$1"
    local bench_name="$2"
    local timeout_sec="$3"
    local description="$4"
    
    echo "📊 Running: $description"
    echo "Command: cargo bench -p $crate --bench $bench_name"
    
    if OUTPUT=$(timeout "${timeout_sec}s" cargo bench -p "$crate" --bench "$bench_name" -- --quiet 2>&1); then
        echo "✅ Completed: $description"
        echo "$OUTPUT" > "$BASELINE_OUTPUT_DIR/${crate}_${bench_name}_output.txt"
        return 0
    else
        echo "⚠️  Failed or timed out: $description"
        echo "ERROR: Benchmark failed or exceeded ${timeout_sec}s timeout" > "$BASELINE_OUTPUT_DIR/${crate}_${bench_name}_output.txt"
        return 1
    fi
}

echo "🚀 Starting Comprehensive Phase $PHASE_VERSION Baseline Capture"
echo ""

# Core System Performance Baselines
echo "═══ CORE SYSTEM PERFORMANCE ═══"
run_benchmark "llmspell-core" "core_benchmarks" 120 "Core system operations (ComponentId, serialization, error handling)"
run_benchmark "llmspell-testing" "minimal_test" 60 "Tool initialization and startup sequence"
run_benchmark "llmspell-testing" "state_operations" 120 "State read/write operations"
run_benchmark "llmspell-testing" "state_persistence" 180 "State persistence with disk I/O"

echo ""
echo "═══ HOOK AND EVENT SYSTEM ═══"
run_benchmark "llmspell-testing" "hook_overhead" 120 "Hook execution overhead measurement"
run_benchmark "llmspell-testing" "event_throughput" 180 "Event system throughput and latency"
run_benchmark "llmspell-testing" "circuit_breaker" 60 "Circuit breaker performance impact"

echo ""
echo "═══ AGENT AND WORKFLOW PERFORMANCE ═══"
run_benchmark "llmspell-testing" "integrated_overhead" 300 "System-wide integration overhead"
run_benchmark "llmspell-workflows" "workflow_hook_overhead" 120 "Workflow execution with hooks"

echo ""
echo "═══ TOOL SYSTEM PERFORMANCE ═══"
run_benchmark "llmspell-tools" "tool_initialization" 60 "Tool creation and registration"
run_benchmark "llmspell-tools" "tool_operations" 120 "Tool execution performance"
run_benchmark "llmspell-tools" "hook_performance" 60 "Tool-specific hook overhead"
run_benchmark "llmspell-tools" "web_tools_benchmark" 180 "Web tools (HTTP, scraping, API)"

echo ""
echo "═══ SESSION AND BRIDGE PERFORMANCE ═══"
run_benchmark "llmspell-sessions" "session_benchmarks" 120 "Session lifecycle and artifact management"
run_benchmark "llmspell-bridge" "session_bench" 60 "Session bridge operations"
run_benchmark "llmspell-bridge" "workflow_bridge_bench" 120 "Workflow bridge Lua/Rust integration"

echo ""
echo "═══ PHASE 8 RAG SYSTEM BASELINES ═══"
echo "This is the critical baseline for Phase 9 comparison"
run_benchmark "llmspell-bridge" "rag_bench" 300 "RAG vector search, ingestion, filtering, chunking"

echo ""
echo "═══ UTILITIES AND FILE OPERATIONS ═══"
run_benchmark "llmspell-utils" "file_utils_benchmarks" 60 "File system utilities"

echo ""
echo "📊 GENERATING COMPREHENSIVE BASELINE REPORT"
echo "=============================================="

# Generate comprehensive baseline report
{
    echo "# Phase $PHASE_VERSION Performance Baselines"
    echo ""
    echo "**Generated**: $(date -u '+%Y-%m-%d %H:%M:%S UTC')"
    echo "**Commit**: $(git rev-parse --short HEAD 2>/dev/null || echo 'unknown')"
    echo "**Purpose**: Establish performance baselines before Phase 9 graph storage implementation"
    echo ""
    
    echo "## System Information"
    echo "- **Platform**: $(uname -s) $(uname -m)"
    echo "- **Rust Version**: $(rustc --version)"
    echo "- **CPU Info**: $(sysctl -n machdep.cpu.brand_string 2>/dev/null || echo 'Unknown')"
    echo ""
    
    echo "## Performance Targets (Phase 8)"
    echo "- **Tool initialization**: <10ms"
    echo "- **Agent creation**: <50ms"
    echo "- **Hook overhead**: <1%"
    echo "- **State operations**: <5ms write, <1ms read"
    echo "- **Vector search**: <10ms across 1M+ vectors"
    echo "- **RAG retrieval**: <5ms with context assembly"
    echo ""
    
    echo "## Core System Baselines"
    echo ""
    
    # Process core benchmarks
    if [ -f "$BASELINE_OUTPUT_DIR/llmspell-core_core_benchmarks_output.txt" ]; then
        echo "### Component Operations"
        grep -E "component_id|serialization|error_handling" "$BASELINE_OUTPUT_DIR/llmspell-core_core_benchmarks_output.txt" | head -10 || echo "No core metrics found"
        echo ""
    fi
    
    # Process tool initialization
    if [ -f "$BASELINE_OUTPUT_DIR/llmspell-testing_minimal_test_output.txt" ]; then
        echo "### Tool System Performance"
        STARTUP_LINE=$(grep "all_tools_startup" "$BASELINE_OUTPUT_DIR/llmspell-testing_minimal_test_output.txt" | head -1)
        if [ -n "$STARTUP_LINE" ]; then
            STARTUP_MS=$(extract_timing "$STARTUP_LINE")
            echo "- **Full Tool Startup**: ${STARTUP_MS}ms"
        fi
        echo ""
    fi
    
    # Process state operations
    if [ -f "$BASELINE_OUTPUT_DIR/llmspell-testing_state_operations_output.txt" ]; then
        echo "### State System Performance" 
        grep -E "state_read|state_write" "$BASELINE_OUTPUT_DIR/llmspell-testing_state_operations_output.txt" | head -10 || echo "No state metrics found"
        echo ""
    fi
    
    echo "## Phase 8 RAG System Baselines (CRITICAL)"
    echo "These are the key baselines for Phase 9 graph storage comparison:"
    echo ""
    
    if [ -f "$BASELINE_OUTPUT_DIR/llmspell-bridge_rag_bench_output.txt" ]; then
        echo "### Vector Search Performance"
        grep -E "vector_search" "$BASELINE_OUTPUT_DIR/llmspell-bridge_rag_bench_output.txt" | head -10 || echo "No vector search metrics found"
        echo ""
        
        echo "### Document Ingestion Performance"
        grep -E "document_ingestion" "$BASELINE_OUTPUT_DIR/llmspell-bridge_rag_bench_output.txt" | head -10 || echo "No ingestion metrics found"
        echo ""
        
        echo "### Filtered Search Performance"
        grep -E "filtered_search" "$BASELINE_OUTPUT_DIR/llmspell-bridge_rag_bench_output.txt" | head -10 || echo "No filtered search metrics found"
        echo ""
        
        echo "### Concurrent Operations Performance"
        grep -E "concurrent_operations" "$BASELINE_OUTPUT_DIR/llmspell-bridge_rag_bench_output.txt" | head -10 || echo "No concurrent ops metrics found"
        echo ""
        
        echo "### Memory Impact Analysis"
        grep -E "memory_impact" "$BASELINE_OUTPUT_DIR/llmspell-bridge_rag_bench_output.txt" | head -10 || echo "No memory metrics found"
        echo ""
    fi
    
    echo "## Hook and Event System Baselines"
    echo ""
    
    if [ -f "$BASELINE_OUTPUT_DIR/llmspell-testing_hook_overhead_output.txt" ]; then
        echo "### Hook Overhead Analysis"
        grep -E "hook_execution|baseline_operation" "$BASELINE_OUTPUT_DIR/llmspell-testing_hook_overhead_output.txt" | head -10 || echo "No hook metrics found"
        echo ""
    fi
    
    if [ -f "$BASELINE_OUTPUT_DIR/llmspell-testing_event_throughput_output.txt" ]; then
        echo "### Event System Throughput"
        grep -E "event_throughput|event_latency" "$BASELINE_OUTPUT_DIR/llmspell-testing_event_throughput_output.txt" | head -10 || echo "No event metrics found"
        echo ""
    fi
    
    echo "## Session and Bridge System Baselines"
    echo ""
    
    if [ -f "$BASELINE_OUTPUT_DIR/llmspell-sessions_session_benchmarks_output.txt" ]; then
        echo "### Session Management Performance"
        grep -E "session_creation|artifact_" "$BASELINE_OUTPUT_DIR/llmspell-sessions_session_benchmarks_output.txt" | head -10 || echo "No session metrics found"
        echo ""
    fi
    
    if [ -f "$BASELINE_OUTPUT_DIR/llmspell-bridge_workflow_bridge_bench_output.txt" ]; then
        echo "### Lua/Rust Bridge Performance"
        grep -E "workflow_bridge|lua_" "$BASELINE_OUTPUT_DIR/llmspell-bridge_workflow_bridge_bench_output.txt" | head -10 || echo "No bridge metrics found"
        echo ""
    fi
    
    echo "## Phase 9 Comparison Guidelines"
    echo ""
    echo "When Phase 9 implements graph storage capabilities:"
    echo ""
    echo "1. **Critical Metrics to Monitor**:"
    echo "   - RAG system performance should not degrade >10%"
    echo "   - Vector search latency should remain <10ms"
    echo "   - Memory usage should not increase >25%"
    echo "   - Hook overhead should remain <1%"
    echo ""
    echo "2. **Expected Graph Storage Overhead**:"
    echo "   - Graph traversal: Target <20ms for complex queries"
    echo "   - Graph ingestion: Target <100ms for document relationship extraction"
    echo "   - Combined RAG+Graph search: Target <30ms total"
    echo ""
    echo "3. **Performance Regression Tests**:"
    echo "   - Re-run this baseline script after Phase 9 implementation"
    echo "   - Compare key metrics with automatic alerting for >15% degradation"
    echo "   - Monitor memory growth patterns"
    echo ""
    echo "## Raw Benchmark Data"
    echo ""
    echo "All raw benchmark outputs are available in:"
    echo "\`$BASELINE_OUTPUT_DIR/\`"
    echo ""
    for file in "$BASELINE_OUTPUT_DIR"/*_output.txt; do
        if [ -f "$file" ]; then
            echo "- \`$(basename "$file")\`"
        fi
    done
    
} > "$BASELINE_OUTPUT_DIR/phase-${PHASE_VERSION}-baseline-report.md"

echo ""
echo "📋 BASELINE CAPTURE COMPLETE"
echo "============================="
echo ""
echo "📄 Comprehensive baseline report generated:"
echo "   $BASELINE_OUTPUT_DIR/phase-${PHASE_VERSION}-baseline-report.md"
echo ""
echo "📊 Raw benchmark data files:"
for file in "$BASELINE_OUTPUT_DIR"/*_output.txt; do
    if [ -f "$file" ]; then
        echo "   $(basename "$file")"
    fi
done
echo ""
echo "🎯 Phase $PHASE_VERSION baseline is now established for Phase 9 comparison!"
echo ""
echo "⚠️  IMPORTANT FOR PHASE 9:"
echo "   - Re-run this script after Phase 9 graph storage implementation"
echo "   - Compare results with automated regression detection"
echo "   - Focus on RAG system performance impact (critical baseline)"
echo ""