#!/bin/bash
# Benchmark graph traversal performance (Task 13c.2.8.7)
#
# Usage:
#   ./scripts/testing/benchmark-graph-traversal.sh
#
# Requirements:
#   - Generated dataset in benchmarks/graph-dataset/
#   - cargo (Rust toolchain)
#   - PostgreSQL running (for PostgreSQL tests)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DATASET_DIR="$SCRIPT_DIR/../../benchmarks/graph-dataset"
BENCH_DIR="$SCRIPT_DIR/../../benchmarks"

echo "=== Graph Traversal Performance Benchmark ==="
echo "Dataset: $DATASET_DIR"
echo

# Check dataset exists
if [ ! -f "$DATASET_DIR/entities.json" ]; then
    echo "Error: Dataset not found. Run generate-graph-dataset.sh first"
    exit 1
fi

echo "Dataset files:"
ls -lh "$DATASET_DIR"/*.json
echo

# Build benchmark binary
echo "Building benchmark binary..."
cd "$SCRIPT_DIR/../.."
cargo build --release --bin llmspell-graph-benchmark 2>&1 | grep -E "(Compiling|Finished)" || true
echo

echo "Running benchmarks..."
echo "This will take ~5-10 minutes (100 traversals per backend)"
echo

# Run benchmark
./target/release/llmspell-graph-benchmark \
    --dataset "$DATASET_DIR" \
    --output "$BENCH_DIR/graph-traversal-results.json" \
    --iterations 100

echo
echo "=== Benchmark Complete ==="
echo "Results: $BENCH_DIR/graph-traversal-results.json"
