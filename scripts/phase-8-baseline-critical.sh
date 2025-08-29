#!/bin/bash
# ABOUTME: Phase 8.10.6 Critical Performance Baseline Capture (Fast Version)
# ABOUTME: Focus on the most critical baselines needed for Phase 9 comparison

set -e

# Configuration  
WORKSPACE_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BASELINE_OUTPUT_DIR="${WORKSPACE_ROOT}/docs/performance/phase-8-baselines"
PHASE_VERSION="8.10.6"

echo "🎯 Phase $PHASE_VERSION CRITICAL Performance Baseline Capture"
echo "================================================================"
echo "Focus: RAG system, Bridge system, and Core operations"
echo "Workspace: $WORKSPACE_ROOT"
echo "Output: $BASELINE_OUTPUT_DIR"
echo ""

# Create output directories
mkdir -p "$BASELINE_OUTPUT_DIR"
mkdir -p "$WORKSPACE_ROOT/docs/performance"

cd "$WORKSPACE_ROOT"

# Function to run specific benchmark with timeout
run_critical_benchmark() {
    local crate="$1"
    local timeout_sec="$2"
    local description="$3"
    
    echo "📊 CRITICAL BASELINE: $description"
    echo "Command: cargo bench -p $crate"
    
    if OUTPUT=$(timeout "${timeout_sec}s" cargo bench -p "$crate" --quiet 2>&1); then
        echo "✅ Completed: $description"
        echo "$OUTPUT" > "$BASELINE_OUTPUT_DIR/${crate}_baseline.txt"
        
        # Extract key metrics immediately
        echo "Key Metrics Found:"
        grep -E "time:\s*\[[0-9.]+" "$BASELINE_OUTPUT_DIR/${crate}_baseline.txt" | head -5 | while read -r line; do
            echo "  $line"
        done
        echo ""
        return 0
    else
        echo "⚠️  Failed: $description"
        return 1
    fi
}

echo "🚀 Starting CRITICAL Phase $PHASE_VERSION Baseline Capture"
echo "Focus on components most impacted by Phase 9 graph storage"
echo ""

# Most Critical - Core System (ComponentId generation, serialization)
echo "═══ 1/3: CORE SYSTEM BASELINE ═══"
run_critical_benchmark "llmspell-core" 120 "ComponentId generation, version ops, serialization"

# Most Critical - Bridge System (RAG, globals injection, Lua/Rust bridge)  
echo "═══ 2/3: BRIDGE SYSTEM BASELINE (MOST CRITICAL) ═══"
echo "This is THE critical baseline - Phase 9 graph globals will be added here"
run_critical_benchmark "llmspell-bridge" 300 "RAG system, bridge injection, Lua/Rust bridge"

# Critical - Session System (will store graph state)
echo "═══ 3/3: SESSION SYSTEM BASELINE ═══"  
run_critical_benchmark "llmspell-sessions" 120 "Session lifecycle, artifact storage"

echo ""
echo "📊 GENERATING CRITICAL BASELINE REPORT"
echo "======================================"

# Generate focused baseline report
{
    echo "# Phase $PHASE_VERSION CRITICAL Performance Baselines"
    echo "## Focus: Phase 9 Graph Storage Impact Areas"
    echo ""
    echo "**Generated**: $(date -u '+%Y-%m-%d %H:%M:%S UTC')"
    echo "**Commit**: $(git rev-parse --short HEAD 2>/dev/null || echo 'unknown')"
    echo "**Purpose**: Critical baselines before Phase 9 graph storage (focused capture)"
    echo ""
    
    echo "## 🎯 Why These Baselines Matter for Phase 9"
    echo ""
    echo "Phase 9 will add \`llmspell-graph\` crate with temporal knowledge graphs. Impact areas:"
    echo ""
    echo "1. **llmspell-bridge**: Graph globals will be injected through bridge system"
    echo "2. **RAG System**: Graph relationships will complement vector search"  
    echo "3. **Session System**: Graph state will need session-based persistence"
    echo "4. **Core System**: Graph structures will use ComponentIds and serialization"
    echo ""
    
    echo "## ⚡ Performance Targets (Phase 8 Baseline)"
    echo "- **Tool initialization**: <10ms"
    echo "- **Agent creation**: <50ms"
    echo "- **Hook overhead**: <1%"
    echo "- **Vector search**: <10ms"
    echo "- **RAG retrieval**: <5ms"
    echo ""
    
    # Extract metrics from each critical system
    echo "## 🔧 Core System Baseline"
    if [ -f "$BASELINE_OUTPUT_DIR/llmspell-core_baseline.txt" ]; then
        echo "ComponentId generation and core operations:"
        echo "\`\`\`"
        grep -E "ComponentId.*time:" "$BASELINE_OUTPUT_DIR/llmspell-core_baseline.txt" | head -5
        echo "\`\`\`"
        echo ""
    fi
    
    echo "## 🌉 Bridge System Baseline (MOST CRITICAL)"
    echo "This is the PRIMARY baseline for Phase 9 comparison:"
    if [ -f "$BASELINE_OUTPUT_DIR/llmspell-bridge_baseline.txt" ]; then
        echo "### RAG System Performance"
        echo "\`\`\`"
        grep -E "(vector_search|document_ingestion|filtered_search).*time:" "$BASELINE_OUTPUT_DIR/llmspell-bridge_baseline.txt" | head -10
        echo "\`\`\`"
        echo ""
        echo "### Session Bridge Performance" 
        echo "\`\`\`"
        grep -E "session.*time:" "$BASELINE_OUTPUT_DIR/llmspell-bridge_baseline.txt" | head -5
        echo "\`\`\`"
        echo ""
        echo "### Workflow Bridge Performance"
        echo "\`\`\`"
        grep -E "workflow.*time:" "$BASELINE_OUTPUT_DIR/llmspell-bridge_baseline.txt" | head -5
        echo "\`\`\`"
    fi
    echo ""
    
    echo "## 📦 Session System Baseline"
    if [ -f "$BASELINE_OUTPUT_DIR/llmspell-sessions_baseline.txt" ]; then
        echo "Session and artifact management:"
        echo "\`\`\`"  
        grep -E "(session|artifact).*time:" "$BASELINE_OUTPUT_DIR/llmspell-sessions_baseline.txt" | head -10
        echo "\`\`\`"
    fi
    echo ""
    
    echo "## 🚨 Phase 9 Critical Performance Monitoring"
    echo ""
    echo "### RED LINE METRICS (Must Not Exceed)"
    echo "- **RAG vector search degradation**: >10% slower"
    echo "- **Bridge globals injection**: >25% slower"  
    echo "- **Session state storage**: >15% slower"
    echo "- **Memory usage increase**: >25% more"
    echo ""
    echo "### GREEN LINE TARGETS (Phase 9 New Features)"
    echo "- **Graph traversal queries**: <20ms"
    echo "- **Document relationship extraction**: <100ms"
    echo "- **Combined RAG+Graph search**: <30ms total"
    echo "- **Graph globals injection**: <5ms additional overhead"
    echo ""
    echo "## 🔬 Phase 9 Regression Testing Protocol"
    echo ""
    echo "\`\`\`bash"
    echo "# After Phase 9 implementation:"
    echo "cd $WORKSPACE_ROOT"
    echo "./scripts/phase-8-baseline-critical.sh  # Capture Phase 9 results"
    echo ""  
    echo "# Compare critical metrics:"
    echo "echo \"Bridge System Comparison:\""
    echo "diff -u docs/performance/phase-8-baselines/llmspell-bridge_baseline.txt \\"
    echo "        docs/performance/phase-9-baselines/llmspell-bridge_baseline.txt"
    echo "\`\`\`"
    echo ""
    
    echo "## 📁 Baseline Data Files"
    for file in "$BASELINE_OUTPUT_DIR"/*_baseline.txt; do
        if [ -f "$file" ]; then
            LINES=$(wc -l < "$file")
            echo "- \`$(basename "$file")\` ($LINES lines)"
        fi
    done
    
} > "$BASELINE_OUTPUT_DIR/phase-${PHASE_VERSION}-CRITICAL-baseline-report.md"

echo ""
echo "🎉 CRITICAL BASELINE CAPTURE COMPLETE"
echo "===================================="
echo ""
echo "📄 Critical baseline report:"
echo "   $BASELINE_OUTPUT_DIR/phase-${PHASE_VERSION}-CRITICAL-baseline-report.md"
echo ""
echo "📊 Critical baseline files:"
for file in "$BASELINE_OUTPUT_DIR"/*_baseline.txt; do
    if [ -f "$file" ]; then
        echo "   $(basename "$file")"
    fi
done
echo ""
echo "🎯 PHASE 9 READINESS ACHIEVED!"
echo ""
echo "✅ Critical baselines established for:"
echo "   - Core system (ComponentId, serialization)"
echo "   - Bridge system (RAG, globals, Lua/Rust bridge)"  
echo "   - Session system (state storage)"
echo ""
echo "⚠️  PHASE 9 DEVELOPMENT GUIDANCE:"
echo "   1. llmspell-bridge is THE critical component - monitor RAG performance closely"
echo "   2. Use these baselines to validate graph storage doesn't degrade existing features"
echo "   3. Focus on memory efficiency for graph structures"
echo "   4. Plan graph globals injection to minimize bridge overhead"
echo ""