#!/bin/bash
# ABOUTME: Phase 8.10.6 Performance Baseline Capture for Phase 9 Comparison (Fixed Version)
# ABOUTME: Comprehensive benchmark suite to establish pre-Phase 9 baselines

set -e

# Configuration
WORKSPACE_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BASELINE_OUTPUT_DIR="${WORKSPACE_ROOT}/docs/performance/phase-8-baselines"
PHASE_VERSION="8.10.6"

echo "🎯 Phase $PHASE_VERSION Performance Baseline Capture (Fixed)"
echo "================================================================"
echo "Workspace: $WORKSPACE_ROOT"
echo "Output: $BASELINE_OUTPUT_DIR"
echo ""

# Create output directories
mkdir -p "$BASELINE_OUTPUT_DIR"
mkdir -p "$WORKSPACE_ROOT/docs/performance"

cd "$WORKSPACE_ROOT"

# Function to run benchmark with timeout and capture
run_benchmark() {
    local crate="$1"
    local timeout_sec="$2"
    local description="$3"
    
    echo "📊 Running: $description"
    echo "Command: cargo bench -p $crate"
    
    if OUTPUT=$(timeout "${timeout_sec}s" cargo bench -p "$crate" 2>&1); then
        echo "✅ Completed: $description"
        echo "$OUTPUT" > "$BASELINE_OUTPUT_DIR/${crate}_output.txt"
        return 0
    else
        echo "⚠️  Failed or timed out: $description"
        echo "ERROR: Benchmark failed or exceeded ${timeout_sec}s timeout" > "$BASELINE_OUTPUT_DIR/${crate}_output.txt"
        return 1
    fi
}

echo "🚀 Starting Comprehensive Phase $PHASE_VERSION Baseline Capture"
echo ""

# Core System Performance Baselines
echo "═══ CORE SYSTEM PERFORMANCE ═══"
run_benchmark "llmspell-core" 120 "Core system operations (ComponentId, version ops, etc.)"
run_benchmark "llmspell-utils" 60 "File system utilities and shared utils"

echo ""
echo "═══ TESTING FRAMEWORK BENCHMARKS ═══"
run_benchmark "llmspell-testing" 300 "All testing framework benchmarks (tools, state, hooks, events)"

echo ""
echo "═══ TOOL SYSTEM PERFORMANCE ═══"
run_benchmark "llmspell-tools" 300 "All tool system benchmarks (init, execution, hooks, web tools)"

echo ""
echo "═══ SESSION MANAGEMENT ═══"
run_benchmark "llmspell-sessions" 120 "Session lifecycle and artifact management"

echo ""
echo "═══ WORKFLOW SYSTEM ═══"
run_benchmark "llmspell-workflows" 120 "Workflow execution with hook overhead"

echo ""
echo "═══ BRIDGE SYSTEM (CRITICAL FOR PHASE 9) ═══"
echo "These bridge benchmarks are critical - Phase 9 will add graph globals through the bridge system"
run_benchmark "llmspell-bridge" 400 "Bridge system including RAG, sessions, workflow bridge"

echo ""
echo "📊 GENERATING COMPREHENSIVE BASELINE REPORT"
echo "=============================================="

# Function to extract key metrics from criterion output
extract_metrics() {
    local file="$1"
    local category="$2"
    
    if [ -f "$file" ]; then
        echo "### $category"
        # Extract benchmark names and times using a more robust pattern
        grep -E "^[A-Za-z_][A-Za-z0-9_/]*.*time:" "$file" | head -20 | while IFS= read -r line; do
            echo "- \`$line\`"
        done | head -10
        echo ""
    else
        echo "### $category"
        echo "❌ No data available"
        echo ""
    fi
}

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
    if command -v sysctl >/dev/null 2>&1; then
        echo "- **CPU Info**: $(sysctl -n machdep.cpu.brand_string 2>/dev/null || echo 'Unknown')"
    fi
    echo ""
    
    echo "## Performance Targets (Phase 8)"
    echo "These are the targets we established for Phase 8, serving as baseline expectations:"
    echo "- **Tool initialization**: <10ms"
    echo "- **Agent creation**: <50ms"
    echo "- **Hook overhead**: <1%"
    echo "- **State operations**: <5ms write, <1ms read"
    echo "- **Vector search**: <10ms across 1M+ vectors"
    echo "- **RAG retrieval**: <5ms with context assembly"
    echo ""
    
    echo "## Critical Phase 9 Impact Areas"
    echo "Phase 9 will add graph storage capabilities. These areas are most likely to be impacted:"
    echo "1. **RAG System**: Graph relationships will be added alongside vector search"
    echo "2. **Bridge System**: New graph globals will be injected through the bridge"
    echo "3. **State System**: Graph state will need to be persisted"
    echo "4. **Memory Usage**: Graph structures will increase memory footprint"
    echo ""
    
    extract_metrics "$BASELINE_OUTPUT_DIR/llmspell-core_output.txt" "Core System Performance"
    extract_metrics "$BASELINE_OUTPUT_DIR/llmspell-testing_output.txt" "Testing Framework (Tools, State, Hooks, Events)"
    extract_metrics "$BASELINE_OUTPUT_DIR/llmspell-tools_output.txt" "Tool System Performance"
    extract_metrics "$BASELINE_OUTPUT_DIR/llmspell-sessions_output.txt" "Session Management"
    extract_metrics "$BASELINE_OUTPUT_DIR/llmspell-workflows_output.txt" "Workflow System"
    
    echo "## CRITICAL: Bridge System Baselines for Phase 9"
    echo "The bridge system is where Phase 9 will add graph globals and integration."
    echo "These are the most important baselines to monitor:"
    echo ""
    extract_metrics "$BASELINE_OUTPUT_DIR/llmspell-bridge_output.txt" "Bridge System (Including RAG)"
    
    echo "## Phase 9 Monitoring Strategy"
    echo ""
    echo "### 🚨 Critical Performance Regressions to Watch For"
    echo "1. **RAG system degradation >10%** - Graph storage should not significantly slow vector search"
    echo "2. **Bridge injection time >25% increase** - Adding graph globals should be efficient"  
    echo "3. **Memory usage >25% increase** - Graph structures should be memory-efficient"
    echo "4. **Tool initialization >15% slower** - Graph-aware tools should not add significant overhead"
    echo ""
    echo "### ✅ Expected Performance Additions in Phase 9"
    echo "- **Graph traversal**: Target <20ms for complex relationship queries"
    echo "- **Graph ingestion**: Target <100ms for document relationship extraction"
    echo "- **Combined RAG+Graph search**: Target <30ms total latency"
    echo "- **Graph globals injection**: Target <5ms additional bridge overhead"
    echo ""
    echo "### 🔄 Regression Testing Process"
    echo "1. Re-run this script after Phase 9 implementation"
    echo "2. Compare key metrics - alert for >15% degradation in critical paths"
    echo "3. Add new graph-specific benchmarks for Phase 9 features"
    echo "4. Monitor memory usage patterns with graph structures"
    echo ""
    echo "## Raw Benchmark Data Files"
    echo ""
    echo "Complete benchmark outputs available in: \`$BASELINE_OUTPUT_DIR/\`"
    echo ""
    for file in "$BASELINE_OUTPUT_DIR"/*_output.txt; do
        if [ -f "$file" ]; then
            echo "- \`$(basename "$file")\` ($(wc -l < "$file") lines)"
        fi
    done
    
    echo ""
    echo "## Usage Instructions for Phase 9 Team"
    echo ""
    echo "\`\`\`bash"
    echo "# After Phase 9 implementation, run comparison:"
    echo "cd $WORKSPACE_ROOT"
    echo "./scripts/phase-8-baseline-fixed.sh  # Generate new baselines"
    echo ""
    echo "# Compare results (manual process):"
    echo "diff docs/performance/phase-8-baselines/phase-8.10.6-baseline-report.md \\"
    echo "     docs/performance/phase-9-baselines/phase-9.x.x-baseline-report.md"
    echo "\`\`\`"
    
} > "$BASELINE_OUTPUT_DIR/phase-${PHASE_VERSION}-baseline-report.md"

echo ""
echo "📋 BASELINE CAPTURE COMPLETE"
echo "============================="
echo ""
echo "📄 Comprehensive baseline report generated:"
echo "   $BASELINE_OUTPUT_DIR/phase-${PHASE_VERSION}-baseline-report.md"
echo ""
echo "📊 Raw benchmark data files generated:"
for file in "$BASELINE_OUTPUT_DIR"/*_output.txt; do
    if [ -f "$file" ]; then
        SIZE=$(wc -l < "$file")
        echo "   $(basename "$file") ($SIZE lines)"
    fi
done
echo ""
echo "🎯 Phase $PHASE_VERSION Performance Baseline Established!"
echo ""
echo "⚠️  CRITICAL FOR PHASE 9 DEVELOPMENT:"
echo "   1. Focus on llmspell-bridge benchmarks - this is where graph globals will be added"
echo "   2. Monitor RAG system performance - graph storage should complement, not degrade vector search"
echo "   3. Watch memory usage patterns - graph structures need to be efficient"
echo "   4. Re-run this script after Phase 9 for automated regression detection"
echo ""
echo "📈 Next Steps:"
echo "   - Review the baseline report for current performance characteristics"
echo "   - Plan Phase 9 graph storage integration with these baselines in mind"
echo "   - Set up automated regression testing using these baselines"
echo ""