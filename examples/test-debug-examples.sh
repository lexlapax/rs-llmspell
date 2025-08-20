#!/bin/bash

# Test script for debug examples
# This script runs all debug examples to ensure they work correctly

set -e  # Exit on error

echo "🧪 Testing LLMSpell Debug Examples"
echo "=================================="

# Get the directory of this script
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Build the project first
echo "📦 Building LLMSpell..."
cd "$PROJECT_ROOT"
cargo build --bin llmspell

if [ $? -ne 0 ]; then
    echo "❌ Build failed!"
    exit 1
fi

echo "✅ Build successful!"

# Test each debug example
EXAMPLES_DIR="$SCRIPT_DIR/lua/debug"
BINARY="$PROJECT_ROOT/target/debug/llmspell"

echo ""
echo "🔧 Testing Debug Examples"
echo "========================"

# Test basic example
echo ""
echo "📋 Testing basic debug example..."
if timeout 30 "$BINARY" run "$EXAMPLES_DIR/debug-basic.lua" > /tmp/debug-basic-output.log 2>&1; then
    echo "✅ Basic debug example completed successfully"
    # Check for expected output
    if grep -q "Basic example complete" /tmp/debug-basic-output.log; then
        echo "   ✓ Expected completion message found"
    else
        echo "   ⚠️  Expected completion message not found"
    fi
else
    echo "❌ Basic debug example failed or timed out"
    echo "   Last 10 lines of output:"
    tail -10 /tmp/debug-basic-output.log | sed 's/^/   /'
    exit 1
fi

# Test performance example  
echo ""
echo "🚀 Testing performance debug example..."
if timeout 60 "$BINARY" run "$EXAMPLES_DIR/debug-performance.lua" > /tmp/debug-performance-output.log 2>&1; then
    echo "✅ Performance debug example completed successfully"
    # Check for expected output
    if grep -q "Performance profiling example complete" /tmp/debug-performance-output.log; then
        echo "   ✓ Expected completion message found"
    else
        echo "   ⚠️  Expected completion message not found"
    fi
else
    echo "❌ Performance debug example failed or timed out"
    echo "   Last 10 lines of output:"
    tail -10 /tmp/debug-performance-output.log | sed 's/^/   /'
    exit 1
fi

# Test filtering example
echo ""
echo "🎯 Testing filtering debug example..."
if timeout 60 "$BINARY" run "$EXAMPLES_DIR/debug-filtering.lua" > /tmp/debug-filtering-output.log 2>&1; then
    echo "✅ Filtering debug example completed successfully"
    # Check for expected output
    if grep -q "Module filtering example complete" /tmp/debug-filtering-output.log; then
        echo "   ✓ Expected completion message found"
    else
        echo "   ⚠️  Expected completion message not found"
    fi
else
    echo "❌ Filtering debug example failed or timed out"
    echo "   Last 10 lines of output:"
    tail -10 /tmp/debug-filtering-output.log | sed 's/^/   /'
    exit 1
fi

# Test comprehensive example (may take longer)
echo ""
echo "🔄 Testing comprehensive debug example..."
if timeout 120 "$BINARY" run "$EXAMPLES_DIR/debug-comprehensive.lua" > /tmp/debug-comprehensive-output.log 2>&1; then
    echo "✅ Comprehensive debug example completed successfully"
    # Check for expected output
    if grep -q "Debug infrastructure demonstration complete" /tmp/debug-comprehensive-output.log; then
        echo "   ✓ Expected completion message found"
    else
        echo "   ⚠️  Expected completion message not found"
    fi
else
    echo "❌ Comprehensive debug example failed or timed out"
    echo "   Last 10 lines of output:"
    tail -10 /tmp/debug-comprehensive-output.log | sed 's/^/   /'
    exit 1
fi

# Run integration tests
echo ""
echo "🧪 Running Debug Integration Tests"
echo "================================="

cd "$PROJECT_ROOT"
if cargo test -p llmspell-bridge --test debug_integration_tests -- --test-threads=1; then
    echo "✅ Integration tests passed!"
else
    echo "❌ Integration tests failed!"
    exit 1
fi

# Performance check - make sure debug infrastructure is efficient
echo ""
echo "⚡ Performance Check"
echo "==================="

echo "Running performance test..."
PERF_START=$(date +%s%N)
timeout 30 "$BINARY" run "$EXAMPLES_DIR/debug-performance.lua" > /tmp/debug-perf-check.log 2>&1
PERF_END=$(date +%s%N)

PERF_DURATION_NS=$((PERF_END - PERF_START))
PERF_DURATION_MS=$((PERF_DURATION_NS / 1000000))

echo "Performance example completed in ${PERF_DURATION_MS}ms"

if [ $PERF_DURATION_MS -lt 30000 ]; then  # Less than 30 seconds
    echo "✅ Performance is acceptable"
else
    echo "⚠️  Performance is slower than expected (>${PERF_DURATION_MS}ms)"
fi

# Summary
echo ""
echo "📊 Test Summary"
echo "==============="
echo "✅ All debug examples executed successfully"
echo "✅ Integration tests passed"
echo "✅ Performance check completed"

# Cleanup
rm -f /tmp/debug-*-output.log /tmp/debug-perf-check.log

echo ""
echo "🎉 All tests completed successfully!"
echo ""
echo "💡 You can now run individual examples with:"
echo "   $BINARY run examples/lua/debug/debug-basic.lua"
echo "   $BINARY run examples/lua/debug/debug-performance.lua"
echo "   $BINARY run examples/lua/debug/debug-filtering.lua"
echo "   $BINARY run examples/lua/debug/debug-comprehensive.lua"